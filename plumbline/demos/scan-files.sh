#!/usr/bin/env bash
# A scanner's own files taken in: LAS, its packed kind (LAZ) and E57, down the same road as a phone's PLY
# (DECISIONS.md call 230; contract 230M). Run from the repository root:  demos/scan-files.sh
#
# Every file is made up by demos/scan-files/make.py (and, for the packed and exchange kinds,
# `made-up-scan-files`, an example of the plans crate, which the crates that read them write with): a room 12 ft by 10 ft and 8 ft high, 9,474
# points. No real scan is in the repository. The figures are worked by hand in STATUS.md ("A scanner's own files").
# Each step ends the script, with FAIL, when the program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline -p plumbline-plans; fi
B="${PLUMBLINE_BIN:-$PWD/target/debug}"
P="$B/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -30; exit 1; }; printf '    %s\n' "$2"; }
notsaid() { if grep -qF -- "$2" <<<"$3"; then echo "FAIL: $1: said, and should not be: $2"; echo "$3" | head -30; exit 1; fi; }
refused() { local out; if out="$("${@:3}" 2>&1)"; then echo "FAIL: $1: taken, and should have been refused"; exit 1; fi; says "$1" "$2" "$out"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# A fresh job with one level; prints its folder.
fresh() { mkdir -p "$T/$1"; "$P" new "$T/$1/job.json" "$1 (not a real job)" >/dev/null; "$P" level "$T/$1/job.json" ground --elevation "0 ft" --height "8 ft" --status Assumed --source "to be checked on site" >"$T/$1/level.txt"; }
level() { made "$(cat "$T/$1/level.txt")"; }
bring() { local d="$1" f="$2"; shift 2; "$P" import scan "$T/$d/job.json" "$T/$f" --level "$(level "$d")" "$@"; }
SHEET="room cut at 4 ft 0 in.png"

echo "== The made-up room, as a PLY, as LAS files of several kinds, packed and as E57 =="
python3 demos/scan-files/make.py "$T"
# The writer of the packed and exchange copies is an example of the plans crate, built here only when it is not already built.
MAKER="$B/examples/made-up-scan-files"
[ -x "$MAKER" ] || { echo "building the example that writes the packed and E57 copies"; cargo build --quiet -p plumbline-plans --example made-up-scan-files; }
"$MAKER" "$T/room.las" "$T"
ls "$T" | tr '\n' ' '; echo

echo
echo "== 1. The LAS is read: the same 9,474 points as the PLY, the same room =="
fresh ply; fresh las
PLY="$(bring ply room.ply)"
LAS="$(bring las room.las)"
says "the PLY, as it was" "scan \"room\": 9,474 points, taken as meters with z up" "$PLY"
notsaid "the PLY says nothing of a file kind" "read as a" "$PLY"
says "the LAS: the same count, the same words" "scan \"room\": 9,474 points, taken as meters with z up" "$LAS"
says "it says what kind it read" "read as a LAS file" "$LAS"
says "a LAS with no coordinate system does not give a unit: said" "the file does not say what its unit is" "$LAS"
says "12 ft by 10 ft (the box), to the inch" "it is about 12' 0\" by 10' 0\"" "$LAS"
says "8 ft floor to ceiling" "floor to ceiling reads about 8' 0\" on the scan" "$LAS"
says "what a LAS holds that a sheet cannot use is said" "not kept: each point's intensity, return numbers, classification, scan angle (only where a point is is kept)" "$LAS"

echo
echo "== 2. Cut at 4 ft: the sheet from the LAS is the sheet from the PLY, dot for dot =="
says "four walls at one height: 49 + 49 + 39 + 39 points" "176 points drawn" "$LAS"
says "(12 + 4) x 24 by (10 + 4) x 24 dots" "384 by 336 dots at 24 to the foot" "$LAS"
cmp -s "$T/ply/$SHEET" "$T/las/$SHEET" || { echo "FAIL: the sheet from the LAS is not the sheet from the PLY"; exit 1; }
echo "    the two sheets are the same picture, byte for byte"

echo
echo "== 3. Traced over the LAS's sheet at the room's four inside corners: 12 + 10 + 12 + 10 = 44.00 LF, Assumed, saying the scan =="
D="$(made "$LAS")"
WT="$(made "$("$P" type "$T/las/job.json" wall "wall 4 in" --thickness "4 in" --status Assumed --source "to be checked on site")")"
"$P" wall "$T/las/job.json" "$(level las)" "$WT" --points "48,48 336,48 336,288 48,288" --closed --height "8 ft" --on "$D" >/dev/null
"$P" check "$T/las/job.json"
says "the walls" "walls 4            44.00 LF" "$("$P" takeoff "$T/las/job.json")"
DR="$("$P" drawings "$T/las/job.json")"
says "the sheet's source names the LAS and what was taken for granted" "scan room.las, cut 4' 0\" above its floor (taken as meters with z up)" "$DR"
says "the sheet is Assumed" "~ room [$D]" "$DR"
python3 - "$T/las/job.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
walls = [e["attributes"]["pl::wall"] for e in d if "pl::wall" in e["attributes"]]
assert len(walls) == 4, len(walls)
ends = sorted({tuple(round(c, 6) for c in p) for w in walls for p in w["line"]["value"]})
assert ends == [(0.0, 0.0), (0.0, 10.0), (12.0, 0.0), (12.0, 10.0)], ends
assert all(w["line"]["status"] == "Assumed" for w in walls), [w["line"]["status"] for w in walls]
print("    their corners on the plan: (0, 0), (12, 0), (12, 10), (0, 10); each Assumed, none Known")
PY

echo
echo "== 4. A kind of LAS it does not read is refused by its name, and one cut short where it is cut; the job left as it was =="
fresh refuse
cp "$T/refuse/job.json" "$T/refuse/before.json"
R() { bring refuse "$1" "${@:2}"; }
refused "a point record format LAS does not define" "point record format 11, which LAS does not define (it defines 0 to 10)" R format11.las
refused "LAS 2.0" "it is LAS 2.0, which this does not read (it reads LAS 1.1 to 1.4)" R version20.las
refused "cut short" "it is cut short: it says 9474 points and holds 188" R cut.las
refused "degrees" "degrees of latitude and longitude, which are not lengths" R degrees.las
refused "two units" "it says feet across and meters up: say which it is with --units m or --units ft" R mixed.las
printf 'solid not a scan\n' > "$T/not.xyz"
refused "a file that is none of them" "nor a LAS, LAZ or E57 file" R not.xyz
refused "a scan handed over as a drawing" "it is a LAS scan, a cloud of points, not a drawing: bring it in with \`plumbline import scan\`" "$P" drawing "$T/refuse/job.json" "$(level refuse)" "scan as drawing" --file "$T/room.las" --status Assumed --source "x"
cmp -s "$T/refuse/job.json" "$T/refuse/before.json" || { echo "FAIL: a refused scan changed the job"; exit 1; }
echo "    the job is as it was after each"
echo
echo "   Said, never guessed: the file that says two units is read when the person says which"
fresh two
TWO="$(bring two mixed.las --units ft)"
says "taken as said, and what the file says is said" "the file's unit is not clear (it says feet across and meters up): taken as feet, as you said" "$TWO"
echo "   A point the file marks withheld (its word for deleted) is left out and counted"
fresh held
HELD="$(bring held withheld.las)"
says "one point fewer" "scan \"withheld\": 9,473 points" "$HELD"
says "and said" "1 point the file marks withheld (its word for deleted), left out" "$HELD"

echo
echo "== 4b. A file that says its own unit has it offered and said; what the person says wins and is said =="
fresh feet; fresh feet2; fresh survey
FEET="$(bring feet feet.las)"
says "the file says feet, and is taken as feet" "taken as feet with z up" "$FEET"
says "said" "the file says its unit is feet" "$FEET"
says "the same room" "it is about 12' 0\" by 10' 0\"" "$FEET"
cmp -s "$T/ply/$SHEET" "$T/feet/feet cut at 4 ft 0 in.png" || { echo "FAIL: the room kept in feet is not the same sheet"; exit 1; }
echo "    the room kept in feet makes the same sheet as the room kept in meters"
FEET2="$(bring feet2 feet.las --units m)"
says "the person's word wins, and both are said" "the file says its unit is feet and you said meters: taken as meters, as you said" "$FEET2"
SURVEY="$(bring survey survey.las)"
says "a LAS 1.4 file with colour, in a system that says meters" "read as a LAS file" "$SURVEY"
says "its unit, from the system written as text" "the file says its unit is meters" "$SURVEY"
says "far from the zero (512000 m east): the same room, the same sheet" "it is about 12' 0\" by 10' 0\"" "$SURVEY"
says "and its colour is said to be left" "colour" "$SURVEY"
says "the shift is in the sheet's own source, where it is read later" "its southwest corner taken to zero: it lay 512,000 m east and 4,200,000 m north in the file's own grid" "$("$P" drawings "$T/survey/job.json")"
cmp -s "$T/ply/$SHEET" "$T/survey/survey cut at 4 ft 0 in.png" || { echo "FAIL: the room far from zero is not the same sheet"; exit 1; }
echo "    the room 512000 m east and 4200000 m north of zero makes the same sheet as the room at zero"

echo
echo "== 5. LAZ (LAS packed by LASzip): read, by the laz crate (Apache-2.0); the same room =="
fresh laz
LAZ="$(bring laz room.laz)"
says "a LAZ" "read as a LAZ file" "$LAZ"
says "the same points" "scan \"room\": 9,474 points" "$LAZ"
says "the same sheet size" "384 by 336 dots at 24 to the foot" "$LAZ"
cmp -s "$T/ply/$SHEET" "$T/laz/$SHEET" || { echo "FAIL: the sheet from the LAZ is not the sheet from the PLY"; exit 1; }
echo "    the sheet from the LAZ is the sheet from the PLY, byte for byte ($(stat -c %s "$T/room.laz") bytes packed, $(stat -c %s "$T/room.las") as LAS)"
head -c 9000 "$T/room.laz" > "$T/short.laz"
refused "a LAZ cut short" "it is cut short or damaged" R short.laz

echo
echo "== 6. E57 (ASTM E2807): read, by the e57 crate (MIT); the same room =="
fresh e57
E57="$(bring e57 room.e57)"
says "an E57" "read as an E57 file" "$E57"
says "the same points" "scan \"room\": 9,474 points" "$E57"
says "an E57 keeps meters, and says so" "the file says its unit is meters" "$E57"
says "and says nothing of up" "the file does not say which way is up" "$E57"
cmp -s "$T/ply/$SHEET" "$T/e57/$SHEET" || { echo "FAIL: the sheet from the E57 is not the sheet from the PLY"; exit 1; }
echo "    the sheet from the E57 is the sheet from the PLY, byte for byte"
head -c 100000 "$T/room.e57" > "$T/short.e57"
refused "an E57 cut short" "it is cut short or damaged: an E57 file ends with its own list" R short.e57

echo
echo "== 6b. Every verb that takes a scan takes the LAS: find-walls says the same of it as of the PLY =="
fresh fw1; fresh fw2
FW1="$("$P" find-walls "$T/fw1/job.json" "$T/shifted.ply" --level "$(level fw1)")"
FW2="$("$P" find-walls "$T/fw2/job.json" "$T/shifted.las" --level "$(level fw2)")"
strip() { grep -vE '^  (the file |read as |not kept)' <<<"$1"; }
[ "$(strip "$FW1")" = "$(strip "$FW2")" ] || { echo "FAIL: find-walls reads the LAS otherwise than the PLY"; diff <(echo "$FW1") <(echo "$FW2") | head -20; exit 1; }
says "a finding is made" "walls found:" "$FW2"
says "from the LAS, and what it says of itself" "the file does not say what its unit is" "$FW2"
echo "    find-walls on the LAS and on the PLY say the same, line for line, save the lines about the file's own words"

echo
echo "== 7. Nothing of the sample whole house moves =="
"$P" sixteen "$T/house.json" >/dev/null
SUM="$(sha256sum "$T/house.json" | cut -c1-16)"
[ "$SUM" = "880d9fd3c364b5f7" ] || { echo "FAIL: the sample whole house is $SUM, not 880d9fd3c364b5f7"; exit 1; }
echo "    plumbline sixteen: sha256 $SUM"
echo "PASS: a scanner's own files (LAS, LAZ, E57) read down the same road as a PLY: the same room, the same sheet, Assumed, units and up said, what cannot be read refused by name"
