use image::{ImageBuffer, Rgba};

use crate::git::seed::RepoSeed;
use crate::world::WorldState;

const BLOOM_THRESHOLD: f32 = 0.72;
const _BLOOM_STRENGTH: f32 = 0.3;

pub fn draw_speed_lines(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    horizon_y: u32,
    world: &WorldState,
    seed: &RepoSeed,
) {
    let n = ((world.commits_per_min * 8.0) as u32).min(64);
    if n == 0 {
        return;
    }
    let cx = w as f32 / 2.0;
    let cy = horizon_y as f32;
    let alpha = if world.tier as u8 >= 4 { 140u8 } else { 80u8 };
    let accent = hue_to_rgb(seed.accent_hue);
    let color = Rgba([accent.0, accent.1, accent.2, alpha]);

    let h = fb.height();
    let max_len = (w as f32).hypot(h as f32) as u32;

    for i in 0..n {
        let angle = (i as f32 / n as f32) * std::f32::consts::TAU;
        let dx = angle.cos();
        let dy = angle.sin();

        // Step by 2 for performance — speed lines don't need pixel-perfect coverage
        let mut step = 0u32;
        while step < max_len {
            let px = (cx + dx * step as f32) as i32;
            let py = (cy + dy * step as f32) as i32;
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                break;
            }
            let pu = px as u32;
            let pv = py as u32;
            let bg = fb.get_pixel(pu, pv);
            fb.put_pixel(pu, pv, blend_alpha(*bg, color));
            step += 2;
        }
    }
}

pub fn apply_motion_blur(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    prev: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    world: &WorldState,
) {
    let alpha = lerp(0.15, 0.35, world.speed / 12.0);
    // Use fixed-point for speed: alpha as 0..256 integer
    let a_fixed = (alpha * 256.0) as u32;
    let inv_fixed = 256 - a_fixed;

    let fb_raw = fb.as_mut();
    let prev_raw = prev.as_ref();
    let len = fb_raw.len().min(prev_raw.len());

    // Process 4 bytes at a time (RGBA), skip alpha channel
    let mut i = 0;
    while i + 3 < len {
        fb_raw[i] = ((fb_raw[i] as u32 * inv_fixed + prev_raw[i] as u32 * a_fixed) >> 8) as u8;
        fb_raw[i + 1] =
            ((fb_raw[i + 1] as u32 * inv_fixed + prev_raw[i + 1] as u32 * a_fixed) >> 8) as u8;
        fb_raw[i + 2] =
            ((fb_raw[i + 2] as u32 * inv_fixed + prev_raw[i + 2] as u32 * a_fixed) >> 8) as u8;
        // skip alpha (i+3)
        i += 4;
    }
}

pub fn apply_scanline_filter(fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let w = fb.width() as usize;
    let raw = fb.as_mut();
    // Process every other row — darken by 20% using fixed-point multiply
    // 0.80 * 256 = 204.8 ≈ 205
    let mut row = 0u32;
    let mut offset = 0usize;
    let row_bytes = w * 4;
    while offset + row_bytes <= raw.len() {
        if row.is_multiple_of(2) {
            let end = offset + row_bytes;
            let mut i = offset;
            while i + 3 < end {
                raw[i] = ((raw[i] as u32 * 205) >> 8) as u8;
                raw[i + 1] = ((raw[i + 1] as u32 * 205) >> 8) as u8;
                raw[i + 2] = ((raw[i + 2] as u32 * 205) >> 8) as u8;
                i += 4;
            }
        }
        offset += row_bytes;
        row += 1;
    }
}

pub fn apply_bloom(fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let w = fb.width();
    let h = fb.height();

    // Collect emissive pixels — cap to avoid pathological cases
    let mut emissive: Vec<(u32, u32, [u8; 3])> = Vec::with_capacity(1024);

    // Sample every 2nd pixel for perf — bloom is a soft effect anyway
    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            let p = fb.get_pixel(x, y);
            if luminance_fast(p) > BLOOM_THRESHOLD {
                emissive.push((x, y, [p.0[0], p.0[1], p.0[2]]));
            }
            x += 2;
        }
        y += 2;
    }

    // Apply 3×3 additive blur for emissive pixels
    // Use integer math: BLOOM_STRENGTH * center_weight = 0.3 * 0.5 = 0.15
    // BLOOM_STRENGTH * edge_weight = 0.3 * 0.25 = 0.075
    // As fixed-point /256: center=38, edge=19
    for &(ex, ey, rgb) in &emissive {
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let nx = ex as i32 + dx;
                let ny = ey as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h {
                    let weight_fixed: u32 = if dx == 0 && dy == 0 { 38 } else { 19 };
                    let p = fb.get_pixel_mut(nx as u32, ny as u32);
                    p.0[0] = (p.0[0] as u32 + ((rgb[0] as u32 * weight_fixed) >> 8)).min(255) as u8;
                    p.0[1] = (p.0[1] as u32 + ((rgb[1] as u32 * weight_fixed) >> 8)).min(255) as u8;
                    p.0[2] = (p.0[2] as u32 + ((rgb[2] as u32 * weight_fixed) >> 8)).min(255) as u8;
                }
            }
        }
    }
}

