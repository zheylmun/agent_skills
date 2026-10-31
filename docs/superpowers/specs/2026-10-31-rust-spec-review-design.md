# rust-spec-review: design

Source: the brainstorm of 2026-10-31, grounded in the quartermaster device
enrolment branch (`~/dev/voidstar/quartermaster`, `feat/real-backend`), where
six rounds of rust-craft and correctness review after implementation found
design fixes that were already visible in the approved spec.

A sixth skill in the `rust-craft` plugin. Unversioned, like everything under
`docs/superpowers`.

## 1. Intent

Today a spec is approved, a plan is written, code is built, and then
`rust-design-review` finds the shape problems: the prose invariant that
should have been a type, the two `Option`s that should have been an enum,
the error collapsed into an absence. Each one is a refactor across call sites
and tests, and several correctness bugs found in the same rounds would never
have existed under the better shape.

`rust-spec-review` runs the design review against the spec, before any code
exists, and iterates on it autonomously until the design has absorbed every
structural fix it can and the remaining ergonomics-versus-correctness
trade-offs are explicit. The user then makes those trade-offs once, in one
sitting, and the plan is written against a spec that will not be redesigned
in code review.

The question behind every finding is the one `rust-design-review` asks, moved
earlier: **what does a caller have to hold in their head that a better type
would have held for them?** The spec equivalent of a load-bearing comment is
a load-bearing sentence.

## 2. Goals, non-goals, anti-goals

Goals:

- A spec that has been through the loop produces a `rust-design-review` at
  PR time whose findings are correctness-neutral: naming, docs, a missed
  `#[must_use]`. Structural findings at PR time mean the loop missed
  something, and that is the skill's failure to investigate.
- Every ergonomics-versus-correctness decision is written down in the spec,
  with its reason, before the plan is written. Later reviews read the
  decision instead of re-deriving the finding.
- The loop runs without the user until it converges. The user sees one
  consolidated list of trade-offs, most consequential first, and nothing
  else.
- The skill reuses rust-craft's vocabulary, passes, and finding shape. It
  invents no parallel standard.

Non-goals:

- Reviewing a spec for product correctness, completeness of requirements, or
  whether it is the right thing to build. That is brainstorming's builder
  check and the user's own judgement.
- Reviewing non-Rust specs. The passes are Rust-shaped; a spec for a
  frontend or a protocol with no Rust surface is out of scope.
- Replacing `rust-design-review` at PR time. Code can still drift from spec,
  and docs, comments, and gate evidence exist only in code.
- A correctness bug hunt. A bug visible in a spec's logic is an aside, exactly
  as it is in `rust-design-review`.

Anti-goals (a failure even if it works):

- A loop that rewrites a user's product decisions as if they were design
  flaws. A decision the spec states with a reason is an authority, not a
  finding, unless the reviewer can show it makes a stated goal unreachable.
- A loop that oscillates: round three re-introducing what round one removed.
- Trade-offs resolved silently. An ergonomic cost the reviewer named and the
  orchestrator absorbed without recording it is the one outcome worse than
  not running the skill.
- Findings against shapes the spec deliberately left to the builder.

## 3. Inputs and where it runs

The skill runs after brainstorming has produced an approved design doc and
before `superpowers:writing-plans`. Its trigger description must say so, and
must also fire on a direct request to review a spec, design doc, or plan for
Rust work.

Inputs, all read by every round's reviewer:

- **The spec.** A superpowers-style design doc is the expected input, but
  any markdown that names Rust shapes qualifies. A spec that names no types,
  signatures, states, errors, or persisted shapes has nothing for the passes
  to bite on; the skill says so in one paragraph and stops rather than
  inventing a surface to review.
- **The existing public surface the spec touches.** When the spec changes a
  crate that exists, the reviewer reads that crate's `pub` items in the
  modules the spec names, so a finding about a new shape accounts for the
  shapes beside it. It does not run `rust-quality-gate`: there is no new code
  to gate, and the existing crate's gate output is PR-time evidence.
- **The rust-craft skills.** `rust-principles`, `rust-api-design`, and
  `rust-design-review` are the standard, cited rather than restated, exactly
  as `rust-design-review` cites them. `rust-embedded` when the spec targets
  `no_std`. A fresh round reviewer has nothing loaded, so the orchestrator
  hands it the absolute paths of those `SKILL.md` files, resolved from this
  skill's own base directory (the sibling `skills/` entries), and tells it to
  read them before the spec. My call: paths, not an instruction to invoke
  skills by name (a subagent may lack the tool) and not an inlined summary
  (a second copy of the standard that would drift). The no-subagent fallback
  reads the same files.
