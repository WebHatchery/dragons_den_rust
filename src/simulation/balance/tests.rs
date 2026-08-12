use super::*;

fn purchase_summary(report: &CycleReport) -> String {
    let mut purchases: Vec<_> = report.purchases.iter().collect();
    purchases.sort_by_key(|(id, _)| *id);
    purchases
        .into_iter()
        .map(|(id, count)| format!("{id}:{count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn first_prestige_reachable_in_a_modest_session() {
    let data = GameData::load().unwrap();
    let report = simulate_career(&data, 1).remove(0);
    let minutes = report.seconds / 60.0;
    eprintln!("time to first prestige (active play): {minutes:.1} min");
    assert!(
        (8.0..=40.0).contains(&minutes),
        "first prestige at {minutes:.1} min is outside the 8–40 min target window"
    );
}

#[test]
fn second_prestige_cycle_is_meaningfully_faster() {
    let data = GameData::load().unwrap();
    let reports = simulate_career(&data, 2);
    let first = &reports[0];
    let second = &reports[1];
    eprintln!(
        "cycle 1: {:.1} min | cycle 2: {:.1} min ({:.0}% of cycle 1)",
        first.seconds / 60.0,
        second.seconds / 60.0,
        second.seconds / first.seconds * 100.0
    );
    assert!(
        first.tree_levels >= 3,
        "first prestige should fund a tree spree"
    );
    assert!(
        second.seconds <= first.seconds * 0.7,
        "cycle 2 took {:.1} min vs cycle 1's {:.1} min",
        second.seconds / 60.0,
        first.seconds / 60.0
    );
}

#[test]
fn prestige_career_checkpoints_remain_playable_and_unlock_new_choices() {
    let data = GameData::load().unwrap();
    let reports = simulate_career(&data, 100);
    let checkpoints = [5, 10, 11, 25, 50, 100];

    for prestige in checkpoints {
        let report = &reports[(prestige - 1) as usize];
        eprintln!(
            "P{:>3}: {:>8.2} min | threshold {:>10.3e} | HP {:>10.0} | tree {:>2} | offline cap {:>7.1}% | {}",
            report.prestige,
            report.seconds / 60.0,
            report.threshold,
            report.hp_gained,
            report.tree_levels,
            report.offline_share * 100.0,
            purchase_summary(report),
        );
        assert!(
            report.seconds.is_finite(),
            "prestige {prestige} is not reachable within the simulation limit"
        );
        assert!(
            report.offline_share >= 1.0,
            "a capped offline return should make meaningful progress at P{prestige}"
        );
    }

    let band_entry_minutes = reports[10].seconds / 60.0;
    let late_minutes = reports[99].seconds / 60.0;
    assert!(
        (5.0..=60.0).contains(&band_entry_minutes),
        "the second-band wall takes {band_entry_minutes:.1} min, outside 5–60 min"
    );
    assert!(
        (10.0..=120.0).contains(&late_minutes),
        "P100 takes {late_minutes:.1} min, outside the 10–120 min long-tail window"
    );

    for def in data.upgrades.iter().filter(|def| {
        def.prestige_required > 0
            && matches!(
                def.effect.stat,
                EffectStat::GoldPerClick | EffectStat::MinionEfficiency | EffectStat::GoldPerSecond
            )
    }) {
        let first_purchase = reports
            .iter()
            .find(|report| report.purchases.contains_key(&def.id))
            .map(|report| report.prestige);
        eprintln!(
            "unlock {} (gate P{}) first bought in {:?}",
            def.id, def.prestige_required, first_purchase
        );
        assert!(
            first_purchase.is_some_and(|cycle| cycle <= def.prestige_required + 2),
            "{} unlocks at P{} but is not meaningfully purchased soon after",
            def.id,
            def.prestige_required
        );
    }

    for def in data.minions.iter().filter(|def| def.prestige_required > 0) {
        let first_purchase = reports
            .iter()
            .find(|report| report.purchases.contains_key(&def.id))
            .map(|report| report.prestige);
        eprintln!(
            "minion {} (gate P{}) first bought in {:?}",
            def.id, def.prestige_required, first_purchase
        );
        assert!(
            first_purchase.is_some_and(|cycle| cycle <= def.prestige_required + 3),
            "{} unlocks at P{} but is not meaningfully purchased soon after",
            def.id,
            def.prestige_required
        );
    }
}
