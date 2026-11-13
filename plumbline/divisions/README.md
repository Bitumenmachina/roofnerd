# The sixteen divisions: what the program has for each

For whoever builds next, and for a harness given one division (DECISIONS.md calls 75 and
76). One place to start from for each trade: what the made-up whole house prices in it
(`plumbline sixteen`), what is drawn in the model for it, and what the program knows of
it and where that comes from. Every figure and name here is made up; an estimator's own
prices and knowledge live on his machine and never in this repository.

The divisions are MasterFormat 1995's sixteen (DECISIONS.md call 57). Nothing can yet
take one division out of a job and put it back: that is PROPOSALS.md 21, not built.

## Division 1. General Requirements

**Priced on the made-up whole house:** 4 lines.

- Superintendent, days on site, by the hr
- Building permit, by the EA
- Temporary power and toilets, by the EA
- Final cleaning, by the EA

**In the model:** Nothing: these are lines on the job.

## Division 2. Site Construction

**Priced on the made-up whole house:** 3 lines.

- Site clearing and grading, by the EA
- Driveway and walks, by the EA
- Excavation and gravel base, by the SF

**In the model:** The ground (a plain plane with a switch). No grading, paving or utilities are drawn.

## Division 3. Concrete

**Priced on the made-up whole house:** 9 lines.

- Concrete slab on grade, 4 in, by the SF
- Concrete footing, in place, by the CY (4 lines)
- Foundation wall, 8 in concrete, formed and placed, by the SF (4 lines)

**In the model:** Strip footings (a kind of their own, `Kind::Footing`) and foundation walls as walls on a level under the first floor; the slab is the first floor's floor. A wall priced as concrete is drawn as concrete. Ground switch under Show.

**What the program knows, and where from:** Footing concrete is length x width x depth / 27 CY (call 70). **Open from judgment 20:** at a T, where one footing ends on the middle of another, that overcounts by half a width x width x depth (J20-1); quantities are rounded to four places before pricing (J20-3). No rebar, forms, pad footings or stepped footings.

## Division 4. Masonry

**Priced on the made-up whole house:** 4 lines.

- Stone veneer wainscot, 3 ft, by the LF (4 lines)

**In the model:** Brick, stone and stucco as a wall's cladding or wainscot, read from what is priced on the wall (`finish.rs`, `dress.rs`).

## Division 5. Metals

**Priced on the made-up whole house:** 1 line.

- Steel ridge beam, by the LF

**In the model:** Nothing of its own: the ridge beam is a line on the ridge.

## Division 6. Wood and Plastics

**Priced on the made-up whole house:** 41 lines.

- Wall framing, 2x6 at 16 in, by the SF (8 lines)
- Partition framing, 2x4 at 16 in, by the SF (8 lines)
- Casing and trim, by the EA (9 lines)
- Base and trim, by the LF (10 lines)
- Cabinets and countertops, by the EA
- Floor framing and subfloor, by the SF
- Roof sheathing, by the SF (4 lines)

**In the model:** Framing: each wall with a framing line as its frame (plates, studs, kings, jacks, headers, sills, cripples), behind the Framing switch. Interior woodwork: base along the walls of each room with base priced, cut at doors. Exterior casing and corner boards with the finishes.

**What the program knows, and where from:** Lumber's dressed sizes are the American Softwood Lumber Standard's (a 2x6 is 1 1/2 x 5 1/2 in). The frame's layout is BIMCraft's `timber.rs` and the IRC's R602.3 and R602.7. Two rules of thumb, said where used: a header's depth by its span (2x6 to 4 ft, 2x8 to 6 ft, 2x10 to 8 ft, 2x12 beyond), and the bottom plate laid whole under a door. Framing is priced by the SF of wall, not by the stud. Base is 5 1/4 in by 5/8 in as drawn, a drawing size; priced by the LF of room perimeter.

## Division 7. Thermal and Moisture Protection

**Priced on the made-up whole house:** 54 lines.

- Sheathing and weather barrier, by the SF (8 lines)
- Batt insulation, by the SF (8 lines)
- Cedar lap siding, installed, by the SF (8 lines)
- Flashing and sealant, by the EA (17 lines)
- Standing seam roofing, by the SF (4 lines)
- Gutter, by the LF (4 lines)
- Hip cap, by the LF (4 lines)
- Ridge cap and vent, by the LF

**In the model:** Siding and roofing as finishes; gutters where priced; with the Finishes switch off, the weather barrier on each wall that has one priced and flashing round each opening that has it priced.

**What the program knows, and where from:** A weather barrier is read by the trade's words for it (weather barrier, house wrap, building paper, WRB, air barrier). **Judgment 20 expected, not yet verified:** "Tyvek" and "felt paper" are not read. Flashing is drawn 4 in wide, a drawing size; priced by the opening.

## Division 8. Doors and Windows

**Priced on the made-up whole house:** 35 lines.

- Entry door, mahogany, six panel, prehung, by the EA
- Door hardware, by the EA (9 lines)
- Window unit, clad wood, double-hung, divided lites, bronze, by the EA (17 lines)
- Interior door, six panel, prehung, by the EA (8 lines)

**In the model:** Windows with frame, sashes, lites and glass; doors with frame, leaf and knob; what each is read from its priced line.

## Division 9. Finishes

**Priced on the made-up whole house:** 52 lines.

- Gypsum board, hung and finished, by the SF (8 lines)
- Paint, two coats, by the SF (8 lines)
- Gypsum board, both sides, by the SF (8 lines)
- Paint, both sides, by the SF (8 lines)
- Hardwood flooring, by the SF (8 lines)
- Ceiling, gypsum board and paint, by the SF (10 lines)
- Tile flooring, by the SF (2 lines)

**In the model:** Nothing of its own yet (drywall, paint and flooring are lines by the SF of wall or room).

## Division 10. Specialties

**Priced on the made-up whole house:** 3 lines.

- Toilet and bath accessories, by the EA (2 lines)
- Closet shelving, by the EA

**In the model:** Nothing.

## Division 11. Equipment

**Priced on the made-up whole house:** 2 lines.

- Kitchen appliances, by the EA
- Laundry appliances, by the EA

**In the model:** Placed things: a piece of equipment as a box of its size, or as one of ten forms where its name says.

## Division 12. Furnishings

**Priced on the made-up whole house:** 17 lines.

- Window treatment, by the EA (17 lines)

**In the model:** Placed things made from makers' sheets (items): vanity, cabinet, table, chair, bed, sofa, desk, wardrobe.

## Division 13. Special Construction

**Priced on the made-up whole house:** 1 line.

- Sauna, prefabricated, by the EA

**In the model:** Nothing.

## Division 14. Conveying Systems

**Priced on the made-up whole house:** 1 line.

- Residential elevator, two stops, by the EA

**In the model:** Nothing.

## Division 15. Mechanical

**Priced on the made-up whole house:** 148 lines.

- Set and connected, by the EA (21 lines)
- Ductwork, installed, by the LF (23 lines)
- Fitting, installed, by the EA (41 lines)
- Piping, installed, by the LF (63 lines)

**In the model:** Ducts, pipes, fittings, equipment and plumbing fixtures, by system, each system with a switch; clashes against structure and each other.

**What the program knows, and where from:** Fittings are made where runs meet; a run through a wall or floor is listed as a hole to cut, not a clash.

## Division 16. Electrical

**Priced on the made-up whole house:** 24 lines.

- Set and connected, by the EA (6 lines)
- Conduit, installed, by the LF (2 lines)
- Cable, pulled, by the LF (16 lines)

**In the model:** Conduit, cable, panel, meter, receptacles and lights, by system.

