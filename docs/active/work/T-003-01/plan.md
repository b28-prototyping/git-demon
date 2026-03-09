# T-003-01 Plan: Sprite Rendering

## Step 1: Promote `project()` visibility

**File**: `src/renderer/sprites.rs:20`
**Change**: `fn project(` → `pub(crate) fn project(`
**Verify**: `cargo build` — no compilation errors

## Step 2: Add test helpers

**File**: `src/renderer/sprites.rs` (append `#[cfg(test)] mod tests` block)

Add:
- `use` imports for test dependencies (HashMap, ImageBuffer, Rgba, RepoSeed, WorldState, VelocityTier, Lane, RoadsideObject)
- `test_seed()` → RepoSeed with accent_hue=180, known author_colors
- `test_world()` → WorldState with camera_z=0, speed=1.0, curve_offset=0, Cruise tier

Pattern matches `road.rs` test helpers exactly.

**Verify**: `cargo test --lib renderer::sprites::tests` compiles (no tests yet)

## Step 3: Projection unit tests

Add 7 tests covering `project()`:

1. `test_project_behind_camera`: camera_z=50, z_world=40 → None
2. `test_project_beyond_draw_distance`: z_world=camera_z+300 (beyond 200 draw dist) → None
3. `test_project_valid_returns_some`: z_world=camera_z+50 → Some
4. `test_project_depth_scale`: z_world at camera_z+100 → scale = (1 - 100/200) = 0.5
5. `test_project_screen_y_monotonic`: two objects at different z → nearer has larger y
6. `test_project_lane_left_right`: same z, different lanes → Left.x < Center.x < Right.x
7. `test_project_curve_shifts_x`: curve_offset=50 → x shifts right vs curve_offset=0

**Verify**: `cargo test --lib renderer::sprites` — all 7 pass

## Step 4: Rendering tests

Add 9 tests covering `draw_sprites()` via framebuffer pixel inspection:

1. `test_draw_sprites_empty`: empty active_objects → blank buffer unchanged
2. `test_commit_billboard_color`: place CommitBillboard at close range → author_color pixels exist
3. `test_commit_billboard_text_near`: place at scale >= 0.35 → white (255,255,255) text pixels
4. `test_commit_billboard_text_suppressed_far`: place far away (scale < 0.35) → no white text pixels in billboard region
5. `test_addition_tower_height_scales`: tower with 400 lines vs 100 lines → more colored pixels for larger
6. `test_deletion_shard_crimson`: place DeletionShard → Rgba(180,30,30) pixels present
7. `test_tier_gate_neon_arch`: place TierGate close → Rgba(255,80,255) pixels present
8. `test_velocity_sign_yellow`: place VelocitySign close → Rgba(255,200,0) pixels present
9. `test_back_to_front_overdraw`: two overlapping objects, near one red, far one blue → red pixels at overlap position

**Verify**: `cargo test --lib renderer::sprites` — all 16 pass

## Step 5: Final verification

- `cargo test` — all tests in entire crate pass
- `cargo clippy` — no warnings
- `cargo build` — clean build

## Testing Strategy

All tests are `#[test]` unit tests in the `sprites.rs` module. No integration tests needed — the rendering pipeline is self-contained. Tests use small framebuffers (200x200 to 400x200) for speed.

Property assertions (not pixel-exact comparisons) ensure tests survive visual tweaks to colors/sizes. Examples:
- "author_color pixels exist" not "pixel at (102, 145) is Rgba(200,100,50)"
- "more lines → more colored pixels" not "exactly 847 green pixels"
