# T-005-01 Research: fix-grid-line-scrolling

## Problem Summary

Grid lines in `draw_grid` (`src/renderer/road.rs:96-147`) are placed at fixed
world-Z positions that never move relative to the camera. As `camera_z` advances,
lines approach but never exit the screen — a Zeno's paradox accumulation effect.
Road stripes in `draw_road` correctly scroll via `z_offset / depth` (line 64).

## Relevant Code

### `src/renderer/road.rs` — `draw_grid` (lines 96-147)

The function draws horizontal neon grid lines across the road surface:

```rust
let grid_spacing = 5.0_f32;
for i in 1..40 {
    let z_world = i as f32 * grid_spacing;                          // BUG: fixed position
    let depth_scale = (1.0 - z_world / world.draw_distance()).clamp(0.0, 1.0);
    // ... maps depth_scale to screen Y via lerp(horizon_y, h, depth_scale)
}
```

Key observations:
- `z_world` is always `i * 5.0` — never changes frame to frame
- `depth_scale` maps world-Z to a 0..1 range based on `draw_distance()`
- Screen Y is computed via `lerp(horizon_y, h, depth_scale)` — same every frame
- Grid lines correctly follow road curvature via `curve_shift` (line 123)
- Alpha blending at `fg_a = 150` (≈59%) gives the neon look

### `src/renderer/road.rs` — `draw_road` (lines 40-94)

Road stripes scroll correctly:

```rust
let stripe = ((world.z_offset / depth.max(0.01)) % (STRIPE_PERIOD * 2.0)) < STRIPE_PERIOD;
```

`z_offset` is the key: it accumulates `speed * dt` each frame (world/mod.rs:62),
creating the forward-motion illusion. The grid doesn't use `z_offset` or `camera_z`.

### `src/world/mod.rs` — WorldState (lines 11-28, 57-63)

Relevant fields:
- `z_offset: f32` — accumulated distance, used for stripe phase
- `camera_z: f32` — accumulated camera position, used for object spawning/despawning
- Both advance by `speed * dt` each frame (lines 62-63)
- `draw_distance()` returns 200.0 (or 240.0 at VelocityDemon)

`z_offset` and `camera_z` are updated identically — both += `speed * dt`.
Effectively interchangeable for this use case. The ticket suggests using `camera_z`.

### Existing Tests

`road.rs` has 16 tests (lines 174-522):
- `test_draw_grid_accent_pixels` — verifies grid modifies road pixels
- `test_draw_grid_alpha_blended` — verifies grid lines are blended, not pure accent
- Neither test checks that grid lines move when `camera_z` changes

### Call Site

`draw_grid` is called from `FrameRenderer::render` at `renderer/mod.rs:96`:
```rust
road::draw_grid(&mut self.fb, w, h, horizon_y, world, seed);
```

No other callers.

## Constraints

- `draw_grid` receives `&WorldState` (read-only) — no state mutation needed
- `camera_z` is already public on WorldState
- Grid spacing (5.0) is a local constant — not configurable
- Loop iterates `1..40` — 39 potential lines, clamped by draw_distance
- `depth_scale < 0.02` filter already skips lines too close to horizon

## Interface Boundaries

The fix is entirely within `draw_grid`. No signature changes needed.
`WorldState` already exposes `camera_z` and `draw_distance()`.

## Assumptions

- `camera_z` increases monotonically during normal operation
- `grid_spacing` of 5.0 world units is appropriate for wrapping interval
- The modulo wrap `camera_z % grid_spacing` handles large `camera_z` values
  without precision issues (f32 modulo is fine for values up to millions)
