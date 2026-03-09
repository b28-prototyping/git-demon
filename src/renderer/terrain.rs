use image::{ImageBuffer, Rgba};

use super::road_table::{self, RoadRow};
use crate::git::seed::RepoSeed;
use crate::world::WorldState;

/// Parallax factor for terrain islands (near-background layer).
/// Scroll at 40% of camera Z advancement.
const ISLAND_PARALLAX: f32 = 0.4;

/// Parallax factor for clouds (mid-background layer).
/// Scroll at 15% of camera Z advancement.
const CLOUD_PARALLAX: f32 = 0.15;

/// Draw islands and clouds over the sea surface.
pub fn draw_terrain(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    _seed: &RepoSeed,
    world: &WorldState,
    rtable: &[RoadRow],
) {
    draw_islands(fb, w, h, horizon_y, world, rtable);
    draw_clouds(fb, w, h, horizon_y, world, rtable);
}

/// Small islands on the ocean surface, scrolling with camera.
fn draw_islands(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    world: &WorldState,
    rtable: &[RoadRow],
) {
    let island_count = 16u32;
    let z_period = 4000.0_f32;
    let cam = &world.camera;

    for i in 0..island_count {
        let hash = i.wrapping_mul(2654435761);
        let hash2 = i.wrapping_mul(1664525).wrapping_add(1013904223);
        let hash3 = i.wrapping_mul(214013).wrapping_add(2531011);

        // World-Z: islands repeat in a cycle, scrolling at ISLAND_PARALLAX rate
        let base_z = (hash as f32 / u32::MAX as f32) * z_period;
        let parallax_z = world.camera_z * ISLAND_PARALLAX;
        let z_rel = ((base_z - (parallax_z % z_period)) + z_period) % z_period;
        let z_world = world.camera.z + z_rel;

        // Project using Camera (1/z)
        let Some((sy, depth_scale)) = cam.project(z_world, h, horizon_y) else {
            continue;
        };
        if depth_scale < 0.005 {
            continue;
        }

        let pitch_shift = cam.pitch_offset(ISLAND_PARALLAX, h);
        let screen_y = (sy + pitch_shift) as i32;
        if screen_y < horizon_y as i32 || screen_y >= h as i32 {
            continue;
        }

        // Use road table for per-segment curve offset; apply slope to surface items
        let (curve_off, slope_off) = if let Some((curve, slope, _)) =
            road_table::lookup_at_z(rtable, z_world, world.camera_z, horizon_y)
        {
            (curve * depth_scale * depth_scale, slope)
        } else {
            (world.curve_offset * depth_scale * depth_scale, 0.0)
        };

        // Lateral position — spread across screen
        let world_x = (hash2 as f32 / u32::MAX as f32 - 0.5) * 600.0;
        let cx = w as f32 / 2.0 + curve_off;
        let spread = cam.road_half(depth_scale) * 1.5;
        let screen_x = (cx + world_x * spread / 300.0) as i32;

        // Apply slope offset — islands sit on the road surface
        let screen_y = screen_y + slope_off as i32;
        if screen_y < horizon_y as i32 || screen_y >= h as i32 {
            continue;
        }

        // Island size scales with perspective
        let size_x = (12.0 + depth_scale * 18.0) as i32;
        let size_y = (4.0 + depth_scale * 6.0) as i32;

        // Island colors — sandy brown to lush green
        let green_var = (hash3 % 40) as u8;
        let island_color = Rgba([15 + green_var / 3, 40 + green_var, 20 + green_var / 2, 255]);
        let sand_color = Rgba([50 + green_var, 45 + green_var / 2, 25, 255]);

        // Draw sandy ring
        for dy in -(size_y + 1)..=(size_y + 1) {
            let row_half = (size_x + 2) * ((size_y + 1) - dy.abs()) / (size_y + 1).max(1);
            for dx in -row_half..=row_half {
                let px = screen_x + dx;
                let py = screen_y + dy;
                if px >= 0 && py >= horizon_y as i32 && (px as u32) < w && (py as u32) < h {
                    fb.put_pixel(px as u32, py as u32, sand_color);
                }
            }
        }

        // Draw green interior
        for dy in -size_y..=size_y {
            let row_half = size_x * (size_y - dy.abs()) / size_y.max(1);
            for dx in -row_half..=row_half {
                let px = screen_x + dx;
                let py = screen_y + dy;
                if px >= 0 && py >= horizon_y as i32 && (px as u32) < w && (py as u32) < h {
                    fb.put_pixel(px as u32, py as u32, island_color);
                }
            }
        }
    }
}

