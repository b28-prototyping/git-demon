use image::{ImageBuffer, Rgba};

use super::font;
use crate::git::seed::RepoSeed;
use crate::world::speed::VelocityTier;
use crate::world::WorldState;

const HUD_HEIGHT: u32 = 18;
const HUD_BG: Rgba<u8> = Rgba([0, 0, 0, 204]);

pub fn draw_hud(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    world: &WorldState,
    seed: &RepoSeed,
) {
    let hud_y = h.saturating_sub(HUD_HEIGHT);

    // Draw background strip
    for y in hud_y..h {
        for x in 0..w {
            let bg = fb.get_pixel(x, y);
            let a = HUD_BG.0[3] as f32 / 255.0;
            let inv = 1.0 - a;
            fb.put_pixel(
                x,
                y,
                Rgba([
                    (HUD_BG.0[0] as f32 * a + bg.0[0] as f32 * inv) as u8,
                    (HUD_BG.0[1] as f32 * a + bg.0[1] as f32 * inv) as u8,
                    (HUD_BG.0[2] as f32 * a + bg.0[2] as f32 * inv) as u8,
                    255,
                ]),
            );
        }
    }

    let text_y = hud_y + 4;
    let white = Rgba([255, 255, 255, 255]);

    // Sector
    let sector_text = format!("SECTOR {}", world.sector());
    font::draw_text(fb, &sector_text, 8, text_y, white, 1);

    // Commits/min
    let cpm_text = format!("{:.1} c/min", world.commits_per_min);
    font::draw_text(fb, &cpm_text, 120, text_y, white, 1);

    // Lines added/deleted
    let lines_text = format!("+{}  -{}", world.lines_added, world.lines_deleted);
    font::draw_text(fb, &lines_text, 240, text_y, white, 1);

    // Files
    let files_text = format!("{} files", world.files_changed);
    font::draw_text(fb, &files_text, 380, text_y, white, 1);

    // Tier badge
    let tier_color = tier_badge_color(world);
    font::draw_text(fb, world.tier.name(), 480, text_y, tier_color, 1);

    // Repo name
    let repo_text = format!("repo: {}", seed.repo_name);
    let repo_x = w.saturating_sub(font::text_width(&repo_text, 1) + 8);
    font::draw_text(fb, &repo_text, repo_x, text_y, white, 1);
}

pub const DEV_PAGE_COUNT: u8 = 3;

#[allow(clippy::too_many_arguments)]
pub fn draw_dev_overlay(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    world: &WorldState,
    seed: &RepoSeed,
    frame_count: u64,
    render_us: u128,
    world_fps: f32,
    render_fps: f32,
    target_render_fps: u32,
    encode_send_us: u128,
    pass: &super::PassTimings,
    page: u8,
) {
    let us = |v: u128| v as f64 / 1000.0;
    let lines: Vec<String> = match page {
        // Page 0: Performance
        0 => vec![
            format!("-- PERF [{}/{}] (p:next) --", page + 1, DEV_PAGE_COUNT),
            format!(
                "frame: {}  raster: {:.1}ms  enc: {:.1}ms",
                frame_count,
                render_us as f64 / 1000.0,
                encode_send_us as f64 / 1000.0,
            ),
            format!(
                "sky:{:.1} ter:{:.1} road:{:.1} spr:{:.1}",
                us(pass.sky_us),
                us(pass.terrain_us),
                us(pass.road_us),
                us(pass.sprites_us),
            ),
            format!(
                "blur:{:.1} bloom:{:.1} scan:{:.1} hud:{:.1}",
                us(pass.blur_us),
                us(pass.bloom_us),
                us(pass.scanline_us),
                us(pass.hud_us),
            ),
            format!(
                "world: {:.0}hz  render: {:.0}/{}fps",
                world_fps, render_fps, target_render_fps,
            ),
            format!("res: {}x{}", w, fb.height()),
        ],
        // Page 1: World state
        1 => vec![
            format!("-- WORLD [{}/{}] (p:next) --", page + 1, DEV_PAGE_COUNT),
            format!(
                "speed: {:.0} -> {:.0}  x{:.1}",
                world.speed, world.speed_target, world.speed_multiplier
            ),
            format!(
                "tier: {:?}  curve_mul: {:.1}",
                world.tier, world.curve_multiplier
            ),
            format!("cpm: {:.2}", world.commits_per_min),
            format!("cam_z: {:.0}  z_off: {:.0}", world.camera_z, world.z_offset),
            format!(
                "curve: {:.1} -> {:.1}  steer: {:.1}",
                world.curve_offset, world.curve_target, world.steer_angle
            ),
            format!(
                "objects: {} active  {} pending",
                world.active_objects.len(),
                world.pending_objects.len()
            ),
        ],
        // Page 2: Repo info
        _ => vec![
            format!("-- REPO [{}/{}] (p:next) --", page + 1, DEV_PAGE_COUNT),
            format!(
                "+{} -{} {} files",
                world.lines_added, world.lines_deleted, world.files_changed
            ),
            format!(
                "hue: {:.0}  rough: {:.2}",
                seed.accent_hue, seed.terrain_roughness
            ),
            format!("repo: {}", seed.repo_name),
        ],
    };

    let line_h = 10u32;
    let pad = 4u32;
    let panel_h = lines.len() as u32 * line_h + pad * 2;
    let panel_w = 240u32;
    let panel_x = 4u32;
    let panel_y = 4u32;

    // Semi-transparent background
    let bg = Rgba([0, 0, 0, 180]);
    for y in panel_y..(panel_y + panel_h).min(fb.height()) {
        for x in panel_x..(panel_x + panel_w).min(w) {
            let existing = *fb.get_pixel(x, y);
            let a = bg.0[3] as f32 / 255.0;
            let inv = 1.0 - a;
            fb.put_pixel(
                x,
                y,
                Rgba([
                    (bg.0[0] as f32 * a + existing.0[0] as f32 * inv) as u8,
                    (bg.0[1] as f32 * a + existing.0[1] as f32 * inv) as u8,
                    (bg.0[2] as f32 * a + existing.0[2] as f32 * inv) as u8,
                    255,
                ]),
            );
        }
    }

    let green = Rgba([0, 255, 128, 255]);
    for (i, line) in lines.iter().enumerate() {
        font::draw_text(
            fb,
            line,
            panel_x + pad,
            panel_y + pad + i as u32 * line_h,
            green,
            1,
        );
    }
}

