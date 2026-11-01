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
- **The document the spec names as governing it**, when there is one. Pass
  8 opens it, so the orchestrator carries its path in the reviewer's input
  list beside the spec's.

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
code the caller must string-match. The remedy is often a finer split of a
named error, and a split of a named enum is a breaking change to a named
shape, so it is above the structural bar.

### 4. Boundaries

Ownership, concurrency, and persistence. Who owns each piece of state the
spec names? Which sentences describe a lock discipline ("the mutex is never
held across a network call") or a write discipline ("every write re-checks
the record still names the id") that structure could carry instead? Which
persisted or wire shape admits values the in-memory type refuses, so that a
load path needs a validation the spec forgot to describe? That mismatch
between two named shapes is the finding, and the missing validation is its
evidence; a lock or a flow the spec never mentions has no shape to review and
is below the structural bar (section 5).

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
calls. An adopted entry in the decisions section is a decision like any
other and covers the shape it introduced; the one exception is a **repair**
(section 6), a finding that shows the shape does not retire what the entry
says it retires.

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
absorbs a new mandatory field). A remedy from passes 1 to 7 is wanted by the
caller obligation its own second part names, so the pass deletes one only
when its sole justification is a future the authority does not promise (an
open set, a hedge, a field the authority does not define), or when the
authority forbids it. Such a remedy is deleted here, not ranked, and the
round's output names each deletion in one line, the shape and the remedy, so
the orchestrator can record it (section 6). If nothing governs the spec, the
review says so in a clause.

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
   is not a cost; `rust-principles` already forbids that answer. A cost
   counts only when it lands on a caller at the described surface: a
   conversion at a call site, a generic parameter in a signature they write,
   a type they must learn. A cost internal to the implementing crate ("one
   place in the library", "the loader has to decide") is "none I can name".

Each finding also carries the section of the spec it is about, so the
orchestrator can edit the right place. A repair carries the decisions entry
it repairs as its design fact.

**The structural bar.** A finding's subject is a shape the spec names or a
sentence the spec states, and its remedy either changes a named shape in a
way that would be a breaking change to retire once code exists, or adds or
removes an obligation on a caller of the described surface. Anything below
that bar is not a finding for this skill: a detail the spec does not fix, a
requirement or completeness gap, a naming or documentation point, a shape
the reviewer would have to invent first. Those go in the round's closing
paragraph. The bar is what lets the loop converge: a fresh reviewer always
finds something, and only structural findings are worth a round.

**Landmines and refinements** (amended 2026-11-01, at the user's request,
after real specs ran to the cap). Each finding is labelled by its remedy. A
**landmine** changes a shape the spec names, stated as before and after (a
type, a signature, a field's type, an enum's variants or exhaustiveness, an
error's split,
which value owns a piece of state, a trait added or removed), and retires a
mistake a caller can make silently: a state the spec calls illegal, two
confusable values, a failure collapsed into an absence, a success or
another failure the caller handles differently, or a
rule the caller keeps by hand. Everything else above the bar is a
**refinement**: a change to what the spec says happens, to its rules or
reasoning, or a shape change that removes a redundancy, narrows a
visibility or simplifies without retiring such a mistake. The kind follows
the remedy, not the pass, and the orchestrator decides it from the entry,
not the reviewer's label; a Declined entry is never a landmine. Both kinds
are sorted, binned and edited the same way, so a clear refinement is
applied without asking and an ambiguous one reaches the user; only
landmines count toward convergence (section 6). The user's reason: prose,
reasoning and one more reshape can be litigated indefinitely, so this phase
catches the structural landmines and leaves the smaller refinements to the
PR-time review. A first version gated on every API change; on a real spec
it still hit the cap with one API entry a round, several of them tidy
reshapes.

A round's output is the ranked findings as prose, a clause for any pass that
found nothing, and a one-line closing paragraph for anything out of scope
or below the structural bar, on `rust-design-review`'s budget. A round that
finds nothing says so in a paragraph.

## 6. The loop

The skill is an orchestrator. Each round is a **fresh subagent** that
receives the inputs in section 3 and the skill file itself, read except for
the orchestrator's own sections (the loop, the user message, applying the
answers, plan mode), so the passes, the finding shape, the out-of-scope
budget and the reviewer's common mistakes arrive together, and returns
findings. It
does not receive the previous round's findings or the orchestrator's sorting
of them; the decisions section in the spec is the only memory that crosses
rounds, and it is in the spec so the reviewer reads it as the spec's author
would have written it.

