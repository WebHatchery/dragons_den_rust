//! Main menu: continue, new game, delete save, and the settings sub-screen.

use crate::data::GameData;
use crate::state::{MenuScreen, MenuState};
use crate::ui::{self, settings, UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::VirtualUi;

pub fn draw(
    data: &GameData,
    menu: &MenuState,
    game_settings: &GameSettings,
    ui: &VirtualUi,
) -> Vec<UiAction> {
    let mouse = ui.mouse_position();
    if menu.screen == MenuScreen::Settings {
        return settings::draw(game_settings, mouse);
    }

    let mut actions = Vec::new();

    draw_text_centered_in_box(
        &data.config.display_name,
        0.0,
        LOGICAL_HEIGHT * 0.22,
        LOGICAL_WIDTH,
        60.0,
        56.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_centered_in_box(
        "Click for gold. Hoard it. Burn it all down for something permanent.",
        0.0,
        LOGICAL_HEIGHT * 0.22 + 66.0,
        LOGICAL_WIDTH,
        30.0,
        19.0,
        dark::TEXT_DIM,
    );

    let button_w = 320.0;
    let x = (LOGICAL_WIDTH - button_w) / 2.0;
    let mut y = LOGICAL_HEIGHT * 0.45;

    if ui::button(
        Rect::new(x, y, button_w, 52.0),
        "Continue",
        menu.save_exists,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::ContinueGame);
    }
    y += 64.0;
    if ui::button(
        Rect::new(x, y, button_w, 52.0),
        "New Game",
        true,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::NewGame);
    }
    y += 64.0;
    if ui::button(
        Rect::new(x, y, button_w, 44.0),
        "Delete Save",
        menu.save_exists,
        ButtonTone::Danger,
        mouse,
    ) {
        actions.push(UiAction::DeleteSave);
    }
    y += 56.0;
    if ui::button(
        Rect::new(x, y, button_w, 44.0),
        "Settings",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenSettings);
    }

    draw_text_centered_in_box(
        "v0.1 framework — see IMPLEMENTATION_PLAN.md",
        0.0,
        LOGICAL_HEIGHT - 40.0,
        LOGICAL_WIDTH,
        24.0,
        14.0,
        dark::TEXT_DIM,
    );

    actions
}
