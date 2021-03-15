use std::{thread, time::Duration};

use anyhow::anyhow;
use messages::{Host2Target, Measurement, Target2Host};
use parking_lot::{Mutex, MutexGuard};
use serialport::SerialPort;

const BAUD_RATE: u32 = 115_200;

#[test]
fn get_measurement_succeeds() -> Result<(), anyhow::Error> {
    let mut target = TargetSerialConn::open()?;
    dbg!(target.get_measurement()?);
    Ok(())
}

#[test]
fn new_measurement_every_2_seconds() -> Result<(), anyhow::Error> {
    let mut target = TargetSerialConn::open()?;

    let samples = (0..3).map(|_| {
        thread::sleep(Duration::from_millis(2_100));
        Ok(dbg!(target.get_measurement()?.unwrap()))
    }).collect::<Result<Vec<_>, anyhow::Error>>()?;

    for pair in samples.windows(2) {
        // all samples should be different
        assert_ne!(pair[0], pair[1]);
        // new measurements should have contiguous IDs
        assert_eq!(pair[0].id.wrapping_add(1), pair[1].id);
        // timestamps should be different
        // (the timestamp can wrap around so we don't use `>=`)
        assert_ne!(pair[0].timestamp, pair[1].timestamp);
    }

    Ok(())
}

#[test]
fn consecutive_sampling_returns_same_measurement() -> Result<(), anyhow::Error> {
    let mut target = TargetSerialConn::open()?;

    // sample faster than the SCD30 update rate
    let first = dbg!(target.get_measurement()?);
    let second = dbg!(target.get_measurement()?);
    let third = dbg!(target.get_measurement()?);

    // at most one new measurement can occur; 2 samples should be the same measurement
    if first != second {
        assert_eq!(second, third);
    }

    if second != third {
        assert_eq!(first, second);
    }

    Ok(())
}

/// A connection between the host and the target over a serial interface
pub struct TargetSerialConn {
    port: Box<dyn SerialPort>,
    rx_bytes: Vec<u8>,
    _guard: MutexGuard<'static, ()>,
}

impl TargetSerialConn {
    /// Opens a serial connection to the target
    // NOTE this operation does NOT use a lock file so a different process is free to operate on
    // the serial port (e.g. `[sudo] cat /dev/ttyACM0`). That can make the rest of this
    // API misbehave.
    pub fn open() -> Result<Self, anyhow::Error> {
        const VID: u16 = 0x1366;
        const PID: u16 = 0x1015;

        static MUTEX: Mutex<()> = parking_lot::const_mutex(());

        let _guard = MUTEX.lock();

        let ports = serialport::available_ports()?;
        for port in ports {
            if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
                if info.vid == VID && info.pid == PID {
                    let port = serialport::new(port.port_name, BAUD_RATE)
                        .timeout(Duration::from_millis(100))
                        .open()?;
                    return Ok(Self {
                        port,
                        rx_bytes: vec![],
                        _guard,
                    });
                }
            }
        }

        Err(anyhow!("device {:04x}:{:04x} is not connected", VID, PID))
    }

    /// Requests the last measurement
    pub fn get_measurement(&mut self) -> Result<Option<Measurement>, anyhow::Error> {
        let resp = self.request(&Host2Target::GetLastMeasurement)?;

        Ok(match resp {
            Target2Host::NotReady => None,
            Target2Host::Measurement(measurement) => Some(measurement),
        })
    }

    /// Sends a request to the target and waits for a response.
    /// Returns the target response.
    fn request(&mut self, request: &Host2Target) -> Result<Target2Host, anyhow::Error> {
        let tx_bytes =
            postcard::to_allocvec_cobs(&request).map_err(|e| anyhow::Error::msg(e.to_string()))?;

        self.port.write_all(dbg!(&tx_bytes))?;

        let mut buffer = [0; 64];

        let delimiter_pos = loop {
            let bytes_read = self.port.read(&mut buffer)?;

            self.rx_bytes.extend_from_slice(&buffer[..bytes_read]);

            if let Some(pos) = self.rx_bytes.iter().position(|byte| *byte == 0) {
                break pos;
            }
        };

        dbg!(&self.rx_bytes);

        let endpos = delimiter_pos + 1;
        let frame = &mut self.rx_bytes[..dbg!(endpos)];
        let res = postcard::from_bytes_cobs::<Target2Host>(dbg!(frame))
            .map_err(|e| anyhow::Error::msg(e.to_string()));

        // pop frame from RX buffer *before* raising any error
        let len = self.rx_bytes.len();
        self.rx_bytes.rotate_left(endpos);
        self.rx_bytes.truncate(len - endpos);

        dbg!(&self.rx_bytes);

        res
    }
}
