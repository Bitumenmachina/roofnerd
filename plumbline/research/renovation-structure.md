# Renovation structure: joists, rafters, beams, headers, posts (2026-11-21)

Question: what already exists, in this repository and online, so that BuildOS draws, counts and prices
an addition's floor and roof framing and a renovation's beams and headers, and flags what an engineer
must size. BuildOS is not an engineering tool. Everything below is written as research, not as a call;
nothing here is in DECISIONS.md.

## 1. What BuildOS has today (read on the machine)

- `crates/world/src/building/framing.rs` (403 lines) lays out a WALL only: bottom plate, two top plates,
  studs on stations, kings, jacks, header, sill, cripples. A port of BIMCraft `timber.rs::layout`
  (MIT OR Apache-2.0). Also `dressed()` (nominal to dressed sizes, PS 20), `named()` (reads
  "2x6 at 16 in" out of a line's words), `header_depth(span)`.
- `header_depth` is a rule of thumb (2x6 to 4 ft, 2x8 to 6, 2x10 to 8, 2x12 beyond) for a one-storey
  wall. It ignores load, snow, roof span, species and grade. DECISIONS call 71 and 137 and PORTMAP row
  `header_depth` all say so ("the code sizes it from its load, IRC R602.7"). As a price it is Assumed;
  as a structural statement it must not be shown as a size anyone may build to.
- Call 137: counted figures `studs`, `plate_length`, `header_length`, `sill_length` on a wall, Assumed
  (they rest on the spacing a line names). Headers priced "two plies" by LF.
- Call 71 states what is not there: "No joists, no rafters, no corner or partition backing, no
  blocking."
- A roof exists in the model (SPEC section 5.5: a Roof solved from a footprint with per-edge pitch or
  gable, giving RoofPlane, Hip, Ridge, Valley, Eave, Rake lines). That is the natural source of rafter
  run, pitch and length. A Floor class exists (SPEC `pl::class`). Strip footings exist (call 70).
  Sixteen-divisions library has only `"Ridge, steel beam and cap"` lines (Unknown price) and no joist,
  rafter, LVL, girder, post or hanger lines (`grep -i joist|rafter|girder|lvl` in crates/ and library/:
  only `finish.rs` matched, and only as a word). No "bearing wall" or "engineer" flag in SPEC, STATUS,
  QUESTIONS or PROPOSALS (grep found nothing).
- `research/lidar-renovation.md` already covers scanning an existing building as the start of a
  renovation job; not repeated.

## 2. BIMCraft (read at /home/user/ipedrax/bimcraft, one commit, 81ebe7f; PORTMAP cites f202abd)

`crates/core/src/timber.rs` is 177 lines and has exactly one thing: `layout` (a wall layer with one
hosted window: plates, studs, kings, jacks, header, sill, cripples) plus `touching_wall`. Everything
BuildOS has from it is already ported. **It has no joists, rafters, trusses, blocking, posts or beam
sizing.** `grep -rniE "joist|rafter|truss"` over all crates finds only IFC class name lists
(`ifc/classes.rs`: IFCBEAM predefined types include JOIST and LINTEL, IFCMEMBER includes RAFTER, POST,
PURLIN, STUD, PLATE), an import filter (`import.rs:88`) and Revit category names. Elsewhere BIMCraft has
beams as straight runs ("beam" category, `structure_tests.rs`), structural columns, isolated and wall
footings, reinforcement (`reinforce.rs`), and per-member volume/formwork/steel quantities. `loads.rs`
is an electrical load table (NBR 5410), not structural. Licence: MIT OR Apache-2.0 (Cargo.toml;
LICENSE-MIT, LICENSE-APACHE). Useful to BuildOS: the IFC class and predefined-type list (use IFCBEAM
JOIST/LINTEL, IFCMEMBER RAFTER/POST/STUD) when export writes framing members. Nothing else to port.

## 3. Span tables as data: what the terms allow (read at the source, 2026-11-21)

| Source | What it holds | Terms found | What BuildOS may do |
|---|---|---|---|
| AWC copyright page https://awc.org/about/copyright-request/ | AWC's Span Tables for Joists and Rafters (STJR), Span Calculator data (NDS 2018 Supplement values), DCA 6, Wood Frame Construction Manual | "All rights reserved"; reproduction, distribution or other use needs prior written consent and attribution; credit line "Courtesy of the American Wood Council"; permission is per item, does not extend to later editions or derivative works; $10 per table, 5 to 7 business days | Do not copy their tables or ship their calculator's data. A written request per table is cheap and possible (a bundled table in a shipped program is the "derivative work" case they say is not covered: ask in words, do not assume). Citing the table by number on a line is fine (a citation is not a copy) |
| AWC Span Options Calculator https://awc.org/codes-standards/calculators-software/spancalc | Free web and phone app; species and grades from NDS 2018 Supplement; WSU-built | Links to an EULA; no API, no embed mentioned | A person may use it beside the program and type the answer in; the program cannot call it (it never opens a connection) |
| ICC code text, https://codes.iccsafe.org (2021 IRC R502.3.1, R802.4, R602.7 and their tables) | The model code as adopted | The site returned 403 to automated reading. ICC's Terms of Use https://www.iccsafe.org/terms-of-use/ (read) exclude "derivative use", copying to third parties, commercial use and "data mining, robots, or similar" without written consent (license@iccsafe.org); a subscriber may copy up to 10% of a title for internal use | Copying the tables into the program is not clear. The law is unsettled for a model code adopted as law: Georgia v. Public.Resource.Org (2020) turned on whether the author is a legislator or judge, and the ICC reads it to protect its model codes; ICC v. UpCodes (S.D.N.Y. May 2020) was a mixed result, accurate copies of adopted code were not shown to infringe, and I found no final judgment (I could not verify the present state, nor a 2023 ASTM case, which my search did not return). Do not rely on it for a shipped data file |
| A repository under MIT that holds a table, e.g. https://github.com/dust2engineering/ontario-floor-joist-span-selector (Ontario's OBC 2024 Table 9.23.4.2.-A, SPF and Douglas fir-larch, MIT) | A Canadian table | MIT covers the repository's code, not the table's owner's rights | Not usable: wrong code (Canada), and an MIT label does not clear the table |

Where tables are free to take: United States federal publications. The USDA Forest Products Laboratory
Wood Handbook is a government work (not checked for this file: confirm at https://www.fpl.fs.usda.gov
before citing). Its beam formulas and property data are facts anyone may compute with. A span can also
be computed from first principles (bending, shear, deflection at L/360 for floors, L/240 for rafters
without a ceiling) from design values, but the design values (Fb, Fv, E by species and grade) are
published in the NDS Supplement (AWC, copyrighted) and by the grading agencies (SPIB, WWPA, NeLMA:
their own terms, not read). That route puts the program in the business of sizing, which it must not.

**Conclusion for (a).** Do not hold any joist, rafter, header or girder size table in the program.
A size on a line is typed by the person (from the engineer's letter, the plans or the maker's sheet)
and is Known only when the engineer's sheet says so, otherwise Assumed with "typed by the builder, not
checked" as its source. The program may cite "IRC R502.3.1(2)" as where the builder looked.
Manufacturers' own span sheets (LVL, I-joist, truss) are a better data path: the program already
imports a maker's sheet as an item with its sizes Assumed and sourced to the sheet's line.

BuildOS already has `slope_factor`, `plan_area`, `surface_area` and `true_length` for roof planes and
hips, ridges and valleys (`derive.rs:435-480`): the rafter length is the plane's true length along a
run, so no new pitch arithmetic is needed, only a layout of members on a plane.

## 4. Code for joist and rafter layout, span lookup and beam checks (searched, licences read from the repository page)

I found nothing worth porting. All searches were by web search and `gh search repos`; GitHub's API was
closed to this session for named repositories, so licences below were read from each repository's page.

| Project | What | Licence (read) | Use |
|---|---|---|---|
| PyNite https://github.com/JWock82/Pynite | General 3D frame and plate finite elements; steel pushover; no wood design | MIT (page label; LICENSE text not shown) | Reference only. A solver is engineering; it is not wanted |
| anaStruct https://github.com/ritchie46/anaStruct | 2D frames and beams | LGPL-3.0 | Never linked (fixed rule). Reference only |
| dust2engineering/ontario-floor-joist-span-selector | Ontario table lookup | MIT, but the data is the Ontario code's | Not usable (section 3) |
| MytikasIndustries/joist-span-finder, sandipan-x/joistspan | Small span finders | No licence on either | Not usable |
| joistpy (PyPI) | Steel bar joist (SJI) shape database | not read | Wrong material |
| Rafter calculators (SkyCiv, Omni, inchcalculator, app-store apps) | Rafter length from rise and run | Commercial or unlicensed | None. The arithmetic is trigonometry and needs no source |

No Rust library for joists, rafters or lumber beam checks turned up (`gh search repos rafter --language
rust` returned only Raft-consensus projects). I did not search crates.io or npm directly: not verified.
IFC side, already in the repo: IfcOpenShell (LGPL) stays an outside checker only. BIMCraft's IFC class
lists (section 2) give the right class and predefined types for framing.

## 5. How estimators take off floor and roof framing (settled practice)

Sources read: JLC Field Guide: Lumber https://www.jlconline.com/how-to/framing/lumber/lumber-field-guide
(joist count, rim stock, roof sheathing; its pitch-ratio table did not come through in the text I could
read); Hover https://hover.to/blog/mastering-lumber-framing-takeoffs (studs, plates, a 10 to 15 %
waste; it gives no joist, rafter or beam formulas); web search snippets for blocking and hangers.
Items marked (usual practice) are what framers and estimators do and are NOT confirmed at a source I
read; confirm them with the framer on the job before they are coded.

