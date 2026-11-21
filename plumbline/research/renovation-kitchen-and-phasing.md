# A kitchen designed by hand, and a job of existing, demolished and new work: what exists (2026-11-21)

Patrick is starting a real renovation. He must design the kitchen and cabinetry himself, and
the job mixes work that stays, work that goes and work that is new. This is a survey, in the
order the repository asks: this tree, BIMCraft's source (on this machine), then the web.
Nothing is built. No code was edited. Web figures are from search results and page fetches
made on 2026-11-21; what could not be opened is said so at the foot.

## 1. What BuildOS has today

**For a kitchen.** A thing is an item (`library/items.csv`, 21 rows, four trades): a name, a
kind, a form it is drawn as, and width, depth and height, each with how sure and where from.
The kitchen rows there are: Range 30 in (30 x 26 x 36, drawn as a box), Refrigerator
(36 x 30 x 70), Dishwasher (24 x 24 x 34), Washer and Dryer, **one** Base cabinet, 36 in
(36 x 24 x 34 1/2, form "base cabinet"), Vanity 36 in, Wardrobe, and Kitchen sink, double
bowl (33 x 22 x 8, form "sink"). All sizes Assumed, "a typical size; the product is not chosen".
(`tables.rs` `items_in` line 673, `items_out` line 637, `items_in_missing` line 1492;
DECISIONS.md calls 105, 107, 109, 111.)

What a person can do today: bring in or write out the item table in a spreadsheet; add,
rename and delete items in the window (call 111); make an item from a maker's sheet
(`spec.rs`: `read` line 317, `item_of` line 455, sizes Assumed and sourced to the line of the
sheet); set a thing down, pick it up, move it, turn it (`editing.rs` `placed_follow`, `turn`);
see it clash live with walls, pipes and other things; see a base cabinet drawn as a plinth,
carcass and worktop (`building/forms.rs`, ported from BIMCraft `family.rs`, 13 forms, 3 ours).
In short: **a person can place sized boxes, one at a time, and see them clash.**

What is missing for kitchen design:
- no cabinet **run**: nothing sets cabinets edge to edge along a wall, or says "this run is 14 ft 6 in";
- no **filler**, **scribe**, end panel, toe kick as a thing, or **corner** cabinet (blind or lazy susan);
- no cabinet **codes** (B36, W3030, SB36, T-): there is one 36-in base and the item list is
  by trade, not by base, wall and tall;
- **wall cabinets and tall cabinets** do not exist as items; the form drawn is the base one only;
- **countertop by the foot**, taken off from the cabinets (area in SF, edge in LF, sink and
  range cutouts, backsplash) does not exist; the "Worktop" is a block of the form, not a priced line;
- no **door style**, drawer count, hardware, finish, or cabinet **cost per box** from a price list;
- no **cut list** or sheet-goods count (a cabinet shop's, or a builder's who buys boxes: a
  list of boxes by code is the one he needs, which is a schedule, not a cut list);
- no appliance **opening** or clearance rule (door swing, 30 in in front, range clearance);
- no way to hold an item's **mounting height** (a wall cabinet is 54 in above the floor, a
  thing set down stands on the floor).

Not found in `research/`: nothing on cabinets. `item-libraries-survey.md` is on workbench
libraries (rgeometry, plexus, fornjot) and is not about kitchens. `lidar-renovation.md` and
PROPOSALS.md 33 are about existing, demolish and new (below).

**For existing, demolish, new.** Nothing is in the model. PROPOSALS.md 33 (2026-11-17)
describes it: a value on every thing (existing is drawn light and not priced; demolition is
drawn dashed and priced as demolition; new is drawn heavy), carried out and in by IFC, "waits
for Patrick's word", 3 to 5 days, not prototyped. `crates/ifc/src/lib.rs` writes no Status.

## 2. What BIMCraft already has (the first thing to port)

BIMCraft is at /home/user/ipedrax/bimcraft, head f202abd85d1cf165325e2f7b250da0b5e51f5b71,
`LICENSE-MIT` and `LICENSE-APACHE` in the root, `Cargo.toml` line 8 `license = "MIT OR
Apache-2.0"`. It is the program BuildOS ports from (PORTMAP.md). Both subjects were searched
with `grep -rniE "demolish|phase" crates` and `grep -rniE "cabinet|casework|furniture" crates`.

