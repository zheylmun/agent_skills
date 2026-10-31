# rust-spec-review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a sixth rust-craft skill, `rust-spec-review`, that runs the design review against a spec before any code exists, loops autonomously until the design converges, and records every ergonomics-versus-correctness decision in the spec.

**Architecture:** One new `skills/rust-spec-review/SKILL.md` in the same form as `rust-design-review`: prose passes, a finding shape, an output contract, a Common Mistakes table, and relative links to the other skills rather than restatement. The skill is an orchestrator: each round is a fresh subagent that reads the spec, the sibling SKILL.md files by absolute path, and any existing crate the spec touches; the orchestrator sorts findings into three bins, edits the spec in place, and appends a `## Design decisions` section that is the only memory across rounds. `rust-design-review` gains one rule so it reads that section at PR time. Skills are documentation, so each task is built RED/GREEN: a seeded fixture spec, a prompt-only scenario, a clean-room run without the skill to see the failure, then with it to see the catch.

**Tech Stack:** Markdown skills with YAML frontmatter; markdown fixtures; clean-room subagents for RED/GREEN runs. No Rust toolchain is needed for any task in this plan.

**Spec:** `docs/superpowers/specs/2026-10-31-rust-spec-review-design.md`. Read it before starting; every section of the skill is derived from it, and section numbers below refer to it.

## Global Constraints

- Skill name, exactly: `rust-spec-review`. Directory `skills/rust-spec-review/`, one file `SKILL.md`.
- Frontmatter has exactly two fields, `name` and `description`, under 1024 characters total. **`description` states triggering conditions only**, never the workflow. Form: `Use when ...`, one line, matching the other five skills.
- Skills cite the other skills with relative links, `[rust-principles](../rust-principles/SKILL.md)`, and never restate a principle, the error-design criteria, or the rustdoc contract.
- The decisions section heading is the literal `## Design decisions`. The three bins are named exactly **Adopted**, **Adopted with a cost**, **Declined**. The round cap is five.
- The four-part finding is: design fact, cost to the caller, change to the spec that retires it, ergonomic cost of that change ("none I can name" is a legal value; "over-engineering" is not).
- Fixtures under `tests/fixtures/` are deliberately flawed and are never improved; `tests/fixtures/README.md` says so and lists every seeded flaw with the pass that catches it.
- Scenarios under `tests/scenarios/` are prompt-only, in the repo's newer style. The transcript of a run belongs in the task's report, not in the file.
- A clean-room RED run is a fresh general-purpose subagent given the fixture paths and the scenario prompt and nothing else. A GREEN run is the same agent given the SKILL.md path first, told to read it and follow it, then the prompt. GREEN agents may read the repo; the skill resolves sibling skills by path.
- Every task ends with a commit. Commit messages end with `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`.

## Review Focus

Failure modes the spec implies that no task's happy path exercises, most likely to bite first. Each has a check pinned to the task named.

1. **A spec with no Rust shapes.** A product brief or a frontend design doc. Expected: one paragraph saying there is nothing for the passes to bite on, no invented surface, no loop. Pinned to Task 3.
2. **A spec that already has a `## Design decisions` section** from a prior invocation. Expected: the loop resumes against it, appends new entries with the next round number, and never duplicates or re-argues an existing entry. Pinned to Task 3.
3. **The round cap.** A spec whose round-five findings are not all covered. Expected: the user message says the loop hit the cap and did not converge, never presents round five as converged. Pinned to Task 3.
4. **A finding against a "Left to the builder" entry.** Expected: declined with that reason, never promoted into a decision. Pinned to Task 2.
5. **No subagent tool.** Expected: the orchestrator runs rounds itself after re-reading the spec from disk and says in the report that rounds were not independent. Pinned to Task 3.

---

### Task 1: The fixture spec and its governing contract

A seeded design doc the rest of the plan reviews against, with one flaw per pass and one stated decision that must survive.

**Files:**
- Create: `tests/fixtures/enrolment_contract.md`
- Create: `tests/fixtures/enrolment_spec.md`
- Create: `tests/scenarios/spec-review-baseline.md`
- Modify: `tests/fixtures/README.md` (append a section)

