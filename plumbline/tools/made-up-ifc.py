#!/usr/bin/env python3
"""Writes demos/ifc/made-up-house.ifc: a small made-up house as another trade's or a
designer's model would arrive, in IFC4, written by hand below (no library).

It is in METRES, as most such models are; the house is sized so its figures are round in
feet (0.3048 m to the foot):

  one storey, "Ground", floor at 0, and a "Roof" level 2.4384 m (8 ft) up
  four outside walls round 9.144 m by 6.096 m (30 ft by 20 ft), 0.1524 m (6 in) thick,
    2.4384 m (8 ft) tall, each drawn on its centre line
  a door 0.9144 m by 2.1336 m (3 ft by 7 ft) in the south wall, its middle 3.048 m (10 ft)
    along it, and a window 1.2192 m by 0.9144 m (4 ft by 3 ft) in the east wall, its
    middle 3.048 m (10 ft) along it, its sill 0.9144 m (3 ft) up
  a floor slab 9.144 m by 6.096 m, 0.1016 m (4 in) thick, its top at the floor
  one room, "Great room", its middle at (4.572, 3.048) m: (15, 10) ft
  a round supply duct 0.1524 m (6 in) across, 6.096 m (20 ft) long, going east from
    (1.524, 3.048) m at 2.1336 m (7 ft) up, on a system "Supply air"
  a cold water pipe 0.0254 m (1 in) across, 3.048 m (10 ft) long, going north from
    (3.048, 1.524) m at the same height: it crosses the duct
  and three things this program has no element for: a beam, a stair and a chair

Nothing in it is a real job's. Run from the repository root:  tools/made-up-ifc.py
"""
import hashlib
import os

OUT = os.path.join(os.path.dirname(__file__), "..", "demos", "ifc", "made-up-house.ifc")
L, W, T, H = 9.144, 6.096, 0.1524, 2.4384
lines = []
ALPHA = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$"


def add(text):
    lines.append(text)
    return "#%d" % len(lines)


def guid():
    # 22 characters of IFC's own alphabet, the first of them 0 to 3, different for every
    # entity and the same on every run.
    digest = hashlib.sha256(b"made-up-house %d" % (len(lines) + 1)).digest()
    return "'%s%s'" % (ALPHA[digest[0] % 4], "".join(ALPHA[b % 64] for b in digest[1:22]))


def pt(*c):
    return add("IFCCARTESIANPOINT((%s))" % ",".join("%r" % float(x) for x in c))


def direction(*c):
    return add("IFCDIRECTION((%s))" % ",".join("%r" % float(x) for x in c))


def axes(origin=(0, 0, 0), z=None, x=None):
    return add("IFCAXIS2PLACEMENT3D(%s,%s,%s)" % (pt(*origin), direction(*z) if z else "$", direction(*x) if x else "$"))


def placed(rel, origin=(0, 0, 0), z=None, x=None):
    return add("IFCLOCALPLACEMENT(%s,%s)" % (rel or "$", axes(origin, z, x)))


def body(profile, depth, position=None, way=(0, 0, 1)):
    solid = add("IFCEXTRUDEDAREASOLID(%s,%s,%s,%r)" % (profile, position or axes(), direction(*way), float(depth)))
    rep = add("IFCSHAPEREPRESENTATION(%s,'Body','SweptSolid',(%s))" % (CONTEXT, solid))
    return add("IFCPRODUCTDEFINITIONSHAPE($,$,(%s))" % rep)


def rect(x, y, at=(0, 0)):
    return add("IFCRECTANGLEPROFILEDEF(.AREA.,$,IFCAXIS2PLACEMENT2D(%s,$),%r,%r)" % (pt(*at), float(x), float(y))) if False else add(
        "IFCRECTANGLEPROFILEDEF(.AREA.,$,%s,%r,%r)" % (add("IFCAXIS2PLACEMENT2D(%s,$)" % pt(*at)), float(x), float(y)))


def circle(r):
    return add("IFCCIRCLEPROFILEDEF(.AREA.,$,%s,%r)" % (add("IFCAXIS2PLACEMENT2D(%s,$)" % pt(0, 0)), float(r)))


