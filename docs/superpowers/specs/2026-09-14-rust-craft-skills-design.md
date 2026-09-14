# Rust Craft Skills — Design

Date: 2026-09-14
Status: Approved, ready for implementation planning

## Purpose

A plugin of composable skills that make an agent write and review Rust the way
this author does. The focus is API design, comments, and documentation — not
low-level mechanics. The agent's defaults in these areas are wrong in specific,
repeatable ways, and these skills correct them.

## Guiding Principles

These are the foundation. Every other skill links here rather than restating
them.

1. **Less code to reason about is better code.** The metric is cognitive load,
   not line count. A newtype that makes an entire class of misuse
   unrepresentable is a net reduction — the caller stops thinking about a whole
   category of thing, so the lines it costs are cheap. Abstraction that only
   relocates complexity is not a reduction.
2. **Simple code is better code.** No abstraction earns its place until it is
   forced.
3. **Code should be self-documenting.** Naming and structure carry the meaning.
4. **An explanatory comment is a design smell.** If code needs a `why`, the
   design is probably wrong — fix the design, not the comment. This explicitly
   overrides the common "comments should say why, not what" guidance.
5. **Ergonomic and hard to misuse, held in tension.** Rust is valuable because
   it can express both. Newtypes add friction at the call site and buy
   correctness. The tension is judged per API, never resolved by blanket rule.
   The newtype is a perfect example. They're useful for making things hard to
   misuse, but they can be a serious ergonomic papercut.

Principle 4 governs inline `//` comments, and governs `///` on private items
exactly the same way — a doc comment on an item nothing outside the crate can
reach is an ordinary comment wearing rustdoc syntax. It does not govern
rustdoc on **public** items: a caller reading docs.rs cannot see the
implementation, so the public doc contract is information the types do not
carry, not an apology for a bad design.

## Repository

`~/dev/zheylmun/agent_skills`, remote `git@github.com:zheylmun/agent_skills.git`.

```
agent_skills/
├── .claude-plugin/
│   ├── marketplace.json
│   └── plugin.json
├── README.md
├── docs/superpowers/specs/
└── skills/
    ├── rust-principles/SKILL.md
    ├── rust-api-design/SKILL.md
    ├── rust-quality-gate/SKILL.md
    ├── rust-design-review/SKILL.md
    └── rust-embedded/SKILL.md
```

Packaged as a plugin so the skills are versioned, diffable, and shareable with
colleagues. The marketplace manifest lets the repo be installed directly from
a local path or from GitHub.

Five skills rather than four: the principles resurface in every other skill,
and duplicating them across four files guarantees drift. `rust-principles` is
the single source; the others link to it — the same discipline the rustdoc
contract demands.

## Skill: rust-principles

Short and declarative. No process, no checklist. States the five principles
above and nothing else. Other skills reference it; it is also worth invoking
alone when the question is a judgment call rather than a task.

## Skill: rust-api-design

Invoked before writing or changing any Rust public surface.

### Type-driven design
- Parse, don't validate: accept the unvalidated form once, return the type that
  proves validity.
- Newtypes where they retire a category of caller thinking, not reflexively.
- Typestate for protocols and state machines where illegal transitions should
  not compile.
- Concrete before generic. No trait until a second implementor exists.

### Surface hygiene
- `#[non_exhaustive]` on public enums and structs that may grow.
- `#[must_use]` where ignoring the return is a bug.
- Borrow in arguments rather than own, unless ownership is genuinely taken.
- Minimal `pub` — the smallest surface that serves the use case.

### Error design
The error type is designed with the same rigor as the success type.
- Every variant is actionable by the caller and carries the context needed to
  act on it.
- Variants the caller cannot distinguish or act on differently get merged.
- `thiserror`-style typed enums at crate boundaries; `#[non_exhaustive]`.
- `anyhow`/`eyre` in binaries only, never in a library.
- Never `Box<dyn Error>` in a public signature.
- Granularity scales with the library's scope: a single error enum is right for
  a focused library; broader libraries warrant per-domain enums.

### The rustdoc contract
- Every public item is documented.
- A terse overview, usually a single sentence. Longer only where usage is
  genuinely nuanced.
- Every argument documented.
- **Intra-doc links on every type reference.** This makes documentation
  explorable and ensures each fact is written down exactly once. The slightly
  larger doc comments are the intended cost.
- Examples where usage is nuanced; examples compile and run as doctests.

## Skill: rust-quality-gate

Runs before a design review. Its output is evidence for the review, not a task
list.

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -W clippy::pedantic
cargo test
cargo hack check --feature-powerset --no-dev-deps
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D missing_docs" cargo doc --no-deps
```

The rustdoc invocation is what makes the link discipline enforceable rather
than aspirational.

**A pedantic clippy hit is a symptom, not a task.** Before fixing or `allow`ing
one, ask what structural fact produced it. Clusters matter more than
individuals: repeated `needless_pass_by_value`, `too_many_arguments`, or
`must_use_candidate` in one module usually name a single bad boundary, and
resolving them locally hides the real finding. The skill reports clusters with
their suspected structural cause. It never runs `clippy --fix` mechanically.

Missing tooling (`cargo-hack`) is reported, not silently skipped.

## Skill: rust-design-review

A design review, deliberately not a bug hunt. Correctness bugs remain the job
of the existing `/code-review`. This skill reviews whether an API is ergonomic,
hard to misuse, and minimal to reason about.

Runs `rust-quality-gate` first, then reviews in this order:

1. **Public surface** — what must a caller learn to use this? What can they get
   wrong that the types could have prevented?
2. **Error design** — is every variant actionable and distinguishable?
3. **Docs** — the contract above, link discipline included.
4. **Cognitive load** — what must a reader hold in their head? Which
   abstraction only moves complexity around?
5. **Gate evidence** — pedantic clusters read as structural findings.

Output is prose at design altitude, most significant finding first. Not a
linter's list.

## Skill: rust-embedded

An overlay, invoked alongside the others when `#![no_std]` or an embedded
target is detected. Deliberately thin — only the deltas from the hosted rules:

- No allocation in library APIs.
- Const generics and fixed capacity over growable buffers.
- `defmt` over `core::fmt` for diagnostics.
- Panic-free library code; fallible construction over `unwrap`.
- Interrupt-safety expressed in the type signature.
- `std` behind a feature gate, with the feature matrix actually checked.

## Out of Scope

- Low-level performance and memory-layout guidance.
- Unsafe-code review discipline.
- MSRV policy, coverage, and miri.

These may become later skills. They are not part of this work.
