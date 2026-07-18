//! The single converged upgrade catalog plus the prestige permanent tree.

use crate::data::{EffectStat, PercentEffect};
use serde::{Deserialize, Serialize};

/// Per-level rate effect: `stat` gains `rate` per purchased level (GDD §5.1).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RateEffect {
    pub stat: EffectStat,
    pub rate: f64,
}

/// A run-scoped upgrade line, reset on prestige.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: f64,
    pub cost_growth: f64,
    pub max_level: u32,
    pub effect: RateEffect,
}

/// A permanent upgrade bought with Hoard Points, persisting across prestiges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrestigeUpgradeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: f64,
    pub cost_growth: f64,
    pub max_level: u32,
    pub effect: PercentEffect,
}
