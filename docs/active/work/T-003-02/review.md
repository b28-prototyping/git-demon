# T-003-02 Review: HUD Overlay

## Summary

The HUD overlay was already fully implemented (commit `8f30046`). This ticket's
work consisted of fixing a minor spec deviation (alpha value) and adding unit
test coverage for the existing implementation.

## Changes

### Modified Files
- `src/renderer/hud.rs`
  - Fixed `HUD_BG` alpha: 200 → 204 (78.4% → 80.0%, matching spec)
  - Changed `tier_badge_color` visibility: `fn` → `pub(crate) fn`
  - Added 12 unit tests in `#[cfg(test)] mod tests`

### No New Files (code)
No new source files created. Only work artifacts in `docs/active/work/T-003-02/`.

## Acceptance Criteria Status

| Criterion | Status |
|-----------|--------|
| 18px strip at frame bottom with 80% alpha black background | PASS (fixed from 78%) |
| Content rendered using bitmap font at 1x scale | PASS |
| Fields: SECTOR N, X.X c/min, +N -N, N files, tier badge, repo: name | PASS |
| Sector is total_commits / 100 | PASS |
| Tier badge colors (all 5 tiers including strobe) | PASS |
| Repo name right-aligned | PASS |
| HUD hidden when --no-hud | PASS |
| Background alpha compositing | PASS |

## Test Coverage

12 new tests added to `renderer::hud::tests`:
- 5 tier badge color tests (one per tier, including strobe verification)
- 1 alpha compositing correctness test
- 2 HUD region boundary tests (modifies bottom strip, doesn't leak above)
- 2 sector calculation tests
- 1 repo name right-alignment test
- 1 constant assertion test

Total lib test suite: **136 tests, all passing**.

### Coverage Gaps
- No test for `draw_dev_overlay()` — it's a debug-only feature, not in acceptance criteria
- No test for field content matching exact format strings (e.g., "2.5 c/min") — tested
  indirectly via pixel presence
- No test for narrow terminal field overlap — documented as known limitation

## Pre-existing Issues

`cargo clippy --lib` and `cargo test` (full binary build) fail due to unstaged changes
from another ticket that partially refactored `FrameRenderer::set_timing()` and
`draw_dev_overlay()` signatures. The `--lib` test suite runs cleanly.

## Open Concerns

1. **Hardcoded x-positions**: HUD fields at fixed pixel offsets (8, 120, 240, 380, 480)
   will overlap on terminals narrower than ~600px. Not a spec violation but worth noting
   for future improvement.

2. **Alpha precision**: The compositing uses `f32` division (`204.0 / 255.0`) which
   introduces minor floating-point rounding. This is consistent with how the rest of
   the renderer handles alpha (same pattern in dev overlay, speed lines, etc.).
