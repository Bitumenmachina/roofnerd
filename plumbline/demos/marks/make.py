#!/usr/bin/env python3
"""Writes the made-up files demos/marks.sh reads (call 227). Run from the repository root:  python3 demos/marks/make.py

Everything is made up. No file written by a marking program was read to write this: the marks are written to the PDF
standard's own notes (ISO 32000-1, 12.5 annotations and 12.9 measure dictionaries), by hand, with no library.

The plan is the made-up plan of tools/made-up-plan.py: a house 30 ft by 20 ft drawn at 1/4 in to the foot, so 18 points to the
foot, its lower-left corner at (126, 126) on a letter sheet laid on its side (792 by 612 points).

The marks of marked-plan.pdf, in the page's order (each a note of the page, its points in the page's own units):
  South wall      a line (Line) along the south wall, 540 points = 30 ft; its own text says 30'-0"
  Outside walls   a run of lines (PolyLine) round the plan, 1,800 points = 100 ft; its text says 100 LF
  Whole floor     a closed shape (Polygon) over the plan, 540 x 360 points = 600 SF; its text says 600 SF
  Bedroom         a closed shape 216 x 180 points = 12 ft x 10 ft = 120 SF; its text says 120 SF
  Receptacle      four stamps (Stamp), a symbol each, all labelled Receptacle: a count of 4 (the standard has no mark for a
                  count: research/pdf-marks.md says how counts are read, and that it is unconfirmed)
  verify header   a note (Text) that measures nothing
  North wall      a line along the north wall, 540 points = 30 ft, whose own text says 31'-0": the two disagree by 1 LF
Each measured mark carries a measure dictionary: the scale 1/4" = 1'-0", which is 1/18 ft to the point.

Variants: floor-650.pdf (Whole floor says 650 SF), no-bedroom.pdf (Bedroom taken out), unreadable.pdf (a mark with a curved
edge, a mark with no scale and a mark in metres), viewport.pdf (a scale set for part of the sheet; a mark with none of its
own). The TIFFs: made-up-plan.tif (the PNG of tools/made-up-plan.py packed as a TIFF with deflate), two-pages.tif (that
sheet and then a smaller one), old-jpeg.tif (a TIFF whose own note says it is packed as old-style JPEG, which is not read).
"""
import os, struct, zlib

HERE = os.path.dirname(os.path.abspath(__file__))
PLANS = os.path.join(HERE, "..", "plans")
X0, Y0, W, H = 126, 126, 540, 360

MEASURE = ("/Measure << /Type /Measure /Subtype /RL /R (1/4\" = 1'-0\") /X [<< /Type /NumberFormat /U (ft) /C 0.0555555556 >>] "
           "/D [<< /Type /NumberFormat /U (ft) /C 1 >>] /A [<< /Type /NumberFormat /U (sq ft) /C 1 >>] >>")


def line(rect, pts, subj, text, nm, measure=MEASURE):
    return f"/Subtype /Line /Rect [{rect}] /L [{pts}] /Subj ({subj}) /Contents ({text}) /NM ({nm}) {measure}"


def shape(sub, pts, subj, text, nm, measure=MEASURE):
    xs, ys = pts[0::2], pts[1::2]
    rect = f"{min(xs)} {min(ys)} {max(xs)} {max(ys)}"
    v = " ".join(str(p) for p in pts)
    return f"/Subtype /{sub} /Rect [{rect}] /Vertices [{v}] /Subj ({subj}) /Contents ({text}) /NM ({nm}) {measure}"


def stamp(x, y):
    return f"/Subtype /Stamp /Rect [{x} {y} {x + 12} {y + 12}] /Subj (Receptacle)"


OUTLINE = [X0, Y0, X0 + W, Y0, X0 + W, Y0 + H, X0, Y0 + H]


def marks(floor="600 SF", bedroom=True):
    m = [
        line(f"{X0} {Y0 - 6} {X0 + W} {Y0 + 6}", f"{X0} {Y0} {X0 + W} {Y0}", "South wall", "30'-0\"", "mk-south"),
        shape("PolyLine", OUTLINE + [X0, Y0], "Outside walls", "100 LF", "mk-outside"),
        shape("Polygon", OUTLINE, "Whole floor", floor, "mk-floor"),
    ]
    if bedroom:
        m.append(shape("Polygon", [X0, Y0, X0 + 216, Y0, X0 + 216, Y0 + 180, X0, Y0 + 180], "Bedroom", "120 SF", "mk-bedroom"))
    m += [stamp(200, 300), stamp(300, 300), stamp(400, 300), stamp(500, 300)]
    m.append("/Subtype /Text /Rect [560 520 580 540] /Contents (verify header) /NM (mk-note)")
    m.append(line(f"{X0} {Y0 + H - 6} {X0 + W} {Y0 + H + 6}", f"{X0} {Y0 + H} {X0 + W} {Y0 + H}", "North wall", "31'-0\"", "mk-north"))
    return m


def content():
    c = ["1.2 w 0 0 0 RG", f"{X0} {Y0} m {X0 + W} {Y0} l {X0 + W} {Y0 + H} l {X0} {Y0 + H} l h S", "0.6 w",
         f"{X0 + 216} {Y0} m {X0 + 216} {Y0 + H} l S",
         "BT /F1 12 Tf 126 520 Td (MADE-UP PLAN, NOT A REAL JOB.  MARKED UP  1/4\" = 1'-0\") Tj ET"]
    return "\n".join(c).encode("latin-1")


