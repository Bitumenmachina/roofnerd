#!/usr/bin/env bash
# Lists every crate this workspace links (normal and build dependencies) with its license,
# and fails if any license is outside the permissive list. Run from the repository root.
set -euo pipefail
cd "$(dirname "$0")/.."
ALLOW='^(MIT|Apache-2\.0|BSD-2-Clause|BSD-3-Clause|ISC|Zlib|0BSD|Unicode-3\.0|Unicode-DFS-2016|CC0-1\.0|BSL-1\.0|Unlicense)$'
bad=0
echo "| Crate | Version | License |"
echo "|---|---|---|"
while IFS='|' read -r name ver lic; do
  [ -z "$name" ] && continue
  echo "| $name | $ver | $lic |"
  # Every alternative in an OR must be checked only until one is allowed; every part of an AND must be allowed.
  ok=0
  IFS=$'\n'
  for alt in $(echo "$lic" | sed -E 's/ OR /\n/g; s#/#\n#g'); do
    all=1
    for part in $(echo "$alt" | sed -E 's/ AND /\n/g; s/[()]//g'); do
      part="$(echo "$part" | sed -E 's/ WITH .*//; s/^ +| +$//g')"
      echo "$part" | grep -Eq "$ALLOW" || all=0
    done
    [ "$all" = 1 ] && ok=1
  done
  unset IFS
  if [ "$ok" = 0 ]; then echo "NOT PERMISSIVE: $name $ver ($lic)" >&2; bad=1; fi
done < <(cargo tree --workspace -e normal,build --prefix none -f '{p}|{l}' | sed -E 's/ \(\*\)//; s/ \(proc-macro\)//; s/ \(\/[^)]*\)//; s/^([^ ]+) v([^| ]+)\|/\1|\2|/' | grep -v '^plumbline' | sort -u)
exit $bad
