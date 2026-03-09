# T-004-02 Structure: Performance Benchmarks

## Files Modified

### `benches/render.rs` — Full rewrite
Replace the existing three benchmark functions with a well-structured benchmark suite.

**Module-level helpers:**
- `fn bench_seed() -> RepoSeed` — deterministic synthetic seed
- `fn bench_world(seed: &RepoSeed) -> WorldState` — populated world with 12+ active objects at various depths and types

**Criterion group 1: `full_pipeline`**
- `bench_render_full_effects` — 1920×960, all effects on, populated scene
- `bench_render_no_effects` — 1920×960, all effects off
- `bench_render_populated_scene` — 1920×960, all effects on, VelocityDemon tier (max load)

**Criterion group 2: `individual_passes`**
- `bench_sky` — sky + sun + bloom_bleed
- `bench_road` — road + grid
- `bench_terrain` — terrain
- `bench_sprites` — sprites with populated objects
- `bench_effects` — motion_blur + scanline_filter + bloom

Each per-pass benchmark creates its own FrameRenderer and renders prerequisite passes before timing the target pass (e.g., road benchmark needs sky already drawn so the buffer is in a realistic state, but only the road draw is timed).

**Entry point:**
```rust
criterion_group!(full_pipeline, bench_render_full_effects, bench_render_no_effects, bench_render_populated_scene);
criterion_group!(individual_passes, bench_sky, bench_road, bench_terrain, bench_sprites, bench_effects);
criterion_main!(full_pipeline, individual_passes);
```

### `docs/active/work/T-004-02/benchmark-results.md` — New
Captured benchmark results after implementation. Machine specs, per-benchmark timing, pass-by-pass breakdown, comparison to 4ms target.

## Files NOT Modified
- `Cargo.toml` — already has criterion and bench config
- `src/**` — no source changes needed; benchmarks exercise the public API

## Public Interface Dependencies
The benchmarks use:
- `git_demon::renderer::FrameRenderer` — `new()`, `render()`
- `git_demon::world::WorldState` — `new()`, `update()`
- `git_demon::world::objects::{Lane, RoadsideObject}`
- `git_demon::world::speed::VelocityTier`
- `git_demon::git::seed::RepoSeed`

All of these are already `pub`. No visibility changes needed.

Individual renderer pass functions (`sky::draw_sky`, `road::draw_road`, etc.) are `pub` within the crate but not `pub` from outside. The per-pass benchmarks need to call them directly.

**Resolution:** The renderer sub-module functions are `pub` (not `pub(crate)`), and `renderer` is `pub mod` in `lib.rs`. Since the bench file uses `git_demon::renderer::sky::draw_sky` etc., these are already accessible. Confirmed by checking: `pub fn draw_sky(...)` in sky.rs, `pub fn draw_road(...)` in road.rs, etc.

## Component Boundaries
- Benchmark helpers (seed/world construction) are local to `benches/render.rs`
- No shared test utilities extracted — the helpers are simple enough to inline
- Each benchmark function is self-contained with setup in Criterion's setup phase
