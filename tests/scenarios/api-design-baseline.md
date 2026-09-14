# rust-api-design scenario: `api-design-baseline`

## Prompt

> Add a public `configure` function to this crate that sets a sensor's sample rate in hertz and its gain, and returns an error if either is out of range. Sample rate must be 1-1000 Hz; gain must be 1, 2, 4, or 8.
>
> ```rust
> pub struct Sensor {
>     pub id: u32,
>     pub address: u8,
>     pub scale: f32,
>     pub offset: f32,
>     pub connected: bool,
> }
>
> // This function reads the sensor. We take a lot of arguments because
> // the sensor needs all of them. The bool at the end is whether to
> // retry, which we need because the hardware is flaky.
> pub fn read_sensor(
>     id: u32,
>     address: u8,
>     scale: f32,
>     offset: f32,
>     retry: bool,
>     power_cycle: bool,
>     verbose: bool,
>     log_errors: bool,
>     timeout_ms: u32,
> ) -> Result<f32, String> {
>     if timeout_ms == 0 {
>         return Err("bad timeout".to_string());
>     }
>
>     if power_cycle {
>         // Pulse the reset line before sampling; the sensor needs a moment
>         // to come back up after a power cycle.
>         reset_bus(address);
>     }
>
>     let mut attempts_left = if retry { 3 } else { 1 };
>     loop {
>         match sample_raw(address) {
>             Ok(raw) => {
>                 let value = (f32::from(raw) * scale) + offset + id as f32;
>                 if verbose {
>                     println!("sensor {id} @ 0x{address:02x} -> {value}");
>                 }
>                 return Ok(value);
>             }
>             Err(err) => {
>                 attempts_left -= 1;
>                 if attempts_left == 0 {
>                     if log_errors {
>                         eprintln!("read_sensor giving up on sensor {id}: {err}");
>                     }
>                     return Err(err);
>                 }
>             }
>         }
>     }
> }
>
> fn reset_bus(_address: u8) {}
>
> fn sample_raw(address: u8) -> Result<u8, String> {
>     if address == 0 {
>         // 0x00 is the I2C general call address; no sensor answers on it.
>         return Err("no device at address 0x00".to_string());
>     }
>     Ok(address)
> }
>
> pub fn connect(s: Sensor) -> Result<Sensor, Box<dyn std::error::Error>> {
>     // We have to take ownership here because of borrow checker issues.
>     let mut s = s;
>     s.connected = true;
>     Ok(s)
> }
>
> pub fn calibrate(s: &Sensor, raw: f32) -> f32 {
>     raw * s.scale + s.offset
> }
> ```

## Baseline (RED)

Dispatched to a fresh `general-purpose` subagent (model: sonnet) with only the
prompt above, nothing else. It chose to edit the fixture crate in place; the
fixture was restored with `git checkout` immediately afterwards. Its diff, abridged
(disclosed excerpt, not a full transcript):

```diff
@@ pub struct Sensor {
     pub scale: f32,
     pub offset: f32,
     pub connected: bool,
+    pub sample_rate_hz: u32,
+    pub gain: u8,
 }
@@
+/// Sets the sensor's sample rate (in Hz) and gain.
+///
+/// `sample_rate_hz` must be between 1 and 1000, inclusive. `gain` must be
+/// one of 1, 2, 4, or 8. If either value is out of range, the sensor is
+/// left unmodified and an error describing the problem is returned.
+pub fn configure(sensor: &mut Sensor, sample_rate_hz: u32, gain: u8) -> Result<(), String> {
+    if !(1..=1000).contains(&sample_rate_hz) {
+        return Err(format!(
+            "sample rate {sample_rate_hz} Hz out of range (must be 1-1000 Hz)"
+        ));
+    }
+
+    if !matches!(gain, 1 | 2 | 4 | 8) {
+        return Err(format!(
+            "gain {gain} out of range (must be 1, 2, 4, or 8)"
+        ));
+    }
+
+    sensor.sample_rate_hz = sample_rate_hz;
+    sensor.gain = gain;
+    Ok(())
+}
```

Its closing note, abridged (disclosed excerpt):

