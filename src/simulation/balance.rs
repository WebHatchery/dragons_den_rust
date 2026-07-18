//! Balance regression: a headless greedy-player simulation that estimates how
//! long an *active* session takes to reach the prestige threshold
//! (GDD §5.4 target: "modest first session"). Test-only — it exercises the
//! real `economy` formulas so a balance change to `game_config.json` that
//! makes the first prestige unreachable (or trivial) fails CI. It also
//! simulates the full prestige loop — burn the hoard, spend Hoard Points on
//! the tree, run again — so CI guards the metric that defines fun in a
//! prestige game: run N+1 must be meaningfully faster than run N.

#![cfg(test)]

use crate::data::{EffectStat, GameData, UpgradeDef};
use crate::simulation::economy;
use crate::simulation::prestige;
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
    /// Permanent percent/multiplier bonuses (prestige tree); empty on run 1.
    percents: Bonuses,
    /// Prestiges already done — gates which upgrade lines the sim may buy (#10),
    /// so run 1 can't spend on lines the real player hasn't unlocked yet.
    prestige: u32,
}

impl Sim {
    fn new() -> Self {
        Self {
            gold: 0.0,
            goblins: 0,
            levels: HashMap::new(),
            percents: Bonuses::default(),
            prestige: 0,
        }
    }

    fn rates(&self, data: &GameData) -> Bonuses {
        economy::rate_bonuses(&data.upgrades, &self.levels)
    }

    fn gpc(&self, data: &GameData) -> f64 {
        economy::gold_per_click(&data.config, &self.rates(data), &self.percents)
    }

    fn gps(&self, data: &GameData) -> f64 {
        economy::gold_per_second(
            &data.config,
            self.goblins,
            &self.rates(data),
            &self.percents,
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
        let after = gain_rate_with(data, self.goblins, &probe, &self.percents);
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
        let after = gain_rate_with(data, self.goblins + 1, &self.levels, &self.percents);
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
fn gain_rate_with(
    data: &GameData,
    goblins: u32,
    levels: &HashMap<String, u32>,
    percents: &Bonuses,
) -> f64 {
    let rates = economy::rate_bonuses(&data.upgrades, levels);
    economy::gold_per_second(&data.config, goblins, &rates, percents)
        + economy::gold_per_click(&data.config, &rates, percents) * CLICK_RATE
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
            // Prestige-gated lines (#10) are invisible until unlocked — the sim
            // must respect the same gate the player does, or run 1 would "buy"
            // lines it can't reach and skew the balance guards.
            if def.prestige_required > sim.prestige {
                continue;
            }
            // Only income-boosting lines move the marginal gain rate. (The
            // hire-discount line lowers goblin cost, not income, so it never
            // clears the payback test here — a conservative omission that keeps
            // the reported time an upper bound on optimal play.)
            if !matches!(
                def.effect.stat,
                EffectStat::GoldPerClick | EffectStat::MinionEfficiency | EffectStat::GoldPerSecond
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

/// Simulated active seconds for `sim` to reach `target` gold from its current
/// state, greedily reinvesting along the way.
fn run_to_target(sim: &mut Sim, data: &GameData, target: f64) -> Option<f64> {
    let mut t = 0.0;
    while t < MAX_SIM_SECS {
        sim.gold += sim.gain_rate(data) * DT;
        reinvest(sim, data, target);
        if sim.gold >= target {
            return Some(t);
        }
        t += DT;
    }
    None
}

/// Greedily spends Hoard Points on the prestige tree, returning the purchased
/// levels. Each candidate is a *package* — any unbought prerequisite levels
/// plus one level of the node — scored by marginal gain-rate per point, so a
/// pure-multiplier node behind utility prereqs (Dragon's Avarice) competes
/// fairly with cheap standalone income nodes. Marginals are valued against the
/// end-of-run economy (`valuation`), the run shape the player just experienced.
fn spend_hoard_points(data: &GameData, mut points: f64, valuation: &Sim) -> HashMap<String, u32> {
    let mut tree: HashMap<String, u32> = HashMap::new();
    loop {
        let base_percents = economy::prestige_percent_bonuses(&data.prestige_upgrades, &tree);
        let base_rate = gain_rate_with(data, valuation.goblins, &valuation.levels, &base_percents);

        let mut best: Option<(f64, HashMap<String, u32>, f64)> = None; // (score, tree', cost)
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
                let p = data.prestige_upgrade(&id).unwrap();
                if tree.get(&id).copied().unwrap_or(0) == 0 {
                    cost += economy::upgrade_cost(p.base_cost, p.cost_growth, 0);
                    probe.insert(id.clone(), 1);
                }
                prereq = p.prereq.clone();
            }
            if cost > points {
                continue;
            }
            let probe_percents = economy::prestige_percent_bonuses(&data.prestige_upgrades, &probe);
            let marginal =
                gain_rate_with(data, valuation.goblins, &valuation.levels, &probe_percents)
                    - base_rate;
            if marginal <= 0.0 {
                continue;
            }
            let score = marginal / cost;
            if best.as_ref().map(|(s, _, _)| score > *s).unwrap_or(true) {
                best = Some((score, probe, cost));
            }
        }

        let Some((_, next_tree, cost)) = best else {
            return tree;
        };
        points -= cost;
        tree = next_tree;
    }
}

#[test]
fn first_prestige_reachable_in_a_modest_session() {
    let data = GameData::load().unwrap();
    let secs = run_to_target(&mut Sim::new(), &data, data.config.prestige_threshold)
        .expect("threshold should be reachable");
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

/// The metric that defines fun in a prestige game, and the regression the
/// engagement review found untested: after burning the hoard and spending the
/// points, the next run must reach *its own (higher) threshold* meaningfully
/// faster than the first run reached its threshold — otherwise prestiging is
/// rationally never worth doing and the loop decelerates into a grind.
#[test]
fn second_prestige_cycle_is_meaningfully_faster() {
    let data = GameData::load().unwrap();

    let mut run1 = Sim::new();
    let cycle1 = run_to_target(&mut run1, &data, data.config.prestige_threshold)
        .expect("first threshold should be reachable");

    // Burn the hoard exactly as `try_prestige` would.
    let points =
        prestige::hoard_points_gained(&data.config, run1.gold, &run1.rates(&data), &run1.percents);
    let tree = spend_hoard_points(&data, points, &run1);
    let spent: u32 = tree.values().sum();
    eprintln!("first prestige: {points:.0} HP, {spent} tree levels bought");
    assert!(
        spent >= 3,
        "first prestige should fund a real spree on the tree"
    );

    let mut run2 = Sim::new();
    run2.percents = economy::prestige_percent_bonuses(&data.prestige_upgrades, &tree);
    // After one prestige, run 2 may buy the prestige-1-gated upgrade lines (#10).
    run2.prestige = 1;
    let target2 = prestige::current_threshold(&data.config, 1);
    let cycle2 =
        run_to_target(&mut run2, &data, target2).expect("second threshold should be reachable");

    eprintln!(
        "cycle 1: {:.1} min to {} | cycle 2: {:.1} min to {} ({:.0}% of cycle 1)",
        cycle1 / 60.0,
        data.config.prestige_threshold,
        cycle2 / 60.0,
        target2,
        cycle2 / cycle1 * 100.0
    );
    assert!(
        cycle2 <= cycle1 * 0.7,
        "cycle 2 took {:.1} min vs cycle 1's {:.1} min — prestige does not accelerate the loop",
        cycle2 / 60.0,
        cycle1 / 60.0
    );
}
