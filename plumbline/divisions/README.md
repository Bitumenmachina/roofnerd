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

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Building permit", in the assembly "General conditions" (goes on a job): general conditions, by `1`
- "Temporary power and toilets", in the assembly "General conditions" (goes on a job): general conditions, by `1`
- "Dumpsters", in the assembly "General conditions" (goes on a job): general conditions, by `1`
- "Final cleaning", in the assembly "General conditions" (goes on a job): general conditions, by `1`

## Division 2. Site Construction

**Priced on the made-up whole house:** 3 lines.

- Site clearing and grading, by the EA
- Driveway and walks, by the EA
- Excavation and gravel base, by the SF

**In the model:** The ground (a plain plane with a switch). No grading, paving or utilities are drawn.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Excavation and grading", in the assembly "Slab on grade, 4 in" (goes on a floor): subcontract, by `plan_area`
- "Gravel base, 4 in", in the assembly "Slab on grade, 4 in" (goes on a floor): material, by `plan_area`

## Division 3. Concrete

**Priced on the made-up whole house:** 9 lines.

- Concrete slab on grade, 4 in, by the SF
- Concrete footing, in place, by the CY (4 lines)
- Foundation wall, 8 in concrete, formed and placed, by the SF (4 lines)

**In the model:** Strip footings (a kind of their own, `Kind::Footing`) and foundation walls as walls on a level under the first floor; the slab is the first floor's floor. A wall priced as concrete is drawn as concrete. Ground switch under Show.

**What the program knows, and where from:** Footing concrete is length x width x depth / 27 CY (call 70). **Open from judgment 20:** at a T, where one footing ends on the middle of another, that overcounts by half a width x width x depth (J20-1); quantities are rounded to four places before pricing (J20-3). No rebar, forms, pad footings or stepped footings.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Vapor barrier under the slab", in the assembly "Slab on grade, 4 in" (goes on a floor): material, by `plan_area`
- "Welded wire reinforcing", in the assembly "Slab on grade, 4 in" (goes on a floor): material, by `plan_area`
- "Concrete, 4 in slab", in the assembly "Slab on grade, 4 in" (goes on a floor): material, by `plan_area`
- "Place and finish concrete", in the assembly "Slab on grade, 4 in" (goes on a floor): subcontract, by `plan_area`

## Division 4. Masonry

**Priced on the made-up whole house:** 4 lines.

- Stone veneer wainscot, 3 ft, by the LF (4 lines)

**In the model:** Brick, stone and stucco as a wall's cladding or wainscot, read from what is priced on the wall (`finish.rs`, `dress.rs`).

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Stone veneer wainscot, 3 ft", in the assembly "Exterior wall, 2x6, lap siding over a stone wainscot" (goes on a wall): subcontract, by `plan_length`
- "Brick veneer", in the assembly "Exterior wall, 2x6, brick veneer" (goes on a wall): subcontract, by `net_area`
- "Brick ties and flashing", in the assembly "Exterior wall, 2x6, brick veneer" (goes on a wall): material, by `net_area`

## Division 5. Metals

**Priced on the made-up whole house:** 1 line.

- Steel ridge beam, by the LF

