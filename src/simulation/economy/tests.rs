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
