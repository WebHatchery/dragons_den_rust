# Prestige balance evidence

The headless balance career now carries permanent Hoard Point purchases through
100 prestiges and uses the production economy, prestige, and capped-offline
formulas. Its greedy active player sustains five clicks per second, buys the
shortest-payback income option, and stops each run at the current threshold.

## Why the threshold needed a second band

With the original `3.5^prestige` curve extended unchanged, the permanent tree
was effectively exhausted shortly after P10. P5 took 0.24 minutes and P10 took
0.07 minutes, but P25 then took 26,754 minutes (18.6 days); P50 was not reachable
inside the simulator's 30-day per-cycle limit. The finite permanent multipliers
could not chase an indefinitely compounding opening-game wall.

The second band therefore begins after ten completed prestiges. It applies a
one-time ×1,000 cliff, then grows by ×1.04 per prestige. This preserves the fast,
rewarding opening, restores a visible wall at P11, and gives the long tail a
slope that remains playable after the tree approaches its cap.

## Tuned checkpoints

| Prestige reached | Active cycle | Threshold | Hoard Points | Tree levels | 12h offline / threshold |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 0.24 min | 1.501e8 | 2,694 | 42 | 27,761× |
| 10 | 0.07 min | 7.882e10 | 84,454 | 63 | 35,224× |
| 11 (band entry) | 11.39 min | 2.759e14 | 7,513,768 | 66 | 71.5× |
| 25 | 2.32 min | 4.777e14 | 10,162,826 | 67 | 351.7× |
| 50 | 5.61 min | 1.273e15 | 17,427,012 | 67 | 142.8× |
| 100 | 33.58 min | 9.050e15 | 51,243,617 | 68 | 23.5× |

Offline ratios above 1× do not grant multiple prestiges: burning the hoard
resets run gold. They mean a capped return can fund the next single burn, so
offline play remains useful even at P100 while active play controls how quickly
successive prestiges can be taken.

Purchase mixes also keep changing with scale. At P5 the simulated run buys 60
Kobolds, 39 Workers, 23 Scavengers, 10 Thieves, and 2 Drakes alongside its run
upgrades. At P100 it buys 147 Kobolds, 116 Workers, 100 Scavengers, 82 Thieves,
69 Drakes, and 60 Wyverns. The income unlocks are used promptly: Molten Veins,
Warband Drums, Cataclysm Claws, and Worldflame are first bought in cycles 2, 3,
4, and 6; the prestige-gated Drake and Wyvern are first bought in cycles 5 and
7. Thus the later shop and minion unlocks change the selected build rather than
merely appearing as sealed content.

Run `cargo test prestige_career_checkpoints -- --nocapture` to reproduce the
checkpoint table and full per-run purchase counts.
