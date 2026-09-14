# Rust Craft Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a five-skill Claude Code plugin that makes an agent write and review Rust with this author's API-design, documentation, and simplicity discipline.

**Architecture:** One plugin repo, five skills in a flat `skills/` namespace. `rust-principles` holds the shared philosophy and every other skill links to it rather than restating it. Skills are documentation, so they are built with TDD-for-documentation: a baseline pressure scenario run against a deliberately flawed fixture crate establishes RED, the skill is written to address the exact failures observed, and the scenario is re-run to confirm GREEN.

**Tech Stack:** Markdown skills with YAML frontmatter, Claude Code plugin manifests (`.claude-plugin/plugin.json`, `.claude-plugin/marketplace.json`), a Rust fixture crate for testing, `cargo` 1.96.1 toolchain, `cargo-hack`.

**Spec:** `docs/superpowers/specs/2026-09-14-rust-craft-skills-design.md` — read it before starting. Every skill's content is derived from it.

## Global Constraints

- Plugin name: `rust-craft`. Version starts at `0.1.0`. Author: Zach Heylmun. Repo: `git@github.com:zheylmun/agent_skills.git`.
- Skill names, exactly: `rust-principles`, `rust-api-design`, `rust-quality-gate`, `rust-design-review`, `rust-embedded`.
- Frontmatter has exactly two fields, `name` and `description`, under 1024 characters total.
- **`description` states triggering conditions only.** Never summarize a skill's workflow in its description — agents follow the description instead of reading the body when it does.
- Skills are reference guides, not narratives. No story about how something was solved once.
- The five guiding principles live in `rust-principles` and nowhere else. Other skills link with a relative markdown link: `[rust-principles](../rust-principles/SKILL.md)`.
- **Toolchain:** the default `stable` toolchain on this machine is broken (missing the `cargo` component). Use `+1.96.1` explicitly, or repair stable in Task 1. `cargo-hack` is not installed; Task 1 installs it.
- Every task ends with a commit. Commit messages end with `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`.

---

### Task 1: Plugin scaffolding and toolchain prerequisites

Makes the repo installable as a plugin and gets the Rust tooling the later tasks depend on.

**Files:**
- Create: `.claude-plugin/plugin.json`
- Create: `.claude-plugin/marketplace.json`
- Create: `.gitignore`
- Create: `README.md`

**Interfaces:**
- Consumes: nothing.
- Produces: an installable plugin whose `skills/` directory later tasks populate. `marketplace.json` must list the plugin under `plugins[0].source` as `"./"` so the repo installs from its own path.

- [ ] **Step 1: Repair the toolchain and install cargo-hack**

```bash
rustup component add cargo --toolchain stable || rustup toolchain install stable --force
cargo +1.96.1 install cargo-hack --locked
```

- [ ] **Step 2: Verify the tooling responds**

```bash
cargo +1.96.1 --version && cargo +1.96.1 clippy --version && \
cargo +1.96.1 fmt --version && cargo +1.96.1 hack --version
```

Expected: four version lines, no errors. If `stable` still fails, that is acceptable — every command in this plan pins `+1.96.1`. Record which toolchain works in `README.md`.

- [ ] **Step 3: Write the plugin manifest**

`.claude-plugin/plugin.json`:

```json
{
  "name": "rust-craft",
  "description": "Rust API design, documentation discipline, and design review skills",
  "version": "0.1.0",
  "author": {
    "name": "Zach Heylmun"
  },
  "homepage": "https://github.com/zheylmun/agent_skills",
  "repository": "https://github.com/zheylmun/agent_skills",
  "license": "MIT",
  "keywords": ["rust", "api-design", "documentation", "code-review"]
}
```

- [ ] **Step 4: Write the marketplace manifest**

`.claude-plugin/marketplace.json`:

```json
{
  "name": "zheylmun-agent-skills",
  "owner": {
    "name": "Zach Heylmun"
  },
  "plugins": [
    {
      "name": "rust-craft",
      "source": "./",
      "description": "Rust API design, documentation discipline, and design review skills"
    }
  ]
}
```

