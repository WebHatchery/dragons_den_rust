# Dragon's Den — Game Design Document

*Draft v0.1 — living document.*

> A dragon-hoarder's idle empire. Click for gold, send goblin minions to work while
> you're away, send expeditions into ruins for treasure, and burn it all down for a
> permanent prestige bonus — over and over, each run a little richer than the last.

Sources: `game_apps/dragons_den/` (React/PHP original), `RustGames/migration_candidates.md`,
`RustGames/standing.md`, `RustGames/docs/GAME_DEVELOPMENT_GUIDE.md`,
`RustGames/docs/CODE_STANDARDS.md`, `RustGames/docs/MACROQUAD_TOOLKIT.md`.

---

## 0. Migration Snapshot

- **Old game:** `game_apps/dragons_den/` — a React 19 + Zustand frontend over a PHP/Slim
  backend that, unusually for this catalog, **is** authoritative for the actual economy
  (gold/goblin math happens server-side; the client does optimistic prediction and
  reconciles). The truly unusual finding, though, is structural: **the repository
  contains two entirely separate, non-overlapping game implementations**, and only one
  of them is reachable from the rendered UI.
  - **Live game** (what a player actually experiences): a minimal, server-authoritative
    idle clicker — click for gold, hire goblins for passive income, explore ruins for
    treasure, prestige at 1,000,000 gold. Four buttons, a resource counter, a sidebar.
  - **Dead code** (fully typed and partially implemented in TypeScript, **zero UI
    wiring** — confirmed by grepping every import of the relevant stores/systems across
    `src/`): a much larger "dragon collection" simulation — dragon breeding/genetics/
    aging, procedural world generation with 9 biomes and a weather system, ancient ruins
    with puzzle/trap/combat encounters, rival dragon lords, a 24-entry elemental ability
    system, an 8-personality bonding system, and an 18-entry achievement system separate
    from the live one. None of it is imported by `App.tsx`/`GamePage.tsx`. Even the
    combat placeholders that exist (`engageCombat`, `encounterRivalLord`) are coin-flip
    stand-ins (60% win rate), not real systems.
  - **Practical consequence:** mechanically, the game a player experiences today has
    almost nothing to do with dragons — it's a goblin-hiring gold clicker with dragon
    flavor text and emoji. The GDD below treats the live loop as the real MVP to port
    and treats the dead code as **design material to selectively harvest for flavor**,
    not a second game to finish building.
- **Why it was picked:** `migration_candidates.md` Tier 1 — "Idle/incremental with
  prestige loop. Zero overlap in Rust roster — idle games are the single lowest-art
  genre that exists (icons + big numbers), yet nothing like it has been built." Confirmed
  by `standing.md`: no existing Rust game in the catalog is an idle/incremental clicker.

- **Art-liability audit.** Even more extreme than the "no art" cases already in this
  catalog: a full grep of `frontend/src` and `frontend/public` for `.png/.jpg/.svg/.webp/
  .gif` returns **zero matches**. There is no `assets/` art folder at all — only a
  `favicon.ico`. Every piece of "iconography" in the live UI is an inline Unicode emoji
  in JSX (🐉 collect gold, 👹 goblins, 🗺️ explore, 🏰 hire, 👑 prestige, 💰/💎/🏆 resource
  labels). The dead code's `elementColors` hex swatches and `icon: 'footsteps'`-style
  string fields in the unused achievement system were never backed by actual image
  files either — they're semantic labels with nothing behind them.

  | Old asset (web) | Art cost | Rust replacement |
  | --- | --- | --- |
  | Inline emoji as button/resource icons | None (Unicode glyphs) | Bitmap font glyphs via `ui`/`TextStyle`, or the toolkit's own icon-substitute set — no commissioning needed either way |
  | Tailwind gradient/color panels | Low (CSS only) | `SurfaceStyle` + `colors` palette |
  | Element color swatches (`dragonElements.ts`, unused) | None (hex values only, never rendered) | Reused directly as a palette for the flavor "Dragon Collection" panel (§0 mechanic table) if that layer is built |

  There is nothing to cut for art reasons — every scope decision below is a systems/
  content call, exactly like `stellar_legacy`'s port before it.

