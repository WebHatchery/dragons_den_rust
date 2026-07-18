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

/// `upgrade_cost(l) = floor(base * growth^l)` — kept from the original. Hire
/// costs share this curve via `config.base_hire_cost` / `hire_cost_growth`.
pub fn upgrade_cost(base_cost: f64, cost_growth: f64, level: u32) -> f64 {
    (base_cost * cost_growth.powi(level as i32)).floor()
}

/// Total cost to buy `count` successive levels starting from `start_level`,
/// summing the per-level floored cost so bulk buys match buying one at a time.
/// Both hire and upgrade share the `floor(base * growth^level)` curve, so this
/// serves both.
pub fn bulk_cost(base_cost: f64, cost_growth: f64, start_level: u32, count: u32) -> f64 {
    (0..count)
        .map(|i| upgrade_cost(base_cost, cost_growth, start_level + i))
        .sum()
}

/// How many successive levels are affordable with `budget`, capped at `max`.
/// Returns `(count, total_cost)`; costs grow geometrically so the loop
/// terminates quickly even for a `u32::MAX` cap.
pub fn affordable_levels(
    base_cost: f64,
    cost_growth: f64,
    start_level: u32,
    budget: f64,
    max: u32,
) -> (u32, f64) {
    let mut spent = 0.0;
    let mut count = 0;
    while count < max {
        let next = upgrade_cost(base_cost, cost_growth, start_level + count);
        if spent + next > budget {
            break;
        }
        spent += next;
        count += 1;
    }
    (count, spent)
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
        let base = data.config.base_hire_cost;
        let growth = data.config.hire_cost_growth;
        assert!((upgrade_cost(base, growth, 0) - 50.0).abs() < 1e-9);
        assert!((upgrade_cost(base, growth, 2) - 72.0).abs() < 1e-9); // floor(50 * 1.44)
        assert!((upgrade_cost(100.0, 1.5, 0) - 100.0).abs() < 1e-9);
        assert!((upgrade_cost(100.0, 1.5, 3) - 337.0).abs() < 1e-9); // floor(337.5)
    }

    #[test]
    fn bulk_cost_sums_per_level_floors() {
        // Three levels from 0 at 100 * 1.5^l: 100 + 150 + 225 = 475.
        assert!((bulk_cost(100.0, 1.5, 0, 3) - 475.0).abs() < 1e-9);
        // Buying zero costs nothing.
        assert!(bulk_cost(100.0, 1.5, 0, 0).abs() < 1e-9);
        // Starting partway matches the single-level curve.
        assert!((bulk_cost(100.0, 1.5, 3, 1) - upgrade_cost(100.0, 1.5, 3)).abs() < 1e-9);
    }

    #[test]
    fn affordable_levels_stops_at_budget() {
        // Budget 470 buys 2 levels (100 + 150 = 250, next 225 → 475 > 470).
        let (count, cost) = affordable_levels(100.0, 1.5, 0, 470.0, 100);
        assert_eq!(count, 2);
        assert!((cost - 250.0).abs() < 1e-9);

        // Budget 475 buys exactly 3.
        let (count, cost) = affordable_levels(100.0, 1.5, 0, 475.0, 100);
        assert_eq!(count, 3);
        assert!((cost - 475.0).abs() < 1e-9);

        // The `max` cap wins when budget is ample.
        let (count, _) = affordable_levels(100.0, 1.5, 0, 1e12, 5);
        assert_eq!(count, 5);

        // Nothing affordable → zero, no spend.
        let (count, cost) = affordable_levels(100.0, 1.5, 0, 50.0, 10);
        assert_eq!(count, 0);
        assert!(cost.abs() < 1e-9);
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
