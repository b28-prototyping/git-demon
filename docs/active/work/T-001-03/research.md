# T-001-03 Research: road-rasterizer-with-curvature

## Current State

The road rasterizer is **already implemented** in `src/renderer/road.rs` (165 lines). It provides two public draw functions (`draw_road`, `draw_grid`) and two public helper functions (`horizon_ratio`, `road_max_half`), plus private utilities (`lerp`, `hue_to_neon`, `hsl_to_rgb_inline`, `blend_alpha`).

## File Map

| File | Role | Relevance |
|------|------|-----------|
| `src/renderer/road.rs` | Road scanline rasterizer + grid | Primary — contains all road rendering logic |
| `src/renderer/mod.rs` | FrameRenderer orchestrator | Calls `road::draw_road()` and `road::draw_grid()` at passes 5-6 |
| `src/world/mod.rs` | WorldState struct | Provides `z_offset`, `curve_offset`, `tier`, `draw_distance()` |
| `src/world/speed.rs` | VelocityTier enum | `VelocityDemon` triggers road geometry expansion |
| `src/git/seed.rs` | RepoSeed struct | Provides `accent_hue` for grid line color |

## Constants Defined in road.rs

| Constant | Value | Purpose |
|----------|-------|---------|
| `BASE_HORIZON_RATIO` | 0.35 | Horizon at 35% down from top |
| `ROAD_MIN_HALF` | 8.0 | Half-width at horizon (px) |
| `ROAD_MAX_HALF` | 480.0 | Half-width at camera (px) |
| `RUMBLE_WIDTH` | 12 | Rumble strip width each side (px) |
| `STRIPE_PERIOD` | 1.0 | World units per stripe cycle |

## Acceptance Criteria vs Implementation

1. **Perspective road** — `draw_road` iterates scanlines from `horizon_y` to `h`, computes `depth = (y - horizon_y) / (h - horizon_y)`, interpolates half-width via `lerp(ROAD_MIN_HALF, max_half, depth)`. ✅ Implemented.

2. **Stripe alternation** — `stripe = ((world.z_offset / depth.max(0.01)) % (STRIPE_PERIOD * 2.0)) < STRIPE_PERIOD`. Uses `z_offset` and `depth` as spec requires. ✅ Implemented.

3. **Rumble strips** — 12px each side, `RUMBLE_WHITE` / `RUMBLE_RED` alternating with stripe phase. ✅ Implemented.

4. **Verge regions** — `VERGE_A` (dark green) / `VERGE_B` (darker green) outside rumble strips. ✅ Implemented.

5. **Curve offset** — `curve_shift = world.curve_offset * depth * depth`, applied to center `cx`. Quadratic with depth for perspective correctness. ✅ Implemented.

6. **Grid lines** — `draw_grid` projects world-Z intervals (`grid_spacing = 5.0`) into screen-Y, draws horizontal lines in accent hue at alpha 150. ✅ Implemented.

7. **Grid follows curve** — Grid uses same `curve_offset * depth * depth` formula. ✅ Implemented.

8. **VelocityDemon expansion** — `road_max_half()` returns `ROAD_MAX_HALF * 1.05` (504.0). ✅ Implemented.

9. **VelocityDemon horizon** — `horizon_ratio()` returns `0.35 + 0.02 = 0.37`. ✅ Implemented.

## Integration Points

- `FrameRenderer::render()` computes `horizon_y = (h * road::horizon_ratio(world)) as u32` and passes it to both road functions.
- `draw_road` writes directly to framebuffer pixels — no alpha blending, full overwrite.
- `draw_grid` reads existing pixels for alpha blending via `blend_alpha()`.
- `draw_grid` uses `world.draw_distance()` which returns 240.0 at VelocityDemon (vs 200.0).

## Identified Gaps

1. **No tests** — `road.rs` has zero test coverage. All other rendering modules (sky, font) have tests. This is the primary gap.

2. **Potential integer overflow** — `cx` is cast to `u32` after adding `curve_shift` (which can be negative). When `curve_offset` is large negative and `depth` is high, `cx_base + curve_shift` can go negative, and casting to `u32` wraps. The `saturating_sub` on `road_l` provides partial safety, but `cx` itself could wrap.

3. **Grid depth inconsistency** — `draw_grid` uses `depth_scale` for road width interpolation but `depth` (computed from screen-Y) for curve shift. `draw_road` uses `depth` for both. This is intentional — grid spacing is world-Z-based, not screen-Y-based — but worth verifying visually.

## Patterns from Other Modules

- `sky.rs` tests: unit tests for `hsl_to_rgb`, `blend_alpha`, `lerp_u8` — pure function testing.
- `font.rs` tests: tests for glyph data correctness and text measurement.
- Pattern: test pure functions directly; rendering functions tested via framebuffer assertions.

## Dependencies for Testing

- `WorldState::new()` requires a `RepoSeed` — can construct a test seed without git repo.
- `RepoSeed` has no `Default` impl but all fields are public, so test helpers can construct directly.
- `ImageBuffer::new()` from `image` crate — zero-initialized RGBA buffer.
