use image::{ImageBuffer, Rgba};
use noise::{NoiseFn, OpenSimplex};

use crate::git::seed::RepoSeed;
use crate::world::WorldState;

const TERRAIN_FREQ: f64 = 0.008;
const TERRAIN_DRIFT: f64 = 0.3;
const LEFT_SEED: u32 = 42;
const RIGHT_SEED: u32 = 137;
const TREE_SEED: u32 = 251;

const TERRAIN_LEFT_COLOR: Rgba<u8> = Rgba([8, 12, 20, 255]);
const TERRAIN_RIGHT_COLOR: Rgba<u8> = Rgba([12, 14, 22, 255]);

// Tree colors
const TREE_TRUNK: Rgba<u8> = Rgba([25, 15, 10, 255]);
const TREE_CANOPY_DARK: Rgba<u8> = Rgba([10, 30, 15, 255]);
const TREE_CANOPY_LIGHT: Rgba<u8> = Rgba([15, 45, 20, 255]);

pub fn draw_terrain(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    _h: u32,
    horizon_y: u32,
    seed: &RepoSeed,
    world: &WorldState,
) {
    let left_noise = OpenSimplex::new(LEFT_SEED);
    let right_noise = OpenSimplex::new(RIGHT_SEED);
    let tree_noise = OpenSimplex::new(TREE_SEED);
    let max_terrain_h = (horizon_y as f32 * 0.6 * seed.terrain_roughness) as u32;

    let road_left_edge = w / 4;
    let road_right_edge = w * 3 / 4;

    // Left terrain silhouette
    for x in 0..road_left_edge {
        let nx = x as f64 * TERRAIN_FREQ;
        let ny = world.time as f64 * TERRAIN_DRIFT;
        let val = left_noise.get([nx, ny]);
        let h = ((val + 1.0) * 0.5 * max_terrain_h as f64) as u32;

        let top = horizon_y.saturating_sub(h);
        for y in top..horizon_y {
            if x < fb.width() && y < fb.height() {
                fb.put_pixel(x, y, TERRAIN_LEFT_COLOR);
            }
        }
    }

    // Right terrain silhouette
    for x in road_right_edge..w {
        let nx = x as f64 * TERRAIN_FREQ;
        let ny = world.time as f64 * TERRAIN_DRIFT + 100.0;
        let val = right_noise.get([nx, ny]);
        let h = ((val + 1.0) * 0.5 * max_terrain_h as f64) as u32;

        let top = horizon_y.saturating_sub(h);
        for y in top..horizon_y {
            if x < fb.width() && y < fb.height() {
                fb.put_pixel(x, y, TERRAIN_RIGHT_COLOR);
            }
        }
    }

    // Trees on left side — procedural placement via noise threshold
    draw_trees(fb, &tree_noise, 0, road_left_edge, horizon_y, world, true);
    // Trees on right side
    draw_trees(fb, &tree_noise, road_right_edge, w, horizon_y, world, false);
}

fn draw_trees(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    noise: &OpenSimplex,
    x_start: u32,
    x_end: u32,
    horizon_y: u32,
    world: &WorldState,
    left_side: bool,
) {
    // Place trees at intervals determined by noise
    let tree_spacing = 40u32;
    let offset = if left_side { 0.0 } else { 500.0 };
    let scroll = (world.camera_z * 8.0) as i32; // trees scroll with camera

    let range_w = x_end.saturating_sub(x_start);
    if range_w < 20 {
        return;
    }

    for i in 0..32 {
        let base_x = i as f64 * tree_spacing as f64 + offset;
        let noise_val = noise.get([base_x * 0.05, world.time as f64 * 0.01 + offset]);

        // Only place tree if noise above threshold (sparse placement)
        if noise_val < 0.1 {
            continue;
        }

        let screen_x = ((base_x as i32 - scroll % (32 * tree_spacing as i32))
            .rem_euclid(range_w as i32)) as u32
            + x_start;

        if screen_x >= x_end || screen_x < x_start {
            continue;
        }

        // Tree depth: further from road edge = closer to horizon = smaller
        let dist_from_road = if left_side {
            (x_end - screen_x) as f32 / range_w as f32
        } else {
            (screen_x - x_start) as f32 / range_w as f32
        };
        let depth = dist_from_road.clamp(0.1, 1.0);

        let trunk_h = (8.0 + depth * 20.0) as u32;
        let trunk_w = (2.0 + depth * 3.0) as u32;
        let canopy_r = (4.0 + depth * 12.0) as u32;

        let base_y = horizon_y.saturating_sub((depth * 5.0) as u32);
        let canopy_color = if noise_val > 0.4 {
            TREE_CANOPY_LIGHT
        } else {
            TREE_CANOPY_DARK
        };

        // Trunk
        draw_rect(
            fb,
            screen_x.saturating_sub(trunk_w / 2),
            base_y.saturating_sub(trunk_h),
            trunk_w,
            trunk_h,
            TREE_TRUNK,
        );
        // Canopy — simple filled triangle approximation (diamond shape)
        let canopy_cx = screen_x;
        let canopy_cy = base_y.saturating_sub(trunk_h + canopy_r / 2);
        draw_diamond(fb, canopy_cx, canopy_cy, canopy_r, canopy_color);
    }
}

