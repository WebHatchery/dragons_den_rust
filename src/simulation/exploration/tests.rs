use super::*;
use crate::data::GameData;

#[test]
fn guaranteed_roll_finds_an_undiscovered_treasure() {
    let data = GameData::load().unwrap();
    let mut rng = SeededRng::new(7);
    let discovered = vec!["common_stone".to_owned()];

    match roll_treasure(&mut rng, &data.treasures, &discovered, 0, 1.0) {
        ExploreOutcome::Found(def) => assert_ne!(def.id, "common_stone"),
        other => panic!("expected a find, got {:?}", other),
    }
}

#[test]
fn zero_chance_never_finds() {
    let data = GameData::load().unwrap();
    let mut rng = SeededRng::new(7);
    assert_eq!(
        roll_treasure(&mut rng, &data.treasures, &[], 0, 0.0),
        ExploreOutcome::NothingFound
    );
}

#[test]
fn full_collection_reports_all_discovered() {
    let data = GameData::load().unwrap();
    let mut rng = SeededRng::new(7);
    let discovered: Vec<String> = data.treasures.iter().map(|d| d.id.clone()).collect();
    assert_eq!(
        roll_treasure(&mut rng, &data.treasures, &discovered, u32::MAX, 1.0),
        ExploreOutcome::AllDiscovered
    );
}

#[test]
fn prestige_gate_locks_treasures_until_enough_prestige() {
    let data = GameData::load().unwrap();
    // The lowest positive gate in the catalog, and every ungated treasure.
    let gate = data
        .treasures
        .iter()
        .map(|t| t.prestige_required)
        .filter(|&p| p > 0)
        .min()
        .expect("catalog should include prestige-gated treasures");
    let ungated: Vec<String> = data
        .treasures
        .iter()
        .filter(|t| t.prestige_required == 0)
        .map(|t| t.id.clone())
        .collect();

    // Below the gate, the reachable pool is exhausted → AllDiscovered.
    let mut rng = SeededRng::new(1);
    assert_eq!(
        roll_treasure(&mut rng, &data.treasures, &ungated, gate - 1, 1.0),
        ExploreOutcome::AllDiscovered
    );

    // At the gate, the newly-unlocked tier becomes findable.
    let mut rng = SeededRng::new(2);
    let mut found_gated = false;
    for _ in 0..200 {
        if let ExploreOutcome::Found(def) =
            roll_treasure(&mut rng, &data.treasures, &ungated, gate, 1.0)
        {
            assert!(def.prestige_required <= gate);
            found_gated = true;
            break;
        }
    }
    assert!(
        found_gated,
        "a gated treasure should be findable once unlocked"
    );
}

#[test]
fn weights_bias_toward_common_drops() {
    let data = GameData::load().unwrap();
    let mut rng = SeededRng::new(42);
    let mut common = 0;
    let mut legendary = 0;
    for _ in 0..500 {
        if let ExploreOutcome::Found(def) = roll_treasure(&mut rng, &data.treasures, &[], 0, 1.0) {
            match def.id.as_str() {
                "common_stone" => common += 1,
                "golden_goblet" => legendary += 1,
                _ => {}
            }
        }
    }
    assert!(
        common > legendary * 5,
        "common {common} vs legendary {legendary}"
    );
}
