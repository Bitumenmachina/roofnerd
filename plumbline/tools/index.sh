#!/usr/bin/env bash
# Two lists that went stale when they were kept by hand, kept by this instead.
#   tools/index.sh            write DECISIONS-index.md: every heading and the line it starts on
#   tools/index.sh --check    fail if that index is not the one DECISIONS.md gives today, or if
#                             README.md does not name a demonstration that is in demos/
# The commit hook runs the check. Run from anywhere.
set -u
cd "$(dirname "$0")/.." || exit 1
make() {
  echo "# Index of DECISIONS.md"
  echo
  echo "Written by \`tools/index.sh\`; not edited by hand. Every heading of \`DECISIONS.md\` and the line it"
  echo "starts on. A call with \"as built\" in its heading is the record of what was made."
  echo
  echo "| Line | Heading |"
  echo "|---:|---|"
  grep -nE '^#{2,3} ' DECISIONS.md | sed -E 's/^([0-9]+):#{2,3} (.*)$/| \1 | \2 |/'
}
if [ "${1:-}" = --check ]; then
  bad=0
  if ! make | cmp -s - DECISIONS-index.md; then echo "DECISIONS-index.md is not what DECISIONS.md gives: run plumbline/tools/index.sh"; bad=1; fi
  for f in demos/*.sh; do
    n=$(basename "$f")
    case $n in window.sh|field-job.sh) continue;; esac
    grep -qF "demos/$n" README.md || { echo "README.md does not name demos/$n"; bad=1; }
  done
  exit $bad
fi
make > DECISIONS-index.md
echo "DECISIONS-index.md: $(grep -c '^| [0-9]' DECISIONS-index.md) headings"
