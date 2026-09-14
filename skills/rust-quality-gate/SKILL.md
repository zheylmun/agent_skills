---
name: rust-quality-gate
description: Use before reviewing Rust code, before merging a Rust change, or when asked to run lints, tests, or checks on a Rust crate
---

## Overview

This gate runs before a design review. Its output is **evidence for that
review, not a task list** — the review reads the findings and decides what
they mean; this skill does not fix them itself. It assumes
[rust-principles](../rust-principles/SKILL.md): a lint hit here is judged by
whether it names a real reduction in cognitive load, not treated as a box to
tick.

## The Suite

A request to "run clippy," "check the lints," or "run the tests" on a Rust
crate is a trigger for this whole suite, not for clippy alone — clippy
cannot see a formatting drift, a feature combination that fails to compile,
an absent test, or an undocumented public item, and a report built from
clippy output only is a partial gate wearing the name of the whole one. Run
all five, in order, even if an earlier one fails — a later command can
surface a different finding, and a skipped command is itself something to
report.

```sh
cargo +1.96.1 fmt --check
cargo +1.96.1 clippy --all-targets --all-features -- -W clippy::pedantic
cargo +1.96.1 test
cargo +1.96.1 hack check --feature-powerset --no-dev-deps
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo +1.96.1 doc --no-deps
```

Pin `+1.96.1` on every command. The default `stable` toolchain on this
machine is broken; an unpinned `cargo fmt` / `cargo clippy` / etc. can fail
for a toolchain reason that has nothing to do with the crate, and that
failure must not be mistaken for a finding. `cargo hack` is a separate
binary (`cargo install cargo-hack`) — if it is missing, the fourth command
fails to launch at all, which is itself something to report, not something
to route around. On this machine, every command also prints a
`warning: failed to auto-clean cache data ... Permission denied` line from
the shared `~/.cargo` registry cache before the crate's own output — that
is environment noise from a cache-directory permission issue, not a finding
about the crate, and is never reported.

The rustdoc invocation is what makes the link discipline in
[rust-principles](../rust-principles/SKILL.md) and
[rust-api-design](../rust-api-design/SKILL.md) enforceable rather than
aspirational: `-D missing_docs` fails the build on the first undocumented
public item, crate root included.

**On a crate with no doc discipline yet, `-D missing_docs` will fail on
almost every line the first time this suite runs.** That is not a reason to
narrow the flag, add a crate-level `#![allow(missing_docs)]`, or run the
command without `-D missing_docs` "just to see the rest of the suite." Keep
the command exactly as written. Report the wholesale failure as a single
cluster — "no public item in this crate is documented" — with the file and
item count, the same way any other cluster is reported below. Scoping the
flag down would make the gate quietly pass on a crate that has never had a
public doc contract, which is the one failure mode this command exists to
catch.

## A Pedantic Hit Is A Symptom, Not A Task

A pedantic clippy warning names a place where the code is more complex than
it needs to be. It does not name the fix. Two moves are always wrong:

- **Never run `clippy --fix` mechanically.** It resolves the warning at the
  call site the lint happened to point at, not the design decision that
  produced it everywhere else it will recur.
- **Never `#[allow]` a lint before asking what structural fact produced it.**
  An `#[allow]` is a decision that the lint is wrong for this case. Make
  that decision after understanding the cause, never as a way to make the
  gate pass without understanding it.

Before touching either tool, ask: *what one structural fact, if changed,
makes this warning and its siblings stop firing?* That question is the
actual output of this step. A pedantic hit found alone is worth a sentence.
A pedantic hit found four times in one module is worth a question about that
module's boundary — see the next section.

**Clusters matter more than individuals.** Three different lints on the same
function, or one lint repeated across a module, are very often one design
problem wearing several names. Fixing each occurrence locally makes the
warnings disappear and hides the finding that mattered.

## Reading Clusters

| Cluster | Usually names |
|---|---|
| Repeated `needless_pass_by_value` | Ownership taken where a borrow would do — usually traceable to one type whose call sites all copy this mistake |
| `too_many_arguments` alongside `fn_params_excessive_bools` on the same function | A missing parameter struct, or a function doing two jobs that a single argument list is straining to cover |
| Widespread `must_use_candidate` | The crate has never decided which return values are meaningful to ignore and which are not |
| `missing_errors_doc` / `missing_panics_doc` across several functions | The error design was never written down — see the Error Design section of [rust-api-design](../rust-api-design/SKILL.md) |
| `-D missing_docs` failing wholesale, not on isolated items | The crate has no doc discipline yet, not a handful of forgotten comments — this is the finding, not noise to filter |

A cluster is not "N warnings of the same lint name" mechanically counted —
it is warnings that share a cause. `must_use_candidate` on three unrelated
one-off functions across the crate is three small findings; `must_use_candidate`
on every function in one module is one finding about that module.

## Reporting

