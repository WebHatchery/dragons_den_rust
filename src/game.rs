//! High-level game loop: owns the state machine, applies UI intents, and
//! handles saving, loading, and notifications.

use crate::data::GameData;
use crate::save;
use crate::simulation::idle_number::format_amount;
use crate::state::gameplay::{BuyError, ExploreResult, UnlockEvent};
use crate::state::{GameState, GameplayState, MenuState, StateTransition};
use crate::ui::{self, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::fx::FloatingTextLayer;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame};

/// Warm gold used for "+N" click gains.
const CLICK_GAIN_COLOR: Color = Color::new(0.98, 0.80, 0.35, 1.0);
/// Amethyst used for the prestige "burn" flourish.
const PRESTIGE_FLOURISH_COLOR: Color = Color::new(0.78, 0.63, 0.98, 1.0);

pub struct Game {
    data: GameData,
    state: GameState,
    notifications: NotificationManager,
    events: EventBus<UiAction>,
    /// Transient "+N" feedback (GDD §9.1); spawned in logical UI space.
    floating: FloatingTextLayer,
    /// Last cursor position in logical UI coords, captured each draw so click
    /// intents (applied a frame later in `update`) can anchor their feedback.
    last_mouse_logical: Vec2,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load()
            .unwrap_or_else(|err| panic!("Dragon's Den embedded data failed to load: {err}"));
        let state = GameState::Menu(MenuState::new(&data.config));
        let mut floating = FloatingTextLayer::new();
        floating.default_font_size = 24.0;
        floating.default_lifetime = 1.0;
        floating.default_rise_speed = 46.0;
        Self {
            data,
            state,
            notifications: NotificationManager::new(),
            events: EventBus::new(),
            floating,
            last_mouse_logical: Vec2::ZERO,
        }
    }

    /// Screenshot harness seeding: boots straight into a named scene.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.state = match scene {
            "menu" => GameState::Menu(MenuState::new(&self.data.config)),
            _ => GameState::Gameplay(Box::new(GameplayState::new_game(&self.data, 42))),
        };
    }

    pub fn update(&mut self, dt: f32) {
        self.notifications.update(dt);
        self.floating.update(dt);

        if let GameState::Gameplay(gameplay) = &mut self.state {
            gameplay.tick(&self.data, dt);

            for event in gameplay.check_unlocks(&self.data) {
                match event {
                    UnlockEvent::Achievement { name } => self
                        .notifications
                        .success(format!("Achievement unlocked: {name}")),
                    UnlockEvent::Dragon { name } => self
                        .notifications
                        .success(format!("Dragon revealed: {name}")),
                }
            }

            if gameplay.autosave_due(&self.data, dt) {
                self.write_save(false);
            }
            if is_key_pressed(KeyCode::Escape) {
                self.events.push(UiAction::BackToMenu);
            }
        }

        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
    }

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);

        let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        self.last_mouse_logical = virtual_ui.mouse_position();
        let actions = match &self.state {
            GameState::Menu(menu) => ui::menu::draw(&self.data, menu, &virtual_ui),
            GameState::Gameplay(gameplay) => ui::draw_gameplay(&self.data, gameplay, &virtual_ui),
        };
        self.floating.draw();
        end_virtual_ui_frame();

        for action in actions {
            self.events.push(action);
        }

        self.notifications
            .draw_with_config(&NotificationRenderConfig {
                anchor: NotificationAnchor::BottomRight,
                ..Default::default()
            });
    }

    fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::NewGame => self.transition(StateTransition::NewGame),
            UiAction::ContinueGame => self.transition(StateTransition::ContinueGame),
            UiAction::BackToMenu => self.transition(StateTransition::BackToMenu),
            UiAction::DeleteSave => self.delete_save(),
            UiAction::SaveNow => self.write_save(true),
            UiAction::SwitchScreen(screen) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    gameplay.screen = screen;
                }
            }
            UiAction::ClickHoard => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    let gained = gameplay.click(&self.data);
                    self.floating.spawn(
                        format!("+{}", format_amount(gained)),
                        self.last_mouse_logical,
                        CLICK_GAIN_COLOR,
                    );
                }
            }
            UiAction::HireGoblin => self.hire_goblin(),
            UiAction::Explore => self.explore(),
            UiAction::BuyUpgrade(id) => self.buy_upgrade(&id),
            UiAction::BuyPrestigeUpgrade(id) => self.buy_prestige_upgrade(&id),
            UiAction::Prestige => self.prestige(),
            UiAction::SetBuyMode(mode) => {
                if let GameState::Gameplay(gameplay) = &mut self.state {
                    gameplay.buy_mode = mode;
                }
            }
        }
    }

    fn transition(&mut self, transition: StateTransition) {
        match transition {
            StateTransition::NewGame => {
                let seed = save::now_timestamp().to_bits();
                self.state =
                    GameState::Gameplay(Box::new(GameplayState::new_game(&self.data, seed)));
                self.notifications.info("A new hoard begins.");
            }
            StateTransition::ContinueGame => match save::read_save(&self.data.config) {
                Ok(saved) => {
                    let (gameplay, offline_gold) =
                        GameplayState::from_save(&self.data, saved, save::now_timestamp());
                    if offline_gold >= 1.0 {
                        self.notifications.success(format!(
                            "While you were away, your goblins gathered {} gold",
                            format_amount(offline_gold)
                        ));
                    }
                    self.state = GameState::Gameplay(Box::new(gameplay));
                }
                Err(err) => self.notifications.warning(format!("Load failed: {err}")),
            },
            StateTransition::BackToMenu => {
                self.write_save(false);
                self.state = GameState::Menu(MenuState::new(&self.data.config));
            }
        }
    }

    fn write_save(&mut self, announce: bool) {
        let GameState::Gameplay(gameplay) = &self.state else {
            return;
        };
        match save::write_save(&self.data.config, &gameplay.run, &gameplay.persistent) {
            Ok(()) => {
                if announce {
                    self.notifications.success("Hoard saved");
                }
            }
            Err(err) => self.notifications.danger(format!("Save failed: {err}")),
        }
    }

    fn delete_save(&mut self) {
        match save::delete_save(&self.data.config) {
            Ok(()) => self.notifications.info("Save deleted"),
            Err(err) => self.notifications.danger(format!("Delete failed: {err}")),
        }
        if let GameState::Menu(menu) = &mut self.state {
            menu.refresh(&self.data.config);
        }
    }

    fn hire_goblin(&mut self) {
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        let requested = gameplay.buy_mode.requested();
        match gameplay.try_hire_bulk(&self.data, requested) {
            Ok((hired, cost)) => self.notifications.success(format!(
                "Hired {} goblin{} for {} gold ({} working)",
                hired,
                if hired == 1 { "" } else { "s" },
                format_amount(cost),
                gameplay.run.goblins
            )),
            Err(_) => self.notifications.warning("Not enough gold to hire"),
        }
    }

    fn explore(&mut self) {
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        match gameplay.try_explore(&self.data) {
            Ok(ExploreResult::Found { name, rarity }) => self
                .notifications
                .success(format!("Expedition found: {name} ({rarity})")),
            Ok(ExploreResult::NothingFound) => self
                .notifications
                .info("The expedition came back empty-clawed"),
            Ok(ExploreResult::AllDiscovered) => self
                .notifications
                .info("Every treasure is already in your hoard"),
            Err(_) => self
                .notifications
                .warning("Not enough gold for an expedition"),
        }
    }

    fn buy_upgrade(&mut self, id: &str) {
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        let name = self
            .data
            .upgrade(id)
            .map(|def| def.name.clone())
            .unwrap_or_else(|| id.to_owned());
        let requested = gameplay.buy_mode.requested();
        match gameplay.try_buy_upgrade_bulk(&self.data, id, requested) {
            Ok((bought, level)) => self.notifications.success(if bought == 1 {
                format!("{name} is now level {level}")
            } else {
                format!("{name} +{bought} → level {level}")
            }),
            Err(BuyError::MaxLevel) => self.notifications.info(format!("{name} is already maxed")),
            Err(_) => self.notifications.warning("Not enough gold"),
        }
    }

    fn buy_prestige_upgrade(&mut self, id: &str) {
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        let name = self
            .data
            .prestige_upgrade(id)
            .map(|def| def.name.clone())
            .unwrap_or_else(|| id.to_owned());
        match gameplay.try_buy_prestige_upgrade(&self.data, id) {
            Ok(level) => self
                .notifications
                .success(format!("{name} is now level {level} (permanent)")),
            Err(BuyError::MaxLevel) => self.notifications.info(format!("{name} is already maxed")),
            Err(_) => self.notifications.warning("Not enough Hoard Points"),
        }
    }

    fn prestige(&mut self) {
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        match gameplay.try_prestige(&self.data) {
            Some(gained) => {
                self.notifications.success(format!(
                    "The hoard burns! +{} Hoard Points",
                    format_amount(gained)
                ));
                self.floating.push(macroquad_toolkit::fx::FloatingText::new(
                    format!("+{} Hoard Points", format_amount(gained)),
                    vec2(ui::LOGICAL_WIDTH * 0.5 - 120.0, ui::LOGICAL_HEIGHT * 0.42),
                    PRESTIGE_FLOURISH_COLOR,
                    34.0,
                    1.8,
                    38.0,
                ));
                self.write_save(false);
            }
            None => self
                .notifications
                .warning("The hoard is not yet large enough to prestige"),
        }
    }
}
