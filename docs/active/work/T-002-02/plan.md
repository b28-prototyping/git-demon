# T-002-02 Plan: git-poller

## Step 1: Fix empty repo handling in poll_once

**Change:** In `poll_once()`, replace the `revwalk.push_head()?` with a conditional that returns an empty `PollResult` when HEAD is unborn.

**Verification:** `cargo build` succeeds. Existing tests pass (`cargo test`).

**Commit:** "T-002-02: handle empty repos gracefully in poll_once"

## Step 2: Add test helper and empty repo test

**Change:** Add `#[cfg(test)] mod tests` to `poller.rs` with:
- Helper function `create_test_repo()` that creates a git repo at a given path with programmatic commits
- `test_empty_repo` — creates repo with no commits, calls `poll_once`, asserts empty PollResult

**Verification:** `cargo test git::poller` — new test passes.

**Commit:** "T-002-02: add poller test infrastructure and empty repo test"

## Step 3: Add core functionality tests

**Change:** Add remaining tests:
- `test_single_commit` — one commit, verify all CommitSummary fields
- `test_diff_stats` — commit with known content changes, verify stats
- `test_window_filtering` — commits at different timestamps, verify window boundary
- `test_commits_per_min` — verify rate calculation
- `test_multiple_commits_ordered` — verify ordering
- `test_root_commit_diff` — verify first commit diffs against empty tree

**Verification:** `cargo test git::poller` — all tests pass. `cargo test` — full suite (86 + new) passes.

**Commit:** "T-002-02: add comprehensive poller unit tests"

## Step 4: Final verification

**Verification:**
- `cargo test` — all tests pass
- `cargo clippy` — no warnings
- `cargo build` — clean build

No commit needed for this step.

## Testing Strategy

All tests are unit tests within `poller.rs` using `#[cfg(test)]`. They create real git repos via `git2::Repository::init()` in temp directories. Each test:
1. Creates a unique temp directory
2. Initializes a git repo with specific commits
3. Calls `poll_once()` directly
4. Asserts on the returned `PollResult`
5. Cleans up the temp directory

No integration tests for `spawn()` — the threading/channel behavior is simple and already validated by the running application. Testing `poll_once()` directly covers all the git logic.
