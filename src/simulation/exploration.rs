//! Expedition treasure rolls (GDD §5.3): a discovery-chance gate, then a
//! rarity-weighted pick among *undiscovered, currently-unlocked* treasures —
//! fixing the original's flat uniform pick that ignored its own rarity tiers.
//! Prestige-gated finds (#8) only enter the pool once enough hoards are burned,
//! giving exploration a long tail that reopens after every prestige.

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
    prestige_count: u32,
    chance: f64,
) -> ExploreOutcome<'a> {
    // The pool is treasures that are both unlocked at the current prestige and
    // not yet in the hoard. An exhausted pool with gated finds still remaining
    // reports `AllDiscovered` — "nothing within reach right now" — so the button
    // stays quiet until the next prestige reopens it.
    let available: Vec<&TreasureDef> = treasures
        .iter()
        .filter(|def| def.prestige_required <= prestige_count && !discovered.contains(&def.id))
        .collect();
    if available.is_empty() {
        return ExploreOutcome::AllDiscovered;
    }
    if f64::from(rng.next_f32()) >= chance {
        return ExploreOutcome::NothingFound;
    }

    let total_weight: u32 = available.iter().map(|def| def.drop_weight).sum();
    let mut roll = rng.below(total_weight.max(1) as usize) as u32;
    for def in &available {
        if roll < def.drop_weight {
            return ExploreOutcome::Found(def);
        }
        roll -= def.drop_weight;
    }
    ExploreOutcome::Found(available[available.len() - 1])
}

#[cfg(test)]
mod tests;
