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
    /// Counts of the extra minion tiers (P4), keyed by `MinionDef::id`. The base
    /// Kobold tier stays in `goblins`. `serde(default)` keeps old saves loadable.
    #[serde(default)]
    pub minion_counts: HashMap<String, u32>,
    /// Active-play seconds since this run began (reset on prestige). Drives the
    /// left rail's "run time" readout. `serde(default)` keeps old saves loadable.
    #[serde(default)]
    pub run_seconds: f64,
}

impl RunState {
    pub fn new() -> Self {
        Self {
            gold: 0.0,
            goblins: 0,
            upgrade_levels: HashMap::new(),
            minion_counts: HashMap::new(),
            run_seconds: 0.0,
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
    /// Vertical scroll offset for the current screen's list; reset to 0 on every
    /// screen switch (only one screen is visible at a time). Transient.
    pub scroll_y: f32,
    /// Whether the settings overlay is open over the gameplay frame. Transient.
    pub settings_open: bool,
    /// Seconds of Hoard Rush income surge remaining (#7). Transient — a fresh
    /// surge is earned by exploring, not restored on load.
    hoard_rush_secs: f64,
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
            scroll_y: 0.0,
            settings_open: false,
            hoard_rush_secs: 0.0,
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
            scroll_y: 0.0,
            settings_open: false,
            hoard_rush_secs: 0.0,
            autosave_accum: 0.0,
        };
        state
            .persistent
            .achievements
            .sync_definitions(achievement_definitions(&data.achievements));

        let earned = offline::offline_gold(
            state.gold_per_second(data),
            now - save.timestamp,
            data.config.offline_cap_hours * 3600.0,
        );
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
            * self.hoard_rush_factor(data)
    }

    pub fn gold_per_second(&self, data: &GameData) -> f64 {
        let rates = self.rates(data);
        let percents = self.percents(data);
        let base = economy::gold_per_second(&data.config, self.run.goblins, &rates, &percents)
            + economy::extra_minion_income(
                &data.minions,
                &self.run.minion_counts,
                &rates,
                &percents,
            );
        base * self.hoard_rush_factor(data)
    }

    /// Whether a Hoard Rush income surge is currently active (#7).
    pub fn hoard_rush_active(&self) -> bool {
        self.hoard_rush_secs > 0.0
    }

    /// Seconds of Hoard Rush remaining, for the UI countdown.
    pub fn hoard_rush_remaining(&self) -> f64 {
        self.hoard_rush_secs.max(0.0)
    }

    /// Live income multiplier from an active Hoard Rush, else 1.0. Deliberately
    /// kept out of `economy` so the balance sim — which never explores — stays an
    /// honest measure of the core loop, unperturbed by this active-play bonus.
    fn hoard_rush_factor(&self, data: &GameData) -> f64 {
        if self.hoard_rush_active() {
            data.config.hoard_rush_multiplier
        } else {
            1.0
        }
    }

    /// (Re)starts the Hoard Rush surge. Re-triggering refreshes the timer to full
    /// but never stacks the multiplier, so spamming Explore merely sustains the
    /// buff instead of compounding it — a bounded active-play reward.
    fn trigger_hoard_rush(&mut self, data: &GameData) {
        self.hoard_rush_secs = data.config.hoard_rush_seconds;
    }

    /// Total minions across every tier (base Kobolds + extra tiers).
    pub fn total_minions(&self) -> u32 {
        self.run.goblins + self.run.minion_counts.values().sum::<u32>()
    }

    /// Current count of an extra minion tier.
    pub fn minion_count(&self, id: &str) -> u32 {
        self.run.minion_counts.get(id).copied().unwrap_or(0)
    }

    /// An extra tier is available once total minions reach its `unlock_at`
    /// and enough prestiges have been burned (`prestige_required`).
    pub fn minion_unlocked(&self, def: &crate::data::MinionDef) -> bool {
        self.total_minions() >= def.unlock_at
            && self.persistent.prestige_count >= def.prestige_required
    }

