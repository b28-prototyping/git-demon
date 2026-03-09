# T-003-01 Structure: Sprite Rendering

## Files Modified

### `src/renderer/sprites.rs`

**Change 1: Promote `project()` visibility**

```
- fn project(
+ pub(crate) fn project(
```

Single-line change. Enables direct unit testing of projection math from within the crate.

**Change 2: Add `#[cfg(test)] mod tests` block**

Append a test module at the bottom of `sprites.rs`. Contains:

- Test helper functions:
  - `test_seed() -> RepoSeed` — deterministic seed (same pattern as `road.rs` tests)
  - `test_world() -> WorldState` — default world state with known values

- Projection unit tests (test `project()` directly):
  - `test_project_behind_camera` — returns None when z_world < camera_z
  - `test_project_beyond_draw_distance` — returns None when too far
  - `test_project_valid_returns_some` — returns Some for in-range object
  - `test_project_depth_scale` — verify scale matches `1.0 - z_rel/draw_dist`
  - `test_project_screen_y_monotonic` — nearer objects have larger y
  - `test_project_lane_left_right` — Left.x < Center.x < Right.x
  - `test_project_curve_shifts_x` — positive curve → larger x

- Rendering tests (test `draw_sprites()` via framebuffer inspection):
  - `test_draw_sprites_empty` — no objects, no panic, buffer unchanged
  - `test_commit_billboard_color` — author_color pixels present
  - `test_commit_billboard_text_near` — text pixels at scale >= 0.35
  - `test_commit_billboard_text_suppressed_far` — no text when scale < 0.35
  - `test_addition_tower_height_scales` — more lines → taller sprite
  - `test_deletion_shard_crimson` — Rgba(180,30,30) pixels present
  - `test_tier_gate_neon_arch` — neon magenta pixels present
  - `test_velocity_sign_yellow` — Rgba(255,200,0) pixels present
  - `test_back_to_front_overdraw` — near object's pixels override far object

## Files NOT Modified

- `src/renderer/mod.rs` — no changes to render pipeline ordering
- `src/world/objects.rs` — no changes to object model
- `src/world/mod.rs` — no changes to spawn logic
- `src/renderer/font.rs` — no changes
- `src/renderer/road.rs` — no changes

## Module Boundaries

All new code is self-contained within `sprites.rs::tests`. No new public API. The only production code change is `fn project` → `pub(crate) fn project`.

## Ordering

1. Change `project()` visibility (1 line)
2. Add test helper functions
3. Add projection unit tests
4. Add rendering tests

Steps 2-4 are all within a single `#[cfg(test)] mod tests` block appended to the file. No ordering dependencies between individual tests.
