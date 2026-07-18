//! Minions screen: minion count, hire cost, and the income breakdown.

use crate::simulation::idle_number::{format_amount, format_rate};
use crate::ui::theme;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let minion = &ctx.data.config.minion_name;
    let minions = &ctx.data.config.minion_name_plural;
    let minion_lower = minion.to_lowercase();
    let minions_lower = minions.to_lowercase();

    let content = ui::panel(rect, &format!("{minion} Minions"));
    let gps = ctx.state.gold_per_second(ctx.data);

    draw_text_centered_in_box(
        &format!(
            "{} {minions_lower} working the hoard",
            ctx.state.run.goblins
        ),
        content.x,
        content.y + 20.0,
        content.w,
        40.0,
        30.0,
        theme::TEXT_BRIGHT,
    );
    draw_text_centered_in_box(
        &format!("generating {} gold per second", format_rate(gps)),
        content.x,
        content.y + 64.0,
        content.w,
        26.0,
        18.0,
        theme::TEXT,
    );

    // Buy-quantity selector: goblins are uncapped, so remaining is u32::MAX.
    let selector_w = 260.0;
    ui::buy_mode_selector(
        Rect::new(
            content.x + (content.w - selector_w) / 2.0,
            content.y + 108.0,
            selector_w,
            34.0,
        ),
        ctx.state.buy_mode,
        ctx.mouse,
        actions,
    );

    let quote = ui::bulk_quote(
        ctx.state.hire_base_cost(ctx.data),
        ctx.data.config.hire_cost_growth,
        ctx.state.run.goblins,
        u32::MAX,
        ctx.state.run.gold,
        ctx.state.buy_mode,
    );
    let label = if quote.count > 1 {
        format!(
            "Hire {} {minions} — {} gold",
            quote.count,
            format_amount(quote.cost)
        )
    } else {
        format!("Hire {minion} — {} gold", format_amount(quote.cost))
    };
    let button_w = 340.0;
    if ui::button(
        Rect::new(
            content.x + (content.w - button_w) / 2.0,
            content.y + 156.0,
            button_w,
            52.0,
        ),
        &label,
        quote.affordable,
        ButtonTone::Positive,
        ctx.mouse,
    ) {
        actions.push(UiAction::HireGoblin);
    }

    draw_text_block(
        &format!("Each {minion_lower} adds passive income that keeps flowing while you're away.\nHire costs grow with every {minion_lower} (50 × 1.2ⁿ). The Minion Efficiency\nupgrade makes every {minion_lower} work harder."),
        content.x + (content.w - 560.0) / 2.0,
        content.y + 240.0,
        560.0,
        90.0,
        16.0,
        6.0,
        theme::TEXT_DIM,
    );
}
