# T-006-01 Plan: commit-cars-on-road

## Step 1: Update object model (`src/world/objects.rs`)

1. Add `RoadLeft` and `RoadRight` to `Lane` enum.
2. Rename `CommitBillboard` to `CommitCar` in `RoadsideObject` enum (same fields).
3. Update `ingest_poll_to_queue()` to emit `CommitCar` instead of `CommitBillboard`.

**Verify:** `cargo check` will fail (downstream references). This is expected — steps 2-4 fix them.

## Step 2: Update spawn logic (`src/world/mod.rs`)

1. In `update()`, change lane assignment: `CommitCar` objects get `RoadLeft`/`RoadRight`, others keep `Left`/`Right`.
2. Rename test `test_ingest_poll_creates_billboard` → `test_ingest_poll_creates_commit_car`, update match.
3. Add test `test_lane_assignment_commit_car`: verify CommitCar gets road lanes, VelocitySign gets verge lanes.

**Verify:** Tests in `world::tests` module compile (sprites.rs still has stale references).

## Step 3: Update projection (`src/renderer/sprites.rs`)

1. Add `RoadLeft`/`RoadRight` arms to `lane_x` match in `project()`.
2. Add test `test_project_road_left_right`: verify road lanes are between center and verge lanes.

**Verify:** Projection tests pass.

## Step 4: Replace CommitBillboard rendering with CommitCar

1. Add constants `COMMIT_CAR_BASE_W = 40.0`, `COMMIT_CAR_BASE_H = 20.0`.
2. Add helper functions: `darken()`, `brighten()`, `draw_commit_car()`.
3. Replace `CommitBillboard` match arm with `CommitCar` arm in `draw_sprites()`:
   - Compute `car_w` and `car_h` from base * scale^2.
   - If `car_w < 4`: draw 2x2 colored dot.
   - If `car_w < 8`: draw colored rect.
   - Else: call `draw_commit_car()` for full wedge.
   - If `scale >= 0.5`: draw commit message text above car.
4. Update existing tests:
   - `test_commit_billboard_color` → `test_commit_car_color`
   - `test_commit_billboard_text_near` → `test_commit_car_text_near` (scale threshold 0.5)
   - `test_commit_billboard_text_suppressed_far` → `test_commit_car_text_suppressed_far`
5. Add test `test_commit_car_lod_dot`: place car far away, verify colored pixels present.

**Verify:** `cargo test` — all tests pass.

## Step 5: Update benchmark (`benches/render.rs`)

1. Change `CommitBillboard` → `CommitCar` in bench object list.
2. Change `Lane::Left`/`Lane::Right` → `Lane::RoadLeft`/`Lane::RoadRight` for commit car entries.

**Verify:** `cargo bench -- --test` compiles and runs.

## Step 6: Final verification

1. `cargo test` — all tests pass.
2. `cargo clippy` — no warnings.
3. `cargo build` — clean build.

## Testing Strategy

| Test | Type | What it verifies |
|------|------|-----------------|
| `test_ingest_poll_creates_commit_car` | Unit | ingest produces CommitCar variant |
| `test_lane_assignment_commit_car` | Unit | CommitCar gets RoadLeft/RoadRight |
| `test_project_road_left_right` | Unit | Road lane X between center and verge |
| `test_commit_car_color` | Render | Author color pixels present |
| `test_commit_car_text_near` | Render | Text at scale >= 0.5 |
| `test_commit_car_text_suppressed_far` | Render | No text at scale < 0.5 |
| `test_commit_car_lod_dot` | Render | Far car produces pixels |
| Existing lane/projection tests | Unit | RoadLeft/RoadRight don't break existing lanes |
