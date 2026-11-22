# Builder (GC / estimator / PM / super in one) — sweep notes

Looked at: pictures and the command line only (frozen `$B`, made-up house in `builder/h.json`). The live window was not run.
Pictures read (30): trades/1-7 + first-floor-alone; clash/04 model+sheet, 07; renow/05, 08, 10 sheet; field/05, 07 sheet; money/03 sheet; price/07 sheet; scanw/06; cabinetsw/08; layoutsw/05 sheet; roofw/03; foundw/04; house/06; layers/03; glass/07; d2/03; first/05; plan/06.
Command line: sixteen, estimate, takeoff, vif, clashes, finishes, check, export subcontractor / pm / estimate.

## 1. Who I am and why I open this
- **Walk it before I build it**: stand in a room, see what is behind the drywall, where the plumber's stack goes, where the duct has to drop.
- **Find what the drawings hid**: what hits what (stack vs. duct, tub vs. wall, wire vs. waste) before the framers and subs are on site, not after.
- **Price it and defend it**: every dollar traced to a quantity, a source and how sure it is; what is left out said plainly.
- **Sequence and hand out scope**: who goes first, what each sub is quoting, what I check on site before I sign.

## 2. What I saw
- `trades/3-electrical.png`, `4-plumbing.png`, `5-hvac.png` (See through on, one trade on): readable as a house first: two storeys of clear glass walls, windows with muntins, a door. My trade's runs show in colour: orange wire, blue/red/brown water and waste, blue duct with a red return riser. Plumbing reads best (fixtures are shapes, hot/cold/waste coloured). Cannot read from the picture: sizes, which run is which, what it crosses. No labels on runs in these pictures; the sizes are only in the right panel when a thing is clicked (`field/05`, `field/07`: "pipe 1, 14.00 LF, Type 1 in, Cold water", "duct 1, 12 x 8, surface 66.67 SF").
- `trades/2-framing.png`, `house/06`: studs, plates, openings with headers, hip roof on top. Good: this is a house I can read. I see wall framing only. No floor joists, no rafters, no ceiling joists, no beams in any picture (the model says so itself in CLAUDE.md). The roof sits on the studs like a lid.
- `trades/1-foundation.png`: strip footing and a foundation wall with the slab level shown. Fine. No excavation, no pad footings, no slab thickness visible.
- `trades/7-waterproofing.png`: a very good picture of a window with sash, grids, trim, flashing and standing-seam. This is the one I would show a client.
- `trades/6-interior-woodwork.png` (Walk): a room with base, a window, sun patch on the floor. Nice for a walk-through; the only cabinet picture is a plan (`cabinetsw/08`).
- `first-floor-alone.png`: stone wainscot and lap siding, door. Reads as a real house.
- `clash/04-sheet.png`: "In the same space 2: duct 1 and Air handler 12.00 in; duct 1 and pipe 1 3.50 in", with where (E, N, up), and a "Not tested" list saying what is not looked at (wall vs wall, door holes, roof). I like the honesty. `clash/07-model.png`: the clash is an orange cross at the spot in 3D.
- `renow/05, 08, 10`: renovation marking (Existing pale, Demolition red); a message in words: "Wall 1 carries 1 cost line, and what stands is not bought again. Say which: Take its lines off, or Keep its lines priced, as work done to it (paint, a patch, taking it down)." That is exactly the renovation question. Good.
- `money/03`, `price/07`: recap Material/Labor/…/Overhead/Profit/Price, by division, "Nothing is priced on 15 things that could be". Reads like my sheet. "no figure" in red instead of zero is right.
- `field/07`, VIF: sorted by dollars with "riding".
- `layoutsw/05`: Layouts side by side (two options). Useful for owner choices.
- `plan/06`, `d2/03`, `roofw/03`, `foundw/04`, `scanw/06`: tracing on a public plan, roof planes with SF/pitch/eave-hip-ridge LF, footing with CY (2.06 CY). Real takeoff words. `scanw/06` shows a bare rectangle for a scan: I could not see the scan's real content.
- Words not mine: "Job / Details / Tools" switches, "Frame F", "Existing, shown pale", "Known, Assumed, Unknown", "Next steps", "Bring in", "Write out", "Designer's model", "BIM model", "Item library". "Rooms, inside their walls" is fine. "Clashes" is OK for a super. Hard to read at normal size: most small grey type.
- Not seen: any picture of a section/cut through a wall or floor, any picture of the demolition/existing-vs-new in 3D (renow is all plan), any schedule/sequence view.

