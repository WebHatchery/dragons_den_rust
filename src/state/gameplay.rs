//! The single active-save state (GDD §11): every UI panel reads from this
//! one struct — there is deliberately no second store to drift out of sync.

mod progression;

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
    /// Expeditions launched (#8 mechanic) — every paid or salvage dig. New
    /// counter, so `serde(default)` keeps old saves loadable.
    #[serde(default)]
    pub expeditions_launched: f64,
    /// Golden Hoard glints collected (#9 mechanic).
    #[serde(default)]
    pub golden_hoards_collected: f64,
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
/// never borrows into the treasure catalog. Not `Eq` — `Found` carries the
/// rarity-scaled rush duration (an `f64`); `PartialEq` still serves the tests.
#[derive(Debug, Clone, PartialEq)]
pub enum ExploreResult {
    NothingFound,
    Found {
        name: String,
        rarity: &'static str,
        /// Seconds of the Hoard Rush the find ignited (rarity-scaled), for the
        /// notification/log to report the surge the player just earned.
        rush_seconds: f64,
    },
    AllDiscovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuyError {
    UnknownId,
    MaxLevel,
    CannotAfford,
}

/// An active Golden Hoard glint (#9): a clickable burst that grants one of the
/// [`GoldenReward`] variants. Transient — never saved; earned by playing.
#[derive(Debug, Clone, Copy)]
pub struct GoldenHoard {
    /// Position in the play area as normalized `[0,1]` coords. The UI maps this
    /// into the center content rect, so the state layer stays ignorant of pixels.
    pub nx: f32,
    pub ny: f32,
    /// Seconds the glint stays clickable before it fades away.
    pub remaining: f64,
}

/// What a collected Golden Hoard pays out (#9). Picked by a state-weighted roll
/// on each click (see [`GameplayState::golden_reward_weights`]) so the reward
/// tends to fit the current economy, while every glint stays a small surprise
/// the way the genre's golden cookies vary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GoldenReward {
    /// Dragon's Frenzy — a temporary ×click-gold surge.
    Frenzy,
    /// Hoard Rush — a temporary ×all-gold surge (the #7 buff).
    Rush,
    /// An instant lump of gold worth a burst of current income.
    Windfall(f64),
}

pub struct GameplayState {
    pub run: RunState,
    pub persistent: PersistentState,
    pub screen: Screen,
    pub buy_mode: BuyMode,
    pub action_log: ActionLog,
    /// Whether the settings overlay is open over the gameplay frame. Transient.
    pub settings_open: bool,
    /// Seconds of Hoard Rush income surge remaining (#7). Transient — a fresh
    /// surge is earned by exploring, not restored on load.
    hoard_rush_secs: f64,
    /// Seconds of Dragon's Frenzy (click surge) remaining (#9). Transient.
    frenzy_secs: f64,
    /// Seconds until the next Golden Hoard glint appears (#9). Transient.
    golden_hoard_cooldown: f64,
    /// The currently-visible Golden Hoard glint, if any (#9). Transient.
    golden_hoard: Option<GoldenHoard>,
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
            settings_open: false,
            hoard_rush_secs: 0.0,
            frenzy_secs: 0.0,
            golden_hoard_cooldown: data.config.golden_hoard_min_interval,
            golden_hoard: None,
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
            settings_open: false,
            hoard_rush_secs: 0.0,
            frenzy_secs: 0.0,
            golden_hoard_cooldown: data.config.golden_hoard_min_interval,
            golden_hoard: None,
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
            * self.frenzy_factor(data)
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

    /// (Re)starts the Hoard Rush surge for `seconds`. Re-triggering never stacks
    /// the multiplier — it takes the *longer* of the current and new timers, so
    /// spamming Explore sustains the buff (bounded, active-play reward) and a
    /// short salvage rush can never cut short a long one a rare find ignited.
    fn trigger_hoard_rush(&mut self, seconds: f64) {
        self.hoard_rush_secs = self.hoard_rush_secs.max(seconds);
    }

    /// How long a Hoard Rush a treasure find ignites should last, lengthening
    /// with rarity (#find-rush): `hoard_rush_seconds * (1 + tier * step)`. A
    /// Common find matches the base salvage/miss rush; a Mythic haul sustains a
    /// far longer income window — the rarity ladder felt in the moment of the
    /// find, not just as a silent passive %.
    fn treasure_find_rush_seconds(&self, data: &GameData, rarity: crate::data::Rarity) -> f64 {
        let step = f64::from(rarity.tier_index()) * data.config.treasure_find_rush_rarity_step;
        data.config.hoard_rush_seconds * (1.0 + step)
    }

    // --- Golden Hoard / Dragon's Frenzy (#9) -----------------------------

    /// Whether a Dragon's Frenzy click surge is currently active.
    pub fn frenzy_active(&self) -> bool {
        self.frenzy_secs > 0.0
    }

    /// Seconds of Dragon's Frenzy remaining, for the UI countdown.
    pub fn frenzy_remaining(&self) -> f64 {
        self.frenzy_secs.max(0.0)
    }

    /// Click-gold multiplier from an active Dragon's Frenzy, else 1.0.
    fn frenzy_factor(&self, data: &GameData) -> f64 {
        if self.frenzy_active() {
            data.config.dragon_frenzy_multiplier
        } else {
            1.0
        }
    }

    /// The currently-visible Golden Hoard glint, if any, for the UI to draw and
    /// hit-test.
    pub fn golden_hoard(&self) -> Option<GoldenHoard> {
        self.golden_hoard
    }

