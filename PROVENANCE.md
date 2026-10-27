# Provenance

The measurements were first recorded in
`JesseBrown1980/Neural-network-inside-a-neural-network-using-trits-to-resolve-Finiteness`,
under `measurements/speculative-brake/`. That folder's history was carried here with
`git subtree split`. Authors, dates and order are unchanged, and only the commit hashes
differ.

| here | original | date (−03:00) | what |
|---|---|---|---|
| `0d5146d` | `7ba9485` | 2026-10-27 05:59 | brake: pre-registration + `spec_brake.py` frozen |
| `fe7b81b` | `e77192b` | 2026-10-27 06:01 | brake: results and receipts |
| `fd47e29` | `54bc978` | 2026-10-27 06:15 | several drafters: pre-registration + `timewarp.py` frozen |
| `dfa38fd` | `aa81144` | 2026-10-27 06:16 | several drafters: results and receipts |
| `6ee9894` | `18c2234` | 2026-10-27 06:24 | 16K: addendum + `timewarp.py` (`TW_SEG`) + `compare_16k.py` frozen |
| `8d0b396` | `8cbca43` | 2026-10-27 06:25 | 16K: results and receipts |

**Code hashes recorded at the freezes:**

- `spec_brake.py`: `b6abf8005488e3822dd7b6b3d6d46ffef7d70564d7528f2e4905c264c2b081a5`
  (commit `0d5146d`).
- `timewarp.py`: one hash in commit `fd47e29` and one in `6ee9894`, the latter after the
  `TW_SEG` edit. With that edit, the 16-byte held-out receipt was re-run and reproduced
  byte-identical before the 16K freeze.

**Smoke tests:** each protocol was smoke-tested on training-region positions only (the
`SPEC_SMOKE` and `TW_SMOKE` settings), never on the held-out anchors. Each
pre-registration records what its smoke test changed before the freeze.
