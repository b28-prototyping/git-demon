# T-003-02 Research: HUD Overlay

## Current State

The HUD overlay is already implemented in `src/renderer/hud.rs` (commit `8f30046`).
This research documents the existing implementation and its integration points.

## Relevant Files

### Primary
- `src/renderer/hud.rs` — `draw_hud()` and `draw_dev_overlay()` functions
- `src/renderer/font.rs` — 5x7 bitmap font, `draw_text()`, `draw_char()`, `text_width()`

### Integration Points
- `src/renderer/mod.rs:115-117` — HUD called as pass 12 in render pipeline, gated by `no_hud`
- `src/renderer/mod.rs:121-133` — Dev overlay called after all passes (always readable)
- `src/main.rs:48-50` — `--no-hud` CLI flag parsed via clap
- `src/world/mod.rs:144-146` — `WorldState::sector()` returns `total_commits / 100`
- `src/world/speed.rs:31-39` — `VelocityTier::name()` returns display strings

### Data Sources
- `WorldState.commits_per_min` — rolling cpm from git poller
- `WorldState.lines_added`, `lines_deleted`, `files_changed` — from last poll
- `WorldState.total_commits` — lifetime count seeded from `RepoSeed`
- `WorldState.tier` — current `VelocityTier` enum variant
- `WorldState.time` — elapsed time for strobe calculation
- `RepoSeed.repo_name` — derived from origin URL or directory name

## Font System

The bitmap font in `font.rs` provides:
- 5x7 pixel glyphs for ASCII 32-126 (95 glyphs)
- `draw_char(fb, ch, x, y, color, scale)` — renders one glyph at given scale
- `draw_text(fb, text, x, y, color, scale)` — renders string with 1px spacing per scale unit
- `text_width(text, scale)` — computes pixel width for right-alignment
- Glyph spacing: `(GLYPH_W + 1) * scale` = 6px per char at 1x scale

## Rendering Pipeline Context

The HUD is drawn as the **last visual pass** (pass 12), after all effects:
1. Sky, sun, terrain, road, grid, sprites, car (passes 1-7.5)
2. Speed lines, motion blur, scanlines, bloom (passes 8-11)
3. **HUD overlay** (pass 12) — draws over post-processed frame
4. Dev overlay (optional, after HUD)

This ordering means the HUD is never affected by bloom, scanlines, or motion blur —
it stays crisp and readable at all times.

## Alpha Compositing

The HUD background uses manual alpha compositing:
```
result_channel = bg_channel * (1 - alpha) + hud_channel * alpha
```
With `HUD_BG = Rgba([0, 0, 0, 200])`, alpha = 200/255 ≈ 0.784 (~78% opacity).
The spec says 80% — this is close enough (within 2%).

## Tier Badge Colors

Implemented in `tier_badge_color()`:
- Flatline: dim gray `(100, 100, 100)`
- Cruise: white `(255, 255, 255)`
- Active: cyan `(0, 255, 255)`
- Demon: orange `(255, 165, 0)`
- VelocityDemon: red↔white strobe at 4Hz using `(world.time * 4.0) as u32 % 2`

The strobe uses `is_multiple_of(2)` which toggles every 0.25s (4Hz).

## HUD Layout

Fields are positioned at hardcoded x-offsets:
- SECTOR N at x=8
- X.X c/min at x=120
- +N -N at x=240
- N files at x=380
- Tier badge at x=480
- repo: name right-aligned at `w - text_width - 8`

Text y-position: `hud_y + 4` (4px padding from top of strip).
All text rendered at scale=1 (5x7 pixels per glyph).

## Constraints and Observations

1. Hardcoded x-positions mean fields may overlap on narrow terminals (<600px wide)
2. No separator characters between fields — relies on spacing
3. The 200 alpha value (78%) is slightly below the spec's 80% target
4. Dev overlay is independent of HUD — has its own background and positioning
5. No tests exist specifically for `draw_hud()` or `tier_badge_color()`
