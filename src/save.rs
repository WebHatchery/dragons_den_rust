//! Save shape, wall-clock timestamps, and toolkit-persistence wrappers.
//!
//! The save stores a unix timestamp so offline progress (GDD §5.2) can be
//! computed on load from real elapsed time.

use crate::data::GameConfig;
use crate::state::gameplay::{PersistentState, RunState};
use macroquad_toolkit::persistence::{
    delete_slot, load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    load_from_slot_with_migration(
        &config.game_name,
        &config.save_slot,
        &config.version,
        |version, value| migrate_save_value(version, value, config),
    )
}

/// Migration hook: currently only re-stamps the version on the modern shape.
/// Older shapes have no shipped players yet, so anything unrecognized fails
/// loudly instead of guessing.
fn migrate_save_value(
    detected_version: Option<String>,
    value: Value,
    config: &GameConfig,
) -> Result<SaveData, String> {
    let payload = value.get("data").cloned().unwrap_or(value);
    match serde_json::from_value::<SaveData>(payload) {
        Ok(mut save) => {
            save.version = config.version.clone();
            Ok(save)
        }
        Err(err) => Err(format!(
            "Unsupported save format {:?}: {}",
            detected_version, err
        )),
    }
}
