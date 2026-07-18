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
}

impl Rarity {
    pub fn label(self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
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
    pub description: String,
    pub effect: PercentEffect,
}
