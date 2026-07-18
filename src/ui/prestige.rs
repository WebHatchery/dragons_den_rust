//! Prestige tab: the "Hoard Legacies" branch tree (center-left) plus a side
//! column with the Burn-the-Hoard card and the derived Prestige Multipliers.

use crate::data::EffectStat;
use crate::simulation::economy;
use crate::simulation::idle_number::format_amount;
use crate::ui::theme;
use crate::ui::{self, prestige_tree, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let tree_w = rect.w * 0.7;
    let tree = Rect::new(rect.x, rect.y, tree_w - 6.0, rect.h);
    let side = Rect::new(rect.x + tree_w + 6.0, rect.y, rect.w - tree_w - 6.0, rect.h);

    let content = ui::panel(
        tree,
        &format!(
            "Hoard Legacies — {} Hoard Points",
            format_amount(ctx.state.persistent.hoard_points)
        ),
    );
    prestige_tree::draw(ctx, content, actions);

    let burn = Rect::new(side.x, side.y, side.w, rect.h * 0.5 - 4.0);
    let mult = Rect::new(
        side.x,
        side.y + rect.h * 0.5 + 4.0,
        side.w,
        rect.h * 0.5 - 4.0,
    );
    draw_burn_panel(ctx, burn, actions);
    draw_multipliers(ctx, mult);
}

fn draw_burn_panel(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(rect, "Burn the Hoard");
    let ready = ctx.state.can_prestige(ctx.data);
    let preview = ctx.state.prestige_preview(ctx.data);
    let threshold = ctx.state.prestige_threshold(ctx.data);

    meter(
        Rect::new(content.x, content.y + 6.0, content.w, 24.0),
        (ctx.state.run.gold / threshold).min(1.0) as f32,
        1.0,
        Color::new(0.95, 0.45, 0.25, 1.0),
        Some(&format!(
            "{} / {} gold",
            format_amount(ctx.state.run.gold),
            format_amount(threshold)
        )),
    );
    draw_text_block(
        &format!(
            "Prestige for +{} Hoard Points\n(prestiged {} times)",
            format_amount(preview),
            ctx.state.persistent.prestige_count
        ),
        content.x,
        content.y + 38.0,
        content.w,
        34.0,
        14.0,
        4.0,
        if ready { theme::TEXT } else { theme::TEXT_DIM },
    );

    if ui::button(
        Rect::new(content.x, content.bottom() - 42.0, content.w, 38.0),
        "PRESTIGE",
        ready,
        ButtonTone::Danger,
        ctx.mouse,
    ) {
        actions.push(UiAction::Prestige);
    }
}

fn draw_multipliers(ctx: &GameplayCtx<'_>, rect: Rect) {
    let content = ui::panel(rect, "Prestige Multipliers");
    let bonuses = economy::prestige_percent_bonuses(
        &ctx.data.prestige_upgrades,
        &ctx.state.persistent.prestige_upgrade_levels,
    );
    let rows = [
        ("Gold per Click", EffectStat::GoldPerClick),
        ("Passive Income", EffectStat::GoldPerSecond),
        ("Minion Efficiency", EffectStat::MinionEfficiency),
        ("Discovery Chance", EffectStat::DiscoveryChance),
        ("Hoard Point Gain", EffectStat::HoardPointGain),
    ];
    for (i, (label, stat)) in rows.iter().enumerate() {
        let y = content.y + 8.0 + i as f32 * 24.0;
        draw_ui_text_ex(
            label,
            content.x,
            y + 15.0,
            TextStyle::new(15.0, theme::TEXT).params(),
        );
        draw_text_centered_in_box(
            &format!("x{:.2}", bonuses.percent_multiplier(*stat)),
            content.right() - 72.0,
            y,
            70.0,
            20.0,
            15.0,
            theme::ACCENT,
        );
    }
}
