//! Minions tab: the base Kobold tier plus the data-driven extra tiers, each a
//! hireable card with count, per-unit income, and unlock gating (P4).

use crate::simulation::idle_number::{format_amount, format_rate};
use crate::ui::theme;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

/// A hireable minion tier resolved for display — the base tier and each extra
/// tier flatten to this so the tab and the bottom strip render uniformly.
pub(crate) struct MinionSlot {
    pub name: String,
    pub count: u32,
    pub rate_each: f64,
    pub base_cost: f64,
    pub cost_growth: f64,
    pub unlocked: bool,
    pub unlock_at: u32,
    pub action: UiAction,
}

/// Builds the ordered tier list: base Kobolds first, then each `minions.json`
/// tier. The base tier's economy comes from `GameConfig`; extras from data.
pub(crate) fn slots(ctx: &GameplayCtx<'_>) -> Vec<MinionSlot> {
    let mut slots = vec![MinionSlot {
        name: ctx.data.config.minion_name.clone(),
        count: ctx.state.run.goblins,
        rate_each: ctx.data.config.gold_per_goblin,
        base_cost: ctx.state.hire_base_cost(ctx.data),
        cost_growth: ctx.data.config.hire_cost_growth,
        unlocked: true,
        unlock_at: 0,
        action: UiAction::HireGoblin,
    }];
    for def in &ctx.data.minions {
        slots.push(MinionSlot {
            name: def.name.clone(),
            count: ctx.state.minion_count(&def.id),
            rate_each: def.base_rate,
            base_cost: def.base_cost,
            cost_growth: def.cost_growth,
            unlocked: ctx.state.minion_unlocked(def),
            unlock_at: def.unlock_at,
            action: UiAction::HireMinion(def.id.clone()),
        });
    }
    slots
}

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(
        rect,
        &format!("Minions ({} working)", ctx.state.total_minions()),
    );
    draw_ui_text_ex(
        &format!(
            "{} / sec total passive income",
            format_rate(ctx.state.gold_per_second(ctx.data))
        ),
        content.x,
        content.y + 18.0,
        TextStyle::new(16.0, theme::TEXT_DIM).params(),
    );

    let selector_w = 260.0;
    ui::buy_mode_selector(
        Rect::new(
            content.right() - selector_w,
            content.y - 2.0,
            selector_w,
            32.0,
        ),
        ctx.state.buy_mode,
        ctx.mouse,
        actions,
    );

    let grid = GridLayout::new(content.x, content.y + 40.0, content.w, 12.0, 2, 120.0);
    for (i, slot) in slots(ctx).iter().enumerate() {
        let (x, y, w, h) = grid.get_item_rect(i, 0.0);
        draw_card(ctx, Rect::new(x, y, w, h), slot, actions);
    }
}

fn draw_card(ctx: &GameplayCtx<'_>, rect: Rect, slot: &MinionSlot, actions: &mut Vec<UiAction>) {
    let accent = if slot.unlocked {
        theme::ACCENT
    } else {
        theme::TEXT_DIM
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(theme::PANEL)
            .with_left_accent(4.0, accent)
            .with_border(1.0, theme::BORDER_DIM),
    );

    if !slot.unlocked {
        draw_ui_text_ex(
            &slot.name,
            rect.x + 16.0,
            rect.y + 30.0,
            TextStyle::new(18.0, theme::TEXT_DIM).params(),
        );
        draw_text_centered_in_box(
            &format!("Unlocks at {} minions", slot.unlock_at),
            rect.x,
            rect.y + rect.h / 2.0,
            rect.w,
            24.0,
            15.0,
            theme::TEXT_DIM,
        );
        return;
    }

    draw_ui_text_ex(
        &format!("{}   Lvl {}", slot.name, slot.count),
        rect.x + 16.0,
        rect.y + 30.0,
        TextStyle::new(18.0, theme::TEXT_BRIGHT).params(),
    );
    draw_ui_text_ex(
        &format!("+{} / sec each", format_amount(slot.rate_each)),
        rect.x + 16.0,
        rect.y + 54.0,
        TextStyle::new(14.0, theme::POSITIVE).params(),
    );

    let quote = ui::bulk_quote(
        slot.base_cost,
        slot.cost_growth,
        slot.count,
        u32::MAX,
        ctx.state.run.gold,
        ctx.state.buy_mode,
    );
    let label = if quote.count > 1 {
        format!("Hire {} — {} gold", quote.count, format_amount(quote.cost))
    } else {
        format!("Hire — {} gold", format_amount(quote.cost))
    };
    if ui::button(
        Rect::new(rect.x + 16.0, rect.bottom() - 46.0, rect.w - 32.0, 38.0),
        &label,
        quote.affordable,
        ButtonTone::Primary,
        ctx.mouse,
    ) {
        actions.push(slot.action.clone());
    }
}
