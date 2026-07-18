//! Rarity-weighted treasure table (GDD §5.3).

use crate::data::PercentEffect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
    /// Prestige-gated top tier (engagement review #8): the strongest finds, only
    /// reachable once the player has burned enough hoards.
    Mythic,
}

impl Rarity {
    pub fn label(self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
            Rarity::Mythic => "Mythic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreasureDef {
    pub id: String,
    pub name: String,
    pub rarity: Rarity,
    /// Relative weight for the rarity-weighted roll among undiscovered treasures.
    pub drop_weight: u32,
    /// Prestiges that must be done before this treasure can be found (#8). The
    /// prestige-gated long tail keeps exploration a chase across many runs.
    /// `serde(default)` → 0 keeps every existing treasure ungated and old saves
    /// loadable.
    #[serde(default)]
    pub prestige_required: u32,
    pub description: String,
    pub effect: PercentEffect,
}
