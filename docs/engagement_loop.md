# Engagement Loop — iteration prompt

One iteration = one focused, committed improvement to Dragon's Den's
engagement, following `ENGAGEMENT_REVIEW.md` (diagnosis + priorities). Balance
P1 (loop acceleration + first wall structure) is done; work through what
remains, then deepen content.

## Each iteration

1. **Orient.** Read `ENGAGEMENT_REVIEW.md`'s status note and `git log --oneline
   -10` to see what's already landed. Do not redo finished work.
2. **Pick exactly ONE item**, in this priority order:
   - **Tier 2 — exploration & mid-game pull** (review items 6–8): scaling
     expedition cost; a payoff so Explore never dead-ends (gold / temporary
     Hoard Rush buff / treasure-dust reroll currency); treasure long tail with
     rarer, stronger, prestige-gated finds.
   - **Tier 3 — session hooks** (review items 9–11): Golden Hoard glint +
     Dragon's Frenzy burst; more prestige-gated qualitative unlocks (new
     upgrade line, second expedition slot); soft-caps with wall-breaker
     unlocks.
   - **Content depth** (after the above): extend the ladders so every system
     has a long chase — more treasures (aim 35+, rarity spread, prestige
     gating), achievements (aim 35+, covering new mechanics), codex dragons
     (12+), a 5th/6th minion tier for deep prestige counts, more run upgrade
     lines, prestige tree tiers 3+ per branch, deeper wall cliffs. Balance
     numbers live in `assets/data/*.json` — content first, code only for
     genuinely new mechanics.
3. **Implement fully.** Project rules: data-driven JSON, formulas in
   `simulation/` with tests beside them, UI stays a pure view layer returning
   `UiAction`s, randomness through the state-owned `SeededRng`, no file over
   800 lines, no new `mod.rs`, toolkit-first.
4. **Validate — all must pass before committing:**
   - `cargo test` (all), `cargo clippy --all-targets --all-features -- -D
     warnings`, `cargo fmt -- --check`.
   - The balance guards must stay green: first prestige 10–40 min, cycle 2
     ≤ 70% of cycle 1. If a new system changes the economy, extend
     `simulation/balance.rs` to cover it rather than weakening asserts.
   - If UI changed: `..\macroquad-toolkit\scripts\capture_ui.ps1 -Scenes
     <changed tabs>`, then READ the PNGs and fix any layout overflow before
     committing.
5. **Record.** Update the status note at the top of `ENGAGEMENT_REVIEW.md`
   (what landed, what's next), then commit everything with a descriptive
   message ending in the Claude co-author trailer.

## Guardrails

- One item per iteration; no drive-by refactors bundled in.
- Never run `publish.ps1`, workspace-wide scripts, or anything that deploys.
- Commit only to this repo (`dragons_den`), on the current branch.
- If validation cannot be made green inside an iteration, revert the working
  tree to the last commit and note the blocker in `ENGAGEMENT_REVIEW.md`
  instead of committing broken work.

