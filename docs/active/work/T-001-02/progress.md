# Progress — T-001-02 sky-and-sun

## Completed

### Step 1–2: Bloom bleed helpers and function (sky.rs)
- Added `draw_bloom_bleed` public function (lines 61–86)
- Added `blend_alpha` private helper (lines 88–97)
- Bloom bleed draws 6 rows at bottom of sky region with accent color
- Alpha fades linearly from 80 (at horizon) to 0 (6px above)
- VelocityDemon hue rotation applied consistently with sky gradient

### Step 3: Wired into render pipeline (mod.rs)
- Added `sky::draw_bloom_bleed()` call after `draw_sun` and before `draw_terrain`
- Terrain silhouettes correctly paint over the bloom bleed

### Step 4: Unit tests (sky.rs)
- Added `#[cfg(test)] mod tests` with 8 tests:
  - `test_hsl_to_rgb_primaries` — red, green, blue
  - `test_hsl_to_rgb_secondaries` — yellow, cyan, magenta
  - `test_hsl_to_rgb_achromatic` — black, white, gray
  - `test_hsl_to_rgb_dark_horizon` — dark horizon color verification
  - `test_lerp_u8_basic` — t=0, t=1, t=0.5
  - `test_lerp_u8_clamp` — out-of-range t values
  - `test_blend_alpha_fully_transparent` — alpha=0 preserves bg
  - `test_blend_alpha_fully_opaque` — alpha=255 replaces bg

### Step 5: Quality checks
- `cargo test` — 8/8 tests pass
- `cargo clippy` — no new warnings (all warnings pre-existing in other modules)
- `cargo build` — clean build

## Deviations from Plan

None. All steps executed as planned.

## Remaining

All steps complete.
