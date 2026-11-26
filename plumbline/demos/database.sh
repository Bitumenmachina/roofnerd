#!/usr/bin/env bash
# "They own their database" (call 223, the command line's half): every table of the job, items, shapes, where the items connect, assemblies
# and prices by supplier, written out as one workbook (a tab to a table) or one folder (a file to a table), and brought back whole, all of
# it or none of it. At the command line, from an empty job, in the seven steps of STATUS.md ("The database in one hand"). The figures the
# program holds on the day: the starter tables are 48 items, 13 shapes (49 rows, a row a block), 5 rows of where items connect (2 items), and
# 42 assemblies in 202 lines; a made-up price list of three suppliers over two materials is 8 rows; one shape and one item of the job's own
# make it 49 items and 14 shapes in 50 rows.
# Run from the repository root:  demos/database.sh
# The job and every figure are made up (no real job data) and live in a temporary folder. Every step fails the script if the program does not
# say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }

# The cells of a workbook, tab by tab, as text a program can compare (what is in each cell, not how the file is packed).
cat > "$T/cells.py" <<'PY'
import sys, zipfile, re
import xml.etree.ElementTree as ET
ns = {"m": "http://schemas.openxmlformats.org/spreadsheetml/2006/main", "r": "http://schemas.openxmlformats.org/officeDocument/2006/relationships"}
z = zipfile.ZipFile(sys.argv[1])
strings = []
if "xl/sharedStrings.xml" in z.namelist():
    for si in ET.fromstring(z.read("xl/sharedStrings.xml")).findall("m:si", ns):
        strings.append("".join(t.text or "" for t in si.iter("{%s}t" % ns["m"])))
book = ET.fromstring(z.read("xl/workbook.xml"))
rels = {r.get("Id"): r.get("Target") for r in ET.fromstring(z.read("xl/_rels/workbook.xml.rels"))}
for sheet in book.find("m:sheets", ns):
    target = rels[sheet.get("{%s}id" % ns["r"])]
    print("TAB", sheet.get("name"))
    for c in ET.fromstring(z.read("xl/" + target.lstrip("/").replace("xl/", ""))).iter("{%s}c" % ns["m"]):
        v = c.find("m:v", ns)
        text = strings[int(v.text)] if c.get("t") == "s" and v is not None else (v.text if v is not None else "")
        print(c.get("r"), repr(text))
PY
cells() { python3 -I "$T/cells.py" "$1"; }
tab_cells() { cells "$1" | awk -v t="$2" '/^TAB /{on=($2==t)} on'; }

echo "== The job: the starter tables, three suppliers' prices, a shape and an item of its own =="
"$P" new "$T/a.json" "Database (not a real job)" >/dev/null
for t in items connections assemblies; do "$P" import $t "$T/a.json" starter >/dev/null; done
cat > "$T/list.csv" <<'ROWS'
Material,Bought by,Price,Supplier,How sure,Where from
Framing stud,EA,3.10,Supplier A,Assumed,a made-up quote
Framing stud,EA,3.25,Supplier B,Assumed,a made-up quote
Framing stud,EA,3.05,Supplier C,Assumed,a made-up quote
Drywall sheet,EA,14.00,Supplier A,Assumed,a made-up quote
Drywall sheet,EA,13.50,Supplier B,Assumed,a made-up quote
Drywall sheet,EA,14.25,Supplier C,Assumed,a made-up quote
ROWS
"$P" import prices "$T/a.json" "$T/list.csv" --add >/dev/null
cat > "$T/shape.csv" <<'ROWS'
Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to
wall-hung closet,porcelain,box,-width / 2,width / 2,0,0.25 * depth,0.4 * height,height
ROWS
cat > "$T/item.csv" <<'ROWS'
Item,Trade,Kind,Drawn as,Width,Depth,Height,How sure,Where from
Wall-hung closet,15 Mechanical,fixture,wall-hung closet,16 in,26 in,30 in,Assumed,a made-up size
ROWS
"$P" import shapes "$T/a.json" "$T/shape.csv" >/dev/null
"$P" import items "$T/a.json" "$T/item.csv" >/dev/null

echo
echo "== 1. Written out as my database: one workbook, a tab to a table, each tab what the table's own writing-out gives =="
echo "      49 items (48 starter and 1 of its own), 50 rows of shapes (14 shapes), 5 rows of connections, 202 rows of assemblies (42), 8 price rows (2 materials by 3 suppliers, and each material's own row)"
"$P" export database "$T/a.json" "$T/my.xlsx" > "$T/out1.txt"
says "$T/out1.txt" "a tab for each table: Shapes 50, Items 49, Connections 5, Assemblies 202, Prices 8"
for t in shapes items connections assemblies prices; do
  "$P" export $t "$T/a.json" "$T/$t.xlsx" >/dev/null
  name="$(printf '%s' "$t" | awk '{print toupper(substr($0,1,1)) substr($0,2)}')"
  # The single table's own workbook has one tab, with the same name: its cells are the database's tab's cells.
  [ "$(tab_cells "$T/my.xlsx" "$name" | tail -n +2)" = "$(tab_cells "$T/$t.xlsx" "$name" | tail -n +2)" ] || { echo "FAIL: the $name tab is not the $t table's own cells"; exit 1; }
  printf '    the %s tab is the %s table, cell for cell\n' "$name" "$t"
done
"$P" export database "$T/a.json" "$T/my" > "$T/out1f.txt"
says "$T/out1f.txt" "a file for each table: Shapes 50, Items 49, Connections 5, Assemblies 202, Prices 8"
for t in shapes items connections assemblies prices; do
  "$P" export $t "$T/a.json" "$T/$t.csv" >/dev/null
  cmp -s "$T/my/$t.csv" "$T/$t.csv" || { echo "FAIL: my/$t.csv is not the $t table written alone"; exit 1; }
  printf '    my/%s.csv is the %s table, byte for byte\n' "$t" "$t"
