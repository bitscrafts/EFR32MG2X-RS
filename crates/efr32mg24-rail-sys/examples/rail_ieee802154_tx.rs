//! 802.15.4: pôr o rádio em 2,4 GHz e transmitir um frame.
//!
//! ## O que este exemplo prova, e o que não prova
//!
//! Prova que a API aceita a configuração de 802.15.4 e que a transmissão chega ao evento de "pacote
//! enviado" do rádio. **Não prova que um frame saiu no ar**: não há um segundo rádio 802.15.4 nesta bancada,
//! e um transceptor não se ouve a si mesmo (`RAIL_StartTx` desliga o receptor). Um rádio reporta envio com a
//! antena desafinada, portanto o relato do próprio rádio é a evidência mais forte que este workspace permite
//! — e é mais fraca do que os testes do EM2 e do HFXO, que tinham tempo de parede do lado do host.
//!
//! ## A tabela de canais não é escrita aqui
//!
//! `RAIL_IEEE802154_Config2p4GHzRadio` substitui `RAIL_ConfigChannels` e, nas palavras do próprio header,
//! "after this call, channels 11-26 will be available, giving the frequencies of those channels on channel
//! page 0, as defined by IEEE 802.15.4-2011 section 8.1.2.2". Escrever uma tabela à mão seria duplicar o
//! vendor — e o item da fila que pedia essa tabela nasceu de uma leitura incompleta deste header.
//!
//! ## O frame
//!
//! Um frame de dados com PAN ID comprimido e endereços curtos, endereçado ao broadcast do PAN:
//!
//! ```text
//! 41 88  seq  FF FF  FF FF  34 12   "dosim"
//! ^^^^^  ^^^  ^^^^^  ^^^^^  ^^^^^  ^^^^^^^
//!  FCF   seq  dstPAN dstA   srcA   payload
//! ```
//!
//! FCF `0x8841`: tipo 1 (dados), sem segurança, sem pedido de ACK, PAN ID comprimido (bit 6), destino e
//! origem em endereço curto (modos 2 e 2). O FCS não está aqui: o rádio acrescenta-o.

#![no_std]
#![no_main]

use core::ptr;

#[path = "../../xiao-mg24-bsp/examples/common/mailbox.rs"]
mod mailbox;

use efr32mg24_rail_sys as rail;
use efr32mg2x_hal::prelude::*;
use mailbox::Status;
use panic_halt as _;

/// Quantos pacotes o RÁDIO diz ter enviado. O contador do loop conta chamadas ao `RAIL_StartTx`; este conta
/// interrupções do rádio, e os dois discordarem é exactamente o caso em que a API parece bem e o ar fica vazio.
static mut TX_PACKET_SENT: u32 = 0;

/// O canal 15 do 802.15.4: 2425 MHz. Os canais 11–26 vêm da chamada de configuração do vendor.
const CHANNEL: u16 = 15;

/// RAIL events. Corre em contexto de interrupção.
unsafe extern "C" fn on_rail_event(_handle: rail::RAIL_Handle_t, events: rail::RAIL_Events_t) {
    // bit 24 de RAIL_Events_t é RAIL_EVENT_TX_PACKET_SENT
    if (events & (1u64 << 24)) != 0 {
        TX_PACKET_SENT = TX_PACKET_SENT.wrapping_add(1);
    }
}

extern "C" {
    /// Carrega as curvas de potência do amplificador no RAIL. Sem isto o dBm converte para 255 ("inválido") e
    /// o amplificador não recebe nível -- enquanto todas as transmissões continuam a completar-se no modem.
    fn sl_rail_util_pa_init();
    /// A configuração de 2,4 GHz que o próprio plugin do vendor usa (modo, tensão, ramp).
    fn sl_rail_util_pa_get_tx_power_config_2p4ghz() -> *mut rail::RAIL_TxPowerConfig_t;
}

/// O símbolo que o RAIL exige existir, mesmo quando o rádio não é multifunções.
#[no_mangle]
pub extern "C" fn RAILCb_PaAutoModeDecision(_handle: rail::RAIL_Handle_t) {}

fn push(out: &mut [u8], mut at: usize, s: &[u8]) -> usize {
    for b in s {
        if at < out.len() {
            out[at] = *b;
            at += 1;
        }
    }
    at
}

