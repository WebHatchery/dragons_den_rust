//! Hoard tab: the chronicle plus expedition and prestige shortcuts. The click
//! target now lives in the persistent left rail (`left_rail.rs`), so this tab is
//! the run's overview rather than the click surface.

use crate::simulation::idle_number::format_amount;
use crate::ui::theme;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, _actions: &mut Vec<UiAction>) {
    let left_w = rect.w * 0.6 - 8.0;
    let log = Rect::new(rect.x, rect.y, left_w, rect.h);
    let right = Rect::new(
        rect.x + rect.w * 0.6 + 8.0,
        rect.y,
        rect.w * 0.4 - 8.0,
        rect.h,
    );

    draw_action_log(ctx, log);
    draw_side_column(ctx, right);
}

/// A rolling list of recent events (GDD §9), newest at the top, complementing
/// the transient toasts.
fn draw_action_log(ctx: &GameplayCtx<'_>, rect: Rect) {
    let content = ui::panel(rect, "Chronicle");
    if ctx.state.action_log.is_empty() {
        draw_text_block(
            "Your deeds will be recorded here — expeditions, discoveries, and\nthe burning of hoards.",
            content.x,
            content.y + 6.0,
            content.w,
            48.0,
            15.0,
            4.0,
            theme::TEXT_DIM,
        );
        return;
    }

    let line_height = 22.0;
    let max_lines = (content.h / line_height).floor() as usize;
    for (index, entry) in ctx
        .state
        .action_log
        .entries()
        .rev()
        .take(max_lines)
        .enumerate()
    {
        // Fade older entries toward the dim end of the palette.
        let color = if index == 0 {
            theme::TEXT_BRIGHT
        } else {
            theme::TEXT
        };
        draw_ui_text_ex(
            entry,
            content.x + 4.0,
            content.y + 14.0 + index as f32 * line_height,
            TextStyle::new(15.0, color).params(),
        );
    }
}

/// Prestige progress overview. The expedition launcher now lives in the
/// persistent bottom strip (`bottom_bar.rs`), so it is reachable from any tab.
fn draw_side_column(ctx: &GameplayCtx<'_>, rect: Rect) {
    let content = ui::panel(rect, "Prestige Progress");
    let threshold = ctx.state.prestige_threshold(ctx.data);
    meter(
        Rect::new(content.x, content.y + 8.0, content.w, 24.0),
        (ctx.state.run.gold / threshold).min(1.0) as f32,
        1.0,
        Color::new(0.95, 0.72, 0.35, 1.0),
        Some(&format!(
            "{} / {}",
            format_amount(ctx.state.run.gold),
            format_amount(threshold)
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
        theme::TEXT_DIM,
    );
}
