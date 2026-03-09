# Progress — T-002-01: repo-seed-computation

## Completed

### Step 1: Add `interval_cv` helper ✅
Added `interval_cv(timestamps: &[i64]) -> f32` function that computes coefficient
of variation of inter-commit intervals. Handles edge cases: <3 timestamps returns
0.0, zero-mean (same timestamp) returns 0.0.

### Step 2: Fix accent_hue derivation ✅
Changed identity priority to: origin URL → root commit SHA → repo name.
- Extracted `origin_url` before revwalk as `Option<String>`
- Changed from tracking HEAD SHA (`first_sha`) to root commit SHA (`root_sha`)
- Root SHA is the last OID in time-sorted revwalk (earliest commit)

### Step 3: Handle empty repo edge case ✅
- `push_head()` result checked with `.is_ok()`
- On failure (empty repo/unborn HEAD), skips revwalk entirely
- Defaults: total_commits=0, empty authors, terrain_roughness=0.1, speed_base=0.0

### Step 4: Compute terrain_roughness from interval variance ✅
Replaced `log10(total_commits) / 5.0` with `(interval_cv(&timestamps) * 0.5).clamp(0.1, 1.0)`.
Timestamps collected during revwalk.

### Step 5: Add unit tests ✅
Added 18 tests in `#[cfg(test)] mod tests`:
- 9 HSL→RGB tests (all 6 primary/secondary hues + white, black, gray)
- 3 hash_to_hue tests (determinism, range, different inputs)
- 6 interval_cv tests (empty, single, two commits, regular, bursty, same-timestamp)

### Step 6: Final verification ✅
- `cargo build` — clean
- `cargo test` — 86 tests pass (18 in seed module)
- `cargo clippy` — no warnings

## Deviations from Plan

None. All steps executed as planned.
