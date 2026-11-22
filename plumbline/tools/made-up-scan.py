#!/usr/bin/env python3
"""A made-up scan of one room, as a PLY point cloud, for demos/scan.sh.

    tools/made-up-scan.py <out.ply> [--binary] [--feet-y-up] [--rough]
                          [--as-scanned] [--no-ceiling] [--turned <degrees>]

A room 20 ft by 12 ft inside and 8 ft high: points 0.05 m apart on its four walls, its
floor and its ceiling, its southwest inside corner at zero. In metres with z up, as a
phone's scan is; with --feet-y-up the same room in feet with y up. Nothing in it is
from a real scan. With --rough, a two-room house as a phone would scan it: scattered
points, each a little out, furniture, strays outside, and the whole scan leaning.

For demos/found.sh (call 173), three more, each on the two-room house:
  --as-scanned   the house as a scanner sees it: the four outside walls one face each (seen
                 from indoors), the partition two faces 4 1/2 in apart (a 2x4 with 1/2 in
                 board each side), the north wall behind the tall cabinets not seen below
                 their top, and the cabinets' own backs not seen. (Implies --rough.)
  --no-ceiling   the same with its ceiling and everything above 7 ft 6 in left out.
  --turned <d>   the same house turned d degrees about its southwest corner, as it stands
                 in the room, before the scanner's lean.
With none of the three the files written are exactly what they were.
"""
import struct, sys

FT = 0.3048
W, D, H = 20 * FT, 12 * FT, 8 * FT
STEP = 0.05


def steps(length):
    n = int(round(length / STEP))
    return [length * i / n for i in range(n + 1)]


def room():
    pts = []
    for x in steps(W):
        for y in steps(D):
            pts.append((x, y, 0.0))
            pts.append((x, y, H))
    for z in steps(H)[1:-1]:
        for x in steps(W):
            pts.append((x, 0.0, z))
            pts.append((x, D, z))
        for y in steps(D)[1:-1]:
            pts.append((0.0, y, z))
            pts.append((W, y, z))
    return pts


