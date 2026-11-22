# How plumbing, HVAC and electrical would use BuildOS on a renovation with an addition

Written 2026-11-22 by a review agent. Method: the frozen command line (bin17) driven on a made-up
renovation, plus the pictures in demos/captured (trades/3-electrical, clash/04-model read), plus the
earlier review E3 (MEP estimators) checked against today's program. All figures made up; the scratch job
is under review/tu3/ (r.json, go.sh, go2.sh, go3.sh). Nothing in the repository changed except this file.

## The job I built

24 x 16 ft house, 6 in walls, a partition with a 3 ft door, two rooms (all Known, "made-up survey").
Existing, marked `existing`: four outside walls, a 6 in supply duct 11 LF (3,3 to 3,14), a 3 in drain
10 LF (12,2 to 12,12), a cable 20 LF (2,15 to 22,15), a 100 A panelboard, a 40 gal water heater, an air
handler, and a 3/4 in cold line 8 LF crossing the partition at (8,10) to (16,10). The partition and the east wall
marked `demolish`. A 12 ft addition drawn (three walls, a bath partition with a door, a west bath wall, a 4 in slab).
New: 12x8 trunk 14 LF and 6 in ducts (hand: 1.5 + 27 + 3.5 + 14 = 46 LF), 3 in drain and 3/4 in cold (3+18+1+10+5 = 37 LF),
cable 7 + 29 + 5 = 41 LF. Placed: kitchen sink, water closet, lavatory, duplex receptacle, range,
register, an island. Then `clashes`, `clashes --clearance "2 in"`, `vif`, `import assemblies starter`,
`apply`, `set ... unit_cost --line`, `estimate by-item`, `export subcontractor|vif|estimate|ifc`.

Every quantity above agreed with my hand sum (46 / 37 / 41 LF by item). Money: ducts $14/LF = $644,
pipe $9/LF = $333, cable $3/LF = $123, four fixtures at $300 = $1,200, all as Assumed, flagged "rests on
something assumed". Overhead and profit unentered: the price says "no figure" and why. That honesty is right.

---

## What is true of all three trades first

1. Everything MEP is one lump: every starter line is cost type Subcontract. There is no rough / top-out /
   trim split, no labor, no material. A trade estimator prices labor hours and materials; this gives a
   number to multiply by a made-up $/LF.
2. Plumbing and HVAC are both "Division 15". The subcontract export has columns Where, Scope, Quantity,
   Unit, How sure, Note. No division, no system, no existing/new. I cannot hand the plumber his scope and the HVAC
   man his: they are one list of "Piping, installed", "Ductwork, installed", "Fixture, set and connected".
3. Existing vs new is honest in the estimate (existing is not bought again) but the TAKEOFF does not separate
   them: `takeoff` says Supply "6 in round 4 pieces 43.00 LF" = existing 11 + new 32 together, and Power "61 LF" =
   existing 20 + new 41. A demolished cable is still in that 61. The money (estimate) is right; the quantity a
   trade reads is mixed.
4. Existing things that stay are not carried into any scope: "existing to extend / cap / reroute" is not a state.
   Only new, existing, demolish, temporary.
5. Applying "Mechanical equipment" to the existing air handler is accepted with a "!" warning, and then the
   line goes straight into the subcontract export as "Air handler, Equipment, set and connected, 1 EA" - the
   subcontractor is asked to set an air handler that is already there.

## Plumbing

**Works.**
- Runs, fixtures and systems by trade, takeoff by pipe size: `takeoff` gave "Pipe 3 in round 3 pieces 31 LF"
  (hand: 3 + 18 + 10 = 31) for sanitary.
- Setting an existing drain and marking it existing keeps it out of the price: `estimate by-item` piping stayed
  at 37 LF (new only).
- `fitting --all` made bends at the corners; the takeoff then names "Bend 3/4 in elbow: 2 EA".
- `measure` on an existing run says what it left behind: "pipe 1 no longer meets pipe 3, pipe 4: they are where
  they were and did not move with it". Good for a field-measured existing line.
