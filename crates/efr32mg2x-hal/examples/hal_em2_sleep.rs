//! EM2: dez sonecas de um segundo, com o SYSRTC a acordar.
//!
//! ## O que este exemplo prova
//!
//! Que o núcleo **entra** em EM2 e **volta**, e que o caminho de volta é o que se pediu: o SYSRTC
//! acorda o núcleo por comparação, e `take_compare_event()` diz se foi ele. Sem essa verificação, um
//! `wfi` que regressa imediatamente parecia igual a um sono de um segundo — o contador diria dez
//! sonecas e o tempo não teria passado.
//!
//! ## Como se lê o resultado
//!
//! `defmt` pela RTT, uma linha por acordar. Com `--release` a compilar e a placa ligada ao lnx:
//!
//! ```text
//! probe-rs run --chip EFR32MG24B220F1536IM48 target/thumbv8m.main-none-eabihf/release/examples/hal_em2_sleep
//! ```
//!
//! O sinal de sucesso não é a ausência de erro, é o **relógio da parede**: dez linhas separadas por
//! cerca de um segundo, e a última depois de ~10 s. Sintomas que se distinguem assim:
//!
//! - dez linhas de imediato -> o sono não aconteceu (`SLEEPDEEP` não entrou, ou o `wfi` regressou);
//! - uma linha e depois nada -> acordou não acordou: a interrupção não estava armada;
//! - nenhuma linha, RTT morta -> entrou em EM2 e o debugger foi com ele (`keep_debugger` em falta).

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use panic_halt as _;

use defmt_rtt as _;

/// Quantas vezes o comparador do SYSRTC acordou o nucleo.
///
/// O handler escreve aqui porque o `main` esta' dentro do `wfi` quando isto acontece: nao ha' como emprestar-lhe o
/// `Sysrtc`, e sem um sitio partilhado onde contar, o exemplo nao conseguia distinguir "acordou dez vezes" de
/// "acordou uma e ficou preso".
static ACORDARES: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// O handler que faltava.
///
/// A primeira versao deste exemplo desmascarava o `NVIC` sem registar handler nenhum, e o proprio HAL avisa disso:
/// "unmasking before a handler exists sends the first event to the weak default handler, which loops forever".
/// Medido na placa: uma linha de RTT e depois silencio -- o nucleo acordou, entrou no `DefaultHandler` e nunca
/// voltou. E' o segundo sintoma da lista escrita antes do teste, e foi o que aconteceu.
/// O simbolo forte que substitui o default.
///
/// O PAC declara `PROVIDE(SYSRTC_APP_IRQHandler = DefaultHandler)` em `device.x`, portanto uma definicao com este
/// nome exacto toma o lugar do handler que nao regressa. O atributo `#[interrupt]` do `cortex-m-rt` nao esta
/// disponivel aqui -- precisa da feature `device`, que a HAL nao liga -- e o nome do vector e' o que o linker
/// procura, nao um detalhe de sintaxe.
#[no_mangle]
pub extern "C" fn SYSRTC_APP_IRQHandler() {
    efr32mg2x_hal::sysrtc::Sysrtc::acknowledge_compare_interrupt();
    ACORDARES.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
}

/// O SYSRTC conta a 32768 ticks por segundo.
const TICKS_1S: u32 = 32_768;

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let mut led = board.output_on(Port::A, 7, false);
    let rtc = board.sysrtc();
    let sleep = board.sleep();

    // Antes de dormir, uma vez: o debugger tem de sobreviver a EM2, senão perdemos a linha de
    // relato no primeiro sono e o que resta é uma placa muda.
    sleep.keep_debugger();

    // A interrupção do comparador tem de estar ligada no periférico antes do primeiro `wfi`.
    rtc.unmask_interrupt();

    // ## Fase 1: o comparador sozinho, sem EM2 nenhum
    //
    // Duas coisas foram mudadas de uma vez -- o sono e o caminho da interrupcao -- e o resultado "uma linha e
    // silencio" nao diz qual das duas falhou. Esta fase tira o sono da equacao: se o comparador acordar o nucleo
    // aqui, o problema esta' no EM2; se ficar calado, o comparador nunca disparou e o EM2 nao tem culpa nenhuma.
    defmt::info!("fase 1: comparador sem EM2");
    // Antes de esperar por uma interrupcao, ver se ha' um contador a andar. Quatro numeros dizem qual das tres
    // coisas esta' em falta: o periferico nao estar ligado (enabled), o relogio nao o fazer contar (running), ou o
    // pedido de comparacao ser recusado (armado). Sem isto, "nao dispara" tem tres causas e nenhum culpado.
    let hz_antes = rtc.now();
    let ligado = rtc.is_enabled();
    let a_correr = rtc.is_running();
    let armado = rtc.schedule_in(TICKS_1S);
    // Uma pausa curta e' uma medida: se o contador nao avancou, nao ha' relogio e nenhuma comparacao vai disparar.
    for _ in 0..4_000_000 {
        cortex_m::asm::nop();
    }
    let hz_depois = rtc.now();
    defmt::info!(
        "contador: enabled={} running={} armado={} antes={} depois={} andou={}",
        ligado,
        a_correr,
        armado,
        hz_antes,
        hz_depois,
        hz_depois.wrapping_sub(hz_antes)
    );
    let antes_da_fase_1 = ACORDARES.load(core::sync::atomic::Ordering::Relaxed);
    let mut voltas = 0u32;
    while ACORDARES.load(core::sync::atomic::Ordering::Relaxed) == antes_da_fase_1 {
        cortex_m::asm::nop();
        voltas += 1;
        if voltas > 40_000_000 {
            defmt::info!("fase 1 FALHOU: 40 M de voltas e o comparador nunca disparou");
            break;
        }
    }
    if ACORDARES.load(core::sync::atomic::Ordering::Relaxed) > antes_da_fase_1 {
        defmt::info!("fase 1 ok: o comparador disparou e o handler correu");
    }

    defmt::info!("EM2: 10 sonecas de ~1 s, SYSRTC a acordar");

    for i in 0..10u32 {
        led.set(true);
        let armado = rtc.schedule_in(TICKS_1S);
        let antes = ACORDARES.load(core::sync::atomic::Ordering::Relaxed);
        sleep.enter();
        // O handler ja' limpou a flag; o que se le aqui e' o contador que ele subiu.
        let depois = ACORDARES.load(core::sync::atomic::Ordering::Relaxed);
        let foi_o_compare = depois > antes;
        led.set(false);
        defmt::info!(
            "acordar #{} armado={} compare={} conta={}",
            i,
            armado,
            foi_o_compare,
            depois
        );
    }

    defmt::info!(
        "fim: 10 sonecas, {} acordares pelo comparador do SYSRTC",
        ACORDARES.load(core::sync::atomic::Ordering::Relaxed)
    );

    // Fica quieto, mas acordado: se entrasse em EM2 outra vez sem interrupção armada, o exemplo
    // nunca chegaria a dizer o resultado.
    loop {
        cortex_m::asm::nop();
    }
}
