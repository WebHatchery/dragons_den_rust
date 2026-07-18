//! The five-branch "Hoard Legacies" prestige tree (UI redesign P5).
//!
//! Renders one column per [`PrestigeBranch`], each a vertical chain of node
//! circles connected top-to-bottom. A node unlocks once its prerequisite has a
//! level; clicking a buyable node spends Hoard Points via `BuyPrestigeUpgrade`.

use crate::data::{PrestigeBranch, PrestigeUpgradeDef};
use crate::simulation::economy;
use crate::simulation::idle_number::format_amount;
use crate::ui::theme;
use crate::ui::{GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

const RADIUS: f32 = 22.0;
const TIER_SPACING: f32 = 84.0;

/// The signature accent for each branch column.
pub fn branch_color(branch: PrestigeBranch) -> Color {
    match branch {
        PrestigeBranch::Greed => theme::ACCENT,
        PrestigeBranch::Power => Color::new(0.82, 0.34, 0.30, 1.0),
        PrestigeBranch::Discovery => Color::new(0.38, 0.58, 0.95, 1.0),
        PrestigeBranch::Legion => theme::POSITIVE,
        PrestigeBranch::Eternity => theme::HOARD_POINT,
    }
}

pub fn draw(ctx: &GameplayCtx<'_>, rect: Rect, actions: &mut Vec<UiAction>) {
    let col_w = rect.w / PrestigeBranch::ALL.len() as f32;
    for (i, branch) in PrestigeBranch::ALL.iter().enumerate() {
        let col_x = rect.x + i as f32 * col_w;
        let cx = col_x + col_w / 2.0;
        let color = branch_color(*branch);

        let mut nodes: Vec<&PrestigeUpgradeDef> = ctx
            .data
            .prestige_upgrades
            .iter()
            .filter(|d| d.branch == *branch)
            .collect();
        nodes.sort_by_key(|d| d.tier);

        draw_header(ctx, &nodes, col_x, rect.y, col_w, color);

        let top = rect.y + 50.0;
        // Connectors first, so the node discs sit on top of the lines.
        for k in 0..nodes.len().saturating_sub(1) {
            let y0 = top + k as f32 * TIER_SPACING + RADIUS * 2.0;
            let y1 = top + (k + 1) as f32 * TIER_SPACING;
            draw_line(cx, y0, cx, y1, 2.0, theme::BORDER_DIM);
        }
        for (k, def) in nodes.iter().enumerate() {
            let cy = top + k as f32 * TIER_SPACING + RADIUS;
            draw_node(ctx, def, cx, cy, color, actions);
        }
    }
}

fn draw_header(
    ctx: &GameplayCtx<'_>,
    nodes: &[&PrestigeUpgradeDef],
    col_x: f32,
    y: f32,
    col_w: f32,
    color: Color,
) {
    let branch = nodes.first().map(|d| d.branch);
    let label = branch.map(|b| b.label()).unwrap_or("");
    let levels: u32 = nodes.iter().map(|d| ctx.state.prestige_level(&d.id)).sum();
    let max: u32 = nodes.iter().map(|d| d.max_level).sum();
    draw_text_centered_in_box(label, col_x, y, col_w, 22.0, 17.0, color);
    draw_text_centered_in_box(
        &format!("{levels} / {max}"),
        col_x,
        y + 22.0,
        col_w,
        18.0,
        13.0,
        theme::TEXT_DIM,
    );
}

fn draw_node(
    ctx: &GameplayCtx<'_>,
    def: &PrestigeUpgradeDef,
    cx: f32,
    cy: f32,
    color: Color,
    actions: &mut Vec<UiAction>,
) {
    let level = ctx.state.prestige_level(&def.id);
    let maxed = level >= def.max_level;
    let unlocked = ctx.state.prestige_prereq_met(def);
    let cost = economy::upgrade_cost(def.base_cost, def.cost_growth, level);
    let affordable = ctx.state.persistent.hoard_points >= cost;
    let clickable = unlocked && !maxed && affordable;
    let hovered = clickable && ctx.mouse.distance(vec2(cx, cy)) <= RADIUS;

    let fill = if !unlocked {
        theme::PANEL_DARK
    } else if maxed {
        color
    } else {
        // Dim the branch color for an unpurchased-but-available node.
        Color::new(color.r * 0.4, color.g * 0.4, color.b * 0.4, 1.0)
    };
    let border = if unlocked { color } else { theme::BORDER_DIM };
    draw_circle(cx, cy, RADIUS, fill);
    draw_circle_lines(cx, cy, RADIUS, if hovered { 3.5 } else { 2.0 }, border);

    let inner = if unlocked {
        theme::TEXT_BRIGHT
    } else {
        theme::TEXT_DIM
    };
    draw_text_centered_in_box(
        &format!("{level}/{}", def.max_level),
        cx - RADIUS,
        cy - 10.0,
        RADIUS * 2.0,
        20.0,
        14.0,
        inner,
    );

    // Name + cost/status beneath the disc.
    draw_text_centered_in_box(
        &def.name,
        cx - TIER_SPACING / 2.0,
        cy + RADIUS + 2.0,
        TIER_SPACING,
        16.0,
        12.0,
        if unlocked {
            theme::TEXT
        } else {
            theme::TEXT_DIM
        },
    );
    let status = if maxed {
        "MAX".to_owned()
    } else if !unlocked {
        "locked".to_owned()
    } else {
        format!("{} HP", format_amount(cost))
    };
    let status_color = if maxed {
        theme::POSITIVE
    } else if affordable && unlocked {
        theme::ACCENT
    } else {
        theme::TEXT_DIM
    };
    draw_text_centered_in_box(
        &status,
        cx - TIER_SPACING / 2.0,
        cy + RADIUS + 18.0,
        TIER_SPACING,
        16.0,
        12.0,
        status_color,
    );

    if hovered && is_mouse_button_released(MouseButton::Left) {
        actions.push(UiAction::BuyPrestigeUpgrade(def.id.clone()));
    }
}