### Phases: BIMCraft has them, whole, and they are Revit's

`crates/core/src/phasing.rs` (1,347 lines, the module). Its head comment (lines 1 to 9):
"A project has phases in order (Existing, New Construction): each element is made in one
(its `Phase Created`, the first unless it says) and may be taken down in a later one (`Phase
Demolished`); a view has a phase, and a phase filter that says how each of the four things an
element can be in that phase (new, existing, demolished, temporary: made and taken down
within it) shows: as its category does, as the project's overrides for it say, or not at all.
... values elements hold among their parameters."

- `Status` enum, lines 40 to 51: New, Existing, Demolished, Temporary. `Status::name` line 54.
- `Presentation` line 67 (how a status shows: by category, overridden, not displayed);
  `PhaseFilter` line 79 (`of`, `set`); `default_phases` line 120 (Existing, Demolition,
  New Construction); `filters` line 125 (Show All, Show Complete, Show New, Show Previous +
  Demolished, Show Previous + New); `overrides` line 146 (per status line weight, dash, colour).
- `index` line 176, `value` line 182 (a door or window with no phase of its own follows its
  wall: a rule BuildOS has too, for openings), and **`status(m, e, at)` line 192**: from a
  created phase and a demolished phase, and the phase looked at, one of the four. This is the
  one function that matters: 14 lines.
- Commands: `phase` (add, rename, combine; `commands.rs` line 1834), `phase_filter`, a view's
  `phase` and `phase_filter` (`commands.rs` lines 3749 and 3895).
- Drawing: `ui-egui/src/section.rs` line 303 and 398 (a Demolish tool: click an element and
  its `Phase Demolished` is set to the view's phase); `graphics.rs` line 120 (Phase and Phase
  Filter group); tests in `ui-egui/src/tests.rs` lines 5197 to 5335 (phasing dialog,
  filters, overrides, "what is drawn in a view with a phase of its own is made in that phase").
- **IFC out: `crates/core/src/ifc.rs` lines 1617 to 1619.** "A phased project's elements say
  what the work does to them, as IFC's `Status` (PEnum_ElementStatus) in their `Pset_*Common`:
  taken down in any phase, DEMOLISH (TEMPORARY when made in that phase too); made in the first
  phase, EXISTING; else NEW ... A project with no phases says nothing: all of it is new work."
  The `Pset_*Common` per element is chosen at `ifc.rs` line 665 to 677. Also a property set of
  its own, `BIMCraft_Phasing` (line 90), carries the phase created and demolished by name, and
  the project's phases in order (lines 1633 to 1667).

What it lacks for BuildOS: it has no price; a phase here is a drawing filter, not an
estimating one. It has no import of Status (IFC in). It is a view-by-phase model with
filters, much more than a renovation estimate needs. **The part to port is `Status`, the
Status-in-IFC mapping and the draw-by-status idea; not the phase list, the filters or the
per-view phase.**

### Cabinets and casework: BIMCraft has a category and one form, and no kitchen

- `crates/core/src/family.rs` line 31: `Category::Casework`; line 94: casework and furniture
  are written as IFCFURNITURE; line 157: IFC classes read in as furniture (IFCFURNISHINGELEMENT,
  IFCSYSTEMFURNITUREELEMENT).
- `family.rs` lines 2095 to 2125: the library's **Wardrobe** and **Base Cabinet** (millimetres,
  1800 x 600 x 900; plinth set back 60 for toes, carcass, a worktop block over the front;
  types 1800, 1200, 600). This is the only cabinet in the source, and it is what BuildOS
  ported as `forms.rs`.
- `sample/tower.rs` line 335 and `sample/highrise.rs` line 52: a Base Cabinet placed in a
  sample. `commands.rs` line 1073: a family's category may be `casework`.
- `crates/rvt/src/generic.rs` line 64, `inventory.rs` line 172: Revit's Casework category
  (-2001000) read as an envelope only.
