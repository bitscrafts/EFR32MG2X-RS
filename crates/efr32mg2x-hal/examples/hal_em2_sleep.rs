//! EM2: ten one-second naps, with the SYSRTC doing the waking.
//!
//! ## What this example proves
//!
//! That the core **enters** EM2 and **comes back**, and that the way back is the one that was asked for: the SYSRTC
//! wakes the core by comparison, and `take_compare_event()` says whether it was that. Without the check, a `wfi` that
//! returns immediately looks just like a one-second sleep -- the counter would report ten naps and no time would have
//! passed.
//!
//! ## How to read the result
//!
//! `defmt` over RTT, one line per wake-up. With `--release` built and the board attached to lnx:
//!
//! ```text
//! probe-rs run --chip EFR32MG24B220F1536IM48 target/thumbv8m.main-none-eabihf/release/examples/hal_em2_sleep
//! ```
//!
//! The sign of success is not the absence of an error, it is the **wall clock**: ten lines about a second apart, the
//! last one after ~10 s. The symptoms it tells apart:
//!
//! - ten lines at once -> the sleep did not happen (`SLEEPDEEP` never took, or the `wfi` returned);
//! - one line and then nothing -> it woke, it did not wake: the interrupt was not armed;
//! - no line at all, dead RTT -> it entered EM2 and the debugger went with it (`keep_debugger` missing).

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
    let led = board.output_on(Port::A, 7, false);
    let rtc = board.sysrtc().expect("SYSRTC is handed out once");
    let sleep = board.sleep();

    // Before sleeping, once: the debugger has to survive EM2, or the report line is lost on the first sleep and what
    // is left is a silent board.
    sleep.keep_debugger();

    // The comparator's interrupt has to be enabled in the peripheral before the first `wfi`.
    rtc.unmask_interrupt();

    // ## Phase 1: the comparator alone, with no EM2 at all
    //
    // Two things were changed at once -- the sleep and the interrupt path -- and "one line then silence" does not say
    // which of the two failed. This phase takes the sleep out of the equation: if the comparator wakes the core
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

    // Quiet, but awake: entering EM2 again with no interrupt armed would mean this example never got to say what the
    // result was.
    loop {
        cortex_m::asm::nop();
    }
}
