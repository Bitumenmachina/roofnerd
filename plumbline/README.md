# plumbline (working name)

A 3D estimating program for the whole house, every trade. Trace the drawings you are
given, build the 3D house from the tracing, price it division by division (the sixteen
divisions), and get the list of what has to be verified in the field with the dollars
riding on each question. The job is one model; the 3D view, the estimate sheet, the
workbook and the lists are views of it. Every value is Known, Assumed or Unknown and
says where it came from. The program sends no job data and no costs anywhere.

## Install on Linux

Tested on Ubuntu 24.04. Three steps, in order.

**1. What the build needs** (a C toolchain, the X11 development files the window is
built against, and a Vulkan driver; Mesa's runs on any machine, with or without a
graphics card):

```
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl git libx11-dev libxkbcommon-x11-0 mesa-vulkan-drivers
```

**2. Rust**, through rustup. The repository names the version it builds with
(`rust-toolchain.toml`, 1.97.1) and rustup fetches it on the first build:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
```

**3. The program:**

```
git clone -b claude/nifty-tesla-qmn44n https://github.com/Bitumenmachina/roofnerd
cd roofnerd/plumbline
cargo build                         # the first build takes a while: it builds the 3D engine
cargo test --workspace              # the model, the command line, the window's crate
cargo run -p plumbline-app          # a new job, job.json, in the model window and the sheet window
```

(plumbline is on the branch named there until it is merged; the repository's default
branch holds the earlier program only.)

The window opens on an empty job with **Next steps** down its left side: add a level,
add a drawing and set its scale, draw the walls, and so on to a price. To try it on a
real plan, `tools/fetch-plans.sh` fetches a public-domain USDA farmhouse plan into
`plans/` (it needs the network once); in the window, Add drawing takes its name.

To run the demonstrations as well (each is a script that drives the real program and
checks what it did; the ones ending in `w` drive the window on a virtual display):

```
sudo apt-get install -y xvfb xdotool python3 python3-pil python3-openpyxl
demos/d1.sh                         # one of the command-line ones
demos/pricew.sh                     # one of the window ones; its pictures land in demos/captured/price
```

Where things stand is in [STATUS.md](STATUS.md). The model is in [SPEC.md](SPEC.md).

| File | What it holds |
|---|---|
| STATUS.md | Each deliverable: what was run, what it showed, what remains |
| SPEC.md | The world data model and the saved-job format |
| DECISIONS.md | Every build-on, borrow, or write call, with its evidence |
| LICENSES.md | Every linked crate and its license; every candidate's license |
| QUESTIONS.md | Open questions for the owner and the assumption in use meanwhile |
| PROPOSALS.md | Work outside a deliverable. Not built until accepted. |
| HANDOFF_REBASE.md | The task that re-based the building model on BIMCraft, and the rules it was done under (done; STATUS.md has the account) |
| HANDOFF_DRAW.md | The task of drawing and editing in the window, its fence, and where it stands |
| HANDOFF_CLASH.md | The task of fittings, connections and clash detection, and where it stands |
| VISUAL.md | The visual standard: what every screen and every picture of the house is held to, line by line |
| PORTMAP.md | Each public item of the building model: ported from where, or ours and how that was checked |

## Run it

```
cargo test                 # the world-model checks
tools/landscape.sh         # re-run the D0 landscape checks (needs network)
demos/d1.sh                # the D1 demonstration
demos/units.sh             # the units demonstration: 30 in, 25' 6 1/2", stretch-out
demos/d3.sh                # the D3 demonstration: a parapet's four lines, sheets and pounds
demos/d4.sh                # the D4 demonstration: priced from allowances, priced from its lines, rolled up, reconciled
demos/d5.sh                # the D5 demonstration: the estimate sheet, the recap, suppliers side by side, a change of material
demos/d5w.sh               # the same in the window
demos/draw.sh              # a small house modelled from an empty job at the command line, one verb at a time
demos/draww.sh             # the same house drawn in the window's plan view with the pointer and typed lengths; pictures in demos/captured/draw
demos/fit.sh               # fittings worked out from the runs that meet, runs joined to what they serve, and what the takeoff says is still open
demos/clash.sh             # what is in the same space as what, what passes through a wall, and what is only close
demos/clashw.sh            # fittings, a connection and the clashes in the window; pictures in demos/captured/clash
demos/plan.sh              # a plan laid under a level, given its scale from one dimension and traced over, at the command line
demos/field.sh             # what is measured on site put on a traced job: the floor above moves and the price with it, one clash clears, one becomes a finding
demos/fieldw.sh            # the same in the window, each reading typed as a laser in keyboard mode types it; pictures in demos/captured/field
demos/div.sh               # sixteen divisions: a whole house's lines in all sixteen, the recap by division, and the workbook's Divisions sheet
demos/library.sh           # the starter assemblies: no price in them, a house's takeoff in fourteen divisions, three prices by division
demos/pricew.sh            # the same in the window: Starter assemblies, an assembly picked from the list and changed, a price typed on the sheet
demos/dress.sh             # what the house is dressed in, read from what is priced on it: each wall, the roof, a window and a door, with the line that says so
demos/dressw.sh            # the same in the window: one cladding to a wall, the priced sample house, and the same house bare; pictures in demos/captured/dress
demos/vif.sh               # the verify-in-field list: each question, what to measure, and the dollars riding on it, before and after measuring
demos/export.sh            # the estimate as a workbook of formulas (worked again by a second hand), as CSV, three lenses, and the building as IFC4 checked by IfcOpenShell
demos/imports.sh           # a price list and an assembly library out as CSV and back; a supplier's list from CSV and from a workbook; a prior estimate set against the job
demos/model.sh             # another trade's model (IFC) brought in: what comes in, what is listed instead, and the clash it hid
demos/planw.sh             # the same in the window, on a real scanned sheet and on a vector plan; pictures in demos/captured/plan
demos/build.sh             # a floor from the outline its walls enclose, rooms and a hip roof, at the command line, with the figures worked by hand
demos/buildw.sh            # the same in the window, traced over the made-up plan with the Floor, Room and Roof tools; pictures in demos/captured/build
tools/fetch-plans.sh       # fetches the public-domain plan set the demonstrations trace (USDA Plan No. 7213) into plans/
demos/d4w.sh               # the same in the window: the sheet's Budget tab, and prices typed on what is selected
demos/d2.sh                # the 3D prototype, driven on a virtual display; pictures in demos/captured/d2
demos/house.sh             # a whole house with its heating, plumbing and wiring, walked round; pictures in demos/captured/house
cargo run -p plumbline-app -- job.json    # the model window and the sheet window (every button says its key)
tools/licenses.sh          # every linked crate and its license; fails on a non-permissive one
tools/portmap.sh           # the building model accounted for against BIMCraft's source, item by item
cargo run -p plumbline -- sample job.json
cargo run -p plumbline -- house house.json     # the sample whole house
cargo run -p plumbline -- sixteen house.json  # the same house with a made-up price on everything, in all sixteen divisions
cargo run -p plumbline -- takeoff house.json   # its walls, floors and rooms, and each system's runs by size, its fittings and what is joined
cargo run -p plumbline -- clashes house.json   # what is in the same space as what; add --clearance "2 in" for what is closer than that
cargo run -p plumbline -- price job.json       # what the job comes to from its allowances and from its cost lines, and the two compared
cargo run -p plumbline -- show job.json
cargo run -p plumbline -- set job.json parapet-0024 height "30 in" --status Known --source "detail 3/A-501"
```

A length is typed the way the drawing or the tape gives it (`2.5 ft`, `30 in`, `19 1/2"`,
`25' 6 1/2"`) and is kept in that unit. A length with no unit is refused, never read as feet.

