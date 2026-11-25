//! The IMU on the hardware I2C bus: routed pins, a power gate, one byte back.
//!
//! ## What this proves, and what every other I2C example could not
//!
//! Every example in this crate so far talked to the sensor through a bit-banged bus, because the
//! hardware path could not be reached: `I2c0::new` asked for a `FrozenClocks` that owns the CMU, and
//! the `Board` owns the same CMU, so no `Board` could build an `I2c0`. The routing registers were
//! never written either, so even a driver built by hand would have reached no pad.
//!
//! This example is the answer to both. It routes SDA = PB2 and SCL = PB3 -- the opposite of the
//! vendor's Arduino variant, which is why scanning the documented pins finds nothing -- puts them in
//! open-drain, powers the sensor on PD5, and asks the chip who it is.
//!
//! A `WHO_AM_I` of `0x6A` means: the peripheral reached a real pin, the pins were driven the right
//! way, the bus rate was right, and the device answered. Any one of those wrong and it returns an
//! error or a different byte.
//!
//! ## One thing this does not do
//!
//! It does not wait after powering the sensor. If the read fails, the first suspect is that delay,
//! not the routing -- and saying so here is cheaper than rediscovering it.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::i2c::{Config, Speed};
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::prelude::*;
use efr32mg2x_hal::route::PinMode;
use panic_halt as _;

use defmt_rtt as _;

/// The IMU answers here when its SA0 pin is strapped high, which is how this board wires it.
const IMU_ADDR: u8 = 0x6A;
/// `WHO_AM_I` on the LSM6DS3TR-C.
const WHO_AM_I: u8 = 0x0F;
/// The value it answers with.
const EXPECTED: u8 = 0x6A;

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();

    // The sensor is behind a power gate: an unpowered IMU is indistinguishable from an absent one.
    let _power = board.output_on(Port::D, 5, true);
    // The sensor needs time to come up. Without this the first read races the power gate.
    board.delay().delay_ms(50);

    // SDA = PB2, SCL = PB3. Mode first, then route, then enable -- the Board does that order.
    match board.route_i2c0((Port::B, 3), (Port::B, 2), PinMode::OpenDrainPullUp) {
        Ok(()) => defmt::info!("routed: SCL = PB3, SDA = PB2, open-drain with pull-up"),
        Err(e) => {
            defmt::error!("routing refused: {:?}", defmt::Debug2Format(&e));
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    let mut i2c = board
        .i2c0(Config::new(Speed::Standard100kHz), board.clock_hz())
        .expect("I2C0 is handed out once");

    let mut id = [0u8; 1];
    match i2c.write_read(IMU_ADDR, WHO_AM_I, &mut id) {
        Ok(()) => {
            if id[0] == EXPECTED {
                defmt::info!(
                    "WHO_AM_I = {=u8:#04x} -- the hardware bus reached the sensor",
                    id[0]
                );
            } else {
                defmt::warn!("WHO_AM_I = {=u8:#04x}, expected {=u8:#04x}: something answered, the wrong thing", id[0], EXPECTED);
            }
        }
        Err(e) => defmt::error!(
            "no answer on the hardware bus: {:?}",
            defmt::Debug2Format(&e)
        ),
    }

    loop {
        cortex_m::asm::wfi();
    }
}
