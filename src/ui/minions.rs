//! Minions screen: goblin count, hire cost, and the income breakdown.

use crate::simulation::idle_number::{format_amount, format_rate};
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(rect, "Goblin Minions");
    let cost = ctx.state.next_hire_cost(ctx.data);
    let gps = ctx.state.gold_per_second(ctx.data);

    draw_text_centered_in_box(
        &format!("{} goblins working the hoard", ctx.state.run.goblins),
        content.x,
        content.y + 20.0,
        content.w,
        40.0,
        30.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_centered_in_box(
        &format!("generating {} gold per second", format_rate(gps)),
        content.x,
        content.y + 64.0,
        content.w,
        26.0,
        18.0,
        dark::TEXT,
    );

    let button_w = 340.0;
    if ui::button(
        Rect::new(
            content.x + (content.w - button_w) / 2.0,
            content.y + 120.0,
            button_w,
            52.0,
        ),
        &format!("Hire Goblin — {} gold", format_amount(cost)),
        ctx.state.run.gold >= cost,
        ButtonTone::Positive,
        ctx.mouse,
    ) {
        actions.push(UiAction::HireGoblin);
    }

    draw_text_block(
        "Each goblin adds passive income that keeps flowing while you're away.\nHire costs grow with every goblin (50 × 1.2ⁿ). The Minion Efficiency\nupgrade makes every goblin work harder.",
        content.x + (content.w - 560.0) / 2.0,
        content.y + 200.0,
        560.0,
        90.0,
        16.0,
        6.0,
        dark::TEXT_DIM,
    );
}
