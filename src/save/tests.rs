//! Current-save persistence tests. There is deliberately no legacy fixture:
//! the first public release accepts only the current versioned shape.

use super::*;
use crate::data::{GameConfig, GameData};
use crate::state::gameplay::GameplayState;
use macroquad_toolkit::persistence::{get_app_data_path, save_string_atomic};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SLOT_ID: AtomicU64 = AtomicU64::new(0);

struct TestSlot {
    config: GameConfig,
}

impl TestSlot {
    fn new(data: &GameData, label: &str) -> Self {
        let id = NEXT_SLOT_ID.fetch_add(1, Ordering::Relaxed);
        let mut config = data.config.clone();
        config.game_name = format!(
            "dragons_den_save_test_{}_{}_{}",
            std::process::id(),
            label,
            id
        );
        Self { config }
    }

    fn overwrite(&self, contents: &str) {
        let path = get_app_data_path(
            &self.config.game_name,
            &format!("save_{}.json", self.config.save_slot),
        )
        .expect("test save path");
        save_string_atomic(path, contents).expect("write test save bytes");
    }
}

impl Drop for TestSlot {
    fn drop(&mut self) {
        let _ = delete_save(&self.config);
        if let Some(path) = get_app_data_path(
            &self.config.game_name,
            &format!("save_{}.json", self.config.save_slot),
        ) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::remove_dir(parent);
            }
        }
    }
}

#[test]
fn current_save_round_trip_preserves_run_and_persistent_progression() {
    let data = GameData::load().unwrap();
    let slot = TestSlot::new(&data, "roundtrip");
    let mut state = GameplayState::new_game(&data, 0xD3A6_0001);
    state.run.gold = 9_876.5;
    state.run.goblins = 17;
    state.run.run_seconds = 321.25;
    state.run.upgrade_levels.insert("click_power".into(), 3);
    state.run.minion_counts.insert("shaman".into(), 2);
    state.persistent.hoard_points = 42.5;
    state.persistent.prestige_count = 4;
    state
        .persistent
        .prestige_upgrade_levels
        .insert("ancient_claws".into(), 2);
    state
        .persistent
        .discovered_treasures
        .push("sunken_crown".into());
    state.persistent.unlocked_dragons.push("ember_wyrm".into());
    state.persistent.stats.clicks_total = 123.0;

    write_save(&slot.config, &state.run, &state.persistent).unwrap();
    let loaded = read_save(&slot.config).unwrap();

    assert!(save_exists(&slot.config));
    assert_eq!(loaded.version, slot.config.version);
    assert!(loaded.timestamp > 0.0);
    assert_eq!(
        serde_json::to_value(&loaded.run).unwrap(),
        serde_json::to_value(&state.run).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.persistent).unwrap(),
        serde_json::to_value(&state.persistent).unwrap()
    );
}

#[test]
fn offline_earnings_use_the_timestamp_in_a_current_save() {
    let data = GameData::load().unwrap();
    let slot = TestSlot::new(&data, "offline");
    let mut state = GameplayState::new_game(&data, 0xD3A6_0002);
    state.run.gold = 125.0;
    state.run.goblins = 10;
    let expected_rate = state.gold_per_second(&data);
    let saved = SaveData {
        version: slot.config.version.clone(),
        timestamp: 1_000.0,
        run: state.run.clone(),
        persistent: state.persistent.clone(),
    };
    macroquad_toolkit::persistence::save_to_slot_with_version(
        &slot.config.game_name,
        &slot.config.save_slot,
        &saved,
        &slot.config.version,
    )
    .unwrap();

    let loaded = read_save(&slot.config).unwrap();
    let (restored, earned) = GameplayState::from_save(&data, loaded, 1_060.0);

    assert!((earned - expected_rate * 60.0).abs() < 1e-6);
    assert!((restored.run.gold - (125.0 + earned)).abs() < 1e-6);
}

#[test]
fn autosave_and_explicit_save_calls_keep_the_current_shape() {
    let data = GameData::load().unwrap();
    let slot = TestSlot::new(&data, "save_paths");
    let mut state = GameplayState::new_game(&data, 0xD3A6_0003);

    // Game::update and the visible SAVE action both reach this same writer;
    // only the caller's notification choice differs.
    state.run.gold = 10.0;
    write_save(&slot.config, &state.run, &state.persistent).unwrap();
    let autosave = read_save(&slot.config).unwrap();

    state.run.gold = 20.0;
    write_save(&slot.config, &state.run, &state.persistent).unwrap();
    let explicit = read_save(&slot.config).unwrap();

    assert_eq!(autosave.version, slot.config.version);
    assert_eq!(explicit.version, slot.config.version);
    assert_eq!(autosave.run.gold, 10.0);
    assert_eq!(explicit.run.gold, 20.0);
    assert_eq!(
        serde_json::to_value(&autosave.persistent).unwrap(),
        serde_json::to_value(&explicit.persistent).unwrap()
    );
}

#[test]
fn malformed_save_fails_with_a_source_clear_error() {
    let data = GameData::load().unwrap();
    let slot = TestSlot::new(&data, "malformed");
    slot.overwrite("{ this is not valid JSON");

    let error = read_save(&slot.config).unwrap_err();

    assert!(
        error.contains("JSON parse error"),
        "unexpected error: {error}"
    );
}

#[test]
fn unknown_save_version_is_rejected_without_a_legacy_migration() {
    let data = GameData::load().unwrap();
    let slot = TestSlot::new(&data, "version");
    let state = GameplayState::new_game(&data, 0xD3A6_0004);
    let saved = SaveData {
        version: slot.config.version.clone(),
        timestamp: 1_000.0,
        run: state.run,
        persistent: state.persistent,
    };
    macroquad_toolkit::persistence::save_to_slot_with_version(
        &slot.config.game_name,
        &slot.config.save_slot,
        &saved,
        "0.0.0-legacy",
    )
    .unwrap();

    let error = read_save(&slot.config).unwrap_err();

    assert!(
        error.contains("Unsupported save version"),
        "unexpected error: {error}"
    );
    assert!(error.contains("0.0.0-legacy"), "unexpected error: {error}");
}

#[test]
fn current_wrapper_with_incompatible_payload_fails_clearly() {
    let data = GameData::load().unwrap();
    let slot = TestSlot::new(&data, "payload");
    let state = GameplayState::new_game(&data, 0xD3A6_0005);
    let saved = SaveData {
        version: "0.0.0-legacy".into(),
        timestamp: 1_000.0,
        run: state.run,
        persistent: state.persistent,
    };
    macroquad_toolkit::persistence::save_to_slot_with_version(
        &slot.config.game_name,
        &slot.config.save_slot,
        &saved,
        &slot.config.version,
    )
    .unwrap();

    let error = read_save(&slot.config).unwrap_err();

    assert!(
        error.contains("Unsupported save version"),
        "unexpected error: {error}"
    );
    assert!(error.contains("0.0.0-legacy"), "unexpected error: {error}");
}
