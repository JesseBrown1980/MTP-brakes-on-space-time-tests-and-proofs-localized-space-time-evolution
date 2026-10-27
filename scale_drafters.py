"""Drafters up the x3 series: D = 27, 81, 243, 729, 2187, 6561, 19683.
Exact integers, integer ticks, json=0. See PREREGISTRATION-SCALE.md.

usage:
  python scale_drafters.py run     ENWIK8 TW_CACHE OUT [D ...]   (optional env SC_LATS=8,64 SC_MODES=...)
  python scale_drafters.py combine OUT [OUT ...]
"""
import hashlib
import os
import sys

import spec_brake as sb
import timewarp as tw

SERIES = [27, 81, 243, 729, 2187, 6561, 19683]
POOL = 27                       # verifier agents: the D = 27 slices, drafter i -> agent i mod 27
BASE_SEGMENTS = 83 * 81         # work budget per configuration, in segments
if os.environ.get("SC_SMOKE") == "1":            # smoke test only: training region, tiny
    sb.HELD_LO, sb.HELD_HI = 1_000_000, 1_400_000
    tw.ANCHOR_MOD = 131072
    SERIES = [27, 81, 243]


def anchors():
    out = []
    for p in range(sb.HELD_LO, sb.HELD_HI):
        h = hashlib.sha256(b"TW-V1|" + str(p).encode()).digest()
        if int.from_bytes(h[:8], "big") % tw.ANCHOR_MOD == 0:
            out.append(p)
    return out


