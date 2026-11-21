//! O driver de tempo do Embassy sobre o SYSRTC, com um waker que conta.
//!
//! ## O que isto fecha
//!
//! A crate `embassy-efr32` tem o driver, a escolha do prazo e a conversão 64→32 bits testadas no host, e a própria
//! documentação dela nomeia o que faltava: *"That a comparator interrupt wakes a task on the board does **not**
//! [prove] -- it needs the board, a deadline shorter than a second and a waker that prints."* Este exemplo é esse
//! teste, e o waker conta em vez de imprimir porque o registo tem de caber num sítio que se lê por SWD.
//!
//! ## Porque é que não há `embassy-executor` aqui
//!
//! O que se quer provar é a **cadeia do driver**: `schedule_wake` escolhe o prazo, arma o comparador, a
//! interrupção dispara, e o waker que ficou guardado é chamado. Um executor é o consumidor dessa cadeia, não parte
//! dela -- e prová-lo agora provaria também o executor, que é código de outra pessoa e esconderia qual das duas
//! metades falhou. Com um waker próprio, o que falha é sempre o driver.
//!
//! `embassy_time_driver::now()` e `schedule_wake()` são as funções globais que qualquer consumidor (incluindo
//! `embassy-time`) usa; são elas que este exemplo chama.
//!
//! ## O controlo
//!
//! O total é medido pelo tick de 64 bits do próprio driver e por um segundo relógio independente (o DWT), como no
//! `hal_sysrtc_wide`. O DWT é reiniciado **depois** do último `wfi`: mediu-se que ele congela no primeiro sono e
//! não retoma.
//!
//! Lê-se com `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{RawWaker, RawWakerVTable, Waker};

use cortex_m_rt::entry;
use embassy_efr32::Platform;
use panic_halt as _;
use xiao_mg24_bsp::hal::board::Board;
use xiao_mg24_bsp::hal::pin::Port;
use xiao_mg24_bsp::hal::sysrtc::Sysrtc;

/// Quantas vezes o waker foi chamado.
static ACORDARES: AtomicU32 = AtomicU32::new(0);

/// Uma espera de 100 ms em ticks do SYSRTC.
const TICKS_100MS: u64 = 3_277;

/// Quantas esperas o exemplo faz.
const ESPERAS: u32 = 10;

/// O `Platform` que o driver usa: o relógio vem do HAL, o comparador também.
struct SysrtcPlatform;

impl Platform for SysrtcPlatform {
    fn ticks_64() -> u64 {
        // A versão global do relógio, e não um handle: o `Platform` é um traço de funções estáticas, chamado de
        // contextos que não podem possuir o periférico. O HAL guarda o endereço quando o módulo é tomado, e o
        // alargamento é o mesmo estado global -- um relógio, uma resposta.
        Sysrtc::now_wide_global()
    }

    fn arm(value: u32, _rearm: bool) {
        // O `_rearm` é verdadeiro quando o prazo é mais distante do que os 32 bits do contador. Com prazos de
        // 100 ms nunca acontece; quando acontecer, o driver volta a chamar `schedule_wake` no prazo que armou
        // agora, e essa chamada é que rearma. Ignorá-lo aqui não perde a espera -- perde-se, sim, se este
        // `arm` fosse o único sítio onde alguém rearmasse, e não é.
        Sysrtc::schedule_at_global(value);
    }
}

// Instala este driver como o driver de tempo do `embassy-time-driver`.
embassy_efr32::time_driver!(SysrtcPlatform);

// --- o waker ---------------------------------------------------------------------------------

unsafe fn waker_clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &VTABLE)
}

unsafe fn waker_wake(_p: *const ()) {
    ACORDARES.fetch_add(1, Ordering::Relaxed);
}

unsafe fn waker_wake_by_ref(_p: *const ()) {
    ACORDARES.fetch_add(1, Ordering::Relaxed);
}

unsafe fn waker_drop(_p: *const ()) {}

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop);

