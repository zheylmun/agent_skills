# rust-craft

`rust-craft` is a Claude Code plugin of composable skills for writing and
reviewing Rust the way this author does, with a focus on API design, comment
discipline, and documentation rather than low-level mechanics.

## Skills

- **rust-principles** — the five guiding principles (less code to reason
  about, simplicity, self-documenting code, comments as a design smell,
  ergonomics vs. misuse-resistance) that every other skill here builds on.
- **rust-api-design** — type-driven design, surface hygiene, error design, and
  the rustdoc contract to apply before writing or changing a public Rust
  surface.
- **rust-quality-gate** — runs `fmt`, `clippy`, tests, `cargo-hack`, and a
  docs lint as evidence for a design review, and reports clusters of pedantic
  clippy hits as structural findings rather than a task list.
- **rust-design-review** — a design review (not a bug hunt) covering public
  surface, error design, docs, and cognitive load, using `rust-quality-gate`
  output as evidence.
- **rust-embedded** — a thin overlay of `#![no_std]`/embedded-specific
  deltas, invoked alongside the other skills when an embedded target is
  detected.

## Install

```
/plugin marketplace add ~/dev/zheylmun/agent_skills
/plugin install rust-craft@zheylmun-agent-skills
```

## Toolchain

The default `stable` Rust toolchain on this machine is missing the `cargo`
component (`rustup component add cargo --toolchain stable` reports the
component as already up to date, but `cargo +stable --version` still fails
with "the 'cargo' binary ... is not applicable to the 'stable-aarch64-apple-darwin'
toolchain"). The attempted repair did not fix it, and a forced reinstall of
`stable` was intentionally not attempted (slow, uncertain payoff).

The `1.96.1` toolchain works and is what every `cargo` invocation in this
project pins to, e.g. `cargo +1.96.1 clippy`, `cargo +1.96.1 fmt`,
`cargo +1.96.1 hack check`.

## Testing

`tests/fixtures/sensor_kit` is a small crate with deliberately seeded design
flaws (a nine-parameter function, four of them bools, a `pub` field beside a
validating constructor, `Box<dyn Error>`, undocumented public items, no
tests) used as a fixture for `rust-quality-gate` and `rust-design-review`.

`tests/scenarios/*.md` are RED/GREEN prompts: each one is a task handed to a
fresh agent first with no skill loaded (RED, showing the failure a skill is
meant to catch) and then with the relevant `SKILL.md` loaded ahead of the
same prompt (GREEN). The scenario files record the prompt only, not the
transcript — this repo's convention settled on that as the skills matured,
so a scenario file stays a stable regression fixture instead of a snapshot of
one run. To re-run one: dispatch a clean-room agent (no repo access, code
carried inline in the prompt) with just the prompt for RED, or the target
`SKILL.md`'s text followed by the prompt for GREEN, and compare the output
against the skill's stated deltas.

## Spec

See
[`docs/superpowers/specs/2026-09-14-rust-craft-skills-design.md`](docs/superpowers/specs/2026-09-14-rust-craft-skills-design.md)
for the full design.
