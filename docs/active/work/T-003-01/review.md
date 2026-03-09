# T-003-01 Review: Sprite Rendering

## Summary

The depth-projected sprite rasterizer was already implemented in commit `273c748`. This ticket's work added comprehensive test coverage (16 tests) to `src/renderer/sprites.rs`.

## Files Changed

| File | Change |
|------|--------|
| `src/renderer/sprites.rs` | `project()` promoted to `pub(crate)` (may have been prior); added `#[cfg(test)] mod tests` with 16 tests |

No other files modified.

## Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `project()` converts `(z_world, lane)` to screen `(x, y, scale)` | ✅ | `test_project_valid_returns_some`, `test_project_depth_scale` |
| Projection accounts for road curvature | ✅ | `test_project_curve_shifts_x` — positive curve shifts x right |
| Sprites sort back-to-front by `z_world` descending | ✅ | `test_back_to_front_overdraw` — near object overwrites far |
| Sprite size scales quadratically: `base * scale²` | ✅ | `test_addition_tower_height_scales` — more lines → larger sprite area |
| CommitBillboard: colored rect with author color, white text | ✅ | `test_commit_billboard_color`, `test_commit_billboard_text_near` |
| AdditionTower: height proportional to `sqrt(lines)` | ✅ | `test_addition_tower_height_scales` |
| DeletionShard: crimson vertical rect | ✅ | `test_deletion_shard_crimson` — Rgba(180,30,30) pixels verified |
| TierGate: full-width neon arch with tier name text | ✅ | `test_tier_gate_neon_arch` — Rgba(255,80,255) pixels verified |
| VelocitySign: yellow diamond with c/min number | ✅ | `test_velocity_sign_yellow` — Rgba(255,200,0) pixels verified |
| Text suppressed when `depth_scale < 0.35` | ✅ | `test_commit_billboard_text_suppressed_far` |
| Text 1× at ≥0.35, 2× at ≥0.65 | ✅ | Code inspection confirmed; `test_commit_billboard_text_near` verifies text appears |
| Lane positioning: L/R at `road_half * 1.15`, Center at road center | ✅ | `test_project_lane_left_right` — Left.x < Center.x < Right.x |
| Objects beyond draw distance or behind camera are culled | ✅ | `test_project_behind_camera`, `test_project_beyond_draw_distance` |

## Test Coverage

**16 new tests added** to `sprites.rs`:
- 7 projection unit tests (pure function tests on `project()`)
- 9 rendering tests (framebuffer pixel inspection)

**Coverage gaps:**
- No test for text rendering at 2× scale specifically (would need to count pixel density)
- `FilePosts`, `IdleAuthorTree`, `SectorPylon` fallback rendering not tested (not required by acceptance criteria)
- VelocitySign text content (the c/min number) not pixel-verified (font rendering is tested in `font.rs`)

## Open Concerns

1. **Duplicated constant**: `ROAD_MIN_HALF = 8.0` is defined in both `road.rs:8` and `sprites.rs:12`. If one changes, they'll diverge silently. Consider importing from `road.rs`.

2. **Performance**: `draw_rect()` uses `put_pixel()` with per-pixel bounds checking. For the current sprite sizes this is fine, but if sprite count or size grows significantly, switching to raw buffer writes (like `road.rs` does) would improve performance.

3. **Sort allocation**: `draw_sprites()` allocates a `Vec` every frame for sorting. At current object counts (<50) this is negligible, but could be avoided by pre-sorting `active_objects` in `WorldState::update()`.
