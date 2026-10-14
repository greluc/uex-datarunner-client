#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Lucas Greuloch
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

if [ "$#" -lt 4 ]; then
  echo "usage: measure-linux.sh <probe> <strace-summary> <absolute ORT dir> <output dir> [hold seconds] [linger seconds]" >&2
  exit 2
fi

probe=$1
summary=$2
ort_dir=$3
out=$4
hold=${5:-10}
linger=${6:-120}

command -v strace > /dev/null || { echo "strace is not installed" >&2; exit 2; }
mkdir -p "$out"
report="$out/summary.txt"
: > "$report"

for variant in env env-only api none; do
  home="$out/home-$variant"
  mkdir -p "$home"
  for run in 1 2; do
    log="$out/$variant-$run"
    status=0
    env -i PATH="$PATH" HOME="$home" XDG_CACHE_HOME="$home/.cache" \
      strace -f -tt -s 256 -e trace=connect,sendto,sendmsg,sendmmsg -o "$log.strace" \
      "$probe" --variant "$variant" --ort-dir "$ort_dir" --hold "$hold" --linger "$linger" \
      > "$log.stdout" 2> "$log.stderr" || status=$?
    {
      echo "== variant $variant, run $run (same HOME as run 1), exit status $status"
      cat "$log.stdout"
      if [ -s "$log.stderr" ]; then
        echo "stderr:"
        cat "$log.stderr"
      fi
      "$summary" "$log.strace" || echo "strace-summary failed on $log.strace"
      echo "strace lines naming microsoft: $(grep -c -F microsoft "$log.strace" || true)"
      echo "files under the isolated HOME:"
      find "$home" -type f -printf '  %P (%s bytes)\n' | sort
      echo
    } >> "$report"
  done
done

cat "$report"
