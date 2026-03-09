# git-demon

> A sci-fi cyber racecar screensaver for your terminal, driven by live git
> activity. You are not reading metrics. You are driving through them.

---

## Overview

`git-demon` is a Rust binary crate. It opens a git repository, watches it for
commit activity, and renders a pseudo-perspective OutRun/Rad Racer highway in
your terminal. The road speed and roadside scenery are live functions of what
your coding agents are doing. The visual reference is Rad Racer — single
vanishing point, alternating road stripe segments, depth-scaled sprites — but
executed at true pixel resolution via the Kitty Graphics Protocol, with a
full software rasterizer writing into a raw RGBA framebuffer each frame.

The primary use case is a tmux corner pane running while agents work.

---

## Crate identity

```
name        : git-demon
type        : binary crate (lib surface for tui-toys integration)
edition     : 2021
license     : MIT OR Apache-2.0
```

---

## Concept

You are in a racecar. The road is your git history. Roadside objects are the
data layer — commit messages on billboards, line additions as tower blocks, file
deletions as graveyards, idle authors as dead trees. Speed is set by
commits-per-minute in a rolling window. A quiet repo is a twilight cruise. A
hot agent session pushes you to Velocity Demon: the stripes blur, neon arches
flash tier names across the road, speed lines radiate from the vanishing point.

The aesthetic is cyberspace sci-fi: deep black sky, electric neon road accent
colors seeded from the repo's identity hash, CRT scanline filter, bloom on
emissive elements, motion blur at high speed. Not ASCII art. Not character
blocks. Actual pixels delivered through the terminal's graphics protocol.

---

## Dependencies

```toml
[dependencies]
ratatui           = "0.30"
ratatui-image     = "10"       # Kitty / Sixel / iTerm2 / halfblock auto-detect
crossterm         = "0.29"
image             = "0.25"     # ImageBuffer<Rgba<u8>> — the framebuffer type

git2              = "0.20"     # libgit2 bindings, no subprocess

noise             = "0.9"      # OpenSimplex for terrain + sky shimmer
rand              = "0.10"
rand_pcg          = "0.10"     # PCG64 seeded from commit SHA — deterministic

chrono            = "0.4"
crossbeam-channel = "0.5"      # git poller thread → render loop
clap              = { version = "4", features = ["derive"] }
anyhow            = "1"

[dev-dependencies]
criterion         = "0.8"
```

No GPU. No Bevy. No async runtime. One background thread for git polling,
everything else on the main thread at 60fps.

---

## Module layout

```
src/
  main.rs               CLI entry, terminal setup/teardown, main loop
  lib.rs                Public surface for tui-toys dispatch

  renderer/
    mod.rs              FrameRenderer — owns pixel buffer, calls all passes
    sky.rs              Sky gradient, sun/moon disc, hue rotation at peak tier
    road.rs             Pseudo-perspective scanline rasterizer, neon grid lines
    terrain.rs          Simplex noise verge silhouettes, left and right
    sprites.rs          Depth-projected sprite rasterizer, billboard text
    hud.rs              Bottom strip overlay, pixel font, live stats
    effects.rs          Bloom, motion blur, scanline filter, speed lines
    font.rs             Builtin 5×7 bitmap font, ASCII 32–126, no file I/O

  world/
    mod.rs              WorldState — all simulation state, update(dt)
    speed.rs            Velocity tier transitions, lerp logic
    objects.rs          RoadsideObject enum, spawn queue, lifecycle

  git/
    mod.rs              Public git interface
    poller.rs           Background thread, crossbeam sender, configurable interval
    seed.rs             RepoSeed — computed once from full history via git2
    stats.rs            Rolling window metrics from PollResult
```

---

## Rendering pipeline

### Framebuffer

Every frame a `ImageBuffer<Rgba<u8>>` of dimensions `pixel_w × pixel_h` is
rendered and sent to the terminal. Dimensions are derived at startup by querying
the terminal for its cell pixel size:

```rust
let (cell_w, cell_h) = picker.font_size();  // e.g. 12 × 24 px per cell
let pixel_w = terminal_cols * cell_w;
let pixel_h = terminal_rows * cell_h;
```

On a typical Kitty setup (160×40 terminal, 12×24 font) this is **1920×960 true
pixels** — more than enough to render sub-pixel smooth road geometry, readable
billboard text, and soft bloom.

The buffer is reused across frames (no allocation in the hot path). A second
buffer holds the previous frame for motion blur.

### Protocol

`ratatui_image::picker::Picker::from_query_stdio()` detects the terminal's
graphics protocol at startup, in priority order:

```
Kitty → Sixel → iTerm2 → halfblock (Unicode ▀▄)
```

The halfblock path is a graceful fallback. The Kitty path is the target: raw
RGBA chunks sent over the terminal's stdio, GPU-composited at the terminal side.
Kitty at 60fps is confirmed viable; the bottleneck is the rasterizer, not the
protocol.

For tmux: `set -g allow-passthrough on` in `~/.tmux.conf` is required for Kitty
protocol passthrough.

### Draw order (per frame)

```
1.  sky()               top 35% — vertical gradient, repo-hued
2.  sun()               fixed disc above-right of horizon, complement hue
3.  terrain_left()      left verge silhouette, simplex noise heightmap
4.  terrain_right()     right verge silhouette
5.  road_scanlines()    pseudo-perspective trapezoid, bottom 65%
6.  road_grid()         faint perspective grid lines on road surface, neon accent
7.  roadside_objects()  sprites sorted back-to-front by depth
8.  speed_lines()       radial 1px streaks from vanishing point (Demon+ only)
9.  motion_blur()       blend 15–35% of previous frame over current
10. scanline_filter()   darken every other row 20% — CRT feel
11. bloom_pass()        3×3 additive blur on pixels above luminance threshold
12. hud_overlay()       bottom 18px strip, pixel font stats
```

