//! The single active-save state (GDD §11): every UI panel reads from this
//! one struct — there is deliberately no second store to drift out of sync.

use crate::data::{AchievementDef, GameData, StatKey};
use crate::save::SaveData;
use crate::simulation::economy;
use crate::simulation::exploration::{self, ExploreOutcome};
use crate::simulation::offline;
use crate::simulation::prestige;
use crate::simulation::Bonuses;
use macroquad_toolkit::achievements::{Achievement, Achievements};
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Gameplay sub-views, matching the GDD §9 tab list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Hoard,
    Minions,
    Upgrades,
    Treasures,
    Achievements,
    Prestige,
    Dragons,
}

/// How many levels a hire/upgrade button buys at once (GDD §9 QoL). Transient
/// UI preference — not part of the save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuyMode {
    One,
    Ten,
    Max,
}

impl BuyMode {
    pub const ALL: [BuyMode; 3] = [BuyMode::One, BuyMode::Ten, BuyMode::Max];

    pub fn label(self) -> &'static str {
        match self {
            BuyMode::One => "x1",
            BuyMode::Ten => "x10",
            BuyMode::Max => "Max",
        }
    }

    /// The requested level count; `Max` asks for everything affordable.
    pub fn requested(self) -> u32 {
        match self {
            BuyMode::One => 1,
            BuyMode::Ten => 10,
            BuyMode::Max => u32::MAX,
        }
    }
}

/// A capped ring of recent player-facing events shown on the Hoard screen
/// (GDD §9) to complement the transient toasts. Newest is pushed last;
/// transient — not part of the save.
#[derive(Debug, Clone, Default)]
pub struct ActionLog {
    entries: VecDeque<String>,
}

impl ActionLog {
    const CAP: usize = 8;

    pub fn push(&mut self, message: impl Into<String>) {
        self.entries.push_back(message.into());
        while self.entries.len() > Self::CAP {
            self.entries.pop_front();
        }
    }

    /// Entries oldest-first; callers reverse for newest-first display.
    pub fn entries(&self) -> impl DoubleEndedIterator<Item = &String> {
        self.entries.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Screen {
    pub const ALL: [Screen; 7] = [
        Screen::Hoard,
        Screen::Minions,
        Screen::Upgrades,
        Screen::Treasures,
        Screen::Achievements,
        Screen::Prestige,
        Screen::Dragons,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Screen::Hoard => "Hoard",
            Screen::Minions => "Minions",
            Screen::Upgrades => "Upgrades",
            Screen::Treasures => "Treasures",
            Screen::Achievements => "Achievements",
            Screen::Prestige => "Prestige",
            Screen::Dragons => "Codex",
        }
    }
}

/// Run-scoped economy — reset on prestige (GDD §5.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub gold: f64,
    pub goblins: u32,
    pub upgrade_levels: HashMap<String, u32>,
}

impl RunState {
    pub fn new() -> Self {
        Self {
            gold: 0.0,
            goblins: 0,
            upgrade_levels: HashMap::new(),
        }
    }
}

/// Lifetime counters that survive prestige, referenced by unlock conditions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LifetimeStats {
    pub clicks_total: f64,
    pub gold_total_earned: f64,
    pub upgrades_purchased: f64,
}

/// Everything preserved across prestige resets (GDD §5.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub hoard_points: f64,
    pub prestige_count: u32,
    pub prestige_upgrade_levels: HashMap<String, u32>,
    pub discovered_treasures: Vec<String>,
    pub unlocked_dragons: Vec<String>,
    pub achievements: Achievements,
    pub stats: LifetimeStats,
    pub rng: SeededRng,
}

impl PersistentState {
    fn new(data: &GameData, seed: u64) -> Self {
        Self {
            hoard_points: 0.0,
            prestige_count: 0,
            prestige_upgrade_levels: HashMap::new(),
            discovered_treasures: Vec::new(),
            unlocked_dragons: Vec::new(),
            achievements: Achievements::from_definitions(achievement_definitions(
                &data.achievements,
            )),
            stats: LifetimeStats::default(),
            rng: SeededRng::new(seed),
        }
    }
}

