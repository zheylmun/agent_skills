# Test fixtures

This directory contains crates used to test the Rust skills in this plugin.

**These crates are deliberately bad.** `sensor_kit` in particular is seeded
with real design flaws on purpose, so that later tasks can point a skill at
it and confirm the skill actually catches the problems it claims to catch.

Do not "fix" anything in a fixture crate. Do not add documentation, silence
warnings, or otherwise improve it. Making a fixture better breaks the tests
that depend on it staying broken. If a fixture needs to change, it's because
a flaw isn't being detected as intended — not because the code offends you.

## `sensor_kit`

A standalone crate (not part of any workspace) at `tests/fixtures/sensor_kit`
that compiles cleanly with plain `cargo build` but is riddled with API and
style problems that `cargo clippy --all-targets -- -W clippy::pedantic` and a
careful reviewer should both flag.

Seeded flaws, in order of what they test, each paired with the skill that
should catch it:

1. `read_sensor` — nine arguments, four `bool` parameters, primitive
   obsession on `address`/`id`, stringly-typed error.
   Caught by: **rust-api-design**.
2. The block comment — an explanatory `why` comment standing in for a design
   fix.
   Caught by: **rust-principles**.
3. `connect` — `Box<dyn Error>` in a public signature, takes ownership
   needlessly, and a comment blaming the borrow checker.
   Caught by: **rust-api-design**.
4. Every public item is undocumented — no `///` anywhere, no
   `#![warn(missing_docs)]`.
   Caught by: **rust-api-design** (the doc contract) and mechanically by
   **rust-quality-gate** (missing-docs lint).
5. `Sensor` — all fields `pub`, no `#[non_exhaustive]`, no invariants.
   Caught by: **rust-api-design**.
6. `calibrate` — no `#[must_use]` on a pure function.
   Caught by: **rust-api-design**.
7. Features `std` and `logging` are declared but unused, so the feature
   powerset is vacuous.
   Caught by: **rust-quality-gate** (feature-matrix check).

### Adjustment made to the fixture

Flaw 1 as originally specified (six arguments, one trailing `bool`) does not
trip `clippy::pedantic`'s `too_many_arguments` (default threshold: more than
7 arguments) or `fn_params_excessive_bools` (default threshold: more than 3
`bool` parameters) — both lints that a later task depends on. Without
changing what the flaw *is* (too many primitive/bool parameters standing in
for a proper request type), `read_sensor` was given three additional `bool`
parameters (`power_cycle`, `verbose`, `log_errors`, added to the original
`retry`) to push it to 9 arguments and 4 `bool` parameters.

The first pass wired those new bools into the function with arbitrary
two-flag conjunctions (e.g. failing only when both `power_cycle` and
`verbose` were true) purely to keep the parameters "used." Code review
correctly called that out as unrealistic — no real sensor read fails for
that reason, and a synthetic-looking lint trap doesn't pressure-test the
review skills honestly. `read_sensor` was rewritten so each `bool` has its
own plausible, independent job, the way accreted embedded code actually
looks: `retry` bounds a real retry loop around a (deterministic, address-0
special-cased) raw read, `power_cycle` gates a reset-bus call before
sampling, `verbose` gates a diagnostic line on a successful read, and
`log_errors` gates a diagnostic line when retries are exhausted. No bool is
combined with another bool in a condition. This keeps all four `bool`
parameters and all 9 arguments — lint coverage did not regress — while
making the "why do we have this many flags" flaw read as real accreted
code rather than a puzzle box. The stringly-typed `Result<f32, String>`
error was kept unchanged, as required.

This intensifies the seeded flaw rather than changing what it is, and all
five lints (`too_many_arguments`, `fn_params_excessive_bools`,
`must_use_candidate`, `missing_errors_doc`, `cast_precision_loss`) still
fire under `cargo clippy --all-targets --all-features -- -W
clippy::pedantic`.

## `enrolment_spec.md` and `enrolment_contract.md`

A design doc for a device enrolment client, at
`tests/fixtures/enrolment_spec.md`, and the short server contract it answers
to, at `tests/fixtures/enrolment_contract.md`. They are the input to
**rust-spec-review**, which reviews a design before any code exists. These
are markdown fixtures, not crates, and the prohibition above applies to them
identically: do not tidy the prose, resolve the inconsistencies, or move a
sentence to where it reads better. The spec is written as a competent
author's natural prose, and each flaw sits inside a section that otherwise
makes sense; that is the point.

