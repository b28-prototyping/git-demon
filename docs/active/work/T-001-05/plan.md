# Plan — T-001-05: effects-passes

## Step 1: Read WorldState and RepoSeed constructors
Understand how to create minimal test instances of WorldState and RepoSeed for use in test helpers.

## Step 2: Write test helper functions
Create `make_world()` and `make_seed()` helpers that produce valid instances with controllable parameters (speed, cpm, tier).

## Step 3: Write helper function tests
Test `lerp`, `luminance_fast`, `blend_alpha`, `hue_to_rgb` with known input/output pairs.

**Verification**: `cargo test --lib renderer::effects`

## Step 4: Write scanline filter tests
- Create a 4×4 buffer with known pixel values
- Apply `apply_scanline_filter()`
- Assert even rows darkened by ~20%, odd rows unchanged

**Verification**: `cargo test --lib renderer::effects`

## Step 5: Write motion blur tests
- Create two 4×4 buffers (current and previous) with distinct known values
- Create WorldState with speed=0 and speed=12
- Apply `apply_motion_blur()`
- Verify blending matches `lerp(0.15, 0.35, speed/12.0)`
- Verify alpha channel bytes are not modified

**Verification**: `cargo test --lib renderer::effects`

## Step 6: Write bloom tests
- Create a small buffer with one bright white pixel (luminance > 0.72)
- Apply `apply_bloom()`
- Verify neighboring pixels received additive glow
- Test clamping: set neighbors to 250, verify they don't exceed 255
- Create a buffer with only dark pixels, verify no change

**Verification**: `cargo test --lib renderer::effects`

## Step 7: Write speed lines tests
- Create a 32×32 buffer, WorldState with cpm=2.0 (gives N=16 lines), tier=Demon
- Apply `draw_speed_lines()`
- Verify at least some pixels along radial lines from center changed
- Test with cpm=0.0, verify no pixels changed

**Verification**: `cargo test --lib renderer::effects`

## Step 8: Run full test suite and clippy
- `cargo test` — all tests pass
- `cargo clippy` — no warnings

## Testing Strategy
- All tests are unit tests in `src/renderer/effects.rs`
- Small buffers (4×4 to 32×32) for speed
- Tests verify pixel-level numerical correctness
- No integration tests needed — effect ordering is structural and verified by code inspection
