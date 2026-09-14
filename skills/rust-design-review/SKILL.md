---
name: rust-design-review
description: Use when asked to review a Rust crate, module, or public API, or before merging a Rust change that alters a public surface
---

## Overview

This is a **design** review, deliberately not a bug hunt. Correctness bugs —
the off-by-one, the wrong branch, the missed `None` — belong to the built-in
`/code-review` and are not this skill's output. What this skill reviews is
whether the API is **ergonomic, hard to misuse, and minimal to reason about**.

It assumes [rust-principles](../rust-principles/SKILL.md) as its standard and
never restates it: when a finding rests on a principle, cite the principle and
move on. The criteria for the docs and error passes belong to
[rust-api-design](../rust-api-design/SKILL.md) and are likewise cited, not
respelled here.

The question behind every finding is the same one: **what does a caller have
to hold in their head that a better type would have held for them?**

## Before You Read Any Code

Run [rust-quality-gate](../rust-quality-gate/SKILL.md) first, in full, and
keep its report open beside you for the whole review. Read no source until it
has finished.

Do not run clippy yourself, do not run a subset of the suite, and do not start
reading `lib.rs` while it runs. The gate's report is the map of where the
crate's structural pressure already shows; reading code before you have it
means you find what you happened to open first, which is how a review ends up
file-ordered.

Three things the gate hands you need explicit handling:

- **Clusters.** Each cluster is the gate's *question* about a structural
  cause. Answering it is pass 5 of this review.
- **Vacuous passes.** The gate reports a `cargo +1.96.1 test` run with zero
  tests, or a feature powerset with nothing to vary, as a finding rather than
  as clean.
  Carry it into your review as one: "this crate's public contract has never
  been executed" is a design fact about the surface, not a clean line to pass
  over. It outranks the lint-level nits — it is never dropped to make room for
  a missing `#[must_use]` — but it does not outrank a structural finding, and
  it is never the leading one. Rank it like anything else, by the three
  criteria under `## Output`: an unexercised suite costs a caller no ability
  to use the API and is not a breaking change to retire, so it lands low.
  Carry it in, near the end.
- **Missing tooling.** If the gate reported a command it could not run, say in
  your review which pass is therefore unevidenced. Never present a partial
  gate's output as a complete review.

## Review Order

Work these five passes in order. The order is deliberate: each pass narrows
what the next one can still be surprised by, and the first pass is the one
that most often produces the leading finding.

### 1. Public surface

*What must a caller learn before they can use this? What can they get wrong
that the types could have prevented?*

Enumerate every `pub` item and read it as a stranger who has only docs.rs.
For each, ask what invariant the caller is now responsible for maintaining by
hand — a `pub` field beside a validating constructor, a primitive that is
confusable with the primitive next to it in the same signature, a legal call
order that nothing in the types enforces, a fixed set of legal values admitted
as a number. Every one of those is a caller obligation the type system could
have absorbed.

A long parameter list is a surface finding, not a style finding: it is the
shape a missing type takes.

### 2. Error design

*Is every variant actionable and distinguishable? Does any variant force the
caller to parse a string?*

Judge against the Error Design section of
[rust-api-design](../rust-api-design/SKILL.md). The highest-value finding in
this pass is usually not a badly shaped variant but an error that should not
exist: ask which failures a better argument type would delete from the
signature entirely.

### 3. Docs

*Does the public surface meet the rustdoc contract?*

The contract is The Rustdoc Contract in
[rust-api-design](../rust-api-design/SKILL.md), link discipline included. Do
not restate it; check against it. Rank what you find by what a caller cannot
learn: an undocumented crate root or an unstated `# Errors` contract costs a
caller more than a missing intra-doc link, and a doc that restates a bound the
type already carries is a copy waiting to drift.

When the gate reported a wholesale `-D missing_docs` failure, this pass has
exactly one finding — the crate has no doc contract — not one finding per
item.

### 4. Cognitive load

*What must a reader hold in their head? Which abstraction only moves
complexity around?*