**Interfaces:**
- Consumes: nothing.
- Produces: the two fixture files every later GREEN run reads, and the seeded-flaw list in the README that is the oracle for Task 2's GREEN check. Later tasks refer to the flaws by the numbers below.

- [ ] **Step 1: Write the governing contract**

`tests/fixtures/enrolment_contract.md`, about half a page, in the voice of a server team's API contract for a device enrolment flow. It must fix these facts, because the fixture spec's flaws depend on them:

- Three endpoints: `POST /api/devices/enrol`, `POST /api/devices/finalize`, `GET /api/devices/products`. Signed requests carry headers `Device-Id`, `Device-Counter`, `Device-Signature`.
- `finalize` answers exactly four ways, enumerated as closed: `200` finalized, `409` still pending, `410` denied, `404` expired. The contract says "this set is closed; a new outcome is a new contract version."
- The counter is a `u64` the device increments before each signed request; the server rejects a repeat with `401 replay` and a body carrying `last_counter`.
- The server does not validate device names beyond length 1 to 64.
- No field or endpoint for a "user code".

- [ ] **Step 2: Write the fixture spec with the seeded flaws**

`tests/fixtures/enrolment_spec.md`, about two pages, in the form of a superpowers design doc (Intent, Goals, sections with named Rust shapes, Left to the builder, Open with the user). Its first paragraph names `enrolment_contract.md` as the contract it answers to. Derived from the shapes of a device-enrolment client but naming no real product, repo, server, or company. Seed exactly these, each in the form stated:

1. **Pass 1, named surface.** `Client::new(base: &str)`, and `finalize(&self, key: &DeviceKey, pending_id: &str)` beside `Enrolled { pub device_id: String, ... }` in the same module, so a pending id and a device id are confusable `String`s.
2. **Pass 2, states.** `Record { pending: Option<PendingRecord>, device: Option<DeviceRecord> }` with the prose "exactly one of pending or device is present, or neither; a file with both prefers device".
3. **Pass 3, errors.** "A keychain item that is not 32 bytes is `StoreError::Corrupt` and is treated as no key."
4. **Pass 4, boundaries.** "Every write re-checks that the record still names the id being updated, and the mutex is never held across a network or keychain call."
5. **Pass 5, prose rules.** "The machine remembers `cancelled: Option<PendingId>`; `Begin` clears it, so a new attempt is never hijacked by the old one." (Note this one uses a newtype name in prose while flaw 1 uses `&str`; the inconsistency is deliberate and is itself evidence.)
6. **Pass 7, cognitive load.** `trait BrowserOpener { fn open(&self, url: &str) -> io::Result<()>; }` with one described implementor that shells out to `open`, no test double mentioned, and the sentence "so a future platform can supply its own".
7. **Pass 8, governing authority.** `#[non_exhaustive] pub enum FinalizeError { Pending, Denied, Expired, Rejected, Unreachable }` with the sentence "non-exhaustive so the server can add outcomes without breaking us".
8. **The decision that must survive pass 6.** Under a heading "Decisions": "The counter is persisted through a `CounterStore` trait (`fn reserve(&mut self) -> io::Result<u64>`), with a file store in the app and a memory store in the library's tests, rather than a typestate over the request. Both stores are real; the trait has two implementors from the first commit."
9. **Left to the builder** includes: "The exact shape of the tick timer that wakes the machine at its next deadline." Nothing in the spec body describes that timer, so a reviewer who proposes a timer type is reviewing a builder decision.

- [ ] **Step 3: Write the scenario prompt**

`tests/scenarios/spec-review-baseline.md`:

```markdown
# Scenario: spec-review-baseline

The design doc at `tests/fixtures/enrolment_spec.md` has been approved and
answers to the contract at `tests/fixtures/enrolment_contract.md`. Review
the design before the implementation plan is written.
```

- [ ] **Step 4: Append the fixture section to the README**

Under a new `## enrolment_spec.md and enrolment_contract.md` heading in `tests/fixtures/README.md`: one paragraph on what they are, then the nine seeded items from Step 2 as a numbered list, each with "Caught by: **rust-spec-review** pass N" (item 8: "Survives pass 6"; item 9: "Declined as left to the builder"). State that these are markdown fixtures, not crates, and that the prohibition on improving fixtures applies to them identically.

