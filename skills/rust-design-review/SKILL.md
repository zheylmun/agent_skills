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
to hold in their head that a better type would have held for them?** Where the
crate answers to a requirement set or a standard, pass 7 adds a second: **what
requirement justifies this?**

## Before You Read Any Code

Run [rust-quality-gate](../rust-quality-gate/SKILL.md) first, in full, and
keep its report open beside you for the whole review. Read no source until it
has finished.

Do not run clippy yourself, do not run a subset of the suite, and do not start
reading `lib.rs` while it runs. The gate's report is the map of where the
crate's structural pressure already shows; reading code before you have it
means you find what you happened to open first, which is how a review ends up
file-ordered.

The gate reports **observations**; this review produces **findings**. The two
words are not interchangeable here — an observation becomes a finding only
once this review has said what it costs a caller and what would retire it.
Three things the gate hands you need explicit handling:

- **Clusters.** Each cluster is the gate's *question* about a structural
  cause. Answering it is pass 6 of this review.
- **Vacuous passes.** The gate reports a `cargo test` run with zero tests, or
  a feature powerset with nothing to vary, as an observation rather than as
  clean.
  Carry it into your review as a finding: "this crate's public contract has never
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
- **Not applicable.** A command the gate reported as not applicable to this
  crate or target is not a finding against the crate, and it is never carried
  in as the vacuous-test finding. The pass it would have evidenced is declared
  unevidenced, the same way missing tooling is handled.

## Scope: Whole Crate Or Just The Change

Settle this before pass 1, because it decides what "every `pub` item" means.

- **Whole-crate review** — asked for a review of a crate, a module, or an API,
  or reviewing a crate for the first time. Every pass covers the full public
  surface.
- **Change-scoped review** — asked to review before merging a change: a PR, a
  branch diff, a set of staged edits. Read the diff first
  (`git diff <base>...HEAD`) and scope the **findings** to the surface the
  change adds, moves, or alters, plus anything the change made newly reachable.

In change-scoped mode the passes are run in full but the output is not a
crate-wide audit:

- A pre-existing problem the change does not touch is not a finding. This is
  [rust-api-design](../rust-api-design/SKILL.md)'s "legacy code in the file is
  not a standard to meet" read in the other direction: new surface meets the
  contract, and old surface next to it is not the review's business today.
- **Crate-wide gate evidence is noted once, in one sentence, and does not
  become per-item findings.** A wholesale `-D missing_docs` failure on a crate
  that never had a doc contract, a clippy cluster that predates the branch, a
  powerset whose bounds the gate reported — each gets a single line saying it
  is crate-wide and pre-existing, and the review moves on. The exception is a
  crate-wide problem the change *extends*: twenty more undocumented `pub` items
  are this change's finding, not the crate's history.
- The gate still runs in full — a smaller run is a different gate, and its
  crate-wide output is still the map. What narrows is what you report, not what
  you run.

A twenty-line PR that produces a full-crate audit is the behavior that makes
people stop invoking this skill. Say which mode you are in, in the first line
of the review.

## Review Order

Work these seven passes in order. The order is deliberate: each pass narrows
what the next one can still be surprised by, and the first pass is the one
that most often produces the leading finding. Pass 7 runs last because it
audits the other six — including the remedies they proposed.

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

### 4. Comments

*Which comment is standing in for a design the code should have carried?*

This pass is not optional and not a subsection of the next one. Principle 4 of
[rust-principles](../rust-principles/SKILL.md) — an explanatory comment is a
design smell — is the most distinctive opinion in this plugin and the one a
review most easily skips, because a comment is cheap to read past and never
shows up in gate output.

Read every `//` comment in the surface under review, and every `///` on a
**private** item (principle 4 governs those identically — a doc comment on
something nothing outside the crate can reach is an ordinary comment in
rustdoc syntax). For each, ask the pass's question: **if the design were
right, would this sentence need to exist?** Three shapes are findings, and the
comment is never the finding itself — the design fact it is covering for is:

- **A comment naming an invariant the types do not enforce** — "callers must
  call `open` first", "`gain` must be 1, 2, 4, or 8", "safe to call from an
  ISR". Each names a caller obligation a typestate, an enum, or a `Mutex` would
  have absorbed; the comment is the evidence that nobody did.
- **A comment defending a shape** — "we take nine parameters here because…",
  an `#[allow]` with a justification attached. The prose is load-bearing where
  the structure should have been.