def run(enwik, cache, outpath, series):
    if sb.md5(enwik) != sb.MD5:
        raise SystemExit("CORPUS|md5_mismatch|json=0")
    data = open(enwik, "rb").read()
    pool = [tw.Agent(os.path.join(cache, "D27", "s%d" % j)) for j in range(POOL)]
    base = anchors()
    lines = ["SCHDR|base_anchors=%d|pool=%d|SEG=%d|json=0" % (len(base), POOL, tw.SEG)]
    for D in series:
        tw.M = 3 * D
        n = max(1, min(len(base), BASE_SEGMENTS // tw.M))
        an = base[:n]
        agents = [pool[i % POOL] for i in range(D)]
        lats = [int(x) for x in os.environ["SC_LATS"].split(",")] if os.environ.get("SC_LATS") else tw.LATS
        modes = os.environ["SC_MODES"].split(",") if os.environ.get("SC_MODES") else tw.MODES
        for L in lats:
            for mode in modes:
                tot = {k: 0 for k in ("computations", "rollbacks", "mispredicts", "messages", "cascades",
                                      "absorbed", "in_time", "merged", "unabsorbed", "makespan",
                                      "cons_bound", "exact_fail", "done")}
                reach = {}
                for p in an:
                    ctx0 = data[p - sb.VER_N:p]
                    truth = tw.sequential(agents, D, ctx0)
                    final, st = tw.simulate(agents, D, L, ctx0, mode, truth)
                    tot["exact_fail"] += final != truth
                    for k in ("computations", "rollbacks", "mispredicts", "messages", "cascades",
                              "unabsorbed", "makespan"):
                        tot[k] += st[k]
                    for k in ("absorbed", "in_time", "merged"):
                        tot[k] += st["ends"][k]
                    tot["cons_bound"] += tw.conservative_bound(D, L)
                    for r in st["reach"]:
                        reach[r] = reach.get(r, 0) + 1
                    tot["done"] += 1
                lines.append(
                    "SC|D=%d|M=%d|L=%d|mode=%s|anchors=%d|exact_fail=%d|computations=%d|segments=%d|"
                    "rollbacks=%d|mispredicts=%d|messages=%d|cascades=%d|absorbed=%d|in_time=%d|merged=%d|"
                    "unabsorbed=%d|makespan=%d|conservative_bound=%d|reach_hist=%s|json=0"
                    % (D, tw.M, L, mode, tot["done"], tot["exact_fail"], tot["computations"], tw.M * len(an),
                       tot["rollbacks"], tot["mispredicts"], tot["messages"], tot["cascades"], tot["absorbed"],
                       tot["in_time"], tot["merged"], tot["unabsorbed"], tot["makespan"], tot["cons_bound"],
                       ",".join("%d:%d" % kv for kv in sorted(reach.items())) or "-"))
                print(lines[-1][:170], flush=True)
    with open(outpath, "w") as f:
        f.write("\n".join(lines) + "\n")


def combine(paths):
    rows = {}
    for path in paths:
        for line in open(path):
            parts = line.strip().split("|")
            if parts[0] == "SC":
                f = dict(x.split("=", 1) for x in parts[1:] if "=" in x)
                rows[(int(f["D"]), int(f["L"]), f["mode"])] = f
    Ds = sorted({k[0] for k in rows})
    g = {
        "C1_exact": all(int(f["exact_fail"]) == 0 for f in rows.values()),
        "C2_conservative": all(int(f["rollbacks"]) == 0 and int(f["makespan"]) == int(f["conservative_bound"])
                               for k, f in rows.items() if k[2] == "conservative"),
        "C3_oracle": all(int(f["rollbacks"]) == 0 for k, f in rows.items() if k[2] == "oracle"),
        "C4_coverage": all((D, L, m) in rows for D in Ds for L in tw.LATS for m in tw.MODES),
    }
    out = []
    for D in Ds:
        for L in tw.LATS:
            for om in ("optimistic", "optimistic_v"):
                o, c, orc = rows[(D, L, om)], rows[(D, L, "conservative")], rows[(D, L, "oracle")]
                h = {int(a): int(b) for a, b in (x.split(":") for x in o["reach_hist"].split(","))} \
                    if o["reach_hist"] != "-" else {}
                ne = sum(h.values())
                acc, med = 0, 0
                for r in sorted(h):
                    acc += h[r]
                    if 2 * acc >= ne:
                        med = r
                        break
                far = sum(v for r, v in h.items() if r >= 27)
                segs = int(o["segments"])
                out.append("SCRESULT|D=%d|L=%d|mode=%s|anchors=%s|segments=%d|speedup_milli=%d|"
                           "oracle_speedup_milli=%d|waste_milli=%d|mispredict_milli=%d|cascades=%s|"
                           "absorbed_milli=%d|merged_milli=%d|end_milli=%d|median_reach=%d|max_reach=%d|"
                           "reach_ge27_milli=%d|json=0"
                           % (D, L, om, o["anchors"], segs, int(c["makespan"]) * 1000 // int(o["makespan"]),
                              int(c["makespan"]) * 1000 // int(orc["makespan"]),
                              (int(o["computations"]) - segs) * 1000 // segs, int(o["mispredicts"]) * 1000 // segs,
                              o["cascades"], int(o["absorbed"]) * 1000 // max(1, ne),
                              int(o["merged"]) * 1000 // max(1, ne), int(o["unabsorbed"]) * 1000 // max(1, ne),
                              med, max(h) if h else 0, far * 1000 // max(1, ne)))
    for k, ok in g.items():
        out.append("GATE|%s|%s|json=0" % (k, "PASS" if ok else "FAIL"))
    allok = all(g.values())
    lo, hi = Ds[0], Ds[-1]
    for L in tw.LATS:
        for om in ("optimistic", "optimistic_v"):
            def far(D):
                f = rows[(D, L, om)]
                h = {int(a): int(b) for a, b in (x.split(":") for x in f["reach_hist"].split(","))} \
                    if f["reach_hist"] != "-" else {}
                return sum(v for r, v in h.items() if r >= 27), max(1, sum(h.values()))
            a, na = far(lo)
            b, nb = far(hi)
            v = "INCONCLUSIVE" if not allok else ("FAR_SPREAD_GROWS" if b * na > a * nb else
                                                  "FAR_SPREAD_SHRINKS" if b * na < a * nb else "FAR_SPREAD_FLAT")
            out.append("VERDICT|L=%d|mode=%s|D=%d->%d|reach_ge27_milli=%d->%d|%s|json=0"
                       % (L, om, lo, hi, a * 1000 // na, b * 1000 // nb, v))
    print("\n".join(out))


if __name__ == "__main__":
    if sys.argv[1] == "run":
        run(sys.argv[2], sys.argv[3], sys.argv[4], [int(x) for x in sys.argv[5:]] or SERIES)
    elif sys.argv[1] == "combine":
        combine(sys.argv[2:])