**In the model:** Nothing of its own: the ridge beam is a line on the ridge.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Steel ridge beam", in the assembly "Ridge, steel beam and cap" (goes on a ridge): material, by `true_length`

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

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Floor framing", in the assembly "Framed floor" (goes on a floor): material, by `plan_area`
- "Subfloor", in the assembly "Framed floor" (goes on a floor): material, by `plan_area`
- "Wall framing, 2x6 at 16 in", in the assembly "Exterior wall, 2x6, lap siding" (goes on a wall): material, by `face_area`
- "Wall sheathing", in the assembly "Exterior wall, 2x6, lap siding" (goes on a wall): material, by `face_area`
- "Wall framing, 2x6 at 16 in", in the assembly "Exterior wall, 2x6, lap siding over a stone wainscot" (goes on a wall): material, by `face_area`
- "Wall sheathing", in the assembly "Exterior wall, 2x6, lap siding over a stone wainscot" (goes on a wall): material, by `face_area`
- "Wall framing, 2x6 at 16 in", in the assembly "Exterior wall, 2x6, brick veneer" (goes on a wall): material, by `face_area`
- "Wall sheathing", in the assembly "Exterior wall, 2x6, brick veneer" (goes on a wall): material, by `face_area`
- "Wall framing, 2x6 at 16 in", in the assembly "Exterior wall, 2x6, stucco" (goes on a wall): material, by `face_area`
- "Wall sheathing", in the assembly "Exterior wall, 2x6, stucco" (goes on a wall): material, by `face_area`
- "Partition framing, 2x4 at 16 in", in the assembly "Interior partition, 2x4" (goes on a wall): material, by `face_area`
- "Base and trim", in the assembly "Room, hardwood floor" (goes on a room): material, by `perimeter`
- "Base and trim", in the assembly "Bath, tile floor" (goes on a room): material, by `perimeter`
- "Base and trim", in the assembly "Kitchen" (goes on a room): material, by `perimeter`
- "Base and trim", in the assembly "Laundry or utility room" (goes on a room): material, by `perimeter`
- "Roof sheathing", in the assembly "Asphalt shingle roof" (goes on a roofplane): material, by `surface_area`
- "Roof sheathing", in the assembly "Standing seam roof" (goes on a roofplane): material, by `surface_area`
- "Casing and trim at the window", in the assembly "Window" (goes on a window): material, by `1`
- "Casing and trim at the door", in the assembly "Entry door" (goes on a door): material, by `1`
- "Casing and trim at the door", in the assembly "Interior door" (goes on a door): material, by `1`

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

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Weather barrier", in the assembly "Exterior wall, 2x6, lap siding" (goes on a wall): material, by `net_area`
- "Batt insulation", in the assembly "Exterior wall, 2x6, lap siding" (goes on a wall): material, by `net_area`
- "Lap siding", in the assembly "Exterior wall, 2x6, lap siding" (goes on a wall): material, by `net_area`
- "Weather barrier", in the assembly "Exterior wall, 2x6, lap siding over a stone wainscot" (goes on a wall): material, by `net_area`
- "Batt insulation", in the assembly "Exterior wall, 2x6, lap siding over a stone wainscot" (goes on a wall): material, by `net_area`
- "Lap siding", in the assembly "Exterior wall, 2x6, lap siding over a stone wainscot" (goes on a wall): material, by `net_area`
- "Weather barrier", in the assembly "Exterior wall, 2x6, brick veneer" (goes on a wall): material, by `net_area`
- "Batt insulation", in the assembly "Exterior wall, 2x6, brick veneer" (goes on a wall): material, by `net_area`
- "Weather barrier", in the assembly "Exterior wall, 2x6, stucco" (goes on a wall): material, by `net_area`
- "Batt insulation", in the assembly "Exterior wall, 2x6, stucco" (goes on a wall): material, by `net_area`
- "Underlayment", in the assembly "Asphalt shingle roof" (goes on a roofplane): material, by `surface_area`
- "Asphalt shingles", in the assembly "Asphalt shingle roof" (goes on a roofplane): material, by `surface_area`
- "Underlayment", in the assembly "Standing seam roof" (goes on a roofplane): material, by `surface_area`
- "Standing seam roofing", in the assembly "Standing seam roof" (goes on a roofplane): subcontract, by `surface_area`
- "Ridge cap and vent", in the assembly "Ridge, steel beam and cap" (goes on a ridge): material, by `true_length`
- "Hip cap", in the assembly "Hip cap" (goes on a hip): material, by `true_length`
- "Gutter", in the assembly "Gutter" (goes on a eave): material, by `true_length`
- "Flashing and sealant at the window", in the assembly "Window" (goes on a window): material, by `1`

## Division 8. Doors and Windows

**Priced on the made-up whole house:** 35 lines.

- Entry door, mahogany, six panel, prehung, by the EA
- Door hardware, by the EA (9 lines)
- Window unit, clad wood, double-hung, divided lites, bronze, by the EA (17 lines)
- Interior door, six panel, prehung, by the EA (8 lines)

**In the model:** Windows with frame, sashes, lites and glass; doors with frame, leaf and knob; what each is read from its priced line.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Window unit", in the assembly "Window" (goes on a window): material, by `1`
- "Entry door, prehung", in the assembly "Entry door" (goes on a door): material, by `1`
- "Door hardware", in the assembly "Entry door" (goes on a door): material, by `1`
- "Interior door, prehung", in the assembly "Interior door" (goes on a door): material, by `1`
- "Door hardware", in the assembly "Interior door" (goes on a door): material, by `1`

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

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Gypsum board, hung and finished", in the assembly "Exterior wall, 2x6, lap siding" (goes on a wall): material, by `net_area`
- "Paint, two coats", in the assembly "Exterior wall, 2x6, lap siding" (goes on a wall): material, by `net_area`
- "Gypsum board, hung and finished", in the assembly "Exterior wall, 2x6, lap siding over a stone wainscot" (goes on a wall): material, by `net_area`
- "Paint, two coats", in the assembly "Exterior wall, 2x6, lap siding over a stone wainscot" (goes on a wall): material, by `net_area`
- "Gypsum board, hung and finished", in the assembly "Exterior wall, 2x6, brick veneer" (goes on a wall): material, by `net_area`
- "Paint, two coats", in the assembly "Exterior wall, 2x6, brick veneer" (goes on a wall): material, by `net_area`
- "Stucco, three coat", in the assembly "Exterior wall, 2x6, stucco" (goes on a wall): subcontract, by `net_area`
- "Gypsum board, hung and finished", in the assembly "Exterior wall, 2x6, stucco" (goes on a wall): material, by `net_area`
- "Paint, two coats", in the assembly "Exterior wall, 2x6, stucco" (goes on a wall): material, by `net_area`
- "Gypsum board, both sides", in the assembly "Interior partition, 2x4" (goes on a wall): material, by `net_area`
- "Paint, both sides", in the assembly "Interior partition, 2x4" (goes on a wall): material, by `net_area`
- "Hardwood flooring", in the assembly "Room, hardwood floor" (goes on a room): material, by `floor_area`
- "Ceiling, gypsum board and paint", in the assembly "Room, hardwood floor" (goes on a room): material, by `floor_area`
- "Tile flooring", in the assembly "Bath, tile floor" (goes on a room): material, by `floor_area`
- "Ceiling, gypsum board and paint", in the assembly "Bath, tile floor" (goes on a room): material, by `floor_area`
- "Hardwood flooring", in the assembly "Kitchen" (goes on a room): material, by `floor_area`
- "Ceiling, gypsum board and paint", in the assembly "Kitchen" (goes on a room): material, by `floor_area`
- "Tile flooring", in the assembly "Laundry or utility room" (goes on a room): material, by `floor_area`
- "Ceiling, gypsum board and paint", in the assembly "Laundry or utility room" (goes on a room): material, by `floor_area`

