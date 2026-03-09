# T-007-02 Research: Segment-Based Road with Hills

## Current Road Architecture

### Camera (`src/world/camera.rs`)
- `Camera` struct: z, pitch, yaw_offset, fov_scale, horizon_ratio, draw_distance, near_plane
- `project(z_world, screen_h, horizon_y) -> Option<(screen_y, depth_scale)>`: 1/z projection
  - `depth_scale = near_plane / z_rel` (NEAR_PLANE=10.0)
  - `screen_y = horizon_y + (screen_h - horizon_y) * depth_scale`
- `project_x(x_offset, depth_scale, screen_w)`: projects X with yaw_offset
- `road_half(depth_scale)`: ROAD_HALF_WORLD (480.0) * depth_scale * fov_scale
- `sync(speed, tier)`: updates draw_distance and horizon_ratio from speed/tier
- draw_distance: 5000 base, 6000 for VelocityDemon

### WorldState (`src/world/mod.rs`)
- `curve_offset: f32` — global horizontal curvature, applied uniformly to all depths
- `curve_target: f32` — set by git activity bursts (random -60..60)
- `steer_angle: f32` — sinusoidal auto-steering for visual interest
- `update(dt)`: curve_offset lerps toward curve_target + steer_angle
- `camera_z` advances by speed*dt each frame
- Objects spawned at camera_z + SPAWN_DISTANCE (2500), despawned when behind by 125

### Road Rendering (`src/renderer/road.rs`)
- `draw_road()`: fills below-horizon with ocean gradient, no perspective structure
  - Iterates scanlines y from horizon_y to h, depth = (y-horizon_y)/road_rows
  - Pure color gradient, no segment awareness
- `draw_grid()`:
  - **Horizontal lines**: iterates i=1..80, computes z_world from grid_spacing (125.0), uses `cam.project()` for screen_y. Already uses 1/z projection correctly.
  - **Vertical lines**: iterates scanlines, computes depth_scale from (y-horizon_y)/road_rows (linear!), applies `curve_offset * depth_scale²` for curve shift. Uses `cam.road_half()` for spread.
  - Inconsistency: vertical lines recompute depth_scale linearly while horizontal lines use 1/z via camera.

### Sprite Projection (`src/renderer/sprites.rs`)
- `project()` calls `cam.project(z_world)` to get (screen_y, depth_scale)
- Applies curve: `cx = pixel_w/2 + curve_offset * depth_scale²`
- Lane positions derived from road_half * multipliers (1.15 verge, 0.35 road)
- Sprites sorted far-to-near and drawn back-to-front

### Terrain (`src/renderer/terrain.rs`)
- Islands and clouds use `cam.project()` for z-based positioning
- Curve applied: `cx + curve_offset * depth_scale²`
- Both cycle with z_period, repeating as camera advances

### Effects (`src/renderer/effects.rs`)
- `draw_car()`: fixed screen position (w/2, h*0.42), rotated by heading
- `draw_speed_lines()`: radiate from vanishing point
- Motion blur, bloom, scanline filter all post-process the full buffer

## Key Observations

1. **No segment concept exists**: The road is an infinite flat plane. Curvature is a single global `curve_offset` applied quadratically by depth_scale².
2. **Camera.project() is flat**: Returns screen_y purely from 1/z depth — no vertical displacement for hills.
3. **Curve is global, not per-segment**: All code that applies curvature uses `world.curve_offset * depth_scale²`. This produces a smooth curve but no per-segment variation.
4. **Vertical lines use linear depth_scale**: `(y - horizon_y) / road_rows` is linear, not matching the 1/z projection used by horizontal grid lines.
5. **Sprites and terrain already follow curve**: They read curve_offset and apply the same quadratic formula.

## Constraints and Risks

- **Performance**: Current road renders at ~200µs on typical terminals. Adding per-segment projection must stay under +0.5ms.
- **draw_road() is scanline-based**: It fills every pixel below horizon. Segments would need to modulate screen_y per scanline, not per-pixel-column.
- **Grid horizontal lines already iterate by z_world**: These naturally become segment-boundary lines if segments define Z intervals.
- **All curve consumers must adopt segments**: road.rs, sprites.rs, terrain.rs all apply curve_offset independently. A segment-based approach centralizes this.
- **Camera.project() must gain a dy output**: Or a new function must map z_world → (screen_y + cumulative_dy, depth_scale).

## Dependencies

- T-007-01 (Camera struct): DONE. Camera is already extracted with 1/z projection.
- The segment system will live in world/ (data) and renderer/ (projection).

## Data Flow Today

```
WorldState.update(dt):
  camera_z += speed * dt
  curve_offset lerps toward curve_target + steer_angle

Renderer.render():
  horizon_y = camera.horizon_y(h)
  road::draw_road(fb, ..., world)       // ocean fill, no perspective
  road::draw_grid(fb, ..., world)       // h-lines via cam.project(), v-lines via linear depth
  sprites::draw_sprites(fb, ..., world) // cam.project() + curve_offset * ds²
  terrain::draw_terrain(fb, ..., world) // cam.project() + curve_offset * ds²
```

## What Needs to Change

1. A `RoadSegment` struct with curve and slope per Z-interval
2. A `Vec<RoadSegment>` in WorldState, generated/recycled during update()
3. A projection function that, given z_world, returns (screen_y + cumulative_slope_offset, depth_scale, cumulative_curve_offset)
4. Road, grid, sprites, and terrain all use the new projection
5. Slope driven by sine noise + git activity intensity
