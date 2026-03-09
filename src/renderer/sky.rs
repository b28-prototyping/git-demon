use image::{ImageBuffer, Rgba};

use crate::git::seed::RepoSeed;
use crate::world::WorldState;

const SKY_ZENITH: Rgba<u8> = Rgba([5, 5, 15, 255]);

pub fn draw_sky(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    seed: &RepoSeed,
    world: &WorldState,
) {
    let horizon_color = hue_to_dark_rgb(seed.accent_hue, world);

    for y in 0..horizon_y.min(h) {
        let t = y as f32 / horizon_y.max(1) as f32;
        let r = lerp_u8(SKY_ZENITH.0[0], horizon_color.0[0], t);
        let g = lerp_u8(SKY_ZENITH.0[1], horizon_color.0[1], t);
        let b = lerp_u8(SKY_ZENITH.0[2], horizon_color.0[2], t);
        let color = Rgba([r, g, b, 255]);

        for x in 0..w {
            fb.put_pixel(x, y, color);
        }
    }
}

/// Procedural starfield that doubles as the warp-speed effect.
/// At rest: twinkling dots scattered across the sky.
/// At speed: stars stretch into radial streaks from the vanishing point,
/// creating an asteroid-field / hyperspace tunnel look.
pub fn draw_stars(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    horizon_y: u32,
    world: &WorldState,
) {
    if horizon_y == 0 || w == 0 {
        return;
    }
    let star_count = 600u32;
    // Stars converge at 60% of horizon — intentional parallax offset
    // so the distant starfield appears to have a different focal point
    // than the road/ground projection.
    let cx = w as f32 / 2.0;
    let pitch_shift = world.camera.pitch_offset(0.0, horizon_y);
    let cy = horizon_y as f32 * 0.6 + pitch_shift;
    let twinkle_phase = world.time * 3.0;

    let speed_t = (world.speed / 300.0).clamp(0.0, 1.0);
    let streak_len = speed_t * 40.0;

    let fb_h = fb.height();
    let raw = fb.as_mut();
    let stride = w as usize * 4;
    let h_limit = horizon_y.min(fb_h);

    for i in 0..star_count {
        let hash = i.wrapping_mul(2654435761);
        let hash2 = i.wrapping_mul(1664525).wrapping_add(1013904223);

        let angle = (hash as f32 / u32::MAX as f32) * std::f32::consts::TAU;
        let dist_frac = (hash2 as f32 / u32::MAX as f32).sqrt();
        let max_dist = (cx.max(cy) * 1.5).max(100.0);
        let dist = 10.0 + dist_frac * max_dist;

        let sx = cx + angle.cos() * dist;
        let sy = cy + angle.sin() * dist;

        if sy < 0.0 || sy >= horizon_y as f32 || sx < 0.0 || sx >= w as f32 {
            continue;
        }

        // Brightness — cheap integer twinkle instead of sin()
        let base_bright = ((hash >> 5) & 0xFF) as u16;
        let twinkle_idx = ((twinkle_phase * 256.0) as u32).wrapping_add(hash >> 3);
        let twinkle = 180 + (twinkle_idx % 76) as u16; // 180..255 range
        let bright = base_bright * twinkle / 255;

        // Dimmer near horizon — integer approx
        let sy_frac = (sy * 256.0 / horizon_y as f32) as u32;
        let fade = 256u32.saturating_sub(sy_frac * sy_frac / 256);
        let b = ((bright as u32 * fade / 256) as u16).min(255);
        if b < 20 {
            continue;
        }
        let b_blue = (b * 4 / 3).min(255);

        if streak_len < 1.0 {
            let px = sx as u32;
            let py = sy as u32;
            if px < w && py < h_limit {
                let off = py as usize * stride + px as usize * 4;
                if off + 3 < raw.len() {
                    raw[off] = (raw[off] as u16 + b).min(255) as u8;
                    raw[off + 1] = (raw[off + 1] as u16 + b).min(255) as u8;
                    raw[off + 2] = (raw[off + 2] as u16 + b_blue).min(255) as u8;
                }
            }
        } else {
            let dx = sx - cx;
            let dy = sy - cy;
            let len = dx.hypot(dy).max(0.01);
            let ndx = dx / len;
            let ndy = dy / len;

            let this_streak = streak_len * (dist / max_dist).min(1.0);
            let steps = this_streak as i32;
            // Step by 2 at high speed to halve pixel ops
            let step_size = if steps > 10 { 2 } else { 1 };

            let mut s = 0;
            while s <= steps {
                let a = (b as u32 * (steps - s * 7 / 10) as u32 / steps.max(1) as u32) as u16;
                let a_blue = (a * 4 / 3).min(255);

                let px = (sx + ndx * s as f32) as i32;
                let py = (sy + ndy * s as f32) as i32;
                if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h_limit {
                    let off = py as usize * stride + px as usize * 4;
                    if off + 3 < raw.len() {
                        raw[off] = (raw[off] as u16 + a).min(255) as u8;
                        raw[off + 1] = (raw[off + 1] as u16 + a).min(255) as u8;
                        raw[off + 2] = (raw[off + 2] as u16 + a_blue).min(255) as u8;
                    }
                }
                s += step_size;
            }
        }
    }
}

