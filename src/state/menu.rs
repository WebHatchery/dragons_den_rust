//! Main-menu state: continue/new/delete against the single autosave slot,
//! plus the global Settings sub-screen.

use crate::data::GameConfig;
use crate::save;

/// Which face of the menu is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuScreen {
    Main,
    Settings,
}

pub struct MenuState {
    pub save_exists: bool,
    pub screen: MenuScreen,
}

impl MenuState {
    pub fn new(config: &GameConfig) -> Self {
        Self {
            save_exists: save::save_exists(config),
            screen: MenuScreen::Main,
        }
    }

    pub fn refresh(&mut self, config: &GameConfig) {
        self.save_exists = save::save_exists(config);
    }
}
