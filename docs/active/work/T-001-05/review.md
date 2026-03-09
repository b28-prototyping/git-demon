# Review — T-001-05: effects-passes

## Summary

All four post-processing effects (motion blur, scanline filter, bloom, speed lines) were already fully implemented in `src/renderer/effects.rs` and correctly wired into the render pipeline in `src/renderer/mod.rs`. The implementation matches every acceptance criterion in the ticket. This task added comprehensive unit test coverage — 24 new tests — to verify correctness and prevent regressions.

## Files Changed

### Modified
- **`src/renderer/effects.rs`** — Added `#[cfg(test)] mod tests` block (~180 lines) at the end of the file. No production code changes.

### Not Modified
- `src/renderer/mod.rs` — Effect ordering and disable flags already correct.
- `src/main.rs` — CLI args (`--no-blur`, `--no-bloom`, `--no-scanlines`) already wired.
- No new files created outside the work artifact directory.

## Test Coverage

24 new tests added, organized by function:

| Category | Tests | Coverage |
|----------|-------|----------|
| `lerp()` | 3 | Midpoint, clamp below 0, clamp above 1 |
| `luminance_fast()` | 3 | White=1.0, black=0.0, bright exceeds threshold |
| `blend_alpha()` | 3 | Fully opaque, fully transparent, half alpha |
| `hue_to_rgb()` | 3 | Red (0°), green (120°), blue (240°) |
| `apply_scanline_filter()` | 2 | Even rows darkened 20%, odd rows untouched |
| `apply_motion_blur()` | 3 | Zero speed blend, max speed blend, alpha channel preserved |
| `apply_bloom()` | 3 | Neighbor brightening, clamping at 255, dark pixels unaffected |
| `draw_speed_lines()` | 4 | Draws at Demon tier, no-op at zero cpm, count formula, alpha 80/140 |

Full suite: **86 tests pass**, 0 failures, 0 clippy warnings.

## Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Motion blur lerp(0.15, 0.35, speed/12.0) | ✅ | effects.rs:54, verified by `test_motion_blur_at_zero_speed` and `test_motion_blur_at_max_speed` |
| Motion blur --no-blur | ✅ | mod.rs:79, main.rs:36 |
| Scanline darkens even rows 20% | ✅ | effects.rs:83-89, verified by `test_scanline_darkens_even_rows` |
| Scanline --no-scanlines | ✅ | mod.rs:83, main.rs:43 |
| Bloom threshold 0.72, 3×3, strength 0.3 | ✅ | effects.rs:6-7,123-134, verified by `test_bloom_brightens_neighbors` and `test_bloom_ignores_dark_pixels` |
| Bloom --no-bloom | ✅ | mod.rs:87, main.rs:40 |
| Speed lines at Demon+ (tier ≥ 3) | ✅ | mod.rs:75, verified by `test_speed_lines_draws_at_demon_tier` |
| Speed lines N = (cpm*8).min(64) | ✅ | effects.rs:16, verified by `test_speed_lines_count_formula` |
| Speed lines from (cx, horizon_y) | ✅ | effects.rs:18-19 |
| Speed lines alpha 80 Demon, 140 VelocityDemon | ✅ | effects.rs:20, verified by `test_speed_lines_alpha_values` |
| Order: speed lines → blur → scanlines → bloom | ✅ | mod.rs:75-88, structural — correct in code |
| Previous frame buffer swap | ✅ | mod.rs:102 |
| Pixel values clamp at 255 | ✅ | effects.rs:131-133, verified by `test_bloom_clamps_at_255` |

## Open Concerns

- **draw_car() lives in effects.rs** — It's not a post-processing effect; it's a sprite. Could be moved to sprites.rs in a future cleanup ticket, but is out of scope here.
- **No integration test for effect ordering** — The ordering is structural (sequential calls in `render()`) and verified by reading mod.rs. An integration test would require rendering a full frame which needs a valid RepoSeed (from a git repo). Not worth the complexity.

## Known Limitations

- Bloom samples every 2nd pixel in both axes for performance. This means a bright pixel at an odd coordinate won't be detected as emissive. This is intentional (spec says "soft effect") and not a bug.
- Motion blur fixed-point math has ~0.4% rounding error vs floating point. Acceptable for a visual effect.
