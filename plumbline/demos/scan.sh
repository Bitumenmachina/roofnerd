#!/usr/bin/env bash
# A scan of a room (a PLY point cloud) brought in as a sheet to trace over: to scale, in
# the scan's own place, Assumed (DECISIONS.md call 128; Patrick, 2026-11-17: "It's
# approved for ply import"). Run from the repository root:  demos/scan.sh
#
# The scan is made up by tools/made-up-scan.py (no real scan is in the repository): a
# room 20 ft by 12 ft inside and 8 ft high, a point every 0.05 m on its walls, floor and
# ceiling, in metres with z up, its southwest inside corner at the scan's zero. The
# figures are worked by hand in STATUS.md ("A scan brought in as a sheet to trace").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/room.json"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -30; exit 1; }; printf '    %s\n' "$2"; }
refused() { local out; if out="$("${@:3}" 2>&1)"; then echo "FAIL: $1: taken, and should have been refused"; exit 1; fi; says "$1" "$2" "$out"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }

echo "== The made-up scan, as text and as bytes, and the same room in feet with y up =="
python3 tools/made-up-scan.py "$T/room.ply"
python3 tools/made-up-scan.py "$T/bytes.ply" --binary
python3 tools/made-up-scan.py "$T/feet.ply" --feet-y-up
"$P" new "$J" "A scanned room (not a real job)" >/dev/null
L="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "8 ft" --status Assumed --source "to be checked on site")")"

echo
echo "== Brought in: what it read, what it took for granted, and the sheet it made =="
IN="$("$P" import scan "$J" "$T/room.ply" --level "$L")"
D="$(made "$IN")"
says "the points, and what was taken for granted" "36,924 points, taken as metres with z up (--units ft or --up y says otherwise)" "$IN"
says "6.096 m by 3.6576 m" "on plan it is 20' 0\" by 12' 0\"" "$IN"
says "2.4384 m" "floor to ceiling reads 8' 0\" on the scan" "$IN"
says "the sheet: (20 + 4) x 24 by (12 + 4) x 24 dots" "576 by 384 dots at 24 to the foot" "$IN"
says "what is not kept" "not kept: each point's red" "$IN"
[ -s "$T/room cut at 4 ft 0 in.png" ] || { echo "FAIL: no picture of the cut beside the job"; exit 1; }
"$P" check "$J"
DR="$("$P" drawings "$J")"
says "to scale: 576 dots are 24 ft" "24 ft between (0, 0) and (576, 0) on its page: 24 dots to the foot" "$DR"
says "its source, with what was taken for granted" "scan room.ply, cut 4' 0\" above its floor (taken as metres with z up)" "$DR"
says "Assumed" "~ room [$D]" "$DR"

echo
echo "== Traced over it at the room's four inside corners: 20 + 12 + 20 + 12 = 64.00 LF =="
WT="$(made "$("$P" type "$J" wall "wall 4 in" --thickness "4 in" --status Assumed --source "to be checked on site")")"
"$P" wall "$J" "$L" "$WT" --points "48,48 528,48 528,336 48,336" --closed --height "8 ft" --on "$D" >/dev/null
"$P" check "$J"
says "the walls" "walls 4            64.00 LF" "$("$P" takeoff "$J")"
python3 - "$J" "$D" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
walls = [e["attributes"]["pl::wall"] for e in d if "pl::wall" in e["attributes"]]
ends = sorted({tuple(round(c, 6) for c in p) for w in walls for p in w["line"]["value"]})
assert ends == [(0.0, 0.0), (0.0, 12.0), (20.0, 0.0), (20.0, 12.0)], ends
assert all(w["line"]["status"] == "Assumed" and sys.argv[2] in w["line"]["source"] or "room" in w["line"]["source"] for w in walls), [w["line"]["source"] for w in walls]
print("    their corners on the plan: (0, 0), (20, 0), (20, 12), (0, 12); each Assumed, traced from the drawing")
PY

echo
echo "== The same scan as bytes: the same sheet, dot for dot; and in feet with y up: the same room =="
cp "$J" "$T/again.json"; "$P" new "$T/b.json" "bytes (not a real job)" >/dev/null
LB="$(made "$("$P" level "$T/b.json" ground --elevation "0 ft" --height "8 ft" --status Assumed --source "x")")"
"$P" import scan "$T/b.json" "$T/bytes.ply" --level "$LB" >/dev/null
python3 - "$T/room cut at 4 ft 0 in.png" "$T/bytes cut at 4 ft 0 in.png" <<'PY'
import sys
from PIL import Image
a, b = (Image.open(p).convert("L") for p in sys.argv[1:3])
assert a.size == b.size == (576, 384) and a.tobytes() == b.tobytes(), "the two sheets differ"
dark = sum(1 for v in a.tobytes() if v < 128)
# The cut: four walls' points at one height. On the sheet they lie on the room's inside
# line, 48 dots in from each edge: dark dots there and none in the middle of the room.
assert a.getpixel((288, 384 - 1 - 48)) < 128 and a.getpixel((48, 192)) < 128 and a.getpixel((288, 192)) > 128, "the walls are not where the room's are"
print(f"    the same sheet from text and from bytes, 576 by 384, {dark} dark dots, the walls on the room's line and the room clear")
PY
FT="$("$P" import scan "$T/b.json" "$T/feet.ply" --level "$LB" --units ft --up y)"
says "the same room, told it is in feet with y up" "taken as feet with y up" "$FT"
says "the same size" "on plan it is 20' 0\" by 12' 0\"" "$FT"
says "the same height" "floor to ceiling reads 8' 0\" on the scan" "$FT"

echo
echo "== Refused, each in a sentence, and the job left as it was =="
cp "$J" "$T/before.json"
printf 'solid not a scan\n' > "$T/not.ply"
printf 'ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nend_header\n1\n' > "$T/nox.ply"
head -c 4000 "$T/room.ply" > "$T/short.ply"
refused "not a PLY" "it is not a PLY file" "$P" import scan "$J" "$T/not.ply" --level "$L"
refused "no x, y, z" "its points have no x, y and z" "$P" import scan "$J" "$T/nox.ply" --level "$L"
refused "cut short" "it is cut short: it says 36924 points and holds" "$P" import scan "$J" "$T/short.ply" --level "$L"
refused "units" "--units \"yd\": a scan is in m or ft" "$P" import scan "$J" "$T/room.ply" --level "$L" --units yd
refused "up" "--up \"x\": up is z or y" "$P" import scan "$J" "$T/room.ply" --level "$L" --up x
refused "no level said" "say which level the scan is of" "$P" import scan "$J" "$T/room.ply"
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused scan changed the job"; exit 1; }
echo "    the job is as it was after each"
echo "PASS: a scan brought in as a sheet to trace, to scale and in its own place, Assumed, to the figures worked by hand"
