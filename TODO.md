# TODO — Dragon's Den

## Existing engineering and content backlog

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

- [ ] **Finish audio playback.** Add a small licensed sound pack with attribution
  and wire click, purchase, expedition, unlock, prestige, and menu feedback
  through toolkit audio. Add music or remove the inactive music option; verify
  mute and master/SFX/music controls on Windows and WebGL.

- [ ] **Reconcile project documentation and stale comments.** Update `README.md`,
  `gdd.md`, and Codex source comments for the shipped 15 dragons, active Codex
  bonuses, tested 100-prestige curve, and 12-hour offline cap. Align the GDD's
  asset paths, loading APIs, and module layout with implementation; correct the
  obsolete non-test-line/exception comments in `tests/code_standards.rs`.

## UI_STYLE review — 2026-09-20

### Evidence, scope, and implementation order

Reviewed `AGENTS.md`, `UI_STYLE.md`, `CODE_STANDARDS.md`,
`GAME_DEVELOPMENT_GUIDE.md`, `README.md`, and `gdd.md`; no project-local
`PROJECT_AGENTS.md` was found. Inspected all UI screen modules, `src/game.rs`,
menu/gameplay state and actions, the capture harness, and the toolkit's
`VirtualUi` layout/camera conversion. This is an audit, not a UI implementation.

**Visual evidence:** inspected all nine existing `docs/verification/ui_*.png`
images (menu, settings, Hoard, Minions, Upgrades, Treasures, Achievements,
Prestige, Codex), each 1264x681. They show mostly fresh/locked gameplay and are
archived evidence, not captures of a freshly built current revision. Source
inspection corroborates the persistent layout. The Hoard screenshot's settings
symbol differs from the other captures, so do not assume one capture/build date.

**Code evidence:** logical canvas is fixed at 1280x720; `frame::regions()` gives
the center 1018x398 (about 44% of the logical screen), before inner panel padding.
The game deliberately has no spatial world/camera gameplay (GDD §§1, 7); useful
framing here means enlarging the hoard or current comparison, not adding pan/zoom
or a world map. VirtualUi already maps rendering and pointers; no coordinate
conversion defect was established. This is a bespoke ornate frame, not verified
unmodified template UI; apply the template subtraction principles without
claiming its origins are a defect.

**Limits:** no game execution, new captures, browser/touch interaction, resized
rendering, dense/unlocked states, or publish run in this documentation-only pass.
Neither README nor GDD declares a minimum supported viewport. Use **1280x720
normal, 800x600 compact, and 390x844 portrait canvas CSS sizes** as proposed
verification targets, not already-supported or verified sizes. UI-01 must record
the actual support contract; test actual embedded canvas bounds, not just the
outer browser window. At every declared supported size, aim for at least 44x44
CSS-pixel touch targets and readable text without relying on browser zoom.

Existing modal-input, touch-onboarding, prestige-confirmation, and release-check
tasks are merged below (UI-03, UI-04, UI-06, UI-10); their requirements remain open.
Other existing tasks remain above. There were no completed checkboxes to preserve.
Implement composition before details: UI-01 → UI-02; UI-03 underpins new overlays;
UI-04–UI-09 follow those foundations; UI-10 closes the evidence gaps. Retain costs,
risks, useful unlock hints, history access, save recovery, and visible utilities.

### Verified findings — screenshots and/or source, as labeled