- [ ] **Step 5: Write `.gitignore`**

```
target/
Cargo.lock
.DS_Store
```

- [ ] **Step 6: Write `README.md`**

Contents: one paragraph on what the plugin is, the five skill names with a one-line purpose each, install instructions (`/plugin marketplace add ~/dev/zheylmun/agent_skills` then `/plugin install rust-craft@zheylmun-agent-skills`), and a "Toolchain" section recording the working toolchain from Step 2. Link to the spec.

- [ ] **Step 7: Verify the manifests parse**

```bash
python3 -c "import json;[json.load(open(p)) for p in ['.claude-plugin/plugin.json','.claude-plugin/marketplace.json']];print('ok')"
```

Expected: `ok`

- [ ] **Step 8: Install the plugin locally and confirm it loads**

Run `/plugin marketplace add ~/dev/zheylmun/agent_skills` and `/plugin install rust-craft@zheylmun-agent-skills` in Claude Code. Expected: install succeeds. Zero skills listed is correct at this point — `skills/` is empty.

- [ ] **Step 9: Commit**

```bash
git add .claude-plugin README.md .gitignore
git commit -m "$(printf 'Add rust-craft plugin scaffolding\n\nCo-Authored-By: Claude Opus 5 <noreply@anthropic.com>')"
```

---

### Task 2: The fixture crate

A deliberately flawed crate that every later skill is tested against. Without it, the baseline scenarios have nothing concrete to fail on.

**Files:**
- Create: `tests/fixtures/sensor_kit/Cargo.toml`
- Create: `tests/fixtures/sensor_kit/src/lib.rs`
- Create: `tests/fixtures/README.md`

**Interfaces:**
- Consumes: the toolchain from Task 1.
- Produces: a crate at `tests/fixtures/sensor_kit` that compiles, and whose flaws are enumerated in `tests/fixtures/README.md`. Tasks 3–6 reference those flaws by name.

The crate must compile cleanly under plain `cargo build` while containing every flaw the skills are meant to catch. A crate that does not build cannot be reviewed.

- [ ] **Step 1: Write `Cargo.toml`**

```toml
[package]
name = "sensor_kit"
version = "0.1.0"
edition = "2021"
publish = false

[features]
default = []
std = []
logging = []
```

Three features with no `--all-features` coherence give `cargo hack` something real to find.

- [ ] **Step 2: Write `src/lib.rs` with the seeded flaws**

```rust
pub struct Sensor {
    pub id: u32,
    pub address: u8,
    pub scale: f32,
    pub offset: f32,
    pub connected: bool,
}

// This function reads the sensor. We take a lot of arguments because
// the sensor needs all of them. The bool at the end is whether to
// retry, which we need because the hardware is flaky.
pub fn read_sensor(
    id: u32,
    address: u8,
    scale: f32,
    offset: f32,
    retry: bool,
    timeout_ms: u32,
) -> Result<f32, String> {
    if !retry && timeout_ms == 0 {
        return Err("bad timeout".to_string());
    }
    Ok((f32::from(address) * scale) + offset + id as f32)
}

pub fn connect(s: Sensor) -> Result<Sensor, Box<dyn std::error::Error>> {
    // We have to take ownership here because of borrow checker issues.
    let mut s = s;
    s.connected = true;
    Ok(s)
}

pub fn calibrate(s: &Sensor, raw: f32) -> f32 {
    raw * s.scale + s.offset
}
```

Seeded flaws, in order of what they test:
1. `read_sensor` — six arguments, a `bool` parameter, primitive obsession on `address`/`id`, stringly-typed error.
2. The block comment — an explanatory `why` comment standing in for a design fix.
3. `connect` — `Box<dyn Error>` in a public signature, takes ownership needlessly, and a comment blaming the borrow checker.
4. Every public item is undocumented — no `///` anywhere, no `#![warn(missing_docs)]`.
5. `Sensor` — all fields `pub`, no `#[non_exhaustive]`, no invariants.
6. `calibrate` — no `#[must_use]` on a pure function.
7. Features `std` and `logging` are declared but unused, so the feature powerset is vacuous.

