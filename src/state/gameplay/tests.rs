//! Unit tests for [`super::GameplayState`]. Split into a child module (child of
//! `gameplay`, so it still reaches private items via `super`) to keep
//! `gameplay.rs` under the file-size limit as the state surface grows.

use super::*;
use crate::data::StatKey;
use crate::save;

fn setup() -> (GameData, GameplayState) {
    let data = GameData::load().unwrap();
    let state = GameplayState::new_game(&data, 1234);
    (data, state)
}

#[test]
fn m1_loop_click_hire_upgrade_prestige() {
    let (data, mut state) = setup();

    // Click pays base_click and counts toward lifetime stats.
    let gained = state.click(&data);
    assert!((gained - data.config.base_click).abs() < 1e-9);
    assert!((state.run.gold - data.config.base_click).abs() < 1e-9);

    // Hire a goblin, passive income becomes real.
    state.run.gold = 100.0;
    let (hired, _) = state.try_hire_bulk(&data, 1).unwrap();
    assert_eq!(hired, 1);
    assert_eq!(state.run.goblins, 1);
    assert!(state.gold_per_second(&data) > 0.0);

    // Buy a click upgrade; the formula actually reads it (GDD fix).
    state.run.gold = 1000.0;
    state.try_buy_upgrade_bulk(&data, "click_power", 1).unwrap();
    assert!(state.gold_per_click(&data) > data.config.base_click);

    // Prestige converts the hoard into Hoard Points and resets the run.
    state.run.gold = 1_000_000.0;
    let goblins_before = state.run.goblins;
    let gained = state.try_prestige(&data).unwrap();
    assert!(gained >= 10.0);
    assert!(goblins_before > 0);
    assert_eq!(state.run.goblins, 0);
    assert!((state.run.gold).abs() < 1e-9);
    assert_eq!(state.persistent.prestige_count, 1);
    assert!(state.persistent.hoard_points >= 10.0);
}

#[test]
fn bulk_hire_buys_as_many_as_affordable() {
    let (data, mut state) = setup();
    state.run.gold = 200.0;
    let (hired, cost) = state.try_hire_bulk(&data, u32::MAX).unwrap();
    assert!(hired >= 1);
    assert_eq!(state.run.goblins, hired);
    // Cost matches the summed per-level curve and leaves the change behind.
    let expected = economy::bulk_cost(
        data.config.base_hire_cost,
        data.config.hire_cost_growth,
        0,
        hired,
    );
    assert!((cost - expected).abs() < 1e-9);
    assert!((state.run.gold - (200.0 - expected)).abs() < 1e-9);
    // "Max" means the very next goblin is no longer affordable.
    let next = economy::upgrade_cost(
        data.config.base_hire_cost,
        data.config.hire_cost_growth,
        hired,
    );
    assert!(next > state.run.gold);
}

#[test]
fn bulk_upgrade_respects_max_level_and_budget() {
    let (data, mut state) = setup();
    state.run.gold = 1e12;
    // Buying "max" cannot exceed the definition's max_level.
    let def_max = data.upgrade("click_power").unwrap().max_level;
    let (bought, new_level) = state
        .try_buy_upgrade_bulk(&data, "click_power", u32::MAX)
        .unwrap();
    assert_eq!(new_level, def_max);
    assert_eq!(bought, def_max);
    // Already maxed → MaxLevel error.
    assert_eq!(
        state.try_buy_upgrade_bulk(&data, "click_power", 1),
        Err(BuyError::MaxLevel)
    );
}

#[test]
fn upgrade_lines_gate_behind_prestige() {
    let (data, mut state) = setup();
    let gated = data
        .upgrades
        .iter()
        .find(|u| u.prestige_required > 0)
        .expect("catalog should include a prestige-gated upgrade line");
    state.run.gold = 1e12;

    // Locked at prestige 0: the line is not unlocked and buying it is refused.
    assert!(!state.upgrade_unlocked(gated));
    assert_eq!(
        state.try_buy_upgrade_bulk(&data, &gated.id, 1),
        Err(BuyError::CannotAfford)
    );

    // Reaching the required prestige opens the line for purchase.
    state.persistent.prestige_count = gated.prestige_required;
    assert!(state.upgrade_unlocked(gated));
    let (bought, level) = state.try_buy_upgrade_bulk(&data, &gated.id, 1).unwrap();
    assert_eq!(bought, 1);
    assert_eq!(level, 1);
}

