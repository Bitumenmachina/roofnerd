#!/usr/bin/env bash
# ── The vocabulary gate ────────────────────────────────────────────────────
# Two rules from the README, made into a test rather than left as an intention:
#
#   1. Governance words stay out of the product. This program does not tell an
#      estimator that something is "authorized" or "deferred"; it is a takeoff
#      program, and that language never cohered here.
#   2. The three authority markers from the previous build — OBS, INF, STD — do
#      not come across. They were a way of scoring how sure the program was of
#      its own numbers, and they belong to a design this one is not repeating.
#
# Two passes on purpose. The governance words are matched without regard to case;
# the three markers are matched WITH case and as whole words, because a
# case-insensitive \bSTD\b matches `std::` in every Rust file in the project and
# the gate would fail on the day it was written.
set -uo pipefail
cd "$(dirname "$0")/.."

WHERE=(packages/engine/src packages/engine/bin packages/app/src src-tauri/src)
SKIP=(--exclude-dir=node_modules --exclude-dir=target --exclude-dir=dist)
fail=0

echo "vocabulary gate"

if hits=$(grep -rniE "${SKIP[@]}" -e 'authorized|deferred|authority' -e '\bverify\b' "${WHERE[@]}"); then
  echo "  governance language in the product:"
  echo "$hits" | sed 's/^/    /'
  fail=1
else
  echo "  no governance language"
fi

if hits=$(grep -rnwE "${SKIP[@]}" -e 'OBS|INF|STD' "${WHERE[@]}"); then
  echo "  authority markers carried across:"
  echo "$hits" | sed 's/^/    /'
  fail=1
else
  echo "  no authority markers"
fi

# 3. The programmer's plural — "line(s)", "trace(s)" — never reaches a screen. plural() in the
#    engine's words.ts is what this is for; the vocabulary probe bans the words but only sees the
#    states it renders, and "1 line(s) not counted" shipped past it.
#
#    The engine and its command line are in here as well, and they were not: the engine printed
#    "3 line(s) not in the total" under the recap and "takes 2 number(s)" onto a formula line in
#    the sheet, which is the app's own screen. A gate that stops at one package is a gate that
#    reads half the program.
#
#    src-tauri/src is deliberately NOT in this pass: `Ok(s)` is a Rust idiom on every other line
#    of that file and this pattern cannot tell it from a plural. The shell prints no plurals — it
#    hands strings to the window — so there is nothing there for this rule to protect.
if hits=$(grep -rnE "${SKIP[@]}" -e '[a-z]\(s\)' packages/app/src packages/engine/src packages/engine/bin | grep -vE ':[0-9]+:[[:space:]]*(//|/\*|\*)'); then
  echo "  a programmer's plural in the product:"
  echo "$hits" | sed 's/^/    /'
  fail=1
else
  echo "  no programmer's plural"
fi

[ $fail -eq 0 ] && echo "PASS" || echo "FAIL"
exit $fail
