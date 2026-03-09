# T-007-02 Plan: Segment-Based Road with Hills

## Step 1: RoadSegment data model (`src/world/road_segments.rs`)

Create the new module with:
- `RoadSegment` struct (length, curve, slope)
- `SEGMENT_LENGTH` (125.0), `SEGMENT_COUNT` (40)
- `generate_segment(z_start, commits_per_min, time) -> RoadSegment`
  - curve: sampled from sine waves at z_start (matching current auto-steer pattern)
  - slope: `sin(z_start * 0.001) * base_amplitude`, amplitude scales with CPM
- Unit tests: generation produces valid ranges, slope scales with CPM

**Verification**: `cargo test world::road_segments`

## Step 2: WorldState segment lifecycle (`src/world/mod.rs`)

- Add `pub mod road_segments;` to world/mod.rs
- Add `segments: Vec<RoadSegment>` and `segment_z_start: f32` to WorldState
- Initialize in `new()`: generate SEGMENT_COUNT segments starting at z=0
- In `update()`: after advancing camera_z, recycle segments behind camera, append new at far end
- Unit tests: segments advance with camera, count stays at SEGMENT_COUNT

**Verification**: `cargo test world::tests`

## Step 3: RoadRow table builder (`src/renderer/road_table.rs`)

Create the module with:
- `RoadRow` struct (curve_offset, slope_offset, depth_scale, z_world)
- `build_road_table(camera, segments, camera_z, segment_z_start, screen_h, horizon_y) -> Vec<RoadRow>`
  - For each screen row from horizon_y to screen_h:
    - Invert camera projection to get z_world at this row
    - Walk segments from camera to z_world, accumulating curve and slope
    - Store in RoadRow
- `lookup_at_z(table, z_world, camera_z, horizon_y) -> Option<(curve_offset, slope_offset, depth_scale)>`
  - Find row in table closest to z_world (binary search on z_world which is monotonic)
- Unit tests: table size matches road_rows, z_world monotonically decreasing from bottom to top, lookup returns correct values

**Verification**: `cargo test renderer::road_table`

## Step 4: Wire road table into renderer (`src/renderer/mod.rs`)

- Add `pub mod road_table;`
- In `render()`: after computing horizon_y, call `build_road_table()` with world's camera, segments, etc.
- Pass road_table to draw_road, draw_grid, draw_sprites, draw_terrain
- Add `road_table_us` timing to PassTimings

**Verification**: `cargo build` — ensures signatures match

## Step 5: Update road rendering (`src/renderer/road.rs`)

- `draw_road()`: accept `road_table: &[RoadRow]`
  - For each output screen row y, look up the corresponding RoadRow
  - Apply slope_offset to shift the row's fill position
  - Skip rows where slope pushes them off-screen (hill occlusion)
- `draw_grid()`: accept `road_table: &[RoadRow]`
  - Horizontal lines: use road_table entries to find segment boundaries and their screen positions (with slope offset)
  - Vertical lines: use per-row curve_offset from road_table instead of `world.curve_offset`
- Update existing tests to pass road_table; add hill-specific tests

**Verification**: `cargo test renderer::road`

## Step 6: Update sprite projection (`src/renderer/sprites.rs`)

- `project()` and `draw_sprites()`: accept `road_table: &[RoadRow]`
- After camera.project() returns base (screen_y, depth_scale), use `lookup_at_z()` for the sprite's z_world
- Replace global curve_offset with per-segment curve from road_table
- Add slope_offset to screen_y so sprites rise with hills
- Update existing tests

**Verification**: `cargo test renderer::sprites`

## Step 7: Update terrain (`src/renderer/terrain.rs`)

- `draw_terrain()`, `draw_islands()`, `draw_clouds()`: accept `road_table: &[RoadRow]`
- Islands: use road_table curve_offset and slope_offset (they sit on the surface)
- Clouds: use road_table curve_offset only (they float above the surface)
- Update existing tests

**Verification**: `cargo test renderer::terrain`

## Step 8: Final integration test and cleanup

- Run full test suite: `cargo test`
- Run clippy: `cargo clippy`
- Verify build: `cargo build`
- Visual spot-check with `cargo run -- --repo .` if possible

## Testing Strategy

- **Unit tests** in road_segments.rs: generation ranges, CPM scaling
- **Unit tests** in road_table.rs: table properties (monotonic z, correct size), lookup correctness
- **Updated integration tests** in road.rs, sprites.rs, terrain.rs: existing tests pass with road_table parameter added (initially flat segments = same behavior)
- **Hill-specific tests**: verify slope_offset is non-zero for non-flat segments, verify occlusion (rows pushed above horizon are skipped)
- **Performance**: road_table build time tracked via PassTimings