- **The spec's own decisions section**, if a previous round or a previous
  invocation wrote one (section 6).

The skill edits the spec in place. The spec is committed, so the diff is the
record. My call: the skill does not commit; the user commits the revised
spec with the plan, as today.

## 4. The passes

Each round's reviewer runs these in order. The order mirrors
`rust-design-review` where the pass has a code counterpart, so a reader of
both skills sees one method.

### 1. Named surface

Enumerate every type, function, trait, field, constant, and signature the
spec names, and read each as a stranger who has only the spec. For each: what
invariant does the spec hand the caller to maintain by hand? A `pub` field
beside a described validation rule, two `String`s in one signature that mean
different things, a call order the prose states and no type enforces, a
fixed set of legal values described as a number or a string.

Includes shapes named only in prose. "Each start carries a state the
machine remembers" names a field as surely as a code block does.

### 2. States and transitions

For every state table, state machine, or lifecycle the spec describes: can
the spec's own representation reach a state the spec says is illegal? Two
`Option`s where the prose says "exactly one", a boolean beside an enum that
already encodes it, a state the table lists that no transition reaches, a
transition the prose allows that the table does not. "A file with both
prefers the device record" is the signature of this finding: a rule for a
case the type should have made unrepresentable.

### 3. Error design

Judge every error enum, result, and "fails with" sentence against the Error
Design section of `rust-api-design`. Ask first which failures a better
argument type deletes. Ask whether the spec has collapsed an error into an
absence ("a corrupt item is treated as no key") or a success into an error
code the caller must string-match.

### 4. Boundaries