pub fn draw_sun(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    horizon_y: u32,
    seed: &RepoSeed,
    world: &WorldState,
) {
    let cx = (w as f32 * 0.72) as i32;
    let pitch_shift = world.camera.pitch_offset(0.05, horizon_y);
    let cy = horizon_y as i32 - 40 + pitch_shift as i32;
    let pulse = (world.time * (world.commits_per_min * 0.4).min(3.0) * std::f32::consts::TAU).sin();
    let radius = 18.0 + pulse * 3.0;
    let r2 = radius * radius;

    let complement_hue = (seed.accent_hue + 180.0) % 360.0;
    let sun_color = hue_to_bright_rgb(complement_hue);

    let ri = radius as i32 + 1;
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            if (dx * dx + dy * dy) as f32 <= r2 {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 && (px as u32) < fb.width() && (py as u32) < fb.height() {
                    fb.put_pixel(px as u32, py as u32, sun_color);
                }
            }
        }
    }
}

pub fn draw_bloom_bleed(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    horizon_y: u32,
    seed: &RepoSeed,
    world: &WorldState,
) {
    let bleed_rows = 6u32;
    let pitch_shift = world.camera.pitch_offset(0.05, horizon_y) as i32;
    let start_y = (horizon_y as i32 - bleed_rows as i32 + pitch_shift).max(0) as u32;
    let hue = if world.tier as u8 >= 4 {
        (seed.accent_hue + world.time) % 360.0
    } else {
        seed.accent_hue
    };
    let (ar, ag, ab) = hsl_to_rgb(hue, 0.9, 0.35);

    for y in start_y..horizon_y.min(fb.height()) {
        let t = (y - start_y) as f32 / bleed_rows as f32;
        let alpha = (t * 80.0) as u8;
        let fg = Rgba([ar, ag, ab, alpha]);
        for x in 0..w {
            let bg = *fb.get_pixel(x, y);
            fb.put_pixel(x, y, blend_alpha(bg, fg));
        }
    }
}

fn blend_alpha(bg: Rgba<u8>, fg: Rgba<u8>) -> Rgba<u8> {
    let a = fg.0[3] as f32 / 255.0;
    let inv = 1.0 - a;
    Rgba([
        (fg.0[0] as f32 * a + bg.0[0] as f32 * inv) as u8,
        (fg.0[1] as f32 * a + bg.0[1] as f32 * inv) as u8,
        (fg.0[2] as f32 * a + bg.0[2] as f32 * inv) as u8,
        255,
    ])
}

fn hue_to_dark_rgb(hue: f32, world: &WorldState) -> Rgba<u8> {
    let h = if world.tier as u8 >= 4 {
        (hue + world.time) % 360.0
    } else {
        hue
    };
    let (r, g, b) = hsl_to_rgb(h, 0.4, 0.08);
    Rgba([r, g, b, 255])
}

