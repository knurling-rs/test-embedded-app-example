#![no_std]

use core::time::Duration;

use cortex_m::peripheral::{DCB, DWT};
use defmt::unwrap;
pub use nrf52840_hal::pac;
use nrf52840_hal::{
    gpio::{p0, Level},
    pac::{TWIM0, UARTE0},
    twim,
    uarte::{self, Baudrate, Parity},
    Twim, Uarte,
};
pub use scd30::SensorData;

const CYCCNT_FREQUENCY_MHZ: u32 = 64;

pub type Scd30 = scd30::Scd30<Twim<TWIM0>>;
pub type Serial = Uarte<UARTE0>;

/// Peripherals and on-board sensors
pub struct Board {
    pub scd30: Scd30,
    pub serial: Serial,
}

impl Board {
    /// Initializes the board's peripherals and sensors
    pub fn init(mut dcb: DCB, mut dwt: DWT) -> Self {
        dcb.enable_trace();
        unsafe { dwt.cyccnt.write(0) }
        dwt.enable_cycle_counter();
        defmt::timestamp!(
            "{=u32:µs}",
            cortex_m::peripheral::DWT::get_cycle_count() / CYCCNT_FREQUENCY_MHZ
        );

        let dev_periph = unwrap!(nrf52840_hal::pac::Peripherals::take());
        let p0 = p0::Parts::new(dev_periph.P0);

        let scl = p0.p0_30.into_floating_input().degrade();
        let sda = p0.p0_31.into_floating_input().degrade();
        let pins = twim::Pins { scl, sda };
        let twim = Twim::new(dev_periph.TWIM0, pins, twim::Frequency::K100);

        // TXD = 06
        // RXD = 08
        let txd = p0.p0_06.into_push_pull_output(Level::Low).degrade();
        let rxd = p0.p0_08.into_floating_input().degrade();
        let pins = uarte::Pins {
            txd,
            rxd,
            cts: None,
            rts: None,
        };

        let uarte = Uarte::new(
            dev_periph.UARTE0,
            pins,
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );

        Self {
            scd30: Scd30::init(twim),
            serial: uarte,
        }
    }

    pub fn delay(&self, dur: Duration) {
        let start = Instant::now();
        while start.elapsed() < dur {}
    }
}

#[derive(Clone, Copy)]
pub struct Instant(u32);

impl Instant {
    pub fn now() -> Self {
        Instant(cortex_m::peripheral::DWT::get_cycle_count())
    }

    pub fn elapsed(&self) -> Duration {
        let now = Instant::now();
        let elapsed = now.0.wrapping_sub(self.0);
        let micros = elapsed / CYCCNT_FREQUENCY_MHZ;
        Duration::from_micros(micros as u64)
    }

    pub fn as_cycles(self) -> u32 {
        self.0
    }
}
