# T-007-04 Plan: Dynamic Camera Response

## Step 1: Camera struct changes

**Files:** `src/world/camera.rs`

- Add all tuning constants at module level
- Add `burst_fov_offset: f32` field to Camera struct
- Update `new()` and `Default` to initialize it to 0.0
- Add `trigger_burst()` method
- Rename `sync()` to `update()` with the expanded signature
- Implement the body:
  - FOV: target = 1.0 + speed_t * FOV_SPEED_SCALE + burst_fov_offset, spring-damp
  - Pitch: target = -slope * PITCH_SLOPE_FACTOR, clamp ±PITCH_MAX, spring-damp
  - Yaw: target = -curve_offset * YAW_CURVE_FACTOR, clamp ±YAW_MAX, spring-damp
  - Shake: if VelocityDemon, add sin waves to yaw_offset and pitch
  - Draw distance and horizon ratio: keep existing logic
  - Burst: spring burst_fov_offset toward 0.0

**Verification:** `cargo build` compiles (will fail until Step 2 fixes callers)

## Step 2: WorldState integration

**Files:** `src/world/mod.rs`

- Add `burst_cooldown: f32` field, initialized to 0.0 in `new()`
- Add `slope_at_camera()` helper method
- In `update()`:
  - Compute slope via `slope_at_camera()`
  - Decay `burst_cooldown`
  - Replace `camera.sync(speed, tier)` with `camera.update(dt, speed, curve_offset, slope, tier, time, burst_cooldown > 0.0)`
- In `ingest_poll()`:
  - When `commits_per_min > 1.0`, set `burst_cooldown = 0.5` and call `camera.trigger_burst()`

**Verification:** `cargo build` succeeds

## Step 3: Fix test helpers and callers

**Files:** `src/world/camera.rs` (tests), `src/renderer/road.rs` (tests),
           `src/renderer/effects.rs` (tests), any other files calling `camera.sync()`

- Update all `camera.sync()` call sites to use `camera.update()` with appropriate params
- Fix test helper functions that construct WorldState or call sync()
- Update effects.rs `make_world()` test helper to include `burst_cooldown`

**Verification:** `cargo test` passes

## Step 4: Camera unit tests

**Files:** `src/world/camera.rs`

Add tests:
- `test_fov_increases_with_speed` — update with high speed, verify fov_scale > 1.0
- `test_fov_spring_damped` — single dt step doesn't jump to target
- `test_pitch_responds_to_slope` — uphill slope → positive pitch
- `test_pitch_clamped` — extreme slope clamped to ±PITCH_MAX
- `test_yaw_responds_to_curve` — positive curve → negative yaw
- `test_yaw_clamped` — extreme curve clamped to ±YAW_MAX
- `test_shake_only_velocity_demon` — non-VelocityDemon tiers have no shake
- `test_burst_zoom_and_recovery` — trigger burst, verify fov dip, verify recovery
- `test_all_springs_converge` — run many frames, verify values approach targets

**Verification:** `cargo test` passes

## Step 5: WorldState integration tests

**Files:** `src/world/mod.rs`

Add tests:
- `test_slope_at_camera_returns_segment_slope`
- `test_burst_cooldown_set_on_high_cpm`
- `test_burst_cooldown_decays`
- `test_camera_pitch_after_update` — verify pitch changes during world update with hilly segments

**Verification:** `cargo test` passes

## Step 6: Final validation

- `cargo clippy` — no warnings
- `cargo test` — all pass
- `cargo build` — clean build

## Testing Strategy

- **Unit tests** for Camera dynamics: isolated spring behavior, clamping, shake activation
- **Integration tests** for WorldState: verify camera state after world update ticks
- **No visual tests** — camera dynamics are verified through numeric assertions
- **Regression** — existing camera tests (project, road_half, etc.) must still pass
