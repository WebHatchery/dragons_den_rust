# Dragon's Den — Engagement Review & Fun Plan

*Companion to `gdd.md` (design authority) and `IMPLEMENTATION_PLAN.md` (build
status). Where those two describe **what was built and why**, this document
diagnoses **why the built game plays boring** and lays out concrete, prioritized
ways to make it entertaining. All numbers are the ones actually shipping in
`assets/data/*.json` as of this review.*

> **Status update:** Tier 1 has landed, combined with the wall/obstacle
> structure originally parked in Tier 3 (#10): threshold growth 8→3.5, divisor
> 10,000→150, reward exponent sqrt→0.55, a compounding ×2/level "Dragon's
> Avarice" all-gold node, a prestige-2-gated Drake minion tier, and a CI
> regression test asserting cycle 2 ≤ 70% of cycle 1 (currently ~42%).
>
> **Tier 2 #6 (scaling expedition cost) has landed:** `explore_cost` now rises
> `base × 1.25^discovered` (new `explore_cost_growth` config + `economy::
> explore_cost`), so the treasure sweep is an escalating investment rather than
> free button-mashing; a complete set is no longer charged (guaranteed-empty
> outcome). Failed rolls don't raise the price.
>
> **Tier 2 #7 (Explore never dead-ends) has landed:** an expedition that finds
> no new treasure — a miss *or* a completed set — now sparks a **Hoard Rush**,
> a temporary ×2 all-gold surge for 30s (`hoard_rush_multiplier` /
> `hoard_rush_seconds` config; transient `hoard_rush_secs` on `GameplayState`).
> Re-triggering refreshes the timer but never stacks, so the buff is bounded
> and Explore stays a live, rewarding button for the whole game. The surge is
> applied in the `GameplayState` income wrappers (not `economy`), keeping the
> balance sim an honest measure of the core loop. Bottom-bar Expeditions panel
> shows a live "Hoard Rush x2 · Ns left" readout.
>
> **Tier 2 #8 (treasure long tail) has landed:** the catalog grew 16 → 34 with
> a new prestige-gated **Mythic** rarity. `TreasureDef.prestige_required`
> (`serde(default)` 0) gates the strongest finds; `roll_treasure` filters the
> pool to unlocked+undiscovered, so exploration reopens after every prestige
> instead of dead-ending. Mythics span prestige 1–4 (incl. the first
> `all_gold` treasures: +10/+20/+35%). Treasures tab shows gated cards as
> "Sealed · Mythic · Reach Prestige N"; "Treasure Hoarder" achievement retuned
> 16 → 34 (reward 5 → 25 HP) into a real prestige-4 capstone. **Tier 2 is now
> complete.**
>
> **Tier 3 #9 (Golden Hoard active-play burst) has landed:** a clickable golden
> glint spawns every 45–110s and stays clickable for 13s; clicking it grants a
> **Dragon's Frenzy** — a ×7 click-gold surge for 15s (Cookie-Clicker's golden
> cookie, the genre's proven retention hook). All config-driven; transient
> state on `GameplayState` advanced only on live frames (`advance_events`),
> never offline. UI draws the orb (mapped from normalized coords into the play
> area, so state stays pixel-ignorant) + a live Frenzy banner. Also **split the
> gameplay test suite into `state/gameplay/tests.rs`** to bring `gameplay.rs`
> back under the 800-line limit (was 882).
>
> **Tier 3 #10 (prestige-gated qualitative unlocks) has landed:** run-upgrade
> lines can now be gated behind prestige (`UpgradeDef.prestige_required`,
> `serde(default)` 0). Three new lines unlock across prestige 1–3 — Molten
> Veins (+gold/sec), Warband Drums (+minion efficiency), Cataclysm Claws
> (+gold/click) — so prestiging *opens new shop content*, a reason numbers
> alone can't give. Purchase is gated + the shop shows sealed cards with the
> prestige needed. Crucially, `balance.rs` now tracks `Sim.prestige` and
> respects the same gate, so run 1 can't buy unlocked-later lines: guards stay
> honest (first prestige 19.7 min; cycle 2 improved to ~30% of cycle 1 as run 2
> spends the prestige-1 line). Note the minion Drake tier (prestige 2) and the
> #8 treasure gates already deliver the review's other #10 examples.
>
> **Tier 3 #11 (soft-caps + wall-breakers) has landed — Tier 3 is now
> complete.** Base Kobolds hit a soft cap (`minion_soft_cap` 60): beyond it,
> each extra base Kobold earns only `minion_soft_cap_falloff` (34%) until a
> wall-breaker lifts the ceiling. New `EffectStat::MinionCap` channel + a Legion
> tier-2 prestige node **Endless Horde** (+60% cap/level) is the breaker;
> `economy::effective_minions`/`minion_soft_cap` keep the live tick and sim in
> agreement. The cap (60) sits above the first-prestige goblin count (~44), so
> it's a *late-game* wall — guards stay green and unchanged (19.7 min / 30%).
> Minions tab shows a "Kobold soft cap N/60" readout (typed tiers exempt).
> **All of Tiers 1–3 are done.**
>
> **Content depth — achievements 22 → 36:** added two new lifetime counters
> (`expeditions_launched`, `golden_hoards_collected`, both `serde(default)`)
> that let achievements cover the new mechanics — Glint Catcher / Frenzy Addict
> (#9), Ruin Delver / Tomb Raider (#8) — plus deeper tiers of every existing
> ladder (clicks 50k, trillion gold, 100/200 kobolds tying into the #11
> soft-cap wall, 150 upgrades, prestige 3/25/50, a 30-achievement
> Completionist). `StatKey` + `condition_text` + `stat_value` extended to
> match.
>
> **Content depth — codex dragons 8 → 13:** five new dragons spanning the new
> mechanics and deep stats — Tempest Roc (5 Golden Hoards, #9), Ruin Serpent
> (25 expeditions, #8), Molten Colossus (50 kobolds, the first
> `minion_efficiency` dragon), Astral Leviathan (prestige 5) and Worldeater
> Wyrm (trillion gold) — the last two the first `all_gold` codex bonuses.
> Dragons unlock via `check_unlocks`, which the balance sim never runs, so
> guards are untouched. Next content-depth targets: a 5th/6th minion tier for
> deep prestige, more prestige-tree tiers per branch, more run-upgrade lines.

---

## 1. What we actually have (the honest state)

The prototype is *mechanically complete and clean* — this is not a "half-built"
problem. Every system the GDD promised exists and is wired:

- **Click → hire → explore → upgrade → prestige** loop works end-to-end.
- **6 run upgrades**, **3 extra minion tiers** + base Kobold, **17 treasures**,
  **22 achievements**, **8 codex dragons**, **12-node / 5-branch prestige tree**.
- Prestige genuinely grants a permanent currency and permanent multipliers (the
  headline fix over the original, which granted nothing). That fix is real.
- Offline earnings, autosave, settings, ornate-frame UI, floating "+N" juice —
  all present and tested (36 tests, CI-guarded).

So the problem is **not missing features. It's economy tuning and loop shape.**
The game is a correct machine calibrated to be un-fun. That's good news: most of
the fix is JSON, and the rest is small, targeted systems — not a rebuild.

---

## 2. Why it's boring — the three failures, with the math

### Failure A — The prestige loop *decelerates* instead of accelerating

This is the big one, and the user's instinct is exactly right.

- **The wall grows ×8 per prestige.**
  `threshold(n) = 1,000,000 × 8ⁿ` → 1M, 8M, 64M, 512M…
  (`prestige_threshold_growth: 8.0`)
- **The reward grows only ×2.83 per prestige.**
  `HP = floor(sqrt(gold / 10,000))`. At each tier's threshold that's
  `sqrt(100 × 8ⁿ) = 10 × 2.83ⁿ` → 10, 28, 80, 226 HP.
- **The permanent tree is additively hard-capped.** All percent bonuses pool
  into one additive channel per stat (`simulation.rs`,
  `percent_multiplier = 1 + Σpercent/100`). Even *every* gold/sec node maxed
  totals +460% → ×5.6, ever. A one-pool additive cap cannot chase a wall that
  multiplies ×8 each tier.

**Net effect:** each prestige tier takes ~8× longer to reach but pays only
~2.83× the currency, and that currency buys into a bonus pool that tops out
around ×6. So **HP earned per minute of play *falls* every tier**, and the
climb gets slower, not faster. That is the precise opposite of what makes idle
games compulsive.

**Concrete first-prestige trade:** you spend 20–40 minutes of a run (a run that
multiplied your income 10–100×) and get **10 HP**. The cheapest useful node,
Kobold Dynasty (base 5, growth ×2), costs 5 then 10 then 20… so 10 HP buys
**exactly one level → +25% gold/sec, permanently.** You burned half an hour of
compounding growth for a flat +25%. Rationally, you should never prestige. The
player feels this immediately.

> **Root cause the tests missed:** `balance.rs` guards *first-prestige time*
> (10–40 min) but **nothing guards that prestige is worth doing.** The one
> metric that defines fun in a prestige game — "is run N+1 meaningfully faster
> than run N?" — is untested, so it shipped broken.

### Failure B — Exploration is a 5-minute checklist that then dies forever

- Every expedition costs a **flat 100 gold**, forever, for all 17 treasures
  (`explore_cost: 100.0`). Once income clears ~100/sec — which happens in the
  first minute or two — you spam the button and collect the set in a few
  minutes at 30%+ discovery chance.
- Treasures are tiny one-time bonuses. Total gold/sec across *all* gold/sec
  treasures is +1+2+5+10+15 = **+33%**. The whole collection is a rounding
  error next to a single upgrade line.
- After the last treasure, **Explore does nothing but print "All discovered"**
  for the rest of the game. A core verb becomes a dead button.

There is no cost curve, no risk/reward variety, no long tail, and no repeatable
payoff. "Collect treasures" is a two-line story that's over before the game
starts.

### Failure C — No long runway, no scarcity, no walls worth breaking

- Gold is trivially abundant (base click 2 × ~5 clicks/s = 10/s from turn one),
  so nothing forces a *choice* between purchases — you buy everything.
- 6 upgrade lines and 3 minion tiers, most capped at 10–20 levels. Once maxed,
  active play has nothing left to reach for; the numbers plateau instead of
  opening a new exponential band.
- Prestige gives only **+%** — it never *unlocks* anything (a new minion tier,
  a new mechanic, a faster expedition). So there's no qualitative "I want to
  prestige to get X," only a bad quantitative trade (Failure A).

---

## 3. The fix, in priority order

Ordered by **fun-per-effort**. Do Tier 1 first — it's mostly JSON and it makes
the existing machine actually loop. Tiers 2–3 add the durable hooks that keep
idle players coming back for weeks.

### Tier 1 — Make the loop accelerate (mostly JSON + one small code change)

The goal metric: **run N+1 should reach its threshold in ≤ ~50% of run N's
time.** If prestige doesn't visibly speed up the next climb, nothing else
matters.

1. **Slow the wall.** `prestige_threshold_growth: 8.0 → ~3.5`. Tiers should
   come often enough that re-prestiging stays the obvious best move.
   *(JSON only.)*
2. **Make the first prestige generous.** `prestige_divisor: 10,000 → ~250`.
   First prestige jumps from 10 HP to ~63 HP — enough to buy a *real* opening
   boost (several node levels), not one. *(JSON only.)*
3. **Add a multiplicative global-income node to the tree** — e.g. "Dragon's
   Avarice: ×1.5 to *all* gold per level." This is the classic idle engine:
   a geometric permanent multiplier that compounds across prestiges and can
   actually outrun a multiplying wall, which an additive pool never can.
   *(Needs a new multiplicative bonus channel in `simulation.rs` +
   `economy.rs`; ~small, well-isolated.)*
4. **Reward overshoot a little.** Optionally soften the sqrt (e.g.
   `gold^0.55 / divisor`) so pushing to 3–4× threshold before burning feels
   worth it — gives players a *decision* about when to prestige instead of
   burning the instant the button lights. *(One formula line in `prestige.rs`.)*
5. **Add the missing regression test.** Extend `balance.rs` to assert
   *loop acceleration* (time-to-threshold for cycle 2 ≤ K × cycle 1), not just
   first-prestige time. This is what should have caught the un-fun. *(Test.)*

### Tier 2 — Give exploration and the mid-game durable pull (small systems)

6. **Scale the expedition cost.** `explore_cost` should rise with treasures
   found (`base × growth^discovered`) or be a % of current gold, so late
   treasures are a real investment instead of button-mash filler. *(Code:
   currently flat in `try_explore`.)*
7. **Never let Explore dead-end.** After a duplicate/complete roll, pay out
   gold, a **temporary "Hoard Rush" buff** (×N income for 30s), or a
   **treasure-dust** currency spent on rerolls/luck. The button stays alive and
   the action stays repeatable for the whole game. *(Code + optional new
   currency.)*
8. **Extend the treasure long tail.** Add higher-impact, rarer treasures — some
   gated behind prestige tiers — so collection is an ongoing chase, not a
   5-minute sweep. *(Mostly JSON; gating needs a small unlock hook.)*

### Tier 3 — The engagement hooks that create sessions (new systems)

9. **Golden Hoard / active-play burst.** A "golden hoard" glint occasionally
   appears; clicking it grants a **Dragon's Frenzy** (×7 click, or a lump of
   gold, or a lucky-expedition window) for a few seconds. This is the single
   most proven retention driver in the clicker genre (Cookie Clicker's golden
   cookie) and it makes *being present* matter without punishing idlers.
10. **Prestige unlocks, not just multipliers.** Gate qualitative content behind
    `prestige_count`: a new minion tier at prestige 2, a second expedition slot
    at prestige 3, a new upgrade line at prestige 4. Now players *want* to
    prestige for a reason numbers alone can't give.
11. **Soft-caps + wall-breakers.** Introduce gentle diminishing returns that a
    specific prestige/treasure unlock removes — turning "buy everything" into
    "which wall do I break next?" and creating real choices under scarcity.
12. **Later, if retention needs it:** a second meta-layer (ascension) and/or
    optional **challenge runs** (play under a restriction for a permanent
    reward) for the long-tail audience. Keep these behind "do we still have
    engaged players at week 3?" — don't build them speculatively.

---

## 4. Recommended sequencing

1. **Tier 1 first, as one balance pass.** It's ~90% JSON plus one multiplicative
   channel, and it's the difference between "a correct machine" and "a game."
   Ship it, then *actually play two prestige cycles* and confirm the second is
   visibly faster.
2. **Tier 2** to make the mid-game (exploration + treasure chase) hold
   attention between prestiges.
3. **Tier 3** for the hooks that turn "I tried it" into "I left it running and
   came back." Golden Hoard (#9) and prestige-gated unlocks (#10) are the two
   highest-leverage items here.

Everything in Tier 1 respects the project's rules: balance stays in JSON,
formulas stay in `simulation/` with tests beside them, UI stays a pure view
layer. The one structural addition (a multiplicative bonus channel) is a clean,
additive change to `simulation.rs`/`economy.rs`, not a refactor.

---

## 5. The one-sentence version

The game isn't unfinished — it's **tuned so the wall grows faster than the
reward and the treasure loop ends in five minutes**; fix the prestige curve so
each run is visibly faster than the last (Tier 1), give exploration a scaling
cost and a payoff that never dead-ends (Tier 2), and add a golden-hoard active
hook plus prestige-gated *unlocks* (Tier 3), and the same machine becomes a
genuine idle game.
