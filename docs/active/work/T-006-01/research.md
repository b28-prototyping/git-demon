# T-006-01 Research: commit-cars-on-road

## Objective

Replace `CommitBillboard` roadside objects with `CommitCar` objects that render ON the road surface, so commits appear as small colored racecars the player overtakes.

## Current State

### Object Model (`src/world/objects.rs`)

- `RoadsideObject` enum has 8 variants: `CommitBillboard`, `AdditionTower`, `DeletionShard`, `FilePosts`, `VelocitySign`, `TierGate`, `IdleAuthorTree`, `SectorPylon`.
- `CommitBillboard` carries `message: String`, `author: String`, `author_color: Rgba<u8>`.
- `Lane` enum has 3 variants: `Left`, `Right`, `Center`. These control X-placement via the projection system.
- `ingest_poll_to_queue()` creates `CommitBillboard` for every commit. It also creates `AdditionTower` (>50 lines added), `DeletionShard` (>50 lines deleted), and `VelocitySign` per poll.

### Lane & Spawn System (`src/world/mod.rs`)

- `WorldState.active_objects: Vec<(Lane, f32, RoadsideObject)>` — lane, z_world, object.
- Spawn logic in `update()` pops from `pending_objects`, assigns `Center` for `TierGate`, otherwise alternates `Left`/`Right`.
- Lane alternation uses a simple boolean toggle.
- Despawn: objects behind `camera_z - 5.0` are removed.

### Projection (`src/renderer/sprites.rs`)

- `project()` maps `(z_world, lane)` to `SpriteScreenPos { x, y, scale }`.
- `depth_scale = (1.0 - z_rel / draw_dist).clamp(0.0, 1.0)` — 0 at draw distance, 1 at camera.
- `screen_y = lerp(horizon_y, pixel_h, depth_scale)`.
- Lane X offsets: `Left` at `cx - road_half * 1.15`, `Right` at `cx + road_half * 1.15`, `Center` at `cx`.
- The 1.15 factor places Left/Right on the verge (outside the road edge).
- `cx = pixel_w/2 + curve_offset * depth_scale^2` — curvature is baked into all sprite positions.

### Sprite Rendering (`src/renderer/sprites.rs`)

- `draw_sprites()` sorts by z descending (back-to-front), iterates and matches on variant.
- `CommitBillboard` renders as a colored rectangle (BILLBOARD_BASE_W=80, BILLBOARD_BASE_H=40, quadratic scale), with text overlay when `scale >= 0.35`.
- Each variant has its own inline drawing logic in the match arm.
- `draw_rect()` helper does bounds-checked pixel writes.

### Player Car (`src/renderer/effects.rs`)

- `draw_car()` renders at screen bottom center, with `steer_angle * 0.15` lateral shift.
- Full wedge shape: shadow ellipse, trapezoid body, nose triangle, windshield, cockpit slit.
- Dimensions: `car_w=60`, `car_h=30`, `nose_h=12` pixels (screen space, not perspective-scaled).
- Drawn at step 7.5 in the render pipeline — AFTER `draw_sprites()` (step 7), so the player car always occludes sprites.

### Draw Order (`src/renderer/mod.rs`)

```
7.  sprites::draw_sprites()   — all roadside objects, back-to-front
7.5 effects::draw_car()       — player car on top
```

This means commit cars on the road will be drawn by `draw_sprites()` and the player car will overdraw them correctly.

### Road Geometry (`src/renderer/road.rs`)

- Road half-width: `lerp(ROAD_MIN_HALF=8, ROAD_MAX_HALF=480, depth)`.
- Rumble strip is 12px outside the road edge.
- Verge begins after the rumble strip.
- Current Left/Right lanes at `1.15 * road_half` are outside the road+rumble boundary.
- For on-road lanes, `0.35 * road_half` would place cars about 1/3 from center.

### Tests

- `src/world/mod.rs` tests: `test_ingest_poll_creates_billboard` checks for `CommitBillboard` variant.
- `src/renderer/sprites.rs` tests: `test_commit_billboard_color`, `test_commit_billboard_text_near`, `test_commit_billboard_text_suppressed_far` test CommitBillboard rendering.
- Benchmark (`benches/render.rs`): uses `CommitBillboard` in `bench_world()`.

### Dependencies

- T-003-01 (dependency): already done based on current codebase state.

## Constraints

1. `CommitBillboard` is referenced in: objects.rs (enum + ingest), sprites.rs (draw logic + tests), mod.rs tests, bench.
2. The enum is `#[derive(Debug, Clone)]` — adding a variant is straightforward.
3. `Lane` is `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` — adding variants is straightforward.
4. The projection function handles all lane types via a match — new lanes need new arms.
5. The car drawing code in effects.rs is ~100 lines of inline pixel logic. Commit cars need a simpler version.
6. Text rendering threshold is `scale >= 0.35` for billboards; ticket specifies `scale >= 0.5` for commit cars.

## Open Questions

1. Should `CommitBillboard` be removed entirely, or kept alongside `CommitCar`? Ticket says "replaces" — remove it.
2. The ticket says `0.35 * road_half` for on-road lanes. With `road_half` ranging 8..480, this gives 2.8..168px offset from center. At the near camera this is reasonable.
3. Should commit cars have a speed difference from the player (slower, so they're overtaken)? Current objects are stationary (fixed z_world). The illusion of overtaking comes from the player moving forward while objects don't. This is sufficient.

## Files to Modify

| File | Change |
|------|--------|
| `src/world/objects.rs` | Add `CommitCar` variant, add `RoadLeft`/`RoadRight` to `Lane`, change `ingest_poll_to_queue` |
| `src/world/mod.rs` | Update spawn logic to assign `CommitCar` to road lanes |
| `src/renderer/sprites.rs` | Add `RoadLeft`/`RoadRight` projection, add `CommitCar` draw logic |
| `benches/render.rs` | Update `CommitBillboard` references to `CommitCar` |