- **Mechanic carry-over table.**

  | Old mechanic | Disposition | Notes |
  | --- | --- | --- |
  | Click for gold | Keep, fix | Live backend pays a flat `+1`, ignoring the dead client's own claw-upgrade concept. Port unifies these: click income scales with a real upgrade, not a decorative one that was never connected. |
  | Hire goblins / passive income | Keep, redesign scope | Live: `gold_per_second = 1 + goblins*0.1`, no upgrades apply to it at all despite an upgrade named for it existing client-side. Port wires the upgrade for real. |
  | Explore ruins → treasure | Keep, fix contract bug | Live backend requires `ruin_id` + `exploration_type` in the request; the live frontend's API client sends neither — this button is currently broken against its own backend's contract. Port defines one consistent request/response shape and a real rarity-weighted drop table (the original picks uniformly among all not-yet-owned treasures, ignoring the rarity tiers it already defines). |
  | Upgrade shop | Keep, converge | **Two incompatible upgrade catalogs exist simultaneously** — `data/upgrades.ts` (5 entries: Gold Doubling, Minion Efficiency, Treasure Finder, Prestige Bonus, Upgrade Cost Reduction) and `data/upgradeDefinitions.ts` (3 entries: `clawSharpness`, `minionEfficiency`, `treasureHunting`) — and only the second set's IDs are what the live gold-per-click/second formulas actually read. Port ships **one** catalog. |
  | Prestige | Keep, fix — currently rewards nothing | Live `prestige` action resets gold/goblins at the 1,000,000 gate but **grants no bonus of any kind** server-side — the "permanent bonus" is a lie in the button copy. This is the single most important fix: port gives prestige a real currency and permanent multiplier (§5.4). |
  | Achievements | Keep, fix completeness | 10 achievements are defined client-side and mirrored in the backend seed data, but only **4 of 10** are ever actually granted (`first_click`, `treasure_hunter`, `golden_collector`, `minion_master`); the other 6 are unreachable dead entries. Port wires all achievements it ships. |
  | Big-number formatting (`IdleNumber` significand+exponent pairs, K/M/B/T + letter-suffix scaling) | Keep as-is | The one genuinely idle-game-appropriate piece of depth already built, and well done — a proper Cookie-Clicker-style large-number system. Nothing in the live formulas currently reaches those scales, but the port's real prestige-scaled economy will. |
  | Sidebar stat display | Fix, don't replicate | `Sidebar.tsx` reads the **dead** `gameStore` instead of the live `serverGameStore`, so it always shows 0 gold/0 treasures/0 minions regardless of actual progress — a real bug in the original, not a design choice. Port's single source of truth avoids this class of bug entirely (see §11). |
  | Orphaned `GameBoard.tsx` layout (event log, upgrade shop, treasure collection grid, prestige card) | Revive | Fully coded, materially richer than the live layout, just never mounted. This is the layout the port's screens are actually modeled on (§9) rather than the sparser live 4-button screen. |
  | Dragon breeding/genetics/aging, procedural world generation, weather, ancient-ruin puzzles/traps, combat/formations, rival dragon lords | Cut, harvest flavor only | This is a whole unfinished creature-collector/exploration RPG bolted onto an idle game's skeleton — even its own "combat" is a 60% coin flip, not a system. Reviving it as real gameplay would also duplicate a genre already 3-4 deep in this catalog (`iron_fauna`, `monsterhall`, `monstron`, `kaiju_sim`), which is exactly the kind of redundancy `migration_candidates.md` flags as a bad fit elsewhere. Instead: the well-designed *data* (8 elements with color swatches and flavor descriptions, the elemental advantage chart, 8 personalities) is repurposed as flavor/naming for a small "Dragon Collection" codex — thematic unlocks tied to prestige milestones, not a simulation (§5.5, §12). |
  | Server-authoritative backend, WebHatchery JWT auth, guest-session/account-linking | Cut | Unlike `stellar_legacy`'s backend (a pure JSON blob store), this one *does* hold real logic — but none of it needs a server for a standalone Steam/itch release. Local save via `macroquad-toolkit::persistence` replaces it; offline-earnings math (already computed lazily server-side from a stored timestamp) ports directly to a local "time since last save" calculation (§5.2). |

- **Explicitly out of scope for the port:** dragon breeding/genetics, procedural world
  generation and weather, ruin puzzles/traps/combat, rival dragon lords and any turn-based
  battle system, multiplayer/leaderboards/cloud sync, any account/login system.

---

## 1. High Concept

