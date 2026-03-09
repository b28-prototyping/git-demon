use criterion::{criterion_group, criterion_main, Criterion};
use git_demon::git::seed::RepoSeed;
use git_demon::renderer::FrameRenderer;
use git_demon::world::WorldState;

fn bench_render_full(c: &mut Criterion) {
    let seed = RepoSeed::compute(".").expect("Need a git repo to benchmark");
    let mut world = WorldState::new(&seed);
    // Simulate some activity so objects exist
    world.speed = 5.0;
    world.commits_per_min = 2.0;
    world.time = 10.0;

    let mut renderer = FrameRenderer::new(1920, 960, false, false, false, false, false);

    c.bench_function("render_1920x960_all_effects", |b| {
        b.iter(|| {
            world.update(1.0 / 60.0);
            let fb = renderer.render(&world, &seed);
            std::hint::black_box(fb);
        });
    });
}

fn bench_render_no_effects(c: &mut Criterion) {
    let seed = RepoSeed::compute(".").expect("Need a git repo to benchmark");
    let mut world = WorldState::new(&seed);
    world.speed = 5.0;
    world.commits_per_min = 2.0;
    world.time = 10.0;

    let mut renderer = FrameRenderer::new(1920, 960, true, true, true, true, false);

    c.bench_function("render_1920x960_no_effects", |b| {
        b.iter(|| {
            world.update(1.0 / 60.0);
            let fb = renderer.render(&world, &seed);
            std::hint::black_box(fb);
        });
    });
}

fn bench_render_320x200(c: &mut Criterion) {
    let seed = RepoSeed::compute(".").expect("Need a git repo to benchmark");
    let mut world = WorldState::new(&seed);
    world.speed = 5.0;
    world.time = 10.0;

    let mut renderer = FrameRenderer::new(320, 200, false, false, false, false, false);

    c.bench_function("render_320x200_all_effects", |b| {
        b.iter(|| {
            world.update(1.0 / 60.0);
            let fb = renderer.render(&world, &seed);
            std::hint::black_box(fb);
        });
    });
}

criterion_group!(benches, bench_render_full, bench_render_no_effects, bench_render_320x200);
criterion_main!(benches);
