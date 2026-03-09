# T-001-03 Structure: road-rasterizer-with-curvature

## Files Modified

### `src/renderer/road.rs`

**Bug fix in `draw_road`:**
- Change `cx` from `u32` to remain as `f32`
- Compute `road_l` and `road_r` as `f32`, then clamp to `[0, w-1]` as `u32`
- This affects lines 57-61 of the current implementation

**Add test module:**
- `#[cfg(test)] mod tests` at bottom of file
- Test helper: `fn test_seed() -> RepoSeed` — constructs minimal RepoSeed without git
- Test helper: `fn test_world(tier: VelocityTier) -> WorldState` — constructs WorldState directly

### No new files created

All tests go in `road.rs` as inline `#[cfg(test)]` module, following the pattern established by `sky.rs` and `font.rs`.

### No files deleted

## Module Boundaries

No new module boundaries. The test module is private to `road.rs` and uses `super::*` to access private functions (`lerp`, `blend_alpha`, `hsl_to_rgb_inline`, `hue_to_neon`).

## Public Interface

No changes to public interface. `draw_road`, `draw_grid`, `horizon_ratio`, `road_max_half` signatures remain identical.

## Internal Changes

The `draw_road` scanline loop changes from:

```rust
let cx = (cx_base + curve_shift) as u32;
let road_half = lerp(ROAD_MIN_HALF, max_half, depth);
let road_l = cx.saturating_sub(road_half as u32);
let road_r = (cx + road_half as u32).min(w - 1);
```

To:

```rust
let cx = cx_base + curve_shift;
let road_half = lerp(ROAD_MIN_HALF, max_half, depth);
let road_l = (cx - road_half).max(0.0) as u32;
let road_r = ((cx + road_half) as u32).min(w - 1);
```

This keeps `cx` as `f32` (it already is until the cast), and clamps `road_l` via `max(0.0)` instead of relying on `u32::saturating_sub` after a potentially-wrapping cast.

## Test Organization

```
#[cfg(test)]
mod tests {
    // Helpers
    fn test_seed() -> RepoSeed { ... }
    fn test_world(tier: VelocityTier) -> WorldState { ... }

    // Pure function tests
    test_lerp_*
    test_horizon_ratio_*
    test_road_max_half_*
    test_blend_alpha_*
    test_hsl_to_rgb_inline_*
    test_hue_to_neon_*

    // Rendering tests
    test_draw_road_perspective_width
    test_draw_road_stripe_colors_present
    test_draw_road_rumble_colors_present
    test_draw_road_verge_colors_present
    test_draw_road_curve_shifts_center
    test_draw_road_velocity_demon_wider
    test_draw_road_no_panic_extreme_curve
    test_draw_grid_accent_pixels
    test_draw_grid_alpha_blended
}
```

## Ordering

1. Fix the `cx` wrapping bug (3-line change)
2. Add test helpers
3. Add pure function tests
4. Add rendering invariant tests
5. Verify all tests pass + clippy clean
