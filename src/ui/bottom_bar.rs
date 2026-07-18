//! Persistent bottom strip (UI redesign P3 — see `UI_REDESIGN_PLAN.md`).
//!
//! Always visible under the center: compact minion-tier hire cards, the
//! expedition launcher, and the most recent treasure finds.

use crate::simulation::idle_number::format_amount;
use crate::ui::theme;
use crate::ui::treasures::rarity_color;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const GAP: f32 = 8.0;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let minions_w = rect.w * 0.5;
    let col_w = (rect.w - minions_w - GAP * 2.0) / 2.0;
    let minions = Rect::new(rect.x, rect.y, minions_w - GAP, rect.h);
    let expeditions = Rect::new(rect.x + minions_w + GAP, rect.y, col_w, rect.h);
    let treasures = Rect::new(expeditions.right() + GAP, rect.y, col_w, rect.h);

    draw_minions(ctx, minions, actions);
    draw_expeditions(ctx, expeditions, actions);
    draw_recent_treasures(ctx, treasures);
}

/// Compact hire cards for every minion tier (base + extras), one row across.
fn draw_minions(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(
        rect,
        &format!(
            "{} ({})",
            ctx.data.config.minion_name_plural,
            ctx.state.total_minions()
        ),
    );
    let slots = crate::ui::minions::slots(ctx);
    let gap = 6.0;
    let card_w = (content.w - gap * (slots.len() as f32 - 1.0)) / slots.len() as f32;
    for (i, slot) in slots.iter().enumerate() {
        let card = Rect::new(
            content.x + i as f32 * (card_w + gap),
            content.y,
            card_w,
            content.h,
        );
        draw_minion_card(ctx, card, slot, actions);
    }
}

fn draw_minion_card(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    slot: &crate::ui::minions::MinionSlot,
    actions: &mut Vec<UiAction>,
) {
    let accent = if slot.unlocked {
        theme::ACCENT
    } else {
        theme::TEXT_DIM
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(theme::PANEL_DARK)
            .with_left_accent(3.0, accent)
            .with_border(1.0, theme::BORDER_DIM),
    );

    if !slot.unlocked {
        let hint = if ctx.state.persistent.prestige_count < slot.prestige_required {
            format!("Prestige {}", slot.prestige_required)
        } else {
            format!("Unlocks at {}", slot.unlock_at)
        };
        draw_text_centered_in_box(
            &format!("{}\n{}", slot.name, hint),
            rect.x,
            rect.y + rect.h / 2.0 - 16.0,
            rect.w,
            36.0,
            13.0,
            theme::TEXT_DIM,
        );
        return;
    }

    draw_ui_text_ex(
        &format!("{}  x{}", slot.name, slot.count),
        rect.x + 8.0,
        rect.y + 20.0,
        TextStyle::new(14.0, theme::TEXT_BRIGHT).params(),
    );
    let quote = ui::bulk_quote(
        slot.base_cost,
        slot.cost_growth,
        slot.count,
        u32::MAX,
        ctx.state.run.gold,
        ctx.state.buy_mode,
    );
    if ui::button(
        Rect::new(rect.x + 6.0, rect.bottom() - 32.0, rect.w - 12.0, 28.0),
        &format_amount(quote.cost),
        quote.affordable,
        ButtonTone::Primary,
        ctx.mouse,
    ) {
        actions.push(slot.action.clone());
    }
}

/// The expedition launcher (moved off the Hoard tab so it is always reachable).
fn draw_expeditions(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(rect, "Expeditions");
    let cost = ctx.state.explore_cost(ctx.data);
    let chance = ctx.state.discovery_chance(ctx.data);
    let discovered = ctx.state.persistent.discovered_treasures.len();

    // Line 1: cost + find chance (always shown).
    draw_ui_text_ex(
        &format!(
            "Cost {} gold  ·  {:.0}% find",
            format_amount(cost),
            chance * 100.0
        ),
        content.x,
        content.y + 18.0,
        TextStyle::new(14.0, theme::TEXT).params(),
    );
    // Line 2 doubles as the Hoard Rush readout while a surge is live (#7), so an
    // active buff is visible without adding a third line to the short panel.
    if ctx.state.hoard_rush_active() {
        draw_ui_text_ex(
            &format!(
                "Hoard Rush  x{:.0}  ·  {:.0}s left",
                ctx.data.config.hoard_rush_multiplier,
                ctx.state.hoard_rush_remaining().ceil()
            ),
            content.x,
            content.y + 40.0,
            TextStyle::new(14.0, theme::ACCENT).params(),
        );
    } else {
        draw_ui_text_ex(
            &format!(
                "{} / {} treasures found",
                discovered,
                ctx.data.treasures.len()
            ),
            content.x,
            content.y + 40.0,
            TextStyle::new(14.0, theme::TEXT).params(),
        );
    }
    if ui::button(
        Rect::new(content.x, content.bottom() - 34.0, content.w, 32.0),
        "Explore Ruins",
        ctx.state.run.gold >= cost,
        ButtonTone::Primary,
        ctx.mouse,
    ) {
        actions.push(UiAction::Explore);
    }
}

/// The last few discovered treasures, newest first, with a rarity tag.
fn draw_recent_treasures(ctx: &GameplayCtx<'_>, rect: Rect) {
    let content = ui::panel(rect, "Recent Treasures");
    let discovered = &ctx.state.persistent.discovered_treasures;
    if discovered.is_empty() {
        draw_text_block(
            "None yet — send an expedition into the ruins.",
            content.x,
            content.y + 6.0,
            content.w,
            48.0,
            14.0,
            4.0,
            theme::TEXT_DIM,
        );
        return;
    }

    for (i, id) in discovered.iter().rev().take(3).enumerate() {
        let Some(def) = ctx.data.treasure(id) else {
            continue;
        };
        let y = content.y + 4.0 + i as f32 * 22.0;
        draw_ui_text_ex(
            &def.name,
            content.x,
            y + 15.0,
            TextStyle::new(15.0, theme::TEXT).params(),
        );
        draw_text_centered_in_box(
            def.rarity.label(),
            content.right() - 78.0,
            y,
            76.0,
            20.0,
            12.0,
            rarity_color(def.rarity),
        );
    }
}
