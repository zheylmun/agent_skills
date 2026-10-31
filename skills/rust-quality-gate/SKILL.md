---
name: rust-quality-gate
description: Use before reviewing Rust code, before merging a Rust change, or when asked to run lints, tests, or checks on a Rust crate
---

## Overview

This gate runs before a design review. Its output is **evidence for that
review, not a task list** — this skill reports **observations**; the review
reads them and decides what they mean, and the word *finding* is reserved for
what the review produces. This skill does not fix anything itself. It assumes
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
surface a different observation, and a skipped command is itself something to
report.

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -W clippy::pedantic
cargo test
cargo hack check --feature-powerset --no-dev-deps
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo doc --no-deps
```

**Run them unpinned.** The crate's `rust-toolchain.toml` decides the
toolchain; a `+version` pin on the command line overrides that file, and
overriding it is wrong — a target's `rust-std` is installed per toolchain, so
a pinned invocation on a crate whose target was added to its own pinned
toolchain fails with `can't find crate for \`core\``, a failure the crate did
not cause. If a command fails for a toolchain or environment reason rather
than a crate reason — a missing component, a broken default toolchain, a
`~/.cargo` cache-permission warning, no network for a fresh `cargo doc` —
that is tooling, reported under the tooling state below, never as an
observation about the crate. `cargo hack` is a separate binary
(`cargo install cargo-hack`); if it is missing, the fourth command fails to
launch at all, which is itself something to report, not something to route
around.

### On a workspace, pass `--workspace`

At a workspace root `cargo fmt --check` already covers every member, but
clippy, `cargo test`, `cargo hack check`, and `cargo doc` operate on the
**default members only**, silently. On any multi-crate repo the other four
become:

```sh
cargo clippy --workspace --all-targets --all-features -- -W clippy::pedantic
cargo test --workspace
cargo hack check --workspace --feature-powerset --no-dev-deps
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo doc --workspace --no-deps
```

Check for `[workspace]` in the root `Cargo.toml` before running anything.
Reporting a default-member run as though it covered the workspace is the
partial gate wearing the name of the whole one — say which members were
covered whenever the run was not `--workspace`.

### On a crate that cannot build for the host

A crate depending on `cortex-m-rt`, an stm32 HAL, or a `defmt` global logger
does not compile for the host at all. The target flag therefore applies to
**the whole suite, not one command**: `cargo fmt --check` is
target-independent and stays as written; clippy, `cargo hack check`, and
`cargo doc` each take `--target <triple>` (substitute the crate's real triple,
e.g. `thumbv7em-none-eabihf`, never the placeholder):

```sh
cargo fmt --check
cargo clippy --all-targets --target <target> --features <selection> -- -W clippy::pedantic
cargo hack check --feature-powerset --no-dev-deps --target <target>
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo doc --no-deps --target <target>
```

`cargo test` is the one that does not simply take a flag: a test binary built
for a bare-metal triple cannot run on the host. Two answers are legitimate,
and which one the crate chose is itself worth reporting:

- **A host-testable core split** — the target-independent logic lives in
  modules that build for the host, and `cargo test` (no `--target`) exercises
  those, while the HAL-bound layer is not unit-tested.
- **`--target` plus a runner** — `cargo test --target <target>` with a
  `[target.<target>] runner = "probe-rs run --chip <part>"` in `.cargo/config.toml`,
  or a `defmt-test` harness, executing on attached hardware.

If the crate has neither and no device is attached, `cargo test` is **not
applicable on this target** — report it under that state below. It is not
clean, and it is not an observation about the crate's test discipline.

### Bound `--all-features` and the powerset before running them

Both are unbounded as written, and two crate shapes break them:

- **Mutually exclusive features.** A HAL-style crate with one feature per chip
  fails `--all-features` *by design*; the compile error is the feature design
  working, not an observation. Run clippy against one representative selection
  instead (`--features <one chip>`) and name in the report which selection you
  ran and what that leaves unchecked.
- **Combinatorial explosion.** A powerset over 15 features is 32,768 builds.

`cargo hack` has flags for exactly this — use them rather than dropping the
command:

