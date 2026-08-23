//! Treasure collection: discovered/undiscovered grid with rarity badges.

use crate::data::treasures::Rarity;
use crate::ui::icons::{self, Icon};
use crate::ui::theme;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, ScrollArea};

pub(crate) fn rarity_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::new(0.55, 0.55, 0.55, 1.0),
        Rarity::Rare => Color::new(0.35, 0.55, 0.95, 1.0),
        Rarity::Epic => Color::new(0.65, 0.40, 0.90, 1.0),
        Rarity::Legendary => Color::new(0.95, 0.72, 0.35, 1.0),
        Rarity::Mythic => Color::new(0.98, 0.38, 0.52, 1.0),
    }
}

pub fn draw(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    scroll_area: &mut ScrollArea,
    _actions: &mut Vec<UiAction>,
) {
    let discovered = &ctx.state.persistent.discovered_treasures;
    let content = ui::panel(
        rect,
        &format!(
            "Treasure Collection ({}/{})",
            discovered.len(),
            ctx.data.treasures.len()
        ),
    );

    let view = Rect::new(content.x, content.y, content.w, content.h);
    let layout = GridLayout::new(content.x, content.y, content.w, 12.0, 3, 130.0);
    let total = layout.content_height(ctx.data.treasures.len());
    scroll_area.update_at(view, total, ctx.mouse);
    let scroll = scroll_area.offset();
    for (index, def) in ctx.data.treasures.iter().enumerate() {
        let (x, y, w, h) = layout.get_item_rect(index, scroll);
        let card = Rect::new(x, y, w, h);
        if !ui::item_fully_visible(card, view) {
            continue;
        }
        let found = discovered.contains(&def.id);
        let accent = if found {
            rarity_color(def.rarity)
        } else {
            theme::TEXT_DIM
        };

        draw_surface(
            card,
            &SurfaceStyle::new(theme::PANEL)
                .with_left_accent(4.0, accent)
                .with_border(1.0, theme::BORDER_DIM),
        );
        icons::draw(Icon::Treasure, card.right() - 24.0, card.y + 24.0, 13.0);

        if found {
            draw_ui_text_ex(
                &def.name,
                card.x + 16.0,
                card.y + 26.0,
                TextStyle::new(17.0, theme::TEXT_BRIGHT).params(),
            );
            draw_badge(
                Rect::new(card.x + 16.0, card.y + 38.0, 96.0, 24.0),
                def.rarity.label(),
                accent,
                Color::new(0.05, 0.05, 0.06, 1.0),
            );
            draw_text_block(
                &format!("{}\n{}", def.description, ui::effect_text(&def.effect)),
                card.x + 16.0,
                card.y + 68.0,
                card.w - 32.0,
                h - 76.0,
                14.0,
                4.0,
                theme::TEXT_DIM,
            );
        } else if def.prestige_required > ctx.state.persistent.prestige_count {
            // Gated (#8): reveal the rarity and the prestige needed, so the
            // long tail reads as a visible chase rather than a blank mystery.
            draw_ui_text_ex(
                "Sealed",
                card.x + 16.0,
                card.y + 26.0,
                TextStyle::new(17.0, theme::TEXT_DIM).params(),
            );
            draw_badge(
                Rect::new(card.x + 16.0, card.y + 38.0, 96.0, 24.0),
                def.rarity.label(),
                rarity_color(def.rarity),
                Color::new(0.05, 0.05, 0.06, 1.0),
            );
            draw_text_block(
                &format!(
                    "Sealed in deeper ruins.\nReach Prestige {} to uncover it.",
                    def.prestige_required
                ),
                card.x + 16.0,
                card.y + 68.0,
                card.w - 32.0,
                h - 76.0,
                14.0,
                4.0,
                theme::TEXT_DIM,
            );
        } else {
            draw_ui_text_ex(
                "???",
                card.x + 16.0,
                card.y + 26.0,
                TextStyle::new(17.0, theme::TEXT_DIM).params(),
            );
            draw_text_block(
                "An undiscovered treasure sleeps in the ruins.",
                card.x + 16.0,
                card.y + 44.0,
                card.w - 32.0,
                h - 52.0,
                14.0,
                4.0,
                theme::TEXT_DIM,
            );
        }
        ui::tooltip(card, ctx.mouse, &def.name, &def.description);
    }

    ui::draw_scrollbar(scroll_area, view, total);
}