- [ ] **UI-01 — Recompose normal play around collecting and the next purchase.**
  **Scope:** all gameplay, especially fresh Hoard; `src/ui.rs::draw_gameplay`,
  `draw_tab_bar`, `src/ui/frame.rs::regions/draw_header`, `left_rail.rs`,
  `bottom_bar.rs`, `hoard.rs`; README screen description and GDD §9.
  **Observed (screenshots + code):** resource cards, seven equal tabs, hoard rail,
  Chronicle, Prestige Progress, minion cards, Expeditions, and Recent Treasures
  compete through similar borders/headings. The actual hoard art is a 108-pixel
  high inset; most of the initial center is empty history/progress space. Minion
  hiring appears both in its tab and the permanent strip. SAVE is a colored
  filled control even while useful gameplay actions are subdued.
  **Change:** first record UI_STYLE §1 briefs and supported sizes for collection,
  purchase, collection browsing, and prestige phases. Make the hoard/collect
  action the main focus on Hoard with a supporting next-hire/upgrade area. Give
  shops/collections/prestige the main area when selected; collapse or remove the
  permanent click rail and unrelated bottom cards there. Keep an explicit route
  back to collecting and expeditions. Replace large empty progress/history
  panels with a compact next milestone and an expandable Chronicle. Consolidate
  collection navigation; reserve an unmistakably separate, quiet utility area
  for Menu/Save/Settings, never beside gameplay choices in the same action group.
  Remove redundant panel headings, inner frames, corner marks, and nested borders
  as part of the recomposition rather than merely recoloring the dashboard.
  **Accept:** at most 2–3 strongly emphasized regions in normal play, a clear
  first place to look, and most usable area devoted to the current decision.
  Collect is prominent early; comparisons dominate shops; essential actions and
  utilities remain discoverable, without large accidental empty regions.
  **Verify:** fresh, first hire, expedition-capable, each tab, and prestige-ready
  screenshots at all agreed sizes. Trace touch paths Collect → Hire → Upgrades
  → Explore → Hoard and Menu/Settings separately; review hierarchy at a glance.

- [ ] **UI-02 — Reflow the frame and collections instead of shrinking all controls.**
  **Scope:** every screen; `src/game.rs::draw`, `src/ui.rs` logical sizing and
  widgets, `frame.rs::regions`, grid layouts in `minions.rs`, `upgrades.rs`,
  `treasures.rs`, `achievements.rs`, `dragons.rs`, and `settings.rs::minus_plus`.
  **Observed (code; compact visual behavior untested):** one fixed canvas is
  uniformly letterboxed; no compact/portrait layout exists. Bottom hires are
  28 logical pixels high, expedition controls 32, settings steppers 36, and
  prestige discs 44 across before further scaling. At 800 pixels wide, a
  28-pixel target scales to 17.5 pixels. Screenshots already show tiny dense
  bottom cards and prestige labels. Text fitting cannot supply usable targets.
  **Change:** after UI-01, introduce viewport-aware layout modes using toolkit
  layout/pointer helpers. Collapse secondary navigation, use fewer grid columns
  or vertical lists, scroll long views deliberately, and size controls in terms
  of their rendered touch area. Reflow header values, settings, inspectors and
  confirmations; retain the primary action in reach. Do not add parallel local
  coordinate math or solve crowding by further shrinking text.
  **Accept:** all supported sizes meet the declared target/text policy; no
  clipped costs, overlapping labels, inaccessible final rows, or offscreen Back
  controls. Picking agrees with drawing after resize and display scaling.
  **Verify:** proposed 1280x720, 800x600, 390x844 canvases, browser embedded and
  fullscreen, native resize, 100%/150% display scaling and supported text-scale
  extremes. Tap edge controls and drag every long collection to its final item;
  inspect long names, large amounts, and open details. Record unsupported sizes.

- [ ] **UI-03 — Give modals and temporary targets exclusive input ownership.**
  **Scope:** settings over every gameplay tab, forthcoming confirmations and
  inspectors, Golden Hoards; `src/ui.rs::draw_gameplay/button/draw_golden_hoard`,
  `src/ui/left_rail.rs::draw_click_button`, `src/game.rs::update/apply_action`,
  `src/ui/prestige_tree.rs`, and toolkit input/scroll integration.
  **Observed (code, not interaction-tested):** settings is drawn after underlying
  controls have collected actions and lists have updated; Escape still requests
  BackToMenu. Golden Hoards are drawn over interactive center content after its
  actions have been gathered. Generic and radial buttons activate on release
  inside without checking where the press began. These paths permit unintended
  actions beneath overlays or after a drag.
  **Change:** route input to the topmost active surface before processing other
  controls; suppress underlying actions, scrolling and shortcuts. Use toolkit
  press/release ownership and drag-cancellation helpers, preserving the theme.
  Golden Hoard collection must consume its gesture; constrain its position to a
  safe play surface. Retain a visible close/cancel control; Escape may close the
  top modal but must not silently leave gameplay through it.
  **Accept:** one gesture causes one intended action, no purchase/tab change/menu
  transition occurs behind a modal, and dragging into a button cannot activate it.
  **Verify:** at all agreed sizes, tap modal controls above an underlying purchase,
  drag then release over a hire/Collect/prestige node, tap overlapping Golden
  Hoards, close/reopen settings, and test touch-only dismissal plus Escape.

