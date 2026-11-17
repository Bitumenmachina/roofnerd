# Proposals

Work outside a deliverable. Nothing here is built until the owner accepts it.

1. **A constraint solver (z3) for cross-axis rules.** Derivation today is direct
   recomputation, which is enough for geometry-to-quantity. A solver earns its place only
   if rules must be solved in both directions at once across cost, schedule and geometry.
   It adds a C++ dependency. Not needed by D1 through D6 as written.
2. **License checking in a pre-commit hook.** `tools/licenses.sh` is run by hand now.
3. **More geometric consistency checks.** D1 checks each plane and each line against the
   planes it names. It does not check planes against each other: two outlines that
   overlap, or a gap between planes that should meet, pass unnoticed.
4. **UUID paths.** Element paths are short readable strings (`plane-0003`). IFC5 examples
   use UUIDs. If jobs are ever merged from several machines, the counter-based paths will
   collide and UUIDs would be needed.
5. **Metric.** The owner ruled that US measurements stay and did not ask for metric. A
   value in mm, cm or m is refused with a message that says it is metric. Taking metric
   would be a fourth and fifth length unit in `units.rs` and a decision about what derived
   quantities are shown in. A laser set to metres types `7.123m` and is refused the
   same way (QUESTIONS.md Q19).
6. **Pitch in degrees or percent.** A pitch is entered as rise in 12 (`6:12`, `1/4:12`).
   Degrees and percent are refused by name. Some low-slope drawings give percent and some
   manufacturer data gives degrees; nobody has asked for either.
7. **What BIMCraft's building model has that was not taken in the re-base** (DECISIONS.md
   call 29). Each is a field or a part of its model that was read and left, with the
   reason it would be wanted: curved walls (`arc`); a wall's top following the roof or
   floor above it (`attach_top`, `attach_base`), which a gable-end wall needs; openings in
   a floor (a stair well comes out of a floor's area); a floor's slope and shaped top;
   a room's height, for wall finish and volume; room separation lines; families, with
   their forms, types and formulas, in place of `what` and `size`; a door's swing and a
   window's operation; an opening that rises past its wall's top, cut as a notch;
   the mitred plan outline of runs that join; project parameters.
8. **Three or more run ends meeting at a point with no fitting.** BIMCraft's network says
   nothing where runs meet end to end, however many. Three pipe ends at one point is a tee
   that was not entered, the same as a branch on a run's length. Saying so would be a
   fifth warning, ours.
9. **A fitting's size from its connectors.** A fitting's takeoff line is its kind and
   what it is said to be. Its connectors carry a size; grouping fittings by connector
   size, and a transition by its two sizes, would make the count orderable by size.
10. **Putting the fittings in where a run meets another along its length.** The network
    finds each place. A command could split the run there and place a tee, for the
    estimator to accept.
11. **A cable or a duct that ends at equipment joining through it.** Equipment does not
    join what meets it, as BIMCraft has it, so the two supply trunks off one air handler
    are two networks. Whether a unit's supply connectors are one network is a rule this
    market's estimators would have a view on.

12. **Class adders and bond.** The earlier estimating program in this repository adds
    sales tax and escalation on material, burden on labor and on supervision (each at its
    own rate), general liability on subcontract, and bond on the contract amount
    (`packages/engine/src/recap.ts`). The brief's D5 asks for overhead and profit, and
    those are what is built. A real price needs the rest; none has a default there, and
    none would here.
13. **Production by the crew-day.** A foreman says forty squares a day with a crew of
    six. The earlier program takes a rate that way (`model.ts`, `perManHour`); it needs
    the crew and the hours in a day entered. Here a rate is so many to the man-hour, or
    hours to each.
14. **Pricing the job at a chosen supplier.** The sheet shows the generic price and each
    supplier's side by side; the recap is at the generic price. Choosing one supplier (or
    the best quote for each material) to price the job by is not asked for.
15. **A second material on one element.** An element is made of one material, and its
    lines are either made of that or carry a price of their own. Underlayment on a copper
    roof carries its own price and so is not compared across suppliers.
16. **Equipment and Other as classes of cost.** The earlier program has them; the brief
    names material, labor, fabrication, supervision, subcontract and general conditions.
17. **A Bosch GLM laser over Bluetooth** (Patrick, 2026-11-11: to proposals). A laser in
    keyboard mode types its reading at the cursor and needs nothing from the program. A
    Bosch GLM speaks over Bluetooth and would need the program to listen for it.
18. **Finding lines in a scanned plan or an aerial image**, so a tracing snaps to a
    picture as it does to a vector PDF's own lines (QUESTIONS.md Q18).

