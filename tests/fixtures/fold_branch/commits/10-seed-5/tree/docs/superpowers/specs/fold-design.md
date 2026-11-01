# Sample buffer: design

Date: 2026-10-30
Status: Approved, ready for implementation planning

## Intent

Boards that use `sensor_kit` read a sensor with `read_sensor` and then keep
the last few values in an array of their own, to smooth them or to see
whether the sensor is wandering. Each board writes that array and the
averaging around it again. We want the crate to keep recent readings and
average them, so a board reads through one type.

## Goals

- A board keeps a sensor's recent readings without writing a buffer.
- The mean of all, or of the newest few, readings is one call.
- No allocation after construction; a board sizes the buffer once.

## The buffer

A new module, `sensor_kit::buffer`, holds `SampleBuffer`: a fixed-capacity
queue of `f32` readings, oldest first. `push` adds a reading, `latest` gives
the newest, and `iter` walks them oldest first. The capacity is set at
construction and never grows.

## Statistics

A new module, `sensor_kit::stats`, holds free functions over a
`SampleBuffer`: `mean`, over every reading, and `window_mean`, over the
newest few. Each returns `None` when there is nothing to average, never a
NaN.

## The recorder

A new module, `sensor_kit::recorder`, holds `Recorder`, which owns a
`Sensor` and a `SampleBuffer`. `record` reads the sensor once through
`read_sensor`, with retries, keeps the value on success and returns it. A
failed read keeps nothing and returns the error. `Recorder::new` uses a
default capacity; `with_capacity` takes one.

## Design decisions

- **Adopted (round 1).** A full `SampleBuffer` drops its oldest reading to
  take a new one, and `push` never refuses; a buffer of capacity 0 keeps
  nothing. Retires: a `push` that returns an error every caller in a main
  loop would ignore. Revised in round 2.
- **Adopted (round 2).** `window_mean` takes its window as a `SampleCount`,
  a count that is never zero, rather than a `usize`. Retires: the run-time
  rejection of a window of 0, and the `None` a caller could not tell from a
  buffer that was too short.

## Left to the builder

- The buffer's storage.
- The default capacity.
