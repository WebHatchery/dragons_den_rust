//! Dragon Codex: the 8-element flavor gallery (GDD §5.5) — identity and a
//! small passive bonus, never a simulation.

use crate::ui::{self, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let _ = actions; // display-only screen
    let unlocked_count = ctx.state.persistent.unlocked_dragons.len();
    let content = ui::panel(
        rect,
        &format!(
            "Dragon Codex ({}/{})",
            unlocked_count,
            ctx.data.dragons.len()
        ),
    );

    let layout = GridLayout::new(content.x, content.y, content.w, 12.0, 4, 150.0);
    for (index, def) in ctx.data.dragons.iter().enumerate() {
        let (x, y, w, h) = layout.get_item_rect(index, 0.0);
        let card = Rect::new(x, y, w, h);
        let unlocked = ctx.state.persistent.unlocked_dragons.contains(&def.id);
        let swatch = def.swatch();
        let accent = if unlocked {
            swatch
        } else {
            Color::new(swatch.r * 0.3, swatch.g * 0.3, swatch.b * 0.3, 1.0)
        };

        draw_surface(
            card,
            &SurfaceStyle::new(Color::new(0.11, 0.125, 0.16, 1.0))
                .with_left_accent(5.0, accent)
                .with_border(1.0, Color::new(0.5, 0.55, 0.65, 0.35)),
        );
        draw_ui_text_ex(
            if unlocked { &def.name } else { "???" },
            card.x + 16.0,
            card.y + 26.0,
            TextStyle::new(
                17.0,
                if unlocked {
                    dark::TEXT_BRIGHT
                } else {
                    dark::TEXT_DIM
                },
            )
            .params(),
        );
        draw_badge(
            Rect::new(card.x + 16.0, card.y + 38.0, 90.0, 22.0),
            &def.element,
            accent,
            Color::new(0.05, 0.05, 0.06, 1.0),
        );

        let body = if unlocked {
            format!("{}\n{}", def.flavor, ui::effect_text(&def.effect))
        } else {
            ui::condition_text(&def.unlock, ctx.data)
        };
        draw_text_block(
            &body,
            card.x + 16.0,
            card.y + 68.0,
            card.w - 32.0,
            h - 76.0,
            13.0,
            4.0,
            dark::TEXT_DIM,
        );
    }
}