## Division 10. Specialties

**Priced on the made-up whole house:** 3 lines.

- Toilet and bath accessories, by the EA (2 lines)
- Closet shelving, by the EA

**In the model:** Nothing.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Toilet and bath accessories", in the assembly "Bath, tile floor" (goes on a room): material, by `1`
- "Closet shelving", in the assembly "Laundry or utility room" (goes on a room): material, by `1`

## Division 11. Equipment

**Priced on the made-up whole house:** 2 lines.

- Kitchen appliances, by the EA
- Laundry appliances, by the EA

**In the model:** Placed things: a piece of equipment as a box of its size, or as one of ten forms where its name says.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Kitchen appliances", in the assembly "Kitchen" (goes on a room): material, by `1`
- "Laundry appliances", in the assembly "Laundry or utility room" (goes on a room): material, by `1`

## Division 12. Furnishings

**Priced on the made-up whole house:** 17 lines.

- Window treatment, by the EA (17 lines)

**In the model:** Placed things made from makers' sheets (items): vanity, cabinet, table, chair, bed, sofa, desk, wardrobe.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Cabinets", in the assembly "Kitchen" (goes on a room): material, by `1`
- "Countertops", in the assembly "Kitchen" (goes on a room): material, by `1`
- "Window treatment", in the assembly "Window" (goes on a window): material, by `1`

## Division 13. Special Construction

**Priced on the made-up whole house:** 1 line.

- Sauna, prefabricated, by the EA

**In the model:** Nothing.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Prefabricated room, set in place", in the assembly "Sauna or other built room" (goes on a room): subcontract, by `1`

## Division 14. Conveying Systems

**Priced on the made-up whole house:** 1 line.

- Residential elevator, two stops, by the EA

**In the model:** Nothing.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Residential elevator, installed", in the assembly "Residential elevator" (goes on a equipment): subcontract, by `1`

## Division 15. Mechanical

**Priced on the made-up whole house:** 148 lines.

- Set and connected, by the EA (21 lines)
- Ductwork, installed, by the LF (23 lines)
- Fitting, installed, by the EA (41 lines)
- Piping, installed, by the LF (63 lines)

**In the model:** Ducts, pipes, fittings, equipment and plumbing fixtures, by system, each system with a switch; clashes against structure and each other.

**What the program knows, and where from:** Fittings are made where runs meet; a run through a wall or floor is listed as a hole to cut, not a clash.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Ductwork, installed", in the assembly "Duct run" (goes on a duct): subcontract, by `true_length`
- "Piping, installed", in the assembly "Pipe run" (goes on a pipe): subcontract, by `true_length`
- "Fixture, set and connected", in the assembly "Plumbing fixture" (goes on a fixture): subcontract, by `1`
- "Equipment, set and connected", in the assembly "Mechanical equipment" (goes on a equipment): subcontract, by `1`

## Division 16. Electrical

**Priced on the made-up whole house:** 24 lines.

- Set and connected, by the EA (6 lines)
- Conduit, installed, by the LF (2 lines)
- Cable, pulled, by the LF (16 lines)

**In the model:** Conduit, cable, panel, meter, receptacles and lights, by system.

**In the starter assemblies the program ships with** (no prices in them; `plumbline import assemblies <job> starter`):

- "Conduit, installed", in the assembly "Conduit run" (goes on a conduit): subcontract, by `true_length`
- "Cable, pulled", in the assembly "Cable run" (goes on a cable): subcontract, by `true_length`
- "Device, set and connected", in the assembly "Electrical device" (goes on a fixture): subcontract, by `1`
- "Electrical equipment, set and connected", in the assembly "Electrical equipment" (goes on a equipment): subcontract, by `1`
