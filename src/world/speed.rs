#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VelocityTier {
    Flatline = 0,
    Cruise = 1,
    Active = 2,
    Demon = 3,
    VelocityDemon = 4,
}

impl VelocityTier {
    pub fn from_commits_per_min(cpm: f32) -> Self {
        if cpm >= 4.0 {
            Self::VelocityDemon
        } else if cpm >= 1.5 {
            Self::Demon
        } else if cpm >= 0.5 {
            Self::Active
        } else if cpm > 0.0 {
            Self::Cruise
        } else {
            Self::Flatline
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Flatline => "FLATLINE",
            Self::Cruise => "CRUISE",
            Self::Active => "ACTIVE",
            Self::Demon => "DEMON",
            Self::VelocityDemon => "VELOCITY DEMON",
        }
    }
}

pub fn speed_target(commits_per_min: f32) -> f32 {
    10.0 + (commits_per_min * 70.0).min(290.0)
}

// ---------------------------------------------------------------------------
// 6-gear transmission model
// ---------------------------------------------------------------------------

pub const GEAR_COUNT: u8 = 6;
pub const RPM_IDLE: f32 = 1000.0;
pub const RPM_REDLINE: f32 = 8000.0;
pub const RPM_UPSHIFT: f32 = 7500.0;
pub const RPM_DOWNSHIFT: f32 = 2000.0;
pub const SHIFT_COOLDOWN: f32 = 0.15; // seconds of reduced power during shift

/// Gear ratios — higher ratio = more torque, less top speed.
/// Index 0 = gear 1, index 5 = gear 6.
pub const GEAR_RATIOS: [f32; 6] = [3.5, 2.5, 1.8, 1.3, 1.0, 0.8];

/// Final drive converts RPM×ratio into road speed units.
pub const FINAL_DRIVE: f32 = 0.012;

/// Torque curve: peaks mid-range, falls off at low and high RPM.
/// Returns 0.0–1.0 torque multiplier for given RPM.
pub fn torque_at_rpm(rpm: f32) -> f32 {
    // Normalized rpm in 0..1 range
    let t = ((rpm - RPM_IDLE) / (RPM_REDLINE - RPM_IDLE)).clamp(0.0, 1.0);
    // Bell curve peaking at ~0.6 (≈5200 RPM)
    let peak = 0.6;
    let spread = 0.35;
    let x = (t - peak) / spread;
    (1.0 - x * x).max(0.15)
}

/// Convert RPM + gear to road speed.
pub fn rpm_to_speed(rpm: f32, gear: u8) -> f32 {
    let ratio = GEAR_RATIOS[gear.clamp(0, 5) as usize];
    rpm * FINAL_DRIVE / ratio
}

/// Convert road speed + gear to RPM.
pub fn speed_to_rpm(speed: f32, gear: u8) -> f32 {
    let ratio = GEAR_RATIOS[gear.clamp(0, 5) as usize];
    (speed * ratio / FINAL_DRIVE).clamp(RPM_IDLE, RPM_REDLINE)
}

/// Determine which gear should be selected for a target speed.
/// Returns 0-based gear index.
pub fn gear_for_speed(speed: f32) -> u8 {
    // Find the lowest gear where the speed doesn't exceed redline
    for g in 0..GEAR_COUNT {
        let rpm = speed_to_rpm(speed, g);
        if rpm < RPM_UPSHIFT {
            return g;
        }
    }
    GEAR_COUNT - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- VelocityTier thresholds ---

    #[test]
    fn test_tier_flatline() {
        assert_eq!(
            VelocityTier::from_commits_per_min(0.0),
            VelocityTier::Flatline
        );
    }

    #[test]
    fn test_tier_cruise() {
        assert_eq!(
            VelocityTier::from_commits_per_min(0.1),
            VelocityTier::Cruise
        );
    }

    #[test]
    fn test_tier_active_at_threshold() {
        assert_eq!(
            VelocityTier::from_commits_per_min(0.5),
            VelocityTier::Active
        );
    }

    #[test]
    fn test_tier_demon_at_threshold() {
        assert_eq!(VelocityTier::from_commits_per_min(1.5), VelocityTier::Demon);
    }

    #[test]
    fn test_tier_velocity_demon_at_threshold() {
        assert_eq!(
            VelocityTier::from_commits_per_min(4.0),
            VelocityTier::VelocityDemon
        );
    }

    #[test]
    fn test_tier_boundary_below_active() {
        assert_eq!(
            VelocityTier::from_commits_per_min(0.49),
            VelocityTier::Cruise
        );
    }

    #[test]
    fn test_tier_boundary_below_demon() {
        assert_eq!(
            VelocityTier::from_commits_per_min(1.49),
            VelocityTier::Active
        );
    }

    #[test]
    fn test_tier_boundary_below_vdemon() {
        assert_eq!(
            VelocityTier::from_commits_per_min(3.99),
            VelocityTier::Demon
        );
    }

    #[test]
    fn test_tier_high_cpm() {
        assert_eq!(
            VelocityTier::from_commits_per_min(100.0),
            VelocityTier::VelocityDemon
        );
    }

    // --- speed_target ---

    #[test]
    fn test_speed_target_flatline() {
        let s = speed_target(0.0);
        assert!((s - 10.0).abs() < 0.001, "expected 10.0, got {s}");
    }

    #[test]
    fn test_speed_target_mid() {
        // 2.0 * 70.0 = 140.0 → 10.0 + 140.0 = 150.0
        let s = speed_target(2.0);
        assert!((s - 150.0).abs() < 0.001, "expected 150.0, got {s}");
    }

    #[test]
    fn test_speed_target_cap() {
        // 100.0 * 70.0 = 7000.0, capped at 290.0 → 10.0 + 290.0 = 300.0
        let s = speed_target(100.0);
        assert!((s - 300.0).abs() < 0.001, "expected 300.0, got {s}");
    }

    #[test]
    fn test_speed_target_at_vdemon_threshold() {
        // 4.0 * 70.0 = 280.0 → 10.0 + 280.0 = 290.0
        let s = speed_target(4.0);
        assert!((s - 290.0).abs() < 0.001, "expected 290.0, got {s}");
    }

    // --- name ---

    #[test]
    fn test_tier_names() {
        assert_eq!(VelocityTier::Flatline.name(), "FLATLINE");
        assert_eq!(VelocityTier::Cruise.name(), "CRUISE");
        assert_eq!(VelocityTier::Active.name(), "ACTIVE");
        assert_eq!(VelocityTier::Demon.name(), "DEMON");
        assert_eq!(VelocityTier::VelocityDemon.name(), "VELOCITY DEMON");
    }
}
