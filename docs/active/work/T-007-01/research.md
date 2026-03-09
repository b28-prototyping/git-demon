# T-007-01 Research: Camera Struct and Unified Projection

## Current Projection Landscape

Every rendering subsystem computes its own projection independently. There is no shared Camera concept — `WorldState` holds `camera_z` and `draw_distance()` as the only shared state, and each pass derives screen coordinates differently.

### Road (`src/renderer/road.rs`)

- **Horizon**: `horizon_ratio(world)` returns 0.25 base, +0.02 for VelocityDemon, minus up to 0.06 at high speed. Used in `mod.rs:108` to compute `horizon_y`.
- **Horizontal grid lines** (lines 88-120): Uses `depth_scale = (1.0 - z_world / draw_distance).clamp(0,1)` — this is **linear depth**, not 1/z. Screen Y: `lerp(horizon_y, h, depth_scale)`.
- **Vertical grid lines** (lines 126-152): Uses `depth = (y - horizon_y) / road_rows` (screen-space depth). Spread: `lerp(0.0, max_half * 1.5, depth)`. Curve: `curve_offset * depth * depth`.
- **Road surface** (draw_road, lines 34-69): Simple `depth = (y - horizon_y) / road_rows` for gradient only.
- **Constants**: `ROAD_MAX_HALF = 480.0`, `BASE_HORIZON_RATIO = 0.25`.

### Sprites (`src/renderer/sprites.rs`)

- **project()** (lines 21-59): `depth_scale = (1.0 - z_rel / draw_dist).clamp(0,1)` — same **linear depth** as road grid.
- **Screen Y**: `lerp(horizon_y, pixel_h, depth_scale)` — matches road grid.
- **Road width**: `lerp(ROAD_MIN_HALF, max_half, depth_scale)` where `ROAD_MIN_HALF = 8.0` — different from road grid's `lerp(0, max_half * 1.5, depth)`.
- **Curve shift**: `curve_offset * depth_scale * depth_scale` — uses depth_scale^2 while road grid uses `depth * depth` (screen-space). These match conceptually but differ in derivation.
- **Sprite sizing**: `scale * scale` (depth_scale squared) used for width/height — ad-hoc, not consistent with any projection model.

### Terrain (`src/renderer/terrain.rs`)

- **Islands** (lines 21-96): `depth_scale = (1.0 - z_rel / draw_dist).clamp(0,1)`. Screen Y: `horizon_y + depth_scale * road_rows` — equivalent to `lerp(horizon_y, h, depth_scale)`.
- **Lateral spread**: `depth_scale * max_half * 1.5` — yet another formula (multiplicative, not lerp from a min).
- **Clouds** (lines 99-173): Same depth formula but screen_y is scaled by `cloud_altitude` factor for height offset.

### Sky (`src/renderer/sky.rs`)

- **Stars** (lines 35-130): Vanishing point at `(w/2, horizon_y * 0.6)` — **offset from horizon**. Speed streaks radiate from this point.
- **Sun**: Fixed at `x = w * 0.72`, `y = horizon_y - 40`. No projection; static placement.

### Effects (`src/renderer/effects.rs`)

- **Speed lines** (lines 12-86): Vanishing point at `(w/2, horizon_y)` — **at horizon**, different from stars at `horizon_y * 0.6`.
- No depth projection involved; purely screen-space radial effect.

### HUD (`src/renderer/hud.rs`)

- No projection. Fixed bottom strip.

### Player Car (`src/renderer/effects.rs:184-286`)

- Fixed at `(w/2, h * 0.42)`. No projection — camera tracks the ship.

## Identified Inconsistencies

| Property | Road Grid | Sprites | Terrain | Stars | Speed Lines |
|---|---|---|---|---|---|
| Depth formula | linear (1-z/dd) | linear (1-z/dd) | linear (1-z/dd) | N/A | N/A |
| Screen Y | lerp(hy, h, ds) | lerp(hy, h, ds) | hy + ds*rows | N/A | N/A |
| Lateral spread | lerp(0, mh*1.5, d) | lerp(8, mh, ds) | ds*mh*1.5 | N/A | N/A |
| Curve shift | co*d*d (screen) | co*ds*ds (world) | co*ds*ds | N/A | N/A |
| Vanishing point | horizon_y center | horizon_y center | horizon_y center | hy*0.6 center | hy center |
| Sprite scale | N/A | ds^2 | size+ds*k | N/A | N/A |

Key inconsistencies:
1. **Road width at horizon**: Grid uses `lerp(0, max_half*1.5, depth)` (starts at 0), sprites use `lerp(8, max_half, depth)` (starts at 8). Objects placed at road edges won't align with grid lines.
2. **Vanishing point**: Stars radiate from `horizon_y * 0.6`, speed lines from `horizon_y`. Not the same point.
3. **Sprite sizing**: Uses `depth_scale^2` which is `(1-z/dd)^2` — arbitrary, neither linear nor 1/z.
4. **All use linear depth**: The ticket asks for 1/z projection but currently everything is linear.

## WorldState Camera-Related Fields

- `camera_z: f32` — world-Z position, advances by `speed * dt`
- `curve_offset: f32` — lateral offset from curvature
- `steer_angle: f32` — heading angle for car rotation
- `draw_distance()` — 5000.0 or 6000.0 for VelocityDemon
- `speed` — used for FOV shift in `horizon_ratio()`
- `z_offset: f32` — separate from camera_z, used for road wave phase

## Function Signatures That Will Change

All draw functions receive `(fb, w, h, horizon_y, world, seed)` or subsets. The `horizon_y` is computed in `mod.rs:108` from `road::horizon_ratio()`. A Camera struct would need to be threaded through these calls either as part of `WorldState` or as a separate parameter.

## Test Coverage

- `road.rs`: 17 tests covering lerp, horizon_ratio, road_max_half, grid rendering, grid movement
- `sprites.rs`: 16 tests covering projection, lane placement, rendering, LOD
- `terrain.rs`: 3 tests covering pixel modification, horizon boundary, scrolling
- `sky.rs`: 11 tests covering HSL conversion, lerp, blend
- `effects.rs`: 15 tests covering blur, bloom, scanline, speed lines
- `world/mod.rs`: 17 tests covering state, update, ingest, despawn

Total: ~79 tests. Many construct `WorldState` directly or via `WorldState::new()`. Adding a `camera` field will require updating test helpers.

## Constraints

1. `WorldState` is `pub` with all fields pub — adding/removing fields is an API change affecting all test helpers
2. `horizon_ratio()` and `road_max_half()` are `pub` functions used across modules
3. `draw_distance()` is a method on `WorldState` — natural candidate to move to Camera
4. The `hud.rs` test helper (`make_world()` at line 227) constructs `WorldState` without gear/rpm/throttle fields — already out of date, will need updating
