# TODO — Dragon's Den

- [ ] **Migrate tests to the crate's `tests/` directory (§11).** Add `src/lib.rs`
  and make `main.rs` use it; move all `src/**/tests.rs` and the test-only balance
  simulator out of `src/`, removing test-module declarations there. Preserve
  regression coverage through intentional public APIs; organize the economy,
  gameplay, and save suites by responsibility and consolidate related cases
  toward five per feature without dropping distinct coverage.

- [ ] **Validate game data at startup (§5.3).** Add project-owned semantic
  validation after toolkit parsing in `src/data.rs`: unique/nonempty IDs in every
  catalog (including minions), resolvable and acyclic prestige prerequisites,
  valid costs/growth/probabilities/intervals, unlock thresholds, and color values.
  Return catalog/field-specific errors and test malformed inputs. Use the
  documented typed `include_json!` path for embedded assets.

- [ ] **Move player-facing text and configuration into JSON (§5.3).** Replace
  hardcoded labels, tooltips, notifications, and log templates in `src/ui/`,
  `src/ui.rs`, `src/game.rs`, and state/data label methods with a typed text
  catalog loaded through the toolkit. Move settings step sizes and the action-log
  capacity into configuration; keep text lookup and color conversion in the UI
  instead of coupling `src/data/dragons.rs` to Macroquad `Color` (§2.1).

- [ ] **Share purchase quotes with gameplay (§§4.4, 7.1).** Move `BulkQuote`
  and `bulk_quote` from `src/ui.rs` into the simulation/state layer and use the
  same rules for display and execution. Replace multi-field purchase/load result
  tuples with named results; cover x1/x10/Max, insufficient funds, and level caps.

- [ ] **Bring module and function boundaries into compliance (§§1.4, 2.2, 4, 10).**
  Extract cohesive helpers from the over-100-line menu/settings/shop draw
  functions and split event/buff handling from the 608-line `state/gameplay.rs`.
  Remove unused `_actions` parameters in Hoard, Treasures, and Achievements;
  group stepper inputs to remove its undocumented Clippy suppression and rename
  the shadowed `hex` binding in `data/dragons.rs`.

- [ ] **Use toolkit input handling and isolate modal input (§7).** Replace the
  generic release-hit-test button implementation in `src/ui.rs` with toolkit
  widgets/input helpers while preserving the theme. Prevent underlying buttons,
  scrolling, and gameplay shortcuts from processing input while settings or a
  confirmation is open. Verify drag cancellation and that modal taps cannot
  buy, prestige, switch tabs, or return to the menu underneath.

- [ ] **Add visible touch onboarding (§7.5).** Provide dismissible instructions
  naming the actual CLICK, hire, expedition, Prestige, and MENU controls and the
  drag-to-scroll gesture. Verify a new player can complete the loop and recover
  from a failed save load without a keyboard; fix the load-error notice in
  `ui/menu.rs`, which currently overlaps the Settings button.

- [ ] **Confirm prestige before resetting the run.** Replace the direct
  `UiAction::Prestige` execution with a modal showing the Hoard Points gained,
  reset effects, and CONFIRM PRESTIGE / CANCEL controls. Recheck eligibility on
  confirmation and cover cancellation, confirmation, and duplicate activation.

- [ ] **Finish audio playback.** Add a small licensed sound pack with attribution
  and wire click, purchase, expedition, unlock, prestige, and menu feedback
  through toolkit audio. Add music or remove the inactive music option; verify
  mute and master/SFX/music controls on Windows and WebGL.

- [ ] **Reconcile project documentation and stale comments.** Update `README.md`,
  `gdd.md`, and Codex source comments for the shipped 15 dragons, active Codex
  bonuses, tested 100-prestige curve, and 12-hour offline cap. Align the GDD's
  asset paths, loading APIs, and module layout with implementation; correct the
  obsolete non-test-line/exception comments in `tests/code_standards.rs`.

- [ ] **Complete release acceptance verification.** Expand the default
  `scripts/capture_ui.ps1` scenes from menu/hoard/settings to all seven tabs plus
  menu/settings, and seed locked, unlocked, scrolled, and prestige-ready states.
  Replace matching captures directly in `docs/verification/`. Exercise new/save
  recovery, offline return, purchases, expeditions, golden hoards, prestige,
  settings, and every collection on Windows and browser touch at representative
  viewport sizes. Run formatting, Clippy, tests, and parameterless `./publish.ps1`
  after the game changes; report failures or blockers.
