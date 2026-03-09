# Research: T-007-03 parallax-depth-layers

## Current Rendering Pipeline

The renderer (`src/renderer/mod.rs`) draws in this order:
1. Sky gradient + stars + sun + bloom bleed (`sky.rs`)
2. Ocean surface (`road.rs::draw_road`)
3. Islands + clouds (`terrain.rs::draw_terrain`)
4. Perspective grid (`road.rs::draw_grid`)
5. Sprites + player car (`sprites.rs`, `effects.rs::draw_car`)
6. Post-processing: motion blur, scanlines, bloom
7. HUD overlay

All passes receive `horizon_y` from `world.camera.horizon_y(h)`.

## Camera System (T-007-01 Dependency)

`src/world/camera.rs` provides a `Camera` struct with:
- `z: f32` — world-Z position, synced from `world.camera_z`
- `pitch: f32` — declared but always 0.0 (reserved)
- `horizon_ratio: f32` — fraction from top to horizon (0.19–0.27)
- `draw_distance: f32`, `near_plane: f32` (10.0)
- `project(z_world, screen_h, horizon_y) -> Option<(screen_y, depth_scale)>` — 1/z projection
- `project_x(x_offset, depth_scale, screen_w) -> f32`
- `road_half(depth_scale) -> f32`
- `vanishing_point(screen_w, screen_h) -> (f32, f32)`

`Camera::sync(speed, tier)` updates `draw_distance` and `horizon_ratio` based on speed/tier. Currently does NOT use `pitch` at all.

## How Each Layer Currently Uses camera_z

### Stars (`sky.rs:35-133`)
- Stars are procedural from a hash of index `i`. Position is computed as angle+distance from a convergence point `(cx, cy)` where `cy = horizon_y * 0.6`.
- Stars do NOT scroll with `camera_z` — they already have parallax factor 0.0 for position.
- Speed streaks stretch radially from the convergence point based on `world.speed`.
- **Already correct for parallax** — no camera_z dependency in star positions.

### Sun (`sky.rs:135-163`)
- Fixed screen position: `cx = w * 0.72`, `cy = horizon_y - 40`.
- No camera_z dependency. Pulsates with `world.time`.
- **Already correct** — near-stationary (parallax ~0.0).

### Bloom Bleed (`sky.rs:165-190`)
- Fixed band at `horizon_y - bleed_rows` to `horizon_y`.
- No camera_z dependency. **Already correct.**

### Islands (`terrain.rs:20-93`)
- 16 islands cycling in `z_period = 4000.0`.
- `z_rel = ((base_z - (world.camera_z % z_period)) + z_period) % z_period` — **scrolls at 100% of camera_z**.
- Uses `cam.project(z_world, h, horizon_y)` for vertical position.
- Ticket wants 40% parallax factor.

### Clouds (`terrain.rs:96-169`)
- 20 clouds cycling in `z_period = 5000.0`.
- Same camera_z scrolling pattern: `z_rel = ((base_z - (world.camera_z % z_period)) + z_period) % z_period` — **scrolls at 100% of camera_z**.
- Uses `cam.project()` for depth_scale, but positions vertically via custom `cloud_altitude` formula.
- Ticket wants 15% parallax factor.

### Ocean Surface (`road.rs:30-65`)
- Per-scanline gradient with wave shimmer using `world.z_offset * 0.05`.
- No per-pixel Z-projection. Fills entire below-horizon area.
- **Road surface is parallax 1.0 by definition** — it's the ground plane.

### Grid Lines (`road.rs:68-156`)
- Horizontal: uses `world.camera_z % grid_spacing` for offset, `cam.project()` for position.
- Vertical: per-scanline from horizon to bottom.
- **Parallax 1.0** — correct, they're on the road surface.

### Sprites (`sprites.rs:18-48, 50-211`)
- Project with `cam.project(z_world, pixel_h, horizon_y)`.
- World-Z stored in `active_objects` as absolute position.
- **Parallax 1.0** — correct, they're roadside objects.

### Speed Lines (`effects.rs:12-85`)
- Radiate from `camera.vanishing_point()`.
- Based on `world.time` and `world.speed`, not camera_z.
- **No parallax change needed.**

### Player Car (`effects.rs:183-285`)
- Fixed screen position (`w/2, h*0.42`).
- **No parallax change needed.**

## WorldState Fields Relevant to Parallax

- `camera_z: f32` — main camera Z, advances by `speed * dt` each frame
- `z_offset: f32` — same as camera_z (both advance by `speed * dt`)
- `camera: Camera` — Camera struct with `z` field synced to `camera_z`
- `speed: f32` — current speed
- `time: f32` — cumulative time

## What Needs to Change

1. **Islands**: Replace `world.camera_z` in Z-cycling with `world.camera_z * 0.4`
2. **Clouds**: Replace `world.camera_z` in Z-cycling with `world.camera_z * 0.15`
3. **Camera pitch integration**: When `camera.pitch != 0`, shift layer screen positions
4. **Sun/moon**: Apply tiny parallax (0.05) to vertical position via pitch

## Constraints

- Render order must remain: sky, ocean, islands+clouds, grid, sprites, effects
- Islands must stay below horizon (checked by test `test_draw_terrain_stays_below_horizon`)
- Clouds must stay below horizon (same boundary check)
- No new allocations or draw calls (performance constraint from ticket)
- Existing tests: `test_islands_scroll_with_camera` asserts islands change when camera_z changes — this will still pass with 0.4 factor since `camera_z * 0.4` still changes

## Camera Pitch Status

`Camera.pitch` exists but is always 0.0. The ticket says pitch integration ties into T-007-02 (hills). Since T-007-02 is not a dependency and pitch is just a reserved field, we should:
- Wire up pitch-based vertical shifts so they activate when pitch becomes non-zero
- But avoid depending on T-007-02 being complete

## Key Insight

The parallax implementation is surprisingly simple. Islands and clouds already use a cycling-Z pattern where the scroll rate is directly controlled by how `camera_z` scales in the Z-offset formula. Multiplying `camera_z` by a parallax factor in those formulas is the entire change for scroll rates. Pitch-based shifts are additive screen-Y offsets proportional to `(1.0 - parallax) * pitch`.
