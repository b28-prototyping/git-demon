# Structure — T-001-05: effects-passes

## Files Modified

### `src/renderer/effects.rs`
**Change**: Append a `#[cfg(test)] mod tests` block at the end of the file.

No production code changes. No new files. No module boundary changes.

## Test Module Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    // --- Helper function tests ---
    fn test_lerp_clamps_and_interpolates()
    fn test_luminance_fast_known_values()
    fn test_blend_alpha_full_opaque()
    fn test_blend_alpha_half_transparent()
    fn test_blend_alpha_zero_alpha()
    fn test_hue_to_rgb_primary_hues()

    // --- Scanline filter tests ---
    fn test_scanline_darkens_even_rows()
    fn test_scanline_leaves_odd_rows()

    // --- Motion blur tests ---
    fn test_motion_blur_at_zero_speed()
    fn test_motion_blur_at_max_speed()
    fn test_motion_blur_preserves_alpha_channel()

    // --- Bloom tests ---
    fn test_bloom_brightens_neighbors_of_bright_pixel()
    fn test_bloom_clamps_at_255()
    fn test_bloom_ignores_dark_pixels()

    // --- Speed lines tests ---
    fn test_speed_lines_at_demon_tier()
    fn test_speed_lines_zero_cpm()

    // Helper to create a WorldState for testing
    fn make_world(speed: f32, cpm: f32, tier: VelocityTier) -> WorldState
    fn make_seed() -> RepoSeed
}
```

## Public Interface

No changes to public API. All test helpers are `fn` (not `pub`), scoped to the test module.

## Dependencies

Tests use only existing dependencies:
- `image::ImageBuffer`, `image::Rgba` — already in scope
- `crate::world::WorldState`, `crate::world::speed::VelocityTier` — already imported
- `crate::git::seed::RepoSeed` — already imported

No new dev-dependencies needed.

## Ordering

Single atomic change — append test module to effects.rs. No ordering constraints.