- Searched `kitchen|worktop|countertop|filler|toe` in `crates/core/src`: hits are dimensions,
  loads and demand (electrical) and `render/dimensions.rs`; **no cabinet run, no filler, no
  corner, no countertop quantity, no cut list, no code naming.**

So BIMCraft gives phasing and the one cabinet form; for the kitchen itself it has nothing
more than BuildOS already took.

## 3. Cabinetry and kitchens, online

### 3a. Open data: sizes and naming

- **KCMA** (Kitchen Cabinet Manufacturers Association) publishes **ANSI/KCMA A161.1-2022**,
  a performance and construction standard for kitchen and vanity cabinets (strength, finish,
  durability). Found as `kcma.org/sites/default/files/2024-08/KCMA A161.1 2022 High Res.pdf`.
  It is not a naming or sizes standard. (Search result only; the PDF was not opened.)
- **The codes are the trade's by habit and each maker's by catalog.** Letter plus width plus
  (for walls) height, in inches: B36 base, 36 wide; SB36 sink base; W3030 wall, 30 wide, 30
  high; with tall (T or U, width and height) and corner and filler codes varying by maker.
  From catalogs found in search (a distributor's, a Rugby/KCD one, MFS Supply's spec book
  v2025.2 and a Home Depot-hosted catalog PDF): base **34 1/2 in high, 24 in deep**; wall
  **12 in deep**, heights 30, 36, 42; refrigerator-end cabinets 24 in deep; widths step by
  **3 in** (base 9 or 12 to 48; wall 9 to 42). **Tall cabinet sizes were not confirmed.**
  Maker catalogs are copyrighted documents; the sizes in them are facts, and the program
  reads sheets for sizes already (`spec.rs`). A maker's code table typed in by hand
  from a spec book the builder holds is his data, Assumed and sourced to the page.
- **IFC libraries of cabinet objects:** none found that are open and kitchen-specific.
  BIMobject and manufacturers' own sites offer cabinets as IFC and RFA behind logins and
  terms that were not read. **Not verified.**

### 3b. Open-source cabinet and kitchen code

| What | Licence at its source | What it does | Use |
|---|---|---|---|
| **dprojects/Woodworking** (FreeCAD workbench, Python), github.com/dprojects/Woodworking | **MIT** (sidebar and README: "licensed under MIT license, only with the copyright preserved") | simple cabinets and furniture from panels; dowels, fittings, drilling; **`getDimensions`: automatic cut list** with weight and cost, exported to CSV, JSON, HTML, Markdown | Port is permitted. It is Python on FreeCAD's API, not kitchen planning (no runs, fillers, counters). Worth reading for its cut list columns |
| **petervanderwalt/cabinet-studio** (JavaScript, browser) | **none stated** (no LICENSE file on the page) | parametric cabinets: carcass, doors, drawers, shelves, overlay, reveal, toe kick; copy counts for a run; panel DXF, nested DXF; a room-level layout is "upcoming" | Unlicensed means all rights reserved. Reference only until the author adds a licence |
| **lairdubois OpenCutList** (SketchUp extension, Ruby) | **GPL-3.0** (sidebar and "This code is under the GNU GPLv3 license") | cut lists and bills from a SketchUp model | Reference only; never linked. It runs in SketchUp, which is not open; no use beside the program |
| **Sweet Home 3D** (Java) | **GPL** for the program and its two library editors; bundled 3D models and textures "GNU GPL or CC-BY" per its licence page (sweethome3d.com/license) | home layout, furniture catalogs (kitchens among them), a furniture library editor | A separate program used beside, exchanging a file (OBJ out), is the owner's choice. Its models are GPL or CC-BY: not to be copied into the repository |
| **FreeCAD** itself | LGPL (from memory; **not verified**, the repository page could not be reached from here) | parametric CAD; the Woodworking workbench above runs in it | Beside, as a drawing tool whose STEP or IFC can be read, if he likes |
| **AutoParam3DMax/kitchen_cabinet_iLogic** | MIT (search listing) | a parametric cabinet in Autodesk Inventor iLogic | Needs Inventor; not usable |
| Blender add-ons, OpenSCAD libraries for cabinets | not found with a stated licence in this pass | | Not verified; a further search would be needed |

