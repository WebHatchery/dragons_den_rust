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

/// Divides any minion's base hire cost by the `HireDiscount` channel
/// (`1 / (1 + sum(level * rate))`), so discount upgrades cheapen a hire without
/// ever reaching free. Shared by the base Kobold curve and the typed minion
/// tiers so a single stat cheapens the *whole* army, not just Kobolds.
pub fn apply_hire_discount(base_cost: f64, rates: &Bonuses) -> f64 {
    base_cost / rates.factor(EffectStat::HireDiscount)
}

/// Effective base goblin hire cost after run upgrades: the `HireDiscount` line
/// divides the whole curve, so hires get steadily cheaper without ever reaching
/// free. Feeds the same `upgrade_cost` / `bulk_cost` / `affordable_levels`
/// helpers via a scaled base.
pub fn hire_base_cost(config: &GameConfig, rates: &Bonuses) -> f64 {
    apply_hire_discount(config.base_hire_cost, rates)
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
mod tests;
