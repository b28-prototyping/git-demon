# T-002-03 Plan: world-simulation

## Step 1: Update speed.rs thresholds and formula

1. Change `from_commits_per_min()` thresholds to 4.0/1.5/0.5/0.0
2. Update comments to reflect new calibration
3. Change `speed_target()` to `0.4 + (cpm * 2.8).min(11.6)`
4. Update comments with new speed table

**Verify:** `cargo build` succeeds.

## Step 2: Add speed.rs unit tests

Add `#[cfg(test)] mod tests` to speed.rs:
- `test_tier_flatline` — 0.0 cpm → Flatline
- `test_tier_cruise` — 0.1 cpm → Cruise (above 0, below 0.5)
- `test_tier_active` — 0.5 cpm → Active
- `test_tier_demon` — 1.5 cpm → Demon
- `test_tier_velocity_demon` — 4.0 cpm → VelocityDemon
- `test_tier_boundary_below_active` — 0.49 cpm → Cruise
- `test_tier_boundary_below_demon` — 1.49 cpm → Active
- `test_tier_boundary_below_vdemon` — 3.99 cpm → Demon
- `test_speed_target_flatline` — 0.0 → 0.4
- `test_speed_target_cap` — 100.0 → 12.0 (0.4 + 11.6)
- `test_speed_target_mid` — 2.0 → 6.0 (0.4 + 5.6)
- `test_tier_name` — verify all name() strings

**Verify:** `cargo test --lib world::speed` passes.

## Step 3: Update mod.rs initial values

1. Change `speed` init from `1.5` to `0.4`
2. Change `speed_target` init from `1.5 + seed.speed_base * 2.8`
   to `0.4 + seed.speed_base * 2.8`

**Verify:** `cargo build` succeeds.

## Step 4: Add mod.rs unit tests

Add `#[cfg(test)] mod tests` to world/mod.rs:

Helper: `test_seed()` → RepoSeed with known values.
Helper: `test_poll_result(commits, cpm)` → PollResult with known data.

Tests:
- `test_new_defaults` — verify initial field values
- `test_update_speed_lerp` — after update(dt), speed moves toward target
- `test_update_z_advances` — z_offset and camera_z increase by speed*dt
- `test_update_tier_recomputed` — tier updates from commits_per_min
- `test_update_despawn` — objects behind camera removed
- `test_update_spawn_pending` — pending objects placed into active_objects
- `test_ingest_poll_creates_billboard` — commit → CommitBillboard
- `test_ingest_poll_creates_addition_tower` — >50 lines_added → AdditionTower
- `test_ingest_poll_creates_deletion_shard` — >50 lines_deleted → DeletionShard
- `test_ingest_poll_tier_gate` — tier change → TierGate at NEAR_SPAWN
- `test_ingest_poll_curve_shift` — cpm > 1.0 → curve_target changes
- `test_draw_distance_normal` — non-VDemon → 200.0
- `test_draw_distance_vdemon` — VelocityDemon → 240.0
- `test_lane_alternation` — objects alternate Left/Right

**Verify:** `cargo test --lib world` passes.

## Step 5: Full verification

Run `cargo test` and `cargo clippy` to confirm no regressions.

**Verify:** All tests pass, no clippy warnings.
