# T-002-02 Review: git-poller

## Summary of Changes

### File Modified: `src/git/poller.rs`

**Production change (lines 69–80):** Added early return for unborn HEAD in `poll_once()`. When `revwalk.push_head()` fails (repo has no commits), the function now returns an empty `PollResult` with zeroed fields instead of propagating the error. This satisfies the AC requirement: "Handles repos with no commits gracefully (empty PollResult)."

**Test module added (lines 143–397):** Added `#[cfg(test)] mod tests` with 8 unit tests and 2 helper functions.

No other files were created or modified. No dependencies added.

## Acceptance Criteria Status

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `spawn()` starts background thread | Pass | `poller.rs:44` — `thread::spawn()` |
| First poll immediate | Pass | `poller.rs:46` — runs before loop |
| Walks commits in window with Sort::TIME | Pass | `poller.rs:81` + `test_window_filtering` |
| PollResult includes required fields | Pass | `poller.rs:8-16` — all fields present |
| CommitSummary includes required fields | Pass | `poller.rs:19-27` + `test_single_commit` |
| Diff stats via git2::Diff::stats() | Pass | `poller.rs:104-105` + `test_diff_stats` |
| Thread exits on receiver drop | Pass | `poller.rs:53-54` — `send().is_err() → break` |
| No subprocess calls | Pass | Pure git2 API throughout |
| Handles empty repos gracefully | Pass | `poller.rs:69-80` + `test_empty_repo` |

All 9 acceptance criteria are satisfied.

## Test Coverage

### New Tests (8 total, all pass)

| Test | What it verifies |
|------|-----------------|
| `test_empty_repo` | Empty repo → empty PollResult (not Err) |
| `test_single_commit` | CommitSummary field mapping (sha, message, author, stats) |
| `test_diff_stats` | Diff stat accuracy with known content changes |
| `test_window_filtering` | Time-window boundary (30min excludes old, 180min includes all) |
| `test_commits_per_min` | Rate calculation: 3 commits / 10 min = 0.3 |
| `test_multiple_commits_ordered` | TIME sort order (newest first), author preservation |
| `test_root_commit_diff` | Root commit diffs against empty tree (all lines as additions) |
| `test_sha_short_format` | SHA is exactly 7 hex chars |

### Coverage Gaps

- **`GitPoller::spawn()` not directly tested.** The thread lifecycle, channel send/recv, and sleep loop are not tested. This is intentional — testing threading behavior requires timing-dependent assertions that would be flaky. The spawn path is implicitly tested by running the application.
- **Error paths in poll_once.** Tests don't cover `Repository::open()` failure (invalid path) or `revwalk()` creation failure. These paths simply propagate errors and are covered by git2's own tests.
- **`RollingStats` untested.** Trivial 4-field copy from PollResult. Not worth a dedicated test.

### Verification Results

```
cargo test git::poller  → 8/8 pass
cargo clippy            → 0 warnings
cargo build             → clean
```

4 pre-existing failures in `renderer::sprites::tests` are unrelated (from another ticket's uncommitted work on sprites.rs).

## Open Concerns

1. **Thread is detached.** `spawn()` does not return a `JoinHandle`. If the poller thread panics, it silently dies and the main loop simply receives no more `PollResult` values. For a screensaver, this is acceptable — the car continues driving at Flatline speed. For a more robust application, the JoinHandle should be stored and checked. Not in scope for this ticket.

2. **Initial send ignores channel error.** Line 47: `let _ = tx.send(result)`. If the receiver is dropped before the first poll completes, this silently discards the result. The thread then enters the loop and exits on the next `send().is_err()`. Functionally harmless — at most one wasted sleep cycle.

3. **`RollingStats` appears unused.** `git/stats.rs` defines `RollingStats::from_poll()` but no consumer calls it. WorldState reads PollResult fields directly. May be dead code or scaffolding for future smoothing logic. Not in scope for this ticket.

## No Known Regressions

The production change is a single early-return guard that only triggers for repos with no commits (unborn HEAD). All existing behavior for repos with commits is unchanged — the `push_head()` call succeeds and execution continues as before.
