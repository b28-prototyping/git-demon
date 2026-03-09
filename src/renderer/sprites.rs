use image::{ImageBuffer, Rgba};

use super::font;
use super::road;
use crate::git::seed::RepoSeed;
use crate::world::objects::{Lane, RoadsideObject};
use crate::world::WorldState;

const COMMIT_CAR_BASE_W: f32 = 40.0;
const COMMIT_CAR_BASE_H: f32 = 20.0;
const TIER_GATE_BASE_W: f32 = 80.0;

pub const ROAD_MIN_HALF: f32 = 8.0;

pub struct SpriteScreenPos {
    pub x: u32,
    pub y: u32,
    pub scale: f32,
}

pub(crate) fn project(
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
        Lane::RoadLeft => (cx - road_half_here * 0.35).max(0.0) as u32,
        Lane::RoadRight => (cx + road_half_here * 0.35) as u32,
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
            RoadsideObject::CommitCar {
                message,
                author_color,
                ..
            } => {
                let car_w = (COMMIT_CAR_BASE_W * pos.scale * pos.scale) as u32;
                let car_h = (COMMIT_CAR_BASE_H * pos.scale * pos.scale) as u32;

                if car_w < 4 {
                    // LOD: colored dot
                    draw_rect(fb, pos.x, pos.y.saturating_sub(2), 2, 2, *author_color);
                } else if car_w < 8 {
                    // LOD: colored rectangle
                    draw_rect(
                        fb,
                        pos.x.saturating_sub(car_w / 2),
                        pos.y.saturating_sub(car_h),
                        car_w,
                        car_h,
                        *author_color,
                    );
                } else {
                    // LOD: full wedge car shape
                    draw_commit_car(fb, pos.x, pos.y, car_w, car_h, *author_color);
                }

                if pos.scale >= 0.5 {
                    let text_scale = if pos.scale >= 0.65 { 2 } else { 1 };
                    let text_color = Rgba([255, 255, 255, 255]);
                    font::draw_text(
                        fb,
                        message,
                        pos.x.saturating_sub(car_w / 2),
                        pos.y.saturating_sub(car_h + 10),
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
                let gate_w = (TIER_GATE_BASE_W * 3.0 * pos.scale * pos.scale) as u32;
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

fn darken(c: Rgba<u8>, factor: f32) -> Rgba<u8> {
    Rgba([
        (c.0[0] as f32 * factor) as u8,
        (c.0[1] as f32 * factor) as u8,
        (c.0[2] as f32 * factor) as u8,
        c.0[3],
    ])
}

fn brighten(c: Rgba<u8>, factor: f32) -> Rgba<u8> {
    Rgba([
        ((c.0[0] as f32 * factor).min(255.0)) as u8,
        ((c.0[1] as f32 * factor).min(255.0)) as u8,
        ((c.0[2] as f32 * factor).min(255.0)) as u8,
        c.0[3],
    ])
}

fn draw_commit_car(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    cx: u32,
    bottom_y: u32,
    car_w: u32,
    car_h: u32,
    author_color: Rgba<u8>,
) {
    let body_color = author_color;
    let dark_side = darken(author_color, 0.5);
    let light_side = brighten(author_color, 1.3);
    let nose_h = car_h / 3;
    let fw = fb.width();
    let fh = fb.height();

    // Main body — trapezoid wider at back, narrower at front
    let body_top = bottom_y.saturating_sub(car_h);
    for dy in 0..car_h {
        let t = dy as f32 / car_h.max(1) as f32; // 0=top, 1=bottom
        let half_w = ((car_w as f32 / 2.0) * (0.5 + t * 0.5)) as u32;
        let color = if t < 0.3 {
            light_side
        } else if t < 0.7 {
            body_color
        } else {
            dark_side
        };
        let row_y = body_top + dy;
        if row_y >= fh {
            continue;
        }
        for dx in 0..=half_w * 2 {
            let px = cx.saturating_sub(half_w) + dx;
            if px < fw {
                fb.put_pixel(px, row_y, color);
            }
        }
    }

    // Nose — triangle above body
    let nose_bottom = body_top;
    for dy in 0..nose_h {
        let t = dy as f32 / nose_h.max(1) as f32; // 0=tip, 1=base
        let half_w = ((car_w as f32 / 4.0) * t) as u32;
        let row_y = nose_bottom.saturating_sub(nose_h) + dy;
        if row_y >= fh {
            continue;
        }
        for dx in 0..=half_w * 2 {
            let px = cx.saturating_sub(half_w) + dx;
            if px < fw {
                fb.put_pixel(px, row_y, light_side);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::seed::RepoSeed;
    use crate::world::objects::{Lane, RoadsideObject};
    use crate::world::speed::VelocityTier;
    use crate::world::WorldState;
    use image::{ImageBuffer, Rgba};
    use std::collections::HashMap;

    const BLACK: Rgba<u8> = Rgba([0, 0, 0, 0]);

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

    fn test_world() -> WorldState {
        let seed = test_seed();
        let mut w = WorldState::new(&seed);
        w.camera_z = 0.0;
        w.speed = 1.0;
        w.curve_offset = 0.0;
        w.tier = VelocityTier::Cruise;
        w
    }

    fn horizon_y(h: u32, world: &WorldState) -> u32 {
        (h as f32 * road::horizon_ratio(world)) as u32
    }

    // --- Projection unit tests ---

    #[test]
    fn test_project_behind_camera() {
        let mut world = test_world();
        world.camera_z = 50.0;
        let result = project(40.0, Lane::Center, &world, 400, 200, 70);
        assert!(result.is_none(), "Object behind camera should return None");
    }

    #[test]
    fn test_project_beyond_draw_distance() {
        let world = test_world();
        // draw_distance is 200.0 for Cruise tier, camera_z=0
        // z_rel = 300, depth_scale = (1 - 300/200) clamped to 0.0 → < 0.02 → None
        let result = project(300.0, Lane::Center, &world, 400, 200, 70);
        assert!(
            result.is_none(),
            "Object beyond draw distance should return None"
        );
    }

    #[test]
    fn test_project_valid_returns_some() {
        let world = test_world();
        let result = project(50.0, Lane::Center, &world, 400, 200, 70);
        assert!(result.is_some(), "In-range object should return Some");
    }

    #[test]
    fn test_project_depth_scale() {
        let world = test_world();
        // z_world=100, camera_z=0, draw_dist=200
        // depth_scale = 1 - 100/200 = 0.5
        let pos = project(100.0, Lane::Center, &world, 400, 200, 70).unwrap();
        assert!(
            (pos.scale - 0.5).abs() < 0.01,
            "Expected scale ~0.5, got {}",
            pos.scale
        );
    }

    #[test]
    fn test_project_screen_y_monotonic() {
        let world = test_world();
        let (w, h) = (400, 200);
        let hy = horizon_y(h, &world);

        let far = project(150.0, Lane::Center, &world, w, h, hy).unwrap();
        let near = project(50.0, Lane::Center, &world, w, h, hy).unwrap();

        assert!(
            near.y > far.y,
            "Nearer object should have larger y: near={}, far={}",
            near.y,
            far.y
        );
    }

    #[test]
    fn test_project_lane_left_right() {
        let world = test_world();
        let (w, h) = (400, 200);
        let hy = horizon_y(h, &world);

        let left = project(50.0, Lane::Left, &world, w, h, hy).unwrap();
        let center = project(50.0, Lane::Center, &world, w, h, hy).unwrap();
        let right = project(50.0, Lane::Right, &world, w, h, hy).unwrap();

        assert!(
            left.x < center.x,
            "Left.x ({}) should be < Center.x ({})",
            left.x,
            center.x
        );
        assert!(
            center.x < right.x,
            "Center.x ({}) should be < Right.x ({})",
            center.x,
            right.x
        );
    }

    #[test]
    fn test_project_road_left_right() {
        let world = test_world();
        let (w, h) = (400, 200);
        let hy = horizon_y(h, &world);

        let road_left = project(50.0, Lane::RoadLeft, &world, w, h, hy).unwrap();
        let center = project(50.0, Lane::Center, &world, w, h, hy).unwrap();
        let road_right = project(50.0, Lane::RoadRight, &world, w, h, hy).unwrap();
        let verge_left = project(50.0, Lane::Left, &world, w, h, hy).unwrap();
        let verge_right = project(50.0, Lane::Right, &world, w, h, hy).unwrap();

        assert!(
            road_left.x > verge_left.x,
            "RoadLeft ({}) should be closer to center than Left ({})",
            road_left.x,
            verge_left.x
        );
        assert!(
            road_left.x < center.x,
            "RoadLeft ({}) should be left of Center ({})",
            road_left.x,
            center.x
        );
        assert!(
            road_right.x > center.x,
            "RoadRight ({}) should be right of Center ({})",
            road_right.x,
            center.x
        );
        assert!(
            road_right.x < verge_right.x,
            "RoadRight ({}) should be closer to center than Right ({})",
            road_right.x,
            verge_right.x
        );
    }

    #[test]
    fn test_project_curve_shifts_x() {
        let (w, h) = (400, 200);

        let mut world_straight = test_world();
        world_straight.curve_offset = 0.0;
        let hy = horizon_y(h, &world_straight);
        let straight = project(50.0, Lane::Center, &world_straight, w, h, hy).unwrap();

        let mut world_curved = test_world();
        world_curved.curve_offset = 80.0;
        let curved = project(50.0, Lane::Center, &world_curved, w, h, hy).unwrap();

        assert!(
            curved.x > straight.x,
            "Positive curve should shift x right: straight={}, curved={}",
            straight.x,
            curved.x
        );
    }

    // --- Rendering tests ---

    #[test]
    fn test_draw_sprites_empty() {
        let (w, h) = (200, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let world = test_world();
        let seed = test_seed();
        let hy = horizon_y(h, &world);

        let before: Vec<u8> = fb.as_raw().clone();
        draw_sprites(&mut fb, w, h, hy, &world, &seed);
        assert_eq!(
            fb.as_raw().as_slice(),
            before.as_slice(),
            "Empty objects should not change buffer"
        );
    }

    #[test]
    fn test_commit_car_color() {
        let (w, h) = (1200, 400);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let seed = test_seed();
        let author_color = Rgba([200, 100, 50, 255]);

        let mut world = test_world();
        let hy = horizon_y(h, &world);
        world.active_objects.push((
            Lane::RoadLeft,
            20.0,
            RoadsideObject::CommitCar {
                message: "test".to_string(),
                author: "dev".to_string(),
                author_color,
            },
        ));

        draw_sprites(&mut fb, w, h, hy, &world, &seed);

        let has_author_color = fb.pixels().any(|p| *p == author_color);
        assert!(
            has_author_color,
            "CommitCar should render author_color pixels"
        );
    }

    #[test]
    fn test_commit_car_text_near() {
        let (w, h) = (1200, 400);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let seed = test_seed();
        let author_color = Rgba([200, 100, 50, 255]);
        let white = Rgba([255, 255, 255, 255]);

        let mut world = test_world();
        let hy = horizon_y(h, &world);
        // Place very close so scale >= 0.5 (z_rel small → scale close to 1.0)
        world.active_objects.push((
            Lane::RoadLeft,
            10.0,
            RoadsideObject::CommitCar {
                message: "Hello".to_string(),
                author: "dev".to_string(),
                author_color,
            },
        ));

        draw_sprites(&mut fb, w, h, hy, &world, &seed);

        let has_white = fb.pixels().any(|p| *p == white);
        assert!(has_white, "Near CommitCar should have white text pixels");
    }

    #[test]
    fn test_commit_car_text_suppressed_far() {
        let (w, h) = (400, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let seed = test_seed();
        let author_color = Rgba([200, 100, 50, 255]);
        let white = Rgba([255, 255, 255, 255]);

        let mut world = test_world();
        let hy = horizon_y(h, &world);
        // Place far: z_rel=160, scale = 1 - 160/200 = 0.2 < 0.5
        world.active_objects.push((
            Lane::RoadRight,
            160.0,
            RoadsideObject::CommitCar {
                message: "Hello".to_string(),
                author: "dev".to_string(),
                author_color,
            },
        ));

        draw_sprites(&mut fb, w, h, hy, &world, &seed);

        let has_white = fb.pixels().any(|p| *p == white);
        assert!(!has_white, "Far CommitCar (scale<0.5) should suppress text");
    }

    #[test]
    fn test_addition_tower_height_scales() {
        let (w, h) = (1200, 400);
        let seed = test_seed();
        let color = Rgba([100, 200, 100, 255]);

        let count_colored = |lines: u32| -> usize {
            let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
            let mut world = test_world();
            let hy = horizon_y(h, &world);
            world.active_objects.push((
                Lane::Center,
                20.0,
                RoadsideObject::AdditionTower { lines, color },
            ));
            draw_sprites(&mut fb, w, h, hy, &world, &seed);
            fb.pixels().filter(|p| **p == color).count()
        };

        let small = count_colored(100);
        let large = count_colored(400);
        assert!(
            large > small,
            "More lines should produce more pixels: 100L={small}, 400L={large}"
        );
    }

    #[test]
    fn test_deletion_shard_crimson() {
        let (w, h) = (400, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let seed = test_seed();
        let crimson = Rgba([180, 30, 30, 255]);

        let mut world = test_world();
        let hy = horizon_y(h, &world);
        world.active_objects.push((
            Lane::Left,
            20.0,
            RoadsideObject::DeletionShard { lines: 200 },
        ));

        draw_sprites(&mut fb, w, h, hy, &world, &seed);

        let has_crimson = fb.pixels().any(|p| *p == crimson);
        assert!(has_crimson, "DeletionShard should render crimson pixels");
    }

    #[test]
    fn test_tier_gate_neon_arch() {
        let (w, h) = (400, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let seed = test_seed();
        let neon = Rgba([255, 80, 255, 255]);

        let mut world = test_world();
        let hy = horizon_y(h, &world);
        world.active_objects.push((
            Lane::Center,
            20.0,
            RoadsideObject::TierGate {
                tier: VelocityTier::Demon,
            },
        ));

        draw_sprites(&mut fb, w, h, hy, &world, &seed);

        let neon_count = fb.pixels().filter(|p| **p == neon).count();
        assert!(
            neon_count > 10,
            "TierGate should render neon magenta arch pixels, got {neon_count}"
        );
    }

    #[test]
    fn test_velocity_sign_yellow() {
        let (w, h) = (400, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let seed = test_seed();
        let yellow = Rgba([255, 200, 0, 255]);

        let mut world = test_world();
        let hy = horizon_y(h, &world);
        world.active_objects.push((
            Lane::Left,
            20.0,
            RoadsideObject::VelocitySign {
                commits_per_min: 2.5,
            },
        ));

        draw_sprites(&mut fb, w, h, hy, &world, &seed);

        let has_yellow = fb.pixels().any(|p| *p == yellow);
        assert!(has_yellow, "VelocitySign should render yellow pixels");
    }

    #[test]
    fn test_commit_car_lod_dot() {
        let (w, h) = (400, 200);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let seed = test_seed();
        let author_color = Rgba([200, 100, 50, 255]);

        let mut world = test_world();
        let hy = horizon_y(h, &world);
        // Place very far: scale small enough that car_w < 4 (dot LOD)
        // scale = 1 - 180/200 = 0.1, car_w = 40 * 0.01 = 0.4 → dot
        world.active_objects.push((
            Lane::RoadLeft,
            180.0,
            RoadsideObject::CommitCar {
                message: "far".to_string(),
                author: "dev".to_string(),
                author_color,
            },
        ));

        draw_sprites(&mut fb, w, h, hy, &world, &seed);

        let has_color = fb.pixels().any(|p| *p == author_color);
        assert!(
            has_color,
            "Far CommitCar (dot LOD) should render author_color pixels"
        );
    }

    #[test]
    fn test_back_to_front_overdraw() {
        let (w, h) = (1200, 400);
        let mut fb = ImageBuffer::from_pixel(w, h, BLACK);
        let seed = test_seed();

        let far_color = Rgba([0, 0, 200, 255]);
        let near_color = Rgba([200, 0, 0, 255]);

        let mut world = test_world();
        let hy = horizon_y(h, &world);

        // Far object (will be drawn first due to back-to-front sort)
        world.active_objects.push((
            Lane::Center,
            25.0,
            RoadsideObject::AdditionTower {
                lines: 500,
                color: far_color,
            },
        ));
        // Near object at same lane, closer (will be drawn second, overwriting)
        world.active_objects.push((
            Lane::Center,
            15.0,
            RoadsideObject::AdditionTower {
                lines: 500,
                color: near_color,
            },
        ));

        draw_sprites(&mut fb, w, h, hy, &world, &seed);

        let has_near = fb.pixels().any(|p| *p == near_color);
        assert!(
            has_near,
            "Near object should be visible (overdraw far object)"
        );
    }
}
