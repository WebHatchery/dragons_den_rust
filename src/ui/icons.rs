//! Procedural vector icons (UI redesign P6), drawn from primitives so the game
//! ships no art assets — matching the GDD §0 "lowest-art genre" stance. Each
//! icon draws centered at `(cx, cy)` within radius `r`.

use crate::ui::theme;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub enum Icon {
    Coin,
    Minion,
    Gem,
}

pub fn draw(icon: Icon, cx: f32, cy: f32, r: f32) {
    match icon {
        Icon::Coin => coin(cx, cy, r),
        Icon::Minion => minion(cx, cy, r),
        Icon::Gem => gem(cx, cy, r),
    }
}

/// A gold coin with a rim and a struck mark.
pub fn coin(cx: f32, cy: f32, r: f32) {
    draw_circle(cx, cy, r, Color::new(0.85, 0.63, 0.24, 1.0));
    draw_circle_lines(cx, cy, r, 2.0, Color::new(0.52, 0.36, 0.12, 1.0));
    draw_circle_lines(cx, cy, r * 0.62, 1.5, Color::new(0.98, 0.84, 0.42, 0.9));
    draw_line(
        cx,
        cy - r * 0.4,
        cx,
        cy + r * 0.4,
        2.0,
        Color::new(0.42, 0.28, 0.08, 1.0),
    );
}

/// A round green minion face with pointed ears and eyes.
pub fn minion(cx: f32, cy: f32, r: f32) {
    let skin = Color::new(0.38, 0.56, 0.30, 1.0);
    draw_triangle(
        vec2(cx - r * 0.85, cy - r * 0.25),
        vec2(cx - r * 0.35, cy - r * 0.65),
        vec2(cx - r * 0.15, cy + r * 0.05),
        skin,
    );
    draw_triangle(
        vec2(cx + r * 0.85, cy - r * 0.25),
        vec2(cx + r * 0.35, cy - r * 0.65),
        vec2(cx + r * 0.15, cy + r * 0.05),
        skin,
    );
    draw_circle(cx, cy, r * 0.72, skin);
    let eye = Color::new(0.95, 0.86, 0.25, 1.0);
    draw_circle(cx - r * 0.28, cy - r * 0.04, r * 0.14, eye);
    draw_circle(cx + r * 0.28, cy - r * 0.04, r * 0.14, eye);
    draw_circle(cx - r * 0.28, cy - r * 0.04, r * 0.06, BLACK);
    draw_circle(cx + r * 0.28, cy - r * 0.04, r * 0.06, BLACK);
}

/// A purple prestige gem (diamond with a facet line).
pub fn gem(cx: f32, cy: f32, r: f32) {
    draw_poly(cx, cy, 4, r, 0.0, theme::HOARD_POINT);
    draw_poly_lines(cx, cy, 4, r, 0.0, 1.5, Color::new(0.82, 0.72, 0.95, 0.9));
    draw_line(
        cx - r * 0.7,
        cy,
        cx + r * 0.7,
        cy,
        1.0,
        Color::new(0.82, 0.72, 0.95, 0.55),
    );
}

/// A small mound of coins topped with a crown — the left rail's hoard art.
pub fn hoard_pile(rect: Rect) {
    let cx = rect.x + rect.w / 2.0;
    let scale = rect.w.min(rect.h);
    let base_y = rect.y + rect.h * 0.66;
    let gold = Color::new(0.85, 0.65, 0.28, 1.0);
    let rim = Color::new(0.55, 0.38, 0.14, 1.0);
    // Overlapping coins forming a mound (bottom row wider, tapering up).
    let coins = [
        (-0.26, 0.10, 0.14),
        (0.0, 0.12, 0.16),
        (0.26, 0.10, 0.14),
        (-0.13, -0.04, 0.13),
        (0.13, -0.04, 0.13),
        (0.0, -0.18, 0.12),
    ];
    for (dx, dy, cr) in coins {
        let x = cx + dx * scale;
        let y = base_y + dy * scale;
        draw_circle(x, y, cr * scale, gold);
        draw_circle_lines(x, y, cr * scale, 1.5, rim);
    }
    // A little crown on the peak.
    let peak_y = base_y - 0.30 * scale;
    let cw = 0.30 * scale;
    let gold_bright = Color::new(0.95, 0.78, 0.36, 1.0);
    draw_triangle(
        vec2(cx - cw / 2.0, peak_y),
        vec2(cx - cw / 2.0, peak_y - 0.14 * scale),
        vec2(cx - cw / 4.0, peak_y),
        gold_bright,
    );
    draw_triangle(
        vec2(cx, peak_y),
        vec2(cx, peak_y - 0.2 * scale),
        vec2(cx + cw / 4.0, peak_y),
        gold_bright,
    );
    draw_triangle(
        vec2(cx + cw / 2.0, peak_y),
        vec2(cx + cw / 2.0, peak_y - 0.14 * scale),
        vec2(cx + cw / 4.0, peak_y),
        gold_bright,
    );
    draw_rectangle(cx - cw / 2.0, peak_y, cw, 0.05 * scale, gold_bright);
}
