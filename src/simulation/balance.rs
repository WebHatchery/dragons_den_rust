//! Headless balance simulation for the prestige career.
//!
//! The simulator deliberately stays test-only, but exercises the production
//! economy, prestige, and offline formulas. It models an engaged greedy player:
//! run purchases compete by payback time, permanent Hoard Point purchases carry
//! between cycles, and each run stops at its current prestige threshold.

#![cfg(test)]

use crate::data::{EffectStat, GameData, MinionDef, UpgradeDef};
use crate::simulation::{economy, offline, prestige, Bonuses};
use std::collections::HashMap;

const CLICK_RATE: f64 = 5.0;
const MAX_SIM_SECS: f64 = 30.0 * 24.0 * 3600.0;

#[derive(Clone)]
struct Sim {
    gold: f64,
    goblins: u32,
    minions: HashMap<String, u32>,
    levels: HashMap<String, u32>,
    percents: Bonuses,
    prestige: u32,
    purchases: HashMap<String, u32>,
}

impl Sim {
    fn new(prestige: u32, percents: Bonuses) -> Self {
        Self {
            gold: 0.0,
            goblins: 0,
            minions: HashMap::new(),
            levels: HashMap::new(),
            percents,
            prestige,
            purchases: HashMap::new(),
        }
    }

    fn rates(&self, data: &GameData) -> Bonuses {
        economy::rate_bonuses(&data.upgrades, &self.levels)
    }

    fn passive_rate(&self, data: &GameData) -> f64 {
        passive_rate_with(
            data,
            self.goblins,
            &self.minions,
            &self.levels,
            &self.percents,
        )
    }

    fn gain_rate(&self, data: &GameData) -> f64 {
        gain_rate_with(
            data,
            self.goblins,
            &self.minions,
            &self.levels,
            &self.percents,
        )
    }

    fn total_minions(&self) -> u32 {
        self.goblins + self.minions.values().sum::<u32>()
    }

    fn upgrade_marginal(
        &self,
        data: &GameData,
        def: &UpgradeDef,
        before: f64,
    ) -> Option<(f64, f64)> {
        let level = self.levels.get(&def.id).copied().unwrap_or(0);
        if level >= def.max_level {
            return None;
        }
        let cost = economy::upgrade_cost(def.base_cost, def.cost_growth, level);
        let mut levels = self.levels.clone();
        levels.insert(def.id.clone(), level + 1);
        let marginal =
            gain_rate_with(data, self.goblins, &self.minions, &levels, &self.percents) - before;
        (marginal > 0.0).then_some((cost, marginal))
    }

    fn goblin_marginal(&self, data: &GameData, before: f64) -> (f64, f64) {
        let rates = self.rates(data);
        let base = economy::hire_base_cost(&data.config, &rates);
        let cost = economy::upgrade_cost(base, data.config.hire_cost_growth, self.goblins);
        let marginal = gain_rate_with(
            data,
            self.goblins + 1,
            &self.minions,
            &self.levels,
            &self.percents,
        ) - before;
        (cost, marginal.max(1e-9))
    }

    fn minion_marginal(&self, data: &GameData, def: &MinionDef, before: f64) -> Option<(f64, f64)> {
        if self.prestige < def.prestige_required || self.total_minions() < def.unlock_at {
            return None;
        }
        let count = self.minions.get(&def.id).copied().unwrap_or(0);
        let rates = self.rates(data);
        let base = economy::apply_hire_discount(def.base_cost, &rates);
        let cost = economy::upgrade_cost(base, def.cost_growth, count);
        let mut minions = self.minions.clone();
        minions.insert(def.id.clone(), count + 1);
        let marginal =
            gain_rate_with(data, self.goblins, &minions, &self.levels, &self.percents) - before;
        Some((cost, marginal.max(1e-9)))
    }

    fn record_purchase(&mut self, key: &str) {
        *self.purchases.entry(key.to_owned()).or_default() += 1;
    }
}