Seeded items, numbered as later tasks refer to them:

1. `Client::new(base: &str)`, and `finalize(&self, key: &DeviceKey,
   pending_id: &str)` beside `Enrolled { pub device_id: String, ... }` in the
   same module, so a pending id and a device id are confusable `String`s.
   Caught by: **rust-spec-review** pass 1.
2. `Record { pending: Option<PendingRecord>, device: Option<DeviceRecord> }`
   with the prose "exactly one of pending or device is present, or neither; a
   file with both prefers device".
   Caught by: **rust-spec-review** pass 2.
3. "A keychain item that is not 32 bytes is `StoreError::Corrupt` and is
   treated as no key."
   Caught by: **rust-spec-review** pass 3.
4. "Every write re-checks that the record still names the id being updated,
   and the mutex is never held across a network or keychain call."
   Caught by: **rust-spec-review** pass 4.
5. "The machine remembers `cancelled: Option<PendingId>`; `Begin` clears it,
   so a new attempt is never hijacked by the old one." This names a
   `PendingId` newtype in prose while item 1 uses `&str`; the inconsistency is
   deliberate and is itself evidence.
   Caught by: **rust-spec-review** pass 5.
6. `trait BrowserOpener { fn open(&self, url: &str) -> io::Result<()>; }`
   with one described implementor that shells out to `open`, no test double
   mentioned, and the sentence "so a future platform can supply its own".
   Caught by: **rust-spec-review** pass 7.
7. `#[non_exhaustive] pub enum FinalizeError { Pending, Denied, Expired,
   Rejected, Unreachable }` with the sentence "non-exhaustive so the server
   can add outcomes without breaking us", where the contract enumerates
   `finalize`'s outcomes and says the set is closed.
   Caught by: **rust-spec-review** pass 8.
8. Under "Decisions": the counter is persisted through a `CounterStore` trait
   (`fn reserve(&mut self) -> io::Result<u64>`) with a file store in the app
   and a memory store in the library's tests, rather than a typestate over
   the request; both stores are real and the trait has two implementors from
   the first commit. This is a stated decision with a reason, and a naive
   reviewer flags it.
   Survives pass 6.
9. Under "Left to the builder": "The exact shape of the tick timer that wakes
   the machine at its next deadline." Nothing in the spec body describes that
   timer, so a reviewer who proposes a timer type is reviewing a builder
   decision.
   Declined as left to the builder.

### `enrolment_spec_reviewed.md` and `enrolment_plan.md`

The input to **rust-spec-review**'s plan mode, under the same prohibition:
do not tidy either one, and do not bring the plan into line with the spec.

`tests/fixtures/enrolment_spec_reviewed.md` is `enrolment_spec.md` as it
would read after the loop, hand-authored so no test depends on a live run.
It departs from `enrolment_spec.md` in exactly these ways: items 1 to 7 above
are rewritten in the body (`PendingId`, `DeviceId` and a `ServerUrl` parsed
once; `Record` an enum of `Pending` or `Device` with a `BothRecorded` load
error and no tie-break; a missing key as `Ok(None)` and `StoreError::Corrupt`
returned, not treated as no key; writes through `Record::update_if(id, ..)`;
a per-start `AttemptId`; a plain function in place of `BrowserOpener`; an
exhaustive `FinalizeError`), and a `## Design decisions` section records them,
item 1 as Adopted with a cost and items 2 to 7 as Adopted, plus one Declined
entry for a typestate over `Client`. Items 8 and 9 are unchanged.

`tests/fixtures/enrolment_plan.md` is a four-task implementation plan against
the reviewed spec. It departs from that spec in exactly three places, and is
faithful everywhere else:

1. Task 2's `DeviceFile` holds `pending: Option<PendingRecord>` and
   `device: Option<DeviceRecord>`, and `load` prefers `device` when both are
   present: the shape and the tie-break the item 2 entry retired.
2. Task 3's `products` takes the device id as a bare `String`, departing from
   the item 1 entry.
3. No task implements `Record::update_if`, the item 4 entry's shape.

Caught by: **rust-spec-review** plan mode, as three items and nothing else.

## `refinement_branch`