## 3. Missing or wrong for me
- No joists, rafters, beams, headers sized, ceiling framing: the thing I fight with subs over (a duct drop through a floor system; a stack through a joist bay). `clashes` lists "floor and soil stack" etc. 71 "through a wall or floor (a hole to cut)", which is noise without joists: the real question (is the stack in a joist, a bearing wall, a header?) cannot be asked.
- Clash list is long and mixes fixtures-in-walls (tub against partition by 0.45 in, toilet vs wet wall 1.80 in) with real conflicts. A toilet "clashing" with its own wet wall is not a clash. I want trade-vs-trade filters (plumb vs HVAC vs structure) and "hole to cut" vs "conflict" in separate lists with the hole's size and what it cuts (bearing? fire-rated?).
- Clash of a duct against a *hall partition at 8.4 ft* is shown as a hole to cut; I need to know if there is a header or a top plate there. Not available.
- No schedule, sequence or phases anywhere (estimate has Supervision 480 hr as a flat line). I cannot see "demo, rough framing, rough MEP, inspection, insulation, board" or what needs to be done before something is covered. PM export is just lines + hours.
- Inspections, permits, hold points: only a "Building permit" lump line. VIF is only about measure/confirm values; no "inspect before cover" list (rough-in inspections), no "what I check on site before I sign" besides assumptions.
- Labor: sample estimate shows material only for framing, siding, drywall (Labor column blank except super). The sub/PM exports carry "Material" for framing. A GC prices labor and material for his own crew; here it seems labor is optional per line (`labor on|off`). I could not see labor hours per trade.
- Renovation: Existing / Demolish / New marks exist (renow pictures), but no pictures of the 3D with demo in red, no protection, dust walls, shoring, disposal quantities, hidden conditions (lead, asbestos, knob-and-tube) as flagged unknowns. Division 2 demolition quantity: not seen.
- Allowance line exists but not as a visible "allowance vs. selection" tracker for the owner.
- Scan import shows just a rectangle (scanw/06): I could not verify a LiDAR scan gives me out-of-plumb walls, sloped floors, actual joist depths: crucial in old houses. Not seen.
- "Assumed" numbers: 255 unknown in the sample. Fine, but the top bar number "$468,876.85" is shown as one big figure with "$17,060.62 riding" in small. Good that it says it; I would want the range (low/high) on the total.
- Small type, dim labels on frosted glass; right panel text is dense. On a laptop on a tailgate it will be hard.
- The sixteen-division sample includes prices only as made-up; no way seen to compare with my own cost history besides "Prior estimate" import.
- Words: "Clashes 21" on the Show list as a layer is fine; "Known/Assumed/Unknown" as a layer is odd but explained.

## 4. What the best tools show today
- Navisworks: Clash Detective (Manage tier) audits/reports interferences with issue integration; TimeLiner links model objects to a schedule for a 4D simulation, costs on tasks, and can combine with clash for time-based checks. Source: https://www.autodesk.com/products/navisworks/overview and https://help.autodesk.com/cloudhelp/2023/ENU/Navisworks-Timeliner/files/GUID-D0D36E3D-F1D0-43B6-AB4E-2E7799B340A3.htm (via search summary).
- Bluebeam Revu: length/area/volume/count, cutouts for net quantities, scale per viewport, Snap to Content on vector, custom columns (unit cost, waste), Quantity Link to Excel. Sources: https://blog.bluebeam.com/revu-estimation-quantity-takeoffs/ and https://www.bluebeam.com/solutions/takeoffs.
- (Fetches of two vendor pages were refused with 403/404; I relied on the search summaries and did not check other products' pages.)
- What none seem to do (as far as these pages say) and this program could: one free model where the estimate, the clash and the VIF dollars are the same object (a clash "has $2,241 riding"); renovation marks that change the price ("take its lines off / keep priced"); "no figure" instead of zero; the sub scope export without my prices (`export subcontractor` works: 190 rows, no prices). Ahead would be: a schedule driven by the same marks, and a clash that names the member it cuts.

## 5. My ten
1. **Floor and roof framing** (joists, rafters, beams, headers) drawn and priced; a hole through them is named "cuts joist 3 of 14 / in header". Done when a stack through the floor in the sample reports the joist bay and the hole size, and the framing picture shows joists with LF and the estimate has the line.
2. **Clash list in builder's categories**: "Conflict" (duct vs pipe), "Hole to cut" (through bearing wall / joist / fire-rated / plain), "Fixture on its wall" (hidden by default); filter by two trades with the Show switches. Done when the sample's 21 shows ~8 real conflicts and the toilet-vs-wet-wall ones are not in it.
3. **Sequence view**: phases (demo, rough framing, rough MEP, inspection, insulation, board, finish) from the cost lines; one click shows the house at the end of a phase in 3D, with the clashes that exist at that phase. Done when I can step phase by phase and read what is covered.
4. **Inspection and hold points list** beside VIF: "rough plumbing: stack, vent, tub trap, before cover" per level, each with a tick box in the printed field list. Done when `export vif` has a second sheet of holds.
5. **Renovation in 3D**: demo in red, existing pale, new full, with a demo quantity (SF, LF, CY of debris), protection and dust-wall lines, and hidden-condition unknowns (lead, asbestos, wiring). Done when the Demolition switch is shown on a 3D picture and its quantities are in Division 2.
6. **Section cut** through a wall or floor (a slider) showing layers and what runs inside, sizes labelled on the runs. Done when I can click a cut line and read "2x6, batt, 5/8 board, 3/4 in pipe at 12 in up".
7. **Labor visible on every line**: hours, crew, rate, per trade, with a "my crew / sub" switch; recap by trade in hours. Done when the sample framing shows hours and a sub scope for each.
8. **Total as a range**: price low / expected / high driven by Assumed/Unknown, and a "what to verify first" bar. Done when the top bar shows a range and clicking an Unknown shows what it moves.
9. **Scan to real conditions**: show out-of-plumb and sloped floors and joist depths from a scan, as Assumed values to confirm. Done when a scan picture shows the actual walls, not a rectangle.
10. **Bigger type and a printable one-page job sheet** for the truck: price, Assumed list with dollars, holds, scope per sub. Done when the type reads at arm's length and the sheet prints in one page.

## 6. What must not change
- "no figure" in red, never zero; "Nothing is priced on 15 things that could be".
- The take-off / keep-lines question when a wall is existing.
- VIF sorted by dollars with the dollars riding; the "Not tested" list on the clash sheet.
- Sub scope export with none of my prices.
- Every number can show what it is made of (the "AS DRAWN / TAKEOFF" panel with formula line, e.g. "plan length x 2 x (width + depth)/12").
- Trade words in the estimate (SF, LF, SQ, CY, EA); the sixteen divisions; the dressed house (siding, stone, windows with grids).
- Layouts side by side for owner options.
