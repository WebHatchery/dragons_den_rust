//! Procedural vector icons (UI redesign P6), drawn from primitives so the game
//! ships no art assets — matching the GDD §0 "lowest-art genre" stance. Each
//! icon draws centered at `(cx, cy)` within radius `r`.

use crate::data::PrestigeBranch;
use crate::ui::theme;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub enum Icon {
    Coin,
    Minion,
    Gem,
    Settings,
    Treasure,
    Prestige(PrestigeBranch),
}

pub fn draw(icon: Icon, cx: f32, cy: f32, r: f32) {
    match icon {
        Icon::Coin => coin(cx, cy, r),
        Icon::Minion => minion(cx, cy, r),
        Icon::Gem => gem(cx, cy, r),
        Icon::Settings => settings(cx, cy, r),
        Icon::Treasure => treasure(cx, cy, r),
        Icon::Prestige(branch) => prestige(cx, cy, r, branch),
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

/// A small settings cog used anywhere a control opens configuration.
pub fn settings(cx: f32, cy: f32, r: f32) {
    draw_circle_lines(cx, cy, r * 0.62, 2.5, theme::TEXT_BRIGHT);
    draw_circle(cx, cy, r * 0.22, theme::PANEL_DARK);
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::PI / 4.0;
        let inner = vec2(cx + a.cos() * r * 0.62, cy + a.sin() * r * 0.62);
        let outer = vec2(cx + a.cos() * r * 0.92, cy + a.sin() * r * 0.92);
        draw_line(inner.x, inner.y, outer.x, outer.y, 3.0, theme::TEXT_BRIGHT);
    }
}

/// A faceted chest silhouette for collection and discovery screens.
pub fn treasure(cx: f32, cy: f32, r: f32) {
    let gold = Color::new(0.82, 0.57, 0.22, 1.0);
    draw_rectangle(cx - r * 0.72, cy - r * 0.18, r * 1.44, r * 0.72, gold);
    draw_rectangle_lines(
        cx - r * 0.72,
        cy - r * 0.18,
        r * 1.44,
        r * 0.72,
        2.0,
        theme::BORDER,
    );
    draw_arc(
        cx,
        cy - r * 0.18,
        6,
        r * 0.72,
        180.0,
        1.0,
        180.0,
        theme::ACCENT,
    );
    draw_rectangle(
        cx - r * 0.08,
        cy + r * 0.04,
        r * 0.16,
        r * 0.25,
        theme::PANEL_DARK,
    );
}

/// Branch-specific sigils keep the prestige tree scannable even without color.
pub fn prestige(cx: f32, cy: f32, r: f32, branch: PrestigeBranch) {
    let c = Color::new(1.0, 0.92, 0.68, 0.95);
    match branch {
        PrestigeBranch::Greed => draw_poly(cx, cy, 6, r * 0.62, 0.0, c),
        PrestigeBranch::Power => {
            draw_triangle(
                vec2(cx, cy - r * 0.7),
                vec2(cx - r * 0.45, cy + r * 0.5),
                vec2(cx + r * 0.1, cy + r * 0.18),
                c,
            );
            draw_triangle(
                vec2(cx + r * 0.05, cy - r * 0.1),
                vec2(cx + r * 0.55, cy - r * 0.1),
                vec2(cx - r * 0.15, cy + r * 0.7),
                c,
            );
        }
        PrestigeBranch::Discovery => {
            draw_circle_lines(cx, cy, r * 0.52, 2.5, c);
            draw_line(
                cx + r * 0.35,
                cy + r * 0.35,
                cx + r * 0.72,
                cy + r * 0.72,
                3.0,
                c,
            );
        }
        PrestigeBranch::Legion => {
            draw_circle(cx, cy - r * 0.3, r * 0.25, c);
            draw_line(
                cx - r * 0.6,
                cy + r * 0.55,
                cx + r * 0.6,
                cy + r * 0.55,
                3.0,
                c,
            );
            draw_line(cx, cy - r * 0.02, cx, cy + r * 0.55, 3.0, c);
        }
        PrestigeBranch::Eternity => draw_circle_lines(cx, cy, r * 0.58, 3.0, c),
    }
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
