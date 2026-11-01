# Periodic sampling: design

Date: 2026-10-28
Status: Approved, ready for implementation planning

## Intent

Firmware that uses `sensor_kit` today calls `read_sensor` from its own main
loop and keeps its own timer, retry count and reset logic around it. Every
board does this slightly differently. We want one sampler in the crate that
reads a sensor at a fixed period, keeps the last few readings for smoothing,
and resets the bus when a sensor stops answering.

## Goals

- A board's main loop makes one call per iteration and gets a value when one
  is due.
- No timer or thread inside the sampler; the caller owns the clock.
- A sensor that stops answering is reset without the caller noticing, and
  the caller can tell a timeout from an ordinary failed read.

## The sampler

A new module, `sensor_kit::sampling`, holds one type.

```rust
pub struct Sampler { /* private */ }

impl Sampler {
    pub fn new(sensor: Sensor, period_ms: u32, timeout_ms: u32) -> Sampler;
    pub fn poll(&mut self, now_ms: u32) -> Result<Option<f32>, SampleError>;
    pub fn reading(&self, raw: bool, smoothed: bool) -> Option<f32>;
    pub fn into_sensor(self) -> Sensor;
}

pub enum SampleError {
    TimedOut,
    Read(String),
}
```

The clock is the board's free-running millisecond counter, which wraps at
`u32::MAX` after about 49 days; a board that has been up for months polls
across the wrap like any other moment. `poll` takes a sample when at least
`period_ms` has passed since the last poll that sampled, and the first poll
always samples. It returns `Ok(Some(value))` with the calibrated value,
`Ok(None)` when no sample is due, and `Err(SampleError::Read(msg))` when the
read failed. When reads have failed for `timeout_ms` since the last good read
(or since the first poll), it pulses the sensor's reset line, starts the
timeout again, and returns `Err(SampleError::TimedOut)` instead.

The sampler keeps the last four good readings. `reading` returns the latest
one calibrated, the raw count when `raw` is set, or the mean of the last four
calibrated readings when `smoothed` is set. Raw counts are not averaged, so
passing both flags is a caller error. It returns `None` before the first good
read.

## Design decisions

- **Adopted (round 1).** `Sampler::new` takes the `Sensor` by value and the
  sampler owns it until `into_sensor` gives it back, never a `&Sensor`
  borrow. Retires: a lifetime parameter on `Sampler` that every board's
  long-lived task struct would carry, and the borrow that would keep the
  board from storing the sampler beside the sensor's other state.
- **Adopted (round 1).** `Sampler::poll` returns `Result<Option<f32>,
  SampleError>`: `Ok(None)` when no sample is due, `Err(SampleError::Read)`
  for a failed read and `Err(SampleError::TimedOut)` when the reset fires,
  never a bare `Option<f32>`. Retires: the caller guessing whether a missing
  value was a timeout, an ordinary failed read, or nothing due.