```sh
cargo hack check --feature-powerset --no-dev-deps \
  --mutually-exclusive-features stm32f401,stm32f411,stm32f427 \
  --group-features defmt,defmt-rtt \
  --exclude-features unstable-docs \
  --depth 2
```

`--mutually-exclusive-features` stops the illegal combinations being built at
all, `--group-features` collapses features that are only ever enabled together,
`--exclude-features` drops one that is not a real compile-time fork, and
`--depth 2` bounds the run to every feature and every pair — where nearly all
real feature-interaction breakage lives.

If a bounded run still wedges — running for hours, or filling the disk — stop
it and report what you ran, how far it got, and which combinations remain
unbuilt. A partial matrix honestly described is evidence; a partial matrix
reported as a clean powerset is not.

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
warnings disappear and hides the observation that mattered.

## Reading Clusters

| Cluster | Usually names |
|---|---|
| Repeated `needless_pass_by_value` | Ownership taken where a borrow would do — usually traceable to one type whose call sites all copy this mistake |
| `too_many_arguments` alongside `fn_params_excessive_bools` on the same function | A missing parameter struct, or a function doing two jobs that a single argument list is straining to cover |
| Widespread `must_use_candidate` | The crate has never decided which return values are meaningful to ignore and which are not |
| `missing_errors_doc` / `missing_panics_doc` across several functions | The error design was never written down — see the Error Design section of [rust-api-design](../rust-api-design/SKILL.md) |
| `-D missing_docs` failing wholesale, not on isolated items | The crate has no doc discipline yet, not a handful of forgotten comments — this is the observation, not noise to filter |

A cluster is not "N warnings of the same lint name" mechanically counted —
it is warnings that share a cause. `must_use_candidate` on three unrelated
one-off functions across the crate is three small observations;
`must_use_candidate` on every function in one module is one observation about
that module.

## Reporting

The report is the contract the design review consumes. It reports
**observations**, never findings — a finding is what the review makes of an
observation, and this report stops one step short of that on purpose.

State the outcome of **all five commands**, not only clippy's — silence about
a command reads as "never run," so a command that passed is reported as clean
explicitly, by name, not omitted because it has nothing bad to say.

Every command lands in exactly one of **three states**:

1. **Ran** — clean, or with the observations below. A bounded or
   selection-narrowed run (see The Suite) is a *ran* state, reported with the
   bound and what it leaves uncovered.
2. **Not applicable on this crate or target** — the command cannot be
   meaningfully run here and no tooling is missing: `cargo test` on a
   bare-metal crate with no host-testable core and no attached device,
   `--all-features` on a crate whose chip-select features are mutually
   exclusive by design. Say which command, which triple or feature design
   makes it inapplicable, and what is therefore unevidenced. **This state is
   not a failure and is never reported as an observation about the crate** —
   the compile error a host build produces on a `cortex-m-rt` crate is the
   target being wrong, not the code.
3. **Tooling absent** — the command could not launch or could not complete for
   an environment reason: `cargo-hack` not installed, a missing component or
   broken default toolchain, a target's `rust-std` not added, no network for a
   fresh `cargo doc`. Name the command and the reason.

The distinction between 2 and 3 matters: state 3 is fixable by installing
something, state 2 is a fact about this crate that will still hold tomorrow.
Collapsing either into "ran and failed" is how a gate hands a review a wall of
fabricated evidence.

Per command, in the *ran* state:

