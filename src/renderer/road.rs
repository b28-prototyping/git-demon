use image::{ImageBuffer, Rgba};

use crate::git::seed::RepoSeed;
use crate::world::speed::VelocityTier;
use crate::world::WorldState;

const BASE_HORIZON_RATIO: f32 = 0.35;
const ROAD_MIN_HALF: f32 = 8.0;
const ROAD_MAX_HALF: f32 = 480.0;
const RUMBLE_WIDTH: u32 = 12;
const PERSPECTIVE_SCALE: f32 = 10.0;
const STRIPE_PERIOD: f32 = 6.0;

const STRIPE_LIGHT: Rgba<u8> = Rgba([90, 90, 90, 255]);
const STRIPE_DARK: Rgba<u8> = Rgba([70, 70, 70, 255]);
const VERGE_A: Rgba<u8> = Rgba([15, 40, 15, 255]);
const VERGE_B: Rgba<u8> = Rgba([10, 30, 10, 255]);
const RUMBLE_WHITE: Rgba<u8> = Rgba([200, 200, 200, 255]);
const RUMBLE_RED: Rgba<u8> = Rgba([180, 30, 30, 255]);

pub fn horizon_ratio(world: &WorldState) -> f32 {
    if world.tier == VelocityTier::VelocityDemon {
        BASE_HORIZON_RATIO + 0.02
    } else {
        BASE_HORIZON_RATIO
    }
}

