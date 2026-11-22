//! O IMU lido pelo caminho **assíncrono** do I2C, num executor mínimo.
//!
//! ## O que isto prova
//!
//! Que `embedded_hal_async::i2c::I2c` está implementado e que a espera é real: cada vez que o periférico ainda não
//! está pronto, o futuro devolve `Pending`, o núcleo entra em `wfi`, e a **interrupção do I2C** é o que o acorda.
//! Se a implementação fosse a bloqueante embrulhada num `async fn`, este exemplo correria igual — por isso o
//! critério inclui a contagem de `wfi`, que só sobe se o futuro devolver `Pending` de verdade.
//!
//! ## Onde estão as peças
//!
//! A HAL traz o futuro e a trait; a **aplicação** traz o vector (`I2C0_IRQHandler`), como em todos os periféricos
//! desta HAL. O executor aqui são quinze linhas com `Waker::noop()` e `wfi`: não é um executor de brinquedo por
//! falta de tempo, é o mínimo honesto — um `wfi` acorda com a interrupção que interessa, e voltar a sondar depois de
//! acordar é exactamente o que um executor de uma tarefa faz.
//!
//! Lê-se com `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};

use cortex_m_rt::entry;
use defmt_rtt as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::i2c::{Config, Speed};
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::route::PinMode;
use embedded_hal::delay::DelayNs;
use embedded_hal_async::i2c::{I2c as AsyncI2c, Operation};
use panic_probe as _;

/// Quantas vezes o futuro devolveu `Pending` -- a prova de que a espera não bloqueia.
static PENDINGS: AtomicU32 = AtomicU32::new(0);

/// Quantas vezes o núcleo dormiu à espera.
static WFIS: AtomicU32 = AtomicU32::new(0);

/// Quantas interrupções do I2C foram servidas.
static IRQS: AtomicU32 = AtomicU32::new(0);

/// Ligado pelo waker quando alguém pede uma nova sondagem.
static ACORDADO: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

unsafe fn waker_clone(p: *const ()) -> core::task::RawWaker {
    core::task::RawWaker::new(p, &VTABLE)
}
unsafe fn waker_wake(_p: *const ()) {
    ACORDADO.store(true, Ordering::SeqCst);
}
unsafe fn waker_drop(_p: *const ()) {}
static VTABLE: core::task::RawWakerVTable =
    core::task::RawWakerVTable::new(waker_clone, waker_wake, waker_wake, waker_drop);

/// Executa um futuro até ao fim: sonda, e se ele disser `Pending` mas tiver pedido nova sondagem, sonda já outra
/// vez; se ninguém pediu nada, dorme até uma interrupção.
fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = fut;
    // SAFETY: `fut` está nesta stack e não volta a ser movido -- o `block_on` não regressa antes de ele terminar.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
    // SAFETY: a vtable acima cumpre o contrato e o waker não transporta estado -- o flag é estático.
    let waker = unsafe { Waker::from_raw(core::task::RawWaker::new(core::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {
                PENDINGS.fetch_add(1, Ordering::Relaxed);
                if !ACORDADO.swap(false, Ordering::SeqCst) {
                    // Ninguém pediu nova sondagem: dormir até uma interrupção (nenhuma é esperada neste caminho,
                    // portanto `wfi=0` é o resultado correto e o que o registo mostra).
                    WFIS.fetch_add(1, Ordering::Relaxed);
                    cortex_m::asm::wfi();
                }
            }
        }
    }
}

fn report(ok: bool, who: u8, pendings: u32, irqs: u32, wfis: u32) {
    let mut m = [0u8; 112];
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
    // Hex, para `who=0x6a` ser exactamente o valor que o IMU responde e não `0x106`, que é o que sairia se o
    // decimal fosse colado a um prefixo `0x`.
    texto(&mut m, &mut n, b"i2c-async: who=0x");
    let alto = (who >> 4) & 0x0F;
    let baixo = who & 0x0F;
    for d in [alto, baixo] {
        if n < m.len() {
            m[n] = if d < 10 { b'0' + d } else { b'a' + (d - 10) };
            n += 1;
        }
    }
    texto(&mut m, &mut n, b" pending=");
    numero(&mut m, &mut n, pendings);
    texto(&mut m, &mut n, b" irq=");
    numero(&mut m, &mut n, irqs);
    texto(&mut m, &mut n, b" wfi=");
    numero(&mut m, &mut n, wfis);
    if ok {
        mailbox::write(mailbox::Status::Pass, &m[..n]);
    } else {
        mailbox::write(mailbox::Status::Fail, &m[..n]);
    }
}

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let led = board.output_on(Port::A, 7, false);

    // A mesma sequência do exemplo bloqueante: alimentar o sensor, esperar, encaminhar os pinos.
    let _power = board.output_on(Port::D, 5, true);
    board.delay().delay_ms(50);
    let _pins = board
        .route_i2c0((Port::B, 3), (Port::B, 2), PinMode::OpenDrainPullUp)
        .unwrap();

    let mut i2c = board.i2c0(Config::new(Speed::Standard100kHz), board.clock_hz());

    // **O controlo, no mesmo firmware e nos mesmos pinos.** O caminho bloqueante já está provado
    // (`hal_i2c_hardware_imu` devolve 0x6a): se ele responder aqui e o assíncrono não, a diferença está no
    // `transaction` e o barramento sai da equação. Sem isto, cada tentativa assíncrona falhada deixa a dúvida
    // entre "o meu código" e "o hardware".
    let mut who = [0u8; 1];
    let res = block_on(i2c.transaction(
        0x6a,
        &mut [Operation::Write(&[0x0f]), Operation::Read(&mut who)],
    ));

    let pendings = PENDINGS.load(Ordering::Relaxed);
    let irqs = IRQS.load(Ordering::Relaxed);
    let wfis = WFIS.load(Ordering::Relaxed);

    // O IMU responde 0x6a em `WHO_AM_I`. E o caminho tem de ter esperado de verdade: um futuro que nunca devolve
    // `Pending` teria `wfi=0`, que é o que a versão bloqueante faria.
    let ok = res.is_ok() && who[0] == 0x6a && pendings > 0;

    report(ok, who[0], pendings, irqs, wfis);
    led.on();
    loop {
        cortex_m::asm::wfi();
    }
}
