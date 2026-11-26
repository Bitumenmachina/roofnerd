#!/usr/bin/env bash
# The bench (DECISIONS.md call 195, step A): one item looked at alone, in the window.
# The made-up whole house, the starter tool chest brought in, "Vanity, 36 in" set down
# twice; the vanity opened with Change and put on the bench; its width typed on its
# panel; back to the house with the button and with Esc; undone; a water heater (never
# set down) put on the bench; and the same typing done again without the bench, to
# show that the saved job is the same to the byte.
# Run from the repository root:  demos/benchw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard. The job and every size
# are made up (no real job data). Every step fails the script if the program does not
# do what the step says. STATUS.md ("The bench: an item looked at alone") has the
# figures worked by hand.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/benchw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/benchw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/benchw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
mkdir "$T/a" "$T/b"; JA="$T/a/bench.json"; JB="$T/b/bench.json"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | tail -40; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job: the made-up whole house, priced in all sixteen divisions, twice (one for the bench, one for the same typing without it) =="
"$P" sixteen "$JA" >/dev/null
cp "$JA" "$JB"
"$P" check "$JA"

# The two sessions share what puts the house in the state the bench starts from.
cat > "$T/common.sh" <<'COMMON'
# The tool chest in, the vanity set down twice on the ground in front of the house, saved.
start() {
  open_window "$1"
  sleep 8; park
  on "Starter tool chest"; sleep 5
  on_at "Item Vanity, 36 in"; sleep 4
  click 560 760; sleep 5
  on "Item Vanity, 36 in"; sleep 4
  click 900 760; sleep 5
  esc; park
  xdotool key F2; drawn_for 3 3
}
# The item opened on the right.
open_item() { on_at "Change item $1" "${2:-5}"; sleep 5; park; }
# The map as it stands, kept under a name.
keep_map() { sleep 3; cp "$T/map.txt" "$T/map-$1.txt"; }
COMMON

echo
echo "== Session A: the bench entered, a size typed, back to the house, Esc, Undo, the heater, another item's Change =="
cat > "$T/session-a.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
. "$T/common.sh"
start "$JA"
cp "$JA" "$T/saved-0.json"
open_item "Vanity, 36 in"
shot; keep_map 01                          # 01 the house, the vanity's panel open: what the bench is entered from
on_right "Bench"; sleep 12; park
shot; keep_map 02                          # 02 the vanity alone on its floor, 36 in wide
xdotool key 1; drawn_for 3 4; keep_map 02k       # a number is a tool's key: none is taken in hand there
xdotool key t; drawn_for 3 3; xdotool key v; drawn_for 3 3; keep_map 02t
# A value half typed, and Esc: that is all Esc does there; the bench stays.
on_right "Width"; sleep 3
xdotool type --delay 250 -- "4"; drawn_for 3 2; esc; keep_map 02e
on_right "Width"; sleep 3
typed "48 in"; sleep 8; park
shot; keep_map 03                          # 03 48 in wide, said, and how many follow
# Orbit, wheel and drag work as they do on the house: the wheel closes on what is under the pointer,
# a right-drag turns the view about it, and Back to the house still goes to the view that was left.
xdotool mousemove 800 520; drawn_for 2 2
for i in 1 2 3; do xdotool click 4; drawn_for 2 2; done; sleep 4; keep_map 03w
xdotool mousemove 800 520; drawn_for 2 2; xdotool mousedown 3; drawn_for 2 1
for i in 1 2 3 4 5; do xdotool mousemove_relative -- 40 0; sleep 1; done; xdotool mouseup 3; drawn_for 2 6; park; keep_map 03d
shot                                       # 03b zoomed and turned: still the one item, alone
on_right "Back to the house"; sleep 12; park
shot; keep_map 04                          # 05 the house again, as it was
xdotool key F2; drawn_for 3 3; cp "$JA" "$T/saved-1.json"
on_right "Bench"; sleep 10; park
keep_map 05b
esc; sleep 12; park
shot; keep_map 05                          # 06 Esc: the same return
on "Undo"; sleep 6; park
xdotool key F2; drawn_for 3 3; cp "$JA" "$T/saved-2.json"
open_item "Water heater, 50 gal" 5
on_right "Bench"; sleep 12; park
shot; keep_map 06                          # 07 the water heater, never set down, alone
on_at "Change item Vanity, 36 in" 4; sleep 12; park
keep_map 07                                # 07 another item's Change, on the bench: that item is on it
esc; sleep 8
close_window
SESSION
chmod +x "$T/session-a.sh"
timeout 2400 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-a.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: session A stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
cp "$T/map.txt.said" "$T/said-a.txt"
rm -f "$T/map.txt" "$T/map.txt.said" "$T/missing.txt"

