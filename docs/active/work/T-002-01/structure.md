# Structure — T-002-01: repo-seed-computation

## Files Modified

### `src/git/seed.rs` (primary, ~170 lines after changes)

All changes are in this one file. No new files, no deletions.

#### Struct `RepoSeed` — no changes
Fields remain identical. The struct interface is stable.

#### `RepoSeed::compute()` — modified

**accent_hue derivation** (replace lines 55-57):
```
Before: identity = first_sha (HEAD)
After:  identity = origin_url if remote exists, else root_commit_sha, else repo_name
```

The `repo_name` extraction already queries the remote. Refactor to capture the origin
URL string separately for hue derivation.

**Revwalk changes** (modify lines 34-53):
- Change `push_head()` to be fallible — wrap in `if let Ok(())` or match
- Collect commit timestamps into a `Vec<i64>` for interval variance calculation
- Track `last_sha` (root commit) instead of `first_sha` (HEAD)

**terrain_roughness computation** (replace lines 68-73):
```
Before: log10(total_commits) / 5.0
After:  coefficient_of_variation(inter_commit_intervals) * 0.5
```

New helper: `compute_interval_variance(timestamps: &[i64]) -> f32`
- Takes sorted-descending timestamps
- Computes inter-commit intervals
- Returns coefficient of variation (stddev / mean)
- Returns 0.0 for fewer than 2 intervals (≤2 commits)

**Empty repo path** (new branch in compute):
When `push_head()` fails, skip revwalk. Derive accent_hue from origin URL or
repo directory name. All numeric fields default to minimum values.

#### Helper functions — modified

`hash_to_hue` and `hsl_to_rgb` — no functional changes. Remain private.

New private function:
```rust
fn interval_cv(timestamps: &[i64]) -> f32
```

#### Tests — new `#[cfg(test)] mod tests` block

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // hsl_to_rgb known values
    fn test_hsl_red()        // (0, 1.0, 0.5) → (255, 0, 0)
    fn test_hsl_green()      // (120, 1.0, 0.5) → (0, 255, 0)
    fn test_hsl_blue()       // (240, 1.0, 0.5) → (0, 0, 255)
    fn test_hsl_white()      // (0, 0.0, 1.0) → (255, 255, 255)
    fn test_hsl_black()      // (0, 0.0, 0.0) → (0, 0, 0)
    fn test_hsl_gray()       // (0, 0.0, 0.5) → (127, 127, 127)
    fn test_hsl_desaturated() // (60, 0.5, 0.5) → known value

    // hash_to_hue properties
    fn test_hash_deterministic()  // same input → same output
    fn test_hash_range()          // output in [0, 360)
    fn test_hash_different_inputs() // different strings → (likely) different hues

    // interval_cv
    fn test_cv_regular_intervals()  // equal spacing → cv ≈ 0
    fn test_cv_bursty()             // mixed spacing → cv > 0
    fn test_cv_single_commit()      // → 0.0
    fn test_cv_empty()              // → 0.0
}
```

## Module Boundaries

No changes to module boundaries. `RepoSeed` remains in `git::seed`, re-exported
via `git::mod.rs`. No public API changes — all modifications are internal to
`compute()`.

## Ordering

Changes are isolated to one file. No cross-file dependencies to sequence.
