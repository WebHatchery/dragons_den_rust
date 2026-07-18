//! Upgrade shop: the single converged catalog (GDD §0 — one catalog, wired).

use crate::simulation::idle_number::format_amount;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let content = ui::panel(rect, "Upgrade Shop");

    // Buy-quantity selector spans the top; the card grid starts below it.
    let selector_w = 260.0;
    ui::buy_mode_selector(
        Rect::new(content.right() - selector_w, content.y, selector_w, 34.0),
        ctx.state.buy_mode,
        ctx.mouse,
        actions,
    );

    let grid_top = content.y + 46.0;
    let layout = GridLayout::new(content.x, grid_top, content.w, 12.0, 2, 110.0);

    for (index, def) in ctx.data.upgrades.iter().enumerate() {
        let (x, y, w, h) = layout.get_item_rect(index, 0.0);
        let card = Rect::new(x, y, w, h);
        let level = ctx
            .state
            .run
            .upgrade_levels
            .get(&def.id)
            .copied()
            .unwrap_or(0);
        let maxed = level >= def.max_level;
        let remaining = def.max_level.saturating_sub(level);
        let quote = ui::bulk_quote(
            def.base_cost,
            def.cost_growth,
            level,
            remaining,
            ctx.state.run.gold,
            ctx.state.buy_mode,
        );

        draw_surface(
            card,
            &SurfaceStyle::new(Color::new(0.11, 0.125, 0.16, 1.0))
                .with_left_accent(4.0, if maxed { dark::TEXT_DIM } else { dark::ACCENT })
                .with_border(1.0, Color::new(0.5, 0.55, 0.65, 0.35)),
        );
        draw_ui_text_ex(
            &format!("{}  (Lv {}/{})", def.name, level, def.max_level),
            card.x + 16.0,
            card.y + 28.0,
            TextStyle::new(18.0, dark::TEXT_BRIGHT).params(),
        );
        draw_text_block(
            &def.description,
            card.x + 16.0,
            card.y + 40.0,
            card.w - 180.0,
            40.0,
            14.0,
            4.0,
            dark::TEXT_DIM,
        );
        draw_ui_text_ex(
            &format!(
                "+{} {} per level",
                def.effect.rate,
                ui::stat_label(def.effect.stat)
            ),
            card.x + 16.0,
            card.y + h - 14.0,
            TextStyle::new(14.0, dark::TEXT).params(),
        );

        let label = if maxed {
            "MAX".to_owned()
        } else if quote.count > 1 {
            format!("×{}  {} gold", quote.count, format_amount(quote.cost))
        } else {
            format!("{} gold", format_amount(quote.cost))
        };
        if ui::button(
            Rect::new(card.right() - 176.0, card.y + h - 54.0, 160.0, 40.0),
            &label,
            !maxed && quote.affordable,
            ButtonTone::Positive,
            ctx.mouse,
        ) {
            actions.push(UiAction::BuyUpgrade(def.id.clone()));
        }
    }
}
