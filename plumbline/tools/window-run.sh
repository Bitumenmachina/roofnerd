#!/usr/bin/env bash
# Window demonstrations, one at a time, on a frozen copy of the two programs, with nothing beside them.
#   tools/window-run.sh all                 every window demonstration
#   tools/window-run.sh navw tradesw        the ones named
# Run from plumbline/. It builds nothing: build first (cargo build -p plumbline -p plumbline-app).
# It waits until no cargo or rustc is running and no command-line demonstration of tools/check.sh is,
# lets in a command that waited on the last window run, holds the lock tools/quiet.sh reads, runs each
# demonstration on a virtual display, runs a failure once more before it is believed, and writes
#   target/window-run/results.txt     PASS or FAIL, the name, the run, the times
#   target/window-run/<name>.<run>.log
# WINDOW_RUN_KEEP=1 adds to the results of an earlier call, for a long run done in parts.
# It ends with status 0 only if every demonstration passed (on its first or its second run).
set -u
cd "$(dirname "$0")/.." || exit 1
LOCK="${BUILDOS_LOCK:-/tmp/buildos-window.lock}"
OUT=target/window-run; FROZEN=target/frozen
is_window() { grep -q 'plumbline-app\|open_window\|xvfb\|xdotool' "$1"; }
names=()
if [ "${1:-}" = all ]; then
  for f in demos/*.sh; do n=$(basename "$f" .sh); case $n in window|field-job) continue;; esac; is_window "$f" && names+=("$n"); done
else names=("$@"); fi
[ ${#names[@]} -gt 0 ] || { sed -n 2,11p "$0"; exit 2; }
for n in "${names[@]}"; do [ -f "demos/$n.sh" ] || { echo "no demonstration demos/$n.sh"; exit 2; }; done
SRC="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-target}/debug}"
[ -x "$SRC/plumbline" ] && [ -x "$SRC/plumbline-app" ] || { echo "no built programs in $SRC: build first"; exit 2; }
mkdir -p "$OUT" "$FROZEN"; cp "$SRC/plumbline" "$SRC/plumbline-app" "$FROZEN/" || exit 1
# WINDOW_RUN_KEEP=1 adds to the results of an earlier call (a long run is done in parts) instead of starting them again.
[ -n "${WINDOW_RUN_KEEP:-}" ] || : > "$OUT/results.txt"; echo "on $(git rev-parse --short HEAD)$(git diff --quiet HEAD -- . || echo ' with changes not committed'), programs frozen $(date -u +%F' '%H:%M) UTC" >> "$OUT/results.txt"
# A command that waited on the last window run goes first (tools/quiet.sh says it is waiting); a command-line
# demonstration of tools/check.sh is finished before a window demonstration starts ($LOCK.busy, believed for 20 minutes).
waiting() { [ -n "$(find "$LOCK.waiting" -newermt '-40 seconds' 2>/dev/null)" ]; }
busy() { [ -n "$(find "$LOCK.busy" -newermt '-20 minutes' 2>/dev/null)" ]; }
while [ -e "$LOCK" ] || waiting; do sleep 15; done
trap 'rm -f "$LOCK"' EXIT
touch "$LOCK"
bad=0
for n in "${names[@]}"; do
  r=FAIL
  for run in 1 2; do
    while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null || busy; do sleep 10; done
    s=$(date -u +%H:%M)
    if PLUMBLINE_BIN="$PWD/$FROZEN" timeout "${WINDOW_RUN_LIMIT:-5400}" xvfb-run -a -s "-screen 0 2500x980x24" bash "demos/$n.sh" > "$OUT/$n.$run.log" 2>&1; then r=PASS; else r=FAIL; fi
    echo "$r $n (run $run) $s to $(date -u +%H:%M)" | tee -a "$OUT/results.txt"
    [ $r = PASS ] && break
  done
  [ $r = PASS ] || bad=1
done
echo "this call: ${#names[@]} named; in the results so far, $(grep -c '^PASS' "$OUT/results.txt") passed and $(grep '^FAIL' "$OUT/results.txt" | grep -c '(run 2)') failed twice" | tee -a "$OUT/results.txt"
exit $bad
