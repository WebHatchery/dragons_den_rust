# Dragon's Den — Implementation Plan / Handover

*Companion to `gdd.md` (the design authority). This file tracks what the
framework already provides, the conventions it establishes, and the work that
remains, milestone by milestone (GDD §13).*

---

## 1. What is already built (the framework)

The template has been fully converted; nothing of the template's grid/fog/camera
demo remains. `cargo test` (30 tests, incl. a balance-regression sim),
`cargo clippy -D warnings`, and `cargo fmt --check` all pass. Verified
screenshots in `docs/verification/`: `ui_menu`, `ui_hoard`, `ui_settings`,
`ui_treasures`, `ui_achievements`, `ui_prestige`.

### Data (all content is JSON — never hardcode balance in Rust)

| File | Contents | Status |
| --- | --- | --- |
| `assets/data/game_config.json` | Base rates, cost growths, prestige gate/divisor, autosave interval | Prototype values per GDD §5.1 |
| `assets/data/upgrades.json` | The 4 converged upgrade lines (GDD §8 prototype target) | Done |
| `assets/data/prestige_upgrades.json` | 9 permanent-tree nodes across all 5 effect stats | Done (M3) |
| `assets/data/treasures.json` | 16 treasures, rarity-weighted, all with real effects | Done (M3) |
| `assets/data/achievements.json` | 22 achievements, **all** with wired conditions + rewards | Done (M3) |
| `assets/data/dragons.json` | 8-element codex with original hex swatches, unlock conditions, small bonuses | Done |

Shared condition/effect vocabulary (in `src/data.rs`): `StatKey` (7 stats),
`StatCondition { stat, gte }`, `EffectStat` (5 stats), `PercentEffect`.
Adding content = adding JSON entries; new *kinds* of effect need a new
`EffectStat` variant plus a formula hook in `simulation/economy.rs`.

### Simulation (`src/simulation/` — pure functions, all unit-tested)

