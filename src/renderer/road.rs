use image::{ImageBuffer, Rgba};

use super::road_table::RoadRow;
use crate::git::seed::RepoSeed;
use crate::world::speed::VelocityTier;
use crate::world::WorldState;

const BASE_HORIZON_RATIO: f32 = 0.25;
const ROAD_MAX_HALF: f32 = 480.0;

pub fn horizon_ratio(world: &WorldState) -> f32 {
    let base = if world.tier == VelocityTier::VelocityDemon {
        BASE_HORIZON_RATIO + 0.02
    } else {
        BASE_HORIZON_RATIO
    };
    // Sprint FOV: horizon drops at high speed for a sense of velocity.
    let speed_t = (world.speed / 300.0).clamp(0.0, 1.0);
    base - speed_t * 0.06
}

pub fn road_max_half(world: &WorldState) -> f32 {
    if world.tier == VelocityTier::VelocityDemon {
        ROAD_MAX_HALF * 1.05
    } else {
        ROAD_MAX_HALF
    }
}

/// Fill below horizon with ocean surface.
/// The ocean is a flat background fill — hill slope offsets are applied
/// only to grid lines and objects, not to the base ocean gradient.
pub fn draw_road(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    world: &WorldState,
    _seed: &RepoSeed,
    _road_table: &[RoadRow],
) {
    let raw = fb.as_mut();
    let stride = w as usize * 4;
    let road_rows = (h - horizon_y).max(1) as f32;

    // One wave value per row (cheap) instead of per pixel
    let z_phase = world.z_offset * 0.05;

    for y in horizon_y..h {
        let depth = (y - horizon_y) as f32 / road_rows;
        // Per-row wave shimmer — single sin per scanline
        let wave = ((y as f32 * 0.3 + z_phase).sin() * 3.0) as i16;
        // Ocean gradient: dark deep blue near horizon, lighter near camera
        let r = ((2.0 + depth * 8.0) as i16 + wave).clamp(0, 255) as u8;
        let g = ((8.0 + depth * 25.0) as i16 + wave).clamp(0, 255) as u8;
        let b = ((25.0 + depth * 55.0) as i16 + wave * 2).clamp(0, 255) as u8;

        let row_offset = y as usize * stride;
        for x in 0..w {
            let off = row_offset + x as usize * 4;
            if off + 3 < raw.len() {
                raw[off] = r;
                raw[off + 1] = g;
                raw[off + 2] = b;
                raw[off + 3] = 255;
            }
        }
    }
}