Floor, one run of joists over a rectangle:
- Joists = floor width / spacing + 1 (JLC). Rim (band) joists: building length x 2 / stock length,
  stock lengths chosen as multiples of the spacing (JLC). Doubled joists under a bearing partition,
  and around a stair or chase opening (headers and trimmers) are counted by hand (usual practice).
- Blocking or bridging: solid blocking between joists, one row per span of 8 ft or over was the
  figure one source gave; priced per piece with the spacing (a labor reference). Hangers: one per
  joist end that frames into a header or beam (usual practice; no formula found).
- Subfloor: area / 32 SF per 4 x 8 sheet, plus a couple of sheets for jogs (JLC). Waste on lumber
  10 to 15 % (Hover). BuildOS already carries a waste on a counted line (call 137).
- The count is a count of pieces, and a piece has a stock length: a 14 ft span takes 14 ft or 16 ft
  stock (usual practice). BuildOS's counted lines already buy in whole pieces.

Roof:
- Common rafter length = horizontal run x line-length ratio for the pitch (JLC: "multiply the correct
  line length ratio ... by the horizontal run"), then add the ridge plumb cut (half the ridge
  thickness back) and the overhang. Count = ridge length / spacing + 1, per side, plus a pair at each
  gable end (usual practice). A hip or valley uses the hip/valley ratio; a pair of jack rafters adds
  up to one common (JLC). Roof sheathing = perimeter incl. overhangs x rafter line length / 32 (JLC).
