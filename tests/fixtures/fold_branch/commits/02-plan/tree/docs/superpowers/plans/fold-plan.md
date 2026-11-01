# Sample Buffer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep a sensor's recent readings in the crate and average them, so a board reads through one type.

**Architecture:** Three new modules in `sensor_kit`: `buffer` (the fixed-capacity queue), `stats` (free functions over it), and `recorder` (a `Sensor` plus a buffer, read through `read_sensor`).

**Tech Stack:** Rust 2021, std only, no new dependencies.

**Spec:** `docs/superpowers/specs/fold-design.md`.

## Global Constraints

- No new dependencies.
- Existing items in `src/lib.rs` are not changed; each task only adds a `pub mod` line there.
- Every task ends green: `cargo fmt --check`, `cargo build --all-targets`, `cargo test`.

---

### Task 1: SampleBuffer

**Files:**
- Create: `src/buffer.rs`
- Modify: `src/lib.rs` (add `pub mod buffer;`)

**Interfaces:**
- Produces: `SampleBuffer::new(capacity: usize) -> SampleBuffer`, `push(&mut self, value: f32)`, `latest(&self) -> Option<&f32>`, `len`, `is_empty`, `iter(&self) -> impl Iterator<Item = f32> + '_` (oldest first).

- [ ] **Step 1: Write the failing tests** `latest_is_the_newest_push` and `a_full_buffer_drops_the_oldest` in `src/buffer.rs`.
- [ ] **Step 2: Run `cargo test buffer`.** Expected: FAIL, `SampleBuffer` not found.
- [ ] **Step 3: Implement** `SampleBuffer` over a `VecDeque<f32>` created with the capacity; a full buffer pops the front before pushing.
- [ ] **Step 4: Run `cargo test buffer`.** Expected: PASS.
- [ ] **Step 5: Commit** `Add SampleBuffer`.

### Task 2: Buffer statistics

**Files:**
- Create: `src/stats.rs`
- Modify: `src/lib.rs` (add `pub mod stats;`)

**Interfaces:**
- Consumes: `SampleBuffer::{iter, len, is_empty}` from Task 1.
- Produces: `stats::mean(buffer: &SampleBuffer) -> Option<f32>`, `stats::window_mean(buffer: &SampleBuffer, window: usize) -> Option<f32>`; `window_mean` is `None` when `window` is 0 or the buffer holds fewer than `window` readings.

- [ ] **Step 1: Write the failing tests** `mean_of_an_empty_buffer_is_none`, `mean_averages_every_reading`, `window_mean_averages_the_newest` and `window_mean_needs_enough_readings`.
- [ ] **Step 2: Run `cargo test stats`.** Expected: FAIL, `stats` not found.
- [ ] **Step 3: Implement** both functions.
- [ ] **Step 4: Run `cargo test stats`.** Expected: PASS.
- [ ] **Step 5: Commit** `Add buffer statistics`.

### Task 3: Recorder

**Files:**
- Create: `src/recorder.rs`
- Modify: `src/lib.rs` (add `pub mod recorder;`)

**Interfaces:**
- Consumes: `SampleBuffer::{new, push}` from Task 1; `read_sensor` and `Sensor` from `src/lib.rs`.
- Produces: `recorder::DEFAULT_CAPACITY: usize` (8), `Recorder::new(sensor: Sensor)`, `Recorder::with_capacity(sensor: Sensor, capacity: usize)`, `record(&mut self) -> Result<f32, String>`, `buffer(&self) -> &SampleBuffer`.

- [ ] **Step 1: Write the failing tests** `records_into_the_buffer` and `a_failed_read_keeps_nothing` (a `Sensor` at address 0 never answers).
- [ ] **Step 2: Run `cargo test recorder`.** Expected: FAIL, `recorder` not found.
- [ ] **Step 3: Implement** `record` as one `read_sensor` call with `retry` set, the other flags clear and a 100 ms timeout, pushing the value on success.
- [ ] **Step 4: Run `cargo test`.** Expected: PASS.
- [ ] **Step 5: Commit** `Add Recorder`.
