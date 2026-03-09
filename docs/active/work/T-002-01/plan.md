# Plan — T-002-01: repo-seed-computation

## Implementation Steps

### Step 1: Add `interval_cv` helper function

Add a new private function below `hsl_to_rgb` in seed.rs:

```rust
fn interval_cv(timestamps: &[i64]) -> f32
```

Takes timestamps sorted descending. Computes inter-commit intervals (differences
between consecutive elements). Returns coefficient of variation (stddev / mean).
Returns 0.0 if fewer than 2 intervals.

**Verify**: Unit tests for regular, bursty, single, and empty cases.

### Step 2: Fix accent_hue derivation

Modify `compute()` to derive `accent_hue` from:
1. Remote origin URL string (if remote exists)
2. Root commit SHA (last in time-sorted walk, if commits exist)
3. `repo_name` (final fallback for empty repos)

Extract the origin URL string before the revwalk. Capture the last SHA during the
walk (root commit). After the walk, determine identity priority.

**Verify**: Manual test — `cargo run -- --repo .` should still work. The accent_hue
will change for repos with a remote (now uses URL instead of HEAD SHA). This is a
correct behavior change.

### Step 3: Handle empty repo edge case

Wrap `revwalk.push_head()` in a match/if-let. On failure, skip the revwalk loop
entirely. Set defaults: total_commits=0, empty authors, terrain_roughness=0.1,
speed_base=0.0.

**Verify**: `cargo test` passes. Manual test with `git init /tmp/empty-test &&
cargo run -- --repo /tmp/empty-test` should not crash.

### Step 4: Compute terrain_roughness from interval variance

Replace the `log10(total_commits) / 5.0` formula with:
1. Collect timestamps during revwalk into `Vec<i64>`
2. Call `interval_cv(&timestamps)` after the walk
3. `terrain_roughness = (cv * 0.5).clamp(0.1, 1.0)`

**Verify**: `cargo test` and `cargo clippy`.

### Step 5: Add unit tests

Add `#[cfg(test)] mod tests` block to seed.rs:
- HSL→RGB: 7 known-value tests covering all 6 hue sectors + edge cases
- hash_to_hue: determinism, range, different-inputs
- interval_cv: regular, bursty, single-commit, empty

**Verify**: `cargo test` — all new tests pass.

### Step 6: Final verification

- `cargo build` — clean compile
- `cargo test` — all tests pass
- `cargo clippy` — no warnings

## Testing Strategy

| What | Type | Verification |
|------|------|-------------|
| `hsl_to_rgb` known values | Unit | Exact RGB match for primary colors + neutrals |
| `hash_to_hue` determinism | Unit | Two calls with same input return same value |
| `hash_to_hue` range | Unit | Output is in [0.0, 360.0) |
| `interval_cv` regular | Unit | CV ≈ 0 for equally-spaced timestamps |
| `interval_cv` bursty | Unit | CV > 0 for varied intervals |
| `interval_cv` edge cases | Unit | 0-1 commits → 0.0 |
| Empty repo | Integration | `compute()` returns Ok with defaults, no panic |
| Full build | Build | `cargo build` succeeds |
| Lint | Lint | `cargo clippy` clean |

## Commit Plan

Single commit for all changes since they're all in one file and form a cohesive unit.
