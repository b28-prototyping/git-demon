# T-002-02 Design: git-poller

## Problem Statement

The git poller implementation is functionally complete but has two gaps:
1. Empty repos (no commits / unborn HEAD) produce errors instead of empty PollResults
2. Zero test coverage for the poller module

## Approach 1: Fix Empty Repo + Add Unit Tests with git2 Fixtures

**Description:** Fix the `poll_once` function to handle unborn HEAD gracefully. Add comprehensive unit tests using in-memory git repositories created via `git2::Repository::init()` with programmatic commits.

**Pros:**
- Tests exercise the real git2 code paths — high confidence
- No new dependencies needed (git2 already available, tempdir via std::env::temp_dir)
- Fixture repos are fast to create (< 1ms per test)
- Tests verify the actual integration with libgit2

**Cons:**
- Tests create filesystem artifacts (temp dirs) — need cleanup
- git2 signature/commit creation is verbose
- Tests are technically integration tests (touch filesystem), not pure unit tests

**Assessment:** Best approach. The verbosity is a one-time cost in a helper function. Temp dir cleanup is handled by test scoping or explicit cleanup.

## Approach 2: Mock-based Testing

**Description:** Abstract git2 behind a trait, inject mock implementations in tests.

**Pros:**
- Pure unit tests, no filesystem
- Can test error paths easily

**Cons:**
- Requires significant refactoring (trait extraction, generics or dyn dispatch)
- Over-engineers a 131-line module
- Mock doesn't validate actual git2 behavior
- Adds complexity for no production benefit

**Assessment:** Rejected. The module is simple enough that real git2 fixtures are more valuable than mocks.

## Approach 3: Test Only via Integration Tests

**Description:** Add integration tests in `tests/` directory that test the full spawn→recv cycle.

**Pros:**
- Tests the threading and channel behavior end-to-end

**Cons:**
- Slow (must wait for thread scheduling, sleep intervals)
- Flaky timing-dependent assertions
- Hard to test edge cases like empty repos

**Assessment:** Rejected as primary strategy. Unit tests of `poll_once` are faster and more reliable. However, the spawn→recv path is already implicitly tested by running the binary.

## Decision: Approach 1

### Empty Repo Fix

Change `poll_once` to catch the `push_head()` error for unborn HEAD and return an empty PollResult instead of propagating the error. This matches the AC: "handles repos with no commits gracefully (empty PollResult)".

Specifically:
```rust
// Before (fails on empty repo):
revwalk.push_head()?;

// After (returns empty result):
if revwalk.push_head().is_err() {
    return Ok(PollResult { commits: vec![], commits_per_min: 0.0, ... });
}
```

This is the minimal fix. The repo can still be opened (it exists), but HEAD doesn't point to any commit yet.

### Test Strategy

Add `#[cfg(test)] mod tests` to `poller.rs` with a helper function that creates a temporary git repo with N commits at specified timestamps. Tests cover:

1. **Empty repo** — init repo, no commits → empty PollResult
2. **Single commit** — one commit → 1 CommitSummary with correct fields
3. **Window filtering** — commits inside and outside window → only in-window commits returned
4. **Diff stats** — commit with known file changes → correct lines_added/deleted/files_changed
5. **Commits per min calculation** — known commit count, known window → correct ratio
6. **Multiple commits** — ordering, sha_short format, message truncation
7. **Root commit diff** — first commit (no parent) → diffs against empty tree

### Helper Function Design

```rust
fn create_test_repo(dir: &std::path::Path, commits: &[(&str, &str, &str)]) -> git2::Repository
```

Takes a directory path and a slice of `(message, author, file_content)` tuples. Creates a repo, adds commits with the given metadata. Returns the repo handle. Each commit modifies a single file (`test.txt`) with the given content, making diff stats predictable.

For timestamp control: use `git2::Time::new(seconds, offset)` to set commit times explicitly, enabling window-boundary tests.

### What We Won't Change

- `RollingStats` — unused but harmless. Not in scope for this ticket.
- Re-opening repo each poll — correct design for observing active repos.
- Thread lifecycle — already works correctly.
- `CommitSummary` fields — already match AC exactly.
