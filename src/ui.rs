//! UI root: the action enum, shared widgets, and gameplay screen chrome.
//! UI is a pure view layer — it reads state and returns `UiAction` intents;
//! `Game::apply_action` is the only place state changes.

pub mod achievements;
pub mod dragons;
pub mod frame;
pub mod hoard;
pub mod left_rail;
pub mod menu;
pub mod minions;
pub mod prestige;
pub mod settings;
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
    Explore,
    BuyUpgrade(String),
    BuyPrestigeUpgrade(String),
    Prestige,
    SetBuyMode(BuyMode),
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
    frame::draw_bottom_placeholder(regions.bottom);

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

    if state.settings_open {
        draw_settings_overlay(settings, mouse, &mut actions);
    }

    actions
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
        let tone = if active {
            ButtonTone::Primary
        } else {
            ButtonTone::Secondary
        };
        if button(rect, screen.label(), !active, tone, ctx.mouse) {
            actions.push(UiAction::SwitchScreen(*screen));
        }
        if active {
            draw_rectangle(rect.x, rect.bottom() - 3.0, rect.w, 3.0, dark::ACCENT);
        }
    }
}

// --- shared widgets ------------------------------------------------------

/// Standard panel chrome; returns the inset content rect.
pub(crate) fn panel(rect: Rect, title: &str) -> Rect {
    let style = SurfaceStyle::new(Color::new(0.08, 0.085, 0.105, 0.97))
        .with_border(1.0, Color::new(0.38, 0.45, 0.58, 0.65))
        .with_header(42.0, Color::new(0.105, 0.12, 0.15, 1.0))
        .with_header_divider(1.0, Color::new(0.38, 0.45, 0.58, 0.4));
    draw_surface_with_title(rect, Some(title), &style, TextStyle::new(18.0, dark::TEXT));
    Rect::new(rect.x + 18.0, rect.y + 56.0, rect.w - 36.0, rect.h - 74.0)
}

pub(crate) fn button(rect: Rect, text: &str, enabled: bool, tone: ButtonTone, mouse: Vec2) -> bool {
    let style = ButtonStyle::from_tone(tone);
    let hovered = enabled && rect.contains_point(mouse);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let activated = hovered && is_mouse_button_released(MouseButton::Left);
    let fill = if !enabled {
        style.disabled
    } else if pressed {
        style.pressed
    } else if hovered {
        style.hovered
    } else {
        style.normal
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(fill).with_border(1.0, style.border),
    );
    draw_text_centered_in_box_ex(
        text,
        rect.x + 8.0,
        rect.y + if pressed { 2.0 } else { 0.0 },
        rect.w - 16.0,
        rect.h,
        TextStyle::new(
            17.0,
            if enabled {
                style.text_color
            } else {
                dark::TEXT_DIM
            },
        ),
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
