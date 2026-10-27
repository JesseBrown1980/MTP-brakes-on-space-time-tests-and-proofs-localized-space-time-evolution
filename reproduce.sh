#!/usr/bin/env bash
# Rerun all three measurements and compare against the sealed receipts. json=0
# usage: bash reproduce.sh /path/to/enwik8 [work_dir]
set -euo pipefail
E=${1:?path to enwik8}
W=${2:-./_reproduce}
H=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$W"
[ "$(md5sum "$E" | cut -c1-32)" = a1fa5ffddb56f4953e226637dabbb36a ] || { echo "REPRO|corpus_md5_mismatch|json=0"; exit 2; }

cd "$H"
python3 spec_brake.py build "$E" "$W/spec_cache" >/dev/null
for v in order2 oracle null greedy; do python3 spec_brake.py run "$E" "$W/spec_cache" "$v" "$W/$v.hbp" >/dev/null; done
python3 spec_brake.py combine "$W/order2.hbp" "$W/oracle.hbp" "$W/null.hbp" "$W/greedy.hbp" > "$W/RESULT.hbp"

python3 timewarp.py build "$E" "$W/tw_cache" >/dev/null
python3 timewarp.py run "$E" "$W/tw_cache" "$W/TIMEWARP.hbp" >/dev/null
python3 timewarp.py combine "$W/TIMEWARP.hbp" > "$W/TIMEWARP-RESULT.hbp"
TW_SEG=16384 python3 timewarp.py run "$E" "$W/tw_cache" "$W/TIMEWARP-16K.hbp" >/dev/null
TW_SEG=16384 python3 timewarp.py combine "$W/TIMEWARP-16K.hbp" > "$W/TIMEWARP-16K-RESULT.hbp"
python3 compare_16k.py "$W/TIMEWARP-RESULT.hbp" "$W/TIMEWARP-16K-RESULT.hbp" > "$W/COMPARE-16-vs-16K.hbp"

# Wall-time (ns) fields differ from machine to machine; everything else must match exactly.
fail=0
for f in RESULT TIMEWARP TIMEWARP-RESULT TIMEWARP-16K TIMEWARP-16K-RESULT COMPARE-16-vs-16K order2 oracle null greedy; do
  if diff <(sed -E 's/\|ns=[0-9]+//' "receipts/$f.hbp") <(sed -E 's/\|ns=[0-9]+//' "$W/$f.hbp") >/dev/null; then
    echo "REPRO|$f|MATCH|json=0"
  else
    echo "REPRO|$f|MISMATCH|json=0"; fail=1
  fi
done
[ $fail = 0 ] && echo "REPRO_VERDICT|PASS|json=0" || { echo "REPRO_VERDICT|FAIL|json=0"; exit 1; }
