#!/bin/sh
set -eu

binary=${1:-./target/release/femfetch}
runs=${2:-100}

if [ ! -x "$binary" ]; then
    printf 'not executable: %s\n' "$binary" >&2
    exit 1
fi

case $runs in
    ''|*[!0-9]*|0) printf 'runs must be a positive integer\n' >&2; exit 1 ;;
esac

printf 'binary: %s bytes\n' "$(wc -c < "$binary" | tr -d ' ')"

measure() {
    name=$1
    shift
    i=0
    while [ "$i" -lt 10 ]; do
        "$binary" --no-config "$@" >/dev/null 2>/dev/null
        i=$((i + 1))
    done

    python3 - "$binary" "$runs" "$name" "$@" <<'PY'
import statistics
import subprocess
import sys
import time

binary, runs, name, *args = sys.argv[1:]
runs = int(runs)
values = []
for _ in range(runs):
    started = time.perf_counter_ns()
    subprocess.run([binary, "--no-config", *args], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
    values.append((time.perf_counter_ns() - started) / 1_000_000)
values.sort()
p95 = values[max(0, int(len(values) * 0.95 + 0.999999) - 1)]
print(f"{name:16} median={statistics.median(values):.3f} ms p95={p95:.3f} ms mean={statistics.mean(values):.3f} ms n={runs}")
PY
}

measure default
measure plain --plain --no-color
measure json --json
measure representative --plain --no-color --modules os,cpu,memory,disk

if [ -x /usr/bin/time ]; then
    rss=$(/usr/bin/time -f '%M' "$binary" --no-config --plain --no-color >/dev/null 2>&1 || true)
    [ -n "$rss" ] && printf 'plain peak RSS: %s KiB\n' "$rss"
fi
