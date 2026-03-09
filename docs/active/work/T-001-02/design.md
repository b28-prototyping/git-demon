# Design — T-001-02 sky-and-sun

## Problem

Two items remain incomplete:
1. **6px bloom bleed** of road accent color at the bottom of the sky region
2. **Unit tests** for HSL helper functions

Everything else (gradient, sun disc, hue rotation, pulsing) is already implemented and correct.

## Design Decisions

### 1. Bloom Bleed Implementation

**Option A: Draw bloom strip in `draw_sky` before road overwrites**
- Add 6 rows of accent-colored glow at the bottom of the sky region (rows `horizon_y-6` to `horizon_y-1`)
- Use alpha blending with a fade gradient so it blends into the sky gradient
- Pros: Simple, self-contained in sky module, drawn before road so positioning is natural
- Cons: Terrain silhouettes may paint over some of it (but that's desirable — bloom behind terrain)

**Option B: Separate `draw_horizon_bloom` function called after road**
- Draw the bloom strip after road rendering, blending onto whatever is there
- Pros: Can reference road surface colors
- Cons: Would blend onto road pixels at horizon, which are supposed to be above the road; more complex ordering

**Option C: Add to `apply_bloom` post-process**
- Add special-case bright pixels at the horizon line so the existing bloom pass picks them up
- Pros: Uses existing bloom infrastructure
- Cons: Existing bloom is 3x3 and threshold-based — would need to be bright enough to trigger (luminance > 0.72). The accent at s=1.0 l=0.55 has luminance ~0.55, below threshold. Would require hacks.

**Decision: Option A** — Draw the bloom strip directly in `draw_sky`. The bloom bleed is fundamentally a sky-region visual effect (it makes the horizon glow with road accent color). Drawing it at the end of `draw_sky` places it correctly: behind terrain silhouettes and immediately above the road. A 6-row strip with linearly decaying alpha gives a smooth glow.

### 2. Bloom Bleed Color

The ticket says "road accent color." The road grid uses `hue_to_neon(seed.accent_hue)` → HSL(hue, 1.0, 0.55). This is the neon accent.

For the bloom bleed, use the same hue at high saturation but lower lightness to avoid overwhelming the dark sky gradient. HSL(accent_hue, 0.9, 0.35) with alpha decaying from ~80 to 0 over 6 rows gives a subtle warm glow.

### 3. Test Strategy

**Option A: Inline `#[cfg(test)]` module in sky.rs**
- Test `hsl_to_rgb` with known color values (red, green, blue, cyan, magenta, yellow, white, black)
- Test `lerp_u8` edge cases (0→255, 255→0, t=0.5)
- Pros: Close to code, standard Rust convention
- Cons: Functions are private — must be tested from within the module

**Option B: Integration tests in `tests/` directory**
- Would need public exports of helpers
- Cons: Forces public API changes for test access

**Decision: Option A** — `#[cfg(test)]` module in sky.rs. The helper functions are private and should stay private. Rust's `#[cfg(test)]` module has access to private items in the same file.

### 4. HSL→RGB Deduplication

The `hsl_to_rgb` function is duplicated across 3+ modules. This is a cleanup opportunity but NOT in scope for this ticket. The ticket says "HSL helper functions are correct and tested" — we test the sky.rs copy. Deduplication would be a separate task.

### 5. Bloom Bleed Positioning

The bloom strip occupies rows `horizon_y-6` through `horizon_y-1`. These 6 rows are at the very bottom of the sky region. Since `draw_sky` already fills these rows with the gradient, the bloom bleed alpha-blends the accent color on top of the gradient's darkest (most horizon-like) colors. The result: a warm colored glow connecting sky to road.

At VelocityDemon tier, the bloom bleed hue should rotate along with the sky hue — use the same `hue_to_dark_rgb`-style rotation logic to keep the effect consistent.

## Rejected Alternatives

- **Bloom via post-process**: Too indirect. The existing bloom pass operates on luminance threshold (0.72) and the accent color at appropriate intensity wouldn't trigger it without hacks.
- **Bloom as separate render pass**: Unnecessary complexity. A 6-row alpha-blended strip in `draw_sky` is 15 lines of code.
- **Public HSL module**: Over-engineering for this ticket. The duplication exists but doesn't cause bugs — each copy is correct.
- **Bloom bleed after road**: Wrong Z-order. The bleed should be behind terrain, which paints over sky.

## Summary of Changes

1. Add `draw_bloom_bleed` helper in sky.rs — 6-row accent glow at bottom of sky region
2. Call it at end of `draw_sky` (or as a separate call from mod.rs after `draw_sky`)
3. Add `#[cfg(test)] mod tests` in sky.rs with HSL→RGB and lerp_u8 tests
