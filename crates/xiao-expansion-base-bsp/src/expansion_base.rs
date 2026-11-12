//! The Expansion Base facade: this board, with its devices already wired.
//!
//! ## What the layers own
//!
//! ```text
//!   driver (HAL)   how to talk to an SSD1306 / BME68x / PCF8563 / LSM6DS3TR-C
//!   BSP (here)     which device is on this board, at which address, on which bus,
//!                  behind which power gate, and which pins they contend for
//!   application     what to do with the readings
//! ```
//!
//! The point of this file is that an application never sees `0x3C`, `0x51`, `0x76`,
//! `0x6A`, `PC04`, `PB2` or `PD5`. It asks for `oled()`, `environment()`, `rtc()`,
//! `imu()` and gets something ready to use.
//!
//! ## Two buses, and why that is not an accident
//!
//! The panel, the BME68x and the RTC sit on the XIAO's I2C pins (PC04/PC05, the
//! pins the Expansion Base connects to). The IMU on the Sense module has its own
//! pair (PB2/PB3) and its own power gate (PD5). A facade that hid this would have
//! to expose a "which bus" argument anyway, so it hides the pins and keeps the two
//! buses distinct — measured facts, documented in
//! `docs/04-reference/IMU-INVESTIGATION.md`.
//!
//! ## Conflicts
//!
//! Every device here is on a bus, so construction cannot fail because two of them
//! want one pin. The one genuine conflict is the microSD card, whose chip select
//! shares a pin on this board; when that driver lands, this facade is where the
//! conflict is refused rather than silently resolved.

use core::cell::RefCell;

use efr32mg24_hal::board::i2c_static;
use efr32mg24_hal::bus::{BitBangI2c, Bus, I2cBus, SharedBus};
use efr32mg24_hal::display::Ssd1306;
use efr32mg24_hal::pin::{Input, Output, Port};
use efr32mg24_hal::rtc::pcf8563::Pcf8563;
use efr32mg24_hal::sensors::bme68x::Bme68x;
use efr32mg24_hal::sensors::lsm6ds3trc::{AccelOdr, AccelRange, Config, ImuError, Lsm6ds3trc};
use xiao_mg24_bsp::Module;

/// The XIAO MG24 Sense mounted on the Seeed Expansion Base.
pub struct ExpansionBase {
    /// The XIAO module this carrier sits on. Every pin, gate and bus below is reached through it.
    module: Module,
    /// PC04/PC05: panel, BME68x, RTC.
    base_bus: RefCell<BitBangI2c<'static>>,
    /// PB2/PB3: the on-board IMU.
    imu_bus: RefCell<BitBangI2c<'static>>,
    imu_powered: bool,
}

impl ExpansionBase {
    /// Take the board and prepare both buses.
    ///
    /// Returns `None` if the peripherals were already taken, which is what the PAC
    /// singleton enforces.
    pub fn take() -> Option<Self> {
        let module = Module::take()?;
        // The buses are built from the `'static` GPIO view so this struct can own
        // them; borrowing from the module would make it self-referential.
        let base_bus = RefCell::new(i2c_static(Bus::new(Port::C, 1 << 4, 1 << 5)));
        let imu_bus = RefCell::new(i2c_static(Bus::new(Port::B, 1 << 2, 1 << 3)));
        Some(Self {
            module,
            base_bus,
            imu_bus,
            imu_powered: false,
        })
    }

    /// A busy-wait delay driven by SysTick's calibrated clock.
    ///
    /// The facade exposes it because every example needs one and nobody should be writing
    /// `cortex_m::asm::delay(n)`: that is not *n* cycles. It inflated a buzzer calibration
    /// by 3x before anyone measured it.
    ///
    /// # Why the peripheral is stolen
    ///
    /// The HAL's `Board` owns the core peripherals and its `into_delay` consumes the
    /// board, which the facade cannot do — it owns the board for its whole life. SysTick is
    /// otherwise untouched: the HAL reads `core` only to build a `Delay`, and no example
    /// calls `split`. Stealing the handle therefore creates a second view of registers that
    /// nothing else programs. Two delays alive at once would mean two owners of the same
    /// countdown, so the example takes one and keeps it.
    pub fn delay(&mut self) -> efr32mg24_hal::delay::Delay {
        // SAFETY: SysTick is exclusively the delay's; nothing else in this workspace
        // programs it, which is what makes the second view harmless.
        let syst = unsafe { cortex_m::Peripherals::steal().SYST };
        efr32mg24_hal::delay::Delay::with_sysclk(syst, efr32mg24_hal::clock::DEFAULT_SYSCLK_HZ)
    }

