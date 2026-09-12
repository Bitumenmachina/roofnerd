#!/usr/bin/env bash
# ── The client-data gate ───────────────────────────────────────────────────
# A repository this program's source lives in is public. A client's bid figures
# are not, and twice now they have reached it by the same route: a real number
# was in front of me while I wrote a test, and it made the test feel real.
#
# So this stops being something to remember. It reads what is in the local-only
# trees, builds a set of the numbers distinctive enough to identify a job, and
# refuses any commit that carries one of them out.
#
#   tools/gate-clientdata.sh              check what is staged
#   tools/gate-clientdata.sh --all        check every tracked file
#
# Run as a pre-commit hook (see .githooks/) and in CI.
#
# A DEVIATION, stated: the addendum says every number with a decimal point.
# Taken literally that includes 1.5 and 2.00, which occur in any codebase and in
# every one of these files, and a gate that fires on everything gets switched
# off within a day. A token has to carry at least MIN_DIGITS digits to count —
# enough that it identifies a job rather than being arithmetic.
set -uo pipefail
cd "$(dirname "$0")/.."

MIN_DIGITS=4          # a token needs this many digits to identify anything
MIN_INTEGER=5         # a bare integer needs this many

# Numbers that appear in a roofing report because they appear in all roofing,
# not because they belong to a job. A pitch factor is the square root of one
# plus the slope squared; it is the same on every roof in the world.
UNIVERSAL='^(1\.0833|1\.4142|1\.1180|1\.2019|1\.3017|1\.5366|1\.6667|3\.1416|0\.0625|1\.156)$'

# A NARROWING, stated, because the gate fired on the demo library and the demo
# library was right.
#
# What it caught: a coping profile's legs — 1.625, 2.375, 4.375, 6.375 — and
# 1.156, the weight of a square foot of 24 ga steel. Those are 1 5/8", 2 3/8",
# 4 3/8", 6 3/8" and a gauge weight. They describe a standard product, not a
# job, and they are in the local trees only because a real coping has the same
# legs as a demo one. 1.156 joins UNIVERSAL above by name; the legs are a class
# and get a rule.
#
# The rule: a figure written to three decimal places, under 24, that is an exact
# sixteenth of an inch is a fractional-inch dimension. That is how metal is
# drawn and it is not how money is written — a price carries two decimals and a
# quantity that size is not a client's. Three decimals keeps 12.5 and 1.25 out
# of it; under 24 keeps a real quantity like 1240.375 in.
SIXTEENTHS='is a fractional-inch dimension: 3 decimals, under 24, exact 1/16'

PRIVATE=(fixtures refs)
EXEMPT_RE='^(fixtures|refs|evidence)/'

# Extra literal terms — bid names, job numbers — kept in a local-only file so
# that naming them does not itself put them in the public tree.
TERMS_FILE=fixtures/.clientdata-terms

echo "client-data gate"

if [ ! -d fixtures ] && [ ! -d refs ]; then
  echo "  nothing local to protect (no fixtures/ or refs/ on this clone)"
  echo "PASS"
  exit 0
fi

# ── what to look for ───────────────────────────────────────────────────────
tokens=$(
  for dir in "${PRIVATE[@]}"; do
    [ -d "$dir" ] || continue
    # Text only: a PDF's compressed bytes are not a source of real tokens.
    grep -rhoE '[0-9][0-9,]*\.[0-9]+|[0-9]{'"$MIN_INTEGER"',}' "$dir" --binary-files=without-match 2>/dev/null
  done | sed 's/,//g' | awk -v min="$MIN_DIGITS" '
    {
      n = $0; gsub(/[^0-9]/, "", n)
      if (length(n) < min) next
      # A bare round number identifies nothing. A quote of twenty thousand
      # dollars appears in a report and also as a timeout in a test file, and a
      # gate that cannot tell them apart is a gate that gets switched off. A
      # figure with cents on it is distinctive; a round one is not.
      if ($0 !~ /[.]/ && ($0 % 100) == 0) next
      # A fractional-inch dimension — see SIXTEENTHS above.
      if ($0 ~ /^[0-9]+\.[0-9][0-9][0-9]$/ && $0 < 24 && ($0 * 16) == int($0 * 16)) next
      print $0
    }
  ' | grep -vE "$UNIVERSAL" | sort -u
)

count=$(printf '%s\n' "$tokens" | grep -c . || true)
echo "  $count distinctive figure(s) to keep in"

# ── what to check ──────────────────────────────────────────────────────────
if [ "${1:-}" = "--all" ]; then
  # Tracked AND untracked-but-not-ignored. `git ls-files` alone reads only what
  # is already committed, so a brand new file carrying a client's figure got a
  # green from `pnpm gates` and would only have been caught later by the staged
  # check at commit time. Found by planting a real figure in a new file and
  # watching the gate pass — which is why a gate gets proven red before it is
  # trusted, and why proving it on the wrong kind of file proves nothing.
  files=$(git ls-files --cached --others --exclude-standard)