- **Pitch:** You're a dragon guarding a hoard that never stops growing. Click for gold,
  put goblins to work while you're away, send expeditions into old ruins for treasure —
  then, when the hoard is big enough, burn it all down and start again richer, wiser,
  and permanently stronger.
- **Genre:** Idle/incremental with a prestige loop. The single lowest-art genre that
  exists, and the only one in the Rust catalog with zero prior entries (`standing.md`).
- **Perspective & presentation:** UI-only, single-screen. No world view, no camera, no
  sprites — resource counters, buttons, a shop grid, a treasure/achievement gallery. This
  is `ui`-module territory almost exclusively, same lean footprint as `stellar_legacy`'s
  port but even more so (no event-modal complexity either).
- **Tone:** Playful and a little greedy — a dragon's smug satisfaction at a bigger pile.
  Light, not dry (contrast with `stellar_legacy`'s ledger tone) — flavor text can be fun
  about goblins and hoards without undermining the numbers.
- **Comparables:** *Cookie Clicker* (the click+idle+prestige backbone), *Adventure
  Capitalist* (multiple parallel income sources feeding one hoard), *Melvor Idle*
  (structured, legible upgrade progression instead of pure chaos-scaling).
- **Audience:** Idle-game players who want honest, well-tuned number-go-up progression —
  the exact opposite of the original's inconsistent, partially-wired formulas.
- **Scope:** Full game, but idle games are inherently compact — see §13; this is the
  shortest runway of any port documented so far in this catalog.
- **Platforms:** itch.io + Steam via WebGL and native Windows (standard for this catalog).
  Idle games benefit unusually well from a WebGL/browser-tab presence — worth leaning
  into for discoverability.

---

## 2. Design Pillars

1. **Every number that goes up is real.** No panel implies depth that isn't backed by an
   actual formula — this catalog has already hit that failure mode once
   (`stellar_legacy`'s Legacy Deck/Chronicle stubs) and this game's original hits it
   *twice* (the entire dead RPG layer, and the live game's own "prestige bonus" that
   grants nothing). The port fixes both.
2. **One formula, one source of truth.** The original shipped two incompatible upgrade
   catalogs and a Sidebar reading the wrong store. The port has exactly one economy
   model and one state owner per stat (§11) — no client/server split to reconcile
   because there's no server.
3. **Idle time is real time.** Offline progress must be calculated honestly from
   elapsed wall-clock time on load, at the same rate the game would have earned it live
   — not a flat/placeholder rate disconnected from the active-play formulas.
4. **Prestige must feel like progress, not punishment.** Resetting the hoard has to
   hand back something permanently stronger than what was lost, on a curve that rewards
   returning rather than a one-time "congratulations, you now have zero of everything."
5. **Dragon flavor earns its keep without becoming a second game.** The elemental/
   personality content from the original's dead RPG is genuinely well-made — it's kept
   as color and identity for the collection layer, but never grows combat, breeding, or
   an overworld. If a system starts needing turn order or a map, that's a sign it's
   drifted out of scope (§12).

---

## 3. Core Loop

**Moment-to-moment loop:**

1. Click the hoard to collect gold (scales with the click-power upgrade line).
2. Spend gold hiring/upgrading goblin minions for passive income.
3. Send an expedition into a ruin for a chance at treasure (rarity-weighted, each
   treasure grants a small permanent passive bonus once discovered).
4. Spend gold in the upgrade shop across a few converging lines (click power, minion
   efficiency, treasure luck, prestige-currency gain).
5. Watch achievements unlock as milestones are crossed; each grants a small one-time
   bonus, not just a badge.
6. Return later (or leave the game running) to collect idle income accrued while away.

**Prestige loop** (the session-spanning meta-loop):

1. Grow the hoard toward the prestige threshold.
2. Prestige: convert the current hoard into a permanent currency (§5.4) and reset gold/
   goblins/active upgrades.
3. Spend the permanent currency on a small permanent-multiplier tree — these persist
   across every future prestige.
4. Each subsequent run is faster because of the accumulated permanent multipliers;
   treasures and achievements already unlocked stay unlocked (matching the original's
   choice to preserve those through prestige).
5. Occasionally, crossing a prestige-tier or collection milestone unlocks a themed
   "dragon" entry in the collection codex (flavor + a small passive bonus) — see §5.5.

---

## 4. Player Role & Verbs

- **The player is:** the dragon — never a named character, never voiced, purely the
  hoard's owner. No portrait, no avatar art needed (matches §0's art audit).
- **The player directly controls:** clicking, hiring/upgrading goblins, sending
  expeditions, spending gold in the upgrade shop, choosing when to prestige, spending
  prestige currency on permanent upgrades.
- **The player does NOT control:** individual goblins' behavior (aggregate passive
  income only), the specific treasure an expedition finds (weighted-random), exact
  achievement timing (milestone-triggered).
