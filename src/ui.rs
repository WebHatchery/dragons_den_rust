//! UI root: the action enum, shared widgets, and gameplay screen chrome.
//! UI is a pure view layer — it reads state and returns `UiAction` intents;
//! `Game::apply_action` is the only place state changes.

pub mod achievements;
pub mod bottom_bar;
pub mod dragons;
pub mod frame;
pub mod hoard;
pub mod icons;
pub mod left_rail;
pub mod menu;
pub mod minions;
pub mod prestige;
pub mod prestige_tree;
pub mod settings;
pub mod theme;
pub mod treasures;
pub mod upgrades;

use crate::data::{EffectStat, GameData, PercentEffect, StatCondition, StatKey};
use crate::simulation::economy;
use crate::simulation::idle_number::format_amount;
use crate::state::gameplay::{BuyMode, GameplayState, Screen};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::{RectExt, VirtualUi};

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    // Menu
    NewGame,
    ContinueGame,
    DeleteSave,
    // Gameplay
    BackToMenu,
    SaveNow,
    SwitchScreen(Screen),
    ClickHoard,
    HireGoblin,
    /// Hire an extra minion tier by `MinionDef::id` (P4).
    HireMinion(String),
    Explore,
    BuyUpgrade(String),
    BuyPrestigeUpgrade(String),
    Prestige,
    SetBuyMode(BuyMode),
    /// The player clicked an active Golden Hoard glint (#9).
    CollectGoldenHoard,
    // Settings
    OpenSettings,
    CloseSettings,
    ChangeSetting(SettingChange),
    /// New clamped vertical scroll offset for the active list screen.
    SetScroll(f32),
}

/// Which audio group a volume change targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeChannel {
    Master,
    Sfx,
    Music,
}

/// A single settings edit, resolved to a concrete delta/toggle in `game.rs`
/// (step sizes live there). Payload-free so `UiAction` stays `Eq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingChange {
    VolumeUp(VolumeChannel),
    VolumeDown(VolumeChannel),
    UiScaleUp,
    UiScaleDown,
    AutosaveUp,
    AutosaveDown,
    ToggleFullscreen,
    ToggleShowFps,
}

/// Read-only view context handed to every gameplay screen.
pub struct GameplayCtx<'a> {
    pub data: &'a GameData,
    pub state: &'a GameplayState,
    pub mouse: Vec2,
}

pub fn draw_gameplay(
    data: &GameData,
    state: &GameplayState,
    settings: &GameSettings,
    ui: &VirtualUi,
) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let mouse = ui.mouse_position();
    let ctx = GameplayCtx { data, state, mouse };

    let regions = frame::regions();
    frame::draw_header(&ctx, regions.header, &mut actions);
    draw_tab_bar(&ctx, regions.tabs, &mut actions);
    left_rail::draw(&ctx, regions.left_rail, &mut actions);
    bottom_bar::draw(&ctx, regions.bottom, &mut actions);

    let content = regions.center;
    match state.screen {
        Screen::Hoard => hoard::draw(&ctx, content, &mut actions),
        Screen::Minions => minions::draw(&ctx, content, &mut actions),
        Screen::Upgrades => upgrades::draw(&ctx, content, &mut actions),
        Screen::Treasures => treasures::draw(&ctx, content, &mut actions),
        Screen::Achievements => achievements::draw(&ctx, content, &mut actions),
        Screen::Prestige => prestige::draw(&ctx, content, &mut actions),
        Screen::Dragons => dragons::draw(&ctx, content, &mut actions),
    }

    // Active-play overlays (#9): a Golden Hoard glint and the Frenzy banner sit
    // above the panels but below the settings modal.
    draw_frenzy_banner(&ctx, regions.center);
    if !state.settings_open {
        draw_golden_hoard(&ctx, regions.center, &mut actions);
    }

    if state.settings_open {
        draw_settings_overlay(settings, mouse, &mut actions);
    }

    actions
}

