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
mod tests {
    use super::*;
    use crate::data::GameData;

    #[test]
    fn first_prestige_grants_points() {
        let data = GameData::load().unwrap();
        let none = Bonuses::default();
        assert!(!can_prestige(&data.config, 999_999.0, 0));
        assert!(can_prestige(&data.config, 1_000_000.0, 0));
        let expected = (1_000_000.0 / data.config.prestige_divisor)
            .powf(data.config.prestige_exponent)
            .floor();
        let gained = hoard_points_gained(&data.config, 1_000_000.0, &none, &none);
        assert!((gained - expected).abs() < 1e-9);
        // A first prestige must fund a real opening spree on the tree, not one
        // token level (the un-fun the engagement review diagnosed).
        assert!(gained >= 50.0, "first prestige pays only {gained} HP");
    }

    #[test]
    fn overshooting_the_threshold_pays_superlinearly_vs_sqrt() {
        let data = GameData::load().unwrap();
        let none = Bonuses::default();
        let at = hoard_points_gained(&data.config, 1_000_000.0, &none, &none);
        let over = hoard_points_gained(&data.config, 4_000_000.0, &none, &none);
        // sqrt would pay exactly 2x for 4x gold; the softened exponent pays more,
        // making "push past the threshold before burning" a real decision.
        assert!(over > at * 2.0, "4x gold pays {over} vs {at} at threshold");
    }

    #[test]
    fn threshold_rises_with_each_prestige() {
        let data = GameData::load().unwrap();
        let base = data.config.prestige_threshold;
        let growth = data.config.prestige_threshold_growth;
        assert!((current_threshold(&data.config, 0) - base).abs() < 1e-6);
        assert!((current_threshold(&data.config, 1) - base * growth).abs() < 1e-3);
        assert!((current_threshold(&data.config, 2) - base * growth * growth).abs() < 1e-3);
        // A hoard that clears tier 0 need not clear tier 1.
        let tier0 = current_threshold(&data.config, 0);
        assert!(can_prestige(&data.config, tier0, 0));
        assert!(!can_prestige(&data.config, tier0, 1));
    }

    #[test]
    fn hoard_greed_scales_gain() {
        let data = GameData::load().unwrap();
        let mut rates = Bonuses::default();
        rates.add(EffectStat::HoardPointGain, 0.5);
        let none = Bonuses::default();
        let expected = ((1_000_000.0 / data.config.prestige_divisor)
            .powf(data.config.prestige_exponent)
            * 1.5)
            .floor();
        let gained = hoard_points_gained(&data.config, 1_000_000.0, &rates, &none);
        assert!((gained - expected).abs() < 1e-9);
    }
}
