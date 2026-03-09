# T-007-04 Research: Dynamic Camera Response

## Scope

Add physically-motivated camera dynamics that react to speed, curvature, road slope,
velocity tier, and git activity bursts. The Camera struct already exists (T-007-01/02)
with fields for pitch, yaw_offset, fov_scale, horizon_ratio, and draw_distance.
This ticket wires those fields to dynamic spring-damped behaviors.

## Current Camera State (`src/world/camera.rs`)

The `Camera` struct has all the fields the ticket needs:

| Field           | Current Use                                      |
|-----------------|--------------------------------------------------|
| `z`             | World position, set from `camera_z` each frame   |
| `pitch`         | Declared, used by `pitch_offset()` for parallax, but never dynamically updated |
| `yaw_offset`    | Declared, used in `project_x()`, but never dynamically updated |
| `fov_scale`     | Used in `road_half()` and `project_x()`, always 1.0 |
| `horizon_ratio` | Updated by `sync()` based on speed + VelocityDemon |
| `draw_distance` | Updated by `sync()` based on VelocityDemon tier  |

The `sync()` method is called once per frame in `WorldState::update()`.
It only adjusts `draw_distance` and `horizon_ratio`. It does not touch
`pitch`, `yaw_offset`, or `fov_scale`.

## Camera Usage in Renderers

### pitch_offset() consumers
- `sky.rs:draw_stars()` — shifts star convergence point (`parallax_factor=0.0`)
- `sky.rs:draw_sun()` — shifts sun position (`parallax_factor=0.05`)
- `sky.rs:draw_bloom_bleed()` — shifts bloom bleed band (`parallax_factor=0.05`)
- `terrain.rs:draw_islands()` — shifts island Y positions (`parallax_factor=0.4`)
- `terrain.rs:draw_clouds()` — shifts cloud Y positions (`parallax_factor=0.15`)

### yaw_offset consumer
- `camera.rs:project_x()` — shifts center point: `cx + yaw_offset * depth_scale`
  - Used by `sprites.rs:project()` indirectly via `cam.road_half()` and lane positioning

### fov_scale consumers
- `camera.rs:road_half()` — `ROAD_HALF_WORLD * depth_scale * fov_scale`
- `camera.rs:project_x()` — `cx + x_offset * depth_scale * fov_scale`

### vanishing_point() consumers
- `effects.rs:draw_speed_lines()` — radial streak origin
- Note: vanishing_point() currently ignores yaw_offset, always returns screen center

## Data Sources for Camera Dynamics

| Dynamic           | Source in WorldState                    | Available? |
|-------------------|-----------------------------------------|-----------|
| Speed (normalized)| `world.speed` (0..~900 range after T-007-02 rescale) | Yes |
| Curvature         | `world.curve_offset` (f32, ±~80 range)  | Yes |
| Road slope        | `world.segments` via road_table          | Partially — need to sample slope at camera position |
| Velocity tier     | `world.tier` (VelocityTier enum)         | Yes |
| Time              | `world.time` (f32 seconds)               | Yes |
| Git burst         | `world.commits_per_min` (detection in `ingest_poll`) | Yes, but no explicit burst flag |

### Slope at Camera

The road_table is built per-frame during rendering. During `WorldState::update()`,
there is no road_table available. However, `world.segments` is available, and
`segment_z_start` tells us which segment the camera is in.

We can compute the slope at the camera position directly from segments:
```
let seg_idx = ((camera_z - segment_z_start) / SEGMENT_LENGTH) as usize;
let slope = segments.get(seg_idx).map(|s| s.slope).unwrap_or(0.0);
```

### Git Burst Detection

Currently `ingest_poll()` shifts `curve_target` when `commits_per_min > 1.0`.
There is no burst event flag. We can add a simple `burst_cooldown: f32` field
that gets set on poll ingestion and decays in `update()`.

## Spring Damping Pattern

All camera dynamics should use exponential lerp (spring damping):
```
value += (target - value) * spring_constant * dt;
```

This is already the pattern used for:
- `curve_offset` lerping toward `curve_target` (spring=2.5)
- `throttle` lerping toward target (spring=3.0)
- `horizon_ratio` in `sync()` is set directly (not spring-damped, but changes are small)

## Constraints from Acceptance Criteria

- Maximum pitch: ±2 degrees = ±0.0349 radians
- Maximum lateral offset: ±15px at 200w terminal width
  - At typical 1920px: ±15 * (1920/200) = ±144px, but spec says "at 200w"
  - `project_x()` applies `yaw_offset * depth_scale`, so the offset scales with depth
  - For depth_scale=1.0 (nearest), yaw_offset=15 means ±15px shift at 200w
- All movements spring-damped (no discontinuous jumps)
- Camera dynamics tunable via constants

## Speed Normalization

The ticket pseudocode uses `speed_normalized` and `world.speed / 300.0`.
After T-007-02, the speed range is 30–900. The max meaningful speed for
normalization purposes is ~300 (the `speed_target` cap). Using
`(speed / 300.0).clamp(0.0, 1.0)` matches the existing pattern in
`road.rs:horizon_ratio()` and `effects.rs:draw_speed_lines()`.

## Existing `road.rs` Functions

`road.rs` has standalone `horizon_ratio()` and `road_max_half()` functions
that duplicate logic now in `Camera::sync()`. These may need cleanup but
that's outside this ticket's scope.

## Summary of Gaps

1. `Camera::sync()` doesn't update `pitch`, `yaw_offset`, or `fov_scale`
2. No slope sampling at camera position during world update
3. No burst detection flag for zoom lurch
4. No camera shake logic
5. `vanishing_point()` doesn't account for `yaw_offset`
6. No spring-damped FOV widening
