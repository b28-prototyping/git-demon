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

pub fn draw_sun(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    horizon_y: u32,
    seed: &RepoSeed,
    world: &WorldState,
) {
    let cx = (w as f32 * 0.72) as i32;
    let cy = horizon_y as i32 - 40;
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
    let start_y = horizon_y.saturating_sub(bleed_rows);
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
}
