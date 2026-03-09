# T-002-03 Design: world-simulation

## Problem Statement

The world simulation code is functionally complete but has two calibration
discrepancies with the specification, and zero unit test coverage.

### Discrepancy 1: Speed Target Formula

**Current:** `1.5 + (cpm * 28.0).min(28.5)` → range [1.5, 30.0]
**Spec AC:** `0.4 + (cpm * 2.8).min(11.6)` → range [0.4, 12.0]

The current formula was introduced during initial implementation with a comment
about "calibrated for solo/small-team repos." The spec formula produces more
conservative speeds that match the spec table:

| Tier | Spec Target Speed | Current Speed | AC Speed |
|------|------------------|---------------|----------|
| Flatline (0 cpm) | 0.4 | 1.5 | 0.4 |
| Cruise (0.03 cpm) | ~1.2 | ~2.3 | ~0.48 |
| Active (0.5 cpm) | 3.0 | ~15.5 | 1.8 |
| Demon (1.5 cpm) | 6.5 | ~30.0 | 4.6 |
| VDemon (4.0 cpm) | 12.0 | ~30.0 | 11.6 |

The AC formula matches the spec table at VelocityDemon (12.0) and Flatline (0.4).

### Discrepancy 2: VelocityTier Thresholds

**Current:** 0 / >0 / 0.15 / 0.5 / 1.0
**Spec AC:** 0 / >0 / 0.5 / 1.5 / 4.0

The current thresholds were lowered "for solo/small-team repos with 30-min window."
The spec thresholds are higher, meaning tiers are harder to reach. This matches
the spec table which lists specific cpm thresholds.

## Options Considered

### Option A: Exact AC Values

Change `speed_target()` to `0.4 + (cpm * 2.8).min(11.6)`.
Change tier thresholds to 0 / 0.5 / 1.5 / 4.0.

Pros: Matches acceptance criteria exactly.
Cons: None — the ACs are explicit about formulas and thresholds.

### Option B: Keep Recalibrated Values, Document Deviation

Keep current code, update ACs.

Pros: Current values may feel better for solo repos.
Cons: Violates the ticket's acceptance criteria. Not our call.

### Option C: Make Values Configurable

Add CLI flags for threshold/speed tuning.

Pros: Flexible.
Cons: Over-engineering. Not in the ticket scope.

## Decision: Option A

The acceptance criteria are explicit and specific. The formulas and thresholds are
stated as verifiable conditions. We implement them exactly.

The `speed.rs` comments about "calibrated for solo/small-team repos" were a
well-intentioned adjustment, but the ticket ACs override them. If recalibration
is needed later, that's a separate ticket.

## Test Strategy

The world module needs unit tests covering all ACs. No integration tests needed —
the simulation is pure computation with no I/O.

Tests needed:
1. **VelocityTier::from_commits_per_min** — boundary tests at each threshold
2. **speed_target()** — verify formula at key cpm values
3. **WorldState::new()** — verify initial field values from a known seed
4. **WorldState::update()** — verify speed lerp, z_offset advance, tier recomputation,
   object spawn/despawn
5. **WorldState::ingest_poll()** — verify object conversion, tier gate spawn,
   curve target shift
6. **WorldState::draw_distance()** — verify 1.2x at VelocityDemon

Test helper: A `test_seed()` function returning a deterministic `RepoSeed` (already
used in `sprites.rs` tests — we can replicate the pattern).

## Impact on Renderer

Changing speed values affects visual appearance but not correctness. The renderer
reads `world.speed` and `world.z_offset` for stripe animation speed, and
`world.tier` for effect thresholds. Lower speeds mean slower stripe crawl;
higher tier thresholds mean effects activate later. Both are intentional per spec.
