---
name: rust-refinement-loop
description: Use when a Rust branch is feature-complete and is being prepared for a pull request, or when asked to refine, polish, or run the review loop on a Rust branch
---

## Overview

Before a branch becomes a PR, the same rounds get run by hand: a design
review, an adversarial hunt for bugs, the fixes, a simplify pass, and again.
The rounds are mechanical, and the judgement in them sits at a few points.
This skill runs them as a loop. It applies what is clearly right without
asking, holds back what is a genuine trade-off, and stops at break points so
the user decides the trade-offs in one sitting.

It is an orchestrator. It reviews nothing itself. Each cycle runs three
stages, design, correctness and simplify, always in that order, each by a
fresh subagent. The orchestrator sorts what comes back, applies and commits
the clear changes one at a time, keeps the branch's spec and plan describing
what the code does, and holds the pending trade-offs in its own notes. It
ends at the first clean cycle (Stopping).

Code converges where prose does not. A fresh reviewer can always reinterpret
a sentence, but on a branch that has absorbed its structural fixes and its
confirmed bugs, fresh reviewers run dry. The stop rule relies on that, and on
the sort below keeping tidying and unconfirmed suspicion from counting as
work.

It assumes [rust-principles](../rust-principles/SKILL.md) as its standard.
The engines it runs are [rust-design-review](../rust-design-review/SKILL.md)
with [rust-quality-gate](../rust-quality-gate/SKILL.md) first, an
adversarial correctness review, and the built-in `simplify`. The cost screen
and the decisions-entry format belong to
[rust-spec-review](../rust-spec-review/SKILL.md), and this skill links to
them rather than restating them.

**The loop never pushes, never force-pushes, and never rewrites history.**
No amend, no rebase, no squash, no reset of a commit it made. Opening the
PR and folding the history are the user's, at a break point.

## What It Reads

**The branch diff against its base.** Every review in the loop is
change-scoped, in [rust-design-review](../rust-design-review/SKILL.md)'s
sense: a pre-existing problem the branch does not touch is not a finding.
Find the base before cycle 1, in this order:

1. A base the user named.
2. A local `main`.
3. `origin/HEAD`, the remote's default branch
   (`git symbolic-ref refs/remotes/origin/HEAD`), when it is set.
4. Otherwise, ask the user and wait.

Do not infer a base from a branch's name alone: a local `trunk` or
`develop` with no `origin/HEAD` pointing at it is a convention this skill
cannot see, and a wrong base turns every review into a review of someone
else's work. Record the merge base once; every stage's diff is
`git diff <merge-base>...HEAD`, which includes the loop's own commits.

Non-Rust files in the diff (a frontend, a script, a workflow file) are not
reviewed, simplified or changed. Note them once, for the break-point
message.

**The branch's spec and plan**, under `docs/superpowers/specs` and
`docs/superpowers/plans`: the ones the user names, or else the ones the
branch's own commits touch (`git log <merge-base>..HEAD --name-only` over
those two directories). When the commits touch more than one spec, ask which
governs. The spec's `## Design decisions` section, if it has one, governs
what stage 1 may re-raise, exactly as
[rust-design-review](../rust-design-review/SKILL.md) already reads it; the
orchestrator hands the reviewer the spec's path and does nothing more to
enforce it.

**The rust-craft skills**, by absolute path. A fresh subagent has none of
them loaded, so resolve each sibling `SKILL.md` from this skill's own base
directory and hand the reviewer the paths, as What A Round Reads in
[rust-spec-review](../rust-spec-review/SKILL.md) does, never a summary. Stage
1 gets [rust-principles](../rust-principles/SKILL.md),
[rust-api-design](../rust-api-design/SKILL.md),
[rust-quality-gate](../rust-quality-gate/SKILL.md) and
[rust-design-review](../rust-design-review/SKILL.md), plus
[rust-embedded](../rust-embedded/SKILL.md) when the crate is `no_std`.
Stages 2 and 3 get none: a correctness reviewer handed the design standard
drifts into design.

**When the branch has no spec**, the loop still runs. The decisions made at
break points then live only in the break-point message, and the message says
so. The loop does not create a spec.

