#!/usr/bin/env bash
# A floor, two rooms and a roof on a house traced from a plan, at the command line: the
# made-up vector plan is laid under a storey and given its scale; its outline and its
# partition are traced; the floor is the outline those walls enclose; each room is a
# point inside it and a name; the roof is its footprint's four corners, a pitch and the
# height of its eaves.
# Run from the repository root:  demos/build.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures expected are
# worked by hand in the comments, and in STATUS.md ("Floors, rooms and roofs").
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
READ=(--status Assumed --source "read off the drawing")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
figure() { # $1 path, $2 measure
  python3 - "$J" "$1" "$2" <<'PY'
import sys, json
e = next(x for x in json.load(open(sys.argv[1]))["data"] if x["path"] == sys.argv[2])
d = e["attributes"].get("pl::measure", {}).get(sys.argv[3])
print("none" if d is None or d["value"] is None else ("%s %s" % (round(d["value"], 2), d["status"])))
PY
}
is() { [ "$2" = "$3" ] || { echo "FAIL: $1: got $2, wanted $3"; exit 1; }; printf '    %-58s %s\n' "$1" "$2"; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3"; exit 1; }; printf '    %s\n' "$2"; }
refused() { local want="$1"; shift; local out
  cp "$J" "$T/kept.json"
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-220
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
  cmp -s "$J" "$T/kept.json" || { echo "FAIL: a refusal changed the file"; exit 1; }
}

echo "== The plan laid under a storey, its scale set, and its walls traced =="
cp demos/plans/made-up-plan.pdf "$T/"
"$P" new "$J" "Built from a plan (not a real job)"
S="$(made "$("$P" storey "$J" ground --elevation "0 ft" --height "8 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
FT="$(made "$("$P" type "$J" floor "slab 4 in" --thickness "4 in" "${K[@]}")")"
D="$(made "$("$P" drawing "$J" "$S" "A-1 floor plan" --file "$T/made-up-plan.pdf" "${K[@]}")")"
"$P" calibrate "$J" "$D" --from "126,90" --to "666,90" --distance "30 ft" "${READ[@]}" >/dev/null
"$P" lineup "$J" "$D" --page-point "126,126" --plan-point "0,0" --status Assumed --source "the house's southwest corner" >/dev/null

echo
echo "== What is refused before there are walls =="
refused "no walls enclose" "$P" floor "$J" "$S" "$FT" --walls --status Assumed --source "the walls"

# The outline, 540 by 360 points at 18 to the foot: 30 ft by 20 ft. The partition, 216
# points from the west side: 12 ft.
"$P" wall "$J" "$S" "$WT" --points "126,126 666,126 666,486 126,486" --closed --height "8 ft" --on "$D" | sed 's/^/    /'
"$P" wall "$J" "$S" "$WT" --points "342,126 342,486" --height "8 ft" --on "$D" | sed 's/^/    /'

echo
echo "== The floor: the outline its walls enclose =="
OUT="$("$P" floor "$J" "$S" "$FT" --walls --status Assumed --source "the walls as traced")"; echo "    $OUT"
F="$(made "$OUT")"
# The walls' centre lines: 30 ft by 20 ft. Traced walls, so Assumed.
is "the floor, 30 ft by 20 ft" "$(figure "$F" plan_area)" "600.0 Assumed"
refused "already" "$P" floor "$J" "$S" "$FT" --walls --status Assumed --source "again"

echo
echo "== Two rooms, each a point inside it and a name =="
# Places on the page: (234, 306) is 6 ft east and 10 ft north; (504, 306) is 21 ft east.
W="$(made "$("$P" room "$J" "$S" "West room" --at "234,306" --on "$D")")"
E="$(made "$("$P" room "$J" "$S" "East room" --at "504,306" --on "$D")")"
# Inside the walls' faces: 3 in to 11 ft 9 in by 3 in to 19 ft 9 in is 11.5 by 19.5; and
# 12 ft 3 in to 29 ft 9 in is 17.5 by 19.5.
is "the west room, 11.5 ft by 19.5 ft" "$(figure "$W" floor_area)" "224.25 Assumed"
is "the east room, 17.5 ft by 19.5 ft" "$(figure "$E" floor_area)" "341.25 Assumed"

echo
echo "== The roof: its footprint, 6:12 on every side, its eaves 8 ft up =="
"$P" roof "$J" "Main roof" --points "0,0 30,0 30,20 0,20" --pitch 6:12 --eave-height "8 ft" --status Assumed --source "traced off the plan; the pitch off the elevations" | sed 's/^/    /'
OUT="$("$P" show "$J")"
# A hip roof. Ridge 30 - 20 = 10 LF. Each hip runs 10 east, 10 north and 5 up: 15 LF; four
# are 60 LF. Eaves 100 LF. On plan two sides of 200 SF and two ends of 100 SF, each times
# the square root of 1.25: 223.61 and 111.80. 670.82 SF in all.
TOTALS="$("$P" lines "$J" 2>/dev/null || true)"
python3 - "$J" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
kind = lambda e: e["attributes"].get("pl::class")
def total(k, m):
    return round(sum(e["attributes"]["pl::measure"][m]["value"] for e in data if kind(e) == k), 2)
got = {"planes": sum(1 for e in data if kind(e) == "RoofPlane"), "surface": total("RoofPlane", "surface_area"), "ridge": total("Ridge", "true_length"), "hips": total("Hip", "true_length"), "eaves": total("Eave", "true_length")}
want = {"planes": 4, "surface": 670.82, "ridge": 10.0, "hips": 60.0, "eaves": 100.0}
for k in want:
    if got[k] != want[k]:
        print(f"FAIL: the roof's {k}: got {got[k]}, wanted {want[k]}"); sys.exit(1)
    print(f"    the roof's {k:<52} {got[k]}")
areas = sorted(round(e["attributes"]["pl::measure"]["surface_area"]["value"], 2) for e in data if kind(e) == "RoofPlane")
if areas != [111.8, 111.8, 223.61, 223.61]:
    print("FAIL: the four planes:", areas); sys.exit(1)
print("    its four planes                                            ", areas)
PY

echo
echo "== What is refused, and the file is byte for byte what it was after each =="
refused "outside" "$P" room "$J" "$S" "Nowhere" --at "40,40" "${K[@]}"
refused "cross or touch" "$P" roof "$J" "Bow tie" --points "0,0 30,20 30,0 0,20" --pitch 6:12 --eave-height "8 ft" "${K[@]}"

echo
"$P" check "$J" | tail -1 | sed 's/^/    /'
echo "PASS: a floor by the outline of its walls, two rooms and a hip roof on a house traced from a plan, each to the figures worked by hand"
