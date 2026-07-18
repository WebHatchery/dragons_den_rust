//! Hoard screen: the click target plus expedition and prestige shortcuts.

use crate::simulation::idle_number::{format_amount, format_rate};
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let left = Rect::new(rect.x, rect.y, rect.w * 0.6 - 8.0, rect.h);
    let right = Rect::new(
        rect.x + rect.w * 0.6 + 8.0,
        rect.y,
        rect.w * 0.4 - 8.0,
        rect.h,
    );

    draw_click_target(ctx, left, actions);
    draw_side_column(ctx, right, actions);
}

fn draw_click_target(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(rect, "The Hoard");
    let hovered = content.contains_point(ctx.mouse);
    let fill = if hovered && is_mouse_button_down(MouseButton::Left) {
        Color::new(0.24, 0.19, 0.08, 1.0)
    } else if hovered {
        Color::new(0.20, 0.16, 0.07, 1.0)
    } else {
        Color::new(0.16, 0.13, 0.06, 1.0)
    };
    draw_surface(
        content,
        &SurfaceStyle::new(fill).with_border(2.0, Color::new(0.95, 0.72, 0.35, 0.8)),
    );

    draw_text_centered_in_box(
        &format_amount(ctx.state.run.gold),
        content.x,
        content.y + content.h * 0.30,
        content.w,
        60.0,
        54.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_centered_in_box(
        "gold in the hoard — click to collect more",
        content.x,
        content.y + content.h * 0.30 + 64.0,
        content.w,
        26.0,
        17.0,
        dark::TEXT_DIM,
    );
    draw_text_centered_in_box(
        &format!(
            "+{} per click   |   +{} per second",
            format_amount(ctx.state.gold_per_click(ctx.data)),
            format_rate(ctx.state.gold_per_second(ctx.data))
        ),
        content.x,
        content.bottom() - 44.0,
        content.w,
        26.0,
        17.0,
        dark::TEXT,
    );

    if hovered && is_mouse_button_released(MouseButton::Left) {
        actions.push(UiAction::ClickHoard);
    }
}

fn draw_side_column(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let expedition = Rect::new(rect.x, rect.y, rect.w, rect.h * 0.55 - 6.0);
    let prestige = Rect::new(
        rect.x,
        rect.y + rect.h * 0.55 + 6.0,
        rect.w,
        rect.h * 0.45 - 6.0,
    );

    let content = ui::panel(expedition, "Expedition");
    let cost = ctx.data.config.explore_cost;
    let chance = ctx.state.discovery_chance(ctx.data);
    let discovered = ctx.state.persistent.discovered_treasures.len();
    draw_text_block(
        &format!(
            "Send goblins into the old ruins.\nCost: {} gold\nDiscovery chance: {:.0}%\nTreasures found: {}/{}",
            format_amount(cost),
            chance * 100.0,
            discovered,
            ctx.data.treasures.len()
        ),
        content.x,
        content.y + 6.0,
        content.w,
        110.0,
        17.0,
        6.0,
        dark::TEXT,
    );
    if ui::button(
        Rect::new(content.x, content.bottom() - 48.0, content.w, 44.0),
        "Explore Ruins",
        ctx.state.run.gold >= cost,
        ButtonTone::Primary,
        ctx.mouse,
    ) {
        actions.push(UiAction::Explore);
    }

    let content = ui::panel(prestige, "Prestige Progress");
    meter(
        Rect::new(content.x, content.y + 8.0, content.w, 24.0),
        (ctx.state.run.gold / ctx.data.config.prestige_threshold).min(1.0) as f32,
        1.0,
        Color::new(0.95, 0.72, 0.35, 1.0),
        Some(&format!(
            "{} / {}",
            format_amount(ctx.state.run.gold),
            format_amount(ctx.data.config.prestige_threshold)
        )),
    );
    draw_text_block(
        if ctx.state.can_prestige(ctx.data) {
            "The hoard is ready to burn — open the Prestige tab."
        } else {
            "Grow the hoard to unlock prestige on the Prestige tab."
        },
        content.x,
        content.y + 44.0,
        content.w,
        60.0,
        16.0,
        4.0,
        dark::TEXT_DIM,
    );
}