Take the crate as a whole. Count the concepts a caller must carry
simultaneously to make one correct call. Look for an abstraction that made a
call site shorter without making anything unrepresentable — by
[rust-principles](../rust-principles/SKILL.md) that is relocation, not
reduction. Explanatory comments are evidence here: a comment defending why the
code is shaped as it is marks a place the design, not the prose, should have
carried the meaning.

### 5. Gate evidence

*What one structural fact, if changed, makes this cluster and its siblings
stop firing?*

Now answer each of the gate's questions. Take the clusters in the order of the
design facts behind them, not in the order the gate printed them — and if two
clusters turn out to share one cause, they are one finding.

**A cluster is evidence, never a finding.** This is the line the whole skill
rests on. A first, cheap screen:

> Would this finding still be true if someone silenced the lint with
> `#[allow]`?

If the answer is no, you have restated clippy, not reviewed a design. Delete
it and go back to the cluster for the fact underneath it.
`too_many_arguments` on a nine-parameter function is not the finding;
"`read_sensor` implements two jobs — sampling and a retry/power policy — and
the caller pays for both at every call, including the one that wants neither"
is the finding, and it survives an `#[allow]` untouched.

Consequently, **a lint name is never the subject of a sentence in your
output.** Lint names may appear only as supporting evidence for a fact already
stated in the crate's own vocabulary.

**That screen catches the lint's *name*, not the lint's *content*, so it is
not what does the real work here.** "`read_sensor` takes nine parameters, four
of them booleans" names no lint and survives an `#[allow]` untouched, yet it
is `too_many_arguments` in the crate's vocabulary — a paraphrase, not a
finding. What actually enforces the distinction is the three-part finding
shape under `## Output`: a paraphrase of a lint can state the fact, but it
cannot say what the caller is thereby forced to think about, nor what change
would relieve them of it, because the lint never knew. **If a candidate
finding cannot fill in parts 2 and 3, it is a restatement of the gate's
evidence regardless of the words it uses.** Use the `#[allow]` screen to throw
out the obvious cases fast; use the three-part shape to decide.

## Output

Prose, at design altitude, **most significant finding first**.

Significance is not file position and not lint count. Rank by, in order:

1. How many callers are affected, and whether they can work around it.
2. Whether fixing it later is a breaking change — a `pub` field or a
   parameter list is far more expensive to retire after publication than an
   undocumented item is.
3. How much thinking the caller is doing on the API's behalf right now.

Each finding names three things, in this order:

1. **The design fact** — stated in the crate's own vocabulary, without a lint
   name or a file-and-line as its subject.
2. **The cost to the caller** — what they must remember, re-check, or can
   silently get wrong.
3. **The change that would retire it** — what the caller stops having to
   think about once it lands. Name what the caller is relieved of, not merely
   the refactor's name; "extract a struct" with nothing said about what the
   caller stops tracking is not a remedy, it is a rename.

### Why this skill names remedies and the gate does not

[rust-quality-gate](../rust-quality-gate/SKILL.md) forbids naming a remedy on
purpose: its report is evidence that must not smuggle in its own conclusion,
because the gate has read lint output and not the design. It asks questions so
this review can answer them without being led.

This review has read the whole surface, so it is the step that is *entitled*
to answer. That is not a contradiction between the two skills; it is the
division of labor between them. The gate asks, the review answers. If you find
yourself writing a remedy while running the gate, you are in the wrong step —
and if you find yourself only re-asking the gate's questions here, you have
stopped one step short of the review.

This output is explicitly **not**:

- a linter's list, or anything reorderable back into one;
- one bullet per warning;
- file-ordered, or ordered by anything other than significance;
- a severity table, or a bulleted skeleton, standing in for prose.

Numbering the findings is fine — the number is the rank, and a heading per
finding helps. What is not fine is a finding that has collapsed into its
heading: each one is a paragraph a reader can follow without the source open.

A short review of three real findings is worth more than a complete inventory
of fifteen. Read that as a floor and a standard, never as a cap: it licenses
you to drop the nits, not to drop a finding you owe. **A mandated carry-in is
never dropped to keep the review short** — every one of the five passes is
run, and every gate outcome lands somewhere (next paragraph). If honest work
through the five passes yields seven findings, write seven. If the crate's
design is sound, say so in a paragraph and stop.

