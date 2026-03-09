pub mod effects;
pub mod font;
pub mod hud;
pub mod menu;
pub mod road;
pub mod sky;
pub mod sprites;
pub mod terrain;

use image::{ImageBuffer, Rgba};

use crate::git::seed::RepoSeed;
use crate::world::WorldState;

#[derive(Default, Clone, Copy)]
pub struct PassTimings {
    pub sky_us: u128,
    pub terrain_us: u128,
    pub road_us: u128,
    pub sprites_us: u128,
    pub effects_us: u128,
    pub blur_us: u128,
    pub bloom_us: u128,
    pub scanline_us: u128,
    pub hud_us: u128,
}

pub struct FrameRenderer {
    pub pixel_w: u32,
    pub pixel_h: u32,
    fb: ImageBuffer<Rgba<u8>, Vec<u8>>,
    prev_fb: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub no_blur: bool,
    pub no_bloom: bool,
    pub no_scanlines: bool,
    pub no_hud: bool,
    pub dev: bool,
    pub dev_page: u8,
    frame_count: u64,
    last_render_us: u128,
    world_fps: f32,
    render_fps: f32,
    target_render_fps: u32,
    encode_send_us: u128,
    pass_timings: PassTimings,
}

impl FrameRenderer {
    pub fn new(
        pixel_w: u32,
        pixel_h: u32,
        no_blur: bool,
        no_bloom: bool,
        no_scanlines: bool,
        no_hud: bool,
        dev: bool,
    ) -> Self {
        Self {
            pixel_w,
            pixel_h,
            fb: ImageBuffer::new(pixel_w, pixel_h),
            prev_fb: ImageBuffer::new(pixel_w, pixel_h),
            no_blur,
            no_bloom,
            no_scanlines,
            no_hud,
            dev,
            dev_page: 0,
            frame_count: 0,
            last_render_us: 0,
            world_fps: 0.0,
            render_fps: 0.0,
            target_render_fps: 0,
            encode_send_us: 0,
            pass_timings: PassTimings::default(),
        }
    }

    pub fn set_timing(
        &mut self,
        world_fps: f32,
        render_fps: f32,
        target_render_fps: u32,
        encode_send_us: u128,
    ) {
        self.world_fps = world_fps;
        self.render_fps = render_fps;
        self.target_render_fps = target_render_fps;
        self.encode_send_us = encode_send_us;
    }

    pub fn resize(&mut self, pixel_w: u32, pixel_h: u32) {
        self.pixel_w = pixel_w;
        self.pixel_h = pixel_h;
        self.fb = ImageBuffer::new(pixel_w, pixel_h);
        self.prev_fb = ImageBuffer::new(pixel_w, pixel_h);
    }

    pub fn render(
        &mut self,
        world: &WorldState,
        seed: &RepoSeed,
    ) -> &ImageBuffer<Rgba<u8>, Vec<u8>> {
        use std::time::Instant;
        let render_start = Instant::now();
        self.frame_count += 1;
        let (w, h) = (self.pixel_w, self.pixel_h);
        let horizon_y = (h as f32 * road::horizon_ratio(world)) as u32;

        // 1-2. Sky + sun + bloom bleed
        let t = Instant::now();
        sky::draw_sky(&mut self.fb, w, h, horizon_y, seed, world);
        sky::draw_sun(&mut self.fb, w, horizon_y, seed, world);
        sky::draw_bloom_bleed(&mut self.fb, w, horizon_y, seed, world);
        self.pass_timings.sky_us = t.elapsed().as_micros();

        // 3-4. Terrain
        let t = Instant::now();
        terrain::draw_terrain(&mut self.fb, w, h, horizon_y, seed, world);
        self.pass_timings.terrain_us = t.elapsed().as_micros();

        // 5-6. Road scanlines + grid
        let t = Instant::now();
        road::draw_road(&mut self.fb, w, h, horizon_y, world, seed);
        road::draw_grid(&mut self.fb, w, h, horizon_y, world, seed);
        self.pass_timings.road_us = t.elapsed().as_micros();

        // 7. Sprites + player car + speed lines
        let t = Instant::now();
        sprites::draw_sprites(&mut self.fb, w, h, horizon_y, world, seed);
        effects::draw_car(&mut self.fb, w, h, world);
        if world.tier_index() >= 3 {
            effects::draw_speed_lines(&mut self.fb, w, horizon_y, world, seed);
        }
        self.pass_timings.sprites_us = t.elapsed().as_micros();

        // 8. Motion blur
        let t = Instant::now();
        if !self.no_blur {
            effects::apply_motion_blur(&mut self.fb, &self.prev_fb, world);
        }
        self.pass_timings.blur_us = t.elapsed().as_micros();

        // 9. Scanline filter
        let t = Instant::now();
        if !self.no_scanlines {
            effects::apply_scanline_filter(&mut self.fb);
        }
        self.pass_timings.scanline_us = t.elapsed().as_micros();

        // 10. Bloom
        let t = Instant::now();
        if !self.no_bloom {
            effects::apply_bloom(&mut self.fb);
        }
        self.pass_timings.bloom_us = t.elapsed().as_micros();

        // 11. HUD
        let t = Instant::now();
        if !self.no_hud {
            hud::draw_hud(&mut self.fb, w, h, world, seed);
        }
        self.pass_timings.hud_us = t.elapsed().as_micros();

        // Dev stats overlay (after all effects, so it's always readable)
        self.last_render_us = render_start.elapsed().as_micros();
        if self.dev {
            hud::draw_dev_overlay(
                &mut self.fb,
                w,
                world,
                seed,
                self.frame_count,
                self.last_render_us,
                self.world_fps,
                self.render_fps,
                self.target_render_fps,
                self.encode_send_us,
                &self.pass_timings,
                self.dev_page,
            );
        }

        // Swap buffers for next frame's motion blur
        std::mem::swap(&mut self.prev_fb, &mut self.fb);
        // prev_fb now has the rendered frame, fb is stale — swap reference
        &self.prev_fb
    }
}
