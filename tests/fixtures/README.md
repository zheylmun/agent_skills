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

Seeded flaws, in order of what they test:

1. `read_sensor` — six arguments, a `bool` parameter, primitive obsession on
   `address`/`id`, stringly-typed error.
2. The block comment — an explanatory `why` comment standing in for a design
   fix.
3. `connect` — `Box<dyn Error>` in a public signature, takes ownership
   needlessly, and a comment blaming the borrow checker.
4. Every public item is undocumented — no `///` anywhere, no
   `#![warn(missing_docs)]`.
5. `Sensor` — all fields `pub`, no `#[non_exhaustive]`, no invariants.
6. `calibrate` — no `#[must_use]` on a pure function.
7. Features `std` and `logging` are declared but unused, so the feature
   powerset is vacuous.

### Adjustment made to the fixture

Flaw 1 as originally specified (six arguments, one trailing `bool`) does not
trip `clippy::pedantic`'s `too_many_arguments` (default threshold: more than
7 arguments) or `fn_params_excessive_bools` (default threshold: more than 3
`bool` parameters) — both lints that a later task depends on. Without
changing what the flaw *is* (too many primitive/bool parameters standing in
for a proper request type), `read_sensor` was given two additional `bool`
parameters (`power_cycle`, `verbose`, `log_errors` — three in total added to
the original `retry`) to push it to 9 arguments and 4 `bool` parameters, and
each new `bool` is read in the function body so it doesn't also trip an
unrelated "unused parameter" warning. This makes the seeded flaw more
pronounced, not different, and both lints now fire alongside
`must_use_candidate`, `missing_errors_doc`, and `cast_precision_loss`.
