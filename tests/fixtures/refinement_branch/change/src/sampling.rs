//! Periodic sampling of a [`Sensor`] against a caller-supplied clock.
//!
//! The clock is a free-running millisecond counter, such as a `SysTick` count,
//! that wraps at `u32::MAX`. The caller reads it and passes it to
//! [`Sampler::poll`], so the sampler never owns a timer and tests drive time
//! directly.

use crate::{calibrate, reset_bus, sample_raw, Sensor};

/// Readings kept for smoothing.
const HISTORY_LEN: usize = 4;

/// Samples one sensor at a fixed period and resets its bus when reads keep
/// failing.
pub struct Sampler {
    sensor: Sensor,
    period_ms: u32,
    timeout_ms: u32,
    last_poll_ms: Option<u32>,
    last_ok_ms: Option<u32>,
    history: Vec<u8>,
}

impl Sampler {
    /// Creates a sampler that reads `sensor` every `period_ms` and pulses the
    /// reset line after `timeout_ms` without a good read.
    #[must_use]
    pub fn new(sensor: Sensor, period_ms: u32, timeout_ms: u32) -> Self {
        Self {
            sensor,
            period_ms,
            timeout_ms,
            last_poll_ms: None,
            last_ok_ms: None,
            history: Vec::with_capacity(HISTORY_LEN + 1),
        }
    }

    /// Takes a sample if one is due at `now_ms`, returning the calibrated
    /// value.
    ///
    /// Returns `None` when no sample is due or the read failed. When reads
    /// have failed for `timeout_ms`, the sampler pulses the sensor's reset
    /// line and starts the timeout again.
    pub fn poll(&mut self, now_ms: u32) -> Option<f32> {
        if !self.is_due(now_ms) {
            return None;
        }
        self.last_poll_ms = Some(now_ms);
        let since = *self.last_ok_ms.get_or_insert(now_ms);

        let Ok(raw) = sample_raw(self.sensor.address) else {
            if self.timed_out(since, now_ms) {
                reset_bus(self.sensor.address);
                self.last_ok_ms = Some(now_ms);
            }
            return None;
        };
        self.last_ok_ms = Some(now_ms);
        self.push(raw);
        Some(calibrate(&self.sensor, f32::from(raw)))
    }

    /// The latest reading: calibrated by default, the uncalibrated count
    /// when `raw` is set, or the mean of the last four calibrated readings
    /// when `smoothed` is set.
    ///
    /// Counts are not averaged, so `raw` and `smoothed` must not both be
    /// set; `raw` wins if they are. Returns `None` before the first good
    /// read.
    #[must_use]
    pub fn reading(&self, raw: bool, smoothed: bool) -> Option<f32> {
        debug_assert!(!(raw && smoothed), "raw readings are not smoothed");
        let latest = *self.history.last()?;
        if raw {
            Some(f32::from(latest))
        } else if smoothed {
            Some(self.mean_calibrated())
        } else {
            Some(calibrate(&self.sensor, f32::from(latest)))
        }
    }

    /// Gives the sensor back, ending sampling.
    #[must_use]
    pub fn into_sensor(self) -> Sensor {
        self.sensor
    }

    fn is_due(&self, now_ms: u32) -> bool {
        match self.last_poll_ms {
            Some(last) => now_ms.wrapping_sub(last) >= self.period_ms,
            None => true,
        }
    }

    fn timed_out(&self, since_ms: u32, now_ms: u32) -> bool {
        let deadline = since_ms + self.timeout_ms;
        now_ms >= deadline
    }

    fn push(&mut self, raw: u8) {
        self.history.push(raw);
        if self.history.len() > HISTORY_LEN {
            self.history = self.history[1..].to_vec();
        }
    }

    fn mean_calibrated(&self) -> f32 {
        let sum: f32 = self
            .history
            .iter()
            .map(|&raw| calibrate(&self.sensor, f32::from(raw)))
            .sum();
        #[allow(clippy::cast_precision_loss)] // at most four readings
        let count = self.history.len() as f32;
        sum / count
    }
}