- **A comment restating a fact a type or constant already holds** — a copy
  waiting to drift, the same failure the rustdoc corollary in
  [rust-api-design](../rust-api-design/SKILL.md) names.

What is *not* a finding: rustdoc on public items (that is pass 3's contract,
and a caller on docs.rs cannot see the body), a `// SAFETY:` block justifying
`unsafe` (required by convention, and soundness is out of scope), and a link
to an external reference a datasheet page or an RFC number the code cannot
carry.

If this pass finds a comment of the first two shapes, **it produces a finding
and that finding is not dropped as a nit** — see the comment rule under
`## Output` for how it ranks. If it finds none, say so in a clause: "no comment
in the reviewed surface is standing in for a missing type" is a real result and
the only way a reader can tell this pass ran.

### 5. Cognitive load

*What must a reader hold in their head? Which abstraction only moves
complexity around?*

Take the crate as a whole. Count the concepts a caller must carry
simultaneously to make one correct call. Look for an abstraction that made a
call site shorter without making anything unrepresentable — by
[rust-principles](../rust-principles/SKILL.md) that is relocation, not
reduction. The comments pass 4 surfaced are evidence here too: several of them
clustered in one module is a load finding about that module, over and above
each one's own design fact.

### 6. Gate evidence

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

### 7. Governing authority

*What requirement justifies this — and what justifies the remedy I just
proposed?*

Run this pass when the crate answers to something outside itself: a
requirement set, a protocol standard, a safety case, a wire format someone
else owns. The signs are a `docs/requirements/` tree, doc comments citing
requirement IDs, a standard named in the crate root, traceability tooling in
CI. If nothing governs the crate, say so in a clause — "no external authority
governs this crate, so pass 7 is not applicable" — and move on. Saying it is
how a reader knows the pass ran.

Where an authority does govern, general Rust-library instinct stops being the
standard and becomes a source of error, because it optimises for an unknown
future population of callers and a governed crate does not have one. Work both
directions:

- **Surface the authority does not ask for.** An extensibility hook, a
  defaulted parameter, a configuration knob, a variant, a trait — anything no
  requirement names. Speculative generality is ordinarily a matter of taste;
  here it is unjustified surface, and it is unjustified in a form a reviewer
  can check.
- **Your own remedies, from passes 1 through 6.** Every change you are about
  to recommend must cite the thing that wants it. This is the direction that
  gets skipped, because a remedy arrives feeling like an improvement rather
  than like a claim — and a remedy justified by a requirement that does not
  exist is the same defect as the code it was meant to fix.

**In a governed crate, a breaking change is information.** It tells every
consumer that the authority changed and their handling needs revisiting. A
hedge that suppresses the break — an enum left open where the governing
document enumerates the set exhaustively, a default that absorbs a new
mandatory value, a wildcard-friendly shape — converts a loud signal into a
silent one. That is drift, and the fact that the mechanism is idiomatic Rust
does not make it less so.

The screen for this pass is one line:

> Name the requirement. Then open it and read it.

A citation that *resolves* is not a citation that *supports*. Traceability
tooling checks that an identifier exists, never that its text says what the
code claims for it, so a wrong-but-resolving citation is invisible to every
automated check the crate has — and is exactly the kind of thing a reviewer
who reads the body catches in one minute. When a doc comment says a variant
is "reported by REQ-123", open REQ-123 and confirm it mentions that variant at
all.

Two things are **not** findings here. A decision the authority is silent on
but the language forces — some representation has to be chosen — is the
crate's to make. And the authority's own choices, however odd, are not this
review's to relitigate.

This pass is the one that can *remove* findings rather than add them: a
remedy from an earlier pass that cannot name its requirement is deleted, not
ranked. What it adds ranks like anything else, and usually ranks high, because
unjustified public surface affects every caller and is breaking to retire.

## Output

Prose, at design altitude, **most significant finding first**.

Significance is not file position and not lint count. Rank by, in order:

1. How many callers are affected, and whether they can work around it.
2. Whether fixing it later is a breaking change — a `pub` field or a
   parameter list is far more expensive to retire after publication than an
   undocumented item is.
3. How much thinking the caller is doing on the API's behalf right now.

