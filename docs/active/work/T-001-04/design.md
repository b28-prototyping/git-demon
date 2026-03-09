# Design — T-001-04: terrain-silhouettes

## Situation

The terrain silhouette implementation already exists in `src/renderer/terrain.rs` and
satisfies all seven acceptance criteria. The remaining gap is test coverage — the module
has zero tests while peer modules (sky.rs) have comprehensive test suites.

## Approach Options

### Option A: Tests Only

Add a `#[cfg(test)]` module to `terrain.rs` with unit tests that verify:
- Left and right terrain produce different noise patterns (different seeds)
- Terrain height scales with `terrain_roughness`
- Silhouettes fill from top to horizon (no gaps)
- Colors match spec (left=cool, right=warmer)
- Boundary conditions (zero roughness, max roughness, zero horizon)
- Time drift produces changing output

No code changes to the implementation.

### Option B: Tests + Minor Cleanup

Same tests as Option A, plus:
- Remove redundant `x < fb.width()` bounds checks (provably unnecessary)
- Remove unused `_h` parameter

### Option C: Tests + Performance Optimization

Same as Option A, plus cache `OpenSimplex` instances across frames by moving them
into `FrameRenderer`. This would require changing `draw_terrain` signature and
adding fields to `FrameRenderer`.

## Decision: Option A — Tests Only

**Rationale:**
- The implementation is correct and complete. All acceptance criteria are met.
- The redundant bounds checks in Option B are harmless and provide defense-in-depth.
  Removing them saves negligible CPU and risks introducing a bug if the road-edge
  calculation ever changes.
- The `_h` parameter maintains signature consistency with other draw functions. Removing
  it would make `draw_terrain` the odd one out.
- Option C's caching is premature optimization — `OpenSimplex::new()` is trivial and
  terrain is not a hot path compared to road scanlines.
- The project needs test parity across renderer modules. sky.rs has 8 tests; terrain.rs
  has 0. This is the primary deliverable.

**Rejected:**
- Option B: Cleanup risk outweighs negligible benefit.
- Option C: Over-engineering for no measurable gain.

## Test Design

Tests will use a helper function to create a small framebuffer and seed/world fixtures,
then call `draw_terrain` and inspect the pixel buffer.

### Test Cases

1. **`test_left_right_different_noise`** — Run draw_terrain, verify left and right
   silhouettes have different height profiles (not identical column-by-column).

2. **`test_roughness_scales_height`** — Call with roughness=0.1 and roughness=1.0,
   verify max terrain height is proportionally different.

3. **`test_silhouette_filled`** — For each terrain column, verify all pixels from
   the terrain top to horizon_y are filled (non-black/non-sky-color).

4. **`test_colors_match`** — Verify left terrain pixels are TERRAIN_LEFT_COLOR and
   right terrain pixels are TERRAIN_RIGHT_COLOR.

5. **`test_time_drift`** — Call at time=0.0 and time=10.0, verify output differs.

6. **`test_zero_roughness`** — With terrain_roughness at minimum (0.1), verify
   terrain still renders but stays very low.

7. **`test_no_terrain_below_horizon`** — Verify no terrain pixels are written
   at y >= horizon_y.

8. **`test_terrain_boundaries`** — Verify left terrain stays in [0, w/4) and
   right terrain stays in [w*3/4, w).
