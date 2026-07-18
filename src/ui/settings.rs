//! Global settings sub-screen (GDD §9): audio volumes, display flags, and the
//! autosave interval, all backed by the toolkit `GameSettings`. Pure view —
//! every control returns a `ChangeSetting` intent; `game.rs` mutates, applies,
//! and persists.

use crate::ui::{self, SettingChange, UiAction, VolumeChannel, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::settings::GameSettings;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub fn draw(settings: &GameSettings, mouse: Vec2) -> Vec<UiAction> {
    let mut actions = Vec::new();

    draw_text_centered_in_box(
        "Settings",
        0.0,
        LOGICAL_HEIGHT * 0.12,
        LOGICAL_WIDTH,
        50.0,
        44.0,
        dark::TEXT_BRIGHT,
    );

    let panel_w = 620.0;
    let panel = Rect::new(
        (LOGICAL_WIDTH - panel_w) / 2.0,
        LOGICAL_HEIGHT * 0.2,
        panel_w,
        432.0,
    );
    let content = ui::panel(panel, "Audio, Display & Gameplay");

    let mut y = content.y + 8.0;
    let row_h = 46.0;

    stepper_row(
        content,
        y,
        "Master Volume",
        &percent(settings.master_volume),
        VolumeChannel::Master,
        mouse,
        &mut actions,
    );
    y += row_h;
    stepper_row(
        content,
        y,
        "Sound Effects",
        &percent(settings.sfx_volume),
        VolumeChannel::Sfx,
        mouse,
        &mut actions,
    );
    y += row_h;
    stepper_row(
        content,
        y,
        "Music",
        &percent(settings.music_volume),
        VolumeChannel::Music,
        mouse,
        &mut actions,
    );
    y += row_h;

    // UI scale reuses the stepper visuals but its own intents.
    scale_row(content, y, settings.ui_text_scale, mouse, &mut actions);
    y += row_h;

    autosave_row(content, y, settings.autosave_interval, mouse, &mut actions);
    y += row_h;

    toggle_row(
        content,
        y,
        "Fullscreen",
        settings.fullscreen,
        SettingChange::ToggleFullscreen,
        mouse,
        &mut actions,
    );
    y += row_h;
    toggle_row(
        content,
        y,
        "Show FPS",
        settings.show_fps,
        SettingChange::ToggleShowFps,
        mouse,
        &mut actions,
    );

    draw_ui_text_ex(
        "Sound effects will play once audio packs ship; volumes are saved now.",
        content.x,
        content.bottom() - 6.0,
        TextStyle::new(13.0, dark::TEXT_DIM).params(),
    );

    let back = Rect::new(
        (LOGICAL_WIDTH - 220.0) / 2.0,
        panel.bottom() + 18.0,
        220.0,
        48.0,
    );
    if ui::button(back, "Back", true, ButtonTone::Secondary, mouse) {
        actions.push(UiAction::CloseSettings);
    }

    actions
}

fn percent(value: f32) -> String {
    format!("{}%", (value * 100.0).round() as i32)
}

fn label_and_value(content: Rect, y: f32, label: &str, value: &str) {
    draw_ui_text_ex(
        label,
        content.x + 4.0,
        y + 26.0,
        TextStyle::new(18.0, dark::TEXT).params(),
    );
    // Value sits just left of the +/- controls.
    draw_text_centered_in_box(
        value,
        content.right() - 176.0,
        y,
        88.0,
        36.0,
        18.0,
        dark::TEXT_BRIGHT,
    );
}

fn minus_plus(content: Rect, y: f32, mouse: Vec2) -> (bool, bool) {
    let minus = ui::button(
        Rect::new(content.right() - 84.0, y, 36.0, 36.0),
        "-",
        true,
        ButtonTone::Secondary,
        mouse,
    );
    let plus = ui::button(
        Rect::new(content.right() - 40.0, y, 36.0, 36.0),
        "+",
        true,
        ButtonTone::Secondary,
        mouse,
    );
    (minus, plus)
}

#[allow(clippy::too_many_arguments)]
fn stepper_row(
    content: Rect,
    y: f32,
    label: &str,
    value: &str,
    channel: VolumeChannel,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    label_and_value(content, y, label, value);
    let (minus, plus) = minus_plus(content, y, mouse);
    if minus {
        actions.push(UiAction::ChangeSetting(SettingChange::VolumeDown(channel)));
    }
    if plus {
        actions.push(UiAction::ChangeSetting(SettingChange::VolumeUp(channel)));
    }
}

fn scale_row(content: Rect, y: f32, scale: f32, mouse: Vec2, actions: &mut Vec<UiAction>) {
    label_and_value(content, y, "UI Text Scale", &format!("{scale:.2}x"));
    let (minus, plus) = minus_plus(content, y, mouse);
    if minus {
        actions.push(UiAction::ChangeSetting(SettingChange::UiScaleDown));
    }
    if plus {
        actions.push(UiAction::ChangeSetting(SettingChange::UiScaleUp));
    }
}

fn autosave_row(content: Rect, y: f32, interval: f32, mouse: Vec2, actions: &mut Vec<UiAction>) {
    label_and_value(
        content,
        y,
        "Autosave Every",
        &format!("{}s", interval.round() as i32),
    );
    let (minus, plus) = minus_plus(content, y, mouse);
    if minus {
        actions.push(UiAction::ChangeSetting(SettingChange::AutosaveDown));
    }
    if plus {
        actions.push(UiAction::ChangeSetting(SettingChange::AutosaveUp));
    }
}

fn toggle_row(
    content: Rect,
    y: f32,
    label: &str,
    on: bool,
    change: SettingChange,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_ui_text_ex(
        label,
        content.x + 4.0,
        y + 26.0,
        TextStyle::new(18.0, dark::TEXT).params(),
    );
    let tone = if on {
        ButtonTone::Positive
    } else {
        ButtonTone::Secondary
    };
    if ui::button(
        Rect::new(content.right() - 84.0, y, 80.0, 36.0),
        if on { "On" } else { "Off" },
        true,
        tone,
        mouse,
    ) {
        actions.push(UiAction::ChangeSetting(change));
    }
}
