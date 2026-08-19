use super::*;
use crate::data::GameData;

#[test]
fn load_error_is_persistent_recovery_state_without_hiding_new_game() {
    let data = GameData::load().unwrap();
    let mut config = data.config.clone();
    config.game_name = format!("dragons_den_menu_test_{}", std::process::id());
    let mut menu = MenuState::new(&config);

    assert!(!menu.save_exists);
    menu.record_load_error("Save could not be loaded: incompatible version".into());

    assert_eq!(menu.screen, MenuScreen::Main);
    assert!(menu.load_error.is_some());
    assert!(!menu.save_exists);
}

#[test]
fn refreshing_after_delete_clears_the_load_error() {
    let data = GameData::load().unwrap();
    let mut config = data.config.clone();
    config.game_name = format!("dragons_den_menu_refresh_test_{}", std::process::id());
    let mut menu = MenuState::new(&config);
    menu.record_load_error("bad save".into());

    menu.refresh(&config);

    assert!(menu.load_error.is_none());
}