## The window

```
cargo run -p plumbline -- new job.json "A job"
cargo run -p plumbline-app -- job.json
```

The model window is laid out the way a building game's is:

* **Along the top**: Save, Undo, Redo, the view (3D, Plan, Walk), and at all times the
  job's price in the color of how sure it is.
* **Down the left**: Next steps (what this job needs next, read from the job; each step
  is ticked when it is done, and the first that is not says how), the levels, the
  drawings under the level on show, what of the building is on show, and its systems.
* **Along the bottom**: the build bar. Every tool is a button with its mark, in three
  groups: Building (Wall, Door, Window, Floor, Room, Roof), Mechanical, electrical,
  plumbing (Duct, Pipe, Conduit, Cable, Fitting, Equipment, Fixture, Connect) and Edit
  (Move, Delete). The number keys 1 to 9 and 0 take the first ten in hand.
* **Over the build bar**: the fields of the tool in hand (a wall's thickness and
  height), each typed into where it stands, and what to do with the tool.
* **Down the right**: what is selected, as fields and buttons. A value that can be
  measured says Measure or Check; what can be done to the thing is a button on it.

Every button says the key that does the same. The words on screen are US English and a
drawing set's: a floor of the building is a level. (The job file and the command line
keep the model's own word, level.)

## Draw in the window

