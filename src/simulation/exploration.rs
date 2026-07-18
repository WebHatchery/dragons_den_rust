//! Expedition treasure rolls (GDD §5.3): a discovery-chance gate, then a
//! rarity-weighted pick among *undiscovered* treasures — fixing the
//! original's flat uniform pick that ignored its own rarity tiers.

use crate::data::TreasureDef;
use macroquad_toolkit::rng::SeededRng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExploreOutcome<'a> {
    /// The roll failed — the expedition came back empty-clawed.
    NothingFound,
    Found(&'a TreasureDef),
    /// Every treasure is already in the hoard; nothing left to discover.
    AllDiscovered,
}

pub fn roll_treasure<'a>(
    rng: &mut SeededRng,
    treasures: &'a [TreasureDef],
    discovered: &[String],
    chance: f64,
) -> ExploreOutcome<'a> {
    let undiscovered: Vec<&TreasureDef> = treasures
        .iter()
        .filter(|def| !discovered.contains(&def.id))
        .collect();
    if undiscovered.is_empty() {
        return ExploreOutcome::AllDiscovered;
    }
    if f64::from(rng.next_f32()) >= chance {
        return ExploreOutcome::NothingFound;
    }

    let total_weight: u32 = undiscovered.iter().map(|def| def.drop_weight).sum();
    let mut roll = rng.below(total_weight.max(1) as usize) as u32;
    for def in &undiscovered {
        if roll < def.drop_weight {
            return ExploreOutcome::Found(def);
        }
        roll -= def.drop_weight;
    }
    ExploreOutcome::Found(undiscovered[undiscovered.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;

    #[test]
    fn guaranteed_roll_finds_an_undiscovered_treasure() {
        let data = GameData::load().unwrap();
        let mut rng = SeededRng::new(7);
        let discovered = vec!["common_stone".to_owned()];

        match roll_treasure(&mut rng, &data.treasures, &discovered, 1.0) {
            ExploreOutcome::Found(def) => assert_ne!(def.id, "common_stone"),
            other => panic!("expected a find, got {:?}", other),
        }
    }

    #[test]
    fn zero_chance_never_finds() {
        let data = GameData::load().unwrap();
        let mut rng = SeededRng::new(7);
        assert_eq!(
            roll_treasure(&mut rng, &data.treasures, &[], 0.0),
            ExploreOutcome::NothingFound
        );
    }

    #[test]
    fn full_collection_reports_all_discovered() {
        let data = GameData::load().unwrap();
        let mut rng = SeededRng::new(7);
        let discovered: Vec<String> = data.treasures.iter().map(|d| d.id.clone()).collect();
        assert_eq!(
            roll_treasure(&mut rng, &data.treasures, &discovered, 1.0),
            ExploreOutcome::AllDiscovered
        );
    }

    #[test]
    fn weights_bias_toward_common_drops() {
        let data = GameData::load().unwrap();
        let mut rng = SeededRng::new(42);
        let mut common = 0;
        let mut legendary = 0;
        for _ in 0..500 {
            if let ExploreOutcome::Found(def) = roll_treasure(&mut rng, &data.treasures, &[], 1.0) {
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
}