/// Perspective grid lines on the void floor (Tron-style).
pub fn draw_grid(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    world: &WorldState,
    seed: &RepoSeed,
    road_table: &[RoadRow],
) {
    let cx_base = w as f32 / 2.0;
    let accent = hue_to_neon(seed.accent_hue);
    let cam = &world.camera;
    let raw = fb.as_mut();
    let stride = w as usize * 4;

    // --- Horizontal grid lines at regular world-Z intervals ---
    let grid_spacing = 125.0_f32;
    let camera_offset = world.camera_z % grid_spacing;
    for i in 1..80 {
        let z_rel = i as f32 * grid_spacing - camera_offset;
        if z_rel <= 0.0 {
            continue;
        }
        let z_world = world.camera_z + z_rel;
        let Some((sy, depth_scale)) = cam.project(z_world, h, horizon_y) else {
            continue;
        };
        if depth_scale < 0.005 {
            continue;
        }

        // Apply slope offset from road table
        let row_idx = (sy as u32).saturating_sub(horizon_y) as usize;
        let slope_off = road_table
            .get(row_idx)
            .map(|r| r.slope_offset)
            .unwrap_or(0.0);
        let dest_y = (sy + slope_off) as i32;
        if dest_y < horizon_y as i32 || dest_y >= h as i32 {
            continue;
        }
        let y = dest_y as u32;

        let fg_a = (depth_scale * 70.0) as u32;
        let inv_a = 255 - fg_a;
        let row_off = y as usize * stride;

        for x in 0..w {
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

    // --- Vertical grid lines converging to vanishing point ---
    // Use per-row curve offsets from road table for segment-aware curvature.
    let road_rows = (h - horizon_y).max(1) as f32;
    let num_vert = 24i32;
    let half_n = num_vert / 2;

    for i in 0..num_vert {
        let lane = (i as f32 - half_n as f32 + 0.5) / half_n as f32;

        for y in (horizon_y + 1)..h {
            let row_idx = (y - horizon_y) as usize;
            let (depth_scale, curve_shift, slope_off) = if let Some(row) = road_table.get(row_idx) {
                (
                    row.depth_scale,
                    row.curve_offset * row.depth_scale * row.depth_scale,
                    row.slope_offset,
                )
            } else {
                let ds = (y - horizon_y) as f32 / road_rows;
                (ds, world.curve_offset * ds * ds, 0.0)
            };
            let dest_y = (y as f32 + slope_off) as i32;
            if dest_y < horizon_y as i32 || dest_y >= h as i32 {
                continue;
            }
            let cx = cx_base + curve_shift;
            let spread = cam.road_half(depth_scale) * 1.5;
            let x = (cx + lane * spread) as i32;

            if x >= 0 && (x as u32) < w {
                let fg_a = (depth_scale * 30.0) as u32;
                let inv_a = 255 - fg_a;
                let off = dest_y as usize * stride + x as usize * 4;
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
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

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
        w.speed = 0.0;
        w.z_offset = 10.0;
        w.curve_offset = 0.0;
        w.camera.sync(w.speed, w.tier);
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
    fn test_camera_horizon_ratio_normal() {
        let world = test_world(VelocityTier::Cruise);
        assert!((world.camera.horizon_ratio - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_camera_horizon_ratio_velocity_demon() {
        let world = test_world(VelocityTier::VelocityDemon);
        assert!((world.camera.horizon_ratio - 0.27).abs() < 1e-6);
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

    // --- Void floor tests ---

    #[test]
    fn test_draw_road_fills_below_horizon() {
        let (w, h) = (200, 200);
        let mut fb = ImageBuffer::new(w, h);
        let world = test_world(VelocityTier::Cruise);
        let seed = test_seed();
        let horizon_y = world.camera.horizon_y(h);

        draw_road(&mut fb, w, h, horizon_y, &world, &seed, &[]);

        for y in horizon_y..h {
            for x in 0..w {
                let p = fb.get_pixel(x, y);
                assert_eq!(p.0[3], 255, "Void pixel at ({x},{y}) should be opaque");
                assert!(
                    p.0[0] > 0 || p.0[1] > 0 || p.0[2] > 0,
                    "Void pixel at ({x},{y}) should have some brightness"
                );
            }
        }
    }

    #[test]
    fn test_draw_road_void_gradient() {
        let (w, h) = (200, 200);
        let mut fb = ImageBuffer::new(w, h);
        let world = test_world(VelocityTier::Cruise);
        let seed = test_seed();
        let horizon_y = world.camera.horizon_y(h);

        draw_road(&mut fb, w, h, horizon_y, &world, &seed, &[]);

        let near = fb.get_pixel(w / 2, h - 1);
        let far = fb.get_pixel(w / 2, horizon_y);
        assert!(
            near.0[0] > far.0[0],
            "Near void ({}) should be brighter than far void ({})",
            near.0[0],
            far.0[0]
        );
    }

    #[test]
    fn test_draw_road_sky_untouched() {
        let (w, h) = (200, 200);
        let marker = Rgba([42, 42, 42, 255]);
        let mut fb = ImageBuffer::from_pixel(w, h, marker);
        let world = test_world(VelocityTier::Cruise);
        let seed = test_seed();
        let horizon_y = world.camera.horizon_y(h);

        draw_road(&mut fb, w, h, horizon_y, &world, &seed, &[]);

        for y in 0..horizon_y {
            for x in 0..w {
                assert_eq!(
                    *fb.get_pixel(x, y),
                    marker,
                    "Sky pixel ({x},{y}) should be untouched"
                );
            }
        }
    }

    #[test]
    fn test_draw_road_no_panic_extreme_curve() {
        let (w, h) = (100, 100);
        let mut fb = ImageBuffer::new(w, h);
        let seed = test_seed();
        let mut world = test_world(VelocityTier::Cruise);
        world.curve_offset = -80.0;
        let horizon_y = world.camera.horizon_y(h);
        draw_road(&mut fb, w, h, horizon_y, &world, &seed, &[]);

        world.curve_offset = 80.0;
        draw_road(&mut fb, w, h, horizon_y, &world, &seed, &[]);
    }

    // --- Grid tests ---

    #[test]
    fn test_draw_grid_modifies_pixels() {
        let (w, h) = (400, 200);
        let seed = test_seed();
        let world = test_world(VelocityTier::Cruise);
        let horizon_y = world.camera.horizon_y(h);

        let mut fb_road = ImageBuffer::new(w, h);
        draw_road(&mut fb_road, w, h, horizon_y, &world, &seed, &[]);

        let mut fb_grid = ImageBuffer::new(w, h);
        draw_road(&mut fb_grid, w, h, horizon_y, &world, &seed, &[]);
        draw_grid(&mut fb_grid, w, h, horizon_y, &world, &seed, &[]);

        let changed = fb_road
            .enumerate_pixels()
            .filter(|(x, y, p)| *y > horizon_y && fb_grid.get_pixel(*x, *y) != *p)
            .count();
        assert!(changed > 0, "Grid should modify at least some pixels");
    }

    #[test]
    fn test_draw_grid_lines_move_with_camera() {
        let (w, h) = (400, 200);
        let seed = test_seed();

        // Count horizontal grid line rows (full-width changes)
        let hline_rows = |camera_z: f32| -> std::collections::HashSet<u32> {
            let mut world = test_world(VelocityTier::Cruise);
            world.camera_z = camera_z;
            world.camera.z = camera_z;
            let horizon_y = world.camera.horizon_y(h);

            let mut fb_road = ImageBuffer::new(w, h);
            draw_road(&mut fb_road, w, h, horizon_y, &world, &seed, &[]);

            let mut fb_grid = ImageBuffer::new(w, h);
            draw_road(&mut fb_grid, w, h, horizon_y, &world, &seed, &[]);
            draw_grid(&mut fb_grid, w, h, horizon_y, &world, &seed, &[]);

            let mut rows = std::collections::HashSet::new();
            for y in (horizon_y + 1)..h {
                // Count how many pixels changed on this row
                let changed = (0..w)
                    .filter(|&x| fb_grid.get_pixel(x, y) != fb_road.get_pixel(x, y))
                    .count();
                // Horizontal line modifies most of the row
                if changed > w as usize / 2 {
                    rows.insert(y);
                }
            }
            rows
        };

        let rows_a = hline_rows(0.0);
        let rows_b = hline_rows(62.5);

        assert!(
            !rows_a.is_empty(),
            "Grid should produce visible hlines at camera_z=0"
        );
        assert!(
            !rows_b.is_empty(),
            "Grid should produce visible hlines at camera_z=62.5"
        );
        assert_ne!(
            rows_a, rows_b,
            "Hlines should be at different Y positions when camera_z differs"
        );
    }

    #[test]
    fn test_draw_grid_hline_count_stable() {
        let (w, h) = (400, 200);
        let seed = test_seed();

        let count_hlines = |camera_z: f32| -> usize {
            let mut world = test_world(VelocityTier::Cruise);
            world.camera_z = camera_z;
            world.camera.z = camera_z;
            let horizon_y = world.camera.horizon_y(h);

            let mut fb_road = ImageBuffer::new(w, h);
            draw_road(&mut fb_road, w, h, horizon_y, &world, &seed, &[]);

            let mut fb_grid = ImageBuffer::new(w, h);
            draw_road(&mut fb_grid, w, h, horizon_y, &world, &seed, &[]);
            draw_grid(&mut fb_grid, w, h, horizon_y, &world, &seed, &[]);

            let mut count = 0;
            for y in (horizon_y + 1)..h {
                let changed = (0..w)
                    .filter(|&x| fb_grid.get_pixel(x, y) != fb_road.get_pixel(x, y))
                    .count();
                if changed > w as usize / 2 {
                    count += 1;
                }
            }
            count
        };

        let counts: Vec<usize> = [0.0, 25.0, 62.5, 122.5, 125.0, 257.5]
            .iter()
            .map(|&cz| count_hlines(cz))
            .collect();

        let min = *counts.iter().min().unwrap();
        let max = *counts.iter().max().unwrap();
        assert!(
            max - min <= 1,
            "Horizontal grid line count should stay stable, got counts: {counts:?}"
        );
    }
}
