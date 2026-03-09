# Review — T-002-01: repo-seed-computation

## Summary of Changes

### File modified: `src/git/seed.rs`

**Before**: 127 lines, no tests
**After**: 281 lines including 18 unit tests

### Functional changes

1. **accent_hue derivation** — Changed identity source from HEAD SHA to: origin URL
   (first priority) → root commit SHA (fallback) → repo name (empty repo fallback).
   This means repos with a remote will now get a different `accent_hue` than before
   (visual appearance change on upgrade for repos with remotes).

2. **terrain_roughness** — Changed from `log10(total_commits) / 5.0` to coefficient
   of variation of inter-commit intervals scaled by 0.5. Repos with regular commit
   cadence get smoother terrain; bursty repos get rougher terrain. Values remain
   clamped to `[0.1, 1.0]`.

3. **Empty repo handling** — `push_head()` failure no longer propagates as an error.
   Empty repos produce a valid `RepoSeed` with sensible defaults.

4. **New helper**: `interval_cv(timestamps: &[i64]) -> f32` — computes coefficient
   of variation for inter-commit time intervals.

## Acceptance Criteria Coverage

| Criterion | Status |
|-----------|--------|
| `RepoSeed::compute()` opens repo and walks full history | ✅ |
| `accent_hue` from remote origin URL (fallback: first commit SHA) | ✅ |
| Same repo → same `accent_hue` | ✅ (deterministic hash) |
| `terrain_roughness` from commit frequency variance, clamped [0.1, 1.0] | ✅ |
| `author_colors` maps author name → stable `Rgba<u8>` via name hash | ✅ (unchanged) |
| `repo_name` from remote URL basename (strip `.git`) or dir name | ✅ (unchanged) |
| `total_commits` is full count | ✅ (unchanged) |
| Handles edge cases: empty repo, no remote, detached HEAD | ✅ |
| HSL→RGB unit tested | ✅ (9 tests) |

## Test Coverage

**18 new unit tests** added to `seed.rs`:
- `hsl_to_rgb`: 9 tests covering all 6 hue sectors (R, Y, G, C, B, M) plus
  white, black, and mid-gray edge cases
- `hash_to_hue`: 3 tests — determinism, range [0,360), different inputs diverge
- `interval_cv`: 6 tests — empty, single commit, two commits, regular intervals,
  bursty intervals, same-timestamp

**Not tested** (would require integration test with real/temp git repos):
- `RepoSeed::compute()` end-to-end with actual git repos
- Remote URL extraction path
- Root commit SHA capture during revwalk

These would require creating temporary git repos in tests with `git2::Repository::init`
and populating them with commits. Reasonable follow-up work but not in the AC.

## Open Concerns

1. **Visual change on upgrade**: Repos with remotes will now get a different `accent_hue`
   (hashed from URL instead of HEAD SHA). This is intentional per the AC — same repo
   identity → same appearance, even across clones — but existing users will see a
   color shift.

2. **terrain_roughness behavioral change**: The new variance-based formula may produce
   different roughness values than the old log-count formula. Repos with very regular
   commit patterns will trend toward 0.1 (minimum), while bursty repos will be rougher.

3. **Duplicate HSL→RGB**: Three copies still exist across seed.rs, sky.rs, and road.rs.
   Not in scope for this ticket but worth a deduplication pass later.

4. **timestamp precision**: `commit.time().seconds()` gives Unix epoch seconds. This
   is sufficient for interval variance but coarser than sub-second if a repo has
   automated commits at high frequency.

## Build Verification

- `cargo build` — clean compile
- `cargo test` — 86 total tests pass (18 in seed module)
- `cargo clippy` — no warnings
