use super::speed::VelocityTier;

/// Near-plane distance for 1/z projection. Objects at z_rel == NEAR_PLANE
/// have depth_scale == 1.0 (fill the screen below horizon).
pub const NEAR_PLANE: f32 = 10.0;

/// How strongly camera pitch shifts layers vertically on screen.
const PITCH_SENSITIVITY: f32 = 0.5;

/// World-space half-width of the road. At depth_scale=1.0 (camera's feet),
/// the road extends ROAD_HALF_WORLD pixels to each side of center.
pub const ROAD_HALF_WORLD: f32 = 480.0;

const BASE_HORIZON_RATIO: f32 = 0.25;
const BASE_DRAW_DISTANCE: f32 = 5000.0;

// --- Dynamic camera tuning constants ---

/// Max FOV widening at max speed (25% wider).
const FOV_SPEED_SCALE: f32 = 0.25;
/// FOV spring response rate.
const FOV_SPRING: f32 = 4.0;
/// Radians of pitch per unit slope.
const PITCH_SLOPE_FACTOR: f32 = 0.03;
/// Pitch spring response rate.
const PITCH_SPRING: f32 = 3.0;
/// Maximum pitch in radians (±2 degrees).
const PITCH_MAX: f32 = 0.0349;
/// Pixels of yaw per unit curve_offset.
const YAW_CURVE_FACTOR: f32 = 0.15;
/// Yaw spring response rate.
const YAW_SPRING: f32 = 2.0;
/// Maximum lateral offset in pixels.
const YAW_MAX: f32 = 15.0;
/// Lateral shake amplitude in pixels (VelocityDemon only).
const SHAKE_X_AMP: f32 = 1.5;
/// Pitch shake amplitude in radians (VelocityDemon only).
const SHAKE_Y_AMP: f32 = 0.001;
/// Lateral shake frequency in Hz.
const SHAKE_X_FREQ: f32 = 47.0;
/// Pitch shake frequency in Hz.
const SHAKE_Y_FREQ: f32 = 31.0;
/// FOV zoom-in offset on git activity burst.
const BURST_FOV_INITIAL: f32 = -0.1;
/// Burst FOV recovery spring rate.
const BURST_FOV_SPRING: f32 = 2.0;

