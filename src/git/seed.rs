use std::collections::HashMap;

use image::Rgba;

pub struct RepoSeed {
    pub accent_hue: f32,
    pub saturation: f32,
    pub terrain_roughness: f32,
    pub speed_base: f32,
    pub author_colors: HashMap<String, Rgba<u8>>,
    pub total_commits: u64,
    pub repo_name: String,
}

impl RepoSeed {
    pub fn compute(repo_path: &str) -> Result<Self, git2::Error> {
        let repo = git2::Repository::open(repo_path)?;

        // Capture origin URL for both repo_name and accent_hue identity
        let origin_url = repo
            .find_remote("origin")
            .ok()
            .and_then(|r| r.url().map(|u| u.to_string()));

        // Derive repo name from remote URL basename or directory name
        let repo_name = origin_url
            .as_deref()
            .map(|u| {
                u.rsplit('/')
                    .next()
                    .unwrap_or(u)
                    .trim_end_matches(".git")
                    .to_string()
            })
            .unwrap_or_else(|| {
                std::path::Path::new(repo_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".into())
            });

        // Walk all commits for stats — handle empty repo / unborn HEAD
        let mut revwalk = repo.revwalk()?;
        let has_head = revwalk.push_head().is_ok();

        let mut total_commits: u64 = 0;
        let mut authors: HashMap<String, u64> = HashMap::new();
        let mut root_sha: Option<String> = None;
        let mut timestamps: Vec<i64> = Vec::new();

        if has_head {
            revwalk.set_sorting(git2::Sort::TIME)?;

            for oid in revwalk {
                let oid = oid?;
                let commit = repo.find_commit(oid)?;
                let author = commit.author().name().unwrap_or("unknown").to_string();
                *authors.entry(author).or_insert(0) += 1;
                timestamps.push(commit.time().seconds());
                // Track last OID — the root (earliest) commit
                root_sha = Some(oid.to_string());
                total_commits += 1;
            }
        }

        // Accent hue: origin URL → root commit SHA → repo name
        let identity = origin_url
            .as_deref()
            .map(|s| s.to_string())
            .or(root_sha)
            .unwrap_or_else(|| repo_name.clone());
        let accent_hue = hash_to_hue(&identity);

        // Author colors
        let author_colors: HashMap<String, Rgba<u8>> = authors
            .keys()
            .map(|name| {
                let hue = hash_to_hue(name);
                let rgb = hsl_to_rgb(hue, 0.8, 0.6);
                (name.clone(), Rgba([rgb.0, rgb.1, rgb.2, 255]))
            })
            .collect();

        // Terrain roughness from commit frequency variance
        let cv = interval_cv(&timestamps);
        let terrain_roughness = (cv * 0.5).clamp(0.1, 1.0);

        let speed_base = if total_commits > 0 {
            (total_commits as f32 / 1000.0).min(1.0)
        } else {
            0.0
        };

        Ok(Self {
            accent_hue,
            saturation: 0.8,
            terrain_roughness,
            speed_base,
            author_colors,
            total_commits,
            repo_name,
        })
    }
}

fn hash_to_hue(s: &str) -> f32 {
    let mut hash: u32 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    (hash % 360) as f32
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h2 = h / 60.0;
    let x = c * (1.0 - (h2 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h2 as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

/// Coefficient of variation of inter-commit intervals.
/// Timestamps must be sorted descending (newest first, as from TIME-sorted revwalk).
/// Returns 0.0 for fewer than 2 intervals (≤2 commits).
fn interval_cv(timestamps: &[i64]) -> f32 {
    if timestamps.len() < 3 {
        return 0.0;
    }

    let intervals: Vec<f64> = timestamps
        .windows(2)
        .map(|w| (w[0] - w[1]).abs() as f64)
        .collect();

    let n = intervals.len() as f64;
    let mean = intervals.iter().sum::<f64>() / n;

    if mean < 1.0 {
        return 0.0; // All commits at same timestamp
    }

    let variance = intervals.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();

    (stddev / mean) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- hsl_to_rgb ---

    #[test]
    fn test_hsl_red() {
        assert_eq!(hsl_to_rgb(0.0, 1.0, 0.5), (255, 0, 0));
    }

    #[test]
    fn test_hsl_green() {
        assert_eq!(hsl_to_rgb(120.0, 1.0, 0.5), (0, 255, 0));
    }

    #[test]
    fn test_hsl_blue() {
        assert_eq!(hsl_to_rgb(240.0, 1.0, 0.5), (0, 0, 255));
    }

    #[test]
    fn test_hsl_white() {
        assert_eq!(hsl_to_rgb(0.0, 0.0, 1.0), (255, 255, 255));
    }

    #[test]
    fn test_hsl_black() {
        assert_eq!(hsl_to_rgb(0.0, 0.0, 0.0), (0, 0, 0));
    }

    #[test]
    fn test_hsl_mid_gray() {
        let (r, g, b) = hsl_to_rgb(0.0, 0.0, 0.5);
        // 0.5 * 255 = 127.5, truncated to 127
        assert_eq!((r, g, b), (127, 127, 127));
    }

    #[test]
    fn test_hsl_yellow() {
        // HSL(60, 1.0, 0.5) → RGB(255, 255, 0)
        assert_eq!(hsl_to_rgb(60.0, 1.0, 0.5), (255, 255, 0));
    }

    #[test]
    fn test_hsl_cyan() {
        // HSL(180, 1.0, 0.5) → RGB(0, 255, 255)
        assert_eq!(hsl_to_rgb(180.0, 1.0, 0.5), (0, 255, 255));
    }

    #[test]
    fn test_hsl_magenta() {
        // HSL(300, 1.0, 0.5) → RGB(255, 0, 255)
        assert_eq!(hsl_to_rgb(300.0, 1.0, 0.5), (255, 0, 255));
    }

    // --- hash_to_hue ---

    #[test]
    fn test_hash_deterministic() {
        let a = hash_to_hue("hello");
        let b = hash_to_hue("hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_hash_range() {
        for s in &["", "a", "hello world", "git@github.com:user/repo.git", "🦀"] {
            let hue = hash_to_hue(s);
            assert!(
                hue >= 0.0 && hue < 360.0,
                "hue {hue} out of range for {s:?}"
            );
        }
    }

    #[test]
    fn test_hash_different_inputs() {
        let a = hash_to_hue("repo-alpha");
        let b = hash_to_hue("repo-beta");
        assert_ne!(a, b);
    }

    // --- interval_cv ---

    #[test]
    fn test_cv_empty() {
        assert_eq!(interval_cv(&[]), 0.0);
    }

    #[test]
    fn test_cv_single_commit() {
        assert_eq!(interval_cv(&[1000]), 0.0);
    }

    #[test]
    fn test_cv_two_commits() {
        // Only 1 interval, need at least 2 for meaningful variance
        assert_eq!(interval_cv(&[2000, 1000]), 0.0);
    }

    #[test]
    fn test_cv_regular_intervals() {
        // Perfectly regular: intervals all 100s → stddev=0, cv=0
        let ts: Vec<i64> = (0..10).rev().map(|i| i * 100).collect();
        let cv = interval_cv(&ts);
        assert!(cv.abs() < 0.001, "expected ~0, got {cv}");
    }

    #[test]
    fn test_cv_bursty() {
        // Mix of 10s and 1000s intervals → high CV
        let ts = vec![5000, 4990, 4980, 3980, 3970, 3960, 2960, 2950, 2940, 1940];
        let cv = interval_cv(&ts);
        assert!(cv > 0.5, "expected bursty CV > 0.5, got {cv}");
    }

    #[test]
    fn test_cv_same_timestamp() {
        // All commits at same time → mean ≈ 0 → returns 0.0
        let ts = vec![1000, 1000, 1000, 1000];
        assert_eq!(interval_cv(&ts), 0.0);
    }
}