All passes write directly into `&mut ImageBuffer<Rgba<u8>>`. No intermediate
render targets except the previous-frame buffer retained for pass 9.

---

## Road rasterizer

Classic scanline pseudo-perspective with road curvature. Each pixel row `y`
in the road region maps to a depth value; road half-width is linearly
interpolated between a narrow horizon value and a wide near-camera value.
The road center shifts laterally based on a curve offset that smoothly
interpolates toward a target set by git activity, creating gentle S-curves.

```rust
const HORIZON_RATIO: f32 = 0.35;  // horizon sits 35% down from top of frame
const ROAD_MIN_HALF: f32 = 8.0;   // pixels at horizon
const ROAD_MAX_HALF: f32 = 480.0; // pixels at camera (half screen width)
const RUMBLE_WIDTH:  u32 = 12;    // pixels of rumble strip each side
const STRIPE_PERIOD: f32 = 1.0;   // world units per stripe cycle
```

The curve offset is applied quadratically with depth — near pixels shift more
than far pixels — creating a natural perspective-correct curve appearance.

`z_offset` accumulates at `world.speed × dt` each frame — a pure counter
driving stripe swaps. Speed is a `f32` that lerps toward `speed_target` at rate
`4.0 × dt`.

The neon road grid (pass 6) draws thin horizontal lines at perspective-projected
intervals of evenly-spaced world-Z positions, in the repo's accent hue at
alpha 60. At high speed these compress toward the horizon and create a
hyperspace-grid feel.

---

## Depth projection and sprite rendering

All roadside objects live in world space as `(lane: Lane, z_world: f32)` pairs.
`Lane` is `Left | Right | Center`. `z_world` is meters ahead of camera.

Projection to screen accounts for road curvature — the curve offset is applied
to sprite x-positions using the same quadratic depth formula as the road.

Sprites are sorted by `z_world` descending before rasterization so near objects
overdraw far ones correctly.

Rendered size:

```
sprite_px_w = (base_w_px as f32 * scale * scale) as u32   // quadratic — OutRun feel
sprite_px_h = (base_h_px as f32 * scale * scale) as u32
```

Text is rendered via the builtin bitmap font (see `font.rs`). Text is suppressed
when `depth_scale < 0.35` — the object renders as a colored rect only. At
`depth_scale ≥ 0.35` text renders at 1× glyph scale. At
`depth_scale ≥ 0.65` it renders at 2× glyph scale for readability on the approach.

---

## Roadside objects

```rust
pub enum RoadsideObject {
    CommitBillboard { message, author, author_color },
    AdditionTower { lines, color },
    DeletionShard { lines },
    FilePosts { count },
    VelocitySign { commits_per_min },
    TierGate { tier },
    IdleAuthorTree { author, idle_minutes },
    SectorPylon { sector },
}
```

### Spawn logic

On each `PollResult` received from the git poller, `world.ingest_poll()`
converts commits into a batch of `RoadsideObject`s and appends them to
`pending_signs`. Objects are placed at `camera_z + SPAWN_DISTANCE` with
alternating Left/Right lane assignment. Between polls, the road is clear.
Sparseness is intentional — a quiet repo is a quiet road.

`TierGate` is emitted immediately on tier change, bypassing the poll queue,
and spawned at `camera_z + NEAR_SPAWN` so it arrives quickly.

---

## Velocity tiers

| Tier          | Target speed | cpm threshold | Notes                        |
|---------------|-------------|---------------|------------------------------|
| Flatline      | 0.4         | 0             | Stripes barely crawl         |
| Cruise        | 1.2         | > 0           | Comfortable idle             |
| Active        | 3.0         | ≥ 0.5         | Highway speed                |
| Demon         | 6.5         | ≥ 1.5         | Speed lines begin, bloom     |
| VelocityDemon | 12.0        | ≥ 4.0         | Strobe, hue rotation, chaos  |

At `VelocityDemon`:
- Road `DRAW_DISTANCE` increases 20%
- `ROAD_MAX_HALF` expands 5%
- Horizon drops 2%
- Sky hue rotates at 1°/s

---

## CLI

```
git-demon [OPTIONS]

Options:
  --repo <PATH>        Git repository to watch  [default: .]
  --window <MINS>      Commit lookback window in minutes  [default: 30]
  --interval <SECS>    Git poll interval in seconds  [default: 30]
  --fps <N>            Target framerate  [default: 60]
  --no-blur            Disable motion blur
  --no-bloom           Disable bloom pass
  --no-scanlines       Disable CRT scanline filter
  --no-hud             Hide bottom HUD strip
  -h, --help
  -V, --version
```

---

## Performance targets

| Metric                              | Target                    |
|-------------------------------------|---------------------------|
| Software raster time per frame      | < 4 ms at 1920×960        |
| Kitty encode + send per frame       | < 8 ms (zlib compressed)  |
| Total frame budget at 60fps         | 16.7 ms                   |
| Framebuffer memory (2 buffers)      | < 15 MB at 1920×960       |
| Git poll (background thread)        | Non-blocking to render loop |
| Heap allocation in render hot path  | Zero after startup         |

---

## Differences from original spec

- **Road curvature is included** — the road curves based on git activity bursts
- Updated dependency versions to latest stable (ratatui 0.30, crossterm 0.29, etc.)
- Added `clap` for CLI parsing and `anyhow` for error handling
