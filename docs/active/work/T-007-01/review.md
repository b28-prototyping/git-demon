# T-007-01 Review: Camera Struct and Unified Projection

## Summary of Changes

### New Files
| File | Lines | Purpose |
|---|---|---|
| `src/world/camera.rs` | ~140 | Camera struct with 1/z projection, sync, vanishing_point, road_half |

### Modified Files
| File | Change |
|---|---|
| `src/world/mod.rs` | Added `camera: Camera` field, `pub mod camera`, sync in update() |
| `src/renderer/mod.rs` | Horizon computed via `world.camera.horizon_y(h)` |
| `src/renderer/road.rs` | Grid uses `camera.project()` and `camera.road_half()`. Removed production `lerp()` |
| `src/renderer/sprites.rs` | project() delegates to `camera.project()` + `camera.road_half()`. Removed `ROAD_MIN_HALF`, `use road` |
| `src/renderer/terrain.rs` | Islands/clouds use `camera.project()` + `camera.road_half()`. Removed `use road` |
| `src/renderer/effects.rs` | Speed lines use `camera.vanishing_point()`. Added camera to test helper |
| `src/renderer/sky.rs` | Added documentation comment for star parallax offset |
| `src/renderer/hud.rs` | Added camera + gear/rpm/throttle to test helper |

### Deleted Code
- `sprites::ROAD_MIN_HALF` constant
- Production `road::lerp()` (kept in test module)
- Inline depth formulas in sprites.rs, terrain.rs (replaced by camera.project())

## Acceptance Criteria Status

- [x] `Camera` struct exists with z, pitch, yaw_offset, fov_scale, draw_distance
- [x] Single `project(z_world)` function uses 1/z (reciprocal depth) instead of linear lerp
- [x] Road grid, sprites, terrain, and effects all use the same projection
- [x] Vanishing point is consistent across road/effects subsystems (stars have intentional parallax offset)
- [x] Near geometry is visually stretched vs far geometry (1/z effect: NEAR_PLANE/z_rel)
- [x] Existing tests pass; new unit tests for Camera::project() edge cases
- [ ] No visual regression at idle speed — cannot verify automatically; requires manual screenshot comparison

## Test Coverage

### New tests (16 in camera.rs)
- project() behind camera → None
- project() at near_plane → depth_scale=1.0, screen_y=screen_h
- project() beyond draw_distance → None
- project() depth_scale decreases with distance
- project() screen_y monotonic (near > far)
- project() screen_y within bounds
- project() 1/z values at known distances (z_rel=100→0.1, z_rel=50→0.2)
- project_x() centered, offset
- vanishing_point()
- road_half() at full scale, scales with depth
- horizon_y()
- sync() draw_distance for VelocityDemon
- sync() horizon_ratio at speed, VelocityDemon

### Updated tests
- road.rs: 15 tests — horizon_ratio tests now check camera.horizon_ratio, grid tests sync camera.z
- sprites.rs: 18 tests — z_world values adjusted for 1/z model
- terrain.rs: 3 tests — camera.z synced
- effects.rs: 15 tests — camera field added to test helper
- hud.rs: 12 tests — camera + gear/rpm/throttle fields added
- world/mod.rs: 17 tests — unchanged (WorldState::new() handles camera init)

Total: 180 tests pass (163 lib + 8 bin + 9 integration), 0 clippy warnings.

## Open Concerns

1. **Dual camera state**: `WorldState.camera_z` and `WorldState.camera.z` are kept in sync manually. This is tech debt — a future ticket should remove `camera_z` and have all code access `camera.z` directly. Risk: someone sets one without the other.

2. **Legacy pub functions**: `road::horizon_ratio()` and `road::road_max_half()` still exist but are unused by renderer code. They may be referenced by benchmarks. Should be removed once all callers migrate.

3. **Visual verification needed**: The 1/z projection changes the visual appearance of the grid, sprites, and terrain. Near geometry will be stretched and far geometry compressed compared to the old linear model. This is the intended effect but should be manually verified to ensure no glitches.

4. **NEAR_PLANE=10.0 calibration**: This value means objects at z_rel=10 fill the screen below horizon. If objects spawn too close (z_rel < 10), they'll be culled. Current minimum spawn distance is much larger so this is safe, but worth noting.

5. **Sprite sizing unchanged**: Sprites still use `depth_scale * depth_scale` for width/height. With 1/z depth_scale, this means sprites are `(NEAR_PLANE/z_rel)^2` which falls off very fast. Visually this may make far sprites too small. A future ticket (T-007-02 or T-007-03) should evaluate whether `depth_scale` (not squared) is better for sizing.

6. **Stars parallax**: Stars converge at `horizon_y * 0.6`, documented as intentional. This means the starfield and road don't share the same vanishing point. If a future ticket adds full parallax layering (T-007-04?), this offset should be revisited.
