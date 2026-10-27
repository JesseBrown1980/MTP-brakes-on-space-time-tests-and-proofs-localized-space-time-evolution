# MTP brakes on space-time — tests and proofs

**Jesse Daniel Brown (OP-JESSE).** The picture: the universe processed by agents. Small
multi-token-prediction (MTP) drafters run ahead; larger agents verify; and when one goes
too fast or makes a change too fast, an observer further along says *wait*, putting the
brakes on. This repository holds the measurements of that mechanism, each one
pre-registered, frozen in git before it ran, and sealed with receipts.

Every run uses `enwik8` (MD5 `a1fa5ffddb56f4953e226637dabbb36a`) and hash-selected
measurement points. A held-out position is measured only when its SHA-256 hash is 0 mod a
fixed modulus, so the sample is deterministic, unbiased and the same for anyone who
reruns it. All arithmetic is exact integers, and every receipt line ends in `json=0`.

## The four measurements

### 1. The brake — how far ahead should a drafter run?

[PREREGISTRATION.md](PREREGISTRATION.md) → [RESULTS.md](RESULTS.md)

- **Setup:** an order-2 MTP drafter runs `k` bytes ahead, and one pass of an order-5
  verifier commits the longest agreeing prefix and rolls back the rest.
- **Sample:** 635 hash-selected points, `k = 1 … 27`, verifier cost `r = 1, 4, 16, 64`.

**Braking points measured at every cost:** `k* = 1, 1, 3, 4`, with speed-ups of up to
1.78×. Beyond `k*`, running further ahead loses.

Acceptance of each draft byte *rises* with depth (0.43 → about 0.60). The brake comes
from compounding: the chance of surviving to depth `i` is the product of every earlier
acceptance.

### 2. Several drafters with their own clocks — how far do late corrections spread?

[PREREGISTRATION-TIMEWARP.md](PREREGISTRATION-TIMEWARP.md) →
[RESULTS-TIMEWARP.md](RESULTS-TIMEWARP.md)

- **Setup:** 3, 9 or 27 drafters, each on its own clock and each verified by a different
  agent trained on its own slice of data, share one timeline. They run optimistically,
  roll back on late messages, and re-send only when their output changes.
- **Sample:** 83 hash-selected points.

**Results:**
- Rollback always restored the exact sequential answer.
- With 16-byte pieces, only 3–13% of corrections heal on their own. Most end by running
  into another drafter's correction.
- The fraction reaching the end of the timeline grows from 2.4% (3 drafters, fast
  messages) to 46% (27 drafters, slow messages).

### 3. Giant pieces — the 16K test

[PREREGISTRATION-TIMEWARP-16K.md](PREREGISTRATION-TIMEWARP-16K.md) →
[RESULTS-TIMEWARP-16K.md](RESULTS-TIMEWARP-16K.md)

- **Prediction (Jesse, frozen before the run):** with giant pieces, the spreading still
  happens but is slowed locally by the other MTP agent localizations.
- **Setup:** the same 83 points, with 16,384-byte pieces.

**SLOWED in 9 of 12 configurations:**
- Fewer corrections reach the end in all 12.
- Self-healing about doubles.
- Locally stopped corrections rise in 9 of 12.
- The strongest drops (46% → 11%) come where messages were slow, partly because latency
  becomes negligible next to 16K of work. The like-for-like fast-message rows show a
  smaller slowing.

### 4. The drafter series, ×3 from 27 to 19,683 drafters

[PREREGISTRATION-SCALE.md](PREREGISTRATION-SCALE.md) → [RESULTS-SCALE.md](RESULTS-SCALE.md)

- 56 receipts, the full grid (7 sizes × 2 latencies × 4 modes); every gate passes. The Rust
  port `rust/twsim` reproduces all 58 receipts byte for byte.
- A correction's median reach stays at 3–4 hops (fast messages) and 7–9 hops (slow) across a
  729-fold range in the number of drafters: the spread is local.
- The share reaching the end of the timeline falls (slow messages 46% → 10%) and settles.
- Optimism gains at most 4.7% over waiting and settles at 1.000×, while the oracle ceiling
  grows to 23,975×: the headroom is there, and only better predictions can take it.

## Order of events (the proof that nothing was fitted after the fact)

| commit | what |
|---|---|
| `0d5146d` | brake: pre-registration and code frozen |
| `fe7b81b` | brake: results |
| `fd47e29` | several drafters: pre-registration and code frozen |
| `dfa38fd` | several drafters: results |
| `6ee9894` | 16K: addendum, code and comparator frozen |
| `8d0b396` | 16K: results |

These six commits are the history carried over from
`Neural-network-inside-a-neural-network-using-trits-to-resolve-Finiteness`
(`measurements/speculative-brake/`, commits `7ba9485`, `e77192b`, `54bc978`, `aa81144`,
`18c2234`, `8cbca43`), where the runs were first recorded. See
[PROVENANCE.md](PROVENANCE.md).

## Reproduce

```bash
pip install numpy
bash reproduce.sh /path/to/enwik8      # checks the MD5, builds the tables, reruns measurements 1–3, compares receipts
```

## Files

```
spec_brake.py                  the brake: draft, verify, commit, roll back (speculative sampling, exact integers)
timewarp.py                    several drafters on their own clocks: discrete-event, rollback, lazy cancellation
compare_16k.py                 the frozen 16-byte vs 16K comparator
PREREGISTRATION*.md            each protocol, frozen before its run
RESULTS*.md                    each result, with its gates and verdicts
receipts/                      every raw and combined receipt, with SHA256SUMS
reproduce.sh                   one command, measurements 1–3 (4 runs through run_scale.sh)
```

## Lineage

- **The agents:** the agents the drafters stand in for are in the
  ASOLARIA-UNIVERSE-SIMULATOR (`players/rime_agents.py`: Fischer, MTP, HRM and MCP, Laws
  23–26). Its MTP agent is the frozen order-2 table used here as the drafter.
- **The brakes:** that the brakes are the colour breaks, and that predictors belong to the
  colour they were born in, is from the Book of Laws, voice-07, turns 676–680.
- **Prior work:** speculative sampling follows Leviathan, Kalman and Matias (2023).
  Optimistic execution with rollback follows Jefferson's Time Warp (1985).

**Jesse Daniel Brown (OP-JESSE)** — the system, the laws, the architecture.
