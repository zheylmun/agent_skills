# Scenario: end-to-end

Full-stack check that `rust-design-review` invokes `rust-quality-gate` and
produces design-altitude prose, not a lint list, against
`tests/fixtures/sensor_kit`.

## Substitution

The task brief's Step 7 asks to install the plugin and ask a fresh session to
"Review the crate at tests/fixtures/sensor_kit," confirming it invokes
`rust-design-review`, which invokes `rust-quality-gate`. That was not
available to the implementing agent (no plugin install, no ability to start a
session), so the following substitution was used instead, per an explicit
decision from the task owner: run the real `rust-quality-gate` five-command
suite against the fixture (read-only, pinned `+1.96.1`), then dispatch one
clean-room `general-purpose` subagent (model: sonnet) whose entire context
was the full text of `rust-design-review/SKILL.md`, the full text of
`rust-quality-gate/SKILL.md`, the fixture's `Cargo.toml` and `src/lib.rs`
inlined, and the gate's real output inlined and framed as "already run for
you, do not re-derive it." The subagent was told explicitly not to read any
file or use any tool, and no repo path was given anywhere in its prompt. The
controller runs the real plugin-install version of this check separately.

## Gate output given to the subagent (real, from `tests/fixtures/sensor_kit`)

```
# rust-quality-gate report: tests/fixtures/sensor_kit

All five commands were run with the pinned toolchain (`+1.96.1`). The
recurring `warning: failed to auto-clean cache data ... Permission denied`
line from the shared `~/.cargo` registry cache is environment noise, not a
finding, and is omitted below.

## cargo +1.96.1 fmt --check
Clean. No formatting drift.

## cargo +1.96.1 clippy --all-targets --all-features -- -W clippy::pedantic
Not clean. Two clusters and two one-off hits:

1. **`too_many_arguments` (9/7) + `fn_params_excessive_bools` (4 bools) on
   `read_sensor`, src/lib.rs:12-54.** Structural question: is `read_sensor`
   doing one job or two — sampling the bus, and a retry/power-cycle/logging
   policy around it — and is that why its signature needs nine parameters,
   four of them bare `bool`, to express both?

2. **`missing_errors_doc`, 2 occurrences, distinct functions:**
   `read_sensor` (src/lib.rs:12, returns `Result<f32, String>`) and
   `connect` (src/lib.rs:66, returns `Result<Sensor, Box<dyn Error>>`).
   Neither function's doc comment states the error contract (neither
   function has a doc comment at all).

3. One-off: `cast_precision_loss` on the `id as f32` cast, src/lib.rs:37.

4. One-off: `must_use_candidate` on `calibrate`, src/lib.rs:73.

## cargo +1.96.1 test
Vacuous. Exit 0, but the crate has zero tests: `running 0 tests` for both
the unit-test binary and the doctest run. The suite has never exercised the
crate's public contract.

## cargo +1.96.1 hack check --feature-powerset --no-dev-deps
Vacuous. `Cargo.toml` declares two non-default features (`std`, `logging`)
alongside `default = []`, and the powerset built and checked all 8
combinations without failure — but grepping src/lib.rs finds no
`#[cfg(feature = "…")]`, `cfg!(feature = "…")`, `#[cfg(not(feature = "…"))]`,
`#[cfg(all(…))]`/`#[cfg(any(…))]` naming a feature, or `#[cfg_attr(feature =
"…", …)]` anywhere in the crate, and `Cargo.toml` has no `[dependencies]`
section at all, so neither feature gates an `optional = true` dependency
either. Nothing in the crate reads `std` or `logging`; the 8-combination
powerset could not have compiled differently from any other, and passing it
proves nothing about either feature.

## RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo +1.96.1 doc --no-deps
Wholesale failure — the crate has no doc discipline yet, not a handful of
forgotten items. `-D missing_docs` fails on the crate root itself (no `//!`)
and then on every public item: `Sensor` and all five of its fields,
`read_sensor`, `connect`, and `calibrate` — 9 missing-docs errors across
src/lib.rs, build halted at `could not document sensor_kit`.
```

## Design review produced (verbatim)

> # Design Review: `tests/fixtures/sensor_kit`
>
> `cargo +1.96.1 fmt --check` is clean, so formatting is not a topic here. Everything else in the gate's report — the clippy clusters, the vacuous test run, the vacuous feature powerset, and the wholesale `missing_docs` failure — is addressed below or in the closing paragraph.
>
> ## 1. `Sensor`'s public fields make `connect` a decoration, not a constructor
>
> `Sensor` exposes `id`, `address`, `scale`, `offset`, and `connected` as plain `pub` fields, and `connect(s: Sensor) -> Result<Sensor, Box<dyn Error>>` exists specifically to flip `connected` to `true`. But because the field is public, nothing stops a caller from writing `sensor.connected = true` directly and skipping `connect` entirely — the function is not a gate on the invariant it appears to guard, it's an alternate path to the same state that happens to also exist. Every caller of `Sensor` is affected by this, because the type never actually enforces "connected" means "went through `connect`" — and because `connected` is a `pub` field today, tightening it later (making it private behind an accessor, or replacing the boolean with a typestate like `Sensor<Disconnected>` / `Sensor<Connected>`) is a breaking change to the struct's public field list, which is a strictly more expensive thing to retire once callers depend on constructing `Sensor` as a plain struct literal. The fix that retires the cost is to stop treating `connected` (and arguably the whole struct) as caller-assignable data: private fields plus a constructor (or a typestate split) means a caller can no longer forge "connected" by hand, and every function downstream that currently has to trust the boolean could instead trust the type.
>
> ## 2. `read_sensor` re-derives `Sensor`'s identity fields instead of taking a `Sensor`, and conflates sampling with a retry/logging policy
>
> `read_sensor(id: u32, address: u8, scale: f32, offset: f32, retry: bool, power_cycle: bool, verbose: bool, log_errors: bool, timeout_ms: u32)` takes `id`, `address`, `scale`, and `offset` as loose positional parameters even though those are exactly `Sensor`'s fields (minus `connected`). A caller holding a `Sensor` cannot pass it — they have to destructure it back into four primitives, which means the struct and the function that reads it are not actually connected to each other, and any caller has to keep the two representations in sync by hand. Worse, `scale: f32` and `offset: f32` sit adjacent in the signature with identical types, so a transposed call compiles silently and produces a wrong calibrated value with no error at any level. This is exactly the gate's first cluster (`too_many_arguments` + `fn_params_excessive_bools`), and having read the function, the structural cause is visible directly: `read_sensor` is doing two jobs — sampling the bus, and a retry/power-cycle/verbosity/logging policy wrapped around that sample — and the four trailing bools are that second job smuggled into the first job's parameter list. The comment at the top of the function ("We take a lot of arguments because the sensor needs all of them") is defending that shape in prose rather than the types encoding it, which is the same relocation-not-reduction smell in miniature. Every caller pays for this: they must supply a full identity quadruple by hand on every read, keep `scale`/`offset` ordering correct with no help from the compiler, and decide all four policy bools even when they want none of them. Retiring it is a breaking signature change (which is why it's worth doing now rather than after the crate has downstream callers): having `read_sensor` take `&Sensor` deletes the duplicated-identity problem and the swappable-`f32` problem in one move, and separating a `ReadPolicy`-shaped type (or splitting "sample once" from "sample with retry/power-cycle" as two functions) would mean a caller who wants a plain read no longer has to answer four unrelated policy questions to get one.
>
> ## 3. The error types promise more than the code can produce, and less than a caller needs
>
> `read_sensor` returns `Result<f32, String>`, which collapses three distinguishable failure modes — a zero timeout, no device answering at the given address, and retries exhausted — into unstructured text; any caller who wants to react differently to "bad configuration" versus "hardware didn't respond" has to match on string content, which the gate's `missing_errors_doc` hit on this function is a symptom of (there's no error contract to document because there's no error type to document it against). In the other direction, `connect` returns `Result<Sensor, Box<dyn std::error::Error>>`, but nothing in its body can produce an `Err` — it unconditionally sets `connected = true` and returns `Ok`. That's an error variant with zero callers who can ever legitimately handle it, forced onto every caller of `connect` anyway (the `?` or `match` they have to write for a failure that structurally cannot happen). This is the gate's second cluster (`missing_errors_doc` on both functions), and the fix for it isn't a doc comment on either — it's that `read_sensor` should return an enum whose variants are the three real failure modes so a caller can match without parsing text, and `connect` should return `Sensor` (or `&mut Sensor` / nothing, once field-privacy from finding 1 removes the ownership shuffle the current signature is built around) rather than a `Result` wrapping a failure that never occurs.
>
> ## 4. The crate has no doc contract at all
>
> `-D missing_docs` fails wholesale — the crate root has no `//!`, and every public item (`Sensor` and all five of its fields, `read_sensor`, `connect`, `calibrate`) is undocumented. Per the gate's own framing this is one finding, not nine: the crate has never had a doc discipline, so there's no per-item gap to itemize, only a contract to establish. Once findings 1–3 land and the public surface actually changes shape, writing the doc contract (including the `# Errors` sections findings 2/3 create the need for) is the point at which this stops being a wholesale failure.
>
> Two things belong in this review only as carried-in gate findings, ranked low because neither costs a caller any present ability to use the API and neither is expensive to retire: `cargo test` ran zero tests in both the unit and doctest runners, so nothing about `read_sensor`'s retry/power-cycle branches, `connect`'s (currently infallible) path, or `calibrate`'s arithmetic has ever been executed — the crate's public contract is unverified, not merely undocumented. And the feature powerset is vacuous in the stronger sense the gate describes: `std` and `logging` are declared in `Cargo.toml` but nothing in `src/lib.rs` reads either one (no `cfg`, no `cfg_attr`, no optional dependency), so the 8 combinations the gate checked could not have compiled differently from each other — the crate is carrying two features that do nothing. The two one-off clippy hits (`cast_precision_loss` on `id as f32`, `must_use_candidate` on `calibrate`) are minor enough to fold in here rather than stand as findings of their own.
>
> Two things are out of this review's scope and belong to `/code-review` instead: `sample_raw`'s `f32::from(raw) * scale + offset + id as f32` adds the sensor's numeric `id` into a physically-calibrated reading, which looks like a correctness bug rather than a design choice, and `timeout_ms` is validated for `== 0` but never otherwise used to bound anything in the retry loop, so the parameter currently does less than its name promises. Performance and `unsafe`-soundness have no owner in this plugin and don't arise here regardless (there's no `unsafe` in the crate).

## Verdict

Leads with a design fact (`Sensor`'s public fields making `connect` a
decoration, not an enforced invariant), not a lint — `too_many_arguments`,
`missing_errors_doc`, and the other clippy names appear only as supporting
evidence for facts already stated in the crate's own vocabulary, exactly as
`rust-design-review` requires. All five gate commands are accounted for
(`fmt` acknowledged clean in the opening line; clippy's two clusters and two
one-offs become findings 2 and part of the closing paragraph; the vacuous
test run and vacuous feature powerset are carried in near the end, ranked
low, never leading; the wholesale `missing_docs` failure is one finding, not
nine). Two real bugs noticed while reading (`id` bleeding into the
calibration formula; `timeout_ms` validated but never enforced) are held to
one line each in the closing paragraph and explicitly handed to
`/code-review` rather than leading the review — matching the skill's
bug-hunt guardrail.