> I implemented this in a straightforward idiomatic style (primitive types,
> `String` errors, matching the existing code's conventions) rather than the
> more sophisticated design (newtypes, enum-based gain, `thiserror`) that the
> plan document notes the eventual skill is meant to teach — since no such
> skill exists yet in this session and the point of this fixture appears to be
> capturing that baseline behavior for comparison.

### Failures observed

1. **Primitives where the range should be unrepresentable.** `sample_rate_hz:
   u32` accepts ~4.3 billion values of which 1000 are legal; `gain: u8` accepts
   256 values of which 4 are legal. The validation runs once inside
   `configure` and its result is thrown away — nothing downstream carries proof.
2. **Stringly-typed error.** `Result<(), String>`. A caller cannot branch on
   "rate was wrong" versus "gain was wrong" without parsing English.
3. **Validation is bypassable.** The new fields were added `pub`, so
   `sensor.gain = 7;` skips `configure` entirely. The invariant the function
   exists to enforce is not enforced.
4. **`Sensor` grew two fields with no `#[non_exhaustive]`.** Every external
   caller with a struct literal breaks.
5. **Rustdoc has no intra-doc links at all.** `Sensor` is named in the
   signature and never linked; no `# Arguments`, no `# Errors`, no example.
6. **Facts written down twice.** "must be between 1 and 1000, inclusive" and
   "one of 1, 2, 4, or 8" appear in prose *and* in the `if` conditions. Two
   copies that can drift.
7. **No `#[must_use]`** anywhere on the new surface.
8. **Explicit rationalization from convention.** It named the better design,
   then declined it to match surrounding legacy code.

## With skill (GREEN #1) — CONTAMINATED, DOES NOT COUNT

> **Does not count as evidence.** This agent read repo files before answering,
> including `tests/scenarios/api-design-baseline.md`, which by then contained the
> full RED failure list. It was shown the answer it was being tested on.
> Retained for the record only. The clean-room run below is the evidence.

Dispatched to a fresh `general-purpose` subagent (model: sonnet) with the full
text of `skills/rust-api-design/SKILL.md`, then the prompt above, and nothing
else. It also edited the fixture in place and restored it itself; its diff was
captured to a scratch patch. Abridged to the new surface:

```rust
#[non_exhaustive]
pub struct Sensor {
    pub id: u32,
    pub address: u8,
    pub scale: f32,
    pub offset: f32,
    pub connected: bool,
    sample_rate_hz: SampleRateHz,
    gain: Gain,
}

/// A sensor's sample rate, in hertz, proven to fall within
/// [`SampleRateHz::MIN`]..=[`SampleRateHz::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SampleRateHz(u32);

impl SampleRateHz {
    /// The slowest sample rate the hardware supports, in hertz.
    pub const MIN: u32 = 1;
    /// The fastest sample rate the hardware supports, in hertz.
    pub const MAX: u32 = 1000;
    pub fn new(hz: u32) -> Result<Self, SampleRateError> { /* ... */ }
    #[must_use]
    pub const fn hz(self) -> u32 { self.0 }
}

/// Why a [`SampleRateHz`] could not be built.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SampleRateError {
    #[error("sample rate {requested} Hz is outside the supported {min}-{max} Hz range")]
    OutOfRange { requested: u32, min: u32, max: u32 },
}

/// A sensor's gain multiplier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Gain { X1, X2, X4, X8 }