- Water heater vs cold line: "Water heater 40 gal and pipe 5 in each other by 10.80 in" - my routing error,
  correctly found; useful in the design phase.
- A pipe through a new wall: "wall 9 and pipe 7 passes through it (a hole to cut)" - that is a core-drill/bore count.

**Half there.**
- Fittings are not priced: the starter "Pipe run" assembly has no fitting line, so "4 fittings: nothing priced"
  shows up as a warning in the recap. One $/EA fitting for every size is what a plumber would reject.
- "Verify in field" is made for existing conditions, which is right, but on a drawn-not-built pipe it says
  "Measure either one".
- Clearances: `clashes --clearance "2 in"` lists the water heater and cold line at 0.82 in; fine for insulation
  but there is no code clearance (WC 15 in side / 21 in front).

**Missing.**
- Pipe material (PEX, copper, PVC DWV, cast iron) and fixture units. A type is only a diameter ("3 in round").
- Slope: a drain is flat or a riser. A run at 1/4 in/ft is "not there".
- Stack (upright run) fittings and the vent-to-waste tee (E3 found them refused; the new run through `fitting --all`
  still passed over the vent and the in-line joins).
- The new drain tied to the existing one: I drew it starting at the existing pipe's point; the program says
  "pipe-0023 and pipe-0041 leave (12, 2) the same way, one over the other" and calls them a clash (3.00 in). There
  is no tie-in, so a tie-in to an existing line shows as a clash and gets no fitting.
- A pipe in a wall that is marked demolish. My existing cold line (pipe-0028) crosses the demolished partition.
  `clashes` says nothing (the wall is "in nothing's way"), `vif` says nothing, `estimate` says nothing, and the
  takeoff still counts the 8 LF. What a plumber must know is "this line is in the wall coming down: cap it,
  reroute it, or kill it". Nothing says that.
- Demolish for a run or a fixture has no cost line: `mark ... demolish` on pipe-0028 and cable-0024 priced nothing.
- Existing water heater capacity, gas, expansion tank, supply size: no field on equipment to hold them
  (`set equipment-0026 gallons` is refused: "is not a field that can be set"). The name "40 gal" is only text.
- Fixtures drawn as boxes; no rough-in connection points; a vanity (furnishing) does not carry its lavatory.

**Wrong.**
- A fixture against its own wall is a clash: "wall 8 and Kitchen sink in each other by 3.00 in", "wall 8 and
  Range 4.20 in", "wall 8 and Duplex receptacle 1.20 in". That is how they are installed. In a kitchen, these are
  the first things in the list; they bury the one real one (pipe vs island, 0.38 in).
- "Kitchen sink and Range in each other by 9.00 in" ranks FIRST in `vif` at $420, because both have a priced line.
  That is my layout error (right to flag) but the ranking by dollars is by the price of the line on the thing,
  not the cost of being wrong.

**On paper instead:** the riser diagram, fixture-unit count, trap arms and vents, and the hot/cold/waste take-off by
material and fitting. The program could only be the 3D coordination check.

## HVAC and sheet metal

**Works.**
- Duct by size: 6 in round and 12 in x 8 in separately in `takeoff`; `estimate by-item` sums 46 LF (right).
- Register as a fixture, air handler as equipment; takeoff shows "joined: 2 networks; 4 open ends" with each open
  end named and placed - the useful thing for a trade: it tells me what is not connected.
- Duct vs cable, duct vs register clashes with a height; a duct 4 in over a trunk is catchable
  (the trunk-through-stack example of E3 remains the best case).
- Existing duct marked existing is not priced.

**Half there.**
- "Fitting, installed" exists in the full sample but fittings are not priced in my job (no assembly line on them).
- A branch drawn as a separate run and not joined shows "no fitting at (...)" - good signal, but the in-line
  join "most often one run drawn in two pieces: a coupling is put only where it is asked for" is passed over.