The report is the contract the design review consumes. State the outcome of
**all five commands**, not only clippy's — silence about a command reads as
"never run," so a command that passed is reported as clean explicitly, by
name, not omitted because it has nothing bad to say:

- **`cargo fmt --check`** — clean, or the files with formatting drift.
  Drift is its own line; it is not a lint and does not belong folded into
  the clippy clusters below.
- **`cargo clippy ... -W clippy::pedantic`** — clean, or clusters (below).
- **`cargo test`** — clean (tests ran and passed), the failing tests by
  name, or — the vacuous case — that the command exited zero because the
  crate has **no tests to run**. A test command that ran nothing has not
  exercised anything; report that absence as a finding, not as "clean."
- **`cargo hack check --feature-powerset --no-dev-deps`** — clean, the
  feature combination that failed to compile and why, or — the vacuous
  case — that nothing in the crate actually reads a feature, so no
  combination could have compiled differently from any other. That covers
  both a matrix with only one legal combination and a matrix with many
  legal combinations where nothing reads the feature at all: the
  combination count is not the test, whether any of them could have
  differed is. Before reporting this command clean, grep the source for
  `feature` inside any `cfg`/`cfg_attr` form — not just the two literal
  shapes `#[cfg(feature = "…")]` and `cfg!(feature = "…")`, which miss
  `#[cfg(not(feature = "…"))]`, `#[cfg(all(…))]`, `#[cfg(any(…))]`, and
  `#[cfg_attr(feature = "…", …)]` — and check `Cargo.toml` for an
  `optional = true` dependency or a `dep:name` feature, either of which
  makes a feature read even when it never appears in a source `cfg`. A
  feature that only toggles an optional dependency is still a feature the
  powerset is testing. A powerset that cannot fail is not testing anything
  either; report that the same way, naming the declared features that
  nothing reads.
- **the rustdoc command** — clean, broken intra-doc links by location, or
  the missing-docs cluster (see The Suite above).

For clippy clusters and a wholesale `-D missing_docs` failure, state:

1. **The lint names** in the cluster.
2. **The count** — how many occurrences, and how many distinct functions or
   items.
3. **The files** (and functions) it appears in.
4. **The suspected structural cause, as a question about the design** — not
   an answer, and never a fix. `read_sensor` taking nine parameters, four
   of them bare `bool`, is reported as *"is `read_sensor` doing one job or
   two — sampling the bus, and a retry/power-cycle policy — and is that why
   its signature needs nine parameters to express both?"*, never as
   "extract a `ReadOptions` struct" or any other named remedy: the question
   must not smuggle its own answer.

Never a flat list of unrelated individual warnings standing in for cluster
analysis, and never a fix list for any command — both are the review's job
to weigh, not this gate's job to pre-empt. This does not forbid a one-line
finding: a single unrelated hit (one `must_use_candidate` with no siblings,
one unformatted file) is reported as the one-sentence finding it is: see
"a cluster is not N warnings mechanically counted" above. What is forbidden
is presenting several warnings that share a cause as a warning-by-warning
list instead of naming the cluster.

If any suite command could not be run at all — `cargo-hack` not installed,
every `+1.96.1` invocation failing because the toolchain isn't installed,
network unavailable for a fresh `cargo doc` — report that explicitly as
**missing tooling**, naming which command and why. Do not silently drop it
from the report and do not present the remaining commands' output as if it
were the whole suite.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "I'll just run `clippy --fix` and see what's left." | `--fix` resolves the symptom at the one call site the lint pointed at and leaves the structural cause to recur at the next one. Read the cluster before touching any tool. |
| "These are just pedantic lints, I'll `#![allow(clippy::pedantic)]` at the crate root." | That silences the exact signal this gate exists to surface, permanently, for every future change. An `#[allow]` is a decision made after understanding the cause, scoped to the one case where the lint is genuinely wrong — never a blanket opt-out. |
| "I found 40 warnings, here they are." | Forty warnings are rarely forty findings. Group them by shared cause first — see Reading Clusters — and report the handful of structural questions that remain, not the raw list. |
| "The crate has no docs yet, I'll skip the rustdoc command." | That is precisely the finding `-D missing_docs` exists to catch. Run it, then report the wholesale failure as one cluster — see The Suite. |
| "I fixed each `too_many_arguments` warning by adding an `#[allow]` with a comment explaining why." | An explanatory comment defending an `#[allow]` is the same design smell [rust-principles](../rust-principles/SKILL.md) names for any other comment: it is evidence the design, not the comment, needs to change. |
| "`cargo fmt --check` failed, so I ran it without the toolchain pin to move on." | An unpinned command on this machine can fail because the default `stable` toolchain is broken, not because of anything in the crate. Use `+1.96.1` on every command in the suite; a failure that persists with the pin is a real finding. |
| "`cargo test` exited 0, so that command is clean." | Exit 0 with zero tests run is not clean — the crate has never been exercised. Report "no tests exist" as its own finding, the same as a failing test would be. |