pub struct Camera {
    /// World-Z position (advances with speed).
    pub z: f32,
    /// Vertical tilt in radians (positive = look up).
    pub pitch: f32,
    /// Lateral offset from curvature lag.
    pub yaw_offset: f32,
    /// FOV multiplier: 1.0 = default, >1.0 = wider.
    pub fov_scale: f32,
    /// Screen fraction from top to horizon (0.0–1.0).
    pub horizon_ratio: f32,
    /// Maximum visible Z depth from camera.
    pub draw_distance: f32,
    /// Near-plane constant for 1/z projection.
    pub near_plane: f32,
    /// Burst zoom offset (negative = zoom in), decays toward 0.
    burst_fov_offset: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        Self {
            z: 0.0,
            pitch: 0.0,
            yaw_offset: 0.0,
            fov_scale: 1.0,
            horizon_ratio: BASE_HORIZON_RATIO,
            draw_distance: BASE_DRAW_DISTANCE,
            near_plane: NEAR_PLANE,
            burst_fov_offset: 0.0,
        }
    }

    /// Trigger a momentary zoom-in from a git activity burst.
    pub fn trigger_burst(&mut self) {
        self.burst_fov_offset = BURST_FOV_INITIAL;
    }

    /// Update all camera dynamics for this frame.
    ///
    /// Applies spring-damped FOV widening, pitch response to slope,
    /// lateral lag from curvature, camera shake at VelocityDemon,
    /// and burst zoom recovery.
    pub fn update(
        &mut self,
        dt: f32,
        speed: f32,
        curve_offset: f32,
        slope: f32,
        tier: VelocityTier,
        time: f32,
    ) {
        let speed_t = (speed / 300.0).clamp(0.0, 1.0);

        // --- FOV: widens with speed, dips on burst ---
        let fov_target = 1.0 + speed_t * FOV_SPEED_SCALE + self.burst_fov_offset;
        self.fov_scale += (fov_target - self.fov_scale) * FOV_SPRING * dt;

        // Decay burst offset toward 0
        self.burst_fov_offset += (0.0 - self.burst_fov_offset) * BURST_FOV_SPRING * dt;

        // --- Pitch: responds to road slope ---
        let pitch_target = (-slope * PITCH_SLOPE_FACTOR).clamp(-PITCH_MAX, PITCH_MAX);
        self.pitch += (pitch_target - self.pitch) * PITCH_SPRING * dt;

        // --- Yaw: lateral lag from curvature ---
        let yaw_target = (-curve_offset * YAW_CURVE_FACTOR).clamp(-YAW_MAX, YAW_MAX);
        self.yaw_offset += (yaw_target - self.yaw_offset) * YAW_SPRING * dt;

        // --- Shake at VelocityDemon ---
        if tier == VelocityTier::VelocityDemon {
            self.yaw_offset += (time * SHAKE_X_FREQ).sin() * SHAKE_X_AMP;
            self.pitch += (time * SHAKE_Y_FREQ).sin() * SHAKE_Y_AMP;
        }

        // --- Draw distance ---
        self.draw_distance = if tier == VelocityTier::VelocityDemon {
            BASE_DRAW_DISTANCE * 1.2
        } else {
            BASE_DRAW_DISTANCE
        };

        // --- Horizon ratio ---
        let base = if tier == VelocityTier::VelocityDemon {
            BASE_HORIZON_RATIO + 0.02
        } else {
            BASE_HORIZON_RATIO
        };
        self.horizon_ratio = base - speed_t * 0.06;
    }

    /// Synchronize camera parameters from world state (non-dynamic path).
    /// Position (z) is set externally; this updates derived values.
    pub fn sync(&mut self, speed: f32, tier: VelocityTier) {
        self.update(0.0, speed, 0.0, 0.0, tier, 0.0);
    }

    /// Project a world-Z coordinate to screen space using 1/z depth.
    /// Returns `Some((screen_y, depth_scale))` where depth_scale is in (0, 1].
    /// Returns `None` if the object is behind the camera or beyond draw distance.
    pub fn project(&self, z_world: f32, screen_h: u32, horizon_y: u32) -> Option<(f32, f32)> {
        let z_rel = z_world - self.z;
        if z_rel < self.near_plane || z_rel > self.draw_distance {
            return None;
        }
        let depth_scale = (self.near_plane / z_rel).clamp(0.0, 1.0);
        let screen_y = horizon_y as f32 + (screen_h as f32 - horizon_y as f32) * depth_scale;
        Some((screen_y, depth_scale))
    }

    /// Project a world-X offset at a given depth_scale to screen-X.
    pub fn project_x(&self, x_offset: f32, depth_scale: f32, screen_w: u32) -> f32 {
        let cx = screen_w as f32 / 2.0 + self.yaw_offset * depth_scale;
        cx + x_offset * depth_scale * self.fov_scale
    }

    /// The screen point where all perspective lines converge.
    pub fn vanishing_point(&self, screen_w: u32, screen_h: u32) -> (f32, f32) {
        (screen_w as f32 / 2.0, screen_h as f32 * self.horizon_ratio)
    }

    /// Road half-width at a given depth_scale.
    pub fn road_half(&self, depth_scale: f32) -> f32 {
        ROAD_HALF_WORLD * depth_scale * self.fov_scale
    }

    /// Compute horizon pixel row for a given screen height.
    pub fn horizon_y(&self, screen_h: u32) -> u32 {
        (screen_h as f32 * self.horizon_ratio) as u32
    }

    /// Screen-Y pixel offset for a parallax layer when camera is pitched.
    /// `parallax_factor`: 0.0 = infinitely far (full shift), 1.0 = ground plane (no shift).
    /// Positive pitch (look up) shifts distant layers downward on screen.
    pub fn pitch_offset(&self, parallax_factor: f32, screen_h: u32) -> f32 {
        self.pitch * (1.0 - parallax_factor) * screen_h as f32 * PITCH_SENSITIVITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cam() -> Camera {
        Camera::new()
    }

    #[test]
    fn test_project_behind_camera() {
        let c = cam();
        // z_world=5, camera at z=0, z_rel=5 <= NEAR_PLANE(10) → None
        assert!(c.project(5.0, 200, 50).is_none());
    }

    #[test]
    fn test_project_at_near_plane() {
        let c = cam();
        // z_rel = 10.0 = NEAR_PLANE → depth_scale = 1.0
        let (sy, ds) = c.project(10.0, 200, 50).unwrap();
        assert!(
            (ds - 1.0).abs() < 0.001,
            "depth_scale at near_plane should be 1.0, got {ds}"
        );
        // screen_y = 50 + (200 - 50) * 1.0 = 200
        assert!(
            (sy - 200.0).abs() < 0.1,
            "screen_y at near_plane should be screen_h, got {sy}"
        );
    }

    #[test]
    fn test_project_beyond_draw_distance() {
        let c = cam();
        assert!(c.project(6000.0, 200, 50).is_none());
    }

    #[test]
    fn test_project_depth_scale_decreases_with_distance() {
        let c = cam();
        let (_, ds_near) = c.project(100.0, 200, 50).unwrap();
        let (_, ds_far) = c.project(1000.0, 200, 50).unwrap();
        assert!(
            ds_near > ds_far,
            "near ds={ds_near} should > far ds={ds_far}"
        );
    }

    #[test]
    fn test_project_screen_y_monotonic() {
        let c = cam();
        let (sy_near, _) = c.project(100.0, 200, 50).unwrap();
        let (sy_far, _) = c.project(1000.0, 200, 50).unwrap();
        assert!(
            sy_near > sy_far,
            "near sy={sy_near} should > far sy={sy_far}"
        );
    }

    #[test]
    fn test_project_screen_y_within_bounds() {
        let c = cam();
        let (sy, _) = c.project(500.0, 200, 50).unwrap();
        assert!(sy >= 50.0, "screen_y should be >= horizon_y");
        assert!(sy <= 200.0, "screen_y should be <= screen_h");
    }

    #[test]
    fn test_project_1_over_z_values() {
        let c = cam();
        // z_rel=100, depth_scale = 10/100 = 0.1
        let (_, ds) = c.project(100.0, 200, 50).unwrap();
        assert!((ds - 0.1).abs() < 0.001, "expected 0.1, got {ds}");
        // z_rel=50, depth_scale = 10/50 = 0.2
        let (_, ds) = c.project(50.0, 200, 50).unwrap();
        assert!((ds - 0.2).abs() < 0.001, "expected 0.2, got {ds}");
    }

    #[test]
    fn test_project_x_centered() {
        let c = cam();
        let x = c.project_x(0.0, 0.5, 400);
        assert!((x - 200.0).abs() < 0.1, "zero offset should center at {x}");
    }

    #[test]
    fn test_project_x_offset() {
        let c = cam();
        let x = c.project_x(100.0, 0.5, 400);
        // cx=200, offset=100*0.5*1.0=50 → 250
        assert!((x - 250.0).abs() < 0.1, "expected 250, got {x}");
    }

    #[test]
    fn test_vanishing_point() {
        let c = cam();
        let (vx, vy) = c.vanishing_point(400, 200);
        assert!((vx - 200.0).abs() < 0.1);
        assert!((vy - 50.0).abs() < 0.1); // 200 * 0.25 = 50
    }

    #[test]
    fn test_road_half_at_full_scale() {
        let c = cam();
        let rh = c.road_half(1.0);
        assert!((rh - ROAD_HALF_WORLD).abs() < 0.1);
    }

    #[test]
    fn test_road_half_scales_with_depth() {
        let c = cam();
        let near = c.road_half(0.8);
        let far = c.road_half(0.2);
        assert!(near > far);
    }

    #[test]
    fn test_horizon_y() {
        let c = cam();
        assert_eq!(c.horizon_y(200), 50); // 200 * 0.25 = 50
    }

    #[test]
    fn test_sync_velocity_demon_draw_distance() {
        let mut c = cam();
        c.sync(0.0, VelocityTier::VelocityDemon);
        assert!((c.draw_distance - 6000.0).abs() < 0.1);
    }

    #[test]
    fn test_sync_horizon_ratio_at_speed() {
        let mut c = cam();
        c.sync(300.0, VelocityTier::Cruise);
        // speed_t = 1.0, horizon = 0.25 - 0.06 = 0.19
        assert!((c.horizon_ratio - 0.19).abs() < 0.001);
    }

    #[test]
    fn test_sync_velocity_demon_horizon() {
        let mut c = cam();
        c.sync(0.0, VelocityTier::VelocityDemon);
        assert!((c.horizon_ratio - 0.27).abs() < 0.001);
    }

    #[test]
    fn test_pitch_offset_zero_pitch() {
        let c = cam();
        assert!((c.pitch_offset(0.0, 200)).abs() < 0.001);
        assert!((c.pitch_offset(0.5, 200)).abs() < 0.001);
        assert!((c.pitch_offset(1.0, 200)).abs() < 0.001);
    }

    #[test]
    fn test_pitch_offset_nonzero() {
        let mut c = cam();
        c.pitch = 0.1;
        // parallax=0.0 (sky): offset = 0.1 * 1.0 * 200 * 0.5 = 10.0
        let off = c.pitch_offset(0.0, 200);
        assert!((off - 10.0).abs() < 0.01, "expected 10.0, got {off}");
        // parallax=0.4 (islands): offset = 0.1 * 0.6 * 200 * 0.5 = 6.0
        let off = c.pitch_offset(0.4, 200);
        assert!((off - 6.0).abs() < 0.01, "expected 6.0, got {off}");
    }

    #[test]
    fn test_pitch_offset_full_parallax() {
        let mut c = cam();
        c.pitch = 0.5;
        // parallax=1.0 (ground): offset = 0.5 * 0.0 * 200 * 0.5 = 0.0
        assert!((c.pitch_offset(1.0, 200)).abs() < 0.001);
    }

    // --- Dynamic camera response tests ---

    #[test]
    fn test_fov_increases_with_speed() {
        let mut c = cam();
        // Run several frames at high speed
        for _ in 0..60 {
            c.update(0.016, 300.0, 0.0, 0.0, VelocityTier::Active, 0.0);
        }
        assert!(
            c.fov_scale > 1.1,
            "fov_scale should widen with high speed, got {}",
            c.fov_scale
        );
    }

    #[test]
    fn test_fov_spring_damped() {
        let mut c = cam();
        // Single small dt step should not reach target
        c.update(0.016, 300.0, 0.0, 0.0, VelocityTier::Active, 0.0);
        // Target is 1.25, should not reach it in one frame
        assert!(
            c.fov_scale < 1.25,
            "fov_scale should not jump to target in one frame, got {}",
            c.fov_scale
        );
        assert!(
            c.fov_scale > 1.0,
            "fov_scale should have started moving, got {}",
            c.fov_scale
        );
    }

    #[test]
    fn test_pitch_responds_to_slope() {
        let mut c = cam();
        // Uphill slope (positive slope)
        for _ in 0..60 {
            c.update(0.016, 100.0, 0.0, 0.5, VelocityTier::Active, 0.0);
        }
        // Target: -0.5 * 0.03 = -0.015 (look down on uphill)
        assert!(
            c.pitch < -0.01,
            "pitch should be negative on uphill slope, got {}",
            c.pitch
        );
    }

    #[test]
    fn test_pitch_clamped() {
        let mut c = cam();
        // Extreme slope
        for _ in 0..120 {
            c.update(0.016, 100.0, 0.0, 10.0, VelocityTier::Active, 0.0);
        }
        assert!(
            c.pitch.abs() <= PITCH_MAX + 0.001,
            "pitch should be clamped to ±PITCH_MAX, got {}",
            c.pitch
        );
    }

    #[test]
    fn test_yaw_responds_to_curve() {
        let mut c = cam();
        // Positive curve_offset → camera should lag opposite (negative yaw)
        for _ in 0..60 {
            c.update(0.016, 100.0, 80.0, 0.0, VelocityTier::Active, 0.0);
        }
        assert!(
            c.yaw_offset < -1.0,
            "yaw should shift negative on positive curve, got {}",
            c.yaw_offset
        );
    }

    #[test]
    fn test_yaw_clamped() {
        let mut c = cam();
        // Extreme curve
        for _ in 0..120 {
            c.update(0.016, 100.0, 500.0, 0.0, VelocityTier::Active, 0.0);
        }
        assert!(
            c.yaw_offset.abs() <= YAW_MAX + SHAKE_X_AMP + 0.1,
            "yaw should be clamped near ±YAW_MAX, got {}",
            c.yaw_offset
        );
    }

    #[test]
    fn test_shake_only_velocity_demon() {
        // Non-VelocityDemon: update with zero inputs, yaw and pitch should stay near zero
        let mut c = cam();
        for i in 0..60 {
            c.update(
                0.016,
                100.0,
                0.0,
                0.0,
                VelocityTier::Demon,
                i as f32 * 0.016,
            );
        }
        assert!(
            c.yaw_offset.abs() < 0.1,
            "non-VelocityDemon should have no shake, got yaw={}",
            c.yaw_offset
        );

        // VelocityDemon: yaw should have shake component
        let mut c2 = cam();
        let mut max_yaw = 0.0f32;
        for i in 0..120 {
            c2.update(
                0.016,
                100.0,
                0.0,
                0.0,
                VelocityTier::VelocityDemon,
                i as f32 * 0.016,
            );
            max_yaw = max_yaw.max(c2.yaw_offset.abs());
        }
        assert!(
            max_yaw > 0.5,
            "VelocityDemon should have visible shake, max_yaw={}",
            max_yaw
        );
    }

    #[test]
    fn test_burst_zoom_and_recovery() {
        let mut c = cam();
        c.trigger_burst();
        assert!(
            c.burst_fov_offset < 0.0,
            "burst should set negative fov offset"
        );

        // Run a few frames — burst should recover
        for _ in 0..120 {
            c.update(0.016, 100.0, 0.0, 0.0, VelocityTier::Active, 0.0);
        }
        assert!(
            c.burst_fov_offset.abs() < 0.01,
            "burst fov offset should decay near zero, got {}",
            c.burst_fov_offset
        );
    }

    #[test]
    fn test_burst_temporarily_reduces_fov() {
        let mut c = cam();
        // Settle at a base FOV
        for _ in 0..60 {
            c.update(0.016, 100.0, 0.0, 0.0, VelocityTier::Active, 0.0);
        }
        let settled_fov = c.fov_scale;

        // Trigger burst and update one frame
        c.trigger_burst();
        c.update(0.016, 100.0, 0.0, 0.0, VelocityTier::Active, 0.0);

        assert!(
            c.fov_scale < settled_fov,
            "burst should temporarily reduce fov: settled={}, burst={}",
            settled_fov,
            c.fov_scale
        );
    }

    #[test]
    fn test_all_springs_converge() {
        let mut c = cam();
        // Drive with constant inputs for many frames
        for i in 0..300 {
            c.update(
                0.016,
                200.0,
                50.0,
                0.3,
                VelocityTier::Active,
                i as f32 * 0.016,
            );
        }
        // FOV target: 1.0 + (200/300) * 0.25 ≈ 1.167
        let fov_target = 1.0 + (200.0_f32 / 300.0).clamp(0.0, 1.0) * FOV_SPEED_SCALE;
        assert!(
            (c.fov_scale - fov_target).abs() < 0.02,
            "fov should converge: expected ~{}, got {}",
            fov_target,
            c.fov_scale
        );
        // Pitch target: -0.3 * 0.03 = -0.009
        let pitch_target = (-0.3 * PITCH_SLOPE_FACTOR).clamp(-PITCH_MAX, PITCH_MAX);
        assert!(
            (c.pitch - pitch_target).abs() < 0.005,
            "pitch should converge: expected ~{}, got {}",
            pitch_target,
            c.pitch
        );
        // Yaw target: -50.0 * 0.15 = -7.5
        let yaw_target = (-50.0 * YAW_CURVE_FACTOR).clamp(-YAW_MAX, YAW_MAX);
        assert!(
            (c.yaw_offset - yaw_target).abs() < 0.5,
            "yaw should converge: expected ~{}, got {}",
            yaw_target,
            c.yaw_offset
        );
    }
}