- `economy.rs` — gold/click, gold/sec, discovery chance, hire/upgrade cost
  curves. Upgrade levels genuinely feed the formulas (the GDD's core fix).
- `offline.rs` — offline gold at the live rate; uncapped (GDD §12 Q3 decision:
  leaning uncapped; if a cap is ever wanted it goes here).
- `exploration.rs` — chance gate then **rarity-weighted** roll among
  undiscovered treasures; `SeededRng` from the toolkit, serialized in the save.
- `prestige.rs` — `floor(sqrt(gold/divisor))` scaled by Hoard Greed + percent
  bonuses.
- `idle_number.rs` — K/M/B/T…No suffix formatting over `f64`, scientific
  fallback. *Note:* plain `f64`, not the original's significand+exponent pair —
  ample for v1 scales; revisit only if balance pushes past ~1e300.

### State (`src/state/`)

- `GameState` machine: `Menu` / `Gameplay`, explicit `StateTransition`s applied
  in `game.rs`.
- `GameplayState` = `RunState` (gold, goblins, upgrade levels — reset on
  prestige) + `PersistentState` (hoard points, prestige tree, treasures, codex,
  toolkit `Achievements`, lifetime stats, RNG — survives prestige). **This is
  the single source of truth**; UI never owns state (GDD Pillar 2).
- `check_unlocks()` evaluates all achievement/codex conditions each frame,
  loops until stable so meta-achievements chain, applies rewards, and returns
  events for toasts.
- Save/load via toolkit persistence (`save.rs`), autosave every 10s (config),
  on prestige, and on menu exit; offline earnings applied in `from_save` using
  the stored wall-clock timestamp (`miniquad::date::now()` — WASM-safe).

### UI (`src/ui/` — pure view layer)

Every panel reads `GameplayCtx` and returns `UiAction` intents; only
`Game::apply_action` mutates state. Screens: menu, hoard (click target +
expedition + prestige progress), minions, upgrades, treasures, achievements,
prestige (burn + permanent tree), dragon codex. Header shows gold / rate /
Hoard Points; tab bar switches screens; Esc saves and exits to menu.

List screens (upgrades, treasures, achievements, prestige tree) scroll via the
mouse wheel: `ui::apply_scroll` reads the wheel and emits `SetScroll`; the
transient `GameplayState::scroll_y` (reset on tab switch) offsets
`GridLayout::get_item_rect`, and `ui::item_fully_visible` culls the partial
edge rows (macroquad has no scissor).

Capture harness: `DRAGONS_DEN_CAPTURE_SCENE` = `menu` | `settings` | any tab
label (`hoard`, `treasures`, `achievements`, `prestige`, …); an unknown name
boots a fresh gameplay session (seed 42) on the Hoard screen.

---

## 2. Conventions the next agent must keep

1. **Balance lives in JSON.** If you're typing a number into a `.rs` file that
   a designer might tune, stop and move it to `assets/data/`.
2. **UI returns intents.** New interactions = new `UiAction` variant handled in
   `game.rs`, never state mutation inside a `ui/` module.
3. **Formulas only in `simulation/`**, with unit tests beside them. The UI and
   `game.rs` call `GameplayState`'s derived-economy methods, never re-derive.
4. **One state owner.** Anything that must survive prestige goes in
   `PersistentState`; anything reset by prestige goes in `RunState`. Update
   `try_prestige` and the save shape together, and add a migration in
   `save.rs::migrate_save_value` once real players have saves.
5. Repo-wide rules apply: 800-line file cap, no `mod.rs`, toolkit-first,
   `cargo fmt` + clippy `-D warnings` + `cargo test` before publish.

---

## 3. Remaining work

### Toward M1 → M2 (playable prototype)

- [x] **Click feedback juice** — floating "+N" at the cursor on hoard clicks
  (toolkit `fx::FloatingTextLayer`, GDD §9.1). Prestige "burn" flourish spawns a
  larger central "+N Hoard Points" float. Owned by `Game`, drawn in logical UI
  space; click intents anchor to the cursor pos captured each draw.
- [x] **Action log** on the Hoard screen (GDD §9) — the "Chronicle" panel below
  the click target shows the last 8 events newest-first (expedition results,
  achievement/dragon unlocks, prestige burns). Backed by a transient
  `ActionLog` ring on `GameplayState`, fed from the event sites in `game.rs`
  alongside the toasts. Verified in `docs/verification/ui_hoard.png`.
- [x] **Buy-max / buy-10 affordances** on hire and upgrades. `BuyMode`
  (x1/x10/Max) is a transient field on `GameplayState`, toggled via a shared
  `buy_mode_selector` on both the Minions and Upgrades screens. Economy gained
  `bulk_cost` + `affordable_levels` (unit-tested); state gained `try_hire_bulk`
  / `try_buy_upgrade_bulk`. x1/x10 require the full amount affordable; Max buys
  as much as gold allows. Buttons show the resolved count and total cost.
- [x] **Settings screen** (GDD §9) — reached from the main menu (`MenuScreen`
  sub-view). Edits the toolkit `GameSettings`: master/SFX/music volumes (+/-
  steppers), UI text scale, Fullscreen and Show-FPS toggles. Every control
  returns a `ChangeSetting` intent; `game.rs` mutates, `sanitize()`s,
  `apply_display()`s, and persists under `config.game_name` on each change.
  Loaded + applied at startup. Show-FPS overlay is live. New capture scene
  `settings`; verified in `docs/verification/ui_settings.png`.
  - [ ] **Autosave interval as a setting**: deferred — `GameSettings` has no
    such field and forking the shared toolkit is out of scope for this loop.
    Interval still comes from `game_config.json`. Add a toolkit field (or a
    game-local settings key) when ready.
  - [ ] **Audio SFX** (click/purchase/unlock blips) via toolkit `audio`:
    deferred — no sound assets exist yet. Volumes persist and are ready to feed
    a `SoundManager` once packs ship.
- [x] **Balance pass on prototype values** — added a headless greedy-player
  simulation (`simulation/balance.rs`, test-only) that reinvests by best
  payback against a dynamic "coast to threshold" horizon and reports minutes to
  first prestige. Tuned `game_config.json` (base_click 1→2, gold_per_goblin
  1→5, hire_cost_growth 1.2→1.15) to bring optimal-play first prestige from
  ~146 min to **~29 min** (real, less-optimal play a bit longer). The test
  locks it into a 10–40 min window so future config edits can't silently break
  pacing. Value-dependent unit tests now derive expectations from `config`
  instead of hardcoding the old constants.
- [x] Fixed `scripts/capture_ui.ps1` — the template wrapper hardcoded
  `-Prefix "GAME_TEMPLATE"`, so it set `GAME_TEMPLATE_CAPTURE_*` env vars the
  game (which reads `DRAGONS_DEN_*`) ignored; no PNG was written and the shared
  script reported the missing output. Dropped the override (the shared script
  derives `DRAGONS_DEN` from the package name) and defaulted `-Scenes` to
  `menu,hoard,settings`. Verified end-to-end: all three PNGs regenerate.

### Toward M3 (content-complete, GDD §8 full targets)

- [~] Content expansion (GDD §8 targets):
  - [x] treasures 5 → **16**, achievements 10 → **22**, prestige tree 3 → **9**.
    All are pure JSON; `treasure_hoarder`'s "discover every" gate moved 5 → 16.
    Required scroll support first — added generic mouse-wheel scrolling +
    edge-cull to all four list screens (see §UI note). Loader smoke test now
    asserts content lower bounds instead of exact counts. Verified renders:
    `ui_treasures`, `ui_achievements`, `ui_prestige`.
  - [ ] Upgrade lines 4 → 6–8 — deferred to its own iteration: new gold-relevant
    lines feed the balance sim, so this needs a re-tune + re-check of the
    10–40 min window. (Dragons stay at 8 — fixed by the element set.)
- [ ] **Multi-tier prestige** (GDD §12 Q2): rising thresholds per prestige
  count instead of the single 1M gate. Extend `game_config.json` with a
  threshold curve and `prestige.rs` accordingly.
- [ ] Exercise big numbers at real scale; if `f64` precision ever bites,
  upgrade `idle_number.rs` to significand+exponent (and flag it as a toolkit
  candidate per GDD §10).
- [ ] Offline-progress cap decision (GDD §12 Q3) after balance testing.

### Release checklist (per repo standards)

- [ ] `.\publish.ps1` from this directory; verify at `http://127.0.0.1/dragons_den/`.
- [ ] Refresh `catalog_thumbnail.png` (currently the bare menu capture —
  regenerate once the menu has more visual identity).
- [ ] Stale `dist/` still contains template-era artifacts (`game_template.*`) —
  delete before first publish so only `dragons_den` artifacts ship.
- [ ] Add the game to `standing.md` once past prototype.

---

## 4. Known gaps / deliberate deferrals

- **No per-click toast**: clicking updates the counter only; feedback juice is
  the top M2 item above.
- **Prestige seeds a fresh RNG only on New Game** — RNG state persists through
  prestige (correct: prevents re-rolling expedition luck by prestiging).
- **`save.rs` migration** currently only accepts the modern shape (fails loudly
  otherwise) — fine pre-release, must grow real migrations after first ship.
- **Treasure/achievement/dragon content beyond the prototype set** is a pure
  JSON exercise; no code changes needed until a new effect *kind* appears.