**All five of the gate's commands land somewhere in your review.** The gate
reports an outcome for each one — `fmt`, clippy, tests, the feature powerset,
rustdoc — and each outcome either becomes or informs a finding, is
acknowledged in a clause as clean, or is declared unevidenced because the
command could not run. Silence about a command reads as never run, in your
review exactly as in the gate's report.

## Out Of Scope

Name these if you see them, in one line, and move on — do not let the review
drift into them:

| Out of scope | Belongs to |
|---|---|
| Correctness bugs | the built-in `/code-review` |
| Formatting and style | `cargo +1.96.1 fmt`, already run by the gate |
| Performance and allocation cost | **no owner in this plugin** — say so when you hand it off, rather than implying a skill will pick it up |
| `unsafe` soundness | **no owner in this plugin** — same; and see the carve-out below, which takes precedence over the budget |

Drifting into a bug hunt is the most common way this review fails, because
bugs are easier to find and feel more concrete than a design fact. A bug you
noticed is a one-line aside at the end, never a leading finding.

Everything out of scope shares **one short closing paragraph**, after the last
finding — not a section, not a heading, not a paragraph each. The budget is
deliberately smaller than it feels like it should be: a bug is concrete and
argues well for its own space, and the moment it gets a heading the review has
become the bug hunt it is defined against. If the out-of-scope material will
not fit in that paragraph, the right move is to say so in one sentence and
recommend `/code-review`, not to expand the budget.

**One carve-out, and it overrides the budget: a soundness or security problem
is stated plainly, in full, wherever it needs to be.** Unsound `unsafe`, a
data race, a memory-safety hole, a leaked credential — these are not "out of
scope" in the sense the budget is about. The budget exists to stop a design
review turning into a bug hunt, not to compress a soundness finding into a
subordinate clause. Say it clearly, say it early if it is that serious, and
name that it is outside the review's remit — the reader needs the fact more
than the review needs its shape. Nothing else in this section gets this
exemption: an ordinary logic bug does not become soundness because it is
serious.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "It compiles and clippy is nearly clean, so the design is fine." | Compiling is the floor, not a grade. Every finding in this review is about code that compiles — that is the premise, not a defense. |
| "I'll go through the file top to bottom so I don't miss anything." | File order is the source's order, not the design's. It buries the most expensive finding under whatever happened to be declared first. Order by significance; see Output. |
| "The gate found `too_many_arguments` and `fn_params_excessive_bools`, so those are two findings." | Those are one piece of evidence for one design fact, and the fact is the finding. Apply the `#[allow]` test in pass 5: a finding that a lint suppression would make untrue was never a design finding. |
| "I listed the lints the gate reported, which is what it gave me." | The gate's report is evidence, deliberately stopping short of conclusions so this step can draw them. Restating it adds nothing and skips the only work this skill does. |
| "I recommended extracting a `ReadOptions` struct." | That is a refactor, not a remedy, until you say what the caller stops having to think about. Name the relief or the recommendation is unjudgeable. |
| "The gate said `cargo +1.96.1 test` exited 0, so testing is fine." | The gate reports a zero-test run as a finding, not as clean. Carry it in as one — it outranks the lint-level nits and is never dropped for them. |
| "An unexercised crate is the biggest problem here, so I led with it." | It outranks the nits, not the structure. By the three ranking criteria an unexercised suite costs a caller no ability to use the API and is not breaking to retire, so it belongs near the end of the review, not at the front. |
| "I found a real bug while reading, so I led with it." | Correctness belongs to `/code-review`. Mention it in one line at the end; leading with it turns a design review into the bug hunt this skill exists not to be. |
| "I put the bugs at the end, under their own heading, like the skill said." | The position was right and the budget was not. Out-of-scope material gets one short closing paragraph in total — a heading of its own is how it grows back into the review. |
| "I'll note both the missing `#[must_use]` and the public fields." | Not at the same altitude. A `pub` field that makes an invariant unenforceable is a promise to every future caller; a missing `#[must_use]` is a one-line attribute. Ranking them side by side is how a review flattens into a list. |
| "There were forty things wrong, so the review is forty items." | Forty symptoms are rarely forty designs. Group by cause until each remaining item is a distinct design fact, then order those by significance. |
