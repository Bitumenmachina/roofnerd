#!/usr/bin/env bash
# The whole check of the command line's side, as the lead runs it before work is accepted.
#   tools/check.sh            the six crates' tests, every command-line demonstration, the gates,
#                             the licences, the port map (when BIMCRAFT names BIMCraft's source),
#                             and the sample whole house's checksum
#   tools/check.sh --window   the window's own tests as well (a long build the first time)
# Run from plumbline/. Each test program is built, run and deleted before the next: the model's
# test programs together are over 6 GB. Every cargo command waits for a timed window run to end
# (tools/quiet.sh). It writes target/check/summary.txt and one log for each demonstration, and
# ends with status 0 only if nothing failed.
set -u
cd "$(dirname "$0")/.." || exit 1
HOUSE=880d9fd3c364b5f7   # sha256 of `plumbline sixteen`, first 16: it moves only with a decision written first
OUT=target/check; rm -rf "$OUT"; mkdir -p "$OUT"
T="${CARGO_TARGET_DIR:-target}"
crates="plumbline-world plumbline plumbline-sheets plumbline-ifc plumbline-roofing plumbline-plans"
[ "${1:-}" = --window ] && crates="$crates plumbline-app"
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
clean() { find "$T/debug/deps" -maxdepth 1 -type f -executable ! -name '*.so' ! -name 'plumbline-*' -size +20M -delete 2>/dev/null; }
bad=0
{
  echo "on $(git rev-parse --short HEAD)$(git diff --quiet HEAD -- . || echo ' with changes not committed'), $(date -u +%F' '%H:%M) UTC"
  for c in $crates; do
    d=$(cargo metadata --no-deps --format-version 1 2>/dev/null | python3 -c "import sys,json,os; print([os.path.dirname(p['manifest_path']) for p in json.load(sys.stdin)['packages'] if p['name']=='$c'][0])")
    { tools/quiet.sh cargo test -p "$c" -j"${JOBS:-2}" --lib --bins 2>&1; clean
      for t in "$d"/tests/*.rs; do [ -e "$t" ] || continue
        tools/quiet.sh cargo test -p "$c" -j"${JOBS:-2}" --test "$(basename "$t" .rs)" 2>&1; clean
      done
    } | grep -E '^test result|FAILED|panicked|^error' | grep -v 'no library targets found' \
      | awk -v c="$c" '/test result/ {p+=$4; f+=$6; i+=$8; next} {print c": "$0; e=1} END {print c, p+0, "passed", f+0, "failed", i+0, "ignored"; exit (f>0 || e || p==0)}' || bad=1
  done
  tools/quiet.sh cargo build -p plumbline -j"${JOBS:-2}" >/dev/null 2>&1 || { echo "the command line does not build"; bad=1; }
  export PLUMBLINE_BIN="$PWD/$T/debug"; case $T in /*) PLUMBLINE_BIN="$T/debug";; esac
  pass=0; fail=""
  for f in demos/*.sh; do
    n=$(basename "$f" .sh)
    case $n in window|field-job) continue;; esac
    grep -q 'plumbline-app\|open_window\|xvfb\|xdotool' "$f" && continue
    if timeout 900 bash "$f" > "$OUT/$n.log" 2>&1 && ! grep -q 'FAIL' "$OUT/$n.log"; then pass=$((pass+1)); else fail="$fail $n"; fi
  done
  echo "command-line demonstrations: $pass passed; failed:${fail:- none}"; [ -z "$fail" ] || bad=1
  for g in gate-language.sh gate-egress.sh "gate-clientdata.sh --all"; do ( cd .. && bash tools/$g >/dev/null 2>&1 ) && echo "$g: pass" || { echo "$g: FAIL"; bad=1; }; done
  bash tools/licenses.sh >/dev/null 2>&1 && echo "licences: pass" || { echo "licences: FAIL"; bad=1; }
  if [ -n "${BIMCRAFT:-}" ]; then bash tools/portmap.sh >/dev/null 2>&1 && echo "port map: pass" || { echo "port map: FAIL"; bad=1; }; else echo "port map: not run (BIMCRAFT does not name BIMCraft's source)"; fi
  "$PLUMBLINE_BIN/plumbline" sixteen "$OUT/sixteen.json" >/dev/null 2>&1
  h=$(sha256sum "$OUT/sixteen.json" 2>/dev/null | cut -c1-16)
  [ "$h" = "$HOUSE" ] && echo "the sample whole house: $h, as it was" || { echo "the sample whole house: $h, NOT $HOUSE"; bad=1; }
  [ $bad = 0 ] && echo "check: PASS" || echo "check: FAIL"
} 2>&1 | tee "$OUT/summary.txt"
grep -q '^check: PASS' "$OUT/summary.txt"
