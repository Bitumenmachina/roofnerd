#!/usr/bin/env bash
# Units demonstration: the owner's ruling that overruled "feet only" (DECISIONS.md call 16).
# Run from the repository root:  demos/units.sh
# It uses only the sample job (made up; no real job data) and a temporary folder.
# Every step fails the script if the program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT

# The value of one derived figure, as `show` prints it.
figure() { "$P" show "$1" | grep -A40 "^ *$2 " | grep -m1 " $3 " | sed -E "s/.* $3 +//"; }
# One attribute of one element, as it stands in the saved file.
saved() {
  python3 - "$1" "$2" "$3" <<'PY'
import sys, json
f, path, key = sys.argv[1:]
for e in json.load(open(f))["data"]:
    if e["path"] == path:
        for line in json.dumps({key: e["attributes"][key]}, indent=2, ensure_ascii=False).splitlines()[1:-1]:
            print("  " + line)
PY
}
# The same file with one unit rewritten or removed by hand. $4 is the new unit, or "" to remove it.
hand_edit() {
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys, json
src, dst, path, unit = sys.argv[1:]
j = json.load(open(src))
for e in j["data"]:
    if e["path"] == path:
        if unit: e["attributes"]["pl::height"]["unit"] = unit
        else: del e["attributes"]["pl::height"]["unit"]
json.dump(j, open(dst, "w"))
PY
}
# The changed lines between two saves, each under the element it belongs to, then the
# list of elements touched. $3 is the list expected.
changes() {
  python3 - "$1" "$2" "$3" <<'PY'
import sys, re, difflib
before, after, expected = sys.argv[1:]
def owners(path):
    out, cur = [], None
    for line in open(path):
        m = re.search(r'"path": "([^"]+)"', line)
        if m: cur = m.group(1)
        out.append((cur, line.rstrip("\n")))
    return out
a, b = owners(before), owners(after)
last, touched = None, []
for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, [l for _, l in a], [l for _, l in b], autojunk=False).get_opcodes():
    if tag == "equal": continue
    for who, line, sign in [(w, l, "-") for w, l in a[i1:i2]] + [(w, l, "+") for w, l in b[j1:j2]]:
        if who != last:
            print(f"  [{who}]"); last = who
            if who not in touched: touched.append(who)
        print(f"    {sign} {line.strip()}")
print("elements with any change:", ", ".join(touched) or "none")
if touched != [e for e in expected.split(",") if e]:
    print("FAIL: expected exactly", expected); sys.exit(1)
PY
}
refused() { # $1 = the words the refusal has to contain; the rest = the command
  local want="$1"; shift
  local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "  refused: $out" | cut -c1-240
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}

"$P" sample "$T/job.json" >/dev/null
SRC=(--status Known --source "sample detail 3/A-501")

echo "== 1. A parapet height of 30 in gives the same face area as 2.5 ft =="
cp "$T/job.json" "$T/in.json"; cp "$T/job.json" "$T/ft.json"
"$P" set "$T/in.json" parapet-0024 height "30 in" "${SRC[@]}" | head -1
"$P" set "$T/ft.json" parapet-0024 height "2.5 ft" "${SRC[@]}" | head -1
A="$(figure "$T/in.json" parapet-0024 face_area)"; B="$(figure "$T/ft.json" parapet-0024 face_area)"
echo "  entered as 30 in:  face_area $A     entered as 2.5 ft:  face_area $B     (20 LF of parapet)"
[ "$A" = "50 SF" ] && [ "$B" = "50 SF" ] || { echo "FAIL: expected 50 SF both ways"; exit 1; }
echo "  PASS"

echo
echo "== 2. 25' 6\", 25.5 ft, 306 in and 25' 6 1/2\" are each accepted and each mean what they say =="
for typed in "25' 6\"" "25.5 ft" "306 in" "25' 6 1/2\"" "25' 6 1/16\""; do
  cp "$T/job.json" "$T/h.json"
  SET="$("$P" set "$T/h.json" parapet-0024 height "$typed" "${SRC[@]}" | head -1)"
  printf '  typed %-13s %-46s face_area %s\n' "$typed" "$SET" "$(figure "$T/h.json" parapet-0024 face_area)"
  case "$typed" in
    "25' 6 1/2\"")  want="510.8333 SF";;   # 20 LF x 306.5 in / 12
    "25' 6 1/16\"") want="510.1042 SF";;   # 20 LF x 306.0625 in / 12: one sixteenth is 0.1042 SF on 20 LF
    *)              want="510 SF";;        # 20 LF x 25.5 ft
  esac
  [ "$(figure "$T/h.json" parapet-0024 face_area)" = "$want" ] || { echo "FAIL: expected $want"; exit 1; }
  "$P" check "$T/h.json" >/dev/null
done
echo "  PASS: three ways of writing 25.5 ft agree; a half inch and a sixteenth each show"