#[test]
fn prestige_below_threshold_is_refused() {
    let (data, mut state) = setup();
    state.run.gold = 999_999.0;
    assert!(state.try_prestige(&data).is_none());
    assert_eq!(state.persistent.prestige_count, 0);
}

#[test]
fn prestige_upgrades_persist_and_boost_next_run() {
    let (data, mut state) = setup();
    state.persistent.hoard_points = 10.0;
    state
        .try_buy_prestige_upgrade(&data, "ancient_claws")
        .unwrap();
    assert!((state.persistent.hoard_points - 5.0).abs() < 1e-9);

    state.run.gold = 1_000_000.0;
    state.try_prestige(&data).unwrap();
    // +25% permanent click bonus survives the reset.
    assert!((state.gold_per_click(&data) - data.config.base_click * 1.25).abs() < 1e-9);
}

#[test]
fn expedition_cost_scales_and_complete_set_is_free() {
    let (data, mut state) = setup();
    // Guaranteed discovery so each successful expedition raises the cost.
    state.run.gold = 1e12;
    let first_cost = state.explore_cost(&data);
    assert!((first_cost - data.config.explore_cost).abs() < 1e-9);

    // Discover one treasure at a 100% find rate; the next cost is higher.
    let discovered_before = state.persistent.discovered_treasures.len();
    // Force a certain find by exhausting the roll against a full catalog is
    // awkward, so drive it directly through the discovery list instead.
    state
        .persistent
        .discovered_treasures
        .push(data.treasures[0].id.clone());
    let second_cost = state.explore_cost(&data);
    assert!(second_cost > first_cost, "cost must rise after a discovery");
    assert_eq!(
        state.persistent.discovered_treasures.len(),
        discovered_before + 1
    );

    // Complete the set: exploring is refused-free — no gold is spent.
    state.persistent.discovered_treasures = data.treasures.iter().map(|d| d.id.clone()).collect();
    let gold_before = state.run.gold;
    assert_eq!(state.try_explore(&data), Ok(ExploreResult::AllDiscovered));
    assert!((state.run.gold - gold_before).abs() < 1e-9);
}

#[test]
fn expedition_without_new_treasure_grants_hoard_rush() {
    let (data, mut state) = setup();
    state.run.goblins = 10;

    // Complete the set so the roll is a guaranteed AllDiscovered, then read
    // the un-rushed rate *with* those treasure bonuses already applied.
    state.persistent.discovered_treasures = data.treasures.iter().map(|d| d.id.clone()).collect();
    let rate_no_rush = state.gold_per_second(&data);
    assert!(!state.hoard_rush_active());

    state.run.gold = 1e9;
    let gold_before = state.run.gold;
    assert_eq!(state.try_explore(&data), Ok(ExploreResult::AllDiscovered));

    // The surge is live, income is multiplied, and the salvage run was free.
    assert!(state.hoard_rush_active());
    assert!((state.run.gold - gold_before).abs() < 1e-9);
    let mult = data.config.hoard_rush_multiplier;
    assert!((state.gold_per_second(&data) - rate_no_rush * mult).abs() < 1e-6);

    // Re-triggering refreshes the timer but never stacks — still ×mult, not
    // ×mult².
    state.try_explore(&data).unwrap();
    assert!((state.gold_per_second(&data) - rate_no_rush * mult).abs() < 1e-6);

    // It drains with time and expires back to the base rate.
    state.tick(&data, data.config.hoard_rush_seconds as f32 + 0.1);
    assert!(!state.hoard_rush_active());
    assert!((state.gold_per_second(&data) - rate_no_rush).abs() < 1e-6);
}

