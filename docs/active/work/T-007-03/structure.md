# Structure: T-007-03 parallax-depth-layers

## Files Modified

### `src/renderer/terrain.rs`
- `draw_islands()`: Change camera_z reference from `world.camera_z` to `world.camera_z * ISLAND_PARALLAX` in the z_rel cycling formula. Add const `ISLAND_PARALLAX: f32 = 0.4`.
- `draw_clouds()`: Change camera_z reference from `world.camera_z` to `world.camera_z * CLOUD_PARALLAX` in the z_rel cycling formula. Add const `CLOUD_PARALLAX: f32 = 0.15`.
- Add pitch-based vertical offset to both `draw_islands` and `draw_clouds` screen_y calculations.
- Update/add tests for parallax rates and pitch offsets.

### `src/renderer/sky.rs`
- `draw_stars()`: Add pitch-based vertical shift to convergence point `cy`. When `camera.pitch > 0`, stars shift down on screen (more sky visible). Formula: `cy += camera.pitch * h * PITCH_SENSITIVITY`.
- `draw_sun()`: Add pitch-based vertical shift to `cy`. Same formula with parallax factor 0.05 (near-full shift).
- `draw_bloom_bleed()`: Add matching pitch shift to `start_y` so bloom tracks the horizon.
- No changes to `draw_sky()` gradient — it fills to horizon_y which is computed elsewhere.

### `src/world/camera.rs`
- Add `pitch_offset(parallax_factor: f32, screen_h: u32) -> f32` method that returns the screen-Y pixel shift for a given parallax layer. Formula: `self.pitch * (1.0 - parallax_factor) * screen_h as f32 * PITCH_SENSITIVITY`. Returns 0.0 when pitch is 0.0.
- Add const `PITCH_SENSITIVITY: f32 = 0.5`.

## Files NOT Modified

- `src/renderer/mod.rs` — render order unchanged, no new passes
- `src/renderer/road.rs` — road/grid at parallax 1.0, no change
- `src/renderer/sprites.rs` — sprites at parallax 1.0, no change
- `src/renderer/effects.rs` — screen-space effects, no change
- `src/renderer/hud.rs` — UI overlay, no change
- `src/world/mod.rs` — no new state fields
- `src/world/speed.rs` — unchanged

## Constants

All in their respective files:
- `terrain.rs::ISLAND_PARALLAX = 0.4`
- `terrain.rs::CLOUD_PARALLAX = 0.15`
- `camera.rs::PITCH_SENSITIVITY = 0.5`

## Public Interface Changes

- New method: `Camera::pitch_offset(parallax_factor: f32, screen_h: u32) -> f32`
  - Used by sky.rs and terrain.rs
  - Returns 0.0 when pitch is 0.0 (current state), so zero-cost until T-007-02

## Test Changes

### `src/world/camera.rs` tests
- `test_pitch_offset_zero_pitch`: pitch=0.0 → offset is 0.0
- `test_pitch_offset_nonzero`: pitch=0.1 → offset proportional to (1-parallax)
- `test_pitch_offset_full_parallax`: parallax=1.0 → offset is always 0.0

### `src/renderer/terrain.rs` tests
- `test_islands_parallax_rate`: Verify islands scroll at 40% of camera_z rate
- `test_clouds_parallax_rate`: Verify clouds scroll at 15% of camera_z rate
- `test_clouds_scroll_slower_than_islands`: Differential test
- Existing `test_islands_scroll_with_camera` still passes (islands still move)
- Existing `test_draw_terrain_stays_below_horizon` still passes

### `src/renderer/sky.rs` tests
- `test_stars_no_scroll_with_camera_z`: Stars should not change when only camera_z changes
- `test_stars_shift_with_pitch`: Stars should shift vertically when pitch changes
