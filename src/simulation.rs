//! Stateless simulation services (GDD §11): pure functions that receive
//! state and data, and return results. No rendering, no persistence.

#[cfg(test)]
mod balance;
pub mod economy;
pub mod exploration;
pub mod idle_number;
pub mod offline;
pub mod prestige;

use crate::data::EffectStat;
use std::collections::HashMap;

/// Additive per-stat bonuses, aggregated from data-driven sources.
///
/// Used in two roles kept deliberately separate in the formulas (GDD §5.1):
/// - *rate* bonuses from run-scoped upgrade levels (`level * rate`, additive)
/// - *percent* bonuses from treasures, dragon codex, and prestige upgrades
///
/// Percent sources pool additively per stat, which hard-caps their lifetime
/// impact. Compounding prestige nodes instead feed the separate multiplicative
/// channel ([`Bonuses::mul`]/[`Bonuses::product`]) so they grow geometrically.
#[derive(Debug, Clone, Default)]
pub struct Bonuses {
    values: HashMap<EffectStat, f64>,
    multipliers: HashMap<EffectStat, f64>,
}

impl Bonuses {
    pub fn add(&mut self, stat: EffectStat, amount: f64) {
        *self.values.entry(stat).or_default() += amount;
    }

    /// Folds a factor into the multiplicative channel for a stat.
    pub fn mul(&mut self, stat: EffectStat, factor: f64) {
        *self.multipliers.entry(stat).or_insert(1.0) *= factor;
    }

    /// The multiplicative channel's product for a stat (1.0 when untouched).
    pub fn product(&self, stat: EffectStat) -> f64 {
        self.multipliers.get(&stat).copied().unwrap_or(1.0)
    }

    /// The raw additive sum for a stat (0.0 when nothing contributes).
    pub fn sum(&self, stat: EffectStat) -> f64 {
        self.values.get(&stat).copied().unwrap_or(0.0)
    }

    /// `1.0 + sum` — for rate bonuses used as `(1 + level * rate)` factors.
    pub fn factor(&self, stat: EffectStat) -> f64 {
        1.0 + self.sum(stat)
    }

    /// `1.0 + sum/100` — for percent bonuses used as multipliers.
    pub fn percent_multiplier(&self, stat: EffectStat) -> f64 {
        1.0 + self.sum(stat) / 100.0
    }
}