- Registers: counted as "Fixture" - "Fixture, set and connected" at the plumbing price. I priced a register at
  $300 (same item as a lavatory); that is only a made-up artefact, but the item is the same.

**Missing.**
- A load: nothing holds a CFM, a tonnage, a static pressure. Does the existing air handler have the capacity for
  a 12 ft addition? The program cannot even hold the number; "Air handler ~24 x 22 x 48 in" is a box size.
  There is no manual J / S / D step and no room CFM to size a register or a run.
- Boots, takeoffs, end caps, dampers, flex; duct material (galvanized, flex, ductboard), insulation, wrap, gauge.
- Equivalent length and pressure drop; a long run to the addition is the real concern with an existing unit.
- Returns and the whole return path (the system type exists; the job does not carry it).
- Existing duct to cap or extend: same gap as above.
- Mini-split / heat pump for an addition (a very common choice) is only a box: "Heat pump, outdoor unit" in items.

**Wrong.**
- Sheet metal is priced by LF only, one rate for 6 in round and 14 x 8: an estimator prices by pound or by
  perimeter x length and by fitting count.
- Ducts at the same height as the cable (my 8 ft): six of my 14 clashes are duct-versus-cable; a cable stapled to a
  joist beside a duct is not a clash. A tool that says "3.20 in in each other" for a 0.4 in cable on a 12 in duct
  trains me to ignore the list.

**On paper instead:** Manual J / S / D, the register schedule, and the equipment pick. BuildOS is the 3D check
for whether the trunk fits the joist bay and the wall.

## Electrical

**Works.**
- Cable by LF (7 + 29 + 5 = 41, right), panelboard and device as equipment and fixture.
- `receptacles` on a room (call 147): hand check for my 11.5 x 5.5 in clear bath: perimeter 34 LF, less the 3 ft
  door = 31, / 12 = 2.58, ceiling 3 EA; the program said 3 EA. The arithmetic is right and it states what it
  leaves out (fireplaces, fixed cabinets, counters, baths, halls).
- Subcontract scope for the cable and device lines.

**Half there.**
- The receptacle count is a floor, not a take-off. For the living room that now absorbed the demolished wall
  and the addition, the room is "Living, 544 sq ft, 11 receptacles" (one combined space) - it counts the
  wall run but not the kitchen counter rule (every 4 ft, 24 in) and not the island. A kitchen new, being open
  to the living through the demolished wall, gets "unknown - Living was placed first and has the space this
  room's point is in". Correct, but not helpful: the room tool does not split an open plan.
- The bath got 3 EA by the wall rule; the code needs one GFCI at the basin. The note says "a bathroom...
  has rules of its own... not worked here", but the number 3 is still printed in the room panel and rolls
  into the room's figure. That is a number that misleads an electrician (he would overbuy on the bath).
- It does not turn a count into a priced line: no "Receptacle, rough and trim" line on a room; a placed
  duplex is a Fixture and I priced it $45 as an "Electrical device", one EA.

**Missing.**
- Panel capacity: nothing can hold "100 A, 20 spaces, 14 used" and nothing says whether a 100 A existing
  panel can take a new range (50 A), the bath heat, and the kitchen counters. `set equipment-0025 amps` is refused.
  "Panelboard 100 A" is a name only. NEC 220.82 load, spaces, bus rating.
- Circuits: the only systems are "Power" and "Lighting". No circuit, breaker, AFCI/GFCI, load in VA.
- Cable by conductor and gauge (12/2 vs 6/3): the type is "0.4 in round". No homerun, no drop at each box,
  no staples, boxes, covers, connectors. `fitting` refuses cable, so electrical fittings are zero.
- Devices: the starter item library has one electrical item (Panelboard, 200 A). No switches, GFCI, smokes, lights,
  fans, disconnects; no symbols on the 3D (trades/3-electrical.png shows four orange lines and no devices).
- Existing cable to extend: the existing cable run (20 LF) ends mid-wall; no tie-in, and my new cable overlaps it
  (clash "cable 1 and cable 3 in each other by 0.40 in at (12, 15, 8)").
