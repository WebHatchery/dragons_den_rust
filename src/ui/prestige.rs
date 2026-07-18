//! Prestige screen: threshold progress, Hoard Point preview, the burn
//! button, and the permanent-upgrade tree.

use crate::simulation::economy;
use crate::simulation::idle_number::format_amount;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let top = Rect::new(rect.x, rect.y, rect.w, 150.0);
    let tree = Rect::new(rect.x, rect.y + 162.0, rect.w, rect.h - 162.0);

    draw_burn_panel(ctx, top, actions);
    draw_tree_panel(ctx, tree, actions);
}

fn draw_burn_panel(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(rect, "Burn the Hoard");
    let ready = ctx.state.can_prestige(ctx.data);
    let preview = ctx.state.prestige_preview(ctx.data);

    meter(
        Rect::new(content.x, content.y + 6.0, content.w - 240.0, 26.0),
        (ctx.state.run.gold / ctx.data.config.prestige_threshold).min(1.0) as f32,
        1.0,
        Color::new(0.95, 0.45, 0.25, 1.0),
        Some(&format!(
            "{} / {} gold",
            format_amount(ctx.state.run.gold),
            format_amount(ctx.data.config.prestige_threshold)
        )),
    );
    draw_ui_text_ex(
        &format!(
            "Prestige now for +{} Hoard Points  (prestiged {} times)",
            format_amount(preview),
            ctx.state.persistent.prestige_count
        ),
        content.x,
        content.y + 62.0,
        TextStyle::new(
            16.0,
            if ready {
                dark::TEXT_BRIGHT
            } else {
                dark::TEXT_DIM
            },
        )
        .params(),
    );

    if ui::button(
        Rect::new(content.right() - 220.0, content.y + 4.0, 220.0, 50.0),
        "PRESTIGE",
        ready,
        ButtonTone::Danger,
        ctx.mouse,
    ) {
        actions.push(UiAction::Prestige);
    }
}

fn draw_tree_panel(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(
        rect,
        &format!(
            "Permanent Upgrades — {} Hoard Points",
            format_amount(ctx.state.persistent.hoard_points)
        ),
    );

    let view = Rect::new(content.x, content.y, content.w, content.h);
    let layout = GridLayout::new(content.x, content.y, content.w, 12.0, 3, 130.0);
    let total = layout.content_height(ctx.data.prestige_upgrades.len());
    let scroll = ui::apply_scroll(ctx.state.scroll_y, total, view, ctx.mouse, actions);
    for (index, def) in ctx.data.prestige_upgrades.iter().enumerate() {
        let (x, y, w, h) = layout.get_item_rect(index, scroll);
        let card = Rect::new(x, y, w, h);
        if !ui::item_fully_visible(card, view) {
            continue;
        }
        let level = ctx
            .state
            .persistent
            .prestige_upgrade_levels
            .get(&def.id)
            .copied()
            .unwrap_or(0);
        let maxed = level >= def.max_level;
        let cost = economy::upgrade_cost(def.base_cost, def.cost_growth, level);

        draw_surface(
            card,
            &SurfaceStyle::new(Color::new(0.13, 0.11, 0.17, 1.0))
                .with_left_accent(4.0, Color::new(0.75, 0.55, 0.95, 1.0))
                .with_border(1.0, Color::new(0.5, 0.55, 0.65, 0.35)),
        );
        draw_ui_text_ex(
            &format!("{}  (Lv {}/{})", def.name, level, def.max_level),
            card.x + 16.0,
            card.y + 26.0,
            TextStyle::new(17.0, dark::TEXT_BRIGHT).params(),
        );
        draw_text_block(
            &format!(
                "{}\n{} per level — persists through prestige",
                def.description,
                ui::effect_text(&def.effect)
            ),
            card.x + 16.0,
            card.y + 38.0,
            card.w - 32.0,
            44.0,
            14.0,
            4.0,
            dark::TEXT_DIM,
        );

        let label = if maxed {
            "MAX".to_owned()
        } else {
            format!("{} HP", format_amount(cost))
        };
        if ui::button(
            Rect::new(card.x + 16.0, card.y + h - 48.0, 150.0, 38.0),
            &label,
            !maxed && ctx.state.persistent.hoard_points >= cost,
            ButtonTone::Primary,
            ctx.mouse,
        ) {
            actions.push(UiAction::BuyPrestigeUpgrade(def.id.clone()));
        }
    }
}