**Rank a comment finding by the design fact it stands in for, never by the
comment.** A comment is not a caller-facing item, so ranking the *comment*
sorts it last every time — which is how principle 4 would quietly vanish from
every review. The finding is "nothing in the types enforces that `open` is
called before `send`, and a comment is holding that contract"; rank *that* by
the three criteria above, and it will usually land high, because an unenforced
call order affects every caller and is breaking to retire later. If the design
fact genuinely is minor, it ranks minor — the rule is that the comment's own
unrankability never decides it.

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
never dropped to keep the review short** — every one of the seven passes is run,
every gate outcome lands somewhere (next paragraph), and a pass-4 comment
standing in for a missing type is a mandated carry-in exactly as a vacuous
gate outcome is. If honest work
through the seven passes yields eight findings, write eight. If the crate's
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
| Formatting and style | `cargo fmt`, already run by the gate |
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
| "The gate found `too_many_arguments` and `fn_params_excessive_bools`, so those are two findings." | Those are one piece of evidence for one design fact, and the fact is the finding. Apply the `#[allow]` test in pass 6: a finding that a lint suppression would make untrue was never a design finding. |
| "I listed the lints the gate reported, which is what it gave me." | The gate's report is evidence, deliberately stopping short of conclusions so this step can draw them. Restating it adds nothing and skips the only work this skill does. |
| "I recommended extracting a `ReadOptions` struct." | That is a refactor, not a remedy, until you say what the caller stops having to think about. Name the relief or the recommendation is unjudgeable. |
| "The gate said `cargo test` exited 0, so testing is fine." | The gate reports a zero-test run as an observation, not as clean. Carry it in as one — it outranks the lint-level nits and is never dropped for them. |
| "An unexercised crate is the biggest problem here, so I led with it." | It outranks the nits, not the structure. By the three ranking criteria an unexercised suite costs a caller no ability to use the API and is not breaking to retire, so it belongs near the end of the review, not at the front. |
| "I found a real bug while reading, so I led with it." | Correctness belongs to `/code-review`. Mention it in one line at the end; leading with it turns a design review into the bug hunt this skill exists not to be. |
| "I put the bugs at the end, under their own heading, like the skill said." | The position was right and the budget was not. Out-of-scope material gets one short closing paragraph in total — a heading of its own is how it grows back into the review. |
| "I'll note both the missing `#[must_use]` and the public fields." | Not at the same altitude. A `pub` field that makes an invariant unenforceable is a promise to every future caller; a missing `#[must_use]` is a one-line attribute. Ranking them side by side is how a review flattens into a list. |
| "The comments were fine — they explained the tricky parts well." | Principle 4 says a comment explaining a tricky part is evidence the part should not have been tricky. Pass 4 asks what the comment is standing in for; a well-written comment carrying an invariant the types could hold is a clearer finding, not an excused one. |
| "The only thing pass 4 found was a comment, and a comment is a nit, so I cut it for length." | The comment is not the finding — the unenforced invariant behind it is, and that ranks by its cost to callers like anything else. See the comment rule under Output. |
| "The PR was twenty lines, so I audited the whole crate's public surface while I was in there." | Change-scoped mode exists for this. Scope the findings to the surface the change touches and note crate-wide gate evidence once, in a sentence. An unrequested full-crate audit is how this review stops getting invoked. |
| "The crate has a requirements directory, but I reviewed the API on its merits like any other Rust crate." | Then you measured it against a standard it does not answer to. Pass 7 exists because general-library instinct optimises for an unknown future population of callers, and a governed crate does not have one. |
| "`#[non_exhaustive]` is just good practice for a published enum." | Not where a governing document enumerates the set exhaustively. There, a consumer failing to compile is the correct signal that the authority changed, and the attribute converts it into a `_ => {}` that absorbs the signal silently. Pass 7 asks which requirement asked for the enum to be open. |
| "My remedy adds a little surface, but it makes the API more future-proof." | Future-proof against what? In a governed crate, name the requirement that wants the surface. A remedy justified by a requirement that does not exist is the same defect as the code it was meant to fix — and pass 7 deletes it rather than ranking it. |
| "The doc comment cites a requirement ID, so that claim is traced." | Traceability tooling checks that the identifier exists, never that its text supports the claim. Open it and read it; a wrong-but-resolving citation passes every automated check the crate has. |
| "There were forty things wrong, so the review is forty items." | Forty symptoms are rarely forty designs. Group by cause until each remaining item is a distinct design fact, then order those by significance. |
