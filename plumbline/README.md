# BuildOS

(Its working name was plumbline, and the folder, the crates and the two commands are still called that: `plumbline` and `plumbline-app`. DECISIONS.md call 115 says what was renamed and what is owed.)

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
sudo apt-get install -y build-essential pkg-config curl git libx11-dev libxcursor1 libxi6 libxkbcommon-x11-0 mesa-vulkan-drivers
```

(The window opens these libraries by name when it starts, so a missing one shows as a
window that will not open, not as a build that fails. `tools/installs.sh` checks this
list against what the running window has open.)

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
demos/fixes197w.sh          # the six window points of call 197 and what came with them (set down listed, the Cabinets card, a refused corner, a mark left, the swing, an assembly changed, Priced as), a session at a time (ONLY="a b c d d2 e"); pictures in demos/captured/fixes197w
demos/plan.sh              # a plan laid under a level, given its scale from one dimension and traced over, at the command line
demos/field.sh             # what is measured on site put on a traced job: the floor above moves and the price with it, one clash clears, one becomes a finding
demos/fieldw.sh            # the same in the window, each reading typed as a laser in keyboard mode types it; pictures in demos/captured/field
demos/div.sh               # sixteen divisions: a whole house's lines in all sixteen, the recap by division, and the workbook's Divisions sheet
demos/library.sh           # the starter assemblies: no price in them, a house's takeoff in fourteen divisions, three prices by division
demos/partwork.sh          # work on part of a thing typed as a measure ("60 SF", with how sure and where from) and the starter assembly "Take down a partition", each against a figure worked by hand
demos/hidden.sh            # what a wall the work opens may be hiding: bearing and what is in it asked in the field, what a bearing wall that comes down needs, and what is in the way of it
demos/stretch.sh           # a wall moved takes the ends of the walls that meet it: an addition made longer, a partition slid between two walls, a move refused for a window it would strand, and --alone, each against a figure worked by hand
demos/joists.sh            # joists counted across a floor and rafters across a roof plane where a priced line names them: the short and the long way, an L, a gable and a hip roof, the starter library's Framed floor
demos/structure.sh         # a beam, its posts and their pad footings: typed sizes (nothing is sized), figures, price, the clash test, the list for the engineer, the BIM file, the three starter assemblies
demos/carries.sh           # what a wall carries and what it stands on, worked from what is drawn above and below it and the roof (and said to be only that): the joists said and not said, a wall to demolish with a wall on it as a caution and a field question with its dollars riding, a beam answering it, a wall said not bearing with a wall on it, nothing else moving, and a header's depth said to be a rule of thumb
demos/trim.sh              # the outside trim priced (call 185): each eave's and rake's overhang beside the wall under it, each wall's outside corners and corner board, each opening's casing, as the program works them and as worked by hand; the starter lines (fascia, soffit, downspouts with no quantity until typed, rake fascia and soffit, corner boards, exterior casing) with no price and then with made-up prices to the cent; an L (five outside corners, one inside); a roof alone (no wall under its edge: no overhang, and why); and the sample houses the same to the byte; and `plumbline trim <job> --pieces`, where each piece of outside trim lies (fascia, rake fascia, soffit, rake soffit, corner boards, casing), drawn only where a cost line prices it, the pieces of a kind adding up to its line (call 184, step 1); and what the judge of call 191 found: a line priced on another figure said and set against the total, a soffit with no overhang "no figure", the corners of a soffit said to be in two runs, quantities to 2 places
demos/sixteen-trim.sh      # the sample whole house prices its outside trim (call 194), at the command line on `plumbline sixteen`: corner boards on each outside wall (on the ground level from the top of the 3 ft stone, by the new figure corner_board_length_above_wainscot with the wainscot entered on the line), exterior casing at each window, fascia and soffit at each eave; each added line set beside the figure worked by hand (128 LF, 289 LF, 144 LF, 108 SF, 33 lines, $3,721.40), the recap and the price moved by exactly that and nothing else; a ground wall's boards 14 LF and refused with the entry taken off; `trim --pieces` laying two of 7 ft that start 3 ft up and the pieces adding to the priced 128 LF and 289 LF; `trim` saying what it said; the starter wainscot wall 14 LF where it was 20; no casing line on the front door and no corner boards on the foundation walls
demos/gable.sh             # a wall under a rake rises to the roof (call 192, the model's half): the gable ends of a gable roof, a shed roof and a hip roof and a house with no roof, each wall's gable area and its top as points beside the figure worked by hand; the starter outside wall priced on net_area + gable_area (the siding, sheathing, weather barrier and hanging bought over the gable) and the inside of it not; the frame of a gable said not to be counted
demos/kitchen.sh           # a kitchen's base that does not run behind the cabinets, the lump lines said to be in the price beside the things set, and the countertop priced and its corner leg said
demos/spec.sh              # items made from manufacturers' sheets (PDF), placed, moved clear of a clash, and out to an IFC4 model
demos/items.sh             # the item library: the things that can be placed, by trade, in a table that is brought in, written out the same, changed and brought back
demos/found.sh             # the foundation drawn: strip footings and foundation walls, footing concrete by the CY, and out to an IFC4 model
demos/foundation.sh        # foundation work taken off from the starter library: forms, concrete, bars, anchor bolts, dampproofing and a drain, each against a figure worked by hand, and the same figures in the IFC model
demos/shingle.sh           # a shingle roof priced whole: the roof's own figures, and sheathing, underlayment, ice barrier, shingles, starter, cap and drip edge in whole packs, each against a figure worked by hand
demos/framing.sh           # a wall's frame counted as a framer counts it: studs by the piece, plates in 16 ft pieces, headers in two plies, rough sills, each against a figure worked by hand
demos/tradesw.sh           # the layers in the window, a click each: finish off to the weather barrier, walls as frames, the ground off the foundation, one floor looked into
demos/moneyw.sh            # what the window says when money is at stake: a roof measured off the verify-in-field list, a labor line added, a priced wall deleted, a markup typed ten times too large
demos/roofw.sh             # a roof priced once in the window: an assembly given to one plane goes on all four, a price typed once prices every plane, a rate said in squares
demos/pagew.sh             # a plan and its page picked from a list in the window, with two clicks and nothing typed
demos/gablew.sh            # priced work taken off in the window: a wall's assembly changed, a hip made a gable; what went, and the job cost before and after
demos/itemlibw.sh          # the item library in the window: starter items by trade, one set down, an item opened and changed, the library written out and read back
demos/turnw.sh             # a panelboard set down into a wall on plan, turned a quarter to clear it, and widened in the library where it stands
demos/itemneww.sh          # an item typed into the job's library in the window, given its trade, renamed with what stands, kept from deletion while one stands, and another taken out
demos/benchw.sh            # the bench (call 195, step A): an item opened and put on a floor of its own, alone, drawn as it is set down, with its width, depth and height dimensioned; no tool offered; a size typed on its panel is Assumed and says how many set down from it follow; back to the house by the button and by Esc to the view that was left; the water heater, never set down, the same way; the saved job the same to the byte as without the bench
demos/quantityw.sh         # what a cost line is priced by, changed on the line in the window ("face area x 1.10"), and a figure the thing does not have refused
demos/scan.sh              # a scan of a room (a PLY point cloud) brought in as a sheet to trace: to scale, in its own place, Assumed; traced walls to the foot; text and bytes alike; refusals
demos/findwalls.sh          # walls, doors, windows and what stands short found in a scan, said in full and laid only on the word (call 173): two faces make a wall and its thickness, a wall seen from one side needs a type named, what is not seen is taken as wall and said, nothing is found without a ceiling; the house turned 17 degrees the same; and what call 179 asked of it (needs numpy: `tools/made-up-scan.py --scene list` names each made-up house and its right answer): a scan too thin or read the wrong way is said and nothing is found or laid on its word, a wall hidden behind cabinets is found and said not seen, whether the walls close is said, a bank of cabinets to the ceiling is laid only with `--take`, two ceiling heights and a slope are read, a wall is never laid leaning, a wall that cannot be laid is named and the rest laid (`--anyway` lays what is not likely a house); and what call 188 asked of it after three real phone scans (the scene `phone`: rough, bowed, off plumb, uneven and scanned in passes): a bowed wall is found and its bow said, a wall scanned in passes is found once and how many faces were joined is said, a house on its side is said by the shape of its floor and ceiling, a large scan says how it is going and is thinned first, and a left-out face of 6 ft or more is offered by its number as likely a wall and laid only with `--take`; and what call 189 asked of it (`--by-ceiling`: the walls found the way the published Cloud2BIM method finds them, by the points near the ceiling, where furniture does not reach, laid on the plan as how dense they are, their outlines made straight, opposite faces paired into walls, each wall's face then read for openings: the same words, the same doubts, the same laying)
demos/planwalls.sh          # walls, doors and windows found on a drawn plan (a PDF with lines of its own; call 181): the program's own plan set is brought into an empty job, scaled from one dimension and its walls asked for: pairs of lines a wall's thickness apart make a wall, a line across ends it, evenly spaced lines (a stair, boards) are not walls, a pair that meets nothing is in doubt and laid only with `--take`, a break in both lines is a door or (with lines across it) a window and is laid only where `--doors` and `--windows` give the heights, dimension lines, lettering, the border and the title block are left out and counted, a second laying is refused, everything laid is Assumed and sourced to the sheet; and, from sheets of its own (call 186), what a judge broke: a dashed pair (a beam) is no wall, a pier between a door and a window is part of its wall, a wall drawn in its layers is one wall and a line beside a wall's face is no second wall, a house drawn with single lines is told so, faces that run past a jamb do not end the wall, what the page cuts off and a picture on it are said, a head is above the floor and its sill, a sheet not lined up and a name that is no drawing are said, and a scale no drawing is made at is cautioned; and, from sheets drawn with what three real city sheets showed (call 188, items 6 to 9), a sheet of more than one drawing says so and lays none until `--plan <n>` or `--within` says which is the plan, a wall drawn with its faces twice is found once and says how many pairs of lines were joined, a shower's glass inside a room is no window and a tag is no opening, and the sheet's own dimension strings are listed (`--dimensions`) and offered to scale by (`--scale-by <n>`, Assumed, its source the string; strings that disagree are said); needs python3 (standard library only); and what the judge of call 191 found: --plan takes one drawing's walls and no others, a gap wider than 8 ft with nothing drawn in it is two walls, much of a sheet in doubt lays nothing unasked, a Known scale is kept unless --anyway, laid walls are named when the scale changes
demos/findwalls.sh          # walls, doors, windows and what stands short found in a scan, said in full and laid only on the word (call 173): two faces make a wall and its thickness, a wall seen from one side needs a type named, what is not seen is taken as wall and said, nothing is found without a ceiling; the house turned 17 degrees the same; and what call 179 asked of it (needs numpy: `tools/made-up-scan.py --scene list` names each made-up house and its right answer): a scan too thin or read the wrong way is said and nothing is found or laid on its word, a wall hidden behind cabinets is found and said not seen, whether the walls close is said, a bank of cabinets to the ceiling is laid only with `--take`, two ceiling heights and a slope are read, a wall is never laid leaning, a wall that cannot be laid is named and the rest laid (`--anyway` lays what is not likely a house)
demos/planwalls.sh          # walls, doors and windows found on a drawn plan (a PDF with lines of its own; call 181): the program's own plan set is brought into an empty job, scaled from one dimension and its walls asked for: pairs of lines a wall's thickness apart make a wall, a line across ends it, evenly spaced lines (a stair, boards) are not walls, a pair that meets nothing is in doubt and laid only with `--take`, a break in both lines is a door or (with lines across it) a window and is laid only where `--doors` and `--windows` give the heights, dimension lines, lettering, the border and the title block are left out and counted, a second laying is refused, everything laid is Assumed and sourced to the sheet; needs python3 (standard library only)
demos/movew.sh             # a wall moved in the window: its floor, footing and roof go with it and the card says so, one Undo takes it all back, a move across a wall is refused; pictures in demos/captured/movew
demos/tracew.sh            # a wall traced over a scan, set by the scan's face (call 190), in the window: a made-up house scan is brought in, its sheet's straight lines are places to click, the Wall tool is on Face over it and lays each wall with its body on the side away from the room (center 3 in off the line clicked for a 6 in type) and says which side it took, Other side lays them on the other, a partition seen on both faces is laid between them with the scan's thickness said beside the type's, a click with no face near stays where it was put and says so, Centerline lays the same clicks on the faces; opened again without the scan's file the faces are still found; pictures in demos/captured/tracew; needs python3 with PIL and numpy
demos/lightw.sh            # the same views of the whole house for a before and after of the light (SUN=1 for the Sun control); pictures in demos/captured/light
demos/trimw.sh             # the outside trim drawn (call 184, step 1): the house of demos/trim.sh with its trim lines given, opened in the window; the Show list's Trim switch reads the model's count (27), each fascia, soffit, corner board and piece of casing is a board where the model lays it and casts its shadow, Trim off is the house as it was drawn with no trim priced, a fascia line taken off an eave takes its board off, a click on a piece selects what it trims; measured against the program's map of where each piece is on screen; pictures in demos/captured/trimw; needs python3 with PIL and numpy
demos/gableendw.sh         # the gable drawn (call 192, the window's half): the house of demos/trimw.sh under a gable roof, a shed roof and a hip roof; the wall carried up to the rake and sided like the wall under it (the middle of the gable reads the siding's colour, and on the program before it does not), its panel reads Gable 40.00 SF and 204.00 SF under the shed's high edge, none on a wall under an eave, Finishes off shows the weather barrier on the gable, a hip-roofed house is the picture of the program before dot for dot (GABLE_BEFORE_BIN names that program); pictures in demos/captured/gableendw; needs python3 with PIL and numpy
demos/glassw.sh            # moving round the model, measured: the wheel closes on the pointer, a turn is about it, a flick carries on; pictures in demos/captured/glass
demos/carryw.sh            # a vanity picked up in the model and carried clear of a clash, red while it is in it; pictures in demos/captured/carry
demos/itemsw.sh            # in the window: a maker's sheet made into an item, one set down with a click, and the BIM model, workbook, CSV and lenses written out; pictures in demos/captured/items
demos/toolsw.sh            # a tool shows the sizes it will place and typing is never thrown away; pictures in demos/captured/tools
demos/pricew.sh            # the same in the window: Starter assemblies, an assembly picked from the list and changed, a price typed on the sheet
demos/dress.sh             # what the house is dressed in, read from what is priced on it: each wall, the roof, a window and a door, with the line that says so
demos/dressw.sh            # the same in the window: one cladding to a wall, the priced sample house, and the same house bare; pictures in demos/captured/dress
demos/vif.sh               # the verify-in-field list: each question, what to measure, and the dollars riding on it, before and after measuring
demos/export.sh            # the estimate as a workbook of formulas (worked again by a second hand), as CSV, three lenses, and the building as IFC4 checked by IfcOpenShell
demos/imports.sh           # a price list and an assembly library out as CSV and back; a supplier's list from CSV and from a workbook; a prior estimate set against the job
demos/model.sh             # another trade's model (IFC) brought in: what comes in, what is listed instead, and the clash it hid
demos/ifc-whole.sh         # a model brought in whole (call 197) and what is connected (call 200): the made-up whole house written out as IFC4 and brought into an empty job: its fixtures and equipment where they stood, each connection of a thing and each end of a run written as a connection point and the joined ones tied, so the open ends by system (supply air 4, cold water 1, sanitary 1, vent 1, power 1, the rest none) and the three hot-water runs with no size come back as they were, a run 0.5 ft from a fixture with nothing tied reads open and says so, its fittings on their runs (and where the file's word for one is not what the runs make, the place and both words), its footings, its roof (a hip, and a gable made on two edges) by the change the Roof tool makes, its names; what is left out says what follows; IfcOpenShell as the outside checker (the model, and each tie's two points at one place)
demos/ifc-runs.sh          # another trade's model into a job that has a building (call 210): the made-up whole house is the job; a designer's model made in the script (four walls and a floor over the house's own, one duct through a partition, one air handler) is brought in whole (the duct and the air handler come in, the walls and the floor are left out for lying over the house's, and the reply says only its runs can be chosen) and as `--only runs` (1 duct and 1 piece of equipment; 4 walls and 1 floor counted as not asked for; its level is the house's first floor and none is added; the price stays $468,050.22; the one clash it makes, a hole to cut where the duct goes through the partition at 24, is counted from the clash list before (47 holes) and after (48) and is on the list to verify in the field), the same file again refused, everything Assumed and sourced to the file, a run on a level at a height the job lacks put on the level it stands within, a model of walls alone brings nothing
demos/change-assembly.sh    # an assembly changed for another writes the quantities of the one it becomes (call 199): a wall changed for the wainscot wall and back, each shared line named with what it was and what it is, a quantity typed by hand left as entered and what the new assembly would write said
demos/itemcost.sh           # an item carries its cost (call 196, the bench's step B): "Water heater, 50 gal" set down twice and said to be set down and not priced; priced as "Bought and set", the two follow; the price list prices it from two suppliers side by side; "Set in place" at 4 hr and $85.00/hr; priced as another assembly and back, the hours named when they go; the chest written out is a priced table that comes back to the byte
demos/pricelist.sh           # a price set where it is seen (call 207, the model's half): the price list as rows (`price <file> list`) and one change, `set <item or material, by name> price`, that sets or clears the estimator's price or a named supplier's and says the lines it prices, on how many things, and the job cost before and after: $1,850.00 each makes $3,700.00, a new supplier's $1,790.00 makes $3,580.00; cleared is no price and never $0.00, $0.00 typed is taken and marked, refusals leave the job as it was, four suppliers, a rename, a list over a typed price, and the list written out comes back byte for byte
demos/shapes.sh               # an item's shape as data (call 198, the bench's step C): the 36 in vanity's three blocks printed from the rows of the starter shape table; the starter shapes held to the code they replaced; a shape of the user's own brought in with the chest, an item drawn as it, its blocks worked by hand; the two tables written out and brought into a second job, the same to the byte; an item naming a shape that is not there refused; a job saved before opens as it did; a cost line written min(net_area, 100 SF); and (call 205) a shape changed by hand, in seven steps: the vanity's shape kept as a new shape, a block added and its six figures typed, a figure that leaves the box refused (how far, which way, at which size), a starter shape not changed, a starter's name refused on the way in, a block taken away, the job saved and written out the same to the byte
demos/connections.sh         # where an item connects (call 211, the bench's step D, the model's side): an item of three connections shown at its sizes in inches; one set down has them, Assumed, naming the item; a pipe ending at one joins it and is no clash, one 3 in short of one is a clash that says so and is refused by connect; the item's height typed, the thing follows and a run left standing off is counted; the thing turned and its connections with it; three refusals; the rows written out and brought into a new job to the byte; the BIM file with three connection points, read back; the sample house byte for byte as before
demos/judged-1125.sh          # what the command-line judge of 2026-11-25 found, fixed (call 206): nothing typed on a line lost unsaid when an assembly or an item's pricing changes (a quantity of 7 on a $1,000.00 line: $7,000.00 named); a shape held inside its item's box at the item's own sizes; a model brought into a job with a building (the level at the job's height is the job's, and the living room 614.72 SF to 365.72 SF is named); and the words and refusals of points 4 to 14
demos/planw.sh             # the same in the window, on a real scanned sheet and on a vector plan; pictures in demos/captured/plan
demos/build.sh             # a floor from the outline its walls enclose, rooms and a hip roof, at the command line, with the figures worked by hand
demos/buildw.sh            # the same in the window, traced over the made-up plan with the Floor, Room and Roof tools; pictures in demos/captured/build
tools/fetch-plans.sh       # fetches the public-domain plan set the demonstrations trace (USDA Plan No. 7213) into plans/
demos/d4w.sh               # the same in the window: the sheet's Budget tab, and prices typed on what is selected
demos/d2.sh                # the 3D prototype, driven on a virtual display; pictures in demos/captured/d2
demos/house.sh             # a whole house with its heating, plumbing and wiring, walked round; pictures in demos/captured/house
demos/sweep-money.sh       # what the sweep found wrong in dollars (call 175): siding not bought over stone, base not across a doorway, a thing set against its wall is not a clash, a hole counted once, the BIM file's own word for existing, demolished and new; each against a figure worked by hand
cargo run -p plumbline-app -- job.json    # the model window and the sheet window (every button says its key)
tools/licenses.sh          # every linked crate and its license; fails on a non-permissive one
tools/installs.sh          # the README's install list brings every system library the running window opens
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

The model fills the window. What is on screen with it is a layer of frosted glass
panels floating over it (the scene behind each one, blurred, under a tint), each with
round corners and a shadow:

* **Along the top**: the job, Save, Undo, Redo, and at all times the job's price in the
  color of how sure it is, what is to verify and the dollars riding on it.
* **Over the model's top right**: the view (3D, Plan, Walk) and Frame, which brings the
  whole job back into view.
* **Down the left**: Next steps (what this job needs next, read from the job), the
  levels, Show (a switch for the ground, the finishes, the framing, each trade, the
  roof, see-through, how sure, the clashes), the systems, the drawings, Bring in
  (price lists, assemblies, a prior estimate, an IFC model, a maker's sheet), Items
  (things to place, made from makers' sheets), and Write out (the BIM model, the
  estimate, the lenses). A level and a system each have a switch of their own. Click
  a level and it is shown alone, looked down into.
* **Along the bottom**: the build bar. Every tool is a button with its mark: Building
  (Wall, Door, Window, Floor, Room, Roof), Mechanical, electrical, plumbing (Duct, Pipe,
  Conduit, Cable, Fitting, Equipment, Fixture, Connect) and Edit (Move, Delete). The
  number keys 1 to 9 and 0 take the first ten in hand when no tool is in hand. With a
  tool in hand a number is the start of a length (once a line has its start) and never
  another tool: click the tool you want, or press Esc first.
* **Over the build bar**: the sizes of the tool picked. They are shown every time a
  tool is picked, filled with what it will place; type over any of them, and Enter
  takes the tool in hand.
* **Down the right**: what is selected, as fields and buttons. A value that can be
  measured says Measure or Check.

**Moving round the model** is by the pointer, the way a map is moved:

* The **wheel** closes on what is under the pointer (and backs off from it).
* A **drag** (left button, with no tool in hand; or the middle button) slides the
  ground with the hand: the place grabbed stays under the pointer.
* A **right-drag** turns the view about what is under the pointer.
* Let go while moving and it **carries on** and eases to a stop.
* A **double-click** flies to the thing clicked.
* Over a panel the wheel scrolls that panel, and the model stays still.

**Picking a thing up.** Click a fixture, a piece of equipment or a furnishing to select
it; press on it and carry it over the floor. While it is in the same space as another
thing it is drawn red and the line across the screen says what it is in and by how
much; let go and it stands there. The whole carry is one Undo.

Every button says the key that does the same. The words on screen are US English and a
drawing set's: a floor of the building is a level.

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
plumbline import ifc job.json their-model.ifc --status Assumed --source "the mechanical contractor's model of 3 March" --only runs   # another trade's model of the same building: only its runs and what they serve; its walls and floors are not asked for
plumbline import spec job.json sheets/vanity.pdf        # a maker's sheet made into an item to place
plumbline place job.json <level> <item> --at "7,0.875" --status Assumed --source "set by eye"
```