- [ ] **Step 3: Verify it compiles**

```bash
cd tests/fixtures/sensor_kit && cargo +1.96.1 build
```

Expected: builds with warnings but no errors. If it errors, fix the fixture — the flaws must be design flaws, not compile failures.

- [ ] **Step 4: Confirm the flaws are visible to clippy**

```bash
cd tests/fixtures/sensor_kit && cargo +1.96.1 clippy --all-targets -- -W clippy::pedantic 2>&1 | grep -c warning
```

Expected: a non-zero count including at least `too_many_arguments`, `fn_params_excessive_bools`, and `must_use_candidate`. If any of those three are absent, adjust the fixture until they appear — Task 5 depends on them.

- [ ] **Step 5: Write `tests/fixtures/README.md`**

State that this crate is a test fixture, is deliberately bad, and must never be improved. List the seven seeded flaws from Step 2 verbatim, each with the skill that should catch it.

- [ ] **Step 6: Commit**

```bash
git add tests/fixtures
git commit -m "$(printf 'Add sensor_kit fixture crate with seeded design flaws\n\nCo-Authored-By: Claude Opus 5 <noreply@anthropic.com>')"
```

---

### Task 3: rust-principles

The foundation skill. Short, declarative, no process. Every other skill links here.

**Files:**
- Create: `skills/rust-principles/SKILL.md`

**Interfaces:**
- Consumes: the spec's "Guiding Principles" section.
- Produces: `skills/rust-principles/SKILL.md`, linked from Tasks 4, 5, 6, and 7 as `[rust-principles](../rust-principles/SKILL.md)`.

- [ ] **Step 1: Write the baseline scenario (RED)**

Create the scenario file `tests/scenarios/principles-baseline.md` containing this exact prompt:

> Here is a Rust function. Add a helper that converts a raw ADC count to millivolts, and make the code easier for the next reader to follow.
>
> ```rust
> pub fn read(raw: u16) -> u16 { raw }
> ```

- [ ] **Step 2: Run the baseline against a subagent with no skill, and record it**

Dispatch a general-purpose subagent with that prompt and nothing else. Record its full output in `tests/scenarios/principles-baseline.md` under a `## Baseline (RED)` heading.

Expected failures — the skill exists to fix these, so confirm they actually occur: explanatory comments on self-evident code, a trait or generic introduced for one implementor, a doc comment explaining *why* rather than the contract. **If the agent does none of these, the skill is not needed as written — report that before continuing.**

- [ ] **Step 3: Write the skill**

`skills/rust-principles/SKILL.md`. Frontmatter exactly:

```yaml
---
name: rust-principles
description: Use when writing, reviewing, or refactoring any Rust code, before deciding on abstractions, comments, or API shape
---
```

Body sections:
- `## Overview` — one sentence: these are the standing principles for Rust in this codebase; other Rust skills build on them.
- `## The Principles` — all five from the spec, verbatim, each with its explanation. Principle 1 must state that the metric is cognitive load, not line count, and that a newtype retiring a class of misuse is a net reduction. Principle 4 must state explicitly that it overrides the common "comments say why, not what" advice.
- `## Rustdoc Is Not Exempt From #3, And Not Governed By #4` — the spec's paragraph on why public docs are a contract, not an apology for bad design.
- `## Applying Principle 5` — the newtype as the canonical tension: it makes misuse hard and is a real ergonomic papercut at every call site. Judged per API.
- `## Common Mistakes` — a table with at least these rows, each with the rationalization on the left and the correction on the right: "This comment explains a subtlety" → the subtlety belongs in the design; "A trait makes this extensible" → no trait until a second implementor exists; "Fewer lines is simpler" → fewer things to reason about is simpler, which is not the same; "A newtype is overkill here" → ask what the caller must remember without it.

- [ ] **Step 4: Re-run the scenario with the skill loaded (GREEN)**

