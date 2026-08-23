# TODO — Dragon's Den

_Release review: 23 August 2026. This is the canonical, implementation-focused
release backlog; completed work and deliberate non-goals are not repeated here._

## Release blockers

- [ ] **Make every collection screen fully usable with touch.** The 15-card
  Dragon Codex in `src/ui/dragons.rs` has no scroll or culling, so it draws
  over the persistent bottom strip at the 1280×720 game resolution. The shared
  `apply_scroll` helper only reads the mouse wheel, leaving long upgrades,
  treasures, and achievements inaccessible on touch-only browsers. Add visible
  drag/swipe scrolling (prefer a toolkit improvement if broadly reusable), use
  it for the Codex and all long lists, and capture the corrected states. Done
  means every entry is reachable by touch and no card overdraws another region.

- [ ] **Set the first-public-release identity and remove stale player-facing
  copy.** The menu still says “v0.1 framework — see IMPLEMENTATION_PLAN.md”,
  but that file does not exist. Choose the release version, align
  `Cargo.toml` and `assets/data/game_config.json`, replace the footer with
  real release text, and update `game_page.json` to describe tap/click controls
  and the visible **MENU** button instead of presenting `Esc` as the only route
  back to the menu. Confirm the version/slot policy before publishing so the
  first public save format is intentional.

## Release-completion work

- [ ] **Finish the audio path.** Settings persist master, SFX, and music volume,
  but no sound is loaded or played; the settings view explicitly says audio will
  arrive when packs ship. Select and record the licence for a small sound pack,
  then wire click, purchase, expedition, unlock, prestige, and menu feedback
  through the toolkit audio path. Verify mute and each volume channel in Windows
  and WebGL.

- [ ] **Protect the irreversible prestige action.** `PRESTIGE` burns the current
  run immediately even though the GDD describes a confirmation. Add a touch-safe
  confirmation modal that clearly states the Hoard Points gained, reset effects,
  and visible **CONFIRM PRESTIGE** / **CANCEL** targets. Cover cancellation and
  confirmation with focused state/UI tests.

- [ ] **Reconcile the design and public documentation with the shipped game.**
  The data ships 15 Codex entries, but `README.md`, `gdd.md`, and source comments
  still describe an 8-element fixed Codex. GDD §12 also calls three already-made
  choices open: codex bonuses are live, the prestige curve is tested through 100
  prestiges, and offline earnings have a 12-hour cap. Update the documents and
  content targets to the release decision, or deliberately trim the game back
  to its documented eight entries.

- [ ] **Turn the current checks into a release acceptance pass.** Expand
  `scripts/capture_ui.ps1` so its default run covers all seven gameplay tabs
  plus menu and settings, with representative locked, unlocked, scrolled, and
  prestige-ready states. Replace the images in `docs/verification/` and perform
  a browser touch and Windows smoke pass covering a new save, offline return,
  purchase, expedition, golden hoard, prestige confirmation, save recovery, and
  every scrollable collection. Re-run `cargo fmt --check`, clippy, tests, and
  `publish.ps1` after each release candidate.

## Deliberate non-goals

- Legacy save migration is unnecessary before the first public build because
  there are no player saves to retain; schedule it only when a later save-shape
  change needs backward compatibility.
- Dragon breeding, combat, a world map, multiplayer, and account features remain
  outside the v1 scope in `gdd.md`.
