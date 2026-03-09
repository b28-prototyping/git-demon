# CLAUDE.md

## Project

git-demon (Rust) — A sci-fi cyber racecar screensaver for your terminal, driven by live git activity.

### Build and Test

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Run
cargo run -- --repo .
```

### Source Layout

```
src/
  main.rs               CLI entry, terminal setup/teardown, main loop
  lib.rs                Public surface (re-exports modules)
  renderer/
    mod.rs              FrameRenderer — owns pixel buffer, orchestrates all passes
    sky.rs              Sky gradient, sun/moon disc
    road.rs             Pseudo-perspective scanline rasterizer with curvature
    terrain.rs          Simplex noise verge silhouettes
    sprites.rs          Depth-projected sprite rasterizer
    hud.rs              Bottom HUD strip overlay
    effects.rs          Bloom, motion blur, scanline filter, speed lines
    font.rs             Builtin 5×7 bitmap font
  world/
    mod.rs              WorldState — simulation state, update(dt)
    speed.rs            VelocityTier enum, speed targets
    objects.rs          RoadsideObject enum, spawn logic
  git/
    mod.rs              Public git interface
    poller.rs           Background thread git polling
    seed.rs             RepoSeed — computed once from full history
    stats.rs            Rolling window metrics
```

### Key Architecture

- Software rasterizer writes into `ImageBuffer<Rgba<u8>>` each frame
- `ratatui-image` handles Kitty/Sixel/iTerm2/halfblock protocol detection
- One background thread polls git via `git2`, sends `PollResult` over crossbeam channel
- Main thread renders at 60fps, drains git channel non-blocking
- Road has curvature driven by git activity bursts
- All colors seeded from repo identity hash for deterministic appearance

### Directory Conventions

```
docs/active/tickets/    # Ticket files (markdown with YAML frontmatter)
docs/active/stories/    # Story files (same frontmatter pattern)
docs/active/work/       # Work artifacts, one subdirectory per ticket ID
docs/specification.md   # Full project specification
```

---

The RDSPI workflow definition is in docs/knowledge/rdspi-workflow.md and is injected into agent context by lisa automatically.