Between rounds the orchestrator sorts every finding by one ordered
procedure, applied to every finding a round returns, first match wins. Each
finding gets exactly one disposition. The orchestrator's round notes are kept
with the round reports in the session's scratch space, outside the repo:
never beside the spec, never committed, never in the spec.

1. **Below the structural bar** (section 5), or out of scope: an aside, kept
   in the round notes with the round's closing paragraph. No entry. An
   out-of-scope soundness or security point is still an aside here, and the
   user message carries it in full, ahead of the trade-offs (section 7).
2. **Covered by a recorded decision**: the author's own, under whatever
   heading, or an entry in the decisions section, including a Declined entry
   from a pass-8 deletion and an adopted entry whose shape the finding is
   about. The covering decision is named in the round notes; nothing in the
   spec changes. An adopted entry is a decision like any other. A fresh
   reviewer always finds something to say about a shape the previous round
   introduced, and if each such finding were new, the loop would cascade:
   every adoption names new shapes, and every new shape draws findings. So a
   finding against a shape an adopted entry introduced, whether a
   refinement, a different design, or a proposal to replace or retire it, is
   covered by that entry unless it is a repair (step 3) or would restore
   what the entry retired (step 4). A finding is about the shape its remedy
   changes, not the shapes its evidence mentions. A first version keyed
   covering on every shape a finding touched; on a real spec almost every
   late finding also touched an author's shape, nothing was covered, and
   the loop cascaded to the cap.
   A finding whose remedy changes a shape
   stated only inside the author's own decisions section is covered here
   too, since the loop does not edit that section.
   A finding that shows the decision makes a stated goal or anti-goal
   unreachable is not covered and goes on to step 5, where a goal-breaking
   finding against the author's own decision, or against an entry in the
   decisions section, is Declined, never Adopted: adopting it would leave
   two contradictory adopted entries, and the user decides between them.
3. **A repair**: the finding shows an adopted shape does not retire what its
   entry says it retires. The orchestrator revises the existing entry in
   place, same round, with "Revised in round N." appended, and
   edits the spec's shape to the minimum that makes the retirement true. The
   bin stays unless the repair lands a cost on a caller at the described
   surface that passes the cost screen; then the entry moves to adopted with
   a cost and gains `Cost:`, so it reaches the keep-or-flip list. A
   repair never introduces a new shape unless the retirement cannot be made
   true without one, and never produces a new entry, with two exceptions:
   a repair that would reverse its entry, restoring what the entry retired,
   is Declined with the reason "reverses round N"; and an entry is repaired
   once, so a second repair finding against it is Declined with the reason
   "repaired twice; needs the user", which ends the chain. Fresh eyes on an
   adoption that failed is the point of a fresh round; a refinement of one
   that works is not, and a third opinion on a repair is the user's to give.
4. **Would restore a shape an entry retired**: Declined with the reason
   "reverses round N", and it goes to the user. The loop never undoes its own
   adoption. This is the oscillation guard, and it applies only here and in
   step 3: a finding that is merely against a shape an entry introduced is
   step 2.
5. **Otherwise, a new entry**, binned by the cost screen:

| Bin | Rule | Effect on the spec | Reaches the user |
|---|---|---|---|
| **Adopted** | The reviewer named no ergonomic cost at the caller's surface (a cost internal to the implementing crate counts as none, and so does one that fails the cost screen) | The shape is rewritten in place; a one-line entry in the decisions section | No, beyond the diff |
| **Adopted with a cost** | The reviewer named a real cost, one that lands on a caller at the described surface and passes the cost screen | The shape is rewritten in place, provisionally; the decisions section records both the cost and the obligation it retires | Yes, with both sides, so the user can flip it |
| **Declined** | The remedy needs product knowledge the spec does not give; or the finding breaks a stated goal by way of the author's own decision, with the reason "breaks <the goal named>; the author decided otherwise; needs the user", since the first anti-goal forbids the loop rewriting the author's decisions; or it breaks a stated goal by way of an entry in the decisions section, with the reason "breaks <the goal named>; changes round N's shape; needs the user", since the loop never holds two adopted shapes for one question | Nothing changes in the body; the decisions section records the finding and why it was declined | Yes, so the user can overrule |

**Pass 8 deletions are remembered**, separately from the steps. A remedy a
round's pass 8 deleted because no authority wants it is recorded as a
**Declined** entry naming the shape and the remedy, with the reason "no
authority wants it", unless a decision already records it, so the next
round's pass 6 sees it. It reaches the user with the other declined items.
Otherwise a deletion lasts one round, and a later reviewer proposes the same
remedy as new.

