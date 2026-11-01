use crate::buffer::SampleBuffer;
use crate::{read_sensor, Sensor};

/// How many readings a [`Recorder`] keeps unless told otherwise.
pub const DEFAULT_CAPACITY: usize = 8;

/// Reads one sensor on demand and keeps its recent readings.
pub struct Recorder {
    sensor: Sensor,
    buffer: SampleBuffer,
}

impl Recorder {
    /// A recorder that keeps the last [`DEFAULT_CAPACITY`] readings.
    pub fn new(sensor: Sensor) -> Self {
        Self::with_capacity(sensor, DEFAULT_CAPACITY)
    }

    /// A recorder that keeps the last `capacity` readings.
    pub fn with_capacity(sensor: Sensor, capacity: usize) -> Self {
        Self {
            sensor,
            buffer: SampleBuffer::new(capacity),
        }
    }

    /// Reads the sensor once, with retries, and keeps the value. A failed
    /// read keeps nothing.
    pub fn record(&mut self) -> Result<f32, String> {
        let s = &self.sensor;
        let value = read_sensor(
            s.id, s.address, s.scale, s.offset, true, false, false, false, 100,
        )?;
        self.buffer.push(value);
        Ok(value)
    }

    /// The readings kept so far.
    pub fn buffer(&self) -> &SampleBuffer {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sensor(address: u8) -> Sensor {
        Sensor {
            id: 0,
            address,
            scale: 1.0,
            offset: 0.0,
            connected: true,
        }
    }

    #[test]
    fn records_into_the_buffer() {
        let mut recorder = Recorder::new(sensor(4));
        assert_eq!(recorder.record(), Ok(4.0));
        assert_eq!(recorder.buffer().len(), 1);
    }

    #[test]
    fn a_failed_read_keeps_nothing() {
        let mut recorder = Recorder::new(sensor(0));
        assert!(recorder.record().is_err());
        assert!(recorder.buffer().is_empty());
    }
}
