# T-002-02 Structure: git-poller

## Files Modified

### src/git/poller.rs

**Production code change:**
- `poll_once()` function: Add early return for unborn HEAD (repos with no commits)
  - After `revwalk.push_head()`, catch the error and return an empty `PollResult`
  - All other error paths (repo open, revwalk creation) remain propagated

**Test module addition:**
- Add `#[cfg(test)] mod tests` block at end of file
- Contains helper function `create_test_repo()` and 7-8 test functions

### No New Files

All changes are within the existing `poller.rs`. No new modules, no Cargo.toml changes.

## Interface Changes

None. `PollResult`, `CommitSummary`, `GitPoller::spawn()`, and `poll_once()` signatures remain identical. The only behavioral change is that `poll_once()` now returns `Ok(empty_result)` instead of `Err` for empty repos.

## Module Boundary

The `poll_once` function is currently `fn poll_once(...)` (private). Tests in the same module can call it directly. No visibility changes needed.

## Test Helper Design

```
create_test_repo(dir, commits) -> Repository

  commits: &[TestCommit]
  TestCommit { message, author, file_content, timestamp_offset_secs }

  - Calls Repository::init(dir)
  - For each commit:
    - Writes file_content to "test.txt" in workdir
    - Stages via index.add_path("test.txt")
    - Creates commit with specified author signature and timestamp
    - Timestamp = base_time + offset (base_time = now - 1 hour, giving room for window tests)
```

The helper is ~30 lines. Each test creates a `tempfile::TempDir` or `std::env::temp_dir()` subdirectory.

Since `tempfile` is not in dev-dependencies and we want to avoid adding deps, we'll use `std::env::temp_dir()` with a unique suffix per test (thread ID + test name hash or similar). Cleanup via `std::fs::remove_dir_all` in test body.

## Test Inventory

| Test | Purpose | Key assertion |
|------|---------|---------------|
| `test_empty_repo` | Unborn HEAD handling | PollResult.commits is empty, no error |
| `test_single_commit` | Basic field mapping | sha_short is 7 chars, message/author correct |
| `test_diff_stats` | Diff stat accuracy | lines_added/deleted/files_changed match |
| `test_window_filtering` | Time window boundary | Old commits excluded, recent included |
| `test_commits_per_min` | Rate calculation | commits.len() / window_minutes |
| `test_multiple_commits_ordered` | Ordering | Commits in TIME order (newest first) |
| `test_root_commit_diff` | No-parent diff | First commit diffs against empty tree correctly |

## Change Ordering

1. Fix `poll_once()` empty repo handling (1 line change + empty PollResult construction)
2. Add test helper `create_test_repo()`
3. Add tests one by one, verifying each passes

## Risk Assessment

- **Low risk**: The production change is a single early-return guard
- **No downstream impact**: Consumers already handle empty PollResults (WorldState with 0 commits_per_min stays at Flatline tier)
- **Test isolation**: Each test uses its own temp directory, no shared state