- **Core verb list:** *Collect* (click), *Hire* (goblins), *Explore* (ruins/treasure),
  *Upgrade* (shop), *Prestige* (reset for permanent currency), *Unlock* (achievements/
  dragon codex entries).

---

## 5. Systems & Mechanics

### 5.1 Core Economy

| Stat | Meaning | Notes |
| --- | --- | --- |
| Gold | Primary spendable resource | Reset on prestige |
| Goblins | Passive-income count | Reset on prestige |
| Gold/click | Click income | Scales with click-power upgrade |
| Gold/sec | Passive income | Scales with goblin count *and* the efficiency upgrade (the original never actually wired this multiplier in) |
| Hoard Points | Permanent prestige currency | Persists across prestige resets |

```text
gold_per_click  = base_click * (1 + click_power_level * click_power_rate)
gold_per_second = base_passive + goblins * (1 + minion_efficiency_level * efficiency_rate)
hire_cost(n)    = base_hire_cost * hire_cost_growth ^ n         // kept from original: 50 * 1.2^n
upgrade_cost(l) = floor(base_cost * upgrade_cost_growth ^ l)     // kept from original: base * 1.5^level
```

Both scaling formulas (`hire_cost`, `upgrade_cost`) are kept as-is from the original —
standard, well-behaved idle-game exponential curves. What's fixed is that
`gold_per_click`/`gold_per_second` in the port actually read their respective upgrade
levels, unlike the live backend's flat, upgrade-blind version.

### 5.2 Offline / Idle Progress

```text
seconds_elapsed = now - last_save_timestamp
offline_gold = gold_per_second * seconds_elapsed
```

Carried over directly from the backend's lazy-on-load calculation (`processIdleEarnings`)
— this was the one part of the original's server logic that was already correct and
simple. The port computes it once on load, same formula the live tick uses, so idle and
active income are always consistent with each other (no separate "offline rate" to
tune independently and let drift, which is a common idle-game bug class this avoids
pre-emptively).

### 5.3 Treasure & Exploration

```text
explore_chance = base_discovery_chance (kept concept: ~30%)
on success: pick a treasure by rarity weight (common > rare > epic > legendary),
            not uniformly among all undiscovered (fixes the original's flat pick)
each treasure grants a small permanent passive_bonus, stacking, once discovered
```

| Rarity | Example effect shape |
| --- | --- |
| Common | negligible flat bonus |
| Rare | small % bonus to one stat |
| Epic | moderate % bonus, sometimes cross-stat |
| Legendary | meaningful % bonus, always visible in the collection UI |

### 5.4 Prestige

```text
prestige_threshold = configurable gate (kept concept: 1,000,000 gold for the first tier)
hoard_points_gained = floor(sqrt(gold / prestige_divisor))
on prestige:
  gold, goblins, run-scoped upgrades -> reset
  hoard_points += hoard_points_gained
  treasures, achievements, dragon codex -> preserved (kept from original)
hoard_points spent on a small permanent-multiplier tree (click power, passive income,
  treasure luck), each purchase persisting through every future prestige
```

This is the single biggest mechanical fix over the original: the live game's prestige
resets everything and grants **nothing** in return — a genuinely broken reward loop for
the pillar it's named after. The port makes the "permanent bonus" the button copy always
claimed to give.

### 5.5 Dragon Collection (flavor layer, not a simulation)

A small codex of thematic dragon entries — one per element, reusing the original's
8-element roster (fire/ice/earth/air/shadow/light/poison/lightning) and its already-
designed color swatches (`dragonElements.ts`) purely as identity, not stats-bearing
combatants:

```text
unlock_condition = a prestige tier reached OR a collection milestone
                    (e.g. "discover 3 legendary treasures")
on unlock: codex entry revealed (name, element, flavor text, color) + a small
           permanent passive bonus, same shape as a treasure bonus
```

