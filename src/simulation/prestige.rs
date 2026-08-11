//! Prestige math (GDD §5.4) — the port's single biggest fix over the
//! original, whose prestige reset everything and granted nothing.

use crate::data::{EffectStat, GameConfig};
use crate::simulation::Bonuses;

/// Gold required for the next prestige given how many have already been done
/// (GDD §12 Q2): `base * growth^prestige_count`. Tier 0 is the configured base.
pub fn current_threshold(config: &GameConfig, prestige_count: u32) -> f64 {
    config.prestige_threshold * config.prestige_threshold_growth.powi(prestige_count as i32)
}

pub fn can_prestige(config: &GameConfig, gold: f64, prestige_count: u32) -> bool {
    gold >= current_threshold(config, prestige_count)
}

/// `floor((gold / divisor)^exponent)`, scaled by the Hoard Greed upgrade line
/// and any percent bonuses to hoard-point gain. The exponent sits above sqrt
/// so overshooting the threshold before burning is rewarded.
pub fn hoard_points_gained(
    config: &GameConfig,
    gold: f64,
    rates: &Bonuses,
    percents: &Bonuses,
) -> f64 {
    let base = (gold.max(0.0) / config.prestige_divisor).powf(config.prestige_exponent);
    (base
        * rates.factor(EffectStat::HoardPointGain)
        * percents.percent_multiplier(EffectStat::HoardPointGain))
    .floor()
}

#[cfg(test)]
mod tests;
