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
  Before: `alloc::vec::Vec<u16>`, or a `&mut Vec<u16>` the caller can grow
  without bound. After: `heapless::Vec<u16, N>` where a length must be tracked,
  or `[u16; N]` / a caller-supplied `&mut [u16]` where it need not be.
  `heapless::Vec` is the answer to this rule, not an instance of the problem:
  it is const-generic, fixed-capacity, and allocation-free, and it keeps the
  length that a bare `[u16; N]` throws away.

- **`defmt` for diagnostics; `core::fmt` stays for trait impls.** The rule is
  about *diagnostic output*, not about which traits a type derives.
  Before: `write!(out, "sample={sample}")` on the device's log path. After:
  `defmt::debug!("sample={}", sample)` — `defmt` defers formatting off-device,
  where `core::fmt`'s machinery costs real code size and cycles.
  This does **not** retire `Debug` or `Display`. A public type still derives
  `Debug` ([rust-api-design](../rust-api-design/SKILL.md) calls its absence a
  breaking omission) and a `thiserror`-style error still generates `Display`.
  The embedded delta is one derive added, not one removed:
  `#[derive(Debug, defmt::Format)]` on every public type and every error enum,
  so the same value is readable both through a host-side `{:?}` and through a
  `defmt` transport at a fraction of the size. Gate the `defmt::Format` derive
  behind the crate's `defmt` feature if it has one — that is one more arm of
  the powerset below.

- **Panic-free library code; fallible construction over `unwrap`.** A panic
  in a library with no unwinding and no operator to read a backtrace is a
  halted device, not a stack trace.
  Before: `let rate = SampleRate::new(hz).unwrap();`. After:
  `pub fn new(hz: u32) -> Result<Self, RateError>` and the caller handles it.

- **Interrupt-safety expressed in the type signature.** Whether a type may be
  shared with an interrupt handler is a fact about its API, not a comment
  near its definition.
  Before: `// safe to call from an ISR` above a plain `static mut DEVICE:
  Option<Device>`. After: `static DEVICE: critical_section::Mutex<RefCell<
  Option<Device>>>` — the type itself requires every access, main-line or
  ISR, to go through a critical section, so the safety claim is enforced by
  the compiler instead of asserted in prose.

- **`std` behind a feature gate, with the matrix actually checked.** A crate
  that supports both hosted and bare-metal builds is only proven to support
  both when the powerset that excludes `std` is part of the gate, not just
  the default feature set.
  Before: `#[cfg(feature = "std")]` sprinkled in, never built without it.
  After: the same code checked with `--no-default-features` as part of the
  suite below.

## Gate Changes

Run [rust-quality-gate](../rust-quality-gate/SKILL.md) as written, under its
"On a crate that cannot build for the host" section — that is where the target
handling for the **whole suite** lives, and it is not one command's flag. Two
points carry extra weight here:

- **`--target` applies to every buildable command, not just `cargo hack`.** A
  crate pulling in `cortex-m-rt`, an stm32 HAL, or a `defmt` global logger does
  not link for the host, so clippy, `cargo hack check`, and `cargo doc` each
  take `--target <triple>` (the crate's real triple — `thumbv7em-none-eabihf`
  for a Cortex-M4/M7, `riscv32imc-unknown-none-elf` for a RISC-V part; never
  the placeholder). `cargo test` gets the host-testable-core split or a
  probe-rs / `defmt-test` runner, and is otherwise reported as *not applicable
  on this target* — never as a failure. Never add a `+toolchain` pin to fix a
  target error: it overrides the crate's `rust-toolchain.toml` and looks for
  `rust-std` under the wrong toolchain.
- **The powerset matters more here**, because the `std` gate is a real
  compile-time fork, not a vacuous feature nothing reads, and because
  `defmt` is usually a second one. A powerset that only ever compiles for the
  host has not tested the `no_std` path at all, regardless of how many feature
  combinations it ran. Bound it as the gate describes — a HAL-style crate's
  chip-select features are mutually exclusive by design, so
  `--mutually-exclusive-features` and `--depth 2` are how this command stays
  finite rather than being dropped.

## Common Mistakes

| Rationalization | Correction |
|---|---|
| "I need a slice of unknown length, so I'll pull in `alloc` for a `Vec`." | Have the caller supply the buffer instead — `&mut [T]` or `[T; N]` — and return a slice or count. `alloc` is a last resort, not a default. |
| "This `unwrap` can't actually fail." | `unwrap` is wrong either way, but the fix depends on which is true. If it can fail, model the failure — return `Result` and let the caller decide, since a `no_std` library has no backtrace to hand them when it does. If it truly cannot fail, a `Result` no caller can ever get an `Err` out of is not the fix: per [rust-api-design](../rust-api-design/SKILL.md), "the best error handling is an error that cannot occur." Say the guarantee in the type instead — a narrower argument type, a constructor that cannot be called with a bad value, or an infallible signature — not an invented variant nobody can handle. |
| "`core::fmt` is fine, it's just for a debug print." | It is fine on the host; on the target it is the formatting machinery and its code size, paid on every build whether the print fires or not. Use `defmt` for on-device diagnostics. |
| "`defmt` over `core::fmt`, so I dropped `#[derive(Debug)]` and the `thiserror` `Display`." | Overread. The rule governs the log path, not the trait set. Derive both — `#[derive(Debug, defmt::Format)]` — and keep the generated `Display`; the host-side tooling and every `{:?}` a caller writes depend on them. |
| "The host build fails with `can't find crate for \`core\``, so I pinned a toolchain." | A `+version` pin overrides the crate's `rust-toolchain.toml`, and `rust-std` for a bare-metal triple is installed per toolchain — the pin is what caused that error, not what fixes it. Run unpinned with `--target`. |