No breeding, no aging, no combat, no world map — if a future revision wants that, it's
a different, larger design document (see §12), not an extension of this one.

### 5.6 Achievements

All shipped achievements are fully wired to a real, checkable condition — the port does
not repeat the original's 4-of-10 gap. Each grants a small one-time bonus (gold, hoard
points, or a codex reveal), not just a checkbox.

### 5.7 Randomness & Determinism

- **What's randomized:** treasure discovery rolls, which treasure is found (weighted).
- **What must stay deterministic:** the core tick and offline-progress calculation —
  isolated behind the toolkit's `rng` module, seeded, per `CODE_STANDARDS.md` §5, rather
  than ad hoc `Math.random()`-equivalents scattered through formulas as in the original.

---

## 6. Data Model (`assets/*.json`)

```json
// assets/upgrades.json — the single converged upgrade catalog (replaces the original's two)
{
  "id": "click_power",
  "name": "Claw Sharpness",
  "base_cost": 100,
  "cost_growth": 1.5,
  "max_level": 20,
  "effect": { "stat": "gold_per_click", "rate": 0.5 }
}
```

```json
// assets/treasures.json — rarity-weighted treasure table with real drop weights
{ "id": "golden_goblet", "rarity": "legendary", "drop_weight": 1, "effect": { "stat": "gold_per_click", "percent": 5 } }
```

```json
// assets/achievements.json — fully-wired condition + reward per entry
{ "id": "first_thousand", "condition": { "stat": "gold_total_earned", "gte": 1000 }, "reward": { "gold": 100 } }
```

```json
// assets/dragons.json — the 8-element flavor codex (§5.5)
{ "id": "ember_wyrm", "element": "fire", "unlock": { "type": "prestige_tier", "value": 1 }, "effect": { "stat": "gold_per_second", "percent": 3 } }
```

| File | Defines | Loaded via |
| --- | --- | --- |
| `assets/upgrades.json` | Converged upgrade catalog | `data_loader::load_json_file` |
| `assets/treasures.json` | Rarity-weighted treasure table | `DataRegistry` |
| `assets/achievements.json` | Achievement conditions + rewards | `DataRegistry` |
| `assets/dragons.json` | 8-element flavor codex + unlock conditions | `data_loader::load_json_file` |
| `assets/data/game_config.json` | Base rates, cost-growth constants, prestige divisor, autosave interval | `data_loader::load_json_file` |

Embed-only (`include_str!` fallback for WASM) is sufficient — no disk hot-reload need,
same call as `stellar_legacy`.

---

## 7. World & Progression Structure

- **World layout:** none — single screen, no map, no camera (§1).
- **Session length:** open-ended; a single prestige cycle can run minutes to hours
  depending on tuning, with the full arc (first prestige through several permanent-
  upgrade tiers) sized for a multi-session idle game rather than one sitting.
- **Progression stages:**

  | Stage | Trigger | What changes |
  | --- | --- | --- |
  | Early game | New save | Click + first goblins, first upgrade tier |
  | Mid game | Goblins scaling, first treasures found | Upgrade shop fully open, achievements start unlocking |
  | Prestige-ready | Threshold reached | Prestige button available |
  | Post-prestige | Any prestige completed | Permanent-multiplier tree available, run resets, next cycle faster |

