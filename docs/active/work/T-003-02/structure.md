# T-003-02 Structure: HUD Overlay

## Files Modified

### `src/renderer/hud.rs`
- **Change**: Fix `HUD_BG` alpha from 200 to 204
- **Change**: Add `#[cfg(test)] mod tests` block with unit tests

No new files created. No files deleted. No module boundaries changed.

## Test Structure

Tests added to `src/renderer/hud.rs::tests`:

### Alpha compositing tests
- `test_hud_background_alpha`: Verify background blend produces expected pixel values
  with alpha=204 over a known background color

### Tier badge color tests
- `test_tier_badge_flatline`: Returns dim gray (100, 100, 100)
- `test_tier_badge_cruise`: Returns white (255, 255, 255)
- `test_tier_badge_active`: Returns cyan (0, 255, 255)
- `test_tier_badge_demon`: Returns orange (255, 165, 0)
- `test_tier_badge_velocity_demon_strobe`: Alternates red/white based on time

### HUD content tests
- `test_hud_draws_pixels`: Verify HUD modifies framebuffer in the bottom 18px
- `test_hud_sector_calculation`: Verify sector = total_commits / 100
- `test_hud_repo_right_aligned`: Verify repo text ends near right edge

## Dependencies

No new dependencies. Tests use existing:
- `image::ImageBuffer` / `image::Rgba` for framebuffer construction
- `crate::world::WorldState` and `crate::git::seed::RepoSeed` for test data
- `crate::world::speed::VelocityTier` for tier enum construction

## Public Interface

No changes to public interface. `draw_hud()` and `draw_dev_overlay()` signatures
remain identical. The `tier_badge_color()` function remains private — tests access
it from within the same module.
