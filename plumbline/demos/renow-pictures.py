# The pictures of demos/renow.sh, measured (call 178 item 2). Run alone against a folder of
# pictures to prove a check red:  python3 -I demos/renow-pictures.py <folder> <px.txt>
# Every figure below is measured on the picture the window saved; none is read from the model.
#   Existing reads as existing, in 3D: over the south wall's patch the colour's strength (largest
#   channel less smallest, per pixel, averaged) falls by over a third from unmarked (01) to
#   existing (10) and, as a stricter test than call 178's own, by over 45 percent (on the
#   picture of call 169's wash, 0.15 of a pale light, it fell 40 percent and the wall was not told
#   from new work by eye); the spread of brightness stays over 10 (the brick is still brick).
#   With the switch off (11) the wall is as new work is: strength within 15 percent of 01.
#   On the plan (07): the existing south wall's fill is lighter than a new wall's (the north
#   one) by over 60 of 255, and the partition that comes down has a red dashed outline (over
#   40 red pixels) and no fill (its middle is paper).
import sys, os
from PIL import Image
out = sys.argv[1]
v = [float(x) for x in open(sys.argv[2]).read().split()]
def crop(name, box):
    return Image.open(f"{out}/{name}-model.png").convert("RGB").crop(tuple(int(round(b)) for b in box))
def px(name, box): return list(crop(name, box).getdata())
def strength(name, box):
    p = px(name, box); return sum(max(q) - min(q) for q in p) / len(p)
def spread(name, box):
    l = [(q[0] * 299 + q[1] * 587 + q[2] * 114) / 1000 for q in px(name, box)]
    m = sum(l) / len(l); return (sum((x - m) ** 2 for x in l) / len(l)) ** 0.5
def lum(name, box):
    l = [(q[0] * 299 + q[1] * 587 + q[2] * 114) / 1000 for q in px(name, box)]; return sum(l) / len(l)
def red(name, box): return sum(1 for q in px(name, box) if q[0] - q[2] > 80)
ok = True
def say(good, text):
    global ok
    ok = ok and good
    print("    %s: %s" % (text, "as wanted" if good else "FAIL"))
south3d = (800, 500, 1100, 650)
a, b = strength("01", south3d), strength("10", south3d)
say(b < a * 2 / 3, "south wall in 3D, colour strength unmarked (01) %.1f, existing (10) %.1f: falls over a third (%.0f percent)" % (a, b, 100 * (1 - b / a)))
say(b < a * 0.55, "...and over 45 percent (call 178 stricter test)")
s_ex = spread("10", south3d)
say(s_ex > 10, "existing brick keeps its pattern: spread of brightness %.2f (unmarked %.2f), wanted over 10" % (s_ex, spread("01", south3d)))
if os.path.exists(f"{out}/11-model.png"):
    c = strength("11", south3d)
    say(abs(c - a) < 0.15 * a, "switch off (11): strength %.1f against unmarked %.1f, the wall as new work is" % (c, a))
(qx0, qy0), (qx1, qy1), k = (v[8], v[9]), (v[10], v[11]), v[14]
sx0, sy, sx1 = v[4], v[3], v[6]
nx, ny = v[15], v[16]
south = (sx0, sy - 5, sx1, sy + 5); north = (sx0, ny - 5, sx1, ny + 5)
ls, ln = lum("07", south), lum("07", north)
say(ls > ln + 60, "plan (07): existing south wall fill lum %.0f, new north wall fill lum %.0f: existing lighter by over 60" % (ls, ln))
part = (qx0 - k * 0.32, qy1, qx0 + k * 0.32, qy0)
r4, r8, r9, r2 = red("04", part), red("08", part), red("09", part), red("02", part)
say(r4 > 40 and r2 < 5, "partition on the plan: red pixels as opened (02) %d, comes down (04) %d: a red dashed outline" % (r2, r4))
say(r8 < 5, "Demolition off (08): red pixels %d, the house as the work leaves it" % r8)
say(abs(r9 - r4) <= 0.25 * r4, "Demolition on again (09): red pixels %d as on (04) %d" % (r9, r4))
mid = (qx0 - k * 0.1, qy1, qx0 + k * 0.1, qy0)
say(lum("04", mid) > 225, "partition has no fill (04): the middle of the wall is paper, lum %.0f" % lum("04", mid))
sys.exit(0 if ok else 1)