fn draw_rect(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: Rgba<u8>,
) {
    let fw = fb.width();
    let fh = fb.height();
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < fw && py < fh {
                fb.put_pixel(px, py, color);
            }
        }
    }
}

fn draw_diamond(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    cx: u32,
    cy: u32,
    r: u32,
    color: Rgba<u8>,
) {
    let fw = fb.width();
    let fh = fb.height();
    for dy in 0..=r {
        let half_w = r.saturating_sub(dy);
        for dx in 0..=half_w {
            // Top half
            let ty = cy.saturating_sub(dy);
            if cx + dx < fw && ty < fh {
                fb.put_pixel(cx + dx, ty, color);
            }
            if dx > 0 && cx >= dx && cx - dx < fw && ty < fh {
                fb.put_pixel(cx - dx, ty, color);
            }
            // Bottom half
            let by = cy + dy;
            if cx + dx < fw && by < fh {
                fb.put_pixel(cx + dx, by, color);
            }
            if dx > 0 && cx >= dx && cx - dx < fw && by < fh {
                fb.put_pixel(cx - dx, by, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_seed(terrain_roughness: f32) -> RepoSeed {
        RepoSeed {
            accent_hue: 200.0,
            saturation: 0.8,
            terrain_roughness,
            speed_base: 0.5,
            author_colors: HashMap::new(),
            total_commits: 100,
            repo_name: "test-repo".to_string(),
        }
    }

    fn make_world(time: f32, seed: &RepoSeed) -> WorldState {
        let mut w = WorldState::new(seed);
        w.time = time;
        w
    }

    #[test]
    fn test_left_right_different_noise() {
        let seed = make_seed(0.8);
        let world = make_world(5.0, &seed);
        let (w, h) = (200, 100);
        let horizon_y = 50;
        let mut fb = ImageBuffer::new(w, h);
        draw_terrain(&mut fb, w, h, horizon_y, &seed, &world);

        // Collect terrain top for a left column and a right column
        let left_x = 10u32;
        let right_x = 170u32;

        let left_top = (0..horizon_y).find(|&y| *fb.get_pixel(left_x, y) == TERRAIN_LEFT_COLOR);
        let right_top = (0..horizon_y).find(|&y| *fb.get_pixel(right_x, y) == TERRAIN_RIGHT_COLOR);

        // Both sides should have terrain
        assert!(left_top.is_some(), "left terrain should be present");
        assert!(right_top.is_some(), "right terrain should be present");
        // They should differ (different seeds produce different heights)
        assert_ne!(
            left_top, right_top,
            "left and right terrain should have different profiles"
        );
    }

    #[test]
    fn test_roughness_scales_height() {
        let (w, h) = (200, 100);
        let horizon_y = 50;

        let seed_low = make_seed(0.1);
        let world_low = make_world(3.0, &seed_low);
        let mut fb_low = ImageBuffer::new(w, h);
        draw_terrain(&mut fb_low, w, h, horizon_y, &seed_low, &world_low);

        let seed_high = make_seed(1.0);
        let world_high = make_world(3.0, &seed_high);
        let mut fb_high = ImageBuffer::new(w, h);
        draw_terrain(&mut fb_high, w, h, horizon_y, &seed_high, &world_high);

        // Find highest terrain pixel (lowest y) across all left columns
        let highest_low = (0..w / 4)
            .filter_map(|x| (0..horizon_y).find(|&y| *fb_low.get_pixel(x, y) == TERRAIN_LEFT_COLOR))
            .min();
        let highest_high = (0..w / 4)
            .filter_map(|x| {
                (0..horizon_y).find(|&y| *fb_high.get_pixel(x, y) == TERRAIN_LEFT_COLOR)
            })
            .min();

        assert!(highest_low.is_some() && highest_high.is_some());
        // Higher roughness should produce taller terrain (lower y value)
        assert!(
            highest_high.unwrap() < highest_low.unwrap(),
            "roughness=1.0 should produce taller terrain than roughness=0.1 (got {} vs {})",
            highest_high.unwrap(),
            highest_low.unwrap()
        );
    }

    #[test]
    fn test_silhouette_filled() {
        let seed = make_seed(0.8);
        let world = make_world(2.0, &seed);
        let (w, h) = (200, 100);
        let horizon_y = 50;
        let mut fb = ImageBuffer::new(w, h);
        draw_terrain(&mut fb, w, h, horizon_y, &seed, &world);

        // Check a left terrain column: all pixels from top to horizon should be filled
        let x = 15u32;
        if let Some(top) = (0..horizon_y).find(|&y| *fb.get_pixel(x, y) == TERRAIN_LEFT_COLOR) {
            for y in top..horizon_y {
                assert_eq!(
                    *fb.get_pixel(x, y),
                    TERRAIN_LEFT_COLOR,
                    "gap in left silhouette at ({x}, {y})"
                );
            }
        }

        // Check a right terrain column
        let x = 175u32;
        if let Some(top) = (0..horizon_y).find(|&y| *fb.get_pixel(x, y) == TERRAIN_RIGHT_COLOR) {
            for y in top..horizon_y {
                assert_eq!(
                    *fb.get_pixel(x, y),
                    TERRAIN_RIGHT_COLOR,
                    "gap in right silhouette at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn test_colors_match() {
        let seed = make_seed(0.8);
        let world = make_world(1.0, &seed);
        let (w, h) = (200, 100);
        let horizon_y = 50;
        let mut fb = ImageBuffer::new(w, h);
        draw_terrain(&mut fb, w, h, horizon_y, &seed, &world);

        let transparent = Rgba([0, 0, 0, 0]);

        // Left terrain should only contain TERRAIN_LEFT_COLOR (or black/tree colors)
        for x in 0..w / 4 {
            for y in 0..horizon_y {
                let px = *fb.get_pixel(x, y);
                if px != transparent && px != TERRAIN_LEFT_COLOR {
                    // Could be tree pixels — they're valid too
                    assert!(
                        px == TREE_TRUNK || px == TREE_CANOPY_DARK || px == TREE_CANOPY_LIGHT,
                        "unexpected color in left terrain at ({x},{y}): {px:?}"
                    );
                }
            }
        }

        // Right terrain
        for x in (w * 3 / 4)..w {
            for y in 0..horizon_y {
                let px = *fb.get_pixel(x, y);
                if px != transparent && px != TERRAIN_RIGHT_COLOR {
                    assert!(
                        px == TREE_TRUNK || px == TREE_CANOPY_DARK || px == TREE_CANOPY_LIGHT,
                        "unexpected color in right terrain at ({x},{y}): {px:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_time_drift() {
        let seed = make_seed(0.6);
        let (w, h) = (200, 100);
        let horizon_y = 50;

        let world_a = make_world(0.0, &seed);
        let mut fb_a = ImageBuffer::new(w, h);
        draw_terrain(&mut fb_a, w, h, horizon_y, &seed, &world_a);

        let world_b = make_world(10.0, &seed);
        let mut fb_b = ImageBuffer::new(w, h);
        draw_terrain(&mut fb_b, w, h, horizon_y, &seed, &world_b);

        // At least some terrain pixels should differ between the two times
        let mut differs = false;
        for x in 0..w / 4 {
            for y in 0..horizon_y {
                if fb_a.get_pixel(x, y) != fb_b.get_pixel(x, y) {
                    differs = true;
                    break;
                }
            }
            if differs {
                break;
            }
        }
        assert!(differs, "terrain should change over time due to drift");
    }

    #[test]
    fn test_no_terrain_below_horizon() {
        let seed = make_seed(1.0);
        let world = make_world(0.0, &seed);
        let (w, h) = (200, 100);
        let horizon_y = 50;
        let mut fb = ImageBuffer::new(w, h);
        draw_terrain(&mut fb, w, h, horizon_y, &seed, &world);

        for x in 0..w {
            for y in horizon_y..h {
                assert_eq!(
                    *fb.get_pixel(x, y),
                    Rgba([0, 0, 0, 0]),
                    "terrain pixel found below horizon at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn test_terrain_boundaries() {
        let seed = make_seed(0.8);
        let world = make_world(1.0, &seed);
        let (w, h) = (200, 100);
        let horizon_y = 50;
        let mut fb = ImageBuffer::new(w, h);
        draw_terrain(&mut fb, w, h, horizon_y, &seed, &world);

        // Middle band (between road edges) should have no terrain colors
        for x in (w / 4)..(w * 3 / 4) {
            for y in 0..horizon_y {
                let px = *fb.get_pixel(x, y);
                assert!(
                    px != TERRAIN_LEFT_COLOR && px != TERRAIN_RIGHT_COLOR,
                    "terrain color found in middle band at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn test_zero_horizon_safe() {
        let seed = make_seed(0.5);
        let world = make_world(0.0, &seed);
        let (w, h) = (200, 100);
        let mut fb = ImageBuffer::new(w, h);
        // Should not panic with horizon_y = 0
        draw_terrain(&mut fb, w, h, 0, &seed, &world);

        // No terrain silhouette pixels should be written (trees may still render)
        for x in 0..w {
            for y in 0..h {
                let px = *fb.get_pixel(x, y);
                assert!(
                    px != TERRAIN_LEFT_COLOR && px != TERRAIN_RIGHT_COLOR,
                    "terrain silhouette pixel found at ({x}, {y}) with horizon_y=0"
                );
            }
        }
    }
}
