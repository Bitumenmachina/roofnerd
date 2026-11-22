# Sweep notes: a plumber and an HVAC mechanic (residential, a bathroom moves, an addition goes on)

Method. Pictures only plus the frozen command line (bin22) on a made-up whole house (`sixteen`, file `plumber-hvac/h.json`; copies d.json, m.json for demolish and move tries). I did not see the live window. "Not seen" = may exist, no picture shows it.

Pictures read (31): trades/4-plumbing, 5-hvac, first-floor-alone, 2-framing; layers/01-05; house/05, 07; clash/01, 02, 03, 04, 05, 06, 07-model, 07-sheet; carry/01, 03, 05-model, 05-sheet; renow/05, 10; scanw/10; field/05; glass/05; layoutsw/05; d2/06; money/03.
CLI run: `sixteen`, `takeoff`, `clashes`, `clashes --clearance`, `vif`, `show`, `mark ... demolish --clear`, `move`, `export subcontractor --division 15`.

---
## 1. Who I am and why I open this

**Plumber.** (1) Where is the stack, and what does the moved bathroom have to reach: drain, vent, hot, cold, at what size and fall. (2) What is in the wall that comes down, and what do I cap, reroute or save. (3) What crosses a joist, a beam or a footing, and what holes do I drill. (4) Count: fixtures, rough-in heights, pipe by size and material, fittings, so I can bid rough, top-out and trim.

**HVAC mechanic.** (1) Does the trunk and branch fit: joist bays, soffits, chases, the wall that opens. (2) What does the addition add to the load, and can the existing air handler carry it. (3) Registers and returns by room, with sizes. (4) Clearances: air handler service side, flue, water heater, cable, pipe.

Both: I want one picture where my run is readable (colour, size, label), a clash that tells me what to do, and a moved vanity that makes the clashes change in front of me.

---
## 2. What I saw

**trades/4-plumbing.png** (See through + Plumbing on). Five-second read: a ghosted house, a few thin blue, red and brown lines, two fixture boxes, a pale vertical stack. I can tell cold from hot (blue/red) and that a brown line goes down. I cannot read a single size, a slope, a label or which line is the vent. The pipes are hairlines against the glass walls; the stack is a pale column that gets lost. Three of the four duct-looking boxes are fixtures. Nothing says "stack" on screen. Walls and glass frames behind compete with the lines.

**trades/5-hvac.png** (Mechanical on). Much better: trunks and branches are fat, blue, readable as a duct system; a pink vertical is the return riser. A purple outline appears (refrigerant, I suppose). No size labels, no CFM, no register symbols visible, no air-handler label. I can tell where trunks run through the building but not whether a 16x8 trunk fits the bay: there are no joists to compare against. See-through works for HVAC far better than for plumbing because ducts are big.

**layers/05-model.png and house/05-model.png** (first floor alone, MEP on). The best "one building" picture: ducts (blue), cable (orange), pipe (cyan/red/brown), return (pink). The Systems panel at the left (house/05) lists Supply air 195 LF, Return air 22 LF, Refrigerant 34.5 LF, Cold water 69.6 LF, Hot water 50 LF, Sanitary 54.8 LF, then more below, each with a colour key and a switch. That is exactly the mechanic's language and is the best thing on screen for me. Missing: a size or label on the run itself.

**house/07-model.png** (plan, first floor). Rooms named with SF, ducts as pale blue bands, cable orange, plumbing thin. A selected cable shows "Cable 0.4 in round, assumed", length 12.50 LF, height 9.5 ft, system Lighting. No pipe or duct tags on the plan, no register symbols, no fixture symbols that read as a lavatory or water closet. A plumber reading a plan expects fixture symbols, pipe sizes, and "up"/"down" arrows; none seen.

**clash/04..06, 07-model/07-sheet** (a 1 in pipe, a duct, the Fitting tool). The Fitting tool says "Click where runs meet" and the status says "Added 6 in round tee, a tee; duct 1 cut there." That is the right trade behaviour. The selected pipe panel reads Type "1 in" (no material), Height above its level 8 ft, System Cold water, True length 14.00 LF. Clash sheet: "In the same space 1 . 1 to verify: duct 1 and pipe 1, duct and pipe, 3.50 in, where E 4' 0", N 2' 0", up 8' 0"". It says what hit what and how far, but not what to do (move which, by how much, or go around/under) and "where" is in coordinates, not a room or a bay. "Not tested" lists "walls and floors not tested against each other", "a joint is not a clash", "the roof's planes are not tested". It does not list joists/studs/beams, because there are none to test; the list is honest about what it skips but silent about that.