    /// Advances the active-play events that only tick while the player is present
    /// (#9): the Dragon's Frenzy drain and the Golden Hoard spawn/expiry clock.
    /// Called from the live update loop only — never from offline catch-up, since
    /// being there is the whole point of the hook.
    pub fn advance_events(&mut self, data: &GameData, dt: f32) {
        let dt = f64::from(dt);
        self.frenzy_secs = (self.frenzy_secs - dt).max(0.0);
        match &mut self.golden_hoard {
            Some(glint) => {
                glint.remaining -= dt;
                if glint.remaining <= 0.0 {
                    self.golden_hoard = None;
                    self.schedule_next_glint(data);
                }
            }
            None => {
                self.golden_hoard_cooldown -= dt;
                if self.golden_hoard_cooldown <= 0.0 {
                    self.spawn_golden_hoard(data);
                }
            }
        }
    }

    /// Places a glint at a seeded random spot; visible for `golden_hoard_lifetime`.
    fn spawn_golden_hoard(&mut self, data: &GameData) {
        self.golden_hoard = Some(GoldenHoard {
            nx: self.persistent.rng.next_f32(),
            ny: self.persistent.rng.next_f32(),
            remaining: data.config.golden_hoard_lifetime,
        });
    }

    /// Schedules the next glint a seeded random `[min, max]` seconds out.
    fn schedule_next_glint(&mut self, data: &GameData) {
        let cfg = &data.config;
        let span = (cfg.golden_hoard_max_interval - cfg.golden_hoard_min_interval).max(0.0);
        self.golden_hoard_cooldown =
            cfg.golden_hoard_min_interval + f64::from(self.persistent.rng.next_f32()) * span;
    }

    /// State-aware weights for the three Golden Hoard payouts `(frenzy, rush,
    /// windfall)`. Rather than a blind 1/3, the roll leans toward the reward that
    /// actually helps the current economy: Frenzy (a click surge) when the
    /// economy is click-dominant (early game, few minions), Rush (a passive
    /// surge) when it is passive-dominant — so a Rush is never wasted on a
    /// minion-less hoard. Click gold is scaled by an assumed clicks/second so it
    /// compares to passive on a per-second footing. Every weight keeps a floor,
    /// so all three stay possible and the surprise survives; Windfall carries a
    /// constant bias since its floored lump always pays *something*.
    fn golden_reward_weights(&self, data: &GameData) -> (f64, f64, f64) {
        let rates = self.rates(data);
        let percents = self.percents(data);
        let passive = economy::gold_per_second(&data.config, self.run.goblins, &rates, &percents)
            + economy::extra_minion_income(
                &data.minions,
                &self.run.minion_counts,
                &rates,
                &percents,
            );
        let click = economy::gold_per_click(&data.config, &rates, &percents)
            * data.config.golden_reward_assumed_clicks_per_second;
        let total = passive + click;
        let passive_share = if total > 0.0 { passive / total } else { 0.0 };
        let floor = data.config.golden_reward_weight_floor;
        (
            floor + (1.0 - passive_share),
            floor + passive_share,
            floor + data.config.golden_reward_windfall_bias,
        )
    }

    /// Collects the active glint (a player click landed on it): consumes it,
    /// pays out one state-weighted reward, and arms the next glint. Returns the
    /// reward, or `None` if there was no glint to collect.
    pub fn collect_golden_hoard(&mut self, data: &GameData) -> Option<GoldenReward> {
        self.golden_hoard.take()?;
        self.persistent.stats.golden_hoards_collected += 1.0;
        self.schedule_next_glint(data);

        // Weighted pick over the three payouts on the state-owned RNG: one draw,
        // walked against the state-aware weights (deterministic per save).
        let (w_frenzy, w_rush, w_windfall) = self.golden_reward_weights(data);
        let roll = f64::from(self.persistent.rng.next_f32()) * (w_frenzy + w_rush + w_windfall);
        let reward = if roll < w_frenzy {
            self.frenzy_secs = data.config.dragon_frenzy_seconds;
            GoldenReward::Frenzy
        } else if roll < w_frenzy + w_rush {
            self.trigger_hoard_rush(data.config.hoard_rush_seconds);
            GoldenReward::Rush
        } else {
            // A burst of current income, floored so an early idle grab still
            // pays out something meaningful rather than nothing.
            let lump = (self.gold_per_second(data) * data.config.golden_windfall_seconds)
                .max(data.config.base_click * 50.0);
            self.earn(lump);
            GoldenReward::Windfall(lump)
        };
        Some(reward)
    }

    /// The effective base-Kobold soft cap after prestige wall-breakers (#11).
    pub fn base_minion_soft_cap(&self, data: &GameData) -> f64 {
        economy::minion_soft_cap(&data.config, &self.percents(data))
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

    /// Effective hire-cost base for a typed minion tier after the `HireDiscount`
    /// channel — the same discount that cheapens base Kobolds now cheapens every
    /// tier, so the discount upgrade lines pay off across the whole army instead
    /// of only the base tier. One source of truth for both the charge and the
    /// Minions-panel cost display.
    pub fn minion_hire_base_cost(&self, data: &GameData, def: &crate::data::MinionDef) -> f64 {
        economy::apply_hire_discount(def.base_cost, &self.rates(data))
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
        let base_cost = self.minion_hire_base_cost(data, def);
        let (count, cost) = economy::affordable_levels(
            base_cost,
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
}

#[cfg(test)]
mod tests;
