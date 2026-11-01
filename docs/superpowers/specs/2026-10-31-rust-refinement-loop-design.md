# rust-refinement-loop: design

Source: the brainstorm of 2026-10-31, after `rust-spec-review` landed. A
seventh skill in the `rust-craft` plugin. Unversioned, like everything under
`docs/superpowers`.

## 1. Intent

Before a Rust branch becomes a PR, the user runs rounds by hand: a rust-craft
design review, an adversarial correctness review, fixes, a simplify pass, and
again. On the device auth branch in quartermaster that was six rounds, and the
late rounds found nothing substantive. The work is mechanical enough to run
unattended and judgement-heavy only at a few points.

`rust-refinement-loop` runs those rounds as a loop. It applies what is clearly
right without asking, holds back what is a genuine trade-off, and stops at
regular break points so the user decides the trade-offs in one sitting. It ends
when a whole cycle comes back clean.

Code converges faster than prose. The spec loop needed rulings on convergence
because a fresh reviewer can always reinterpret a sentence. The user's
experience on the device auth branch is that code reviews run dry: the last
rounds found no design improvement and no confirmed bug. The stop rule below
relies on that.

## 2. Goals, non-goals, anti-goals

Goals:

- A branch leaves the loop with no clear structural improvement left
  unapplied, no confirmed bug left unfixed, and every questionable trade-off
  either decided by the user or listed for them.
- The user is interrupted only at break points: every fifth cycle, and at the
  end.
- Every decision the user makes at a break point is written into the branch's
  spec and plan, so later cycles and the PR-time review do not raise it again.
- The history the loop leaves is easy for the planned "simplify branch
  history" skill to fold: one commit per applied change, each naming what it
  fixes.

Non-goals:

- Opening the PR, pushing, or rewriting history. History simplification is a
  separate skill, run at a break point when the user chooses.
- Reviewing non-Rust code on the branch. Frontend or script changes on a mixed
  branch are left alone and named once in the summary.
- Performance or `unsafe` soundness review, beyond what the existing reviews
  already say about them.

Anti-goals (a failure even if it works):

- Applying a structural change with a real ergonomic cost without the user
  deciding it.
- A "fix" for a bug nobody confirmed.
- A loop that never ends because the simplify pass can always tidy something.
- Leaving the branch red: a cycle that ends with failing tests or a failing
  quality gate.
- Re-raising a decision the spec already records.

## 3. Where it runs and what it reads

Invoked when a Rust branch is feature-complete and the user is preparing a
PR, or on a direct request to refine, polish, or run the review loop on a
branch. The trigger description says so.

Inputs:

- **The branch diff** against its base. Every review is change-scoped, in
  `rust-design-review`'s sense. Pre-existing problems the branch does not
  touch are not findings.
- **The branch's spec and plan**, under `docs/superpowers/specs` and
  `docs/superpowers/plans`, found by the branch's own commits touching them or
  named by the user. The spec's `## Design decisions` section, if present,
  governs what the design review may re-raise, exactly as `rust-design-review`
  already reads it.
- **The rust-craft skills**, by path from this skill's own base directory, as
  `rust-spec-review` hands them to its reviewers.

If the branch has no spec, the loop still runs. Decisions made at break
points then live only in the break-point summary, and the summary says so.
My call: the loop does not create a spec.

## 4. One cycle

A cycle has three stages, always in this order. Each stage's reviewer is a
fresh subagent that sees the current branch, not the previous cycle's
findings, so it cannot anchor on them. The orchestrator holds the findings
and the cycle's bookkeeping.

### Stage 1: design

Run `rust-design-review`, change-scoped, with `rust-quality-gate` first as
that skill already requires. Sort each finding:

- **Clear.** The remedy retires a caller obligation and has no ergonomic cost
  at the described surface, using the cost screen `rust-spec-review` defines:
  a cost counts only if the author, shown both shapes, could plausibly pick the
  old one for ergonomics. A cost that recurs at every caller of a public
  signature always counts. Clear findings are applied.
