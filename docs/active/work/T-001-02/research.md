# Research — T-001-02 sky-and-sun

## Scope

This ticket covers the sky gradient, sun/moon disc, VelocityDemon hue rotation, sun pulsing, horizon bloom bleed, and HSL helper unit tests.

## Existing Implementation

### sky.rs (src/renderer/sky.rs, 99 lines)

The module already contains two public functions and four private helpers:

**`draw_sky(fb, w, h, horizon_y, seed, world)`** — lines 8–29
- Fills rows 0..horizon_y with a vertical gradient
- Zenith constant: `SKY_ZENITH = Rgba([5, 5, 15, 255])` — near-black blue
- Horizon color: `hue_to_dark_rgb(seed.accent_hue, world)` → HSL(hue, 0.4, 0.08)
- Linear interpolation per-channel via `lerp_u8`
- Gradient parameter `t = y / horizon_y` — top is zenith, bottom is horizon

**`draw_sun(fb, w, horizon_y, seed, world)`** — lines 31–58
- Center: `(w * 0.72, horizon_y - 40)` — fixed position upper-right
- Complement hue: `(seed.accent_hue + 180.0) % 360.0`
- Radius: `18.0 + pulse * 3.0` where pulse = `sin(time * freq * TAU)`
- Frequency: `(commits_per_min * 0.4).min(3.0)` — capped at 3 Hz
- Rasterized as filled circle: `dx² + dy² <= r²`
- Color: `hue_to_bright_rgb(complement_hue)` → HSL(hue, 0.9, 0.7)

**`hue_to_dark_rgb(hue, world)`** — lines 61–69
- At VelocityDemon (tier as u8 >= 4): rotates hue by `world.time` degrees
- Since `world.time` increments by `dt` each frame (~16.7ms at 60fps), rotation is ~1°/s when `dt` sums to 1.0 per second — correct

**`hue_to_bright_rgb(hue)`** — lines 71–74
- HSL(hue, 0.9, 0.7) — bright pastel for sun disc

**`lerp_u8(a, b, t)`** — line 76–78
- Clamped linear interpolation for u8 channels

**`hsl_to_rgb(h, s, l)`** — lines 80–98
- Standard HSL→RGB algorithm using chroma/intermediate decomposition
- Correct: matches the canonical six-sector mapping

### Render Pipeline (src/renderer/mod.rs)

The render order is:
1. `sky::draw_sky()` — gradient background
2. `sky::draw_sun()` — sun disc over gradient
3. `terrain::draw_terrain()` — verge silhouettes over sky edges
4. `road::draw_road()` — road surface from horizon_y down
5. `road::draw_grid()` — neon grid lines over road
6. `sprites::draw_sprites()` — roadside objects
7. `effects::draw_speed_lines()` — radial streaks (Demon+)
8. `effects::apply_motion_blur()` — temporal blend with prev frame
9. `effects::apply_scanline_filter()` — darken even rows
10. `effects::apply_bloom()` — 3x3 additive blur on bright pixels
11. `hud::draw_hud()` — bottom overlay

Sky is passes 1–2. The bloom bleed at the horizon is NOT currently implemented.

### Horizon Position (src/renderer/road.rs)

`horizon_ratio(world)` returns 0.35 normally, 0.37 at VelocityDemon.
`horizon_y = (h * horizon_ratio) as u32` — computed in mod.rs line 48.

### Road Accent Color

Grid lines use `hue_to_neon(seed.accent_hue)` → HSL(hue, 1.0, 0.55) — full-saturation neon.
This is the "road accent color" referenced in the bloom bleed acceptance criterion.

### WorldState Fields Used by Sky

- `world.time: f32` — elapsed seconds, used for hue rotation and sun pulse
- `world.tier: VelocityTier` — checked for VelocityDemon threshold
- `world.commits_per_min: f32` — drives sun pulse frequency

### RepoSeed Fields Used by Sky

- `seed.accent_hue: f32` — 0–360, deterministic from repo identity hash

### Duplicated Code

The `hsl_to_rgb` function is implemented identically in:
- `sky.rs` (lines 80–98)
- `road.rs` as `hsl_to_rgb_inline` (lines 136–154)
- `effects.rs` as `hue_to_rgb` (lines 125–142, simplified s=1 l=0.5 case)
- `git/seed.rs` (likely similar)

All copies use the same correct algorithm. No external color library in Cargo.toml.

### Test Infrastructure

No existing tests in the project — no `tests/` directory, no `#[cfg(test)]` modules found.

## What Exists vs What's Missing

| Acceptance Criterion | Status | Location |
|---|---|---|
| `draw_sky` fills sky with gradient zenith→horizon | ✅ Done | sky.rs:8–29 |
| Gradient uses correct HSL→RGB conversion | ✅ Done | sky.rs:80–98 |
| VelocityDemon sky hue rotation at 1°/s | ✅ Done | sky.rs:62–66 |
| `draw_sun` at (0.72w, horizon_y-40) complement hue | ✅ Done | sky.rs:38–45 |
| Sun radius pulses ±3px at (cpm*0.4).min(3.0) Hz | ✅ Done | sky.rs:40–41 |
| 6px bloom bleed of road accent at sky bottom | ❌ Missing | — |
| HSL helpers are correct and tested | ⚠️ Partial | Correct but untested |

## Constraints

- No external color library — all color math is inline
- Framebuffer is `ImageBuffer<Rgba<u8>>` — all operations via `put_pixel`/`get_pixel`
- Sky is drawn first, terrain and road overwrite parts of it afterward
- Bloom bleed must be drawn in sky pass (before road overwrites bottom rows)
- The bloom bleed is at the seam between sky and road — row `horizon_y` and neighboring rows
