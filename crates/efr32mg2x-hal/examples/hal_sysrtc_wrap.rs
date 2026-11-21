//! A volta do contador de 32 bits, exercida em milissegundos em vez de 36,4 horas.
//!
//! ## O que isto prova
//!
//! O contador do SYSRTC tem 32 bits a 32768 Hz: volta a zero a cada **36,4 horas**. O tick de 64 bits precisa de
//! saber quantas voltas já houve, e havia duas maneiras de o saber:
//!
//! - **Por sondagem** (`WideTick`): compara cada leitura com a anterior e conta uma volta quando o valor desce.
//!   Funciona, mas exige que alguém chame `now_wide` pelo menos uma vez a cada 36,4 h -- quem não chamar perde
//!   exactamente 36,4 h do relógio, em silêncio.
//! - **Por interrupção** (`GRP0_IF.OVF`): o periférico conta as voltas e o `now_wide` não precisa de ninguém.
//!
//! Este exemplo arma a interrupção, **põe o contador 4096 ticks antes da volta** e verifica que o tick de 64 bits
//! atravessa a volta a subir, não a descer. Se o bit alto viesse da sondagem, ou não existisse, o segundo valor
//! seria *menor* que o primeiro.
//!
//! ## Porque é que a sondagem tem de ficar desligada
//!
//! As duas contariam a mesma volta: a sondagem veria o contador descer e somaria uma, e a interrupção somaria
//! outra. O tick saltaria 36,4 h para a frente. Por isso `enable_overflow_interrupt` retira a sondagem do caminho
//! -- é uma escolha, não uma soma.
//!
//! Lê-se com `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::sysrtc::Sysrtc;
use panic_probe as _;

/// Quantas voltas o handler contou.
static VOLTAS_NO_HANDLER: AtomicU32 = AtomicU32::new(0);

/// Quantas vezes o handler correu, para distinguir "não voltou" de "voltou e não serviu o evento".
static ENTRADAS_NO_HANDLER: AtomicU32 = AtomicU32::new(0);

/// O valor posto no contador: 4096 ticks antes da volta, ou seja 125 ms.
const ANTES_DA_VOLTA: u32 = 0xFFFF_F000;

/// Ticks de 4096 a 32768 Hz.
const TICKS_ATE_VOLTAR: u32 = 0x0000_1000;

/// Um limite de voltas no ciclo, para um erro não ficar a girar para sempre.
const LIMITE_VOLTAS: u32 = 40_000_000;

/// O PAC declara `PROVIDE(SYSRTC_APP_IRQHandler = DefaultHandler)` em `device.x`, portanto uma definição com este
/// nome exacto toma o lugar do handler que não regressa.
#[no_mangle]
pub extern "C" fn SYSRTC_APP_IRQHandler() {
    ENTRADAS_NO_HANDLER.fetch_add(1, Ordering::Relaxed);
    // Os dois eventos partilham um vector: servir só o comparador deixaria o flag da volta posto e o vector
    // voltaria a entrar para sempre.
    let (_, voltou) = Sysrtc::service_interrupts();
    if voltou {
        VOLTAS_NO_HANDLER.fetch_add(1, Ordering::Relaxed);
    }
}

fn report(ok: bool, wraps: u32, entradas: u32, antes: u32, depois: u32, delta: u32, ticks: u32) {
    let mut m = [0u8; 140];
    let mut n = 0usize;

    fn texto(m: &mut [u8], n: &mut usize, s: &[u8]) {
        for b in s {
            if *n < m.len() {
                m[*n] = *b;
                *n += 1;
            }
        }
    }
    fn numero(m: &mut [u8], n: &mut usize, v: u32) {
        let mut d = [0u8; 10];
        let mut k = 0usize;
        let mut v = v;
        loop {
            d[k] = b'0' + (v % 10) as u8;
            v /= 10;
            k += 1;
            if v == 0 || k == d.len() {
                break;
            }
        }
        while k > 0 {
            k -= 1;
            if *n < m.len() {
                m[*n] = d[k];
                *n += 1;
            }
        }
    }

    texto(&mut m, &mut n, b"wrap: wraps=");
    numero(&mut m, &mut n, wraps);
    texto(&mut m, &mut n, b" irq=");
    numero(&mut m, &mut n, entradas);
    texto(&mut m, &mut n, b" before=");
    numero(&mut m, &mut n, antes);
    texto(&mut m, &mut n, b" after=");
    numero(&mut m, &mut n, depois);
    texto(&mut m, &mut n, b" delta=");
    numero(&mut m, &mut n, delta);
    texto(&mut m, &mut n, b" ticks=");
    numero(&mut m, &mut n, ticks);

    if ok {
        mailbox::write(mailbox::Status::Pass, &m[..n]);
    } else {
        mailbox::write(mailbox::Status::Fail, &m[..n]);
    }
}

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let mut led = board.output_on(Port::A, 7, false);
    let rtc = board.sysrtc();

    // Armar primeiro a interrupção, senão a volta acontece antes de alguém a contar.
    rtc.enable_overflow_interrupt();
    rtc.unmask_interrupt();

    // 4096 ticks antes da volta. É o que torna um evento de 36,4 h num evento de 125 ms.
    rtc.set_counter(ANTES_DA_VOLTA);

    let antes = rtc.now_wide();
    let baixo_antes = antes as u32;

    // Não se chama `now_wide` durante a volta de propósito: o valor tem de vir do handler, não de uma leitura.
    let mut voltas_de_espera = 0u32;
    while VOLTAS_NO_HANDLER.load(Ordering::Relaxed) == 0 {
        cortex_m::asm::wfi();
        voltas_de_espera += 1;
        if voltas_de_espera > LIMITE_VOLTAS {
            break;
        }
    }

    let depois = rtc.now_wide();
    let wraps = VOLTAS_NO_HANDLER.load(Ordering::Relaxed);
    let entradas = ENTRADAS_NO_HANDLER.load(Ordering::Relaxed);
    let delta = depois.wrapping_sub(antes) as u32;
    let ticks = rtc.now();

    // O tick tem de subir. Com o bit alto vindo da sondagem desligada, ou ausente, `depois` seria menor que
    // `antes` e este teste diria exactamente isso.
    let subiu = depois > antes;
    // A distância entre os dois instantes é o que faltava para a volta mais o que já passou depois dela. Se o bit
    // alto não tivesse entrado, `delta` seria uma diferença de 32 bits sem sentido -- daí a janela.
    let coerente = delta >= TICKS_ATE_VOLTAR && delta <= TICKS_ATE_VOLTAR + 3_276;
    let ok = wraps == 1 && subiu && coerente;

    report(
        ok,
        wraps,
        entradas,
        baixo_antes,
        depois as u32,
        delta,
        ticks,
    );
    led.on();

    loop {
        cortex_m::asm::wfi();
    }
}
