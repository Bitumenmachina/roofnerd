#!/usr/bin/env bash
# Lists every crate this workspace links (normal and build dependencies) with its license,
# and fails if any license expression can only be met by accepting a non-permissive
# license. Run from the repository root.
#
#   tools/licenses.sh              the table; exit 1 if any crate fails
#   tools/licenses.sh --self-test  check the expression logic against known cases
#   tools/licenses.sh --write      write the table, its count and the direct dependencies into LICENSES.md
#   tools/licenses.sh --check      fail if LICENSES.md does not hold what --write would write
set -euo pipefail
cd "$(dirname "$0")/.."
if [ "${1:-}" = "--self-test" ]; then
  exec python3 tools/license_expr.py --self-test
fi
table() {
  cargo tree --workspace -e normal,build --prefix none -f '{p}|{l}' \
    | sed -E 's/ \(\*\)//; s/ \(proc-macro\)//; s/ \(\/[^)]*\)//; s/^([^ ]+) v([^| ]+)\|/\1|\2|/' \
    | grep -v '^plumbline' | sort -u \
    | python3 tools/license_expr.py
}
case "${1:-}" in
  --write|--check)
    T=$(mktemp)
    table > "$T" || { cat "$T"; rm -f "$T"; exit 1; }
    cargo metadata --no-deps --format-version 1 > "$T.meta"
    r=0; python3 tools/licenses_doc.py "$T" "$T.meta" "$1" || r=$?
    rm -f "$T" "$T.meta"; exit $r;;
esac
table