`gh` and the GitHub API were refused for these repositories in this session, so licences are
from the repositories' pages as fetched, not from `LICENSE` text.

**Reading of 3b: no open-source kitchen planner was found that both fits and can be ported.**
The only permissive one (Woodworking) is a panel and cut-list tool for FreeCAD. The kitchen
logic Patrick needs is small and settled enough (3c) to write.

### 3c. How professional kitchen tools structure a design

Copy, do not invent. What the trade's tools (2020, ProKitchen, Cabinet Vision, KCD and the
maker-linked planners) do, as published by them and by the NKBA, in short:

- **A run is the unit.** Cabinets are set in a run along a wall from a start, left to right,
  each a catalog code and a width; the run's length is the sum; what is left over to the wall
  is a **filler** (3 in standard, cut to fit, up to about 6 in) or a **scribe** strip; an
  inside corner takes a **blind corner** cabinet with a filler, or a lazy susan; an end gets
  an **end panel**. The run is checked: sum of boxes plus fillers equals the wall length, or the
  tool says by how much it does not.
- **Toe kick** is a 4 in high, 3 in deep recess in the base (BuildOS's form already has a 60
  mm one from BIMCraft). **Countertop** is takeoff by the run: LF of front, SF of top
  (depth 25 1/2 in over a 24 in base), plus backsplash LF, plus cutouts for sink and cooktop,
  with seams at corners.
- **Appliance openings** are fixed dimensions (a dishwasher 24 in wide, a range 30 in, a
  refrigerator opening commonly 36 in wide); they take their place in the run as a gap, not a
  box.
- **NKBA Kitchen Planning Guidelines** (NKBA, kb.nkba.org/uploads/2022/05/Kitchen-Planning-Guidelines.pdf,
  2022). The page itself could not be opened (HTTP 403). From its search snippet: the work
  triangle's three legs total **26 ft or less, no leg under 4 ft or over 9 ft**, and no major
  traffic through it. Other figures are from secondary sites and are recorded here as **not
  verified against the NKBA document**: walkways at least 36 in, a work aisle at least 42 in
  (48 in for two cooks), a 24 in landing beside a cooktop or sink. A builder can hold the PDF
  and have the clearances typed in as his own rules, each sourced to its guideline number.

## 4. Existing, to demolish, new

**The standard.** IFC's common property sets carry a **Status** with the enumeration
`PEnum_ElementStatus`. From the buildingSMART IFC 4.3 documentation (search extract of
`Pset_WallCommon`, ifc43-docs.standards.buildingsmart.org): "New" - element designed as new
addition; "Existing" - element exists and remains; "Demolish" - element existed but is to be
demolished; "Temporary" - element will exist only temporary. The property is repeated in
every common set (Pset_WallCommon, Pset_SlabCommon, Pset_DoorCommon ... ). The buildingSMART
forum thread "How to handle model construction phases in IFC4" says these are set "on an
element-by-element basis" through that Status property; a furnishing element's Status belongs
on its subtype's set (IfcFurniture: `Pset_FurnitureTypeCommon`), not on IfcFurnishingElement.
Same thread: Status in different psets carries different identifiers in bSDD, a known
deficiency. Whether the Status property is in IFC4 itself or arrived in 4.3 was **not
confirmed** from these pages; `lidar-renovation.md` says IFC4 has it and IFC2x3 does not,
which the IFC4 documentation was not re-read to prove here. Values are case-sensitive in
Bonsai's filters (OSArch forum: `Pset_WallCommon.Status = "Existing"`).

