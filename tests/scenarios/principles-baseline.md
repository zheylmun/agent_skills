# rust-principles scenario: `principles-baseline`

## Prompt

> Here is a Rust function. Add a helper that converts a raw ADC count to millivolts, and make the code easier for the next reader to follow.
>
> ```rust
> pub fn read(raw: u16) -> u16 { raw }
> ```

## Baseline (RED)

Dispatched to a fresh `general-purpose` subagent (model: sonnet) with only the prompt above, nothing else. Full output:

---

Here is the updated version:

```rust
/// ADC reference voltage, in millivolts. Adjust to match the board's Vref.
const ADC_REFERENCE_MV: u32 = 3300;

/// Number of distinct ADC output codes (e.g. 4096 for a 12-bit ADC,
/// 1024 for a 10-bit ADC). Adjust to match the ADC's resolution.
const ADC_RESOLUTION: u32 = 4096;

/// Converts a raw ADC count into millivolts, assuming a linear transfer
/// function: `millivolts = raw / ADC_RESOLUTION * ADC_REFERENCE_MV`.
fn adc_count_to_millivolts(raw: u16) -> u16 {
    // Widen to u32 first so the multiplication can't overflow u16
    // before the division brings it back into range.
    let millivolts = (raw as u32 * ADC_REFERENCE_MV) / ADC_RESOLUTION;
    millivolts as u16
}

/// Reads a raw ADC sample and returns it converted to millivolts.
pub fn read(raw: u16) -> u16 {
    adc_count_to_millivolts(raw)
}
```

Notes on the changes:

- Split the conversion math into its own `adc_count_to_millivolts` helper, so `read` reads as "get a sample, convert it" rather than mixing acquisition and math inline.
- Named the two assumptions the original code silently baked in (reference voltage and ADC resolution) as `const`s with doc comments, so the next reader can see and adjust them without hunting through the arithmetic.
- Did the multiply in `u32` before dividing back down to `u16`, with a comment explaining why — `raw as u32 * ADC_REFERENCE_MV` can exceed `u16::MAX` (e.g. 4095 * 3300 ≈ 13.5M) before the division brings it back into a sane millivolt range, so doing the multiply in `u16` would silently overflow.
- Added doc comments (`///`) explaining what each function does and the conversion formula, since "raw ADC count to millivolts" isn't self-evident from a bare signature.

One thing worth flagging: I assumed a 3.3 V reference and 12-bit (4096-code) ADC since none were specified in the original — if your actual hardware uses different values (e.g. 5 V reference, 10-bit ADC), just update the two constants at the top.

---

### Assessment against expected failures

- **Explanatory comment on self-evident code**: present. `// Widen to u32 first so the multiplication can't overflow u16 before the division brings it back into range.` justifies a design choice inline rather than the design being restructured to make the choice unnecessary or self-evident (e.g. via a checked/widening operation named for what it does). The doc comments on the two `const`s (`/// ADC reference voltage, in millivolts. Adjust to match the board's Vref.`) are also explanatory prose glued onto names that were already self-evident (`ADC_REFERENCE_MV`, `ADC_RESOLUTION`), on private, non-public items — not rustdoc-contract exempt.
- **Trait or generic introduced for one implementor**: absent. The agent did not introduce any trait or generic here.
- **Doc comment explaining why rather than contract**: partially present. The doc comment on `adc_count_to_millivolts` states the formula (closer to a "what"/contract), but the `const` doc comments above lean toward "why you might change this" rather than a contract statement.

Two of the three anticipated failure categories occurred (explanatory comments, and comment-as-apology-for-design on the widening step and on self-evident constant names); per the brief, the baseline is not clean, so the skill proceeds.

## With skill (GREEN)

Dispatched to a fresh `general-purpose` subagent (model: sonnet) with the full text of `skills/rust-principles/SKILL.md` prepended, followed by the same prompt as above, nothing else. Full output:

---

Applying the principles here: the constants become the "why" instead of a comment, and the helper's name states the conversion instead of describing it.

```rust
const ADC_FULL_SCALE_COUNTS: u32 = 4095;
const ADC_REFERENCE_MILLIVOLTS: u32 = 3300;

pub fn read(raw: u16) -> u16 {
    millivolts_from_adc_counts(raw)
}

fn millivolts_from_adc_counts(raw: u16) -> u16 {
    (u32::from(raw) * ADC_REFERENCE_MILLIVOLTS / ADC_FULL_SCALE_COUNTS) as u16
}
```

