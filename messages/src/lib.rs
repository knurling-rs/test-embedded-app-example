#![cfg_attr(not(test), no_std)]

use serde_derive::{Deserialize, Serialize};

/// A message sent from the host to the target
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Host2Target {
    GetLastMeasurement,
}

/// A message sent from the target to the host
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Target2Host {
    NotReady,
    Measurement(Measurement),
}

/// A measurement reported by the target
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Measurement {
    /// The measurement identifier; this is a monotonically increasing counter
    pub id: u32,
    /// A timestamp in unspecified units; it may wrap around
    pub timestamp: u32,
    /// The CO2 concentration in parts per million (ppm)
    pub co2: f32,
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use super::{Host2Target, Measurement, Target2Host};

    /// Max payload size for a USB (2.0 Full Size) HID packet
    const MAX_SIZE: usize = 64;

    #[test]
    fn host2target_message_size() -> postcard::Result<()> {
        let msg = Host2Target::GetLastMeasurement;
        let bytes = postcard::to_allocvec(&msg)?;
        assert!(dbg!(bytes).len() <= MAX_SIZE);
        Ok(())
    }

    #[test]
    fn target2host_not_ready_message_size() -> postcard::Result<()> {
        let msg = Target2Host::NotReady;
        let bytes = postcard::to_allocvec(&msg)?;
        assert!(dbg!(bytes).len() <= MAX_SIZE);
        Ok(())
    }

    #[quickcheck]
    fn target2host_measurement_message_size(
        id: u32,
        timestamp: u32,
        co2: f32,
    ) -> postcard::Result<()> {
        let msg = Target2Host::Measurement(Measurement { id, timestamp, co2 });
        let bytes = postcard::to_allocvec(&msg)?;
        assert!(bytes.len() <= MAX_SIZE);
        Ok(())
    }
}