A price list is a CSV file or a workbook with a row of headings: what is priced, what
it is bought by, and the price. The program finds the headings, sets the prices it can,
and lists every row and column it did not use and why. An estimate made before is kept
beside the job and set against it line by line with `compare`; it adds nothing to the
job's price. A model from another trade or a designer (IFC) comes in as levels, walls
with their doors and windows, floors, rooms, ducts, pipes, trays and cables; what the
program has no element for, or cannot read the shape of, is listed with why.

A **maker's sheet** (a cut sheet, as a PDF with words on it) is read into an *item*: a
thing that can be placed any number of times. What is read off it is its name, its
maker, its model number, its width, depth and height, and its weight. Each is Assumed
and its source is the sheet, the page and the line as written (`vanity.pdf, page 1:
"Overall dimensions: 36" W x 21" D x 34-1/2" H"`); what the sheet does not say is
Unknown and is listed to be entered. A size is taken only where the sheet writes it
with a unit and says which way it runs, or writes three in a row. Equipment, fixtures
and furnishings (casework, furniture: Division 12) are the three kinds of thing an
item can be; the name says which, or `--as` does. A scanned sheet is a picture of
words and is refused with that reason. In the window it is Bring in, Maker's sheet,
and the item appears under Items: click it, then click where one stands.

