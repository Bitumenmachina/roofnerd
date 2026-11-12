#!/usr/bin/env bash
# A tool shows the sizes it will place, and typing is never thrown away: the two worst
# findings of the eighteenth judgment (J18-1, J18-2), walked the way the estimator
# walked them. A door is placed by typing only its width and Enter; the Door tool is
# picked again and shows what it will place; a click while its sizes are open places
# nothing and says so; a new width typed over the old one places a door of that width.
# Run from the repository root:  demos/toolsw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display
# (Xvfb, xdotool and a Vulkan driver; Mesa's software one will do). The job is made
# up. The figures are worked by hand in STATUS.md ("A tool shows the sizes it will
# place").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/toolsw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/toolsw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/tools}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }

echo "== The job: one wall, 24 ft long and 9 ft high =="
"$P" new "$J" "Typed sizes (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$J" "$S" "$WT" --points "0,0 24,0" --height "9 ft" "${K[@]}" >/dev/null
"$P" check "$J" >/dev/null

echo
echo "== The session =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
on "Plan"; sleep 6
on "Door"; sleep 3; park
shot                                    # 01 the door's fields: a height of 6' 8" and a sill of 0 ft already in them
xdotool type --delay 250 -- "3' 6\""; sleep 1
xdotool key Return; sleep 4             # the width, then Enter: it is in hand
at 6 0; park
shot                                    # 02 a door 3' 6" wide at 6 ft
esc; esc
on "Door"; sleep 3; park
shot                                    # 03 picked again: its fields show 3' 6", 6' 8", 0 ft
at 12 0; park
shot                                    # 04 a click with the sizes still open: nothing placed, and it says so
on "Door"; sleep 3
xdotool type --delay 250 -- "2' 8\""; sleep 1
xdotool key Return; sleep 4
at 18 0; park
shot                                    # 05 a second door, 2' 8" wide, at 18 ft
esc; esc
xdotool key F2; sleep 5
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
[ ! -e "$T/missing.txt" ] || { echo "FAIL: a control was not on screen, or the plan was not"; cat "$T/missing.txt"; exit 1; }
for n in 01 02 03 04 05; do [ -s "$OUT/$n-model.png" ] || { echo "FAIL: no picture $n"; exit 1; }; done

echo
echo "== The job, read back =="
python3 - "$J" <<'PY'
import json, sys
data = json.load(open(sys.argv[1]))["data"]
by = {e["path"]: e for e in data}
doors = [e for e in data if "pl::opening" in e.get("attributes", {})]
sizes = []
for d in doors:
    o = d["attributes"]["pl::opening"]
    t = by[o["type"]["value"]]["attributes"]["pl::opening_type"]
    feet = lambda v: v["value"] if v["unit"] == "ft" else (v["value"] / 12 if v["unit"] == "in" else v["value"][0] + v["value"][1] / 12)
    sizes.append((round(feet(o["offset"]), 2), round(feet(t["width"]), 4), round(feet(t["height"]), 4)))
sizes.sort()
print("    doors (where along the wall, width, height, in feet):", sizes)
assert len(sizes) == 2, f"{len(sizes)} doors: the click with the sizes open placed one, or a door was lost"
(a0, w0, h0), (a1, w1, h1) = sizes
assert abs(a0 - 6) < 0.3 and abs(a1 - 18) < 0.3, (a0, a1)
assert (w0, h0) == (3.5, 6.6667), (w0, h0)      # 3' 6" by 6' 8"
assert (w1, h1) == (2.6667, 6.6667), (w1, h1)    # 2' 8" by 6' 8": typed over the 3' 6"
print("    3' 6\" by 6' 8\" at 6 ft, and 2' 8\" by 6' 8\" at 18 ft: each the size that was on screen")
PY
OUTS="$("$P" show "$J")"
grep -qE "opening area +41\.1" <<<"$OUTS" || { echo "FAIL: the wall's openings are not 41.11 SF"; grep -i "opening" <<<"$OUTS" | head -5; exit 1; }
echo "    the wall's openings: 41.11 SF (3.5 x 6.6667 + 2.6667 x 6.6667)"
echo
echo "PASS: a door went in from its width and Enter; picked again the tool showed what it would place; a click with its sizes open placed nothing; and a width typed over the old one placed that width"
