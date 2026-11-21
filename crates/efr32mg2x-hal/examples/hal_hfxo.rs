//! HFXO: correr do cristal, e provar que o relógio do sistema o seguiu.
//!
//! ## O que este exemplo mede
//!
//! Não é "o cristal arrancou" — isso o `select_hfxo` devolve. É que **o resto do sistema saiba que ele arrancou**:
//! cinco `delay_ms(1000)` depois de subir o núcleo para os 39 MHz têm de durar cerca de um segundo cada no relógio
//! da parede (as datas das linhas de RTT, que são do host).
//!
//! Se o `SysTick` ficasse calibrado para os 19 MHz de reset, cada 1000 ms pedidos durariam ~0,49 s, e o total dos
//! cinco seria ~2,5 s em vez de ~5 s. Esse é o número que distingue "o cristal arrancou" de "o sistema sabe".
//!
//! ## O que acontece se o cristal não arrancar
//!
//! `ClockError::HfxoTimeout`, e o `Board` fica no relógio que tinha. O exemplo diz isso e continua — uma placa que
//! fica muda é indistinguível de uma avariada.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::prelude::*;
use panic_halt as _;

use defmt_rtt as _;

/// O cristal do XIAO MG24.
const HFXO_HZ: u32 = 39_000_000;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    defmt::info!("antes: clock_hz = {} Hz", board.clock_hz().0);

    match board.select_hfxo(HFXO_HZ) {
        Ok(hz) => defmt::info!("HFXO: a correr a {} Hz", hz.0),
        Err(_) => {
            defmt::info!("HFXO FALHOU: o cristal nao estabilizou (HfxoTimeout), fico no HFRCO")
        }
    }

    // Construido DEPOIS da mudanca de relogio, de proposito: e' o que prova que o delay se calibra pelo relogio em
    // vigor e nao pelo de reset.
    let mut delay = board.delay();

    for i in 0..5u32 {
        delay.delay_ms(1000);
        defmt::info!("marca {} (1000 ms pedidos)", i);
    }

    defmt::info!("fim: cinco segundos pedidos");

    loop {
        cortex_m::asm::nop();
    }
}
