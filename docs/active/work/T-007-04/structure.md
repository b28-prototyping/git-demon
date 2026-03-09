# T-007-04 Structure: Dynamic Camera Response

## Files Modified

### `src/world/camera.rs`

**Constants added (module level):**
```
FOV_SPEED_SCALE    = 0.25   // Max FOV widening at max speed
FOV_SPRING         = 4.0    // FOV response rate
PITCH_SLOPE_FACTOR = 0.03   // Radians of pitch per unit slope
PITCH_SPRING       = 3.0    // Pitch response rate
PITCH_MAX          = 0.0349 // ±2 degrees in radians
YAW_CURVE_FACTOR   = 0.15   // Pixels of yaw per unit curve_offset
YAW_SPRING         = 2.0    // Yaw response rate
YAW_MAX            = 15.0   // Maximum lateral offset in pixels
SHAKE_X_AMP        = 1.5    // Lateral shake amplitude (pixels)
SHAKE_Y_AMP        = 0.001  // Pitch shake amplitude (radians)
SHAKE_X_FREQ       = 47.0   // Hz
SHAKE_Y_FREQ       = 31.0   // Hz
BURST_FOV_OFFSET   = -0.1   // Zoom-in on burst
BURST_FOV_SPRING   = 2.0    // Burst recovery rate
```

**Camera struct changes:**
- Add field `burst_fov_offset: f32` (private spring state for burst zoom)

**Method changes:**
- Rename `sync()` → `update()` with expanded signature:
  ```rust
  pub fn update(
      &mut self,
      dt: f32,
      speed: f32,
      curve_offset: f32,
      slope: f32,
      tier: VelocityTier,
      time: f32,
      burst_active: bool,
  )
  ```
- Body of `update()`:
  1. Compute FOV target from speed + burst, spring-damp `fov_scale`
  2. Compute pitch target from slope, clamp, spring-damp `pitch`
  3. Compute yaw target from curve_offset, clamp, spring-damp `yaw_offset`
  4. Apply shake additively if tier == VelocityDemon
  5. Update draw_distance and horizon_ratio (existing sync logic)
  6. Decay burst_fov_offset toward 0.0

**New() and Default changes:**
- Initialize `burst_fov_offset: 0.0`

**Trigger burst method:**
```rust
pub fn trigger_burst(&mut self) {
    self.burst_fov_offset = BURST_FOV_OFFSET;
}
```

### `src/world/mod.rs`

**WorldState changes:**
- Add field `burst_cooldown: f32` (initialized to 0.0)

**update() changes:**
- Compute slope at camera position from segments:
  ```rust
  let slope = self.slope_at_camera();
  ```
- Replace `self.camera.sync(speed, tier)` with:
  ```rust
  self.camera.update(dt, self.speed, self.curve_offset, slope, self.tier, self.time, self.burst_cooldown > 0.0);
  ```
- Decay `burst_cooldown`:
  ```rust
  self.burst_cooldown = (self.burst_cooldown - dt).max(0.0);
  ```

**ingest_poll() changes:**
- On burst detection (`commits_per_min > 1.0`), set `burst_cooldown = 0.5`
  and call `self.camera.trigger_burst()`

**New helper method:**
```rust
fn slope_at_camera(&self) -> f32 {
    // Index into segments array based on camera_z position
}
```

### `src/world/road_segments.rs`

No changes.

### `src/renderer/` files

No changes needed. All renderers already consume `camera.pitch`, `camera.yaw_offset`,
and `camera.fov_scale` through the Camera API methods (`pitch_offset()`, `project_x()`,
`road_half()`). Dynamic values will flow through automatically.

### Test files

**`src/world/camera.rs` tests:**
- Test FOV increases with speed
- Test FOV springs toward target (not instant)
- Test pitch responds to slope
- Test pitch clamped to ±PITCH_MAX
- Test yaw responds to curve_offset
- Test yaw clamped to ±YAW_MAX
- Test shake only at VelocityDemon
- Test burst zoom triggers and recovers
- Test all values spring-damped (no discontinuities)

**`src/world/mod.rs` tests:**
- Test burst_cooldown set on high-cpm poll
- Test burst_cooldown decays in update
- Test slope_at_camera returns reasonable values

## Module Boundaries

Camera knows about:
- VelocityTier (for shake activation and draw_distance)
- Spring constants and limits (self-contained tuning)

Camera does NOT know about:
- Road segments (slope passed as parameter)
- Git activity (burst triggered externally)
- Curve generation (curve_offset passed as parameter)

WorldState orchestrates:
- Computes slope from segments
- Detects bursts from poll results
- Passes all parameters to Camera::update()

## Ordering

1. Add constants and `burst_fov_offset` field to Camera
2. Implement `Camera::update()` and `Camera::trigger_burst()`
3. Add `burst_cooldown` and `slope_at_camera()` to WorldState
4. Wire `WorldState::update()` to call `Camera::update()`
5. Wire `WorldState::ingest_poll()` to trigger burst
6. Add tests for Camera dynamics
7. Add tests for WorldState burst/slope integration