done

echo
echo "== 2. Brought into a new, empty job: each tab's count is what was written =="
"$P" new "$T/b.json" "Second (not a real job)" >/dev/null
"$P" import database "$T/b.json" "$T/my.xlsx" > "$T/in2.txt"
for line in "(a workbook, 5 tabs): brought in whole" "the tab Shapes: 50 rows under its headings" "the tab Items: 49 rows under its headings" "the tab Connections: 5 rows under its headings" "the tab Assemblies: 202 rows under its headings" "the tab Prices: 8 rows under its headings" "in all: 104 set; no row refused"; do says "$T/in2.txt" "$line"; done
lacks "$T/in2.txt" "not used: 1"

echo
echo "== 3. Written out again from the second job: the same workbook, cell for cell; as a folder, the same files byte for byte =="
"$P" export database "$T/b.json" "$T/my2.xlsx" >/dev/null
[ "$(cells "$T/my.xlsx")" = "$(cells "$T/my2.xlsx")" ] || { echo "FAIL: the second workbook is not the first, cell for cell"; exit 1; }
echo "    the two workbooks are the same, cell for cell ($(cells "$T/my.xlsx" | wc -l) lines of cells)"
"$P" export database "$T/b.json" "$T/my2" >/dev/null
diff -r "$T/my" "$T/my2" >/dev/null || { echo "FAIL: the second folder is not the first"; exit 1; }
echo "    the two folders are the same, file for file, byte for byte"
"$P" new "$T/c.json" "Third (not a real job)" >/dev/null
"$P" import database "$T/c.json" "$T/my" > "$T/in3.txt"
says "$T/in3.txt" "(a folder, 5 files): brought in whole"
"$P" export database "$T/c.json" "$T/my3" >/dev/null
diff -r "$T/my" "$T/my3" >/dev/null || { echo "FAIL: the folder brought in and written out is not the folder"; exit 1; }
echo "    the folder brought into a third job and written out again is the same, byte for byte"

echo
echo "== 4. Brought into a job that already has it all: every row is the same, nothing is set, the job is what it was =="
before="$(cksum < "$T/a.json")"
"$P" import database "$T/a.json" "$T/my.xlsx" > "$T/in4.txt"
says "$T/in4.txt" "in all: 0 set; no row refused"
says "$T/in4.txt" "the same as the job already has: 49"
lacks "$T/in4.txt" "added"
[ "$(cksum < "$T/a.json")" = "$before" ] || { echo "FAIL: the job changed"; exit 1; }
echo "    the job is byte for byte what it was"

echo
echo "== 5. A tab added by hand (Notes) and a column added to Items (My code): both named as not read; everything else comes in =="
mkdir "$T/edited"
cp "$T"/my/*.csv "$T/edited/"
awk -F'\n' 'NR==1{print $0",My code"; next} {print $0",Z-" NR}' "$T/my/items.csv" > "$T/edited/items.csv"
printf 'Remember\ncall the supplier\n' > "$T/edited/notes.csv"
"$P" new "$T/d.json" "Fourth (not a real job)" >/dev/null
"$P" import database "$T/d.json" "$T/edited" > "$T/in5.txt"
says "$T/in5.txt" 'not read: the tab "notes"'
says "$T/in5.txt" 'not read: the column "My code"'
says "$T/in5.txt" "in all: 104 set; no row refused"
"$P" export database "$T/d.json" "$T/my4" >/dev/null
diff -r "$T/my" "$T/my4" >/dev/null || { echo "FAIL: what came in is not what was written"; exit 1; }
echo "    what was read is what was written, and the job has no code of its own or note"

echo
echo "== 6. One bad row (an item with a width that is not a length): nothing at all comes in, and the reply names the tab, the row and why =="
mkdir "$T/bad"
cp "$T"/my/*.csv "$T/bad/"
sed '5s/,\([0-9.]* in\),/,sixteen-ish,/' "$T/my/items.csv" > "$T/bad/items.csv"
cmp -s "$T/my/items.csv" "$T/bad/items.csv" && { echo "FAIL: the bad row was not made"; exit 1; }
"$P" new "$T/e.json" "Fifth (not a real job)" >/dev/null
before="$(cksum < "$T/e.json")"
if "$P" import database "$T/e.json" "$T/bad" > "$T/in6.txt" 2>&1; then echo "FAIL: a database with a bad row was taken"; cat "$T/in6.txt"; exit 1; fi
says "$T/in6.txt" "nothing was brought in"
says "$T/in6.txt" "tab Items, row 5"
says "$T/in6.txt" "sixteen-ish"
says "$T/in6.txt" "the job is as it was"
[ "$(cksum < "$T/e.json")" = "$before" ] || { echo "FAIL: a refused database altered the job"; exit 1; }
echo "    the job is byte for byte what it was: no shape, no item, no price came in"

echo
echo "== 7. The starter database in one bringing-in is the starter tables one by one =="
"$P" new "$T/one.json" "Starter (not a real job)" >/dev/null
for t in items connections assemblies; do "$P" import $t "$T/one.json" starter >/dev/null; done
"$P" new "$T/whole.json" "Starter (not a real job)" >/dev/null
"$P" import database "$T/whole.json" starter > "$T/in7.txt"
says "$T/in7.txt" "the starter database: brought in whole"
says "$T/in7.txt" "the tab Items: 48 rows under its headings"
cmp -s "$T/one.json" "$T/whole.json" || { echo "FAIL: the starter database is not the starter tables"; exit 1; }
echo "    the two jobs are the same, byte for byte"

echo
echo "database.sh: all seven steps passed"