fn waker() -> Waker {
    // SAFETY: a vtable acima cumpre o contrato, e o waker não transporta estado nenhum -- o contador é estático.
    unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
}

/// O handler do comparador: reconhece o evento e entrega-o ao driver.
///
/// Sem a entrega, o comparador dispara e o waker nunca é chamado: o núcleo acorda, não faz nada e volta a dormir.
#[no_mangle]
pub extern "C" fn SYSRTC_APP_IRQHandler() {
    Sysrtc::acknowledge_compare_interrupt();
    embassy_efr32::on_alarm::<SysrtcPlatform>();
}

fn report(ok: bool, wakes: u32, ms: u32, wall_ms: u32) {
    let mut m = [0u8; 128];
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

    texto(&mut m, &mut n, b"emb: wakes=");
    numero(&mut m, &mut n, wakes);
    texto(&mut m, &mut n, b" driver_ms=");
    numero(&mut m, &mut n, ms);
    texto(&mut m, &mut n, b" wall_ms=");
    numero(&mut m, &mut n, wall_ms);

    if ok {
        mailbox::write(mailbox::Status::Pass, &m[..n]);
    } else {
        mailbox::write(mailbox::Status::Fail, &m[..n]);
    }
}

#[entry]
fn main() -> ! {
    // O mailbox da BSP guarda o ultimo registo; escrever por cima e o que se quer aqui.
    let board = Board::take().unwrap();
    let mut led = board.output_on(Port::A, 7, false);

    // `board.sysrtc()` também regista a base no HAL, que é o que os acessores globais usam.
    board.sysrtc().unmask_interrupt();

    let hw_hz: u32 = board.clock_hz().into();
    let w = waker();

    // O tempo do driver é o tempo do SYSRTC: o `now()` daqui é a mesma contagem que `Platform::ticks_64` devolve.
    let t0 = embassy_time_driver::now();
    for _ in 0..ESPERAS {
        let alvo = embassy_time_driver::now() + TICKS_100MS;
        let antes = ACORDARES.load(Ordering::Relaxed);
        // É esta chamada que decide o prazo e arma o comparador -- o caminho que este exemplo existe para provar.
        embassy_time_driver::schedule_wake(alvo, &w);
        let mut voltas = 0u32;
        while ACORDARES.load(Ordering::Relaxed) == antes {
            cortex_m::asm::wfi();
            voltas += 1;
            if voltas > 4_000_000 {
                break;
            }
        }
    }
    let ticks = (embassy_time_driver::now() - t0) as u32;
    let wakes = ACORDARES.load(Ordering::Relaxed);

    // O controlo independente, medido depois dos sonos: o DWT congela no primeiro `wfi`, por isso a fase que ele
    // mede é uma espera a girar, com o mesmo prazo do driver.
    let _ = unsafe { xiao_mg24_bsp::hal::dwt::start() };
    let ciclos0 = xiao_mg24_bsp::hal::dwt::cycles();
    let t_gira0 = embassy_time_driver::now();
    while (embassy_time_driver::now() - t_gira0) < (ESPERAS as u64) * TICKS_100MS {
        cortex_m::asm::nop();
    }
    let ciclos = xiao_mg24_bsp::hal::dwt::cycles().wrapping_sub(ciclos0);

    let driver_ms = (ticks as u64 * 1000 / 32_768) as u32;
    let wall_ms = if hw_hz == 0 {
        0
    } else {
        (ciclos as u64 * 1000 / u64::from(hw_hz)) as u32
    };

    let esperado = ESPERAS * 100;
    let margem = esperado / 20;
    let perto = |v: u32| v >= esperado - margem && v <= esperado + margem;
    let ok = wakes >= ESPERAS && perto(driver_ms) && perto(wall_ms);

    report(ok, wakes, driver_ms, wall_ms);
    led.on();

    loop {
        cortex_m::asm::wfi();
    }
}
