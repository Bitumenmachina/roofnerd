#!/usr/bin/env python3
"""A made-up scan of one room, as a PLY point cloud, for demos/scan.sh.

    tools/made-up-scan.py <out.ply> [--binary] [--feet-y-up]

A room 20 ft by 12 ft inside and 8 ft high: points 0.05 m apart on its four walls, its
floor and its ceiling, its southwest inside corner at zero. In metres with z up, as a
phone's scan is; with --feet-y-up the same room in feet with y up. Nothing in it is
from a real scan.
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


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    binary, feet = "--binary" in sys.argv, "--feet-y-up" in sys.argv
    pts = room()
    if feet:  # the same room in feet, y up: (east, up, south) keeps it right-handed
        pts = [(x / FT, z / FT, -y / FT) for x, y, z in pts]
    head = "ply\nformat %s 1.0\ncomment a made-up room, not a real scan\nelement vertex %d\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nend_header\n" % ("binary_little_endian" if binary else "ascii", len(pts))
    with open(args[0], "wb") as f:
        f.write(head.encode())
        for x, y, z in pts:
            f.write(struct.pack("<fffB", x, y, z, 128) if binary else ("%.4f %.4f %.4f 128\n" % (x, y, z)).encode())


if __name__ == "__main__":
    main()
