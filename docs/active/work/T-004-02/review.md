# T-004-02 Review: Performance Benchmarks

## Summary of Changes

### Files Modified
- **`benches/render.rs`** — Complete rewrite. Replaced 3 git-dependent benchmarks with 8 comprehensive benchmarks using synthetic data.

### Files Created
- **`docs/active/work/T-004-02/research.md`** — Codebase analysis
- **`docs/active/work/T-004-02/design.md`** — Approach selection
- **`docs/active/work/T-004-02/structure.md`** — File-level blueprint
- **`docs/active/work/T-004-02/plan.md`** — Implementation steps
- **`docs/active/work/T-004-02/progress.md`** — Implementation tracking
- **`docs/active/work/T-004-02/benchmark-results.md`** — Captured timing data with analysis

## Acceptance Criteria Evaluation

| Criterion | Status | Notes |
|---|---|---|
| Criterion benchmark in `benches/render.rs` exercises `FrameRenderer::render()` at 1920x960 | Done | Three full-pipeline benchmarks at 1920x960 |
| Benchmark creates realistic WorldState with active objects and non-zero speed | Done | 12 active objects, speed=5.0, cpm=2.0, curve_offset=20.0 |
| Reports per-frame render time in ms | Done | Criterion reports mean/median/stddev |
| Separate benchmarks for individual passes if total exceeds target | Done | 5 per-pass benchmarks: sky, road, terrain, sprites, effects |
| No terminal I/O in benchmarks | Done | Synthetic RepoSeed, no picker/terminal queries |
| `cargo bench` runs without errors | Done | All 8 benchmarks complete successfully |
| Document results in work directory | Done | `benchmark-results.md` with full breakdown |

## Test Coverage
- All 166 existing tests pass — no regressions
- Benchmarks themselves exercise the render pipeline with populated scenes, providing integration-level coverage
- `cargo clippy --benches` clean

## Key Improvements Over Previous Benchmarks
1. **Reproducible**: Synthetic `RepoSeed` instead of `RepoSeed::compute(".")` — results are machine-independent (no git repo dependency)
2. **Realistic**: 12 active roadside objects across the draw distance vs empty scene
3. **Granular**: Per-pass breakdown identifies effects (46.8%) and road (32.7%) as dominant costs
4. **Complete**: VelocityDemon variant tests worst-case performance

## Open Concerns

1. **Full pipeline exceeds 4ms target**: 4.66 ms median with all effects, 4.42 ms at VelocityDemon. The rasterizer core (no effects) is 1.97 ms — well within budget. The overshoot is in post-processing effects (motion blur, scanline filter, bloom at 2.18 ms combined) and road rasterizer (1.52 ms). Performance optimization of these passes is out of scope for this ticket.

2. **Benchmark variability**: Some benchmarks show 8-17% outliers (thermal throttling, OS scheduling). This is normal for CPU-bound benchmarks on desktop systems. CI would need dedicated hardware or longer measurement windows for stable regression detection.

3. **No CI integration**: Benchmarks run locally only. Adding Criterion threshold-based regression detection to CI is future work.