fn passive_rate_with(
    data: &GameData,
    goblins: u32,
    minions: &HashMap<String, u32>,
    levels: &HashMap<String, u32>,
    percents: &Bonuses,
) -> f64 {
    let rates = economy::rate_bonuses(&data.upgrades, levels);
    economy::gold_per_second(&data.config, goblins, &rates, percents)
        + economy::extra_minion_income(&data.minions, minions, &rates, percents)
}

fn gain_rate_with(
    data: &GameData,
    goblins: u32,
    minions: &HashMap<String, u32>,
    levels: &HashMap<String, u32>,
    percents: &Bonuses,
) -> f64 {
    let rates = economy::rate_bonuses(&data.upgrades, levels);
    economy::gold_per_second(&data.config, goblins, &rates, percents)
        + economy::extra_minion_income(&data.minions, minions, &rates, percents)
        + economy::gold_per_click(&data.config, &rates, percents) * CLICK_RATE
}

#[derive(Clone)]
enum Purchase {
    Goblin,
    Minion(String),
    Upgrade(String),
}

fn candidates(sim: &Sim, data: &GameData) -> Vec<(Purchase, f64, f64)> {
    let mut choices = Vec::new();
    let before = sim.gain_rate(data);
    let (cost, marginal) = sim.goblin_marginal(data, before);
    choices.push((Purchase::Goblin, cost, marginal));

    for def in &data.minions {
        if let Some((cost, marginal)) = sim.minion_marginal(data, def, before) {
            choices.push((Purchase::Minion(def.id.clone()), cost, marginal));
        }
    }
    for def in &data.upgrades {
        if def.prestige_required > sim.prestige
            || !matches!(
                def.effect.stat,
                EffectStat::GoldPerClick | EffectStat::MinionEfficiency | EffectStat::GoldPerSecond
            )
        {
            continue;
        }
        if let Some((cost, marginal)) = sim.upgrade_marginal(data, def, before) {
            choices.push((Purchase::Upgrade(def.id.clone()), cost, marginal));
        }
    }
    choices
}

/// Reinvests in the affordable purchase with the shortest payback, provided it
/// repays before coasting to the threshold. Returns the cheapest future price,
/// allowing the outer simulation to jump directly to the next decision.
fn reinvest(sim: &mut Sim, data: &GameData, target: f64) -> Option<f64> {
    loop {
        let rate = sim.gain_rate(data).max(1e-9);
        let horizon = ((target - sim.gold).max(0.0)) / rate;
        let choices = candidates(sim, data);
        let mut best: Option<(f64, Purchase, f64)> = None;
        let mut next_cost: Option<f64> = None;

        for (purchase, cost, marginal) in choices {
            if cost > sim.gold {
                next_cost = Some(next_cost.map_or(cost, |current| current.min(cost)));
                continue;
            }
            let payback = cost / marginal;
            if payback <= horizon
                && best
                    .as_ref()
                    .map(|(current, _, _)| payback < *current)
                    .unwrap_or(true)
            {
                best = Some((payback, purchase, cost));
            }
        }

        let Some((_, purchase, cost)) = best else {
            return next_cost;
        };
        sim.gold -= cost;
        match purchase {
            Purchase::Goblin => {
                sim.goblins += 1;
                sim.record_purchase("kobold");
            }
            Purchase::Minion(id) => {
                *sim.minions.entry(id.clone()).or_default() += 1;
                sim.record_purchase(&id);
            }
            Purchase::Upgrade(id) => {
                *sim.levels.entry(id.clone()).or_default() += 1;
                sim.record_purchase(&id);
            }
        }
    }
}

fn run_to_target(sim: &mut Sim, data: &GameData, target: f64) -> Option<f64> {
    let mut elapsed = 0.0;
    while elapsed < MAX_SIM_SECS {
        let next_cost = reinvest(sim, data, target);
        if sim.gold >= target {
            return Some(elapsed);
        }
        let rate = sim.gain_rate(data).max(1e-9);
        let to_target = (target - sim.gold) / rate;
        let to_purchase = next_cost
            .filter(|cost| *cost > sim.gold)
            .map(|cost| (cost - sim.gold) / rate)
            .unwrap_or(to_target);
        let step = to_target.min(to_purchase).max(1e-9);
        sim.gold += rate * step;
        elapsed += step;
    }
    None
}

