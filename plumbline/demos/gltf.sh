#!/usr/bin/env bash
# The house written out as glTF (call 229, contract 229M). At the command line. The job is the made-up whole house (`plumbline sixteen`):
# 40 ft by 28 ft on its centre lines, the foundation's floor at -4 ft, the first floor at 0, the second at 10 ft. By hand (STATUS.md, "The
# house written out as glTF"):
#  1. `export gltf` writes one .glb; the reply says 236 parts and what is not carried (prices, certainty, the window's patterns, what a
#     thing set down is drawn as, a roof plane's thickness, and the two fittings with no size).
#  2. The count of parts is the count of things the BIM file writes a body or a line for, counted here from the BIM file written from the
#     same job: 238 fittings, runs, fixtures, walls... written less the 2 fittings with no size (no body there either) = 236. (STATUS.md
#     said "the count of things the BIM file writes"; that count has those 2 in it, and the BIM file gives them no shape.)
#  3. In metres: the first floor's walls stand 12.3444 m by 8.6868 m (40.5 ft by 28.5 ft: centre lines and the wall's own thickness),
#     which is at least 12.192 m by 8.534 m (40 ft by 28 ft) and no more than 0.4 m over; the lowest point is the foundation's footing
#     and is at or below -1.2192 m (-4 ft).
#  4. Not mirrored: the front door's part is on the south side of the house's middle (north is -Z in the format, so south is +Z) and the
#     kitchen sink's on the north.
#  5. Each thing the clash list names is a part, under the name the clash list gives it.
#  6. The first floor's south wall has the front door and two windows cut: the wall's box less its part's volume is 35.005 ft3 =
#     0.9912 m3 (3 ft x 6 ft 8 in + 2 x 5 ft x 5 ft, all 0.5 ft through).
#  7. The file is whole by its own table of contents (read here) and passes the Khronos glTF Validator, an outside checker
#     (tools/gltf-check.sh); where it is not installed this says "not checked" and does not call it passed.
#  8. `plumbline sixteen` writes the job it wrote, byte for byte (sha256 beginning 880d9fd3c364b5f7), and exporting changes neither it nor
#     the BIM file.
# Run from the repository root:  demos/gltf.sh
# Every figure is made up. Every step fails the script if the program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
fail() { echo "FAIL: $*"; exit 1; }
command -v python3 >/dev/null || { echo "demos/gltf.sh needs python3"; exit 2; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }

"$P" sixteen "$T/house.json" >/dev/null
sha256sum "$T/house.json" | cut -c1-16 > "$T/sum.txt"

echo "== 1. The house written out =="
"$P" export gltf "$T/house.json" "$T/house.glb" > "$T/out.txt"
says "$T/out.txt" "as 236 parts in one glTF file, in metres"
says "$T/out.txt" "20 walls, 17 windows"
says "$T/out.txt" "not carried:"
says "$T/out.txt" "prices, and how sure each value is (Known, Assumed or Unknown)"
says "$T/out.txt" "the patterns the window draws (lap siding, brick, shingles)"
says "$T/out.txt" "what a thing set down is drawn as in the window"
says "$T/out.txt" "hot to the kitchen sink, elbow 1: it has no shape"
says "$T/out.txt" "hot to the kitchen sink, elbow 2: it has no shape"
says "$T/out.txt" "a roof plane has no thickness"

echo "== 2. The count of parts is the count of things the BIM file writes a body or a line for =="
"$P" export ifc "$T/house.json" "$T/before.ifc" > "$T/ifc.txt"
BIM="$(python3 -I - "$T/before.ifc" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
def top_args(body):
    out, depth, cur, quote = [], 0, "", False
    for c in body:
        if c == "'" : quote = not quote
        if not quote and c == "(": depth += 1
        if not quote and c == ")": depth -= 1
        if not quote and depth == 0 and c == ",":
            out.append(cur); cur = ""
        else: cur += c
    out.append(cur); return out
n = 0
for m in re.finditer(r"^#\d+=(IFC[A-Z0-9]+)\((.*)\);$", text, re.M):
    name, body = m.groups()
    if name == "IFCOPENINGELEMENT": continue
    a = top_args(body)
    if len(a) >= 7 and a[5].startswith("#") and a[6].startswith("#"): n += 1