/// Draws the active Golden Hoard glint (#9) and pushes a collect action when the
/// player clicks it. `area` is the central play region the glint floats within.
fn draw_golden_hoard(ctx: &GameplayCtx<'_>, area: Rect, actions: &mut Vec<UiAction>) {
    let Some(glint) = ctx.state.golden_hoard() else {
        return;
    };
    const R: f32 = 26.0;
    let pad = R + 8.0;
    let cx = area.x + pad + glint.nx * (area.w - pad * 2.0);
    let cy = area.y + pad + glint.ny * (area.h - pad * 2.0);
    let hovered = vec2(cx, cy).distance(ctx.mouse) <= R;

    // A warm golden orb that fades and shrinks its halo as the window closes.
    let fade = (glint.remaining / 1.5).clamp(0.35, 1.0) as f32;
    let scale = if hovered { 1.15 } else { 1.0 };
    draw_circle(cx, cy, R * 1.7, Color::new(0.98, 0.80, 0.30, 0.14 * fade));
    draw_circle(cx, cy, R * scale, Color::new(0.99, 0.84, 0.38, fade));
    draw_circle(
        cx - R * 0.3,
        cy - R * 0.3,
        R * 0.38,
        Color::new(1.0, 0.97, 0.82, fade),
    );
    draw_circle_lines(cx, cy, R * scale, 2.0, Color::new(1.0, 0.93, 0.66, fade));

    if hovered && is_mouse_button_released(MouseButton::Left) {
        actions.push(UiAction::CollectGoldenHoard);
    }
}

/// A prominent banner while Dragon's Frenzy is active (#9), centered at the top
/// of the play area.
fn draw_frenzy_banner(ctx: &GameplayCtx<'_>, area: Rect) {
    if !ctx.state.frenzy_active() {
        return;
    }
    let w = 380.0;
    let h = 30.0;
    let rect = Rect::new(area.x + (area.w - w) / 2.0, area.y + 4.0, w, h);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.30, 0.16, 0.04, 0.92)).with_border(1.5, theme::ACCENT),
    );
    draw_text_centered_in_box(
        &format!(
            "DRAGON'S FRENZY!  x{:.0} click  ·  {:.0}s",
            ctx.data.config.dragon_frenzy_multiplier,
            ctx.state.frenzy_remaining().ceil()
        ),
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        16.0,
        theme::ACCENT,
    );
}

/// Dims the frame and draws the shared settings panel as a gameplay overlay.
fn draw_settings_overlay(settings: &GameSettings, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.72),
    );
    actions.extend(settings::draw(settings, mouse));
}

fn draw_tab_bar(ctx: &GameplayCtx<'_>, bar: Rect, actions: &mut Vec<UiAction>) {
    let tab_w = (bar.w - (Screen::ALL.len() as f32 - 1.0) * 8.0) / Screen::ALL.len() as f32;
    for (index, screen) in Screen::ALL.iter().enumerate() {
        let rect = Rect::new(bar.x + index as f32 * (tab_w + 8.0), bar.y, tab_w, bar.h);
        let active = *screen == ctx.state.screen;
        if active {
            // The active tab reads as a lit, gold-topped panel with an underline.
            draw_surface(
                rect,
                &SurfaceStyle::new(shade(theme::PANEL_HEADER, 1.5))
                    .with_border(1.0, theme::BORDER)
                    .with_top_highlight(2.0, theme::ACCENT),
            );
            draw_text_centered_in_box_ex(
                screen.label(),
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                TextStyle::new(17.0, theme::TEXT_BRIGHT),
            );
            draw_rectangle(rect.x, rect.bottom() - 3.0, rect.w, 3.0, theme::ACCENT);
        } else if button(rect, screen.label(), true, ButtonTone::Secondary, ctx.mouse) {
            actions.push(UiAction::SwitchScreen(*screen));
        }
    }
}

// --- shared widgets ------------------------------------------------------

/// Standard panel chrome; returns the inset content rect.
pub(crate) fn panel(rect: Rect, title: &str) -> Rect {
    let style = SurfaceStyle::new(theme::PANEL)
        .with_border(1.0, theme::BORDER)
        .with_header(42.0, theme::PANEL_HEADER)
        .with_header_divider(1.0, theme::BORDER_DIM);
    draw_surface_with_title(
        rect,
        Some(title),
        &style,
        TextStyle::new(18.0, theme::TEXT_BRIGHT),
    );
    // Etched depth: a thin inset bronze line plus gold corner brackets.
    draw_rectangle_lines(
        rect.x + 3.0,
        rect.y + 3.0,
        rect.w - 6.0,
        rect.h - 6.0,
        1.0,
        Color::new(
            theme::BORDER_DIM.r,
            theme::BORDER_DIM.g,
            theme::BORDER_DIM.b,
            0.6,
        ),
    );
    theme::draw_corner_marks(rect, theme::BORDER);
    Rect::new(rect.x + 18.0, rect.y + 56.0, rect.w - 36.0, rect.h - 74.0)
}

