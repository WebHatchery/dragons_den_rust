//! The 8-element dragon codex — flavor plus a small passive bonus (GDD §5.5).

use crate::data::{PercentEffect, StatCondition};
use macroquad::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragonDef {
    pub id: String,
    pub name: String,
    pub element: String,
    /// Hex color from the original `dragonElements.ts` palette, e.g. "#FF4500".
    pub color: String,
    pub flavor: String,
    pub unlock: StatCondition,
    pub effect: PercentEffect,
}

impl DragonDef {
    /// Parses the `#RRGGBB` swatch; falls back to white on malformed data.
    pub fn swatch(&self) -> Color {
        parse_hex_color(&self.color).unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0))
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    Some(Color::new(
        ((value >> 16) & 0xFF) as f32 / 255.0,
        ((value >> 8) & 0xFF) as f32 / 255.0,
        (value & 0xFF) as f32 / 255.0,
        1.0,
    ))
}

#[cfg(test)]
mod tests;
