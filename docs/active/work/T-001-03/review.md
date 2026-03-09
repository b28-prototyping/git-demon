# T-001-03 Review: road-rasterizer-with-curvature

## Summary of Changes

### Files Modified

| File | Change |
|------|--------|
| `src/renderer/road.rs` | Bug fix (cx wrapping) + 22 unit tests added |

No files created or deleted (tests are inline `#[cfg(test)]` module).

### Bug Fix: Integer Wrapping on `cx`

**Before:** `cx` was cast to `u32` before computing road boundaries. When `curve_offset` was large negative (up to -80 per spec) on narrow buffers, `cx_base + curve_shift` could go negative, wrapping to `u32::MAX`.

**After:** `cx` stays as `f32`. `road_l` computed via `(cx - road_half).max(0.0) as u32`, clamping to zero instead of wrapping. This is a 2-line change with zero visual impact at normal resolutions.

The linter also applied performance optimizations to the scanline loop: raw buffer access instead of `put_pixel`, pre-computed `road_rows` constant, and `rumble_l` pre-computation outside the inner loop.

**Cleanup:** `blend_alpha` was moved under `#[cfg(test)]` since `draw_grid` was rewritten to use inline integer alpha blending and no longer calls it. This eliminates a dead-code warning in non-test builds.

## Test Coverage

### Pure function tests (13 tests)
| Test | Covers |
|------|--------|
| `test_lerp_at_zero/one/midpoint` | `lerp()` boundary and midpoint |
| `test_horizon_ratio_normal/velocity_demon` | `horizon_ratio()` — 0.35 vs 0.37 |
| `test_road_max_half_normal/velocity_demon` | `road_max_half()` — 480.0 vs 504.0 |
| `test_blend_alpha_opaque/transparent/half` | `blend_alpha()` all alpha extremes |
| `test_hsl_to_rgb_red/green` | `hsl_to_rgb_inline()` primary colors |
| `test_hue_to_neon_returns_opaque` | `hue_to_neon()` alpha = 255 |

### Rendering invariant tests (9 tests)
| Test | Covers |
|------|--------|
| `test_draw_road_perspective_width` | Road wider at bottom than horizon |
| `test_draw_road_stripe_colors_present` | Both stripe tones appear |
| `test_draw_road_rumble_colors_present` | Both rumble colors appear |
| `test_draw_road_verge_colors_present` | Both verge colors appear |
| `test_draw_road_curve_shifts_center` | Curve offset shifts road center |
| `test_draw_road_velocity_demon_wider` | VelocityDemon expands road width |
| `test_draw_road_no_panic_extreme_curve` | ±80 offset on 100×100 buffer is safe |
| `test_draw_grid_accent_pixels` | Grid modifies road pixels |
| `test_draw_grid_alpha_blended` | Grid uses alpha blend, not solid overwrite |

### Coverage Gaps
- No test for grid lines following curve offset specifically (tested indirectly via accent pixel presence)
- No test for `draw_distance()` interaction with grid (tested indirectly — grid renders on Cruise tier)
- No visual/snapshot regression tests (intentional — too brittle for procedural renderer)

## Acceptance Criteria Verification

All 9 acceptance criteria are met by the existing implementation:

1. ✅ Perspective road — scanline rasterizer with depth-interpolated width
2. ✅ Stripe alternation — `z_offset / depth` modulo stripe period
3. ✅ Rumble strips — 12px white/red alternating bands
4. ✅ Verge regions — alternating green tones outside rumble
5. ✅ Curve offset — `curve_offset * depth²` quadratic shift
6. ✅ Grid lines — neon accent horizontal lines at Z intervals, alpha 150
7. ✅ Grid follows curve — same `curve_offset * depth²` formula
8. ✅ VelocityDemon expansion — `ROAD_MAX_HALF * 1.05`, horizon `+0.02`
9. ✅ `horizon_ratio()` and `road_max_half()` tier-aware

## Build Status

- `cargo test` — 44 tests pass (22 new + 22 pre-existing)
- `cargo clippy` — no new warnings
- `cargo build` — clean

## Open Concerns

1. **Grid depth_scale vs depth inconsistency**: `draw_grid` uses `depth_scale` (world-Z based) for road width interpolation but `depth` (screen-Y based) for curve shift. This is likely intentional for correct visual spacing but could produce subtle alignment artifacts at extreme depths. Low risk.

2. **Pre-existing clippy warnings**: `BLOOM_STRENGTH` unused in `effects.rs`, `proto` assignment in `main.rs`. Not introduced by this ticket.
