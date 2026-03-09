# Progress — T-001-04: terrain-silhouettes

## Completed

### Step 1: Test helpers ✓
Added `#[cfg(test)] mod tests` with `make_seed()` and `make_world()` helpers.

### Step 2: Core behavior tests ✓
- `test_left_right_different_noise` — verifies different seeds produce different profiles
- `test_roughness_scales_height` — verifies roughness=1.0 produces taller terrain than 0.1
- `test_silhouette_filled` — verifies no gaps in filled columns
- `test_colors_match` — verifies left/right use correct colors (also allows tree colors)

### Step 3: Boundary and drift tests ✓
- `test_time_drift` — verifies terrain changes between time=0.0 and time=10.0
- `test_no_terrain_below_horizon` — verifies no terrain pixels at y >= horizon_y
- `test_terrain_boundaries` — verifies middle band has no terrain colors
- `test_zero_horizon_safe` — verifies no panic and no silhouette pixels with horizon_y=0

### Step 4: Full verification ✓
- `cargo test --lib renderer::terrain` — 8/8 pass
- No warnings from terrain test code
- Pre-existing failures in road.rs (4 tests) are unrelated

## Deviations from Plan

1. **test_zero_horizon_safe adjusted:** Original plan expected zero pixels written.
   The tree drawing code (`draw_trees`) can still place canopy pixels even with
   `horizon_y=0` since trees extend above the horizon. Test was adapted to check
   that no *terrain silhouette* colors appear, while allowing tree colors.

2. **Code evolved since research:** The `terrain.rs` file now includes tree drawing
   (`draw_trees`, `draw_rect`, `draw_diamond`) that was added by another ticket.
   Tests account for tree colors in pixel verification.

## Remaining
None — all planned steps complete.
