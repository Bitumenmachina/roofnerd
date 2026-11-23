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


# ---- call 179: scenes for the scan's walls (new flags only; the files above are as they were) ----
#
#   tools/made-up-scan.py <out.ply> --scene <name> [--density N] [--turned D] [--noise CM]
#                         [--set key=value ...] [--seed N] [--binary]
#
# A made-up house of any size as a scanner sees it, built from faces: each wall a face or two,
# furniture and cabinets as boxes, a floor and a ceiling, points scattered uniformly at
# --density to the square foot of each surface (default 100), each a little out (--noise,
# default 1.5 cm), one in fifty a stray, the whole scan leaning half a degree. Nothing in it
# is from a real scan. --scene list names them; each says what it is and what the right
# answer is, in feet, in its own docstring (--scene <name> --explain).

def scene_main():
    import math, random
    try:
        import numpy as np
    except ImportError:
        sys.exit("--scene needs numpy")
    argv = sys.argv[1:]
    def opt(name, default=None, cast=float):
        if name in argv:
            return cast(argv[argv.index(name) + 1])
        return default
    name = opt("--scene", cast=str)
    if name == "list" or name not in SCENES:
        for k, f in SCENES.items():
            print("%-12s %s" % (k, (f.__doc__ or "").strip().splitlines()[0]))
        if name not in ("list", None) and name not in SCENES:
            sys.exit("no scene called %s" % name)
        return
    if "--explain" in argv:
        print((SCENES[name].__doc__ or "").strip())
        return
    params = {}
    for i, a in enumerate(argv):
        if a == "--set":
            k, v = argv[i + 1].split("=", 1)
            params[k] = float(v)
    dens = opt("--density", 100.0)
    noise = opt("--noise", 1.5) / 100.0 / FT          # feet
    turned = opt("--turned", 0.0)
    seed = int(opt("--seed", 4243))
    out = [a for a in argv if not a.startswith("--") and "=" not in a][0]
    rng = np.random.default_rng(seed)
    S = Scene(rng, dens)
    SCENES[name](S, params)
    P = S.points()
    P = P + rng.normal(0, noise, P.shape)
    n_stray = len(P) // 50
    lo, hi = P.min(axis=0), P.max(axis=0)
    strays = np.column_stack([rng.uniform(lo[0] - 6, hi[0] + 6, n_stray), rng.uniform(lo[1] - 6, hi[1] + 6, n_stray), rng.uniform(-1, hi[2] + 2, n_stray)])
    P = np.vstack([P, strays])
    if turned:
        t = math.radians(turned)
        P = np.column_stack([P[:, 0] * math.cos(t) - P[:, 1] * math.sin(t), P[:, 0] * math.sin(t) + P[:, 1] * math.cos(t), P[:, 2]])
    lean = math.radians(opt("--lean", 0.5))
    P = np.column_stack([P[:, 0] * math.cos(lean) - P[:, 2] * math.sin(lean), P[:, 1], P[:, 0] * math.sin(lean) + P[:, 2] * math.cos(lean)])
    if "--units-ft" in argv:                                   # written in feet, not meters (for a scan read the wrong way)
        M = P
    else:
        M = P * FT
    if "--up-y" in argv:                                       # the file's up is its second axis
        M = np.column_stack([M[:, 0], M[:, 2], -M[:, 1]])
    if "--far" in argv:                                        # taken far from zero, as a georeferenced scan is
        M = M + np.array([1234.5, 567.25, 0.0]) * (1 if "--units-ft" in argv else FT)
    head = "ply\nformat %s 1.0\ncomment a made-up house, not a real scan\nelement vertex %d\nproperty float x\nproperty float y\nproperty float z\nend_header\n" % ("binary_little_endian" if "--binary" in argv else "ascii", len(M))
    with open(out, "wb") as f:
        f.write(head.encode())
        if "--binary" in argv:
            f.write(M.astype("<f4").tobytes())
        else:
            np.savetxt(f, M, fmt="%.4f")