Reasoning against the principles:

- **No comment needed (#4).** The magic numbers `4095` and `3300` were the thing that would have demanded a comment ("12-bit ADC, 3.3V reference"). Naming them `ADC_FULL_SCALE_COUNTS` / `ADC_REFERENCE_MILLIVOLTS` puts the "why" into the type system's naming instead of prose next to the code.
- **Self-documenting (#3).** `read` now reads as "read → convert counts to millivolts" from its body alone; `millivolts_from_adc_counts` states input and output units in its name, so a caller never has to guess what unit `raw` or the return value is in.
- **No premature newtype (#2, #5).** There's only one call site and one conversion, so wrapping `u16` in `Millivolts`/`AdcCounts` newtypes would be relocating complexity, not reducing it, per the guidance in principle 5's table ("ask what the caller must remember without it — if nothing, it's overkill"). If a second unit-bearing value shows up elsewhere in this driver (e.g., a temperature or current reading also derived from raw counts), that's the forcing case to introduce newtypes so raw counts and millivolts can't be mixed up at a call site.
- Widened to `u32` for the multiply to avoid overflow (`raw * 3300` can exceed `u16::MAX`), then narrowed back — this is inherent to the arithmetic, not something a comment should explain.

If the real reference voltage/resolution differ from 3.3 V / 12-bit, only the two constants need to change — the shape of the function stays the same.

---

### Assessment against GREEN expectations

- No explanatory `//` comments: confirmed, none present in the code. (The overflow-widening rationale that triggered a `//` comment in the RED run appears only in the agent's prose explanation to the reader, not as a comment in the code — and the agent explicitly names *why* no comment is needed there: "this is inherent to the arithmetic, not something a comment should explain," directly invoking Principle 4.)
- No trait: confirmed.
- No premature generic: confirmed.
- Newtype for millivolts: not used. A reason was stated instead, and it correctly applies the skill's own test from the `## Common Mistakes` table ("A newtype is overkill here" → "ask what the caller must remember without it"): single call site, value never escapes into a context where it could be confused with a raw count, so there's nothing a caller must remember. The agent also names the forcing case that would flip the judgment (a second unit-bearing value entering the same driver) — this is the per-API judgment Principle 5 and `## Applying Principle 5` call for, not a blanket rule.

GREEN passed on the first run with the skill loaded. No rationalization surfaced that the `## Common Mistakes` table did not already anticipate — the agent's "no newtype" reasoning is a correct application of the existing "A newtype is overkill here" row, not a loophole. Step 5 (REFACTOR) required no changes to the skill; no additional row was added.

### Correction (post-commit review)

The assessment above was wrong, and the skill as committed at `a501fcf` had two real gaps this GREEN run exploited without the skill catching it:

1. **The rustdoc carve-out was unscoped.** SKILL.md said principle 4 "does not govern rustdoc," full stop, with no restriction to public items. RED's Failure 2 (explanatory `///` comments on private, self-evident constants) was graded a violation only because *this document's* grading notes above added the "private items are not exempt" qualifier — that qualifier did not exist in the skill text an agent actually reads. An agent could cite the unscoped rule as license to do exactly what RED did.
2. **The "newtype is overkill" row was satisfiable by assertion.** Its correction text ended "If the answer is 'nothing,' it may genuinely be overkill" — no evidence required. This GREEN run's reasoning ("There's only one call site... wrapping would be relocating complexity, not reducing it") used principle 1's own language for a *non*-reduction to wave away the paradigm case principle 1 names as a reduction, on a `pub fn read(raw: u16) -> u16` — a public function whose argument and return are the same primitive carrying different units, the textbook confusable pair. The table row let this pass as "genuinely overkill" without ever naming what the caller could confuse.

Both gaps are fixed in the skill (see amendments below), and the scenario was re-run against the amended skill. See `## With skill (GREEN) — re-run after amendment` below for the result.

## Skill amendments (post-review)

Applied to `skills/rust-principles/SKILL.md` after the findings above:

- **Rustdoc carve-out scoped to public items.** `## Rustdoc Is Not Exempt From #3, And Not Governed By #4` now states explicitly that the carve-out applies only to items that are `pub` and reachable from outside the crate, that a `///` on a private item is an ordinary comment principle 4 governs in full, and gives a boundary check ("is this item `pub` and reachable from outside the crate?").
- **"This comment explains a subtlety" row disambiguated**: now states `//` or `///` on a private item makes no difference, with public rustdoc named as the one exception.
- **"A newtype is overkill here" row tightened**: the correction no longer allows "the answer is nothing" as a bare assertion — it now requires naming the two things that could be confused and where, or saying explicitly that no confusable pair can be named.
- **New row added**: `"Only one call site, so no newtype."` → count the `pub` boundary, not the call sites inside the crate today; a `pub fn` is a call site waiting to happen at every external caller. This targets the exact rationalization this GREEN run used.
- **`## Applying Principle 5` and Principle 1** cross-referenced to the same boundary-counting standard, so the guidance is consistent wherever the tension is discussed, not just in the mistakes table.
- **Principle 2 wording fixed** (unrelated minor finding): the original "generalize when a real second case demands it, not because a third might exist someday" mis-stated where the temptation to over-generalize actually strikes. Reworded: the pull to generalize shows up the moment a second case is *imaginable*; the correction is to wait until a second case is *real*, not to wait for a third.

## With skill (GREEN) — re-run after amendment

Dispatched to a fresh `general-purpose` subagent (model: sonnet) with the full text of the *amended* `skills/rust-principles/SKILL.md` prepended, followed by the same prompt as above, nothing else. Full output:

---

Following the rust-principles skill, this snippet is exactly the "single call site, so no newtype" trap called out in the skill's own mistakes table: `raw: u16` and the returned `u16` are a raw ADC count and a converted millivolts value sitting in the same primitive type across a `pub` boundary — a confusable pair. I'll make that unrepresentable with newtypes, and make the ADC's reference voltage an explicit input rather than a hidden constant (which would otherwise need a comment to justify).

```rust
/// A raw sample from the ADC, in counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdcCount(pub u16);

/// A voltage, in millivolts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Millivolts(pub u16);

impl AdcCount {
    const MAX: u16 = 4095;

    /// Converts this count to the voltage it represents, given the ADC's
    /// reference voltage.
    pub fn to_millivolts(self, reference: Millivolts) -> Millivolts {
        let scaled = u32::from(self.0) * u32::from(reference.0) / u32::from(Self::MAX);
        Millivolts(scaled as u16)
    }
}

/// Reads the current ADC sample as a voltage, using the given reference
/// voltage.
pub fn read(raw: AdcCount, reference: Millivolts) -> Millivolts {
    raw.to_millivolts(reference)
}
```

Why it's shaped this way:

- `AdcCount` and `Millivolts` are the two things a caller could confuse in the original `u16 -> u16` signature; wrapping them makes that mixup a compile error instead of a runtime bug, at the cost of one extra field access at each call site (Principle 5's tension, resolved in favor of correctness since this sits on the crate's public boundary).
- `to_millivolts` does the actual conversion and is named for exactly what it does, so no comment is needed to explain the arithmetic (Principle 4) — the `u32` widening is there so the multiply can't overflow a `u16`, which is visible in the type rather than stated in prose.
- The reference voltage is a parameter, not a hardcoded constant, so there's no "why 3300?" comment to write — the caller supplies the fact, the function doesn't assert it.
- `read`'s job shrinks to "call the conversion," which is what its name always implied but the original body didn't deliver.

No repo files were changed — this conversation has no matching Rust source file (only the skill's own scenario/docs mention this snippet, and those weren't touched). This is the direct answer to the prompt; let me know if you want it applied to an actual crate file.

---

### Assessment against GREEN expectations (re-run)

- No explanatory `//` comments: confirmed, none present.
- No trait: confirmed.
- No premature generic: confirmed.
- Newtype for millivolts, with tradeoff named: confirmed, and improved over the first GREEN run — the agent introduced **both** `AdcCount` and `Millivolts`, named the specific confusable pair (raw count vs. converted voltage, both previously bare `u16`), named the boundary that makes it matter (`pub` signature reachable by every external caller, not "only one call site"), and named the cost (one extra field access per call site). It explicitly self-identified the rationalization it was avoiding: *"this snippet is exactly the 'single call site, so no newtype' trap called out in the skill's own mistakes table."*
- Bonus: the reference voltage, which RED and the first GREEN run both hardcoded as an unexplained constant, is now a parameter — removing the "why 3300?" question rather than answering it with a comment or a doc note.

The re-run passed cleanly. No new rationalization surfaced that isn't already covered by the amended `## Common Mistakes` table; no further amendment was made.
