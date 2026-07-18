//! Persistent left hoard rail (UI redesign P2 — see `UI_REDESIGN_PLAN.md`).
//!
//! Always visible regardless of the active tab: hoard art, the big circular
//! click target, live hoard income, and run time since the last prestige.

use crate::simulation::idle_number::{format_amount, format_rate};
use crate::ui::theme;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(rect, "The Hoard");

    // Hoard art placeholder — real art lands in P6.
    let art = Rect::new(content.x, content.y, content.w, 108.0);
    draw_surface(
        art,
        &SurfaceStyle::new(theme::PANEL_DARK).with_border(1.0, Color::new(0.95, 0.72, 0.35, 0.55)),
    );
    draw_text_centered_in_box(
        "hoard art (P6)",
        art.x,
        art.y + art.h / 2.0 - 10.0,
        art.w,
        20.0,
        13.0,
        theme::TEXT_DIM,
    );

    draw_click_button(ctx, content, art.bottom() + 18.0, actions);
    draw_footer(ctx, content);
}

/// The big circular CLICK target. Uses radial hit-testing so only the disc is
/// clickable, matching the mockup's round button.
fn draw_click_button(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    top_y: f32,
    actions: &mut Vec<UiAction>,
) {
    let radius = content.w.min(160.0) / 2.0 - 6.0;
    let cx = content.x + content.w / 2.0;
    let cy = top_y + radius;
    let hovered = ctx.mouse.distance(vec2(cx, cy)) <= radius;
    let fill = if hovered && is_mouse_button_down(MouseButton::Left) {
        Color::new(0.26, 0.20, 0.09, 1.0)
    } else if hovered {
        Color::new(0.21, 0.17, 0.08, 1.0)
    } else {
        Color::new(0.16, 0.13, 0.06, 1.0)
    };
    draw_circle(cx, cy, radius, fill);
    draw_circle_lines(
        cx,
        cy,
        radius,
        3.0,
        Color::new(theme::GOLD.r, theme::GOLD.g, theme::GOLD.b, 0.9),
    );

    let box_x = cx - radius;
    let box_w = radius * 2.0;
    draw_text_centered_in_box(
        "CLICK",
        box_x,
        cy - 28.0,
        box_w,
        32.0,
        26.0,
        theme::TEXT_BRIGHT,
    );
    draw_text_centered_in_box(
        &format!("+{}", format_amount(ctx.state.gold_per_click(ctx.data))),
        box_x,
        cy + 4.0,
        box_w,
        24.0,
        18.0,
        Color::new(0.98, 0.80, 0.35, 1.0),
    );

    if hovered && is_mouse_button_released(MouseButton::Left) {
        actions.push(UiAction::ClickHoard);
    }
}

/// Income + run-time readouts anchored to the bottom of the rail.
fn draw_footer(ctx: &GameplayCtx<'_>, content: Rect) {
    let gps = ctx.state.gold_per_second(ctx.data);
    let minions = ctx.data.config.minion_name_plural.to_lowercase();
    let base = content.bottom() - 130.0;

    draw_ui_text_ex(
        "HOARD INCOME",
        content.x,
        base,
        TextStyle::new(13.0, theme::TEXT_DIM).params(),
    );
    draw_ui_text_ex(
        &format!("+{} / sec", format_rate(gps)),
        content.x,
        base + 26.0,
        TextStyle::new(20.0, theme::POSITIVE).params(),
    );
    draw_ui_text_ex(
        &format!("({} {minions})", ctx.state.total_minions()),
        content.x,
        base + 46.0,
        TextStyle::new(14.0, theme::TEXT_DIM).params(),
    );

    draw_ui_text_ex(
        "RUN TIME",
        content.x,
        base + 82.0,
        TextStyle::new(13.0, theme::TEXT_DIM).params(),
    );
    draw_ui_text_ex(
        &format_duration(ctx.state.run.run_seconds),
        content.x,
        base + 104.0,
        TextStyle::new(18.0, theme::TEXT_BRIGHT).params(),
    );
    draw_ui_text_ex(
        "since last prestige",
        content.x,
        base + 122.0,
        TextStyle::new(12.0, theme::TEXT_DIM).params(),
    );
}

/// Formats an elapsed-seconds count as `Xh Ym Zs`, dropping empty leading units.
fn format_duration(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}h {m}m {s}s")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}
