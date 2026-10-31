---
name: rust-spec-review
description: Use when a design doc or spec for Rust work has been approved and the implementation plan or any code is not yet written, or when asked to review a spec, design doc, or plan for a Rust crate
---

## Overview

This is [rust-design-review](../rust-design-review/SKILL.md) moved earlier:
the same method, run against a spec before any code exists. Every shape
finding that review makes at PR time (the prose invariant that should have
been a type, the two `Option`s that should have been an enum, the error
collapsed into an absence) is already visible in the spec that approved the
shape, and here it costs an edit to a paragraph instead of a refactor across
call sites and tests.

The question behind every finding is the one that review asks: **what does a
caller have to hold in their head that a better type would have held for
them?** In code, the evidence is a load-bearing comment. In a spec, it is a
load-bearing sentence: a "must", an "exactly one", a "the caller re-checks"
that exists because the shape beside it does not carry the rule.

It assumes [rust-principles](../rust-principles/SKILL.md) as its standard and
[rust-api-design](../rust-api-design/SKILL.md) for the type-driven and error
design criteria, and cites both rather than restating them, exactly as
[rust-design-review](../rust-design-review/SKILL.md) does. It does not replace
that review. Code can drift from a spec, and docs, comments, and gate evidence
exist only in code, so the PR-time review still runs; a spec that has been
through this skill should leave it only correctness-neutral findings.

## What A Round Reads

A round is one reviewer working the passes below once. It reads four inputs,
in this order:

1. **The rust-craft skills.** [rust-principles](../rust-principles/SKILL.md),
   [rust-api-design](../rust-api-design/SKILL.md), and
   [rust-design-review](../rust-design-review/SKILL.md), plus
   [rust-embedded](../rust-embedded/SKILL.md) when the spec targets `no_std`.
   A fresh reviewer has none of them loaded, so the orchestrator resolves
   each sibling `SKILL.md` from this skill's own base directory (the
   neighbouring entries under `skills/`) and hands the reviewer their
   absolute paths, and the reviewer reads them before it reads the spec.
   Paths, not an instruction to invoke the skills by name, because a reviewer
   may have no tool to invoke them with; and not a summary, because a second
   copy of the standard drifts from the first.
2. **The spec.** A superpowers-style design doc is the expected input, but
   any markdown that names Rust shapes qualifies.
3. **The existing public surface the spec touches.** When the spec changes a
   crate that already exists, read that crate's `pub` items in the modules
   the spec names, so a finding about a new shape accounts for the shapes
   beside it. Do not run [rust-quality-gate](../rust-quality-gate/SKILL.md):
   there is no new code to gate, and the existing crate's gate output is
   PR-time evidence.
4. **The spec's own decisions.** Its decisions section, if a previous round
   or the author wrote one, and any sentence where the author marks a call
   as decided. Pass 6 reads them against every finding.

Where the spec names something it answers to (a server contract, a wire
format another repo owns, a requirement set), pass 8 opens that document
too. It is read, not skimmed: the pass works from its sentences.

**A spec with no Rust shapes stops here.** If it names no types, signatures,
states, errors, or persisted shapes, the passes have nothing to bite on. Say
so in one paragraph and stop. Do not invent a surface in order to review it:
a reviewer who drafts the types and then critiques its own draft is
reviewing itself, not the spec.

## The Passes

Work these eight passes in order. Where a pass has a counterpart in
[rust-design-review](../rust-design-review/SKILL.md), the order mirrors it, so
a reader of both skills sees one method. Pass 6 audits passes 1 to 5 against
the spec's own decisions, and pass 8 runs last because it audits every remedy
the others proposed.

Read the whole spec as the surface, prose included. A shape named only in a
sentence ("each start carries a state the machine remembers") is named as
surely as one in a code block.

### 1. Named surface

*What must a caller learn before they can use this? What can they get wrong
that the types the spec names could have prevented?*