19. **Squaring up a scan that lies crooked on its page.** A drawing has a scale and a
    place on the plan and no turn. USDA Plan No. 7213's floor plan lies about a quarter
    of a degree off square on its scan: its 41 ft 3 in dimension line climbs 3 pixels in
    619. Walls traced square by their typed lengths are right; the sheet under them
    drifts by about 2 1/2 in from one end of the house to the other. Two points picked on
    a line known to be level would give the turn.

20. **Foundation, framing, waterproofing and interior woodwork drawn in the model**,
    each with a switch of its own (from Patrick's request of 2026-11-12 for a snapshot
    of each trade: DECISIONS.md call 69, QUESTIONS.md Q23). Today they are estimate
    lines with nothing in the model behind them. A footing and foundation wall under
    each exterior wall; plates, studs and headers in each framed wall; the weather
    barrier under the finish and flashing at openings; base along each room and casing
    at each opening. Each estimate line would take its quantity from what is drawn.

21. **A division handed out whole and taken back: the seam for a harness per trade**
    (from Patrick, 2026-11-13: DECISIONS.md call 75, QUESTIONS.md Q24). Three parts,
    in the order they would be built, each a command-line verb first and a button
    after, each round-tripping without loss as the job file and CSV already must:
    * **Out:** `plumbline division <job> <n> --out <folder>` writes that division and
      nothing else: its cost lines with the element each is on, its quantity and
      formula, its price, how sure each is and where from (CSV); the assemblies that
      put lines in it; the materials those lines are made of with their prices and
      quotes; and what is still to verify in the field that its dollars ride on.
    * **In:** `plumbline division <job> <n> --in <folder> --source "<who>"` takes the
      same files back: a changed price, waste, stretch-out or formula is set on the
      line it belongs to, as Assumed, sourced to who brought it; a line of another
      division in the folder is refused, not taken; what changed is listed, and the
      price before and after.
    * **What a division knows:** a folder per division in the repository
      (`plumbline/divisions/06-wood-and-plastics/`) holding, in plain text, what the
      program itself knows of that trade and where each thing comes from: its
      assemblies, its units, each rule of thumb it uses (a header's depth by its span;
      a 4 in flashing strip is a drawing size) with its source. Made-up figures only.
      A harness reads it; an estimator's own knowledge and real prices sit beside the
      program on his machine, in a folder git never sees.
    Not proposed: the harnesses themselves, their models, or where they run.

22. **A door's hand and swing** (2026-11-13, from reading the plans as a person).
    A door is held by its type, its place along the wall and its sill; nothing says
    which side it is hung on or which way it opens, so the plan draws a door as a
    leaf in the wall with no swing, and the BIM file says nothing of it. Proposed: a
    hand (left, right) and a swing (in, out) on each door, each Known, Assumed or
    Unknown like any value (traced off a plan: Assumed; never guessed, so a door
    brought in without one draws no arc); the arc on the plan; the door's operation
    in the BIM file; a swing that crosses a fixture or another door's swing listed
    with the clashes. Not built: to draw an arc today would be to make up the hand.

## 23. A supplier's list prices the cost lines of the same name (2026-11-16, from judgment 22A)

Today a price list brought in sets the prices of the job's materials, and a cost
line takes a material's price only when the thing it is on is made of that material.
A line that is only named as an item on the list ("Lap siding", on a wall given the
starter assembly) keeps its own price or none. An estimator who brings in a
supplier's list expects those lines priced.

Proposed: on bringing a list in, each cost line with no price whose name is an item
of the list, and which is priced by the unit the item is bought by, takes the list's
price as its own, Assumed, sourced to the list and the item; a line that already
has a price keeps it, and the message says which did and which did not. It changes
what an import does to the money, so it waits for Patrick's word. Until then the
import says which lines it did not price and why (STATUS.md, J22A-15).

## 24. A library file a builder can read: the takeoff a line is priced by, in words (2026-11-16, from judgment 23)

An assembly library written out as CSV or XLSX has, in its Quantity column, the names
the program works a line's quantity from: `face_area`, `net_area`, `true_length`,
`plan_length * stretch_out`. On screen those are said as "face area", "net area";
in the file they are as the program keeps them, because the file is read back by
them and round-trips to the byte.

Proposed: the file says them as the screen does ("net area"), and reads either
spelling. It changes a file format people may already have saved, so it waits for
Patrick's word.

## 25. A run's type has a trade name: "12-2 NM", "3/4 in type L copper" (2026-11-16, from judgments 22 and 24)

A cable, a pipe or a conduit is listed by its size alone: "Cable 0.4 in round". An
electrician calls a cable by its conductors and its jacket (12-2 NM), a plumber a
pipe by its material and type as well as its size. Both judges of the words marked
it (J22B-21, J24B-34). A run's type holds a size and nothing else.