The input to **rust-refinement-loop**, which works over a branch rather than
a crate. `tests/fixtures/refinement_branch/setup.sh <dir> [--no-spec]` builds
a fresh git repo at `<dir>` (it refuses a non-empty one; no network). `main`
is one commit holding a copy of `sensor_kit` without its `target/` or
lockfile, plus a `.gitignore` for both. `feat/sensor-sampling`, checked out
at the end, adds three commits: the feature (`src/sampling.rs`, a
`pub mod sampling;` line prepended to a copy of `sensor_kit`'s `lib.rs`, and
`scripts/watch-sampling.sh`), a happy-path test (`tests/sampling.rs`), and
the spec at `docs/superpowers/specs/sampling-design.md`. `--no-spec` omits
the spec commit. The files the branch commits live under `change/` and the
spec is `spec.md`; `sensor_kit` itself is only read. The branch is green at
the start (tests and `cargo fmt --check` pass, and the new code adds no
`clippy::pedantic` or docs-build warning), apart from `sensor_kit`'s own
seeded gate failures.

The never-improve rule applies to everything here: do not fix a seed in
`change/` or `spec.md`, do not make the spec agree with the code, and do not
add the test that would catch seed 3. Each seed sits in code that otherwise
reads as competent; that is the point.

Seeded items, numbered as later tasks refer to them, each with what the loop
should do to it:

1. `Sampler::reading(&self, raw: bool, smoothed: bool) -> Option<f32>`:
   two flags whose combination `raw && smoothed` is illegal (a
   `debug_assert!`, and "`raw` wins" in the rustdoc and the spec). An enum
   of latest, raw, or smoothed retires the illegal case at no caller cost.
   **Applied** as a clear structural win, with the spec's text and
   decisions updated to match.
2. `Sampler::new(sensor: Sensor, period_ms: u32, timeout_ms: u32)`: two
   confusable `u32`s at a public signature. Newtypes would cost a
   conversion at every caller, so this is a genuine trade-off. **Listed**
   for the user at the break point, not applied.
3. `Sampler::timed_out` computes `since_ms + self.timeout_ms` on a clock the
   spec says wraps at `u32::MAX`. A failed read whose `since_ms` (the last
   good read, or the first poll) is within `timeout_ms` of the wrap
   overflows: with a `Sensor` built with `address: 0` (every read fails),
   `Sampler::new(sensor, 10, 100)` then `poll(u32::MAX - 5)` panics with "attempt to add with overflow" in a
   debug build, and in release the deadline wraps low, so the bus is reset
   on the first failed read. No test covers it. **Fixed with a test**
   committed alongside (`now_ms.wrapping_sub(since_ms) >= timeout_ms`).
4. `Sampler::mean_calibrated` divides by a count that is `0.0` for an empty
   history, which reads as a NaN bug. Its one caller, `reading`, returns
   `None` through `self.history.last()?` before reaching it, so no input
   gets a NaN out. (`is_due`'s `wrapping_sub` beside it is likewise correct
   on the wrapping clock.) **Dropped as unconfirmed**: no failing test or
   trace can be produced, and the code is not changed.
5. `Sampler::push` trims the history with
   `self.history = self.history[1..].to_vec()`, a needless reallocation
   where `remove(0)` (or a ring) does. No lint fires on it. **Simplified**
   by the simplify stage only; neither review stage should raise it as a
   finding.
6. The spec's `## Design decisions` has one entry: `Sampler::new` takes the
   `Sensor` by value and owns it until `into_sensor`, never a `&Sensor`
   borrow. A reviewer may challenge the ownership. The code follows the entry. **Not re-raised**: reported at
   most as decided in one line.
7. The spec's body and a second `## Design decisions` entry (Adopted,
   round 1) record `poll(&mut self, now_ms: u32) -> Result<Option<f32>,
   SampleError>` with `SampleError::{TimedOut, Read(String)}`; the code
   returns `Option<f32>` and has no `SampleError`, so a timeout, a failed
   read and "not due" are all `None`. A departure from a recorded
   decision. **Fixed to match the spec**.
8. `scripts/watch-sampling.sh`, a non-Rust file outside the crate's source.
   **Named once and left untouched**.

With `--no-spec`, seeds 6 and 7 have no spec to measure against: the loop
should run anyway and list what a spec would have recorded.