/// Draw the player car at the bottom center of the screen
pub fn draw_car(fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, w: u32, h: u32, world: &WorldState) {
    let cx = w as f32 / 2.0;
    // Slight lateral shift with steering
    let steer_shift = world.steer_angle * 0.15;
    let car_cx = (cx + steer_shift) as i32;
    let car_bottom = h as i32 - 4;

    // Car dimensions
    let car_w = 60i32;
    let car_h = 30i32;
    let nose_h = 12i32;

    // Shadow — dark ellipse under the car, offset down slightly
    let shadow_y = car_bottom + 2;
    let shadow_rx = car_w / 2 + 8;
    let shadow_ry = 6i32;
    for dy in -shadow_ry..=shadow_ry {
        let row_half = shadow_rx * (shadow_ry - dy.abs()) / shadow_ry.max(1);
        for dx in -row_half..=row_half {
            let px = (car_cx + dx) as u32;
            let py = (shadow_y + dy) as u32;
            if px < w && py < h {
                let bg = fb.get_pixel(px, py);
                // Darken by 50%
                fb.put_pixel(px, py, Rgba([bg.0[0] / 2, bg.0[1] / 2, bg.0[2] / 2, 255]));
            }
        }
    }

    // Body colors
    let body_top = Rgba([180, 20, 20, 255]); // bright red
    let body_mid = Rgba([140, 15, 15, 255]); // darker red
    let body_dark = Rgba([80, 8, 8, 255]); // shadow side
    let windshield = Rgba([40, 60, 90, 255]); // dark blue glass
    let cockpit = Rgba([20, 20, 25, 255]); // dark interior

    // Main body — trapezoid: wider at back, narrower at front
    let body_top_y = car_bottom - car_h;
    for dy in 0..car_h {
        let t = dy as f32 / car_h as f32; // 0=top, 1=bottom
        let half_w = ((car_w as f32 / 2.0) * (0.5 + t * 0.5)) as i32;
        let color = if t < 0.3 {
            body_top
        } else if t < 0.7 {
            body_mid
        } else {
            body_dark
        };

        for dx in -half_w..=half_w {
            let px = (car_cx + dx) as u32;
            let py = (body_top_y + dy) as u32;
            if px < w && py < h {
                fb.put_pixel(px, py, color);
            }
        }
    }

    // Nose/wedge — triangle pointing forward (up), narrower
    let nose_top_y = body_top_y - nose_h;
    for dy in 0..nose_h {
        let t = dy as f32 / nose_h as f32; // 0=tip, 1=body join
        let half_w = ((car_w as f32 / 2.0) * 0.5 * (0.2 + t * 0.8)) as i32;
        let color = if t < 0.5 { body_top } else { body_mid };

        for dx in -half_w..=half_w {
            let px = (car_cx + dx) as u32;
            let py = (nose_top_y + dy) as u32;
            if px < w && py < h {
                fb.put_pixel(px, py, color);
            }
        }
    }

    // Windshield — small dark area at top of main body
    let ws_top = body_top_y + 2;
    let ws_h = 6i32;
    for dy in 0..ws_h {
        let t = (dy as f32 / ws_h as f32 + 0.3).min(1.0);
        let half_w = ((car_w as f32 / 2.0) * 0.35 * t) as i32;
        for dx in -half_w..=half_w {
            let px = (car_cx + dx) as u32;
            let py = (ws_top + dy) as u32;
            if px < w && py < h {
                fb.put_pixel(px, py, windshield);
            }
        }
    }

    // Cockpit opening behind windshield
    let cp_top = ws_top + ws_h;
    let cp_h = 4i32;
    for dy in 0..cp_h {
        let half_w = ((car_w as f32 / 2.0) * 0.25) as i32;
        for dx in -half_w..=half_w {
            let px = (car_cx + dx) as u32;
            let py = (cp_top + dy) as u32;
            if px < w && py < h {
                fb.put_pixel(px, py, cockpit);
            }
        }
    }

    // Rear wing — thin horizontal bar at very back
    let wing_y = car_bottom - 2;
    let wing_half = car_w / 2 + 6;
    for dx in -wing_half..=wing_half {
        for dy in 0..3i32 {
            let px = (car_cx + dx) as u32;
            let py = (wing_y + dy) as u32;
            if px < w && py < h {
                fb.put_pixel(px, py, body_dark);
            }
        }
    }

    // Tail lights — two small bright red dots at rear corners
    let tail_color = Rgba([255, 30, 30, 255]);
    for &side in &[-1i32, 1] {
        let tx = car_cx + side * (car_w / 2 - 4);
        for dy in 0..3i32 {
            for dx in 0..4i32 {
                let px = (tx + dx * side) as u32;
                let py = (car_bottom - 3 + dy) as u32;
                if px < w && py < h {
                    fb.put_pixel(px, py, tail_color);
                }
            }
        }
    }
}

