use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use image::DynamicImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::StatefulImage;

use git_demon::git::poller::GitPoller;
use git_demon::git::seed::RepoSeed;
use git_demon::renderer::FrameRenderer;
use git_demon::world::WorldState;

#[derive(Parser, Debug)]
#[command(
    name = "git-demon",
    version,
    about = "Sci-fi cyber racecar screensaver driven by live git activity"
)]
pub struct Args {
    /// Git repository to watch
    #[arg(long, default_value = ".")]
    pub repo: String,

    /// Commit lookback window in minutes
    #[arg(long, default_value_t = 30)]
    pub window: u32,

    /// Git poll interval in seconds
    #[arg(long, default_value_t = 30)]
    pub interval: u32,

    /// Target framerate
    #[arg(long, default_value_t = 60)]
    pub fps: u32,

    /// Disable motion blur
    #[arg(long)]
    pub no_blur: bool,

    /// Disable bloom pass
    #[arg(long)]
    pub no_bloom: bool,

    /// Disable CRT scanline filter
    #[arg(long)]
    pub no_scanlines: bool,

    /// Hide bottom HUD strip
    #[arg(long)]
    pub no_hud: bool,

    /// Show dev stats overlay
    #[arg(long)]
    pub dev: bool,
}

fn compute_pixel_dims(
    picker: &Picker,
    terminal: &ratatui::Terminal<impl ratatui::backend::Backend>,
) -> (u32, u32) {
    let (cell_w, cell_h) = picker.font_size();
    let size = terminal
        .size()
        .unwrap_or(ratatui::layout::Size::new(160, 40));
    let pixel_w = size.width as u32 * cell_w as u32;
    let pixel_h = size.height as u32 * cell_h as u32;
    (pixel_w.max(320), pixel_h.max(200))
}

fn run(args: Args) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let (tx, rx) = crossbeam_channel::unbounded();

    GitPoller::spawn(&args.repo, args.window, args.interval, tx)?;

    let seed = RepoSeed::compute(&args.repo)?;
    let picker = Picker::from_query_stdio()
        .map_err(|e| anyhow::anyhow!("Terminal graphics detection failed: {e}"))?;
    let (pixel_w, pixel_h) = compute_pixel_dims(&picker, &terminal);

    let mut world = WorldState::new(&seed);
    let mut renderer = FrameRenderer::new(
        pixel_w,
        pixel_h,
        args.no_blur,
        args.no_bloom,
        args.no_scanlines,
        args.no_hud,
        args.dev,
    );
    let img = DynamicImage::ImageRgba8(image::ImageBuffer::new(pixel_w, pixel_h));
    #[allow(unused_assignments)]
    let mut proto: StatefulProtocol = picker.new_resize_protocol(img);

    let target_dt = Duration::from_secs_f64(1.0 / args.fps as f64);
    let mut last = Instant::now();

    // FPS tracking
    let mut frame_times: Vec<Instant> = Vec::with_capacity(120);
    #[allow(unused_assignments)]
    let mut actual_fps: f32 = 0.0;
    let mut encode_send_us: u128 = 0;

    loop {
        let dt = last.elapsed().as_secs_f32().min(0.05);
        last = Instant::now();

        // Track actual FPS over rolling 1-second window
        frame_times.push(last);
        let cutoff = last - Duration::from_secs(1);
        frame_times.retain(|t| *t > cutoff);
        actual_fps = frame_times.len() as f32;

        while let Ok(result) = rx.try_recv() {
            world.ingest_poll(&result, &seed);
        }

        world.update(dt);

        // Pass timing info to renderer for dev overlay
        renderer.set_timing(actual_fps, args.fps, encode_send_us);

        let fb = renderer.render(&world, &seed);

        // Measure encode + send
        let encode_start = Instant::now();
        let img = DynamicImage::ImageRgba8(fb.clone());
        proto = picker.new_resize_protocol(img);

        terminal.draw(|frame| {
            let image = StatefulImage::default();
            frame.render_stateful_widget(image, frame.area(), &mut proto);
        })?;
        encode_send_us = encode_start.elapsed().as_micros();

        if event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(k) if matches!(k.code, KeyCode::Char('q') | KeyCode::Esc) => break,
                Event::Resize(..) => {
                    let (w, h) = compute_pixel_dims(&picker, &terminal);
                    renderer.resize(w, h);
                }
                _ => {}
            }
        }

        if let Some(remaining) = target_dt.checked_sub(last.elapsed()) {
            std::thread::sleep(remaining);
        }
    }

    ratatui::restore();
    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        ratatui::restore();
        eprintln!("git-demon: {e}");
        std::process::exit(1);
    }
}
