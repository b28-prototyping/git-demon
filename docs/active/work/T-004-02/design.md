# T-004-02 Design: Performance Benchmarks

## Problem
The existing `benches/render.rs` has three benchmarks but they: (1) depend on the local git repo for `RepoSeed`, (2) don't populate any roadside objects, and (3) lack individual pass benchmarks. The acceptance criteria require realistic scenes, per-pass breakdowns, and documented results.

## Options Considered

### Option A: Patch existing benchmarks in-place
Add a synthetic `RepoSeed` constructor, populate objects manually, add per-pass benchmarks to the same file.

**Pros:** Single file, minimal change.
**Cons:** File grows large. Mixing full-pipeline and per-pass benchmarks in one criterion group produces a single report.

### Option B: Separate benchmark groups
Create separate criterion groups for full-pipeline and per-pass benchmarks within the same `benches/render.rs`. Each group gets its own Criterion config (measurement time, sample size).

**Pros:** Clean organization. Criterion naturally groups related benchmarks. Still one bench binary.
**Cons:** Slightly more code.

### Option C: Multiple bench files
Split into `benches/render.rs` (full pipeline) and `benches/passes.rs` (individual passes).

**Pros:** Clean separation.
**Cons:** Two bench binaries, two `[[bench]]` sections, more maintenance for a small project. No real benefit since Criterion groups already provide separation.

## Decision: Option B

Single file with two criterion groups. The full-pipeline group benchmarks the complete render at 1920×960 with realistic world state. The per-pass group benchmarks individual passes (sky, road+grid, terrain, sprites, effects) independently.

## Key Design Decisions

### 1. Synthetic RepoSeed (no git I/O)
Construct `RepoSeed` with fixed values matching test helpers used elsewhere. This makes benchmarks reproducible across machines and avoids git dependency.

### 2. Realistic WorldState with objects
Create a helper that:
- Constructs WorldState from seed
- Sets speed=5.0, commits_per_min=2.0 (Demon tier)
- Manually creates and inserts 10-15 active_objects spanning the draw distance
- Includes CommitBillboard, AdditionTower, DeletionShard, VelocitySign, TierGate
- Sets curve_offset for visual interest

### 3. Per-pass benchmarks
Each pass benchmarked independently with its own pre-rendered framebuffer state:
- **sky**: draw_sky + draw_sun + draw_bloom_bleed
- **road**: draw_road + draw_grid (these always run together)
- **terrain**: draw_terrain
- **sprites**: draw_sprites (with populated active_objects)
- **effects**: apply_motion_blur + apply_scanline_filter + apply_bloom

Speed lines and car are cheap secondary passes — not worth individual benchmarks.

### 4. Benchmark configuration
- Full pipeline: default Criterion (100 samples, 5s measurement)
- Per-pass: same defaults — individual passes are fast enough

### 5. Results documentation
Run benchmarks and capture output in `docs/active/work/T-004-02/benchmark-results.md`. Include machine info, frame times, and comparison to 4ms target.

## What This Does NOT Do
- Does not add flamegraph profiling (out of scope)
- Does not add CI performance regression checks (future work)
- Does not benchmark the Kitty encode/send path (terminal I/O, out of scope for rasterizer)
