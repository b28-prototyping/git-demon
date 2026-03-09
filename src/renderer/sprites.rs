use image::{ImageBuffer, Rgba};

use super::font;
use super::road;
use crate::git::seed::RepoSeed;
use crate::world::objects::{Lane, RoadsideObject};
use crate::world::WorldState;

const BILLBOARD_BASE_W: f32 = 80.0;
const BILLBOARD_BASE_H: f32 = 40.0;

pub const ROAD_MIN_HALF: f32 = 8.0;

pub struct SpriteScreenPos {
    pub x: u32,
    pub y: u32,
    pub scale: f32,
}

fn project(
    z_world: f32,
    lane: Lane,
    world: &WorldState,
    pixel_w: u32,
    pixel_h: u32,
    horizon_y: u32,
) -> Option<SpriteScreenPos> {
    let z_rel = z_world - world.camera_z;
    if z_rel <= 0.0 {
        return None;
    }

    let draw_dist = world.draw_distance();
    let depth_scale = (1.0 - z_rel / draw_dist).clamp(0.0, 1.0);
    if depth_scale < 0.02 {
        return None;
    }

    let screen_y = lerp(horizon_y as f32, pixel_h as f32, depth_scale) as u32;
    let max_half = road::road_max_half(world);
    let road_half_here = lerp(ROAD_MIN_HALF, max_half, depth_scale);

    let cx = pixel_w as f32 / 2.0 + world.curve_offset * depth_scale * depth_scale;

    let lane_x = match lane {
        Lane::Left => (cx - road_half_here * 1.15).max(0.0) as u32,
        Lane::Right => (cx + road_half_here * 1.15) as u32,
        Lane::Center => cx as u32,
    };

    Some(SpriteScreenPos {
        x: lane_x,
        y: screen_y,
        scale: depth_scale,
    })
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn draw_sprites(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    horizon_y: u32,
    world: &WorldState,
    _seed: &RepoSeed,
) {
    // Sort by z_world descending (far objects first)
    let mut sorted: Vec<_> = world.active_objects.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (lane, z_world, obj) in sorted {
        let Some(pos) = project(*z_world, *lane, world, w, h, horizon_y) else {
            continue;
        };

        match obj {
            RoadsideObject::CommitBillboard {
                message,
                author_color,
                ..
            } => {
                let sw = (BILLBOARD_BASE_W * pos.scale * pos.scale) as u32;
                let sh = (BILLBOARD_BASE_H * pos.scale * pos.scale) as u32;
                draw_rect(
                    fb,
                    pos.x.saturating_sub(sw / 2),
                    pos.y.saturating_sub(sh),
                    sw,
                    sh,
                    *author_color,
                );

                if pos.scale >= 0.35 {
                    let text_scale = if pos.scale >= 0.65 { 2 } else { 1 };
                    let text_color = Rgba([255, 255, 255, 255]);
                    font::draw_text(
                        fb,
                        message,
                        pos.x.saturating_sub(sw / 2) + 2,
                        pos.y.saturating_sub(sh / 2),
                        text_color,
                        text_scale,
                    );
                }
            }
            RoadsideObject::AdditionTower { lines, color } => {
                let tower_h = ((*lines as f32).sqrt() * 4.0 * pos.scale * pos.scale) as u32;
                let tower_w = (12.0 * pos.scale * pos.scale) as u32;
                draw_rect(
                    fb,
                    pos.x.saturating_sub(tower_w / 2),
                    pos.y.saturating_sub(tower_h),
                    tower_w,
                    tower_h,
                    *color,
                );
            }
            RoadsideObject::DeletionShard { lines } => {
                let shard_h = ((*lines as f32).sqrt() * 3.0 * pos.scale * pos.scale) as u32;
                let shard_w = (8.0 * pos.scale * pos.scale) as u32;
                let color = Rgba([180, 30, 30, 255]);
                draw_rect(
                    fb,
                    pos.x.saturating_sub(shard_w / 2),
                    pos.y.saturating_sub(shard_h),
                    shard_w,
                    shard_h,
                    color,
                );
            }
            RoadsideObject::TierGate { tier } => {
                let gate_h = (60.0 * pos.scale * pos.scale) as u32;
                let gate_w = (BILLBOARD_BASE_W * 3.0 * pos.scale * pos.scale) as u32;
                let neon = Rgba([255, 80, 255, 255]);
                draw_rect(
                    fb,
                    pos.x.saturating_sub(gate_w / 2),
                    pos.y.saturating_sub(gate_h),
                    gate_w,
                    3,
                    neon,
                );
                draw_rect(
                    fb,
                    pos.x.saturating_sub(gate_w / 2),
                    pos.y.saturating_sub(gate_h),
                    3,
                    gate_h,
                    neon,
                );
                draw_rect(
                    fb,
                    pos.x + gate_w / 2,
                    pos.y.saturating_sub(gate_h),
                    3,
                    gate_h,
                    neon,
                );

                if pos.scale >= 0.35 {
                    let text_scale = if pos.scale >= 0.65 { 2 } else { 1 };
                    font::draw_text(
                        fb,
                        tier.name(),
                        pos.x.saturating_sub(gate_w / 4),
                        pos.y.saturating_sub(gate_h / 2),
                        neon,
                        text_scale,
                    );
                }
            }
            RoadsideObject::VelocitySign { commits_per_min } => {
                let size = (20.0 * pos.scale * pos.scale) as u32;
                let color = Rgba([255, 200, 0, 255]);
                draw_rect(
                    fb,
                    pos.x.saturating_sub(size / 2),
                    pos.y.saturating_sub(size),
                    size,
                    size,
                    color,
                );

                if pos.scale >= 0.35 {
                    let text = format!("{:.1}", commits_per_min);
                    font::draw_text(
                        fb,
                        &text,
                        pos.x.saturating_sub(size / 2) + 2,
                        pos.y.saturating_sub(size / 2),
                        Rgba([0, 0, 0, 255]),
                        1,
                    );
                }
            }
            _ => {
                let size = (16.0 * pos.scale * pos.scale) as u32;
                let color = Rgba([100, 100, 120, 255]);
                draw_rect(
                    fb,
                    pos.x.saturating_sub(size / 2),
                    pos.y.saturating_sub(size),
                    size.max(2),
                    size.max(2),
                    color,
                );
            }
        }
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