fn push_num(out: &mut [u8], mut at: usize, mut v: u32) -> usize {
    let mut d = [0u8; 10];
    let mut n = 0;
    loop {
        d[n] = b'0' + (v % 10) as u8;
        v /= 10;
        n += 1;
        if v == 0 || n == d.len() {
            break;
        }
    }
    while n > 0 {
        n -= 1;
        if at < out.len() {
            out[at] = d[n];
            at += 1;
        }
    }
    at
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    // ## Marcadores, um por passo
    //
    // O mailbox guarda um registo: o ultimo escrito e' o que se perde. Se o exemplo travar numa chamada ao
    // vendor, o registo diz onde parou -- e "nao ha registo" passa a significar "travou antes do primeiro".
    mailbox::write(Status::Info, b"154: main entered");

    let mut board = efr32mg2x_hal::board::Board::take().unwrap();
    let mut delay = board.delay();

    // O rádio é clockado pelo cristal. Arrancá-lo não chega: tem de ser seleccionado, ou o RAIL continua no
    // HFRCO interno e recusa a configuração (medido antes: `radioClock=19000kHz cfg=39000kHz`).
    unsafe {
        efr32mg24_emlib_sys::emlib_hfxo_start();
        efr32mg24_emlib_sys::emlib_clock_select_hfxo();
    }
    let mut delay = board.delay();

    mailbox::write(Status::Info, b"154: hfxo selected");

    let mut cfg: rail::RAIL_Config_t = unsafe { core::mem::zeroed() };
    cfg.eventsCallback = Some(on_rail_event);
    let handle = unsafe { rail::RAIL_Init(&mut cfg, None) };
    if handle.is_null() {
        mailbox::write(Status::Fail, b"154: RAIL_Init returned null");
        loop {
            delay.delay_ms(500);
        }
    }

    mailbox::write(Status::Info, b"154: rail init ok");

    // O amplificador, na ordem que o plugin do vendor espera.
    unsafe { sl_rail_util_pa_init() };
    let pa_cfg = unsafe { sl_rail_util_pa_get_tx_power_config_2p4ghz() };
    let pa_st = if pa_cfg.is_null() {
        1
    } else {
        unsafe { rail::RAIL_ConfigTxPower(handle, pa_cfg) }
    };
    let dbm_st = unsafe { rail::RAIL_SetTxPowerDbm(handle, 200) };

    mailbox::write(Status::Info, b"154: pa configured");

    // 802.15.4 em 2,4 GHz: esta chamada traz os canais 11-26 e substitui `RAIL_ConfigChannels`.
    let phy_st = unsafe { rail::RAIL_IEEE802154_Config2p4GHzRadio(handle) };

    // ## O estado de protocolo, DEPOIS do PHY
    //
    // Estava antes e o `RAIL_IEEE802154_Init` recusava com `init=1` (`RAIL_STATUS_INVALID_PARAMETER`). O
    // `Config2p4GHzRadio` "toma o lugar do `RAIL_ConfigChannels`", e nos exemplos do vendor o canal configura-se
    // primeiro e o protocolo depois -- o Init precisa de um canal ja definido para validar os seus parametros.
    //
    // A estrutura e' `zeroed()` com o `framesMask` posto: enderecos a NULL (sem filtragem), ACK desligado, tempos
    // a zero (documentados como "tao rapido quanto possivel"), nem promiscuo nem coordenador de PAN.
    let mut std_cfg: rail::RAIL_IEEE802154_Config_t = unsafe { core::mem::zeroed() };
    std_cfg.framesMask = rail::RAIL_IEEE802154_ACCEPT_STANDARD_FRAMES as u8;
    // Os valores que o exemplo do header do vendor usa, um a um: ACK automatico com 672 us (54-12 simbolos a
    // 16 us), e as quatro transicoes a voltar para RX. Zero em `rxTransitions` e' `RAIL_RF_STATE_INACTIVE`, que
    // nao e' um estado para onde se volte -- dai o `INVALID_PARAMETER` que se media.
    std_cfg.ackConfig.enable = true;
    std_cfg.ackConfig.ackTimeout = 672;
    std_cfg.ackConfig.rxTransitions.success = 2; // RAIL_RF_STATE_RX
    std_cfg.ackConfig.rxTransitions.error = 2;
    std_cfg.ackConfig.txTransitions.success = 2;
    std_cfg.ackConfig.txTransitions.error = 2;
    // As transicoes de estado, como no exemplo do header.
    std_cfg.timings.idleToRx = 100;
    std_cfg.timings.idleToTx = 100;
    std_cfg.timings.rxToTx = 192;
    std_cfg.timings.txToRx = 192;
    let init_st = unsafe { rail::RAIL_IEEE802154_Init(handle, &std_cfg) };
    mailbox::write(Status::Info, b"154: protocol init");

    mailbox::write(Status::Info, b"154: phy configured");

    // ## O metodo de dados, antes da FIFO
    //
    // Uma transmissao por pedido, nao um fluxo continuo. O exemplo BLE vizinho configura isto e sem ele a
    // `RAIL_SetTxFifo` nao chega a regressar -- medido aqui: o ultimo marcador era "phy configured", e o
    // seguinte, que devia vir imediatamente depois da FIFO, nunca apareceu.
    let dc = rail::RAIL_DataConfig_t {
        txSource: 0, // TX_PACKET_DATA
        rxSource: 0, // RX_PACKET_DATA
        txMethod: 0, // PACKET_MODE
        rxMethod: 0, // PACKET_MODE
    };
    unsafe { rail::RAIL_ConfigData(handle, &dc) };
    mailbox::write(Status::Info, b"154: data method set");

    // A FIFO de TX: o `RAIL_StartTx` não leva o pacote, os bytes entram aqui primeiro.
    // ## Alinhamento: o buffer da FIFO e' alinhado a 4 bytes de proposito
    //
    // `[0u8; 128]` na pilha tem alinhamento 1, e a `RAIL_SetTxFifo` nao regressava com ele. Um tipo com
    // `repr(align(4))` e' a forma de dar ao vendor o que a assinatura dele espera sem inventar um endereco fixo.
    #[repr(align(4))]
    struct FifoBuf([u8; 128]);
    let mut fifo = FifoBuf([0u8; 128]);
    let tx_buf = &mut fifo.0;
    unsafe { rail::RAIL_SetTxFifo(handle, tx_buf.as_mut_ptr(), 0, 128) };

    // Um frame de dados 802.15.4: broadcast do PAN, curto em ambas as pontas.
    let mut frame = [0u8; 32];
    frame[0] = 0x41; // FCF baixo: dados, PAN ID comprimido
    frame[1] = 0x88; // FCF alto: destino curto (2), origem curta (2)
    frame[2] = 0x07; // número de sequência
    frame[3..5].copy_from_slice(&[0xFF, 0xFF]); // PAN de destino: broadcast
    frame[5..7].copy_from_slice(&[0xFF, 0xFF]); // endereço de destino: broadcast
    frame[7..9].copy_from_slice(&[0x34, 0x12]); // endereço de origem
    let n = 9 + push(&mut frame[9..], 0, b"dosim");

    mailbox::write(Status::Info, b"154: frame built");

    let escrito = unsafe { rail::RAIL_WriteTxFifo(handle, frame.as_ptr(), n as u16, true) };

    let mut mw = [0u8; 40];
    let mut aw = push(&mut mw, 0, b"154: wrote=");
    aw = push_num(&mut mw, aw, u32::from(escrito));
    mailbox::write(Status::Info, &mw[..aw]);

    let tx_st =
        unsafe { rail::RAIL_StartTx(handle, CHANNEL, rail::RAIL_TX_OPTIONS_DEFAULT, ptr::null()) };

    // Pedir o evento DEPOIS de toda a configuracao: a configuracao do PHY pode repor a mascara, e um pedido
    // feito antes dela nao chegaria ao rádio que transmite.
    unsafe { rail::RAIL_ConfigEvents(handle, 1u64 << 24, 1u64 << 24) };

    let mut mt = [0u8; 40];
    let mut at2 = push(&mut mt, 0, b"154: started=");
    at2 = push_num(&mut mt, at2, u32::from(tx_st));
    mailbox::write(Status::Info, &mt[..at2]);

    // Espera limitada pelo evento do rádio. O ciclo do CPU é a base de tempo porque o `Delay` do HAL é
    // calibrado pelo relógio que o Board conhece, e aqui o cristal foi seleccionado por fora (emlib).
    // ## Ligar o contador de ciclos, senao a espera e' infinita
    //
    // `dwt::cycles()` so' LE o contador; quem o liga e' `dwt::start()`. Sem esta chamada o `CYCCNT` fica em
    // zero, `since(t0)` devolve zero para sempre, e um laco "limitado" nunca termina. Medido aqui: o ultimo
    // marcador era "started=0" e o registo final nunca aparecia.
    let _ = unsafe { efr32mg2x_hal::dwt::start() };
    let t0 = efr32mg2x_hal::dwt::cycles();
    while efr32mg2x_hal::dwt::since(t0) < 39_000 * 50 {
        core::hint::spin_loop();
    }

    let hw = unsafe { TX_PACKET_SENT };

    let mut m = [0u8; 120];
    let mut at = 0;
    at = push(&mut m, at, b"154: init=");
    at = push_num(&mut m, at, u32::from(init_st));
    at = push(&mut m, at, b" phy=");
    at = push_num(&mut m, at, u32::from(phy_st));
    at = push(&mut m, at, b" pa=");
    at = push_num(&mut m, at, u32::from(pa_st));
    at = push(&mut m, at, b" dbm=");
    at = push_num(&mut m, at, u32::from(dbm_st));
    at = push(&mut m, at, b" w=");
    at = push_num(&mut m, at, u32::from(escrito));
    at = push(&mut m, at, b" tx=");
    at = push_num(&mut m, at, u32::from(tx_st));
    at = push(&mut m, at, b" hw=");
    at = push_num(&mut m, at, hw);

    // Passa só se a configuração foi aceite (0 = RAIL_STATUS_NO_ERROR), o `StartTx` foi aceite e o rádio
    // reportou pelo menos um envio. O que isto NÃO diz é que um frame saiu no ar -- ver o topo do ficheiro.
    let ok = phy_st == 0 && tx_st == 0 && hw > 0;
    mailbox::write(if ok { Status::Pass } else { Status::Fail }, &m[..at]);

    loop {
        delay.delay_ms(500);
    }
}
