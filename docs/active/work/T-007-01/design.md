# T-007-01 Design: Camera Struct and Unified Projection

## Decision: Approach Selection

### Option A: Camera struct with 1/z projection (ticket spec)

Add `Camera` struct to `src/world/camera.rs` with `project(z_world) -> Option<(screen_y, depth_scale)>` using true reciprocal depth: `depth_scale = near_plane / z_rel`. All subsystems call `camera.project()` instead of ad-hoc formulas.

**Pros**: True 1/z gives better near-geometry stretching. Single projection eliminates inconsistencies. Clean separation of camera logic from world state.

**Cons**: Changes visual output at every depth — every test comparing pixel positions will break. 1/z compression near horizon means grid lines bunch up more aggressively. Requires `NEAR_PLANE` calibration.

### Option B: Unified linear projection, defer 1/z to T-007-02

Add Camera struct but keep the current `depth_scale = (1 - z_rel / draw_dist)` linear model. Unify all subsystems to use the same formula and vanishing point. 1/z switch happens in a later ticket.

**Pros**: Minimal visual disruption. Fixes inconsistencies without changing the depth model. Tests need only minor updates. Lower risk.

**Cons**: Doesn't deliver the 1/z perspective that S-007 is about. Two-step migration means Camera API needs to accommodate both models.

### Option C: Camera trait with swappable projection

Define a `Projection` trait and let Camera hold a strategy. Start with linear, swap to 1/z.

**Pros**: Maximum flexibility.

**Cons**: Over-engineered for the actual use case. Only one projection model will ever be active.

## Decision: Option A — Camera with 1/z projection

Rationale: The ticket explicitly specifies 1/z projection and the acceptance criteria require "Near geometry is visually stretched vs far geometry (1/z effect visible)." Deferring 1/z defeats the purpose. The visual change is expected and desired. Test updates are mechanical.

## Camera Struct Design

```rust
pub struct Camera {
    pub z: f32,              // world-Z position (was world.camera_z)
    pub pitch: f32,          // vertical tilt (radians), unused initially but reserved
    pub yaw_offset: f32,     // lateral offset from curvature lag
    pub fov_scale: f32,      // 1.0 default, >1.0 wider
    pub horizon_ratio: f32,  // computed: screen fraction top-to-horizon
    pub draw_distance: f32,  // max visible Z
    pub near_plane: f32,     // calibration constant for 1/z
}
```

### Projection Function

```
project(z_world, screen_h, horizon_y) -> Option<(screen_y, depth_scale)>
  z_rel = z_world - self.z
  if z_rel <= near_plane || z_rel > draw_distance: None
  depth_scale = near_plane / z_rel  // 1/z: 1.0 at near_plane, ~0 at far
  screen_y = horizon_y + (screen_h - horizon_y) * depth_scale
```

### NEAR_PLANE Calibration

Current linear model: at `z_rel = 0`, `depth_scale = 1.0` (bottom of screen). With 1/z, `depth_scale = near_plane / z_rel`. For `depth_scale = 1.0` at camera's feet, `near_plane` equals the minimum visible z_rel. Looking at sprite culling: objects at `z_rel <= 0` are culled. Road surface starts at horizon and fills to bottom. A near_plane of ~10.0 means objects at z_rel=10 fill the screen height below horizon. This is reasonable — closer objects are behind the camera.

### project_x()

```
project_x(x_offset, depth_scale, screen_w) -> screen_x
  cx = screen_w / 2.0 + yaw_offset * depth_scale
  cx + x_offset * depth_scale * fov_scale
```

This replaces all the ad-hoc lateral spread formulas. Road half-width at any depth = `road_half_world * depth_scale` where `road_half_world` is the physical road width. No more `ROAD_MIN_HALF`.

### Vanishing Point

```
vanishing_point(screen_w, screen_h) -> (f32, f32)
  (screen_w / 2.0, screen_h * horizon_ratio)
```

All systems (stars, speed lines, grid convergence) use this single point.

## Camera::update()

Move horizon_ratio computation, draw_distance selection, and camera_z advancement into `Camera::update(dt, speed, tier, curve_offset)`. This removes `horizon_ratio()` and `road_max_half()` as standalone functions from road.rs.

## What Gets Rejected

- **Separate `Projection` trait**: Not needed. Only one projection model.
- **Camera as separate parameter**: Adding a second parameter to every draw function clutters signatures. Instead, `Camera` lives inside `WorldState` so all subsystems access it via `world.camera`.
- **Keeping ROAD_MIN_HALF**: This constant only existed because linear depth produced zero spread at horizon. With 1/z, far objects naturally have small but nonzero spread. The road width at any depth is `ROAD_HALF_WORLD * depth_scale`.
- **Keeping horizon_ratio() as pub fn**: Move into Camera::update(). Callers access `world.camera.horizon_ratio`.

## Migration Strategy

1. Add Camera struct and place it in WorldState
2. Migrate road.rs grid to use camera.project()
3. Migrate sprites.rs to use camera.project() + camera.project_x()
4. Migrate terrain.rs to use camera.project()
5. Update sky.rs and effects.rs vanishing points
6. Remove orphaned functions (horizon_ratio, road_max_half, ROAD_MIN_HALF)
7. Update all test helpers

## Visual Impact

The 1/z projection will compress far geometry more and stretch near geometry. Grid lines near the camera will be spaced further apart. Objects near the camera will appear larger relative to the linear model. This is the intended "ground rushing under you" effect from S-007.

## Road Half-Width Constant

Currently `ROAD_MAX_HALF = 480.0` is a screen-pixel value at max depth_scale. With 1/z, we need a world-space road width. At `depth_scale = 1.0` (near_plane), `road_half_world * 1.0` should equal the screen half-width of the road. Setting `ROAD_HALF_WORLD = 480.0` preserves the near-camera appearance. The VelocityDemon 1.05x multiplier applies to this world constant.
