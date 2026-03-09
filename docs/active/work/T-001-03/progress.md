# T-001-03 Progress: road-rasterizer-with-curvature

## Completed

### Step 1: Fix cx wrapping bug
- Changed `cx` from `u32` cast to staying as `f32` in `draw_road` scanline loop
- Changed `road_l` computation from `cx.saturating_sub(road_half as u32)` to `(cx - road_half).max(0.0) as u32`
- Prevents integer wrapping when `curve_offset` is large negative on narrow buffers

### Step 2: Test helpers
- Added `test_seed()` — constructs `RepoSeed` without git repo (accent_hue=180, all fields populated)
- Added `test_world(tier)` — constructs `WorldState` with specified tier and reasonable defaults

### Step 3: Pure function tests (8 tests)
- `test_lerp_at_zero`, `test_lerp_at_one`, `test_lerp_midpoint`
- `test_horizon_ratio_normal`, `test_horizon_ratio_velocity_demon`
- `test_road_max_half_normal`, `test_road_max_half_velocity_demon`
- `test_blend_alpha_opaque`, `test_blend_alpha_transparent`, `test_blend_alpha_half`
- `test_hsl_to_rgb_red`, `test_hsl_to_rgb_green`
- `test_hue_to_neon_returns_opaque`

### Step 4: Rendering invariant tests (9 tests)
- `test_draw_road_perspective_width` — road wider at bottom than horizon
- `test_draw_road_stripe_colors_present` — both stripe colors appear
- `test_draw_road_rumble_colors_present` — both rumble colors appear
- `test_draw_road_verge_colors_present` — both verge colors appear
- `test_draw_road_curve_shifts_center` — positive curve shifts road right
- `test_draw_road_velocity_demon_wider` — VelocityDemon produces wider road
- `test_draw_road_no_panic_extreme_curve` — ±80 curve on 100×100 buffer doesn't panic
- `test_draw_grid_accent_pixels` — grid modifies pixels on road surface
- `test_draw_grid_alpha_blended` — grid pixels are blended, not pure accent

### Step 5: Cleanup
- Moved `blend_alpha` under `#[cfg(test)]` since `draw_grid` was rewritten to use raw buffer access and no longer calls it
- `draw_road` also uses raw buffer access (hoisted `stride`, `row_offset`, `rumble_l` out of inner loop)

### Step 6: Final verification
- `cargo test` — 44 tests pass (22 new road tests + 22 existing)
- `cargo build` — clean
- `cargo clippy` — no new warnings from road.rs (pre-existing warnings in other files only)

## Deviations from Plan

1. Used wider test buffers (1200px) for curve/width tests because ROAD_MAX_HALF (480) exceeds half the original test buffer widths, causing road to fill entire bottom row.
2. Used `0.01` epsilon instead of `1e-6` for `road_max_half` VelocityDemon test due to f32 precision with `1.05` multiplier.
3. Grid accent test changed from closeness-to-accent-color check to before/after pixel comparison, which is more robust to varying blend results.
4. `draw_road` and `draw_grid` were optimized with raw buffer access (avoiding per-pixel `put_pixel` overhead) — this was done by a concurrent session and retained.

## Remaining
None — all steps complete.