- **Save/persistence model:** local save via `macroquad_toolkit::persistence`
  (`save_to_slot_with_version`), autosaved on an interval (kept concept from the
  original's `AUTO_SAVE_INTERVAL`, 10s) plus on prestige and on exit. No server sync —
  this fully replaces the original's authoritative-backend model (§0); offline-progress
  (§5.2) is computed from the save's stored timestamp on load.

---

## 8. Content Inventory

| Content type | Prototype target | Full target |
| --- | ---: | ---: |
| Upgrade lines (converged catalog) | 4 (click power, minion efficiency, treasure luck, hoard-point gain) | 6–8 |
| Treasures (rarity-weighted) | 5 (as original) | 15–20 across 4 rarity tiers |
| Achievements (fully wired) | 10 (as original count, but all reachable) | 20–25 |
| Dragon codex entries | 8 (one per element, as original's element roster) | 8 (fixed — matches the element set) |
| Prestige permanent-upgrade tree nodes | 3 | 8–10 |

The original's counts are a reasonable prototype target almost everywhere — the fix
isn't "more content," it's "the content that already exists actually works." The full
targets exist to keep repeat prestige cycles from feeling identical, same rationale as
`stellar_legacy`'s event-count expansion.

---

## 9. UI/UX & Screen Flow

Modeled on the original's orphaned `GameBoard.tsx` layout (richer than what was actually
shipped to players — see §0) rather than the sparse 4-button live screen.

| Screen | Purpose | Toolkit pieces |
| --- | --- | --- |
| Main Menu | New/continue/load slot, settings | `VirtualUi`, `SurfaceStyle`, buttons |
| Hoard (main view) | Click target, resource counters, gold/sec display, action log | `GridLayout`, meters, badges, `NotificationManager` (floating "+N" click feedback) |
| Minions / Hire | Goblin count, hire button with live cost, passive-income breakdown | `TextStyle`, buttons |
| Upgrade Shop | Converged upgrade list with cost/level/effect preview | `ScrollTabs`, `GridLayout` |
| Treasure Collection | Discovered/undiscovered grid, rarity badges | `GridLayout`, badges, tooltips |
| Achievements | Checklist with condition + reward preview | `ScrollTabs` |
| Prestige | Threshold progress, hoard-points preview, permanent-upgrade tree | `GridLayout`, meters |
| Dragon Codex | Flavor entries, unlock conditions, per-entry passive bonus | `GridLayout`, `TextStyle` |
| Pause/Settings | Autosave interval, audio, save/load | `VirtualUi` |

Interaction flow:

1. Player clicks the hoard or waits for passive income; floating "+N" feedback confirms
   each click.
2. Player spends gold in Minions/Hire or Upgrade Shop as thresholds allow.
3. Player triggers an expedition from the Hoard view or a dedicated button; result
   posts to the action log and, on success, updates Treasure Collection.
4. Achievements unlock silently in the background with a toast/log entry, never
   blocking play.
5. When the prestige threshold is crossed, the Prestige screen becomes available;
   confirming it resets the run and opens the permanent-upgrade tree for the earned
   Hoard Points.

---

## 10. Toolkit Mapping

| Need | Toolkit module | Using it? | Notes |
| --- | --- | --- | --- |
| Input handling | `input` | Yes | Buttons only |
| Widgets/layout/text | `ui` (`VirtualUi`, `GridLayout`, `SurfaceStyle`, `TextStyle`, meters, badges, tabs, scroll) | Yes | Carries essentially the whole game — leanest UI footprint in the catalog alongside `stellar_legacy` |
| Textures/manifest | `assets` (`AssetManager`) | No | Zero art exists to manage (§0) |
| Camera/pan/zoom | `camera` | No | No world view |
| Cross-system messaging | `events` (`EventBus<UiAction>`) | Yes | Click/hire/explore/upgrade/prestige actions |
| Palette | `colors` | Yes | Reuse the original's element color swatches for the Dragon Codex |
| Vector/grid math | `math` | No | No spatial layout beyond simple grids |
| Frame timing | `timing` | Yes | Standard frame loop; autosave interval timing |
| Particles/juice | `fx` | Maybe | Floating "+N" click-feedback juice, prestige "reset" flourish |
| User settings | `settings` | Yes | Autosave interval, audio |
| Unlocks/achievements | `achievements` | Yes | Direct fit — this toolkit module is exactly what §5.6 needs |
| Dev overlay | `debug` | Yes | Standard |
| Deterministic randomness | `rng` | Yes | Seeded, for treasure-drop rolls only |
| Sprite animation | `sprite` | No | No sprites |
| Procedural images | `raster` | No | Not needed |
| Headless screenshot capture | `capture` | Yes (required for every game) | See `docs/screenshot_capture_harness_guide.md` |
| Save/load | `persistence` (`save_to_slot_with_version`, etc.) | Yes | Replaces the original's server-authoritative model entirely (§7) |
| Tile grid / fog / pathing | `FlatGrid`, `FogState`, line-of-sight, flood-fill | No | No spatial world |
| Big-number formatting | *(none — project-local)* | Yes, project-local | The `IdleNumber` significand+exponent concept doesn't map to an existing toolkit module; port it as a small project-local `idle_number.rs` rather than forcing a toolkit fit. Worth flagging as a candidate future toolkit addition if another idle game ever gets built (per `CODE_STANDARDS.md`'s "reach for the toolkit first, but flag genuine gaps" rule). |

