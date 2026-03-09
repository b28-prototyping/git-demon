# Structure — T-001-02 sky-and-sun

## Files Modified

### src/renderer/sky.rs (MODIFY)

**Current state:** 99 lines — two public functions, four private helpers.

**Changes:**

1. **Add `draw_bloom_bleed` public function** (new, ~20 lines)
   - Signature: `pub fn draw_bloom_bleed(fb, w, horizon_y, seed, world)`
   - Draws 6 rows of accent-colored glow at rows `horizon_y-6..horizon_y`
   - Color: neon accent from `seed.accent_hue` (same hue as road grid)
   - At VelocityDemon: hue rotates with `world.time` (consistent with sky)
   - Alpha fades linearly from ~80 at row `horizon_y-1` to ~0 at row `horizon_y-6`
   - Uses alpha blending over existing sky gradient pixels

2. **Add private helper `bloom_accent_color(hue, world, alpha)`** (~8 lines)
   - Returns `Rgba<u8>` with HSL(hue, 0.9, 0.35) at given alpha
   - Handles VelocityDemon hue rotation

3. **Add private helper `blend_alpha(bg, fg)`** (~8 lines)
   - Standard alpha blend — same pattern used in road.rs and effects.rs
   - Needed for bloom bleed compositing

4. **Add `#[cfg(test)] mod tests`** (~60 lines)
   - `test_hsl_to_rgb_primaries` — red(0°), green(120°), blue(240°)
   - `test_hsl_to_rgb_secondaries` — yellow(60°), cyan(180°), magenta(300°)
   - `test_hsl_to_rgb_achromatic` — white(s=0,l=1), black(s=0,l=0), gray(s=0,l=0.5)
   - `test_hsl_to_rgb_dark_horizon` — verify the specific dark horizon color
   - `test_lerp_u8_edges` — t=0 returns a, t=1 returns b, t=0.5 midpoint
   - `test_lerp_u8_clamp` — values stay in 0..255 range

### src/renderer/mod.rs (MODIFY)

**Change:** Add call to `sky::draw_bloom_bleed()` after `draw_sun` and before `draw_terrain`.

Current render order (passes 1-2):
```rust
sky::draw_sky(...);      // pass 1
sky::draw_sun(...);      // pass 2
```

New render order (passes 1-3):
```rust
sky::draw_sky(...);           // pass 1
sky::draw_sun(...);           // pass 2
sky::draw_bloom_bleed(...);   // pass 2.5 — bloom bleed before terrain
```

The bloom bleed must be before terrain so verge silhouettes paint over the glow correctly.

## Files NOT Modified

- `road.rs` — no changes, just references its accent hue approach
- `effects.rs` — no changes, bloom bleed is not part of post-processing
- `world/mod.rs` — no changes, existing fields sufficient
- `git/seed.rs` — no changes
- `Cargo.toml` — no new dependencies

## Module Boundaries

- `sky.rs` remains self-contained — imports only `image`, `RepoSeed`, `WorldState`
- No new public types or traits
- No cross-module coupling introduced
- The duplicated `blend_alpha` helper stays private in sky.rs (same as road.rs and effects.rs each have their own copy)

## Interface Summary

New public function:
```rust
pub fn draw_bloom_bleed(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    horizon_y: u32,
    seed: &RepoSeed,
    world: &WorldState,
)
```

No changes to existing public interfaces.
