use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, Criterion};
use image::{ImageBuffer, Rgba};

use git_demon::git::seed::RepoSeed;
use git_demon::renderer::FrameRenderer;
use git_demon::world::objects::{Lane, RoadsideObject};
use git_demon::world::speed::VelocityTier;
use git_demon::world::WorldState;

const W: u32 = 1920;
const H: u32 = 960;

fn bench_seed() -> RepoSeed {
    let mut author_colors = HashMap::new();
    author_colors.insert("Alice".to_string(), Rgba([100, 200, 100, 255]));
    author_colors.insert("Bob".to_string(), Rgba([100, 100, 200, 255]));
    author_colors.insert("Carol".to_string(), Rgba([200, 100, 100, 255]));

    RepoSeed {
        accent_hue: 180.0,
        saturation: 0.8,
        terrain_roughness: 0.6,
        speed_base: 0.5,
        author_colors,
        total_commits: 500,
        repo_name: "bench-repo".into(),
    }
}

fn bench_world(seed: &RepoSeed) -> WorldState {
    let mut world = WorldState::new(seed);
    world.speed = 5.0;
    world.speed_target = 5.0;
    world.commits_per_min = 2.0;
    world.time = 10.0;
    world.camera_z = 50.0;
    world.z_offset = 50.0;
    world.curve_offset = 20.0;
    world.curve_target = 30.0;
    world.tier = VelocityTier::Demon;

    // Populate active objects across the draw distance
    let objects = vec![
        (
            Lane::RoadLeft,
            60.0,
            RoadsideObject::CommitCar {
                message: "fix: resolve edge case".into(),
                author: "Alice".into(),
                author_color: Rgba([100, 200, 100, 255]),
            },
        ),
        (
            Lane::Right,
            70.0,
            RoadsideObject::AdditionTower {
                lines: 150,
                color: Rgba([100, 200, 100, 255]),
            },
        ),
        (
            Lane::RoadLeft,
            80.0,
            RoadsideObject::CommitCar {
                message: "feat: add new module".into(),
                author: "Bob".into(),
                author_color: Rgba([100, 100, 200, 255]),
            },
        ),
        (
            Lane::Right,
            90.0,
            RoadsideObject::DeletionShard { lines: 80 },
        ),
        (
            Lane::Center,
            100.0,
            RoadsideObject::TierGate {
                tier: VelocityTier::Demon,
            },
        ),
        (
            Lane::Left,
            110.0,
            RoadsideObject::VelocitySign {
                commits_per_min: 2.0,
            },
        ),
        (
            Lane::RoadRight,
            120.0,
            RoadsideObject::CommitCar {
                message: "refactor: clean up API".into(),
                author: "Carol".into(),
                author_color: Rgba([200, 100, 100, 255]),
            },
        ),
        (
            Lane::Left,
            130.0,
            RoadsideObject::AdditionTower {
                lines: 300,
                color: Rgba([200, 100, 100, 255]),
            },
        ),
        (
            Lane::RoadRight,
            140.0,
            RoadsideObject::CommitCar {
                message: "docs: update readme".into(),
                author: "Alice".into(),
                author_color: Rgba([100, 200, 100, 255]),
            },
        ),
        (
            Lane::Left,
            160.0,
            RoadsideObject::DeletionShard { lines: 120 },
        ),
        (
            Lane::Right,
            180.0,
            RoadsideObject::VelocitySign {
                commits_per_min: 1.5,
            },
        ),
        (
            Lane::RoadLeft,
            200.0,
            RoadsideObject::CommitCar {
                message: "chore: bump deps".into(),
                author: "Bob".into(),
                author_color: Rgba([100, 100, 200, 255]),
            },
        ),
    ];

    world.active_objects = objects;
    world
}

fn bench_world_vdemon(seed: &RepoSeed) -> WorldState {
    let mut world = bench_world(seed);
    world.commits_per_min = 5.0;
    world.speed = 12.0;
    world.speed_target = 12.0;
    world.tier = VelocityTier::VelocityDemon;
    world
}

// --- Full pipeline benchmarks ---

fn bench_render_full_effects(c: &mut Criterion) {
    let seed = bench_seed();
    let mut world = bench_world(&seed);
    let mut renderer = FrameRenderer::new(W, H, false, false, false, false, false);

    c.bench_function("full_pipeline/all_effects_1920x960", |b| {
        b.iter(|| {
            world.update(1.0 / 60.0);
            let fb = renderer.render(&world, &seed);
            std::hint::black_box(fb);
        });
    });
}

fn bench_render_no_effects(c: &mut Criterion) {
    let seed = bench_seed();
    let mut world = bench_world(&seed);
    let mut renderer = FrameRenderer::new(W, H, true, true, true, true, false);

    c.bench_function("full_pipeline/no_effects_1920x960", |b| {
        b.iter(|| {
            world.update(1.0 / 60.0);
            let fb = renderer.render(&world, &seed);
            std::hint::black_box(fb);
        });
    });
}

