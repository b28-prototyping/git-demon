# T-004-02 Research: Performance Benchmarks

## Current State

### Existing Benchmark File
`benches/render.rs` already exists with three benchmark functions:
- `bench_render_full` — 1920×960, all effects enabled
- `bench_render_no_effects` — 1920×960, blur/bloom/scanlines/HUD disabled
- `bench_render_320x200` — 320×200, all effects enabled

### Benchmark Infrastructure
- `Cargo.toml` already has `criterion = "0.8"` in `[dev-dependencies]`
- `[[bench]] name = "render" harness = false` already configured
- Criterion group includes all three functions

### Problems with Current Benchmarks

1. **RepoSeed::compute(".")** — benchmarks depend on the actual git repo at `.`, making results non-reproducible across machines. Also performs I/O (revwalk) during setup.

2. **No active roadside objects** — `WorldState::new(&seed)` starts with empty `active_objects` and `pending_objects`. The benchmark sets `speed`, `commits_per_min`, and `time` but never calls `ingest_poll()` or manually populates objects. This means the sprite pass is benchmarking an empty scene.

3. **No individual pass benchmarks** — acceptance criteria requires separate benchmarks for road, sky, terrain, effects if total exceeds target. Currently only full-pipeline benchmarks exist.

4. **No benchmark results documented** — acceptance criteria requires documenting results in work directory.

### Render Pipeline Architecture

`FrameRenderer::render()` in `src/renderer/mod.rs` calls 12 passes in order:
1. `sky::draw_sky` — gradient fill, top 35% of buffer
2. `sky::draw_sun` — disc rasterization
3. `sky::draw_bloom_bleed` — horizon alpha blend
4. `terrain::draw_terrain` — simplex noise + tree procedural placement
5. `road::draw_road` — scanline rasterizer, bottom 65%
6. `road::draw_grid` — perspective grid lines
7. `sprites::draw_sprites` — depth-sorted object rendering
8. `effects::draw_car` — player car at bottom center
9. `effects::draw_speed_lines` — radial lines (Demon+ only)
10. `effects::apply_motion_blur` — blend with previous frame
11. `effects::apply_scanline_filter` — darken even rows
12. `effects::apply_bloom` — emissive pixel blur

### WorldState Construction for Benchmarks
`WorldState::new(seed)` requires a `&RepoSeed`. Tests across the codebase construct `RepoSeed` directly with literal field values (no git I/O). The pattern is consistent:
```rust
RepoSeed {
    accent_hue: 180.0,
    saturation: 0.8,
    terrain_roughness: 0.5,
    speed_base: 0.5,
    author_colors: HashMap::new(),
    total_commits: 100,
    repo_name: "test-repo".into(),
}
```

### Object Population
`ingest_poll_to_queue()` in `src/world/objects.rs` creates objects from `PollResult`. For a realistic scene, we need:
- Several `CommitBillboard`s with text
- `AdditionTower`s (lines > 50)
- `DeletionShard`s (lines > 50)
- `VelocitySign`s
- `TierGate`

After adding to pending_objects, `world.update(dt)` moves them to active_objects with proper z-placement.

### Performance Target
Spec says: < 4 ms software raster time per frame at 1920×960.

### Dependencies
- T-004-01 (dependency): already complete — this ticket can proceed
- Criterion 0.8 is already in Cargo.toml

## Key Files
| File | Role |
|---|---|
| `benches/render.rs` | Benchmark file (needs enhancement) |
| `src/renderer/mod.rs` | FrameRenderer, render() orchestrator |
| `src/world/mod.rs` | WorldState, update(), ingest_poll() |
| `src/world/objects.rs` | RoadsideObject, ingest_poll_to_queue() |
| `src/git/seed.rs` | RepoSeed struct and compute() |
| `src/git/poller.rs` | PollResult, CommitSummary structs |
| `Cargo.toml` | Already has criterion configured |

## Constraints
- Benchmarks must not perform terminal I/O
- FrameRenderer::render() is pure rasterizer — no I/O dependency
- RepoSeed::compute() does git I/O — must be avoided in benchmark hot path
- Criterion bench setup runs outside the timed loop — I/O is acceptable there, but synthetic seeds are more reproducible
