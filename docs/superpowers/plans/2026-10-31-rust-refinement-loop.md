# rust-refinement-loop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a seventh rust-craft skill, `rust-refinement-loop`, that runs design review, adversarial correctness review, and simplify over a Rust branch in cycles, applies the clear changes, holds the trade-offs for the user at break points every fifth cycle, and stops at the first clean cycle.

**Architecture:** One new `skills/rust-refinement-loop/SKILL.md` in the form of the other rust-craft skills. It is an orchestrator: each stage's reviewer is a fresh subagent; the orchestrator sorts findings, applies and commits clear changes one per commit, keeps the branch's spec and plan true, and holds pending trade-offs in notes outside the repo. It is tested against a scratch git branch built by a setup script from `sensor_kit` plus a seeded change and a short spec.

**Tech Stack:** Markdown skills with YAML frontmatter; a POSIX shell setup script; git; the `sensor_kit` Rust fixture crate; clean-room subagents for RED/GREEN runs.

**Spec:** `docs/superpowers/specs/2026-10-31-rust-refinement-loop-design.md`. Read it before starting; section numbers below refer to it.

## Global Constraints

- Skill name, exactly: `rust-refinement-loop`. Directory `skills/rust-refinement-loop/`, one file `SKILL.md`.
- Frontmatter has exactly two fields, `name` and `description`, under 1024 characters total; the description states triggering conditions only, in the `Use when ...` form.
- Cite other skills by relative link and never restate them. The cost screen and the decisions-entry format belong to `[rust-spec-review](../rust-spec-review/SKILL.md)`; this skill links to them.
- Break points every fifth cycle and at the end. A cycle is clean when stage 1 applies nothing and adds nothing new to the trade-off list and stage 2 confirms nothing; simplify never counts.
- The loop never pushes, never force-pushes, and never rewrites history.
- `code-review` is used at `high` at most; never `max` or `ultra`.
- Fixtures under `tests/fixtures/` are never improved. `tests/fixtures/sensor_kit` is never modified; the scratch branch is built from a copy.
- Scenarios under `tests/scenarios/` are prompt-only.
- Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, or the attribution the implementer's harness specifies.

## Review Focus

Failure modes the spec implies that no task's happy path would exercise, most likely first, each pinned to a task.

1. **A crate that fails the quality gate before cycle 1.** `sensor_kit` has no docs and carries clippy clusters. Expected: a baseline is recorded and only new failures block. Pinned to Task 3's GREEN.
2. **A pending trade-off raised again by every fresh reviewer.** Expected: matched to the list, not new, and the loop still ends clean. Pinned to Task 3's GREEN (the newtype seed).
3. **A mixed branch.** A non-Rust file in the change. Expected: left alone and named once in the message. Pinned to Task 1's seed list and Task 3's GREEN.
4. **A fix that cannot be made green.** Expected: discarded uncommitted, listed as "could not be applied green", loop continues. Pinned to Task 3's prompt checks.
5. **An ambiguous branch base** (no `main`, upstream named otherwise). Expected: the orchestrator asks. Pinned to Task 3's prompt checks.

---

### Task 1: The scratch-branch fixture

**Files:**
- Create: `tests/fixtures/refinement_branch/setup.sh`
- Create: `tests/fixtures/refinement_branch/change/` (the files the branch commits, overlaid on a copy of `sensor_kit`)
- Create: `tests/fixtures/refinement_branch/spec.md`
- Create: `tests/scenarios/refinement-loop-baseline.md`
- Modify: `tests/fixtures/README.md` (append a section)

**Interfaces:**
- Consumes: `tests/fixtures/sensor_kit` (read-only).
- Produces: `setup.sh <target-dir> [--no-spec]`, which creates a fresh git repo at `<target-dir>` with a base commit on `main` holding a copy of `sensor_kit`, then a branch `feat/sensor-sampling` with two or three commits adding the change and, unless `--no-spec`, the spec at `docs/superpowers/specs/sampling-design.md`. It refuses a non-empty target. Later tasks run it into the scratchpad. The seed numbering below is what Task 3's checks refer to.

- [ ] **Step 1: Write the change and the spec**

The change adds a small, plausible feature to `sensor_kit` (a periodic sampling API, say) and plants exactly these, each in code that otherwise reads as competent:

1. **Clear structural win.** A new `pub` function taking two `bool` flags whose combination has one illegal case, where an enum retires the illegal case at no caller cost.
2. **Questionable trade-off.** A new public signature with two confusable `u32`s (a period and a timeout) where newtypes would cost a conversion at every caller.
3. **Real bug.** An off-by-one or overflow with a concrete failing input, e.g. a sample count that wraps at a boundary. No existing test covers it.
4. **False bug.** Code that looks wrong but is correct (a deliberate saturating operation, or a branch guarded earlier) so a reviewer who does not trace it would "fix" it.
5. **Simplify-only tidy.** A needless clone or a verbose match that only simplify would change.
6. **A recorded decision the diff respects.** The spec's `## Design decisions` section has one entry in `rust-spec-review`'s format (read its `## The Loop` section for the format) stating a shape a naive reviewer would challenge, with a reason; the code follows it.
7. **A departure.** The spec states one shape (e.g. a function returns `Result<_, SampleError>`) that the code does differently (returns `Option`).
8. **A non-Rust file.** A small shell script or markdown note in the change, outside the crate.

