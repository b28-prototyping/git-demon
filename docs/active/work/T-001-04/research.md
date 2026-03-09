# Research — T-001-04: terrain-silhouettes

## Existing Implementation

`src/renderer/terrain.rs` already contains a complete `draw_terrain` function (58 lines).
The module is wired into the render pipeline at step 3-4 in `renderer/mod.rs:65`.

### Current Code Analysis

**Function signature:**
```rust
pub fn draw_terrain(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32, _h: u32, horizon_y: u32,
    seed: &RepoSeed, world: &WorldState,
)
```

**Constants:**
- `TERRAIN_FREQ: f64 = 0.008` — spatial frequency for noise sampling
- `TERRAIN_DRIFT: f64 = 0.3` — temporal drift speed
- `LEFT_SEED: u32 = 42` — noise seed for left terrain
- `RIGHT_SEED: u32 = 137` — noise seed for right terrain
- `TERRAIN_LEFT_COLOR: Rgba([8, 12, 20, 255])` — cool dark blue-black
- `TERRAIN_RIGHT_COLOR: Rgba([12, 14, 22, 255])` — slightly warmer dark

**Left terrain:** Iterates `x in 0..w/4`, samples `OpenSimplex` at `(x * FREQ, time * DRIFT)`,
fills from computed top down to `horizon_y` with `TERRAIN_LEFT_COLOR`.

**Right terrain:** Iterates `x in w*3/4..w`, samples different `OpenSimplex` instance
(seed 137) at `(x * FREQ, time * DRIFT + 100.0)`, fills with `TERRAIN_RIGHT_COLOR`.

**Max height:** `(horizon_y * 0.6 * seed.terrain_roughness) as u32`

## Acceptance Criteria Mapping

| Criterion | Status | Evidence |
|-----------|--------|----------|
| draw_terrain renders L/R silhouettes above horizon | Implemented | Lines 29-41 (left), 44-57 (right) |
| Height from OpenSimplex at (x*FREQ, time*DRIFT) | Implemented | Lines 30-33, 46-49 |
| L/R use different noise seeds | Implemented | LEFT_SEED=42, RIGHT_SEED=137; also +100.0 offset on right y |
| Max height scales with terrain_roughness | Implemented | Line 25: `horizon_y * 0.6 * seed.terrain_roughness` |
| Left=cool shadow, right=warmer tint | Implemented | LEFT=[8,12,20], RIGHT=[12,14,22] |
| Silhouettes are filled (top to horizon) | Implemented | `for y in top..horizon_y` loops |
| Terrain drifts via TERRAIN_DRIFT | Implemented | `world.time * TERRAIN_DRIFT` in noise y-coord |

All seven acceptance criteria are satisfied by existing code.

## Dependencies

- `noise` crate v0.9 (`OpenSimplex`, `NoiseFn` trait) — in Cargo.toml
- `RepoSeed.terrain_roughness: f32` — computed in `git/seed.rs:69-73`, clamped [0.1, 1.0]
- `WorldState.time: f32` — accumulated in `world/mod.rs:56`, incremented by `dt` each frame

## Integration Points

- Called from `renderer/mod.rs:65` after sky/sun/bloom-bleed, before road
- Draw order is correct: terrain overwrites sky pixels, road overwrites terrain below horizon
- `horizon_y` computed in `renderer/mod.rs:56` as `h * horizon_ratio(world)`
- Road edges at horizon are at `w/4` and `w*3/4` — matches terrain boundaries

## Gaps Identified

1. **No tests.** `terrain.rs` has zero `#[cfg(test)]` module. Every other renderer pass
   (sky.rs) has unit tests. This module needs parity.
2. **Bounds safety relies on runtime checks.** The inner loop checks `x < fb.width() && y < fb.height()`
   each iteration. These guards are correct but the `x` check is redundant for left terrain
   (always `x < w/4 <= w = fb.width()`) and right terrain (`x < w = fb.width()`). Only the `y`
   check is genuinely needed (when `horizon_y >= fb.height()`).
3. **`_h` parameter is unused.** The function accepts `h` but prefixes it with `_`. Horizon_y
   already encodes the needed height information. This is fine — it keeps the signature
   consistent with other draw functions.

## Patterns Observed

- All renderer passes follow the same signature pattern: `(fb, w, h, horizon_y, seed, world)`
- Color constants are defined as module-level `const Rgba<u8>` values
- Noise is instantiated per-call (not cached). For terrain this is fine — `OpenSimplex::new()`
  is trivial (no table precomputation in the noise crate's implementation).
- Other renderer modules (sky.rs) have comprehensive tests covering color math, blending,
  and boundary conditions.
