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

/// `floor(sqrt(gold / divisor))`, scaled by the Hoard Greed upgrade line and
/// any percent bonuses to hoard-point gain.
pub fn hoard_points_gained(
    config: &GameConfig,
    gold: f64,
    rates: &Bonuses,
    percents: &Bonuses,
) -> f64 {
    let base = (gold.max(0.0) / config.prestige_divisor).sqrt();
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
        // sqrt(1_000_000 / 10_000) = 10
        let gained = hoard_points_gained(&data.config, 1_000_000.0, &none, &none);
        assert!((gained - 10.0).abs() < 1e-9);
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
        let gained = hoard_points_gained(&data.config, 1_000_000.0, &rates, &none);
        assert!((gained - 15.0).abs() < 1e-9);
    }
}