    /// Current purchased level of a prestige tree node.
    pub fn prestige_level(&self, id: &str) -> u32 {
        self.persistent
            .prestige_upgrade_levels
            .get(id)
            .copied()
            .unwrap_or(0)
    }

    /// A prestige node unlocks once its prerequisite has at least one level.
    pub fn prestige_prereq_met(&self, def: &crate::data::PrestigeUpgradeDef) -> bool {
        match &def.prereq {
            None => true,
            Some(id) => self.prestige_level(id) >= 1,
        }
    }

    /// Hires up to `requested` of an extra minion tier, buying as many as gold
    /// allows. Returns `(hired, total_cost)`.
    pub fn try_hire_minion(
        &mut self,
        data: &GameData,
        id: &str,
        requested: u32,
    ) -> Result<(u32, f64), BuyError> {
        let def = data
            .minions
            .iter()
            .find(|d| d.id == id)
            .ok_or(BuyError::UnknownId)?;
        if !self.minion_unlocked(def) {
            return Err(BuyError::CannotAfford);
        }
        let (count, cost) = economy::affordable_levels(
            def.base_cost,
            def.cost_growth,
            self.minion_count(id),
            self.run.gold,
            requested,
        );
        if count == 0 {
            return Err(BuyError::CannotAfford);
        }
        self.run.gold -= cost;
        *self.run.minion_counts.entry(id.to_owned()).or_insert(0) += count;
        Ok((count, cost))
    }

    pub fn discovery_chance(&self, data: &GameData) -> f64 {
        economy::discovery_chance(&data.config, &self.rates(data), &self.percents(data))
    }

    /// Current expedition cost, rising with treasures already discovered (#6).
    pub fn explore_cost(&self, data: &GameData) -> f64 {
        economy::explore_cost(&data.config, self.persistent.discovered_treasures.len())
    }

    // --- actions ---------------------------------------------------------

    fn earn(&mut self, amount: f64) {
        self.run.gold += amount;
        self.persistent.stats.gold_total_earned += amount;
    }

    /// Passive income for one frame; also advances the run clock. Earnings use
    /// the current Hoard Rush factor, so the surge is drained *after* it pays out.
    pub fn tick(&mut self, data: &GameData, dt: f32) {
        self.earn(self.gold_per_second(data) * f64::from(dt));
        self.run.run_seconds += f64::from(dt);
        self.hoard_rush_secs = (self.hoard_rush_secs - f64::from(dt)).max(0.0);
    }

