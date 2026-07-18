//! Game state machine: exactly one state is active, transitions are explicit
//! values returned from `update()` and applied by `Game::transition()`.

pub mod gameplay;
pub mod menu;

pub use gameplay::GameplayState;
pub use menu::MenuState;

pub enum GameState {
    Menu(MenuState),
    Gameplay(Box<GameplayState>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateTransition {
    /// Start a fresh save (overwrites nothing until the first autosave).
    NewGame,
    /// Load the existing save slot and apply offline progress.
    ContinueGame,
    /// Save and return to the main menu.
    BackToMenu,
}
