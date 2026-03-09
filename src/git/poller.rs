use std::thread;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use crossbeam_channel::Sender;

#[derive(Debug, Clone)]
pub struct PollResult {
    pub commits: Vec<CommitSummary>,
    pub commits_per_min: f32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub files_changed: u32,
    pub window_minutes: u32,
    pub polled_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CommitSummary {
    pub sha_short: String,
    pub message: String,
    pub author: String,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub files_changed: u32,
    pub timestamp: DateTime<Utc>,
}

pub struct GitPoller;

impl GitPoller {
    pub fn spawn(
        repo_path: &str,
        window_minutes: u32,
        interval_secs: u32,
        tx: Sender<PollResult>,
    ) -> Result<(), git2::Error> {
        // Verify the repo can be opened
        let _ = git2::Repository::open(repo_path)?;

        let path = repo_path.to_string();
        let interval = Duration::from_secs(interval_secs as u64);

        thread::spawn(move || {
            // Initial poll immediately
            if let Ok(result) = poll_once(&path, window_minutes) {
                let _ = tx.send(result);
            }

            loop {
                thread::sleep(interval);
                if let Ok(result) = poll_once(&path, window_minutes) {
                    if tx.send(result).is_err() {
                        break; // receiver dropped
                    }
                }
            }
        });

        Ok(())
    }
}

fn poll_once(repo_path: &str, window_minutes: u32) -> Result<PollResult, git2::Error> {
    let repo = git2::Repository::open(repo_path)?;
    let cutoff = Utc::now() - chrono::Duration::minutes(window_minutes as i64);

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    let mut total_added: u32 = 0;
    let mut total_deleted: u32 = 0;
    let mut total_files: u32 = 0;

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let time = commit.time();
        let commit_dt = Utc
            .timestamp_opt(time.seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        if commit_dt < cutoff {
            break;
        }

        // Diff stats
        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
        let stats = diff.stats()?;

        let lines_added = stats.insertions() as u32;
        let lines_deleted = stats.deletions() as u32;
        let files_changed = stats.files_changed() as u32;

        total_added += lines_added;
        total_deleted += lines_deleted;
        total_files += files_changed;

        commits.push(CommitSummary {
            sha_short: oid.to_string()[..7].to_string(),
            message: commit.summary().unwrap_or("").to_string(),
            author: commit.author().name().unwrap_or("unknown").to_string(),
            lines_added,
            lines_deleted,
            files_changed,
            timestamp: commit_dt,
        });
    }

    let commits_per_min = if window_minutes > 0 {
        commits.len() as f32 / window_minutes as f32
    } else {
        0.0
    };

    Ok(PollResult {
        commits,
        commits_per_min,
        lines_added: total_added,
        lines_deleted: total_deleted,
        files_changed: total_files,
        window_minutes,
        polled_at: Utc::now(),
    })
}
