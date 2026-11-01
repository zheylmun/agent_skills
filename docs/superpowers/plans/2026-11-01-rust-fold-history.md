# rust-fold-history Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an eighth rust-craft skill, `rust-fold-history`, that folds a refined branch into one commit per plan task, rewrites the plan to the refined design, verifies the result, writes a handoff, and ends with a ready-to-paste `/compact` line; and have `rust-refinement-loop` offer it at break points when it is worth it.

**Architecture:** One new `skills/rust-fold-history/SKILL.md` in the form of the other rust-craft skills, plus small edits to `skills/rust-refinement-loop/SKILL.md`. It is tested against a scratch git repo built by a setup script: a copy of `sensor_kit` on `main`, then a branch with a spec, a three-task plan, three task commits, and seeded fix-up commits.

**Tech Stack:** Markdown skills with YAML frontmatter; a POSIX shell setup script; git; the `sensor_kit` Rust fixture crate; clean-room subagents for RED/GREEN runs.

**Spec:** `docs/superpowers/specs/2026-11-01-rust-fold-history-design.md`. Read it before starting; section numbers below refer to it.

## Global Constraints

- Skill name, exactly: `rust-fold-history`. Directory `skills/rust-fold-history/`, one file `SKILL.md`.
- Frontmatter has exactly two fields, `name` and `description`, under 1024 characters total; the description states triggering conditions only, in the `Use when ...` form.
- Cite other skills by relative link and never restate them. The base rule belongs to `[rust-refinement-loop](../rust-refinement-loop/SKILL.md)`; this skill links to it.
- The skill never pushes, never force-pushes, never edits settings, and never deletes the backup branch.
- The folded tree must equal the original tree; a difference stops the skill with the branch untouched.
- Green per folded commit: `cargo fmt --check`, a build of all targets, and the tests.
- Fixtures under `tests/fixtures/` are never improved. `tests/fixtures/sensor_kit` is never modified; the scratch repo is built from a copy.
- Scenarios under `tests/scenarios/` are prompt-only.
- On this machine, run cargo by hand as `cargo +1.96.1` (the default toolchain lacks the `cargo` component). Never pin a toolchain in a skill or fixture.
- Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, or the attribution the implementer's harness specifies.

## Review Focus

Failure modes the spec implies that no task's happy path would exercise, most likely first, each pinned to a task.

1. **A fold whose final tree differs by one hunk** (a fix-up dropped or applied twice). Expected: the skill stops before moving the branch and says which paths differ. Pinned to Task 2's prompt checks.
2. **A branch that merged its base mid-way.** Expected: the root is the merge base, base changes stay out of task commits. Pinned to Task 2's prompt checks.
3. **A branch with an upstream.** Expected: no push; the message gives `git push --force-with-lease` and says the branch now differs from its remote. Pinned to Task 2's GREEN (the fixture gets a bare remote).
4. **A red intermediate commit that no hunk move fixes.** Expected: the branch is not moved, the user is asked, naming the commit and failing test. Pinned to Task 2's prompt checks.
5. **A refinement-loop break point below the offer threshold.** Expected: no mention of folding. Pinned to Task 3's scenario.

---

### Task 1: The fold-branch fixture and RED

