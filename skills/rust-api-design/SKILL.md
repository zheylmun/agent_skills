---
name: rust-api-design
description: Use when writing or changing any public Rust API, adding a function, type, trait, or error to a crate's surface, or writing rustdoc
---

## Overview

A public Rust surface is a promise to callers you will never meet: make the
wrong call impossible to write, and document the contract the types cannot
carry. This skill assumes [rust-principles](../rust-principles/SKILL.md) and
applies it to the public boundary.

Scope: everything here governs items that are `pub` and reachable from outside
the crate. Private items are governed by
[rust-principles](../rust-principles/SKILL.md) alone.

## Type-Driven Design

### Parse, don't validate

Accept the unvalidated form exactly once, at the boundary, and return a type
that proves validity. A validation that returns `Ok(())` and throws the proof
away forces every later reader to re-establish the invariant by hand.

```rust
// Before: the check happens, the proof evaporates.
pub fn set_rate(hz: u32) -> Result<(), String> {
    if !(1..=1000).contains(&hz) { return Err("bad rate".into()); }
    Ok(())
}

// After: the only way to hold a SampleRate is to have passed the check.
pub struct SampleRate(u32);

impl SampleRate {
    pub fn new(hz: u32) -> Result<Self, RateError> { /* ... */ }
}

pub fn set_rate(rate: SampleRate) { /* no check needed, ever */ }
```

A parsed type keeps its invariant only if its field is private. A `pub` field,
or a `pub` setter that skips the constructor, reopens every hole the parse
closed.

One function may be the parse boundary and take the raw form — that is what a
boundary is. What must not happen is the raw form travelling past it: parse on
the first line, and every subsequent line, field, and call sees only the proven
type.

### Newtypes where they retire caller thinking

A newtype earns its place when it removes a category of mistake, not because
the value happens to be a primitive. The test is the one from
[rust-principles](../rust-principles/SKILL.md): name the confusable pair. If a
`u32` of hertz and a `u32` of milliseconds can be swapped at a `pub` signature,
that is a pair, and the wrapper pays for itself. If you cannot name one, say so
and use the primitive.

### An enumeration is an enum

When the legal values are a fixed, named set, a numeric parameter is the wrong
type. It admits every illegal value and then rejects most of them at runtime.

```rust
// Before: 256 values, 4 of them legal, checked at runtime.
pub fn set_gain(gain: u8) -> Result<(), String> { /* matches!(gain, 1|2|4|8) */ }

// After: 4 values, all legal, checked by the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Gain { X1, X2, X4, X8 }

pub fn set_gain(gain: Gain) { /* no error path at all */ }
```

Note what happened to the error: an entire variant stopped existing. Removing
an error case by choosing a better type is the strongest form of this skill.

### Typestate for protocols

When a sequence of calls has illegal orderings, encode the state in the type so
the illegal ordering does not compile.

```rust
// Before: send() on a closed link is a runtime error.
pub struct Link { open: bool }
impl Link {
    pub fn send(&self, byte: u8) -> Result<(), LinkError> { /* check self.open */ }
}

// After: there is no send() to call until open() has been called.
use core::marker::PhantomData;

pub struct Disconnected;
pub struct Connected;
pub struct Link<S> { _state: PhantomData<S> }

impl Link<Disconnected> {
    pub fn open(self) -> Link<Connected> { Link { _state: PhantomData } }
}

impl Link<Connected> {
    pub fn send(&self, byte: u8) { /* ... */ }
}
```

Reach for this when illegal transitions are real and enumerable. It costs
generic machinery in every signature, so a two-state flag nobody misuses is not
worth it.

### Concrete before generic

Write the concrete version. No trait until a second implementor exists — not a
second imagined one, a second real one.

```rust
// Before: a trait with exactly one implementor, invented for the future.
pub trait Transport { fn send(&self, bytes: &[u8]) -> Result<(), IoError>; }
pub struct I2cBus;
impl Transport for I2cBus { /* ... */ }

// After: the only thing that exists.
pub struct I2cBus;
impl I2cBus { pub fn send(&self, bytes: &[u8]) -> Result<(), IoError> { /* ... */ } }
```

The trait is cheap to add when the second implementor arrives. Unpicking a
trait from a published surface is not.

## Surface Hygiene

