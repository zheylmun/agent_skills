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
  clippy hits as structural observations rather than a task list.
- **rust-design-review** — a design review (not a bug hunt) covering public
  surface, error design, docs, and cognitive load, using `rust-quality-gate`
  output as evidence.
- **rust-spec-review** — runs the design review against a spec before any
  code exists, loops fresh reviewers until it converges, records the
  ergonomics-versus-correctness decisions in the spec's `## Design decisions`
  section, and has a plan mode that checks a written plan against them.
- **rust-refinement-loop** — runs the design review, an adversarial
  correctness review, and simplify over a feature-complete branch in cycles
  before a PR, applies the clear changes, brings the genuine trade-offs to
  you every five cycles, and stops at the first clean cycle.
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

The `1.96.1` toolchain works, so commands run **by hand in this repo** may be
pinned to it locally, e.g. `cargo +1.96.1 clippy`, `cargo +1.96.1 fmt`,
`cargo +1.96.1 hack check`.

That pin is a workaround for this machine only and is deliberately **not** in
the skills. `rust-quality-gate` runs its suite unpinned so that a crate's own
`rust-toolchain.toml` decides — a command-line `+version` overrides that file,
and a bare-metal target's `rust-std` is installed per toolchain, so a pinned
invocation fails with `can't find crate for \`core\`` on exactly the embedded
crates the skills are meant to gate. A pinned skill would also fail for every
colleague who installs this plugin.

Every command in this repo also prints a
`warning: failed to auto-clean cache data ... Permission denied` line from the
shared `~/.cargo` registry cache before the crate's own output. That is a
cache-directory permission issue on this machine, not a finding about any
crate.

## Testing

`tests/fixtures/sensor_kit` is a small crate with deliberately seeded design
flaws (a nine-parameter function, four of them bools, a `pub` field beside a
validating constructor, `Box<dyn Error>`, undocumented public items, no
tests) used as a fixture for `rust-quality-gate` and `rust-design-review`.

`tests/fixtures/enrolment_spec.md`, `enrolment_contract.md`,
`enrolment_spec_reviewed.md`, and `enrolment_plan.md` are the markdown
fixtures for `rust-spec-review`: a spec, its contract, the spec after review,
and a plan written from it. They are specs and plans, not crates, and follow
the same never-improve rule as `sensor_kit`: they keep their seeded flaws.

`tests/fixtures/refinement_branch/` is a fixture for `rust-refinement-loop`:
`setup.sh <dir> [--no-spec]` builds a scratch git branch from a copy of
`sensor_kit`, a seeded change on it, and a short spec (`--no-spec` omits the
spec).

`tests/scenarios/*.md` are RED/GREEN prompts: each one is a task handed to a
fresh agent first with no skill loaded (RED, showing the failure a skill is
meant to catch) and then with the relevant `SKILL.md` loaded ahead of the
same prompt (GREEN). Files differ on whether they also carry the transcript:
`principles-baseline.md`, `api-design-baseline.md`, and `end-to-end.md`
record the full run; `quality-gate-baseline.md`, `design-review-baseline.md`,
and `embedded-baseline.md` record the prompt only, with the transcript kept
in that task's report instead. `spec-review-baseline.md`,
`spec-review-plan-mode.md`, `design-review-decisions.md`,
`refinement-loop-baseline.md`, and `refinement-loop-break-point.md` are
prompt-only too. The newer, prompt-only style is the intended
convention going forward — it keeps a scenario file a stable regression
fixture rather than a snapshot of one run — but the existing full-transcript
files were left as they are rather than rewritten. To re-run one: dispatch a
clean-room agent (no repo access, code carried inline in the prompt) with
just the prompt for RED, or the target `SKILL.md`'s text followed by the
prompt for GREEN, and compare the output against the skill's stated deltas.

## Spec

See
[`docs/superpowers/specs/2026-09-14-rust-craft-skills-design.md`](docs/superpowers/specs/2026-09-14-rust-craft-skills-design.md)
for the full design.