fn hue_to_bright_rgb(hue: f32) -> Rgba<u8> {
    let (r, g, b) = hsl_to_rgb(hue, 0.9, 0.7);
    Rgba([r, g, b, 255])
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0) as u8
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h2 = h / 60.0;
    let x = c * (1.0 - (h2 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h2 as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsl_to_rgb_primaries() {
        assert_eq!(hsl_to_rgb(0.0, 1.0, 0.5), (255, 0, 0));
        assert_eq!(hsl_to_rgb(120.0, 1.0, 0.5), (0, 255, 0));
        assert_eq!(hsl_to_rgb(240.0, 1.0, 0.5), (0, 0, 255));
    }

    #[test]
    fn test_hsl_to_rgb_secondaries() {
        assert_eq!(hsl_to_rgb(60.0, 1.0, 0.5), (255, 255, 0));
        assert_eq!(hsl_to_rgb(180.0, 1.0, 0.5), (0, 255, 255));
        assert_eq!(hsl_to_rgb(300.0, 1.0, 0.5), (255, 0, 255));
    }

    #[test]
    fn test_hsl_to_rgb_achromatic() {
        assert_eq!(hsl_to_rgb(0.0, 0.0, 0.0), (0, 0, 0));
        assert_eq!(hsl_to_rgb(0.0, 0.0, 1.0), (255, 255, 255));
        let gray = hsl_to_rgb(0.0, 0.0, 0.5);
        assert!((gray.0 as i16 - 127).unsigned_abs() <= 1);
        assert_eq!(gray.0, gray.1);
        assert_eq!(gray.1, gray.2);
    }

    #[test]
    fn test_hsl_to_rgb_dark_horizon() {
        // HSL(0, 0.4, 0.08) — dark red used for horizon when accent_hue=0
        let (r, g, b) = hsl_to_rgb(0.0, 0.4, 0.08);
        assert!(r > g && r > b, "red channel should dominate for hue=0");
        assert!(
            r <= 30,
            "lightness 0.08 should produce very dark color, got r={r}"
        );
    }

    #[test]
    fn test_lerp_u8_basic() {
        assert_eq!(lerp_u8(0, 255, 0.0), 0);
        assert_eq!(lerp_u8(0, 255, 1.0), 255);
        let mid = lerp_u8(0, 255, 0.5);
        assert!((mid as i16 - 127).unsigned_abs() <= 1);
    }

    #[test]
    fn test_lerp_u8_clamp() {
        assert_eq!(lerp_u8(0, 255, -1.0), 0);
        assert_eq!(lerp_u8(0, 255, 2.0), 255);
    }

    #[test]
    fn test_blend_alpha_fully_transparent() {
        let bg = Rgba([100, 150, 200, 255]);
        let fg = Rgba([255, 0, 0, 0]);
        let result = blend_alpha(bg, fg);
        assert_eq!(result.0[0], 100);
        assert_eq!(result.0[1], 150);
        assert_eq!(result.0[2], 200);
    }

    #[test]
    fn test_blend_alpha_fully_opaque() {
        let bg = Rgba([100, 150, 200, 255]);
        let fg = Rgba([255, 0, 0, 255]);
        let result = blend_alpha(bg, fg);
        assert_eq!(result.0[0], 255);
        assert_eq!(result.0[1], 0);
        assert_eq!(result.0[2], 0);
    }

    #[test]
    fn test_stars_shift_with_pitch() {
        use crate::git::seed::RepoSeed;
        use std::collections::HashMap;

        let seed = RepoSeed {
            accent_hue: 180.0,
            saturation: 0.8,
            terrain_roughness: 0.5,
            speed_base: 0.5,
            author_colors: HashMap::new(),
            total_commits: 100,
            repo_name: "test-repo".to_string(),
        };

        let render = |pitch: f32| -> Vec<u8> {
            let mut world = crate::world::WorldState::new(&seed);
            world.camera.pitch = pitch;
            let (w, horizon_y) = (200u32, 80u32);
            let mut fb = ImageBuffer::from_pixel(w, horizon_y, Rgba([0, 0, 0, 255]));
            draw_stars(&mut fb, w, horizon_y, &world);
            fb.into_raw()
        };

        let no_pitch = render(0.0);
        let with_pitch = render(0.3);
        assert_ne!(
            no_pitch, with_pitch,
            "pitch should shift star convergence point"
        );
    }
}