**Revit** holds it as two phase pointers per element, Phase Created and Phase Demolished; the
four statuses come from those and the phase looked at (SketchUp's help page "revit phases
sketchup" and Autodesk help "About Phasing"). A new project has two phases, Existing and New
Construction. Revit's IFC export exports one phase at a time, and phase status is not carried
in a standard property by default (community posts; not vendor-confirmed).

**Bonsai (BlenderBIM)**: no phase object found in IFC itself beyond that Status; Bonsai
users filter by it (OSArch forum "Versioning/Phasing?"). Bonsai's own handling of demolition
was not read beyond that.

**The smallest faithful way for BuildOS to hold it:** **one value per element with four
words**, New, Existing, Demolish, Temporary, taken from IFC's own names. It is IFC's own
design, it round-trips out and in without loss, and BuildOS has no use for phases-in-order or
views-by-phase for one renovation (BIMCraft's Created-and-Demolished pair is the general case
of this and is more than needed; it can be added later by naming the phase). Default: no
value means New (every job today; the same rule as BIMCraft's "no phases says nothing: all of
it is new work"). It is Known, Assumed or Unknown like every value (a wall traced from a scan
is Existing, Assumed until measured), and says where it came from.

**How demolition is priced.** Consumer cost sites (HomeGuide, Angi, HomeAdvisor; lead
generators, ranges differ by 2x between them, so treat as shape and not as price): whole
kitchen demolition about $5 to $12.50 per SF of floor; **cabinet removal by the LF**, $15 to
$35 per LF; a non-bearing wall removal as a lump per wall or about $0.50 to $1.50 per SF of
wall for demolition only, debris and patching separate; flooring removal $1 to $5 per SF;
**dumpster** $265 to $620 per week for 15 yd (or $200 to $800 a week); permits, utility
disconnects, hazardous material (asbestos $5 to $20 per SF) as separate lines; **10 to 20 %
contingency** on the demolition number for what is behind the walls (one selective-demolition
contractor guide). Professional estimators price selective demolition by the same units a
builder already uses: SF of wall, floor or ceiling removed, LF of cabinet, base, rail, pipe
or conduit, EA for fixtures, doors, windows, and LS for the dumpster and disposal by the
load. This is divisions 02 (Site Construction) of the sixteen in MasterFormat 1995, which
the program has. The published unit-price books (RSMeans) are copyrighted and were not
opened. The program's own divisions, assemblies and "Assumed" rule would hold the rates.

## 5. Ranked recommendation

### For designing a kitchen this month

**1. A cabinet run tool on the existing items, with a code table (biggest gain).**
- *Adopt:* trade's settled practice from 3c and a maker's catalog the builder picks (sizes
  read from its sheet by `spec.rs`). Cut-list ideas from **Woodworking (MIT)**
  github.com/dprojects/Woodworking if a cut list is wanted later; port only with its
  copyright preserved and an entry in PORTMAP-like notes.
- *Write in BuildOS:* items for base, wall, tall, sink base, corner, filler and end panel by
  code (B36, W3030, SB36 ...) in `library/items.csv` (sizes Assumed, sourced to the catalog
  page); a **run**: a start on a wall, a list of items, the sum against the wall length with
  a filler for the difference and a plain message when it is over; wall cabinets with a
  mounting height; a countertop line by the foot (LF of front, SF of top, backsplash LF, cutouts)
  worked in the core, so it prices; a cabinet schedule by code with counts.
- *Could go wrong:* codes are each maker's, so a B36 is not the same box between makers
  (fix: the code is a name only; the sizes are the item's, sourced); one arithmetic for the
  run's length must live in the core; the box form is not the cabinet (a drawn door style and
  drawers are visual only and must not be priced as if known).

**2. Kitchen rules as checks, from the NKBA guidelines.** *Adopt:* the NKBA PDF, cited by
guideline number. *Write:* work aisle, walkway, landing and appliance clearances as
clash-like checks, Assumed rule by rule. *Wrong:* secondary-site figures were not checked
against the PDF; the guideline is a guideline, so the result is a "check", not a finding.

**3. Beside, not linked: Sweet Home 3D (GPL) for looking at a layout** if he wants a quick
picture before the run tool exists; exchange by OBJ. The owner's choice, never in the tree.

### For existing, demolish, new

**1. One value per element, four words, taken from IFC (smallest faithful design).**
- *Adopt:* IFC `PEnum_ElementStatus` (buildingSMART, open standard, no licence trouble) and
  **BIMCraft's `phasing.rs`** (MIT OR Apache-2.0, commit f202abd): the `Status` enum (line
  40), the status-to-IFC mapping (`ifc.rs` lines 1617 to 1619), its Pset choice (line 665),
  and the Demolish tool on a section (`section.rs` lines 303 and 398). Not the phase list,
  filters or per-view phase.
