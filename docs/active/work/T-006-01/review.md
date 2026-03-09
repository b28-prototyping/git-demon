# T-006-01 Review: commit-cars-on-road

## Summary

Replaced `CommitBillboard` with `CommitCar` — commits now appear as small colored racecars on the road surface instead of billboards on the roadside verge.

## Files Modified

| File | Change |
|------|--------|
| `src/world/objects.rs` | Added `RoadLeft`/`RoadRight` to `Lane` enum; renamed `CommitBillboard` → `CommitCar`; updated `ingest_poll_to_queue()` |
| `src/world/mod.rs` | Updated spawn logic to assign `CommitCar` to `RoadLeft`/`RoadRight`; renamed+added tests |
| `src/renderer/sprites.rs` | Added road lane projection arms; replaced billboard rendering with 3-level LOD car rendering; added helpers `darken()`, `brighten()`, `draw_commit_car()`; renamed+added tests |
| `benches/render.rs` | Updated variant names and lanes for commit car entries |

## Acceptance Criteria Coverage

| Criterion | Status | How |
|-----------|--------|-----|
| Each commit spawns a colored car on the road | Done | `ingest_poll_to_queue` emits `CommitCar` |
| Cars colored by author | Done | `author_color` from `RepoSeed` passed through, used for body/dark/light sides |
| Cars appear in road lanes and follow curvature | Done | `RoadLeft`/`RoadRight` at `0.35 * road_half`; curvature via `cx` formula |
| Cars grow in perspective as they approach | Done | Quadratic scale: `base * scale^2` |
| Commit message label appears when close | Done | Text drawn when `scale >= 0.5` |
| Other roadside objects remain on verge | Done | Only `CommitCar` gets road lanes; others keep `Left`/`Right` |
| Player car always renders in front | Done | Draw order unchanged: sprites at step 7, player car at step 7.5 |
| Tests updated | Done | 3 tests renamed, 2 new tests added |

## Test Coverage

**New tests (2):**
- `test_lane_assignment_commit_car` — verifies CommitCar gets road lanes, VelocitySign gets verge lanes
- `test_project_road_left_right` — verifies road lanes are between center and verge lanes
- `test_commit_car_lod_dot` — verifies far car (dot LOD) renders author_color pixels

**Updated tests (3):**
- `test_ingest_poll_creates_commit_car` (was `..._billboard`)
- `test_commit_car_color` (was `test_commit_billboard_color`)
- `test_commit_car_text_near` / `test_commit_car_text_suppressed_far`

**Total:** 161 tests, all passing. No test gaps identified for the scope of this ticket.

## Open Concerns

1. **LOD thresholds are untested at the boundary.** The `car_w < 4` and `car_w < 8` thresholds are tested indirectly (dot LOD test places a very far car, rect LOD at medium distance). A pixel-exact boundary test would require precise scale computation. Risk: low — the LOD levels are all visually reasonable.

2. **Wedge shape is simplified.** The commit car wedge (trapezoid body + triangle nose) is simpler than the player car (which has shadow, windshield, cockpit, tail lights). This is intentional — commit cars are smaller (2/3 size) and seen briefly during overtaking. Enhancement opportunity for a future ticket if more detail is desired.

3. **No visual regression testing.** The rendered output is tested via pixel color presence but not screenshot comparison. This is consistent with the existing test approach for all other sprite types.

4. **`darken()`/`brighten()` could be shared.** Similar color manipulation exists in `effects.rs` (`hue_to_rgb`). These are simple 3-line functions and don't warrant a shared module for now.
