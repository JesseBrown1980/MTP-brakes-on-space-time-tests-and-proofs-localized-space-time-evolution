# Drafter series ×3 from 27 to 19,683: results

The protocol is [`PREREGISTRATION-SCALE.md`](PREREGISTRATION-SCALE.md), frozen at `920eba3`
before any run. The raw receipts are in [`receipts/scale/`](receipts/scale/) (56 files). The
combined result is [`receipts/SCALE-RESULT.hbp`](receipts/SCALE-RESULT.hbp), the grid gate is
[`receipts/SCALE-GRID-GATE.hbp`](receipts/SCALE-GRID-GATE.hbp), and hashes are in
[`receipts/SHA256SUMS-SCALE`](receipts/SHA256SUMS-SCALE). All values are integers (‰ = per mille).

## Acceptance

- **Grid:** `check_scale_grid.py` reports `rows=56 expected=56 problems=0`, `COMPLETE`. Every
  `(D, L, mode)` cell is present once, with the frozen `M = 3D`, anchors and segment totals,
  and `exact_fail = 0`.
- **Gates:** C1 exact, C2 conservative, C3 oracle and C4 coverage all PASS.
- **Independent re-run:** the Rust port `rust/twsim` reproduces all 56 scale receipts and both
  earlier timewarp receipts byte for byte (`rust/twsim/REGRESSION.hbp`: expected 58, matched
  58, mismatched 0, `COMPLETE`).

## Verdicts (frozen rule: far spread at D = 19,683 against D = 27)

| L | mode | reach ≥ 27 hops, ‰ | verdict |
|---|---|---|---|
| 8 | optimistic | 102 → 107 | `FAR_SPREAD_GROWS` |
| 8 | optimistic_v | 98 → 103 | `FAR_SPREAD_GROWS` |
| 64 | optimistic | 394 → 357 | `FAR_SPREAD_SHRINKS` |
| 64 | optimistic_v | 393 → 374 | `FAR_SPREAD_SHRINKS` |

The frozen rule compares only the two endpoints. Across the whole series the far-spread
fraction stays in a narrow band: 98–128 ‰ at L = 8 and 354–394 ‰ at L = 64. The verdicts
record small movements inside those bands.

## Full series, optimistic mode

| D | L | speedup ‰ | oracle ceiling ‰ | waste ‰ | absorbed ‰ | reach end ‰ | median reach | reach ≥ 27 ‰ |
|---|---|---|---|---|---|---|---|---|
| 27 | 8 | 948 | 20,240 | 17,049 | 53 | 137 | 3 | 102 |
| 81 | 8 | 988 | 59,120 | 34,192 | 101 | 52 | 3 | 119 |
| 243 | 8 | 1,001 | 173,360 | 59,807 | 121 | 39 | 3 | 128 |
| 729 | 8 | 998 | 520,400 | 143,251 | 116 | 37 | 3 | 122 |
| 2,187 | 8 | 999 | 1,575,680 | 379,544 | 107 | 33 | 3 | 113 |
| 6,561 | 8 | 1,000 | 4,726,400 | 1,119,238 | 104 | 32 | 3 | 110 |
| 19,683 | 8 | 999 | 14,155,520 | 3,292,318 | 104 | 32 | 3 | 107 |
| 27 | 64 | 1,009 | 33,312 | 26,497 | 132 | 463 | 8 | 394 |
| 81 | 64 | 1,019 | 99,125 | 57,831 | 312 | 182 | 8 | 376 |
| 243 | 64 | 1,014 | 294,687 | 99,805 | 373 | 116 | 7 | 366 |
| 729 | 64 | 1,002 | 884,750 | 227,431 | 367 | 106 | 8 | 366 |
| 2,187 | 64 | 1,001 | 2,666,000 | 621,957 | 337 | 99 | 8 | 354 |
| 6,561 | 64 | 1,000 | 7,997,937 | 1,809,912 | 338 | 100 | 8 | 356 |
| 19,683 | 64 | 1,000 | 23,975,750 | 5,350,396 | 340 | 100 | 8 | 357 |

The `optimistic_v` rows (in `SCALE-RESULT.hbp`) follow the same pattern.

## What the series shows

1. **A correction's reach is local and does not grow with the timeline.** Median reach is 3–4
   hops at L = 8 and 7–9 at L = 64 for every D from 27 to 19,683, a 729-fold range.
2. **The share of corrections that reach the end falls as the timeline grows** (L = 64:
   463 ‰ → 100 ‰; L = 8: 137 ‰ → 32 ‰) and settles from about D = 2,187 onward. Most
   cascades stop by merging or being absorbed, not by running out of timeline.
3. **Optimism gains almost nothing over waiting.** Speedup over the conservative schedule
   is 948–1,047 ‰ (at most 4.7 % faster, at D = 81) and settles at 1,000 ‰ (±1) from
   D = 2,187 onward. Mispredictions are 91.8–99.8 %.
4. **The headroom is there but unused.** The oracle ceiling grows about ×3 per ×3 step in D
   (20× → 14,155× at L = 8; 33× → 23,975× at L = 64), and so does wasted work (17× →
   3,292×; 26× → 5,350×). The drafters can only take the ceiling with better predictions,
   not with more optimism.
