# T-002-03 Review: world-simulation

## Summary of Changes

### Files Modified

- **src/world/speed.rs** — Updated `VelocityTier::from_commits_per_min()` thresholds
  from (1.0/0.5/0.15/0.0) to (4.0/1.5/0.5/0.0) per spec. Updated `speed_target()`
  formula from `1.5 + (cpm * 28.0).min(28.5)` to `0.4 + (cpm * 2.8).min(11.6)`.
  Added 14 unit tests.

- **src/world/mod.rs** — Updated `WorldState::new()` initial speed from 1.5 to 0.4
  and initial speed_target formula to match. Added 20 unit tests covering all
  WorldState methods.

### Files Created

- `docs/active/work/T-002-03/research.md`
- `docs/active/work/T-002-03/design.md`
- `docs/active/work/T-002-03/structure.md`
- `docs/active/work/T-002-03/plan.md`
- `docs/active/work/T-002-03/progress.md`
- `docs/active/work/T-002-03/review.md` (this file)

## Acceptance Criteria Verification

| AC | Status | Evidence |
|---|---|---|
| `WorldState::new()` initializes from `RepoSeed` | PASS | `test_new_defaults` |
| `update(dt)` advances z_offset, camera_z, speed lerp, curve, tier | PASS | `test_update_*` (5 tests) |
| Speed lerps at 4.0 * dt toward spec formula | PASS | `test_update_speed_lerp`, `test_speed_target_*` |
| VelocityTier transitions at 0/0.5/1.5/4.0 | PASS | `test_tier_*` (9 tests) |
| ingest_poll converts commits to objects | PASS | `test_ingest_poll_creates_*` (4 tests) |
| Objects spawn at camera_z + SPAWN_DISTANCE | PASS | `test_update_spawn_pending`, `test_lane_alternation` |
| TierGate at camera_z + NEAR_SPAWN on tier change | PASS | `test_ingest_poll_tier_gate_on_change` |
| Curve target shifts on >1 cpm | PASS | `test_ingest_poll_curve_shift_on_burst` |
| Despawn behind camera | PASS | `test_update_despawn` |
| draw_distance +20% at VelocityDemon | PASS | `test_draw_distance_velocity_demon` |

## Test Coverage

- **world::speed** — 14 tests: all tier boundaries, speed_target formula at key values,
  tier names. Full coverage of both public functions.

- **world::mod** — 20 tests: WorldState construction, update loop mechanics (speed lerp,
  z advancement, time tracking, tier recomputation, despawn, spawn), ingest_poll
  (billboard, tower, shard creation, tier gate, curve shift), draw_distance,
  sector, tier_index, lane alternation.

- **world::objects** — No new tests added. `ingest_poll_to_queue` is tested indirectly
  via WorldState tests. The function is straightforward mapping logic.

Total new tests: 34. All pass.

## Pre-existing Issues

- `renderer::effects::tests::test_speed_lines_alpha_values` — fails before and after
  this change. Unrelated; the test expects 3 distinct alpha values but the speed
  lines code produces 2. This is a pre-existing test/code mismatch in effects.rs.

## Open Concerns

- **Visual tuning** — The speed and tier threshold changes will make the simulation
  noticeably slower and harder to tier up. This matches the spec but may feel
  sluggish during demos. A future ticket could add `--fast` or `--demo` mode.

- **Curve target determinism** — `ingest_poll` uses `rand::rng()` for curve_target
  randomization, making exact curve behavior non-deterministic in tests. The test
  only checks range bounds. A seeded RNG could be injected for full determinism,
  but this is not required by the ACs.
