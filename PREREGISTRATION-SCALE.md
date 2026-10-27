# Drafters up the ×3 series — pre-registration (frozen before the held-out run)

**Request (Jesse Daniel Brown).** Run the drafters out to 81, 243, the 700s, and keep
going up to the 16,000s.

**Series:** `D = 27, 81, 243, 729, 2187, 6561, 19683`. 19,683 = 3⁹ is the member of the
series in the 16,000s.

## What carries over unchanged from `PREREGISTRATION-TIMEWARP.md`

16-byte segments, the speeds `1 + SHA-256("TW-SPEED|i")[0] mod 4` ticks per byte, latency
`L = 8, 64`, the four modes (`optimistic`, `optimistic_v`, `conservative`, `oracle`), the
cascade definitions (absorbed, in time, merged, end), the base set of 83 hash-selected
held-out anchors (`TW-V1`, mod 65536) and gates C1–C4.

## What scales, fixed now

- **Timeline:** `M = 3·D` segments, so each drafter owns 3.
- **Agents:** quality is held fixed. Every drafter `i` is verified by agent `i mod 27`,
  one of the 27 order-5 verifiers (each with its own order-2 draft) built from the
  27 disjoint 740,740-byte training slices of the `D = 27` run. Neighbouring drafters, the
  ones that hand corrections to each other, are always verified by different agents.
- **Anchors:** the first `n_D = max(1, min(83, 6723 // M))` of the 83 base anchors in
  position order. That gives `n = 83, 27, 9, 3, 1, 1, 1` for the series, keeping the work
  per configuration near 6,723 segments, except at the top where one anchor carries 6,561,
  19,683 or 59,049 segments.
- **Jobs:** each `(D, L, mode)` runs as its own process (`run_scale.sh`). Results do not
  depend on the split.

## Regression (before the freeze)

`D = 27` under these rules is exactly the frozen run: `M = 81`, 83 anchors and the
`D = 27` slices. Its counts, ends, makespans and reach histograms reproduce
`receipts/TIMEWARP.hbp` exactly for all 8 `(L, mode)` rows.

## Measured per `(D, L, mode)`

Speed-up against waiting, the oracle ceiling, waste, mispredictions, cascades, how
cascades end, median and maximum reach, and **far spread**: the fraction of cascade
endings whose reach is at least 27 hops. That is a fixed distance, comparable across
timelines of every length.

## Verdicts, fixed now

If any gate fails, the verdict is `INCONCLUSIVE`. Otherwise, for each optimistic mode and
`L`, compare far spread at `D = 27` against `D = 19683`:

| verdict | far spread at 19,683 compared with 27 |
|---|---|
| `FAR_SPREAD_GROWS` | higher |
| `FAR_SPREAD_SHRINKS` | lower |
| `FAR_SPREAD_FLAT` | equal |

Every intermediate `D` is reported in full.

## Smoke test (before the freeze)

`SC_SMOKE=1` (training region, 2 anchors, `D = 27, 81, 243`) passed every gate. Timing
runs at `D = 729` and `2187` on the training region showed cost growing about 8× per ×3
step. That fixed the anchor rule above with a floor of 1 instead of 3.
