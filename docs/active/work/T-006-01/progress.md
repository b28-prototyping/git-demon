# T-006-01 Progress: commit-cars-on-road

## Completed

### Step 1: Update object model (`src/world/objects.rs`)
- Added `RoadLeft` and `RoadRight` to `Lane` enum
- Renamed `CommitBillboard` to `CommitCar` (same fields)
- Updated `ingest_poll_to_queue()` to emit `CommitCar`

### Step 2: Update spawn logic (`src/world/mod.rs`)
- Changed lane assignment: `CommitCar` gets `RoadLeft`/`RoadRight`, others keep `Left`/`Right`
- Renamed test `test_ingest_poll_creates_billboard` → `test_ingest_poll_creates_commit_car`
- Added test `test_lane_assignment_commit_car`

### Step 3: Update projection (`src/renderer/sprites.rs`)
- Added `RoadLeft`/`RoadRight` arms to `lane_x` match in `project()`
- Added test `test_project_road_left_right`

### Step 4: Replace CommitBillboard rendering with CommitCar
- Added constants `COMMIT_CAR_BASE_W=40`, `COMMIT_CAR_BASE_H=20`, `TIER_GATE_BASE_W=80`
- Added helper functions: `darken()`, `brighten()`, `draw_commit_car()`
- Replaced `CommitBillboard` match arm with `CommitCar` (3-level LOD: dot, rect, wedge)
- Text label shows at `scale >= 0.5` (per ticket spec)
- Updated 3 existing tests, added `test_commit_car_lod_dot`

### Step 5: Update benchmark (`benches/render.rs`)
- Changed `CommitBillboard` → `CommitCar`
- Changed lanes to `RoadLeft`/`RoadRight` for commit car entries

### Step 6: Final verification
- `cargo test`: 161 tests pass (0 failures)
- `cargo clippy`: no new warnings (all warnings pre-existing from menu.rs/main.rs)
- `cargo build`: clean

## Deviations from Plan
- Added `TIER_GATE_BASE_W` constant — `TierGate` rendering previously used `BILLBOARD_BASE_W` which was renamed. Replaced with a dedicated constant rather than keeping the old name.

## Remaining
- None. All implementation steps complete.