- *Write:* a Status on every wall, opening, floor, room, run, placed thing and fitting, saved
  in the job (a promise to old jobs: absent means New); the estimate reads it: **Existing is
  drawn light and not priced; Demolish is drawn dashed and priced from a demolition assembly
  by SF, LF or EA (with a haul-off line); New is as today**; a Status shown in a list and a
  filter; IFC out as Status in the right `Pset_*Common` (IfcFurniture's own set for
  furniture) and IFC in. A demolished thing still takes its own quantity from the model.
- *Could go wrong:* it changes the saved job and the estimate's meaning (PROPOSALS.md 33
  says it waits for Patrick's word: ask him); a total that leaves Existing out must say it
  did; the IFC Status property is defined separately in each Pset, so the writer needs a
  table per class (BIMCraft's is the guide); furniture Status goes on the subtype's set.

**2. Demolition assemblies in division 02**, by SF, LF and EA, with a dumpster by the load and
a contingency line, all Assumed and sourced "typed by the builder". Rates come from his own
subs' quotes, not from consumer cost sites.

## What could not be verified

- The NKBA PDF and the KCMA A161.1 PDF were not opened (403 or search only); the clearance
  figures other than the work triangle are secondary.
- Tall cabinet sizes, filler widths and the 4 in by 3 in toe kick: from memory of the trade,
  not confirmed in a source this pass.
- Licences of FreeCAD, Blender add-ons and OpenSCAD libraries; licences of the four repos in
  3b are from their pages, not from the `LICENSE` text (`gh` was refused).
- Whether IFC4 (not only 4.3) has `Status` in the common sets, and how Bonsai itself writes
  demolition.
- Demolition unit costs are from lead-generation sites, ranges that disagree; no RSMeans or
  estimator's book was seen.
- Open IFC cabinet libraries: none found; BIMobject and makers' terms not read.

## Sources

- buildingSMART IFC 4.3, Pset_WallCommon: https://ifc43-docs.standards.buildingsmart.org/IFC/RELEASE/IFC4x3/HTML/lexical/Pset_WallCommon.htm
- buildingSMART forum, construction phases in IFC4: https://forums.buildingsmart.org/t/how-to-handle-model-construction-phases-in-ifc4/823
- OSArch forum, versioning and phasing in BlenderBIM: https://community.osarch.org/discussion/1030/
- Autodesk, About Phasing: https://help.autodesk.com/cloudhelp/2023/ENU/Revit-DocumentPresent/files/GUID-BDCE1B94-58D0-401B-863B-2708D36D54EA.htm
- SketchUp, Revit phases: https://help.sketchup.com/en/revit-phases-sketchup
- NKBA Kitchen Planning Guidelines: https://kb.nkba.org/uploads/2022/05/Kitchen-Planning-Guidelines.pdf
- KCMA A161.1: https://kcma.org/sites/default/files/2024-08/KCMA%20A161.1%202022%20High%20Res.pdf
- Woodworking (MIT): https://github.com/dprojects/Woodworking
- cabinet-studio (no licence stated): https://github.com/petervanderwalt/cabinet-studio
- OpenCutList (GPL-3.0): https://github.com/lairdubois/lairdubois-opencutlist-sketchup-extension
- Sweet Home 3D licence: https://sweethome3d.com/license.jsp
- HomeGuide, kitchen demolition: https://www.homeguide.com/costs/kitchen-demolition-cost
- HomeGuide, cabinet removal: https://www.homeguide.com/costs/cost-to-remove-kitchen-cabinets
- Angi, interior demolition: https://www.angi.com/articles/how-much-does-it-cost-demo-interior.htm
- Block Renovation, selective demolition: https://www.blockrenovation.com/guides/selective-demolition-contractors
- BIMCraft (MIT OR Apache-2.0), /home/user/ipedrax/bimcraft: `crates/core/src/phasing.rs`, `ifc.rs`, `family.rs`
