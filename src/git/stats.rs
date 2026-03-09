use super::PollResult;

pub struct RollingStats {
    pub commits_per_min: f32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub files_changed: u32,
}

impl RollingStats {
    pub fn from_poll(result: &PollResult) -> Self {
        Self {
            commits_per_min: result.commits_per_min,
            lines_added: result.lines_added,
            lines_deleted: result.lines_deleted,
            files_changed: result.files_changed,
        }
    }
}
