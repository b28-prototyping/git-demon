# T-005-01 Structure: fix-grid-line-scrolling

## Files Modified

### `src/renderer/road.rs`

**Function: `draw_grid`** (lines 108-112)

Current:
```rust
let grid_spacing = 5.0_f32;
for i in 1..40 {
    let z_world = i as f32 * grid_spacing;
```

Modified:
```rust
let grid_spacing = 5.0_f32;
let camera_offset = world.camera_z % grid_spacing;
for i in 1..40 {
    let z_world = i as f32 * grid_spacing - camera_offset;
    if z_world <= 0.0 { continue; }
```

Changes:
1. Add `camera_offset` computation before the loop
2. Subtract `camera_offset` from each `z_world`
3. Add early-continue guard for `z_world <= 0.0`

No signature changes. No new public API. No new constants.

**Test additions** (after existing tests, ~line 522):

Add two new tests:
1. `test_draw_grid_lines_move_with_camera` — render grid at two different
   `camera_z` values, verify grid line screen positions differ
2. `test_draw_grid_line_count_stable` — render at multiple `camera_z` values,
   verify the number of grid-affected rows stays approximately constant

## Files NOT Modified

- `src/world/mod.rs` — no changes needed, `camera_z` already exists and is public
- `src/renderer/mod.rs` — call site unchanged, same signature
- No new files created

## Module Boundaries

The fix is entirely internal to `draw_grid`. The function signature and its
contract with `FrameRenderer::render` are unchanged. The only new data dependency
is reading `world.camera_z`, which is already passed via `&WorldState`.

## Ordering

Single atomic change — no sequencing concerns.
