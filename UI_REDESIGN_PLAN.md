# UI Redesign Plan — "Ornate Frame" Layout

*Reference mockup: [`image.png`](./image.png) (Prestige screen shown).*
*Living plan — phase checkboxes track delivery. Companion to `IMPLEMENTATION_PLAN.md`.*

The mockup keeps the current 7-tab structure but replaces the **single-screen, one-panel-at-a-time**
layout with a **persistent frame**: an always-visible hoard/click rail on the left, a persistent
minions/expeditions/treasures strip along the bottom, richer resource cards in the header, and a
tab-swapped **center** region. The Prestige center is a branching "Hoard Legacies" skill tree, a big
step up from today's flat upgrade grid.

This is a large redesign; it is phased so every phase compiles, passes CI, and is screenshot-verifiable
via the capture harness before the next begins.

---

## 1. Gap analysis — current vs. mockup

| Area | Today (`src/ui/`) | Mockup target |
| --- | --- | --- |
| Frame | `draw_gameplay`: header + tab bar + **one** screen filling the content rect | Persistent header + **left hoard rail** + **bottom strip** + tab-swapped **center** |
| Header | Title, 3 flat badges (Gold, gold/sec, Hoard Pts), Save/Menu | 3 framed **resource cards** (icon + big value + `+rate/sec` sub-line) + Save/Menu + **settings gear** |
| Click target | Lives inside the Hoard tab (`hoard.rs`) | Persistent **left rail**: hoard art, circular CLICK (+per-click), Hoard Income, **Run Time since prestige** |
| Minions | One homogeneous `goblins: u32` count (`minions.rs`) | **Typed roster** (Kobold/Worker/Scavenger/Thief…), per-type level + rate + hire cost + unlock gate; `n/max` cap |
| Prestige | Flat scrollable grid of `prestige_upgrades.json` (`prestige.rs`) | **"Hoard Legacies"** 5-branch tree (Greed/Power/Discovery/Legion/Eternity), `0/10` per branch, tiered nodes, prereq **locks**, connectors |
| Prestige side | Burn meter + Prestige button | **Burn the Hoard** card (bonfire, "+N Hoard Points", prestiged-count) + **Prestige Multipliers** summary (xN per stat) |
| Bottom strip | — | **Minions** type cards + **Expeditions** panel (count badge, View button) + **Recent Treasures** list |
| Visual style | Flat `SurfaceStyle` rects, bitmap-font text | Ornate gold-etched frames, corner ornaments, textured panels, **iconography** everywhere |
| Icons/art | None (text only) | Coin, minion face, hoard-point crystal, bonfire, chest, per-branch glyphs, minion portraits, treasure icons |

**Naming note.** The mockup predates the goblin→kobold rebrand (it labels a minion type "Goblin"). Keep the
generic tab/category label **"Minions"**; individual **type names live in JSON** and should follow the kobold
theme (see `[[minion-types]]` decision in §9). Internal keys (`goblins` field, `Goblins` stat, `gold_per_goblin`)
stay as-is per the rebrand's save-compat decision.

---

## 2. Principles this plan must honor (from `CODE_STANDARDS.md` / `AGENTS.md`)

- **Data-driven first.** Branch/node trees, minion types, multiplier rows, and the icon manifest are **JSON under
  `assets/data/`**, loaded via `data_loader` — no hardcoded rosters or balance in Rust.
- **Toolkit-first.** The ornate frame, corner ornaments, resource cards, and icon rendering are candidate
  **`macroquad-toolkit` upgrades** (it already has `SurfaceStyle`, plaques w/ corner marks, asset/texture cache,
  raster, tooltips), not project-local one-offs. Only diverge if genuinely game-specific.
- **800-line hard limit per `.rs`.** Each new region is its own module (`ui/frame.rs`, `ui/prestige_tree.rs`,
  `ui/bottom_bar.rs`, …). Do not grow `ui.rs`.
- **UI is a pure view layer.** Panels read state and push `UiAction` variants; `Game::apply_action` is the only
  mutator. New interactions (buy node, hire type N, open expeditions) become **new `UiAction` variants**.
