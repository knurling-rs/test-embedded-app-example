#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[defmt_test::tests]
mod tests {
    use core::time::Duration;

    use board::Board;
    use defmt::{assert_eq, unwrap};

    #[init]
    fn init() -> Board {
        let cm_periph = unwrap!(cortex_m::Peripherals::take());
        Board::init(cm_periph.DCB, cm_periph.DWT)
    }

    #[test]
    fn confirm_firmware_version(board: &mut Board) {
        const EXPECTED: [u8; 2] = [3, 66];

        assert_eq!(EXPECTED, board.scd30.get_firmware_version().unwrap())
    }

    #[test]
    fn data_ready_within_two_seconds(board: &mut Board) {
        board
            .scd30
            .start_continuous_measurement()
            .unwrap();

        // do this twice because there may be a cached measurement 
        // (the sensor is never power-cycled / reset)
        for _ in 0..2 {
            board.delay(Duration::from_millis(2_100));
            assert!(board.scd30.data_ready().unwrap());

            // clear data ready flag
            let _ = board.scd30.read_measurement();
        }
    }

    #[test]
    fn reasonable_co2_value(board: &mut Board) {
        // range reported by the sensor when using I2C
        const MIN_CO2: f32 = 0.;
        const MAX_CO2: f32 = 40_000.;

        // do this twice for good measure
        for _ in 0..2 {
            while !board.scd30.data_ready().unwrap() {}

            let measurement = board.scd30.read_measurement().unwrap();
            assert!(measurement.co2.is_finite());
            assert!(measurement.co2 >= MIN_CO2);
            assert!(measurement.co2 <= MAX_CO2);
        }
    }
}
