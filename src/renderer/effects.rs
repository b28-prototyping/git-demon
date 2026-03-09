use image::{ImageBuffer, Rgba};

use crate::git::seed::RepoSeed;
use crate::world::WorldState;

const BLOOM_THRESHOLD: f32 = 0.72;
const _BLOOM_STRENGTH: f32 = 0.3;

/// Draw star-field speed streaks radiating from the vanishing point.
/// At low speed: sparse dim dots. At high speed: long bright streaks
/// like the Millennium Falcon entering hyperspace.
pub fn draw_speed_lines(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    _horizon_y: u32,
    world: &WorldState,
    seed: &RepoSeed,
) {
    let h = fb.height();
    let (cx, cy) = world.camera.vanishing_point(w, h);

    // Speed factor: 0.0 at rest, 1.0 at max base speed
    let speed_t = (world.speed / 300.0).clamp(0.0, 1.0);

    // Number of streaks scales with speed and activity
    // Only draw when there's meaningful speed
    if speed_t < 0.01 && world.commits_per_min < 0.01 {
        return;
    }
    let base_n = world.commits_per_min * 12.0;
    let n = ((base_n + speed_t * 100.0) as u32).clamp(10, 120);

    let accent = hue_to_rgb(seed.accent_hue);

    // Streak length scales with speed: short dots → long trails
    // Distances proportional to screen diagonal so it works at any resolution
    let diag = (w as f32).hypot(h as f32);
    let min_start = diag * (0.05 + (1.0 - speed_t) * 0.1);
    let max_len_f = diag * (0.05 + speed_t * 0.5);
    let alpha_base = (60.0 + speed_t * 140.0) as u8;

    // Use a deterministic hash for stable star positions
    let time_phase = (world.time * 2.0) as u32;

    for i in 0..n {
        // Pseudo-random angle and offset per streak, slowly rotating
        let hash = (i.wrapping_mul(2654435761)).wrapping_add(time_phase.wrapping_mul(17));
        let angle = (hash as f32 / u32::MAX as f32) * std::f32::consts::TAU;
        let offset_frac = (hash >> 8) as f32 / (u32::MAX >> 8) as f32;

        let dx = angle.cos();
        let dy = angle.sin();

        // Each streak starts at a random distance from center
        let start = min_start + offset_frac * min_start * 2.0;
        let streak_len = (max_len_f * (0.3 + offset_frac * 0.7)) as u32;

        // Fade alpha along streak: bright at start, dim at end
        let mut step = 0u32;
        while step < streak_len {
            let dist = start + step as f32;
            let px = (cx + dx * dist) as i32;
            let py = (cy + dy * dist) as i32;
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                break;
            }

            // Alpha fades along the streak
            let fade = 1.0 - (step as f32 / streak_len as f32);
            let a = (alpha_base as f32 * fade) as u8;
            // Color: white core at high speed, accent tint at low speed
            let white_mix = speed_t * 0.7;
            let r = (accent.0 as f32 * (1.0 - white_mix) + 255.0 * white_mix) as u8;
            let g = (accent.1 as f32 * (1.0 - white_mix) + 255.0 * white_mix) as u8;
            let b = (accent.2 as f32 * (1.0 - white_mix) + 255.0 * white_mix) as u8;
            let color = Rgba([r, g, b, a]);

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

/// Draw the player ship centered on screen.
/// Camera tracks the ship — it stays centered, the world moves around it.
/// Nose points in the direction of travel.
pub fn draw_car(fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, w: u32, h: u32, world: &WorldState) {
    // Camera tracks the ship: no lateral offset
    let cx = w as f32 / 2.0;
    // Hover above the sea — 42% gives vertical space below
    let cy = h as f32 * 0.42;

    // Real heading angle: nose points where we're going
    let heading = (world.steer_angle * 0.012).clamp(-0.5, 0.5);
    let cos_b = heading.cos();
    let sin_b = heading.sin();

    // Helper: rotate point around ship center and plot
    let plot = |fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, dx: f32, dy: f32, color: Rgba<u8>| {
        let rx = dx * cos_b - dy * sin_b;
        let ry = dx * sin_b + dy * cos_b;
        let px = (cx + rx) as i32;
        let py = (cy + ry) as i32;
        if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h {
            fb.put_pixel(px as u32, py as u32, color);
        }
    };

    // Helper: plot with darkening for shadow
    let plot_darken = |fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, dx: f32, dy: f32| {
        let rx = dx * cos_b - dy * sin_b;
        let ry = dx * sin_b + dy * cos_b;
        let px = (cx + rx) as i32;
        let py = (cy + ry) as i32;
        if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h {
            let bg = fb.get_pixel(px as u32, py as u32);
            fb.put_pixel(
                px as u32,
                py as u32,
                Rgba([bg.0[0] / 2, bg.0[1] / 2, bg.0[2] / 2, 255]),
            );
        }
    };

    // Ship colors
    let hull = Rgba([40, 65, 90, 255]);
    let hull_light = Rgba([55, 85, 115, 255]);
    let wing_color = Rgba([25, 45, 65, 255]);
    let cockpit_color = Rgba([20, 35, 55, 255]);
    let engine_glow = Rgba([0, 200, 255, 255]);
    let engine_hot = Rgba([120, 230, 255, 255]);

    // Shadow ellipse beneath ship
    let shadow_rx = 38i32;
    let shadow_ry = 5i32;
    for dy in -shadow_ry..=shadow_ry {
        let hw = shadow_rx * (shadow_ry - dy.abs()) / shadow_ry.max(1);
        for dx in -hw..=hw {
            plot_darken(fb, dx as f32, 4.0 + dy as f32);
        }
    }

    // Fuselage — elongated arrow body
    let nose = -28i32;
    let tail = 8i32;
    let body_half = 7i32;
    for dy in nose..tail {
        let t = (dy - nose) as f32 / (tail - nose) as f32;
        let hw = (body_half as f32 * (0.15 + t * 0.85)) as i32;
        let color = if t < 0.35 { hull_light } else { hull };
        for dx in -hw..=hw {
            plot(fb, dx as f32, dy as f32, color);
        }
    }

    // Delta wings — swept back from mid-body
    let wing_start = -10i32;
    let wing_end = 6i32;
    let wing_span = 30i32;
    for dy in wing_start..wing_end {
        let t = (dy - wing_start) as f32 / (wing_end - wing_start) as f32;
        let span = (wing_span as f32 * t) as i32;
        for dx in body_half..(body_half + span) {
            plot(fb, dx as f32, dy as f32, wing_color);
            plot(fb, -(dx as f32), dy as f32, wing_color);
        }
    }

    // Cockpit canopy
    for dy in (nose + 4)..(nose + 14) {
        let t = (dy - nose - 4) as f32 / 10.0;
        let hw = (body_half as f32 * 0.5 * (0.2 + t * 0.8)) as i32;
        for dx in -hw..=hw {
            plot(fb, dx as f32, dy as f32, cockpit_color);
        }
    }

    // Engine exhaust glow
    for dx in -5i32..=5 {
        for dy in tail..(tail + 5) {
            let t = (dy - tail) as f32 / 5.0;
            let color = if t < 0.4 { engine_hot } else { engine_glow };
            plot(fb, dx as f32, dy as f32, color);
        }
    }

    // --- Thruster smoke rings (hovercraft exhaust) ---
    draw_thruster_rings(fb, w, h, cx, cy, heading, world);
}

/// Draw expanding smoke rings behind the ship, like hovercraft exhaust.
/// Multiple concentric ellipses at different ages expand and fade over time.
fn draw_thruster_rings(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    cx: f32,
    cy: f32,
    heading: f32,
    world: &WorldState,
) {
    let cos_h = heading.cos();
    let sin_h = heading.sin();

    // Throttle intensity drives ring brightness and count
    let intensity = world.throttle.clamp(0.0, 1.0);
    if intensity < 0.05 {
        return;
    }

    // RPM normalized 0..1 for pulsing
    let rpm_t = ((world.rpm - 1000.0) / 7000.0).clamp(0.0, 1.0);

    // Number of visible rings (more at higher throttle)
    let ring_count = (3.0 + intensity * 5.0) as u32;

    // Time phase for ring animation — rings expand outward over time
    let phase = world.time * (2.0 + rpm_t * 4.0); // faster spin at high RPM

    for i in 0..ring_count {
        // Each ring has a different age (0 = newest, ring_count-1 = oldest)
        let age = (phase + i as f32 * 0.4) % (ring_count as f32 * 0.4);
        let age_t = age / (ring_count as f32 * 0.4); // 0..1

        // Ring expands as it ages
        let base_rx = 4.0 + age_t * 20.0 * (0.5 + intensity * 0.5);
        let base_ry = 2.0 + age_t * 8.0 * (0.5 + intensity * 0.5);

        // Offset behind the ship — rings drift backward
        let drift = 10.0 + age_t * 25.0;

        // Alpha fades as ring ages
        let alpha = ((1.0 - age_t) * intensity * 180.0) as u8;
        if alpha < 8 {
            continue;
        }

        // Color: cyan core → blue-white → dim blue as it ages
        let r = (20.0 + (1.0 - age_t) * 100.0 * rpm_t) as u8;
        let g = (80.0 + (1.0 - age_t) * 150.0) as u8;
        let b = (180.0 + (1.0 - age_t) * 75.0) as u8;
        let ring_color = Rgba([r, g, b, alpha]);

        // Draw the ellipse ring (outline only, not filled)
        let rx = base_rx as i32;
        let ry = base_ry as i32;
        if rx < 1 || ry < 1 {
            continue;
        }

        // Bresenham-style ellipse outline
        let steps = (rx.max(ry) * 6) as u32;
        for s in 0..steps {
            let angle = (s as f32 / steps as f32) * std::f32::consts::TAU;
            let ex = angle.cos() * rx as f32;
            let ey = angle.sin() * ry as f32;

            // Rotate ring by heading and offset behind ship
            let local_x = ex;
            let local_y = ey + drift;
            let rx2 = local_x * cos_h - local_y * sin_h;
            let ry2 = local_x * sin_h + local_y * cos_h;

            let px = (cx + rx2) as i32;
            let py = (cy + ry2) as i32;
            if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h {
                let bg = *fb.get_pixel(px as u32, py as u32);
                fb.put_pixel(px as u32, py as u32, blend_alpha(bg, ring_color));
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
            camera: crate::world::camera::Camera::new(),
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
            speed_multiplier: 1.0,
            curve_multiplier: 1.0,
            speed_hold_time: 0.0,
            curve_hold_time: 0.0,
            gear: 0,
            rpm: 1000.0,
            throttle: 0.0,
            shift_cooldown: 0.0,
            just_shifted: false,
            segments: {
                use crate::world::road_segments;
                let mut segs = Vec::with_capacity(road_segments::SEGMENT_COUNT);
                for i in 0..road_segments::SEGMENT_COUNT {
                    let z = i as f32 * road_segments::SEGMENT_LENGTH;
                    segs.push(road_segments::generate_segment(z, 0.0, 0.0));
                }
                segs
            },
            segment_z_start: 0.0,
            burst_cooldown: 0.0,
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
        assert!(
            neighbor.0[0] > 10,
            "neighbor should be brighter than original 10, got {}",
            neighbor.0[0]
        );
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
        assert!(
            changed > 0,
            "speed lines should modify at least some pixels"
        );
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
