# T-007-02 Structure: Segment-Based Road with Hills

## New Files

### `src/world/road_segments.rs`
Contains `RoadSegment` struct and segment generation/recycling logic.

**Public interface:**
```rust
pub struct RoadSegment {
    pub length: f32,
    pub curve: f32,
    pub slope: f32,
}

pub const SEGMENT_LENGTH: f32 = 125.0;
pub const SEGMENT_COUNT: usize = 40;

pub fn generate_segment(z_start: f32, commits_per_min: f32, time: f32) -> RoadSegment;
```

### `src/renderer/road_table.rs`
Contains `RoadRow` struct and the per-frame table builder.

**Public interface:**
```rust
pub struct RoadRow {
    pub curve_offset: f32,
    pub slope_offset: f32,
    pub depth_scale: f32,
    pub z_world: f32,
}

pub fn build_road_table(
    camera: &Camera,
    segments: &[RoadSegment],
    camera_z: f32,
    screen_h: u32,
    horizon_y: u32,
) -> Vec<RoadRow>;

pub fn lookup_at_z(table: &[RoadRow], z_world: f32, camera_z: f32, horizon_y: u32) -> Option<(f32, f32, f32)>;
```

`lookup_at_z` returns `(curve_offset, slope_offset, depth_scale)` for a given z_world by finding the matching row in the table (binary search or linear scan on z_world).

## Modified Files

### `src/world/mod.rs`
- Add `pub mod road_segments;`
- Add field `pub segments: Vec<RoadSegment>` to `WorldState`
- Add field `pub segment_z_start: f32` — world-Z of the first segment
- In `WorldState::new()`: initialize segments covering 0..draw_distance
- In `WorldState::update()`: recycle segments behind camera, append new ones at far end
- Slope/curve generation uses `commits_per_min` and `time` for variation

### `src/renderer/mod.rs`
- Add `pub mod road_table;`
- In `FrameRenderer::render()`: build road table once, pass to draw_road, draw_grid, draw_sprites, draw_terrain
- Add `road_table_us` to `PassTimings` for perf tracking

### `src/renderer/road.rs`
- `draw_road()`: use road table's slope_offset to shift each row's screen position
  - For each output row y, find the road table row and apply slope_offset
  - Rows pushed off-screen (above horizon or below bottom) are skipped → natural occlusion
- `draw_grid()`:
  - Horizontal lines: drawn at segment boundaries using road table values
  - Vertical lines: use per-row curve_offset from road table instead of global curve_offset

### `src/renderer/sprites.rs`
- `project()`: after camera.project() gives base (screen_y, depth_scale), look up road table for curve_offset and slope_offset at that z_world
- Replace `world.curve_offset * depth_scale²` with road table's curve_offset
- Add slope_offset to screen_y

### `src/renderer/terrain.rs`
- `draw_islands()` and `draw_clouds()`: same as sprites — use road table lookup for curve_offset
- Slope_offset applied to islands (they sit on the road surface) but NOT to clouds (they float above)

## Module Boundaries

```
world/road_segments.rs  — Data: RoadSegment, generation, recycling
renderer/road_table.rs  — Projection: per-row lookup table from segments + camera
renderer/road.rs        — Rendering: uses road table
renderer/sprites.rs     — Rendering: uses road table for sprite projection
renderer/terrain.rs     — Rendering: uses road table for island/cloud projection
```

## Function Signature Changes

### `road::draw_road()`
Add parameter: `road_table: &[RoadRow]`

### `road::draw_grid()`
Add parameter: `road_table: &[RoadRow]`

### `sprites::draw_sprites()`
Add parameter: `road_table: &[RoadRow]`

### `sprites::project()`
Add parameter: `road_table: &[RoadRow]`

### `terrain::draw_terrain()`, `draw_islands()`, `draw_clouds()`
Add parameter: `road_table: &[RoadRow]`

## Ordering Constraints

1. world/road_segments.rs first (data model, no dependencies)
2. renderer/road_table.rs second (depends on road_segments + camera)
3. WorldState changes (segment lifecycle)
4. road.rs changes (uses road table)
5. sprites.rs and terrain.rs (use road table)
6. renderer/mod.rs (wire everything together)