/// Base (fill, border, text) for a warm-themed button tone.
fn tone_colors(tone: ButtonTone) -> (Color, Color, Color) {
    match tone {
        ButtonTone::Primary => (
            Color::new(0.62, 0.46, 0.20, 1.0),
            theme::BORDER,
            Color::new(0.08, 0.06, 0.03, 1.0),
        ),
        ButtonTone::Positive => (
            Color::new(0.26, 0.34, 0.16, 1.0),
            theme::POSITIVE,
            theme::TEXT_BRIGHT,
        ),
        ButtonTone::Danger => (
            Color::new(0.30, 0.20, 0.42, 1.0),
            theme::HOARD_POINT,
            theme::TEXT_BRIGHT,
        ),
        // Secondary and any future tones: a bronze-outlined dark button.
        _ => (
            Color::new(0.16, 0.12, 0.07, 1.0),
            theme::BORDER_DIM,
            theme::ACCENT,
        ),
    }
}

/// Multiplies a color's RGB by `f` (for hover-brighten / press-darken).
fn shade(c: Color, f: f32) -> Color {
    Color::new(
        (c.r * f).clamp(0.0, 1.0),
        (c.g * f).clamp(0.0, 1.0),
        (c.b * f).clamp(0.0, 1.0),
        c.a,
    )
}

pub(crate) fn button(rect: Rect, text: &str, enabled: bool, tone: ButtonTone, mouse: Vec2) -> bool {
    let (base, border, text_color) = tone_colors(tone);
    let hovered = enabled && rect.contains_point(mouse);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let activated = hovered && is_mouse_button_released(MouseButton::Left);
    let fill = if !enabled {
        theme::PANEL_DARK
    } else if pressed {
        shade(base, 0.82)
    } else if hovered {
        shade(base, 1.18)
    } else {
        base
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(fill).with_border(1.0, if enabled { border } else { theme::BORDER_DIM }),
    );
    draw_text_centered_in_box_ex(
        text,
        rect.x + 8.0,
        rect.y + if pressed { 2.0 } else { 0.0 },
        rect.w - 16.0,
        rect.h,
        TextStyle::new(17.0, if enabled { text_color } else { theme::TEXT_DIM }),
    );
    activated
}

pub(crate) fn stat_label(stat: EffectStat) -> &'static str {
    match stat {
        EffectStat::GoldPerClick => "gold per click",
        EffectStat::GoldPerSecond => "gold per second",
        EffectStat::MinionEfficiency => "minion efficiency",
        EffectStat::DiscoveryChance => "discovery chance",
        EffectStat::HoardPointGain => "Hoard Point gain",
        EffectStat::HireDiscount => "hire discount",
        EffectStat::AllGold => "all gold",
    }
}

pub(crate) fn effect_text(effect: &PercentEffect) -> String {
    format!("+{}% {}", effect.percent, stat_label(effect.stat))
}

/// What a bulk purchase would cost right now for the current [`BuyMode`],
/// shared by the hire and upgrade screens so their affordances match.
pub(crate) struct BulkQuote {
    /// Levels this purchase would grant (0 when nothing is affordable).
    pub count: u32,
    /// Gold required; for an unaffordable `Max` this is the next single level.
    pub cost: f64,
    pub affordable: bool,
}

/// Resolves a bulk quote against the `floor(base * growth^level)` curve.
/// `remaining` caps how many levels are left (`u32::MAX` for the uncapped
/// goblin hire). `x1`/`x10` require the full requested amount to be affordable;
/// `Max` buys as many as gold allows.
pub(crate) fn bulk_quote(
    base_cost: f64,
    cost_growth: f64,
    level: u32,
    remaining: u32,
    gold: f64,
    mode: BuyMode,
) -> BulkQuote {
    let requested = mode.requested().min(remaining);
    if requested == 0 {
        return BulkQuote {
            count: 0,
            cost: 0.0,
            affordable: false,
        };
    }
    match mode {
        BuyMode::Max => {
            let (count, cost) =
                economy::affordable_levels(base_cost, cost_growth, level, gold, requested);
            if count == 0 {
                BulkQuote {
                    count: 0,
                    cost: economy::upgrade_cost(base_cost, cost_growth, level),
                    affordable: false,
                }
            } else {
                BulkQuote {
                    count,
                    cost,
                    affordable: true,
                }
            }
        }
        BuyMode::One | BuyMode::Ten => {
            let cost = economy::bulk_cost(base_cost, cost_growth, level, requested);
            BulkQuote {
                count: requested,
                cost,
                affordable: gold >= cost,
            }
        }
    }
}