Dispatch a fresh subagent with the same prompt, with `skills/rust-principles/SKILL.md` contents prepended. Record output under `## With skill (GREEN)`.

Expected: no explanatory comments, no trait, no premature generic, and either a newtype for millivolts with its tradeoff named, or a stated reason for not using one.

- [ ] **Step 5: Close loopholes (REFACTOR)**

Any rationalization the agent used that the skill did not anticipate gets a row in `## Common Mistakes`. Re-run Step 4 once after amending. Record the result.

- [ ] **Step 6: Commit**

```bash
git add skills/rust-principles tests/scenarios/principles-baseline.md
git commit -m "$(printf 'Add rust-principles skill\n\nCo-Authored-By: Claude Opus 5 <noreply@anthropic.com>')"
```

---

### Task 4: rust-api-design

**Files:**
- Create: `skills/rust-api-design/SKILL.md`
- Create: `tests/scenarios/api-design-baseline.md`

**Interfaces:**
- Consumes: `[rust-principles](../rust-principles/SKILL.md)` and the fixture crate from Task 2.
- Produces: `skills/rust-api-design/SKILL.md`, referenced by Task 6's review checklist for the doc contract and error rules.

- [ ] **Step 1: Write the baseline scenario (RED)**

`tests/scenarios/api-design-baseline.md` with this exact prompt:

> Add a public `configure` function to this crate that sets a sensor's sample rate in hertz and its gain, and returns an error if either is out of range. Sample rate must be 1–1000 Hz; gain must be 1, 2, 4, or 8.
>
> [contents of `tests/fixtures/sensor_kit/src/lib.rs`]

- [ ] **Step 2: Run the baseline and record it**

Dispatch a subagent with the prompt alone. Record under `## Baseline (RED)`.

Expected failures: `u32`/`f32` primitives instead of types that make the ranges unrepresentable, a stringly-typed or `Box<dyn Error>` error, undocumented public items, doc comments without intra-doc links, no `#[must_use]` or `#[non_exhaustive]`. **If these do not occur, report it before writing the skill.**

- [ ] **Step 3: Write the skill**

`skills/rust-api-design/SKILL.md`. Frontmatter exactly:

```yaml
---
name: rust-api-design
description: Use when writing or changing any public Rust API, adding a function, type, trait, or error to a crate's surface, or writing rustdoc
---
```

Body sections, all content drawn from the spec's "Skill: rust-api-design":
- `## Overview` — one sentence, plus the link to `[rust-principles](../rust-principles/SKILL.md)`.
- `## Type-Driven Design` — parse don't validate; newtypes where they retire caller thinking, not reflexively; typestate for protocols; concrete before generic; no trait until a second implementor exists. Each with a short before/after Rust block.
- `## Surface Hygiene` — `#[non_exhaustive]`, `#[must_use]`, borrow over own, minimal `pub`. A quick-reference table.
- `## Error Design` — the error type is designed with the rigor of the success type; every variant actionable and carrying context; indistinguishable variants merged; `thiserror` at crate boundaries with `#[non_exhaustive]`; `anyhow`/`eyre` in binaries only; never `Box<dyn Error>` in a public signature; granularity scales with library scope.
- `## The Rustdoc Contract` — every public item documented; terse, usually one sentence, longer only where usage is nuanced; every argument documented; **intra-doc links on every type reference**, so each fact is written once and the docs are explorable; examples where usage is nuanced, compiled as doctests. Include one full worked example showing the link syntax on arguments and return types.
- `## Common Mistakes` — at minimum: reaching for `String` errors; a `bool` parameter where an enum belongs; documenting *what the code does* instead of the contract; omitting links because the doc comment gets longer.

- [ ] **Step 4: Re-run the scenario with the skill (GREEN)**

Same prompt, skill prepended. Record under `## With skill (GREEN)`.

Expected: a `SampleRate` (or equivalent) constructed fallibly, a `Gain` enum rather than a number, a `thiserror` enum with actionable variants, full rustdoc with intra-doc links on every type reference.

- [ ] **Step 5: Verify the produced code actually compiles**