#[test]
fn treasure_find_rush_lengthens_with_rarity() {
    let (data, state) = setup();

    // A Common find matches the base salvage/miss rush; each rarer tier lasts
    // strictly longer, so the rarity ladder is felt in the moment of the find.
    let base = data.config.hoard_rush_seconds;
    assert!((state.treasure_find_rush_seconds(&data, Rarity::Common) - base).abs() < 1e-9);
    use crate::data::Rarity;
    let durations: Vec<f64> = [
        Rarity::Common,
        Rarity::Rare,
        Rarity::Epic,
        Rarity::Legendary,
        Rarity::Mythic,
    ]
    .iter()
    .map(|&r| state.treasure_find_rush_seconds(&data, r))
    .collect();
    for pair in durations.windows(2) {
        assert!(pair[1] > pair[0], "rarer finds must sustain a longer rush");
    }
}

#[test]
fn a_treasure_find_ignites_a_rarity_scaled_hoard_rush() {
    let (data, mut state) = setup();
    state.run.gold = 1e9;
    assert!(!state.hoard_rush_active());

    // Explore until an actual find lands (seeded RNG makes this terminate fast),
    // then confirm the find both surged the income buff and set its timer to the
    // duration the found treasure's rarity dictates.
    let mut found = None;
    for _ in 0..500 {
        if let Ok(ExploreResult::Found {
            name, rush_seconds, ..
        }) = state.try_explore(&data)
        {
            found = Some((name, rush_seconds));
            break;
        }
    }
    let (name, rush_seconds) = found.expect("an expedition should eventually find a treasure");

    assert!(state.hoard_rush_active(), "a find must ignite a Hoard Rush");
    let rarity = data
        .treasures
        .iter()
        .find(|t| t.name == name)
        .expect("found treasure resolves in the catalog")
        .rarity;
    let expected = state.treasure_find_rush_seconds(&data, rarity);
    assert!(
        (rush_seconds - expected).abs() < 1e-9,
        "reported rush {rush_seconds} should match the rarity-scaled {expected}"
    );
    assert!(state.hoard_rush_remaining() <= expected + 1e-9);
}

#[test]
fn a_live_hoard_rush_makes_expeditions_luckier() {
    let (data, mut state) = setup();
    let base = state.discovery_chance(&data);
    assert!(!state.hoard_rush_active());

    // Ignite a rush directly so only the rush flag changes (discovering treasures
    // would also shift the percent bonuses and muddy the comparison), then confirm
    // the find chance rose by exactly the configured lucky-window bonus, still
    // under the 0.95 ceiling — closing the explore→rush→explore loop.
    state.trigger_hoard_rush(data.config.hoard_rush_seconds);
    assert!(state.hoard_rush_active());

    let boosted = state.discovery_chance(&data);
    let expected = (base + data.config.hoard_rush_discovery_bonus).min(0.95);
    assert!(
        boosted > base,
        "a live Hoard Rush should make expeditions luckier"
    );
    assert!((boosted - expected).abs() < 1e-9);
    assert!(
        boosted <= 0.95 + 1e-9,
        "still clamped to the discovery ceiling"
    );

    // When the surge drains, the chance falls back to the un-rushed base.
    state.tick(&data, data.config.hoard_rush_seconds as f32 + 0.1);
    assert!(!state.hoard_rush_active());
    assert!((state.discovery_chance(&data) - base).abs() < 1e-9);
}

#[test]
fn golden_hoard_click_grants_a_reward() {
    let (data, mut state) = setup();
    assert!(state.golden_hoard().is_none());
    // Nothing to collect until a glint has spawned.
    assert!(state.collect_golden_hoard(&data).is_none());

    // Fast-forward past the initial cooldown so a glint appears.
    state.advance_events(&data, data.config.golden_hoard_min_interval as f32 + 0.1);
    assert!(
        state.golden_hoard().is_some(),
        "a glint should spawn after the cooldown"
    );

    // Clicking it consumes the glint and pays out one of the three rewards; the
    // side effect must match whichever variant was rolled.
    state.run.goblins = 5;
    let gold_before = state.run.gold;
    let reward = state
        .collect_golden_hoard(&data)
        .expect("a glint was present to collect");
    assert!(state.golden_hoard().is_none());
    match reward {
        GoldenReward::Frenzy => assert!(state.frenzy_active()),
        GoldenReward::Rush => assert!(state.hoard_rush_active()),
        GoldenReward::Windfall(gold) => {
            assert!(gold > 0.0);
            assert!(state.run.gold >= gold_before + gold - 1e-6);
        }
    }
}

