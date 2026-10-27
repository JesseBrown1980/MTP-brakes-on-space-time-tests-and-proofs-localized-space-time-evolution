"""Acceptance gate for the drafter series (PREREGISTRATION-SCALE.md), written before its
results exist. The combined verdict is accepted only if the raw receipts hold exactly the
intended grid. json=0

  D in {27, 81, 243, 729, 2187, 6561, 19683} x L in {8, 64} x 4 modes = 56 rows,
  each exactly once, with M = 3D, anchors = max(1, min(83, 6723 // M)),
  segments = M * anchors, and exact_fail = 0.

usage: python check_scale_grid.py SC-*.hbp
"""
import sys

DS = [27, 81, 243, 729, 2187, 6561, 19683]
LS = [8, 64]
MODES = ["optimistic", "optimistic_v", "conservative", "oracle"]


def main(paths):
    seen, problems = {}, []
    for p in paths:
        for line in open(p):
            parts = line.strip().split("|")
            if parts[0] != "SC":
                continue
            f = dict(x.split("=", 1) for x in parts[1:] if "=" in x)
            key = (int(f["D"]), int(f["L"]), f["mode"])
            if key in seen:
                problems.append("duplicate_%d_%d_%s" % key)
            seen[key] = f
    for D in DS:
        M = 3 * D
        n = max(1, min(83, 6723 // M))
        for L in LS:
            for m in MODES:
                f = seen.get((D, L, m))
                if f is None:
                    problems.append("missing_%d_%d_%s" % (D, L, m))
                    continue
                if (int(f["M"]), int(f["anchors"]), int(f["segments"]), int(f["exact_fail"])) != (M, n, M * n, 0):
                    problems.append("bad_row_%d_%d_%s" % (D, L, m))
    extra = sorted(k for k in seen if k[0] not in DS or k[1] not in LS or k[2] not in MODES)
    problems += ["unexpected_%d_%d_%s" % k for k in extra]
    print("SCALE_GRID|rows=%d|expected=%d|problems=%d|json=0" % (len(seen), len(DS) * len(LS) * len(MODES), len(problems)))
    for pr in problems[:20]:
        print("SCALE_GRID_PROBLEM|%s|json=0" % pr)
    print("SCALE_GRID_VERDICT|%s|json=0" % ("COMPLETE" if not problems else "INCOMPLETE"))
    raise SystemExit(0 if not problems else 2)


if __name__ == "__main__":
    main(sys.argv[1:])