With nothing in the job, click any tool: it asks for the first level (its name, the
elevation of its floor, and floor-to-floor, like `Ground`, `0 ft`, `9 ft`). Then:

* Click a tool. The first time, its fields open over the build bar (a wall: thickness
  and height). Type each, Tab or a comma to the next, Enter to take the tool in hand. It
  draws with nothing until they are typed. Click a field later to change it.
* Click to place points. The pointer snaps to ends, middles, crossings, the square and
  the angle guides, and otherwise to the inch; the line at the bottom says which. With
  one point placed, type a length (`24 ft`) and Enter: it goes that far toward the
  pointer. Esc ends the chain; Esc again puts the tool down.
* A door or a window is one click on a wall. A fixture or equipment is one click.
* **Floor** is one click on the level: the floor fills the outline its walls enclose.
  **Room** is a click inside the room, then its name. **Roof** is each corner of its
  outline, then the first again; every side has the pitch typed, and a plane is made a
  gable afterwards with its Gable button.
* Select something and use Move (two points, or one and a typed length) or Delete.
  On plan, a click on a door or window in its wall selects the opening.
* Fitting: one click on a joint. What meets says what it is (two ends at an angle, an
  elbow; three, a tee; a branch on a run's side cuts the run and makes a tee). Connect:
  one click where a run ends at a fixture or a piece of equipment.
* Ctrl+Z undoes and Ctrl+Y redoes (Z and Y alone do too). Ctrl+S or F2 saves.
* Under Show: each trade, See through, Known, Assumed, Unknown (everything drawn in the
  color of how sure it is), and Clashes (each clash marked on the model in red). The
  sheet's Clashes tab lists what is in the same space as what and what passes through a
  wall or a floor.

## Trace a plan

The plan you were given goes under the level it is a plan of: a PDF (drawn or scanned),
a PNG or a JPEG. Under Drawings, on the left:

* **Add drawing**: type the file's name, and its page if it is not the first
  (`plans/a-1.pdf, 2`). Or drop the file on the window. The sheet appears under the plan
  and says it has no scale yet. Nothing can be traced over it until it has one.
* **Set scale**: click the two ends of a dimension on the sheet, then type what the
  dimension says (`41' 3"`). The sheet is now to scale. On a PDF that was drawn, not
  scanned, the pointer snaps to the sheet's own lines.
* **Line up**: click a place on the sheet, then the place on the plan it is.
* Then draw over it with the tools above. What is traced is flagged Assumed and says
  which drawing it was traced from. A length typed off a dimension goes that far, to a
  ten-thousandth of a foot.

The job keeps where the plan file is, never the plan itself. If the file changes or
moves, the drawing says so.

At the command line: `plumbline drawing job.json <level> "A-1" --file plans/a-1.pdf`,
`plumbline calibrate job.json <drawing> --from "126,90" --to "666,90" --distance "30 ft"`,
`plumbline drawings job.json`, and `--on <drawing>` on `wall`, `door`, `window`, `room`,
`run` and `place` to give points as places on the page.

## Bring in what you already have

```
plumbline import prices job.json list.xlsx --supplier "Their name" --status Known --source "their list of 3 March"
plumbline export prices job.json prices.csv
plumbline import assemblies job.json assemblies.csv
plumbline import prior job.json last-estimate.csv --status Assumed --source "our estimate of last year"
plumbline compare job.json
plumbline import ifc job.json their-model.ifc --status Assumed --source "the mechanical contractor's model of 3 March"
```

A price list is a CSV file or a workbook with a row of headings: what is priced, what
it is bought by, and the price. The program finds the headings, sets the prices it can,
and lists every row and column it did not use and why. An estimate made before is kept
beside the job and set against it line by line with `compare`; it adds nothing to the
job's price. A model from another trade or a designer (IFC) comes in as levels, walls
with their doors and windows, floors, rooms, ducts, pipes, trays and cables; what the
program has no element for, or cannot read the shape of, is listed with why.

## Send it out

```
plumbline export estimate job.json estimate.xlsx        # a workbook: each amount, the recap and the price are formulas
plumbline export estimate job.json estimate.csv         # the same lines as values
plumbline export supplier job.json to-quote.xlsx        # what to quote, by what is bought; none of our prices
plumbline export subcontractor job.json scope.csv       # the scope classed Subcontract; none of our prices
plumbline export pm job.json for-the-pm.xlsx            # every line with its hours, the budget by class, what is to verify; no markup
plumbline export ifc job.json building.ifc              # the building as an IFC4 model, lengths in feet
```

In the workbook, change a quantity or a price and the amounts, the recap and the price
follow: the formulas are the program's own arithmetic, to the cent. Each lens is the one
estimate with what its reader is not shown left out, and says at its head what that is.
The IFC model carries every value's status and source with it (what is Assumed here is
still said to be Assumed there), and the command lists what a building model has no
place for. An estimate written as CSV reads back in with `import prior`.

