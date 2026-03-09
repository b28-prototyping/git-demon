use image::{ImageBuffer, Rgba};

use super::font;
use crate::git::seed::RepoSeed;
use crate::world::speed::VelocityTier;
use crate::world::WorldState;

const HUD_HEIGHT: u32 = 18;
const HUD_BG: Rgba<u8> = Rgba([0, 0, 0, 200]);

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

#[allow(clippy::too_many_arguments)]
pub fn draw_dev_overlay(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    world: &WorldState,
    seed: &RepoSeed,
    frame_count: u64,
    render_us: u128,
    actual_fps: f32,
    target_fps: u32,
    encode_send_us: u128,
) {
    let lines = [
        "-- DEV --".to_string(),
        format!(
            "frame: {}  render: {:.1}ms  fps: {:.1}/{}  enc: {:.1}ms",
            frame_count,
            render_us as f64 / 1000.0,
            actual_fps,
            target_fps,
            encode_send_us as f64 / 1000.0,
        ),
        format!("res: {}x{}", w, fb.height()),
        format!("speed: {:.2} -> {:.2}", world.speed, world.speed_target),
        format!("tier: {:?}", world.tier),
        format!("cpm: {:.2}", world.commits_per_min),
        format!("cam_z: {:.1}  z_off: {:.1}", world.camera_z, world.z_offset),
        format!(
            "curve: {:.1} -> {:.1}  steer: {:.1}",
            world.curve_offset, world.curve_target, world.steer_angle
        ),
        format!(
            "objects: {} active  {} pending",
            world.active_objects.len(),
            world.pending_objects.len()
        ),
        format!(
            "+{} -{} {} files",
            world.lines_added, world.lines_deleted, world.files_changed
        ),
        format!(
            "hue: {:.0}  rough: {:.2}",
            seed.accent_hue, seed.terrain_roughness
        ),
        format!("repo: {}", seed.repo_name),
    ];

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

fn tier_badge_color(world: &WorldState) -> Rgba<u8> {
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