The spec is about a page, superpowers design-doc form, naming the shapes of seeds 1, 2, 6, and 7. Add one test that passes for the change's happy path, so the branch is green at the start apart from `sensor_kit`'s pre-existing gate failures.

- [ ] **Step 2: Write `setup.sh`**

POSIX shell, `set -eu`, no network. It copies `sensor_kit` (excluding `target/`), commits it on `main` as the base, branches, overlays `change/`, and commits in two or three commits a real branch would have (feature, test, spec). With `--no-spec` it omits the spec commit.

- [ ] **Step 3: Verify the fixture**

Run into a fresh scratchpad directory, then `cargo test` and `cargo fmt --check` in the crate.
Expected: setup succeeds; tests pass; fmt passes; `git log --oneline main..feat/sensor-sampling` shows the branch commits; seed 3's failing input, run by hand in a scratch test, fails. Run once more with `--no-spec` and confirm no spec file. Run into a non-empty directory and confirm it refuses.

- [ ] **Step 4: Scenario and README**

`tests/scenarios/refinement-loop-baseline.md`:

```markdown
# Scenario: refinement-loop-baseline

Run `tests/fixtures/refinement_branch/setup.sh <dir>`. The branch
`feat/sensor-sampling` in `<dir>` is feature-complete. Get it ready for a
PR.
```

README section: what the fixture is, how to build it, the eight seeds numbered with what should happen to each (1 applied, 2 listed, 3 fixed with a test, 4 dropped unconfirmed, 5 simplified, 6 not re-raised, 7 fixed to match the spec, 8 named once and untouched), and that the never-improve rule applies.

- [ ] **Step 5: RED run**

A clean-room subagent with only the scenario prompt (the directory built for it). Record which seeds it handles and how: whether it "fixes" seed 4, applies seed 2 without asking, touches seed 8, or stops after one pass.

- [ ] **Step 6: Commit**

```bash
git add tests/fixtures/refinement_branch tests/scenarios/refinement-loop-baseline.md tests/fixtures/README.md
git commit -m "Fixture: a seeded scratch branch for rust-refinement-loop"
```

---

### Task 2: The cycle

The skill's first half: what it reads, the three stages, the sort, green, commits, and keeping the spec true. Verified by prompt checks on the sort, not an end-to-end run.

**Files:**
- Create: `skills/rust-refinement-loop/SKILL.md`

**Interfaces:**
- Consumes: Task 1's seed numbering (for checks only; no seed wording goes into the skill).
- Produces: sections `## Overview`, `## What It Reads`, `## One Cycle` (with `### Stage 1: Design`, `### Stage 2: Correctness`, `### Stage 3: Simplify`, `### Green And Commits`), `## Keeping The Spec True`, `## Common Mistakes`. Task 3 inserts `## Stopping`, `## Break Points`, and `## Applying The Answers` before `## Common Mistakes`.

- [ ] **Step 1: Frontmatter and Overview** from spec sections 1 and 3; trigger on a feature-complete Rust branch before a PR, or a direct request to refine, polish, or run the review loop on a branch.

- [ ] **Step 2: What It Reads** from spec section 3: branch diff against base (the base found from `main` or the upstream, asking when neither is clear), the spec and plan, the sibling skills by absolute path from this skill's base directory, the no-spec case.