print(n)
PY
)"
[ "$BIM" = 236 ] || fail "the BIM file has $BIM things with a body or a line, not 236"
says "$T/ifc.txt" "41 fittings"
echo "    the BIM file writes 236 things with a body or a line (41 fittings written, 2 of them with no size and so no shape); the glTF has 236 parts"

echo "== 3 to 6, 7 (its own table of contents). The file read back =="
python3 -I - "$T/house.glb" "$T/clashes.txt" "$P" "$T/house.json" <<'PY' || exit 1
import json, struct, subprocess, sys, re
path, _, prog, job = sys.argv[1:5]
b = open(path, "rb").read()
def bad(m): print("FAIL:", m); sys.exit(1)
if b[:4] != b"glTF" or struct.unpack("<I", b[4:8])[0] != 2: bad("not a glb of version 2")
if struct.unpack("<I", b[8:12])[0] != len(b): bad("the length in the header is not the file's")
jl = struct.unpack("<I", b[12:16])[0]
if b[16:20] != b"JSON" or jl % 4: bad("the first chunk is not the JSON, or is not whole words")
j = json.loads(b[20:20 + jl]); at = 20 + jl
bl = struct.unpack("<I", b[at:at + 4])[0]
if b[at + 4:at + 8] != b"BIN\0" or bl % 4 or at + 8 + bl != len(b): bad("the second chunk is not the binary, or does not end the file")
bin_ = b[at + 8:]
if j["buffers"][0]["byteLength"] > len(bin_): bad("the buffer is longer than its chunk")
for v in j["bufferViews"]:
    if v["byteOffset"] + v["byteLength"] > j["buffers"][0]["byteLength"]: bad("a view past the buffer")
acc = j["accessors"]
for a in acc:
    v = j["bufferViews"][a["bufferView"]]
    size = 4 * {"SCALAR": 1, "VEC3": 3}[a["type"]]
    stride = v.get("byteStride", size)
    if a.get("byteOffset", 0) + (a["count"] - 1) * stride + size > v["byteLength"]: bad("an accessor past its view")
print("    every length the file's own table of contents gives is there:", len(j["nodes"]), "nodes,", len(acc), "accessors,", len(bin_), "bytes of data")
def pts(i):
    a = acc[i]; v = j["bufferViews"][a["bufferView"]]
    o = v["byteOffset"] + a.get("byteOffset", 0); s = v.get("byteStride", 12)
    return [struct.unpack("<3f", bin_[o + k * s:o + k * s + 12]) for k in range(a["count"])]
def idx(i):
    a = acc[i]; v = j["bufferViews"][a["bufferView"]]; o = v["byteOffset"] + a.get("byteOffset", 0)
    return list(struct.unpack("<%dI" % a["count"], bin_[o:o + 4 * a["count"]]))
parent = {}
for n, node in enumerate(j["nodes"]):
    for c in node.get("children", []): parent[c] = n
parts = []
for n, node in enumerate(j["nodes"]):
    if "mesh" not in node: continue
    prim = j["meshes"][node["mesh"]]["primitives"][0]
    p = pts(prim["attributes"]["POSITION"])
    level = j["nodes"][parent[n]]["name"] if n in parent and j["nodes"][parent[n]]["extras"].get("kind") == "levels" else None
    parts.append(dict(name=node["name"], path=node["extras"]["path"], kind=node["extras"]["kind"], level=level, pts=p, prim=prim))