pub fn road_max_half(world: &WorldState) -> f32 {
    if world.tier == VelocityTier::VelocityDemon {
        ROAD_MAX_HALF * 1.05
    } else {
        ROAD_MAX_HALF
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn draw_road(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    world: &WorldState,
    _seed: &RepoSeed,
) {
    let max_half = road_max_half(world);
    let cx_base = w as f32 / 2.0;
    let road_rows = (h - horizon_y).max(1) as f32;
    let raw = fb.as_mut();
    let stride = w as usize * 4;

    for y in horizon_y..h {
        let depth = (y - horizon_y) as f32 / road_rows;

        let curve_shift = world.curve_offset * depth * depth;
        let cx = cx_base + curve_shift;

        let road_half = lerp(ROAD_MIN_HALF, max_half, depth);
        let road_l = (cx - road_half).max(0.0) as u32;
        let road_r = ((cx + road_half) as u32).min(w - 1);

        // Perspective projection: SCALE/depth gives world-Z for this scanline,
        // z_offset scrolls uniformly so stripes appear/disappear at equal rates.
        // Scale period with speed to prevent wagon-wheel aliasing at high multipliers.
        let period = STRIPE_PERIOD * (1.0 + world.speed * 0.05);
        let world_z = PERSPECTIVE_SCALE / depth.max(0.01) + world.z_offset;
        let stripe = (world_z % (period * 2.0)) < period;

        let road_color = if stripe { STRIPE_LIGHT } else { STRIPE_DARK };
        let verge_color = if stripe { VERGE_A } else { VERGE_B };
        let rumble_color = if stripe { RUMBLE_WHITE } else { RUMBLE_RED };

        let row_offset = y as usize * stride;
        let rumble_l = road_l.saturating_sub(RUMBLE_WIDTH);

        for x in 0..w {
            let color = if x < rumble_l {
                verge_color
            } else if x < road_l {
                rumble_color
            } else if x <= road_r {
                road_color
            } else if x <= road_r + RUMBLE_WIDTH {
                rumble_color
            } else {
                verge_color
            };
            let off = row_offset + x as usize * 4;
            if off + 3 < raw.len() {
                raw[off] = color.0[0];
                raw[off + 1] = color.0[1];
                raw[off + 2] = color.0[2];
                raw[off + 3] = 255;
            }
        }
    }
}

pub fn draw_grid(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    world: &WorldState,
    seed: &RepoSeed,
) {
    let max_half = road_max_half(world);
    let cx_base = w as f32 / 2.0;
    let accent = hue_to_neon(seed.accent_hue);

    // Horizontal grid lines at regular world-Z intervals, scrolling with camera
    let grid_spacing = 5.0_f32;
    let camera_offset = world.camera_z % grid_spacing;
    for i in 1..40 {
        let z_world = i as f32 * grid_spacing - camera_offset;
        if z_world <= 0.0 {
            continue;
        }
        let depth_scale = (1.0 - z_world / world.draw_distance()).clamp(0.0, 1.0);
        if depth_scale < 0.02 {
            continue;
        }

        let y = lerp(horizon_y as f32, h as f32, depth_scale) as u32;
        if y >= h || y < horizon_y {
            continue;
        }

        let depth = (y - horizon_y) as f32 / (h - horizon_y).max(1) as f32;
        let curve_shift = world.curve_offset * depth * depth;
        let cx = cx_base + curve_shift;

        let road_half = lerp(ROAD_MIN_HALF, max_half, depth_scale);
        let road_l = (cx - road_half).max(0.0) as u32;
        let road_r = ((cx + road_half) as u32).min(w - 1);

        let fg_a = 150u32;
        let inv_a = 255 - fg_a;
        let raw = fb.as_mut();
        let stride = w as usize * 4;
        let row_off = y as usize * stride;

        for x in road_l..=road_r {
            let off = row_off + x as usize * 4;
            if off + 3 < raw.len() {
                raw[off] = ((accent.0[0] as u32 * fg_a + raw[off] as u32 * inv_a) / 255) as u8;
                raw[off + 1] =
                    ((accent.0[1] as u32 * fg_a + raw[off + 1] as u32 * inv_a) / 255) as u8;
                raw[off + 2] =
                    ((accent.0[2] as u32 * fg_a + raw[off + 2] as u32 * inv_a) / 255) as u8;
            }
        }
    }
}

fn hue_to_neon(hue: f32) -> Rgba<u8> {
    let (r, g, b) = hsl_to_rgb_inline(hue, 1.0, 0.55);
    Rgba([r, g, b, 255])
}

fn hsl_to_rgb_inline(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
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
fn blend_alpha(bg: Rgba<u8>, fg: Rgba<u8>) -> Rgba<u8> {
    let a = fg.0[3] as f32 / 255.0;
    let inv_a = 1.0 - a;
    Rgba([
        (fg.0[0] as f32 * a + bg.0[0] as f32 * inv_a) as u8,
        (fg.0[1] as f32 * a + bg.0[1] as f32 * inv_a) as u8,
        (fg.0[2] as f32 * a + bg.0[2] as f32 * inv_a) as u8,
        255,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_seed() -> RepoSeed {
        RepoSeed {
            accent_hue: 180.0,
            saturation: 0.8,
            terrain_roughness: 0.5,
            speed_base: 0.5,
            author_colors: HashMap::new(),
            total_commits: 100,
            repo_name: "test-repo".to_string(),
        }
    }

    fn test_world(tier: VelocityTier) -> WorldState {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.tier = tier;
        w.z_offset = 10.0;
        w.curve_offset = 0.0;
        w
    }

    // --- Pure function tests ---

    #[test]
    fn test_lerp_at_zero() {
        assert_eq!(lerp(5.0, 15.0, 0.0), 5.0);
    }

    #[test]
    fn test_lerp_at_one() {
        assert_eq!(lerp(5.0, 15.0, 1.0), 15.0);
    }

    #[test]
    fn test_lerp_midpoint() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_horizon_ratio_normal() {
        let world = test_world(VelocityTier::Cruise);
        assert!((horizon_ratio(&world) - 0.35).abs() < 1e-6);
    }

    #[test]
    fn test_horizon_ratio_velocity_demon() {
        let world = test_world(VelocityTier::VelocityDemon);
        assert!((horizon_ratio(&world) - 0.37).abs() < 1e-6);
    }

    #[test]
    fn test_road_max_half_normal() {
        let world = test_world(VelocityTier::Cruise);
        assert!((road_max_half(&world) - 480.0).abs() < 1e-6);
    }

    #[test]
    fn test_road_max_half_velocity_demon() {
        let world = test_world(VelocityTier::VelocityDemon);
        assert!((road_max_half(&world) - 504.0).abs() < 0.01);
    }

    #[test]
    fn test_blend_alpha_opaque() {
        let bg = Rgba([100, 100, 100, 255]);
        let fg = Rgba([200, 50, 50, 255]);
        let result = blend_alpha(bg, fg);
        assert_eq!(result.0[0], 200);
        assert_eq!(result.0[1], 50);
        assert_eq!(result.0[2], 50);
    }

    #[test]
    fn test_blend_alpha_transparent() {
        let bg = Rgba([100, 100, 100, 255]);
        let fg = Rgba([200, 50, 50, 0]);
        let result = blend_alpha(bg, fg);
        assert_eq!(result.0[0], 100);
        assert_eq!(result.0[1], 100);
        assert_eq!(result.0[2], 100);
    }

    #[test]
    fn test_blend_alpha_half() {
        let bg = Rgba([0, 0, 0, 255]);
        let fg = Rgba([200, 100, 50, 128]);
        let result = blend_alpha(bg, fg);
        // alpha ≈ 0.502, so result ≈ fg * 0.502
        assert!((result.0[0] as i32 - 100).abs() <= 2);
        assert!((result.0[1] as i32 - 50).abs() <= 2);
        assert!((result.0[2] as i32 - 25).abs() <= 2);
    }

    #[test]
    fn test_hsl_to_rgb_red() {
        let (r, g, b) = hsl_to_rgb_inline(0.0, 1.0, 0.5);
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_hsl_to_rgb_green() {
        let (r, g, b) = hsl_to_rgb_inline(120.0, 1.0, 0.5);
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_hue_to_neon_returns_opaque() {
        let c = hue_to_neon(90.0);
        assert_eq!(c.0[3], 255);
    }

    // --- Rendering invariant tests ---

    #[test]
    fn test_draw_road_perspective_width() {
        let (w, h) = (200, 200);
        let mut fb = ImageBuffer::new(w, h);
        let world = test_world(VelocityTier::Cruise);
        let seed = test_seed();
        let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;

        draw_road(&mut fb, w, h, horizon_y, &world, &seed);

        // Count road pixels (non-verge) near horizon vs near bottom
        let count_road = |y: u32| -> u32 {
            (0..w)
                .filter(|&x| {
                    let p = fb.get_pixel(x, y);
                    *p != VERGE_A && *p != VERGE_B
                })
                .count() as u32
        };

        let near_horizon = count_road(horizon_y + 2);
        let near_bottom = count_road(h - 1);
        assert!(
            near_bottom > near_horizon,
            "Road should be wider at bottom ({near_bottom}) than near horizon ({near_horizon})"
        );
    }

    #[test]
    fn test_draw_road_stripe_colors_present() {
        let (w, h) = (200, 200);
        let mut fb = ImageBuffer::new(w, h);
        let mut world = test_world(VelocityTier::Cruise);
        world.z_offset = 5.0; // Ensure both stripe phases are hit across scanlines
        let seed = test_seed();
        let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;

        draw_road(&mut fb, w, h, horizon_y, &world, &seed);

        let has_light = fb.pixels().any(|p| *p == STRIPE_LIGHT);
        let has_dark = fb.pixels().any(|p| *p == STRIPE_DARK);
        assert!(has_light, "STRIPE_LIGHT should appear");
        assert!(has_dark, "STRIPE_DARK should appear");
    }

    #[test]
    fn test_draw_road_rumble_colors_present() {
        let (w, h) = (400, 200);
        let mut fb = ImageBuffer::new(w, h);
        let mut world = test_world(VelocityTier::Cruise);
        world.z_offset = 5.0;
        let seed = test_seed();
        let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;

        draw_road(&mut fb, w, h, horizon_y, &world, &seed);

        let has_white = fb.pixels().any(|p| *p == RUMBLE_WHITE);
        let has_red = fb.pixels().any(|p| *p == RUMBLE_RED);
        assert!(has_white, "RUMBLE_WHITE should appear");
        assert!(has_red, "RUMBLE_RED should appear");
    }

    #[test]
    fn test_draw_road_verge_colors_present() {
        let (w, h) = (400, 200);
        let mut fb = ImageBuffer::new(w, h);
        let mut world = test_world(VelocityTier::Cruise);
        world.z_offset = 5.0;
        let seed = test_seed();
        let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;

        draw_road(&mut fb, w, h, horizon_y, &world, &seed);

        let has_a = fb.pixels().any(|p| *p == VERGE_A);
        let has_b = fb.pixels().any(|p| *p == VERGE_B);
        assert!(has_a, "VERGE_A should appear");
        assert!(has_b, "VERGE_B should appear");
    }

    #[test]
    fn test_draw_road_curve_shifts_center() {
        // Buffer must be wider than 2 * ROAD_MAX_HALF (960) so road doesn't fill entire row
        let (w, h) = (1200, 200);
        let seed = test_seed();

        // Render with no curve
        let mut fb_straight = ImageBuffer::new(w, h);
        let mut world_straight = test_world(VelocityTier::Cruise);
        world_straight.curve_offset = 0.0;
        let horizon_y = (h as f32 * horizon_ratio(&world_straight)) as u32;
        draw_road(&mut fb_straight, w, h, horizon_y, &world_straight, &seed);

        // Render with positive curve
        let mut fb_curved = ImageBuffer::new(w, h);
        let mut world_curved = test_world(VelocityTier::Cruise);
        world_curved.curve_offset = 50.0;
        draw_road(&mut fb_curved, w, h, horizon_y, &world_curved, &seed);

        // Find road center at bottom row (midpoint of non-verge pixels)
        let bottom_y = h - 1;
        let road_center = |fb: &ImageBuffer<Rgba<u8>, Vec<u8>>| -> f32 {
            let road_pixels: Vec<u32> = (0..w)
                .filter(|&x| {
                    let p = fb.get_pixel(x, bottom_y);
                    *p != VERGE_A && *p != VERGE_B
                })
                .collect();
            if road_pixels.is_empty() {
                return w as f32 / 2.0;
            }
            (road_pixels[0] + road_pixels[road_pixels.len() - 1]) as f32 / 2.0
        };

        let center_straight = road_center(&fb_straight);
        let center_curved = road_center(&fb_curved);
        assert!(center_curved > center_straight, "Positive curve should shift road center right: straight={center_straight}, curved={center_curved}");
    }

    #[test]
    fn test_draw_road_velocity_demon_wider() {
        // Buffer must be wider than 2 * 504 (VelocityDemon max half) = 1008
        let (w, h) = (1200, 200);
        let seed = test_seed();

        let count_road_at_bottom = |tier: VelocityTier| -> u32 {
            let mut fb = ImageBuffer::new(w, h);
            let world = test_world(tier);
            let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;
            draw_road(&mut fb, w, h, horizon_y, &world, &seed);
            let bottom_y = h - 1;
            (0..w)
                .filter(|&x| {
                    let p = fb.get_pixel(x, bottom_y);
                    *p != VERGE_A && *p != VERGE_B
                })
                .count() as u32
        };

        let normal_width = count_road_at_bottom(VelocityTier::Cruise);
        let demon_width = count_road_at_bottom(VelocityTier::VelocityDemon);
        assert!(
            demon_width > normal_width,
            "VelocityDemon road should be wider: normal={normal_width}, demon={demon_width}"
        );
    }

    #[test]
    fn test_draw_road_no_panic_extreme_curve() {
        let (w, h) = (100, 100);
        let mut fb = ImageBuffer::new(w, h);
        let seed = test_seed();

        let mut world = test_world(VelocityTier::Cruise);
        world.curve_offset = -80.0;
        let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;
        draw_road(&mut fb, w, h, horizon_y, &world, &seed);
        // If we get here without panic, the test passes

        world.curve_offset = 80.0;
        draw_road(&mut fb, w, h, horizon_y, &world, &seed);
        // Same — no panic with extreme positive offset
    }

    #[test]
    fn test_draw_grid_accent_pixels() {
        let (w, h) = (400, 200);
        let seed = test_seed();
        let world = test_world(VelocityTier::Cruise);
        let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;

        // Render road only
        let mut fb_road = ImageBuffer::new(w, h);
        draw_road(&mut fb_road, w, h, horizon_y, &world, &seed);

        // Render road + grid
        let mut fb_grid = ImageBuffer::new(w, h);
        draw_road(&mut fb_grid, w, h, horizon_y, &world, &seed);
        draw_grid(&mut fb_grid, w, h, horizon_y, &world, &seed);

        // Count pixels that changed between road-only and road+grid
        let changed = fb_road
            .enumerate_pixels()
            .filter(|(x, y, p)| *y > horizon_y && fb_grid.get_pixel(*x, *y) != *p)
            .count();

        assert!(
            changed > 0,
            "Grid should modify at least some pixels on road"
        );
    }

    #[test]
    fn test_draw_grid_alpha_blended() {
        let (w, h) = (400, 200);
        let mut fb = ImageBuffer::new(w, h);
        let seed = test_seed();
        let world = test_world(VelocityTier::Cruise);
        let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;

        draw_road(&mut fb, w, h, horizon_y, &world, &seed);
        draw_grid(&mut fb, w, h, horizon_y, &world, &seed);

        // Grid lines should NOT be pure accent color (they're alpha-blended)
        let accent = hue_to_neon(seed.accent_hue);
        let pure_accent_count = fb
            .pixels()
            .filter(|p| p.0[0] == accent.0[0] && p.0[1] == accent.0[1] && p.0[2] == accent.0[2])
            .count();
        // Some might accidentally match, but it shouldn't be many
        assert!(
            pure_accent_count < 10,
            "Grid pixels should be blended, not pure accent ({pure_accent_count} pure)"
        );
    }

    #[test]
    fn test_draw_grid_lines_move_with_camera() {
        let (w, h) = (400, 200);
        let seed = test_seed();

        // Helper: render road+grid, return set of Y rows that grid modified
        let grid_rows = |camera_z: f32| -> std::collections::HashSet<u32> {
            let mut world = test_world(VelocityTier::Cruise);
            world.camera_z = camera_z;
            let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;

            let mut fb_road = ImageBuffer::new(w, h);
            draw_road(&mut fb_road, w, h, horizon_y, &world, &seed);

            let mut fb_grid = ImageBuffer::new(w, h);
            draw_road(&mut fb_grid, w, h, horizon_y, &world, &seed);
            draw_grid(&mut fb_grid, w, h, horizon_y, &world, &seed);

            let mut rows = std::collections::HashSet::new();
            for y in (horizon_y + 1)..h {
                for x in 0..w {
                    if fb_grid.get_pixel(x, y) != fb_road.get_pixel(x, y) {
                        rows.insert(y);
                        break;
                    }
                }
            }
            rows
        };

        let rows_a = grid_rows(0.0);
        let rows_b = grid_rows(2.5); // half grid_spacing

        assert!(
            !rows_a.is_empty(),
            "Grid should produce visible lines at camera_z=0"
        );
        assert!(
            !rows_b.is_empty(),
            "Grid should produce visible lines at camera_z=2.5"
        );
        assert_ne!(
            rows_a, rows_b,
            "Grid lines should be at different Y positions when camera_z differs"
        );
    }

    #[test]
    fn test_draw_grid_line_count_stable() {
        let (w, h) = (400, 200);
        let seed = test_seed();

        let count_grid_rows = |camera_z: f32| -> usize {
            let mut world = test_world(VelocityTier::Cruise);
            world.camera_z = camera_z;
            let horizon_y = (h as f32 * horizon_ratio(&world)) as u32;

            let mut fb_road = ImageBuffer::new(w, h);
            draw_road(&mut fb_road, w, h, horizon_y, &world, &seed);

            let mut fb_grid = ImageBuffer::new(w, h);
            draw_road(&mut fb_grid, w, h, horizon_y, &world, &seed);
            draw_grid(&mut fb_grid, w, h, horizon_y, &world, &seed);

            let mut count = 0;
            for y in (horizon_y + 1)..h {
                for x in 0..w {
                    if fb_grid.get_pixel(x, y) != fb_road.get_pixel(x, y) {
                        count += 1;
                        break;
                    }
                }
            }
            count
        };

        let counts: Vec<usize> = [0.0, 1.0, 2.5, 4.9, 5.0, 10.3]
            .iter()
            .map(|&cz| count_grid_rows(cz))
            .collect();

        let min = *counts.iter().min().unwrap();
        let max = *counts.iter().max().unwrap();
        assert!(
            max - min <= 1,
            "Grid line count should stay stable across camera positions, got counts: {counts:?}"
        );
    }
}
