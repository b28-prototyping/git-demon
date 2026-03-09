# T-007-01 Plan: Camera Struct and Unified Projection

## Step 1: Create `src/world/camera.rs` with Camera struct and tests

- Define `Camera` struct with fields: z, pitch, yaw_offset, fov_scale, horizon_ratio, draw_distance, near_plane
- Implement `Camera::new()` with sensible defaults
- Implement `Camera::project(z_world, screen_h, horizon_y) -> Option<(f32, f32)>` with 1/z
- Implement `Camera::project_x(x_offset, depth_scale, screen_w) -> f32`
- Implement `Camera::vanishing_point(screen_w, screen_h) -> (f32, f32)`
- Implement `Camera::road_half(depth_scale) -> f32`
- Implement `Camera::horizon_y(screen_h) -> u32`
- Implement `Camera::update(dt, speed, tier, curve_offset)`
- Write unit tests: project returns None behind camera, None beyond draw_distance, correct depth_scale at known z, screen_y monotonicity, project_x centering, vanishing_point consistency, road_half scaling
- **Verify**: `cargo test -p git-demon world::camera` passes

## Step 2: Integrate Camera into WorldState

- Add `pub mod camera;` to `src/world/mod.rs`
- Add `pub camera: Camera` field to `WorldState`
- Initialize in `WorldState::new()`: `camera: Camera::new()` with draw_distance from tier
- In `WorldState::update()`: call `self.camera.update(dt, self.speed, self.tier, self.curve_offset)` and sync `self.camera.z` advancement (replace `self.camera_z += speed * dt`)
- Remove `camera_z` field — replace all `self.camera_z` with `self.camera.z`
- Delegate `draw_distance()` to `self.camera.draw_distance`
- Update all test helpers in `world/mod.rs` tests to construct WorldState with camera field
- **Verify**: `cargo test -p git-demon world::` passes

## Step 3: Update renderer/mod.rs horizon computation

- Change line 108: `let horizon_y = world.camera.horizon_y(h);`
- Remove `use` of `road::horizon_ratio` if present
- **Verify**: `cargo build` succeeds

## Step 4: Migrate road.rs to Camera projection

- In `draw_grid()` horizontal lines: replace `depth_scale = (1.0 - z_world / world.draw_distance())` with `world.camera.project(z_world_abs, h, horizon_y)` where z_world_abs = world.camera.z + z_world
- In `draw_grid()` vertical lines: replace `lerp(0, max_half * 1.5, depth)` with `world.camera.road_half(depth_scale)` using depth from camera.project per-scanline (or keep screen-space iteration and compute depth_scale per row)
- Remove `horizon_ratio()` function
- Remove `road_max_half()` function
- Remove `BASE_HORIZON_RATIO` and `ROAD_MAX_HALF` constants
- Update road.rs tests: replace `horizon_ratio(&world)` calls with `world.camera.horizon_y(h)`, update test helpers
- **Verify**: `cargo test -p git-demon renderer::road` passes

## Step 5: Migrate sprites.rs to Camera projection

- Replace `project()` function body: call `world.camera.project(z_world, pixel_h, horizon_y)` for screen_y and depth_scale
- Use `world.camera.road_half(depth_scale)` for road width
- Compute lane_x via road_half * lane_factor instead of lerp(ROAD_MIN_HALF, max_half, depth_scale)
- Add curve shift: `world.camera.project_x(lane_offset, depth_scale, pixel_w)` or manual cx + offset
- Remove `ROAD_MIN_HALF`
- Change sprite sizing from `scale * scale` to `scale` (1/z already compresses far objects)
- Update sprites.rs tests: replace direct WorldState construction, update expected scale values
- **Verify**: `cargo test -p git-demon renderer::sprites` passes

## Step 6: Migrate terrain.rs to Camera projection

- `draw_islands()`: Replace inline depth computation with `world.camera.project(z_abs, h, horizon_y)`
- Replace lateral spread with `world.camera.project_x()` or equivalent using road_half
- `draw_clouds()`: Same migration, keeping cloud altitude offset
- Remove `use crate::renderer::road` import
- Update terrain tests
- **Verify**: `cargo test -p git-demon renderer::terrain` passes

## Step 7: Unify vanishing points in sky.rs and effects.rs

- `sky::draw_stars()`: Change `cy = horizon_y as f32 * 0.6` to `let (_, cy) = world.camera.vanishing_point(w, horizon_y * 2)` or simply use `horizon_y as f32` — accept the visual change of stars converging at horizon
- Actually: keep stars at `horizon_y * 0.6` but document that this is intentional parallax offset, not a projection inconsistency. The stars are a background layer and should NOT converge at the road vanishing point.
- `effects::draw_speed_lines()`: Already at `horizon_y`. Optionally switch to `world.camera.vanishing_point()` for consistency.
- **Verify**: `cargo test` full suite passes

## Step 8: Update hud.rs test helper

- The `make_world()` in hud.rs tests is missing gear/rpm/throttle fields — add camera field too
- **Verify**: `cargo test -p git-demon renderer::hud` passes

## Step 9: Final cleanup and verification

- `cargo clippy` — fix any warnings
- `cargo test` — all tests pass
- Remove any dead imports
- Verify no pub functions were removed that are used outside the crate

## Testing Strategy

### Unit tests (Step 1, in camera.rs)
- project() returns None for z behind camera
- project() returns None for z beyond draw_distance
- project() at near_plane returns depth_scale ≈ 1.0
- project() depth_scale decreases with distance
- project() screen_y is between horizon_y and screen_h
- project_x() returns screen center when x_offset=0 and yaw_offset=0
- road_half() returns ROAD_HALF_WORLD at depth_scale=1.0
- horizon_y() returns correct pixel row
- update() advances z by speed*dt

### Integration tests (existing tests updated)
- Road grid still renders visible grid lines
- Grid lines still move with camera_z
- Sprites project correctly (near > far in screen Y)
- Terrain stays below horizon
- All existing behavior tests pass with updated expected values

### Visual verification (manual, not automated)
- Near grid lines are wider-spaced than far ones (1/z effect)
- Vanishing point appears consistent across road and speed lines
- No visual artifacts at extreme speeds or curvature