Enumerate every type, function, trait, field, constant, and signature the
spec names, and read each as a stranger who has only the spec. For each, ask
what invariant the spec hands the caller to maintain by hand: a `pub` field
beside a described validation rule, two `String`s in one signature or one
module that mean different things, a call order the prose states and no type
enforces, a fixed set of legal values described as a number or a string, an
argument the prose says must be well formed and the signature accepts as any
`&str`. The criteria are Type-Driven Design in
[rust-api-design](../rust-api-design/SKILL.md); the newtype judgement is
principle 5 of [rust-principles](../rust-principles/SKILL.md), so name the
confusable pair and where it meets.

A long parameter list is a surface finding, not a style finding, here as in
code.

### 2. States and transitions

*Can the spec's own representation reach a state the spec says is illegal?*

For every state table, state machine, lifecycle, or persisted record the
spec describes, compare what the prose permits with what the named shape
admits. Two `Option`s where the prose says "exactly one", a boolean beside an
enum that already encodes it, a state the table lists that no transition
reaches, a transition the prose allows that the table does not. A
tie-breaking rule ("a file with both prefers the device record") is the
signature of this finding: a rule for a case the type should have made
unrepresentable.

### 3. Error design

*Is every failure the spec describes actionable and distinguishable, and is
any of it an error that a better shape would delete?*

Judge every error enum, result, and "fails with" sentence against Error
Design in [rust-api-design](../rust-api-design/SKILL.md). Ask first which
failures a better argument type deletes. Then ask whether the spec has
collapsed an error into an absence ("a corrupt item is treated as no key"),
which hands the caller a recovery it never chose and hides the failure from
everything above it, or a success into an error code the caller must
string-match. The remedy is often a finer split of a named error, and a
split of a named enum is a breaking change to a named shape, so it is above
the structural bar in Findings.

### 4. Boundaries

*Who owns each piece of state, and which ownership, locking, or persistence
rule is the prose carrying that structure could carry instead?*

Ownership, concurrency, and persistence. Who owns each piece of state the
spec names, and is it owned in one place? Which sentences describe a lock
discipline ("the mutex is never held across a network call") or a write
discipline ("every write re-checks the record still names the id") that
structure could carry instead? Which persisted or wire shape admits values the
in-memory type refuses, so that a load path needs a validation the spec
forgot to describe? That mismatch between two named shapes is the finding,
and the missing validation is its evidence. A lock or a flow the spec never
mentions has no shape to review, and is below the structural bar.

### 5. Prose rules

*Which sentence is standing in for a shape the spec should have named?*

The counterpart of [rust-design-review](../rust-design-review/SKILL.md)'s
comments pass and, as there, the one most easily skipped, because a sentence
in a spec reads as a design decision rather than as evidence. Principle 4 of
[rust-principles](../rust-principles/SKILL.md) governs it.

Read every sentence of the form "must", "never", "always", "exactly one",
"is treated as", "the caller re-checks", "it is safe to", "by construction".
Each is a candidate. Ask the pass's question: **if the design were right,
would this sentence need to exist?** The sentence is never the finding; the
shape it is covering for is. A sentence that names a type the rest of the
spec does not use is evidence twice over: the author reached for the shape
and the surface did not follow.

Passes 2 to 4 each name sentence shapes of their own: the tie-break rule,
the error "treated as" an absence, the lock or write discipline. Those stay
with their passes. Every other candidate sentence belongs to this pass,
even when the shape it covers for is a state or a field an earlier pass also
read: a config loader's "a reader never sees a half-applied reload", or a
serial driver's "it is safe to call `send` from the interrupt handler".
Filing such a finding under an earlier pass is how this pass ends up
reporting nothing while the spec is full of rules. If it finds nothing, say
so in a clause, "no sentence in the spec is standing in for a missing
shape", because that is the only way a reader knows it ran.

### 6. Decisions audit

*Has the spec's author already decided this, with a reason?*