- [ ] **Step 5: RED run**

Dispatch a clean-room subagent with only the two fixture paths and the scenario prompt. Record in the task report which of the nine items it mentions, in what form (a nit list, a product question, a proposed implementation), and whether it flagged item 8. Expected: a mixed list of product questions and wording nits, at most three of items 1 to 7 named as shape problems, no three-part findings, and item 8 flagged or item 9 designed. If RED already catches six or more of the seven flaws as three-part findings, the fixture is too obvious: sharpen the prose so the flaw is stated as a rule rather than a code block, and rerun.

- [ ] **Step 6: Commit**

```bash
git add tests/fixtures/enrolment_contract.md tests/fixtures/enrolment_spec.md tests/scenarios/spec-review-baseline.md tests/fixtures/README.md
git commit -m "Fixture: a seeded enrolment spec and contract for rust-spec-review"
```

---

### Task 2: The skill's passes and finding shape

The reviewer half of the skill: what one round reads, the eight passes, and the four-part finding. Verified by a single-round GREEN run.

**Files:**
- Create: `skills/rust-spec-review/SKILL.md`

**Interfaces:**
- Consumes: Task 1's fixtures and seeded-flaw list.
- Produces: `SKILL.md` with frontmatter and the sections `## Overview`, `## What A Round Reads`, `## The Passes` (eight `###` subsections), `## Findings`, `## Out Of Scope`, and a `## Common Mistakes` table. Task 3 adds `## The Loop`, `## What The User Receives`, and `## Applying The Answers` between `## Findings` and `## Out Of Scope`; Task 4 adds `## Plan Mode` after `## Applying The Answers`. Section names are fixed here so the tasks do not collide.

- [ ] **Step 1: Write the frontmatter and Overview**

Frontmatter `name: rust-spec-review`. Description, trigger-only, covering: an approved design doc or spec for Rust work, before the implementation plan or any code; a direct request to review a spec, design doc, or plan for a Rust crate. Overview from spec section 1: the question behind every finding, the sentence-as-comment framing, and the relationship to `rust-design-review` (same method, moved earlier; PR-time review still runs). Cite `rust-principles`, `rust-api-design`, and `rust-design-review` by relative link.

- [ ] **Step 2: Write "What A Round Reads"**

From spec section 3, the four inputs. State the path rule: the orchestrator resolves the sibling `SKILL.md` files from this skill's own base directory and hands absolute paths to the reviewer, which reads them before the spec. State the no-Rust-shapes stop: one paragraph, no invented surface. State that `rust-quality-gate` is not run and why. State that when the spec touches an existing crate the reviewer reads the `pub` items of the modules the spec names.

- [ ] **Step 3: Write the eight passes**

One `###` per pass from spec section 4, in order: Named surface, States and transitions, Error design, Boundaries, Prose rules, Decisions audit, Cognitive load, Governing authority. Each opens with its italic question, as `rust-design-review` does. Pass 5 carries the word list ("must", "never", "always", "exactly one", "is treated as", "the caller re-checks", "it is safe to", "by construction") and the clause rule when nothing is found. Pass 6 carries the rule that a finding covered by a recorded decision is dropped unless the decision makes a stated goal unreachable, and the rule that a "Left to the builder" entry is never a finding's subject. Pass 8 carries "a remedy that cannot name what wants it is deleted, not ranked" and the not-applicable clause.

- [ ] **Step 4: Write "Findings"**

The four-part shape from spec section 5, the spec-section field, the ranking rule by reference to `rust-design-review`'s three criteria, and the round-output contract (ranked prose, a clause per empty pass, one closing out-of-scope paragraph, a paragraph when nothing is found).

- [ ] **Step 5: Write "Out Of Scope" and the first Common Mistakes rows**