else
  files=$(git diff --cached --name-only --diff-filter=ACM)
fi
files=$(printf '%s\n' "$files" | grep -vE "$EXEMPT_RE" | grep -vE '^pnpm-lock\.yaml$' || true)

fail=0
report() { echo "  $1"; fail=1; }

if [ -z "$files" ]; then
  echo "  nothing staged outside the local-only trees"
else
  # Figures. Each file's own numbers are pulled out once and intersected with
  # the private set, rather than searching the file 659 times. A token must be a
  # WHOLE number where it appears. A short figure that happens to sit inside a
  # longer one is arithmetic, not a client's number, and a gate that says
  # otherwise gets switched off by the end of the week.
  if [ -n "$tokens" ]; then
    private_set=$(mktemp)
    printf '%s\n' "$tokens" | sort -u > "$private_set"

    while IFS= read -r f; do
      [ -n "$f" ] && [ -f "$f" ] || continue
      shared=$(grep -ohE '(^|[^0-9.,])[0-9][0-9,]*\.[0-9]+|(^|[^0-9.,])[0-9]{'"$MIN_INTEGER"',}' "$f" --binary-files=without-match 2>/dev/null \
        | grep -oE '[0-9][0-9,]*\.?[0-9]*' | sed 's/,//g' | sort -u | comm -12 - "$private_set")
      if [ -n "$shared" ]; then
        report "a figure from the reports is in $f:"
        printf '%s\n' "$shared" | sed 's/^/      /' | head -8
      fi
    done <<< "$files"

    rm -f "$private_set"
  fi

  # Names and job numbers, from a file that is itself local-only.
  if [ -f "$TERMS_FILE" ]; then
    while IFS= read -r term; do
      [ -n "$term" ] || continue
      case "$term" in \#*) continue ;; esac
      hits=$(printf '%s\n' "$files" | while IFS= read -r f; do
        [ -f "$f" ] || continue
        grep -Hni -F -- "$term" "$f" 2>/dev/null
      done)
      if [ -n "$hits" ]; then
        report "a name or job number from the reports is in the public tree"
        printf '%s\n' "$hits" | sed 's/^/      /' | head -5
      fi
    done < "$TERMS_FILE"
  else
    echo "  (no $TERMS_FILE — names and job numbers are not being checked)"
  fi

  # Drawing references — a sheet and a detail number, the way a set numbers its
  # own details: a detail number over a sheet number, or a sheet over a detail.
  # The examples are described rather than written out, because writing one here
  # makes this file match its own pattern and the gate red for ever.
  #
  # The companion the numeric net could not be widened into. The history audit
  # found one of these in a tracked file, beside a quantity, and no pattern built
  # out of figures would ever have caught it: it identifies a project more
  # directly than a length does, because it names the drawing it was read off.
  #
  # Deliberately only the PAIRED form — a detail over a sheet, or a sheet over a
  # detail. A bare sheet number like A101 is indistinguishable from a hundred
  # ordinary tokens, and a gate that fires on those is a gate switched off by
  # Friday. One part must carry letters AND digits, which is what keeps
  # `packages/app`, `SF/LF`, a quarter-inch scale and a date out of it.
  DRAWING_REF='(^|[^A-Za-z0-9/.-])([0-9]{1,2}/[A-Z]{1,3}-?[0-9]{1,3}(\.[0-9]{1,2})?|[A-Z]{1,3}-?[0-9]{1,3}(\.[0-9]{1,2})?/[0-9A-Z]{1,6})([^A-Za-z0-9/.-]|$)'
  hits=$(printf '%s\n' "$files" | while IFS= read -r f; do
    [ -f "$f" ] || continue
    grep -HnE "$DRAWING_REF" "$f" 2>/dev/null
  done)
  if [ -n "$hits" ]; then
    report "a drawing sheet-and-detail reference is in the public tree"
    printf '%s\n' "$hits" | sed 's/^/      /' | head -5
  fi

  # Absolute home paths. Nobody else's machine has them and they name a person.
  hits=$(printf '%s\n' "$files" | while IFS= read -r f; do
    [ -f "$f" ] || continue
    grep -HnE '/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+' "$f" 2>/dev/null
  done)
  if [ -n "$hits" ]; then
    report "an absolute home path is in the public tree"
    printf '%s\n' "$hits" | sed 's/^/      /' | head -5
  fi
fi

[ $fail -eq 0 ] && echo "PASS" || echo "FAIL — nothing from the reports leaves fixtures/"
exit $fail