- **Questionable.** The remedy has a real cost, or changes behaviour a user of
  the product could notice, or contradicts a stated decision on the grounds
  that it breaks a goal. Not applied. Added to the trade-off list for the next
  break point.
- **Decided.** A finding `rust-design-review` reports in one line as decided
  in the spec. Nothing to do.

Two of `rust-design-review`'s full-finding cases need a fixed sort, because
the spec is the authority on what was decided:

- **A departure** (the code is not the shape the spec decided) is Clear: the
  code is changed to match the spec.
- **A repair** (the decided shape does not do what its entry says) is
  Questionable: changing a decision is the user's call.

Findings that are not about a caller obligation sort by the same test. A
finding whose remedy costs no caller anything (a missing rustdoc contract, a
missing `#[must_use]`, unrequested public surface a governing authority does
not ask for) is Clear and applied. A finding whose remedy is substantial new
work beyond the branch's own change (a crate whose suite has never run needs a
test suite written) is Questionable, so the user decides whether this branch
takes it on; that check comes before the next one, so such work is never
waved through as tidying. A tidying-sized change no caller can see (the
naming or structure of private code the branch touched, a test for a line the
branch changed, the wording of a private item's doc) is not a finding at all:
it goes in the orchestrator's notes. Private code is stage 3's to tidy and a
test that shows a bug is stage 2's, and a design review that applies tidying
finds more tidying every cycle, so no cycle would ever be clean. Public rustdoc is not tidying, because a
caller reads it: a missing contract on a `pub` item stays Clear. A bug `rust-design-review` mentions in its closing aside is
handed to stage 2 as a candidate, where the confirmation rule decides it.

### Stage 2: correctness

An adversarial correctness review of the branch diff, by a fresh subagent
whose brief is to break the code, not to approve it. A finding is
**confirmed** only with evidence: a failing test the reviewer wrote, or a
concrete trace from inputs to the wrong output that the orchestrator checks
against the code. Unconfirmed findings are dropped and counted. Confirmed
findings are fixed, each with the failing test committed alongside the fix.

My call: the built-in `code-review` skill at `high` effort is an acceptable
engine for this stage if it is available, with its findings then put through
the confirmation rule above. The skill never asks for `max` or `ultra`, which
are the user's to request. If neither `code-review` nor `simplify` is
available, the orchestrator gives a fresh subagent that stage's brief in
plain words instead, and the summary says which engine ran.

### Stage 3: simplify

Run the built-in `simplify` pass. In cycle 1 it covers the whole branch diff,
so the branch's own code is simplified at least once. In later cycles it
covers only what that cycle changed. It applies its own cleanups. Its output never counts against a clean
cycle (section 5), so tidying cannot keep the loop alive.

### After each stage

**Green** means: the crate's tests pass, `cargo fmt --check` passes, and
clippy and the docs build report nothing new compared with the branch as it
stood before cycle 1. My call: `rust-quality-gate` reports observations, not a
verdict, and many crates fail its doc lint or carry clippy clusters before the
branch began, so the blocking test is "no new failures", recorded once at the
start. The gate's observations still feed stage 1 as evidence every cycle.

Every applied change is checked green before it is committed. A change that
cannot be made green in two attempts is discarded (it was never committed, so
discarding it rewrites nothing), and its finding goes on the trade-off list as
"could not be applied green". The loop then carries on. A confirmed bug whose
fix could not be made green is therefore never silently dropped: it reaches
the user at the next break point.

Each applied change is its own commit, with a message that names the finding
it fixes and which stage produced it. That is what the history skill later
folds.

## 5. Stopping

**A cycle is clean** when stage 1 applies nothing and adds nothing new to the
trade-off list, and stage 2 confirms nothing. Simplify's output does not
count. A fresh reviewer will raise a pending trade-off again in every cycle
until the user decides it; the orchestrator matches it to the item already on
the list, and a re-raised item is not new. My call: pending items are kept in
the orchestrator's notes, not written into the spec before the user decides
them.

**The loop ends** at the first clean cycle.

**Break points** come every fifth cycle and at the end. At a break point the
loop stops and gives the user one message:

1. One line: cycles run in this batch, and whether the loop ended clean or is
   pausing.
2. The trade-off list, most consequential first, each with the change, its
   cost, and what it would retire. These are the items to decide.
3. What was applied, in a short list grouped by stage, with commit ids.
4. What was dropped: the count of unconfirmed correctness findings, and any
   non-Rust code left untouched.

Nothing else. The rounds' detail stays in the orchestrator's notes outside
the repo.

**Applying the answers.** For each trade-off the user decides, the
orchestrator applies or declines the change. Then, if the loop paused, the
next batch of five starts. If the loop had ended clean but the user approved a
change, one more cycle runs so that change is itself reviewed, and the loop
ends at the next clean cycle. My call: the user can also say "stop here", and
at a break point the user may run the history skill before the next batch.

**Keeping the spec true.** The user keeps the spec and plan describing what
the branch actually does, and this loop does the same. Without it, every
applied change would make the code depart from the spec, and PR-time
`rust-design-review` would report each one. So:

- When a Clear change alters a shape the spec or plan names, the orchestrator
  updates that text in the same commit and adds an **Adopted** entry to the
  spec's `## Design decisions` section.
- A trade-off the user approves is recorded as **Adopted with a cost**, with
  the cost they accepted. One the user declines is recorded as **Declined**,
  with their reason.
- Entries use `rust-spec-review`'s format, and the round number continues
  from the highest round already in the section: one new round per break
  point, so `rust-spec-review`'s resume numbering stays intact.

When the branch has no spec, these entries have nowhere to go. The changes
are still made, and each break-point message lists what a spec would have
recorded.

**The cap.** There is no hard cap beyond the break points. If two consecutive
batches end without a clean cycle, the second break-point message says so
plainly and recommends stopping, because a branch that will not converge in
ten cycles is telling the user something about its design.

## 6. Changes to other skills

None required. `rust-design-review` already reads the spec's decisions
section and is change-scoped. `rust-spec-review` owns the entry format; this
skill links to it.

`README.md` gains the skill's bullet.

## 7. Testing

The repo's RED/GREEN convention. A fixture is needed that is a branch, not a
crate: `tests/fixtures/sensor_kit` plus a small scripted diff applied on a
scratch git branch, seeded with one clear structural win, one questionable
trade-off (a newtype at a public signature), one real bug with a failing
input, one plausible-looking false bug, and one tidy-up only simplify would
make.

The fixture also carries a short spec for the change, with one
`## Design decisions` entry the scripted diff respects and one shape the diff
departs from.

- **GREEN:** one run of the loop on the scratch branch. It applies the clear
  win and updates the spec's text and decisions to match, fixes the
  departure, lists the newtype, fixes the real bug with a test, drops the
  false bug as unconfirmed, does not re-raise the recorded decision, ends
  clean within the first batch, and the break-point message has the four
  parts.
- **No-spec run:** the same branch with the spec removed. The loop runs, and
  the message lists what a spec would have recorded.
- **Break-point check:** a prompt check that asks what the loop does after
  cycle 5 with an unclean cycle, expecting the four-part message and a pause.

## 8. Constraints

- Single `SKILL.md`, the form of the other rust-craft skills, citing them by
  relative link.
- Nothing depends on the superpowers plugin. Without a subagent tool the
  orchestrator runs each stage itself and says in the summary that reviewers
  were not independent.
- The loop never pushes, never force-pushes, and never rewrites history.

## 9. Left to the builder

- The skill's exact trigger wording.
- The correctness reviewer's prompt, within section 4's confirmation rule.
- How the orchestrator finds the branch base, with `main` and the upstream as
  the expected candidates, asking when neither is clear.
- The commit message format, within section 4's requirement.
