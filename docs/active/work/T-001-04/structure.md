# Structure — T-001-04: terrain-silhouettes

## Files Modified

### `src/renderer/terrain.rs`

**Change:** Add `#[cfg(test)] mod tests` block at end of file.

**No changes to existing code.** The implementation is complete and correct.

**New code structure:**

```
// ... existing code (lines 1-58) unchanged ...

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::seed::RepoSeed;
    use crate::world::WorldState;

    // Helper: create minimal RepoSeed with configurable roughness
    fn make_seed(terrain_roughness: f32) -> RepoSeed { ... }

    // Helper: create WorldState at a given time
    fn make_world(time: f32, seed: &RepoSeed) -> WorldState { ... }

    // 8 test functions (see Design for full list)
}
```

## No Files Created or Deleted

This is a test-only change to an existing file.

## Module Boundaries

- Tests are internal to `terrain.rs` — they use `super::*` to access private constants
  (`TERRAIN_LEFT_COLOR`, `TERRAIN_RIGHT_COLOR`, `TERRAIN_FREQ`, etc.)
- Test helpers construct `RepoSeed` and `WorldState` directly using struct literals.
  Both types have `pub` fields, so this works without needing builder methods.
- `WorldState::new(seed)` is the canonical constructor; tests will use it and then
  override `time` as needed for drift tests.

## Dependencies

- No new crate dependencies required.
- `image::ImageBuffer` and `image::Rgba` are already in scope.
- `noise` crate is already a dependency.

## Interface Stability

No public interfaces change. `draw_terrain` signature remains:
```rust
pub fn draw_terrain(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32, _h: u32, horizon_y: u32,
    seed: &RepoSeed, world: &WorldState,
)
```