**carry/03 and 05** (a vanity moved by hand). The vanity turns red, and a message reads "Harbor Vanity 36: in the same space as Water closet (1 11/16 in into each other). Keep going, or let go and it stands here." Undo reads "Undone... It is clear of everything." This is the thing I asked for: move it, the clash lights and clears. It is about the vanity box only; it does not say the lavatory's drain and supply must follow, nor what they now cross.

**renow/05, 10, d2/06, scanw/10, layoutsw/05, field/05, money/03, glass/05.** Renovation pictures: existing shown pale, demolition shown red (renow/10 shows a red wall section) with switches for each. Good and plain. No picture shows a pipe or duct inside the red wall; no picture of the "In the way of demolition" list (it exists on the command line, see below). field/05 shows a duct as "12 x 8", Assumed, with duct surface area 66.67 SF: a sheet-metal man's number, good.

**trades/2-framing, layers/03.** Framing is wall studs and plates only. No floor joists, no beam, no rafters drawn, so I cannot see a joist bay or what a pipe crosses.

**Words not in my trade.** "In the same space", "Not tested", "Open items", "Assumed/Known/Unknown" (these are the program's own and explained). "Fitting" tool next to "Equipment" and "Fixture" is right. Plumber's words missing: stack, vent, trap, cleanout, wye, DFU/fixture unit, rough-in, top-out, trim, closet flange. HVAC's missing: CFM, tons, static, boot, damper, plenum, register vs grille (it does say "Ceiling register 4x10", "Return grille 20x20").

---
## 3. What is missing or wrong for me (one line each)

Plumber
1. Looked for fall on the drain. Got: the building drain is level (both ends -1.6 ft); no slope field anywhere in the model (grep). Matters: a drain at 1/4 in per ft is the first inspection item; the length to reach the stack changes the depth at the main.
2. Looked for pipe material (PVC DWV, cast iron, PEX, copper). Got a diameter only: "Pipe 3 in round", all priced $9.50/LF whether 1/2 in or 4 in. Matters: wrong bid by hundreds per bathroom.
3. Looked for fixture units and a vent/trap-arm check. Got none. Matters: failed rough-in inspection when a moved lavatory sits farther than its trap arm allows.
4. Looked at what a moved fixture must reach. Got: the takeoff names "open: lavatory 1: its Cold water connection at (8, 18.8, 2)" and "cold to lavatory 1... second end at (16, 21.8, 2)" after I moved it 8 ft: useful, but no distance, no "nearest cold line is here", no waste or vent shown open. Matters: the whole moved-bathroom bid is that run.
5. Looked for what crosses a joist, beam or footing. Got: only "Through a wall or a floor (a hole to cut)", 71 of them in the sample, one line per pipe piece (every pipe in a wet wall listed separately). No joist, so no notching/boring. Matters: structural change order and an engineer's letter.
6. Wall that comes down: GOOD. After `mark wall demolish --clear`, `clashes` says "In the way of demolition: 12 ... cap it, reroute it or take it out", and `vif` repeats it. But it is listed one pipe piece at a time (soil stack, hot, cold, waste each a row), cold/hot/waste for one fixture are not grouped, and the soil stack in a wall that comes down gets the same "cap it" as a branch. Matters: the stack is the whole job; a stack in a coming-down wall is a different, larger decision.
7. The fixtures against their own wall are still clashes: "wet wall and Lavatory in each other by 1.20 in", "Water closet 1.80 in", "partition and Bathtub 0.45 in", "north wall and Kitchen sink 1.80 in" lead the list. Matters: a plumber learns to ignore the list; 8 of 21 "same space" findings in the sample are mountings.
8. Water heater clashes: "Water heater 50 gal and cold main 6.38 in", with $2,148 riding. The dollars are the line prices on the two items, not the cost of the problem. Matters: sorting by dollars puts an equipment price over the real problem.
9. Pipes whose size is "not known" are not tested at all (5 "Not tested"; hot to the kitchen sink). Matters: the unknown pipe is the one most likely to hit something; it says so, which is honest, but the vif row is only "?type".
10. Price: fittings $38 each for every size; tub and water closet both "Set and connected $640". No rough/top-out/trim split, no hours. Matters: I cannot give a bid in my own form.
11. Existing pipe marked existing appears in the subcontract export as "Piping, installed 0.6 LF, existing to remain": the Work column says so, but a plumber scanning the quantity column sees LF he does not install.

HVAC
12. Looked for a load. Nothing holds a CFM, tons, BTU or static pressure (grep over crates: none). The heat pump and air handler are boxes "24 x 30 x 60 in, a typical size; the product is not specified". Matters: whether the existing unit carries the addition is the first question; the program cannot hold the answer even as an Assumed number.
13. Registers by room: the takeoff counts "Ceiling register 4 x 10: 9 EA" and "Return grille 20 x 20: 2 EA", but nothing groups them by room or ties a CFM to a register. Matters: a register schedule is how the mechanic reads the job.
14. Joist bays and chases: ducts are drawn at 8.4 ft and 20 ft; no joists, so "does the 16 x 8 trunk fit the bay" cannot be answered. The clash list shows each branch "through hall partition... a hole to cut" (partition holes at 8.4 ft up), which is useful (a hole count), but the 8-inch deep trunk through a 2x10 bay cannot be tested. Matters: the most common HVAC renovation surprise.
15. Duct clashes: duct vs soil stack 3.50 in at (18.5, 22, 8.4); duct vs vent through the roof 5.80 in. Real findings, correct. But the row says only "in each other by"; it does not say "trunk crosses the 3 in stack: offset the trunk or go over". Matters: that is the decision.
16. Open ends: takeoff says "4 open ends; 10 places where a run meets another with no fitting", each listed. Good (the useful thing); but takeoffs, boots, end caps, dampers are not items, so every open end looks equally wrong.
17. Return path: a return grille on each floor and a 20x10 return riser exist, so returns are carried; but no check that supply CFM has a return (can't, no CFM).
18. Clearances: `--clearance "2 in"` finds 60 (e.g. "floor and Air handler 0.00 in apart", "Return grille and floor 1.20 in"): one number for all, floors included. Matters: no service clearance in front of an air handler or a water heater, no flue clearance.
19. Equipment price: air handler $4,300 and heat pump $7,800 as "Set and connected" lumps; nothing about line set, pad, disconnect, condensate. Matters: bid completeness.

Both
20. The 3D view gives no label on a run: no size, no system name on screen without a click. Clicking shows it in the right panel (seen for a pipe and a duct).
21. No "show me only this system and its clashes" combined: I can switch systems on and off (good), and Clashes is its own switch.
22. Moving a fixture moves its pipe? Not seen in any picture; on the command line `move` of the lavatory left its pipes where they were and produced open ends (this is honest and useful, but a plumber will expect a "stretch the runs with it" option).

---
## 4. What the best tools show today

- Revit pipe slope: Mechanical Settings holds a Slopes list; a pipe whose ends are at different elevations takes a slope shown as rise/run, angle, percent or ratio ([Autodesk help](https://help.autodesk.com/cloudhelp/2014/ENU/Revit/files/GUID-B9C38F9B-A11B-4369-B879-0A641A3E725F.htm)).
- Revit fixture units to flow, and pipe/duct sizing by friction and velocity; a MEP Sizing Calculator replaces a ductulator ([Autodesk MEP sizing](https://help.autodesk.com/cloudhelp/2027/ENU/Revit-MEPEng/files/GUID-89D54FC4-7B1A-459F-98D1-1C6B0E4E40BB.htm), [Autodesk course module](https://www.autodesk.com/learn/ondemand/course/introduction-to-bim-for-mep-design/module/60tHzErS7jMEZ6BoSOsGr0)).
- Trimble SysQue: for MEP contractors, clash detection and coordination, fabrication-ready models ([listing](https://platform.softwareone.com/product/trimble-sysque/PCP-4897-2963)); Trimble Connect lets a clash be assigned to the party who fixes it ([Trimble blog](https://www.trimble.com/en/blog/construction/article/clash-detection-in-bim-projects)).
- ACCA-approved load software (Wrightsoft Right-J, Elite RHVAC): room-by-room Manual J loads, Manual D duct sizing with fan static pressure, Manual S equipment selection; room CFM is what selects registers and sizes ducts ([ACCA approved software](https://www.acca.org/approved-software), [ACHR/Elite](https://www.achrnews.com/articles/135541-elite-software-hvac-design-software)).

Where this program could be ahead rather than level: (a) none of those start from a renovation: existing Known/Assumed, demolish and "in the way of demolition" with a cap/reroute/take-out instruction already exist here and are better than anything in that list for a renovation bid. (b) The verify-in-field list with dollars, driven by clashes, is not in those tools. (c) A moved fixture showing open ends and clashes live is a trade-specific question those tools answer only after a re-route. (d) Honest Known/Assumed on the load, which load software does not carry (it gives a number with no source). A plain, sourced "Assumed 24,000 BTU/h for the addition (rule of thumb, not Manual J)" would be ahead of what a bidding mechanic does now.

---
## 5. My ten, in order

1. **Joists, beams and bays drawn, and tested.** Draw floor joists (the model already counts them, call 165: 19 joists, rim 48 LF) as thin members in See through, with a Framing switch for floors; a pipe or duct crossing one says "crosses 3 joists: bore (3 holes) / notch" and a duct deeper than the bay says "12 x 8 trunk in a 9 1/4 in bay: fits, 1 in left". Done when the sample clash list contains a pipe-vs-joist and duct-vs-bay row and the row count of holes appears in the subcontract scope.
2. **Slope and material on a pipe.** Fields "Fall 1/4 in per ft" and "PVC DWV / cast iron / PEX / copper" on a pipe type; the panel shows them; a drain flatter than its minimum is a finding; the stack is a vertical run drawn with a name. Done when `show` of the building drain gives a fall and the price differs by material.
3. **Group the demolition list by wall and by system.** "Wet wall comes down: stack 3 in (cap/reroute), hot 3/4 in, cold 3/4 in, waste 1 1/2 in, trunk 16x8" with one row per system and the stack called out first. Click selects them all. Done when the 12 rows in my try read as 4 to 5 and the stack row says what a stack in a coming-down wall means.
4. **A moved fixture shows what it must reach.** After a move: draw a dashed line from each open connection to the nearest run of the same system with the length, and list it ("lavatory 1: cold 8.5 ft, hot 8.5 ft, waste 11 ft, vent: none within 5 ft"). Done when the carry demonstration prints those lengths in the status line.
5. **Mounting is not a clash.** A fixture against its own wall, a cable ending at a device, and a branch into its own trunk are filtered or shown grey under "set against". Done when the sample "same space" count drops from 21 and the first row is a real crossing (trunk vs stack).
6. **A clash says what to do.** Each row ends with the trade's own options: "offset the trunk over the 3 in stack, or move the stack"; "bore 2 1/2 in through 3 joists". Done when every sheet row has an action sentence and a room name (not only E/N/up).
7. **Load and capacity as Assumed numbers.** Equipment gets Tons/BTU/CFM/gallons and a room gets Assumed CFM; the addition adds a labelled figure; the sheet says "air handler 3.5 tons Assumed; addition adds 1,200 CFM Assumed: not checked against a Manual J". Done when the VIF list asks for the nameplate.
8. **Labels in the view.** With a trade on, runs carry a small tag "3 in SAN", "16x8 SA", "Reg 4x10", on hover or zoom, and fixtures have trade symbols in plan; the stack is labelled and drawn heavier. Done when picture trades/4-plumbing reads size and system without a click.
9. **Rough / top-out / trim and sheet-metal by weight.** Split fixture lines into rough, top-out and trim with hours; duct priced by size class; fittings priced by size. Done when the subcontract sheet has the three phases and the 16x8 elbow is not the same price as the 6 in.
10. **Clearances by trade.** Air handler service side, water heater, closet-flange side clearance, as named clearances, with the generic one kept. Done when the list gives "Water closet 15 in side: 12 in" instead of one 2 in number.

---
## 6. What must not change

- The Systems list with colour key, LF per system and a switch for each (house/05), and the per-trade switches.
- See through and the first-floor-alone view: the first lets me see through the house; the second reads best.
- "In the way of demolition ... cap it, reroute it or take it out", and `vif` carrying it.
- The takeoff's "open ends" and "no fitting at (x,y,z)" lists, and "joined: n networks".
- The vanity move with the live red box, "Keep going, or let go and it stands here", and "It is clear of everything" on undo.
- "A hole to cut" for a run through a wall or floor; "Not tested" saying what was skipped, including pipes with no known size.
- Subcontract export with Work, Division and System columns and `--division 15`/`--system`.
- Known/Assumed/Unknown on every size; "a typical size; the product is not specified" on equipment.
- Plain trade words on screen (Duct, Pipe, Fitting, Equipment, Fixture; "12 x 8").