echo
echo "== 3. A value is shown and saved in the unit it was entered in; a unit change touches that value and what derives from it =="
echo "  as saved after entering 30 in:"
saved "$T/in.json" parapet-0024 pl::height
"$P" show "$T/in.json" | grep -E "^ +(=)? +. ?(height|face_area) " | sed 's/^ */    shown:  /'
cp "$T/in.json" "$T/in2ft.json"
"$P" set "$T/in2ft.json" parapet-0024 height "2.5 ft" "${SRC[@]}" | sed 's/^/  /'
echo "  the file diff for 30 in -> 2.5 ft (same height, written the other way):"
changes "$T/in.json" "$T/in2ft.json" "parapet-0024"
cp "$T/in.json" "$T/in2ftin.json"
"$P" set "$T/in2ftin.json" parapet-0024 height "2' 7 1/2\"" --status Assumed --source "scaled off the elevation" | sed 's/^/  /'
echo "  the file diff for 30 in -> 2' 7 1/2\" Assumed (a different height, a different unit, a different status):"
changes "$T/in.json" "$T/in2ftin.json" "parapet-0024"
"$P" show "$T/in2ftin.json" | grep -E "^ +(=)? +. ?(height|face_area) " | sed 's/^ */    shown:  /'
"$P" check "$T/in2ft.json" >/dev/null; "$P" check "$T/in2ftin.json" >/dev/null
echo "  PASS: only the parapet changed, and on it only the height and the figure derived from it"

echo
echo "== 4. A unit the program does not know is refused by name; nothing is read as feet =="
cp "$T/job.json" "$T/r.json"
refused "never read as feet"  "$P" set "$T/r.json" parapet-0024 height "30"       "${SRC[@]}"
refused "metric"              "$P" set "$T/r.json" parapet-0024 height "762 mm"   "${SRC[@]}"
refused "metric"              "$P" set "$T/r.json" parapet-0024 height "0.76 m"   "${SRC[@]}"
refused '"cubits"'            "$P" set "$T/r.json" parapet-0024 height "3 cubits" "${SRC[@]}"
refused '"yd"'                "$P" set "$T/r.json" parapet-0024 height "1 yd"     "${SRC[@]}"
refused "degrees"             "$P" set "$T/r.json" plane-0017 pitch "14 deg"      "${SRC[@]}"
cmp -s "$T/job.json" "$T/r.json" || { echo "FAIL: a refused entry changed the file"; exit 1; }
echo "  and typed straight into the file:"
hand_edit "$T/in.json" "$T/mm.json" parapet-0024 mm
refused "metric" "$P" show "$T/mm.json"
hand_edit "$T/in.json" "$T/furlong.json" parapet-0024 furlong
refused '"furlong"' "$P" show "$T/furlong.json"
hand_edit "$T/in.json" "$T/nounit.json" parapet-0024 ""
refused "never read as feet" "$P" show "$T/nounit.json"
echo "  PASS: every one refused by name, and the job file is untouched"

echo
echo "== 5. A stretch-out in inches against a length in feet, with nobody converting anything =="
"$P" show "$T/job.json" | grep -A12 "^ *headwall-0021 " | grep -E "true_length|stretch-out|sheet area" | sed 's/^ */    /'
echo "  the line's quantity as saved, formula and all:"
saved "$T/job.json" headwall-0021 pl::line_quantities
[ "$(figure "$T/job.json" headwall-0021 "sheet area: Copper headwall flashing")" = "24 SF" ] || { echo "FAIL: 12 LF x 24 in / 12 should be 24 SF"; exit 1; }
echo "  the brief's stretch-outs on the same 12 LF, each typed as the detail gives it:"
for pair in "12 in=12 SF" "8 in=8 SF" "36 in=36 SF" "24 in=24 SF" "19 1/2\"=19.5 SF" "2 ft=24 SF"; do
  typed="${pair%%=*}"; want="${pair##*=}"
  cp "$T/job.json" "$T/s.json"
  "$P" set "$T/s.json" headwall-0021 stretch_out "$typed" --status Known --source "detail" >/dev/null
  got="$(figure "$T/s.json" headwall-0021 "sheet area: Copper headwall flashing")"
  printf '    stretch-out %-9s x 12 LF  =  %s\n' "$typed" "$got"
  [ "$got" = "$want" ] || { echo "FAIL: expected $want"; exit 1; }
  "$P" check "$T/s.json" >/dev/null
done
cp "$T/job.json" "$T/s36.json"
"$P" set "$T/s36.json" headwall-0021 stretch_out "36 in" --status Known --source "sample detail 5/A-501" | sed 's/^/  /'
echo "  the file diff for stretch-out 24 in -> 36 in:"
changes "$T/job.json" "$T/s36.json" "headwall-0021"
echo "  PASS"
