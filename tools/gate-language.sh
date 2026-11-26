#!/usr/bin/env bash
# ── The language gate ──────────────────────────────────────────────────────
# What a person reads on a screen or in a reply of BuildOS, held by a test and
# not by an intention. It reads the program's own source (plumbline/crates/*/src).
#
#   1. Governance words stay out of the product: "authorized", "deferred",
#      "authority". ("verify" is not on the list here: "verify in the field" is
#      a builder's own phrase and the program's VIF list is named by it.)
#   2. The three authority markers of an older design, OBS, INF and STD, do not
#      come across. Matched WITH case and as whole words: a case-blind \bSTD\b
#      matches `std::` in every Rust file.
#   3. The programmer's plural, "line(s)", never reaches a person. Only what is
#      inside quotation marks is read: `Some(s)` is Rust on every other line.
#   4. The window says no file word (call 178): a job is named by its title; a
#      table brought in is counted in what it holds, never in rows; a file's
#      name is said without its ending.
# Comment lines are not read by any of the four.
#
# Until 2026-11-26 rules 1 to 3 read the earlier program (roofnerd), which stood
# at the root of this branch, and rule 4 alone read BuildOS. That program left
# the branch (plumbline/DECISIONS.md call 237) and the three rules now read
# BuildOS, which they had never read. Each was proved red on a line written for
# the purpose and green on the tree: `tools/gate-language.sh --self-test`.
set -uo pipefail
cd "$(dirname "$0")/.."
SRC=plumbline/crates
if [ "${1:-}" = "--self-test" ]; then
  T=$(mktemp -d); mkdir -p "$T/a/src" "$T/app/src"
  printf 'fn f() { say("not authorized"); }\n'            > "$T/a/src/one.rs"
  printf 'const K: &str = "INF";\n'                       > "$T/a/src/two.rs"
  printf 'fn g(n: u32) { println!("{n} line(s) left"); }\n' > "$T/a/src/three.rs"
  printf 'fn h() { said = format!("Wrote {name}"); }\n'   > "$T/app/src/four.rs"
  printf '// authorized, INF, line(s): a comment is not read\nfn ok(s: u32) -> Option<u32> { Some(s) }\n' > "$T/a/src/clean.rs"
  SRC="$T"; APP="$T/app/src"; SELF=1
else
  APP=plumbline/crates/app/src; SELF=0
fi
[ -d "$SRC" ] || { echo "language gate"; echo "  no $SRC here: nothing was read"; echo "FAIL"; exit 1; }
mapfile -t FILES < <(find "$SRC" -path '*/src/*' -name '*.rs' -not -path '*/target/*' | sort)
[ ${#FILES[@]} -gt 0 ] || { echo "language gate"; echo "  no source found under $SRC"; echo "FAIL"; exit 1; }
code() { grep -vE ':[0-9]+:[[:space:]]*(//|/\*|\*)'; }
fail=0; fired=""

echo "language gate (${#FILES[@]} source files read)"

if hits=$(grep -niE -e 'authorized|deferred|authority' "${FILES[@]}" /dev/null | code); then
  echo "  governance language in the product:"; echo "$hits" | sed 's/^/    /'; fail=1; fired="$fired 1"
else echo "  no governance language"; fi

if hits=$(grep -nwE -e 'OBS|INF|STD' "${FILES[@]}" /dev/null | code); then
  echo "  authority markers carried across:"; echo "$hits" | sed 's/^/    /'; fail=1; fired="$fired 2"
else echo "  no authority markers"; fi

hits=$(awk -F'"' '$0 !~ /^[[:space:]]*(\/\/|\/\*|\*)/ { for (i = 2; i <= NF; i += 2) if ($i ~ /[a-z]\(s\)/) { print FILENAME ":" FNR ": " $i; break } }' "${FILES[@]}")
if [ -n "$hits" ]; then
  echo "  a programmer's plural said to a person:"; echo "$hits" | sed 's/^/    /'; fail=1; fired="$fired 3"
else echo "  no programmer's plural"; fi

if hits=$(grep -rnE -e 'scan file' -e 'count\([^;]*"rows?"' -e '\(\{f\}\)' -e '(Wrote|Brought in) \{(name|file)\}' "$APP" | code); then
  echo "  a file word on a screen of the window:"; echo "$hits" | sed 's/^/    /'; fail=1; fired="$fired 4"
else echo "  no file word on a screen of the window"; fi

if [ $SELF = 1 ]; then
  rm -rf "$T"
  [ "$fired" = " 1 2 3 4" ] && { echo "self-test: each of the four rules fired on its own line, and the comment and Some(s) did not: PASS"; exit 0; }
  echo "self-test: FAIL (fired:$fired; wanted 1 2 3 4)"; exit 1
fi
[ $fail -eq 0 ] && echo "PASS" || echo "FAIL"
exit $fail