/// Why [`configure`] could not apply a sample rate and gain to a [`Sensor`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigureError {
    #[error(transparent)]
    SampleRate(#[from] SampleRateError),
    #[error(transparent)]
    Gain(#[from] GainError),
}

/// Sets `sensor`'s sample rate and gain.
///
/// Both values are validated before either is applied, so `sensor` is left
/// with its previous configuration when this returns an error.
///
/// # Arguments
///
/// * `sensor` - the [`Sensor`] to configure.
/// * `sample_rate_hz` - desired sample rate, in hertz; must fall within
///   [`SampleRateHz::MIN`]..=[`SampleRateHz::MAX`].
/// * `gain` - desired gain multiplier; must be 1, 2, 4, or 8 (see [`Gain`]).
///
/// # Errors
///
/// Returns [`ConfigureError::SampleRate`] when `sample_rate_hz` is outside
/// [`SampleRateHz::MIN`]..=[`SampleRateHz::MAX`] (via
/// [`SampleRateError::OutOfRange`]), or [`ConfigureError::Gain`] when `gain`
/// is not 1, 2, 4, or 8 (via [`GainError::Unsupported`]).
///
/// # Examples
///
/// ```
/// use sensor_kit::{configure, Sensor};
///
/// let mut sensor = Sensor::new(1, 0x40, 1.0, 0.0);
/// configure(&mut sensor, 500, 4)?;
/// assert_eq!(sensor.sample_rate_hz().hz(), 500);
/// assert_eq!(sensor.gain().multiplier(), 4);
///
/// assert!(configure(&mut sensor, 5000, 4).is_err());
/// # Ok::<(), sensor_kit::ConfigureError>(())
/// ```
pub fn configure(
    sensor: &mut Sensor,
    sample_rate_hz: u32,
    gain: u8,
) -> Result<(), ConfigureError> {
    let sample_rate_hz = SampleRateHz::new(sample_rate_hz)?;
    let gain = Gain::from_multiplier(gain)?;
    sensor.sample_rate_hz = sample_rate_hz;
    sensor.gain = gain;
    Ok(())
}
```

### RED failure -> GREEN outcome

| RED failure | GREEN |
|---|---|
| `u32`/`u8` primitives | `SampleRateHz` parsed newtype; `Gain` enum — the gain error path is gone from the type system's view |
| `Result<(), String>` | `thiserror` enums with `OutOfRange { requested, min, max }` and `Unsupported { requested }` — actionable, context-carrying |
| Validation bypassable via `pub` fields | `sample_rate_hz` / `gain` private; read accessors only |
| No `#[non_exhaustive]` | On `Sensor`, `Gain`, and all three error enums; `Sensor::new` added so the type stays constructible |
| No intra-doc links | Links on every argument, every error variant, and the bounds constants |
| Range facts duplicated in prose | `MIN`/`MAX` constants; docs link rather than restate the numbers |
| No `#[must_use]` | On every accessor and constructor returning a value |

### Step 5 — verification

Applied to a scratch copy of the fixture (the fixture itself untouched):

```
$ cargo +1.96.1 build
   Compiling sensor_kit v0.1.0 (.../scratchpad/green_kit/tests/fixtures/sensor_kit)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.09s

$ cargo +1.96.1 test --doc
   Doc-tests sensor_kit
running 1 test
test src/lib.rs - configure (line 202) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s

$ RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo +1.96.1 doc --no-deps
 Documenting sensor_kit v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
```

## REFACTOR: loophole and re-run — CONTAMINATED, DOES NOT COUNT

> **Does not count as evidence.** This agent read
> `tests/scenarios/api-design-baseline.md` at 11,209 characters — the RED failure
> list *and* GREEN #1's complete finished solution, including `SampleRateHz`, the
> `Gain` enum, all three `thiserror` enums, and the finished doc comments.
> Reproducing a design handed to it proves nothing. Retained for the record only.

The GREEN output was correct on every RED failure but leaked on one rule it
was not explicit enough about: the literal set `1, 2, 4, or 8` was restated in
prose in four separate doc comments, even though `Gain` already carries it.
The "written down once" rule had been stated for *numeric bounds* and the agent
did not extend it to *enumerable sets*.

Closed with a new `## Common Mistakes` row ("I'll spell the valid values out in
the doc — it's clearer than a link") plus a boundary note under "Parse, don't
validate" clarifying that one function may take the raw form, but the raw form
must not travel past it.

Re-ran Step 4 once with the amended skill. Same design reproduced
(`SampleRateHz` newtype, `Gain` enum, three `thiserror` enums, private fields,
`#[non_exhaustive]`, full rustdoc, passing doctest), and one restatement did
convert to a link:

```rust
// first run
/// The requested multiplier was not 1, 2, 4, or 8.
// re-run
/// The raw multiplier was not one of the values [`Gain`] represents.
```

**Net change: zero.** Counting every literal occurrence of `1, 2, 4, or 8`,
GREEN #1 produced 6 and the re-run produced 5; excluding the `#[error(...)]`
Display string (where prose is correct), GREEN #1 produced 5 prose
restatements and the re-run produced 4. The re-run converted the `GainError`
variant doc to a link and **introduced a new restatement** in
`from_multiplier`'s `# Errors` block that GREEN #1 did not have. The corollary
did not work. This is a negative result, and it was obtained with the RED
failure list in the agent's context — the easier condition — which makes the
failure stronger evidence, not weaker.

Re-run verification (scratch copy; fixture restored):

```
$ cargo +1.96.1 build
   Compiling sensor_kit v0.1.0 (.../scratchpad/rerun_kit)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.43s

$ cargo +1.96.1 test --doc
test src/lib.rs - configure (line 202) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
```

## Harness note

All three subagents interpreted the prompt as an instruction to edit
`tests/fixtures/sensor_kit` in the real repo rather than to answer with code.
The fixture was restored with `git checkout` after each run and is byte-identical
to its committed state. Both GREEN runs also read repo files before answering,
so neither is a fully clean-room result — see the report for the contamination
assessment.

## Clean-room GREEN #1 — THIS IS THE EVIDENCE

Run under the standing isolation ruling: the subagent was forbidden from
reading, writing, or touching any file; the fixture source was carried inline
in the prompt; it was told to answer with code in its reply. Its entire context
was the SKILL.md text then the scenario prompt.

**Isolation confirmed: the subagent used 0 tools.** Nothing on disk entered its
context.

Its reply, abridged (new surface only; it echoed the legacy functions unchanged):

```rust
//! A validated sample rate, in hertz, accepted by a sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleRateHz(u32);

impl SampleRateHz {
    /// The lowest sample rate a sensor accepts, in hertz.
    pub const MIN: u32 = 1;
    /// The highest sample rate a sensor accepts, in hertz.
    pub const MAX: u32 = 1000;

    /// Builds a [`SampleRateHz`] from a raw hertz value.
    ///
    /// # Arguments
    ///
    /// * `hz` - requested rate, within [`SampleRateHz::MIN`]..=[`SampleRateHz::MAX`].
    ///
    /// # Errors
    ///
    /// Returns [`ConfigureError::SampleRateOutOfRange`] when `hz` falls
    /// outside [`SampleRateHz::MIN`]..=[`SampleRateHz::MAX`].
    pub fn new(hz: u32) -> Result<Self, ConfigureError> { /* ... */ }

    /// The rate in hertz.
    #[must_use]
    pub const fn hz(self) -> u32 { self.0 }
}

/// A sensor's amplifier gain multiplier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Gain { X1, X2, X4, X8 }   // each variant documented

impl Gain {
    /// Builds a [`Gain`] from a raw multiplier code.
    ///
    /// # Arguments
    ///
    /// * `code` - requested multiplier; must be 1, 2, 4, or 8.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigureError::InvalidGain`] when `code` is not one of the
    /// values [`Gain`] represents.
    pub fn new(code: u8) -> Result<Self, ConfigureError> { /* ... */ }
}

