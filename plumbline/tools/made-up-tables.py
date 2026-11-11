#!/usr/bin/env python3
"""Writes the made-up tables the import demonstration brings in, into demos/tables:

  tinners-yard.csv   a supplier's price list as that supplier might send it: its name
                     and a title above the headings, its own headings, five rows
  tinners-yard.xlsx  the same list as a workbook, written by hand below (a zip of XML,
                     no library): one price is a formula with the figure it last worked
                     out to beside it, as a spreadsheet saves it, and there is a second
                     sheet, "Terms"
  prior-estimate.csv a made-up earlier estimate of four lines

The supplier, its prices and the estimate are made up; none is a real job's.
Run from the repository root:  tools/made-up-tables.py
"""
import os, zipfile

OUT = os.path.join(os.path.dirname(__file__), "..", "demos", "tables")
ABOVE = [["Tinners Yard (a made-up supplier)"], ["Price list"]]
HEAD = ["Item", "UOM", "Price", "Lead time"]
ROWS = [
    ["Copper, 16 oz", "lb", "$9.05", "2 weeks"],
    ["copper,20 OZ", "LB", "9.40", "3 weeks"],
    ["Zinc, 0.7 mm", "lb", "3.10", "stock"],
    ["Copper, 16 oz", "sheet", "$95.00", "2 weeks"],
    ["Lead-coated copper", "lb", "call", ""],
]
PRIOR = [
    ["Description", "Quantity", "Unit", "Unit cost", "Amount"],
    ["Standing seam panels", "700", "lb", "8.50", "5950.00"],
    ["Underlayment", "650", "SF", "0.90", "585.00"],
    ["Gutters", "120", "LF", "14.00", ""],  # no amount written: it is worked from the other two
    ["Dumpster", "1", "EA", "450.00", "450.00"],
]


def csv_text(rows):
    def cell(c):
        return '"%s"' % c.replace('"', '""') if any(ch in c for ch in ',"\n') else c
    return "".join(",".join(cell(c) for c in row) + "\n" for row in rows)


def xml(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def cell(col, row, value):
    ref = "%s%d" % ("ABCDEFGH"[col], row)
    if isinstance(value, tuple):  # a formula and what it last worked out to
        return '<c r="%s"><f>%s</f><v>%s</v></c>' % (ref, xml(value[0]), value[1])
    if isinstance(value, (int, float)):
        return '<c r="%s"><v>%s</v></c>' % (ref, value)
    return '<c r="%s" t="inlineStr"><is><t>%s</t></is></c>' % (ref, xml(value))


def sheet(rows):
    body = "".join('<row r="%d">%s</row>' % (r + 1, "".join(cell(c, r + 1, v) for c, v in enumerate(row) if v != "")) for r, row in enumerate(rows))
    return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>%s</sheetData></worksheet>' % body


def workbook(path):
    # Prices as figures, as a workbook holds them; the 20 oz copper's is the 16 oz
    # copper's plus 35 cents, a formula, saved with the 9.4 it last worked out to.
    prices = ABOVE + [HEAD] + [
        ["Copper, 16 oz", "lb", 9.05, "2 weeks"],
        ["copper,20 OZ", "LB", ("C4+0.35", 9.4), "3 weeks"],
        ["Zinc, 0.7 mm", "lb", 3.1, "stock"],
        ["Copper, 16 oz", "sheet", 95, "2 weeks"],
        ["Lead-coated copper", "lb", "call", ""],
    ]
    terms = [["Terms"], ["Net 30. Prices hold for 14 days. (Made up.)"]]
    parts = {
        "[Content_Types].xml": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/><Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>',
        "_rels/.rels": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>',
        "xl/workbook.xml": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Prices" sheetId="1" r:id="rId1"/><sheet name="Terms" sheetId="2" r:id="rId2"/></sheets></workbook>',
        "xl/_rels/workbook.xml.rels": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/></Relationships>',
        "xl/worksheets/sheet1.xml": sheet(prices),
        "xl/worksheets/sheet2.xml": sheet(terms),
    }
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, text in parts.items():
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))  # the same bytes every run
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, text)


os.makedirs(OUT, exist_ok=True)
with open(os.path.join(OUT, "tinners-yard.csv"), "w", newline="") as f:
    f.write(csv_text([r + [""] * (4 - len(r)) for r in ABOVE] + [HEAD] + ROWS))
workbook(os.path.join(OUT, "tinners-yard.xlsx"))
with open(os.path.join(OUT, "prior-estimate.csv"), "w", newline="") as f:
    f.write(csv_text(PRIOR))
print("wrote", ", ".join(sorted(os.listdir(OUT))), "in demos/tables")