**The cost screen.** The reviewer's fourth part is a claim; the
orchestrator's sort is the decision. A cost is real only if the spec's
author, shown both shapes side by side, could plausibly choose the old one
for ergonomics. A one-time, local cost fails the screen: one more match arm,
one more argument carrying a value the caller already holds, one conversion
in one place. Those are "none I can name", and the finding is adopted, not
adopted with a cost. This is what keeps the keep-or-flip list to genuine
trade-offs. A conversion or construction that recurs at every caller of a
public signature passes the screen, however the reviewer describes it: it is
the newtype tension that `rust-principles` principle 5 leaves to per-API
judgment, which is the author's call, so such a finding is adopted with a
cost, never plain adopted.

The orchestrator never invents a fourth bin or a sixth step. A finding it
cannot place is declined with "could not sort" as the reason, which puts it
in front of the user. A finding that ends nowhere is neither in the spec nor
answered, and the next round raises it again.

**Editing the spec.** Adopted changes are made in the spec's own voice and
at the spec's own altitude. The orchestrator changes the shape the finding
names and the sentences that described the obligation it retires; the prose
rule a finding was about is deleted, not softened, when the new shape makes
it unnecessary, except a Goals, Non-goals or Anti-goals bullet, which stays
because pass 6 reads decisions against it. It does not restyle unrelated
prose and does not add commentary to the body. A "Left to the builder"
entry is never promoted into a decision by the loop.

**The decisions section.** Appended to the spec under a heading `## Design
decisions` if absent, before "Left to the builder" and "Open with the user"
when those exist. One entry per decision: the bin and the round in bold, one
sentence (the shape now in the spec for an adopted entry, the finding for a
declined one), `Cost:` for an adopted-with-a-cost entry, `Retires:` for any
adopted entry, and `Reason:` for a declined entry only; an adopted entry
never carries `Reason:`. A repair appends "Revised in round N."; the user's
answer appends "Revised by the user." Example:

> - **Adopted (round 1).** A job's retry policy is an enum of never, a fixed
>   count, or until a deadline, never a count where zero means forever.
>   Retires: the "a retry count of 0 retries without limit" rule.
> - **Adopted with a cost (round 2).** `Queue::push` takes a `Priority`
>   parsed once from the 1-to-9 range, not a `u8`. Cost: every caller that
>   forwards a priority from its own config converts at the boundary.
>   Retires: a priority of 0 or 12 reaching the scheduler, which would
>   otherwise clamp it silently.
> - **Declined (round 2).** `pop` returns a `Lease` that acknowledges the job
>   when dropped. Reason: the spec has a separate process acknowledge a job
>   after it reports, so no one value holds a job from pop to acknowledge;
>   needs the user.

**Convergence.** The loop has converged after a round that produces no new
landmine (section 5). Refinements, repairs, covered findings and Declined
entries (a pass-8 deletion included) do not count: refinements are applied
but cannot buy a round, repairs and coverings are movements inside
decisions already taken, and a Declined entry changes nothing in the spec.
A round that adds a landmine is not the last, since nothing has read the
spec it left. The converging round's refinements and repairs are the edits
no fresh round rereads; that trade is accepted, and `rust-design-review` at
PR time covers them. A hard cap of five rounds per invocation, my call. A
fresh reviewer has a find rate of its own, so the cap can still be hit on
honest runs; what the skill owes the user at the cap is an account of what
happened, not a blanket verdict.

**At the cap**, the first line of the user message gives the per-round
new landmine counts, pass-8 Declined entries excluded as for convergence, and
names one of two things. **Tapering:** the counts are non-increasing over
the last three rounds and the last round added at most two; the line then
names the last two rounds' entries, the ones the user should glance at.
**Oscillating:** a "repaired twice" Declined entry or a "reverses round N"
Declined entry was written in this invocation; it is checked first. The word
describes the reviewers' proposals, not the spec, since a guard that held
means the spec did not move. A run that is neither is reported with its
counts alone. None is called
converged.

**Resume.** A spec that already has a decisions section, from a previous
invocation, is read and appended to: one section, earlier entries left as
they are except by a repair. Round numbers continue from the highest round
the section records, counting "Revised in round N." markers as well as the
rounds in the bold bin labels, so an invocation after a spec whose highest
is round 2 starts at round 3, and the cap of five counts this invocation's
rounds only.

## 7. What the user receives

One message, after convergence or the cap. In order:

1. One line: rounds run, findings adopted, items awaiting the user, the
   landmines and refinements added (two counts), and whether the loop converged or stopped at the cap, and at the cap whether
   it was tapering or oscillating as section 6 defines them, with the
   per-round counts and, when tapering, the last two rounds' entries by
   name. Then, ahead of the trade-offs,
   `rust-design-review`'s soundness and security carve-out, unchanged: a
   soundness or security problem any round noticed (unsound `unsafe`, a data
   race, a memory-safety hole, a leaked credential) is stated plainly and in
   full, with a note that it is outside the remit, never compressed into the
   asides sentence in item 5. Absent when no round noticed one.
2. The **adopted with a cost** items, most consequential first by
   `rust-design-review`'s ranking, each as the shape now in the spec, the
   cost, and what it retires, with the question "keep or flip" implied.
3. The **declined** items, same order, each with the reason.
4. One line pointing at the spec diff for the adopted bin. It may name, in
   one clause each, any adopted entry whose change the product's end user
   would observe (a new state the user sees, a failure that used to be
   silent), so a behaviour change does not hide behind "see the diff". It
   is not a bin; the entry stays adopted and asks nothing.
5. Optionally, one sentence collecting the rounds' out-of-scope asides (a
   bug noticed in the spec's logic, a requirement gap, a below-bar point),
   on the budget `rust-design-review` gives its closing paragraph. Present
   only when a round produced asides. It is one sentence however many there
   are: past what one sentence carries, it ends "and N more in the round
   reports", where N counts distinct asides rather than one aside repeated
   across rounds, and the round reports, outside the repo, are where they
   live.

Nothing else. No per-round narrative, no restatement of the spec.

**Applying the answers.** When the user replies, the orchestrator applies
each answer to the spec in the same conversation, without re-running the
loop. Keeping an "adopted with a cost" item changes nothing. Flipping one
restores the prior shape in the body and the sentences the change deleted,
taken from the diff or git history, and rewrites its decision entry as
declined, keeping the entry's round number, with `Reason:` the user's reason
and "Revised by the user." appended. Overruling a declined item adopts it:
the orchestrator rewrites the shape and the entry as it would have between
rounds, keeping the round number and appending the same marker. An answer
that is itself a new design (neither keep nor flip) is applied as the user
states it and its entry rewritten as adopted, keeping the original round
number, with `Retires:` and "Revised by the user." appended; the marker, not
a `Reason:`, records that the shape is the user's. Then the skill is done,
and the next step is `writing-plans`.

## 8. Plan mode

A second, lighter mode, invoked after `writing-plans`, or on a direct request
to review a plan. One round, no loop. The reviewer reads the plan and the
spec it implements: the one the request names, else the one the plan names
(in a superpowers plan, its `**Spec:**` line); if neither names one, it
asks. When the two disagree, it checks against the spec the request names
and says in a clause that the plan names another. It reports:

- any task whose described code departs from a shape the spec decided,
  naming the decision;
- any decided shape no task implements;
- any task that re-introduces a prose rule an entry's `Retires:` names.

Each item names the plan task and quotes the decision. One plan fact is one
item: a task that swaps a decided shape for the one it retired is reported
once, under the first check, with the restored rule named in it.

It makes no new design findings: a plan that is faithful to a reviewed spec
has nothing structural left to find, and a choice the spec left to the
builder is the plan's to make. "Decided" means an entry in the spec's
decisions section or a decision of the author's own, or, where no entry
covers a shape, the shape the spec states outright; the three checks run
either way. When the spec has no decisions section, the report leads with
one line saying the spec has not been through the loop and recommending it,
then gives the three checks against the spec's stated shapes. It does not
edit the plan; the orchestrator reports and the user or `writing-plans`
fixes.

## 9. Changes to the other skills

**`rust-design-review`** gains one rule, in the Scope section: before pass 1,
look for a design decisions section in a spec under `docs/superpowers/specs`
that names the crate or module under review, or in a spec handed over with
the request. A decision applies only while the code is the shape its entry
decided, and covers the question it decides and no other. A finding that
matches a recorded decision is reported in one line as decided in the spec,
with the entry quoted, and is not argued again. Three findings are written
in full and name the entry: code that departs from the decided shape; a
repair, where the decided shape does not do what its entry says (it does not
retire what `Retires:` says); and, as in pass 6 above, a decision that makes
a stated goal or anti-goal unreachable, which names the goal it breaks. This
is the half of the loop that makes the decisions section worth writing.

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
  a prior live run mutating the first fixture; plus a plan against it,
  `tests/fixtures/enrolment_plan.md`, with three departures: one task
  re-introduces the two-`Option` shape, one takes a bare `String` device id,
  and no task implements the decided write method. GREEN names each task
  and quotes each decision, and nothing else.

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