- Service upgrade, meter, grounding, trim: not modelled.

**Wrong.**
- The takeoff reports "Power 61 LF" which is existing 20 + new 41, including a cable I marked to demolish.
- Duct/cable/register "in the same space" rows: a cable ending at a device is a termination, not a clash.

**On paper instead:** the panel schedule and load calc, circuit list by room with homerun lengths, and a device count
by type. The only part I would take from BuildOS is the plan-length of cable and the room perimeter.

## Clash detection between trades and structure

**Works.**
- Existing is Known; a new thing meeting it is shown as a question to verify, with dollars riding: `vif` heads
  with ~$2,465 on 18 lines; export vif prints a field sheet with a box to tick.
- "Through a wall or a floor (a hole to cut)" is the right idea: wall 9 and pipe 7 at (30, 6, 3).
- Demolished walls leave the test (partition and east wall: "in nothing's way once the work is done"): right.
- Clearance mode: "Closer than 2.00 in: 9 (1 finding, 8 to verify)".
- Move a thing with live clashes (carry/): the program can do what a builder wants to do with the vanity.

**Half there.**
- It tests against walls and floors only as solids, not against joists, rafters, studs (not drawn: framing for
  walls only, no joists), so a pipe in a floor bay or a duct in a joist cavity is the question a mechanic asks.
- Existing vs new are not told apart in the list. An existing run and a new run overlapping is an
  "intentional tie-in", not a clash; the program does not know.
- "Closer than a clearance" has one number for all trades; no code clearance.

**Missing.**
- A pipe or duct in a wall that is to demolish (see above): silent. The most valuable renovation clash,
  and it is not there.
- Existing in demolished floor, in a beam to be opened, a joist to be notched: not structure.
- A way to see "what is in this wall / this bay" before cutting. LiDAR: `import scan` (PLY) exists on the
  command line, cuts a plan sheet to trace; it carries no ducts or pipes: that is by-eye tracing.
- Priority: a hard overlap is not ranked above a fixture touching its own wall.

**Wrong.**
- A fixture touching its own wall, a receptacle in a wall, and a cable meeting a device are clashes (E3's
  finding, still true: 3 of my 14 are exactly that).

---

## Ranked list for the group: ten things for THIS job

1. **Existing run or device in a wall/floor marked demolish: say "cap, reroute or remove", count them, and list
   them in the scope.** MISSING. Silent today; it is the first thing a plumber or electrician must know before the saw.
2. **Separate scopes by trade: division/system column and an existing/new/demo column in `export subcontractor`,
   so plumbing, HVAC and electrical each get theirs.** MISSING (one list, no division, no system).
3. **Takeoff and scope separate existing, new, demolish; an existing item must not enter the subcontract
   export as "set and connected".** PARTLY (money right, quantities mixed, existing air handler exported).
4. **Tie-in to an existing run: a connection to something existing is a fitting and a line, not a clash.**
   MISSING (shown as "in each other", no fitting).
5. **Equipment and panel capacity: amps, spaces, tons, CFM, gallons, as Known/Assumed fields, and an
   "enough / not enough" check.** MISSING (fields refused; name only).
6. **Electrical: circuits, devices, wire type by gauge and box/drop count; a device library.**
   MISSING (one starter item; "0.4 in round").
7. **Rough / trim / top-out as separate lines with labor, and demolition as a line (cap, remove, patch).**
   MISSING (all Subcontract lumps; demolish is unpriced).
8. **Clash quality: a thing against its own wall or its own cable is a mounting, not a clash; code clearance
   (WC, panel working space).** PARTLY (clearance mode exists; mounting noise dominates).
9. **Plumbing: pipe material, slope, DWV fittings by name, stacks and vent joins, fixture units.**
   MISSING (type is a diameter; fittings refused on stacks).
10. **Receptacle and room counts that do not mislead: bath and kitchen counters, open-plan split, and a
    count turned into a priced line.** PARTLY (arithmetic right; bath 3 EA misleads; open room gives "unknown").
