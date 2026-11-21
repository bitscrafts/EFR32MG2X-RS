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

    defmt::info!("EM2: 10 sonecas de ~1 s, SYSRTC a acordar");

    let mut acordares_com_comparacao = 0u32;
    for i in 0..10u32 {
        led.set(true);
        let armado = rtc.schedule_in(TICKS_1S);
        sleep.enter();
        let foi_o_compare = rtc.take_compare_event();
        if foi_o_compare {
            acordares_com_comparacao += 1;
        }
        led.set(false);
        defmt::info!(
            "acordar #{} armado={} compare={} conta={}",
            i,
            armado,
            foi_o_compare,
            acordares_com_comparacao
        );
    }

    defmt::info!(
        "fim: 10 sonecas, {} acordares pelo comparador do SYSRTC",
        acordares_com_comparacao
    );

    // Fica quieto, mas acordado: se entrasse em EM2 outra vez sem interrupção armada, o exemplo
    // nunca chegaria a dizer o resultado.
    loop {
        cortex_m::asm::nop();
    }
}
