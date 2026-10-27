#!/usr/bin/env bash
# One process per (D, L, mode) job, two at a time, longest first; finished jobs are skipped.
# usage: bash run_scale.sh ENWIK8 TW_CACHE OUTDIR
set -uo pipefail
E=$1; C=$2; O=$3; H=$(cd "$(dirname "$0")" && pwd); mkdir -p "$O"
jobs=()
for D in 19683 6561 2187 729 243 81 27; do
  for L in 64 8; do for m in optimistic optimistic_v conservative oracle; do jobs+=("$D $L $m"); done; done
done
printf '%s\n' "${jobs[@]}" | xargs -P 2 -L 1 bash -c '
  D=$0; L=$1; m=$2; f="'"$O"'/SC-$D-$L-$m.hbp"
  [ -s "$f" ] && exit 0
  cd "'"$H"'" && SC_LATS=$L SC_MODES=$m python3 scale_drafters.py run "'"$E"'" "'"$C"'" "$f.tmp" $D >/dev/null && mv "$f.tmp" "$f" && echo "DONE $D $L $m"'
