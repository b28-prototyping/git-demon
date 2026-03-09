# Review — T-001-04: terrain-silhouettes

## Summary of Changes

### Files Modified
- **`src/renderer/terrain.rs`** — Added `#[cfg(test)] mod tests` block (~120 lines)
  containing 2 helper functions and 8 unit tests.

### Files Created
- `docs/active/work/T-001-04/progress.md` — implementation progress tracking

### No Files Deleted

## What Changed

The terrain silhouette implementation was already complete before this ticket started.
All seven acceptance criteria were satisfied by existing code. The sole deliverable
was adding test coverage to reach parity with peer renderer modules (sky.rs has 8
tests; terrain.rs had 0).

### Tests Added (8 total)

| Test | What it verifies |
|------|-----------------|
| `test_left_right_different_noise` | Different seeds (42 vs 137) produce different height profiles |
| `test_roughness_scales_height` | `terrain_roughness=1.0` produces taller terrain than `0.1` |
| `test_silhouette_filled` | No gaps in filled columns from top to horizon |
| `test_colors_match` | Left terrain uses `[8,12,20]`, right uses `[12,14,22]`; tree colors allowed |
| `test_time_drift` | Noise output changes between `time=0.0` and `time=10.0` |
| `test_no_terrain_below_horizon` | No terrain pixels written at `y >= horizon_y` |
| `test_terrain_boundaries` | No terrain colors in middle band (`w/4` to `w*3/4`) |
| `test_zero_horizon_safe` | No panic with `horizon_y=0`, no silhouette pixels written |

### Test Helpers

- `make_seed(roughness) -> RepoSeed` — minimal fixture with configurable roughness
- `make_world(time, seed) -> WorldState` — world at given time via `WorldState::new`

## Test Coverage

- **Terrain silhouette drawing:** Well covered — noise sampling, height scaling,
  fill correctness, color assignment, boundary behavior, time drift, edge cases.
- **Tree drawing (`draw_trees`):** Not directly tested by dedicated tests. Trees are
  implicitly exercised (they run during `draw_terrain` calls) and `test_colors_match`
  validates their colors don't corrupt terrain regions. Dedicated tree tests belong
  to the trees ticket, not this one.
- **`draw_rect` / `draw_diamond` helpers:** Exercised indirectly via tree drawing.

## Pre-existing Issues (Not Introduced by This Change)

- 4 failing tests in `road.rs` (`test_road_max_half_velocity_demon`,
  `test_draw_grid_accent_pixels`, `test_draw_road_curve_shifts_center`,
  `test_draw_road_velocity_demon_wider`) — these pre-date this ticket.
- `BLOOM_STRENGTH` unused warning in `effects.rs` — pre-existing.
- `road_pixel_before` unused variable in `road.rs` tests — pre-existing.

## Open Concerns

None. The implementation was already correct and integrated. Tests confirm
correctness and provide regression safety. No behavioral changes were made.
