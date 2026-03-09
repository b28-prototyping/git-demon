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
    if revwalk.push_head().is_err() {
        // Unborn HEAD — repo has no commits yet
        return Ok(PollResult {
            commits: Vec::new(),
            commits_per_min: 0.0,
            lines_added: 0,
            lines_deleted: 0,
            files_changed: 0,
            window_minutes,
            polled_at: Utc::now(),
        });
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Create a test git repo with programmatic commits.
    /// Each entry is (message, author_name, file_content).
    /// Commits are created in order; timestamps are set to `now - 5 min + i * 10 sec`
    /// so all commits fall within a reasonable recent window.
    fn create_test_repo(dir: &Path, commits: &[(&str, &str, &str)]) -> git2::Repository {
        let repo = git2::Repository::init(dir).expect("init repo");

        // Configure a default committer for the repo
        let mut config = repo.config().expect("config");
        config.set_str("user.name", "Test").expect("set user.name");
        config.set_str("user.email", "test@test.com").expect("set user.email");

        let base_time = chrono::Utc::now().timestamp() - 300; // 5 minutes ago

        for (i, (message, author_name, content)) in commits.iter().enumerate() {
            let file_path = dir.join("test.txt");
            std::fs::write(&file_path, content).expect("write file");

            let mut index = repo.index().expect("index");
            index
                .add_path(Path::new("test.txt"))
                .expect("add to index");
            index.write().expect("write index");
            let tree_oid = index.write_tree().expect("write tree");
            let tree = repo.find_tree(tree_oid).expect("find tree");

            let sig_time = git2::Time::new(base_time + (i as i64) * 10, 0);
            let sig = git2::Signature::new(author_name, "test@test.com", &sig_time)
                .expect("signature");

            let parent = if i == 0 {
                None
            } else {
                Some(repo.head().unwrap().peel_to_commit().unwrap())
            };

            let parents: Vec<&git2::Commit> = parent.iter().collect();

            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
                .expect("commit");
        }

        repo
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("git-demon-test-{}-{}", name, std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).ok();
        }
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn test_empty_repo() {
        let dir = temp_dir("empty");
        let _repo = git2::Repository::init(&dir).expect("init");

        let result = poll_once(dir.to_str().unwrap(), 30).expect("poll_once should succeed");

        assert!(result.commits.is_empty());
        assert_eq!(result.commits_per_min, 0.0);
        assert_eq!(result.lines_added, 0);
        assert_eq!(result.lines_deleted, 0);
        assert_eq!(result.files_changed, 0);
        assert_eq!(result.window_minutes, 30);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_single_commit() {
        let dir = temp_dir("single");
        create_test_repo(&dir, &[("Initial commit", "Alice", "hello world\n")]);

        let result = poll_once(dir.to_str().unwrap(), 30).expect("poll");

        assert_eq!(result.commits.len(), 1);
        let c = &result.commits[0];
        assert_eq!(c.sha_short.len(), 7);
        assert_eq!(c.message, "Initial commit");
        assert_eq!(c.author, "Alice");
        assert_eq!(c.files_changed, 1);
        assert!(c.lines_added > 0); // "hello world\n" = 1 line added

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_diff_stats() {
        let dir = temp_dir("diff");
        create_test_repo(
            &dir,
            &[
                ("first", "Bob", "line1\nline2\nline3\n"),
                ("second", "Bob", "line1\nchanged\nline3\nnew_line\n"),
            ],
        );

        let result = poll_once(dir.to_str().unwrap(), 30).expect("poll");

        assert_eq!(result.commits.len(), 2);
        // Total stats should reflect both commits
        assert!(result.lines_added > 0);
        assert!(result.files_changed > 0);

        // Second commit: changed 1 line (delete + add) and added 1 line
        let second = &result.commits[0]; // newest first (TIME sort)
        assert_eq!(second.message, "second");
        assert_eq!(second.files_changed, 1);
        // line2→changed (1 del + 1 add), added new_line (1 add) = 2 added, 1 deleted
        assert_eq!(second.lines_added, 2);
        assert_eq!(second.lines_deleted, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_window_filtering() {
        let dir = temp_dir("window");
        let repo = git2::Repository::init(&dir).expect("init");

        let mut config = repo.config().expect("config");
        config.set_str("user.name", "Test").expect("set name");
        config.set_str("user.email", "t@t.com").expect("set email");

        // Create a commit far in the past (2 hours ago)
        let old_time = git2::Time::new(chrono::Utc::now().timestamp() - 7200, 0);
        let old_sig = git2::Signature::new("Test", "t@t.com", &old_time).unwrap();

        let file_path = dir.join("test.txt");
        std::fs::write(&file_path, "old content\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        repo.commit(Some("HEAD"), &old_sig, &old_sig, "old commit", &tree, &[])
            .unwrap();

        // Create a recent commit (1 minute ago)
        let recent_time = git2::Time::new(chrono::Utc::now().timestamp() - 60, 0);
        let recent_sig = git2::Signature::new("Test", "t@t.com", &recent_time).unwrap();

        std::fs::write(&file_path, "new content\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(
            Some("HEAD"),
            &recent_sig,
            &recent_sig,
            "recent commit",
            &tree,
            &[&parent],
        )
        .unwrap();

        // Poll with 30-minute window — should only get the recent commit
        let result = poll_once(dir.to_str().unwrap(), 30).expect("poll");
        assert_eq!(result.commits.len(), 1);
        assert_eq!(result.commits[0].message, "recent commit");

        // Poll with 180-minute window — should get both
        let result_wide = poll_once(dir.to_str().unwrap(), 180).expect("poll wide");
        assert_eq!(result_wide.commits.len(), 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_commits_per_min() {
        let dir = temp_dir("cpm");
        create_test_repo(
            &dir,
            &[
                ("c1", "A", "a\n"),
                ("c2", "A", "b\n"),
                ("c3", "A", "c\n"),
            ],
        );

        let result = poll_once(dir.to_str().unwrap(), 10).expect("poll");
        assert_eq!(result.commits.len(), 3);
        // 3 commits / 10 minutes = 0.3
        assert!((result.commits_per_min - 0.3).abs() < 0.001);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_multiple_commits_ordered() {
        let dir = temp_dir("ordered");
        create_test_repo(
            &dir,
            &[
                ("first", "A", "1\n"),
                ("second", "B", "2\n"),
                ("third", "C", "3\n"),
            ],
        );

        let result = poll_once(dir.to_str().unwrap(), 30).expect("poll");
        assert_eq!(result.commits.len(), 3);
        // TIME sort = newest first
        assert_eq!(result.commits[0].message, "third");
        assert_eq!(result.commits[1].message, "second");
        assert_eq!(result.commits[2].message, "first");

        // Authors should be preserved
        assert_eq!(result.commits[0].author, "C");
        assert_eq!(result.commits[1].author, "B");
        assert_eq!(result.commits[2].author, "A");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_root_commit_diff() {
        let dir = temp_dir("root");
        create_test_repo(&dir, &[("root", "R", "line1\nline2\nline3\n")]);

        let result = poll_once(dir.to_str().unwrap(), 30).expect("poll");
        assert_eq!(result.commits.len(), 1);
        let root = &result.commits[0];
        // Root commit diffs against empty tree — all lines are additions
        assert_eq!(root.lines_added, 3);
        assert_eq!(root.lines_deleted, 0);
        assert_eq!(root.files_changed, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_sha_short_format() {
        let dir = temp_dir("sha");
        create_test_repo(&dir, &[("test sha", "Dev", "content\n")]);

        let result = poll_once(dir.to_str().unwrap(), 30).expect("poll");
        let sha = &result.commits[0].sha_short;
        assert_eq!(sha.len(), 7);
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));

        std::fs::remove_dir_all(&dir).ok();
    }
}
