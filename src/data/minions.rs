//! Additional minion tiers beyond the base Kobold (UI redesign P4).
//!
//! The base tier's economy lives in `GameConfig` (`gold_per_goblin`,
//! `base_hire_cost`, `hire_cost_growth`) and the `run.goblins` count. These
//! extra tiers are fully data-driven: each adds `count * base_rate` to passive
//! income and unlocks once total minions reach `unlock_at`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinionDef {
    pub id: String,
    pub name: String,
    /// Passive gold/sec added per hired minion of this tier (before efficiency).
    pub base_rate: f64,
    /// First-hire cost; subsequent hires follow `base_cost * cost_growth^count`.
    pub base_cost: f64,
    pub cost_growth: f64,
    /// Total minions (all tiers) required before this tier can be hired.
    pub unlock_at: u32,
    /// Prestiges completed before this tier can be hired — a qualitative
    /// prestige reward ("burn the hoard to unlock X"), not just a multiplier.
    #[serde(default)]
    pub prestige_required: u32,
}
