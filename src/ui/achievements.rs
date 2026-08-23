//! Achievement checklist — every entry shows its real condition and reward.

use crate::simulation::idle_number::format_amount;
use crate::ui::theme;
use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, ScrollArea};

pub fn draw(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    scroll_area: &mut ScrollArea,
    _actions: &mut Vec<UiAction>,
) {
    let (unlocked, total) = ctx.state.persistent.achievements.progress();
    let content = ui::panel(rect, &format!("Achievements ({unlocked}/{total})"));

    let view = Rect::new(content.x, content.y, content.w, content.h);
    let layout = GridLayout::new(content.x, content.y, content.w, 10.0, 2, 88.0);
    let total_h = layout.content_height(ctx.data.achievements.len());
    scroll_area.update_at(view, total_h, ctx.mouse);
    let scroll = scroll_area.offset();
    for (index, def) in ctx.data.achievements.iter().enumerate() {
        let (x, y, w, h) = layout.get_item_rect(index, scroll);
        let card = Rect::new(x, y, w, h);
        if !ui::item_fully_visible(card, view) {
            continue;
        }
        let done = ctx.state.persistent.achievements.is_unlocked(&def.id);
        let accent = if done {
            theme::POSITIVE
        } else {
            theme::TEXT_DIM
        };

        draw_surface(
            card,
            &SurfaceStyle::new(theme::PANEL)
                .with_left_accent(4.0, accent)
                .with_border(1.0, theme::BORDER_DIM),
        );
        draw_ui_text_ex(
            &format!("{} {}", if done { "[X]" } else { "[ ]" }, def.name),
            card.x + 16.0,
            card.y + 26.0,
            TextStyle::new(
                17.0,
                if done {
                    theme::TEXT_BRIGHT
                } else {
                    theme::TEXT
                },
            )
            .params(),
        );

        let reward = if def.reward.hoard_points > 0.0 {
            format!("+{} Hoard Points", format_amount(def.reward.hoard_points))
        } else {
            format!("+{} gold", format_amount(def.reward.gold))
        };
        draw_text_block(
            &format!(
                "{}\n{} — reward {}",
                def.description,
                ui::condition_text(&def.condition, ctx.data),
                reward
            ),
            card.x + 16.0,
            card.y + 38.0,
            card.w - 32.0,
            h - 46.0,
            14.0,
            4.0,
            theme::TEXT_DIM,
        );
    }

    ui::draw_scrollbar(scroll_area, view, total_h);
}