echo
echo "== Session B: the same width typed on the panel with the house in view, no bench =="
cat > "$T/session-b.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$T/b-pictures" ICD="$ICD"
mkdir -p "\$OUT"
. "$PWD/demos/window.sh"
. "$T/common.sh"
start "$JB"
open_item "Vanity, 36 in"
on_right "Width"; sleep 3
typed "48 in"; sleep 8; park
xdotool key F2; drawn_for 3 3; cp "$JB" "$T/saved-b.json"
close_window
SESSION
chmod +x "$T/session-b.sh"
timeout 1800 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-b.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: session B stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi

echo
echo "== Pictures =="
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07; do
  [ "$OUT/$n-model.png" -nt "$T/saved-0.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/saved-0.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  14 pictures saved to $OUT (model and sheet, after each of 7 steps)"

echo
echo "== 1 and 2. The bench: alone, with its three sizes; no tool; the panel and the Items list stay =="
python3 - "$T" <<'PY'
import sys
T = sys.argv[1]
def rows(name):
    out = {}
    for line in open(f"{T}/map-{name}.txt"):
        f = line.rstrip("\n").split("\t")
        out.setdefault(f[0], []).append(f[1:])
    return out
def has(m, window, name):
    return any(r[0] == name for r in m.get(window, []))
m1, m2 = rows("01"), rows("02")
tools = ["Wall", "Door", "Window", "Duct", "Pipe", "Move"]
for t in tools:
    assert has(m1, "model", t), f"the house view should offer the {t} tool"
    assert not has(m2, "model", t), f"the bench offers the {t} tool"
print("    the tools (" + ", ".join(tools) + ") are on the map in the house and none is on the bench")
assert has(m2, "model", "Item Vanity, 36 in") and has(m2, "model", "Rename item") and has(m2, "model", "Back to the house"), "the Items list and the panel stay, with the way back"
assert not has(m2, "model", "Bench") and has(m1, "model", "Bench") and not has(m1, "model", "Back to the house")
print("    on the bench: the Items list, the item's panel and Back to the house; in the house: Bench, and no Back to the house")
for w in ("Dimension width 36 in", "Dimension depth 21 in", "Dimension height 34 1/2 in"):
    assert has(m2, "model", w), f"no {w} on the bench"
print("    three dimensions on the bench: 36 in, 21 in, 34 1/2 in")
assert "house" in m1 and "house" not in m2 and "bench" in m2 and "bench" not in m1
print("    the house's corner is on screen at the house and not on the bench")
# A number is a tool's key; T and V are plan and walk: none does anything on the bench.
mk, mt = rows("02k"), rows("02t")
for m in (mk, mt):
    assert "bench" in m and not has(m, "model", "Wall") and not has(m, "model", "Plan"), "a key took a tool, a plan or a walk on the bench"
print("    the 1, T and V keys on the bench took no tool and no view")
PY
SAID="$(cat "$T/said-a.txt")"
says "on the bench" "On the bench: Vanity, 36 in, 36 in wide (Assumed), 21 in deep (Assumed) and 34 1/2 in high (Assumed). Nothing of the house is drawn here." "$SAID"
says "the way back" "Back to the house, or Esc, goes back to where you were." "$SAID"
says "no tool by a key" "this is the bench, where nothing is drawn. Back to the house (Esc) first" "$SAID"

echo
echo "== 3. A size typed on the panel: the dimension, the words, how many follow, how much bigger it is on screen =="
says "the new size" "Vanity, 36 in: width is now 48 in (it was 36 in). Assumed: typed in the window; not yet checked against its maker's sheet. 2 things set down from it follow. To check it, type 48 in again on its width: that makes it Known, checked on site. Its name still reads 36 in: Rename item changes it." "$SAID"
python3 - "$T" <<'PY'
import sys, math
T = sys.argv[1]
def lines(name):
    out = {"bench": {}}
    for line in open(f"{T}/map-{name}.txt"):
        f = line.rstrip("\n").split("\t")
        if f[0] == "bench":
            x1, y1, x2, y2 = map(float, f[2:6]); out["bench"][f[1]] = math.hypot(x2 - x1, y2 - y1)
        elif f[0] == "model":
            out.setdefault("names", set()).add(f[1])
        elif f[0] == "house":
            out["house"] = (float(f[1]), float(f[2]))
    return out
a, b = lines("02"), lines("03")
assert "Dimension width 48 in" in b["names"] and "Dimension width 36 in" not in b["names"], "the dimension does not read 48 in"
ratio = b["bench"]["width"] / a["bench"]["width"]
assert 1.28 <= ratio <= 1.39, f"the width on screen grew by {ratio:.3f}, to be 48/36 = 1.333"
for what in ("depth", "height"):
    assert abs(b["bench"][what] - a["bench"][what]) <= 2.0, f"the {what} on screen changed: {a['bench'][what]:.1f} to {b['bench'][what]:.1f} dots"
print(f"    the dimension reads 48 in; the width on screen is {ratio:.3f} times what it was (48/36 = 1.333); depth {a['bench']['depth']:.1f} to {b['bench']['depth']:.1f} dots, height {a['bench']['height']:.1f} to {b['bench']['height']:.1f} dots")
# Esc with a value half typed lets the value go and the bench stays.
e = lines("02e")
assert e["bench"] and "house" not in e, "Esc on a half-typed value left the bench"
# The wheel and a right-drag work on the bench as on the house: the item is bigger on screen, then seen from another side.
w, d = lines("03w"), lines("03d")
assert w["bench"]["height"] > b["bench"]["height"] * 1.05, f"the wheel did not close on the item: {b['bench']['height']:.1f} to {w['bench']['height']:.1f} dots"
turned = abs(d["bench"]["width"] / d["bench"]["depth"] - w["bench"]["width"] / w["bench"]["depth"])
assert turned > 0.03 and "house" not in d, f"the drag did not turn the view (width to depth {w['bench']['width'] / w['bench']['depth']:.3f} to {d['bench']['width'] / d['bench']['depth']:.3f})"
print(f"    Esc on a half-typed value kept the bench; the wheel took the height on screen from {b['bench']['height']:.1f} to {w['bench']['height']:.1f} dots; a right-drag turned the view (width to depth {w['bench']['width'] / w['bench']['depth']:.2f} to {d['bench']['width'] / d['bench']['depth']:.2f})")
PY
says "Esc on a half-typed value" "Cancelled." "$SAID"

echo
echo "== 4 and 5. Back to the house, by the button and by Esc: the view that was left =="
python3 - "$T" <<'PY'
import sys
T = sys.argv[1]
def house(name):
    for line in open(f"{T}/map-{name}.txt"):
        f = line.rstrip("\n").split("\t")
        if f[0] == "house":
            return float(f[1]), float(f[2])
    raise SystemExit(f"FAIL: the house is not on screen in map {name}")
def names(name):
    return {l.split("\t")[1] for l in open(f"{T}/map-{name}.txt") if l.startswith("model\t")}
h1 = house("01")
for n in ("04", "05"):
    h = house(n)
    assert abs(h[0] - h1[0]) <= 2 and abs(h[1] - h1[1]) <= 2, f"the house corner is at {h} after the bench ({n}), was {h1}"
    assert "Bench" in names(n) and "Back to the house" not in names(n), "the panel did not go back to Bench"
    assert not any(l.startswith("bench\t") for l in open(f"{T}/map-{n}.txt")), "still on the bench"
print(f"    a corner of the house is within 2 dots of where it was ({h1}) after the button and after Esc; the item's panel is still open")
PY
python3 - "$T" "$OUT" <<'PY'
import sys
from PIL import Image, ImageChops
import numpy as np
T, OUT = sys.argv[1], sys.argv[2]
a = Image.open(f"{OUT}/01-model.png").convert("RGB"); b = Image.open(f"{OUT}/05-model.png").convert("RGB")
w, h = a.size
box = (int(w * 0.18), int(h * 0.28), int(w * 0.76), int(h * 0.80))
d = np.asarray(ImageChops.difference(a.crop(box), b.crop(box))).astype(float)
print(f"    the house before and after, over the middle of the model window: mean difference {d.mean():.2f} of 255, {100.0 * (d.max(axis=2) > 24).mean():.2f} % of dots differ by more than 24")
assert d.mean() < 3.0, "the house is not as it was after the bench"
PY
SAIDN="$(grep -c 'Back to the house\.' "$T/said-a.txt" || true)"
[ "$SAIDN" -ge 2 ] || { echo "FAIL: 'Back to the house.' was said $SAIDN times, to be at least 2 (button and Esc)"; exit 1; }
echo "    said: Back to the house. ($SAIDN times)"

echo
echo "== 6. Undo after coming back =="
python3 - "$T" "$P" <<'PY'
import json, sys, subprocess
T, P = sys.argv[1], sys.argv[2]
def facts(path):
    d = json.load(open(path))["data"]
    placed = [e["attributes"]["pl::placed"] for e in d if "pl::placed" in e["attributes"] and str(e["attributes"]["pl::placed"]["what"]["value"]).startswith("Vanity")]
    return None, placed
for name, want in (("saved-1", 4.0), ("saved-2", 3.0)):
    item, placed = facts(f"{T}/{name}.json")
    sizes = [round(p["size"]["value"][0], 4) for p in placed]
    assert sizes == [want, want], f"{name}: set down widths {sizes}, to be {want} ft each"
    print(f"    {name}: the two set down are {want * 12:.0f} in wide")
PY

echo
echo "== 7. The water heater, never set down =="
python3 - "$T" <<'PY'
import sys
T = sys.argv[1]
names = {l.split("\t")[1] for l in open(f"{T}/map-06.txt") if l.startswith("model\t")}
for w in ("Dimension width 22 in", "Dimension depth 22 in", "Dimension height 60 in", "Back to the house"):
    assert w in names, f"the heater's bench has no {w}"
print("    the water heater is on the bench: 22 in, 22 in, 60 in")
names7 = {l.split("\t")[1] for l in open(f"{T}/map-07.txt") if l.startswith("model\t")}
assert any(l.startswith("bench\t") for l in open(f"{T}/map-07.txt")) and "Dimension width 36 in" in names7, "another item's Change on the bench did not put it on the bench"
print("    another item's Change, on the bench, put that item on the bench")
PY
says "the heater" "On the bench: Water heater, 50 gal, 22 in wide (Assumed), 22 in deep (Assumed) and 60 in high (Assumed)." "$SAID"

echo
echo "== 8. The saved job: what the bench left, and the same without it =="
"$P" check "$T/saved-1.json"
LIST="$("$P" items "$T/saved-1.json")"
says "the vanity, changed" "~48 in wide, ~21 in deep, ~34 1/2 in high; 2 placed" "$LIST"
python3 - "$T" <<'PY'
import json, sys
T = sys.argv[1]
d = json.load(open(f"{T}/saved-1.json"))["data"]
item = next(it for it in (e["attributes"].get("pl::item") for e in d) if it and it["width"]["source"].startswith("typed in the window"))
w = item["width"]
assert w["status"] == "Assumed" and w["source"] == "typed in the window; not yet checked against its maker's sheet", w
assert item["depth"]["status"] == "Assumed" and item["height"]["status"] == "Assumed"
for e in d:
    p = e["attributes"].get("pl::placed")
    if p and str(p["what"]["value"]).startswith("Vanity"):
        assert p["size"]["status"] == "Assumed" and p["size"]["value"][0] == 4.0, p["size"]
print("    in the file: the item's width and the two set down, Assumed, 'typed in the window; not yet checked against its maker's sheet'; depth and height as they were")
PY
BEFORE="$("$P" price "$T/saved-0.json")"; AFTER="$("$P" price "$T/saved-1.json")"
[ "$BEFORE" = "$AFTER" ] || { echo "FAIL: the price changed:"; diff <(echo "$BEFORE") <(echo "$AFTER"); exit 1; }
echo "    the price is the figure it was: $(grep -m1 '\$' <<<"$AFTER" | sed -E 's/^ +//') (the whole of the price output is the same, line for line)"
cmp "$T/saved-1.json" "$T/saved-b.json" && echo "    the job saved after the bench is the job saved after the same typing without it, to the byte"
echo "  PASS: an item was put on the bench alone with its three sizes, no tool was offered, a typed size changed it and said how many follow, the house came back as it was by the button and by Esc, Undo took the size back, the water heater went on the bench, and the saved job is the same as without it"
