# J23 notes
## Setup a.json
Hip roof 30x20 6:12 eave 9ft Assumed; 4 walls Assumed; starter library; Asphalt shingle roof on plane-0005/0006.
Hand: slope 1.1180; plane1 trapezoid 200 SF plan -> 223.61 SF; plane2 tri 100 -> 111.80. hips 15 LF true. All agree.
Priced sheathing $40/sheet, underlay $0.50/SF, shingles $120/SQ. Hand 660.13 + 350.06 = 1010.19; OH10 101.02; P8 88.90; price 1200.11. Agrees.
VIF: roof row 1010.19 (6 lines counted once), planes 660.13/350.06 below. header says each counted once. OK.
Word: `lines` verb prints "they imply 9 lines" / "9 entered lines agree" about eaves/hips (call 91 says never call them lines). small/words.
Word: show of starter assembly labor: "(ordered by the hr: blank; price: blank)" - production rate shown as "ordered by the hr". words.
## Item 1 measures (a.json)
- measure plane-0005 "eave height" 9'2": put on roof, all planes 9'2"; dollars unchanged 1010.19 (correct: area independent of eave height). Plane rows drop "measure eave height". OK.
- measure plane-0005 pitch same: roof edge 1 Known, plane 1 leaves list, roof still lists edges 2-4. OK.
- footprint same, edges 2,3,4 same: roof leaves list only after edge 4. OK. Lines go Known (no ~). Job cost stays ~ because it leaves out unpriced labor lines (consistent, Material Known).
- NOTE: no check that eave height 9'2" vs wall height 9 ft (walls Assumed) - not a finding per se.
## J23-1 (serious) set on a solved plane: says Known, file left disagreeing
cp a0.json p.json; plumbline set p.json plane-0005 pitch 7:12 --status Known --source gauge
-> "set plane-0005 pitch = 7 in/12 (Known) / figures worked out again on: plane-0005, hip-0013, hip-0014, ridge-0017"
File now holds pitch 7 Known, slope 1.1577, sheathing 8 sheets $320, shingles $277.85.
But show/estimate re-solve: pitch 6 Assumed, $280. `plumbline check p.json` -> exit 1 "figures in the file do not agree with what is drawn for: hip-0013, hip-0014, plane-0005, ridge-0017".
Same with set plane-0005 eave_height "12 ft" --status Known (says Known; thrown away; check fails).
So: a write that should be refused (call 49/91: thrown away on re-solve) is accepted and reported as Known, and leaves a file that fails check.
Also: roof's own edge cannot be set: set roof-0004 "edge 1" -> not a field. measure roof-0004 "edge 1" 7:12 -> "edge 1 is not one length, so one measurement does not give it; move it to where it is" - a pitch is not a length and cannot be "moved". So a roof whose pitch is found NOT as drawn (7:12 not 6:12) has no way to be corrected. words + gap.
## J23-2 measure eave height accepts -3 ft and 0 ft as measured Known.
measure n.json plane-0005 "eave height" "-3 ft" --source x -> taken, Known. Then "0" -> taken. Then "same" -> "eave height is 0 ft, found as drawn" (0 was measured not drawn).
## J23-3 setting an edge to gable drops a priced plane without saying the dollars
cp a0.json g.json; set g.json roof-0004 edge:2 gable --status Assumed --source plan
-> "gone: plane-0006  (it carried an assembly or cost lines, which went with it)". No dollars said ($350.06 of lines, 6 lines incl 2 labor). Call 91 asks deletes to say what was priced and what it takes out of job cost. Coincidence: price stays 1200.12 since plane 1 grows to 300 SF plan/335.41 SF (hand: 11 sheets $440, $167.71, 3.3541 SQ $402.49 = 1010.20). Checked OK.
Also edge:<n> and move:<n> are not in the usage text nor in the "not a field" error message. Discoverability.
## Item 2 drawing (d.json) - held
Traced 4 walls over made-up-plan.pdf calibrated Assumed. Lap siding $2/SF on wall 1: 30x8=240 SF -> $480. VIF: drawing A-1 "check: ~scale dimension" carrying $480; measure scale "same" takes it off, wall stays. "31 ft" refused with a clear sentence; recalibrate 31 ft says NOT MOVED loudly. Held.
## Item 3 remove - held on money
r.json: wall-0005 (9 lines, lap siding $438 on 219 SF net) with door-0040 (Entry door $900, Hang the door 0.5 EA/hr x $50 = 2 hr $100). remove wall-0005 -> "removed door-0040, wall-0005 / 13 cost lines ... $1,438.00 is out of the job cost (10 of them had no price yet)". Hand 438+900+100=1438, 13 lines, 10 unpriced. Correct. check OK.
remove plane-0005 / hip-0013 refused cleanly (file unchanged? not cmp'd). remove roof-0004 -> 14 things, 10 lines, $1,010.19, 4 unpriced. Correct.
Switched-off line: "4 cost lines went with it: $900.00 is out of the job cost (2 of them had no price yet) (1 was switched off, and in no total)." Correct money; double bracket words small.
## J23-4 candidate: recap Labor row "none" when labor lines exist with no rate
a.json estimate: Labor "none", column LABOR "none", while 4 labor lines have no production rate. The row does not say it leaves lines out (only Job cost/PRICE/left-out list do). "none" reads as no labor. silent-ish.
## Item 4 labor (lab.json) - money held
40 SF/hr and 0.025 hr/SF both -> 240 SF = 6 hr; $45 -> $270. Hang the siding 0.02 hr/SF x 240 = 4.8 hr $216. OH 10 P 0: 966 -> 1062.60. off -> 765.60; on -> back. Hand agrees.
Refused (file unchanged): 0, -40 SF/hr, fast, 40 LF/hr on SF line, 40, 0 SF/hr.
J23-5 words on the rate refusals: "is not an order conversion; write it as a number and what per what, like 30 SF/sheet or 1.25 lb/SF" (no hr example for a labor line); "order factor: must be more than zero (and not beyond 1e9)" - "order factor" and "1e9" are not builder's words. show prints labor rate as "(ordered by the hr: blank; ...)".
No sanity on 0.0001 SF/hr -> 2,400,000 hr, $108,000,000 labor, no warning (overhead >100 warns; this does not). small.
apply --change to brick veneer: shared "Frame and sheathe the wall" keeps 40 SF/hr and $45. Good. But "2 cost lines ... came off, with whatever was priced on them: Lap siding, Hang the siding" - does not say the $480 (and $216 switched off) it takes off. words/silent-ish, cf call 91 for deletes.
## Exports (lab0.json, lab2.json)
J23-6: xlsx Recap "Labor" Hours cell is a typed 10.8, not a formula; Estimate quantity D12 (6 hr) is a value. Workbook says "change a quantity or a price here and the amounts, the recap and the price follow" - hours in the recap do not follow. Production rate (40 SF/hr) and the SF it works on (240) are in neither xlsx nor csv: the hours can't be traced to the rate from the export. Labor line with no rate exports honestly (blank qty, Unknown, note). Switched-off line exports "switched off: in no total". OK.
J23-7 words: export assemblies CSV puts "face_area", "net_area" in the Quantity column, "$?/hr", "unknown SF/hr". software identifiers in a file a builder opens.
## J23-3 stronger (k3.json)
cp a.json k3.json (planes 1,2 priced, job cost 1010.19); set k3.json roof-0004 edge:1 gable --status Known --source "plan says"
-> "gone: plane-0005 (it carried an assembly or cost lines, which went with it)". Job cost 1010.19 -> 636.38 (hand: plane 2 now 187.5 SF plan, 209.63 SF: 7 sheets $280 + $104.82 + 2.0963 SQ $251.56 = 636.38). $660.13 of priced lines gone; the sentence names no dollars and no lines. Remove says "$X is out of the job cost"; this path does not.
## Item 5 O&P (lab0.json)
-5, 1001 refused "has to be a percentage from 0 to 1000" (unchanged). 1000, 150 taken. 
J23-8: call 91's warning for >100 % ("If 25.00 % was meant, type it again") is only in crates/app/src/main.rs:693 (window). CLI `set ... overhead 150` / `profit 250` prints "set job-0001 overhead = 150 (Known)" with no warning. estimate shows 150 % plainly. small/consistency.
"10%" accepted as 10. Profit on cost+OH: (966+96.6)*8% = 85.01, price 1147.61 - agrees CLI and xlsx formulas. pm lens gives job cost 966.00 Assumed, leaves out 6 said.
## Other holds
move:1 "2 ft": plane 1 209 SF plan -> 2.3367 SQ, plane 2 121 -> 1.3528 SQ: agree. Footprint goes Assumed after a move on a Known roof, and returns to list. L roof 825 SF plan sums. Two roofs each listed with own dollars, not double counted. checks pass on all files except p.json/e.json (J23-1).
J23-2 confirmed inconsistent: measure wall-0048 height "-2 ft" or "0" refused "must be more than zero"; roof eave height -3 ft / 0 ft taken as measured Known (z.json; check passes; price unchanged, so no money, but a Known nonsense value).
## Done. Report handed back.