- [ ] **UI-04 — Make prestige an inspectable, informed decision with confirmation.**
  **Scope:** locked, affordable, maxed and reset-ready Prestige;
  `src/ui/prestige.rs::draw/draw_burn_panel/draw_multipliers`,
  `prestige_tree.rs::draw_header/draw_node`, `src/ui.rs::tooltip/UiAction`,
  `src/game.rs::prestige/apply_action`, gameplay state.
  **Observed (screenshots + code):** five small branch chains and a separate
  multiplier panel share the reduced center. Header icons occupy label centers
  in the saved image; node names/status are 12-point text. Only affordable,
  unlocked, non-maxed nodes can hover for details, and releasing there purchases
  immediately. Locked nodes say only "locked". PRESTIGE directly resets the run
  without a confirmation describing losses and retained progress.
  **Change:** give the tree/comparison space reclaimed by UI-01. Tap any node to
  select it and show a readable inspector with effect, current/next level, cost,
  named prerequisite or shortage, and separate Buy control. Separate icons from
  text; reveal detailed multipliers on request. Add a Burn confirmation showing
  exact Hoard Points gained, reset gold/minions/run upgrades, retained permanent
  upgrades/treasures/achievements/Codex, and Confirm/Cancel. Recheck eligibility
  on confirmation and prevent duplicate activation. Keep the relevant currency
  and consequences beside each decision.
  **Accept:** a touch player can compare even unaffordable nodes without spending,
  understand how to unlock one, cancel a reset without change, and recognize the
  permanent result after confirming. Costs and warnings survive the simplification.
  **Verify:** all agreed sizes; inspect locked/affordable/maxed nodes, buy once,
  cancel/confirm a prestige, test changed eligibility and double taps. Capture a
  dense progressed tree and reset confirmation. Preserve useful regression
  coverage for cancellation, confirmation and duplicate/invalid requests in tests/.

- [ ] **UI-05 — Give persistent facts one home and move event history out of the spotlight.**
  **Scope:** header, Hoard, Minions, expedition feedback and offline return;
  `frame.rs::draw_header`, `left_rail.rs::draw_footer`, `bottom_bar.rs`,
  `hoard.rs::draw_action_log`, `src/game.rs` notifications/transition,
  `src/ui.rs::draw_frenzy_banner`, gameplay `action_log`.
  **Observed (screenshots + code):** the same passive income is printed under
  Gold, under Kobolds, in the rail, and on Minions. Minion totals likewise repeat.
  Recent Treasures permanently occupies a panel even when empty, while Chronicle
  dominates fresh Hoard. Existing floating gains and toasts already provide
  temporary feedback and should be retained. Frenzy is positioned over the top
  of the center; actual overlap during play remains unverified. Offline return
  announces the earned amount only through a toast, without a log entry.
  **Change:** keep one compact gold/rate summary, counts by their relevant hire
  decision, and Hoard Points by prestige spending; move run time into details.
  Replace permanent recent-find cards with brief discovery feedback linked to
  the collection, and make Chronicle explicitly expandable/retrievable. Wrap or
  scroll long log entries rather than drawing them past the region. Place active
  Frenzy/Rush duration and effect next to affected collecting/income/exploration
  controls without covering them. Record the offline award in retrievable session
  history so the player can inspect it after the toast expires.
  **Accept:** each normal-play fact has a clear home; event messages disappear but
  resulting ownership, balances, active buffs and important rewards remain clear.
  **Verify:** normal play and repeated hires, long treasure discoveries, no-find
  expeditions, offline return, simultaneous Rush/Frenzy and expired toasts at all
  agreed sizes. Check history access and overlay placement by touch.