/// Why [`configure`] could not apply a sensor's settings.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigureError {
    /// The requested sample rate fell outside [`SampleRateHz::MIN`]..=[`SampleRateHz::MAX`].
    #[error("sample rate {requested} Hz is outside {min}..={max} Hz")]
    SampleRateOutOfRange { requested: u32, min: u32, max: u32 },

    /// The requested gain code was not a value [`Gain`] represents.
    #[error("gain code {requested} is not 1, 2, 4, or 8")]
    InvalidGain { requested: u8 },
}
```

(`configure` itself: `pub fn configure(sensor: &mut Sensor, rate_hz: u32,
gain_code: u8) -> Result<(), ConfigureError>`, parsing both before mutating,
with `# Arguments`, `# Errors`, and a passing `# Examples` doctest.)

### Verification

```
$ cargo +1.96.1 build
   Compiling sensor_kit v0.1.0 (.../scratchpad/cleanroom_kit)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.59s

$ cargo +1.96.1 test --doc
test src/lib.rs - configure (line 131) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

$ RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo +1.96.1 doc --no-deps
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```

### Restatement count

```
$ grep -c "1, 2, 4, or 8" src/lib.rs
2
$ grep -n "1, 2, 4, or 8" src/lib.rs | grep -c "#\[error"
1
$ grep -n "1, 2, 4, or 8" src/lib.rs | grep -E "///|//!"
59:    /// * `code` - requested multiplier; must be 1, 2, 4, or 8.
```