fn achievement_definitions(defs: &[AchievementDef]) -> Vec<Achievement> {
    defs.iter()
        .map(|def| Achievement::new(&def.id, &def.name, &def.description))
        .collect()
}

/// Something newly unlocked this frame, for toast notifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlockEvent {
    Achievement { name: String },
    Dragon { name: String },
}

/// Result of an expedition, with owned strings so the UI/notification layer
/// never borrows into the treasure catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExploreResult {
    NothingFound,
    Found { name: String, rarity: &'static str },
    AllDiscovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuyError {
    UnknownId,
    MaxLevel,
    CannotAfford,
}

pub struct GameplayState {
    pub run: RunState,
    pub persistent: PersistentState,
    pub screen: Screen,
    pub buy_mode: BuyMode,
    pub action_log: ActionLog,
    autosave_accum: f32,
}

impl GameplayState {
    pub fn new_game(data: &GameData, seed: u64) -> Self {
        Self {
            run: RunState::new(),
            persistent: PersistentState::new(data, seed),
            screen: Screen::Hoard,
            buy_mode: BuyMode::One,
            action_log: ActionLog::default(),
            autosave_accum: 0.0,
        }
    }

    /// Restores a save and applies offline earnings (GDD §5.2) at the same
    /// rate the live tick uses. Returns the gold earned while away.
    pub fn from_save(data: &GameData, save: SaveData, now: f64) -> (Self, f64) {
        let mut state = Self {
            run: save.run,
            persistent: save.persistent,
            screen: Screen::Hoard,
            buy_mode: BuyMode::One,
            action_log: ActionLog::default(),
            autosave_accum: 0.0,
        };
        state
            .persistent
            .achievements
            .sync_definitions(achievement_definitions(&data.achievements));

        let earned = offline::offline_gold(state.gold_per_second(data), now - save.timestamp);
        state.earn(earned);
        (state, earned)
    }

    // --- derived economy -------------------------------------------------

    fn rates(&self, data: &GameData) -> Bonuses {
        economy::rate_bonuses(&data.upgrades, &self.run.upgrade_levels)
    }

    /// Percent bonuses from every persistent source: prestige tree,
    /// discovered treasures, and unlocked codex dragons.
    fn percents(&self, data: &GameData) -> Bonuses {
        let mut bonuses = economy::prestige_percent_bonuses(
            &data.prestige_upgrades,
            &self.persistent.prestige_upgrade_levels,
        );
        for id in &self.persistent.discovered_treasures {
            if let Some(def) = data.treasure(id) {
                bonuses.add(def.effect.stat, def.effect.percent);
            }
        }
        for id in &self.persistent.unlocked_dragons {
            if let Some(def) = data.dragons.iter().find(|d| &d.id == id) {
                bonuses.add(def.effect.stat, def.effect.percent);
            }
        }
        bonuses
    }

    pub fn gold_per_click(&self, data: &GameData) -> f64 {
        economy::gold_per_click(&data.config, &self.rates(data), &self.percents(data))
    }

    pub fn gold_per_second(&self, data: &GameData) -> f64 {
        economy::gold_per_second(
            &data.config,
            self.run.goblins,
            &self.rates(data),
            &self.percents(data),
        )
    }

    pub fn discovery_chance(&self, data: &GameData) -> f64 {
        economy::discovery_chance(&data.config, &self.rates(data), &self.percents(data))
    }

    // --- actions ---------------------------------------------------------

    fn earn(&mut self, amount: f64) {
        self.run.gold += amount;
        self.persistent.stats.gold_total_earned += amount;
    }

    /// Passive income for one frame.
    pub fn tick(&mut self, data: &GameData, dt: f32) {
        self.earn(self.gold_per_second(data) * f64::from(dt));
    }

    /// Counts up the autosave interval; true means "save now".
    pub fn autosave_due(&mut self, data: &GameData, dt: f32) -> bool {
        self.autosave_accum += dt;
        if self.autosave_accum >= data.config.autosave_interval {
            self.autosave_accum = 0.0;
            true
        } else {
            false
        }
    }

    /// Click the hoard; returns the gold gained for "+N" feedback.
    pub fn click(&mut self, data: &GameData) -> f64 {
        let gained = self.gold_per_click(data);
        self.earn(gained);
        self.persistent.stats.clicks_total += 1.0;
        gained
    }

    /// Hires up to `requested` goblins, buying as many as gold allows.
    /// Returns `(hired, total_cost)`. `requested == u32::MAX` means "buy max".
    pub fn try_hire_bulk(
        &mut self,
        data: &GameData,
        requested: u32,
    ) -> Result<(u32, f64), BuyError> {
        let (count, cost) = economy::affordable_levels(
            data.config.base_hire_cost,
            data.config.hire_cost_growth,
            self.run.goblins,
            self.run.gold,
            requested,
        );
        if count == 0 {
            return Err(BuyError::CannotAfford);
        }
        self.run.gold -= cost;
        self.run.goblins += count;
        Ok((count, cost))
    }

    /// Pays the expedition cost and rolls for treasure (GDD §5.3).
    pub fn try_explore(&mut self, data: &GameData) -> Result<ExploreResult, BuyError> {
        if self.run.gold < data.config.explore_cost {
            return Err(BuyError::CannotAfford);
        }
        let chance = self.discovery_chance(data);
        self.run.gold -= data.config.explore_cost;

        let outcome = exploration::roll_treasure(
            &mut self.persistent.rng,
            &data.treasures,
            &self.persistent.discovered_treasures,
            chance,
        );
        Ok(match outcome {
            ExploreOutcome::NothingFound => ExploreResult::NothingFound,
            ExploreOutcome::AllDiscovered => ExploreResult::AllDiscovered,
            ExploreOutcome::Found(def) => {
                self.persistent.discovered_treasures.push(def.id.clone());
                ExploreResult::Found {
                    name: def.name.clone(),
                    rarity: def.rarity.label(),
                }
            }
        })
    }

    /// Buys up to `requested` levels of a run upgrade, capped by its max level
    /// and by affordable gold. Returns `(bought, new_level)`.
    pub fn try_buy_upgrade_bulk(
        &mut self,
        data: &GameData,
        id: &str,
        requested: u32,
    ) -> Result<(u32, u32), BuyError> {
        let def = data.upgrade(id).ok_or(BuyError::UnknownId)?;
        let level = self.run.upgrade_levels.get(id).copied().unwrap_or(0);
        if level >= def.max_level {
            return Err(BuyError::MaxLevel);
        }
        let remaining = def.max_level - level;
        let (count, cost) = economy::affordable_levels(
            def.base_cost,
            def.cost_growth,
            level,
            self.run.gold,
            requested.min(remaining),
        );
        if count == 0 {
            return Err(BuyError::CannotAfford);
        }
        self.run.gold -= cost;
        self.run.upgrade_levels.insert(id.to_owned(), level + count);
        self.persistent.stats.upgrades_purchased += f64::from(count);
        Ok((count, level + count))
    }

    /// Buys one level of a permanent prestige upgrade with Hoard Points.
    pub fn try_buy_prestige_upgrade(&mut self, data: &GameData, id: &str) -> Result<u32, BuyError> {
        let def = data.prestige_upgrade(id).ok_or(BuyError::UnknownId)?;
        let level = self
            .persistent
            .prestige_upgrade_levels
            .get(id)
            .copied()
            .unwrap_or(0);
        if level >= def.max_level {
            return Err(BuyError::MaxLevel);
        }
        let cost = economy::upgrade_cost(def.base_cost, def.cost_growth, level);
        if self.persistent.hoard_points < cost {
            return Err(BuyError::CannotAfford);
        }
        self.persistent.hoard_points -= cost;
        self.persistent
            .prestige_upgrade_levels
            .insert(id.to_owned(), level + 1);
        Ok(level + 1)
    }

    pub fn can_prestige(&self, data: &GameData) -> bool {
        prestige::can_prestige(&data.config, self.run.gold)
    }

    /// Hoard Points the current hoard would grant right now.
    pub fn prestige_preview(&self, data: &GameData) -> f64 {
        prestige::hoard_points_gained(
            &data.config,
            self.run.gold,
            &self.rates(data),
            &self.percents(data),
        )
    }

    /// Converts the hoard into Hoard Points and resets the run (GDD §5.4).
    /// Treasures, achievements, codex, and the prestige tree all persist.
    pub fn try_prestige(&mut self, data: &GameData) -> Option<f64> {
        if !self.can_prestige(data) {
            return None;
        }
        let gained = self.prestige_preview(data);
        self.persistent.hoard_points += gained;
        self.persistent.prestige_count += 1;
        self.run = RunState::new();
        Some(gained)
    }

    // --- unlock conditions ----------------------------------------------

    /// Current value for a condition stat (GDD §5.6). Lifetime stats survive
    /// prestige; `Goblins` is deliberately the current-run count.
    pub fn stat_value(&self, key: StatKey) -> f64 {
        match key {
            StatKey::ClicksTotal => self.persistent.stats.clicks_total,
            StatKey::GoldTotalEarned => self.persistent.stats.gold_total_earned,
            StatKey::Goblins => f64::from(self.run.goblins),
            StatKey::TreasuresDiscovered => self.persistent.discovered_treasures.len() as f64,
            StatKey::UpgradesPurchased => self.persistent.stats.upgrades_purchased,
            StatKey::PrestigeCount => f64::from(self.persistent.prestige_count),
            StatKey::AchievementsUnlocked => self.persistent.achievements.progress().0 as f64,
        }
    }

    /// Checks all achievement and codex conditions, applying rewards.
    /// Loops until stable so meta-achievements ("unlock 9 others") can chain
    /// within a single frame.
    pub fn check_unlocks(&mut self, data: &GameData) -> Vec<UnlockEvent> {
        let mut events = Vec::new();
        loop {
            let mut changed = false;

            for def in &data.achievements {
                if self.persistent.achievements.is_unlocked(&def.id) {
                    continue;
                }
                if self.stat_value(def.condition.stat) >= def.condition.gte
                    && self.persistent.achievements.unlock(&def.id)
                {
                    self.earn(def.reward.gold);
                    self.persistent.hoard_points += def.reward.hoard_points;
                    events.push(UnlockEvent::Achievement {
                        name: def.name.clone(),
                    });
                    changed = true;
                }
            }

            for def in &data.dragons {
                if self.persistent.unlocked_dragons.contains(&def.id) {
                    continue;
                }
                if self.stat_value(def.unlock.stat) >= def.unlock.gte {
                    self.persistent.unlocked_dragons.push(def.id.clone());
                    events.push(UnlockEvent::Dragon {
                        name: def.name.clone(),
                    });
                    changed = true;
                }
            }

            if !changed {
                return events;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save;

    fn setup() -> (GameData, GameplayState) {
        let data = GameData::load().unwrap();
        let state = GameplayState::new_game(&data, 1234);
        (data, state)
    }

    #[test]
    fn m1_loop_click_hire_upgrade_prestige() {
        let (data, mut state) = setup();

        // Click pays base_click and counts toward lifetime stats.
        let gained = state.click(&data);
        assert!((gained - data.config.base_click).abs() < 1e-9);
        assert!((state.run.gold - data.config.base_click).abs() < 1e-9);

        // Hire a goblin, passive income becomes real.
        state.run.gold = 100.0;
        let (hired, _) = state.try_hire_bulk(&data, 1).unwrap();
        assert_eq!(hired, 1);
        assert_eq!(state.run.goblins, 1);
        assert!(state.gold_per_second(&data) > 0.0);

        // Buy a click upgrade; the formula actually reads it (GDD fix).
        state.run.gold = 1000.0;
        state.try_buy_upgrade_bulk(&data, "click_power", 1).unwrap();
        assert!(state.gold_per_click(&data) > data.config.base_click);

        // Prestige converts the hoard into Hoard Points and resets the run.
        state.run.gold = 1_000_000.0;
        let goblins_before = state.run.goblins;
        let gained = state.try_prestige(&data).unwrap();
        assert!(gained >= 10.0);
        assert!(goblins_before > 0);
        assert_eq!(state.run.goblins, 0);
        assert!((state.run.gold).abs() < 1e-9);
        assert_eq!(state.persistent.prestige_count, 1);
        assert!(state.persistent.hoard_points >= 10.0);
    }

    #[test]
    fn bulk_hire_buys_as_many_as_affordable() {
        let (data, mut state) = setup();
        state.run.gold = 200.0;
        let (hired, cost) = state.try_hire_bulk(&data, u32::MAX).unwrap();
        assert!(hired >= 1);
        assert_eq!(state.run.goblins, hired);
        // Cost matches the summed per-level curve and leaves the change behind.
        let expected = economy::bulk_cost(
            data.config.base_hire_cost,
            data.config.hire_cost_growth,
            0,
            hired,
        );
        assert!((cost - expected).abs() < 1e-9);
        assert!((state.run.gold - (200.0 - expected)).abs() < 1e-9);
        // "Max" means the very next goblin is no longer affordable.
        let next = economy::upgrade_cost(
            data.config.base_hire_cost,
            data.config.hire_cost_growth,
            hired,
        );
        assert!(next > state.run.gold);
    }

    #[test]
    fn bulk_upgrade_respects_max_level_and_budget() {
        let (data, mut state) = setup();
        state.run.gold = 1e12;
        // Buying "max" cannot exceed the definition's max_level.
        let def_max = data.upgrade("click_power").unwrap().max_level;
        let (bought, new_level) = state
            .try_buy_upgrade_bulk(&data, "click_power", u32::MAX)
            .unwrap();
        assert_eq!(new_level, def_max);
        assert_eq!(bought, def_max);
        // Already maxed → MaxLevel error.
        assert_eq!(
            state.try_buy_upgrade_bulk(&data, "click_power", 1),
            Err(BuyError::MaxLevel)
        );
    }

    #[test]
    fn prestige_below_threshold_is_refused() {
        let (data, mut state) = setup();
        state.run.gold = 999_999.0;
        assert!(state.try_prestige(&data).is_none());
        assert_eq!(state.persistent.prestige_count, 0);
    }

    #[test]
    fn prestige_upgrades_persist_and_boost_next_run() {
        let (data, mut state) = setup();
        state.persistent.hoard_points = 10.0;
        state
            .try_buy_prestige_upgrade(&data, "ancient_claws")
            .unwrap();
        assert!((state.persistent.hoard_points - 5.0).abs() < 1e-9);

        state.run.gold = 1_000_000.0;
        state.try_prestige(&data).unwrap();
        // +25% permanent click bonus survives the reset.
        assert!((state.gold_per_click(&data) - data.config.base_click * 1.25).abs() < 1e-9);
    }

    #[test]
    fn achievements_unlock_with_rewards() {
        let (data, mut state) = setup();
        state.click(&data);
        let events = state.check_unlocks(&data);
        assert!(events
            .iter()
            .any(|e| matches!(e, UnlockEvent::Achievement { name } if name == "First Click")));
        // first_click reward pays 50 gold on top of the click gain.
        assert!(state.run.gold > 50.0);

        // Unlock is one-shot.
        assert!(state.check_unlocks(&data).is_empty());
    }

    #[test]
    fn dragon_codex_unlocks_on_prestige() {
        let (data, mut state) = setup();
        state.run.gold = 1_000_000.0;
        state.try_prestige(&data).unwrap();
        let events = state.check_unlocks(&data);
        assert!(events
            .iter()
            .any(|e| matches!(e, UnlockEvent::Dragon { name } if name == "Ember Wyrm")));
        // The unlocked dragon's +3% gold/sec bonus is live.
        state.run.goblins = 10;
        assert!(state.gold_per_second(&data) > 10.0);
    }

    #[test]
    fn save_roundtrip_applies_offline_progress() {
        let (data, mut state) = setup();
        state.run.goblins = 10;
        let rate = state.gold_per_second(&data);
        let saved = SaveData {
            version: data.config.version.clone(),
            timestamp: 1_000.0,
            run: state.run.clone(),
            persistent: state.persistent.clone(),
        };

        // Loading 60 seconds later earns 60s at the live passive rate.
        let (loaded, earned) = GameplayState::from_save(&data, saved, 1_060.0);
        assert!((earned - rate * 60.0).abs() < 1e-6);
        assert!((loaded.run.gold - rate * 60.0).abs() < 1e-6);

        let _ = save::now_timestamp; // referenced: real loads stamp with wall clock
    }
}
