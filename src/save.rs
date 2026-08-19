//! Save shape, wall-clock timestamps, and toolkit-persistence wrappers.
//!
//! The save stores a unix timestamp so offline progress (GDD §5.2) can be
//! computed on load from real elapsed time.

use crate::data::GameConfig;
use crate::state::gameplay::{PersistentState, RunState};
use macroquad_toolkit::persistence::{
    delete_slot, load_from_slot, peek_slot_version, save_to_slot_with_version, slot_exists,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    /// Unix seconds at save time, from [`now_timestamp`].
    pub timestamp: f64,
    pub run: RunState,
    pub persistent: PersistentState,
}

/// Unix seconds; works on both native and WASM (miniquad wraps `Date.now`).
pub fn now_timestamp() -> f64 {
    macroquad::miniquad::date::now()
}

pub fn save_exists(config: &GameConfig) -> bool {
    slot_exists(&config.game_name, &config.save_slot)
}

pub fn delete_save(config: &GameConfig) -> Result<(), String> {
    delete_slot(&config.game_name, &config.save_slot)
}

pub fn write_save(
    config: &GameConfig,
    run: &RunState,
    persistent: &PersistentState,
) -> Result<(), String> {
    let save = SaveData {
        version: config.version.clone(),
        timestamp: now_timestamp(),
        run: run.clone(),
        persistent: persistent.clone(),
    };
    save_to_slot_with_version(&config.game_name, &config.save_slot, &save, &config.version)
}

pub fn read_save(config: &GameConfig) -> Result<SaveData, String> {
    let detected_version = peek_slot_version(&config.game_name, &config.save_slot)?;
    if detected_version.as_deref() != Some(config.version.as_str()) {
        return Err(format_save_version_error(
            detected_version.as_deref(),
            &config.version,
        ));
    }

    let save: SaveData = load_from_slot(&config.game_name, &config.save_slot)
        .map_err(|err| format!("Current save could not be loaded: {err}"))?;
    validate_current_save(save, config)
}

/// The first release has one save shape and no migration history. Keep the
/// version check explicit so an old or hand-edited save never gets silently
/// reinterpreted as current data.
fn validate_current_save(save: SaveData, config: &GameConfig) -> Result<SaveData, String> {
    if save.version != config.version {
        return Err(format_save_version_error(
            Some(save.version.as_str()),
            &config.version,
        ));
    }

    Ok(save)
}

fn format_save_version_error(detected: Option<&str>, expected: &str) -> String {
    match detected {
        Some(version) => {
            format!("Unsupported save version '{version}'; expected current version '{expected}'")
        }
        None => format!("Save has no version metadata; expected current version '{expected}'"),
    }
}

#[cfg(test)]
mod tests;
