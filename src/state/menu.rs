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
    /// A failed Continue attempt stays visible until the player starts over or
    /// deletes the damaged save, so recovery is not hidden in a short toast.
    pub load_error: Option<String>,
}

impl MenuState {
    pub fn new(config: &GameConfig) -> Self {
        Self {
            save_exists: save::save_exists(config),
            screen: MenuScreen::Main,
            load_error: None,
        }
    }

    pub fn refresh(&mut self, config: &GameConfig) {
        self.save_exists = save::save_exists(config);
        self.load_error = None;
    }

    pub fn record_load_error(&mut self, error: String) {
        self.load_error = Some(error);
    }
}

#[cfg(test)]
mod tests;
