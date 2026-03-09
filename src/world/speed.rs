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
    0.4 + (commits_per_min * 2.8).min(11.6)
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
        assert!((s - 0.4).abs() < 0.001, "expected 0.4, got {s}");
    }

    #[test]
    fn test_speed_target_mid() {
        // 2.0 * 2.8 = 5.6 → 0.4 + 5.6 = 6.0
        let s = speed_target(2.0);
        assert!((s - 6.0).abs() < 0.001, "expected 6.0, got {s}");
    }

    #[test]
    fn test_speed_target_cap() {
        // 100.0 * 2.8 = 280.0, capped at 11.6 → 0.4 + 11.6 = 12.0
        let s = speed_target(100.0);
        assert!((s - 12.0).abs() < 0.001, "expected 12.0, got {s}");
    }

    #[test]
    fn test_speed_target_at_vdemon_threshold() {
        // 4.0 * 2.8 = 11.2 → 0.4 + 11.2 = 11.6
        let s = speed_target(4.0);
        assert!((s - 11.6).abs() < 0.001, "expected 11.6, got {s}");
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