**Before cycle 1**, the working tree must be clean, untracked files
included (`git status --porcelain` is empty), because discarding a
change that cannot be made green (Green And Commits) must not discard the
user's work. If it is not clean, ask. Then record the baseline that green is
measured against.

## One Cycle

A cycle is stage 1, then stage 2, then stage 3. Every change a stage applies
is green-checked and committed before the next stage starts, so each stage
reviews the branch as it now stands.

**Each stage's reviewer is a fresh subagent.** Dispatch a new one per stage
per cycle; never continue or message an earlier one. Its prompt carries the
repository path, the merge base, the stage's brief below, and the paths the
stage reads. It never carries an earlier cycle's findings, the
orchestrator's sort of them, the trade-off list, or a summary of what
changed: a reviewer primed with the last cycle's findings reviews the last
cycle, not the branch. The spec's decisions section is the only memory that
crosses cycles, and it is in the spec so the reviewer reads it as the author
would. The orchestrator keeps the findings, the sort, the trade-off list and
the counts in notes in the session's scratch space, outside the repo.

**Without a subagent tool**, the orchestrator runs each stage itself,
re-reading the branch and the skills from disk each time rather than working
from memory, and the summary says the stages were not independent.

### Stage 1: Design

The reviewer runs [rust-design-review](../rust-design-review/SKILL.md),
change-scoped against the merge base, with
[rust-quality-gate](../rust-quality-gate/SKILL.md) first as that skill
requires, and returns the review as that skill's Output describes. The gate's
observations feed the review every cycle; whether they block a change is a
separate question, answered by the baseline (Green And Commits).

Sort every finding in the review. Take these steps in order; the first that
matches is the finding's one disposition. A finding is sorted whole, by the
change it proposes: a smaller fix the reviewer mentions alongside it is part
of that finding, not a second one to apply while the first waits.

