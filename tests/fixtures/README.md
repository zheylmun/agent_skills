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
