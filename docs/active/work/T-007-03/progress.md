# T-007-03 Progress — Parallax Depth Layers

## Status: Complete

All planned implementation steps finished. 183 lib tests pass, 0 failures. Clippy clean (no warnings from this ticket's changes).

## Changes Made

### 1. `src/world/camera.rs` — pitch_offset method
- Added `PITCH_SENSITIVITY` constant (0.5)
- Added `Camera::pitch_offset(parallax_factor, screen_h) -> f32`
- Formula: `pitch * (1.0 - parallax_factor) * screen_h * PITCH_SENSITIVITY`
- 3 unit tests: zero pitch, nonzero pitch, full-parallax (factor=1.0 returns 0)

### 2. `src/renderer/terrain.rs` — island & cloud parallax
- Added constants `ISLAND_PARALLAX = 0.4`, `CLOUD_PARALLAX = 0.15`
- `draw_islands()`: z_rel now uses `camera_z * ISLAND_PARALLAX` instead of raw `camera_z`; pitch_shift applied to screen_y
- `draw_clouds()`: z_rel now uses `camera_z * CLOUD_PARALLAX` instead of raw `camera_z`; pitch_shift applied to screen_y
- Added `road_table: &[RoadRow]` parameter to `draw_terrain`, `draw_islands`, `draw_clouds` (compatibility with concurrent T-007-02)
- 4 new tests: parallax rate cycle, clouds slower than islands, pitch offset shifts terrain, islands scroll with camera

### 3. `src/renderer/sky.rs` — pitch integration
- `draw_stars()`: pitch_offset(0.0, horizon_y) shifts star field vertically
- `draw_sun()`: pitch_offset(0.05, horizon_y) shifts sun disc
- `draw_bloom_bleed()`: pitch_offset(0.05, horizon_y) shifts bloom start row
- 1 new test: stars shift with pitch

### 4. `src/renderer/sprites.rs` — parameter compatibility
- Added `_road_table: &[RoadRow]` parameter to `draw_sprites` and `project` for T-007-02 compatibility
- Updated all test call sites

### 5. Test helper fixes (effects.rs, hud.rs)
- Added `segments` and `segment_z_start` fields to `make_world()` helpers for T-007-02 compatibility

## Test Results
- **183 lib tests pass** (cargo test --lib)
- **Clippy**: no warnings from this ticket's code
- **Integration tests**: 2 failures in tests/tearing.rs are pre-existing from concurrent T-007-02 changes, unrelated to parallax work