/// Reads the mouse wheel and returns the clamped scroll offset to render a list
/// of `content_height` inside `view`. Emits `SetScroll` only when it changes so
/// the value persists across frames. macroquad has no scissor, so callers cull
/// items whose card isn't fully inside `view` (see [`item_fully_visible`]).
pub(crate) fn apply_scroll(
    current: f32,
    content_height: f32,
    view: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) -> f32 {
    const SCROLL_SPEED: f32 = 48.0;
    let max = (content_height - view.h).max(0.0);
    let (_, wheel) = mouse_wheel();
    let mut scroll = current;
    if wheel != 0.0 && view.contains_point(mouse) {
        scroll -= wheel * SCROLL_SPEED;
    }
    scroll = scroll.clamp(0.0, max);
    if (scroll - current).abs() > f32::EPSILON {
        actions.push(UiAction::SetScroll(scroll));
    }
    scroll
}

/// True when a scrolled card sits fully within the view (used to cull the
/// partially-clipped rows at the top/bottom, keeping panel edges clean).
pub(crate) fn item_fully_visible(card: Rect, view: Rect) -> bool {
    card.y >= view.y - 0.5 && card.bottom() <= view.bottom() + 0.5
}

/// Draws a thin scrollbar down the right edge of `view` so the player can tell
/// a wheel-scrollable list has more content below/above. No-op when everything
/// already fits. Pairs with [`apply_scroll`]: pass the same `content_height`,
/// `view`, and clamped `scroll`.
pub(crate) fn draw_scroll_indicator(view: Rect, content_height: f32, scroll: f32) {
    let overflow = content_height - view.h;
    if overflow <= 0.5 {
        return;
    }
    const BAR_W: f32 = 5.0;
    let track_x = view.right() - BAR_W - 2.0;
    // Track: a faint groove hinting the full scroll range.
    draw_rectangle(
        track_x,
        view.y,
        BAR_W,
        view.h,
        Color::new(
            theme::BORDER_DIM.r,
            theme::BORDER_DIM.g,
            theme::BORDER_DIM.b,
            0.35,
        ),
    );
    // Handle: proportional to the visible fraction, positioned by scroll.
    let handle_h = (view.h * (view.h / content_height)).max(28.0).min(view.h);
    let handle_y = view.y + (scroll / overflow) * (view.h - handle_h);
    draw_rectangle(
        track_x,
        handle_y,
        BAR_W,
        handle_h,
        Color::new(theme::ACCENT.r, theme::ACCENT.g, theme::ACCENT.b, 0.85),
    );
}

/// A small `x1 / x10 / Max` segmented selector; pushes `SetBuyMode` on change.
pub(crate) fn buy_mode_selector(
    rect: Rect,
    current: BuyMode,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let gap = 6.0;
    let seg_w = (rect.w - gap * (BuyMode::ALL.len() as f32 - 1.0)) / BuyMode::ALL.len() as f32;
    for (index, mode) in BuyMode::ALL.iter().enumerate() {
        let seg = Rect::new(rect.x + index as f32 * (seg_w + gap), rect.y, seg_w, rect.h);
        let active = *mode == current;
        let tone = if active {
            ButtonTone::Primary
        } else {
            ButtonTone::Secondary
        };
        if button(seg, mode.label(), !active, tone, mouse) {
            actions.push(UiAction::SetBuyMode(*mode));
        }
    }
}

pub(crate) fn condition_text(condition: &StatCondition, data: &GameData) -> String {
    let minions_working = format!(
        "{} working at once",
        data.config.minion_name_plural.to_lowercase()
    );
    let noun = match condition.stat {
        StatKey::ClicksTotal => "total clicks",
        StatKey::GoldTotalEarned => "total gold earned",
        StatKey::Goblins => &minions_working,
        StatKey::TreasuresDiscovered => "treasures discovered",
        StatKey::UpgradesPurchased => "upgrades purchased",
        StatKey::PrestigeCount => "prestiges",
        StatKey::AchievementsUnlocked => "achievements unlocked",
    };
    format!("Reach {} {}", format_amount(condition.gte), noun)
}
