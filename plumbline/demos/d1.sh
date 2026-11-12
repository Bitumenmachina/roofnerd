#!/usr/bin/env bash
# D1 demonstration: the saved job is the world.
# Run from the repository root:  demos/d1.sh
# It uses only the sample job (made up; no real job data) and a temporary folder.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT

echo "== 1. Write the sample job and confirm the file is exactly what the world saves =="
"$P" sample "$T/before.json"
"$P" check "$T/before.json"
echo "entries in the file: $(grep -c '"path":' "$T/before.json")   distinct paths: $(grep '"path":' "$T/before.json" | sort -u | wc -l)"

echo
echo "== 2. Change one pitch: the porch plane (plane-0017), 3:12 Assumed -> 4:12 Known =="
cp "$T/before.json" "$T/after.json"
"$P" set "$T/after.json" plane-0017 pitch 4 --status Known --source "field measured"
"$P" check "$T/after.json"

echo
echo "== 3. The file diff, with the element each change sits in =="
# For every changed line, print the path of the entry it belongs to once, then the change.
diff --unchanged-line-format='' --old-line-format='%dn|- %L' --new-line-format='%dn|+ %L' "$T/before.json" "$T/after.json" > "$T/changes.txt" || true
python3 - "$T/before.json" "$T/after.json" "$T/changes.txt" <<'PY'
import sys, re
before, after, changes = sys.argv[1:]
def owner_map(path):
    owner, cur = {}, None
    for i, line in enumerate(open(path), 1):
        m = re.search(r'"path": "([^"]+)"', line)
        if m: cur = m.group(1)
        owner[i] = cur
    return owner
ob, oa = owner_map(before), owner_map(after)
last, touched = None, []
for line in open(changes):
    n, rest = line.split('|', 1)
    who = (ob if rest.startswith('-') else oa)[int(n)]
    if who != last:
        print(f"  [{who}]"); last = who
        if who not in touched: touched.append(who)
    print("    " + rest.rstrip())
print()
print("elements with any change:", ", ".join(touched))
expected = ["plane-0017", "eave-0018", "rake-0019", "rake-0020", "headwall-0021"]
if touched != expected:
    print("FAIL: expected exactly", expected); sys.exit(1)
print("PASS: only the plane whose pitch changed and the four lines that lie on it")
PY

echo
echo "== 4. Assumed and Unknown values display as flagged (before the change) =="
"$P" show "$T/before.json" | sed -n '/^Flagged values:/,$p'
"$P" show "$T/before.json" | grep -E "Known, .* Assumed, .* Unknown"

echo
echo "== 5. The same list after the change: the porch pitch and what follows from it are no longer flagged =="
"$P" show "$T/after.json" | sed -n '/^Flagged values:/,$p'
"$P" show "$T/after.json" | grep -E "Known, .* Assumed, .* Unknown"

echo
echo "== 6. A pitch change the drawn linework cannot follow is flagged, not guessed =="
cp "$T/before.json" "$T/conflict.json"
"$P" set "$T/conflict.json" plane-0003 pitch 8 --status Known --source "revised"
"$P" show "$T/conflict.json" | grep -E "disagree" | cut -c1-230