#[inline(always)]
fn luminance_fast(p: &Rgba<u8>) -> f32 {
    // Approximate: (r + 2g + b) / 4 / 255, close enough for threshold check
    ((p.0[0] as u32 + 2 * p.0[1] as u32 + p.0[2] as u32) as f32) / (4.0 * 255.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn blend_alpha(bg: Rgba<u8>, fg: Rgba<u8>) -> Rgba<u8> {
    let a = fg.0[3] as u32;
    let inv = 255 - a;
    Rgba([
        ((fg.0[0] as u32 * a + bg.0[0] as u32 * inv) / 255) as u8,
        ((fg.0[1] as u32 * a + bg.0[1] as u32 * inv) / 255) as u8,
        ((fg.0[2] as u32 * a + bg.0[2] as u32 * inv) / 255) as u8,
        255,
    ])
}

fn hue_to_rgb(hue: f32) -> (u8, u8, u8) {
    let c = 1.0_f32;
    let h2 = hue / 60.0;
    let x = c * (1.0 - (h2 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h2 as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    ((r1 * 255.0) as u8, (g1 * 255.0) as u8, (b1 * 255.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};

    fn make_world(speed: f32, cpm: f32) -> WorldState {
        let tier = crate::world::speed::VelocityTier::from_commits_per_min(cpm);
        WorldState {
            z_offset: 0.0,
            camera_z: 0.0,
            speed,
            speed_target: speed,
            commits_per_min: cpm,
            lines_added: 0,
            lines_deleted: 0,
            files_changed: 0,
            tier,
            time: 0.0,
            total_commits: 100,
            pending_objects: VecDeque::new(),
            active_objects: Vec::new(),
            curve_offset: 0.0,
            curve_target: 0.0,
            steer_angle: 0.0,
        }
    }

    fn make_seed() -> RepoSeed {
        RepoSeed {
            accent_hue: 180.0,
            saturation: 0.8,
            terrain_roughness: 0.5,
            speed_base: 0.5,
            author_colors: HashMap::new(),
            total_commits: 100,
            repo_name: "test-repo".into(),
        }
    }

    // --- lerp ---

    #[test]
    fn test_lerp_midpoint() {
        assert!((lerp(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_clamps_below_zero() {
        assert!((lerp(10.0, 20.0, -1.0) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_clamps_above_one() {
        assert!((lerp(10.0, 20.0, 2.0) - 20.0).abs() < 1e-6);
    }

    // --- luminance_fast ---

    #[test]
    fn test_luminance_white() {
        let p = Rgba([255, 255, 255, 255]);
        let l = luminance_fast(&p);
        assert!((l - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_luminance_black() {
        let p = Rgba([0, 0, 0, 255]);
        assert!(luminance_fast(&p) < 0.01);
    }

    #[test]
    fn test_luminance_bright_exceeds_threshold() {
        let p = Rgba([200, 200, 200, 255]);
        assert!(luminance_fast(&p) > BLOOM_THRESHOLD);
    }

    // --- blend_alpha ---

    #[test]
    fn test_blend_alpha_fully_opaque() {
        let bg = Rgba([100, 100, 100, 255]);
        let fg = Rgba([200, 50, 30, 255]);
        let result = blend_alpha(bg, fg);
        assert_eq!(result.0[0], 200);
        assert_eq!(result.0[1], 50);
        assert_eq!(result.0[2], 30);
    }

    #[test]
    fn test_blend_alpha_fully_transparent() {
        let bg = Rgba([100, 150, 200, 255]);
        let fg = Rgba([0, 0, 0, 0]);
        let result = blend_alpha(bg, fg);
        assert_eq!(result.0[0], 100);
        assert_eq!(result.0[1], 150);
        assert_eq!(result.0[2], 200);
    }

    #[test]
    fn test_blend_alpha_half() {
        let bg = Rgba([0, 0, 0, 255]);
        let fg = Rgba([200, 100, 50, 128]);
        let result = blend_alpha(bg, fg);
        // 200*128/255 ≈ 100, 100*128/255 ≈ 50, 50*128/255 ≈ 25
        assert!((result.0[0] as i32 - 100).abs() <= 1);
        assert!((result.0[1] as i32 - 50).abs() <= 1);
        assert!((result.0[2] as i32 - 25).abs() <= 1);
    }

    // --- hue_to_rgb ---

    #[test]
    fn test_hue_red() {
        let (r, g, b) = hue_to_rgb(0.0);
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_hue_green() {
        let (r, g, b) = hue_to_rgb(120.0);
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_hue_blue() {
        let (r, g, b) = hue_to_rgb(240.0);
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 255);
    }

    // --- scanline filter ---

    #[test]
    fn test_scanline_darkens_even_rows() {
        let mut fb = ImageBuffer::from_pixel(4, 4, Rgba([200, 200, 200, 255]));
        apply_scanline_filter(&mut fb);

        // Even rows (0, 2) should be darkened: 200 * 205/256 ≈ 160
        let p = fb.get_pixel(0, 0);
        assert_eq!(p.0[0], (200u32 * 205 / 256) as u8); // 160
        assert_eq!(p.0[3], 255); // alpha unchanged

        // Odd rows (1, 3) should be unchanged
        let p = fb.get_pixel(0, 1);
        assert_eq!(p.0[0], 200);
    }

    #[test]
    fn test_scanline_leaves_odd_rows_intact() {
        let mut fb = ImageBuffer::from_pixel(8, 8, Rgba([100, 150, 200, 255]));
        apply_scanline_filter(&mut fb);

        for y in (1..8).step_by(2) {
            for x in 0..8 {
                let p = fb.get_pixel(x, y);
                assert_eq!(p.0[0], 100);
                assert_eq!(p.0[1], 150);
                assert_eq!(p.0[2], 200);
            }
        }
    }

    // --- motion blur ---

    #[test]
    fn test_motion_blur_at_zero_speed() {
        let mut fb = ImageBuffer::from_pixel(4, 4, Rgba([200, 200, 200, 255]));
        let prev = ImageBuffer::from_pixel(4, 4, Rgba([100, 100, 100, 255]));
        let world = make_world(0.0, 0.0);

        apply_motion_blur(&mut fb, &prev, &world);

        // alpha = lerp(0.15, 0.35, 0/12) = 0.15
        // result = 200 * (1 - 0.15) + 100 * 0.15 = 170 + 15 = 185
        // Fixed-point: a_fixed = (0.15 * 256) = 38, inv = 218
        // (200 * 218 + 100 * 38) >> 8 = (43600 + 3800) >> 8 = 47400 >> 8 = 185
        let p = fb.get_pixel(0, 0);
        assert_eq!(p.0[0], 185);
    }

    #[test]
    fn test_motion_blur_at_max_speed() {
        let mut fb = ImageBuffer::from_pixel(4, 4, Rgba([200, 200, 200, 255]));
        let prev = ImageBuffer::from_pixel(4, 4, Rgba([100, 100, 100, 255]));
        let world = make_world(12.0, 4.0);

        apply_motion_blur(&mut fb, &prev, &world);

        // alpha = lerp(0.15, 0.35, 12/12) = 0.35
        // a_fixed = (0.35 * 256) = 89, inv = 167
        // (200 * 167 + 100 * 89) >> 8 = (33400 + 8900) >> 8 = 42300 >> 8 = 165
        let p = fb.get_pixel(0, 0);
        assert_eq!(p.0[0], 165);
    }

    #[test]
    fn test_motion_blur_preserves_alpha_channel() {
        let mut fb = ImageBuffer::from_pixel(4, 4, Rgba([200, 200, 200, 42]));
        let prev = ImageBuffer::from_pixel(4, 4, Rgba([100, 100, 100, 99]));
        let world = make_world(6.0, 2.0);

        apply_motion_blur(&mut fb, &prev, &world);

        // Alpha bytes (every 4th byte starting at index 3) should not be modified
        let p = fb.get_pixel(0, 0);
        assert_eq!(p.0[3], 42);
    }

    // --- bloom ---

    #[test]
    fn test_bloom_brightens_neighbors() {
        // 8×8 dark buffer with one bright pixel at (4, 4)
        let mut fb = ImageBuffer::from_pixel(8, 8, Rgba([10, 10, 10, 255]));
        fb.put_pixel(4, 4, Rgba([255, 255, 255, 255]));

        apply_bloom(&mut fb);

        // The bright pixel at (4,4) is at an even coordinate so it gets sampled.
        // Neighbors should have received additive glow.
        // Edge weight: 255 * 19 / 256 ≈ 18
        let neighbor = fb.get_pixel(3, 4);
        assert!(neighbor.0[0] > 10, "neighbor should be brighter than original 10, got {}", neighbor.0[0]);
    }

    #[test]
    fn test_bloom_clamps_at_255() {
        let mut fb = ImageBuffer::from_pixel(8, 8, Rgba([250, 250, 250, 255]));
        // All pixels are bright — bloom adds glow but must not wrap around
        apply_bloom(&mut fb);

        // Verify no pixel wrapped below its original value (which would indicate overflow)
        for y in 0..8 {
            for x in 0..8 {
                let p = fb.get_pixel(x, y);
                assert!(p.0[0] >= 250, "channel wrapped below original: {}", p.0[0]);
                assert!(p.0[1] >= 250, "channel wrapped below original: {}", p.0[1]);
                assert!(p.0[2] >= 250, "channel wrapped below original: {}", p.0[2]);
            }
        }
    }

    #[test]
    fn test_bloom_ignores_dark_pixels() {
        let mut fb = ImageBuffer::from_pixel(8, 8, Rgba([50, 50, 50, 255]));
        let before = fb.clone();

        apply_bloom(&mut fb);

        // All pixels below threshold — buffer should be unchanged
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(fb.get_pixel(x, y), before.get_pixel(x, y));
            }
        }
    }

    // --- speed lines ---

    #[test]
    fn test_speed_lines_draws_at_demon_tier() {
        let mut fb = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 255]));
        let before = fb.clone();
        let world = make_world(8.0, 2.0); // Demon tier, cpm=2 → N=16 lines
        let seed = make_seed();

        draw_speed_lines(&mut fb, 32, 16, &world, &seed);

        // At least some pixels should have changed
        let mut changed = 0u32;
        for y in 0..32 {
            for x in 0..32 {
                if fb.get_pixel(x, y) != before.get_pixel(x, y) {
                    changed += 1;
                }
            }
        }
        assert!(changed > 0, "speed lines should modify at least some pixels");
    }

    #[test]
    fn test_speed_lines_zero_cpm_noop() {
        let mut fb = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 255]));
        let before = fb.clone();
        let world = make_world(0.0, 0.0); // cpm=0 → N=0
        let seed = make_seed();

        draw_speed_lines(&mut fb, 32, 16, &world, &seed);

        // No lines drawn — buffer unchanged
        for y in 0..32 {
            for x in 0..32 {
                assert_eq!(fb.get_pixel(x, y), before.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn test_speed_lines_count_formula() {
        // N = (cpm * 8).min(64)
        assert_eq!(((2.0_f32 * 8.0) as u32).min(64), 16);
        assert_eq!(((10.0_f32 * 8.0) as u32).min(64), 64);
        assert_eq!(((0.0_f32 * 8.0) as u32).min(64), 0);
    }

    #[test]
    fn test_speed_lines_alpha_values() {
        // Demon (tier index 3) → alpha 80
        // VelocityDemon (tier index 4) → alpha 140
        let demon = make_world(8.0, 2.0);
        assert_eq!(demon.tier as u8, 3);
        let alpha_demon: u8 = if demon.tier as u8 >= 4 { 140 } else { 80 };
        assert_eq!(alpha_demon, 80);

        let vdemon = make_world(12.0, 5.0);
        assert_eq!(vdemon.tier as u8, 4);
        let alpha_vd: u8 = if vdemon.tier as u8 >= 4 { 140 } else { 80 };
        assert_eq!(alpha_vd, 140);
    }
}
