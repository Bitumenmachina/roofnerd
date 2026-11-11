#!/usr/bin/env python3
"""Writes the made-up plan the demonstrations trace over: demos/plans/made-up-plan.pdf
(a vector PDF, written by hand below with no library), and the same plan as a PNG and a
JPEG picture when Pillow is installed. The house is made up; it is no real job.

The plan: a house 30 ft by 20 ft drawn at 1/4 in to the foot, so 18 points to the foot.
Its outline is 540 by 360 points with its lower-left corner at (126, 126) on a letter
sheet laid on its side (792 by 612 points). A partition runs north from the south wall
12 ft (216 points) along it. Under the plan is a dimension line from (126, 90) to
(666, 90): 540 points, 30 ft. A door's swing is one curve. Eight straight lines in all.
Run from the repository root:  tools/made-up-plan.py
"""
import os, zlib

OUT = os.path.join(os.path.dirname(__file__), "..", "demos", "plans")
X0, Y0, W, H = 126, 126, 540, 360
LINES = [  # each a straight stroke: x1 y1 x2 y2, in points
    (X0 + 216, Y0, X0 + 216, Y0 + H),       # the partition
    (X0, 90, X0 + W, 90),                    # the dimension line
    (X0, 82, X0, 98), (X0 + W, 82, X0 + W, 98),  # its two ticks
]

def content():
    c = ["1.2 w 0 0 0 RG"]
    c.append(f"{X0} {Y0} m {X0 + W} {Y0} l {X0 + W} {Y0 + H} l {X0} {Y0 + H} l h S")  # the outline: four lines
    c.append("0.6 w")
    for x1, y1, x2, y2 in LINES:
        c.append(f"{x1} {y1} m {x2} {y2} l S")
    # A door 3 ft wide in the partition, 8 ft up it: its swing, a quarter circle of 54 points.
    px, py, r, k = X0 + 216, Y0 + 144, 54, 54 * 0.5523
    c.append(f"{px + r} {py} m {px + r} {py + k} {px + k} {py + r} {px} {py + r} c S")
    c.append("BT /F1 11 Tf %d 70 Td (30'-0\") Tj ET" % (X0 + W / 2 - 18))
    c.append("BT /F1 12 Tf 126 520 Td (MADE-UP PLAN, NOT A REAL JOB.  FLOOR PLAN  1/4\" = 1'-0\") Tj ET")
    return "\n".join(c).encode("latin-1")

def pdf():
    stream = zlib.compress(content())
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 792 612] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
        b"<< /Length %d /Filter /FlateDecode >>\nstream\n" % len(stream) + stream + b"\nendstream",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ]
    out, offsets = b"%PDF-1.4\n", []
    for i, body in enumerate(objects, 1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % i + body + b"\nendobj\n"
    xref = len(out)
    out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objects) + 1)
    for o in offsets:
        out += b"%010d 00000 n \n" % o
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objects) + 1, xref)
    return out

def pictures():
    try:
        from PIL import Image, ImageDraw
    except ImportError:
        print("Pillow is not installed: the PNG and the JPEG were not written again")
        return
    # Two dots to the point: 1584 by 1224. The page's y runs up; a picture's runs down.
    s = 2
    im = Image.new("RGB", (792 * s, 612 * s), "white")
    d = ImageDraw.Draw(im)
    def line(x1, y1, x2, y2, w):
        d.line([(x1 * s, (612 - y1) * s), (x2 * s, (612 - y2) * s)], fill="black", width=w)
    for (x1, y1, x2, y2) in [(X0, Y0, X0 + W, Y0), (X0 + W, Y0, X0 + W, Y0 + H), (X0 + W, Y0 + H, X0, Y0 + H), (X0, Y0 + H, X0, Y0)]:
        line(x1, y1, x2, y2, 3)
    for l in LINES:
        line(*l, 2)
    im.save(os.path.join(OUT, "made-up-plan.png"), optimize=True)
    im.save(os.path.join(OUT, "made-up-plan.jpg"), quality=85)

os.makedirs(OUT, exist_ok=True)
open(os.path.join(OUT, "made-up-plan.pdf"), "wb").write(pdf())
pictures()
print("wrote", os.path.normpath(OUT))
