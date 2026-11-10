#!/usr/bin/env bash
# Lists every crate this workspace links (normal and build dependencies) with its license,
# and fails if any license expression can only be met by accepting a non-permissive
# license. Run from the repository root.
#
#   tools/licenses.sh              the table; exit 1 if any crate fails
#   tools/licenses.sh --self-test  check the expression logic against known cases
set -euo pipefail
cd "$(dirname "$0")/.."
if [ "${1:-}" = "--self-test" ]; then
  exec python3 tools/license_expr.py --self-test
fi
cargo tree --workspace -e normal,build --prefix none -f '{p}|{l}' \
  | sed -E 's/ \(\*\)//; s/ \(proc-macro\)//; s/ \(\/[^)]*\)//; s/^([^ ]+) v([^| ]+)\|/\1|\2|/' \
  | grep -v '^plumbline' | sort -u \
  | python3 tools/license_expr.py