- Ridge board (IRC R802.3: a board, not a beam, unless ties do not run across; then a ridge beam on
  walls or girders); hips and valleys that carry rafters "shall be designed as beams" (Seattle 2021
  SRC ch. 8, the IRC text as amended). Collar ties in the upper third, 48 in or closer (a Jaspector
  summary; the section number differs by edition: R802.3 vs R802.3.1; verify against the adopted
  edition). Rafter ties (ceiling joists) in the lower third.
- Trusses: priced as a delivered, engineered piece by the EA, with the truss maker's own drawing. A
  truss's size is never BuildOS's to give.

Beams, posts, footings (renovation and additions):
- LVL, PSL, glulam and steel beams by the LF in the maker's stock lengths, plies as a multiplier
  ("double LVL" is two lines of LVL by LF, as BuildOS already prices "Headers, two plies"). Posts by
  the EA (built-up 2x or solid), a footing under each post by the EA or by volume (BuildOS has strip
  footings, no pad footing: STATUS lists "pad footings" as a lack). Hangers and post caps and bases by
  the EA. Temporary shoring is a labor line (usual practice).

## 6. A renovation: what a builder is told to flag (in a builder's words)

Sources: JLC, "Opening Up a Bearing Wall" https://www.jlconline.com/Training-the-Trades/opening-up-a-bearing-wall_o/
and "Opening Up Space" https://www.jlconline.com/how-to/framing/opening-up-space_o (both read in full;
trade magazine, advice not code). The permit step is from the second and from a city filing found by
search (not read in full). I found no authoritative homeowner guide and no jurisdiction checklist.

Is it bearing? Joists lying on the wall's plates are "a good indicator"; a solid header over the opening
too. A wall with joists or rafters resting on it, or with a post, beam or wall above it, is treated as
bearing until an engineer says it is not. (Wall running with the joists is usually not carrying them:
usual practice, not sourced here.)

What to flag on each such wall or opening, in these words:
1. "Bearing wall: get the beam sized by an engineer before you cut." The program draws and counts a beam
   that is typed in; it never sizes one.
2. "Where does it land?" Each end of a new beam sits on a post, the post on solid blocking through the
   floor and on a footing or existing pier or grade beam. No direct line to the foundation means a new
   footing, which an engineer specifies (thickness and steel); a transfer beam where a post sits over
   a basement feature (JLC).
