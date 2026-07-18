//! Achievement definitions — every entry has a wired condition (GDD §5.6).

use crate::data::StatCondition;
use serde::{Deserialize, Serialize};

/// One-time reward granted at unlock. Zero-valued fields are simply unused.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AchievementReward {
    #[serde(default)]
    pub gold: f64,
    #[serde(default)]
    pub hoard_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: StatCondition,
    pub reward: AchievementReward,
}