| Rule | When | Why |
|---|---|---|
| `#[non_exhaustive]` on public enums and structs | Any public type that could plausibly gain a variant or field | Adding one later is otherwise a breaking change for every caller with a struct literal or exhaustive `match` |
| `#[must_use]` | On any return whose being ignored is a bug — builders, constructors returning a proven type, pure computations | Silently dropping the result is the exact mistake the type was built to prevent |
| Borrow, don't own | `&T` / `&str` / `&[T]` in arguments unless ownership is genuinely taken | Taking ownership to satisfy the borrow checker is a design failure pushed onto the caller |
| Minimal `pub` | Default to private; promote only what the use case needs | Everything `pub` is a promise you have to keep. Fields holding an invariant are never `pub` |
| `#[derive]` the obvious set | `Debug` always; `Clone`/`Copy`/`PartialEq`/`Eq` where semantically true | Their absence is a breaking omission callers cannot work around |

## Error Design

**The error type is designed with the same rigor as the success type.** It is
half the signature and the half callers hit on their worst day.

- **Every variant is actionable and carries its context.** A caller must be
  able to do something different per variant, and the data needed to do it
  rides on the variant. `InvalidRate` is a shrug; `RateOutOfRange { requested,
  min, max }` lets a caller clamp and retry.
- **Merge variants the caller cannot distinguish.** If two variants always
  lead to the same handling, they are one variant with a field.
- **`thiserror`-style typed enums at crate boundaries**, marked
  `#[non_exhaustive]`.
- **`anyhow` / `eyre` in binaries only.** Never in a library — it erases the
  type the caller needs to match on.
- **Never `Box<dyn Error>` in a public signature.** It is `String` with extra
  steps: the caller can print it and nothing else.
- **Granularity scales with scope.** One enum is right for a focused library;
  a broad library warrants per-domain enums rather than one enum with forty
  variants, most of which no single caller can encounter.
- Before adding a variant, ask whether a better argument type deletes it. The
  best error handling is an error that cannot occur.

## The Rustdoc Contract

For every `pub` item reachable from outside the crate:

1. **Every public item is documented.** Functions, types, variants, fields,
   associated constants, trait methods. No exceptions, including re-exports.
2. **Terse — usually one sentence.** State the contract, not a narrative of the
   implementation. Longer only where usage is genuinely nuanced.
3. **Every argument is documented**, in an `# Arguments` list, saying what the
   argument means and what it must satisfy.
4. **Intra-doc links on every type reference.** Every time a doc comment names
   a type, variant, constant, method, or trait that rustdoc can resolve, write
   it as ``[`Thing`]``. This is the rule of the section: it makes docs
   explorable, and it means each fact is written down exactly once — the
   constraint lives on [`PowerBudget::MAX_MILLIWATTS`] and every mention links
   to it instead of restating a number that can drift. The slightly longer doc
   comment is the intended cost, not a reason to skip it.

   The corollary is a hard rule: **if a fact is already carried by a type, a
   variant set, or a constant, the doc links to it and does not restate it.**
   Write ``must be a [`Gain`]``, not "must be 1, 2, 4, or 8". Write ``within
   [`SampleRateHz::MIN`]..=[`SampleRateHz::MAX`]``, not "between 1 and 1000".
   Grep your finished doc comments for the literal values; every occurrence
   outside the one definition is a copy waiting to drift.
5. **`# Errors`** on every fallible function, naming the variants returned and
   the condition for each — linked.
6. **Examples where usage is nuanced**, in an `# Examples` block. They compile
   and run as doctests, so they are checked prose.

### Worked example

