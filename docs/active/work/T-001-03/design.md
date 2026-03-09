# T-001-03 Design: road-rasterizer-with-curvature

## Problem Statement

The road rasterizer is fully implemented but has zero test coverage, and there is a potential integer overflow bug when `curve_offset` is large negative. The task is to add comprehensive tests and fix any issues found.

## Option A: Tests Only (No Code Changes)

Add unit tests for all public functions and key rendering invariants. Accept the `u32` wrapping behavior as "good enough" since `saturating_sub` prevents most out-of-bounds writes.

**Pros:** Minimal diff, no risk of visual regression.
**Cons:** The `cx` wrapping bug remains — at `curve_offset = -80` and `depth = 1.0`, `cx_base + curve_shift` = `width/2 - 80`. For a 200px-wide buffer that's 20, fine. But for a very narrow buffer (< 160px), it could go negative and wrap to a huge u32, causing road boundaries to be nonsensical. Realistically unlikely at terminal resolutions.

## Option B: Fix cx Wrapping + Tests

Change `cx` from `u32` to `f32` (or `i32`) throughout the scanline loop, compute `road_l` and `road_r` as `i32`, then clamp to `[0, w)` before pixel writes. Add tests.

**Pros:** Eliminates potential panic/wrap at extreme parameters.
**Cons:** Slightly more code churn; road_l/road_r clamping needs care.

## Option C: Comprehensive Rewrite + Tests

Restructure the scanline loop for clarity, extract a `ScanlineParams` struct, etc.

**Pros:** Cleaner abstraction.
**Cons:** Over-engineering. The current code is 85 lines and perfectly readable.

## Decision: Option B

The `cx` wrapping is a real bug that surfaces at extreme `curve_offset` values (which the spec allows up to ±80). On narrow framebuffers (which occur when terminal windows are small), `cx_base` can be as low as ~50px, and `curve_shift` at depth=1.0 can be -80, yielding -30 — which wraps to `u32::MAX - 30`. This cascades into `road_l` and `road_r` being nonsensical.

Fix: keep `cx` as `f32`, compute `road_l` and `road_r` as `f32`, clamp both to `[0, w-1]` before the pixel loop. This is a 3-line change, zero visual impact on normal resolutions, and prevents wrapping at small sizes.

## Test Strategy

### Pure function tests
- `lerp(a, b, t)` — boundary values (t=0, t=1, t=0.5)
- `horizon_ratio` — normal tier vs VelocityDemon
- `road_max_half` — normal tier vs VelocityDemon
- `blend_alpha` — fully opaque, fully transparent, partial
- `hue_to_neon` — known hue values
- `hsl_to_rgb_inline` — primaries, achromatic

### Rendering invariant tests
- `draw_road` on small buffer: verify road pixels exist in expected region
- `draw_road`: horizon scanline row should have narrow road, bottom row should have wide road
- `draw_road`: verify verge, rumble, and road colors appear
- `draw_road` with curve_offset: verify road center shifts
- `draw_road` with VelocityDemon tier: verify wider road
- `draw_grid`: verify accent-colored pixels appear on road
- `draw_grid`: verify grid pixels are alpha-blended (not fully opaque accent)
- `draw_road` with extreme negative curve_offset on narrow buffer: no panic

### What NOT to test
- Exact pixel positions (too brittle, depends on buffer size)
- Exact color values in rendered output (already tested via constant assertions)
- Integration with FrameRenderer (out of scope for this ticket)

## Rejected Alternatives

- **Snapshot/golden-image testing**: Too brittle for a procedural renderer that may evolve. Invariant-based tests are more maintainable.
- **Property-based testing (proptest/quickcheck)**: Overkill for ~6 functions. Targeted edge cases are sufficient.
- **Extracting a ScanlineParams struct**: No second consumer exists. YAGNI.
