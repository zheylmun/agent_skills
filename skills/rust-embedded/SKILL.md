---
name: rust-embedded
description: Use when working on a no_std Rust crate, an embedded target, a hardware driver, or a crate that gates std behind a feature
---

## Overview

This is an overlay, used alongside
[rust-api-design](../rust-api-design/SKILL.md) and
[rust-design-review](../rust-design-review/SKILL.md), never instead of them.
Type-driven design, surface hygiene, error design, the rustdoc contract, and
the design review's five passes all still apply in full — everything here is
only the deltas `#![no_std]` and embedded targets add on top.

## The Deltas

- **No allocation in library APIs.** A `no_std` library cannot assume an
  allocator exists, and even where one is present, allocation on a hot or
  size-constrained path is a cost the caller did not agree to.
  Before: `pub fn read_burst(&mut self, n: usize) -> Vec<u16>`. After:
  `pub fn read_burst<'a>(&mut self, buf: &'a mut [u16]) -> &'a [u16]` — the
  caller owns the storage.

- **Const generics and fixed capacity over growable buffers.** A growable
  buffer hides an allocation or an unbounded runtime size where the hardware
  already fixes the bound.
  Before: `heapless::Vec<u16, 256>` used everywhere out of habit. After:
  `[u16; N]` or a caller-supplied `&mut [u16; N]` sized to what the device
  actually bursts.

- **`defmt` over `core::fmt` for diagnostics.** `core::fmt`'s formatting
  machinery pulls in real code size and cycles on a target where both are
  scarce; `defmt` defers formatting off-device.
  Before: `write!(out, "sample={sample}")`. After:
  `defmt::debug!("sample={}", sample)`.

- **Panic-free library code; fallible construction over `unwrap`.** A panic
  in a library with no unwinding and no operator to read a backtrace is a
  halted device, not a stack trace.
  Before: `let rate = SampleRate::new(hz).unwrap();`. After:
  `pub fn new(hz: u32) -> Result<Self, RateError>` and the caller handles it.

- **Interrupt-safety expressed in the type signature.** Whether a type may be
  touched from an interrupt context is a fact about its API, not a comment
  near its definition.
  Before: `// safe to call from an ISR` above a plain `fn read(&self)`. After:
  a type that only interrupt-safe code can construct or call, e.g. gated
  behind a `Mutex<RefCell<_>>` / critical-section wrapper the ISR path is
  required to go through.

- **`std` behind a feature gate, with the matrix actually checked.** A crate
  that supports both hosted and bare-metal builds is only proven to support
  both when the powerset that excludes `std` is part of the gate, not just
  the default feature set.
  Before: `#[cfg(feature = "std")]` sprinkled in, never built without it.
  After: the same code checked with `--no-default-features` as part of the
  suite below.

## Gate Changes

Run [rust-quality-gate](../rust-quality-gate/SKILL.md) as written, with one
addition: `cargo hack`'s feature powerset matters more here, because the
`std` gate is a real compile-time fork, not a vacuous feature nothing reads.
Add the embedded target so the powerset is checked the way the crate is
actually shipped, not just on the host:

```sh
cargo +1.96.1 hack check --feature-powerset --no-dev-deps --target thumbv7em-none-eabihf
```

A powerset that only ever compiles for the host has not tested the
`no_std` path at all, regardless of how many feature combinations it ran.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "I need a slice of unknown length, so I'll pull in `alloc` for a `Vec`." | Have the caller supply the buffer instead — `&mut [T]` or `[T; N]` — and return a slice or count. `alloc` is a last resort, not a default. |
| "This `unwrap` can't actually fail." | If it truly cannot fail, the fallible path costs nothing to keep and proves the claim to every future caller; if it can fail, a `no_std` library has no backtrace to hand the caller when it does. Return `Result` and let the caller decide. |
| "`core::fmt` is fine, it's just for a debug print." | It is fine on the host; on the target it is the formatting machinery and its code size, paid on every build whether the print fires or not. Use `defmt` for on-device diagnostics. |