Paste the agent's output into a scratch copy of the fixture crate (do not modify `tests/fixtures/sensor_kit` itself) and run:

```bash
cargo +1.96.1 build && cargo +1.96.1 test --doc
```

Expected: builds, and any `# Examples` block passes as a doctest. A skill that produces non-compiling examples is a failed skill — amend and re-run.

- [ ] **Step 6: Close loopholes and commit**

Add a `## Common Mistakes` row for any unanticipated rationalization, re-run Step 4 once, then:

```bash
git add skills/rust-api-design tests/scenarios/api-design-baseline.md
git commit -m "$(printf 'Add rust-api-design skill\n\nCo-Authored-By: Claude Opus 5 <noreply@anthropic.com>')"
```

---

### Task 5: rust-quality-gate

**Files:**
- Create: `skills/rust-quality-gate/SKILL.md`
- Create: `tests/scenarios/quality-gate-baseline.md`

**Interfaces:**
- Consumes: the fixture crate from Task 2; `cargo-hack` from Task 1.
- Produces: `skills/rust-quality-gate/SKILL.md`. Task 6 invokes this skill by name as its first step and consumes its output as evidence, so the "Reporting" section below defines the contract between the two.

- [ ] **Step 1: Write the baseline scenario (RED)**

`tests/scenarios/quality-gate-baseline.md` with this exact prompt:

> Run clippy on the crate at `tests/fixtures/sensor_kit` and deal with what it finds.

- [ ] **Step 2: Run the baseline and record it**

Expected failures: the agent runs `clippy --fix`, or mechanically silences lints with `#[allow]`, or fixes each warning locally without noticing that `too_many_arguments` plus `fn_params_excessive_bools` plus the primitive parameters are one finding — a bad boundary on `read_sensor`. Record verbatim.

- [ ] **Step 3: Write the skill**

`skills/rust-quality-gate/SKILL.md`. Frontmatter exactly:

```yaml
---
name: rust-quality-gate
description: Use before reviewing Rust code, before merging a Rust change, or when asked to run lints, tests, or checks on a Rust crate
---
```

Body sections:
- `## Overview` — the gate's output is evidence for a design review, not a task list. Link to `[rust-principles](../rust-principles/SKILL.md)`.
- `## The Suite` — the five commands verbatim from the spec, in order, as one copy-pasteable block. Note the `+1.96.1` pin and the broken `stable` toolchain from Global Constraints.

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -W clippy::pedantic
cargo test
cargo hack check --feature-powerset --no-dev-deps
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo doc --no-deps
```

- `## A Pedantic Hit Is A Symptom, Not A Task` — the core rule. Never run `clippy --fix` mechanically. Never `#[allow]` before asking what structural fact produced the lint. Clusters matter more than individuals.
- `## Reading Clusters` — a table mapping lint clusters to the structural cause they usually name: repeated `needless_pass_by_value` → ownership taken where a borrow would do, usually one type; `too_many_arguments` + `fn_params_excessive_bools` → a missing parameter struct or a function doing two jobs; widespread `must_use_candidate` → the crate has not decided which returns are meaningful; `missing_errors_doc` / `missing_panics_doc` → the error design was never written down.
- `## Reporting` — the output contract Task 6 consumes: for each cluster, the lint names, the count, the files, and the suspected structural cause stated as a question about the design. Never a fix list. Missing tooling (`cargo-hack` absent, `stable` broken) is reported explicitly, never silently skipped.
- `## Common Mistakes` — `--fix` as a first move; blanket `#![allow(clippy::pedantic)]`; reporting 40 individual warnings instead of 4 clusters; skipping the rustdoc command because the crate "has no docs yet" (that is precisely the finding).

- [ ] **Step 4: Run the full suite against the fixture and record real output**

```bash
cd tests/fixtures/sensor_kit
cargo +1.96.1 fmt --check; cargo +1.96.1 clippy --all-targets --all-features -- -W clippy::pedantic
cargo +1.96.1 test; cargo +1.96.1 hack check --feature-powerset --no-dev-deps
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo +1.96.1 doc --no-deps
```