- [ ] **UI-06 — Teach the loop on demand and disclose advanced systems gradually.**
  **Scope:** new save through first prestige; `src/ui.rs::draw_tab_bar`,
  `hoard.rs`, `bottom_bar.rs`, `minions.rs`, `upgrades.rs`, collection modules,
  `src/state/gameplay.rs` and onboarding state; `game_page.json` control copy.
  **Observed (screenshots + code):** all seven tabs, six hire tiers, Hoard Points,
  prestige progress and permanent empty-collection prose are present from the
  first screen. Minions shows soft-cap prose even at 0/60. No dismissible in-game
  onboarding/help route is implemented; the browser-page control hint alone
  does not teach the sequence, Golden Hoards or first-use systems.
  **Change:** show available choices plus a compact next unlock; put distant
  locked tiers/systems behind visible preview/disclosure controls. Reveal full
  prestige details when relevant, including when achievement-earned points are
  spendable, not solely after the first reset. Show soft-cap warnings near an
  affected hire as the cap approaches, keeping full rules in help. Add brief,
  dismissible first-use prompts naming actual controls for Collect/CLICK, Hire,
  Explore Ruins, Prestige, Golden Hoards, MENU and drag-to-scroll, plus reopenable
  Help. Remove permanent empty-state tutorial paragraphs once taught.
  **Accept:** a new player knows the next action without reading a dashboard;
  future goals remain discoverable and no relevant cost, cap penalty, spendable
  currency or recovery control is hidden. Returning players see completed lessons
  dismissed and can reopen them.
  **Verify:** fresh touch-only loop through first hire/upgrade/expedition, first
  Golden Hoard, first currency award and prestige; test Help dismissal/reopening,
  returning save, and the minion cap boundary at normal/compact/portrait targets.

- [ ] **UI-07 — Make purchase quantities and income effects explicit at the action.**
  **Scope:** minion and upgrade purchases, including any retained quick-hire
  shortcut; `bottom_bar.rs::draw_minion_card`, `minions.rs::draw_card`,
  `upgrades.rs::draw`, `src/ui.rs::buy_mode_selector/bulk_quote`, and the shared
  purchase-quote engineering task above.
  **Observed (code):** bottom hire buttons show only a cost while using global
  `buy_mode`, whose selector is visible only on Minions/Upgrades. A player can
  change to Max, switch tabs, then unknowingly spend for a Max hire. Minion cards
  label base `rate_each` as income per unit even when multipliers or the soft cap
  change the actual contribution. Upgrade effects use raw rates (e.g. +0.05
  discovery chance) that are harder to compare than player-facing units.
  **Change:** consolidate purchases per UI-01; any retained shortcut must show
  Hire quantity, total gold cost and current mode with a nearby way to change it.
  Use the shared state/simulation quote for both display and execution. Label
  base rates honestly or show the actual marginal income, including diminishing
  returns; format chance/multiplier effects in clear units and put shortages or
  prerequisites beside disabled actions.
  **Accept:** players know exactly how many units they buy, the total price and
  useful effect before tapping; switching tabs cannot conceal the active mode.
  **Verify:** x1/x10/Max across tab changes, insufficient funds, final upgrade
  levels, active multipliers and hires crossing the soft cap at all agreed sizes.

- [ ] **UI-08 — Make collections readable and inspectable without leaking unknown entries.**
  **Scope:** Treasures, Achievements and Codex, plus shared tooltip rendering;
  `treasures.rs::draw`, `achievements.rs::draw`, `dragons.rs::draw`,
  `src/ui.rs::tooltip/item_fully_visible`, and collection selection state.
  **Observed (screenshots + code):** repeated unknown-treasure prose fills equal
  cards; Codex uses dark text on dark elemental badges, especially shadow/earth.
  Cards reserve fixed small text areas for description, condition and bonus.
  Treasure tooltip receives the real name/description unconditionally, even for
  "???" and sealed cards, contradicting their undiscovered presentation. There
  is no explicit select/detail/dismiss interaction; retained pointer position is
  assumed to make hover tips touch-accessible. Unlocked long-copy overflow is
  a risk to inspect, not visually verified here.
  **Change:** show concise name/status/effect summaries and a tap-open readable
  inspector for full flavor, bonuses, conditions and rewards; keep unknown
  identity hidden until discovery while revealing the relevant unlock hint.
  Consolidate repeated unknown entries/prose and provide deliberate browsing of
  locked entries. Use readable badge text contrast regardless of element color,
  and keep selection/ownership understandable without color alone. Allow details
  to wrap/scroll and close visibly; show tooltips/inspectors above later cards.
  **Accept:** every owned bonus and achievement reward can be read by touch;
  locked items do not expose hidden names; badges and long details remain legible.
  **Verify:** each collection's first/last row, locked/unlocked and longest entries,
  selection/dismissal and drag cancellation at all agreed sizes, with increased
  text scale. Check that a purchase or scroll cannot be triggered behind details.

