//! Core economy formulas (GDD §5.1). One source of truth: both the live tick
//! and offline progress read these same functions.

use crate::data::{EffectStat, GameConfig, MinionDef, PrestigeUpgradeDef, UpgradeDef};
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
/// Compounding nodes feed the multiplicative channel instead:
/// `(1 + percent/100)^level`.
pub fn prestige_percent_bonuses(
    defs: &[PrestigeUpgradeDef],
    levels: &HashMap<String, u32>,
) -> Bonuses {
    let mut bonuses = Bonuses::default();
    for def in defs {
        let level = levels.get(&def.id).copied().unwrap_or(0);
        if level > 0 {
            if def.compounding {
                bonuses.mul(
                    def.effect.stat,
                    (1.0 + def.effect.percent / 100.0).powi(level as i32),
                );
            } else {
                bonuses.add(def.effect.stat, f64::from(level) * def.effect.percent);
            }
        }
    }
    bonuses
}

/// The `AllGold` stat's combined factor — both channels — applied to every
/// gold income source (click, base passive, extra minion tiers).
pub fn all_gold_multiplier(percents: &Bonuses) -> f64 {
    percents.product(EffectStat::AllGold) * percents.percent_multiplier(EffectStat::AllGold)
}

pub fn gold_per_click(config: &GameConfig, rates: &Bonuses, percents: &Bonuses) -> f64 {
    config.base_click
        * rates.factor(EffectStat::GoldPerClick)
        * percents.percent_multiplier(EffectStat::GoldPerClick)
        * all_gold_multiplier(percents)
}

/// Base-minion count past the soft cap suffers diminishing returns (#11): full
/// value up to `soft_cap`, then each extra minion is worth only `falloff` of
/// one. Wrapped in a helper so the live tick and the balance sim agree exactly.
pub fn effective_minions(goblins: f64, soft_cap: f64, falloff: f64) -> f64 {
    if goblins <= soft_cap {
        goblins
    } else {
        soft_cap + (goblins - soft_cap) * falloff
    }
}

/// The effective base-minion soft cap after prestige wall-breakers (#11): the
/// configured base scaled by the additive `MinionCap` percent channel.
pub fn minion_soft_cap(config: &GameConfig, percents: &Bonuses) -> f64 {
    config.minion_soft_cap * percents.percent_multiplier(EffectStat::MinionCap)
}

pub fn gold_per_second(
    config: &GameConfig,
    goblins: u32,
    rates: &Bonuses,
    percents: &Bonuses,
) -> f64 {
    let per_goblin = config.gold_per_goblin
        * rates.factor(EffectStat::MinionEfficiency)
        * percents.percent_multiplier(EffectStat::MinionEfficiency);
    let cap = minion_soft_cap(config, percents);
    let effective = effective_minions(f64::from(goblins), cap, config.minion_soft_cap_falloff);
    (config.base_passive + effective * per_goblin)
        * rates.factor(EffectStat::GoldPerSecond)
        * percents.percent_multiplier(EffectStat::GoldPerSecond)
        * all_gold_multiplier(percents)
}

/// Passive income from the extra minion tiers (P4), sharing the same efficiency
/// and gold/second modifiers as the base tier. Zero until such minions are hired,
/// so the base economy and balance sim are unaffected.
pub fn extra_minion_income(
    minions: &[MinionDef],
    counts: &HashMap<String, u32>,
    rates: &Bonuses,
    percents: &Bonuses,
) -> f64 {
    let raw: f64 = minions
        .iter()
        .map(|def| f64::from(counts.get(&def.id).copied().unwrap_or(0)) * def.base_rate)
        .sum();
    raw * rates.factor(EffectStat::MinionEfficiency)
        * percents.percent_multiplier(EffectStat::MinionEfficiency)
        * rates.factor(EffectStat::GoldPerSecond)
        * percents.percent_multiplier(EffectStat::GoldPerSecond)
        * all_gold_multiplier(percents)
}

