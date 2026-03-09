# Plan: T-007-03 parallax-depth-layers

## Step 1: Add `Camera::pitch_offset()` method

File: `src/world/camera.rs`

1. Add `const PITCH_SENSITIVITY: f32 = 0.5;` near the top of the file
2. Add method to Camera impl:
   ```rust
   pub fn pitch_offset(&self, parallax_factor: f32, screen_h: u32) -> f32 {
       self.pitch * (1.0 - parallax_factor) * screen_h as f32 * PITCH_SENSITIVITY
   }
   ```
3. Add unit tests: zero pitch returns 0.0, nonzero pitch scales correctly, parallax=1.0 always returns 0.0

Verify: `cargo test -p git-demon camera::tests`

## Step 2: Apply parallax factors to terrain layers

File: `src/renderer/terrain.rs`

1. Add constants:
   ```rust
   const ISLAND_PARALLAX: f32 = 0.4;
   const CLOUD_PARALLAX: f32 = 0.15;
   ```
2. In `draw_islands()`, change the z_rel computation:
   - Before: `let z_rel = ((base_z - (world.camera_z % z_period)) + z_period) % z_period;`
   - After: `let parallax_z = world.camera_z * ISLAND_PARALLAX;` then `let z_rel = ((base_z - (parallax_z % z_period)) + z_period) % z_period;`
   - Keep `z_world = world.camera_z + z_rel` — the projection still uses the real camera position
   - Actually, z_world is used for cam.project() which computes z_rel from camera.z. We need the projected position to reflect the parallax scroll. So: `let z_world = world.camera.z + z_rel;` (unchanged — z_rel is now based on parallax_z so it cycles slower)
3. In `draw_clouds()`, same change with `CLOUD_PARALLAX`
4. Add pitch offset to screen_y in both functions (when pitch != 0)

Verify: `cargo test -p git-demon terrain::tests`

## Step 3: Add pitch shifts to sky layers

File: `src/renderer/sky.rs`

1. In `draw_stars()`: Add pitch offset to convergence point cy:
   ```rust
   let pitch_shift = world.camera.pitch_offset(0.0, horizon_y);
   let cy = horizon_y as f32 * 0.6 + pitch_shift;
   ```
2. In `draw_sun()`: Add pitch offset with parallax 0.05:
   ```rust
   let pitch_shift = world.camera.pitch_offset(0.05, horizon_y);
   let cy = horizon_y as i32 - 40 + pitch_shift as i32;
   ```
3. In `draw_bloom_bleed()`: Add pitch offset with same factor as sun

Verify: `cargo test -p git-demon sky::tests`

## Step 4: Add parallax-specific tests

File: `src/renderer/terrain.rs` (test module)

1. `test_islands_parallax_rate`: Render at camera_z=0 and camera_z=1000. Then render at camera_z=0 and camera_z=2500 (=1000/0.4). Compare pixel changes to verify the 0.4 factor.
2. `test_clouds_scroll_slower_than_islands`: Render terrain at two camera_z values. Measure that cloud pixels change less than island pixels.

File: `src/renderer/sky.rs` (test module)

3. `test_stars_shift_with_pitch`: Set camera.pitch to 0.1, verify star rendering differs from pitch=0.

Verify: `cargo test`

## Step 5: Full build + test + clippy

1. `cargo build` — verify compilation
2. `cargo test` — all tests pass
3. `cargo clippy` — no warnings

## Verification Criteria

- [ ] `Camera::pitch_offset()` returns 0.0 when pitch is 0.0
- [ ] Islands scroll at 40% of camera_z rate (test)
- [ ] Clouds scroll at 15% of camera_z rate (test)
- [ ] Stars don't change with camera_z alone (already true, verified)
- [ ] All existing tests pass
- [ ] No new warnings from clippy
- [ ] No new allocations or draw calls (same render pipeline)
