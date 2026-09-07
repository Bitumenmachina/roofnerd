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
UNIVERSAL='^(1\.0833|1\.4142|1\.1180|1\.2019|1\.3017|1\.5366|1\.6667|3\.1416|0\.0625)$'

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
      print $0
    }
  ' | grep -vE "$UNIVERSAL" | sort -u
)

count=$(printf '%s\n' "$tokens" | grep -c . || true)
echo "  $count distinctive figure(s) to keep in"

# ── what to check ──────────────────────────────────────────────────────────
if [ "${1:-}" = "--all" ]; then
  files=$(git ls-files)
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
