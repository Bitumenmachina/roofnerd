# plumbline (working name)

A 3D estimating platform for high-end residential work. The job is one
entity-component world: every building element is an entity carrying geometry,
condition, cost, schedule, status and provenance. The 3D view, the estimate
sheet, the reports and the schedule are views over that one world. It is being
proven on the building envelope first.

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
demos/plan.sh              # a plan laid under a storey, given its scale from one dimension and traced over, at the command line
demos/field.sh             # what is measured on site put on a traced job: the floor above moves and the price with it, one clash clears, one becomes a finding
demos/fieldw.sh            # the same in the window, each reading typed as a laser in keyboard mode types it; pictures in demos/captured/field
demos/imports.sh           # a price list and an assembly library out as CSV and back; a supplier's list from CSV and from a workbook; a prior estimate set against the job
demos/model.sh             # another trade's model (IFC) brought in: what comes in, what is listed instead, and the clash it hid
demos/planw.sh             # the same in the window, on a real scanned sheet and on a vector plan; pictures in demos/captured/plan
tools/fetch-plans.sh       # fetches the public-domain plan set the demonstrations trace (USDA Plan No. 7213) into plans/
demos/d4w.sh               # the same in the window: the sheet's Money tab, and prices typed on what is selected
demos/d2.sh                # the 3D prototype, driven on a virtual display; pictures in demos/captured/d2
demos/house.sh             # a whole house with its heating, plumbing and wiring, walked round; pictures in demos/captured/house
cargo run -p plumbline-app -- job.json    # the 3D window and the sheet window (keys are listed in the window)
tools/licenses.sh          # every linked crate and its license; fails on a non-permissive one
tools/portmap.sh           # the building model accounted for against BIMCraft's source, item by item
cargo run -p plumbline -- sample job.json
cargo run -p plumbline -- house house.json     # the sample whole house
cargo run -p plumbline -- takeoff house.json   # its walls, floors and rooms, and each system's runs by size, its fittings and what is joined
cargo run -p plumbline -- clashes house.json   # what is in the same space as what; add --clearance "2 in" for what is closer than that
cargo run -p plumbline -- price job.json       # what the job comes to from its allowances and from its cost lines, and the two compared
cargo run -p plumbline -- show job.json
cargo run -p plumbline -- set job.json parapet-0024 height "30 in" --status Known --source "detail 3/A-501"
```

A length is typed the way the drawing or the tape gives it (`2.5 ft`, `30 in`, `19 1/2"`,
`25' 6 1/2"`) and is kept in that unit. A length with no unit is refused, never read as feet.

## Draw in the window

```
cargo run -p plumbline -- new job.json "A job"
cargo run -p plumbline-app -- job.json
```

Press T for the plan. The left panel's Draw section has the tools. With no storey in the
job it offers "New storey" (type its name, its floor's height and the height to the next,
like `Ground, 0 ft, 9 ft`). Then:

* Click a tool (Wall, Door, Window, Duct, Pipe, Conduit, Cable, Fixture, Equipment). The
  first time, it asks what it draws with, on one line (a wall: `6 in, 9 ft`). It draws
  with nothing until that is typed, and shows it under the tool from then on.
* Click to pick points. The pointer snaps to ends, middles, crossings, the square and the
  angle guides, and otherwise to the inch; the bar at the bottom says which. With one
  point picked, type a length (`24 ft`) and Enter: it goes that far toward the pointer.
  Esc ends the chain; Esc again puts the tool down.
* A door or a window is one click on a wall. A fixture or equipment is one click.
* Select something and use "Move what is selected" (two points, or one and a typed
  length) or "Remove what is selected" (or the Delete key).
  On plan, a click on a door or window in its wall selects the opening.
* "Fitting where runs meet": one click on a joint. What meets says what it is (two ends
  at an angle, an elbow; three, a tee; a branch on a run's side cuts the run and makes a
  tee). "Join a run to what it ends at": one click where a run ends at a fixture or a
  piece of equipment.
* Z undoes and Y does again. F2 saves.
* The sheet's Clashes tab lists what is in the same space as what and what passes
  through a wall or a floor; with it open, each clash is marked on the model in red.

## Trace a plan

The plan you were given goes under the storey it is a plan of: a PDF (drawn or scanned),
a PNG or a JPEG. On the plan (T), with the storey chosen:

* "Lay a drawing under this storey": type the file's name, and its page if it is not the
  first (`plans/a-1.pdf, 2`). Or drop the file on the window. The sheet appears under the
  plan and says it has no scale yet. Nothing can be traced over it until it has one.
* "Set the scale from one dimension": click the two ends of a dimension on the sheet,
  then type what the dimension says (`41' 3"`). The sheet is now to scale. On a PDF that
  was drawn, not scanned, the pointer snaps to the sheet's own lines.
* "Line the drawing up": click a place on the sheet, then the place on the plan it is.
* Then draw over it with the tools above. What is traced is flagged Assumed and says
  which drawing it was traced from. A length typed off a dimension goes that far, to a
  ten-thousandth of a foot.

The job keeps where the plan file is, never the plan itself. If the file changes or
moves, the drawing says so.

At the command line: `plumbline drawing job.json <storey> "A-1" --file plans/a-1.pdf`,
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
job's price. A model from another trade or a designer (IFC) comes in as storeys, walls
with their doors and windows, floors, rooms, ducts, pipes, trays and cables; what the
program has no element for, or cannot read the shape of, is listed with why.

## Measure on site

```
plumbline measure job.json storey-0002 "storey height" "8' 10 1/2\"" --source "JM with the laser"
plumbline measure job.json duct-0011 "height above its storey" "7' 10 3/4\"" --to bottom --source "JM with the laser"
plumbline measure job.json duct-0011 "line on the plan" same --source "JM on site"
```

What is traced is Assumed until someone measures it. In the window, pick a thing (a
storey by its row on the left, anything else on the model); each of its values on the
right says **Measure** or **Check**. Click the value and type what the tape reads, or
shoot it with a laser in keyboard mode: the reading lands where it is asked for, and
the Tab or Enter the laser sends ends it. `same` says the value was looked at and is as
drawn. For a duct or a pipe, `bottom` or `top` before the reading says where the shot
was taken to. Who measured can follow a comma. The value is then Known, the clashes
that rested on it are findings or are gone, and the price in the sheet window follows.

A laser has to send its unit with the reading (Leica calls this its text setting), in
feet and inches, with a point for the decimal mark. A figure with no unit, a comma, or
metres is refused, and the refusal says so.

Everything drawn in the window is flagged Assumed and says it was drawn there. Rooms are
placed at the command line: `plumbline room job.json <storey> "name" --at "6,8" --status
Assumed --source "..."`.

`refs/` is for the owner's reference estimates. Git ignores it. Nothing in it is ever committed.

Licensed under either of Apache License 2.0 or the MIT license, at your option.