Confirm each command runs. Any command that errors for a reason other than finding a real flaw is a bug in the suite — fix the command in the skill.

- [ ] **Step 5: Re-run the scenario with the skill (GREEN)**

Expected: the agent runs the whole suite, reports clusters with suspected structural causes, and proposes no mechanical fixes.

- [ ] **Step 6: Close loopholes and commit**

```bash
git add skills/rust-quality-gate tests/scenarios/quality-gate-baseline.md
git commit -m "$(printf 'Add rust-quality-gate skill\n\nCo-Authored-By: Claude Opus 5 <noreply@anthropic.com>')"
```

---

### Task 6: rust-design-review

**Files:**
- Create: `skills/rust-design-review/SKILL.md`
- Create: `tests/scenarios/design-review-baseline.md`

**Interfaces:**
- Consumes: `rust-quality-gate` (invoked as its first step), `rust-api-design` (for the doc and error criteria), `rust-principles`, and the fixture crate.
- Produces: the final skill. Nothing consumes it.

- [ ] **Step 1: Write the baseline scenario (RED)**

`tests/scenarios/design-review-baseline.md` with this exact prompt:

> Review the crate at `tests/fixtures/sensor_kit`.

- [ ] **Step 2: Run the baseline and record it**

Expected failures: the agent produces a bug-hunt or a numbered lint list rather than a design critique; it does not run the gate first; it reports findings in file order rather than by significance; it misses that the `Sensor` struct's public fields make every invariant unenforceable.

- [ ] **Step 3: Write the skill**

`skills/rust-design-review/SKILL.md`. Frontmatter exactly:

```yaml
---
name: rust-design-review
description: Use when asked to review a Rust crate, module, or public API, or before merging a Rust change that alters a public surface
---
```

Body sections:
- `## Overview` — this is a design review, deliberately not a bug hunt. Correctness bugs belong to `/code-review`. This reviews whether the API is ergonomic, hard to misuse, and minimal to reason about. Link to `[rust-principles](../rust-principles/SKILL.md)`.
- `## Before You Read Any Code` — run `rust-quality-gate` and keep its cluster report at hand.
- `## Review Order` — the five numbered passes from the spec, each with the question it asks:
  1. **Public surface** — what must a caller learn to use this? What can they get wrong that the types could have prevented?
  2. **Error design** — is every variant actionable and distinguishable? Does any variant force the caller to parse a string?
  3. **Docs** — the contract from `rust-api-design`, link discipline included.
  4. **Cognitive load** — what must a reader hold in their head? Which abstraction only moves complexity around?
  5. **Gate evidence** — read each cluster as a structural finding.
- `## Output` — prose at design altitude, most significant finding first. Each finding names the design fact, the cost to the caller, and the change that would retire it. Explicitly **not** a linter's list, not one bullet per lint, not file-ordered.
- `## Out Of Scope` — correctness bugs, performance, unsafe review, style already covered by `fmt`. Say so and move on rather than drifting into them.
- `## Common Mistakes` — grading on a curve because the code compiles; listing findings in file order; restating clippy output as findings; proposing a refactor without naming what the caller stops having to think about.

- [ ] **Step 4: Re-run the scenario with the skill (GREEN)**

Expected: the gate runs first; output is prose; the leading finding is about `Sensor`'s public fields or `read_sensor`'s parameter list, not about a missing `#[must_use]`.

- [ ] **Step 5: Close loopholes and commit**

```bash
git add skills/rust-design-review tests/scenarios/design-review-baseline.md
git commit -m "$(printf 'Add rust-design-review skill\n\nCo-Authored-By: Claude Opus 5 <noreply@anthropic.com>')"
```

---

### Task 7: rust-embedded overlay and end-to-end validation

**Files:**
- Create: `skills/rust-embedded/SKILL.md`
- Create: `tests/scenarios/embedded-baseline.md`
- Modify: `README.md`

**Interfaces:**
- Consumes: all four prior skills.
- Produces: the complete plugin.