1. **Already on the trade-off list.** A fresh reviewer raises a pending
   item again every cycle until the user decides it. A finding that matches
   an item already listed, whatever step it would otherwise reach (a
   repair, a goal-breaking finding, a departure listed as "could not be
   applied green"), is that item: nothing is applied, nothing is added, and
   it is not new (Stopping). A finding matches a listed item when it names
   the same code and proposes the same change, however it is worded; a
   different remedy at the same site, or the same remedy somewhere else, is
   a new finding. This step comes first so a re-raised item is never
   re-binned or re-attempted.
2. **An aside.** A bug in the review's closing paragraph is handed to stage 2
   as a candidate, where the confirmation rule decides it like any other. So
   is a soundness or security point, whatever its length. Everything else
   there (performance, crate-wide gate evidence the review names once as
   pre-existing) goes in the notes and is not a finding.
3. **Decided.** A finding the review reports in one line as decided in the
   spec, with the entry quoted. Nothing to do.
4. **A departure** (the code is not the shape a spec entry decided) is
   **Clear**: the code is changed to match the spec. The spec is the
   authority on what was decided, so the spec's text does not move.
5. **A repair** (the decided shape does not do what its entry says, such as
   not retiring what its `Retires:` names) is **Questionable**: changing a
   decision is the user's call.
6. **Goal-breaking.** A finding that names a recorded decision and the stated
   goal or anti-goal it makes unreachable is **Questionable**. The loop never
   overrides a decision on its own judgement that the decision is wrong.
7. **Substantial new work** beyond the branch's own change is
   **Questionable**, whether or not a caller would see it: a test suite for
   a crate whose suite has never run, a module the branch only calls into
   rewritten. How much more work the branch takes on is the user's call.
8. **Tidying no caller sees.** A tidying-sized change no caller of the
   public surface could see or would need to know: the naming or structure
   of private code the branch touched, a test for a line the branch
   changed, the wording of a private item's doc. It goes in the notes and
   is not a finding. Private code is stage 3's, a test that shows a bug is
   stage 2's, and a design review that applies tidying finds more tidying
   every cycle, so no cycle is ever clean. Public rustdoc is not this step:
   a caller reads it, so a missing or wrong contract on a `pub` item is
   sorted by step 9 and is Clear.
9. **Otherwise, by cost.** **Questionable** when any of these holds:
   - the remedy's ergonomic cost passes the cost screen under The Loop in
     [rust-spec-review](../rust-spec-review/SKILL.md), its public-signature
     rule included. A newtype for a value every caller of a `pub fn` passes
     in is Questionable even when the review says "one conversion in one
     place". A caller writing one variant where it wrote two arguments
     converts nothing it already holds, so an enum over illegal
     combinations of arguments costs nothing;
   - the remedy changes behaviour a user of the product could notice: a
     failure the end user used to never see, a default that changes what
     they get.

   **Anything else is Clear.** The cost the review names is a claim; this
   step is the decision. Findings that are not about a caller obligation
   sort by the same test: a missing rustdoc contract, a missing
   `#[must_use]`, or unrequested public surface no governing authority asks
   for costs no caller anything, so it is Clear.

**Clear findings are applied**, by the orchestrator, one at a time in the
review's rank order, each green-checked and committed on its own. An enum
of a user or a group, replacing a `pub` function's user id and group id of
which exactly one may be set, retires the rule at no cost to any caller, so
it is applied, and when the spec names that function its text changes in
the same commit (Keeping The Spec True). **Questionable findings are not
applied.** Each goes on the trade-off list with the change, its cost, and
what it would retire, for the next break point.

The sort is total: step 9 bins whatever the first eight did not, so every
finding ends in exactly one place. A finding that ends nowhere comes back
next cycle and the loop cannot settle on it.

### Stage 2: Correctness

An adversarial review of the branch diff by a fresh subagent whose brief is
to break the code, not to approve it: find inputs that make it return a wrong
answer, panic, hang, lose or duplicate data, or break its own rustdoc
contract or the spec. It gets a disposable worktree of `HEAD` in the scratch
space (`git worktree add --detach`), so the tests it writes never touch the
branch, and stage 1's candidates as places to start, each stated as code to
look at, never as a verdict. It returns, for each finding, either a test
that fails in that worktree, with its file path, code and failure output, or
a concrete trace: the inputs, each step through the code by file and
function, and the wrong output. Remove the worktree afterwards with
`git worktree remove --force`.

A finding that matches an item already on the trade-off list (a confirmed
bug whose fix could not be applied green, say) is that item, as in stage
1's first step: it is not re-confirmed or re-attempted, and it is not new.

**A finding is confirmed only with evidence the orchestrator checks.** The
standard a test must meet is the same however it was written: it asserts
an expectation the code's rustdoc or the spec makes, or it shows a panic, a
hang, or loss of data on some input, which is a violation whether or not
anything promised otherwise.

- **A failing test** is confirmed when the orchestrator adds it to the
  branch's working tree, runs it, and sees it fail for the reason the
  finding states, against that standard. A test that fails to compile,
  asserts behaviour nothing promised, or passes, confirms nothing.
- **A trace** is confirmed when the orchestrator reads the code along it and
  every step holds. Then the orchestrator writes the failing test the trace
  describes, to the same standard. If no test it can write fails, the trace
  did not hold.

**Unconfirmed findings are dropped and counted**, and the working tree is
restored to `HEAD` (Green And Commits) so the test that confirmed nothing
leaves with them. They are not argued, fixed "to be safe", or passed on as
suspicions; the count reaches the break-point message. A stage 1
candidate the reviewer comes back with nothing on is unconfirmed too, and
counted. The one exception is a soundness or security point (a data race
no test can show, an unsound `unsafe` block): it is not reduced to a count
but goes on the trade-off list marked unconfirmed and stated in full, as
the soundness carve-out under Out Of Scope in
[rust-design-review](../rust-design-review/SKILL.md) requires.

**The orchestrator fixes each confirmed finding**, the fix kept to the bug,
and commits its failing test in the same commit as the fix.

The built-in `code-review` at `high` effort is an acceptable engine for this
stage when it is available, run against the merge base without `--fix` or
`--comment`. Its findings come with no failing test, so each goes through
the trace rule above. The loop never asks for `max` or `ultra`: those are the
user's to request.

### Stage 3: Simplify

A fresh subagent runs the built-in `simplify` pass, over Rust files only, in
a disposable worktree of `HEAD` as stage 2 does, so nothing it applies
there touches the branch. In cycle 1 it covers the whole branch diff, so
the branch's own code is simplified at least once. In later cycles it
covers only what that cycle changed: the diff from the commit the cycle
started at to `HEAD`. It reports each cleanup separately, as a sentence
and its diff, and does not apply them to the branch. Remove its worktree
afterwards with `git worktree remove --force`, as stage 2's is.

The orchestrator applies the cleanups one at a time, as in stage 1: each is
green-checked and committed on its own. A cleanup that cannot be made green
in two attempts is discarded and counted in the notes, never put on the
trade-off list. Simplify's output never counts against a clean cycle
(Stopping), so tidying cannot keep the loop alive.

**When an engine is missing**, if `code-review` or `simplify` is not
available, give a fresh subagent that stage's brief in plain words instead,
and record which engine ran, for the summary.

### Green And Commits

**Green** means: the crate's tests pass, `cargo fmt --check` passes, and
clippy and the docs build report nothing new compared with the branch as it
stood before cycle 1. Run them as
[rust-quality-gate](../rust-quality-gate/SKILL.md) does (its clippy and
rustdoc commands, `--workspace` and `--target` as it says), and before
cycle 1 record what they report as the baseline. Compare by lint and item,
not by line number, since every applied change moves lines.

The baseline exists because the gate reports observations, not a verdict,
and many crates fail its doc lint or carry clippy clusters before the branch
began. A crate with no rustdoc at all is not red for this loop; a change
that adds one more undocumented `pub` item is. Tests and `fmt` have no
baseline: if either fails before cycle 1, say which and ask, because no
change could be checked green against it.

**Every applied change is checked green before it is committed.** Clippy
and rustdoc exit zero on warnings, so their exit status says nothing about
"nothing new": compare their output with the baseline, then commit, never
in one chained command. A change
that is not green gets a second attempt. A change that cannot be made green
in two attempts is discarded: restore the working tree to `HEAD`, untracked
files included, from the repository root, so a crate in a subdirectory of
a workspace leaves nothing behind (`git restore --staged --worktree :/`,
then `git clean -fd :/`).
That loses only the change, since everything before it was committed and the
tree was clean at the start, so every untracked file is the loop's own. Its
finding goes on the trade-off list as "could not be applied green" (a
simplify cleanup is only counted, as stage 3 says), and the loop carries
on. A confirmed bug whose fix could not
be made green is therefore never silently dropped; it reaches the user at
the next break point.

**Each applied change is its own commit**, so the history skill the user
may run later can fold them. The subject says what changed; the body's first
line names the finding it fixes and the stage and cycle that produced it:

```
queue: take a RetryPolicy instead of a retry count

Stage 1 (design), cycle 2: a retry count of 0 meant "retry forever",
a rule every caller had to know. Retires: that rule.
```

Follow the repository's own commit conventions for anything else the message
carries.

## Keeping The Spec True

The spec and plan describe what the branch actually does, and the loop keeps
them that way. Without this, every applied change would make the code depart
from the spec, and the PR-time
[rust-design-review](../rust-design-review/SKILL.md) would report each one as
a departure.

- **When a Clear change alters a shape the spec or plan names**, the
  orchestrator updates that text in the same commit as the code, and adds an
  **Adopted** entry to the spec's `## Design decisions` section. Edit the
  prose as Editing the spec under The Loop in
  [rust-spec-review](../rust-spec-review/SKILL.md) says (the spec's voice,
  the retired sentence deleted, no commentary), with one difference: here the
  edit is committed, with the change it describes. A departure fixed to match
  the spec changes no spec text and adds no entry; the entry it departed from
  already says it. A Clear change that alters no named shape (a rustdoc
  contract, a `#[must_use]`) touches neither document.
- **Entries use the format of the decisions section** in
  [rust-spec-review](../rust-spec-review/SKILL.md), placement of a missing
  heading included. Round numbers continue from the highest round already in
  the section, counting `Revised in round N.` markers, as that skill's Resume
  counts them. One new round per break point: at the start of each batch the
  round is the highest recorded plus one, and every entry the batch writes,
  including those its break point's answers write (Applying The Answers),
  carries that number. A later
  [rust-spec-review](../rust-spec-review/SKILL.md) run then resumes from it
  intact.

> - **Adopted (round 3).** `Mailer::send` takes a `Recipient` enum of a user
>   or a group, never a user id and a group id where exactly one is set.
>   Retires: the "pass 0 for the id you are not using" rule.

**When the branch has no spec**, these entries have nowhere to go. The
changes are still made, and the orchestrator keeps, in its notes, the entry
each would have been; every break-point message lists them, so the user can
carry them into a spec.

## Stopping

**A cycle is clean** when stage 1 applies nothing and adds nothing new to
the trade-off list, and stage 2 confirms nothing. Stage 3's cleanups do not
count, applied or not. A finding stage 1 reports as decided, and a finding
that matches an item already on the trade-off list (the sort's first step),
are not new: a fresh reviewer raises a pending item again in every cycle
until the user decides it, and a re-raised item cannot keep the loop alive.

**Pending items live in the orchestrator's notes**, outside the repo, until
the user decides them. They are not written into the spec, the plan, a
commit, or a file in the branch: a pending item in the spec reads to the
next reviewer as a decision, and to the user as one already taken.

**The loop ends at the first clean cycle.** It does not run one more to be
sure, and it does not end on a cycle that was not clean because the next
looks likely to be.

**There is no hard cap** beyond the break points. If two consecutive batches
end without a clean cycle, the second break point's message says so
plainly in its first line and recommends stopping: a branch that does not
converge in ten cycles is telling the user something about its design, and
another batch will not.

## Break Points

A break point comes after every fifth cycle of a batch, and when the loop
ends. A clean fifth cycle is the end, not a pause. At a break point the loop
stops and gives the user one message, with four parts in this order:

1. **One line:** the cycles run in this batch, and whether the loop ended
   clean or is pausing. The same line says, in a clause each, when it
   applies: that the branch has no spec, so this message is the only record
   of its decisions; which stage ran on a plain brief because its engine
   was missing (When an engine is missing); that stages were not
   independent (Without a subagent tool); and, after a second batch in a
   row without a clean cycle, that and the recommendation to stop.
2. **The trade-off list**, most consequential first by the ranking under
   Output in [rust-design-review](../rust-design-review/SKILL.md), each
   with the change, its cost, and what it would retire. A confirmed bug
   whose fix could not be applied green says so and gives the failing test;
   an unconfirmed soundness point says it is unconfirmed and is stated in
   full. Items that touch the same code stand next to each other, each kept
   whole, with one clause saying whether they are alternatives or compose.
   These are the items to decide.
3. **What was applied**, in a short list grouped by stage (design,
   correctness, simplify), one line each with its commit id. When the
   branch has no spec, each change that would have added an entry carries
   that entry beneath it, in the decisions-section format, as do the
   answers to earlier break points, so the user can carry them into a spec.
4. **What was dropped:** the count of unconfirmed correctness findings, the
   count of cleanups discarded as not green, and the non-Rust files in the
   diff, by path, left untouched.

Nothing else: no per-cycle narrative, no findings the sort marked decided,
no restatement of the spec, no question per item. The rounds' detail stays
in the notes and is shown if the user asks. When a part is empty, say so in
a clause ("nothing awaits a decision"). Then wait for the user.

## Applying The Answers

When the user replies, take each item they decided in turn:

- **Approved**: apply the change as a Clear change is applied, checked
  green and committed on its own, with the spec and plan text it alters in
  the same commit (Keeping The Spec True), and an entry
  **Adopted with a cost (round N).** whose `Cost:` is the cost the user
  accepted and whose `Retires:` is what it retires. If it cannot be made
  green in two attempts, discard it as Green And Commits says and tell the
  user in one line. It is not recorded as adopted; it stays on the list,
  marked "could not be applied green", so stage 1's first step matches it
  when it is raised again.
- **Declined**: change no code, and add an entry
  **Declined (round N).** stating the finding, with `Reason:` the user's
  reason, committed on its own.

N is the batch's round (Keeping The Spec True). With no spec, the entries
go in the notes and every later break-point message carries them. An item
the user does not decide stays on the list, and stage 1's first step still
matches it.

Then:

- **If the loop paused**, the next batch of five starts, at a new round.
- **If the loop had ended clean and the user approved a change**, a new
  batch starts, with its own round and its own break point after its fifth
  cycle, so that change is itself reviewed; it ends at the next clean
  cycle. The clean cycle that ended the last batch ran before the approved
  change existed, so it reviewed none of it.
- **If the loop had ended clean and nothing was approved**, it is done once
  the answers are committed. Any undecided items are listed once more in
  the reply, as for "stop here".
- **"Stop here"** ends the loop after the answers given are recorded. The
  undecided items stay in the notes, listed once more in the reply.

At a break point the user may run the history skill to fold the loop's
commits, or ask the loop to run it, before the next batch. The loop never
runs it unasked, since folding rewrites history and that is the user's
call. After it, the commit ids in the notes are stale: re-read the branch,
keep the merge base, and check the tree is clean before the next cycle.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "The reviewer said it's a bug, so I fixed it." | A finding is confirmed only by a failing test the orchestrator reran, or a trace it checked against the code step by step. An unconfirmed finding is dropped and counted, never fixed "to be safe": a fix for a bug nobody confirmed is a change nobody can justify. |
| "The newtype is obviously right, I applied it." | A newtype for a value every caller of a public signature passes costs a conversion at every one of those callers, and the cost screen in [rust-spec-review](../rust-spec-review/SKILL.md) counts that however the reviewer described it. It is Questionable and goes on the trade-off list; principle 5 of [rust-principles](../rust-principles/SKILL.md) leaves that call to the user. |
| "The gate was red before I started, so I can't continue." | Green is "nothing new" for clippy and the docs build, against the baseline recorded before cycle 1. Pre-existing clusters and a crate with no doc contract do not block; only what the loop's own change adds does. Failing tests or `fmt` before cycle 1 are the one case to stop and ask. |
| "I committed and then ran the tests." | Green is checked before the commit. A red commit is in the history for good, because the loop never rewrites it; a change that is not green is retried once and then discarded uncommitted. |
| "I squashed the fixes to keep history tidy." | The loop never rewrites history. One commit per change, each naming its finding and stage, is what the history skill folds when the user chooses to run it. |
| "The code departs from the spec, so I updated the spec to match the code." | A departure is Clear the other way: the code changes to match the spec, because the spec is the authority on what was decided. Changing the decision is a repair, and a repair is Questionable, the user's call. |
| "The reviewer found a tidier way to hold the private state, so I applied it." | A tidying-sized stage 1 finding no caller could see is not a finding: private naming and structure, a test for a changed line, and a private item's doc wording go in the notes. Public rustdoc is caller-visible and stays Clear; substantial new work is Questionable first. Stage 3 tidies private code once per cycle's changes; a design stage that applies tidying never runs dry. |
| "The simplify pass changed something, so the cycle isn't clean." | Simplify never counts. A cycle is clean when stage 1 applies nothing and adds nothing new and stage 2 confirms nothing; if tidying counted, the loop would never settle. |
| "The reviewer raised the newtype again, so it's a new item." | A finding that names the same code and proposes the same change as a listed item is that item. A fresh reviewer raises every pending item until the user decides it; counting it as new means no cycle is ever clean. |
| "Five cycles ran, I'll keep going." | The fifth cycle of a batch is a break point. Stop, give the four-part message, and wait: the trade-offs are the user's, and five more cycles only re-raise them. |
| "The user approved it, so the loop is done." | An approved change is new code no stage has reviewed. Apply it, record it as Adopted with a cost, and run the loop again until the next clean cycle. |
| "I'll put the pending trade-offs in the spec so the next reviewer sees them." | Pending items stay in the orchestrator's notes. In the spec, the next reviewer reads them as decisions, and the user finds them already taken. Only the user's answer writes an entry. |
| "I'll summarise each cycle for the user." | The break-point message has four parts and nothing else. Per-cycle detail stays in the notes, outside the repo, and is shown if the user asks. |
