# TODO — Dragon's Den

All design milestones (M1–M3), the ornate-frame UI redesign, and the engagement
plan's Tiers 1–3 have shipped. What's left:

## Audio

- SFX for click / purchase / unlock via the toolkit `SoundManager` — blocked on a sound
  asset pack. Settings already persist master/SFX/music volumes, ready to feed it.

## Art and UI polish

- Procedural glyphs for prestige branch nodes, treasures, and the settings gear — these
  are still plain colored discs.
- Hover tooltips.
- Ornate depth left over from the theme pass: panel gradient/vignette and corner filigree,
  ideally as a reusable `FramedPanel` in `macroquad-toolkit`.
- Refresh `catalog_thumbnail.png` once the menu has more visual identity.

## Persistence

- Real save migrations in `save.rs`; it accepts only the modern shape and fails loudly
  otherwise, which stops being acceptable after the first public release.