Ownership, concurrency, and persistence. Who owns each piece of state the
spec names? Which sentences describe a lock discipline ("the mutex is never
held across a network call") or a write discipline ("every write re-checks
the record still names the id") that structure could carry instead? Which
persisted or wire shape admits values the in-memory type refuses, so that a
load path needs a validation the spec forgot to describe?

### 5. Prose rules

The counterpart of `rust-design-review`'s comments pass and, as there, the
one most easily skipped. Read every sentence of the form "must", "never",
"always", "exactly one", "is treated as", "the caller re-checks", "it is
safe to", "by construction". Each is a candidate: **if the design were
right, would this sentence need to exist?** The sentence is never the
finding; the shape it is covering for is. If this pass finds nothing, the
review says so in a clause, because that is the only way a reader knows it
ran.

### 6. Decisions audit

Read the spec's stated decisions ("my call", "decided", "the user asked
for", and the decisions section from section 6 below). For each finding from
passes 1 to 5, check whether a recorded decision already covers it. If it
does, the finding is dropped unless the reviewer can show the decision makes
a stated goal or anti-goal unreachable, in which case the finding names the
decision and the goal it breaks. This pass is what stops the loop
oscillating and what keeps the skill from relitigating the user's product
calls.

### 7. Cognitive load

Taken over the whole spec: how many concepts must a caller carry to make one
correct call against the described surface? Which abstraction the spec
proposes relocates complexity rather than removing it? A trait with one
described implementor, a generic parameter the spec never varies, a builder
over three fields.

### 8. Governing authority

The counterpart of `rust-design-review`'s pass 7, and run last for the same
reason: it audits the remedies the other passes proposed. A spec often
answers to something outside itself: a server contract, a wire format
another repo owns, a requirement set, a standard. The quartermaster spec that
motivated this skill names such a contract in its first paragraph. Where one
exists, the reviewer opens it and works both directions: shapes the spec
proposes that the authority does not ask for, and remedies from passes 1 to 7
that the authority forbids or that would hedge a signal it wants loud (an
open enum over a set the contract enumerates exhaustively, a default that
absorbs a new mandatory field). A remedy that cannot name what wants it is
deleted here, not ranked. If nothing governs the spec, the review says so in
a clause.

## 5. Findings

Same three-part shape as `rust-design-review`, same ranking criteria, same
rule that a long parameter list is a surface finding and not a style
finding:

1. **The design fact**, in the spec's own vocabulary.
2. **The cost to the caller**: what they remember, re-check, or silently get
   wrong.
3. **The change that retires it**, stated as a change to the spec, naming
   what the caller stops thinking about.

Plus one field `rust-design-review` does not have, because the loop needs it
to sort:

4. **The ergonomic cost of the change**, named or explicitly "none I can
   name". A newtype costs a conversion at every call site; a typestate costs
   a generic parameter in every signature; an enum over two `Option`s costs
   nothing. The reviewer states which, concretely. "Might be over-engineering"
   is not a cost; `rust-principles` already forbids that answer.

Each finding also carries the section of the spec it is about, so the
orchestrator can edit the right place.

A round's output is the ranked findings as prose, a clause for any pass that
found nothing, and a one-line closing paragraph for anything out of scope,
on `rust-design-review`'s budget. A round that finds nothing says so in a
paragraph.

## 6. The loop

The skill is an orchestrator. Each round is a **fresh subagent** that
receives the inputs in section 3 and the pass list, and returns findings. It
does not receive the previous round's findings or the orchestrator's sorting
of them; the decisions section in the spec is the only memory that crosses
rounds, and it is in the spec so the reviewer reads it as the spec's author
would have written it.

Between rounds the orchestrator sorts every finding into one bin:

| Bin | Rule | Effect on the spec | Reaches the user |
|---|---|---|---|
| **Adopted** | The reviewer named no ergonomic cost, and no recorded decision covers it | The shape is rewritten in place; a one-line entry in the decisions section | No, beyond the diff |
| **Adopted with a cost** | The reviewer named a real cost | The shape is rewritten in place, provisionally; the decisions section records both the cost and the obligation it retires | Yes, with both sides, so the user can flip it |
| **Declined** | The remedy needs product knowledge the spec does not give, contradicts a recorded decision without breaking a goal, or is about something the spec left to the builder | Nothing changes in the body; the decisions section records the finding and why it was declined | Yes, so the user can overrule |

The orchestrator never invents a fourth bin and never leaves a finding
unsorted. A finding it cannot sort is declined with "could not sort" as the
reason, which puts it in front of the user.

**Editing the spec.** Adopted changes are made in the spec's own voice and
at the spec's own altitude. The orchestrator changes the shape the finding
names and the sentences that described the obligation it retires; the prose
rule a finding was about is deleted, not softened, when the new shape makes
it unnecessary. It does not restyle unrelated prose and does not add
commentary to the body. A "Left to the builder" entry is never promoted into
a decision by the loop.

**The decisions section.** Appended to the spec under a heading `## Design
decisions` if absent, before "Left to the builder" and "Open with the user"
when those exist. One entry per decision, each stating the finding in one
sentence, the bin, the reason, and the round. Example:

> - **Adopted (round 1).** The device file's record is an enum of pending or
>   enrolled, never two optional tables. Retires the "prefers device when
>   both present" rule.
> - **Adopted with a cost (round 2).** `PendingId` and `DeviceId` are
>   newtypes. Cost: every caller that forwards an id converts at the
>   boundary. Retires: a pending id passed where a device id was meant, which
>   the signed request would otherwise send.
> - **Declined (round 2).** Typestate over `Client` for enrol, finalize,
>   products. The three calls are made from different owners at different
>   times and the spec does not say the pending key is held by one value;
>   needs the user.

**Convergence.** The loop stops when a round's findings are all covered by
the decisions section, or when a round finds nothing. A hard cap of five
rounds, my call: hitting it means the spec is oscillating, and the skill
says so to the user rather than presenting round five as converged.

**Oscillation guard.** If a round's finding would reverse an adopted
decision, it is sorted as declined with the reason "reverses round N", and
goes to the user. The loop never undoes its own adoption.

## 7. What the user receives

One message, after convergence or the cap. In order:

1. One line: rounds run, findings adopted, items awaiting the user, and
   whether the loop converged or hit the cap.
2. The **adopted with a cost** items, most consequential first by
   `rust-design-review`'s ranking, each as the shape now in the spec, the
   cost, and what it retires, with the question "keep or flip" implied.
3. The **declined** items, same order, each with the reason.
4. One line pointing at the spec diff for the adopted bin.

Nothing else. No per-round narrative, no restatement of the spec.

**Applying the answers.** When the user replies, the orchestrator applies
each answer to the spec in the same conversation, without re-running the
loop. Keeping an "adopted with a cost" item changes nothing. Flipping one
restores the prior shape in the body and rewrites its decision entry as
declined with the user's reason. Overruling a declined item adopts it: the
orchestrator rewrites the shape and the entry, as it would have between
rounds. An answer that is itself a new design (neither keep nor flip) is
applied as the user states it and recorded as the user's decision. Then the
skill is done, and the next step is `writing-plans`.

## 8. Plan mode

A second, lighter mode, invoked after `writing-plans`, or on a direct request
to review a plan. One round, no loop. The reviewer reads the plan and the
spec it implements and reports:

- any task whose described code departs from a shape the spec decided,
  naming the decision;
- any decided shape no task implements;
- any task that re-introduces a prose rule the spec retired.

It makes no new design findings: a plan that is faithful to a reviewed spec
has nothing structural left to find. "Decided" means an entry in the spec's
decisions section or, where the spec has none, a shape the spec states
outright; the three checks run either way. When the spec has no decisions
section, the report leads with one line saying the spec has not been through
the loop and recommending it, then gives the three checks against the spec's
stated shapes. It does not edit the plan; the orchestrator reports and the
user or `writing-plans` fixes.

## 9. Changes to the other skills

**`rust-design-review`** gains one rule, in the Scope section: before pass 1,
look for a design decisions section in a spec under `docs/superpowers/specs`
that names the crate or module under review. A finding that matches a
recorded decision is reported in one line as decided in the spec, with the
entry quoted, and is not argued again. The exception is the same as pass 6
above: a decision that makes a stated goal unreachable is a full finding
that names the decision. This is the half of the loop that makes the
decisions section worth writing.

**`README.md`** gains the skill's row, and the install line is unchanged.

**No change** to `rust-api-design`, `rust-principles`, `rust-quality-gate`,
or `rust-embedded`.

## 10. Testing

The repo's convention is RED/GREEN scenarios under `tests/scenarios` with a
fixture under `tests/fixtures`. For this skill:

- **Fixture:** `tests/fixtures/enrolment_spec.md`, a design doc of about two
  pages with seeded flaws, one per pass: two `Option`s under an "exactly
  one" rule; a `Client::new(base: &str)`; a pending id and a device id both
  `String` in one signature; a corrupt store item "treated as no key"; a
  lock discipline stated in prose; a trait with one described implementor
  and no test double (my call: a `BrowserOpener` trait whose only
  implementor shells out, so the seeded flaw is not confusable with the
  decided trait below); a shape the spec's governing contract does not ask
  for (an `#[non_exhaustive]` on an error enum the contract enumerates
  exhaustively); and one stated decision with a reason that a naive reviewer
  would flag (the counter is persisted through a `CounterStore` trait with a
  file store and a memory store, rather than a typestate, because the two
  stores are both real). The fixture names a short governing contract of its
  own, half a page, under `tests/fixtures/enrolment_contract.md`. The
  fixture is derived from the quartermaster spec's shapes but names nothing
  from that repo.
- **Scenario:** `tests/scenarios/spec-review-baseline.md`, prompt only in
  the repo's newer style. RED: a fresh agent asked to review the fixture
  produces a list of nits and product questions, or starts proposing
  implementation. GREEN: with the skill loaded, every seeded flaw is a
  three-part finding in the right pass, the stated decision survives pass 6
  unflagged, and the fixture's decisions section after one round sorts the
  newtype finding as "adopted with a cost".
- **Plan mode scenario:** a second fixture, `tests/fixtures/
  enrolment_spec_reviewed.md`, hand-authored as the first fixture's
  post-review state with a decisions section, so the test does not depend on
  a prior live run mutating the first fixture; plus a plan against it with
  one task that re-introduces the two-`Option` shape. GREEN names the task
  and the decision.

The loop itself is exercised by running the skill on the fixture end to end
once and checking that round two produces no new finding against the shapes
round one adopted.

## 11. Constraints

- Skills in this repo are a single `SKILL.md` each, prose with tables, a
  Common Mistakes table of rationalizations and corrections, and cite the
  other skills by relative link. This one follows that form.
- The skill names subagents as the mechanism for rounds. When the harness
  has no subagent tool, the orchestrator runs each round itself after
  re-reading the spec from disk, and says in its report that rounds were not
  independent.
- Nothing in the skill depends on the superpowers plugin being installed.
  "After brainstorming, before writing-plans" is the expected place, but the
  input is a markdown file, and a direct request is a sufficient trigger.

## 12. Left to the builder

- Exact wording of the trigger description, within the one-line form the
  other skills use.
- The subagent prompt's exact text, as long as it carries the inputs in
  section 3, the passes in section 4, and the four-part finding in section 5,
  and withholds the previous round.
- Whether the decisions section's entries are a list or a table; the content
  per entry is fixed.
- How the orchestrator locates the existing crate the spec touches when the
  spec does not name a path; reading the spec's module names against the
  workspace is the expected approach.
- The Common Mistakes rows beyond the anti-goals in section 2, which must
  each appear as one.