def pdf(annots, page_extra=""):
    stream = zlib.compress(content())
    first = 6
    refs = " ".join(f"{first + i} 0 R" for i in range(len(annots)))
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 792 612] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> /Annots [{refs}] {page_extra} >>".encode(),
        b"<< /Length %d /Filter /FlateDecode >>\nstream\n" % len(stream) + stream + b"\nendstream",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ] + [f"<< /Type /Annot {a} >>".encode("latin-1") for a in annots]
    out, offsets = b"%PDF-1.7\n", []
    for i, body in enumerate(objects, 1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % i + body + b"\nendobj\n"
    xref = len(out)
    out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objects) + 1)
    for o in offsets:
        out += b"%010d 00000 n \n" % o
    out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objects) + 1, xref)
    return out


# ---- TIFF, written by hand: little-endian, strips of deflate (or what the test says) ----

def png_rgb(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    pos, idat, w = 8, b"", None
    while pos < len(data):
        n, kind = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + n]
        if kind == b"IHDR":
            w, h, depth, ctype, _, _, inter = struct.unpack(">IIBBBBB", body)
            assert (depth, ctype, inter) == (8, 2, 0), "only 8-bit RGB, not interlaced"
        elif kind == b"IDAT":
            idat += body
        pos += 12 + n
    raw, stride, rows, prev = zlib.decompress(idat), w * 3, [], bytearray(w * 3)
    for y in range(h):
        f, row = raw[y * (stride + 1)], bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a = row[i - 3] if i >= 3 else 0
            b = prev[i]
            c = prev[i - 3] if i >= 3 else 0
            if f == 1: row[i] = (row[i] + a) & 255
            elif f == 2: row[i] = (row[i] + b) & 255
            elif f == 3: row[i] = (row[i] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                row[i] = (row[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(bytes(row))
        prev = row
    return w, h, b"".join(rows)


def tiff(pages, compression=8):
    """pages: [(w, h, rgb bytes)]. compression 8 deflate (read), 6 old-style JPEG (not read: only the note is written)."""
    out = bytearray(b"II*\x00\x00\x00\x00\x00")
    next_at = 4
    for w, h, rgb in pages:
        strip = zlib.compress(rgb) if compression == 8 else rgb
        strip_at = len(out)
        out += strip
        if len(out) % 2: out += b"\x00"
        bits_at = len(out)
        out += struct.pack("<3H", 8, 8, 8)
        entries = [(256, 4, 1, w), (257, 4, 1, h), (258, 3, 3, bits_at), (259, 3, 1, compression), (262, 3, 1, 2), (273, 4, 1, strip_at),
                   (277, 3, 1, 3), (278, 4, 1, h), (279, 4, 1, len(strip)), (284, 3, 1, 1)]
        ifd_at = len(out)
        out += struct.pack("<H", len(entries))
        for tag, typ, n, v in entries:
            out += struct.pack("<HHII", tag, typ, n, v) if typ == 4 or tag == 258 else struct.pack("<HHIHH", tag, typ, n, v, 0)
        struct.pack_into("<I", out, next_at, ifd_at)
        next_at = len(out)
        out += b"\x00\x00\x00\x00"
    return bytes(out)


def second_sheet():
    # A smaller second sheet, 800 by 600: white with a black frame and a diagonal. Made up.
    w, h = 800, 600
    rows = []
    for y in range(h):
        row = bytearray(b"\xff" * (w * 3))
        for x in range(w):
            if x < 4 or y < 4 or x >= w - 4 or y >= h - 4 or abs(x * h // w - y) < 2:
                row[x * 3:x * 3 + 3] = b"\x00\x00\x00"
        rows.append(bytes(row))
    return w, h, b"".join(rows)


def write(name, data):
    open(os.path.join(HERE, name), "wb").write(data)


def main():
    write("marked-plan.pdf", pdf(marks()))
    write("floor-650.pdf", pdf(marks(floor="650 SF")))
    write("no-bedroom.pdf", pdf(marks(bedroom=False)))
    extra = [
        # a mark with a curved edge (a circle), and it says a figure
        f"/Subtype /Circle /Rect [200 200 260 260] /Subj (Round bed) /Contents (28 SF) /NM (mk-round) {MEASURE}",
        # a mark with no scale of its own, on a page with none
        f"/Subtype /Polygon /Rect [200 200 300 300] /Vertices [200 200 300 200 300 300] /Subj (Loose area) /Contents (80 SF) /NM (mk-loose)",
        # a figure in metres
        line("0 0 1 1", "126 126 666 126", "Metric run", "9 m", "mk-metric"),
    ]
    write("unreadable.pdf", pdf(marks()[:1] + extra))
    vp = "/VP [<< /Type /Viewport /BBox [0 0 400 400] /Name (Plan) /Measure << /Type /Measure /Subtype /RL /R (x) /X [<< /U (ft) /C 0.0555555556 >>] /D [<< /U (ft) /C 1 >>] /A [<< /U (sq ft) /C 1 >>] >> >>]"
    write("viewport.pdf", pdf(marks()[:1] + [f"/Subtype /Polygon /Rect [200 200 300 300] /Vertices [200 200 300 200 300 300] /Subj (In a part) /Contents (80 SF) /NM (mk-part)"], vp))
    w, h, rgb = png_rgb(os.path.join(PLANS, "made-up-plan.png"))
    write("made-up-plan.tif", tiff([(w, h, rgb)]))
    write("two-pages.tif", tiff([(w, h, rgb), second_sheet()]))
    write("old-jpeg.tif", tiff([(64, 48, bytes(64 * 48 * 3))], compression=6))
    print("wrote", os.path.normpath(HERE))


main()