## Assemblies, and pricing

An assembly is everything priced on one thing: a wall's framing, sheathing, siding,
gypsum board and paint, each line in its own division. The program ships with a
starter set, 30 assemblies for a house's walls, floors, rooms, roofs, doors, windows
and runs, with a line in every one of the sixteen divisions and **no prices**: those
are yours.

In the window: under **Bring in**, click **Starter assemblies**. Click a wall, then
**Assembly**, and pick one from the list. Its lines are on the sheet's Estimate tab,
each with a field for its price; the recap, the sixteen divisions and the price on
the top bar follow as you type. Pick another assembly for the same wall and it
changes: lines the two share keep their prices, and the status line says what came
off.

    cargo run -p plumbline -- import assemblies job.json starter     # the starter set
    cargo run -p plumbline -- apply job.json wall-0004 "Exterior wall, 2x6, lap siding" --source "the plans"
    cargo run -p plumbline -- apply job.json wall-0004 "Exterior wall, 2x6, brick veneer" --source "the plans" --change
    cargo run -p plumbline -- estimate job.json                      # the sheet, the recap, and the cost by division

## The house, dressed

In 3D the house is drawn in what the estimate prices on it. Price "Brick veneer, red"
on a wall and its outside face is red brick; "Cedar lap siding" and it is lap siding;
"Stone veneer wainscot, 3 ft" and the bottom 3 ft is stone. A roof plane takes its
roofing the same way (asphalt shingles, standing seam, slate, tile, shakes), a
window how it operates and whether its lites are divided ("double-hung, 6 over 6,
black"), a door how it is built ("six panel"). A wall with nothing priced on it that
names a cladding is drawn bare: that is a wall nobody has priced the siding for.
Click anything and its Finish says what it is dressed in and which line says so.

    cargo run -p plumbline -- finishes job.json   # the same reading, written out

A line is read by what it is, not by a word in it: "Stone sill at windows" is a sill
and dresses nothing, "Lap siding with brick mold casing" is lap siding, and
"Roofing, standing seam, charcoal" is standing seam. Two finishes priced on one
thing: the first is drawn and the second is said, not dropped.

## What is left to verify

```
plumbline vif job.json
```

The verify-in-field list: every thing with a value that can be measured or checked on
site and is not Known, and every clash that rests on one, with the dollars of the
estimate riding on each, largest first. In the window it is the sheet's **Verify in
field** tab, and the top bar says how many there are and what rides on them; a click on
a row selects the thing in the model window, where its values say Measure or Check.

## Measure on site

```
plumbline measure job.json level-0002 "level height" "8' 10 1/2\"" --source "JM with the laser"
plumbline measure job.json duct-0011 "height above its level" "7' 10 3/4\"" --to bottom --source "JM with the laser"
plumbline measure job.json duct-0011 "line on the plan" same --source "JM on site"
```

What is traced is Assumed until someone measures it. In the window, select a thing (a
level by its row on the left, anything else on the model); each of its values on the
right says **Measure** or **Check**. Click the value and type what the tape reads, or
shoot it with a laser in keyboard mode: the reading lands where it is asked for, and
the Tab or Enter the laser sends ends it. `same` says the value was looked at and is as
drawn. For a duct or a pipe, `bottom` or `top` before the reading says where the shot
was taken to. Who measured can follow a comma. The value is then Known, the clashes
that rested on it are findings or are gone, and the price in the sheet window follows.

A laser has to send its unit with the reading (Leica calls this its text setting), in
feet and inches, with a point for the decimal mark. A figure with no unit, a comma, or
meters is refused, and the refusal says so.

Everything drawn in the window is flagged Assumed and says it was drawn there. At the
command line the same verbs take a status and a source: `plumbline floor job.json
<level> <floor type> --walls --status Assumed --source "..."`, `plumbline room job.json
<level> "name" --at "6,8" --status Assumed --source "..."`.

`refs/` is for the owner's reference estimates. Git ignores it. Nothing in it is ever committed.

Licensed under either of Apache License 2.0 or the MIT license, at your option.