def box(p): return [min(q[k] for q in p) for k in range(3)], [max(q[k] for q in p) for k in range(3)]
# 3. metres
ff = [x for x in parts if x["kind"] == "walls" and x["level"] == "first floor"]
lo = [min(box(x["pts"])[0][k] for x in ff) for k in range(3)]; hi = [max(box(x["pts"])[1][k] for x in ff) for k in range(3)]
w, d = hi[0] - lo[0], hi[2] - lo[2]
if abs(w - 12.3444) > 1e-3 or abs(d - 8.6868) > 1e-3: bad("the first floor's walls are %.4f m by %.4f m, not 12.3444 by 8.6868" % (w, d))
if not (12.192 <= w <= 12.592 and 8.534 <= d <= 8.934): bad("not 40 ft by 28 ft and the overhang")
low = min(box(x["pts"])[0][1] for x in parts)
if low > -1.2192 + 1e-4: bad("the lowest point is %.4f m, above -4 ft" % low)
print("    in metres: the first floor's walls %.4f m by %.4f m (40 ft by 28 ft is 12.192 by 8.534; the rest is the walls' own thickness); lowest point %.4f m (-4 ft is -1.2192)" % (w, d, low))
# 4. not mirrored: y north is -Z, so the middle (14 ft) is Z = -4.2672
mid = -14 * 0.3048
z = lambda x: (box(x["pts"])[0][2] + box(x["pts"])[1][2]) / 2
door = next(x for x in parts if x["name"] == "front door")
sink = [x for x in parts if "kitchen sink" in x["name"].lower() and x["kind"] == "fixtures"][0]
if not z(door) > mid: bad("the front door is not on the south side")
if not z(sink) < mid: bad("the kitchen sink is not on the north side")
print("    not mirrored: the front door at Z %.3f m is south of the middle (%.3f), the kitchen sink at Z %.3f m is north of it" % (z(door), mid, z(sink)))
# 5. the clash list's names
text = subprocess.run([prog, "clashes", job], capture_output=True, text=True, check=True).stdout
named = 0
section = False
for line in text.splitlines():
    if not line.startswith(" "): section = line.startswith("In the same space")
    if not section: continue
    m = re.match(r"^\s+(?:~ )?(.+?) \[([a-z]+-\d+)\] and (.+?) \[([a-z]+-\d+)\]:", line)
    if not m: continue
    for name, p in ((m.group(1), m.group(2)), (m.group(3), m.group(4))):
        part = next((x for x in parts if x["path"] == p or x["path"].endswith("/" + p)), None)
        if part is None: bad("%s is in the clash list and not in the file" % p)
        if name != part["name"]: bad("the clash list calls %s '%s', the file '%s'" % (p, name, part["name"]))
        named += 1
if named < 20: bad("only %d names were held to the clash list" % named)
print("    %d names in the clash list, each a part of the file under the same name" % named)
# 6. the holes
south = next(x for x in ff if x["name"] == "south wall")
p = south["pts"]; ix = idx(south["prim"]["indices"])
vol = 0.0
for k in range(0, len(ix), 3):
    a, b2, c = p[ix[k]], p[ix[k + 1]], p[ix[k + 2]]
    vol += (a[0] * (b2[1] * c[2] - b2[2] * c[1]) - a[1] * (b2[0] * c[2] - b2[2] * c[0]) + a[2] * (b2[0] * c[1] - b2[1] * c[0])) / 6
bl_, bh_ = box(p); whole = (bh_[0] - bl_[0]) * (bh_[1] - bl_[1]) * (bh_[2] - bl_[2])
cut = whole - vol
if abs(cut - 0.99123) > 0.005: bad("the openings take %.5f m3 out of the south wall, not 0.99123" % cut)
print("    the south wall's box %.4f m3 less its part %.4f m3 is %.5f m3: the front door and the two windows, cut (35.005 ft3 is 0.99123 m3)" % (whole, vol, cut))
PY

echo "== 7. The format's own checker =="
set +e
bash tools/gltf-check.sh "$T/house.glb" > "$T/check.txt" 2>&1
CODE=$?
set -e
cat "$T/check.txt" | sed 's/^/    /'
case "$CODE" in
  0) grep -q "0 errors, 0 warnings" "$T/check.txt" || fail "the checker passed without saying no error";;
  3) echo "    NOT CHECKED by the format's own checker here (it is not installed); the file was read by this script's reader only";;
  *) fail "the Khronos glTF Validator finds a fault";;
esac

echo "== 8. The sample house, byte for byte; the BIM file as it was =="
"$P" sixteen "$T/again.json" >/dev/null
sha256sum "$T/again.json" | cut -c1-16 > "$T/sum2.txt"
cmp "$T/sum.txt" "$T/sum2.txt" || fail "the sample whole house moved"
grep -q "^880d9fd3c364b5f7" "$T/sum2.txt" || fail "the sample whole house's sum is not 880d9fd3c364b5f7"
cmp "$T/house.json" "$T/again.json" || fail "the job is not byte for byte"
"$P" export ifc "$T/house.json" "$T/before.ifc" > /dev/null
"$P" export ifc "$T/house.json" "$T/after.ifc" > /dev/null
diff <(grep -v '^FILE_NAME' "$T/before.ifc") <(grep -v '^FILE_NAME' "$T/after.ifc") > /dev/null || fail "the BIM file changed"
echo "    sum 880d9fd3c364b5f7, the job and the BIM file as they were"
echo "gltf: every step passed"
