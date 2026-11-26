#!/usr/bin/env python3
"""Makes the made-up room of demos/scan-files.sh as a PLY and as LAS files of several kinds (call 230).

    make.py <folder>

No real scan is used or kept: a room 12 ft by 10 ft and 8 ft high, a point every quarter foot on its floor, its
ceiling and its four walls (9,474 points), its southwest inside corner at the scan's zero, in meters (a quarter foot
is 0.0762 m, a whole number of tenths of a millimetre, so a LAS that keeps a point as a whole number times 0.0001
holds every point exactly, and the PLY holds the same numbers as text). LAS is written to the layout of the
specification's table 3 (the public header block) and tables 7 and 18 (point records 0 and 7).

  room.ply            the points as text, meters, z up
  room.las            LAS 1.2, point record format 0, no coordinate system: the file says nothing of its unit
  shifted.ply, shifted.las   the same room half a foot east and north of the zero
  survey.las          LAS 1.4, format 7 (with colour), a coordinate system as text saying meters, and a place far
                      from the zero (an offset of 512000 m east, 4200000 m north, 300 m up)
  feet.las            LAS 1.2, format 1, a GeoTIFF record saying feet, the room kept in feet and set half a foot off zero
  mixed.las           the room in feet, the file saying feet across and meters up: it says two things
  degrees.las         a coordinate system of degrees of latitude and longitude
  format11.las        a point record format LAS does not define
  version20.las       LAS 2.0
  cut.las             room.las cut short
  withheld.las        room.las with one point marked withheld
Then `made-up-scan-files room.las <folder>` writes room.laz and room.e57.
"""
import struct
import sys

out = sys.argv[1]
Q = 762  # a quarter foot, in tenths of a millimetre
W, D, H = 48, 40, 32  # 12 ft, 10 ft and 8 ft in quarter feet


def room():
    p = []
    for i in range(W + 1):
        for j in range(D + 1):
            p.append((i * Q, j * Q, 0))
            p.append((i * Q, j * Q, H * Q))
    for k in range(1, H):
        for i in range(W + 1):
            p.append((i * Q, 0, k * Q))
            p.append((i * Q, D * Q, k * Q))
        for j in range(1, D):
            p.append((0, j * Q, k * Q))
            p.append((W * Q, j * Q, k * Q))
    return p


MIN = {0: 20, 1: 28, 2: 26, 3: 34, 6: 30, 7: 36}


def vlr(user, rid, data):
    return struct.pack("<H16sHH32s", 0, user.encode(), rid, len(data), b"made up") + data


def las(path, pts, minor=2, fmt=0, scale=0.0001, offset=(0.0, 0.0, 0.0), vlrs=(), withheld=(), version=None, cut=None):
    header = {0: 227, 1: 227, 2: 227, 3: 235, 4: 375}[minor]
    body = b"".join(vlrs)
    rec = MIN[fmt]
    start = header + len(body)
    h = bytearray(header)
    h[0:4] = b"LASF"
    h[24], h[25] = version or (1, minor)
    h[26:58] = b"made-up demonstration".ljust(32, b"\0")
    struct.pack_into("<H", h, 94, header)
    struct.pack_into("<I", h, 96, start)
    struct.pack_into("<I", h, 100, len(vlrs))
    h[104] = fmt
    struct.pack_into("<H", h, 105, rec)
    struct.pack_into("<I", h, 107, 0 if (minor >= 4 and fmt >= 6) else len(pts))
    for k in range(3):
        struct.pack_into("<d", h, 131 + 8 * k, scale)
        struct.pack_into("<d", h, 155 + 8 * k, offset[k])
    if minor >= 4:
        struct.pack_into("<Q", h, 247, len(pts))
    data = bytearray()
    for n, (x, y, z) in enumerate(pts):
        r = bytearray(rec)
        struct.pack_into("<iii", r, 0, x, y, z)
        if n in withheld:
            r[15] = 0x80 if fmt < 6 else 0x04
        data += r
    blob = bytes(h) + body + bytes(data)
    if cut:
        blob = blob[:cut]
    open(f"{out}/{path}", "wb").write(blob)


pts = room()
with open(f"{out}/room.ply", "w") as f:
    f.write(f"ply\nformat ascii 1.0\nelement vertex {len(pts)}\nproperty float x\nproperty float y\nproperty float z\nend_header\n")
    for x, y, z in pts:
        f.write(f"{x / 1e4:.4f} {y / 1e4:.4f} {z / 1e4:.4f}\n")
las("room.las", pts)
# The same room half a foot off the zero, as a PLY and as a LAS: no wall lies on a whole foot, where the walls finder (which
# cuts the floor into square feet and the height into inches) could tell two files of the same room apart by the last digit
# of a number. The PLY here carries the LAS's own numbers to the last digit, so that the two files are the same points to the bit.
with open(f"{out}/shifted.ply", "w") as f:
    f.write(f"ply\nformat ascii 1.0\nelement vertex {len(pts)}\nproperty float x\nproperty float y\nproperty float z\nend_header\n")
    for x, y, z in pts:
        # The very numbers the LAS works out (whole number times scale, plus offset), written to the last digit.
        f.write(f"{x * 0.0001 + 0.1524!r} {y * 0.0001 + 0.1524!r} {z * 0.0001!r}\n")
las("shifted.las", pts, offset=(0.1524, 0.1524, 0.0))
# Survey: a place far from zero, in a projected system written as text (LAS 1.4, format 7, the colour left at zero).
OFF = (512000.0, 4200000.0, 300.0)
wkt = b'PROJCS["a made-up grid",GEOGCS["a made-up datum",DATUM["d",SPHEROID["s",6378137,298.25]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],UNIT["metre",1]]\0'
las("survey.las", pts, minor=4, fmt=7, offset=OFF, vlrs=[vlr("LASF_Projection", 2112, wkt)])
# Feet: the room in feet (a thousandth of a foot to the whole number), half a foot off zero so no wall lies on a whole foot.


def geokeys(keys):
    v = [1, 1, 0, len(keys)]
    for k, val in keys:
        v += [k, 0, 1, val]
    return struct.pack(f"<{len(v)}H", *v)


ft = [(round(x / 1e4 / 0.3048 * 1000), round(y / 1e4 / 0.3048 * 1000), round(z / 1e4 / 0.3048 * 1000)) for x, y, z in pts]
las("feet.las", ft, fmt=1, scale=0.001, offset=(0.5, 0.5, 0.0), vlrs=[vlr("LASF_Projection", 34735, geokeys([(1024, 1), (3076, 9002)]))])
las("mixed.las", ft, fmt=1, scale=0.001, offset=(0.5, 0.5, 0.0), vlrs=[vlr("LASF_Projection", 34735, geokeys([(1024, 1), (3076, 9002), (4099, 9001)]))])
las("degrees.las", pts, vlrs=[vlr("LASF_Projection", 34735, geokeys([(1024, 2)]))])
las("room-bad.las", pts)  # then the headers below are changed by hand


def patch(name, at, byte):
    b = bytearray(open(f"{out}/room-bad.las", "rb").read())
    b[at] = byte
    open(f"{out}/{name}", "wb").write(bytes(b))


patch("format11.las", 104, 11)
b = bytearray(open(f"{out}/room-bad.las", "rb").read())
b[24], b[25] = 2, 0
open(f"{out}/version20.las", "wb").write(bytes(b))
import os

os.remove(f"{out}/room-bad.las")
las("cut.las", pts, cut=4000)
las("withheld.las", pts, withheld=(5,))
print(f"made {len(pts)} points")