class Scene:
    """Faces, boxes, a floor and a ceiling, in feet. Plan: east x, north y."""
    def __init__(self, rng, dens):
        import numpy as np
        self.np, self.rng, self.dens = np, rng, dens
        self.faces, self.boxes, self.floors, self.ceilings, self.foot = [], [], [], [], []

    def face(self, a, b, top, gaps=(), hide=(), z0=0.0, dens=None):
        """A vertical face from a to b, up to `top` (a height, or a function of (x, y)). `gaps`: (from, to, low, high)
        left empty (an opening); `hide`: (from, to, height) not seen below that height (something stands in front)."""
        self.faces.append((a, b, top, list(gaps), list(hide), z0, dens or self.dens))

    def thick(self, a, b, t, top, gaps=(), hide=(), sides="both", dens=None):
        """A wall t feet thick on the centre line a to b: its two faces (or only the left or the right of the way a to b)."""
        dx, dy = b[0] - a[0], b[1] - a[1]
        L = (dx * dx + dy * dy) ** 0.5
        nx, ny = -dy / L, dx / L
        self.foot.append((a, b, t))
        for sgn, keep in ((1, sides in ("both", "left")), (-1, sides in ("both", "right"))):
            if keep:
                o = sgn * t / 2
                self.face((a[0] + nx * o, a[1] + ny * o), (b[0] + nx * o, b[1] + ny * o), top, gaps, hide, dens=dens)

    def box(self, x0, y0, x1, y1, top, unseen=(), dens=None):
        """A box of furniture or cabinets: its top and its sides; `unseen` names sides not seen (south, north, west, east)."""
        self.boxes.append((x0, y0, x1, y1, top, set(unseen), dens or self.dens))

    def floor(self, x0, y0, x1, y1):
        self.floors.append((x0, y0, x1, y1))

    def ceiling(self, x0, y0, x1, y1, z0, z1=None, slope_axis="y"):
        """A ceiling over a rectangle: at z0, or rising from z0 at the south (or west) edge to z1 at the north (or east)."""
        self.ceilings.append((x0, y0, x1, y1, z0, z0 if z1 is None else z1, slope_axis))

    def ceiling_at(self, x, y):
        for (x0, y0, x1, y1, z0, z1, ax) in self.ceilings:
            if x0 - 1e-6 <= x <= x1 + 1e-6 and y0 - 1e-6 <= y <= y1 + 1e-6:
                f = (y - y0) / (y1 - y0) if ax == "y" else (x - x0) / (x1 - x0)
                return z0 + (z1 - z0) * f
        return max(c[4] for c in self.ceilings)

    def points(self):
        np, rng = self.np, self.rng
        out = []
        def uni(x0, y0, x1, y1, d):
            n = int(round((x1 - x0) * (y1 - y0) * d))
            return np.column_stack([rng.uniform(x0, x1, n), rng.uniform(y0, y1, n)])
        def under(xy):
            keep = np.ones(len(xy), bool)
            for (x0, y0, x1, y1, top, un, d) in self.boxes:
                keep &= ~((xy[:, 0] >= x0) & (xy[:, 0] <= x1) & (xy[:, 1] >= y0) & (xy[:, 1] <= y1))
            for (a, b, t) in self.foot:
                dx, dy = b[0] - a[0], b[1] - a[1]
                L = (dx * dx + dy * dy) ** 0.5
                s = ((xy[:, 0] - a[0]) * dx + (xy[:, 1] - a[1]) * dy) / L
                c = (-(xy[:, 0] - a[0]) * dy + (xy[:, 1] - a[1]) * dx) / L
                keep &= ~((s >= 0) & (s <= L) & (np.abs(c) <= t / 2 + 0.05))
            return keep
        for (x0, y0, x1, y1) in self.floors:
            xy = uni(x0, y0, x1, y1, self.dens)
            xy = xy[under(xy)]
            out.append(np.column_stack([xy, np.zeros(len(xy))]))
        for (x0, y0, x1, y1, z0, z1, ax) in self.ceilings:
            xy = uni(x0, y0, x1, y1, self.dens)
            z = np.array([self.ceiling_at(x, y) for x, y in xy]) if z0 != z1 else np.full(len(xy), z0)
            out.append(np.column_stack([xy, z]))
        for (a, b, top, gaps, hide, z0, d) in self.faces:
            dx, dy = b[0] - a[0], b[1] - a[1]
            L = (dx * dx + dy * dy) ** 0.5
            tops = top if callable(top) else None
            H = max(self.ceiling_at(a[0], a[1]), self.ceiling_at(b[0], b[1])) if tops is None and top is None else (top if tops is None else 20.0)
            n = int(round(L * H * d))
            s, z = rng.uniform(0, L, n), rng.uniform(0, H, n)
            x, y = a[0] + dx * s / L, a[1] + dy * s / L
            keep = np.ones(n, bool)
            if tops is not None or top is None:
                cap = np.array([self.ceiling_at(xx, yy) for xx, yy in zip(x, y)]) if (top is None) else np.array([top(xx, yy) for xx, yy in zip(x, y)])
                keep &= z <= cap
            for (g0, g1, lo, hi) in gaps:
                keep &= ~((s >= g0) & (s <= g1) & (z >= lo) & (z <= hi))
            for (h0, h1, hz) in hide:
                keep &= ~((s >= h0) & (s <= h1) & (z <= hz))
            keep &= z >= z0
            out.append(np.column_stack([x[keep], y[keep], z[keep]]))
        for (x0, y0, x1, y1, top, un, d) in self.boxes:
            n = int(round((x1 - x0) * (y1 - y0) * d))
            out.append(np.column_stack([rng.uniform(x0, x1, n), rng.uniform(y0, y1, n), np.full(n, top)]))
            for side, (a, b) in (("south", ((x0, y0), (x1, y0))), ("east", ((x1, y0), (x1, y1))), ("north", ((x0, y1), (x1, y1))), ("west", ((x0, y0), (x0, y1)))):
                if side in un:
                    continue
                L = ((b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2) ** 0.5
                m = int(round(L * top * d))
                s, z = rng.uniform(0, 1, m), rng.uniform(0, top, m)
                out.append(np.column_stack([a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s, z]))
        return np.vstack(out)


def box_room(S, W, D, H, outside=True, ceiling=True):
    """The four outside walls, one face each, seen from indoors, a floor and a ceiling H high."""
    S.floor(0, 0, W, D)
    if ceiling:
        S.ceiling(0, 0, W, D, H)
    if outside:
        S.face((0, 0), (W, 0), None)
        S.face((W, 0), (W, D), None)
        S.face((W, D), (0, D), None)
        S.face((0, D), (0, 0), None)


def s_house(S, p):
    """A 30 by 16 ft room (--set w= d= to size it), 8 ft high, a partition 4 1/2 in thick (two faces) at 18 ft from the west wall. Right: 5 walls."""
    W, D = p.get("w", 30), p.get("d", 16)
    box_room(S, W, D, p.get("height", 8))
    S.thick((18, 0), (18, D), 0.375, None)


def s_hidden(S, p):
    """A 30 by 16 ft room, 8 ft high, tall cabinets 2 ft deep and 7 ft high hiding --set pct=N of the north wall from its east end. Right: 4 walls (north one wall, the hidden part said)."""
    W, D = 30, 16
    box_room(S, W, D, 8)
    hid = W * p.get("pct", 45) / 100.0
    S.box(W - hid, D - 2, W, D, 7.0, unseen=("north",))
    S.faces = [f for f in S.faces]
    # the north wall behind them is not seen below the cabinets' top
    a, b, top, g, h, z0, d = S.faces[2]
    S.faces[2] = (a, b, top, g, [(0, hid, 7.0)], z0, d)


def s_pack2(S, p):
    """A 20 by 14 ft room, 8 ft high; cabinets 2 ft deep and 6 ft 6 in high against the south, north and west walls, the walls behind them not seen below 6 ft 6 in. Right: 4 walls."""
    W, D = 20, 14
    box_room(S, W, D, 8)
    S.box(1, 0, 19, 2, 6.5, unseen=("south",))
    S.box(1, 12, 19, 14, 6.5, unseen=("north",))
    S.box(0, 2, 2, 12, 6.5, unseen=("west",))
    S.faces[0] = (S.faces[0][0], S.faces[0][1], S.faces[0][2], [], [(1, 19, 6.5)], 0.0, S.dens)
    S.faces[2] = (S.faces[2][0], S.faces[2][1], S.faces[2][2], [], [(1, 19, 6.5)], 0.0, S.dens)
    S.faces[3] = (S.faces[3][0], S.faces[3][1], S.faces[3][2], [], [(2, 12, 6.5)], 0.0, S.dens)


def s_tall(S, p):
    """A 20 by 14 ft room, 8 ft high; a bank of cabinets 10 ft long, 2 ft deep, 7 ft 9 in high, east 2 to 12 ft, against the north wall (its back not seen). Right: 4 walls; the bank is not one."""
    W, D = 20, 14
    box_room(S, W, D, 8)
    S.box(2, 12, 12, 14, p.get("top", 7.8), unseen=("north",))
    a, b, top, g, h, z0, d = S.faces[2]
    S.faces[2] = (a, b, top, g, [(W - 12, W - 2, p.get("top", 7.8))], z0, d)


def s_slope(S, p):
    """A 30 by 16 ft room whose ceiling rises from 8 ft at the south wall to 12 ft at the north; walls to the ceiling. Right: 4 walls, the ceiling said to slope."""
    S.floor(0, 0, 30, 16)
    S.ceiling(0, 0, 30, 16, 8.0, 12.0)
    S.face((0, 0), (30, 0), None)
    S.face((30, 0), (30, 16), None)
    S.face((30, 16), (0, 16), None)
    S.face((0, 16), (0, 0), None)


def s_twoceil(S, p):
    """Two rooms, 15 by 16 ft each, the west 8 ft high and the east 10 ft; a partition 4 1/2 in thick at 15 ft, 8 ft high. Right: 5 walls, the ceiling said to be two heights."""
    S.floor(0, 0, 30, 16)
    S.ceiling(0, 0, 15, 16, 8.0)
    S.ceiling(15, 0, 30, 16, 10.0)
    S.face((0, 0), (30, 0), None)
    S.face((30, 0), (30, 16), None)
    S.face((30, 16), (0, 16), None)
    S.face((0, 16), (0, 0), None)
    S.thick((15, 0), (15, 16), 0.375, 8.0)


def s_closet(S, p):
    """A 20 by 12 ft room, 8 ft high, a closet of two walls 3 1/2 in thick, 2 ft apart (at 6 and 8 ft north, 0 to 6 ft east) and a 2 ft cross wall at 6 ft east; all scanned both faces. Right: 4 outside walls, 3 closet walls."""
    box_room(S, 20, 12, 8)
    t = 3.5 / 12
    S.thick((0, 6), (6, 6), t, None)
    S.thick((0, 8), (6, 8), t, None)
    S.thick((6, 6), (6, 8), t, None)


def s_thickthin(S, p):
    """A 30 by 16 ft room, 8 ft high, a 12 in wall at 10 ft east and a 3 1/2 in wall at 20 ft east, both across; both faces. Right: 4 outside + 2 partitions."""
    box_room(S, 30, 16, 8)
    S.thick((10, 0), (10, 16), 1.0, None)
    S.thick((20, 0), (20, 16), 3.5 / 12, None)


def s_thin(S, p):
    """A 30 by 16 ft room, 8 ft high, a 3 1/2 in wall at 20 ft east. Right: 5 walls."""
    box_room(S, 30, 16, 8)
    S.thick((20, 0), (20, 16), 3.5 / 12, None)


def s_thickthin2(S, p):
    """A 30 by 16 ft room, 8 ft high, a 12 in wall at 10 ft east and a 4 1/2 in wall at 20 ft east. Right: 6 walls."""
    box_room(S, 30, 16, 8)
    S.thick((10, 0), (10, 16), 1.0, None)
    S.thick((20, 0), (20, 16), 4.5 / 12, None)


def s_t18(S, p):
    """A 30 by 16 ft room, 8 ft high, an 18 in wall at 15 ft east, 15 ft 6 in long from the south wall; both faces. Right: 5 walls."""
    box_room(S, 30, 16, 8)
    S.thick((15, 0), (15, 15.5), p.get("t", 18) / 12, None)


def s_l(S, p):
    """An L: 30 by 20 ft with a 14 by 10 ft corner taken out of the north east; six walls, 8 ft high, each one face. Right: 6 walls."""
    S.floor(0, 0, 30, 10)
    S.floor(0, 10, 16, 20)
    S.ceiling(0, 0, 30, 10, 8.0)
    S.ceiling(0, 10, 16, 20, 8.0)
    pts = [(0, 0), (30, 0), (30, 10), (16, 10), (16, 20), (0, 20)]
    for i in range(6):
        S.face(pts[i], pts[(i + 1) % 6], None)


def s_rooms3(S, p):
    """A 30 by 18 ft house, 8 ft high, three rooms: partitions 4 1/2 in at 10 and 20 ft east, doorways 5 to 8 ft and 8 to 11 ft north. Right: 5 walls, 2 doorways."""
    box_room(S, 30, 18, 8)
    S.thick((10, 0), (10, 18), 0.375, None, gaps=[(5, 8, 0, 6.67)])
    S.thick((20, 0), (20, 18), 0.375, None, gaps=[(8, 11, 0, 6.67)])


def s_openings(S, p):
    """A 24 by 12 ft room, 8 ft high: in the south wall two doors (2 ft 4 in at 3 ft, 3 ft at 8 ft; to 6 ft 8 in) and two windows (5 ft at 12 to 17 ft, sill 1 ft, head 5 ft; 2 ft at 20 ft, sill 2, head 7). Right: 4 walls, 2 doors, 2 windows."""
    box_room(S, 24, 12, 8)
    S.faces[0] = ((0, 0), (24, 0), None, [(3, 5 + 1 / 3, 0, 6.67), (8, 11, 0, 6.67), (12, 17, 1, 5), (20, 22, 2, 7)], [], 0.0, S.dens)


def s_furnished(S, p):
    """A 30 by 16 ft room, 8 ft high with a table, a sofa and a chair 6 ft 6 in tall in front of the walls (a packed room). Right: 4 walls."""
    box_room(S, 30, 16, 8)
    S.box(8, 6, 14, 10, 2.5)
    S.box(1, 7, 3.5, 13, 3.0)
    S.box(10, 0.5, 13, 3, 6.5)


def s_nonorth(S, p):
    """A 30 by 16 ft room, 8 ft high, with no north wall at all (the scan was taken with it open). Right: 3 walls; the walls do not close: 2 ends meet nothing, at 0 and 30 ft east, 16 ft north."""
    S.floor(0, 0, 30, 16)
    S.ceiling(0, 0, 30, 16, 8.0)
    S.face((0, 0), (30, 0), None)
    S.face((30, 0), (30, 16), None)
    S.face((0, 16), (0, 0), None)


def s_empty(S, p):
    """An empty room 20 by 12 ft, 8 ft high. Right: 4 walls."""
    box_room(S, 20, 12, 8)


def s_cabin(S, p):
    """A cabin 18 by 12 ft inside, 9 ft ceiling, a closet 4 ft by 3 ft with 3 1/2 in walls in the northeast corner, a window 3 ft wide, sill 3, head 7 in the west wall, a 3 ft doorway in the south wall. Right: 6 walls."""
    H = 9.0
    box_room(S, 18, 12, H)
    S.faces[0] = ((0, 0), (18, 0), None, [(6, 9, 0, 6.67)], [], 0.0, S.dens)
    S.faces[3] = ((0, 12), (0, 0), None, [(3, 6, 3, 7)], [], 0.0, S.dens)
    S.thick((14, 9), (18, 9), 3.5 / 12, None, gaps=[(1, 3.7, 0, 6.67)])
    S.thick((14, 9), (14, 12), 3.5 / 12, None)


def s_masonry(S, p):
    """A 36 by 20 ft house, 8 ft 6 in high: a 10 in masonry partition at 14 ft and a 4 1/2 in partition at 26 ft, doorways in each; a table and a sofa. Right: 6 walls, 2 doorways."""
    H = 8.5
    box_room(S, 36, 20, H)
    S.thick((14, 0), (14, 20), 10 / 12, None, gaps=[(8, 11, 0, 6.67)])
    S.thick((26, 0), (26, 20), 0.375, None, gaps=[(12, 15, 0, 6.67)])
    S.box(3, 6, 9, 9, 2.5)
    S.box(1, 14, 8, 17, 3.0)


def s_bank(S, p):
    """A 26 by 18 ft room, 8 ft 6 in high, a partition 4 1/2 in at 13 ft with a doorway 6 to 9 ft; a cabinet bank 2 ft deep, 8 ft long and 8 ft 3 in high against the east wall (its back not seen) and a tall sofa. Right: 5 walls, 1 doorway; the bank is not a wall."""
    H = 8.5
    box_room(S, 26, 18, H)
    S.thick((13, 0), (13, 18), 0.375, None, gaps=[(6, 9, 0, 6.67)])
    S.box(24, 5, 26, 13, 8.25, unseen=("east",))
    a, b, top, g, h, z0, d = S.faces[1]
    S.faces[1] = (a, b, top, g, [(5, 13, 8.25)], z0, d)
    S.box(2, 5, 4.5, 11, 3.0)


SCENES = {k[2:]: v for k, v in list(globals().items()) if k.startswith("s_") and callable(v)}


if __name__ == "__main__":
    if "--scene" in sys.argv:
        scene_main()
    else:
        main()
