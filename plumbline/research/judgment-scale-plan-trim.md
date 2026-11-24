# scale-plan-trim notes
Made-up sheets: gen.py/run.sh in the folder scale-plan-trim/ (30x20 ft house, 1/4 in/ft, dimension strings by tick lines).
## Part 1 findings (scale)

F1 (wrong money risk, medium-high) A scale the person set as Known is silently replaced by an Assumed one.
 Repro: sheet wrong1.pdf (30x20 house at 1/4 in/ft; one string "30'-0\"" drawn 270 pts long). new; level; drawing A-1 --file wrong1.pdf;
 calibrate --from "126,126" --to "666,126" --distance "30 ft" --status Known --source tape  (=> 1/4"=1'); lineup; trace 2 walls --on;
 find-walls job drawing --scale-by 1.  Said: "scaled by string 1, 30'-0": 1/8" = 1'-0", Assumed, its source the string; the job was written with it (nothing else is changed)".
 Nothing says a Known scale was replaced (and by a lone string, no caution because 1/8 is a real scale). `show` afterwards: "~ scale dimension 30 ft" (Assumed). The walls traced before ARE flagged NOT MOVED (good), but the downgrade of Known is unsaid.
F2 (wrong quantity risk, medium) A single dimension string that is wrong (line drawn 15 ft long marked 30'-0") gives a standard-looking scale (1/8") so no caution fires;
 --scale-by 1 then finds the 30x20 house as 60x40 ft with walls "1' 0" thick" (a 12 in wall) and says nothing. The "only one string ... check it against the sheet" appears only in the --dimensions listing, not repeated with --scale-by output. No plausibility test (thickness/size).
F3 (word) zero string "0'-0\"" listed as "inf to the foot" and "only one string: it gives inf to the foot; say --scale-by 1"; --scale-by 1 then refuses with "scale distance: is more than nothing" (software-ish phrase, no instruction). Tiny "0'-6\"" gives scale "1" = 0' 0 3/4"" and is accepted with caution (ok).
F4 (untidy) A dimension typed in inches only ("360\"", "240 in"), metres ("9.144 m") or mm are not read; listing says "none is a dimension written in feet and inches" (honest, good) -- but with one readable and one unreadable string ("30'-0\"" + "240 in") the unreadable one is not mentioned at all: "1 read". Lone-string scaling then is offered.

F5 (wrong quantity, medium) Walls LAID by find-walls --lay are not treated like traced walls when the scale is later changed.
 Repro: job, drawing wrong1.pdf, calibrate 30 ft/540 pts Assumed, lineup, find-walls --lay (4 walls, 100.00 LF), then find-walls --scale-by 1 (=> 1/8", lists 60x40 ft house).
 takeoff still says 100.00 LF with no note; `show`/`drawings`/`check` carry no "traced before it was rescaled" flag (that flag exists only for walls traced with --on: seen in test k). Laying again says only "already has 4 walls ... Take those away first".
 Right: laid walls came from the sheet at the old scale and the sheet now says another; it should be said as for traced ones.

## Part 2 (drawings on a sheet) -- sheets from gen2.py
F6 (wrong quantity, medium-low) Collinear walls of two houses 15 ft apart are joined into ONE wall and ONE drawing.
 Repro: sheet "three": houses 30x20 at x=0, 28x18 at x=45, 26x16 at x=90 (south walls all on y=0). find-walls (scaled 1/4):
 "wall 1: from 0' 0" east ... to 73' 0" east, 73' 0" long ; door or open doorway: 30' 3" to 44' 9" from its west end, 14' 6" wide" and no word of 3 drawings; the first two houses are one drawing; the third is "in doubt: apart from the plan".
 --lay lays "the walls ... whole across the 1 opening": a 73 LF wall across a 15 ft gap of nothing. Right: 30 + 28 = 58 LF on that line, two drawings. (A 20 ft gap (sheet "like") is split correctly, so the 15 ft rule, which is the door rule, decides.)

F7 (WRONG QUANTITY, worst of part 2) --plan <n> lays a second drawing that sits inside the first one's bounding box.
 Sheet nested.pdf: drawing A = 3 sides of a big U (120 ft south, 60 ft west, 60 ft east pairs, 240 LF), drawing B = a 40x20 house (120 LF) standing 25 ft from A's south wall, inside A's bounding box. Scaled 1/4", lined up.
 find-walls: "2 drawings on this sheet: about 240 ft and about 120 ft", "drawing 1: 3 walls, about 240 ft", "drawing 2: 4 walls, about 120 ft".
 find-walls --plan 1: "taking drawing 1 of the sheet's 2: 0 lines outside it are left out" / "walls found: 7"; --plan 1 --lay: "laid on ground: 7 walls"; takeoff: 360.00 LF (right: 3 walls, 240 LF).
 So the line "0 lines outside it are left out" is false, and the laying is 120 LF of another drawing the person did not ask for, with no word. --plan 2 is right (4 walls, 120 LF).
 (STATUS names a few more/fewer walls; this one is the whole other drawing.)
F8 (misleading, low) --within whose edge crosses a wall: lines that straddle the edge are dropped whole and counted only as "18 lines outside it are left out"; result "no wall found". Nothing says some lines were cut by the box. (like.pdf, box -1,-1 to 15,21 over a 30-ft house.)
F9 (untidy) laying drawing 2 after drawing 1 into one level: "level-0002 already has 5 walls found on sheet A-1: laying it again would make two of each. Take those away first": says "two of each" though the second is a different drawing, and names no way to lay both or take away (no verb named).
Held: --plan 0 / 3 refused with counts; corners in wrong order accepted and said back normalised; laying without --plan refused leaving job unchanged (earlier demo); like-size (120/120) and exact half (50/100) listed as 2 drawings; 49/100 not listed but its walls said "apart from the plan: another plan, a detail or a section", in doubt, not laid; plan+section: section in doubt, not laid; --plan 1 laid 5 walls=120.00 LF matches listing on sheet "like".

## Part 3 (trim --pieces)
NOTE: the STATUS.md section "Where each piece of outside trim lies" and DECISIONS "Call 185" pieces text are not in the repository as read (grep "--pieces" finds nothing in STATUS/DECISIONS); hand figures here are my own from Call 185's rules (eave: length x overhang; corner board 9 ft x outside corners; casing).
F10 (misleading / "list says nothing", medium) A priced line that is not recognised as the piece's length is silently left out of `trim --pieces`, with the kind's total saying it agrees.
 Repro: bash mk.sh-style hip house 24x16 walls, roof -1,-1..25,17 6:12, starter assemblies applied (Gutter on 4 eaves). set eave-0017 quantity "plan_length" --line "Fascia" --status Assumed --source x.
 estimate: Fascia 26+18+26+18 = 88 LF (eave 1's still 26 LF). trim --pieces: "fascia: 3 pieces come to 62 LF; the lines price 62 LF" and NO "Left out" section (grep: 0). Eave 1's fascia is not drawn and the 26 LF the estimate prices on it is not said missing. Same with a typed quantity "26" (EA, 62 LF said).
 Right: "Left out: eave 1 [eave-0017]: Fascia prices 26 ..." as it does for "true_length * 2" (that case is said correctly: "0 pieces come to 0 LF; the lines price 52 LF (they differ: see what is left out)").
 (Equivalent forms "true_length + 0", "true_length*1", "(true_length)" are recognised and drawn.)
F11 (untidy, low) kinds that sum rounded differently: "rake fascia: 4 pieces come to 49.1935 LF; the lines price 49.1936 LF" (gable 32x22 roof, 6:12) -- the line says they agree, the figures differ in the 4th decimal (hand: 4 x 11 x 1.118034 = 49.19350). Also "rake fascia: 2 pieces come to 46.3801 LF; the lines price 46.38 LF" (shed). Zero-overhang roof draws 4 soffit pieces "0 in wide" ("soffit: 4 pieces come to 0 SF").
F12 (wrong quantity, low at 9 in, grows with overhang) Hip-roof soffit is priced length x overhang per eave, so the four corners are counted twice, and the pieces drawn overlap.
 Repro: 30x20 walls (6 in), hip roof -3,-3..33,23 at 4:12 (overhang 2.75 ft): trim --pieces "soffit: 4 pieces come to 341 SF; the lines price 341 SF". Hand: ring between the roof edge 36x26 and the wall face 30.5x20.5 = 936 - 625.25 = 310.75 SF; 341 is 30.25 SF (4 x 2.75^2) too much. At 9 in the excess is 4 x 0.5625 = 2.25 SF (66 vs 63.75 on the 24x16 house). The list says the pieces "add up to what the line prices", which hides it.
F13 (misleading, low) A soffit with unknown overhang shows in the kinds table as "soffit: 0 pieces come to 0 SF; the lines price 0 SF" (roof alone, no wall under it); the estimate says "no figure". "Left out" does say why, but the table row reads as zero.
F14 (untidy, low) Door 3 ft at 1.5 ft along its wall: its casing leg is drawn at x=-0.1458, i.e. out past the wall's end where the corner board (x=-0.1042) stands; two pieces occupy the same place, unsaid. A measured overhang cannot be entered ("overhang already means something on a line; give the input another name"), so an unknown soffit can be priced only by drawing a wall.
Held (trim): hip 24x16 w/ 3 windows: fascia 88, soffit 66 (88 x 0.75), corner boards 72 (8 x 9), casing 48 (3 x 16) all agree with hand; gable 32x22: rake slope 11 x 1.118034 = 12.2984, ridge rise 5.5 ft; shed (ridge on north), L-shape (108 LF fascia, 81 SF soffit, 90 LF corner boards = 10 x 9), house turned 30 degrees (same totals, places rotated correctly), no-overhang roof (0 SF, said), line on one eave only (kind total follows: 26 LF), formula "true_length * 2" ("0 pieces come to 0 LF; the lines price 52 LF (they differ: see what is left out)" + Left out line), equal forms "true_length + 0/*1/(true_length)" drawn, same-named line twice refused, three lines on one eave ("Fascia", "Fascia, primed", "Fascia board") said in Left out as drawn once, no piece was shown Known (all "~"), pieces only where a line prices.
Not tried: trim without --pieces compared with an earlier run of itself (its figures agree with the estimate in every house above); IFC export of pieces; labor lines (refused: needs a production rate).