Read the spec's stated decisions: its decisions section, and any sentence of
the form "my call", "decided", or "the user asked for". A purpose clause
attached to a shape in the body ("the allocator takes its arena as a
parameter so the pool can be sized per caller") is not a decision. It is the
author's intent, and the other passes review whether the shape is what that
intent needs. A decision covers the question it decides and no other: a
decision about how a config file is versioned does not cover a finding about
which struct owns the loaded values.

For each finding from passes 1 to 5, and for each finding you were about to
write, check whether a recorded decision already covers it. If one does, the
finding is dropped, unless you can show the decision makes a stated goal or
anti-goal of the spec unreachable; then the finding names the decision and
the goal it breaks. A decision you would have made differently is not that.
This pass is what keeps a review from relitigating the author's calls and
what stops a loop of reviews from oscillating.

An adopted entry in `## Design decisions` is a decision like any other, and
it covers the shape it introduced. A finding whose subject is that shape is
dropped, with one exception: the finding shows that the shape does not
retire what the entry's `Retires:` says it retires. Then it is a **repair**,
not a new finding: name the entry, the obligation the caller still carries,
and the smallest change to the shape that makes the retirement true. A shape
you would have designed differently, or a further refinement of one that
already does its job, is covered.

A "Left to the builder" entry, or anything the spec explicitly hands to the
implementer, is never a finding's subject, and the spec's open questions for
its user are not findings either. Neither is dropped by this pass, because
neither should have been raised: do not propose a shape for something the
spec deliberately left unshaped.

Say in a clause which decisions this pass read and what each one covered,
including the ones that covered a finding nobody wrote. "The decision to
retry in the serial driver rather than in its callers covers the
retry-placement question; no goal breaks" is the evidence that the decision
was read rather than missed.

### 7. Cognitive load

*How many concepts must a caller carry to make one correct call, and which
proposed abstraction only moves complexity around?*

Take the spec as a whole. Count the concepts a caller must carry to make one
correct call against the described surface. Look for an abstraction the spec
proposes that relocates complexity rather than removing it: a trait with one
described implementor, a generic parameter the spec never varies, a builder
over three fields. By [rust-principles](../rust-principles/SKILL.md) that is
relocation, not reduction, and a second implementor the spec imagines is not
a second case. A trait whose second implementor is real, and named, is not
this finding. The purpose the spec gives an abstraction is weighed like any
other claim: ask whether that purpose needs the abstraction, or whether a
concrete shape would meet it.

### 8. Governing authority

*What does the authority the spec answers to ask for, and what wants the
remedy I just proposed?*

The counterpart of [rust-design-review](../rust-design-review/SKILL.md)'s
pass 7, and run last for the same reason: it audits the remedies the other
passes proposed. A spec often answers to something outside itself, and often
says so in its first paragraph: a server contract, a wire format another
repo owns, a requirement set, a standard. If nothing governs the spec, say so
in a clause, "no external authority governs this spec, so pass 8 is not
applicable", and move on.

Where one exists, open it and read it, then work both directions as that
pass does:

- **Shapes the spec proposes that the authority does not ask for, or that
  hedge a signal it wants loud.** An open enum over a set the authority
  enumerates exhaustively, a default that absorbs a new mandatory field.
  Quote the
  authority's sentence that settles it. General-library guidance such as the
  `#[non_exhaustive]` row in [rust-api-design](../rust-api-design/SKILL.md)'s
  Surface Hygiene is the instinct this pass exists to check: where the
  authority closes a set, a change to the set is a new version of the
  authority, and the break is the signal.
- **Your own remedies from passes 1 to 7.** Each must name what wants it: a
  goal of the spec, a sentence of the authority, a caller obligation an
  earlier pass named. **A remedy that cannot name what wants it is deleted,
  not ranked.** Name each deletion in one line of the round's output, the
  shape and the remedy, so the deletion outlives the round. The authority's
  own choices, however odd, are not this review's to relitigate.

## Findings

Each finding names four things, in this order:

1. **The design fact**, in the spec's own vocabulary, without a lint name or
   a file-and-line as its subject.
2. **The cost to the caller**: what they must remember, re-check, or can
   silently get wrong under the shape as specified.
3. **The change that retires it**, stated as a change to the spec, naming
   what the caller stops thinking about. Say it in prose at the spec's
   altitude: name the shape (an enum over a link's states, a newtype for
   one unit of measure) and the sentences it lets the spec delete. Do not
   write the code. A type definition or an `impl` in a finding is the
   builder's work done early, and the spec does not need it to change.
4. **The ergonomic cost of that change**, named concretely or stated as
   "none I can name". A newtype costs a conversion at every call site that
   forwards the value; a typestate costs a generic parameter in every
   signature that holds it; an enum over two `Option`s costs nothing. "Might
   be over-engineering" is not a cost, and principle 5 of
   [rust-principles](../rust-principles/SKILL.md) already forbids it as an
   answer. A finding whose fourth part is blank is not finished. A cost
   counts only when it lands on a caller at the described surface: a
   conversion at a call site, a generic parameter in a signature they write,
   a type they must learn. A cost internal to the implementing crate ("one
   place in the library", "the loader has to decide") is "none I can name".

**The structural bar.** A finding's subject is a shape the spec names or a
sentence the spec states, and its remedy either changes a named shape in a
way that would be a breaking change to retire once code exists, or adds or
removes an obligation on a caller of the described surface. Anything below
that bar is not a finding for this skill: a detail the spec does not fix, a
requirement or completeness gap, a naming or documentation point, a shape
you would have to invent first. Those go in the round's one closing
paragraph. A fresh reviewer always finds something; the bar is what makes
what it finds worth a round.

Each finding also names the section of the spec it is about, by heading, so
whoever edits the spec can find the place, and the one pass it belongs to,
with any other pass that also reaches it in a parenthesis. The pass is the
earliest whose question the finding answers, including the sentence shapes
passes 2 to 4 name as theirs, with two exceptions: a finding the
authority's text settles is pass 8's, and a prose rule no earlier pass names
is pass 5's. A sentence that is evidence for a finding does not move it to
pass 5: when a prose rule only corroborates a surface finding, the finding
stays with the surface pass.

A repair (pass 6) has the same four parts, with the entry it repairs as its
design fact.

One finding is one design fact with one remedy. Two facts about one type (a
config struct whose `pub` field bypasses a validation rule, and whose
defaults absorb a missing mandatory key) are two findings, because the
spec's author may adopt one and decline the other.

Rank by the three criteria under Output in
[rust-design-review](../rust-design-review/SKILL.md), most significant first.
Read "breaking to retire" as what retiring it costs once code exists: a shape
callers will build on ranks above a sentence one module will read. Rank a
prose-rule finding by the shape it stands in for, never by the sentence, as
that skill ranks a comment finding by the design fact behind it.

A round's output, which goes to the orchestrator under The Loop and never
straight to the user, is:

- the ranked findings, as prose, each a paragraph a reader can follow
  without the spec open, with its pass and its spec section named;
- a clause for every pass that found nothing, and pass 6's clause naming the
  decisions it read;
- one line for each remedy pass 8 deleted, naming the shape and the remedy;
- one short closing paragraph for anything out of scope or below the bar;
- and, when no pass finds anything, a paragraph saying the design is sound
  and why, instead of the findings.

Not a severity table, not a bulleted skeleton, and not ordered by the spec's
section order.

## The Loop

This skill is an orchestrator. It does not review the spec itself; it runs
rounds, sorts what each round returns, edits the spec, and stops when the
spec has absorbed every structural fix it can.

**Before round 1**, read the spec once. If it names no Rust shapes, the one
paragraph under What A Round Reads is the whole response: no round runs, and
nothing below applies.

**Each round is a fresh subagent.** Dispatch a new one every round; never
continue or message a previous round's reviewer. Its prompt carries only:

- this file's absolute path, with the instruction to read What A Round
  Reads, The Passes, and Findings and to return a round's output as Findings
  describes it;
- the absolute paths of the sibling skills listed under What A Round Reads,
  to be read before the spec;
- the spec's absolute path, and the paths of any existing crate modules the
  spec touches.

It never carries the previous round's findings, the orchestrator's sorting of
them, or a summary of what changed. The `## Design decisions` section in the
spec is the only memory that crosses rounds, and it is in the spec so the
reviewer reads it as the author would have written it.

**Sorting.** Apply this procedure to every finding a round returns, in
order. The first step that matches is the finding's one disposition, and
every finding gets exactly one. Your round notes, named below, are kept with
the round reports, never in the spec.

1. **Below the structural bar** (Findings), or out of scope: an aside. It
   gets no entry. Keep it in the round notes, with the round's closing
   paragraph, for the message's last sentence.
2. **Covered by a recorded decision.** The decision may be the author's
   own, under whatever heading or in whatever sentence they marked it, or
   any entry in `## Design decisions`, including a Declined entry (a pass-8
   deletion among them) and an adopted entry whose shape the finding is
   about. Write the covering decision's name in the round notes; nothing in
   the spec changes and no entry is added. A finding against a shape an
   adopted entry introduced (a refinement, a different design, a proposal
   to replace or retire that shape) is covered by that entry unless it is a
   repair, which is step 3. A fresh reviewer always has something to say
   about a shape the previous round introduced, and if each such finding
   were new the loop would cascade. A finding that names a stated goal or
   anti-goal the decision makes unreachable is not covered: it goes on to
   step 5.
3. **A repair** (pass 6): the finding shows that an adopted shape does not
   retire what its entry's `Retires:` says it retires. Revise that entry in
   place: same bin, same round number, its sentence and `Retires:` brought
   true, and `Revised in round N.` appended, once per revising round. Edit
   the shape in the spec to the minimum that makes the retirement true, and
   add a new shape only when the retirement cannot be made true without
   one. A repair never produces a new entry, with one exception: if it
   would reverse the entry, restoring what the entry retired, it is
   **Declined** with the reason "reverses round N".
4. **Would restore a shape an entry retired:** **Declined** with the reason
   "reverses round N", naming that entry's round. The loop never undoes its
   own adoption; only the user does.
5. **Otherwise, a new entry,** binned by the cost screen below:

| Bin | Rule | Effect on the spec | Reaches the user |
|---|---|---|---|
| **Adopted** | The reviewer named no ergonomic cost at the caller's surface ("none I can name", or a cost that fails the cost screen below) | The shape is rewritten in place; a one-line entry in the decisions section | No, beyond the diff |
| **Adopted with a cost** | The reviewer named a cost that lands on a caller at the described surface and passes the cost screen below | The shape is rewritten in place, provisionally; the decisions section records both the cost and the obligation it retires | Yes, with both sides, so the user can flip it |
| **Declined** | The remedy needs product knowledge the spec does not give, contradicts a recorded decision without breaking the goal the finding named, or is about something the spec left to the builder | Nothing changes in the body; the decisions section records the finding and why it was declined | Yes, so the user can overrule |

Separately, and not through the steps: each remedy a round's pass 8
deleted because nothing wants it becomes a **Declined** entry naming the
shape and the remedy, with the reason "no authority wants it", unless a
decision already records it, in which case name that decision in the round
notes. The next round's pass 6 then reads the deletion and does not propose
the remedy again; otherwise a deletion lasts one round. It reaches the user
with the other Declined items.

**The cost screen.** The reviewer's fourth part is a claim; the sort is the
decision. A cost is real only if the spec's author, shown the old shape and
the new one side by side, could plausibly choose the old one for
ergonomics. A one-time, local cost fails the screen: one more match arm, one
more argument carrying a value the caller already holds, one conversion in
one place. So does a cost internal to the implementing crate (Findings, part
4). A finding whose cost fails the screen is **Adopted**, not Adopted with a
cost. That is what keeps the keep-or-flip list to genuine trade-offs.

There is no fourth bin and no sixth step. A finding you cannot place in any
step is **Declined** with the reason "could not sort", which puts it in
front of the user. A finding that ends nowhere, in neither an entry, a
repair marker, nor the round notes, is the defect: it is neither in the spec
nor answered, so the next reviewer raises it again and the loop cannot
converge on it.

**Editing the spec.** Make each adopted change in the spec's own voice and at
its own altitude. Change the shape the finding names and the sentences that
described the obligation it retires. When the new shape makes a prose rule
unnecessary, delete the sentence: do not soften it, qualify it, or leave it
beside the shape that replaced it. Do not restyle unrelated prose, and add no
commentary to the body ("changed in review", "previously"): the history is
the decisions section and the diff. A "Left to the builder" entry is never
promoted into a decision, and an "Open with the user" question is never
answered by the loop. Do not commit; the user commits the revised spec with
the plan.

**The decisions section.** The heading is `## Design decisions`. If it is
absent, add it before "Left to the builder" and "Open with the user" when
those exist, otherwise at the end. A section the author wrote under another
heading (a "Decisions" section, say) stays where it is, unedited; rounds read
it as the author's decisions. Append one entry per sorted finding, in this
form:

- a bold bin name with the round in parentheses: `**Adopted (round N).**`,
  `**Adopted with a cost (round N).**`, or `**Declined (round N).**`;
- one sentence: for an adopted entry, the shape now in the spec; for a
  declined one, the finding;
- for **Adopted with a cost**, `Cost:` and the cost the reviewer named;
- for an adopted entry, `Retires:` and the obligation the caller no longer
  carries;
- for **Declined**, `Reason:` and why. `Reason:` is for Declined entries
  only; an adopted entry carries `Retires:`, never `Reason:`;
- after a repair, `Revised in round N.` at the end, once per revising round;
  after the user's answer (Applying The Answers), `Revised by the user.`

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
entry. Repairs and covered findings do not count: both are movements inside
decisions already taken, so a round whose only output is repairs and
coverings has converged, and so has a round that finds nothing. A round that
adds an entry is never the last, because nothing has yet read the spec it
left, so a spec with findings takes at least two rounds. The cap is five
rounds per invocation. If round five still produces a new entry, stop and
report that the loop **did not converge**: the spec is oscillating, and
round five is not presented as converged.

**Resume.** If the spec already has a `## Design decisions` section, read it
and append to it. Do not start a second section and do not rewrite earlier
entries except by a repair. Number rounds from the highest round already
recorded, counting `Revised in round N.` markers: a spec whose highest is
round 2 starts this invocation at round 3.

**Without a subagent tool**, run each round yourself. Before each round,
re-read the spec from disk and the sibling skills, and work the passes from
the file alone, not from what you remember of the previous round. Say in the
message's first line that rounds were not independent.

## What The User Receives

One message, after the loop converges or hits the cap, with four parts in
this order, an optional fifth, and one carve-out:

1. **One line:** rounds run (by number), findings adopted, items awaiting
   the user, and whether the loop converged or did not converge at the cap;
   and, under the fallback, that rounds were not independent.

   Then, ahead of the trade-offs, the carve-out that
   [rust-design-review](../rust-design-review/SKILL.md) makes in its Out Of
   Scope section, unchanged: **a soundness or security problem any round
   noticed is stated plainly and in full here**, with a note that it is
   outside this review's remit. Unsound `unsafe`, a data
   race, a memory-safety hole, a leaked credential. It is never compressed
   into the asides sentence, and an ordinary logic bug does not become
   soundness because it is serious. Absent when no round noticed one.
2. **The Adopted with a cost items**, most consequential first by the
   ranking under Output in
   [rust-design-review](../rust-design-review/SKILL.md), each as the shape
   now in the spec, its cost, and what it retires. The question "keep or
   flip" is implied; do not ask it per item.
3. **The Declined items**, in the same order, each with its reason.
4. **One line** pointing at the spec's diff for the adopted bin.
5. **Optionally, one sentence** collecting the rounds' asides (a bug noticed
   in the spec's logic, a requirement gap, a below-bar point), on the budget
   [rust-design-review](../rust-design-review/SKILL.md) gives its closing
   paragraph. Present only when a round produced asides; it never leads.
   One sentence, not one per aside: when there are more than it carries,
   name the most consequential and end "and N more in the round reports",
   which you keep and show if the user asks.

Nothing else: no per-round narrative, no list of the plain Adopted items
beyond the diff, no restatement of the spec. Parts 2
and 3 carry only this invocation's entries. When a part is empty, say so in
a clause ("nothing awaits a keep-or-flip"). When the first round
produces no new entry, the first line says the spec was already converged.

## Applying The Answers

When the user replies, apply each answer to the spec in the same
conversation, without re-running the loop:

- **Keep** an Adopted with a cost item: nothing changes.
- **Flip** one: restore the prior shape in the body, and the sentences the
  change deleted, and rewrite its entry as `**Declined (round N).**`, keeping
  the entry's original round number, with the finding as its sentence,
  `Reason:` the user's reason, and `Revised by the user.` appended.
- **Overrule** a declined item: adopt it, rewriting the shape and the entry
  as you would have between rounds (`Retires:`, and `Cost:` when the user
  accepted one), keeping the round number and appending `Revised by the
  user.`
- **A new design** (neither keep nor flip): apply it as the user states it,
  and rewrite the entry as `**Adopted (round N).**`, keeping the original
  round number, stating the user's shape, with `Retires:` the obligation it
  retires and `Revised by the user.` appended. The marker is what records
  that the shape is the user's; an adopted entry never carries `Reason:`.

> - **Adopted (round 2).** `Queue::push` takes a `Priority`, and `Queue`
>   also offers `push_default`, which uses the middle priority, so a caller
>   with no configured priority converts nothing. Retires: a priority of 0
>   or 12 reaching the scheduler. Revised by the user.

Then this skill is done. The next step is the implementation plan
(`superpowers:writing-plans`, where that plugin is installed).

## Out Of Scope

Name these if you see them, in one line, and move on:

| Out of scope | Belongs to |
|---|---|
| Product correctness, completeness of requirements, whether it is the right thing to build | brainstorming's builder check and the user's own judgement |
| Specs with no Rust surface: a frontend, a protocol with no Rust shapes | not this skill; say so in the one paragraph and stop |
| PR-time review of the code that implements the spec | [rust-design-review](../rust-design-review/SKILL.md), which still runs |
| Correctness bugs visible in the spec's logic | the built-in `/code-review` once code exists; until then, an aside |
| Anything below the structural bar in Findings: a detail the spec does not fix, a completeness gap, naming or docs, a shape you would have to invent | the round's closing paragraph; the builder or the plan |

Everything out of scope shares **one short closing paragraph**, after the
last finding, on the same budget as
[rust-design-review](../rust-design-review/SKILL.md)'s: not a section, not a
heading, not a paragraph each. A product question or a bug in the spec's
logic is concrete and argues well for its own space, and the moment it leads
the review, the review has become the thing it is defined against. That
skill's soundness and security carve-out applies here unchanged: in the
round's output, and in the message the orchestrator sends (What The User
Receives), where such a problem is stated in full ahead of the trade-offs.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "The spec says it in prose, so the design handles it." | A sentence that states a rule is the evidence that no shape holds it. Pass 5 asks whether the sentence would need to exist if the design were right; the shape it covers for is the finding. |
| "That's a product decision, I'll leave it." | Only if the spec says so with a reason. A shape the author chose without stating why is a shape, and passes 1 to 5 review it. Pass 6 drops findings for recorded decisions, not for decisions you imagined. |
| "That decision is wrong, so I'll raise it anyway." | A decision with a stated reason is an authority, not a finding. Pass 6 drops the finding unless you can name the stated goal or anti-goal the decision makes unreachable. Preferring a different shape is not that. |
| "The spec leaves that shape to the builder, but it is underspecified, so I'll propose one." | Underspecified is what "Left to the builder" means. An entry on that list is never a finding's subject; proposing its shape reviews a decision the spec deliberately did not make. |
| "`#[non_exhaustive]` is good practice on a public error enum." | Not where the governing contract enumerates the set and calls it closed. There the attribute turns the contract's version change into a silent wildcard arm. Pass 8 quotes the contract's sentence and asks what wants the enum open. |
| "The spec is thin, so I'll propose the surface and review that." | Then you are reviewing your own draft. A spec with no Rust shapes gets one paragraph saying so, and the review stops. A thin spec with some shapes is reviewed on the shapes it names. |
| "This newtype costs nothing." | A newtype costs a conversion at every call site that forwards the value, and principle 5 says that is real. Name the cost; "none I can name" is for changes such as an enum over two `Option`s, where it is true. |
| "I'll write out the corrected types so the finding is concrete." | The change is to the spec, at the spec's altitude. Name the shape and what it retires; a type definition in a finding is the plan's work done early. |
| "I found a gap in the requirements, so I led with it." | Requirement completeness is out of scope. It goes in the one closing paragraph, after the last finding. |
| "The fix was clearly right, so I absorbed the cost silently." | A cost the reviewer named and the loop absorbed without recording is the one outcome worse than not running the skill. It is Adopted with a cost: the entry names the cost, and the user sees it. |
| "Round three disagrees with round one, so I'll go with round three." | A later round is not a better authority. A finding that would restore a shape an adopted entry retired is Declined with "reverses round N" and goes to the user; the loop never undoes its own adoption. A finding that merely wants the adopted shape different is covered by the entry, not declined. |
| "Five rounds ran, so it converged." | Convergence is a round that produces no new entry. Round five with a new entry is the cap, and the message says the loop did not converge. |
| "I'll summarise each round for the user." | The user gets one message in four parts. Per-round narrative is the noise the loop exists to absorb; the diff and the decisions section are the history. |
| "The subagent can't see the skills, so I'll paraphrase them in the prompt." | Hand it the absolute paths and tell it to read them. A paraphrase is a second copy of the standard, and it drifts. |
| "The spec already has decisions, so I'll start a fresh section." | Read the existing `## Design decisions` section and append to it, numbering rounds from its highest. A second section splits the only memory the rounds share. |
| "I'll give round two the round-one findings so it doesn't repeat them." | Then round two reviews round one, not the spec. The decisions section is the only memory across rounds; a covered finding is caught by sorting, not by priming. |
| "The new shape makes the rule redundant, but I'll keep a softer version of the sentence." | Delete it. A softened sentence beside the shape that replaced it tells the next round the shape does not hold the rule. |
| "One more match arm is a cost, so it is Adopted with a cost." | A cost counts when it lands on a caller at the described surface. A match arm inside the library, or one the caller already writes, is "none I can name"; calling it a cost fills the keep-or-flip list with non-choices. |
| "The spec doesn't say how this failure, lock, or flow works, so I'll propose the shape." | That is a completeness gap, below the structural bar. It goes in the closing paragraph. A finding's subject is a shape the spec names or a sentence it states. |
| "Round three found a better shape for what round one adopted, so I'll add it." | An adopted entry is a decision and covers the shape it introduced. Only a repair (the shape does not retire what the entry says) changes it, by revising that entry in place; a refinement is covered and gets nothing. |
| "This finding came up before and I'm not sure what covers it, so I'll let it go." | The sort is total: an aside, a covering decision named in the round notes, a repair marker, or a new entry. A finding that ends nowhere comes back next round, and the loop cannot converge on it. |
| "A round noticed a data race in the spec's design, but bugs are asides, so it goes in the one sentence." | A soundness or security problem is the carve-out, as in rust-design-review: state it in full in the message, ahead of the trade-offs, and name that it is outside the remit. Only an ordinary logic bug is an aside. |
