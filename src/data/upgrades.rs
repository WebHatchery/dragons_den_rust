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
    /// Prestiges required before this line appears in the shop (engagement
    /// review #10). Gating whole upgrade lines behind prestige gives players a
    /// *qualitative* reason to prestige — "unlock a new line" — that a flat
    /// multiplier can't. `serde(default)` → 0 keeps existing lines ungated.
    #[serde(default)]
    pub prestige_required: u32,
}

/// One of the five "Hoard Legacies" prestige tree columns (P5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrestigeBranch {
    Greed,
    Power,
    Discovery,
    Legion,
    Eternity,
}

impl PrestigeBranch {
    pub const ALL: [PrestigeBranch; 5] = [
        PrestigeBranch::Greed,
        PrestigeBranch::Power,
        PrestigeBranch::Discovery,
        PrestigeBranch::Legion,
        PrestigeBranch::Eternity,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PrestigeBranch::Greed => "Greed",
            PrestigeBranch::Power => "Power",
            PrestigeBranch::Discovery => "Discovery",
            PrestigeBranch::Legion => "Legion",
            PrestigeBranch::Eternity => "Eternity",
        }
    }
}

/// A permanent upgrade bought with Hoard Points, persisting across prestiges.
/// Each node sits in a `branch` column at a `tier` (row); a `prereq` node must
/// have at least one level before this node unlocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrestigeUpgradeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: f64,
    pub cost_growth: f64,
    pub max_level: u32,
    pub effect: PercentEffect,
    /// When true, each level *multiplies* the stat by `1 + percent/100`
    /// (compounding across levels) instead of pooling additively — the
    /// geometric permanent bonus that can chase a multiplying prestige wall.
    #[serde(default)]
    pub compounding: bool,
    pub branch: PrestigeBranch,
    pub tier: u32,
    #[serde(default)]
    pub prereq: Option<String>,
}