---

## 11. Architecture Skeleton

```
src/
├── main.rs
├── game.rs             # Game struct, update()/draw() loop
├── state.rs             # GameState enum + re-exports
├── state/
│   ├── menu.rs
│   └── gameplay.rs       # single active-save state: hoard/shop/collection/prestige sub-views
├── data.rs               # data module root
├── data/
│   ├── upgrades.rs
│   ├── treasures.rs
│   ├── achievements.rs
│   └── dragons.rs
├── simulation.rs         # stateless services root
├── simulation/
│   ├── economy.rs        # gold_per_click/gold_per_second, hire/upgrade cost curves (§5.1)
│   ├── offline.rs        # elapsed-time idle-progress calculation (§5.2)
│   ├── exploration.rs    # treasure roll + rarity weighting (§5.3)
│   ├── prestige.rs       # hoard-points formula + permanent-upgrade application (§5.4)
│   └── idle_number.rs    # significand+exponent big-number type, ported from the original's IdleNumber
├── ui.rs
├── ui/
│   ├── hoard.rs
│   ├── minions.rs
│   ├── upgrades.rs
│   ├── treasures.rs
│   ├── achievements.rs
│   ├── prestige.rs
│   └── dragons.rs
└── save.rs
```

- **`GameState` variants:** `Menu`, `Gameplay` (single active save; internal screen enum
  matching §9's tab list — no nested modal complexity the way `stellar_legacy` needed
  for event decisions).
- **Key stateless services (`simulation`):** `economy::tick`, `offline::apply_elapsed`,
  `exploration::roll_treasure`, `prestige::calculate_hoard_points`.
- **Single source of truth, deliberately:** unlike the original's live/dead store split
  and its Sidebar-reads-wrong-store bug, the port has exactly one `Gameplay` state
  struct that every UI panel reads from — there is no second store to drift out of sync
  with, because there is no second store.

---

## 12. Non-Goals / Open Questions

- **Explicitly not building (v1):** dragon breeding/genetics/aging, procedural world
  generation, weather, ancient-ruin puzzles/traps, any turn-based combat or battle
  formation system, rival dragon lords, multiplayer/leaderboards, any account/login
  system. All of these existed only as inert/placeholder code in the original and would,
  if built out for real, duplicate a genre already well-covered elsewhere in this catalog
  (creature-collector/combat: `iron_fauna`, `monsterhall`, `monstron`, `kaiju_sim`).
- **Differentiation note:** the only genre-adjacent risk is drifting toward those
  creature-collector games if the Dragon Codex (§5.5) is ever expanded past flavor. The
  guardrail is Pillar 5 — the moment a codex entry needs a stat block, a battle, or a
  breeding mechanic, that's a different, larger game, not this one.
- **Open questions**, in priority order:

  1. Does the Dragon Codex grant real (if small) permanent bonuses, or should it stay
     purely cosmetic to keep the "idle game, not RPG" boundary unmistakable? Leaning
     toward small bonuses (per §5.5) since purely cosmetic unlocks are a weaker hook,
     but this is worth a playtest check against Pillar 5's drift risk.
  2. How many prestige *tiers* (rising thresholds, not just one reset point) does the
     full game need before it feels like a complete idle-game arc? The original never
     got past a single flat 1,000,000 gate.
  3. Does offline progress need a cap (common in the genre, to preserve some incentive
     to return often) or should it be uncapped given the "idle time is real time"
     pillar? Leaning uncapped for honesty, but flag for balance-pass reconsideration.

---

## 13. Milestones

| Milestone | Proves | Target content |
| --- | --- | --- |
| M1 — Mechanical proof | Click → hire → explore → upgrade → prestige loop works end-to-end with placeholder data, prestige actually grants a permanent bonus | 1 upgrade line, 3 treasures, 1 prestige tier |
| M2 — Playable prototype | Full turn loop with converged upgrade catalog, all achievements wired, offline progress correct on reload | Prototype content targets from §8 |
| M3 — Content-complete | Full upgrade/treasure/achievement/dragon-codex content, multi-tier prestige, big-number formatting exercised at real scale | Full targets from §8 |

After M1, follow the standard per-game loop: `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo test`, then `.\publish.ps1` from the game directory to verify at the shared preview
root — same validation path as every other game in this repo, no exceptions for being new.
