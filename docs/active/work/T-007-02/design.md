# T-007-02 Design: Segment-Based Road with Hills

## Approach Options

### Option A: Per-Scanline Cumulative Offsets (Chosen)

Project segments front-to-back in world Z order. For each scanline row below horizon, determine which segment it falls in and accumulate (curve_dx, slope_dy) from all segments between the camera and that point. Store the result in a per-row lookup table: `road_offsets[y] = (cum_dx, cum_dy, depth_scale, z_world)`.

**Pros**: Simple, cache-friendly (one pass to build table, all consumers index by row). Classic pseudo-3D technique. Natural occlusion — if a hill pushes screen_y upward past an earlier row, those rows just don't get filled (they're behind the crest).

**Cons**: Per-scanline granularity means segments shorter than one scanline's Z-range are invisible. Acceptable — at 200px road height with 5000 draw distance and 1/z compression, the near rows span only a few Z-units each.

### Option B: Per-Segment Back-to-Front Bands

Project each segment as a screen band (top_y, bottom_y). Fill each band with ocean color and grid. Far segments drawn first; near overwrite.

**Pros**: Exact segment boundaries visible. Allows per-segment color/texture variation.

**Cons**: Complex overdraw management. Grid lines must be clipped per band. Road fill becomes non-trivial. Sprites must find their band. More complex than needed for the visual effect.

### Option C: Modify Camera.project() to Include Slope

Add a `project_with_road()` that accumulates slope from the segment list. Each call walks the segment list.

**Pros**: Minimal change to callers.

**Cons**: O(segments) per projection call. Sprites call this per object, terrain per island/cloud. Could be O(40 * 50) = 2000 segment walks per frame. Better to precompute.

## Decision: Option A — Per-Row Lookup Table

Build a precomputed array `RoadRow` for each scanline below horizon during `draw_grid` (or a dedicated pass). All consumers — road fill, grid, sprites, terrain — index into this table.

## Segment Data Model

```rust
pub struct RoadSegment {
    pub length: f32,      // Z-length of this segment
    pub curve: f32,       // horizontal curvature per unit Z
    pub slope: f32,       // vertical slope: +1 = uphill, -1 = downhill
}
```

Stored in `WorldState` as `segments: Vec<RoadSegment>`. ~40 segments covering draw_distance. Each segment is ~125 Z-units long (matching current grid_spacing). Segments recycle as camera_z advances.

## Slope Generation

- Base: `sin(z * 0.001) * 0.3` — gentle rolling hills
- Activity multiplier: at high CPM, amplitude increases to 0.8
- Segment slope sampled when segment is created at far end
- Curvature: current auto-steer sine waves sampled per-segment at creation time

## Per-Row Table Structure

```rust
pub struct RoadRow {
    pub curve_offset: f32,  // cumulative horizontal shift
    pub slope_offset: f32,  // cumulative vertical shift (pixels)
    pub depth_scale: f32,   // 1/z depth at this row
    pub z_world: f32,       // world-Z at this row
}
```

Built once per frame, indexed by screen row (horizon_y..h). All consumers read from this instead of computing their own depth/curve.

## Projection with Hills

For each row from bottom (near) to top (far):
1. Invert screen_y to get z_world via camera's 1/z
2. Find the segment containing this z_world
3. Accumulate curve and slope from all segments between camera and this z_world
4. Adjust the row's final screen_y by the cumulative slope offset
5. Store in the lookup table

The slope offset shifts rows vertically. A hill (positive slope) pushes far rows upward, creating the crest effect. Rows that would be pushed above the horizon are not rendered — they're occluded by the hill.

## Why Not Change Camera.project()

Camera.project() is a pure geometric function. Adding road-specific slope to it would conflate camera projection with road geometry. Instead, we compute road offsets separately and apply them additively. The camera stays clean; road geometry is a layer on top.

## Rejected Alternatives

- **Modifying curve_offset to be per-segment in WorldState.update()**: Would require all consumers to change how they read curve. The lookup table is cleaner.
- **GPU-style segment rendering**: Overkill for a terminal screensaver at 200×60.
- **Storing slope in Camera**: Camera is a viewing frustum; road shape is world geometry.

## Impact on Existing Systems

| System | Change |
|--------|--------|
| road.rs draw_road | Use per-row slope_offset to shift ocean fill |
| road.rs draw_grid | Build the per-row table; hlines at segment boundaries; vlines use per-row curve_offset |
| sprites.rs project | Look up (or interpolate) per-row offsets for sprite's z_world |
| terrain.rs | Same as sprites — use per-row curve/slope |
| effects.rs | No change — post-processing doesn't depend on road geometry |
| WorldState | Owns segment list, generates/recycles segments in update() |
| Camera | No change |
