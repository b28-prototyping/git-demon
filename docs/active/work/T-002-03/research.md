# T-002-03 Research: world-simulation

## Scope

This ticket covers `WorldState` — the simulation core that ties git polling data
to the visual scene. It owns camera position, speed, velocity tier, road curvature,
and the lifecycle of roadside objects.

## Existing Code

### world/mod.rs (147 lines)

`WorldState` struct with fields:
- `z_offset`, `camera_z`, `speed`, `speed_target` — camera/motion state
- `commits_per_min`, `lines_added`, `lines_deleted`, `files_changed` — latest poll stats
- `tier: VelocityTier` — current tier
- `time: f32` — elapsed simulation time
- `total_commits: u64` — from seed
- `pending_objects: VecDeque<RoadsideObject>` — objects awaiting spawn
- `active_objects: Vec<(Lane, f32, RoadsideObject)>` — live objects with lane + z_world
- `curve_offset`, `curve_target`, `steer_angle` — road curvature state

Methods implemented:
- `new(seed)` — initializes with `speed_target = 1.5 + seed.speed_base * 2.8`
- `update(dt)` — lerps speed, advances z_offset/camera_z, updates steer_angle (sinusoidal weave),
  lerps curve_offset, recomputes tier + speed_target, spawns pending objects, despawns behind camera
- `ingest_poll(result, seed)` — copies poll stats, calls `ingest_poll_to_queue`, spawns TierGate
  on tier change, shifts curve_target on activity bursts (>1 cpm)
- `tier_index()` — returns tier as u8
- `draw_distance()` — 200.0, or 240.0 at VelocityDemon
- `sector()` — total_commits / 100

### world/speed.rs (50 lines)

`VelocityTier` enum: Flatline(0), Cruise(1), Active(2), Demon(3), VelocityDemon(4).

Current `from_commits_per_min` thresholds (recalibrated):
- VelocityDemon: >= 1.0 cpm
- Demon: >= 0.5 cpm
- Active: >= 0.15 cpm
- Cruise: > 0.0 cpm
- Flatline: 0.0 cpm

Current `speed_target` formula: `1.5 + (cpm * 28.0).min(28.5)`

### world/objects.rs (90 lines)

`Lane` enum: Left, Right, Center.

`RoadsideObject` enum with 8 variants: CommitBillboard, AdditionTower, DeletionShard,
FilePosts, VelocitySign, TierGate, IdleAuthorTree, SectorPylon.

`ingest_poll_to_queue()` — iterates commits, creates CommitBillboard for each,
AdditionTower if lines_added > 50, DeletionShard if lines_deleted > 50,
plus a VelocitySign per poll.

### Consumers

**renderer/mod.rs** calls `world.tier_index()`, `world.draw_distance()` and reads
`world.speed`, `world.curve_offset`, `world.camera_z`, `world.z_offset`,
`world.active_objects`, `world.tier`, `world.time`.

**renderer/sprites.rs** reads `world.camera_z`, `world.draw_distance()`,
`world.curve_offset`, `world.active_objects`.

**renderer/hud.rs** reads `world.commits_per_min`, `world.tier`, `world.speed`,
`world.total_commits`, `world.sector()`.

**renderer/effects.rs** reads `world.speed`, `world.tier_index()`, `world.time`.

**renderer/road.rs** reads `world.z_offset`, `world.curve_offset`, `world.speed`,
`world.tier`.

**main.rs** creates `WorldState::new(&seed)`, calls `world.ingest_poll(&result, &seed)`
and `world.update(dt)` each frame.

## Acceptance Criteria vs Current State

| AC | Spec Value | Current Code | Match? |
|---|---|---|---|
| speed_target formula | `0.4 + (cpm * 2.8).min(11.6)` | `1.5 + (cpm * 28.0).min(28.5)` | NO |
| VelocityTier thresholds | 0 / 0.5 / 1.5 / 4.0 | 0 / >0 / 0.15 / 0.5 / 1.0 | NO |
| speed lerp rate | 4.0 * dt | 4.0 * dt | YES |
| new() from RepoSeed | sensible defaults | present | YES |
| update(dt) advances all fields | all listed | all present | YES |
| ingest_poll object conversion | 3 object types | present | YES |
| Spawn at camera_z + SPAWN_DISTANCE | alternating L/R | present | YES |
| TierGate at NEAR_SPAWN | on tier change | present | YES |
| Curve shift on >1 cpm | random | present | YES |
| Despawn behind camera | DESPAWN_BEHIND | present | YES |
| draw_distance +20% at VDemon | 1.2x | present | YES |

## Key Discrepancies

1. **Speed target formula** — The current code uses `1.5 + (cpm * 28.0).min(28.5)`,
   producing speeds from 1.5 to 30.0. The ticket AC specifies `0.4 + (cpm * 2.8).min(11.6)`,
   producing speeds from 0.4 to 12.0. The spec table confirms: Flatline=0.4, VDemon=12.0.

2. **VelocityTier thresholds** — The current code uses lower thresholds (>0, 0.15, 0.5, 1.0)
   with a comment saying "calibrated for solo/small-team repos." The ticket AC and spec
   table specify (>0, 0.5, 1.5, 4.0). The spec thresholds match the original design.

3. **No unit tests** — The world module has zero test coverage. `speed.rs`, `mod.rs`,
   and `objects.rs` have no `#[cfg(test)]` blocks.

## Constraints

- `WorldState` fields are `pub` and directly read by renderer passes. Renaming or
  removing fields would require coordinated changes across renderer modules.
- `RepoSeed.speed_base` is used in `new()` to set initial speed_target. With the new
  formula this would be `0.4 + seed.speed_base * 2.8`.
- The `rand::rng()` usage in `ingest_poll` uses thread-local RNG. Fine for simulation,
  but makes curve_target non-deterministic in tests.

## Files Touched by This Ticket

- `src/world/speed.rs` — threshold + formula changes
- `src/world/mod.rs` — update `new()` initial speed_target formula
- No changes needed in `objects.rs` or renderer files (object spawning and
  rendering logic is correct as-is)
