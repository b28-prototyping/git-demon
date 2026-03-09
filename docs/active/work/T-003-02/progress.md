# T-003-02 Progress: HUD Overlay

## Completed Steps

### Step 1: Fix HUD background alpha
- Changed `HUD_BG` alpha from 200 to 204 (exact 80% of 255)
- File: `src/renderer/hud.rs:9`

### Step 2: Add test helpers
- Added `#[cfg(test)] mod tests` block with `make_world()` and `make_seed()` helpers
- Pattern matches existing test helper style in `effects.rs` and `sprites.rs`

### Step 3: Add tier badge color tests
- 5 tests covering all tiers: flatline, cruise, active, demon, velocity_demon_strobe
- Strobe test verifies red/white alternation at different time values

### Step 4: Add HUD rendering tests
- `test_hud_background_alpha_blend`: Verifies compositing math over white bg
- `test_hud_modifies_bottom_strip`: Confirms pixels change in bottom 18 rows
- `test_hud_does_not_modify_above_strip`: Confirms no leakage above HUD
- `test_sector_calculation`: 450 commits → sector 4
- `test_sector_zero`: 50 commits → sector 0
- `test_repo_name_right_aligned`: Rightmost white pixel near right edge
- `test_hud_height_is_18`: Constant check

### Step 5: Full verification
- `cargo test --lib`: **136 tests pass** (12 new HUD tests)
- Note: `cargo clippy --lib` and `cargo test` (full, including binary) fail due to
  pre-existing unstaged changes from another ticket (T-004-01) that partially
  refactored `FrameRenderer::set_timing()` and `draw_dev_overlay()` signatures.
  These failures are not related to T-003-02 changes.

## Deviations from Plan

- Changed `tier_badge_color` visibility from `fn` to `pub(crate) fn` to enable
  direct testing from within the `tests` submodule. This is the minimal visibility
  change needed.

## Remaining

None. All planned work is complete.
