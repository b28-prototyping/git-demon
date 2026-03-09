# T-003-01 Progress: Sprite Rendering

## Completed

### Step 1: `project()` visibility
- Already `pub(crate)` (applied by prior edit or linter)
- Verified: `cargo build` clean

### Step 2: Test helpers
- Added `#[cfg(test)] mod tests` block to `sprites.rs`
- `test_seed()`, `test_world()`, `horizon_y()` helpers following `road.rs` pattern
- `BLACK` constant for blank framebuffer initialization

### Step 3: Projection unit tests (7 tests)
- `test_project_behind_camera` ✓
- `test_project_beyond_draw_distance` ✓
- `test_project_valid_returns_some` ✓
- `test_project_depth_scale` ✓
- `test_project_screen_y_monotonic` ✓
- `test_project_lane_left_right` ✓
- `test_project_curve_shifts_x` ✓

### Step 4: Rendering tests (9 tests)
- `test_draw_sprites_empty` ✓
- `test_commit_billboard_color` ✓
- `test_commit_billboard_text_near` ✓
- `test_commit_billboard_text_suppressed_far` ✓
- `test_addition_tower_height_scales` ✓
- `test_deletion_shard_crimson` ✓
- `test_tier_gate_neon_arch` ✓
- `test_velocity_sign_yellow` ✓
- `test_back_to_front_overdraw` ✓

### Step 5: Final verification
- `cargo test --lib renderer::sprites` — 16/16 passed ✓
- `cargo clippy` — clean ✓
- `cargo build` — clean ✓

## Deviations from Plan

- Rendering tests use 1200×400 framebuffers instead of 400×200. At close range, road_max_half (480px) pushes lane sprites off a 400px-wide buffer. Center lane used for most tests to avoid edge positioning issues.
- Full test suite: 110/110 tests pass (no pre-existing failures).

## Remaining
None — all steps complete.