- **Save compatibility.** Any `RunState`/`PersistentState` change is additive with `#[serde(default)]` + a load
  path that tolerates old saves (existing `from_save` pattern).

---

## 3. Data changes (`assets/data/`)

1. **`prestige_upgrades.json` → branch/tree schema.** Add `branch` (greed|power|discovery|legion|eternity),
   `tier` (row), `prereq` (node id or none), and keep `id`/`effect`/`max_level`/cost. Existing nodes map onto
   branches; **ids stay stable**. The 5 branches ≈ the 5 Prestige-Multiplier rows.
2. **New `minions.json`.** Ordered minion **types**: `id`, `name`, `base_rate`, `base_cost`, `cost_growth`,
   `unlock_at` (total-minion gate). Drives the bottom strip + Minions tab.
3. **New `icons.json` (manifest).** Logical name → asset path (or procedural-raster recipe), so icon choice is data.
4. **`game_config.json`.** Add `minion_cap` tiers (n/max) and any new base values; no code defaults (fail-fast).

## 4. State & simulation

- **Run time:** add `run_started_at` timestamp to `RunState`; format `Xh Ym Zs` (toolkit time helper if present).
- **Typed minions:** replace/extend `goblins: u32` with `minion_counts: HashMap<String,u32>` (keep `goblins` as an
  alias/derived total for save-compat + existing `Goblins` stat/formula until fully migrated). Economy
  `gold_per_second` sums per-type `count * base_rate * efficiency`.
- **Prestige multipliers:** derive the xN rows (Gold/Click, Gold/Minion, Discovery, Minion Efficiency, Treasure
  Quality) from prestige-tree levels; expose `prestige_multipliers(&data) -> [(label, f64)]` for the summary card.
- **Tree unlocks:** a node is buyable iff `prereq` maxed (or none) and Hoard Points ≥ cost; lock state is derived,
  not stored.

## 5. UI structure — persistent frame

Rework `ui::draw_gameplay` into a **frame** that lays out four persistent regions, then dispatches only the center:

```
┌ header (resource cards + save/menu/gear) ───────────────────────┐
│ ┌ left rail ┐ ┌ center (tab-swapped) ───────────┐ (right sub-   │
│ │ hoard art │ │  Hoard | Minions | Upgrades |    │  columns are  │
│ │ CLICK     │ │  Treasures | Achievements |      │  part of each │
│ │ income    │ │  Prestige(tree) | Codex          │  center tab)  │
│ │ run time  │ └──────────────────────────────────┘               │
│ └───────────┘ ┌ bottom strip: Minions · Expeditions · Treasures ┐│
└─────────────────────────────────────────────────────────────────┘
```

New modules (each < 400 lines):
- `ui/frame.rs` — region rects + header cards + left rail + bottom strip; owns the persistent chrome.
- `ui/left_rail.rs` — hoard art, circular CLICK button, income, run time (moves click out of `hoard.rs`).
- `ui/bottom_bar.rs` — minion-type cards, expeditions summary, recent-treasures list.
- `ui/prestige_tree.rs` — the 5-branch node tree + connectors + lock rendering (replaces the grid in `prestige.rs`;
  keep `prestige.rs` for the Burn card + Multipliers summary).
- **Center reflow:** with click now persistent, the **Hoard tab center** becomes the Chronicle/overview.

## 6. Toolkit extensions (upstream to `macroquad-toolkit`)

- **Ornate frame style:** extend `SurfaceStyle` (or add `FramedPanel`) with corner ornaments + double gold border,
  reusing the plaque corner-mark work. One helper, themed via `colors`.
- **Resource card widget:** icon + title + big value + delta sub-line (used ×3 in header, reusable for stat tiles).
- **Icon draw helper:** name → texture (from `AssetManager`) with a procedural-raster fallback so builds without
  art still render placeholders. Keeps icon choice data-driven (§3.3).

## 7. Assets

