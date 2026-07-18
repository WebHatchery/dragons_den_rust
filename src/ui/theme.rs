//! Warm "hoard" palette and ornate chrome (UI redesign PT — see
//! `UI_REDESIGN_PLAN.md`). This is the game's visual identity: gold and bronze
//! on dark brown, deliberately diverging from the toolkit's cool `dark` palette.
//!
//! The constant names mirror `macroquad_toolkit::colors::dark` so screen code
//! can read `theme::TEXT` in place of `dark::TEXT`.

use macroquad::prelude::*;

// --- palette -------------------------------------------------------------

/// Near-black warm brown behind everything.
pub const BACKGROUND: Color = Color::new(0.047, 0.035, 0.024, 1.0);
/// Panel / card fill (dark brown).
pub const PANEL: Color = Color::new(0.090, 0.067, 0.039, 1.0);
/// Panel title band.
pub const PANEL_HEADER: Color = Color::new(0.120, 0.094, 0.063, 1.0);
/// Insets, art wells, disabled fills.
pub const PANEL_DARK: Color = Color::new(0.059, 0.043, 0.027, 1.0);

/// Bright etched frame border (gold).
pub const BORDER: Color = Color::new(0.847, 0.663, 0.290, 1.0);
/// Subtle dividers / card borders (bronze).
pub const BORDER_DIM: Color = Color::new(0.420, 0.325, 0.165, 1.0);
/// Signature gold accent.
pub const ACCENT: Color = Color::new(0.910, 0.710, 0.320, 1.0);
/// Alias for readability where "gold" reads clearer than "accent".
pub const GOLD: Color = ACCENT;

/// Body text (warm cream).
pub const TEXT: Color = Color::new(0.847, 0.800, 0.690, 1.0);
/// Headings (gold-cream).
pub const TEXT_BRIGHT: Color = Color::new(0.950, 0.894, 0.753, 1.0);
/// Muted tan.
pub const TEXT_DIM: Color = Color::new(0.541, 0.490, 0.392, 1.0);

/// Income / "Common" green.
pub const POSITIVE: Color = Color::new(0.560, 0.720, 0.350, 1.0);
/// Purple — Hoard Points and the prestige "burn".
pub const HOARD_POINT: Color = Color::new(0.604, 0.435, 0.769, 1.0);

// --- ornate chrome -------------------------------------------------------

/// Draws short gold L-brackets inside each corner of `rect` — the cheap read of
/// an ornate etched frame without per-corner art. Used by `ui::panel` and the
/// header/region chrome.
pub fn draw_corner_marks(rect: Rect, color: Color) {
    const INSET: f32 = 4.0;
    const LEN: f32 = 12.0;
    const TH: f32 = 2.0;
    let (l, t, r, b) = (
        rect.x + INSET,
        rect.y + INSET,
        rect.right() - INSET,
        rect.bottom() - INSET,
    );
    // Top-left, top-right, bottom-left, bottom-right brackets.
    draw_line(l, t, l + LEN, t, TH, color);
    draw_line(l, t, l, t + LEN, TH, color);
    draw_line(r, t, r - LEN, t, TH, color);
    draw_line(r, t, r, t + LEN, TH, color);
    draw_line(l, b, l + LEN, b, TH, color);
    draw_line(l, b, l, b - LEN, TH, color);
    draw_line(r, b, r - LEN, b, TH, color);
    draw_line(r, b, r, b - LEN, TH, color);
}