**Files:**
- Create: `tests/fixtures/fold_branch/setup.sh`
- Create: `tests/fixtures/fold_branch/` content the script commits (spec, plan, and the per-commit source states or patches; layout is the builder's)
- Create: `tests/scenarios/fold-history-baseline.md`, `tests/scenarios/fold-history-dirty.md`
- Modify: `tests/fixtures/README.md` (append a section)

**Interfaces:**
- Consumes: `tests/fixtures/sensor_kit` (read-only); the hardening in `tests/fixtures/refinement_branch/setup.sh` (`CDPATH=''`, a cleanup trap, `core.hooksPath=/dev/null`, `commit.gpgsign=false`, refuse a non-empty target), copied, not sourced.
- Produces: `setup.sh <target-dir> [--dirty] [--remote]`. It creates a git repo at `<target-dir>` with `main` holding a copy of `sensor_kit`, then branch `feat/sensor-fold` checked out with, in order: a spec commit (`docs/superpowers/specs/fold-design.md`), a plan commit (`docs/superpowers/plans/fold-plan.md`, three tasks in superpowers format), three task commits, then the eight fix-up commits below. `--dirty` leaves one uncommitted edit to a tracked file. `--remote` adds a bare repo at `<target-dir>.remote.git` as `origin` and pushes `main` and the branch with upstream set. The seed numbers below are what Task 2's checks refer to.

- [ ] **Step 1: Write the content.** A small plausible feature on `sensor_kit` (a sample buffer, say) in three plan tasks. Every commit builds and passes `cargo test`. The fix-ups, each a separate commit with a refinement-loop-style message naming its stage:
  1. adds a new type (a newtype, say `SampleCount`) used only by task 2's code;
  2. changes a signature in task 1's code;
  3. one commit touching code from both task 1 and task 3;
  4. and 5. two edits to the spec's `## Design decisions` section, one each;
  6. adds a logical step the plan lacks (a small new module with its own test, wired in);
  7. touches a file task 1 created but uses an item task 3 introduces, so folding it into task 1 leaves a commit that does not build;
  8. a simplify-stage tidy in task 2's code.
- [ ] **Step 2: Write `setup.sh`** (POSIX sh, hardened as above).
- [ ] **Step 3: Verify the fixture.** Run it into a fresh scratchpad directory, with and without each flag. Expected: `git log --oneline main..feat/sensor-fold` shows 13 commits; `cargo +1.96.1 test` passes at every commit (`git rebase --exec` on a throwaway copy, or a loop over `git rev-list`); `--dirty` leaves `git status --porcelain` non-empty; `--remote` leaves `git rev-parse @{u}` resolving.
- [ ] **Step 4: Scenarios and README.** `fold-history-baseline.md`: "This branch has been through the refinement loop. Fold its history before I open the PR." `fold-history-dirty.md`: the same prompt, run on a `--dirty` build. README section: what the fixture is, the flags, the seed list, the expected fold (spec commit, plan commit, four task commits: 6 total), and the prohibition on improving it.
- [ ] **Step 5: RED run.** One clean-room subagent, no skill, on a `--remote` build with the baseline prompt. Record in the task report what it did against spec section 4: backup, tree comparison, per-commit green, plan rewrite, push, handoff and `/compact` line.
- [ ] **Step 6: Commit.**

---

### Task 2: The skill

**Files:**
- Create: `skills/rust-fold-history/SKILL.md`

**Interfaces:**
- Consumes: Task 1's `setup.sh` and seeds; `rust-refinement-loop`'s base rule (its What It Reads section), by link.
- Produces: the handoff at `$(git rev-parse --git-dir)/fold-history/handoff.md` and the fold map beside it, whose fields Task 3's loop edits read: branch, base, root, backup branch, folded commits mapped to tasks, spec and plan paths, loop state when offered by the loop.

- [ ] **Step 1: Frontmatter and Overview** from spec sections 1 and 2. Trigger on a request to fold, squash, simplify or clean up a branch's history before a PR, or a refinement-loop break point where the user accepted a fold. The Overview says plainly that the skill cannot run `/compact` and why, and that it never pushes.
- [ ] **Step 2: When It Runs** from spec section 3: on request, or offered by the loop; the loop's thresholds live in the loop, linked.
- [ ] **Step 3: The Fold** from spec section 4, steps 1 to 6 in order, as numbered steps: preconditions, backup branch name `backup/<branch>/<yyyymmdd-hhmm>`, plan rewrite committed before the fold, the fold map written before building, the docs commits then task commits, tree comparison then per-commit green with the later-first hunk moves and the restart-from-step-4 rule, then `git update-ref` with the expected old sha. Then the no-plan path.
- [ ] **Step 4: The Handoff And The Message** from spec section 5: the handoff's contents, the three-part message and nothing else, the `/compact` line with the example's shape, the optional `SessionStart` hook mentioned once as the user's settings change.
- [ ] **Step 5: Common Mistakes.** Rows at least for: "the trees differ only in whitespace, so it's fine"; "the branch has an upstream, so I'll push to keep it in sync"; "the red commit is intermediate, the final tree is green, so I'll move the branch"; "I'll fix the code while folding since I'm touching it anyway"; "I'll delete the backup branch since the fold verified"; "I'll edit the spec to match the folded commits"; "I can run /compact at the end".
- [ ] **Step 6: Prompt checks.** One subagent each, given the skill, asked what it does and why, quoting the text: (a) after building, `git diff --stat` between the temporary branch and the original HEAD shows one file; (b) the branch merged `main` after task 2; (c) commit 3 of 6 fails to build and both hunk moves fail; (d) the user says "and push it when you're done". Pass when each answer matches spec section 4 or 2.
- [ ] **Step 7: End-to-end GREEN.** Build the fixture with `--remote` into a fresh scratchpad directory. One subagent told to read the skill and run it on the branch with the baseline prompt. Check: final tree equals the original HEAD's tree after the plan commit; 6 commits in `root..HEAD`; every task commit passes fmt, build and tests; seed 1's type is named in the rewritten plan's task 2; seed 6 has its own task; seed 7's change is in task 3's commit or later; the backup branch exists and points at the original HEAD; `git rev-parse origin/feat/sensor-fold` is unchanged; the handoff exists; the message has three parts and a `/compact` line naming the handoff. Then the dirty scenario on a `--dirty` build: nothing rewritten, no backup branch, the message says why.
- [ ] **Step 8: Commit.**

---

### Task 3: The refinement loop offers the fold

**Files:**
- Modify: `skills/rust-refinement-loop/SKILL.md` (Break Points part 1; the "history skill" paragraph after Applying The Answers; the Overview line naming folding; Common Mistakes)
- Create: `tests/scenarios/refinement-loop-fold-offer.md`

**Interfaces:**
- Consumes: Task 2's handoff path and fields.

- [ ] **Step 1: Break Points part 1** gains a clause offering `[rust-fold-history](../rust-fold-history/SKILL.md)` with the counts, when, since the last fold or the branch's start, fix-up commits number at least the plan's tasks, or 10 or more cycles have run, or the conversation is clearly long; otherwise nothing about folding.
- [ ] **Step 2: The history-skill paragraph** names the skill, keeps "never runs it unasked", says the user's answers are applied and committed before the offer, and that on resuming after a compaction the loop reads the handoff and keeps its recorded green baseline.
- [ ] **Step 3: One Common Mistakes row:** "The branch is long, so I'll fold it before the next batch" corrected to: offer it, with the counts; the user decides.
- [ ] **Step 4: Scenario** `refinement-loop-fold-offer.md`, prompt-only, two cases: a break point after batch 2 of a three-task plan with 14 fix-up commits; and one after batch 1 with 2 fix-up commits and a short session. Ask what the break-point message's first line says.
- [ ] **Step 5: Prompt check.** One subagent with the edited loop skill per case. Pass when the first case's line offers the fold with the counts and the second's does not mention it.
- [ ] **Step 6: Commit.**

---

### Task 4: README and plugin

**Files:**
- Modify: `README.md`, `.claude-plugin/plugin.json`

- [ ] **Step 1:** Add a `rust-fold-history` bullet after `rust-refinement-loop`: folds a refined branch into one commit per plan task, rewrites the plan to the refined design, verifies the tree, and hands off for `/compact`, never pushing. In Testing, name the `fold_branch` fixture, its flags and the three scenarios. Set the plugin description to "Rust API design, documentation discipline, design review, and branch refinement skills, before and after the code exists".
- [ ] **Step 2: Verify:** `ls skills/*/SKILL.md | wc -l` prints `8`; `head -4 skills/rust-fold-history/SKILL.md` shows two frontmatter fields; `python3 -m json.tool .claude-plugin/plugin.json` succeeds.
- [ ] **Step 3: Commit.**
