#!/usr/bin/env python3
"""Writes the made-up tables the import demonstration brings in, into demos/tables:

  tinners-yard.csv   a supplier's price list as that supplier might send it: its name
                     and a title above the headings, its own headings, five rows
  tinners-yard.xlsx  the same list as a workbook, written by hand below (a zip of XML,
                     no library): one price is a formula with the figure it last worked
                     out to beside it, as a spreadsheet saves it, and there is a second
                     sheet, "Terms"
  prior-estimate.csv a made-up earlier estimate of four lines
  hidden-and-shown.xlsx  a workbook as a supplier's office leaves one: its first sheet,
                     "Last year", is hidden and holds a stale price; the sheet on show,
                     "Prices", starts at its fifth row, and has a price cell shown as a
                     percentage (9.05%, kept as 0.0905) and one that is a date

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
    if isinstance(value, dict):  # a figure shown a certain way: its style's number in styles.xml
        return '<c r="%s" s="%d"><v>%s</v></c>' % (ref, value["s"], value["v"])
    if isinstance(value, (int, float)):
        return '<c r="%s"><v>%s</v></c>' % (ref, value)
    return '<c r="%s" t="inlineStr"><is><t>%s</t></is></c>' % (ref, xml(value))


def sheet(rows, first=1):
    body = "".join('<row r="%d">%s</row>' % (r + first, "".join(cell(c, r + first, v) for c, v in enumerate(row) if v != "")) for r, row in enumerate(rows))
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


def hidden_and_shown(path):
    old = [HEAD[:3], ["Copper, 16 oz", "lb", 1.11]]
    # Style 1 shows a figure as a percentage, style 2 as a date.
    shown = [HEAD[:3], ["Copper, 16 oz", "lb", 9.05], ["Copper, 20 oz", "lb", {"v": 0.0905, "s": 1}], ["Zinc, 0.7 mm", "lb", {"v": 46100, "s": 2}]]
    ct = '<Override PartName="/xl/%s" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.%s+xml"/>'
    parts = {
        "[Content_Types].xml": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>' + ct % ("workbook.xml", "sheet.main") + ct % ("worksheets/sheet1.xml", "worksheet") + ct % ("worksheets/sheet2.xml", "worksheet") + ct % ("styles.xml", "styles") + "</Types>",
        "_rels/.rels": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>',
        "xl/workbook.xml": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Last year" sheetId="1" state="hidden" r:id="rId1"/><sheet name="Prices" sheetId="2" r:id="rId2"/></sheets></workbook>',
        "xl/_rels/workbook.xml.rels": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>',
        "xl/styles.xml": '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts><fills count="1"><fill><patternFill patternType="none"/></fill></fills><borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs><cellXfs count="3"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/><xf numFmtId="10" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/><xf numFmtId="14" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/></cellXfs></styleSheet>',
        "xl/worksheets/sheet1.xml": sheet(old),
        "xl/worksheets/sheet2.xml": sheet(shown, first=5),
    }
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, text in parts.items():
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, text)


os.makedirs(OUT, exist_ok=True)
hidden_and_shown(os.path.join(OUT, "hidden-and-shown.xlsx"))
with open(os.path.join(OUT, "tinners-yard.csv"), "w", newline="") as f:
    f.write(csv_text([r + [""] * (4 - len(r)) for r in ABOVE] + [HEAD] + ROWS))
workbook(os.path.join(OUT, "tinners-yard.xlsx"))
with open(os.path.join(OUT, "prior-estimate.csv"), "w", newline="") as f:
    f.write(csv_text(PRIOR))
print("wrote", ", ".join(sorted(os.listdir(OUT))), "in demos/tables")
