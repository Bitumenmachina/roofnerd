#!/usr/bin/env bash
# Clashes handed over as an issue list (call 228, contract 228M). At the command line. The job is the made-up whole house
# (`plumbline sixteen`), whose clash list says: in the same space 11 (5 findings, 6 to verify in the field), set against 10, through a
# wall or a floor 47 holes to cut. By hand (STATUS.md, "Clashes handed over as an issue list"):
#  1. `export bcf` writes 11 issues in the clash list's order; the first is the first-floor trunk and the soil stack, "in each other
#     by 3.50 in" as the clash list writes it.
#  2. 5 say they are findings and 6 are questions for the field, in the order and by the mark the clash list gives each (a "~" is a
#     question).
#  3. Each names two things by id: 22 ids, every one in the BIM file written from the same job, on a thing whose name the clash
#     list's name for it holds.
#  4. Each has a view: the line of sight passes within a foot of the clash's place, and the eye is more than 6 ft from it (that it is
#     outside both things' boxes is held by crates/ifc/tests/call228m.rs).
#  5. With --holes: 58 issues, 11 and 47, the 47 of their own kind; without, none is.
#  6. The file passes buildingSMART's own schema, by the outside checker (tools/bcf-check.py; lxml; the schema files are fetched
#     once by that script and are not kept in the repository). Where the checker cannot be run this says so and stops with 2.
#  7. Written twice (different seconds), the two files differ only in the time of writing.
#  8. A job with no clash: the file is written with no issue, and the reply says there was none to hand over.
#  9. `plumbline sixteen` writes the job it wrote, byte for byte (sha256 beginning 880d9fd3c364b5f7); exporting changes neither the
#     job, the clash list nor the verify-in-field list.
# Run from the repository root:  demos/bcf.sh
# Every figure is made up. Every step fails the script if the program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
fail() { echo "FAIL: $*"; exit 1; }
command -v python3 >/dev/null || { echo "demos/bcf.sh needs python3"; exit 2; }

"$P" sixteen "$T/h.json" >/dev/null
sha256sum "$T/h.json" | grep -q '^880d9fd3c364b5f7' || fail "the sample whole house moved: $(sha256sum "$T/h.json")"
"$P" clashes "$T/h.json" > "$T/clashes.before"
"$P" vif "$T/h.json" > "$T/vif.before"

echo "== 1. Eleven issues, in the clash list's order =="
"$P" export bcf "$T/h.json" "$T/h.bcfzip" | tee "$T/said.txt"
grep -q "11 issues from the clashes of " "$T/said.txt" || fail "the reply does not say 11 issues"
grep -q "in the same space: 11 (5 findings, 6 to verify in the field)" "$T/said.txt" || fail "the reply does not say 5 and 6"
grep -q "not handed over: 47 holes to cut" "$T/said.txt" || fail "the reply does not say the 47 holes were left"
grep -q "not handed over: 10 set against a wall" "$T/said.txt" || fail "the reply does not say the 10 set against were left"
grep -q "first-floor trunk, west, piece 2 of 3 \[duct-0100\] and soil stack \[pipe-0246\]: in each other by 3.50 in" "$T/clashes.before" || fail "the clash list does not say the trunk and the stack"

# Read the file as the other program would: the folders, each issue's title, label, things and view, in the file's own order.
cat > "$T/read.py" <<'PY'
import json, re, sys, zipfile
z = zipfile.ZipFile(sys.argv[1])
names = z.namelist()
assert names[0] == "bcf.version", names[:2]
out = []
for n in names:
    if not n.endswith("/markup.bcf"):
        continue
    m = z.read(n).decode()
    v = z.read(n.replace("markup.bcf", "viewpoint.bcfv")).decode()
    num = lambda tag: [float(x) for x in re.search(f"<{tag}>\\s*<X>(.*?)</X>\\s*<Y>(.*?)</Y>\\s*<Z>(.*?)</Z>", v, re.S).groups()]
    out.append({
        "title": re.search("<Title>(.*?)</Title>", m).group(1),
        "label": re.search("<Labels>(.*?)</Labels>", m).group(1),
        "type": re.search('TopicType="(.*?)"', m).group(1),
        "description": re.search("<Description>(.*?)</Description>", m, re.S).group(1),
        "ids": re.findall('IfcGuid="(.{22})"', v),
        "eye": num("CameraViewPoint"), "dir": num("CameraDirection"),
    })
