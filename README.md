# Dragon's Den

An idle/incremental hoard-building game: click for gold, hire goblin minions for
passive income, send expeditions into ruins for treasure, and prestige — burning
the hoard for permanent Hoard Points — over and over, each run richer than the
last.

A Rust + Macroquad port of the WebHatchery `game_apps/dragons_den` React/PHP
game. **`gdd.md` is the design document** and the authority on scope; the
original's broken/unwired mechanics (prestige that granted nothing, upgrade
formulas that ignored upgrade levels) are fixed here by design.

**`IMPLEMENTATION_PLAN.md` tracks what is built and what comes next.**

## Status

Framework complete (pre-M1): the full click → hire → explore → upgrade →
prestige loop works end-to-end with prototype data, all screens render, saves
persist with offline progress. See the plan for remaining milestone work.

## Run

```powershell
cargo run                    # native window
cargo test                   # simulation + state unit tests
cargo clippy --all-targets --all-features -- -D warnings
.\publish.ps1                # build Windows + WebGL and deploy to the preview root
```

## Screenshot capture (headless UI verification)

```powershell
$env:DRAGONS_DEN_CAPTURE_PATH="docs\verification\ui_menu.png"
$env:DRAGONS_DEN_CAPTURE_SCENE="menu"        # or "hoard" (gameplay)
cargo run
```

## Layout

```
assets/data/     game_config, upgrades, prestige_upgrades, treasures,
                 achievements, dragons — ALL balance/content lives here (JSON)
src/data*        serde types + embedded loaders for the catalogs
src/simulation*  pure, tested services: economy, offline, exploration,
                 prestige, idle_number (big-number formatting)
src/state*       GameState machine (Menu / Gameplay) — one source of truth
src/save.rs      save shape + timestamp for offline earnings
src/ui*          pure view layer: reads state, returns UiAction intents
```