add("IFCPERSON($,'Made-up','Estimator',$,$,$,$,$)")
add("IFCORGANIZATION($,'Made-up Builders (not a real firm)',$,$,$)")
add("IFCPERSONANDORGANIZATION(#1,#2,$)")
add("IFCAPPLICATION(#2,'0','tools/made-up-ifc.py','made-up-ifc')")
OWNER = add("IFCOWNERHISTORY(#3,#4,$,.ADDED.,$,$,$,0)")
UNITS = add("IFCUNITASSIGNMENT((%s))" % add("IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.)"))
CONTEXT = add("IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.E-05,%s,$)" % axes())
PROJECT = add("IFCPROJECT(%s,%s,'Made-up house (not a real job)',$,$,$,$,(%s),%s)" % (guid(), OWNER, CONTEXT, UNITS))
site_at = placed(None)
SITE = add("IFCSITE(%s,%s,'Site',$,$,%s,$,$,.ELEMENT.,$,$,$,$,$)" % (guid(), OWNER, site_at))
building_at = placed(site_at)
BUILDING = add("IFCBUILDING(%s,%s,'House',$,$,%s,$,$,.ELEMENT.,$,$,$)" % (guid(), OWNER, building_at))
ground_at = placed(building_at)
GROUND = add("IFCBUILDINGSTOREY(%s,%s,'Ground',$,$,%s,$,$,.ELEMENT.,0.)" % (guid(), OWNER, ground_at))
roof_at = placed(building_at, (0, 0, H))
ROOF = add("IFCBUILDINGSTOREY(%s,%s,'Roof',$,$,%s,$,$,.ELEMENT.,%r)" % (guid(), OWNER, roof_at, H))
add("IFCRELAGGREGATES(%s,%s,$,$,%s,(%s))" % (guid(), OWNER, PROJECT, SITE))
add("IFCRELAGGREGATES(%s,%s,$,$,%s,(%s))" % (guid(), OWNER, SITE, BUILDING))
add("IFCRELAGGREGATES(%s,%s,$,$,%s,(%s,%s))" % (guid(), OWNER, BUILDING, GROUND, ROOF))

on_ground = []
# The four walls, each placed at its start with its own x along it. Its body is a
# rectangle as long as the wall and as thick, with its middle half way along.
walls = {}
for name, start, way, length in [("South wall", (0, 0), (1, 0), L), ("East wall", (L, 0), (0, 1), W), ("North wall", (L, W), (-1, 0), L), ("West wall", (0, W), (0, -1), W)]:
    at = placed(ground_at, (start[0], start[1], 0), x=(way[0], way[1], 0))
    shape = body(rect(length, T, (length / 2, 0)), H)
    walls[name] = (add("IFCWALL(%s,%s,'%s',$,$,%s,%s,$,.STANDARD.)" % (guid(), OWNER, name, at, shape)), at)
    on_ground.append(walls[name][0])


def opening(wall, along, width, height, sill, what, name):
    host, host_at = walls[wall]
    # The hole, and what fills it, are placed at the hole's lower corner nearest the
    # wall's start, with x along the wall.
    at = placed(host_at, (along - width / 2, -T / 2, sill))
    hole = add("IFCOPENINGELEMENT(%s,%s,'Opening',$,$,%s,%s,$,.OPENING.)" % (guid(), OWNER, at, body(rect(width, T, (width / 2, T / 2)), height)))
    add("IFCRELVOIDSELEMENT(%s,%s,$,$,%s,%s)" % (guid(), OWNER, host, hole))
    fill_at = placed(at)
    shape = body(rect(width, 0.05, (width / 2, T / 2)), height)
    if what == "IFCDOOR":
        fill = add("IFCDOOR(%s,%s,'%s',$,$,%s,%s,$,%r,%r,.DOOR.,.SINGLE_SWING_LEFT.,$)" % (guid(), OWNER, name, fill_at, shape, float(height), float(width)))
    else:
        fill = add("IFCWINDOW(%s,%s,'%s',$,$,%s,%s,$,%r,%r,.WINDOW.,.SINGLE_PANEL.,$)" % (guid(), OWNER, name, fill_at, shape, float(height), float(width)))
    add("IFCRELFILLSELEMENT(%s,%s,$,$,%s,%s)" % (guid(), OWNER, hole, fill))
    on_ground.append(fill)


opening("South wall", 3.048, 0.9144, 2.1336, 0.0, "IFCDOOR", "Front door")
opening("East wall", 3.048, 1.2192, 0.9144, 0.9144, "IFCWINDOW", "East window")

