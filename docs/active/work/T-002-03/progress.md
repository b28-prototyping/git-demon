# T-002-03 Progress: world-simulation

## Completed

### Step 1: Updated speed.rs thresholds and formula
- Changed `from_commits_per_min()` thresholds to 4.0/1.5/0.5/0.0 (was 1.0/0.5/0.15/0.0)
- Changed `speed_target()` to `0.4 + (cpm * 2.8).min(11.6)` (was `1.5 + (cpm * 28.0).min(28.5)`)
- Removed stale calibration comments

### Step 2: Added speed.rs unit tests
- 14 tests: tier boundaries, speed_target formula, tier names
- All pass

### Step 3: Updated mod.rs initial values
- Changed `speed` init from 1.5 to 0.4
- Changed `speed_target` init to `0.4 + seed.speed_base * 2.8`

### Step 4: Added mod.rs unit tests
- 20 tests covering: new(), update(), ingest_poll(), draw_distance(), sector(), tier_index()
- All pass

### Step 5: Full verification
- `cargo test --lib world` — 34 tests pass
- `cargo clippy` — no new warnings (1 pre-existing in main.rs)

## Deviations from Plan

- `test_update_speed_lerp` needed adjustment: `update()` recomputes `speed_target` from
  `commits_per_min`, so the test must set `commits_per_min` to maintain the desired target.

## Pre-existing Issues

- `renderer::effects::tests::test_speed_lines_alpha_values` fails — unrelated to this ticket,
  existed before our changes.
