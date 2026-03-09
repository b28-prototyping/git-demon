# T-003-02 Plan: HUD Overlay

## Steps

### Step 1: Fix HUD background alpha
- Change `HUD_BG` in `src/renderer/hud.rs` from `Rgba([0, 0, 0, 200])` to `Rgba([0, 0, 0, 204])`
- Verify: `cargo build` compiles cleanly

### Step 2: Add test helpers
- Add `#[cfg(test)] mod tests` block to `src/renderer/hud.rs`
- Add helper functions: `make_world()`, `make_seed()`, `make_fb()`
- Pattern matches existing test helpers in `effects.rs` and `sprites.rs`

### Step 3: Add tier badge color tests
- Test each of the 5 tiers returns the correct color
- Test VelocityDemon strobe by varying `world.time`
- Verify: `cargo test --lib renderer::hud`

### Step 4: Add HUD rendering tests
- Test that `draw_hud()` modifies pixels in the bottom 18 rows
- Test alpha compositing produces correct blend values
- Test repo name appears near right edge of framebuffer
- Verify: `cargo test --lib renderer::hud`

### Step 5: Full verification
- `cargo test` — all tests pass
- `cargo clippy` — no new warnings
- `cargo build` — clean build

## Testing Strategy

All tests are unit tests within `hud.rs::tests`. No integration tests needed —
the HUD is a pure function of (framebuffer, world, seed) with no side effects.

Test framebuffers use small sizes (e.g., 600x200) to keep tests fast while being
wide enough to avoid field overlap issues.

## Commit Plan

Single commit containing:
1. Alpha fix (200 → 204)
2. All new tests