#[test]
fn golden_hoard_rolls_all_three_rewards_over_time() {
    let (data, mut state) = setup();
    state.run.goblins = 20; // nonzero income so a Windfall pays out
    let mut saw_frenzy = false;
    let mut saw_rush = false;
    let mut saw_windfall = false;
    // The clickable window guarantees a spawn each round; the seeded RNG makes
    // this deterministic, and 60 samples make all three variants a near-certainty.
    for _ in 0..60 {
        state.advance_events(&data, data.config.golden_hoard_max_interval as f32 + 0.1);
        match state.collect_golden_hoard(&data) {
            Some(GoldenReward::Frenzy) => saw_frenzy = true,
            Some(GoldenReward::Rush) => saw_rush = true,
            Some(GoldenReward::Windfall(_)) => saw_windfall = true,
            None => panic!("a glint should be present each round"),
        }
    }
    assert!(
        saw_frenzy && saw_rush && saw_windfall,
        "all three golden rewards should appear: frenzy={saw_frenzy} rush={saw_rush} windfall={saw_windfall}"
    );
}

#[test]
fn lifetime_counters_track_new_mechanics() {
    let (data, mut state) = setup();
    assert_eq!(state.stat_value(StatKey::ExpeditionsLaunched), 0.0);
    assert_eq!(state.stat_value(StatKey::GoldenHoardsCollected), 0.0);

    // Launching an expedition bumps its lifetime counter.
    state.run.gold = 1e6;
    state.try_explore(&data).unwrap();
    assert_eq!(state.stat_value(StatKey::ExpeditionsLaunched), 1.0);

    // Collecting a Golden Hoard bumps its counter.
    state.advance_events(&data, data.config.golden_hoard_min_interval as f32 + 0.1);
    assert!(state.collect_golden_hoard(&data).is_some());
    assert_eq!(state.stat_value(StatKey::GoldenHoardsCollected), 1.0);
}

#[test]
fn golden_hoard_expires_if_left_unclicked() {
    let (data, mut state) = setup();
    // Spawn a glint, then let its clickable window lapse without collecting.
    state.advance_events(&data, data.config.golden_hoard_min_interval as f32 + 0.1);
    assert!(state.golden_hoard().is_some());
    state.advance_events(&data, data.config.golden_hoard_lifetime as f32 + 0.1);
    assert!(
        state.golden_hoard().is_none(),
        "an unclicked glint should disappear"
    );
    assert!(!state.frenzy_active());
}

#[test]
fn achievements_unlock_with_rewards() {
    let (data, mut state) = setup();
    state.click(&data);
    let events = state.check_unlocks(&data);
    assert!(events
        .iter()
        .any(|e| matches!(e, UnlockEvent::Achievement { name } if name == "First Click")));
    // first_click reward pays 50 gold on top of the click gain.
    assert!(state.run.gold > 50.0);

    // Unlock is one-shot.
    assert!(state.check_unlocks(&data).is_empty());
}

#[test]
fn dragon_codex_unlocks_on_prestige() {
    let (data, mut state) = setup();
    state.run.gold = 1_000_000.0;
    state.try_prestige(&data).unwrap();
    let events = state.check_unlocks(&data);
    assert!(events
        .iter()
        .any(|e| matches!(e, UnlockEvent::Dragon { name } if name == "Ember Wyrm")));
    // The unlocked dragon's +3% gold/sec bonus is live.
    state.run.goblins = 10;
    assert!(state.gold_per_second(&data) > 10.0);
}

#[test]
fn save_roundtrip_applies_offline_progress() {
    let (data, mut state) = setup();
    state.run.goblins = 10;
    let rate = state.gold_per_second(&data);
    let saved = SaveData {
        version: data.config.version.clone(),
        timestamp: 1_000.0,
        run: state.run.clone(),
        persistent: state.persistent.clone(),
    };

    // Loading 60 seconds later earns 60s at the live passive rate.
    let (loaded, earned) = GameplayState::from_save(&data, saved, 1_060.0);
    assert!((earned - rate * 60.0).abs() < 1e-6);
    assert!((loaded.run.gold - rate * 60.0).abs() < 1e-6);

    let _ = save::now_timestamp; // referenced: real loads stamp with wall clock
}