- [ ] **Step 3: One Cycle** from spec section 4: fresh subagent per stage, never the previous cycle's findings; stage 1's Clear, Questionable, and Decided bins with the cost screen linked, the departure and repair rules, the non-obligation findings, and the closing-aside handoff to stage 2; stage 2's confirmation rule (a failing test the reviewer wrote, or a trace the orchestrator checks), `code-review` at `high` as an acceptable engine; stage 3's scope (whole branch diff in cycle 1, the cycle's changes after); the engine fallback; green as "tests and fmt pass, clippy and docs nothing new against the baseline recorded before cycle 1"; check green before commit; two attempts then discard uncommitted and list "could not be applied green"; one commit per change naming finding and stage.

- [ ] **Step 4: Keeping The Spec True** from spec section 5's paragraph of that name: a Clear change that alters a named shape updates the spec and plan text in the same commit and adds an **Adopted** entry; entries use `rust-spec-review`'s format, round numbers continuing from the highest in the section, one round per break point; the no-spec listing.

- [ ] **Step 5: First Common Mistakes rows.** "The reviewer said it's a bug, so I fixed it"; "the newtype is obviously right, I applied it"; "the gate was red before I started, so I can't continue"; "I committed and then ran the tests"; "I squashed the fixes to keep history tidy"; "the code departs from the spec, so I updated the spec to match the code".

- [ ] **Step 6: Prompt checks.** One subagent each, given the skill and asked which bin and why, quoting the text:
  - A finding proposing newtypes for two `u32`s in a public signature, cost "one conversion in one place". Expected: Questionable.
  - A finding that a `pub` function's two bools admit an illegal combination, remedy an enum, cost "none I can name". Expected: Clear, applied, with an Adopted entry if the spec names the function.
  - A design-review finding that the code returns `Option` where the spec's decided shape returns `Result`. Expected: Clear, code changed to match the spec.
  - A correctness finding with no failing test and a trace that does not hold against the code. Expected: dropped and counted.

Record the answers; fix the skill prose for any miss.

- [ ] **Step 7: Commit**

```bash
git add skills/rust-refinement-loop/SKILL.md
git commit -m "rust-refinement-loop: one cycle, the sort, green, and keeping the spec true"
```

---

### Task 3: Stopping, break points, and the end-to-end run

**Files:**
- Modify: `skills/rust-refinement-loop/SKILL.md` (insert three sections before `## Common Mistakes`; extend it)
- Create: `tests/scenarios/refinement-loop-break-point.md`

**Interfaces:**
- Consumes: Task 2's sections; Task 1's `setup.sh` and seeds.
- Produces: nothing later tasks consume beyond the finished skill.

- [ ] **Step 1: Stopping** from spec section 5: the clean-cycle definition; re-raised pending items are matched, not new; pending items live in the orchestrator's notes outside the repo; the loop ends at the first clean cycle; no hard cap, but two consecutive batches without a clean cycle are named plainly with a recommendation to stop.

- [ ] **Step 2: Break Points**: every fifth cycle and at the end; the four-part message in order (one line; the trade-off list most consequential first with change, cost, retires; applied changes grouped by stage with commit ids; dropped counts and untouched non-Rust files) and nothing else; the no-spec listing of what a spec would have recorded; the summary naming any engine fallback or non-independent stages.

- [ ] **Step 3: Applying The Answers**: apply or decline each; record **Adopted with a cost** with the accepted cost or **Declined** with the user's reason; the next batch if paused; one more cycle if the loop had ended clean and a change was approved; "stop here"; the history skill may run at a break point.

- [ ] **Step 4: Common Mistakes rows.** "The simplify pass changed something, so the cycle isn't clean"; "the reviewer raised the newtype again, so it's a new item"; "five cycles ran, I'll keep going"; "the user approved it, so the loop is done"; "I'll put the pending trade-offs in the spec so the next reviewer sees them"; "I'll summarise each cycle for the user".

- [ ] **Step 5: Break-point scenario.** `tests/scenarios/refinement-loop-break-point.md`, prompt-only: the loop has run five cycles, cycle 5 confirmed one bug, two trade-offs are pending; what happens next?

- [ ] **Step 6: Prompt checks.** One subagent each with the skill:
  - The break-point scenario. Expected: stops, the four-part message, pauses.
  - Review Focus 4: a confirmed bug fix that fails tests twice. Expected: discarded uncommitted, listed "could not be applied green", loop continues.
  - Review Focus 5: a repo with no `main` and an upstream named `trunk`. Expected: asks the user for the base.

- [ ] **Step 7: End-to-end GREEN.** Build the fixture into a fresh scratchpad directory. One subagent told to read the skill and run it on the branch with the baseline scenario prompt. Then check against the seeds:
  - 1 applied in its own commit, spec text and an Adopted entry updated in the same commit;
  - 2 on the trade-off list, not applied, not duplicated across cycles;
  - 3 fixed, with a failing-then-passing test in the same commit;
  - 4 not changed, counted as dropped;
  - 5 simplified;
  - 6 not re-raised;
  - 7 code changed to match the spec;
  - 8 untouched and named once;
  - the loop ended clean within the first batch; the message has four parts; `sensor_kit`'s pre-existing gate failures did not stop it (Review Focus 1); no push, no history rewrite (`git reflog` shows only commits and checkouts).

  Then the no-spec run on a fresh `--no-spec` build: the loop runs and the message lists what a spec would have recorded. Fix skill prose, never the fixture, for any miss. Record both runs' key lines.

- [ ] **Step 8: Commit**

```bash
git add skills/rust-refinement-loop/SKILL.md tests/scenarios/refinement-loop-break-point.md
git commit -m "rust-refinement-loop: stopping, break points, and applying the answers"
```

---

### Task 4: README

**Files:**
- Modify: `README.md` (Skills list, Testing section)

- [ ] **Step 1:** Add a `rust-refinement-loop` bullet after `rust-spec-review`: runs the design review, an adversarial correctness review, and simplify over a branch in cycles before a PR, applies the clear changes, brings trade-offs to the user every five cycles, and stops at the first clean cycle. In Testing, name the `refinement_branch` fixture and its setup script and the two scenarios.

- [ ] **Step 2: Verify:** `ls skills/*/SKILL.md | wc -l` prints `7`; `head -4 skills/rust-refinement-loop/SKILL.md` shows two frontmatter fields.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "README: rust-refinement-loop"
```
