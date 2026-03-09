# T-002-03 Structure: world-simulation

## Files Modified

### src/world/speed.rs

**Changes:**
1. Update `VelocityTier::from_commits_per_min()` thresholds:
   - VelocityDemon: `>= 4.0` (was 1.0)
   - Demon: `>= 1.5` (was 0.5)
   - Active: `>= 0.5` (was 0.15)
   - Cruise: `> 0.0` (unchanged)
   - Flatline: `0.0` (unchanged)

2. Update `speed_target()` formula:
   - Change from `1.5 + (cpm * 28.0).min(28.5)` to `0.4 + (cpm * 2.8).min(11.6)`
   - Update accompanying comments to reflect new speed values

3. Add `#[cfg(test)] mod tests` with:
   - Tier boundary tests for each threshold
   - speed_target formula verification at key cpm values

### src/world/mod.rs

**Changes:**
1. Update `WorldState::new()`:
   - Change `speed` initial from `1.5` to `0.4`
   - Change `speed_target` formula from `1.5 + seed.speed_base * 2.8`
     to `0.4 + seed.speed_base * 2.8`
   (Both use the same linear coefficient, just different base)

2. Add `#[cfg(test)] mod tests` with:
   - `new()` initialization tests
   - `update(dt)` behavior tests (speed lerp, z advance, despawn)
   - `ingest_poll()` object generation tests
   - `draw_distance()` tier-dependent tests

## Files NOT Modified

- `src/world/objects.rs` — object spawning logic already matches ACs
- `src/renderer/*` — renderer reads WorldState fields, no interface changes
- `src/main.rs` — no API changes
- `src/git/*` — no changes needed

## Module Boundaries

No public interface changes. `WorldState` fields remain `pub`.
`VelocityTier::from_commits_per_min()` and `speed_target()` signatures unchanged.
Only internal constants/formulas change.

## Test Architecture

Tests in `speed.rs` are pure function tests — no state, no setup.
Tests in `mod.rs` need a `test_seed()` helper to create a deterministic `RepoSeed`.
The `PollResult` struct needs test instances with known commit data.

Both test modules are self-contained — no shared test utilities needed across files.