    /// The user LED (PA7, active low).
    pub fn led(&self) -> Output<'_> {
        // PA7, active low: the module's LED, reached through the module crate's wiring so
        // there is one place that says which pin it is.
        self.module.hal().output_on(Port::A, 7, true)
    }

    /// The user button (PC01, pulled up on the base).
    pub fn button(&self) -> Input<'_> {
        // PC01 on the base, shorted to ground, pulled up.
        self.module.hal().input_on(Port::C, 1)
    }

    /// The piezo buzzer on PC03 (TIMER0 CC0), calibrated and silent.
    ///
    /// `None` when the timer never reports a running counter.
    pub fn buzzer(&self) -> Option<efr32mg24_hal::buzzer::Buzzer<'_>> {
        self.module.hal().buzzer_on(Port::C, 3)
    }

    /// The 128×64 OLED at 0x3C.
    ///
    /// `None` when the panel does not answer — a missing display must not stop an
    /// application from doing its job.
    pub fn oled(&self) -> Option<Ssd1306<SharedBus<'_, BitBangI2c<'static>>>> {
        Ssd1306::new(SharedBus::new(&self.base_bus))
    }

    /// The BME68x environment sensor at 0x76 or 0x77.
    pub fn environment(&self) -> Result<Bme68x<SharedBus<'_, BitBangI2c<'static>>>, Bme68xError> {
        Bme68x::new(SharedBus::new(&self.base_bus)).map_err(|_| Bme68xError::NotFound)
    }

    /// The PCF8563 calendar clock at 0x51.
    ///
    /// Always returned: unlike a sensor, a clock that does not answer will simply
    /// report a bus error on the first read, and there is nothing to configure.
    pub fn rtc(&self) -> Pcf8563<SharedBus<'_, BitBangI2c<'static>>> {
        Pcf8563::new(SharedBus::new(&self.base_bus))
    }

    /// The on-board LSM6DS3TR-C, powered and configured for motion.
    ///
    /// Powers the sensor rail on first use: until PD5 is high the part does not
    /// answer at all, so this is the facade's job and not the driver's.
    pub fn imu(&mut self) -> Result<Lsm6ds3trc<SharedBus<'_, BitBangI2c<'static>>>, ImuError> {
        if !self.imu_powered {
            self.module.hal().output_on(Port::D, 5, false).on();
            // The datasheet's power-up time is 10 ms; the bus tolerates waiting far
            // better than a retry loop.
            cortex_m::asm::delay(400_000);
            self.imu_powered = true;
        }
        let config = Config {
            range: AccelRange::G8,
            odr: AccelOdr::Hz416,
        };
        Lsm6ds3trc::with_config(SharedBus::new(&self.imu_bus), config)
    }

    /// The microphones's ADC, powered.
    ///
    /// The mic is Sense-only and analogue: PC8 enables it, PC9 is the input, IADC0
    /// samples it. All of that is board wiring, so it lives here.
    pub fn microphone(
        &mut self,
    ) -> Result<efr32mg24_hal::microphone::Microphone<'_>, efr32mg24_hal::microphone::MicError>
    {
        self.module.hal().output_on(Port::C, 8, false).on();
        self.module.hal().converter().microphone()
    }

    /// Verify the devices the facade claims are present, and report which answered.
    ///
    /// Used by examples as a one-line health check, and by the tests in this crate
    /// to keep the address claims honest.
    pub fn probe(&self) -> Devices {
        let mut bus = self.base_bus.borrow_mut();
        Devices {
            oled: bus.probe(ADDR_OLED),
            rtc: bus.probe(ADDR_RTC),
            environment: bus.probe(ADDR_BME68X_A) || bus.probe(ADDR_BME68X_B),
        }
    }
}

/// The addresses this board uses, in one place.
pub const ADDR_OLED: u8 = 0x3C;
/// PCF8563 calendar.
pub const ADDR_RTC: u8 = 0x51;
/// BME68x with SDO low.
pub const ADDR_BME68X_A: u8 = 0x76;
/// BME68x with SDO high.
pub const ADDR_BME68X_B: u8 = 0x77;

/// Which devices answered a probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Devices {
    /// The OLED answered at 0x3C.
    pub oled: bool,
    /// The RTC answered at 0x51.
    pub rtc: bool,
    /// A BME68x answered at 0x76 or 0x77.
    pub environment: bool,
}

impl Devices {
    /// True when the whole base is present.
    #[must_use]
    pub fn complete(&self) -> bool {
        self.oled && self.rtc && self.environment
    }
}

/// Why the environment sensor could not be created.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bme68xError {
    /// No BME68x answered at either address.
    NotFound,
}