## Send it out

```
plumbline export estimate job.json estimate.xlsx        # a workbook: each amount, the recap and the price are formulas
plumbline export estimate job.json estimate.csv         # the same lines as values
plumbline export supplier job.json to-quote.xlsx        # what to quote, by what is bought; none of our prices
plumbline export subcontractor job.json scope.csv       # the scope classed Subcontract; none of our prices
plumbline export pm job.json for-the-pm.xlsx            # every line with its hours, the budget by class, what is to verify; no markup
plumbline export ifc job.json building.ifc              # the building as an IFC4 model, lengths in feet
```

In the window the same six are under **Write out**, one click each: the file is
written beside the job under the job's own name (`job.ifc`, `job estimate.xlsx`, ...).

In the workbook, change a quantity or a price and the amounts, the recap and the price
follow: the formulas are the program's own arithmetic, to the cent. Each lens is the one
estimate with what its reader is not shown left out, and says at its head what that is.
The IFC model carries every value's status and source with it (what is Assumed here is
still said to be Assumed there), and the command lists what a building model has no
place for. A scan of the building (a PLY point cloud, as a phone's LiDAR scanner writes) comes in with `import scan <job> <scan.ply> --level <level>`: it is cut 4 ft above its floor into a sheet under that level, to scale and in the scan's own place, to trace over as any sheet is; it is taken as metres with z up unless `--units ft` or `--up y` says otherwise, and everything from it is Assumed. An estimate written as CSV reads back in with `import prior`. `estimate <job> by-item` reads the estimate as an estimator writes it: each item once, with how many places it is in, under its division. What is to verify in the field goes out as a sheet to print with `export vif <job> <out.xlsx or out.csv>`, or Field list under Write out in the window.

## Assemblies, and pricing

An assembly is everything priced on one thing: a wall's framing, sheathing, siding,
gypsum board and paint, each line in its own division. The program ships with a
starter set, 30 assemblies for a house's walls, floors, rooms, roofs, doors, windows
and runs, with a line in every one of the sixteen divisions and **no prices**: those
are yours. Eighteen of its lines are labor (framing a wall, hanging siding, setting a
window), each with **no production rate and no wage**: on the sheet such a line reads
"no production rate"; click that and type how fast your crew does it (`40 SF/hr`, or
`0.5 hr/EA`), then its price by the hour.

In the window: under **Bring in**, click **Starter assemblies**. Click a wall, then
**Assembly**, and pick one from the list. Its lines are on the sheet's Estimate tab,
each with a field for its price; the recap, the sixteen divisions and the price on
the top bar follow as you type. Pick another assembly for the same wall and it
changes: lines the two share keep their prices, and the status line says what came
off.

The starter assemblies carry material and subcontract lines and no labor. To put a
line of your own on anything, labor included: click the thing, then **Add cost
line**. Type what it is; click what it is priced by in the list under the field (the
thing's own takeoff: net area, plan length...); and type its cost type. In-house
work is by the hour, so its cost type carries how fast it goes: `labor at 40 SF/hr`.
The line has no price until you type one on the sheet.

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

**A trade at a time.** The same rule goes under the finish. Switch Finishes off and
the siding, the stone and the casing come off: a wall with a weather barrier priced
on it ("Sheathing and weather barrier", "house wrap", "Tyvek") is drawn in its
barrier, and an opening with "Flashing and sealant" priced on it has its flashing
round it. Switch Framing on and each wall with a framing line ("Wall framing, 2x6 at
16 in") is drawn as its frame: plates, studs on their centres, and a header, jacks,
kings, sill and cripples at each opening. A room with base priced on it has its base
drawn at the foot of its walls. Switch Ground off and what is under it is seen:
strip footings and the foundation walls on them, drawn with `plumbline footing` and
`plumbline wall` on a level under the first floor. Nothing is drawn that no line
prices, and no switch changes a figure.

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