Proposed: a run's type carries what it is called in the trade, flagged like every
value, shown wherever its size is shown, and written to IFC as the type's name. A
cable is then listed by that name and not by its jacket's diameter. Not done as a
wording fix because it is a new value in the model, and the made-up house says of
its own cable "the cable is not specified": naming it would be making a
specification up.

## 26. Cabinets and countertops (2026-11-16, from judgment 24)

The made-up house's kitchen is a bare room with a sink in the air at 2 ft 4 in, and
its baths have a tub, a water closet and a lavatory and nothing else. The second
judge of the look failed "it looks like the house" on it (J24B-29, D8). The program
draws a base cabinet under a worktop for anything named a vanity, a cabinet or a
counter (ported from BIMCraft), but no assembly prices casework and the house has
none in it.

Proposed: casework as Division 6 or 12 lines in the starter assemblies (base and
wall cabinets by the LF, countertop by the SF), priced on a run drawn along a wall,
and drawn as what is priced. It is scope: Patrick's.

## 27. Any cost line can be made of a material of its own (2026-11-16, from judgment 24)

A cost line is priced by its own unit cost, or it is "made of" the one material its
element is made of and takes that material's price and its suppliers' quotes. That
came from roofing, where a plane is made of one metal. A wall's assembly has nine
lines and each is a different thing (framing, sheathing, siding, board): none can be
compared across three suppliers, and a price list reaches them only by their names
(call 104).

Proposed: a line names the material it is made of, so every line can carry
suppliers' quotes side by side and follow a price list when it changes. It changes
the job file and SPEC.md, so it waits for Patrick's word.

## 28. An item's shape as data, and a bench to make it on (2026-11-16, from Patrick's message of that day, call 105)

An item is drawn as one of thirteen shapes built into the program, or as its box. A
trade's library wants its own things: a floor-mounted water closet and a wall-hung
one, a tankless heater, a range hood, a bank of mailboxes. Today a new shape is a
change to the program.

Proposed, in two steps.
1. **The shape is in the library.** An item carries its blocks: boxes and rounds,
   each between figures worked from the item's width, depth and height, of a named
   material. That is BIMCraft's `Family` (`crates/core/src/family.rs`: parameters,
   types, formulas, forms, connectors), which is open in front of us and MIT. The
   item library gains a table of blocks beside its table of items; the thirteen
   built-in shapes become rows of the starter library.
2. **A bench.** An item opened alone in the window, on a floor of its own, with its
   box dimensioned; a block added, pulled and sized by hand or by typing; its
   connections (hot, cold, waste, supply, power) placed on it; kept back to the
   library. Clashes still test the box.

It is new scope and a change to the job file, so it waits for Patrick's word. Whether
a geometry library does any of it is QUESTIONS.md Q27.

## 29. A wall runs up to the roof over it: the gable end (2026-11-16, from judgment 24A, J24A-17)

A hip made a gable takes a plane off the roof and leaves a triangle of air over the
end wall: the wall stops at its height, nothing is drawn in the gable, and nothing is
priced there. On a 24 ft gable end at 6:12 that is 72 SF of framing, sheathing,
barrier and siding in no line of the estimate, and a house with a hole in it on the
screen.

Proposed. A wall has a top that may follow the roof: where a wall's line lies under a
gable edge of a roof, the wall's face is its rectangle and the triangle (or the
trapezoid, where the wall is inset under an overhang) up to the roof's underside
over its line. The core works the added area out from the roof's planes over the
wall's line, as its own row on the wall ("gable end, 72.00 SF, worked out from Main
roof at 6:12"), Assumed while the roof's pitch or the wall's place is; face and net
area include it, so every line priced by the wall's area prices it; the window draws
the wall to the roof and dresses it as the wall is dressed. A gable taken back to a
hip takes the triangle with it and says what came out of the job cost.

Why it is here and not built: it changes a wall's face area, which every wall line is
priced by, on any job with a gable; it wants a rule for which wall is "under" an
edge when the roof overhangs; and it is a relation between a wall and a roof that the
model does not hold today. That is a change to the model's rules, so it is Patrick's.
Until then STATUS.md says a gable end is not drawn and not priced, and the estimate
of a gable roof is short by its gable ends.

## 30. A table in, a shop's spec out: trade packs an agent can bring, and fabricated parts (2026-11-17, from Patrick's message of that day, call 113)

Patrick wants other agents, later, to bring a trade's knowledge, its prices and its
specs into the program, and "an auto converter from something simple like an xls to
something more powerful or complex like a machine shop spec or countertop slabs".