- Prefer a **small PNG icon set** under `assets/` (coin, minion, crystal, bonfire, chest, 5 branch glyphs, minion
  portraits, treasure marks) loaded through the toolkit `AssetManager`; embed for WASM.
- Fallback: procedural raster glyphs (toolkit `raster`) so the game is never blocked on commissioned art — matches
  the GDD §0 "lowest-art genre" stance. Decide per-icon during Phase 5.

## 8. Phased delivery

- [x] **P1 — Frame skeleton.** `ui/frame.rs` carves header/left-rail/center/bottom regions; header now shows three
      resource cards + SAVE/MENU/gear (gear opens a settings overlay during gameplay via transient
      `GameplayState::settings_open`). Center renders existing tab screens in the reduced rect; rail + bottom are
      labelled placeholders. Verified via `ui_hoard`, `ui_prestige` captures.
- [x] **P2 — Left rail.** `ui/left_rail.rs`: persistent hoard art placeholder, circular CLICK target (radial
      hit-test), live Hoard Income, and Run Time (new save-safe `RunState::run_seconds`, ticked each frame, reset on
      prestige). Hoard tab reflowed to Chronicle + expedition/prestige column; click removed from `hoard.rs`.
      Verified: rail persists across tabs, run time accrues.
- [x] **P3 — Bottom strip.** `ui/bottom_bar.rs`: persistent three-column strip — minions summary (type-card
      placeholder for P4), the Expeditions launcher (moved off the Hoard tab, so it's reachable from any tab), and a
      Recent Treasures list (newest-first, rarity-tagged, reusing `treasures::rarity_color`). Hoard tab side column
      is now just Prestige Progress. Verified via captures.
- [x] **P4 — Typed minions.** `minions.json` (Worker/Scavenger/Thief tiers) + `data/minions.rs`; additive,
      save-safe `RunState::minion_counts` (base Kobold stays `goblins`); `economy::extra_minion_income` folded into
      `gold_per_second`; `total_minions()` now drives the header/rail count + the `Goblins` stat. Minions tab is a
      tier grid, bottom strip has compact tier cards, both gated by `unlock_at`; new `HireMinion(id)` action. Balance
      sim untouched (extra income is 0 until hired) — 36 tests green. Verified via minions/hoard captures.
- [ ] **P5 — Prestige tree.** Branch schema, `ui/prestige_tree.rs`, connectors + locks, Burn card, **Multipliers**
      summary. *Verify: buy gated by prereq/HP; capture matches mockup.*
- [ ] **P6 — Icons.** PNG/procedural icon set + manifest; wire into resource cards, minion cards, treasure/branch
      glyphs. (Theme colors already landed in PT.) *Verify: full-screen capture vs. `image.png`.*
- [ ] **P7 — Polish.** Hover/press states, tooltips, `+N` float feedback, balance retune, docs (`gdd.md` §9,
      `IMPLEMENTATION_PLAN.md`).

## 8b. Theme pass (PT) — warm "hoard" palette  ← REPRIORITIZED TO NEXT

**Why now:** the mockup's identity is a warm gold-on-dark-brown treasure aesthetic. Correct layout in the toolkit's
cool blue-grey `dark::*` palette still reads as default Rust UI (user feedback). Theming must come *before* the
remaining layout phases (P4/P5) so every new panel is born themed.

The palette lives in a project-local `ui/theme.rs` (a game-specific color identity is a legitimate divergence from
the toolkit's generic `dark` palette; the reusable *ornate-border rendering* is still a candidate toolkit upgrade
later). Approx. values sampled from `image.png`:

| Role | Value (linear-ish RGB) | Use |
| --- | --- | --- |
| `BACKGROUND` | `0.047, 0.035, 0.024` | near-black warm brown behind everything |
| `PANEL` | `0.090, 0.067, 0.039` | panel/card fill (dark brown) |
| `PANEL_HEADER` | `0.120, 0.094, 0.063` | panel title band |
| `PANEL_DARK` | `0.059, 0.043, 0.027` | insets, art wells, disabled |
| `BORDER` (gold) | `0.847, 0.663, 0.290` | bright etched frame border |
| `BORDER_DIM` (bronze) | `0.420, 0.325, 0.165` | subtle dividers/card borders |
| `ACCENT` / `GOLD` | `0.910, 0.710, 0.320` | signature gold accents |
| `TEXT` | `0.847, 0.800, 0.690` | body (warm cream) |
| `TEXT_BRIGHT` | `0.950, 0.894, 0.753` | headings (gold-cream) |
| `TEXT_DIM` | `0.541, 0.490, 0.392` | muted tan |
| `POSITIVE` | `0.560, 0.720, 0.350` | income / "Common" green |
| `NEGATIVE` | `0.757, 0.337, 0.259` | warm red |
| `WARNING` | `0.878, 0.576, 0.180` | amber |
| `HOARD_POINT` | `0.604, 0.435, 0.769` | purple (Hoard Points / prestige) |

**PT-a — foundation (biggest lever): ✅ DONE.** `ui/theme.rs` palette + `draw_corner_marks`; reskinned
`ui::panel`/`ui::button`, background, frame chrome, and all card fills; swept 55 `dark::*` → `theme::*` across 14
files. Whole UI now reads gold-on-brown (verified hoard/prestige captures).
- `ui/theme.rs`: the palette above + `draw_corner_marks(rect, color)` (small gold L-brackets at each corner → the
  "ornate frame" read).
- Rewrite the two shared widgets everything funnels through:
  - `ui::panel()` → dark-brown fill, gold `BORDER`, header band + gold divider, **corner marks**, gold-cream title.
  - `ui::button()` → warm tones: Primary = gold fill/dark text, Secondary = brown + gold-dim outline + gold text,
    Positive = green, Danger/prestige = amethyst (mockup's purple PRESTIGE).
- `clear_background` → `theme::BACKGROUND`.
- Reskin the frame chrome (header, resource cards, left rail, bottom strip) hardcoded colors → theme.
- **Sweep** the 55 `dark::*` references → `theme::*` across the 14 UI files (names line up: BACKGROUND, ACCENT, TEXT,
  TEXT_BRIGHT, TEXT_DIM, POSITIVE).
- Reskin the remaining cool blue-grey card fills (`0.11,0.125,0.16` etc. in prestige/treasures/achievements/
  upgrades) → `theme::PANEL` + `theme::BORDER_DIM`.
- *Verify: capture hoard/prestige/upgrades — the whole UI reads gold-on-brown.*

**PT-b — ornate depth: ✅ DONE.** `ui::panel` gained an inset bronze line under the gold border (etched double-frame
read); the active tab now renders as a lit gold-topped panel with a bright label + gold underline (was falling
through the disabled path and rendering dark); the "Dragon's Den" wordmark is now a gold treatment with a soft drop
shadow. Verified via prestige/upgrades captures. Remaining ornate ideas (panel gradient/vignette, corner filigree as
a toolkit `FramedPanel`) deferred to P7/toolkit work.

Each phase: `cargo fmt` + `cargo clippy -D warnings` + `cargo test`, then `scripts/capture_ui.ps1` for the touched
scenes.

## 9. Open decisions (flag before building)

- `[[minion-types]]` **Type roster & names** — adopt the mockup's Worker/Scavenger/Thief tiers? Base tier named
  "Kobold" (per rebrand) vs. keeping "Goblin" as a distinct type? **Content call — needs sign-off.**
- **Full vs. partial minion migration** — keep the single `goblins` count as the base type (smallest change) or
  fully generalize now? Affects save schema + economy.
- **Art budget** — commissioned PNG icons vs. procedural raster placeholders (P6/P7).
- **Prestige tree size** — nodes-per-branch and the exact `0/10` meaning (levels vs. distinct nodes).

## 10. Out of scope

Dragon breeding/world-gen/combat (already cut in GDD §0); animated art; the persistent frame does **not** add a
world/camera view — it stays UI-only per GDD §1.
