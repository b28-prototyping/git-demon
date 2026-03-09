# T-006-01 Structure: commit-cars-on-road

## File Changes

### 1. `src/world/objects.rs` — Modify

**Lane enum** — Add two variants:

```rust
pub enum Lane {
    Left,
    Right,
    Center,
    RoadLeft,   // NEW
    RoadRight,  // NEW
}
```

**RoadsideObject enum** — Rename variant:

```
CommitBillboard { message, author, author_color }
→ CommitCar { message, author, author_color }
```

Same fields, different name. No structural change to the enum.

**`ingest_poll_to_queue()`** — Change `CommitBillboard` to `CommitCar`:

```rust
queue.push_back(RoadsideObject::CommitCar { ... });
```

### 2. `src/world/mod.rs` — Modify

**Spawn logic in `update()`** — Change lane assignment for `CommitCar`:

Current logic:
```rust
let lane = if TierGate { Center } else if toggle { Right } else { Left };
```

New logic:
```rust
let lane = if TierGate {
    Center
} else if CommitCar {
    if toggle { RoadRight } else { RoadLeft }
} else {
    if toggle { Right } else { Left }
};
```

**Tests** — Update `test_ingest_poll_creates_billboard`:
- Rename to `test_ingest_poll_creates_commit_car`
- Match on `CommitCar` instead of `CommitBillboard`

**New test** — `test_lane_assignment_commit_car`:
- Push a `CommitCar` and a `VelocitySign` to pending
- After update, verify CommitCar got `RoadLeft` or `RoadRight`
- Verify VelocitySign got `Left` or `Right`

### 3. `src/renderer/sprites.rs` — Modify

**Constants** — Add commit car base dimensions:

```rust
const COMMIT_CAR_BASE_W: f32 = 40.0;
const COMMIT_CAR_BASE_H: f32 = 20.0;
```

**`project()` lane_x match** — Add two arms:

```rust
Lane::RoadLeft  => (cx - road_half_here * 0.35).max(0.0) as u32,
Lane::RoadRight => (cx + road_half_here * 0.35) as u32,
```

**`draw_sprites()` match** — Replace `CommitBillboard` arm with `CommitCar` arm:

LOD-based rendering:
- `car_w < 4`: colored dot (2x2 rect)
- `car_w < 8`: colored rect
- `car_w >= 8`: wedge shape with nose, body (author_color), dark sides

Text label when `scale >= 0.5`.

**New helper** — `draw_commit_car()`:

```rust
fn draw_commit_car(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    cx: u32, bottom_y: u32,
    car_w: u32, car_h: u32,
    author_color: Rgba<u8>,
)
```

Draws the wedge shape. Extracted to keep the match arm clean.

**Helper** — `darken()` and `brighten()` for color variants:

```rust
fn darken(c: Rgba<u8>, factor: f32) -> Rgba<u8>
fn brighten(c: Rgba<u8>, factor: f32) -> Rgba<u8>
```

**Tests** — Rename and update:
- `test_commit_billboard_color` → `test_commit_car_color` (use `CommitCar`, `RoadLeft`)
- `test_commit_billboard_text_near` → `test_commit_car_text_near` (threshold 0.5)
- `test_commit_billboard_text_suppressed_far` → `test_commit_car_text_suppressed_far`

**New tests:**
- `test_project_road_left_right`: RoadLeft/RoadRight are between Center and Left/Right
- `test_commit_car_lod_dot`: far car (small scale) produces colored pixels

### 4. `benches/render.rs` — Modify

Replace `CommitBillboard` with `CommitCar` in bench objects. Change lanes from `Left`/`Right` to `RoadLeft`/`RoadRight` for commit entries.

## Files NOT Changed

- `src/renderer/mod.rs` — Draw order unchanged
- `src/renderer/effects.rs` — Player car unchanged
- `src/renderer/road.rs` — Road geometry unchanged
- `src/renderer/hud.rs` — No HUD changes
- `src/world/speed.rs` — No speed changes
- `src/git/` — No git changes

## Module Boundaries

- `objects.rs` exports `Lane`, `RoadsideObject`, `ingest_poll_to_queue` — public interface unchanged except variant/enum renames
- `sprites.rs` internal helper `draw_commit_car` is private (not pub)
- No new public API surfaces

## Ordering

1. objects.rs changes first (enum defines, all downstream refs break)
2. world/mod.rs next (spawn logic + tests)
3. sprites.rs next (projection + rendering + tests)
4. benches/render.rs last (uses public API, should compile after the above)
