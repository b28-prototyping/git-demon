# Plan — T-001-04: terrain-silhouettes

## Step 1: Add test helpers to terrain.rs

Add `#[cfg(test)] mod tests` with two helper functions:

- `make_seed(terrain_roughness: f32) -> RepoSeed` — constructs a minimal seed
  with the given roughness and sensible defaults for other fields.
- `make_world(time: f32, seed: &RepoSeed) -> WorldState` — constructs a WorldState
  via `WorldState::new(seed)` then overrides `time`.

**Verify:** `cargo test --lib renderer::terrain` compiles (no tests yet, but module parses).

## Step 2: Add core behavior tests

Add these test functions:

1. `test_left_right_different_noise` — Create 200×100 fb, horizon=50, call draw_terrain.
   Collect height profiles for left (x=10) and right (x=170). Assert they differ.

2. `test_roughness_scales_height` — Call with roughness=0.1 and 1.0 on same fb size.
   Measure highest terrain pixel (lowest y with terrain color) for each. Assert
   high roughness produces taller terrain.

3. `test_silhouette_filled` — For a terrain column, find the topmost terrain pixel.
   Assert all pixels from that y to horizon_y have terrain color (no gaps).

4. `test_colors_match` — Verify left-side terrain pixels are TERRAIN_LEFT_COLOR
   and right-side terrain pixels are TERRAIN_RIGHT_COLOR.

**Verify:** `cargo test --lib renderer::terrain` — all 4 pass.

## Step 3: Add boundary and drift tests

5. `test_time_drift` — Call at time=0.0 and time=10.0, compare terrain pixels.
   Assert they differ (noise output changes with time).

6. `test_no_terrain_below_horizon` — Verify all pixels at y >= horizon_y are
   unchanged (still default black) after draw_terrain.

7. `test_terrain_boundaries` — Verify terrain pixels only appear in x < w/4
   (left) and x >= w*3/4 (right). The middle band should be untouched.

8. `test_zero_size_safe` — Call with horizon_y=0, verify no panic and no pixels written.

**Verify:** `cargo test --lib renderer::terrain` — all 8 pass.

## Step 4: Full verification

- `cargo test` — all project tests pass
- `cargo clippy` — no warnings
- `cargo build` — clean build

## Testing Strategy

All tests are unit tests within `terrain.rs` using `#[cfg(test)]`.
No integration tests needed — the function is a pure pixel-buffer writer
with no side effects. Tests construct small framebuffers (200×100 or similar)
to keep execution fast.
