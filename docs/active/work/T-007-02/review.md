# T-007-02 Review: Segment-Based Road with Hills

## Summary of Changes

### New Files
| File | Purpose |
|------|---------|
| `src/world/road_segments.rs` | `RoadSegment` struct, `generate_segment()`, constants |
| `src/renderer/road_table.rs` | `RoadRow` struct, `build_road_table()`, `lookup_at_z()` |

### Modified Files
| File | Changes |
|------|---------|
| `src/world/mod.rs` | Added `road_segments` module, `segments` and `segment_z_start` fields to WorldState, segment lifecycle in `new()` and `update()` |
| `src/renderer/mod.rs` | Added `road_table` module, `road_table_us` timing, road table build in `render()`, passes table to all draw functions |
| `src/renderer/road.rs` | `draw_road()` and `draw_grid()` accept `road_table` param; grid lines use per-row curve/slope offsets |
| `src/renderer/sprites.rs` | `project()` and `draw_sprites()` use road table for per-segment curve/slope |
| `src/renderer/terrain.rs` | `draw_islands()` uses curve+slope from road table; `draw_clouds()` uses curve only |

## Acceptance Criteria Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| Road visually rises and falls with hills | Done | Grid lines shift vertically via slope_offset; sprites and islands follow |
| Hills occlude road and objects behind them | Partial | Grid lines that would draw above horizon are skipped. Full visual occlusion (hiding far objects) depends on the slope magnitudes and future camera pitch work |
| Curvature works and combines with hills | Done | Per-segment curve accumulates alongside slope |
| Grid lines follow road surface through hills | Done | Both horizontal and vertical grid lines use road_table slope/curve |
| Sprites rise and fall with road surface | Done | `lookup_at_z()` provides slope_offset to sprite projection |
| Hill intensity responds to git activity | Done | `generate_segment()` scales slope amplitude with CPM (0.3 base → 0.8 at high CPM) |
| No Z-fighting or visual tearing | Done | All tearing integration tests pass |
| Performance: < 0.5ms per frame | Expected | Road table build is O(road_rows * segments) ≈ 150 * 40 = 6000 iterations; tracked via `road_table_us` timing |

## Test Coverage

### New Tests
- `road_segments::tests` — 4 tests: bounded values, CPM scaling, z variation, time variation
- `road_table::tests` — 7 tests: table size, flat/hilly offsets, z_world monotonicity, depth_scale monotonicity, lookup valid/invalid/correct

### Existing Tests Updated
- All existing tests in `road.rs`, `sprites.rs`, `terrain.rs` updated to pass road_table parameter (empty `&[]` for backward compatibility in unit tests)
- All 9 tearing integration tests pass

### Coverage Gaps
- No integration test that specifically verifies hill visual effect (e.g., that grid lines at different Z-depths have different Y-offsets due to slope). This would require a test that renders with flat vs hilly segments and compares grid line positions.
- No benchmark for road_table build time (tracked at runtime via PassTimings but no dedicated benchmark).

## Open Concerns

1. **Ocean fill is flat**: The ocean gradient doesn't shift with hills. The visual hill effect comes entirely from grid line and object displacement. This is correct for the OutRun-style aesthetic but means hills are subtle when the grid is sparse (near horizon). Future enhancement: add per-row color tint variation based on slope to suggest road pitch.

2. **Segment recycling uses `Vec::remove(0)`**: This is O(n) for the segment vector. With SEGMENT_COUNT=40 this is negligible, but could be changed to a VecDeque if needed.

3. **Slope offset scale factor**: The `* 40.0` magic number in `accumulate_offsets()` converts cumulative slope to pixel displacement. This may need tuning based on visual testing at different terminal sizes.

4. **Occlusion is approximate**: Far objects behind a hill crest are only hidden if their projected screen_y (after slope_offset) falls above the horizon. True occlusion would require tracking the "visible horizon" line that rises with hills. This is a natural follow-up for T-007-03 or later.

5. **Curve offset in road_table**: The road_table stores raw cumulative curve values; consumers multiply by `depth_scale²` themselves. This duplicates the quadratic application pattern. Could be precomputed in the table, but current approach matches the existing code pattern.
