//! Persistent bottom strip (UI redesign P3 — see `UI_REDESIGN_PLAN.md`).
//!
//! Always visible under the center: a minions summary (type cards land in P4),
//! the expedition launcher, and the most recent treasure finds.

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

    draw_minions_placeholder(ctx, minions);
    draw_expeditions(ctx, expeditions, actions);
    draw_recent_treasures(ctx, treasures);
}

/// Minion roster summary — the typed hire cards arrive in P4.
fn draw_minions_placeholder(ctx: &GameplayCtx<'_>, rect: Rect) {
    let content = ui::panel(
        rect,
        &format!(
            "{} ({})",
            ctx.data.config.minion_name_plural, ctx.state.run.goblins
        ),
    );
    draw_text_centered_in_box(
        "type cards (P4)",
        content.x,
        content.y + content.h / 2.0 - 10.0,
        content.w,
        20.0,
        14.0,
        theme::TEXT_DIM,
    );
}

/// The expedition launcher (moved off the Hoard tab so it is always reachable).
fn draw_expeditions(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(rect, "Expeditions");
    let cost = ctx.data.config.explore_cost;
    let chance = ctx.state.discovery_chance(ctx.data);
    let discovered = ctx.state.persistent.discovered_treasures.len();

    draw_text_block(
        &format!(
            "Cost {} gold  ·  {:.0}% find\n{} / {} treasures found",
            format_amount(cost),
            chance * 100.0,
            discovered,
            ctx.data.treasures.len()
        ),
        content.x,
        content.y + 2.0,
        content.w,
        34.0,
        14.0,
        4.0,
        theme::TEXT,
    );
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
