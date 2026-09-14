---
name: rust-principles
description: Use when writing, reviewing, or refactoring any Rust code, before deciding on abstractions, comments, or API shape
---

## Overview

These are the standing principles for Rust code in this codebase; every other Rust skill in this plugin builds on them.

## The Principles

1. **Less code to reason about is better code.** The metric is cognitive load,
   not line count. A newtype that makes an entire class of misuse
   unrepresentable is a net reduction — the caller stops thinking about a
   whole category of thing, so the lines it costs are cheap. Abstraction
   that only relocates complexity is not a reduction, no matter how much
   shorter the call site gets.

2. **Simple code is better code.** No abstraction earns its place until it
   is forced. Write the concrete version first. The pull to generalize
   shows up the moment a second case is merely imaginable — resist it
   until a second case is real and in front of you. A real second case is
   what forces the abstraction; an imagined one is a guess.

3. **Code should be self-documenting.** Naming and structure carry the
   meaning. A reader should be able to tell what code does from its shape —
   names, types, structure — without needing prose next to it to translate.

4. **An explanatory comment is a design smell.** If code needs a `why`, the
   design is probably wrong — fix the design, not the comment. This
   explicitly overrides the common "comments should say why, not what"
   guidance: that guidance is wrong. A comment justifying why the code does
   something non-obvious is a signal to rewrite the code until the why is
   obvious, not a place to record the why.

5. **Ergonomic and hard to misuse, held in tension.** Rust is valuable
   because it can express both. Newtypes add friction at the call site and
   buy correctness. The tension is judged per API, never resolved by
   blanket rule. The newtype is a perfect example. They're useful for
   making things hard to misuse, but they can be a serious ergonomic
   papercut.

## Rustdoc Is Not Exempt From #3, And Not Governed By #4

The carve-out below applies only to **public** rustdoc — items that are
`pub` and reachable from outside the crate. It is not a blanket exemption
for `///` syntax. A `///` on a private item is an ordinary comment wearing
rustdoc syntax: principle 4 governs it in full, and a doc comment that
restates an already-self-evident private name, or explains why a private
item is shaped the way it is, is the same design smell as a `//` doing the
same thing.

For public items only: principle 4 does not govern rustdoc, because a
caller reading docs.rs cannot see the implementation — the public doc
contract is information the types do not carry, not an apology for a bad
design. Principle 3 still applies there — the doc states the contract
(what the item does, what it accepts, what it guarantees), not a narrative
of how it was built or why it looks the way it does.

Boundary check before writing a `///`: is this item `pub` and reachable
from outside the crate? If not, principle 4 applies exactly as it does to
`//`.

## Applying Principle 5

The newtype is the canonical tension of Principle 5. It makes an entire
class of misuse unrepresentable — a caller can no longer pass a raw `u16`
where a distinct kind of `u16` was required. It is also a real ergonomic
papercut at every call site that has to construct, unwrap, or convert one.
Neither fact cancels the other, and there is no blanket answer: "always
newtype primitives" and "never newtype primitives" are both wrong for the
same reason — they resolve a per-API judgment call with a rule. Judge it
per API: what does a caller lose by wrapping the value, against what a
caller can no longer get wrong without it. Count the API's public
boundary when weighing this, not the number of call sites that happen to
exist inside the crate today — a `pub` function is a call site waiting to
happen at every caller the crate doesn't control.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "This comment explains a subtlety." | The subtlety belongs in the design, not the comment — `//` or `///` on a private item, it makes no difference. If the code needs a comment to be understood, restructure the code until it doesn't. (Public rustdoc is the one exception: state the contract there, not the subtlety.) |
| "A trait makes this extensible." | No trait until a second implementor exists. Extensibility for a hypothetical caller is speculative complexity, not a reduction in cognitive load. |
| "Fewer lines is simpler." | Fewer things to reason about is simpler. That is not the same as fewer lines — a shorter function that still forces the reader to track an invariant in their head is not simpler. |
| "A newtype is overkill here." | Don't answer this by asserting the caller has nothing to remember — name the two things that could be confused and say where. If you can't name a confusable pair, say so explicitly; if you can, the newtype is buying real safety, even at a single call site. |
| "Only one call site, so no newtype." | Count the `pub` boundary, not the call sites inside this crate today. `pub fn read(raw: u16) -> u16` puts a raw count and a converted value in the same primitive type across a signature every external caller can hit — that is the confusable pair principle 1 is about, independent of how many places currently call it. |