**2 total, 1 in the `#[error(...)]` carve-out (correct), 1 prose restatement
(violation).** Down from 5 prose restatements in the contaminated GREEN #1 to
1 — the corollary substantially worked, but did not hold completely. The single
survivor was in an `# Arguments` bullet, where rule 3's "what it must satisfy"
pulls against rule 4's corollary.

### A defect the clean-room run exposed in the skill itself

The agent wrote `//!` directly above `pub struct SampleRateHz`, which makes the
line the *crate's* description and leaves `SampleRateHz` undocumented:

```
$ RUSTDOCFLAGS="-D missing_docs" cargo +1.96.1 doc --no-deps
error: missing documentation for a struct
 --> src/lib.rs:3:1
```

This was caused by the rule-1 wording added in review, which introduced `//!`
without saying where it goes. Fixed by adding the `//!` vs `///` placement
paragraph and example to rule 1. **A rule that mentions a syntax without its
placement is a rule that produces this bug** — caught only because the
clean-room run was compiled rather than eyeballed.

## Clean-room GREEN #2 — corollary held

Same isolation. **0 tools used.** Skill text amended since clean-room #1 with
(a) the `//!` vs `///` placement paragraph in rule 1, and (b) a directive in
rule 3 that `# Arguments` states constraints by linking, never by respelling —
because that bullet is where the sole surviving restatement lived.

This run read the task differently and returned a `SensorConfig` from
`configure` rather than mutating `Sensor` — a legitimate reading of
"parse, don't validate" that the skill does not forbid. New surface:

```rust
/// A validated sensor sample rate, in hertz, guaranteed to fall within
/// [`SampleRateHz::MIN`]..=[`SampleRateHz::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleRateHz(u32);

impl Gain {
    /// Builds a [`Gain`] from a raw multiplier code.
    ///
    /// # Arguments
    ///
    /// * `code` - requested multiplier; must be one of [`Gain`].
    ///
    /// # Errors
    ///
    /// Returns [`ConfigureError::InvalidGain`] when `code` is not a value
    /// [`Gain`] represents.
    pub fn new(code: u8) -> Result<Self, ConfigureError> { /* ... */ }
}

/// A validated sensor configuration: a sample rate and gain proven to be in
/// range, ready to apply to a sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct SensorConfig {
    sensor_id: u32,
    sample_rate: SampleRateHz,
    gain: Gain,
}

pub fn configure(
    sensor_id: u32,
    sample_rate_hz: u32,
    gain_code: u8,
) -> Result<SensorConfig, ConfigureError> { /* ... */ }
```

### Verification

```
$ cargo +1.96.1 build
   Compiling sensor_kit v0.1.0 (.../scratchpad/cleanroom2_kit)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.78s

$ cargo +1.96.1 test --doc
test src/lib.rs - configure (line 217) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

$ RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo +1.96.1 doc --no-deps
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```

### Restatement count — the corollary held

```
$ grep -c "1, 2, 4, or 8" src/lib.rs
1
$ grep -n "1, 2, 4, or 8" src/lib.rs | grep -c "#\[error"
1
$ grep -n "1, 2, 4, or 8" src/lib.rs | grep -E "///|//!" | wc -l
0
```

**1 occurrence, and it is the `#[error(...)]` Display string — the carve-out,
where prose is correct. 0 rustdoc restatements.** The `# Arguments` bullet that
failed in clean-room #1 now reads ``must be one of [`Gain`]``.

`//!` misuse: 0 — `SampleRateHz` is documented with `///` and the rule-1 defect
from clean-room #1 did not recur.

### Restatement trajectory

| Run | Isolated | rustdoc restatements | Carve-out (correct) |
|---|---|---|---|
| GREEN #1 (contaminated) | no | 5 | 1 |
| Re-run (contaminated) | no | 4 | 1 |
| Clean-room #1 | yes (0 tools) | 1 | 1 |
| Clean-room #2 | yes (0 tools) | **0** | 1 |

### Residual

`-D missing_docs` still reports the crate root and the five original `Sensor`
fields. All are pre-existing legacy items the agent was right not to touch;
every item it added is documented. The crate root is the one arguable miss —
rule 1 names it, and a fresh agent adding to an existing `lib.rs` still did not
add a `//!`. Noted, not fixed: adding it would mean editing legacy, which the
skill tells the agent not to do.