/// Effective base goblin hire cost after run upgrades: the `HireDiscount` line
/// divides the whole curve by `1 + sum(level * rate)`, so hires get steadily
/// cheaper without ever reaching free. Feeds the same `upgrade_cost` /
/// `bulk_cost` / `affordable_levels` helpers via a scaled base.
pub fn hire_base_cost(config: &GameConfig, rates: &Bonuses) -> f64 {
    config.base_hire_cost / rates.factor(EffectStat::HireDiscount)
}

pub fn discovery_chance(config: &GameConfig, rates: &Bonuses, percents: &Bonuses) -> f64 {
    let chance = (config.base_discovery_chance + rates.sum(EffectStat::DiscoveryChance))
        * percents.percent_multiplier(EffectStat::DiscoveryChance);
    chance.clamp(0.0, 0.95)
}

/// Expedition cost, rising with the number of treasures already discovered so
/// late finds are a real investment instead of button-mash filler (engagement
/// review #6): `explore_cost(n) = floor(base * growth^n)`. Only actual
/// discoveries raise it — an unlucky (empty) expedition never does — so bad
/// luck is never punished with a higher price.
pub fn explore_cost(config: &GameConfig, discovered: usize) -> f64 {
    (config.explore_cost * config.explore_cost_growth.powi(discovered as i32)).floor()
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
        assert!((base - data.config.base_click).abs() < 1e-9);

        levels.insert("click_power".to_owned(), 2);
        let upgraded = gold_per_click(
            &data.config,
            &rate_bonuses(&data.upgrades, &levels),
            &percents,
        );
        // (1 + 2 * 0.5) = 2x the base click, whatever the tuned base is.
        assert!((upgraded - data.config.base_click * 2.0).abs() < 1e-9);
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
        assert!((plain - 10.0 * data.config.gold_per_goblin).abs() < 1e-9);

        levels.insert("minion_efficiency".to_owned(), 5);
        let efficient = gold_per_second(
            &data.config,
            10,
            &rate_bonuses(&data.upgrades, &levels),
            &percents,
        );
        // 10 goblins * per_goblin * (1 + 5 * 0.2) = 2x the plain rate.
        assert!((efficient - 20.0 * data.config.gold_per_goblin).abs() < 1e-9);
    }

    #[test]
    fn percent_bonuses_multiply_income() {
        let (data, levels) = setup();
        let rates = rate_bonuses(&data.upgrades, &levels);
        let mut percents = Bonuses::default();
        percents.add(EffectStat::GoldPerClick, 25.0);

        let boosted = gold_per_click(&data.config, &rates, &percents);
        assert!((boosted - data.config.base_click * 1.25).abs() < 1e-9);
    }

    #[test]
    fn cost_curves_use_config_and_floor_per_level() {
        let (data, _) = setup();
        let base = data.config.base_hire_cost;
        let growth = data.config.hire_cost_growth;
        // Hire level 0 is exactly the configured base; higher levels floor the
        // geometric curve. (Values follow config, not hardcoded constants.)
        assert!((upgrade_cost(base, growth, 0) - base).abs() < 1e-9);
        assert!((upgrade_cost(base, growth, 2) - (base * growth.powi(2)).floor()).abs() < 1e-9);
        // Config-independent formula spot-checks.
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
    fn wyrm_appetite_multiplies_passive_income() {
        let (data, mut levels) = setup();
        let percents = Bonuses::default();
        let plain = gold_per_second(
            &data.config,
            10,
            &rate_bonuses(&data.upgrades, &levels),
            &percents,
        );

        levels.insert("wyrm_appetite".to_owned(), 5);
        let boosted = gold_per_second(
            &data.config,
            10,
            &rate_bonuses(&data.upgrades, &levels),
            &percents,
        );
        // (1 + 5 * 0.12) = 1.6x the passive rate.
        assert!((boosted - plain * 1.6).abs() < 1e-9);
    }

    #[test]
    fn goblin_recruiters_discount_cheapens_hires() {
        let (data, mut levels) = setup();
        let full = hire_base_cost(&data.config, &rate_bonuses(&data.upgrades, &levels));
        assert!((full - data.config.base_hire_cost).abs() < 1e-9);

        levels.insert("goblin_recruiters".to_owned(), 10);
        let discounted = hire_base_cost(&data.config, &rate_bonuses(&data.upgrades, &levels));
        // 1 + 10 * 0.1 = 2x divisor → half price, and never free.
        assert!((discounted - data.config.base_hire_cost / 2.0).abs() < 1e-9);
        assert!(discounted > 0.0);
    }

    #[test]
    fn economy_stays_finite_at_scale() {
        let (data, mut levels) = setup();
        // Every gold-relevant line maxed.
        for def in &data.upgrades {
            levels.insert(def.id.clone(), def.max_level);
        }
        let rates = rate_bonuses(&data.upgrades, &levels);
        // Large stacked percent bonuses (all treasures + dragons + a deep tree).
        let mut percents = Bonuses::default();
        percents.add(EffectStat::GoldPerClick, 500.0);
        percents.add(EffectStat::GoldPerSecond, 500.0);

        // A hoard-scale goblin army.
        let gps = gold_per_second(&data.config, 1_000_000_000, &rates, &percents);
        let gpc = gold_per_click(&data.config, &rates, &percents);
        assert!(gps.is_finite() && gps > 0.0);
        assert!(gpc.is_finite() && gpc > 0.0);
        // Discovery stays within its clamp even with everything stacked.
        let chance = discovery_chance(&data.config, &rates, &percents);
        assert!((0.0..=0.95).contains(&chance));
    }

    #[test]
    fn explore_cost_rises_with_each_discovery() {
        let (data, _) = setup();
        let base = data.config.explore_cost;
        let growth = data.config.explore_cost_growth;
        assert!(growth > 1.0, "expedition cost must actually scale");

        // The first expedition (nothing discovered yet) is exactly the base.
        assert!((explore_cost(&data.config, 0) - base).abs() < 1e-9);
        // Each discovery multiplies the base by the floored geometric curve.
        assert!((explore_cost(&data.config, 3) - (base * growth.powi(3)).floor()).abs() < 1e-9);
        // Strictly increasing, so late treasures cost more than early ones.
        assert!(explore_cost(&data.config, 5) > explore_cost(&data.config, 4));
    }

    #[test]
    fn minion_soft_cap_diminishes_past_the_cap() {
        // Below/at the cap, minions count in full.
        assert!((effective_minions(40.0, 60.0, 0.34) - 40.0).abs() < 1e-9);
        assert!((effective_minions(60.0, 60.0, 0.34) - 60.0).abs() < 1e-9);
        // Past the cap, the excess is worth only `falloff` each.
        assert!((effective_minions(80.0, 60.0, 0.5) - 70.0).abs() < 1e-9); // 60 + 20*0.5
                                                                           // The cap itself scales with the MinionCap wall-breaker channel.
        let (data, _) = setup();
        let base = minion_soft_cap(&data.config, &Bonuses::default());
        assert!((base - data.config.minion_soft_cap).abs() < 1e-9);
        let mut breaker = Bonuses::default();
        breaker.add(EffectStat::MinionCap, 100.0); // +100% → double the cap
        assert!((minion_soft_cap(&data.config, &breaker) - base * 2.0).abs() < 1e-9);
    }

    #[test]
    fn passive_income_bends_at_the_soft_cap() {
        let (data, levels) = setup();
        let rates = rate_bonuses(&data.upgrades, &levels);
        let percents = Bonuses::default();
        // Just under the cap: linear in goblins.
        let cap = data.config.minion_soft_cap as u32;
        let under = gold_per_second(&data.config, cap, &rates, &percents);
        // Far past the cap earns less than a naive linear extrapolation would.
        let over = gold_per_second(&data.config, cap * 3, &rates, &percents);
        assert!(over > under, "more minions still earn more");
        assert!(
            over < under * 3.0,
            "but past the cap the third-of-the-army earns diminished income"
        );
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
