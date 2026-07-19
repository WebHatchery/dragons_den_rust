//! Big-number formatting for the idle economy (GDD §10). The implementation
//! now lives in `macroquad_toolkit::ui` as `format_amount` / `format_rate`
//! (idle-genre magnitude suffixes with a scientific fallback); this module
//! keeps the local `idle_number::` vocabulary the rest of the game reads by.

pub use macroquad_toolkit::ui::{format_amount, format_rate};
