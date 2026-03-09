# T-004-02 Plan: Performance Benchmarks

## Steps

### Step 1: Rewrite benches/render.rs
Replace the existing benchmark file with the complete new implementation:

1. Add helper `bench_seed()` returning a synthetic `RepoSeed`
2. Add helper `bench_world()` that populates 12+ active objects
3. Implement full-pipeline benchmarks (all effects, no effects, VelocityDemon)
4. Implement per-pass benchmarks (sky, road, terrain, sprites, effects)
5. Wire up two criterion groups and criterion_main

**Verification:** `cargo bench --bench render -- --test` (dry run without measurement)

### Step 2: Run benchmarks and verify
Run `cargo bench` to confirm all benchmarks execute without error and produce timing output.

**Verification:** All 8 benchmarks complete, no panics, timing results displayed.

### Step 3: Document results
Capture benchmark output into `docs/active/work/T-004-02/benchmark-results.md` with machine info and analysis vs 4ms target.

**Verification:** File exists with timing data.

### Step 4: Run full test suite
Ensure nothing is broken: `cargo test`, `cargo clippy`.

**Verification:** All tests pass, no clippy warnings.

## Testing Strategy
- **Benchmark correctness:** `cargo bench` runs without panics — exercises the full render pipeline with populated scene
- **No regressions:** `cargo test` confirms existing unit tests pass
- **Lint:** `cargo clippy` clean

## Commit Plan
Single commit: "T-004-02: rewrite benchmarks with realistic scene and per-pass breakdown"
