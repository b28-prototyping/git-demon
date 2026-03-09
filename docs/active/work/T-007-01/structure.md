# T-007-01 Structure: Camera Struct and Unified Projection

## New Files

### `src/world/camera.rs`

```
pub const NEAR_PLANE: f32 = 10.0;
pub const ROAD_HALF_WORLD: f32 = 480.0;
const BASE_HORIZON_RATIO: f32 = 0.25;

pub struct Camera {
    pub z: f32,
    pub pitch: f32,
    pub yaw_offset: f32,
    pub fov_scale: f32,
    pub horizon_ratio: f32,
    pub draw_distance: f32,
    pub near_plane: f32,
}

impl Camera {
    pub fn new() -> Self
    pub fn update(&mut self, dt: f32, speed: f32, tier: VelocityTier, curve_offset: f32)
    pub fn project(&self, z_world: f32, screen_h: u32, horizon_y: u32) -> Option<(f32, f32)>
    pub fn project_x(&self, x_offset: f32, depth_scale: f32, screen_w: u32) -> f32
    pub fn vanishing_point(&self, screen_w: u32, screen_h: u32) -> (f32, f32)
    pub fn road_half(&self, depth_scale: f32) -> f32
    pub fn horizon_y(&self, screen_h: u32) -> u32
}

#[cfg(test)] mod tests — unit tests for all Camera methods
```

## Modified Files

### `src/world/mod.rs`

- Add `pub mod camera;`
- Add `camera: Camera` field to `WorldState`
- `WorldState::new()`: initialize `camera: Camera::new()`
- `WorldState::update()`: call `self.camera.update(dt, self.speed, self.tier, self.curve_offset)` instead of manually advancing camera_z
- `WorldState::draw_distance()` → delegate to `self.camera.draw_distance`
- Keep `camera_z` as a convenience alias: `pub fn camera_z(&self) -> f32 { self.camera.z }` or just access `world.camera.z`
- Remove `camera_z` field (replace with `world.camera.z` everywhere)
- Update all test helpers to include camera field

### `src/renderer/mod.rs`

- Change `horizon_y` computation: `let horizon_y = world.camera.horizon_y(h);`
- Remove `road::horizon_ratio(world)` call

### `src/renderer/road.rs`

- Remove `horizon_ratio()` pub fn (moved to Camera)
- Remove `road_max_half()` pub fn (replaced by Camera::road_half)
- Remove `BASE_HORIZON_RATIO` and `ROAD_MAX_HALF` constants
- `draw_grid()`: Replace inline depth/spread with `world.camera.project()` and `world.camera.road_half()`
- `draw_road()`: Unchanged (uses screen-space depth for gradient, no world projection needed)
- Update tests to use Camera-based horizon

### `src/renderer/sprites.rs`

- Remove `ROAD_MIN_HALF` constant
- Remove local `project()` function
- Replace with calls to `world.camera.project()` and `world.camera.project_x()`
- Lane offsets computed via `camera.road_half(depth_scale) * lane_factor`
- Sprite sizing: use `depth_scale` directly (not squared) for consistent 1/z scaling
- Update tests

### `src/renderer/terrain.rs`

- `draw_islands()`: Replace inline depth projection with `world.camera.project()`
- Lateral positioning: use `camera.project_x()` with world_x offset
- `draw_clouds()`: Same changes plus altitude offset applied to projected screen_y
- Update reference to `road::road_max_half` → `world.camera.road_half()`

### `src/renderer/sky.rs`

- `draw_stars()`: Change vanishing point from `(w/2, horizon_y * 0.6)` to `world.camera.vanishing_point(w, h)` — note: vanishing_point returns `(cx, horizon_y)`, so stars now converge at horizon instead of 60% above
- No other projection changes (stars are screen-space procedural)

### `src/renderer/effects.rs`

- `draw_speed_lines()`: Change vanishing point from `(w/2, horizon_y)` to `world.camera.vanishing_point(w, h)` — already consistent but now uses Camera API
- `draw_car()`: Unchanged (fixed screen position, no projection)

### `src/renderer/hud.rs`

- No changes (no projection involved)

## Deleted Code

- `road::horizon_ratio()` — logic moves to `Camera::update()`
- `road::road_max_half()` — replaced by `Camera::road_half()`
- `road::BASE_HORIZON_RATIO` — moves to camera.rs
- `road::ROAD_MAX_HALF` — becomes `camera::ROAD_HALF_WORLD`
- `sprites::ROAD_MIN_HALF` — no longer needed with 1/z
- `sprites::project()` — replaced by Camera methods
- `world::WorldState::camera_z` field — becomes `world.camera.z`
- `world::WorldState::draw_distance()` — delegates to camera

## Module Dependencies

```
world/camera.rs depends on: world/speed.rs (VelocityTier)
world/mod.rs    depends on: world/camera.rs (Camera)
renderer/*      depends on: world/mod.rs (WorldState, which contains Camera)
```

No circular dependencies. Camera is a leaf module in the world package.

## Public Interface Changes

- `road::horizon_ratio()` removed — callers use `world.camera.horizon_ratio` or `world.camera.horizon_y(h)`
- `road::road_max_half()` removed — callers use `world.camera.road_half(depth_scale)`
- `sprites::ROAD_MIN_HALF` removed
- `sprites::project()` visibility can become private or removed
- `WorldState.camera_z` field → `WorldState.camera.z`
- `WorldState.draw_distance()` still works, delegates to camera

## Ordering

1. camera.rs first (no dependents yet)
2. world/mod.rs (add camera field, update helpers)
3. renderer/mod.rs (switch horizon computation)
4. road.rs (migrate grid, remove old functions)
5. sprites.rs (migrate project)
6. terrain.rs (migrate depth)
7. sky.rs + effects.rs (vanishing point)
8. Remove dead code, final cleanup
