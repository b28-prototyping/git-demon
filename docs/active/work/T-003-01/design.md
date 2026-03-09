# T-003-01 Design: Sprite Rendering

## Current State

The sprite rasterizer is fully implemented in `sprites.rs` (commit `273c748`). The core projection, all 5 specified object types, text overlay with scale thresholds, back-to-front sorting, and lane positioning all match the acceptance criteria. What's missing: **test coverage**.

## Design Decision: Test Strategy

### Option A: Unit tests for `project()` only

Test the projection function in isolation. Cheap, fast, validates the core math.

Rejected: Doesn't validate rendering output. The acceptance criteria are about visual correctness (colored rects, text overlay, size scaling) — projection-only tests miss most of the surface area.

### Option B: Full framebuffer rendering tests (chosen)

Render sprites into a small framebuffer and assert pixel-level properties. Same pattern used successfully in `road.rs` tests — render to a buffer, inspect pixel colors, check spatial invariants.

Advantages:
- Validates the full pipeline: projection → sizing → rect drawing → text overlay
- Catches integration bugs (e.g. coordinate off-by-one, wrong color field)
- Matches existing test patterns in the codebase (road.rs has 10+ rendering tests)
- No new dependencies or test infrastructure needed

Trade-off: Tests are slightly slower than pure math tests, but a 200×200 framebuffer renders in microseconds. Acceptable.

### Option C: Snapshot/golden-image tests

Render full frames and compare against saved reference images.

Rejected: Brittle — any visual tweak breaks all snapshots. Not worth the maintenance cost for a screensaver. Property-based assertions are more resilient.

## Test Coverage Plan

### Projection tests

1. **Behind camera culling**: object at `z_world < camera_z` → `project()` returns `None`
2. **Beyond draw distance culling**: object far past draw distance → `None`
3. **Depth scale correctness**: object at known `z_rel` → verify `scale` matches formula
4. **Screen Y monotonicity**: nearer objects have larger `screen_y` (closer to bottom)
5. **Lane X positioning**: Left lane `x < center`, Right lane `x > center`
6. **Curve offset shifts X**: positive curve → sprites shift right, quadratic with depth
7. **VelocityDemon draw distance**: draw distance is 20% larger

### Rendering tests

8. **CommitBillboard renders colored rect**: author_color pixels present in expected region
9. **CommitBillboard text appears at scale ≥ 0.35**: font pixels present when close enough
10. **CommitBillboard text suppressed at scale < 0.35**: no font pixels when too far
11. **AdditionTower height proportional to sqrt(lines)**: taller tower for more lines
12. **DeletionShard is crimson**: Rgba(180,30,30) pixels present
13. **TierGate arch structure**: neon pixels at top bar + two side pillars
14. **TierGate text at close range**: tier name text visible when scale ≥ 0.35
15. **VelocitySign is yellow**: Rgba(255,200,0) pixels present
16. **Back-to-front sort**: near object's pixels overwrite far object when overlapping

### Edge cases

17. **Empty active_objects**: no panic, no pixels changed
18. **Single-pixel sprites**: very far objects degrade to tiny rects without panic
19. **Extreme curve offset**: no out-of-bounds with large curve values

## Design Decision: Making `project()` Testable

Currently `project()` is private (`fn project(...)`). Two options:

### Option A: Make `project()` `pub(crate)` (chosen)

Simple, minimal change, allows unit tests in same module to call it directly while also allowing integration tests in other modules if needed later. The function is already well-defined with clear inputs/outputs.

### Option B: Test only through `draw_sprites()`

Would work but makes projection-specific assertions harder — you'd need to reason about pixel positions backward from rendered output.

## Design Decision: `draw_rect()` Performance

`draw_rect()` uses `fb.put_pixel()` which does bounds checking per pixel. The road rasterizer uses raw buffer indexing for performance. Two options:

### Option A: Keep `put_pixel` (chosen for now)

Sprites are small relative to road scanlines. A 50×50 sprite = 2500 pixels. The road might cover 500×600 = 300,000 pixels. The performance impact of per-pixel bounds checking on sprites is negligible. If benchmarks show otherwise, switch to raw indexing later.

### Option B: Switch to raw buffer writes

Premature optimization. Not warranted by current profiling data.

## No Changes to Object Model

The `RoadsideObject` enum and `Lane` enum are well-structured and match the spec. `FilePosts`, `IdleAuthorTree`, and `SectorPylon` exist in the enum but use the gray rect fallback — this is fine; they can get specialized renderers in future tickets.

## Summary

The implementation is complete. The work for this ticket is adding comprehensive test coverage following the property-based assertion pattern established in `road.rs`. No architectural changes needed. `project()` gets promoted to `pub(crate)` for direct unit testing.
