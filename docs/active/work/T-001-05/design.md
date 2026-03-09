# Design — T-001-05: effects-passes

## Situation

Research confirms all four effects (motion blur, scanline filter, bloom, speed lines) are fully implemented with correct algorithms, correct ordering, and correct CLI disable flags. The code matches every acceptance criterion in the ticket.

The gap is **test coverage** — effects.rs has zero tests while other renderer modules (road, terrain, font, sky) all have `#[cfg(test)]` blocks.

## Options

### Option A: Add unit tests to effects.rs only
Add a `#[cfg(test)]` module to `src/renderer/effects.rs` covering each effect function and helper. No production code changes.

**Pros**: Minimal scope, directly addresses the coverage gap, no risk of breaking working code.
**Cons**: Tests alone don't satisfy ticket AC checkboxes — but the AC is already satisfied by existing code.

### Option B: Refactor effects + add tests
Extract constants, restructure functions, then test.

**Pros**: Cleaner code.
**Cons**: Unnecessary churn on working code. CLAUDE.md says "avoid over-engineering" and "don't add features beyond what was asked."

### Option C: No-op — mark criteria as met
The code already satisfies all acceptance criteria. Write no code.

**Pros**: Zero risk.
**Cons**: Misses the chance to add test coverage for correctness verification.

## Decision: Option A — Add unit tests

The implementation is complete and correct. The remaining work is adding test coverage to verify the effects behave as specified. This is the right scope:

1. **Helper tests**: `luminance_fast`, `lerp`, `blend_alpha`, `hue_to_rgb` — pure functions, easy to test with known values.
2. **Scanline filter test**: Apply to a known buffer, verify even rows darkened by 20%, odd rows unchanged.
3. **Motion blur test**: Create current + previous buffers, verify blending at known speed values.
4. **Bloom test**: Create a buffer with known bright pixels, verify additive glow applied to neighbors, verify clamping at 255.
5. **Speed lines test**: Verify lines only drawn at Demon+ tier, verify count formula, verify alpha values.
6. **Effect order test**: Not a unit test — the ordering is structural in mod.rs and verified by reading the code.

## Rejected

- **Option B**: Over-engineering. The code works and matches spec. No refactoring needed.
- **Option C**: While defensible, adding tests provides regression protection and verifies the AC claims.

## Test Strategy

All tests create small `ImageBuffer<Rgba<u8>>` instances (e.g., 16×16 or 32×32) to keep tests fast. Tests verify:
- Numerical correctness of pixel transformations
- Boundary conditions (clamping at 255, zero-size buffers)
- Tier gating for speed lines
- Correct alpha/blending math
