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
| HANDOFF_CLASH.md | The next task: fittings, connections and clash detection |
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
demos/d4w.sh               # the same in the window: the sheet's Money tab, and prices typed on what is selected
demos/d2.sh                # the 3D prototype, driven on a virtual display; pictures in demos/captured/d2
demos/house.sh             # a whole house with its heating, plumbing and wiring, walked round; pictures in demos/captured/house
cargo run -p plumbline-app -- job.json    # the 3D window and the sheet window (keys are listed in the window)
tools/licenses.sh          # every linked crate and its license; fails on a non-permissive one
tools/portmap.sh           # the building model accounted for against BIMCraft's source, item by item
cargo run -p plumbline -- sample job.json
cargo run -p plumbline -- house house.json     # the sample whole house
cargo run -p plumbline -- takeoff house.json   # its walls, floors and rooms, and each system's runs by size, its fittings and what is joined
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
* Z undoes and Y does again. F2 saves.

Everything drawn in the window is flagged Assumed and says it was drawn there. Rooms are
placed at the command line: `plumbline room job.json <storey> "name" --at "6,8" --status
Assumed --source "..."`.

`refs/` is for the owner's reference estimates. Git ignores it. Nothing in it is ever committed.

Licensed under either of Apache License 2.0 or the MIT license, at your option.
