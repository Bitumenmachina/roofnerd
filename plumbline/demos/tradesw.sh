#!/usr/bin/env bash
# The layers of the house a trade at a time, in the window: the finish taken off to the
# weather barrier and the flashing, the walls drawn as their frames, the ground taken
# away from the foundation, and one level looked down into. The made-up whole house,
# each layer switched with one click under Show or Levels, and each picture measured.
# Then, call 177: a trade's view of the building. The same house with a beam, two posts, two
# pad footings, joists named on its floors and rafters on its roof (made at the command line
# and opened cold), and each trade's button pressed: the switches it set read off the program's
# map, the sentence the status line gave, the tags on the work found by their words, and the
# picture measured. Pictures 06 to 21 are that part.
# Run from the repository root:  demos/tradesw.sh [folder for the pictures]
#
# It drives the real program with a real pointer on a virtual display (Xvfb, xdotool and
# a Vulkan driver; Mesa's software one will do). The job is made up. What each picture
# must show, and the measurement that fails on the picture before the switch, is in
# STATUS.md ("The layers, switched in the window").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/tradesw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/tradesw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/layers}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/house.json"

echo "== The made-up whole house, priced in all sixteen divisions =="
"$P" sixteen "$J" >/dev/null
"$P" estimate "$J" | grep -F "PRICE" > "$T/price-before.txt"
sed 's/^/    /' "$T/price-before.txt"

# The house of call 177: the same house with what carries it, and its joists and rafters named.
# Beam 8,10 to 32,10 on two posts, each on a pad footing; the second floor's joists the short way
# (said, Assumed); the first floor's named and its direction not said; rafters on the roof.
J2="$T/house2.json"; cp "$J" "$J2"
K=(--status Known --source "made-up plan")
"$P" type "$J2" beam "LVL 3 1/2 x 11 7/8" --width "3 1/2 in" --depth "11 7/8 in" "${K[@]}" >/dev/null
"$P" type "$J2" post "6x6 post" --width "5 1/2 in" --depth "5 1/2 in" "${K[@]}" >/dev/null
"$P" type "$J2" pad "Pad 36 x 36" --width "36 in" --depth "36 in" "${K[@]}" >/dev/null
tpath() { "$P" show "$J2" | awk -v n="$1" '$0 ~ "  "n"  \\[type\\]" {print $1; exit}'; }
LEVEL="$("$P" show "$J2" | awk '/\[level\]/ && /first floor/ {print $1; exit}')"
"$P" beam "$J2" "$LEVEL" "$(tpath "LVL 3 1/2 x 11 7/8")" --points "8,10 32,10" --top "9 ft" "${K[@]}" >/dev/null
for x in 8 32; do
  "$P" post "$J2" "$LEVEL" "$(tpath "6x6 post")" --at "$x,10" --height "8 ft" "${K[@]}" >/dev/null
  "$P" pad "$J2" "$LEVEL" "$(tpath "Pad 36 x 36")" --at "$x,10" --thickness "18 in" --top "-1 ft" "${K[@]}" >/dev/null
done
FLOORS=($("$P" show "$J2" | awk '/\[floor\]/ {print $1}'))
for f in "${FLOORS[@]}"; do "$P" line "$J2" "$f" "Floor joists, 2x10 at 16 in" joists --cost-type material >/dev/null; done
"$P" set "$J2" "${FLOORS[1]}" joists_span "short way" --status Assumed --source "usual, made up" >/dev/null
ROOF="$("$P" show "$J2" | awk '/\[roof\]/ {print $1; exit}')"
"$P" line "$J2" "$ROOF" "Rafters, 2x8 at 24 in" rafters --cost-type material >/dev/null
"$P" estimate "$J2" | grep -F "PRICE" > "$T/price2-before.txt"
sed 's/^/    /' "$T/price2-before.txt"