```rust
/// A power budget in milliwatts, proven to be within the regulator's range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerBudget(u16);

/// Why a [`PowerBudget`] could not be built.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PowerBudgetError {
    /// The request exceeded [`PowerBudget::MAX_MILLIWATTS`].
    #[error("{requested} mW exceeds the {max} mW regulator ceiling")]
    ExceedsCeiling {
        /// The rejected request, in milliwatts.
        requested: u16,
        /// The ceiling that rejected it, in milliwatts.
        max: u16,
    },
}

impl PowerBudget {
    /// The largest budget the regulator can supply, in milliwatts.
    pub const MAX_MILLIWATTS: u16 = 500;

    /// Builds a [`PowerBudget`] from a raw milliwatt count.
    ///
    /// # Arguments
    ///
    /// * `milliwatts` - requested budget, at most [`PowerBudget::MAX_MILLIWATTS`].
    ///
    /// # Errors
    ///
    /// Returns [`PowerBudgetError::ExceedsCeiling`] when `milliwatts` is above
    /// [`PowerBudget::MAX_MILLIWATTS`].
    ///
    /// # Examples
    ///
    /// ```
    /// use my_crate::{PowerBudget, PowerBudgetError};
    ///
    /// let budget = PowerBudget::new(250)?;
    /// assert_eq!(budget.milliwatts(), 250);
    ///
    /// assert!(matches!(
    ///     PowerBudget::new(501),
    ///     Err(PowerBudgetError::ExceedsCeiling { .. })
    /// ));
    /// # Ok::<(), PowerBudgetError>(())
    /// ```
    pub fn new(milliwatts: u16) -> Result<Self, PowerBudgetError> {
        if milliwatts > Self::MAX_MILLIWATTS {
            return Err(PowerBudgetError::ExceedsCeiling {
                requested: milliwatts,
                max: Self::MAX_MILLIWATTS,
            });
        }
        Ok(Self(milliwatts))
    }

    /// The budget in milliwatts.
    #[must_use]
    pub const fn milliwatts(self) -> u16 {
        self.0
    }
}
```

Note where the links land: on the argument (`milliwatts` links to the constant
that bounds it), on the return type's error variant, on the type named in the
error's own docs, and on every field of the variant. The number `500` is
written exactly once, in the constant.

Verify the contract holds — a doc that does not build is not a doc:

```bash
cargo +1.96.1 test --doc
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo +1.96.1 doc --no-deps
```

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "A `String` error is fine, the message says what's wrong." | The message is for a human reading a log; the type is for the caller writing a match arm. `String` gives the caller nothing to branch on. Make it a `thiserror` enum with one variant per thing a caller would handle differently. |
| "`Box<dyn Error>` keeps the signature flexible." | It keeps it useless. A caller can print it and nothing else, and it can never be made specific later without a breaking change. Never in a public signature. |
| "A `bool` argument is simpler than an enum." | `f(x, true, false)` is unreadable at the call site and admits no third case without a breaking change. Two named variants cost two lines and are self-documenting. Same for any fixed set of numbers — 1, 2, 4, 8 is an enum, not a `u8`. |
| "The doc says what the function does." | Docs state the contract — what it accepts, what it guarantees, what it returns and when it fails — not a narrative of the implementation. A caller on docs.rs cannot see the body and does not want it described. |
| "Adding links makes the doc comment longer." | That length is the point. The link is how a fact gets written down once instead of copied; the copy is what drifts. Link every type reference rustdoc can resolve. |
| "I validated it in the constructor, so the invariant holds." | Only if the field is private and no `pub` setter bypasses the constructor. A `pub` field next to a validating function means the validation is optional, which means it does not exist. |
| "I'll match the conventions of the surrounding code." | Legacy code in the file is not a standard to meet. New public surface meets this contract even when what is next to it does not; otherwise the file never improves. If the existing code conflicts, leave it or fix it, but do not propagate it. |
| "The caller can just remember the valid range." | That is exactly the thinking a parsed type retires. Every `pub` caller is a caller you will never meet, and the range in your doc comment is not in their editor. |
| "Newtypes everywhere would be over-engineering." | Correct as a blanket rule, which is why the judgment is per-API — see [rust-principles](../rust-principles/SKILL.md). But "over-engineering" is not an argument; name what the caller loses by wrapping, against what they can no longer get wrong. If you cannot name the loss, wrap it. |
| "I'll spell the valid values out in the doc — it's clearer than a link." | Then they are spelled out in every doc that mentions them, and the enum or constant that actually holds them is one more copy. Write the set once, in the type, and link to it: ``see [`Gain`]``, not "must be 1, 2, 4, or 8" repeated in four doc comments. This applies to enumerable sets exactly as it does to numeric bounds. |
| "`#[non_exhaustive]` just annoys callers." | It annoys them once, at the `_ =>` arm. Without it, adding a variant or field breaks every downstream crate, which annoys them considerably more. |
