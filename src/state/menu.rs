//! Main-menu state: continue/new/delete against the single autosave slot.

use crate::data::GameConfig;
use crate::save;

pub struct MenuState {
    pub save_exists: bool,
}

impl MenuState {
    pub fn new(config: &GameConfig) -> Self {
        Self {
            save_exists: save::save_exists(config),
        }
    }

    pub fn refresh(&mut self, config: &GameConfig) {
        self.save_exists = save::save_exists(config);
    }
}
