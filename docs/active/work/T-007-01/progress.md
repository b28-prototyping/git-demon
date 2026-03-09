# T-007-01 Progress: Camera Struct and Unified Projection

## Completed Steps

### Step 1: Create `src/world/camera.rs` ✓
- Created Camera struct with z, pitch, yaw_offset, fov_scale, horizon_ratio, draw_distance, near_plane
- Implemented project() with 1/z depth: `depth_scale = near_plane / z_rel`
- Implemented project_x(), vanishing_point(), road_half(), horizon_y()
- Implemented sync() to update derived values from world state
- Added Default impl per clippy
- 16 unit tests covering all methods and edge cases

### Step 2: Integrate Camera into WorldState ✓
- Added `pub mod camera` to world/mod.rs
- Added `camera: Camera` field to WorldState
- WorldState::new() initializes camera
- WorldState::update() calls camera.sync() with current speed/tier
- camera.z kept in sync with legacy camera_z field
- Fixed effects.rs and hud.rs test helpers (missing camera field)

### Step 3: Update renderer/mod.rs ✓
- Changed horizon_y computation to use `world.camera.horizon_y(h)`

### Step 4: Migrate road.rs to Camera projection ✓
- draw_grid() horizontal lines now use camera.project() for 1/z depth
- draw_grid() vertical lines now use camera.road_half() for spread
- Increased grid iteration from 40 to 80 lines (1/z compresses far space, need more lines to fill)
- Removed production lerp() function (moved to test-only)
- Updated all test helpers to sync camera state
- Updated horizon_ratio tests to test camera.horizon_ratio
- Removed road_max_half tests (functionality moved to Camera::road_half)

### Step 5: Migrate sprites.rs to Camera projection ✓
- Replaced inline project() with calls to camera.project() + camera.road_half()
- Removed ROAD_MIN_HALF constant
- Removed `use super::road` import
- Updated test z_world values for 1/z model (z=20 for scale=0.5, z=200 for dot LOD, z=100 for far text suppression)
- All 18 sprite tests pass

### Step 6: Migrate terrain.rs to Camera projection ✓
- draw_islands() uses camera.project() for 1/z depth
- draw_clouds() uses camera.project() for depth_scale
- Lateral spread uses camera.road_half() instead of road::road_max_half()
- Removed road module dependency
- Updated tests to sync camera.z

### Step 7: Unify vanishing points ✓
- effects.rs draw_speed_lines() uses camera.vanishing_point()
- sky.rs stars keep 0.6*horizon_y offset (intentional parallax, documented)
- All subsystems now use consistent projection from Camera

### Step 8: Cleanup ✓
- cargo clippy clean (0 warnings)
- cargo test: 180 tests pass (163 lib + 8 bin + 9 integration)
- No dead imports or unused code

## Deviations from Plan

1. **Camera::update() → Camera::sync()**: Changed from having Camera advance its own z to having WorldState set camera.z externally and calling sync() for derived values. This avoids double-advancing z and keeps camera_z as the source of truth during the migration period.

2. **Kept camera_z field**: Instead of removing WorldState.camera_z, kept both camera_z and camera.z in sync. This minimizes churn in callers outside the renderer (e.g., main.rs, git poller). A future ticket can consolidate.

3. **Stars vanishing point not changed**: Per design review, stars at 0.6*horizon_y is intentional parallax layering, not a bug. Added documentation comment.

4. **Kept horizon_ratio() and road_max_half() in road.rs**: These pub functions still exist for backward compatibility. They're unused by renderer code now but might be referenced by benchmarks or external callers. Can be removed in a cleanup ticket.
