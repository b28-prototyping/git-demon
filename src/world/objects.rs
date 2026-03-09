use image::Rgba;
use std::collections::VecDeque;

use super::speed::VelocityTier;
use crate::git::seed::RepoSeed;
use crate::git::PollResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lane {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone)]
pub enum RoadsideObject {
    CommitBillboard {
        message: String,
        author: String,
        author_color: Rgba<u8>,
    },
    AdditionTower {
        lines: u32,
        color: Rgba<u8>,
    },
    DeletionShard {
        lines: u32,
    },
    FilePosts {
        count: u32,
    },
    VelocitySign {
        commits_per_min: f32,
    },
    TierGate {
        tier: VelocityTier,
    },
    IdleAuthorTree {
        author: String,
        idle_minutes: u32,
    },
    SectorPylon {
        sector: u32,
    },
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}

pub fn ingest_poll_to_queue(
    result: &PollResult,
    seed: &RepoSeed,
    queue: &mut VecDeque<RoadsideObject>,
) {
    for commit in &result.commits {
        let author_color = seed
            .author_colors
            .get(&commit.author)
            .copied()
            .unwrap_or(Rgba([200, 200, 200, 255]));

        queue.push_back(RoadsideObject::CommitBillboard {
            message: truncate(&commit.message, 28),
            author: commit.author.clone(),
            author_color,
        });

        if commit.lines_added > 50 {
            queue.push_back(RoadsideObject::AdditionTower {
                lines: commit.lines_added,
                color: author_color,
            });
        }

        if commit.lines_deleted > 50 {
            queue.push_back(RoadsideObject::DeletionShard {
                lines: commit.lines_deleted,
            });
        }
    }

    queue.push_back(RoadsideObject::VelocitySign {
        commits_per_min: result.commits_per_min,
    });
}
