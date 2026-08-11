use super::*;

#[test]
fn embedded_data_loads() {
    let data = GameData::load().unwrap();

    assert_eq!(data.config.game_name, "dragons_den");
    // Lower bounds track the GDD §8 content targets; content may grow past
    // them without breaking this loader smoke test.
    assert!(data.upgrades.len() >= 4);
    assert!(data.prestige_upgrades.len() >= 8);
    assert!(data.treasures.len() >= 15);
    assert!(data.achievements.len() >= 20);
    assert!(data.dragons.len() >= 12);
}

#[test]
fn catalog_ids_are_unique_and_resolvable() {
    let data = GameData::load().unwrap();

    for def in &data.upgrades {
        assert!(data.upgrade(&def.id).is_some());
    }
    for def in &data.treasures {
        assert!(data.treasure(&def.id).is_some());
    }

    let mut ids: Vec<&str> = data
        .upgrades
        .iter()
        .map(|d| d.id.as_str())
        .chain(data.prestige_upgrades.iter().map(|d| d.id.as_str()))
        .chain(data.treasures.iter().map(|d| d.id.as_str()))
        .chain(data.achievements.iter().map(|d| d.id.as_str()))
        .chain(data.dragons.iter().map(|d| d.id.as_str()))
        .collect();
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "duplicate ids across catalogs");
}
