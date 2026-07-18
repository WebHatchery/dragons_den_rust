//! Balance regression: a headless greedy-player simulation that estimates how
//! long an *active* first session takes to reach the prestige threshold
//! (GDD §5.4 target: "modest first session"). Test-only — it exercises the
//! real `economy` formulas so a balance change to `game_config.json` that
//! makes the first prestige unreachable (or trivial) fails CI.

#![cfg(test)]

use crate::data::{EffectStat, GameData, UpgradeDef};
use crate::simulation::economy;
use crate::simulation::Bonuses;
use std::collections::HashMap;

/// Clicks per second an engaged player sustains during active play.
const CLICK_RATE: f64 = 5.0;
/// Simulation timestep.
const DT: f64 = 0.25;
/// Give up after this much simulated active play.
const MAX_SIM_SECS: f64 = 6.0 * 3600.0;

struct Sim {
    gold: f64,
    goblins: u32,
    levels: HashMap<String, u32>,
}

impl Sim {
    fn new() -> Self {
        Self {
            gold: 0.0,
            goblins: 0,
            levels: HashMap::new(),
        }
    }

    fn rates(&self, data: &GameData) -> Bonuses {
        economy::rate_bonuses(&data.upgrades, &self.levels)
    }

    fn gpc(&self, data: &GameData) -> f64 {
        economy::gold_per_click(&data.config, &self.rates(data), &Bonuses::default())
    }

    fn gps(&self, data: &GameData) -> f64 {
        economy::gold_per_second(
            &data.config,
            self.goblins,
            &self.rates(data),
            &Bonuses::default(),
        )
    }

    /// Passive-equivalent gold/sec gained by the next level of `def`, valuing
    /// click upgrades through `CLICK_RATE`. `None` when maxed.
    fn upgrade_marginal(&self, data: &GameData, def: &UpgradeDef) -> Option<(f64, f64)> {
        let level = self.levels.get(&def.id).copied().unwrap_or(0);
        if level >= def.max_level {
            return None;
        }
        let cost = economy::upgrade_cost(def.base_cost, def.cost_growth, level);
        let before = self.gain_rate(data);
        let mut probe = self.clone_levels();
        probe.insert(def.id.clone(), level + 1);
        let after = gain_rate_with(data, self.goblins, &probe);
        let marginal = after - before;
        if marginal <= 0.0 {
            None
        } else {
            Some((cost, marginal))
        }
    }

    fn goblin_marginal(&self, data: &GameData) -> (f64, f64) {
        let cost = economy::upgrade_cost(
            data.config.base_hire_cost,
            data.config.hire_cost_growth,
            self.goblins,
        );
        let before = self.gain_rate(data);
        let after = gain_rate_with(data, self.goblins + 1, &self.levels);
        (cost, (after - before).max(1e-9))
    }

    /// Total effective gold/sec: passive plus active clicking.
    fn gain_rate(&self, data: &GameData) -> f64 {
        self.gps(data) + self.gpc(data) * CLICK_RATE
    }

    fn clone_levels(&self) -> HashMap<String, u32> {
        self.levels.clone()
    }
}

/// Effective gold/sec for a hypothetical (goblins, levels) — for marginal calc.
fn gain_rate_with(data: &GameData, goblins: u32, levels: &HashMap<String, u32>) -> f64 {
    let rates = economy::rate_bonuses(&data.upgrades, levels);
    let empty = Bonuses::default();
    economy::gold_per_second(&data.config, goblins, &rates, &empty)
        + economy::gold_per_click(&data.config, &rates, &empty) * CLICK_RATE
}

/// Buys the best-payback affordable purchase repeatedly until none pays back
/// before the player would otherwise coast to the threshold. Using the coast
/// time (`target / current_income`) as the horizon models "only invest if it
/// beats just waiting": aggressive early (income low → long coast), tapering
/// to pure accumulation as income climbs. Upgrades and goblins compete on the
/// same payback metric.
fn reinvest(sim: &mut Sim, data: &GameData, target: f64) {
    loop {
        let horizon = target / sim.gain_rate(data).max(1e-9);
        let mut best: Option<(f64, String)> = None; // (payback, key)
        let consider = |cost: f64, marginal: f64, key: &str, best: &mut Option<(f64, String)>| {
            if cost > sim.gold {
                return;
            }
            let payback = cost / marginal;
            if payback <= horizon && best.as_ref().map(|(p, _)| payback < *p).unwrap_or(true) {
                *best = Some((payback, key.to_owned()));
            }
        };

        let (gcost, gmarg) = sim.goblin_marginal(data);
        consider(gcost, gmarg, "goblin", &mut best);
        for def in &data.upgrades {
            // Only gold-relevant lines matter for reaching the threshold.
            if !matches!(
                def.effect.stat,
                EffectStat::GoldPerClick | EffectStat::MinionEfficiency
            ) {
                continue;
            }
            if let Some((cost, marg)) = sim.upgrade_marginal(data, def) {
                consider(cost, marg, &def.id, &mut best);
            }
        }

        let Some((_, key)) = best else { break };
        if key == "goblin" {
            sim.gold -= gcost;
            sim.goblins += 1;
        } else {
            let level = sim.levels.get(&key).copied().unwrap_or(0);
            let def = data.upgrade(&key).unwrap();
            let cost = economy::upgrade_cost(def.base_cost, def.cost_growth, level);
            sim.gold -= cost;
            sim.levels.insert(key, level + 1);
        }
    }
}

/// Simulated active seconds to first reach the prestige threshold.
fn seconds_to_first_prestige(data: &GameData) -> Option<f64> {
    let mut sim = Sim::new();
    let target = data.config.prestige_threshold;
    let mut t = 0.0;
    while t < MAX_SIM_SECS {
        sim.gold += sim.gain_rate(data) * DT;
        reinvest(&mut sim, data, target);
        if sim.gold >= target {
            return Some(t);
        }
        t += DT;
    }
    None
}

#[test]
fn first_prestige_reachable_in_a_modest_session() {
    let data = GameData::load().unwrap();
    let secs = seconds_to_first_prestige(&data).expect("threshold should be reachable");
    let minutes = secs / 60.0;
    // Print with `cargo test balance -- --nocapture` while tuning.
    eprintln!("time to first prestige (active play): {minutes:.1} min");
    // A first prestige should feel earned but not grindy: roughly 10–40 min of
    // engaged play. These bounds guard against future balance regressions.
    assert!(
        (10.0..=40.0).contains(&minutes),
        "first prestige at {minutes:.1} min is outside the 10–40 min target window"
    );
}