pub(crate) fn tier_badge_color(world: &WorldState) -> Rgba<u8> {
    match world.tier {
        VelocityTier::Flatline => Rgba([100, 100, 100, 255]),
        VelocityTier::Cruise => Rgba([255, 255, 255, 255]),
        VelocityTier::Active => Rgba([0, 255, 255, 255]),
        VelocityTier::Demon => Rgba([255, 165, 0, 255]),
        VelocityTier::VelocityDemon => {
            // Strobe red/white at 4Hz
            if ((world.time * 4.0) as u32).is_multiple_of(2) {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};

    fn make_world() -> WorldState {
        WorldState {
            z_offset: 0.0,
            camera_z: 0.0,
            speed: 1.0,
            speed_target: 1.0,
            commits_per_min: 2.5,
            lines_added: 42,
            lines_deleted: 7,
            files_changed: 3,
            tier: VelocityTier::Active,
            time: 0.0,
            total_commits: 450,
            pending_objects: VecDeque::new(),
            active_objects: Vec::new(),
            curve_offset: 0.0,
            curve_target: 0.0,
            steer_angle: 0.0,
            speed_multiplier: 1.0,
            curve_multiplier: 1.0,
            speed_hold_time: 0.0,
            curve_hold_time: 0.0,
        }
    }

    fn make_seed() -> RepoSeed {
        RepoSeed {
            accent_hue: 180.0,
            saturation: 0.8,
            terrain_roughness: 0.5,
            speed_base: 0.5,
            author_colors: HashMap::new(),
            total_commits: 450,
            repo_name: "test-repo".into(),
        }
    }

    // --- tier badge colors ---

    #[test]
    fn test_tier_badge_flatline() {
        let mut world = make_world();
        world.tier = VelocityTier::Flatline;
        assert_eq!(tier_badge_color(&world), Rgba([100, 100, 100, 255]));
    }

    #[test]
    fn test_tier_badge_cruise() {
        let mut world = make_world();
        world.tier = VelocityTier::Cruise;
        assert_eq!(tier_badge_color(&world), Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn test_tier_badge_active() {
        let mut world = make_world();
        world.tier = VelocityTier::Active;
        assert_eq!(tier_badge_color(&world), Rgba([0, 255, 255, 255]));
    }

    #[test]
    fn test_tier_badge_demon() {
        let mut world = make_world();
        world.tier = VelocityTier::Demon;
        assert_eq!(tier_badge_color(&world), Rgba([255, 165, 0, 255]));
    }

    #[test]
    fn test_tier_badge_velocity_demon_strobe() {
        let mut world = make_world();
        world.tier = VelocityTier::VelocityDemon;

        // At time=0.0: (0.0 * 4.0) as u32 = 0, 0 % 2 == 0 → red
        world.time = 0.0;
        assert_eq!(tier_badge_color(&world), Rgba([255, 0, 0, 255]));

        // At time=0.25: (0.25 * 4.0) as u32 = 1, 1 % 2 != 0 → white
        world.time = 0.26; // slightly past to avoid float boundary
        assert_eq!(tier_badge_color(&world), Rgba([255, 255, 255, 255]));

        // At time=0.50: (0.5 * 4.0) as u32 = 2, 2 % 2 == 0 → red
        world.time = 0.50;
        assert_eq!(tier_badge_color(&world), Rgba([255, 0, 0, 255]));
    }

    // --- HUD background alpha ---

    #[test]
    fn test_hud_background_alpha_blend() {
        // Over a pure white background, HUD_BG (0,0,0,204) should produce:
        // result = 0 * (204/255) + 255 * (1 - 204/255) = 255 * 51/255 = 51
        let (w, h) = (600, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, Rgba([255, 255, 255, 255]));
        let world = make_world();
        let seed = make_seed();

        draw_hud(&mut fb, w, h, &world, &seed);

        let hud_y = h - HUD_HEIGHT;
        // Check a pixel in the HUD region that should only have background
        // (far left, below any text). The blend should darken white to ~51.
        // But text may overlap, so check a pixel at x=0 which is before text starts at x=8
        let p = fb.get_pixel(0, hud_y + 1);
        // alpha = 204/255 ≈ 0.8, inv ≈ 0.2
        // result = 0 * 0.8 + 255 * 0.2 = 51
        let expected = (255.0 * (1.0 - 204.0 / 255.0)) as u8;
        assert_eq!(
            p.0[0], expected,
            "Red channel: expected {expected}, got {}",
            p.0[0]
        );
        assert_eq!(p.0[1], expected, "Green channel");
        assert_eq!(p.0[2], expected, "Blue channel");
        assert_eq!(p.0[3], 255, "Alpha should be fully opaque");
    }

    // --- HUD renders in correct region ---

    #[test]
    fn test_hud_modifies_bottom_strip() {
        let (w, h) = (600, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, Rgba([128, 128, 128, 255]));
        let world = make_world();
        let seed = make_seed();

        draw_hud(&mut fb, w, h, &world, &seed);

        let hud_y = h - HUD_HEIGHT;
        // At least some pixels in the HUD region should differ from the original 128
        let mut changed = 0u32;
        for y in hud_y..h {
            for x in 0..w {
                if fb.get_pixel(x, y).0[0] != 128 {
                    changed += 1;
                }
            }
        }
        assert!(changed > 0, "HUD should modify pixels in bottom strip");
    }

    #[test]
    fn test_hud_does_not_modify_above_strip() {
        let (w, h) = (600, 200);
        let gray = Rgba([128, 128, 128, 255]);
        let mut fb = ImageBuffer::from_pixel(w, h, gray);
        let world = make_world();
        let seed = make_seed();

        draw_hud(&mut fb, w, h, &world, &seed);

        let hud_y = h - HUD_HEIGHT;
        // Pixels above the HUD strip should be unchanged
        for y in 0..hud_y {
            for x in 0..w {
                assert_eq!(
                    *fb.get_pixel(x, y),
                    gray,
                    "Pixel ({x},{y}) above HUD should be unchanged"
                );
            }
        }
    }

    // --- sector calculation ---

    #[test]
    fn test_sector_calculation() {
        let mut world = make_world();
        world.total_commits = 450;
        assert_eq!(world.sector(), 4); // 450 / 100 = 4
    }

    #[test]
    fn test_sector_zero() {
        let mut world = make_world();
        world.total_commits = 50;
        assert_eq!(world.sector(), 0); // 50 / 100 = 0
    }

    // --- repo name right-aligned ---

    #[test]
    fn test_repo_name_right_aligned() {
        let (w, h) = (800, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, Rgba([0, 0, 0, 255]));
        let world = make_world();
        let seed = make_seed();

        draw_hud(&mut fb, w, h, &world, &seed);

        let text_y = h - HUD_HEIGHT + 4;
        let white = Rgba([255, 255, 255, 255]);

        // The repo text should have white pixels near the right edge
        let mut rightmost_white = 0u32;
        for x in 0..w {
            for y in text_y..(text_y + 7) {
                if *fb.get_pixel(x, y) == white {
                    rightmost_white = rightmost_white.max(x);
                }
            }
        }
        // Right edge of text should be within 8px + glyph width of right edge
        assert!(
            rightmost_white > w - 20,
            "Rightmost white pixel at {rightmost_white}, expected near {w}"
        );
    }

    // --- HUD_HEIGHT constant ---

    #[test]
    fn test_hud_height_is_18() {
        assert_eq!(HUD_HEIGHT, 18);
    }
}
