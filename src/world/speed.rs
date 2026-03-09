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
    // Higher baseline: 1.5 even at flatline, scales up with activity
    1.5 + (commits_per_min * 2.8).min(10.5)
}