    /// Counts up toward the player's autosave interval (seconds, from settings);
    /// true means "save now".
    pub fn autosave_due(&mut self, interval_secs: f32, dt: f32) -> bool {
        self.autosave_accum += dt;
        if self.autosave_accum >= interval_secs {
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

    /// Effective base goblin hire cost after the Goblin Recruiters discount.
    pub fn hire_base_cost(&self, data: &GameData) -> f64 {
        economy::hire_base_cost(&data.config, &self.rates(data))
    }

    /// Hires up to `requested` goblins, buying as many as gold allows.
    /// Returns `(hired, total_cost)`. `requested == u32::MAX` means "buy max".
    pub fn try_hire_bulk(
        &mut self,
        data: &GameData,
        requested: u32,
    ) -> Result<(u32, f64), BuyError> {
        let (count, cost) = economy::affordable_levels(
            self.hire_base_cost(data),
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

    /// Pays the expedition cost and rolls for treasure (GDD §5.3). The cost
    /// scales with treasures already found (#6); a complete set is never
    /// charged, since its outcome is a guaranteed empty result.
    pub fn try_explore(&mut self, data: &GameData) -> Result<ExploreResult, BuyError> {
        let cost = self.explore_cost(data);
        if self.run.gold < cost {
            return Err(BuyError::CannotAfford);
        }
        let chance = self.discovery_chance(data);
        let outcome = exploration::roll_treasure(
            &mut self.persistent.rng,
            &data.treasures,
            &self.persistent.discovered_treasures,
            chance,
        );
        // A complete hoard rolls `AllDiscovered` before touching the RNG, so
        // don't charge for an expedition that can't find anything — but it still
        // sparks a Hoard Rush so the button stays worth pressing (#7).
        if outcome == ExploreOutcome::AllDiscovered {
            self.trigger_hoard_rush(data);
            return Ok(ExploreResult::AllDiscovered);
        }
        self.run.gold -= cost;

        Ok(match outcome {
            ExploreOutcome::NothingFound => {
                // A miss is no longer a dead loss: it stirs a Hoard Rush (#7).
                self.trigger_hoard_rush(data);
                ExploreResult::NothingFound
            }
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

    /// Gold needed for the next prestige, rising with each one done (GDD §12 Q2).
    pub fn prestige_threshold(&self, data: &GameData) -> f64 {
        prestige::current_threshold(&data.config, self.persistent.prestige_count)
    }

    pub fn can_prestige(&self, data: &GameData) -> bool {
        prestige::can_prestige(&data.config, self.run.gold, self.persistent.prestige_count)
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
    /// prestige; `Goblins` is deliberately the current-run count (all tiers).
    pub fn stat_value(&self, key: StatKey) -> f64 {
        match key {
            StatKey::ClicksTotal => self.persistent.stats.clicks_total,
            StatKey::GoldTotalEarned => self.persistent.stats.gold_total_earned,
            StatKey::Goblins => f64::from(self.total_minions()),
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
    fn expedition_cost_scales_and_complete_set_is_free() {
        let (data, mut state) = setup();
        // Guaranteed discovery so each successful expedition raises the cost.
        state.run.gold = 1e12;
        let first_cost = state.explore_cost(&data);
        assert!((first_cost - data.config.explore_cost).abs() < 1e-9);

        // Discover one treasure at a 100% find rate; the next cost is higher.
        let discovered_before = state.persistent.discovered_treasures.len();
        // Force a certain find by exhausting the roll against a full catalog is
        // awkward, so drive it directly through the discovery list instead.
        state
            .persistent
            .discovered_treasures
            .push(data.treasures[0].id.clone());
        let second_cost = state.explore_cost(&data);
        assert!(second_cost > first_cost, "cost must rise after a discovery");
        assert_eq!(
            state.persistent.discovered_treasures.len(),
            discovered_before + 1
        );

        // Complete the set: exploring is refused-free — no gold is spent.
        state.persistent.discovered_treasures =
            data.treasures.iter().map(|d| d.id.clone()).collect();
        let gold_before = state.run.gold;
        assert_eq!(state.try_explore(&data), Ok(ExploreResult::AllDiscovered));
        assert!((state.run.gold - gold_before).abs() < 1e-9);
    }

    #[test]
    fn expedition_without_new_treasure_grants_hoard_rush() {
        let (data, mut state) = setup();
        state.run.goblins = 10;

        // Complete the set so the roll is a guaranteed AllDiscovered, then read
        // the un-rushed rate *with* those treasure bonuses already applied.
        state.persistent.discovered_treasures =
            data.treasures.iter().map(|d| d.id.clone()).collect();
        let rate_no_rush = state.gold_per_second(&data);
        assert!(!state.hoard_rush_active());

        state.run.gold = 1e9;
        let gold_before = state.run.gold;
        assert_eq!(state.try_explore(&data), Ok(ExploreResult::AllDiscovered));

        // The surge is live, income is multiplied, and the salvage run was free.
        assert!(state.hoard_rush_active());
        assert!((state.run.gold - gold_before).abs() < 1e-9);
        let mult = data.config.hoard_rush_multiplier;
        assert!((state.gold_per_second(&data) - rate_no_rush * mult).abs() < 1e-6);

        // Re-triggering refreshes the timer but never stacks — still ×mult, not
        // ×mult².
        state.try_explore(&data).unwrap();
        assert!((state.gold_per_second(&data) - rate_no_rush * mult).abs() < 1e-6);

        // It drains with time and expires back to the base rate.
        state.tick(&data, data.config.hoard_rush_seconds as f32 + 0.1);
        assert!(!state.hoard_rush_active());
        assert!((state.gold_per_second(&data) - rate_no_rush).abs() < 1e-6);
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
