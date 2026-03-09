# T-005-01 Progress: fix-grid-line-scrolling

## Completed

### Step 1: Fix `draw_grid` grid line positioning
- Added `camera_offset = world.camera_z % grid_spacing` before loop
- Changed `z_world` to subtract `camera_offset`
- Added `z_world <= 0.0` guard to skip lines behind camera
- `cargo build` succeeds

### Step 2: Add test — grid lines move with camera
- Added `test_draw_grid_lines_move_with_camera`
- Renders grid at `camera_z=0.0` and `camera_z=2.5`, collects affected Y rows
- Asserts the Y row sets differ between the two camera positions
- Test passes

### Step 3: Add test — grid line count stays stable
- Added `test_draw_grid_line_count_stable`
- Renders grid at 6 different `camera_z` values
- Asserts max-min line count difference is ≤ 1
- Test passes

## Verification

- `cargo test` — all 166 tests pass (158 lib + 8 binary)
- `cargo clippy` — clean (one pre-existing warning in main.rs, not related)
- No deviations from plan
