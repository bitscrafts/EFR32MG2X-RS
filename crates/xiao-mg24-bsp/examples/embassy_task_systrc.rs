//! Uma tarefa do `embassy-executor` a dormir com `Timer`, sobre o SYSRTC.
//!
//! ## Porque é este o teste que faltava
//!
//! O driver de tempo está provado (`embassy_driver_systrc`: `schedule_wake` → comparador → interrupção → waker,
//! dez vezes). O que faltava era o **consumidor**: alguém que use `embassy_time::Timer` em código `async` e acorde
//! por ele. O driver e o executor falham de maneiras diferentes -- um executor sem driver fica parado com as tarefas
//! por correr, um driver sem executor chama um waker que ninguém atende -- e provar os dois juntos teria escondido
//! qual dos dois quebrou. Por isso são dois exemplos, e este é o segundo.
//!
//! ## O número que decide
//!
//! `Timer::after_millis(100)` dez vezes, com o tempo medido por `Instant::now()` do próprio `embassy-time` -- ou
//! seja, pelo relógio que o driver fornece. Se o `SYSRTC` estivesse a 19 MHz em vez de 32 768 Hz, ou se o comparador
//! armasse no instante errado, o total sairia fora de 1000 ms.
//!
//! Lê-se com `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use embassy_efr32::Platform;
use embassy_executor::Executor;
use embassy_time::{Duration, Instant, Timer};
use panic_halt as _;
use xiao_mg24_bsp::hal::board::Board;
use xiao_mg24_bsp::hal::sysrtc::Sysrtc;

/// Quantas entregas o comparador fez, para distinguir "dormiu" de "girou".
static ACORDARES: AtomicU32 = AtomicU32::new(0);

/// Quantas esperas a tarefa faz.
const ESPERAS: u32 = 10;

/// O `Platform` que o driver de tempo usa.
struct SysrtcPlatform;

impl Platform for SysrtcPlatform {
    fn ticks_64() -> u64 {
        Sysrtc::now_wide_global()
    }

    fn arm(value: u32, _rearm: bool) {
        Sysrtc::schedule_at_global(value);
    }
}

embassy_efr32::time_driver!(SysrtcPlatform);

/// O comparador: reconhece o evento e entrega-o ao driver, que chama o waker da tarefa à espera.
#[no_mangle]
pub extern "C" fn SYSRTC_APP_IRQHandler() {
    Sysrtc::acknowledge_compare_interrupt();
    ACORDARES.fetch_add(1, Ordering::Relaxed);
    embassy_efr32::on_alarm::<SysrtcPlatform>();
}

/// O executor. `static mut` e um `Option` porque esta é uma placa de um núcleo e o executor tem de viver para
/// sempre: um `StaticCell` faria o mesmo com mais uma dependência.
static mut EXECUTOR: Option<Executor> = None;

#[embassy_executor::task]
async fn dorme_e_conta() {
    let inicio = Instant::now();
    for _ in 0..ESPERAS {
        Timer::after_millis(100).await;
    }
    let ms = inicio.elapsed().as_millis() as u32;

    let mut m = [0u8; 96];
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

    texto(&mut m, &mut n, b"task: waits=");
    numero(&mut m, &mut n, ESPERAS);
    texto(&mut m, &mut n, b" ms=");
    numero(&mut m, &mut n, ms);
    texto(&mut m, &mut n, b" irq=");
    numero(&mut m, &mut n, ACORDARES.load(Ordering::Relaxed));

    let esperado = ESPERAS * 100;
    let margem = esperado / 20;
    let ok = ms >= esperado - margem && ms <= esperado + margem;
    if ok {
        mailbox::write(mailbox::Status::Pass, &m[..n]);
    } else {
        mailbox::write(mailbox::Status::Fail, &m[..n]);
    }

    // A tarefa não termina: um executor sem tarefas sai, e o exemplo tem de continuar a dormir para o registo poder
    // ser lido depois de o programa ter feito o que prometeu.
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    board.sysrtc().unmask_interrupt();

    let executor = unsafe { (*core::ptr::addr_of_mut!(EXECUTOR)).insert(Executor::new()) };
    // `Executor::run` devolve `!`: nunca regressa, e é por isso que não há código depois dele. O compilador disse-o
    // -- "unreachable statement" -- e um `loop {}` a seguir seria uma afirmação falsa sobre o que o executor faz.
    executor.run(|spawner| {
        spawner.spawn(dorme_e_conta()).unwrap();
    })
}
