# Dragon's Den

You're a dragon guarding a hoard that never stops growing. Click for gold, put
kobolds to work while you're away, send expeditions into old ruins for treasure —
then, when the pile is big enough, **burn it all down** and start again richer,
wiser, and permanently stronger. An idle/incremental game about honest,
well-tuned number-go-up progression.

## How you play

**Collect.** Click the hoard on the left rail to rake in gold. Each click throws
up a floating "+N" — early on, clicking is your whole income.

**Hire kobolds.** Spend gold on minions for passive gold-per-second that keeps
earning whether you're watching or not. Higher minion tiers unlock as your den
grows, each a stronger (and pricier) source of idle income.

**Explore ruins.** Launch an expedition for a rarity-weighted shot at treasure.
Every treasure you discover grants a small **permanent** passive bonus that
stacks — commons are a nudge, legendaries are a real boost — and stays with you
even through prestige.

**Upgrade.** Spend gold in the Upgrade Shop across converging lines — claw
sharpness (click power), minion efficiency, treasure luck, and hoard-point gain.
Buy in `x1 / x10 / Max` batches.

**Chase achievements.** Milestones unlock in the background as you cross them,
each handing back a one-time reward (gold, Hoard Points, or a codex reveal) —
never just a badge.

**Prestige.** Once the hoard clears the threshold, **Burn the Hoard**: convert
it into permanent **Hoard Points**, reset your gold, kobolds, and shop upgrades,
and spend those points on the five-branch **Hoard Legacies** tree. Those
multipliers persist across every future run, so each cycle is faster than the
last. Treasures, achievements, and your dragon codex all carry over.

**Come back later.** Offline income accrues honestly from real elapsed time at
the same rate you'd earn it live — leave the den running or close it and collect
what your kobolds hauled in while you were gone.

## The screen

A single ornate frame, no page-flipping for the essentials:

- **Header** — Gold, Minions, and Hoard Points, each with its current per-second rate.
- **Left rail** — the hoard, the big circular **CLICK** target, live income, and run time.
- **Bottom strip** — minion-tier hire cards, the expedition launcher, and your most recent treasures.
- **Center tabs** — Hoard (chronicle + prestige progress), Minions, Upgrades,
  Treasures, Achievements, Prestige, and the Dragon Codex. Only the center swaps
  as you switch tabs.

## Dragon Codex

A small gallery of themed dragons — one per element (fire, ice, earth, air,
shadow, light, poison, lightning) — revealed as you hit prestige tiers and
collection milestones. Each unlock is flavor plus a small permanent bonus. It's
color and identity for your growing legend, not a second game bolted on: no
breeding, no combat, no map.

## Design

`gdd.md` is the design document and the authority on scope. This is a Rust +
Macroquad port of the WebHatchery `game_apps/dragons_den` game, rebuilt so the
mechanics its original only *implied* — a prestige that actually pays out,
upgrades that actually feed the formulas — are real. `IMPLEMENTATION_PLAN.md`
tracks what's built and what's next.
