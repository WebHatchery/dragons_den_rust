//! Core economy formulas (GDD §5.1). One source of truth: both the live tick
//! and offline progress read these same functions.

use crate::data::{EffectStat, GameConfig, PrestigeUpgradeDef, UpgradeDef};
use crate::simulation::Bonuses;
use std::collections::HashMap;

/// Aggregates `level * rate` per stat from the run's upgrade levels.
pub fn rate_bonuses(defs: &[UpgradeDef], levels: &HashMap<String, u32>) -> Bonuses {
    let mut bonuses = Bonuses::default();
    for def in defs {
        let level = levels.get(&def.id).copied().unwrap_or(0);
        if level > 0 {
            bonuses.add(def.effect.stat, f64::from(level) * def.effect.rate);
        }
    }
    bonuses
}

/// Aggregates percent bonuses from prestige upgrade levels (percent scales
/// linearly with level; treasures and dragons contribute via
/// `state::gameplay`'s collection pass using [`Bonuses::add`] directly).
pub fn prestige_percent_bonuses(
    defs: &[PrestigeUpgradeDef],
    levels: &HashMap<String, u32>,
) -> Bonuses {
    let mut bonuses = Bonuses::default();
    for def in defs {
        let level = levels.get(&def.id).copied().unwrap_or(0);
        if level > 0 {
            bonuses.add(def.effect.stat, f64::from(level) * def.effect.percent);
        }
    }
    bonuses
}

pub fn gold_per_click(config: &GameConfig, rates: &Bonuses, percents: &Bonuses) -> f64 {
    config.base_click
        * rates.factor(EffectStat::GoldPerClick)
        * percents.percent_multiplier(EffectStat::GoldPerClick)
}

pub fn gold_per_second(
    config: &GameConfig,
    goblins: u32,
    rates: &Bonuses,
    percents: &Bonuses,
) -> f64 {
    let per_goblin = config.gold_per_goblin * rates.factor(EffectStat::MinionEfficiency);
    (config.base_passive + f64::from(goblins) * per_goblin)
        * percents.percent_multiplier(EffectStat::GoldPerSecond)
}

pub fn discovery_chance(config: &GameConfig, rates: &Bonuses, percents: &Bonuses) -> f64 {
    let chance = (config.base_discovery_chance + rates.sum(EffectStat::DiscoveryChance))
        * percents.percent_multiplier(EffectStat::DiscoveryChance);
    chance.clamp(0.0, 0.95)
}

/// `hire_cost(n) = base * growth^n` — kept from the original (50 * 1.2^n).
pub fn hire_cost(config: &GameConfig, goblins_owned: u32) -> f64 {
    (config.base_hire_cost * config.hire_cost_growth.powi(goblins_owned as i32)).floor()
}

/// `upgrade_cost(l) = floor(base * growth^l)` — kept from the original.
pub fn upgrade_cost(base_cost: f64, cost_growth: f64, level: u32) -> f64 {
    (base_cost * cost_growth.powi(level as i32)).floor()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;

    fn setup() -> (GameData, HashMap<String, u32>) {
        (GameData::load().unwrap(), HashMap::new())
    }

    #[test]
    fn click_income_scales_with_upgrade_level() {
        let (data, mut levels) = setup();
        let percents = Bonuses::default();

        let base = gold_per_click(
            &data.config,
            &rate_bonuses(&data.upgrades, &levels),
            &percents,
        );
        assert!((base - 1.0).abs() < 1e-9);

        levels.insert("click_power".to_owned(), 2);
        let upgraded = gold_per_click(
            &data.config,
            &rate_bonuses(&data.upgrades, &levels),
            &percents,
        );
        // base_click 1.0 * (1 + 2 * 0.5) = 2.0
        assert!((upgraded - 2.0).abs() < 1e-9);
    }

    #[test]
    fn passive_income_reads_minion_efficiency() {
        let (data, mut levels) = setup();
        let percents = Bonuses::default();

        let plain = gold_per_second(
            &data.config,
            10,
            &rate_bonuses(&data.upgrades, &levels),
            &percents,
        );
        assert!((plain - 10.0).abs() < 1e-9);

        levels.insert("minion_efficiency".to_owned(), 5);
        let efficient = gold_per_second(
            &data.config,
            10,
            &rate_bonuses(&data.upgrades, &levels),
            &percents,
        );
        // 10 goblins * 1.0 * (1 + 5 * 0.2) = 20.0
        assert!((efficient - 20.0).abs() < 1e-9);
    }

    #[test]
    fn percent_bonuses_multiply_income() {
        let (data, levels) = setup();
        let rates = rate_bonuses(&data.upgrades, &levels);
        let mut percents = Bonuses::default();
        percents.add(EffectStat::GoldPerClick, 25.0);

        let boosted = gold_per_click(&data.config, &rates, &percents);
        assert!((boosted - 1.25).abs() < 1e-9);
    }

    #[test]
    fn cost_curves_match_original_formulas() {
        let (data, _) = setup();
        assert!((hire_cost(&data.config, 0) - 50.0).abs() < 1e-9);
        assert!((hire_cost(&data.config, 2) - 72.0).abs() < 1e-9); // floor(50 * 1.44)
        assert!((upgrade_cost(100.0, 1.5, 0) - 100.0).abs() < 1e-9);
        assert!((upgrade_cost(100.0, 1.5, 3) - 337.0).abs() < 1e-9); // floor(337.5)
    }

    #[test]
    fn discovery_chance_is_clamped() {
        let (data, levels) = setup();
        let rates = rate_bonuses(&data.upgrades, &levels);
        let mut percents = Bonuses::default();
        percents.add(EffectStat::DiscoveryChance, 100000.0);
        assert!((discovery_chance(&data.config, &rates, &percents) - 0.95).abs() < 1e-9);
    }
}