json.dump(out, open(sys.argv[2], "w"))
PY
python3 -I "$T/read.py" "$T/h.bcfzip" "$T/issues.json"
python3 -I - "$T/issues.json" <<'PY' || exit 1
import json, sys
i = json.load(open(sys.argv[1]))
assert len(i) == 11, len(i)
assert i[0]["title"] == "first-floor trunk, west, piece 2 of 3 and soil stack: in each other by 3.50 in", i[0]["title"]
PY

echo "== 2. Five findings and six questions for the field, as the clash list marks them =="
# The clash list's lines under "In the same space", each starting "~" (a question) or blank (a finding).
awk '/^In the same space:/{on=1;next} /^Set against/{on=0} on{print substr($0,3,1)}' "$T/clashes.before" > "$T/marks.txt"
[ "$(wc -l < "$T/marks.txt")" = 11 ] || fail "the clash list does not have 11 lines"
python3 -I - "$T/issues.json" "$T/marks.txt" <<'PY' || exit 1
import json, sys
issues = json.load(open(sys.argv[1])); marks = open(sys.argv[2]).read().split("\n")[:11]
f = [x["label"] for x in issues]
assert f.count("Finding") == 5 and f.count("To verify in the field") == 6, f
for n, (x, m) in enumerate(zip(issues, marks)):
    want = "To verify in the field" if m == "~" else "Finding"
    assert x["label"] == want, (n, x["label"], m)
    assert ("A question for the field" in x["description"]) == (m == "~"), n
    assert ("A finding" in x["description"]) == (m != "~"), n
print("5 findings, 6 questions, each the clash list's own mark")
PY

echo "== 3. Every id is the BIM file's, on the same thing =="
"$P" export ifc "$T/h.json" "$T/h.ifc" >/dev/null
python3 -I - "$T/issues.json" "$T/h.ifc" <<'PY' || exit 1
import json, re, sys
issues = json.load(open(sys.argv[1])); ifc = open(sys.argv[2]).read().split("\n")
by_id = {}
for line in ifc:
    m = re.match(r"#\d+=IFC\w+\('(.{22})',\$,('(?:[^']|'')*'|\$)", line)
    if m:
        by_id[m.group(1)] = m.group(2).strip("'").replace("''", "'")
ids = [d for x in issues for d in x["ids"]]
assert len(ids) == 22, len(ids)
for x in issues:
    for d in x["ids"]:
        assert d in by_id, f"{d} is not in the BIM file"
        # The file names a thing by its own name; the clash list says it with its size and its type. The file's name is in the title.
        assert by_id[d].casefold() in x["title"].casefold() or by_id[d].casefold().replace("water closet ", "water closet (water closet ") in x["title"].casefold(), (d, by_id[d], x["title"])
print(f"{len(ids)} ids, every one in the BIM file on a thing the title names")
PY

echo "== 4. Each has a view of the place =="
python3 -I - "$T/issues.json" "$T/clashes.before" <<'PY' || exit 1
import json, math, re, sys
issues = json.load(open(sys.argv[1])); lines = open(sys.argv[2]).read().split("\n")
rows = []
on = False
for l in lines:
    if l.startswith("In the same space:"): on = True; continue
    if l.startswith("Set against"): break
    if on:
        m = re.search(r", at \(([-\d.]+), ([-\d.]+), ([-\d.]+)\)", l)
        rows.append([float(g) * 0.3048 for g in m.groups()])
