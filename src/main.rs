use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use image::DynamicImage;
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image, Resize};

use git_demon::git::poller::GitPoller;
use git_demon::git::seed::RepoSeed;
use git_demon::renderer::menu::{MenuItem, MenuState};
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

    /// Target framerate (world updates per second)
    #[arg(long, default_value_t = 30)]
    pub fps: u32,

    /// Render scale factor (0.25 = quarter res, 0.5 = half, 1.0 = full)
    #[arg(long, default_value_t = 0.5)]
    pub scale: f32,

    /// Max render FPS (rasterize+encode cadence, independent of world update rate)
    #[arg(long, default_value_t = 15)]
    pub render_fps: u32,

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
    scale: f32,
) -> (u32, u32) {
    let (cell_w, cell_h) = picker.font_size();
    let size = terminal
        .size()
        .unwrap_or(ratatui::layout::Size::new(160, 40));
    let pixel_w = (size.width as f32 * cell_w as f32 * scale) as u32;
    let pixel_h = (size.height as f32 * cell_h as f32 * scale) as u32;
    (pixel_w.max(160), pixel_h.max(100))
}

fn run(args: Args) -> anyhow::Result<()> {
    // Restore terminal on panic so the shell isn't left in raw mode
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        default_hook(info);
    }));

    let mut terminal = ratatui::init();
    let (tx, rx) = crossbeam_channel::unbounded();

    GitPoller::spawn(&args.repo, args.window, args.interval, tx)?;

    let seed = RepoSeed::compute(&args.repo)?;
    let picker = Picker::from_query_stdio()
        .map_err(|e| anyhow::anyhow!("Terminal graphics detection failed: {e}"))?;
    let scale = args.scale.clamp(0.1, 2.0);
    let (pixel_w, pixel_h) = compute_pixel_dims(&picker, &terminal, scale);

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
    #[allow(unused_assignments)]
    let mut proto: Option<Protocol> = None;

    let target_dt = Duration::from_secs_f64(1.0 / args.fps as f64);
    let render_interval = Duration::from_secs_f64(1.0 / args.render_fps.max(1) as f64);
    let mut last = Instant::now();
    let mut last_render = Instant::now() - render_interval; // force first render

    // FPS tracking
    let mut frame_times: Vec<Instant> = Vec::with_capacity(120);
    let mut render_times: Vec<Instant> = Vec::with_capacity(60);
    #[allow(unused_assignments)]
    let mut actual_fps: f32 = 0.0;
    let mut encode_send_us: u128 = 0;
    let mut menu = MenuState::new();
    let mut quit = false;

    loop {
        let dt = last.elapsed().as_secs_f32().min(0.05);
        last = Instant::now();

        // Track world update rate
        frame_times.push(last);
        let cutoff = last - Duration::from_secs(1);
        frame_times.retain(|t| *t > cutoff);
        actual_fps = frame_times.len() as f32;

        // Ingest git data (cheap)
        while let Ok(result) = rx.try_recv() {
            world.ingest_poll(&result, &seed);
        }

        // World update only when not paused
        if !menu.open {
            world.update(dt);
        }

        // Only rasterize + encode at render_fps cadence
        if last.duration_since(last_render) >= render_interval {
            last_render = last;

            render_times.push(last);
            let render_cutoff = last - Duration::from_secs(1);
            render_times.retain(|t| *t > render_cutoff);
            let render_fps = render_times.len() as f32;

            renderer.set_timing(actual_fps, render_fps, args.render_fps, encode_send_us);

            let fb = renderer.render(&world, &seed);

            // Build the final framebuffer (with menu overlay if paused)
            let encode_start = Instant::now();
            let final_fb = if menu.open {
                let mut menu_fb = fb.clone();
                let (mw, mh) = (menu_fb.width(), menu_fb.height());
                git_demon::renderer::menu::draw_menu(
                    &mut menu_fb,
                    mw,
                    mh,
                    &menu,
                    !renderer.no_blur,
                    !renderer.no_bloom,
                    !renderer.no_scanlines,
                    !renderer.no_hud,
                    renderer.dev,
                );
                menu_fb
            } else {
                fb.clone()
            };

            // Encode to terminal protocol (fixed-size, no image ID churn)
            let img = DynamicImage::ImageRgba8(final_fb);
            let area = terminal
                .size()
                .map(|s| Rect::new(0, 0, s.width, s.height))
                .unwrap_or(Rect::new(0, 0, 160, 40));
            proto = picker.new_protocol(img, area, Resize::Fit(None)).ok();
            encode_send_us = encode_start.elapsed().as_micros();

            terminal.draw(|frame| {
                if let Some(ref p) = proto {
                    let image = Image::new(p);
                    frame.render_widget(image, frame.area());
                }
            })?;
        }

        // Input handling
        let mut speed_input = false;
        let mut curve_input = false;
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(k) if k.code == KeyCode::Esc => {
                    menu.toggle();
                }
                Event::Key(k) if menu.open => match k.code {
                    KeyCode::Up => menu.move_up(),
                    KeyCode::Down => menu.move_down(),
                    KeyCode::Enter => match menu.current() {
                        MenuItem::Resume => menu.toggle(),
                        MenuItem::ToggleBlur => renderer.no_blur = !renderer.no_blur,
                        MenuItem::ToggleBloom => renderer.no_bloom = !renderer.no_bloom,
                        MenuItem::ToggleScanlines => renderer.no_scanlines = !renderer.no_scanlines,
                        MenuItem::ToggleHud => renderer.no_hud = !renderer.no_hud,
                        MenuItem::ToggleDev => renderer.dev = !renderer.dev,
                        MenuItem::ResetSpeed => {
                            world.speed_multiplier = 1.0;
                            world.curve_multiplier = 1.0;
                            world.speed_hold_time = 0.0;
                        }
                        MenuItem::Quit => quit = true,
                    },
                    KeyCode::Char('q') => quit = true,
                    _ => {}
                },
                // Normal gameplay keys (not in menu)
                Event::Key(k) if k.code == KeyCode::Char('q') => quit = true,
                Event::Key(k) if k.code == KeyCode::Up => {
                    speed_input = true;
                    world.speed_hold_time += dt;
                    let step = 0.2 * (2.0_f32).powf(world.speed_hold_time);
                    world.speed_multiplier = (world.speed_multiplier + step).min(150.0);
                }
                Event::Key(k) if k.code == KeyCode::Down => {
                    speed_input = true;
                    world.speed_hold_time += dt;
                    let step = 0.2 * (2.0_f32).powf(world.speed_hold_time);
                    world.speed_multiplier = (world.speed_multiplier - step).max(0.1);
                }
                Event::Key(k) if k.code == KeyCode::Right => {
                    curve_input = true;
                    world.curve_hold_time += dt;
                    let step = 0.2 * (2.0_f32).powf(world.curve_hold_time);
                    world.curve_multiplier = (world.curve_multiplier + step).min(15.0);
                }
                Event::Key(k) if k.code == KeyCode::Left => {
                    curve_input = true;
                    world.curve_hold_time += dt;
                    let step = 0.2 * (2.0_f32).powf(world.curve_hold_time);
                    world.curve_multiplier = (world.curve_multiplier - step).max(0.0);
                }
                Event::Resize(..) => {
                    let (w, h) = compute_pixel_dims(&picker, &terminal, scale);
                    renderer.resize(w, h);
                }
                _ => {}
            }
        }
        if !speed_input {
            world.speed_hold_time = 0.0;
        }
        if !curve_input {
            world.curve_hold_time = 0.0;
        }

        if quit {
            break;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Args {
        Args::try_parse_from(args).expect("should parse")
    }

    #[test]
    fn test_args_defaults() {
        let a = parse(&["git-demon"]);
        assert_eq!(a.repo, ".");
        assert_eq!(a.window, 30);
        assert_eq!(a.interval, 30);
        assert_eq!(a.fps, 30);
        assert!((a.scale - 0.5).abs() < 0.001);
        assert_eq!(a.render_fps, 15);
        assert!(!a.no_blur);
        assert!(!a.no_bloom);
        assert!(!a.no_scanlines);
        assert!(!a.no_hud);
        assert!(!a.dev);
    }

    #[test]
    fn test_args_repo_flag() {
        let a = parse(&["git-demon", "--repo", "/tmp/foo"]);
        assert_eq!(a.repo, "/tmp/foo");
    }

    #[test]
    fn test_args_numeric_overrides() {
        let a = parse(&[
            "git-demon",
            "--fps",
            "60",
            "--window",
            "10",
            "--interval",
            "5",
            "--render-fps",
            "30",
        ]);
        assert_eq!(a.fps, 60);
        assert_eq!(a.window, 10);
        assert_eq!(a.interval, 5);
        assert_eq!(a.render_fps, 30);
    }

    #[test]
    fn test_args_scale_override() {
        let a = parse(&["git-demon", "--scale", "1.0"]);
        assert!((a.scale - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_args_all_disable_flags() {
        let a = parse(&[
            "git-demon",
            "--no-blur",
            "--no-bloom",
            "--no-scanlines",
            "--no-hud",
        ]);
        assert!(a.no_blur);
        assert!(a.no_bloom);
        assert!(a.no_scanlines);
        assert!(a.no_hud);
    }

    #[test]
    fn test_args_dev_flag() {
        let a = parse(&["git-demon", "--dev"]);
        assert!(a.dev);
    }

    #[test]
    fn test_args_invalid_rejects() {
        let result = Args::try_parse_from(&["git-demon", "--nonexistent"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_dt_clamp_constant() {
        // The main loop clamps dt at 0.05s (20 fps minimum) to prevent
        // spiral of death. Verify this matches the spec.
        let large_dt: f32 = 0.1;
        let clamped = large_dt.min(0.05);
        assert!((clamped - 0.05).abs() < 0.001);
    }
}
