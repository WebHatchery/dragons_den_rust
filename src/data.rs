//! Embedded game data: config plus the data-driven catalogs (GDD §6).
//!
//! All balance values live in `assets/data/*.json` and are embedded with
//! `include_str!` so native and WASM builds load identically.

pub mod achievements;
pub mod dragons;
pub mod minions;
pub mod treasures;
pub mod upgrades;

use macroquad_toolkit::data_loader::load_embedded_json_labeled;
use serde::{Deserialize, Serialize};

pub use achievements::AchievementDef;
pub use dragons::DragonDef;
pub use minions::MinionDef;
pub use treasures::TreasureDef;
pub use upgrades::{PrestigeBranch, PrestigeUpgradeDef, UpgradeDef};

const GAME_CONFIG_JSON: &str = include_str!("../assets/data/game_config.json");
const UPGRADES_JSON: &str = include_str!("../assets/data/upgrades.json");
const PRESTIGE_UPGRADES_JSON: &str = include_str!("../assets/data/prestige_upgrades.json");
const TREASURES_JSON: &str = include_str!("../assets/data/treasures.json");
const ACHIEVEMENTS_JSON: &str = include_str!("../assets/data/achievements.json");
const DRAGONS_JSON: &str = include_str!("../assets/data/dragons.json");
const MINIONS_JSON: &str = include_str!("../assets/data/minions.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_name: String,
    pub display_name: String,
    /// Display name for hireable minions (singular / plural). Purely cosmetic —
    /// the internal `goblins` field and `Goblins` stat key are stable save keys
    /// and intentionally do not follow this rename.
    pub minion_name: String,
    pub minion_name_plural: String,
    pub save_slot: String,
    pub version: String,
    pub base_click: f64,
    pub base_passive: f64,
    pub gold_per_goblin: f64,
    pub base_hire_cost: f64,
    pub hire_cost_growth: f64,
    pub explore_cost: f64,
    /// Multiplier applied to the expedition cost per treasure already discovered
    /// (engagement review #6): `explore_cost_n = explore_cost * growth^n`. Turns
    /// the mid-game treasure sweep from free button-mashing into an escalating
    /// investment. Only actual discoveries raise it — failed rolls do not.
    pub explore_cost_growth: f64,
    /// Income multiplier of a Hoard Rush surge (engagement review #7): an
    /// expedition that finds no new treasure (a miss or a completed set) sparks
    /// this temporary ×multiplier on all gold, so Explore never dead-ends into a
    /// useless button. Re-triggering refreshes the timer but never stacks.
    pub hoard_rush_multiplier: f64,
    /// How long a Hoard Rush surge lasts, in seconds.
    pub hoard_rush_seconds: f64,
    pub base_discovery_chance: f64,
    pub prestige_threshold: f64,
    /// Multiplier applied to the prestige threshold per prestige already done
    /// (GDD §12 Q2 multi-tier prestige): `threshold_n = base * growth^n`.
    pub prestige_threshold_growth: f64,
    pub prestige_divisor: f64,
    /// Exponent on `(gold / divisor)` when converting the hoard to Hoard
    /// Points. Slightly above sqrt (0.5) so overshooting the threshold before
    /// burning pays — prestige timing becomes a decision, not a reflex.
    pub prestige_exponent: f64,
    /// Maximum hours of offline earnings credited on load (GDD §12 Q3). A
    /// generous cap keeps a multi-week absence from trivializing progression
    /// while still rewarding daily check-ins; raise it toward uncapped freely.
    pub offline_cap_hours: f64,
}

/// A lifetime/run statistic that unlock conditions can reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatKey {
    ClicksTotal,
    GoldTotalEarned,
    Goblins,
    TreasuresDiscovered,
    UpgradesPurchased,
    PrestigeCount,
    AchievementsUnlocked,
}

/// Threshold condition shared by achievements and dragon-codex unlocks.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatCondition {
    pub stat: StatKey,
    pub gte: f64,
}

/// A stat that percent/rate effects can modify (GDD §5.1, §5.3, §5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectStat {
    GoldPerClick,
    GoldPerSecond,
    MinionEfficiency,
    DiscoveryChance,
    HoardPointGain,
    /// Reduces goblin hire cost as a `1 / (1 + sum)` divisor on the base curve.
    HireDiscount,
    /// Scales every gold income source (click, passive, extra minion tiers).
    /// The only stat with a *compounding* prestige node behind it — the
    /// geometric engine that lets permanent bonuses outrun the prestige wall.
    AllGold,
}

/// Additive percent bonus, used by treasures, dragons, and prestige upgrades.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PercentEffect {
    pub stat: EffectStat,
    pub percent: f64,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub upgrades: Vec<UpgradeDef>,
    pub prestige_upgrades: Vec<PrestigeUpgradeDef>,
    pub treasures: Vec<TreasureDef>,
    pub achievements: Vec<AchievementDef>,
    pub dragons: Vec<DragonDef>,
    pub minions: Vec<MinionDef>,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        Ok(Self {
            config: load_embedded_json_labeled("game_config", GAME_CONFIG_JSON)?,
            upgrades: load_embedded_json_labeled("upgrades", UPGRADES_JSON)?,
            prestige_upgrades: load_embedded_json_labeled(
                "prestige_upgrades",
                PRESTIGE_UPGRADES_JSON,
            )?,
            treasures: load_embedded_json_labeled("treasures", TREASURES_JSON)?,
            achievements: load_embedded_json_labeled("achievements", ACHIEVEMENTS_JSON)?,
            dragons: load_embedded_json_labeled("dragons", DRAGONS_JSON)?,
            minions: load_embedded_json_labeled("minions", MINIONS_JSON)?,
        })
    }

    pub fn upgrade(&self, id: &str) -> Option<&UpgradeDef> {
        self.upgrades.iter().find(|def| def.id == id)
    }

    pub fn prestige_upgrade(&self, id: &str) -> Option<&PrestigeUpgradeDef> {
        self.prestige_upgrades.iter().find(|def| def.id == id)
    }

    pub fn treasure(&self, id: &str) -> Option<&TreasureDef> {
        self.treasures.iter().find(|def| def.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_data_loads() {
        let data = GameData::load().unwrap();

        assert_eq!(data.config.game_name, "dragons_den");
        // Lower bounds track the GDD §8 content targets; content may grow past
        // them without breaking this loader smoke test.
        assert!(data.upgrades.len() >= 4);
        assert!(data.prestige_upgrades.len() >= 8);
        assert!(data.treasures.len() >= 15);
        assert!(data.achievements.len() >= 20);
        assert_eq!(data.dragons.len(), 8);
    }

    #[test]
    fn catalog_ids_are_unique_and_resolvable() {
        let data = GameData::load().unwrap();

        for def in &data.upgrades {
            assert!(data.upgrade(&def.id).is_some());
        }
        for def in &data.treasures {
            assert!(data.treasure(&def.id).is_some());
        }

        let mut ids: Vec<&str> = data
            .upgrades
            .iter()
            .map(|d| d.id.as_str())
            .chain(data.prestige_upgrades.iter().map(|d| d.id.as_str()))
            .chain(data.treasures.iter().map(|d| d.id.as_str()))
            .chain(data.achievements.iter().map(|d| d.id.as_str()))
            .chain(data.dragons.iter().map(|d| d.id.as_str()))
            .collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "duplicate ids across catalogs");
    }
}
