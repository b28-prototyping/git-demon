# T-006-01 Design: commit-cars-on-road

## Decision: Replace CommitBillboard with CommitCar

### Approach A: Add CommitCar alongside CommitBillboard

Keep both variants. `ingest_poll_to_queue` emits `CommitCar` instead of `CommitBillboard`. Old code still compiles but the variant is dead.

**Pros:** No breakage risk, gradual migration.
**Cons:** Dead code, confusing — two commit-representing variants exist. Ticket says "replaces".

### Approach B: Rename CommitBillboard to CommitCar, change fields/rendering

Rename the variant in-place. Update all references. Change Lane assignment and draw logic.

**Pros:** Clean, no dead code. Single atomic change. The variant has the same fields (`message`, `author`, `author_color`), so the rename is trivial.
**Cons:** Touches every reference site at once.

### Decision: Approach B

The field set is identical. The only differences are:
1. Variant name: `CommitBillboard` -> `CommitCar`
2. Lane assignment: `Left`/`Right` (verge) -> `RoadLeft`/`RoadRight` (on-road)
3. Draw logic: rectangle+text -> wedge car shape+text

This is a clean rename+retype. No reason to keep dead code.

## Lane Design

### New Lane Variants

Add `RoadLeft` and `RoadRight` to `Lane` enum.

```rust
pub enum Lane {
    Left,       // existing: 1.15 * road_half (verge)
    Right,      // existing: 1.15 * road_half (verge)
    Center,     // existing: center
    RoadLeft,   // new: 0.35 * road_half (on-road, left of center)
    RoadRight,  // new: 0.35 * road_half (on-road, right of center)
}
```

The 0.35 factor (from the ticket) keeps cars within the road surface but offset enough to create two distinct lanes.

### Spawn Assignment

In `WorldState::update()`, the spawn logic currently alternates `Left`/`Right` for all non-`TierGate` objects. Change this:

- `CommitCar` -> alternate `RoadLeft`/`RoadRight`
- Everything else -> alternate `Left`/`Right` (unchanged)
- `TierGate` -> `Center` (unchanged)

This requires checking the object variant during spawn. The simplest approach: match on the variant before choosing lane.

## Car Drawing Design

### Scale-dependent LOD (from ticket)

The ticket specifies three LOD levels based on projected width:

| `car_w` | Rendering |
|---------|-----------|
| < 4px   | Colored dot (single rect) |
| < 8px   | Colored rectangle |
| >= 8px  | Full wedge with nose, body, dark/light sides |

### Size

~2/3 of the player car. Player car: `car_w=60`, `car_h=30`. Commit car base: `car_w=40`, `car_h=20`. These are scaled by `scale^2` (quadratic, matching the existing OutRun feel).

### Color

Body color = `author_color`. Dark side = darkened author_color. Light side = brightened author_color. This gives each author a distinct car color.

### Text Label

Show commit message above car when `scale >= 0.5` (ticket says 0.5, different from billboard's 0.35). Text scale: 1x when `scale >= 0.5`, 2x when `scale >= 0.65`.

## Projection Changes

Add two arms to the `lane_x` match in `project()`:

```rust
Lane::RoadLeft  => (cx - road_half_here * 0.35).max(0.0) as u32,
Lane::RoadRight => (cx + road_half_here * 0.35) as u32,
```

No other projection changes needed — curvature following is automatic via `cx`.

## Test Updates

### Tests to modify

1. `test_ingest_poll_creates_billboard` -> rename to `test_ingest_poll_creates_commit_car`, match on `CommitCar`.
2. `test_commit_billboard_color` -> rename, use `CommitCar`, place in `RoadLeft`/`RoadRight` lane.
3. `test_commit_billboard_text_near` -> rename, use `CommitCar`, adjust scale threshold (0.5 vs 0.35).
4. `test_commit_billboard_text_suppressed_far` -> rename, use `CommitCar`.
5. Benchmark: change `CommitBillboard` -> `CommitCar`, `Lane::Left`/`Right` -> `RoadLeft`/`RoadRight` for commit entries.

### New tests

1. `test_project_road_lanes` — verify `RoadLeft.x < Center.x < RoadRight.x` and both are inside road edges.
2. `test_commit_car_on_road` — verify commit car pixels appear within road boundaries.
3. `test_commit_car_lod_dot` — verify tiny scale produces colored pixels (dot LOD).
4. `test_lane_assignment_commit_car` — verify CommitCar gets RoadLeft/RoadRight, other objects get Left/Right.

## No Changes Needed

- Z-ordering: back-to-front sort handles everything.
- Player car draw order: already drawn after sprites.
- Despawn logic: unchanged.
- Other roadside objects: unchanged (keep Left/Right verge placement).