Out of scope: product correctness and requirement completeness (brainstorming's builder check), non-Rust specs, PR-time review, bug hunting. One short closing paragraph, same budget as `rust-design-review`. Common Mistakes rows for: "the spec says it in prose, so the design handles it"; "that's a product decision, I'll leave it" when no reason is stated; "that decision is wrong" when a reason is stated and no goal breaks; "I'll suggest a timer type" against a builder item; "`#[non_exhaustive]` is good practice" against a closed contract set; "the spec is thin, I'll propose the surface".

- [ ] **Step 6: GREEN run, one round**

Dispatch a clean-room subagent told to read `skills/rust-spec-review/SKILL.md` and act as one round's reviewer, then the scenario prompt. Check the output against Task 1's list:

- Items 1 to 7 each appear as a four-part finding under the pass named, in the spec's vocabulary, with an ergonomic cost stated (item 1 must name a real cost for the id newtypes; item 2 must say "none I can name" or equivalent).
- Item 8 does not appear as a finding; the decisions-audit clause mentions it as covered.
- Item 9 does not appear; if the reviewer mentions the timer at all it is to say it is the builder's.
- Item 7's finding names the contract's "closed set" sentence.
- No finding has a lint name or a file-and-line as its subject; no finding proposes code.

Record the transcript's key lines in the task report. If an item is missed, fix the pass's prose (not the fixture) and rerun.

- [ ] **Step 7: Commit**

```bash
git add skills/rust-spec-review/SKILL.md
git commit -m "rust-spec-review: the passes and the finding shape"
```

---

### Task 3: The loop, the decisions section, and the user message

The orchestrator half: rounds, bins, editing the spec, convergence, the cap, the oscillation guard, the one message to the user, and applying the answers. Verified by an end-to-end run on a scratch copy of the fixture.

**Files:**
- Modify: `skills/rust-spec-review/SKILL.md` (insert three sections after `## Findings`; extend Common Mistakes)

**Interfaces:**
- Consumes: Task 2's sections and the four-part finding.
- Produces: `## The Loop`, `## What The User Receives`, `## Applying The Answers`. The decisions-entry format (bold bin and round, one-sentence finding, reason, what it retires or costs) is fixed here and Task 4 and Task 5 read it.

- [ ] **Step 1: Write "The Loop"**

From spec section 6. Fresh subagent per round, receiving only section 3's inputs and the pass list, never the previous round's findings. The three-bin table verbatim in content. The editing rules: spec's own voice and altitude, the covering sentence deleted not softened, no restyling, no commentary in the body, a builder item never promoted. The decisions section: heading, placement before "Left to the builder" and "Open with the user", the entry format with the three examples from spec section 6. Convergence: all findings covered, or none; cap five, and hitting it is reported as not converged. Oscillation guard: a reversal is declined with "reverses round N". Resume: an existing decisions section is read and appended to, round numbers continuing from its highest. The no-subagent fallback: run rounds yourself after re-reading the spec from disk, and say rounds were not independent.

- [ ] **Step 2: Write "What The User Receives"**

From spec section 7: the four-part message, in order, and the "nothing else" rule.

- [ ] **Step 3: Write "Applying The Answers"**

From spec section 7's closing paragraph: keep, flip, overrule, and a new design stated by the user; no re-run; the next step is `writing-plans`.

- [ ] **Step 4: Extend Common Mistakes**

Rows for: "I absorbed the cost silently because the fix was clearly right"; "round three disagrees with round one, so I'll go with round three"; "five rounds ran, so it converged"; "I'll summarise each round for the user"; "the subagent can't see the skills, so I'll paraphrase them in the prompt"; "the spec already has decisions, so I'll start a fresh section".

- [ ] **Step 5: End-to-end run on a scratch copy**

Copy both fixtures into the scratchpad directory. Dispatch a subagent told to read `skills/rust-spec-review/SKILL.md` and run it in full on the copy, with the scenario prompt pointed at the copy's paths. Then inspect:

- The copy has a `## Design decisions` section before "Left to the builder", with entries for items 1 to 7 of Task 1's list, each in the entry format and marked with a round.
- Item 2 (two `Option`s) is **Adopted** and the body now describes an enum with no "prefers device" sentence anywhere.
- Item 1 (id newtypes) is **Adopted with a cost** and the body uses the newtypes.
- Item 6 (`BrowserOpener`) is **Adopted** or **Declined** with a stated reason, not silently dropped.
- Item 8 has no entry, or an entry saying it was audited and stands.
- The decisions section records at least two rounds and the last round's findings are all covered, or the report says the cap was hit.
- The subagent's final message to the user has the four parts of "What The User Receives" and no per-round narrative.

Then a second check for Review Focus item 2: run the skill again on the same copy. Expected: no duplicate entries, round numbers continue, and the message says the spec was already converged or reports only new items.

Record both runs' key lines in the task report. Fix skill prose, not the fixture, for any miss.

- [ ] **Step 6: Check Review Focus items 1, 3, and 5 by prompt**

Item 1: dispatch a subagent with the skill and a three-paragraph product brief that names no types; expected one paragraph and no loop. Item 3: ask a subagent, with the skill loaded, what it reports when round five still has an uncovered finding; expected the exact "did not converge" behaviour, quoting the section. Item 5: ask a subagent, with the skill loaded and told it has no Agent tool, how it runs round two; expected re-reading the spec from disk and a stated "rounds were not independent" in its report. Record all three.

- [ ] **Step 7: Commit**

```bash
git add skills/rust-spec-review/SKILL.md
git commit -m "rust-spec-review: the loop, the decisions section, and the user message"
```

---

### Task 4: Plan mode and its fixtures

The lighter second mode, with a hand-authored post-review fixture so the test never depends on a live run.

**Files:**
- Create: `tests/fixtures/enrolment_spec_reviewed.md`
- Create: `tests/fixtures/enrolment_plan.md`
- Create: `tests/scenarios/spec-review-plan-mode.md`
- Modify: `skills/rust-spec-review/SKILL.md` (insert `## Plan Mode` after `## Applying The Answers`; extend Common Mistakes; one line in the description if it does not already cover "plan")
- Modify: `tests/fixtures/README.md` (append)

**Interfaces:**
- Consumes: Task 3's decisions-entry format.
- Produces: the reviewed fixture, which Task 5 also reads.

- [ ] **Step 1: Write the reviewed fixture**

`tests/fixtures/enrolment_spec_reviewed.md`: the Task 1 fixture as it would read after the loop. Body changes: `Record` is an enum `Pending(PendingRecord) | Device(DeviceRecord)` with a `BothRecorded` load error and no tiebreak sentence; `PendingId` and `DeviceId` newtypes; `ServerUrl` parsed once; a store error is `StoreError` and "no key" is `Ok(None)`; the write re-check is a `Record::update_if(id, ..)` method; the cancelled request is a per-start `AttemptId`; `BrowserOpener` is gone, replaced by a function; `FinalizeError` is exhaustive. A `## Design decisions` section with one entry per item in Task 3's format: items 2, 3, 4, 5, 6, 7 **Adopted**; item 1 **Adopted with a cost** naming the conversion at the boundary; one **Declined** entry for a typestate over `Client` with the reason "the three calls are made from different owners; needs the user". Item 8 untouched.

- [ ] **Step 2: Write the plan fixture**

`tests/fixtures/enrolment_plan.md`: a four-task plan in the writing-plans header form against the reviewed fixture. Task 2 of it says "`DeviceFile` holds `pending: Option<PendingRecord>` and `device: Option<DeviceRecord>`; `load` prefers `device` when both are present", re-introducing the retired shape. Task 3 of it implements `products` with a bare `String` device id, departing from the `DeviceId` decision. No task implements `Record::update_if`. Everything else faithful.

- [ ] **Step 3: Write the scenario prompt**

```markdown
# Scenario: spec-review-plan-mode

The plan at `tests/fixtures/enrolment_plan.md` implements the reviewed
design at `tests/fixtures/enrolment_spec_reviewed.md`. Review the plan
before it is executed.
```

- [ ] **Step 4: Write "Plan Mode"**

From spec section 8: one round, no loop; the three checks; "decided" means a decisions entry or, absent one, a stated shape; the leading line when there is no decisions section; no new design findings; the plan is not edited. Add Common Mistakes rows: "the plan has a design problem, so I'll review it as a spec"; "the spec has no decisions section, so plan mode cannot run".

- [ ] **Step 5: GREEN run**

Clean-room subagent, skill path first, then the scenario prompt. Expected: exactly three items, each naming the plan task and quoting the decision entry: the two-`Option` reintroduction (plan task 2, Adopted item 2), the `String` id departure (plan task 3, Adopted-with-a-cost item 1), and `update_if` unimplemented (item 4). No fourth item, no design finding, no edit to the plan. Then a RED-style control: the same prompt with the unreviewed `enrolment_spec.md` named instead. Expected: the leading "not been through the loop" line, then the checks against stated shapes.

- [ ] **Step 6: Append to the fixtures README and commit**

Add the two new fixtures under the Task 1 heading, saying what each departs from.

```bash
git add tests/fixtures/enrolment_spec_reviewed.md tests/fixtures/enrolment_plan.md tests/scenarios/spec-review-plan-mode.md tests/fixtures/README.md skills/rust-spec-review/SKILL.md
git commit -m "rust-spec-review: plan mode and its fixtures"
```

---

### Task 5: rust-design-review reads the decisions section

The other half of the loop: PR-time review honours what the spec decided.

**Files:**
- Modify: `skills/rust-design-review/SKILL.md` (Scope section, after the change-scoped bullets; one Common Mistakes row)
- Create: `tests/scenarios/design-review-decisions.md`

**Interfaces:**
- Consumes: the decisions-entry format from Task 3; `tests/fixtures/sensor_kit` as the code under review.
- Produces: nothing later tasks use.

- [ ] **Step 1: Write the scenario prompt**

`tests/scenarios/design-review-decisions.md`: asks for a whole-crate review of `tests/fixtures/sensor_kit`, and carries inline a short spec excerpt headed `## Design decisions` with one entry: "**Adopted with a cost (round 1).** `Sensor`'s fields stay `pub` and the struct carries no invariant; it is a plain record the firmware fills from a register dump. Cost: a caller can build an inconsistent `Sensor`. Retires: a constructor for a type that has nothing to validate." The prompt says the excerpt is from the crate's spec.

- [ ] **Step 2: RED run**

Clean-room subagent with the current `rust-design-review` text and the prompt. Expected: the `pub` fields finding appears as a full finding, the decision ignored or argued against. Record it.

- [ ] **Step 3: Add the rule**

In the Scope section of `rust-design-review/SKILL.md`, after the change-scoped bullets and before "Say which mode you are in": a paragraph from spec section 9. Before pass 1, look for a `## Design decisions` section in a spec under `docs/superpowers/specs` naming the crate or module under review, or handed to you with the request. A finding that matches a recorded decision is reported in one line as decided in the spec, quoting the entry, and not argued again; the exception is a decision that makes a stated goal of the spec unreachable, which is a full finding naming the decision. Link to `[rust-spec-review](../rust-spec-review/SKILL.md)` as the skill that writes the section. One Common Mistakes row: "the spec decided it, but I still think it's wrong" with the correction that the review's job is to name the goal it breaks or report it in one line.

- [ ] **Step 4: GREEN run**

Same prompt with the revised skill. Expected: the `pub` fields item is a single line quoting the entry; the other sensor_kit findings (nine-parameter function, `Box<dyn Error>`, docs, no tests) are unchanged in rank and form. Record it.

- [ ] **Step 5: Commit**

```bash
git add skills/rust-design-review/SKILL.md tests/scenarios/design-review-decisions.md
git commit -m "rust-design-review: honour the spec's design decisions"
```

---

### Task 6: README and manifest

**Files:**
- Modify: `README.md` (Skills list; Testing section)
- Modify: `.claude-plugin/plugin.json` (description)

- [ ] **Step 1: Add the skill's row**

In the Skills list, after `rust-design-review`: a `rust-spec-review` bullet in the same form, saying it runs the design review against a spec before code, loops until it converges, records decisions in the spec, and has a plan mode. In the Testing section, name the four new fixtures and three new scenarios where the existing ones are listed, and note the markdown fixtures follow the no-improvement rule.

- [ ] **Step 2: Update the manifest description**

`plugin.json` description: "Rust API design, documentation discipline, and design review skills, before and after the code exists". Leave the version.

- [ ] **Step 3: Verify the plugin still lists six skills**

Run: `ls skills/*/SKILL.md | wc -l` and `head -4 skills/rust-spec-review/SKILL.md`.
Expected: `6`, and frontmatter with `name` and `description` only.

- [ ] **Step 4: Commit**

```bash
git add README.md .claude-plugin/plugin.json
git commit -m "README and manifest: rust-spec-review"
```
