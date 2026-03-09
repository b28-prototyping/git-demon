# T-007-04 Design: Dynamic Camera Response

## Decision: Extend Camera::sync() into Camera::update()

### Approach

Replace the current `Camera::sync()` with a `Camera::update()` method that takes
all relevant world-state inputs and applies spring-damped dynamics to `fov_scale`,
`pitch`, `yaw_offset`, plus camera shake. This consolidates all camera behavior
into one method called once per frame from `WorldState::update()`.

### Why Not a Separate CameraDynamics Component?

Considered extracting a `CameraDynamics` struct that would wrap `Camera` and manage
spring targets separately. Rejected because:
- The Camera struct already has all the fields
- Adding a wrapper layer would mean threading two objects through the update call
- The spring targets are simple scalars that can live as private state in Camera
- The codebase prefers flat structs over indirection

### FOV Widening (Feature 1)

**Target:** `fov_scale = 1.0 + speed_t * 0.25` where `speed_t = (speed / 300.0).clamp(0.0, 1.0)`

Spring-damp toward target: `fov_scale += (target - fov_scale) * 4.0 * dt`

Spring constant 4.0 gives ~250ms response — fast enough to feel connected to
acceleration but slow enough to avoid jitter.

The FOV scaling is already wired: `road_half()` and `project_x()` both multiply
by `fov_scale`. No renderer changes needed.

### Pitch Response to Slope (Feature 2)

**Target:** `pitch = -slope * 0.03` (radians)

Clamped to ±0.0349 rad (±2°) per acceptance criteria.
Spring constant 3.0 for a springy lag feel.

Slope at camera position: sample from `segments` array using camera_z position.
This requires passing `segments`, `segment_z_start` into `Camera::update()`,
or pre-computing the slope in `WorldState::update()` and passing it as a parameter.

**Decision:** Pass the slope as a parameter. WorldState computes it from segments
before calling `Camera::update()`. This keeps Camera independent of road segment
internals.

### Lateral Lag from Curvature (Feature 3)

**Target:** `yaw_offset = -curve_offset * 0.15`

Sign is negative because camera lags behind the curve (shifts opposite direction).
The 0.15 multiplier gives visible shift without extreme values.
At curve_offset=80 (max), yaw_offset target = -12px — within the ±15px limit.

Spring constant 2.0 for slow, heavy lateral feel.

Note: `project_x()` already applies `yaw_offset * depth_scale`, so near objects
shift more than far objects — correct perspective behavior.

### Camera Shake at VelocityDemon (Feature 4)

High-frequency additive noise on top of the spring-damped values.
Only active when `tier == VelocityDemon`.

```
shake_x = sin(time * 47.0) * 1.5
shake_y = sin(time * 31.0) * 0.001
```

Applied additively AFTER spring damping, not to the spring target.
This ensures shake doesn't get filtered out by the spring.

47Hz and 31Hz are chosen to be:
- Above any gameplay frequency (steering ~0.3Hz, curve changes ~0.1Hz)
- Incommensurate with each other (no resonance)
- Below visual noise threshold (subtle vibration, not seizure-inducing)

### Speed Burst Zoom (Feature 5)

On `ingest_poll()` with `commits_per_min > 1.0`, set a `burst_fov_offset`
that starts at -0.1 (zoom-in) and decays toward 0.0 each frame.

The total FOV target becomes: `1.0 + speed_t * 0.25 + burst_fov_offset`

Burst decay uses the same spring system: burst_fov_offset springs toward 0.0
at rate 2.0, giving a ~500ms recovery.

**Burst tracking:** Add `burst_cooldown: f32` to `WorldState`. Set to 1.0 on
burst detection in `ingest_poll()`. Decay in `update()`. Camera reads this to
know when to apply the zoom lurch. This avoids adding git-awareness to Camera.

### Vanishing Point Fix

`vanishing_point()` currently returns `(w/2, h * horizon_ratio)`.
It should incorporate `yaw_offset` for consistency:
`(w/2 + yaw_offset, h * horizon_ratio)`

This affects `draw_speed_lines()` — speed streaks will radiate from the
shifted vanishing point, enhancing the curve feel. However, yaw_offset is
small (±15px max) and speed lines already have randomized positions, so the
visual impact is subtle.

**Decision:** Don't change vanishing_point() — it's used for speed line origin
and the shift would be barely noticeable. Keep it centered for visual stability.

### Parameter Summary

| Parameter           | Value   | Spring | Units    |
|---------------------|---------|--------|----------|
| FOV speed scale     | 0.25    | 4.0    | fraction |
| Pitch slope factor  | 0.03    | 3.0    | radians  |
| Pitch max           | 0.0349  | —      | radians  |
| Yaw curve factor    | 0.15    | 2.0    | pixels   |
| Yaw max             | 15.0    | —      | pixels   |
| Shake x amplitude   | 1.5     | —      | pixels   |
| Shake y amplitude   | 0.001   | —      | radians  |
| Shake x frequency   | 47.0    | —      | Hz       |
| Shake y frequency   | 31.0    | —      | Hz       |
| Burst FOV offset    | -0.1    | 2.0    | fraction |

All constants defined at module level for easy tuning.

### Rejected Alternatives

1. **Per-pixel camera transform matrix:** Overkill. The scanline rasterizer
   doesn't use a transform pipeline. Individual field adjustments match the
   architecture.

2. **Camera state machine with modes:** The dynamics are all simultaneous
   and independent. State machines add complexity without benefit.

3. **Separate Camera::shake() method:** Shake is just two sine waves added
   after spring damping. Not worth a separate method call.

4. **Pitch from segment slope with accumulation:** The pitch should reflect
   the immediate road angle, not the accumulated height change. Using the
   current segment's slope directly is physically correct for a vehicle's
   attitude on a slope.
