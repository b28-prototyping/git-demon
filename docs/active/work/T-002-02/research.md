# T-002-02 Research: git-poller

## Scope

The git poller is a background `std::thread` that periodically walks recent commits via `git2`, computes diff stats, and sends `PollResult` values over a crossbeam channel to the main render loop.

## Existing Implementation

The poller is already implemented in `src/git/poller.rs` (131 lines). This ticket is being researched after initial implementation to verify correctness and identify gaps.

### Core Types

**`PollResult`** (poller.rs:8–16): Aggregate poll output.
- `commits: Vec<CommitSummary>` — individual commit data
- `commits_per_min: f32` — rate metric driving speed/tier
- `lines_added/deleted: u32`, `files_changed: u32` — aggregate diff stats
- `window_minutes: u32` — the lookback window used
- `polled_at: DateTime<Utc>` — timestamp of this poll

**`CommitSummary`** (poller.rs:19–27): Per-commit data.
- `sha_short: String` — first 7 chars of OID
- `message: String` — first line via `commit.summary()`
- `author: String` — from `commit.author().name()`
- `lines_added/deleted: u32`, `files_changed: u32` — per-commit diff stats
- `timestamp: DateTime<Utc>` — commit time

**`GitPoller`** (poller.rs:29–62): Unit struct with `spawn()` class method.
- Validates repo path by opening it once, then spawns thread
- Thread owns cloned path string, interval `Duration`, and `Sender<PollResult>`
- First poll runs immediately; loop sleeps then polls
- Exit: `tx.send()` returns `Err` when receiver drops → breaks loop

### Poll Algorithm (poll_once, poller.rs:64–130)

1. `Repository::open(repo_path)` — re-opens repo each poll (no state held between polls)
2. `revwalk` with `push_head()` + `Sort::TIME`
3. Cutoff: `Utc::now() - Duration::minutes(window_minutes)`
4. For each commit until cutoff:
   - Extract metadata (SHA, summary, author, timestamp)
   - Tree-to-tree diff: `diff_tree_to_tree(parent_tree, current_tree, None)`
   - `diff.stats()` for insertions/deletions/files_changed
   - Accumulate totals
5. `commits_per_min = commits.len() / window_minutes`
6. Return `PollResult`

### Graceful Empty Repo Handling

- `revwalk.push_head()` will return `Err` if repo has no HEAD → `poll_once` returns `Err`
- In `spawn()`, initial poll failure is silently ignored (line 47: `if let Ok(result)`)
- Loop poll failures are also silently ignored (line 52: `if let Ok(result)`)
- This means empty repos produce no `PollResult`, which is correct — WorldState defaults to `Flatline` tier
- However, the AC says "empty PollResult" — current code returns `Err`, not an empty `PollResult`

## Integration Points

### Channel Setup (main.rs:74)
Unbounded channel. Poller never blocks. At 30-second intervals vs 60fps drain, backpressure is impossible.

### Consumption (main.rs:116–118)
Non-blocking `try_recv()` drain each frame. Multiple results could arrive if main loop was delayed.

### WorldState (world/mod.rs:104–130)
`ingest_poll()` copies metrics, converts commits to roadside objects, detects tier transitions, randomizes curve on activity bursts.

### RollingStats (git/stats.rs)
Thin extraction wrapper (20 lines). Currently unused by any consumer — WorldState reads PollResult directly.

## Edge Cases

1. **Root commit diff**: `parent(0)` returns `Err` → `parent_tree = None` → diffs against empty tree. Correct behavior.
2. **Merge commits**: Only diffs against `parent(0)` (first parent). Standard `--first-parent` behavior.
3. **Detached thread**: No `JoinHandle` returned. Acceptable for a screensaver.
4. **Timestamp ambiguity**: `timestamp_opt().single().unwrap_or_else(Utc::now)` — harmless fallback.
5. **No deduplication**: Same commits appear in every poll within the window. By design — each poll is a snapshot.

## Gap Analysis

### Missing: Explicit empty repo handling
`poll_once()` calls `revwalk.push_head()?` which returns `Err` for unborn HEAD. The error propagates and is silently discarded. The AC requests "empty PollResult" — should catch the unborn HEAD case and return an empty result instead.

### Missing: Tests
Zero tests for `poller.rs`. Testing requires temporary git repos with programmatic commits via `git2`. The `tempfile` crate is not in dev-dependencies but `git2::Repository::init()` can create repos in temp paths.

### Minor: Initial send ignores channel error
Line 47: `let _ = tx.send(result)` — if receiver is already dropped before first poll completes, thread continues to the loop where it properly checks send result. Functionally harmless race.

## File Inventory

| File | Lines | Role |
|------|-------|------|
| `src/git/poller.rs` | 131 | Poller thread + data types |
| `src/git/mod.rs` | 7 | Module declarations, re-exports |
| `src/git/stats.rs` | 20 | RollingStats wrapper |
| `src/git/seed.rs` | ~280 | Repo identity seeding (separate ticket) |
| `src/main.rs` | 166 | Channel setup, poll drain |
| `src/world/mod.rs` | 147 | Poll ingestion, world state |