/// Adds income-valued permanent purchases to an existing tree. Prerequisite
/// packages are bought atomically and unspent points remain banked.
fn spend_hoard_points(
    data: &GameData,
    points: &mut f64,
    tree: &mut HashMap<String, u32>,
    valuation: &Sim,
) -> u32 {
    let before_levels: u32 = tree.values().sum();
    loop {
        let base_percents = economy::prestige_percent_bonuses(&data.prestige_upgrades, tree);
        let base_rate = gain_rate_with(
            data,
            valuation.goblins,
            &valuation.minions,
            &valuation.levels,
            &base_percents,
        );
        let mut best: Option<(f64, HashMap<String, u32>, f64)> = None;

        for def in &data.prestige_upgrades {
            let level = tree.get(&def.id).copied().unwrap_or(0);
            if level >= def.max_level {
                continue;
            }
            let mut cost = economy::upgrade_cost(def.base_cost, def.cost_growth, level);
            let mut probe = tree.clone();
            probe.insert(def.id.clone(), level + 1);
            let mut prereq = def.prereq.clone();
            while let Some(id) = prereq {
                let prereq_def = data.prestige_upgrade(&id).unwrap();
                if tree.get(&id).copied().unwrap_or(0) == 0 {
                    cost += economy::upgrade_cost(prereq_def.base_cost, prereq_def.cost_growth, 0);
                    probe.insert(id.clone(), 1);
                }
                prereq = prereq_def.prereq.clone();
            }
            if cost > *points {
                continue;
            }
            let percents = economy::prestige_percent_bonuses(&data.prestige_upgrades, &probe);
            let marginal = gain_rate_with(
                data,
                valuation.goblins,
                &valuation.minions,
                &valuation.levels,
                &percents,
            ) - base_rate;
            if marginal <= 0.0 {
                continue;
            }
            let score = marginal / cost;
            if best
                .as_ref()
                .map(|(current, _, _)| score > *current)
                .unwrap_or(true)
            {
                best = Some((score, probe, cost));
            }
        }

        let Some((_, next_tree, cost)) = best else {
            break;
        };
        *points -= cost;
        *tree = next_tree;
    }
    tree.values().sum::<u32>() - before_levels
}

struct CycleReport {
    prestige: u32,
    seconds: f64,
    threshold: f64,
    hp_gained: f64,
    tree_levels: u32,
    offline_share: f64,
    purchases: HashMap<String, u32>,
}

fn simulate_career(data: &GameData, count: u32) -> Vec<CycleReport> {
    let mut reports = Vec::new();
    let mut tree = HashMap::new();
    let mut banked_points = 0.0;

    for prestige_count in 0..count {
        let percents = economy::prestige_percent_bonuses(&data.prestige_upgrades, &tree);
        let mut sim = Sim::new(prestige_count, percents);
        let threshold = prestige::current_threshold(&data.config, prestige_count);
        let seconds = run_to_target(&mut sim, data, threshold).unwrap_or(f64::INFINITY);
        let cap_seconds = data.config.offline_cap_hours * 3600.0;
        let offline_gold = offline::offline_gold(sim.passive_rate(data), cap_seconds, cap_seconds);
        let hp_gained =
            prestige::hoard_points_gained(&data.config, sim.gold, &sim.rates(data), &sim.percents);
        banked_points += hp_gained;
        spend_hoard_points(data, &mut banked_points, &mut tree, &sim);
        reports.push(CycleReport {
            prestige: prestige_count + 1,
            seconds,
            threshold,
            hp_gained,
            tree_levels: tree.values().sum(),
            offline_share: offline_gold / threshold,
            purchases: sim.purchases,
        });
    }
    reports
}

#[cfg(test)]
mod tests;
