use image::{ImageBuffer, Rgba};

use super::font;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Resume,
    ToggleBlur,
    ToggleBloom,
    ToggleScanlines,
    ToggleHud,
    ToggleDev,
    ResetSpeed,
    Quit,
}

impl MenuItem {
    pub const ALL: &[MenuItem] = &[
        MenuItem::Resume,
        MenuItem::ToggleBlur,
        MenuItem::ToggleBloom,
        MenuItem::ToggleScanlines,
        MenuItem::ToggleHud,
        MenuItem::ToggleDev,
        MenuItem::ResetSpeed,
        MenuItem::Quit,
    ];
}

pub struct MenuState {
    pub open: bool,
    pub selected: usize,
}

impl Default for MenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            open: false,
            selected: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.selected = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = MenuItem::ALL.len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        self.selected = (self.selected + 1) % MenuItem::ALL.len();
    }

    pub fn current(&self) -> MenuItem {
        MenuItem::ALL[self.selected]
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_menu(
    fb: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    menu: &MenuState,
    blur_on: bool,
    bloom_on: bool,
    scanlines_on: bool,
    hud_on: bool,
    dev_on: bool,
) {
    // Dim the background
    let raw = fb.as_mut();
    for pixel in raw.chunks_exact_mut(4) {
        pixel[0] /= 3;
        pixel[1] /= 3;
        pixel[2] /= 3;
    }

    let scale = 2u32;
    let line_h = 20u32;
    let items = [
        "RESUME".to_string(),
        format!("BLUR: {}", if blur_on { "ON" } else { "OFF" }),
        format!("BLOOM: {}", if bloom_on { "ON" } else { "OFF" }),
        format!("SCANLINES: {}", if scanlines_on { "ON" } else { "OFF" }),
        format!("HUD: {}", if hud_on { "ON" } else { "OFF" }),
        format!("DEV OVERLAY: {}", if dev_on { "ON" } else { "OFF" }),
        "RESET SPEED x1.0".to_string(),
        "QUIT".to_string(),
    ];

    let title = "-- PAUSED --";
    let title_scale = 2u32;
    let title_w = font::text_width(title, title_scale);
    let title_x = w.saturating_sub(title_w) / 2;

    let panel_h = (items.len() as u32 + 2) * line_h + 20;
    let panel_top = h.saturating_sub(panel_h) / 2;

    // Title
    let cyan = Rgba([0, 255, 255, 255]);
    font::draw_text(fb, title, title_x, panel_top, cyan, title_scale);

    let white = Rgba([255, 255, 255, 255]);
    let highlight = Rgba([255, 200, 0, 255]);
    let dim = Rgba([120, 120, 120, 255]);

    for (i, label) in items.iter().enumerate() {
        let y = panel_top + (i as u32 + 2) * line_h;
        let (color, prefix) = if i == menu.selected {
            (highlight, "> ")
        } else {
            (dim, "  ")
        };

        let full = format!("{prefix}{label}");
        let full_w = font::text_width(&full, scale);
        let full_x = w.saturating_sub(full_w) / 2;
        font::draw_text(fb, &full, full_x, y, color, scale);
    }

    // Controls hint at bottom
    let hint = "UP/DOWN: select  ENTER: confirm  ESC: resume";
    let hint_w = font::text_width(hint, 1);
    let hint_x = w.saturating_sub(hint_w) / 2;
    let hint_y = panel_top + (items.len() as u32 + 3) * line_h;
    font::draw_text(fb, hint, hint_x, hint_y, white, 1);
}
