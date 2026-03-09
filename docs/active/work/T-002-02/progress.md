# T-002-02 Progress: git-poller

## Completed

### Step 1: Fix empty repo handling
- Modified `poll_once()` in `src/git/poller.rs` to catch `push_head()` errors on unborn HEAD
- Returns empty `PollResult` instead of propagating error
- Build verified clean

### Step 2-3: Add test helper and tests
- Added `create_test_repo()` helper function for creating git repos with programmatic commits
- Added `temp_dir()` helper for unique per-test temp directories
- Added 8 unit tests:
  - `test_empty_repo` — verifies empty PollResult for repos with no commits
  - `test_single_commit` — verifies all CommitSummary fields
  - `test_diff_stats` — verifies diff stat accuracy with known content changes
  - `test_window_filtering` — verifies time-based window boundary filtering
  - `test_commits_per_min` — verifies rate calculation (3 commits / 10 min = 0.3)
  - `test_multiple_commits_ordered` — verifies TIME sort ordering (newest first)
  - `test_root_commit_diff` — verifies root commit diffs against empty tree
  - `test_sha_short_format` — verifies 7-char hex format

### Step 4: Final verification
- All 8 new poller tests pass
- All 86 pre-existing tests pass (94 total from poller module perspective)
- 4 sprite tests fail — pre-existing from uncommitted changes to `sprites.rs` by another ticket
- `cargo clippy` clean — no warnings
- `cargo build` clean

## Deviations from Plan

None. Implementation followed the plan exactly.