- [ ] **UI-09 — Separate starting play from save management and keep recovery readable.**
  **Scope:** menu, load failure and settings; `src/ui/menu.rs::draw`,
  `src/state/menu.rs`, `src/game.rs::transition/delete_save`, `settings.rs`.
  **Observed (screenshot + code):** Continue, New Game, Delete Save and Settings
  share one similarly prominent vertical stack; Delete Save has a strong filled
  treatment. New Game and Delete Save execute without confirmation. The load-error
  notice starts at `y + 12` relative to Settings' top, so its rectangle overlaps
  Settings by construction (failure state not visually captured). Settings exposes
  inactive audio controls and permanent development-status prose.
  **Change:** emphasize Continue when a save exists, otherwise New Game; move
  Delete Save into a separate save-management section with explicit confirmation.
  Confirm starting over when an existing save would be replaced. Allocate a
  nonoverlapping persistent error area with readable details and visible recovery
  choices. Keep settings/recovery routes available. Coordinate audio controls
  and removal of development prose with the existing audio task rather than
  pretending unsupported playback works.
  **Accept:** accidental utility taps cannot discard a run; failed Continue leaves
  an understandable recovery screen with working Settings, retry/new-game choices
  as appropriate, and no need for a keyboard or transient toast.
  **Verify:** no save, valid save, failed load, cancel/confirm reset and deletion,
  settings return and long error text at all agreed sizes using isolated test saves.

### Further inspection and release evidence — not verified defects

- [ ] **UI-10 — Complete the visual, touch and release acceptance matrix.**
  **Scope:** `scripts/capture_ui.ps1`, `src/game.rs::begin_capture_scene`,
  `src/main.rs`, `docs/verification/`, and all changed screens above.
  **Gap (verified harness inspection):** default captures include only menu,
  Hoard and settings. Other tab names select fresh gameplay; there are no seeded
  dense, selected, prestige-ready, failure or active-event scenes. Existing
  screenshots cannot establish minimum-size usability, current-build behavior,
  tooltip stacking, modal isolation, long-entry overflow or touch correctness.
  **Change:** expand the default scene set to all seven tabs plus menu/settings,
  add deterministic isolated fixtures for supported dense/late-game, unlocked,
  scrolled, selected, reset-ready, load-failure, Golden Hoard, Rush/Frenzy and
  offline-return states. After the layout/interaction changes, capture the
  declared normal/minimum sizes and relevant portrait layout; replace equivalent
  captures directly in `docs/verification/`, without nested directories. Record
  scene, actual canvas size, build revision and tested interactions. Investigate
  remaining risks before describing them as confirmed failures.
  **Accept:** UI_STYLE §9 checklist has evidence for every affected screen:
  2–3 attention regions, useful focus, no repeated irrelevant facts, contextual
  systems, readable text/targets, touch inspection and dismissal, and durable
  understanding after feedback expires. Browser/native limitations are explicit.
  **Verify:** on Windows and browser touch, exercise new/continue/save recovery,
  offline return, all purchases and collections, scroll to last entries, events,
  prestige, settings and navigation at the declared sizes and display scales.
  Run formatting, Clippy, tests and parameterless `.\publish.ps1` after meaningful
  game changes, report results/blockers, and commit each independently useful
  implementation change. Compilation or screenshots alone do not close this task.