3. "Needs an engineer" triggers, per JLC: span over 12 ft, any beam longer than 14 ft, a flush
   (depth-restricted) beam of multiple LVL, flitch or steel, a point load such as a beam bearing on the
   old wall, an unclear load path, an exterior wall, holes through a joist beyond a wiring hole, and
   any permit or owner requirement.
4. "Shore it first": temporary walls on both sides of the wall to be cut, and below it and stacked
   through floors, left up until every fastener is driven (JLC). Shoring that has to bridge a
   basement obstacle: engineer.
5. "Permit": the building department may ask for the engineer's stamped plan before work, or a letter
   signing off after; local rules govern.
6. "Fasteners as scheduled": the engineer's nailing or screw pattern is part of the beam (JLC: 4 in
   from ends, staggered at 24 in), and plies are not glued (LVL).
7. An opening widened in a bearing wall puts new point loads on the jambs: confirm the new jamb sits
   over joists that run to the girder below (JLC).

Every one of these is a flag on an element, never a computed pass or fail, and is shown "Unknown:
engineer to size" until the person types the engineer's size and marks its source.

## 7. Recommendation, ranked

1. **Rafters and joists as counted members laid on what the model already has.** Biggest gain this month
   for an addition. Adopt: nothing external; arithmetic is public (count = length / spacing + 1, rim =
   perimeter x 2 / stock, true length = run x slope factor already in `derive.rs`). Write in BuildOS: a
   `joists` layout (floor rectangle or polygon, spacing, span direction, size typed, rim, doubled at
   openings, blocking rows, hangers at headers) and a `rafters` layout (from `RoofPlane`: common
   rafters at spacing, ridge, hip and valley and jacks, overhang, collar ties), each emitting EA and LF
   figures like call 137's `studs`, Assumed, sourced to the spacing and the typed size. Draw them with
   the Framing switch. Stays a flag: the SIZE. Could go wrong: plane edges that are not rectangles;
   double counting the plane's surface with the rafters; jacks and hips are where hand estimators
   disagree (JLC gives only a ratio), so check against a framer's hand count on a made-up gable and
   hip.
2. **Beams, headers and posts typed, priced and flagged.** A beam element (LVL/steel/glulam, plies, LF,
   bearing ends) with a post and footing line at each end; the existing `header_depth` rule of thumb
   is relabelled "drawn size, not an engineer's" and refuses to price an opening over 6 ft without a
   typed size. Adopt: IFC IFCBEAM (JOIST, LINTEL) and IFCMEMBER (RAFTER, POST) from BIMCraft's class
   list (MIT OR Apache-2.0). Written in BuildOS: the beam element, the flag texts of section 6, pad footing.
   Stays a flag: every size and every load path. Wrong: a flag read as approval; keep Unknown until
   typed and say the source.
3. **A renovation flag layer: bearing or not, shoring, engineer, permit.** Cheap, high value for the
   owner's real job: a per-wall "bearing: yes/no/Unknown" (Assumed from joist direction in the model),
   on a removed wall or widened opening a VIF-style list entry ("engineer to size, shoring needed,
   footing below?") with dollars for shoring and beam. Adopt: JLC's trigger list as the wording
   (cited, paraphrased, not copied). Wrong: being a checklist that implies sufficiency.
4. **Maker's sheets for engineered members** (LVL, I-joist, truss): import through the existing sheet
   path, sizes Assumed and sourced to the sheet's line. This is the legal route to real span data
   (the maker's own, published for the trade): read each maker's terms before bundling; unverified.
5. **Span tables: do not embed.** Ask AWC in writing (about $10 a table, 5 to 7 business days, per
   edition) only if a "typed size is under the table's span" hint is wanted later; that would be a
   hint on an Assumed value and still never a pass. Not this month.

## 8. What I could not verify

- ICC's own page and IRC 2021 text: 403 to my reader, so R502.3.1, R602.7 and R802 were taken from
  secondary copies (Seattle code, Jaspector, a JLC article) and not checked in the ICC edition.
- The present state of ICC v. UpCodes and any later model-code copyright ruling.
- The USDA Wood Handbook's public-domain status, and the terms of SPIB, WWPA, NeLMA and the
  manufacturers' (Weyerhaeuser, Boise Cascade, LP) span sheets and software.
- Rafter line-length ratios and hip/valley ratio values (the JLC page omitted its table; they are
  trigonometry and can be derived and checked by hand).
- Hanger and blocking quantity rules, and whether pad footing and post lines belong to BuildOS's
  existing divisions (3 Concrete, 6 Wood).
- crates.io and npm were not searched. GitHub licences were read from pages, not from the licence files.