assert len(rows) == 11
for x, at in zip(issues, rows):
    to = [a - e for a, e in zip(at, x["eye"])]
    d = x["dir"]; along = sum(t * k for t, k in zip(to, d))
    assert abs(sum(k * k for k in d) - 1) < 1e-4
    off = math.dist([t - along * k for t, k in zip(to, d)], [0, 0, 0])
    assert along > 6 * 0.3048, (x["title"], along)
    assert off < 0.3048, (x["title"], off)
print("11 views: the line of sight passes within a foot of the place, from more than 6 ft away")
PY

echo "== 5. Holes only when asked =="
"$P" export bcf "$T/h.json" "$T/holes.bcfzip" --holes | tee "$T/said-holes.txt"
grep -q "58 issues" "$T/said-holes.txt" || fail "58 issues not said"
python3 -I "$T/read.py" "$T/holes.bcfzip" "$T/holes.json"
python3 -I - "$T/holes.json" "$T/issues.json" <<'PY' || exit 1
import json, sys
h = json.load(open(sys.argv[1])); p = json.load(open(sys.argv[2]))
assert len(h) == 58 and sum(x["type"] == "Hole to cut" for x in h) == 47 and sum(x["type"] == "Clash" for x in h) == 11
assert all(x["type"] == "Clash" for x in p)
assert [x["title"] for x in h[:11]] == [x["title"] for x in p]
print("58 = 11 + 47, the 47 of their own kind; without --holes none is")
PY

echo "== 6. buildingSMART's own schema, by the outside checker =="
set +e
python3 tools/bcf-check.py "$T/holes.bcfzip" > "$T/checked.txt"; rc=$?
set -e
cat "$T/checked.txt"
case $rc in
  0) python3 tools/bcf-check.py "$T/h.bcfzip" | tail -1 ;;
  2) echo "demos/bcf.sh could not check the file against buildingSMART's schema: the checker says why above. This step is NOT passed."; exit 2 ;;
  *) fail "the outside checker finds a fault" ;;
esac

echo "== 7. Written twice, they differ only in the time =="
sleep 2
"$P" export bcf "$T/h.json" "$T/h2.bcfzip" >/dev/null
python3 -I - "$T/h.bcfzip" "$T/h2.bcfzip" <<'PY' || exit 1
import re, sys, zipfile
a, b = zipfile.ZipFile(sys.argv[1]), zipfile.ZipFile(sys.argv[2])
assert a.namelist() == b.namelist()
t = re.compile(r"\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ")
n = 0
for name in a.namelist():
    x, y = a.read(name).decode(), b.read(name).decode()
    if x != y:
        n += 1
    assert t.sub("T", x) == t.sub("T", y), name
times = {m for name in a.namelist() for m in t.findall(a.read(name).decode())} | {m for name in b.namelist() for m in t.findall(b.read(name).decode())}
assert len(times) == 2, times
print(f"{len(a.namelist())} files; {n} differ, in the time of writing and nothing else")
PY

echo "== 8. A job with no clash =="
"$P" sample "$T/s.json" >/dev/null
"$P" export bcf "$T/s.json" "$T/s.bcfzip" | tee "$T/said-none.txt"
grep -q "there was no clash in the same space to hand over" "$T/said-none.txt" || fail "the reply does not say there was none"
python3 -I - "$T/s.bcfzip" <<'PY' || exit 1
import zipfile, sys
n = zipfile.ZipFile(sys.argv[1]).namelist()
assert n == ["bcf.version"], n
PY
python3 tools/bcf-check.py "$T/s.bcfzip" | tail -1

echo "== 9. The sample whole house does not move =="
sha256sum "$T/h.json" | grep -q '^880d9fd3c364b5f7' || fail "exporting changed the job"
"$P" clashes "$T/h.json" | cmp - "$T/clashes.before" || fail "the clash list changed"
"$P" vif "$T/h.json" | cmp - "$T/vif.before" || fail "the verify-in-field list changed"
"$P" sixteen "$T/h2.json" >/dev/null; sha256sum "$T/h2.json" | grep -q '^880d9fd3c364b5f7' || fail "sixteen writes another job"
echo "demos/bcf.sh: all nine steps pass"