def rough(as_scanned=False, no_ceiling=False, turned=0.0):
    """A house as a phone would scan it, made up: two rooms, 30 ft by 20 ft in all and 8 ft
    high, a partition 18 ft from the west wall with a 3 ft doorway, a window (no points:
    glass does not scan) in the south wall, a table, a sofa, and cabinets 7 ft tall along
    the north wall of the east room. The points are scattered, not on a grid; each is out
    by about 1.5 cm; the floor is missed under the furniture; one in fifty is a stray
    outside the house; and the whole scan leans half a degree, as one not levelled does.
    The same every time (a fixed seed)."""
    import math, random
    r = random.Random(7213)
    W, D, H, PX = 30 * FT, 20 * FT, 8 * FT, 18 * FT
    pts = []
    boxes = [  # furniture: west, south, east, north, top (m)
        (4 * FT, 6 * FT, 10 * FT, 9 * FT, 2.5 * FT),      # a table
        (1 * FT, 14 * FT, 8 * FT, 17 * FT, 3 * FT),       # a sofa
        (20 * FT, 18 * FT, 29 * FT, 20 * FT, 7 * FT),     # tall cabinets on the north wall
    ]
    under = lambda x, y: any(b[0] <= x <= b[2] and b[1] <= y <= b[3] for b in boxes)
    def scatter(n, make):
        for _ in range(n):
            p = make()
            if p is not None:
                pts.append(p)
    scatter(60000, lambda: (lambda x, y: None if under(x, y) else (x, y, 0.0))(r.uniform(0, W), r.uniform(0, D)))
    if not no_ceiling:
        scatter(60000, lambda: (r.uniform(0, W), r.uniform(0, D), H))
    def wall(x0, y0, x1, y1, n, gaps=()):
        def one():
            t, z = r.random(), r.uniform(0, H)
            along = t * math.hypot(x1 - x0, y1 - y0)
            if any(g[0] <= along <= g[1] and g[2] <= z <= g[3] for g in gaps):
                return None
            return (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, z)
        scatter(n, one)
    wall(0, 0, W, 0, 30000, gaps=[(8 * FT, 13 * FT, 3 * FT, 7 * FT)])   # south, with a window 5 ft wide
    wall(W, 0, W, D, 20000)
    # The north wall: behind the tall cabinets (20 ft to 29 ft east) it is not seen below their top.
    wall(0, D, W, D, 30000, gaps=[(20 * FT, 29 * FT, 0, 7 * FT)] if as_scanned else ())
    wall(0, 0, 0, D, 20000)
    doorway = [(9 * FT, 12 * FT, 0, 6.67 * FT)]
    if as_scanned:                                                       # the partition: two faces, 4 1/2 in apart
        for side in (-1, 1):
            wall(PX + side * 2.25 / 12 * FT, 0, PX + side * 2.25 / 12 * FT, D, 20000, gaps=doorway)
    else:
        wall(PX, 0, PX, D, 20000, gaps=doorway)                          # the partition, with its doorway
    for n, (x0, y0, x1, y1, top) in enumerate(boxes):                    # furniture: its top and its sides
        scatter(4000, lambda: (r.uniform(x0, x1), r.uniform(y0, y1), top))
        for (a, b, c, d) in ((x0, y0, x1, y0), (x1, y0, x1, y1), (x0, y1, x1, y1), (x0, y0, x0, y1)):
            if as_scanned and n == 2 and b == y1 and d == y1:            # the cabinets' backs, against the wall: not seen
                continue
            scatter(1500, lambda: (lambda t: (a + (c - a) * t, b + (d - b) * t, r.uniform(0, top)))(r.random()))
    noisy = [(x + r.gauss(0, 0.015), y + r.gauss(0, 0.015), z + r.gauss(0, 0.015)) for x, y, z in pts]
    strays = [(r.uniform(-6, W + 6), r.uniform(-6, D + 6), r.uniform(-1, H + 2)) for _ in range(len(noisy) // 50)]
    every = noisy + strays
    if turned:                                                           # the house turned about its southwest corner
        t = math.radians(turned)
        every = [(x * math.cos(t) - y * math.sin(t), x * math.sin(t) + y * math.cos(t), z) for x, y, z in every]
    lean = math.radians(0.5)                                             # about the north-south line through the west wall
    out = [(x * math.cos(lean) - z * math.sin(lean), y, x * math.sin(lean) + z * math.cos(lean)) for x, y, z in every]
    if no_ceiling:                                                       # nothing above 7 ft 6 in
        out = [p for p in out if p[2] <= 7.5 * FT]
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    binary, feet = "--binary" in sys.argv, "--feet-y-up" in sys.argv
    as_scanned = "--as-scanned" in sys.argv
    extra = as_scanned or "--no-ceiling" in sys.argv or "--turned" in sys.argv
    if extra and not (as_scanned or "--rough" in sys.argv):
        sys.exit("--no-ceiling and --turned are for the house: give --as-scanned or --rough")
    turned = 0.0
    if "--turned" in sys.argv:
        i = sys.argv.index("--turned")
        turned = float(sys.argv[i + 1])
        args = [a for a in args if a != sys.argv[i + 1]]
    pts = rough(as_scanned, "--no-ceiling" in sys.argv, turned) if (as_scanned or "--rough" in sys.argv) else room()
    if feet:  # the same room in feet, y up: (east, up, south) keeps it right-handed
        pts = [(x / FT, z / FT, -y / FT) for x, y, z in pts]
    head = "ply\nformat %s 1.0\ncomment a made-up room, not a real scan\nelement vertex %d\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nend_header\n" % ("binary_little_endian" if binary else "ascii", len(pts))
    with open(args[0], "wb") as f:
        f.write(head.encode())
        for x, y, z in pts:
            f.write(struct.pack("<fffB", x, y, z, 128) if binary else ("%.4f %.4f %.4f 128\n" % (x, y, z)).encode())


if __name__ == "__main__":
    main()