- [ ] **Step 1: Write the baseline scenario (RED)**

`tests/scenarios/embedded-baseline.md` with this exact prompt:

> Add a function to this `#![no_std]` driver that reads a burst of samples from the device and returns them to the caller.
>
> ```rust
> #![no_std]
> pub struct Device;
> impl Device {
>     pub fn read_one(&mut self) -> u16 { 0 }
> }
> ```

- [ ] **Step 2: Run the baseline and record it**

Expected failures: returning a `Vec`, reaching for `alloc`, `unwrap` in library code, `core::fmt` for diagnostics, no capacity bound in the signature.

- [ ] **Step 3: Write the skill**

`skills/rust-embedded/SKILL.md`. Frontmatter exactly:

```yaml
---
name: rust-embedded
description: Use when working on a no_std Rust crate, an embedded target, a hardware driver, or a crate that gates std behind a feature
---
```

Deliberately thin — only the deltas from the hosted rules. Body:
- `## Overview` — an overlay, used alongside `[rust-api-design](../rust-api-design/SKILL.md)` and `[rust-design-review](../rust-design-review/SKILL.md)`, never instead of them. Everything in those still applies.
- `## The Deltas` — the spec's six bullets: no allocation in library APIs; const generics and fixed capacity over growable buffers; `defmt` over `core::fmt`; panic-free library code with fallible construction over `unwrap`; interrupt-safety expressed in the type signature; `std` behind a feature gate with the matrix actually checked. Each with a one-line before/after.
- `## Gate Changes` — `cargo hack` matters more here because `std` gating is real; add the target to the check.
- `## Common Mistakes` — pulling in `alloc` to return a slice; `unwrap` on a `Result` that "cannot fail"; formatting with `core::fmt` in a hot or size-constrained path.

- [ ] **Step 4: Re-run the scenario with the skill (GREEN)**

Expected: a caller-provided `&mut [u16]` buffer or a const-generic array, a fallible return, no allocation.

- [ ] **Step 5: End-to-end validation across all five skills**

Reinstall the plugin and confirm all five skills load:

```bash
python3 -c "
import pathlib,re
for p in sorted(pathlib.Path('skills').glob('*/SKILL.md')):
    t=p.read_text(); m=re.match(r'---\n(.*?)\n---', t, re.S)
    assert m, f'{p}: no frontmatter'
    assert 'name:' in m.group(1) and 'description:' in m.group(1), f'{p}: missing field'
    assert len(m.group(1))<1024, f'{p}: frontmatter too long'
    assert p.parent.name in m.group(1), f'{p}: name does not match directory'
    print(p.parent.name, 'ok')
"
```

Expected: five `ok` lines.

- [ ] **Step 6: Verify every cross-skill link resolves**

```bash
python3 -c "
import pathlib,re
bad=[]
for p in pathlib.Path('skills').glob('*/SKILL.md'):
    for link in re.findall(r'\]\((\.\./[^)]+)\)', p.read_text()):
        if not (p.parent/link).resolve().exists(): bad.append((str(p),link))
print('broken:',bad or 'none')
"
```

Expected: `broken: none`. This is the same discipline the rustdoc contract demands — a link that does not resolve is a fact written in the wrong place.

- [ ] **Step 7: Full-stack run against the fixture**

With the plugin installed, ask a fresh session: *"Review the crate at tests/fixtures/sensor_kit."* Confirm it invokes `rust-design-review`, which invokes `rust-quality-gate`, and produces design-altitude prose. Record the transcript in `tests/scenarios/end-to-end.md`.

- [ ] **Step 8: Finalize `README.md` and commit**

Update the skill list with final descriptions and add a "Testing" section describing the scenario files and how to re-run them.

```bash
git add skills/rust-embedded tests/scenarios README.md
git commit -m "$(printf 'Add rust-embedded overlay and end-to-end validation\n\nCo-Authored-By: Claude Opus 5 <noreply@anthropic.com>')"
```

- [ ] **Step 9: Push**

```bash
git push -u origin main
```