# The slab: its outline as a closed line, pushed down from the floor by its thickness.
outline = add("IFCPOLYLINE((%s))" % ",".join(pt(x, y) for x, y in [(0, 0), (L, 0), (L, W), (0, W), (0, 0)]))
slab_shape = body(add("IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,$,%s)" % outline), 0.1016, way=(0, 0, -1))
on_ground.append(add("IFCSLAB(%s,%s,'Floor slab',$,$,%s,%s,$,.FLOOR.)" % (guid(), OWNER, placed(ground_at), slab_shape)))

# The room: a space with the inside of the walls as its footprint.
inside = add("IFCPOLYLINE((%s))" % ",".join(pt(x, y) for x, y in [(T / 2, T / 2), (L - T / 2, T / 2), (L - T / 2, W - T / 2), (T / 2, W - T / 2), (T / 2, T / 2)]))
space = add("IFCSPACE(%s,%s,'101',$,$,%s,%s,'Great room',.ELEMENT.,.INTERNAL.,$)" % (guid(), OWNER, placed(ground_at), body(add("IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,$,%s)" % inside), H)))
add("IFCRELAGGREGATES(%s,%s,$,$,%s,(%s))" % (guid(), OWNER, GROUND, space))

# The duct and the pipe: each a round shape pushed along its own z, which is laid along
# the run by its placement.
duct = add("IFCDUCTSEGMENT(%s,%s,'Supply duct',$,$,%s,%s,$,.RIGIDSEGMENT.)" % (guid(), OWNER, placed(ground_at, (1.524, 3.048, 2.1336), z=(1, 0, 0), x=(0, 1, 0)), body(circle(0.0762), 6.096)))
pipe = add("IFCPIPESEGMENT(%s,%s,'Cold water pipe',$,$,%s,%s,$,.RIGIDSEGMENT.)" % (guid(), OWNER, placed(ground_at, (3.048, 1.524, 2.1336), z=(0, 1, 0), x=(1, 0, 0)), body(circle(0.0127), 3.048)))
on_ground += [duct, pipe]
air = add("IFCDISTRIBUTIONSYSTEM(%s,%s,'Supply air',$,$,$,.AIRCONDITIONING.)" % (guid(), OWNER))
add("IFCRELASSIGNSTOGROUP(%s,%s,$,$,(%s),$,%s)" % (guid(), OWNER, duct, air))
water = add("IFCDISTRIBUTIONSYSTEM(%s,%s,'Cold water',$,$,$,.DOMESTICCOLDWATER.)" % (guid(), OWNER))
add("IFCRELASSIGNSTOGROUP(%s,%s,$,$,(%s),$,%s)" % (guid(), OWNER, pipe, water))

# Three things this program has no element for.
on_ground.append(add("IFCBEAM(%s,%s,'Ridge beam',$,$,%s,%s,$,.BEAM.)" % (guid(), OWNER, placed(ground_at, (0, 3.048, 2.3), z=(1, 0, 0), x=(0, 1, 0)), body(rect(0.1, 0.3), L))))
on_ground.append(add("IFCSTAIR(%s,%s,'Attic stair',$,$,%s,%s,$,.STRAIGHT_RUN_STAIR.)" % (guid(), OWNER, placed(ground_at, (7, 4, 0)), body(rect(1.0, 3.0), 2.0))))
on_ground.append(add("IFCFURNITURE(%s,%s,'Chair',$,$,%s,%s,$,.CHAIR.)" % (guid(), OWNER, placed(ground_at, (2, 2, 0)), body(rect(0.5, 0.5), 0.9))))
add("IFCRELCONTAINEDINSPATIALSTRUCTURE(%s,%s,$,$,(%s),%s)" % (guid(), OWNER, ",".join(on_ground), GROUND))

head = "ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION(('ViewDefinition [ReferenceView]'),'2;1');\nFILE_NAME('made-up-house.ifc','2026-01-01T00:00:00',('Made-up'),('Made-up Builders (not a real firm)'),'tools/made-up-ifc.py','tools/made-up-ifc.py','');\nFILE_SCHEMA(('IFC4'));\nENDSEC;\nDATA;\n"
with open(OUT, "w") as f:
    f.write(head + "".join("#%d=%s;\n" % (i + 1, text) for i, text in enumerate(lines)) + "ENDSEC;\nEND-ISO-10303-21;\n")
print("wrote demos/ifc/made-up-house.ifc: %d entities" % len(lines))
