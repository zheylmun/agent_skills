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
everything above it, or folded a non-failure into an error the caller must
match past.

### 4. Boundaries

*Who owns each piece of state, and which ownership, locking, or persistence
rule is the prose carrying that structure could carry instead?*

Ownership, concurrency, and persistence. Who owns each piece of state the
spec names, and is it owned in one place? Which sentences describe a lock
discipline ("the mutex is never held across a network call") or a write
discipline ("every write re-checks the record still names the id") that a
shape could carry: a guard that cannot cross the call, a write that takes
the id it is guarded by? Which persisted or wire shape admits values the
in-memory type refuses, so that a load path needs a validation the spec
forgot to describe?

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
spec does not use (a newtype in prose beside a `&str` in the signature) is
evidence twice over: the author reached for the shape and the surface did
not follow.

Passes 2 to 4 each name sentence shapes of their own: the tie-break rule,
the error "treated as" an absence, the lock or write discipline. Those stay
with their passes. This pass owns the rest, most often a guarantee ("so a
stale reply is never applied", "it is safe to") kept by a remembered side
value or by every handler remembering one check. Such a finding is this
pass's even though the shape it covers is a state or a field; filing it
under an earlier pass is how this pass ends up reporting nothing while the
spec is full of rules. If it finds nothing, say
so in a clause, "no sentence in the spec is standing in for a missing
shape", because that is the only way a reader knows it ran.

### 6. Decisions audit

*Has the spec's author already decided this, with a reason?*

Read the spec's stated decisions: its decisions section, and any sentence of
the form "my call", "decided", or "the user asked for". A purpose clause
attached to a shape in the body ("behind a trait, so the core stays
portable", "left open so new cases do not break callers") is not a
decision. It is the author's intent, and the other passes review whether the
shape is what that intent needs. A decision covers a finding only when its
reason answers the finding: it says why this shape and not the one the
finding proposes. A reason that a different shape would serve equally well
covers nothing.

For each finding from passes 1 to 5, and for each finding you were about to
write, check whether a recorded decision already covers it. If one does, the
finding is dropped, unless you can show the decision makes a stated goal or
anti-goal of the spec unreachable; then the finding names the decision and
the goal it breaks. A decision you would have made differently is not that.
This pass is what keeps a review from relitigating the author's calls and
what stops a loop of reviews from oscillating.

A "Left to the builder" entry, or anything the spec explicitly hands to the
implementer, is never a finding's subject, and the spec's open questions for
its user are not findings either. Neither is dropped by this pass, because
neither should have been raised: do not propose a shape for something the
spec deliberately left unshaped.

Say in a clause which decisions this pass read and what each one covered,
including the ones that covered a finding nobody wrote. "The `X` decision
covers the trait-over-typestate choice; no goal breaks" is the evidence that
the decision was read rather than missed.

### 7. Cognitive load

*How many concepts must a caller carry to make one correct call, and which
proposed abstraction only moves complexity around?*

Take the spec as a whole. Count the concepts a caller must carry to make one
correct call against the described surface. Look for an abstraction the spec
proposes that relocates complexity rather than removing it: a trait with one
described implementor, a generic parameter the spec never varies, a builder
over three fields. By [rust-principles](../rust-principles/SKILL.md) that is
relocation, not reduction, and "so a later caller can supply their own" is
an imagined second case, not a real one. A trait whose second implementor is
real, and named, is not this finding. Nor is a goal the trait is said to
serve a defence of it when a concrete shape serves the same goal: keeping a
dependency out of a crate is a reason to keep the call out of the crate, not
a reason for a trait.

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
  enumerates exhaustively, a default that absorbs a new mandatory field, a
  catch-all variant for outcomes the authority says cannot occur. Quote the
  authority's sentence that settles it. General-library guidance such as the
  `#[non_exhaustive]` row in [rust-api-design](../rust-api-design/SKILL.md)'s
  Surface Hygiene is the instinct this pass exists to check: where the
  authority closes a set, a change to the set is a new version of the
  authority, and the break is the signal.
- **Your own remedies from passes 1 to 7.** Each must name what wants it: a
  goal of the spec, a sentence of the authority, a caller obligation an
  earlier pass named. **A remedy that cannot name what wants it is deleted,
  not ranked.** The authority's own choices, however odd, are not this
  review's to relitigate.

## Findings

Each finding names four things, in this order:

1. **The design fact**, in the spec's own vocabulary, without a lint name or
   a file-and-line as its subject.
2. **The cost to the caller**: what they must remember, re-check, or can
   silently get wrong under the shape as specified.
3. **The change that retires it**, stated as a change to the spec, naming
   what the caller stops thinking about. Say it in prose at the spec's
   altitude: name the shape (an enum over the record's states, a newtype
   for one kind of id) and the sentences it lets the spec delete. Do not
   write the code. A type definition or an `impl` in a finding is the
   builder's work done early, and the spec does not need it to change.
4. **The ergonomic cost of that change**, named concretely or stated as
   "none I can name". A newtype costs a conversion at every call site that
   forwards the value; a typestate costs a generic parameter in every
   signature that holds it; an enum over two `Option`s costs nothing. "Might
   be over-engineering" is not a cost, and principle 5 of
   [rust-principles](../rust-principles/SKILL.md) already forbids it as an
   answer. A finding whose fourth part is blank is not finished.

Each finding also names the section of the spec it is about, by heading, so
whoever edits the spec can find the place, and the one pass it belongs to,
with any other pass that also reaches it in a parenthesis. The pass is the
earliest whose question the finding answers, including the sentence shapes
passes 2 to 4 name as theirs, with two exceptions: a finding the
authority's text settles is pass 8's, and a prose rule no earlier pass names
is pass 5's. A sentence that is evidence for a finding does not move it to
pass 5: a newtype named in prose beside a `&str` in a signature is evidence
for pass 1's finding about that value.

One finding is one design fact with one remedy. Two facts about one type (an
enum left open against a closed set, and a non-failure folded into the same
enum) are two findings, because the spec's author may adopt one and decline
the other.

Rank by the three criteria under Output in
[rust-design-review](../rust-design-review/SKILL.md), most significant first.
Read "breaking to retire" as what retiring it costs once code exists: a shape
callers will build on ranks above a sentence one module will read. Rank a
prose-rule finding by the shape it stands in for, never by the sentence, as
that skill ranks a comment finding by the design fact behind it.

A round's output is:

- the ranked findings, as prose, each a paragraph a reader can follow
  without the spec open, with its pass and its spec section named;
- a clause for every pass that found nothing, and pass 6's clause naming the
  decisions it read;
- one short closing paragraph for anything out of scope;
- and, when no pass finds anything, a paragraph saying the design is sound
  and why, instead of the findings.

Not a severity table, not a bulleted skeleton, and not ordered by the spec's
section order.

## Out Of Scope

Name these if you see them, in one line, and move on:

| Out of scope | Belongs to |
|---|---|
| Product correctness, completeness of requirements, whether it is the right thing to build | brainstorming's builder check and the user's own judgement |
| Specs with no Rust surface: a frontend, a protocol with no Rust shapes | not this skill; say so in the one paragraph and stop |
| PR-time review of the code that implements the spec | [rust-design-review](../rust-design-review/SKILL.md), which still runs |
| Correctness bugs visible in the spec's logic | the built-in `/code-review` once code exists; until then, an aside |

Everything out of scope shares **one short closing paragraph**, after the
last finding, on the same budget as
[rust-design-review](../rust-design-review/SKILL.md)'s: not a section, not a
heading, not a paragraph each. A product question or a bug in the spec's
logic is concrete and argues well for its own space, and the moment it leads
the review, the review has become the thing it is defined against. That
skill's soundness and security carve-out applies here unchanged.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "The spec says it in prose, so the design handles it." | A sentence that states a rule is the evidence that no shape holds it. Pass 5 asks whether the sentence would need to exist if the design were right; the shape it covers for is the finding. |
| "That's a product decision, I'll leave it." | Only if the spec says so with a reason. A shape the author chose without stating why is a shape, and passes 1 to 5 review it. Pass 6 drops findings for recorded decisions, not for decisions you imagined. |
| "That decision is wrong, so I'll raise it anyway." | A decision with a stated reason is an authority, not a finding. Pass 6 drops the finding unless you can name the stated goal or anti-goal the decision makes unreachable. Preferring a different shape is not that. |
| "I'll suggest a timer type for the tick." | The spec left it to the builder. A "Left to the builder" entry is never a finding's subject; proposing its shape reviews a decision the spec deliberately did not make. |
| "`#[non_exhaustive]` is good practice on a public error enum." | Not where the governing contract enumerates the set and calls it closed. There the attribute turns the contract's version change into a silent wildcard arm. Pass 8 quotes the contract's sentence and asks what wants the enum open. |
| "The spec is thin, so I'll propose the surface and review that." | Then you are reviewing your own draft. A spec with no Rust shapes gets one paragraph saying so, and the review stops. A thin spec with some shapes is reviewed on the shapes it names. |
| "This newtype costs nothing." | A newtype costs a conversion at every call site that forwards the value, and principle 5 says that is real. Name the cost; "none I can name" is for changes such as an enum over two `Option`s, where it is true. |
| "I'll write out the corrected types so the finding is concrete." | The change is to the spec, at the spec's altitude. Name the shape and what it retires; a type definition in a finding is the plan's work done early. |
| "I found a gap in the requirements, so I led with it." | Requirement completeness is out of scope. It goes in the one closing paragraph, after the last finding. |
