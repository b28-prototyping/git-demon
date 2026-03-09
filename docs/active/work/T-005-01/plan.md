# T-005-01 Plan: fix-grid-line-scrolling

## Steps

### Step 1: Fix `draw_grid` grid line positioning

File: `src/renderer/road.rs`, function `draw_grid`

1. Before the `for` loop (after `let grid_spacing = 5.0_f32;`), add:
   ```rust
   let camera_offset = world.camera_z % grid_spacing;
   ```

2. Change the `z_world` computation inside the loop:
   ```rust
   let z_world = i as f32 * grid_spacing - camera_offset;
   if z_world <= 0.0 { continue; }
   ```

Verification: `cargo build` succeeds, existing tests pass.

### Step 2: Add test — grid lines move with camera

Add `test_draw_grid_lines_move_with_camera` to `road.rs` tests:
- Render grid with `camera_z = 0.0`
- Collect set of Y coordinates that have grid-modified pixels
- Render grid with `camera_z = 2.5` (half grid_spacing)
- Collect set of Y coordinates again
- Assert the two sets differ

This directly verifies the acceptance criterion "grid lines scroll toward
the camera as camera_z advances."

### Step 3: Add test — grid line count stays stable

Add `test_draw_grid_line_count_stable` to `road.rs` tests:
- Render grid at several `camera_z` values (0, 1, 2.5, 4.9, 5.0, 10.3)
- For each, count the number of distinct Y rows with grid-modified pixels
- Assert all counts are within ±1 of each other

This verifies "lines do not accumulate — the number of visible grid lines
stays roughly constant."

### Verification

```bash
cargo test           # all tests pass
cargo clippy         # no warnings
```

## Testing Strategy

- **Unit tests**: Two new tests in `road.rs::tests` (steps 2-3)
- **Existing tests**: `test_draw_grid_accent_pixels` and
  `test_draw_grid_alpha_blended` continue to pass (grid still renders,
  still blends)
- **Visual verification**: Not automated, but the fix can be confirmed by
  running `cargo run -- --repo .` and observing grid line motion
- **Acceptance criteria coverage**:
  - "Grid lines scroll" → Step 2 test
  - "Lines wrap smoothly" → Step 3 test (count stability implies wrapping)
  - "Lines do not accumulate" → Step 3 test
  - "Grid lines follow curve" → existing `test_draw_grid_accent_pixels`
    (curve_offset=0 but curve_shift code path unchanged)
  - "Existing road.rs tests pass" → `cargo test`