- **`cargo fmt --check`** — clean, or the files with formatting drift.
  Drift is its own line; it is not a lint and does not belong folded into
  the clippy clusters below. Always report it — "run `cargo fmt`" is
  useful on its own, and it is the one thing in this whole suite that is
  fixed by a single command with nothing to decide. It is also the thing
  a reader is most likely to have simply not run yet.

  Attribute it correctly, though. Check when the crate was last touched
  (`git log -1 --format=%ad`). `rustfmt`'s default style changes between
  releases, so a crate untouched for a year or more shows drift its
  author never introduced. Say which it is: "N files need `cargo fmt`
  — the crate predates the current rustfmt, so this is toolchain churn,
  not drift you left behind" reads very differently from the same
  sentence about a crate committed last week, and only the second says
  anything about how the code is being maintained.

  Either way the fix is the same and belongs in CI, where drift never
  accumulates far enough to need attributing. If `cargo fmt --check`
  isn't in the crate's CI, that absence is worth one clause.

  **`cargo fmt --check` passing does not mean the crate is within its declared
  width.** rustfmt wraps code and leaves comments alone — `wrap_comments` is
  nightly-only — so on a stable toolchain a doc comment of any length passes.
  On a crate that declares `max_width` in `rustfmt.toml`, check the comments
  yourself, reading the limit from the file rather than assuming rustfmt's
  default of 100:

  ```sh
  python3 -c 'import re,sys,pathlib
  cfg = pathlib.Path("rustfmt.toml")
  m = re.search(r"^\s*max_width\s*=\s*(\d+)", cfg.read_text(), re.M) if cfg.is_file() else None
  lim = int(m.group(1)) if m else 100
  for p in sys.argv[1:]:
      for n, line in enumerate(pathlib.Path(p).read_text().splitlines(), 1):
          if len(line) > lim: print(f"{p}:{n}: {len(line)} cols, limit {lim}")' \
    $(git ls-files '*.rs')
  ```

  Use `python3` and not `awk` here: `awk`'s `length` counts **bytes** on BSD
  awk and in a C locale, so one em dash in a doc comment inflates the count by
  two and a line that fits gets reported as over. A width check that
  miscounts the very prose it exists to measure is worse than none.

  Report what it finds as its own line, distinct from drift: these are lines
  `cargo fmt` will not fix, so "run `cargo fmt`" is not the remedy. If the
  crate declares no `max_width`, there is no limit to hold it to and this is
  not an observation. If it declares one and nothing in CI enforces it on
  comments, that absence is worth the same clause as a missing `fmt` check —
  a declared limit nothing checks is one the author is being asked to hold by
  eye.
- **`cargo clippy ... -W clippy::pedantic`** — clean, the clusters (below),
  or — the common case on a mature crate — a handful of isolated hits that
  share no cause, reported as the one-sentence observations they are rather
  than dressed up as clusters. See the carve-out below "Never a flat list."
  Name the feature selection if it was not `--all-features`.
- **`cargo test`** — clean (tests ran and passed), the failing tests by
  name, or — the vacuous case — that the command exited zero because the
  crate has **no tests to run**. A test command that ran nothing has not
  exercised anything; report that absence as an observation, not as "clean."
  On a crate that cannot run host tests, this is state 2 instead — see The
  Suite — and the absence of host tests is not held against it.
- **`cargo hack check --feature-powerset --no-dev-deps`** — clean, the
  feature combination that failed to compile and why, or — the vacuous
  case — that nothing in the crate actually reads a feature, so no
  combination could have compiled differently from any other. That covers
  both a matrix with only one legal combination and a matrix with many
  legal combinations where nothing reads the feature at all: the
  combination count is not the test, whether any of them could have
  differed is. Before reporting this command clean, run both of these
  verbatim — do not improvise the search:

  ```sh
  rg -n 'cfg!?_?attr?\s*\(|feature\s*=' --glob '*.rs' src/
  rg -n 'optional\s*=\s*true|dep:' Cargo.toml
  ```

  The first catches every `cfg` shape, not only `#[cfg(feature = "…")]` and
  `cfg!(feature = "…")` — it also hits `#[cfg(not(...))]`, `#[cfg(all(...))]`,
  `#[cfg(any(...))]`, and `#[cfg_attr(feature = "…", …)]`. The second catches
  a feature that is read only by enabling an optional dependency, which never
  appears in a source `cfg` at all and is still a feature the powerset tests.
  Two silent outputs together mean nothing reads a feature. A powerset that
  cannot fail is not testing anything; report that the same way, naming the
  declared features that nothing reads.
- **the rustdoc command** — clean, broken intra-doc links by location, or
  the missing-docs cluster (see The Suite above).

For clippy clusters and a wholesale `-D missing_docs` failure, state:

1. **The lint names** in the cluster.
2. **The count** — how many occurrences, and how many distinct functions or
   items. **Never count by reading the human-readable output.** `--all-targets`
   compiles the library and the test target separately, so every warning in
   library code is emitted twice, and a crate with benches or examples emits
   it more. Counting the printed warnings reports a number that can be double
   the truth or worse, and hands the review an inflated figure as fact. Count
   distinct `(file, line, lint)` triples instead:

   ```sh
   cargo clippy --all-targets --all-features --message-format=json -- -W clippy::pedantic 2>/dev/null \
     | python3 -c 'import sys,json,collections
   c=collections.Counter()
   for l in sys.stdin:
       try: m=json.loads(l)
       except ValueError: continue
       if m.get("reason")!="compiler-message": continue
       d=m["message"]; code=(d.get("code") or {}).get("code","")
       if not code.startswith("clippy::"): continue
       for s in d.get("spans",[]):
           if s.get("is_primary"): c[(code,s["file_name"],s["line_start"])]=1
   n=collections.Counter(k[0] for k in c)
   for lint,k in n.most_common(): print(k,lint)'
   ```

   If you report a count you did not derive this way, say so and say it is
   unverified.
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
observation: a single unrelated hit (one `must_use_candidate` with no
siblings, one unformatted file) is reported as the one-sentence observation it
is: see "a cluster is not N warnings mechanically counted" above. What is
forbidden is presenting several warnings that share a cause as a
warning-by-warning list instead of naming the cluster.

Whatever the state, do not silently drop a command from the report, and never
present the remaining commands' output as if it were the whole suite.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "I'll just run `clippy --fix` and see what's left." | `--fix` resolves the symptom at the one call site the lint pointed at and leaves the structural cause to recur at the next one. Read the cluster before touching any tool. |
| "These are just pedantic lints, I'll `#![allow(clippy::pedantic)]` at the crate root." | That silences the exact signal this gate exists to surface, permanently, for every future change. An `#[allow]` is a decision made after understanding the cause, scoped to the one case where the lint is genuinely wrong — never a blanket opt-out. |
| "I found 40 warnings, here they are." | Forty warnings are rarely forty observations. Group them by shared cause first — see Reading Clusters — and report the handful of structural questions that remain, not the raw list. |
| "The crate has no docs yet, I'll skip the rustdoc command." | That is precisely the observation `-D missing_docs` exists to catch. Run it, then report the wholesale failure as one cluster — see The Suite. |
| "I fixed each `too_many_arguments` warning by adding an `#[allow]` with a comment explaining why." | An explanatory comment defending an `#[allow]` is the same design smell [rust-principles](../rust-principles/SKILL.md) names for any other comment: it is evidence the design, not the comment, needs to change. |
| "The crate wouldn't build, so I added a toolchain pin like `cargo +1.90.0 clippy` to get past it." | A command-line pin overrides the crate's `rust-toolchain.toml` and looks for the target's `rust-std` under the pinned toolchain instead, which is how a working embedded crate starts failing with `can't find crate for \`core\``. Run unpinned and let the crate's toolchain file decide; a toolchain-level failure is reported as tooling absent, not as an observation. |
| "The host build of this `cortex-m-rt` crate fails, so clippy and the tests are findings." | That is the wrong target, not a fact about the crate. The target flag applies to the whole suite; a command that still cannot run is reported as **not applicable on this target**, which is a distinct state from both clean and failed. |
| "`--all-features` fails to compile, so the feature design is broken." | On a crate with mutually exclusive chip-select features, `--all-features` failing is the feature design working. Bound the run — `--mutually-exclusive-features`, `--group-features`, `--exclude-features`, `--depth` — and report which selection you ran. |
| "`cargo fmt --check` passed, so the crate is within its declared width." | Only its code is. rustfmt does not wrap comments on stable, so a doc comment of any length passes. Check comment width separately against the `max_width` the crate declares — see the `cargo fmt` entry under Reporting. |
| "I ran the suite at the workspace root, so every crate is covered." | Only `cargo fmt` is. Clippy, tests, `cargo hack`, and `cargo doc` cover default members only, silently. Pass `--workspace`, or say which members the report actually covers. |
| "`cargo test` exited 0, so that command is clean." | Exit 0 with zero tests run is not clean — the crate has never been exercised. Report "no tests exist" as its own observation, the same as a failing test would be. |
