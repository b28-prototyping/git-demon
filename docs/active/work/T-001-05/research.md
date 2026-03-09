# Research — T-001-05: effects-passes

## Current State

All four effects already exist in `src/renderer/effects.rs` and are fully wired into the render pipeline in `src/renderer/mod.rs`. The ticket dependencies (T-001-02 road, T-001-03 renderer) are complete — the framebuffer, world state, and render orchestration are all in place.

## File Inventory

### `src/renderer/effects.rs` (306 lines)
Contains all four effect functions plus two supporting drawing functions:

1. **`draw_speed_lines()`** (L9–47) — Radiates `N = (cpm * 8).min(64)` lines from vanishing point `(w/2, horizon_y)`. Uses accent hue from seed. Alpha is 140 at tier ≥ 4 (VelocityDemon), 80 otherwise. Steps by 2 pixels for performance. Uses `blend_alpha()` compositing.

2. **`apply_motion_blur()`** (L49–72) — Blends current frame with previous using `lerp(0.15, 0.35, speed/12.0)`. Fixed-point integer math (0..256 scale). Operates on raw byte slices, skips alpha channel. Processes 4 bytes at a time (RGBA pixel).

3. **`apply_scanline_filter()`** (L74–96) — Darkens every even row (row % 2 == 0) by 20% using fixed-point multiply (205/256 ≈ 0.80). Operates on raw byte slices for speed.

4. **`apply_bloom()`** (L98–138) — Identifies pixels above luminance 0.72 (sampled every 2nd pixel in both axes for perf). Applies 3×3 additive blur with BLOOM_STRENGTH 0.3. Fixed-point weights: center=38/256, edge=19/256. Clamps at 255 via `.min(255)`.

5. **`draw_car()`** (L140–265) — Player car sprite. Not an "effect" per se but lives here.

### Helper functions (effects.rs)
- `luminance_fast()` — Approximate luminance: `(r + 2g + b) / 4 / 255`
- `lerp()` — Linear interpolation with clamped t
- `blend_alpha()` — Alpha compositing (fg over bg)
- `hue_to_rgb()` — HSV hue to RGB at full saturation/value

### `src/renderer/mod.rs` (107 lines)
`FrameRenderer` struct owns:
- `fb: ImageBuffer<Rgba<u8>>` — current frame
- `prev_fb: ImageBuffer<Rgba<u8>>` — previous frame for motion blur
- Flags: `no_blur`, `no_bloom`, `no_scanlines`, `no_hud`, `dev`

**Render order** (L52–105):
1. Sky → Sun → Bloom bleed → Terrain → Road → Grid → Sprites → Car
2. Speed lines (if tier ≥ 3, i.e., Demon+)
3. Motion blur (if !no_blur)
4. Scanline filter (if !no_scanlines)
5. Bloom (if !no_bloom)
6. HUD → Dev overlay
7. Buffer swap: `std::mem::swap(&mut prev_fb, &mut fb)`, returns `&prev_fb`

### `src/main.rs` (130 lines)
CLI args include `--no-blur`, `--no-bloom`, `--no-scanlines`. These are passed to `FrameRenderer::new()`.

### `src/world/speed.rs` (41 lines)
`VelocityTier` enum: Flatline(0), Cruise(1), Active(2), Demon(3), VelocityDemon(4).
Tier thresholds: 0, >0, ≥0.5, ≥1.5, ≥4.0 cpm.

### `src/world/mod.rs` (146 lines)
`WorldState` holds `speed`, `commits_per_min`, `tier`, `steer_angle`, etc.
`tier_index()` returns `tier as u8`.

## Acceptance Criteria Mapping

| Criterion | Code Location | Status |
|-----------|---------------|--------|
| Motion blur with lerp(0.15, 0.35, speed/12) | effects.rs:54 | ✅ Implemented |
| Motion blur --no-blur | mod.rs:79, main.rs:36 | ✅ Implemented |
| Scanline darkens even rows 20% | effects.rs:83,87-89 | ✅ Implemented |
| Scanline --no-scanlines | mod.rs:83, main.rs:43 | ✅ Implemented |
| Bloom threshold 0.72 | effects.rs:6 | ✅ Implemented |
| Bloom 3×3, BLOOM_STRENGTH 0.3 | effects.rs:7,123-134 | ✅ Implemented |
| Bloom --no-bloom | mod.rs:87, main.rs:40 | ✅ Implemented |
| Speed lines at Demon+ | mod.rs:75 (tier ≥ 3) | ✅ Implemented |
| Speed lines N = (cpm*8).min(64) | effects.rs:16 | ✅ Implemented |
| Speed lines from (cx, horizon_y) | effects.rs:18-19 | ✅ Implemented |
| Speed lines alpha 80/140 | effects.rs:20 | ✅ Implemented |
| Effect order: speed→blur→scanlines→bloom | mod.rs:75-88 | ✅ Correct order |
| Previous frame buffer swap | mod.rs:102 | ✅ Implemented |
| Pixel values clamp at 255 | effects.rs:131-133 (.min(255)) | ✅ In bloom; motion blur uses fixed-point that can't overflow u8 |

## Observations

1. **All four effects are fully implemented** with correct algorithms matching the spec.
2. **Effect ordering matches spec**: speed lines → motion blur → scanlines → bloom.
3. **No tests exist** for any effects functions. The `#[cfg(test)]` blocks exist in road.rs, terrain.rs, font.rs, and sky.rs but not effects.rs.
4. **Performance**: All hot-path effects use fixed-point integer math, raw byte slice access, and strategic sampling (bloom samples every 2nd pixel, speed lines step by 2).
5. **Overflow safety**: Bloom explicitly clamps with `.min(255)`. Motion blur and scanline use fixed-point math that naturally stays in u8 range. Speed lines use `blend_alpha()` which divides by 255.
6. **draw_car() is in effects.rs** — not an effect, but collocated here.

## Constraints

- No heap allocation in render hot path (project requirement).
- Bloom pre-allocates Vec with capacity 1024; this is fine as it reuses across iterations.
- The `prev_fb` swap is zero-copy (just pointer swaps via `std::mem::swap`).

## Open Questions

- Should tests cover pixel-level correctness or just structural/smoke behavior?
- The ticket asks for effects but all are already implemented — the work is primarily adding test coverage.