/// Semi-transparent clouds floating between the ship and the sea.
fn draw_clouds(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    world: &WorldState,
    rtable: &[RoadRow],
) {
    let cloud_count = 20u32;
    let z_period = 5000.0_f32;
    let cam = &world.camera;
    let road_rows = (h - horizon_y).max(1) as f32;

    for i in 0..cloud_count {
        let hash = i.wrapping_mul(3266489917);
        let hash2 = i.wrapping_mul(668265263).wrapping_add(374761393);
        let hash3 = i.wrapping_mul(1103515245).wrapping_add(12345);

        // World-Z: clouds cycle, scrolling at CLOUD_PARALLAX rate
        let base_z = (hash as f32 / u32::MAX as f32) * z_period;
        let parallax_z = world.camera_z * CLOUD_PARALLAX;
        let z_rel = ((base_z - (parallax_z % z_period)) + z_period) % z_period;
        let z_world = world.camera.z + z_rel;

        // Project using Camera (1/z)
        let Some((_, depth_scale)) = cam.project(z_world, h, horizon_y) else {
            continue;
        };
        if depth_scale < 0.005 {
            continue;
        }

        // Clouds sit higher than sea surface — between horizon and ship
        // Map into the upper portion of the below-horizon area
        let cloud_altitude = 0.3 + (hash3 as f32 / u32::MAX as f32) * 0.4; // 0.3..0.7
        let pitch_shift = cam.pitch_offset(CLOUD_PARALLAX, h);
        let screen_y =
            (horizon_y as f32 + depth_scale * road_rows * cloud_altitude + pitch_shift) as i32;
        if screen_y < horizon_y as i32 || screen_y >= h as i32 {
            continue;
        }

        // Use road table for per-segment curve (clouds don't use slope — they float)
        let curve_off = if let Some((curve, _, _)) =
            road_table::lookup_at_z(rtable, z_world, world.camera_z, horizon_y)
        {
            curve * depth_scale * depth_scale
        } else {
            world.curve_offset * depth_scale * depth_scale
        };

        // Lateral position
        let world_x = (hash2 as f32 / u32::MAX as f32 - 0.5) * 800.0;
        let cx = w as f32 / 2.0 + curve_off;
        let spread = cam.road_half(depth_scale) * 1.5;
        let screen_x = (cx + world_x * spread / 400.0) as i32;

        // Cloud size — wider than tall, scales with perspective
        let size_x = (20.0 + depth_scale * 40.0) as i32;
        let size_y = (5.0 + depth_scale * 10.0) as i32;

        // Semi-transparent white — uniform alpha, no per-pixel fade
        let alpha = (depth_scale * 70.0) as u32;
        let inv = 255 - alpha;
        let brightness = (200 + hash % 55).min(255);

        for dy in -size_y..=size_y {
            let row_half = size_x * (size_y - dy.abs()) / size_y.max(1);
            for dx in -row_half..=row_half {
                let px = screen_x + dx;
                let py = screen_y + dy;
                if px >= 0 && py >= horizon_y as i32 && (px as u32) < w && (py as u32) < h {
                    let bg = fb.get_pixel(px as u32, py as u32);
                    fb.put_pixel(
                        px as u32,
                        py as u32,
                        Rgba([
                            ((brightness * alpha + bg.0[0] as u32 * inv) / 255) as u8,
                            ((brightness * alpha + bg.0[1] as u32 * inv) / 255) as u8,
                            ((brightness * alpha + bg.0[2] as u32 * inv) / 255) as u8,
                            255,
                        ]),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_seed() -> RepoSeed {
        RepoSeed {
            accent_hue: 200.0,
            saturation: 0.8,
            terrain_roughness: 0.5,
            speed_base: 0.5,
            author_colors: HashMap::new(),
            total_commits: 100,
            repo_name: "test-repo".to_string(),
        }
    }

    #[test]
    fn test_draw_terrain_modifies_pixels() {
        let seed = make_seed();
        let mut world = WorldState::new(&seed);
        world.camera_z = 100.0;
        world.camera.z = 100.0;
        let (w, h) = (400, 200);
        let transparent = Rgba([0, 0, 0, 0]);
        let mut fb = ImageBuffer::from_pixel(w, h, transparent);

        draw_terrain(&mut fb, w, h, 50, &seed, &world, &[]);

        let changed = fb.pixels().filter(|p| **p != transparent).count();
        assert!(
            changed > 0,
            "terrain (islands/clouds) should modify some pixels"
        );
    }

    #[test]
    fn test_draw_terrain_stays_below_horizon() {
        let seed = make_seed();
        let mut world = WorldState::new(&seed);
        world.camera_z = 500.0;
        world.camera.z = 500.0;
        let (w, h) = (400, 200);
        let horizon_y = 50u32;
        let transparent = Rgba([0, 0, 0, 0]);
        let mut fb = ImageBuffer::from_pixel(w, h, transparent);

        draw_terrain(&mut fb, w, h, horizon_y, &seed, &world, &[]);

        for y in 0..horizon_y {
            for x in 0..w {
                assert_eq!(
                    *fb.get_pixel(x, y),
                    transparent,
                    "no terrain pixels above horizon at ({x},{y})"
                );
            }
        }
    }

    #[test]
    fn test_islands_scroll_with_camera() {
        let seed = make_seed();
        let (w, h) = (400, 200);
        let horizon_y = 50u32;

        let render = |cam_z: f32| -> Vec<u8> {
            let mut world = WorldState::new(&seed);
            world.camera_z = cam_z;
            world.camera.z = cam_z;
            let transparent = Rgba([0, 0, 0, 0]);
            let mut fb = ImageBuffer::from_pixel(w, h, transparent);
            draw_islands(&mut fb, w, h, horizon_y, &world, &[]);
            fb.into_raw()
        };

        let a = render(0.0);
        let b = render(500.0);
        assert_ne!(a, b, "islands should shift as camera moves");
    }

    #[test]
    fn test_islands_parallax_rate() {
        // Islands scroll at 40% of camera_z. Moving camera_z by X should
        // produce the same island layout as moving by X/0.4 at full rate.
        // Equivalently: camera_z=1000 should look the same as camera_z=2500*0.4=1000.
        let seed = make_seed();
        let (w, h) = (400, 200);
        let horizon_y = 50u32;

        let render = |cam_z: f32| -> Vec<u8> {
            let mut world = WorldState::new(&seed);
            world.camera_z = cam_z;
            world.camera.z = cam_z;
            let transparent = Rgba([0, 0, 0, 0]);
            let mut fb = ImageBuffer::from_pixel(w, h, transparent);
            draw_islands(&mut fb, w, h, horizon_y, &world, &[]);
            fb.into_raw()
        };

        // At camera_z=0 and camera_z=1000, islands should differ
        let at_0 = render(0.0);
        let at_1000 = render(1000.0);
        assert_ne!(at_0, at_1000, "islands should differ at different camera_z");

        // The parallax cycle length is z_period / ISLAND_PARALLAX = 4000 / 0.4 = 10000.
        // So at camera_z=10000 the island layout should wrap back to camera_z=0.
        let at_cycle = render(10000.0);
        assert_eq!(
            at_0, at_cycle,
            "islands should repeat after full parallax cycle"
        );
    }

    #[test]
    fn test_clouds_scroll_slower_than_islands() {
        // Render at two camera_z values. Count changed pixels for islands vs clouds.
        // Clouds (0.15) should change less than islands (0.4) for the same camera_z delta.
        let seed = make_seed();
        let (w, h) = (400, 200);
        let horizon_y = 50u32;

        let render_islands = |cam_z: f32| -> Vec<u8> {
            let mut world = WorldState::new(&seed);
            world.camera_z = cam_z;
            world.camera.z = cam_z;
            let transparent = Rgba([0, 0, 0, 0]);
            let mut fb = ImageBuffer::from_pixel(w, h, transparent);
            draw_islands(&mut fb, w, h, horizon_y, &world, &[]);
            fb.into_raw()
        };

        let render_clouds = |cam_z: f32| -> Vec<u8> {
            let mut world = WorldState::new(&seed);
            world.camera_z = cam_z;
            world.camera.z = cam_z;
            let transparent = Rgba([0, 0, 0, 0]);
            let mut fb = ImageBuffer::from_pixel(w, h, transparent);
            draw_clouds(&mut fb, w, h, horizon_y, &world, &[]);
            fb.into_raw()
        };

        let island_diff = render_islands(0.0)
            .iter()
            .zip(render_islands(200.0).iter())
            .filter(|(a, b)| a != b)
            .count();

        let cloud_diff = render_clouds(0.0)
            .iter()
            .zip(render_clouds(200.0).iter())
            .filter(|(a, b)| a != b)
            .count();

        // Both should have some difference
        assert!(island_diff > 0, "islands should change with camera_z");
        assert!(cloud_diff > 0, "clouds should change with camera_z");
        // Clouds should change less since they scroll slower
        assert!(
            cloud_diff < island_diff,
            "clouds ({cloud_diff} changed bytes) should change less than islands ({island_diff})"
        );
    }

    #[test]
    fn test_pitch_offset_shifts_terrain() {
        let seed = make_seed();
        let (w, h) = (400, 200);
        let horizon_y = 50u32;

        let render = |pitch: f32| -> Vec<u8> {
            let mut world = WorldState::new(&seed);
            world.camera_z = 100.0;
            world.camera.z = 100.0;
            world.camera.pitch = pitch;
            let transparent = Rgba([0, 0, 0, 0]);
            let mut fb = ImageBuffer::from_pixel(w, h, transparent);
            draw_terrain(&mut fb, w, h, horizon_y, &seed, &world, &[]);
            fb.into_raw()
        };

        let no_pitch = render(0.0);
        let with_pitch = render(0.2);
        assert_ne!(
            no_pitch, with_pitch,
            "pitch should shift terrain layer positions"
        );
    }
}
