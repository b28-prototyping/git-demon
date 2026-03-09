# T-007-02 Progress: Segment-Based Road with Hills

## Completed Steps

### Step 1: RoadSegment data model
- Created `src/world/road_segments.rs`
- `RoadSegment` struct with `curve` and `slope` fields
- `generate_segment()` produces curve from sine waves (matching auto-steer pattern) and slope from sine noise with CPM-driven amplitude
- Constants: `SEGMENT_LENGTH = 125.0`, `SEGMENT_COUNT = 40`
- Unit tests: bounded values, CPM scaling, variation with z and time

### Step 2: WorldState segment lifecycle
- Added `pub mod road_segments` to `world/mod.rs`
- Added `segments: Vec<RoadSegment>` and `segment_z_start: f32` fields to WorldState
- Initialized in `new()`: generates SEGMENT_COUNT segments from z=0
- Added recycling in `update()`: drops segments behind camera, appends new at far end
- Segment generation uses current `commits_per_min` and `time` for variety

### Step 3: RoadRow table builder
- Created `src/renderer/road_table.rs`
- `RoadRow` struct: curve_offset, slope_offset, depth_scale, z_world
- `build_road_table()`: inverts camera projection per scanline row, accumulates curve/slope from segments
- `lookup_at_z()`: finds closest row for a given world-Z coordinate
- Unit tests: table size, flat segments produce zero offsets, z_world monotonic, hilly segments produce offsets, lookup correctness

### Step 4: Wire road table into renderer
- Added `pub mod road_table` to `renderer/mod.rs`
- Added `road_table_us` to PassTimings
- `render()` builds road table once per frame, passes to all drawing functions

### Step 5: Update road rendering
- `draw_road()`: accepts road_table param; ocean fill remains flat (correct — ocean is background)
- `draw_grid()`: horizontal lines apply slope_offset; vertical lines use per-row curve_offset from road table instead of global `world.curve_offset`

### Step 6: Update sprite projection
- `project()` and `draw_sprites()`: accept road_table, use `lookup_at_z()` for per-segment curve and slope offsets
- Sprites rise/fall with road surface via slope_offset

### Step 7: Update terrain
- `draw_islands()`: uses road_table curve and slope offsets (islands sit on surface)
- `draw_clouds()`: uses road_table curve offset only (clouds float above)

### Step 8: Integration and cleanup
- All tests pass (`cargo test`)
- Clippy clean (`cargo clippy`)
- Build succeeds (`cargo build`)

## Deviations from Plan

1. **Ocean fill stays flat**: Originally planned to apply slope_offset to ocean fill. This caused visual tearing (gaps between displaced rows) and failed the tearing integration tests. Corrected: ocean is flat background fill; hills are expressed through grid lines, sprites, and terrain displacement only. This matches real pseudo-3D racers where the road surface color is flat and hills are conveyed through grid/stripe displacement.

2. **Parameter naming**: Used `rtable` instead of `road_table` for function parameters in sprites.rs and terrain.rs to avoid shadowing the `road_table` module name in scope.
