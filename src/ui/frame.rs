//! Persistent gameplay frame (UI redesign P1 — see `UI_REDESIGN_PLAN.md`).
//!
//! Splits the logical screen into four regions — a header of resource cards, a
//! persistent left hoard rail, a tab-swapped center, and a bottom strip — and
//! draws the header chrome. Only the center changes per tab; the rail and bottom
//! strip are drawn by their own modules (`left_rail.rs`, `bottom_bar.rs`).

use crate::simulation::idle_number::{format_amount, format_rate};
use crate::ui::icons::{self, Icon};
use crate::ui::theme;
use crate::ui::{self, GameplayCtx, UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const MARGIN: f32 = 12.0;
const HEADER_H: f32 = 92.0;
const TAB_H: f32 = 38.0;
const RAIL_W: f32 = 230.0;
const BOTTOM_H: f32 = 152.0;
const GAP: f32 = 8.0;

/// The four persistent regions plus the tab-bar strip, in logical coords.
pub struct FrameRegions {
    pub header: Rect,
    pub tabs: Rect,
    pub left_rail: Rect,
    pub center: Rect,
    pub bottom: Rect,
}

/// Computes the frame layout from the fixed logical resolution.
pub fn regions() -> FrameRegions {
    let full_w = LOGICAL_WIDTH - MARGIN * 2.0;
    let header = Rect::new(MARGIN, 8.0, full_w, HEADER_H);
    let tabs = Rect::new(MARGIN, header.bottom() + 6.0, full_w, TAB_H);

    let main_top = tabs.bottom() + 6.0;
    let screen_bottom = LOGICAL_HEIGHT - MARGIN;
    let left_rail = Rect::new(MARGIN, main_top, RAIL_W, screen_bottom - main_top);

    let content_x = left_rail.right() + GAP;
    let content_w = LOGICAL_WIDTH - content_x - MARGIN;
    let bottom = Rect::new(content_x, screen_bottom - BOTTOM_H, content_w, BOTTOM_H);
    let center = Rect::new(content_x, main_top, content_w, bottom.y - GAP - main_top);

    FrameRegions {
        header,
        tabs,
        left_rail,
        center,
        bottom,
    }
}

/// Draws the header: title, three resource cards, and Save / Menu / settings.
pub fn draw_header(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    draw_surface(
        rect,
        &SurfaceStyle::new(theme::PANEL)
            .with_border(1.0, theme::ACCENT)
            .with_top_highlight(2.0, Color::new(0.95, 0.72, 0.35, 0.75)),
    );

    // Title with a soft drop shadow and gold fill (the mockup's gold wordmark).
    let title = &ctx.data.config.display_name;
    let tx = rect.x + 22.0;
    let ty = rect.y + rect.h / 2.0 + 11.0;
    draw_ui_text_ex(
        title,
        tx + 2.0,
        ty + 2.0,
        TextStyle::new(33.0, Color::new(0.0, 0.0, 0.0, 0.55)).params(),
    );
    draw_ui_text_ex(title, tx, ty, TextStyle::new(33.0, theme::ACCENT).params());

    // Right-aligned controls: gear, then Menu, then Save.
    let btn_y = rect.y + (rect.h - 40.0) / 2.0;
    let gear = Rect::new(rect.right() - 60.0, btn_y, 44.0, 40.0);
    if ui::button(gear, "*", true, ButtonTone::Secondary, ctx.mouse) {
        actions.push(UiAction::OpenSettings);
    }
    let menu = Rect::new(gear.x - 104.0, btn_y, 96.0, 40.0);
    if ui::button(menu, "MENU", true, ButtonTone::Secondary, ctx.mouse) {
        actions.push(UiAction::BackToMenu);
    }
    let save = Rect::new(menu.x - 104.0, btn_y, 96.0, 40.0);
    if ui::button(save, "SAVE", true, ButtonTone::Positive, ctx.mouse) {
        actions.push(UiAction::SaveNow);
    }

    // Three resource cards, centered in the gap between title and controls.
    let card_w = 220.0;
    let card_h = 68.0;
    let group_w = card_w * 3.0 + GAP * 2.0;
    let mut x = (LOGICAL_WIDTH - group_w) / 2.0;
    let card_y = rect.y + (rect.h - card_h) / 2.0;
    let gps = ctx.state.gold_per_second(ctx.data);
    let rate = format!("+{} / sec", format_rate(gps));

    resource_card(
        Rect::new(x, card_y, card_w, card_h),
        Icon::Coin,
        "Gold",
        &format_amount(ctx.state.run.gold),
        Some(&rate),
    );
    x += card_w + GAP;
    resource_card(
        Rect::new(x, card_y, card_w, card_h),
        Icon::Minion,
        &ctx.data.config.minion_name_plural,
        &ctx.state.total_minions().to_string(),
        Some(&rate),
    );
    x += card_w + GAP;
    resource_card(
        Rect::new(x, card_y, card_w, card_h),
        Icon::Gem,
        "Hoard Points",
        &format_amount(ctx.state.persistent.hoard_points),
        None,
    );
}

/// A single header resource card: procedural icon + title + big value + rate.
fn resource_card(rect: Rect, icon: Icon, title: &str, value: &str, rate: Option<&str>) {
    draw_surface(
        rect,
        &SurfaceStyle::new(theme::PANEL).with_border(1.0, theme::BORDER_DIM),
    );

    let chip_cx = rect.x + 32.0;
    let chip_cy = rect.y + rect.h / 2.0;
    icons::draw(icon, chip_cx, chip_cy, 18.0);

    let text_x = chip_cx + 32.0;
    draw_ui_text_ex(
        title,
        text_x,
        rect.y + 22.0,
        TextStyle::new(14.0, theme::TEXT_DIM).params(),
    );
    draw_ui_text_ex(
        value,
        text_x,
        rect.y + 46.0,
        TextStyle::new(26.0, theme::TEXT_BRIGHT).params(),
    );
    if let Some(rate) = rate {
        draw_ui_text_ex(
            rate,
            text_x,
            rect.y + 62.0,
            TextStyle::new(13.0, theme::POSITIVE).params(),
        );
    }
}