What is there to build on: every table the program reads (prices, assemblies, items,
prior estimates) comes in as CSV or XLSX, row by row, with every row that was not
used named and why, and every value Assumed with its source. Nothing is written out
that a shop could cut from: the outputs are a workbook, CSV and IFC4.

Proposed, in three steps, each usable alone.
1. **A trade pack.** One folder an agent or a person hands over for a trade: its price
   list, its assemblies, its items, and a page of what it knows in words (rules of
   thumb, waste, what goes with what), each a table or a text the program already
   reads or a builder can. Brought in as one thing, said row by row, taken back out as
   one thing. This is the door the per-division harness of calls 75 and 76 would use
   (PROPOSALS.md 21).
2. **A fabricated part.** A thing that is cut from stock and is more than its box: a
   countertop run (its outline, thickness, edge, cutouts for a sink and a cooktop, its
   seams, its backsplash), a sheet-metal piece (its flat pattern and bends, which is
   the stretch-out the estimate already carries), a cabinet box. Typed as rows of a
   table (a run 96 in by 25 1/2 in, a 33 in sink cutout 12 in from the left), drawn
   in the house, priced by its area, its edge and its cutouts, and nested on slabs or
   sheets of a named size so that the count of slabs is a figure with its working
   shown and not a guess.
3. **Out to the shop.** The same part written as what a fabricator takes: a cut list
   as a workbook, and its outline as a drawing a saw or a waterjet reads.

It is new scope, a change to what the job holds, and for step 3 a new file written
out, so it waits for Patrick's word. Which format a shop wants is QUESTIONS.md Q28.

## 31. A door's hand and its swing (2026-11-17, from call 117's fifth point)

A plan is read by its door swings: which side the hinges are on and which room the
leaf opens into. The job holds a door's width, height and place in its wall and
nothing of its hand, so the plan draws a door as a gap with its frame and no leaf. To
draw a swing today would be to make one up, and a made-up swing sends the wrong
hardware (a left-hand lockset on a right-hand door).

Proposed: a door carries its hand as the trade names it (LH, RH, LHR, RHR, by the
hinge side seen from outside the room it secures), Unknown until it is said. On the
plan of its level the leaf is drawn open a quarter with its arc; an Unknown hand is
drawn as the gap is now and is a question on the field list. The hand is written to
the BIM file (IFC's operation type on the door) and read from one.

It is a new value in the saved job and a new thing a person sets, so it waits for
Patrick's word.

## 32. A LiDAR scan of the house brought in (2026-11-17, Patrick: "I use Scaniverse: what is the ability to import any of the lidar models to this program and how much time would that take")

**Today: none directly.** The program brings in plan sheets (PDF, PNG, JPEG), tables
(CSV, XLSX), a designer's BIM model (IFC), makers' sheets and its own job. It reads
no mesh and no point cloud. The one way a scan helps today: a picture of the scan
taken straight down, brought in as a drawing, scaled from one length measured on
site, and traced.

**What the phone app writes** (its community forum's list; not its own manual):
meshes as OBJ, FBX, GLB, USDZ, STL; point clouds as PLY and LAS.

**Proposed, in three steps, each useful alone.**

1. **The scan as something to trace over, at true size.** Bring in a mesh (GLB or
   OBJ first: both have permissively licensed readers; to be checked by
   `tools/licenses.sh`) or a PLY point cloud. A scan is already in real units, so
   there is no scale to set: only where it sits and which way is north. It is shown
   in 3D as a ghost under the model, and cut at a height (4 ft above a floor) into a
   plan outline that the Wall tool snaps to. Everything traced from it is Assumed,
   sourced to the scan's file. Loss is reported: colour and texture are not kept.
2. **The scan as a tape.** Two points picked on the scan give a length; given to a
   value on the verify-in-field list it answers the question as a reading does, with
   "measured on the scan <file>" as its source. Whether such a reading makes a value
   Known or leaves it Assumed is Patrick's (a phone scan drifts over the length of a
   house; a laser on site does not): the working answer would be Assumed, said so.
3. **Walls and rooms found in the scan without tracing.** Fitting planes to a scan
   and reading walls, openings and rooms out of them. Not a reader but a piece of
   research: results would arrive Assumed and want checking one by one.

**How long.** Step 1: two to three days of building, with its demonstration, its
pictures read and a judge. Step 2: one to two days more. Step 3: weeks, with no
promise of how good; not to be started before 1 and 2 are used on a real house.
A scan with no mesh (a "splat") is a picture, not a measure, and is out.

It is a new kind of file brought in and a new thing the job holds, so it waits for
Patrick's word. FBX and USDZ are left out at first (no settled permissive reader);
LAS after PLY.