echo
echo "== The session: each layer with one click =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
park
shot                                    # 01 as it opens: dressed in what is priced
on "Show Finishes"; sleep 6; park
shot                                    # 02 the finish off: the weather barrier, and flashing at the openings
on "Show Finishes"; on "Show Framing"; sleep 6; park
shot                                    # 03 the walls as their frames
on "Show Framing"; on "Show Ground"; sleep 6; park
shot                                    # 04 the ground away: the foundation under the house
on "Show Ground"; on "Level first floor"; sleep 10; park
shot                                    # 05 the first floor alone, looked down into
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
[ ! -e "$T/missing.txt" ] || { echo "FAIL: a switch was not on screen"; cat "$T/missing.txt"; exit 1; }
for n in 01 02 03 04 05; do [ -s "$OUT/$n-model.png" ] || { echo "FAIL: no picture $n"; exit 1; }; done
echo "  5 pictures of the model saved under $OUT (and the sheet beside each)"

echo
echo "== A trade at a time: the buttons at the head of Show, from a house opened cold =="
OUT2="$T/p2"; mkdir -p "$OUT2"
cat > "$T/session2.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT2" ICD="$ICD"
. "$PWD/demos/window.sh"
# What a switch of the Show list reads, from the program's own map; a failure is written down
# and the run goes on, so that one run says everything that is wrong.
st() { awk -F'\t' -v n="\$1" '\$1=="state" && \$2==n {print \$3}' "\$PLUMBLINE_MAP"; }
expect() { local got; got="\$(st "\$1")"; echo "\$STEP: \$1 reads \$got" >> "\$T/states.txt"; [ "\$got" = "\$2" ] || echo "after \$STEP: \$1 reads \$got, wanted \$2" >> "\$T/fails.txt"; }
said_is() { local last; last="\$(tail -1 "\$PLUMBLINE_MAP.said")"; echo "\$STEP said: \$last" >> "\$T/states.txt"; [ "\$last" = "\$1" ] || echo "after \$STEP the status line said: \$last (wanted: \$1)" >> "\$T/fails.txt"; }
tags() { grep -P '^model\tTag ' "\$PLUMBLINE_MAP" > "\$T/tags-\$STEP.txt" || : > "\$T/tags-\$STEP.txt"; }
# (call 232, point 5(b): each tag's box is kept too, for the count of wires drawn to leave out)
boxes() { grep -P '^size\tmodel\tTag ' "\$PLUMBLINE_MAP" > "\$T/boxes-\$STEP.txt" || : > "\$T/boxes-\$STEP.txt"; }
trade() { STEP="\$1"; on "\$1"; sleep 9; park; sleep 3; }
open_window "$J2"
park
STEP=opened; grep -P '^state' "\$PLUMBLINE_MAP" > "\$T/state-opened.txt"; tags
shot                                    # 01 as it opens
trade Plumbing
for w in "Ground on" "Finishes on" "Framing off" "Roof on" "Mechanical off" "Electrical off" "Plumbing on" "Sizes on" "Level all"; do expect \${w% *} \${w##* }; done
expect "See through" on
said_is "Plumbing: the rest of the house shown as glass, sizes on."
tags; boxes
shot                                    # 02 plumbing: the rest as glass, pipes tagged
trade HVAC
for w in "Mechanical on" "Electrical off" "Plumbing off" "Sizes on"; do expect \${w% *} \${w##* }; done
expect "See through" on
said_is "HVAC: the rest of the house shown as glass, sizes on."
tags; boxes
shot                                    # 03 HVAC
trade Electrical
for w in "Mechanical off" "Electrical on" "Plumbing off" "Sizes on"; do expect \${w% *} \${w##* }; done
said_is "Electrical: the rest of the house shown as glass, sizes on."
tags; boxes
shot                                    # 04 electrical
trade Framing
for w in "Framing on" "Finishes off" "Roof on" "Mechanical off" "Electrical off" "Plumbing off" "Sizes on" "Ground on"; do expect \${w% *} \${w##* }; done
expect "See through" on
said_is "Framing: the walls as their frames, with the joists, rafters, beams and posts, the rest of the house shown as glass, sizes on."
tags; boxes
shot                                    # 05 framing: joists, rafters, beam and posts
trade Foundation
for w in "Ground off" "Framing off" "Sizes on" "Mechanical off"; do expect \${w% *} \${w##* }; done
expect "See through" on
said_is "Foundation: the ground taken away, the footings and foundation walls solid and the house above them shown as glass, sizes on."
tags; boxes
shot                                    # 06 foundation
trade Outside
for w in "Ground on" "Finishes on" "Framing off" "Roof on" "Sizes off" "Mechanical on" "Plumbing on"; do expect \${w% *} \${w##* }; done
expect "See through" off
said_is "Outside: the house as it will be built."
tags; boxes
shot                                    # 07 outside
trade Inside
for w in "Roof off" "Finishes on" "Sizes off"; do expect \${w% *} \${w##* }; done
expect "Level" "first floor"
expect "See through" off
said_is "Inside: the roof off and the level looked into, with its finishes."
tags; boxes
shot                                    # 08 inside
trade All
STEP=All; grep -P '^state' "\$PLUMBLINE_MAP" > "\$T/state-all.txt"
said_is "All: every switch as it was before a trade was pressed."
shot                                    # 09 all: as it was
# Sizes by itself, over the house as it will be built: the work inside is not tagged, and it says so.
STEP=sizes-alone; on "Show Sizes"; sleep 5
said_is "Sizes: switched on. The work inside is tagged when the house is seen through (X) or its frame is shown."
tags; boxes
on "Show Sizes"; sleep 3
# A far view of the plumbing: a run is never thinner than two dots.
trade Plumbing
STEP=far
for i in 1 2 3 4 5; do xdotool mousemove 800 450; sleep 0.4; xdotool click 5; sleep 1.2; done
sleep 8; park
shot                                    # 10 far off: the runs are as wide as two dots
trade All
on "Frame"; sleep 10                    # the view comes back to the house
# Each clash says its room, on the model and on the sheet.
STEP=clashes; on "Show Clashes"; sleep 8; park
tags; boxes
shot                                    # 11 the clashes marked, each saying its room
sheet "Clashes"; sleep 6; park
shot                                    # 12 the sheet's list of clashes: room first
# The tables further down the sheet are read as the wheel brings them on screen.
: > "\$T/counts.txt"
for i in 1 2 3 4 5 6 7 8; do
  grep -P '^(sheet|away)\tCount of' "\$PLUMBLINE_MAP" >> "\$T/counts.txt" || true
  xdotool mousemove 2000 600; sleep 0.5; for j in 1 2 3 4 5 6; do xdotool click 5; sleep 1.5; done; sleep 3
done
close_window
SESSION
chmod +x "$T/session2.sh"
rm -f "$T/fails.txt" "$T/states.txt"
timeout 1800 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session2.sh" || true
[ ! -e "$T/missing.txt" ] || { echo "FAIL: a control was not on screen"; cat "$T/missing.txt"; exit 1; }
for n in 01 02 03 04 05 06 07 08 09 10 11 12; do [ -s "$OUT2/$n-model.png" ] || { echo "FAIL: no picture $n of the trades"; exit 1; }; done
for f in "$OUT2"/*.png; do b="$(basename "$f")"; n="${b%%-*}"; cp "$f" "$OUT/$(printf %02d $((10#$n + 5)))-${b#*-}"; done
sed 's/^/    /' "$T/states.txt"
if [ -s "$T/fails.txt" ]; then echo "FAIL: what the buttons set:"; sed 's/^/    /' "$T/fails.txt"; exit 1; fi
# "All" puts every switch back as it was before the first trade was pressed.
cmp -s "$T/state-opened.txt" "$T/state-all.txt" || { echo "FAIL: All did not put the switches back"; diff "$T/state-opened.txt" "$T/state-all.txt"; exit 1; }
echo "  12 more pictures saved under $OUT (06 to 17); the buttons set what they say, and All put every switch back"

echo
echo "== What the pictures hold, measured =="
python3 - "$OUT" "$T" <<'PY'
import sys
from PIL import Image
out = sys.argv[1]
td = sys.argv[2]
def pixels(n, box):
    im = Image.open(f"{out}/{n:02d}-model.png").convert("RGB").crop(box)
    return [im.getpixel((x, y)) for x in range(im.width) for y in range(im.height)]
def mean(n, box):
    px = pixels(n, box)
    return tuple(round(sum(p[i] for p in px) / len(px)) for i in range(3))
ok = True
def check(what, got, good, want):
    global ok
    ok = ok and good
    print("    %-66s %s  %s" % (what, got, "as wanted" if good else "FAIL: wanted " + want))
# Call 184, step 2 (the sun moved): the house's shadow lay to the right of it, at (1140, 550) to
# (1170, 570), under the old fixed sun; the sun is now where it would be at ten in the morning in
# April and the shadow lies to the left, so the place sampled is (390, 690) to (420, 710). On 01
# it reads (141, 144, 146), on 04 with the ground away (186, 198, 211): the same test, the
# shadow's place moved with the sun.
WALL, ROOF, SHADOW, FRAME = (860, 540, 900, 580), (880, 290, 920, 310), (390, 690, 420, 710), (700, 480, 1100, 600)
cedar = lambda c: c[0] - c[2] > 24                      # warm: the siding
wrapped = lambda c: c[2] >= c[0] and c[0] > 130         # pale and cool: the weather barrier
# Call 184, step 2: in the new light the siding is lit to (169, 130, 95), which the old test
# (red over 150, well over blue, red over green over blue) took for lumber: 0.75 of the wall on 01
# against 0.10 allowed. Lumber is yellower than cedar: red is under 28 over green, where the
# siding's is 39 (the lumber of 03 reads (163, 146, 120), red over green by 17). The test is
# narrowed, not loosened; on 03 it still finds 0.58 of the wall, and on 10 the roof 0.73 and the
# floor 1.00 as before.
lumber = lambda p: p[0] > 150 and p[0] - p[2] > 35 and p[0] > p[1] > p[2] and p[0] - p[1] < 28
framed = lambda n: sum(1 for p in pixels(n, FRAME) if lumber(p)) / len(pixels(n, FRAME))
# 01, as it opens: cedar on the wall, the roof on, the house's shadow on the ground.
w1, r1, s1, f1 = mean(1, WALL), mean(1, ROOF), mean(1, SHADOW), framed(1)
check("01 as opened: the south wall is its siding", w1, cedar(w1), "red well over blue (cedar)")
# 02, Finishes off: the same place is the weather barrier; the test fails on 01.
w2 = mean(2, WALL)
check("02 the finish off: the same place is the weather barrier", w2, wrapped(w2), "pale and cool")
check("   the barrier test on 01 (the siding) fails", w1, not wrapped(w1), "the siding not to pass for a barrier")
check("   the siding test on 02 fails", w2, not cedar(w2), "the barrier not to pass for siding")
# 03, Framing on: studs with the house seen between them: between a third and
# nine-tenths of the wall is lumber. On 01 none of it is.
f3 = framed(3)
check("03 the frame: lumber over the walls, with gaps between", round(f3, 2), 0.30 < f3 < 0.90, "between 0.30 and 0.90 lumber")
check("   on 01 none of the wall is lumber", round(f1, 2), f1 < 0.10, "under 0.10")
# 04, Ground off: the house's shadow has nothing to fall on.
s4 = mean(4, SHADOW)
check("04 the ground away: no shadow where the shadow fell", s4, s4[0] > 170, "the backdrop, lighter than 170")
check("   on 01 the shadow is there", s1, s1[0] < 160, "the shadow, darker than 160")
# 04 again: under the house there is a foundation to see. Below the stone the south
# foundation wall is cast concrete: one flat gray, where on 01 the same place is the
# stone above the ground, all joints.
FOUND = (720, 670, 800, 730)
def spread(n, box):
    g = [sum(p) / 3 for p in pixels(n, box)]
    m = sum(g) / len(g)
    return round((sum((x - m) ** 2 for x in g) / len(g)) ** 0.5, 1)
c4, d4, d1 = mean(4, FOUND), spread(4, FOUND), spread(1, FOUND)
check("04 the ground away: the foundation wall is concrete, gray", c4, max(c4) - min(c4) < 10 and 100 < c4[0] < 170, "a gray between 100 and 170")
check("   and flat, as cast (how far its shade spreads)", d4, d4 < 8, "under 8")
check("   on 01 the same place is stone, all joints", d1, d1 > 15, "over 15")
# 05, the first floor alone: where the roof was there is none.
r5 = mean(5, ROOF)
check("05 the first floor alone: no roof where the roof was", r5, r5[0] > 150, "lighter than 150")
check("   on 01 the roof is there", r1, r1[0] < 120, "the roofing, darker than 120")

# ---- Call 177: a trade's view ----
import colorsys
def hsv(p):
    h, s, v = colorsys.rgb_to_hsv(*[x / 255 for x in p]); return h * 360, s, v
HOUSE = (400, 300, 1180, 780)
def tag_boxes(step):
    """Each tag's box on the picture, from the map: its place (the middle) and its size."""
    try:
        rows = [l.rstrip("\n").split("\t") for l in open(f"{td}/boxes-{step}.txt")]
        at = {r[1][4:]: (int(r[2]), int(r[3])) for r in (l.rstrip("\n").split("\t") for l in open(f"{td}/tags-{step}.txt")) if len(r) == 4}
    except OSError:
        return []
    out = []
    for r in rows:
        if len(r) >= 5 and r[2][4:] in at:
            x, y = at[r[2][4:]]
            out.append((x - int(r[3]) / 2 - 2, y - int(r[4]) / 2 - 2, x + int(r[3]) / 2 + 2, y + int(r[4]) / 2 + 2))
    return out
def kinds(n, step=None, masked=True):
    """Pipe (cold water, vent: cyan and green), duct (blue) and wire (orange) pixels in the house's box,
    the pale ones: the tags' own dots are saturated and are left out."""
    c = {"pipe": 0, "duct": 0, "wire": 0}
    # call 232, point 5(b): the count is of wires drawn; a tag's own swatch is not a wire, so the tags' boxes are left out
    skip = tag_boxes(step) if step and masked else []
    im = Image.open(f"{out}/{n:02d}-model.png").convert("RGB")
    for xx in range(HOUSE[0], HOUSE[2]):
      for yy in range(HOUSE[1], HOUSE[3]):
        if any(b[0] <= xx <= b[2] and b[1] <= yy <= b[3] for b in skip):
            continue
        p = im.getpixel((xx, yy))
        h, s, v = hsv(p)
        if not (0.30 < s < 0.85 and v > 0.35):
            continue
        if 60 <= h < 200: c["pipe"] += 1
        elif 200 <= h < 260: c["duct"] += 1
        elif 15 <= h < 60: c["wire"] += 1
    return c
pl, hv, el = kinds(7, "Plumbing"), kinds(8, "HVAC"), kinds(9)
print("    (call 232, point 5(b) the wire-coloured dots: plumbing %d with the tags' boxes left out, %d without; HVAC %d, %d; as opened, no tags: %d, %d)" % (pl["wire"], kinds(7, "Plumbing", False)["wire"], hv["wire"], kinds(8, "HVAC", False)["wire"], kinds(1, "opened")["wire"], kinds(1, "opened", False)["wire"]))
def gone(c, ref): return c < 0.1 * ref and c < 100
check("07 plumbing: pipe colour present", pl["pipe"], pl["pipe"] >= 150, "at least 150 pipe pixels")
check("   and no duct colour", pl["duct"], gone(pl["duct"], hv["duct"]), "under a tenth of the HVAC picture's, %d" % hv["duct"])
check("   and no wire colour", pl["wire"], gone(pl["wire"], el["wire"]), "under a tenth of the electrical picture's, %d" % el["wire"])
check("08 HVAC: duct colour present", hv["duct"], hv["duct"] >= 1000, "at least 1000 duct pixels")
check("   and no pipe colour", hv["pipe"], gone(hv["pipe"], pl["pipe"]), "under a tenth of the plumbing picture's, %d" % pl["pipe"])
check("   and no wire colour", hv["wire"], gone(hv["wire"], el["wire"]), "under a tenth of the electrical picture's")
check("09 electrical: wire colour present", el["wire"], el["wire"] >= 500, "at least 500 wire pixels")
check("   and no pipe colour", el["pipe"], gone(el["pipe"], pl["pipe"]), "under a tenth of the plumbing picture's")
check("   and no duct colour", el["duct"], gone(el["duct"], hv["duct"]), "under a tenth of the HVAC picture's")

# The tags, found on the program's map by their words, inside the window, and written on the picture:
# ink where the tag is, on the picture after and not on the picture before (06, as it opened).
def tags(step):
    try:
        rows = [l.rstrip("\n").split("\t") for l in open(f"{td}/tags-{step}.txt")]
    except OSError:
        rows = []
    return [(r[1][4:], int(r[2]), int(r[3])) for r in rows if len(r) == 4]
def pill(n, at, half=(50, 6)):
    """How much of the box round a tag's place is the tag's pale rounded fill."""
    x, y = at
    px = pixels(n, (max(x - half[0], 0), max(y - half[1], 0), x + half[0], y + half[1]))
    return round(sum(1 for p in px if min(p) >= 222 and max(p) - min(p) <= 16) / len(px), 2)
def wears(step, n, words, floor=0.45):
    found = [t for t in tags(step) if t[0] == words]
    if not found:
        check(f"   the tag \"{words}\" is on the map", "none", False, "one")
        return
    w, x, y = found[0]
    inside = 270 < x < 1230 and 110 < y < 800
    after, before = pill(n, (x, y)), pill(6, (x, y))
    check(f"   the tag \"{words}\" is inside the window and written there", (x, y, after, before), inside and after >= floor and before < 0.10, f"inside, its fill over at least {floor} of the place here and under 0.10 of it on the opened picture")
wears("Plumbing", 7, "3 in waste stack")
wears("Plumbing", 7, "3/4 in cold water")
wears("HVAC", 8, "16 x 8 supply")
wears("Electrical", 9, "0.4 in power")
wears("Foundation", 11, "footing 24 in x 12 in")
wears("Foundation", 11, "Pad 36 x 36")
wears("Framing", 10, "LVL 3 1/2 x 11 7/8")
wears("Framing", 10, "joists: direction not said")
def apart(step):
    t = tags(step)
    bad = [(a[0], b[0]) for i, a in enumerate(t) for b in t[i + 1:] if abs(a[1] - b[1]) < 45 and abs(a[2] - b[2]) < 9]
    check(f"   {step}: no tag lies on another", len(bad), not bad, "none piled on one another")
for step in ("Plumbing", "HVAC", "Electrical", "Framing", "Foundation"):
    apart(step)
check("   as it opens there are no tags", len(tags("opened")), len(tags("opened")) == 0, "none")
check("   the house as it will be built wears none (Sizes is off)", len(tags("Outside")), len(tags("Outside")) == 0, "none")
check("   Sizes alone over the solid house tags nothing inside it", len(tags("sizes-alone")), len(tags("sizes-alone")) == 0, "none")
# Of the pieces of a run end to end only the longest wears the tag: the map has fewer "3/4 in cold water" than there are pieces.
cold = [t for t in tags("Plumbing") if t[0] == "3/4 in cold water"]
check("   one tag for the pieces of a run end to end (3/4 in cold water)", len(cold), 1 <= len(cold) <= 2, "one or two, where its pieces are many")

# The frame drawn. Lumber-coloured ink over the roof where the rafters are, and across the floor where
# the joists are. 03 is the walls as their frames on this house with no rafters or joists named (the
# first part of this run): its roof is the roofing, and what it has across the floor is studs.
ROOFBOX, FLOORBOX = (560, 300, 1100, 380), (640, 455, 1000, 500)
def lum(n, box):
    px = pixels(n, box); return round(sum(1 for p in px if lumber(p)) / len(px), 3)
r10, r3, f10, f3b = lum(10, ROOFBOX), lum(3, ROOFBOX), lum(10, FLOORBOX), lum(3, FLOORBOX)
check("10 the frame: lumber across the roof where the rafters are", (r10, r3), r10 > 0.40 and r3 < 0.05, "over 0.40 here, under 0.05 on 03 (no rafters drawn)")
check("   and across the floor where the joists are", (f10, f3b), f10 > 0.90 and f3b < 0.60, "over 0.90 here, under 0.60 on 03 (studs only)")

# A run is never thinner than two dots: the vent (2 in, green) seen from far off, row by row.
def vent_widths(n, box=(730, 392, 820, 414)):
    im = Image.open(f"{out}/{n:02d}-model.png").convert("RGB")
    widths = []
    for y in range(box[1], box[3]):
        w = sum(1 for x in range(box[0], box[2]) if (lambda h, s, v: 60 <= h <= 130 and s > 0.3 and v > 0.35)(*hsv(im.getpixel((x, y)))))
        if w:
            widths.append(w)
    # Not the two rows at the ends of the run, where a line's end is only part of a dot.
    return widths[1:-1]
vw = vent_widths(15)
check("15 far off: a 2 in vent is at least two dots wide on every row", (min(vw) if vw else 0, len(vw)), len(vw) >= 6 and min(vw) >= 2, "at least 6 rows, none under 2 dots")

# A clash says its room: tags on its marks.
cl = [t for t in tags("clashes") if t[0].startswith("clash ")]
import re
cl = [re.sub(r" #\d+$", "", t[0]) for t in cl]
check("16 the clashes' marks each say a room (or that it is outside any)", cl[:4], len(cl) >= 2 and all(t.startswith("clash in ") or t == "clash outside any room" for t in cl), "at least two, each \"in <room>\" or \"outside any room\"")
sys.exit(0 if ok else 1)
PY

echo
echo "== The window's clash counts are the model's (calls 175, 177, 179 C5): in the same space, set against, holes, laid in =="
# The tables' counts as the sheet window says them, from the program's map, against what the
# command line's `clashes` says of the same house: each table's first number.
"$P" clashes "$J2" > "$T/clashes.txt"
window_count() { { grep -F "Count of $1" "$T/counts.txt" || true; } | head -1 | sed -E "s/.*Count of $1: ([0-9]+).*/\\1/"; }
model_n() { grep -E "^$1" "$T/clashes.txt" | head -1 | grep -oE '\): [0-9]+|: [0-9]+' | tail -1 | grep -oE '[0-9]+'; }
for pair in "In the same space|In the same space" "Set against: not clashes|Set against" "Through a wall or a floor: holes to cut|Through a wall or a floor" "Laid in a floor or a wall: no hole to cut|Laid in a floor or a wall"; do
  win="${pair%%|*}"; mod="${pair##*|}"
  w="$(window_count "$win")"; m="$(model_n "$mod")"
  [ -n "$m" ] || { echo "FAIL: the command line gives no count for: $mod"; cat "$T/clashes.txt"; exit 1; }
  # A table with nothing in it is not drawn: the model's 0 is the window's none.
  [ "${w:-0}" = "$m" ] || { echo "FAIL: $win: the window says ${w:-nothing}, the model says $m"; cat "$T/counts.txt"; exit 1; }
  echo "    $win: $m in the window and in the model"
done

echo
echo "== A switch changes what is drawn and no figure =="
"$P" estimate "$J" | grep -F "PRICE" > "$T/price-after.txt"
cmp -s "$T/price-before.txt" "$T/price-after.txt" || { echo "FAIL: the price changed"; exit 1; }
sed 's/^/    /' "$T/price-after.txt"
"$P" estimate "$J2" | grep -F "PRICE" > "$T/price2-after.txt"
cmp -s "$T/price2-before.txt" "$T/price2-after.txt" || { echo "FAIL: the price of the house with its frame changed"; exit 1; }
sed 's/^/    /' "$T/price2-after.txt"

echo
echo "PASS: with one click each, the finish came off to the weather barrier, the walls were drawn as their frames, the ground was taken from the foundation, and the first floor was looked down into; each measured, and each measurement fails on the picture before its switch. And each trade's button set the switches it says, wrote its tags, and drew its work; All put them back"
echo "The pictures are the evidence: read them."
