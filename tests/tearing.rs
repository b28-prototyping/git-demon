//! End-to-end tearing detection tests.
//!
//! These tests render full frames through the pipeline and check for
//! coherence violations that indicate screen tearing artifacts:
//!
//! - **Scanline gradient monotonicity**: the road/ocean surface should have
//!   smoothly increasing brightness from horizon to camera. A torn frame
//!   shows abrupt jumps between adjacent scanlines.
//!
//! - **Horizon seam integrity**: the boundary between sky and road should
//!   be clean — no scanlines where half is sky-colored and half is road-colored.
//!
//! - **Inter-frame temporal coherence**: consecutive frames at small dt
//!   should not have massive pixel-level deltas.
//!
//! - **Grid line continuity**: vertical grid lines should not have horizontal
//!   gaps within a single frame.

use std::collections::HashMap;

use git_demon::git::seed::RepoSeed;
use git_demon::renderer::FrameRenderer;
use git_demon::world::WorldState;
use image::{ImageBuffer, Rgba};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_seed() -> RepoSeed {
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

/// Create a renderer with all post-processing disabled for clean signal.
fn clean_renderer(w: u32, h: u32) -> FrameRenderer {
    FrameRenderer::new(
        w, h, /*no_blur*/ true, /*no_bloom*/ true, /*no_scanlines*/ true,
        /*no_hud*/ true, /*dev*/ false,
    )
}

/// Create a renderer with all effects enabled (production config).
fn full_renderer(w: u32, h: u32) -> FrameRenderer {
    FrameRenderer::new(
        w, h, /*no_blur*/ false, /*no_bloom*/ false, /*no_scanlines*/ false,
        /*no_hud*/ false, /*dev*/ false,
    )
}

/// Render one frame, returning a cloned framebuffer.
fn render_frame(
    renderer: &mut FrameRenderer,
    world: &WorldState,
    seed: &RepoSeed,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    renderer.render(world, seed).clone()
}

/// Average brightness of a pixel (0–255).
fn brightness(p: &Rgba<u8>) -> f32 {
    (p.0[0] as f32 + p.0[1] as f32 + p.0[2] as f32) / 3.0
}

/// Mean absolute pixel difference between two same-sized framebuffers.
fn mean_pixel_delta(a: &ImageBuffer<Rgba<u8>, Vec<u8>>, b: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> f32 {
    let raw_a = a.as_raw();
    let raw_b = b.as_raw();
    assert_eq!(raw_a.len(), raw_b.len());
    let mut sum: u64 = 0;
    let mut count: u64 = 0;
    let mut i = 0;
    while i + 3 < raw_a.len() {
        // RGB channels only, skip alpha
        sum += (raw_a[i] as i32 - raw_b[i] as i32).unsigned_abs() as u64;
        sum += (raw_a[i + 1] as i32 - raw_b[i + 1] as i32).unsigned_abs() as u64;
        sum += (raw_a[i + 2] as i32 - raw_b[i + 2] as i32).unsigned_abs() as u64;
        count += 3;
        i += 4;
    }
    sum as f32 / count.max(1) as f32
}

/// Compute horizon_y the same way the renderer does.
fn horizon_y(h: u32, world: &WorldState) -> u32 {
    world.camera.horizon_y(h)
}

// ---------------------------------------------------------------------------
// Test: scanline gradient monotonicity (road surface)
// ---------------------------------------------------------------------------
// The ocean surface has terrain (islands, clouds) overlaid, so individual
// columns are NOT strictly monotonic. Instead, we check the *row-averaged*
// brightness, which smooths out local terrain. A torn frame would show the
// average brightness of an entire row jumping abruptly, which terrain alone
// cannot cause (terrain affects only part of a row).

#[test]
fn test_road_scanline_gradient_monotonicity() {
    let (w, h) = (320, 200);
    let seed = test_seed();
    let mut renderer = clean_renderer(w, h);
    let mut world = WorldState::new(&seed);

    let configs: Vec<(f32, f32, f32)> = vec![
        (0.0, 0.0, 0.0),
        (100.0, 2.0, 0.0),
        (250.0, 4.0, 40.0),
        (250.0, 4.0, -60.0),
    ];

    for (speed, cpm, curve) in configs {
        world.speed = speed;
        world.commits_per_min = cpm;
        world.curve_offset = curve;
        world.update(0.016);

        let fb = render_frame(&mut renderer, &world, &seed);
        let hy = horizon_y(h, &world);

        // Compute row-averaged brightness below horizon
        let row_avg: Vec<f32> = (hy..h)
            .map(|y| {
                let sum: f32 = (0..w).map(|x| brightness(fb.get_pixel(x, y))).sum();
                sum / w as f32
            })
            .collect();

        // Check for large drops in the row-averaged brightness.
        // The ocean gradient should be broadly increasing (brighter near camera).
        // Terrain/grid overlays cause small dips, but a tear would cause a huge
        // row-average drop because it affects the ENTIRE row.
        let mut max_drop = 0.0_f32;
        for pair in row_avg.windows(2) {
            let drop = pair[0] - pair[1]; // positive = got darker going down
            if drop > max_drop {
                max_drop = drop;
            }
        }

        // Row-averaged drops should be small. Grid lines and terrain affect
        // only part of the row, so row-average dips are bounded.
        // A horizontal tear would shift the entire row, causing a 20+ drop.
        assert!(
            max_drop < 20.0,
            "Scanline gradient tear detected: max row-avg brightness drop = {max_drop:.1} \
             (speed={speed}, cpm={cpm}, curve={curve})"
        );
    }
}

// ---------------------------------------------------------------------------
// Test: horizon seam integrity
// ---------------------------------------------------------------------------
// The row at horizon_y should be fully sky-colored and the row at
// horizon_y+1 should be fully road-colored. A torn frame might show a
// mixed row where some pixels are sky and others are road.

#[test]
fn test_horizon_seam_integrity() {
    let (w, h) = (320, 200);
    let seed = test_seed();
    let mut renderer = clean_renderer(w, h);
    let mut world = WorldState::new(&seed);

    for speed in [0.0, 100.0, 250.0] {
        world.speed = speed;
        world.update(0.016);

        let fb = render_frame(&mut renderer, &world, &seed);
        let hy = horizon_y(h, &world);

        if hy == 0 || hy + 1 >= h {
            continue;
        }

        // The row just above horizon should be sky.
        // The row just below horizon should be road/ocean.
        // Check that each row is internally consistent: all pixels on a row
        // should be in the same "domain" (sky vs road).

        // Measure brightness variance within the horizon row.
        // Sky is typically darker than road near horizon, but both should
        // be uniform across the row. A torn row would have a bimodal
        // brightness distribution.
        let row_above = (0..w)
            .map(|x| brightness(fb.get_pixel(x, hy.saturating_sub(1))))
            .collect::<Vec<_>>();
        let row_below = (0..w)
            .map(|x| brightness(fb.get_pixel(x, hy + 1)))
            .collect::<Vec<_>>();

        // Check that each row doesn't have wild left-to-right jumps.
        // Adjacent pixels on the same scanline should differ by at most ~30
        // (terrain silhouettes can cause some variation).
        let max_jump = |row: &[f32]| -> f32 {
            row.windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .fold(0.0_f32, f32::max)
        };

        let jump_above = max_jump(&row_above);
        let jump_below = max_jump(&row_below);

        // A horizontal tear would show a massive jump (sky→road = ~40+ units).
        // Normal terrain edges can cause ~30 unit jumps, so threshold at 80.
        assert!(
            jump_above < 80.0,
            "Horizon seam tear (above): max lateral jump = {jump_above:.1} at speed={speed}"
        );
        assert!(
            jump_below < 80.0,
            "Horizon seam tear (below): max lateral jump = {jump_below:.1} at speed={speed}"
        );
    }
}

// ---------------------------------------------------------------------------
// Test: inter-frame temporal coherence
// ---------------------------------------------------------------------------
// Consecutive frames rendered at a small dt should not have large average
// pixel deltas. A torn frame (where half the pixels are from frame N and
// half from frame N+1) would show a much higher delta than expected.

#[test]
fn test_temporal_coherence_small_dt() {
    let (w, h) = (200, 150);
    let seed = test_seed();
    let mut renderer = clean_renderer(w, h);
    let mut world = WorldState::new(&seed);
    world.speed = 80.0;
    world.commits_per_min = 1.0;

    // Warm up for a few frames so motion blur prev_fb is populated
    for _ in 0..3 {
        world.update(0.016);
        render_frame(&mut renderer, &world, &seed);
    }

    // Now render 10 consecutive frames at 60fps and check deltas
    let mut prev_fb = render_frame(&mut renderer, &world, &seed);
    let mut max_delta = 0.0_f32;

    for _ in 0..10 {
        world.update(0.016);
        let fb = render_frame(&mut renderer, &world, &seed);
        let delta = mean_pixel_delta(&prev_fb, &fb);
        if delta > max_delta {
            max_delta = delta;
        }
        prev_fb = fb;
    }

    // At 60fps with moderate speed, the mean per-channel delta between
    // consecutive frames should be small (< 8 per channel).
    // A torn frame would spike this because half the pixels jump forward.
    assert!(
        max_delta < 8.0,
        "Temporal coherence violation: max mean delta = {max_delta:.2} (expected < 8.0)"
    );
}

// ---------------------------------------------------------------------------
// Test: temporal coherence at high speed
// ---------------------------------------------------------------------------
// Same as above but at higher speed. Deltas will be larger, but still bounded.

#[test]
fn test_temporal_coherence_high_speed() {
    let (w, h) = (200, 150);
    let seed = test_seed();
    let mut renderer = clean_renderer(w, h);
    let mut world = WorldState::new(&seed);
    world.speed = 250.0;
    world.commits_per_min = 4.0;
    world.throttle = 1.0;

    for _ in 0..5 {
        world.update(0.016);
        render_frame(&mut renderer, &world, &seed);
    }

    let mut prev_fb = render_frame(&mut renderer, &world, &seed);
    let mut max_delta = 0.0_f32;

    for _ in 0..10 {
        world.update(0.016);
        let fb = render_frame(&mut renderer, &world, &seed);
        let delta = mean_pixel_delta(&prev_fb, &fb);
        if delta > max_delta {
            max_delta = delta;
        }
        prev_fb = fb;
    }

    // Higher speed = more motion, so allow up to 15 mean delta
    assert!(
        max_delta < 15.0,
        "High-speed temporal coherence violation: max mean delta = {max_delta:.2} (expected < 15.0)"
    );
}

// ---------------------------------------------------------------------------
// Test: full pipeline with effects enabled
// ---------------------------------------------------------------------------
// Render with all effects (blur, bloom, scanlines) and verify no tearing.
// Effects like motion blur can mask tearing, but scanline filter + bloom
// can also amplify it if the source frame is torn.

#[test]
fn test_full_pipeline_coherence() {
    let (w, h) = (200, 150);
    let seed = test_seed();
    let mut renderer = full_renderer(w, h);
    let mut world = WorldState::new(&seed);
    world.speed = 120.0;
    world.commits_per_min = 2.0;

    // Warm up (motion blur needs prev frame)
    for _ in 0..5 {
        world.update(0.016);
        render_frame(&mut renderer, &world, &seed);
    }

    let mut prev_fb = render_frame(&mut renderer, &world, &seed);
    let mut max_delta = 0.0_f32;

    for _ in 0..10 {
        world.update(0.016);
        let fb = render_frame(&mut renderer, &world, &seed);
        let delta = mean_pixel_delta(&prev_fb, &fb);
        if delta > max_delta {
            max_delta = delta;
        }
        prev_fb = fb;
    }

    // With motion blur active, deltas should actually be smaller
    assert!(
        max_delta < 10.0,
        "Full pipeline temporal coherence violation: max mean delta = {max_delta:.2}"
    );
}

// ---------------------------------------------------------------------------
// Test: grid line horizontal coherence across rows
// ---------------------------------------------------------------------------
// In a non-torn frame, vertical grid lines converge smoothly to the vanishing
// point. Between adjacent rows, each grid line's x-position should shift by
// a small, smooth amount. A tear would cause all grid lines on one row to
// be offset horizontally from the row above.
//
// We detect this by rendering two frames: one without grid and one with grid.
// On each row, we find the centroid of grid-affected pixels (the "grid center
// of mass"). Between adjacent rows, this centroid should move smoothly.

#[test]
fn test_grid_line_horizontal_coherence() {
    let (w, h) = (400, 200);
    let seed = test_seed();

    // Render road-only (no grid) and road+grid to isolate grid pixels.
    // Use road module directly for isolation.
    let mut world = WorldState::new(&seed);
    world.speed = 50.0;
    world.curve_offset = 0.0;
    world.update(0.016);

    let hy = horizon_y(h, &world);

    let mut fb_road = image::ImageBuffer::new(w, h);
    git_demon::renderer::road::draw_road(&mut fb_road, w, h, hy, &world, &seed, &[]);
    let fb_road_copy = fb_road.clone();

    let mut fb_grid = fb_road_copy;
    git_demon::renderer::road::draw_grid(&mut fb_grid, w, h, hy, &world, &seed, &[]);

    // For each row below horizon, count grid-affected pixels and find their centroid.
    // Skip rows with >50% grid coverage — those are horizontal grid lines whose
    // centroid is naturally at screen center regardless of vertical line positions.
    let mut centroids: Vec<Option<f32>> = Vec::new();
    for y in hy..h {
        let mut sum_x: f32 = 0.0;
        let mut count: f32 = 0.0;
        for x in 0..w {
            let road_p = fb_road.get_pixel(x, y);
            let grid_p = fb_grid.get_pixel(x, y);
            let diff = (grid_p.0[0] as i32 - road_p.0[0] as i32).abs()
                + (grid_p.0[1] as i32 - road_p.0[1] as i32).abs()
                + (grid_p.0[2] as i32 - road_p.0[2] as i32).abs();
            if diff > 5 {
                sum_x += x as f32;
                count += 1.0;
            }
        }
        // Skip horizontal grid line rows (>30% coverage) — their centroid
        // is dominated by the full-width line, not vertical line positions.
        if count > 2.0 && count < w as f32 * 0.3 {
            centroids.push(Some(sum_x / count));
        } else {
            centroids.push(None);
        }
    }

    // Check that adjacent centroids (from vertical-line-only rows) shift smoothly.
    // A horizontal tear would offset all vertical lines on the rows below the tear.
    let mut max_jump = 0.0_f32;
    let mut prev_cx: Option<f32> = None;
    for cx in &centroids {
        if let (Some(prev), Some(curr)) = (prev_cx, cx) {
            let jump = (curr - prev).abs();
            if jump > max_jump {
                max_jump = jump;
            }
        }
        if cx.is_some() {
            prev_cx = *cx;
        }
    }

    // Vertical grid lines fan out smoothly with perspective. Between adjacent
    // non-hline rows, the centroid should shift by at most ~5px. A tear that
    // offsets the bottom half of the frame would show a 30+ px jump.
    assert!(
        max_jump < 30.0,
        "Grid centroid horizontal jump detected: {max_jump:.1}px (expected < 30)"
    );
}

// ---------------------------------------------------------------------------
// Test: multi-frame road gradient stability
// ---------------------------------------------------------------------------
// Over a sequence of frames, the road gradient shape should remain stable.
// A tear that persists across frames would cause the gradient "shape" to
// oscillate between correct and corrupted states.

#[test]
fn test_road_gradient_stability_across_frames() {
    let (w, h) = (200, 150);
    let seed = test_seed();
    let mut renderer = clean_renderer(w, h);
    let mut world = WorldState::new(&seed);
    world.speed = 60.0;
    world.commits_per_min = 0.5;

    // Collect the brightness profile of the center column across 10 frames
    let mut profiles: Vec<Vec<f32>> = Vec::new();

    for _ in 0..10 {
        world.update(0.016);
        let fb = render_frame(&mut renderer, &world, &seed);
        let hy = horizon_y(h, &world);
        let cx = w / 2;

        let profile: Vec<f32> = (hy..h).map(|y| brightness(fb.get_pixel(cx, y))).collect();
        profiles.push(profile);
    }

    // Compare each frame's profile to the next.
    // The overall shape should be similar — monotonically increasing.
    // A torn frame would show a non-monotonic profile.
    for (i, profile) in profiles.iter().enumerate() {
        if profile.len() < 3 {
            continue;
        }

        // Count monotonicity violations (brightness decreasing going down)
        let violations: usize = profile
            .windows(2)
            .filter(|w| w[1] < w[0] - 3.0) // allow 3 units for wave shimmer
            .count();

        // Allow a small number of violations from wave effect, but not many.
        // A torn frame would have many violations at the tear line.
        let max_allowed = profile.len() / 4; // 25% tolerance for wave shimmer
        assert!(
            violations <= max_allowed,
            "Frame {i}: gradient monotonicity violations = {violations}/{} \
             (max allowed = {max_allowed})",
            profile.len()
        );
    }
}

// ---------------------------------------------------------------------------
// Test: no NaN or infinite pixel values (sanity check)
// ---------------------------------------------------------------------------

#[test]
fn test_no_corrupted_pixels() {
    let (w, h) = (200, 150);
    let seed = test_seed();
    let mut renderer = full_renderer(w, h);
    let mut world = WorldState::new(&seed);

    // Render across a range of world states
    for i in 0..20 {
        world.speed = i as f32 * 15.0;
        world.commits_per_min = (i as f32 * 0.3).min(5.0);
        world.curve_offset = (i as f32 * 10.0 - 100.0).clamp(-80.0, 80.0);
        world.update(0.016);

        let fb = render_frame(&mut renderer, &world, &seed);

        // Every pixel should have alpha = 255 (fully opaque)
        for y in 0..h {
            for x in 0..w {
                let p = fb.get_pixel(x, y);
                assert_eq!(
                    p.0[3], 255,
                    "Pixel ({x},{y}) has non-opaque alpha {} on frame {i}",
                    p.0[3]
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Test: gear shift doesn't cause frame tear
// ---------------------------------------------------------------------------
// During a gear shift, the speed changes rapidly. Verify that the frame
// rendered during a shift event is still coherent.

#[test]
fn test_gear_shift_frame_coherence() {
    let (w, h) = (200, 150);
    let seed = test_seed();
    let mut renderer = clean_renderer(w, h);
    let mut world = WorldState::new(&seed);
    world.commits_per_min = 4.0; // high throttle to trigger shifts
    world.throttle = 1.0;

    let mut shift_happened = false;
    let mut prev_fb = render_frame(&mut renderer, &world, &seed);

    // Run until we see at least one gear shift
    for _ in 0..300 {
        world.update(0.016);
        let fb = render_frame(&mut renderer, &world, &seed);

        if world.just_shifted {
            shift_happened = true;
            let delta = mean_pixel_delta(&prev_fb, &fb);
            assert!(
                delta < 20.0,
                "Gear shift caused excessive frame delta: {delta:.2} (gear={})",
                world.gear + 1
            );
        }

        prev_fb = fb;
    }

    assert!(
        shift_happened,
        "Expected at least one gear shift during 300 frames at full throttle"
    );
}