fn bench_render_velocity_demon(c: &mut Criterion) {
    let seed = bench_seed();
    let mut world = bench_world_vdemon(&seed);
    let mut renderer = FrameRenderer::new(W, H, false, false, false, false, false);

    c.bench_function("full_pipeline/velocity_demon_1920x960", |b| {
        b.iter(|| {
            world.update(1.0 / 60.0);
            let fb = renderer.render(&world, &seed);
            std::hint::black_box(fb);
        });
    });
}

// --- Individual pass benchmarks ---

fn bench_sky(c: &mut Criterion) {
    let seed = bench_seed();
    let world = bench_world(&seed);
    let horizon_y = (H as f32 * git_demon::renderer::road::horizon_ratio(&world)) as u32;
    let mut fb = ImageBuffer::new(W, H);

    c.bench_function("pass/sky", |b| {
        b.iter(|| {
            git_demon::renderer::sky::draw_sky(&mut fb, W, H, horizon_y, &seed, &world);
            git_demon::renderer::sky::draw_sun(&mut fb, W, horizon_y, &seed, &world);
            git_demon::renderer::sky::draw_bloom_bleed(&mut fb, W, horizon_y, &seed, &world);
            std::hint::black_box(&fb);
        });
    });
}

fn bench_road(c: &mut Criterion) {
    let seed = bench_seed();
    let world = bench_world(&seed);
    let horizon_y = (H as f32 * git_demon::renderer::road::horizon_ratio(&world)) as u32;
    let mut fb = ImageBuffer::new(W, H);
    // Pre-render sky so buffer is in realistic state
    git_demon::renderer::sky::draw_sky(&mut fb, W, H, horizon_y, &seed, &world);

    c.bench_function("pass/road", |b| {
        b.iter(|| {
            git_demon::renderer::road::draw_road(&mut fb, W, H, horizon_y, &world, &seed);
            git_demon::renderer::road::draw_grid(&mut fb, W, H, horizon_y, &world, &seed);
            std::hint::black_box(&fb);
        });
    });
}

fn bench_terrain(c: &mut Criterion) {
    let seed = bench_seed();
    let world = bench_world(&seed);
    let horizon_y = (H as f32 * git_demon::renderer::road::horizon_ratio(&world)) as u32;
    let mut fb = ImageBuffer::new(W, H);

    c.bench_function("pass/terrain", |b| {
        b.iter(|| {
            git_demon::renderer::terrain::draw_terrain(&mut fb, W, H, horizon_y, &seed, &world);
            std::hint::black_box(&fb);
        });
    });
}

fn bench_sprites(c: &mut Criterion) {
    let seed = bench_seed();
    let world = bench_world(&seed);
    let horizon_y = (H as f32 * git_demon::renderer::road::horizon_ratio(&world)) as u32;
    let mut fb = ImageBuffer::new(W, H);
    // Pre-render road so sprites draw over realistic background
    git_demon::renderer::sky::draw_sky(&mut fb, W, H, horizon_y, &seed, &world);
    git_demon::renderer::road::draw_road(&mut fb, W, H, horizon_y, &world, &seed);

    c.bench_function("pass/sprites", |b| {
        b.iter(|| {
            git_demon::renderer::sprites::draw_sprites(&mut fb, W, H, horizon_y, &world, &seed);
            std::hint::black_box(&fb);
        });
    });
}

fn bench_effects(c: &mut Criterion) {
    let seed = bench_seed();
    let world = bench_world(&seed);
    let mut fb = ImageBuffer::new(W, H);
    let prev_fb = ImageBuffer::new(W, H);
    // Fill with some data so bloom/scanline have realistic input
    let horizon_y = (H as f32 * git_demon::renderer::road::horizon_ratio(&world)) as u32;
    git_demon::renderer::sky::draw_sky(&mut fb, W, H, horizon_y, &seed, &world);
    git_demon::renderer::road::draw_road(&mut fb, W, H, horizon_y, &world, &seed);

    c.bench_function("pass/effects", |b| {
        b.iter(|| {
            git_demon::renderer::effects::apply_motion_blur(&mut fb, &prev_fb, &world);
            git_demon::renderer::effects::apply_scanline_filter(&mut fb);
            git_demon::renderer::effects::apply_bloom(&mut fb);
            std::hint::black_box(&fb);
        });
    });
}

criterion_group!(
    full_pipeline,
    bench_render_full_effects,
    bench_render_no_effects,
    bench_render_velocity_demon
);
criterion_group!(
    individual_passes,
    bench_sky,
    bench_road,
    bench_terrain,
    bench_sprites,
    bench_effects
);
criterion_main!(full_pipeline, individual_passes);
