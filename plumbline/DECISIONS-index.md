# Index of DECISIONS.md

Written by `tools/index.sh`; not edited by hand. Every heading of `DECISIONS.md` and the line it
starts on. A call with "as built" in its heading is the record of what was made.

| Line | Heading |
|---:|---|
| 14 | What the starting landscape had wrong or missing |
| 32 | Every candidate, rated |
| 74 | Calls |
| 76 | 1. The world: build on `bevy_ecs` 0.20.1, on its own |
| 85 | 2. The 3D window: build on Bevy 0.20. Proven on a virtual display (see call 23). |
| 106 | 3. Roof from a footprint: borrow BIMCraft's weighted straight skeleton |
| 128 | 4. BIMCraft as a whole: borrow from, do not build on |
| 137 | 5. Saved job: write. IFC5 is the pattern only. |
| 143 | 6. IFC parsing and IFC4 export: write the export; link no parser |
| 149 | 7. Solid geometry kernel: none |
| 158 | 8. Constraint solver: not used |
| 163 | 9. Spreadsheet: build on `rust_xlsxwriter` for XLSX out; IronCalc is the formula-engine candidate |
| 181 | 10. Plans and images to trace: build on `hayro` for PDF, `image` for images |
| 186 | 11. Takeoff and estimating projects: reference only or not used |
| 194 | 12. Capture: no call until the owner rules on D7 |
| 198 | 13. This project's own license: MIT OR Apache-2.0 |
| 202 | 14. Platform exceptions to "Rust by default" |
| 208 | 15. Model calls made in D1 (ours to make; recorded so they can be overruled) |
| 236 | 16. Units: held as entered, converted only for arithmetic (owner ruling, 2026-11-10) |
| 285 | 17. A roof's lines are read off its planes: write (D2 groundwork) |
| 304 | 18. Low-slope solvers already exist in the owner's earlier program: borrow candidate |
| 330 | 19. Calls made fixing what the third judgment found |
| 353 | 20. Calls made fixing what the fourth judgment found |
| 372 | 21. Model calls made in D3 (ours to make; recorded so they can be overruled) |
| 413 | 22. Calls made fixing what the fifth judgment found |
| 428 | 23. Decisions made on the way to the 3D prototype (the owner handed them over) |
| 470 | 24. Calls made fixing what the sixth judgment found |
| 497 | 25. The windows drawn again (owner, 2026-11-11: the prototype "looks cheap and tacky") |
| 552 | 26. The whole building, not the roof alone (owner, 2026-11-11) |
| 622 | 27. Calls made fixing what the eighth judgment found |
| 645 | 28. What was already on the machine and was not read (owner's question, 2026-11-11) |
| 704 | 29. The building model re-based on BIMCraft: what each thing becomes (2026-11-11) |
| 858 | 30. Model calls made in D4, macro to micro (ours to make; recorded so they can be overruled) |
| 902 | 31. Calls made fixing what the ninth judgment found (2026-11-11) |
| 943 | Call 32. D5: the estimate sheet, the recap, materials and suppliers (2026-11-11) |
| 994 | Call 33. The tenth judgment: what was decided in fixing it (2026-11-11) |
| 1022 | Call 34. Drawing and editing: the undo history (2026-11-11) |
| 1039 | Call 35. Drawing and editing: the verbs (2026-11-11) |
| 1067 | Call 36. Drawing and editing: snapping (2026-11-11) |
| 1088 | Call 37. The eleventh judgment: what was decided in fixing it (2026-11-11) |
| 1120 | Call 38. Drawing and editing: the tools in the window (2026-11-11) |
| 1162 | Call 39. Fittings, connections and a run cut in two (2026-11-11) |
| 1207 | Call 40. Clash detection (2026-11-11) |
| 1245 | Call 41. The twelfth judgment: what was decided in fixing it (2026-11-11) |
| 1298 | Call 42. Patrick's rulings of 2026-11-11, as he gave them, and how they were folded in |
| 1381 | Call 43. Plans under the tracing: what is built on, and the plan set (2026-11-11) |
| 1435 | Call 44. The thirteenth judgment: what was decided in fixing it (2026-11-11) |
| 1509 | Call 45. Tables brought in: what is built on, and what each table is (2026-11-11) |
| 1554 | Call 46. The fourteenth judgment: what was decided in fixing it (2026-11-11) |
| 1616 | Call 47. A model brought in: what is read, and what is listed instead (2026-11-11) |
| 1657 | Call 48. The language pass: one place where every word on screen is said (2026-11-11) |
| 1687 | Call 49. Field entry: what a laser types, and what a measurement does to the job (2026-11-11) |
| 1763 | Call 50. The fifteenth judgment, part A: what was decided in fixing it (2026-11-11) |
| 1858 | Call 51. The sixteenth judgment: what was decided in fixing it (2026-11-12) |
| 1919 | Call 52. The fifteenth judgment, part B: the words on screen (2026-11-12) |
| 2000 | Call 53. Floors, rooms and roofs drawn in the window: what is ported and what each tool does (2026-11-12) |
| 2033 | Call 54. The verify-in-field list: what a question is and what rides on it (2026-11-12, before it is built) |
| 2066 | Call 55. The screen, redrawn: what building games get right, and what is taken from them (2026-11-12) |
| 2180 | Call 56. Bringing in from the window, and a new job from the window (2026-11-12) |
| 2210 | Call 57. Patrick, 2026-11-12: the words are mine to choose; the focus is all sixteen divisions; the house is dressed |
| 2292 | Call 58. The house is dressed in what the estimate prices (2026-11-12) |
| 2345 | Call 59. What the seventeenth judgment changed, and why (2026-11-12) |
| 2416 | Call 60. Assemblies: a starter library, picked from a list, and one word for it (2026-11-12) |
| 2454 | Call 61. A figure is named by its words; a run cut in two is one length (2026-11-12) |
| 2481 | Call 62. Patrick, 2026-11-12 (later): the window is to feel like Google Earth and look like glass with depth |
| 2549 | Call 63. What the window is rebuilt from: two surveys, and the numbers taken (2026-11-12) |
| 2607 | Call 64. A tool picked shows the sizes it will place, every time (2026-11-12) |
| 2634 | Call 65. A fixture or a piece of furniture is written to the BIM file as the sort IFC names (2026-11-12) |
| 2667 | Call 66. From judge 19's notes: how sure a clash is shows on its mark; one count; places and prompts reworded (2026-11-12) |
| 2709 | Call 67. After judge 19's report: nothing is lost or left out without a word, and what the first pictures showed (2026-11-12) |
| 2749 | Call 68. The house is lit by a sun and by its sky (2026-11-12) |
| 2790 | Call 69. Patrick, 2026-11-12 (night): one snapshot of each of seven trades |
| 2821 | Call 70. A strip footing is a kind of its own, held as a wall is (2026-11-12) |
| 2862 | Call 71. A wall whose framing is priced can be drawn as its frame (2026-11-12) |
| 2899 | Call 72. The finish can be taken off the walls, and what keeps the water out is seen (2026-11-12) |
| 2926 | Call 73. The base in a room is drawn where it is priced (2026-11-12) |
| 2950 | Call 74. The made-up whole house has its foundation drawn, and priced by what is drawn (2026-11-12) |
| 2978 | Call 75. Patrick, 2026-11-13: each of the sixteen divisions will have its own AI harness |
| 3012 | Call 76. Patrick, 2026-11-13: the two rulings on harnesses stand, and the repository is to be left clean for the next run |
| 3034 | Call 77. Walls whose heights are equally sure are said in one sentence (2026-11-13) |
| 3046 | Call 78. Where a run's height is taken to, and what the sixteen leave out, said plainly (2026-11-13) |
| 3065 | Call 79. A detail of one line that cannot be extended says "its one line" (2026-11-13) |
| 3075 | Call 80. A drawing's scale is always said as a scale (2026-11-13) |
| 3087 | Call 81. On plan a room carries its name and its floor area (2026-11-13) |
| 3100 | Call 82. Hours on a labor line with no price are said beside the recap's hours (2026-11-16) |
| 3112 | Call 83. Dollars ride on a measurement only where the quantity is in question (2026-11-16) |
| 3126 | Call 84. A coverage on a price list buys whole ones by the same rule as one typed (2026-11-16) |
| 3140 | Call 85. One value entered on a line, standing alone, is what was entered (2026-11-16) |
| 3154 | Call 86. The first wall of a new job: what is offered is real, an example says it is one, and a number typed is never a tool (2026-11-16) |
| 3181 | Call 87. A wall type and a footing type are what they were made as (2026-11-16) |
| 3196 | Call 88. The command line says a drawing's scale as the window does (2026-11-16) |
| 3206 | Call 89. Words and figures on the estimate window, from the look judge (2026-11-16) |
| 3231 | Call 90. A cost line is added in the window, and in-house work says how fast it goes (2026-11-16) |
| 3257 | Call 91. A roof drawn from its footprint is measured on site, at the roof or at any plane of it (2026-11-16) |
| 3304 | Call 92. The starter assemblies carry labor, with no production rate and no wage in them (2026-11-16) |
| 3338 | Call 93. The bars are cards with the model between them, and a panel has a shadow on plan (2026-11-16) |
| 3363 | Call 94. Picking a plan: a file, a folder and a page are a builder's words; the page is picked, not typed (2026-11-16) |
| 3390 | Call 95. What the twenty-third judgment found on the money path, and what was decided (2026-11-16) |
| 3436 | Call 96. Over a dressed roof the takeoff's colours wait until they are asked for (2026-11-16) |
| 3460 | Call 97. What the twenty-fourth judge's look and words changed in the window (2026-11-16) |
| 3523 | Call 98. Every panel has a switch; the lists a file is picked from, in a builder's words (2026-11-16) |
| 3549 | Call 99. A plan is not lit; a floor's underside is a ceiling; no grid with the ground off (2026-11-16) |
| 3564 | Call 100. A roof is priced once; a rate can be said in squares; "entered on screen" (2026-11-16) |
| 3590 | Call 101. More of the twenty-fourth judge's marked words (2026-11-16) |
| 3615 | Call 102. A sink, a water heater and an outdoor unit are drawn as what they are (2026-11-16) |
| 3637 | Call 103. A roof change that drops a priced plane says the job cost before and after (2026-11-16) |
| 3654 | Call 104. A price list prices the cost lines named as its items, where they have no price (2026-11-16) |
| 3680 | Call 105. Patrick, 2026-11-16: a workbench for items, by trade, that can be modified like a database (and how it was read) |
| 3733 | Call 106. A step taken back says what the step was; "doors and windows" wants one of each (2026-11-16) |
| 3754 | Call 107. What the twenty-fifth judge found on the item library, as it wrote it (2026-11-16, night) |
| 3807 | Call 108. A supplier's price list prices the job where the job has no price (2026-11-16, night) |
| 3866 | Call 109. What failed the look and the words a third time: a plan is of one level; a door in feet and inches; no "(IFC)" on a button (2026-11-16, night) |
| 3922 | Call 110. From the twenty-sixth judge's notes: a typed length to the cent, a comma in a line's name, a markup's source (2026-11-16, late in the night) |
| 3955 | Call 111. The item library is kept by hand in the window: an item added, renamed, taken out (2026-11-16, late in the night) |
| 3988 | Call 112. A wall typed north is north: what the finer place showed (2026-11-17) |
| 4014 | Call 113. Patrick, 2026-11-17: what comes in and goes out, and other agents bringing a trade's knowledge, prices and specs (and how it was read) |
| 4047 | Call 114. Patrick, 2026-11-17: "Allowances are allowed and noted as such" (QUESTIONS.md Q26, ruled) |
| 4077 | Call 115. Patrick, 2026-11-17: "rebrand to BuildOS" (and how far it was taken today) |
| 4106 | Call 116. Patrick, 2026-11-17: what the lead holds and what is handed off (and a correction) |
| 4127 | Call 117. Looked at as someone who has to work it, at a desk and on site; the estimate read by item (2026-11-17) |
| 4192 | Call 118. The job's money on the Job panel; "not coded" said quietly; a door's swing is not made up (2026-11-17; the rest of call 117's second, fifth and sixth points) |
| 4217 | Call 119. A dimension is written in feet and inches; a quantity keeps its decimals (2026-11-17; call 117's fourth point, and the words judges' "lengths written four ways in one panel") |
| 4239 | Call 120. The estimate by item at the command line; an item's quantity written to the places it needs (2026-11-17) |
| 4255 | Call 121. What the pictures of the full run showed, read by a worker; a drag begins where the button goes down (2026-11-17) |
| 4278 | Call 122. Words and small things from the pictures read (2026-11-17; `research/pictures-read-5be55e3.md` 2, 10, 12, 17, 20, 23, 29) |
| 4307 | Call 123. An allowance is on its own row wherever the job cost is read by division, and its dollars ride on the field list (2026-11-17; the twenty-seventh judge's notes, J27A-5, -10, -11, -15, read while it was still at work) |
| 4333 | Call 124. Seven stops and silences from the veteran judge's notes (2026-11-17; J27A-9, -12, -13, -17, -19, -20, read while it was still at work) |
| 4349 | Call 125. What a line is priced by is changed on the line (2026-11-17; J27A-16) |
| 4364 | Call 126. An allowance that stops being carried is said, to the cent; Undo brings the roof back to the window; an item nobody has set down is not on the field list (2026-11-17; the twenty-seventh judgment, J27A-28, -22, -27) |
| 4383 | Call 127. The far ground fades into the sky without a line (2026-11-17; J27B-1, VISUAL.md C2) |
| 4401 | Call 128. Patrick, 2026-11-17: "It's approved for ply import" (PROPOSALS.md 32, the first piece) |
| 4468 | Call 129. Patrick, 2026-11-18: the program stays in this repository; the scope is every trade and the design professions; a second builder, Opus 5.5, is approved for named work |
| 4522 | Call 130. A formula can come out in cubic yards; a wall and a floor say how thick they are, and a floor its perimeter (2026-11-18; the second builder, for foundation work, call 129) |
| 4555 | Call 131. Each thing in the IFC model carries its base quantities, from the figures the estimate is priced on (2026-11-18; the second builder; the review's IFC finding) |
| 4582 | Call 132. The starter library takes off a foundation: a strip footing, a poured foundation wall, and a foundation drain (2026-11-18; the second builder; call 129's first trade) |
| 4628 | Call 133. A formula works from a figure as it was worked out, not as it is written (2026-11-18; the second builder; found taking off the foundation) |
| 4654 | Call 134. A roof has figures of its own, worked from its planes and lines, so a roof is priced as a roofer prices it: whole (2026-11-18; the second builder; roofing, call 129) |
| 4690 | Call 135. The starter library prices a shingle roof whole: an assembly that goes on the roof (2026-11-18; the second builder; the roofers' judgment of the review) |
| 4734 | Call 136. A roof priced whole and plane by plane is said to be counted twice (2026-11-18; the second builder; the judge of the whole roof) |
| 4749 | Call 137. A wall's frame is counted, and the starter walls buy their lumber by the piece and the foot (2026-11-18; the second builder; the framer's judgment of the review) |
| 4805 | Call 138. roofnerd's core comes across whole, as a crate, its tests with it (2026-11-19; the second builder; call 129's "learned specifics … ported from roof nerd", step 1 of `research/roofnerd-port.md`) |
| 4826 | Call 139. A low-slope area holds its drains, and its tapered insulation is laid out by roofnerd's solver into figures a line can price (2026-11-19; the second builder; step 1 of `research/roofnerd-port.md`) |
| 4898 | Call 140. A taper's layout does not depend on the order its drains are typed in; what it rests on is said where a person reads it (2026-11-19; the second builder; the judge of call 139) |
| 4941 | Call 141. A roof priced whole dresses its planes (2026-11-19; the second builder; the pictures of the shingle roof's window run) |
| 4951 | Call 142. What a taper rests on is said where its money is; the field list asks about the drains; a roof is laid out the same however it is traced (2026-11-19; the second builder; the second judge of the taper) |
| 4985 | Call 143. A partition's board and paint are bought for both its faces, and gypsum board carries its waste (2026-11-19; the second builder; Division 9, from the review) |
| 5009 | Call 144. A room has the area of its walls' board, as a drywall estimator takes it off (2026-11-19; the second builder; Division 9) |
| 5029 | Call 145. A slab on grade is bought by its own thickness (2026-11-19; the second builder; Division 3) |
| 5056 | Call 146. A room's wall area takes in what stands inside it; it is refused where an opening does not fit; board priced by the room and by its walls is said to be counted twice (2026-11-19; the second builder; the judge of calls 143 to 145) |
| 5080 | Call 147. A room has the fewest receptacles its walls need, by NEC 210.52(A) (2026-11-19; the second builder; Division 16) |
| 5105 | Call 148. Existing to remain, to demolish, new: a renovation job (2026-11-21; Patrick's go-ahead on PROPOSALS.md 33: "you own codebase", asked for his own renovation) |
| 5151 | Call 149. The renovation mark in the window: seen, and made with three buttons (2026-11-21; Patrick: "run until we have a working prototype"; call 148's "not yet") |
| 5175 | Call 150. A door or window to demolish is filled in; the mark's edges, from its judge (2026-11-21; the judge of calls 147 and 148) |
| 5203 | Call 151. The tool chest (2026-11-22; Patrick's word) |
| 5216 | Call 152. A room line parts two rooms where no wall stands (2026-11-22; plan step A2; the design and MEP reviews: an open kitchen read "floor area unknown") |
| 5245 | Call 153. The room line drawn in the window; the scan sheet made fit to trace (2026-11-22; plan steps A1 and A2, their window halves) |
| 5265 | Call 154. Two layouts side by side (2026-11-22; plan step A3; the design review's "biggest single gain") |
| 5289 | Call 155. A kitchen set from the tool chest: cabinet boxes by code, in a run along a wall, with its countertop (2026-11-22; plan step A4; call 151; the design and finishes reviews) |
| 5339 | Call 156. The kitchen run and the layouts in the window (2026-11-22; plan steps A3 and A4, their window halves) |
| 5361 | Call 157. A room's run of wall; the kitchen and layouts set right, from their judge (2026-11-22; the judge of calls 152, 154, 155) |
| 5416 | Call 158. Every count and list says what is built, what stays and what comes down, and whose it is (2026-11-22; plan step B5; the GC, MEP and finishes reviews) |
| 5441 | Call 159. Work on part of a thing, typed as a measure; taking a partition down as a scope (2026-11-22; plan step B6; the finishes and GC reviews: "a typed patch of 60 SF is refused"; "this is the whole renovation job") |
| 5477 | Call 161. The five new things done from the screen alone (2026-11-22; the judge of the window's pieces: a scan file "no", a second layout "no", cabinets and a room line "partly") |
| 5520 | Call 163. The scan's message in the window, in the words a builder reads (2026-11-22; the lead's reading of the scan pictures retaken for call 161) |
| 5543 | Call 160. What a wall the work opens may be hiding, asked in the field (2026-11-22; plan step B7; the GC, MEP and structure reviews: "the remodel's money is in what is behind the wall"; "a run through a demolished wall is silent everywhere") |
| 5597 | Call 162. A kitchen's base, its lump lines and its countertop set right (2026-11-22; the judge of call 157: J-C-C, J-C-D, J-C-G) |
| 5642 | Call 164. A beam, its posts and their pad footings (2026-11-22; plan step C8, its first half; `research/structure-port-survey.md`; `research/renovation-structure.md`) |
| 5713 | Call 165. Joists and rafters counted (2026-11-22; plan step C8, its second half; JLC's Lumber field guide: "joists = floor width / spacing + 1"; `research/renovation-structure.md`) |
| 5759 | Call 160, as built: an opening that is taken out and filled in does not open the wall (2026-11-22; the lead, reading the worker's diff) |
| 5770 | Call 166. One cost line taken off a thing (2026-11-22; found accepting call 162: the estimate says "take the line off" and nothing could) |
| 5782 | Call 167. What the judge of calls 159 to 166 found, set right (2026-11-22; `research/judgment-calls-159-166.md`) |
| 5830 | Call 168. A plan set on paper: the plan as it stands, the demolition plan and the proposed plan (2026-11-22; plan step D10; `research/sheets-port-survey.md`; the design and GC reviews) |
| 5884 | Call 169. The window, from what its second judge met (2026-11-22; `research/judgment-call161-window.md`: three of the five things done from the screen, two partly) and the window's half of calls 159 to 167 |
| 5942 | Call 170. A wall moved, and the walls that meet it follow (2026-11-22; the window's second judge: "no way to make the addition longer but to draw it again"; Patrick's job: rooms whose footprints change) |
| 5978 | Call 171. The window's half of calls 164, 168 and 170, and the counts three ways (2026-11-22) |
| 6014 | Call 172. What the program is for, in Patrick's words, and the order of work turned to it (2026-11-22) |
| 6068 | Call 173. Walls, openings and rooms found in a scan (2026-11-22; call 172's first) |
| 6120 | Call 174. Patrick, 2026-11-22: the builder runs on its own; a sweep by everyone who looks at the building (and what is no longer said to him) |
| 6159 | Call 175. What the sweep found wrong in dollars, and what is decided (2026-11-22) |
| 6214 | Call 176. What a wall carries and what it stands on (2026-11-22; the sweep's worst finding) |
| 6258 | Call 177. A trade's view of the building: one switch, the frame drawn, sizes on the work (2026-11-22; the sweep, asked by seven of nine; Patrick: "a 3d view of a building is needed for everyone" and "the toggle on MEPs as a see through") |
| 6326 | Call 178. What the pictures of call 169 showed: the window made clean (2026-11-22; a reader's notes on every retaken picture, `research/pictures-read-call169.md`, and the lead's own reading) |
| 6395 | Call 175, as built (2026-11-22) |
| 6435 | Call 179. What three judges found in calls 164 to 175, and what is decided (2026-11-22) |
| 6565 | Call 180. What the second judge found in the load path, the moved wall and the figures, and what is decided (2026-11-23) |
| 6638 | Call 181. Walls found on a drawn plan (2026-11-23; call 172's second) |
| 6696 | Call 182. What the second judge found on the plan set, and what is decided (2026-11-23) |
| 6742 | Call 183. What the second judge found in the scan's walls, and what is decided (2026-11-23) |
| 6792 | Call 184. The plan for "hyper realistic" (2026-11-23; Patrick: "What is your plan for hyper realistic ?") |
| 6844 | Call 185. The outside trim priced: fascia, soffit, gutters, corner boards, casing (2026-11-23; call 184 step 1's model half; the envelope reviewer of the sweep: "drawn but not priced") |
| 6879 | Call 183, as built (2026-11-23) |
| 6897 | Call 186. What the first judge found in the walls of a drawn plan, and what is decided (2026-11-23) |
| 6947 | Call 185, as built (2026-11-23) |
| 6958 | Call 186, as built, and two of its rules narrowed (2026-11-23) |
| 6981 | Call 187. The window first: what the lead changes after Patrick asked whether there is progress (2026-11-23) |
| 7025 | Call 188. What real files showed, and what is fixed first (2026-11-23) |
| 7067 | Call 188, as built, and call 189: the scan's finder goes to the published method (2026-11-23) |
| 7249 | Call 191. What the judge of the sheet's scale, two drawings and the trim pieces found, and what is decided (2026-11-23) |
| 7399 | Call 192. A wall under a rake rises to the roof: the gable (2026-11-24) |
| 7449 | Call 193. What the judge who drove the window found, and what is decided (2026-11-24) |
| 7605 | Call 194. The sample whole house prices its outside trim (2026-11-24) |
| 7673 | Call 195. Patrick, 2026-11-24: the bench for items, asked a third time, is taken up (and how it was read) |
| 7764 | Call 196. The bench, step B, designed: how an item carries its cost (2026-11-24; the lead's design, not built) |
| 7810 | Call 197. What the second judge who drove the window found, and what is decided (2026-11-24) |
| 7897 | Call 198. The bench, step C, designed: an item's shape as data (2026-11-24; the lead's design, not built) |
| 7943 | Call 199. An assembly changed for another writes the quantities of the one it becomes (2026-11-24) |
| 7969 | Call 200. A model brought in whole, as built, and what its worker held for a ruling (2026-11-24) |
| 8023 | Call 201. An item carries its cost, as built; the rulings on what its worker held; and a check that had never stopped anything (2026-11-24) |
| 8081 | Call 202. The file says what is connected, as built; an item's size as typed is the model's (2026-11-24) |
| 8130 | Call 203. An item's shape as data, as built; what a block may say; what stays the window's (2026-11-24) |
| 8170 | Call 204. A corner board is as tall as its wall is at that corner; and three things a model still leaves out (2026-11-24) |
| 8232 | Call 205. A shape changed by hand: the model's changes, and what the bench will show (2026-11-24; the lead's design, not built) |
| 8292 | Call 206. What the judge of the item's cost, the shapes, a model out and back and the corner boards found, and what is decided (2026-11-25) |
| 8395 | Call 207. A price is typed where it is seen: the price list in the window, and the model's change for it (2026-11-25; the lead's design, not built) |
| 8457 | Call 208. Four things left with two readings, brought to one (2026-11-25) |
| 8498 | Call 209. What the third judge who drove the window found, and what is decided (2026-11-25) |
| 8593 | Call 210. Another trade's model into a job that has a building: only its runs and equipment (2026-11-25; the lead's design; the model's side built, see "as built" below) |
| 8624 | Call 210, the model's side, as built; bringing a model in whole stands as it is (2026-11-25) |
| 8662 | Call 211. Where an item connects: the bench, step D (2026-11-25; the lead's design, not built) |
| 8739 | Call 212. A shape's refusal in a builder's words (2026-11-25; wording only) |
| 8760 | Call 211, the model's side, as built, and what was ruled on the way; call 212 as built (2026-11-25) |
| 8823 | Call 213. The command line judged a second time on 2026-11-25: twenty-two findings ruled on |
| 8909 | Call 214. A maker's model brought in as an item: the bench, step E (2026-11-25; the lead's design, not built) |
| 8967 | Call 215. A cabinet by rule: the bench, step F (2026-11-25; the lead's design, not built) |
| 9016 | Call 213, as built, and what was ruled on the way (2026-11-25) |
| 9064 | Call 216. Patrick, 2026-11-25: stop and review everything against the tools of the trade, the shop, games and the best-made software; clean code that runs on other machines; remote now, a field app later |
| 9132 | Call 216, the gate put right (2026-11-25) |
| 9149 | Call 217. Patrick, 2026-11-25, on the review: the lead makes the calls; two windows stay; no more talk of another machine; what the review was for; run to the finish without him |
| 9208 | Call 218. How the window moves: the wheel from the whole lot to a hinge, tilts, pans and flights (2026-11-25; the lead's design, not built) |
| 9269 | Call 219. How tools feel: the thing shown before the click, sizes at the pointer, a door moved, a copy, one search (2026-11-25; the lead's design, not built) |
| 9327 | Call 220. Patrick, 2026-11-25: the main thing to remember (any common file in and out; they own their database; full depth without overwhelming; clean code, clean screens) |
| 9391 | Call 221. A CAD drawing in and out: DXF (2026-11-25; the lead's design, not built) |
| 9430 | Call 222. Two windows on any screen, work never lost, three ways to stand in front of a job, a plan in the second window (2026-11-25; the lead's design, not built) |
| 9493 | Call 223. "They own their database": every table written out as one workbook and brought back whole (2026-11-25; the lead's design, not built) |
| 9524 | Call 214, as built, and two rulings made on the way (2026-11-25) |
| 9574 | Call 221, as built: a CAD drawing in and out (2026-11-26) |
| 9626 | Call 218, as built: how the window moves (2026-11-26) |
| 9683 | Call 222, part A, as built: any screen, and work never lost (2026-11-26) |
| 9727 | Call 224. What part A turned up, ruled; and part C is built before part B (2026-11-26; the lead's) |
| 9763 | Call 219, the model's side, as built: a change tried and taken back, an opening moved, copies, find (2026-11-26) |
| 9812 | Call 223, as built: the database written out as one workbook and brought back whole (2026-11-26) |
| 9850 | Call 225. What the cabinet rules move, ruled before any expectation moves (2026-11-26; the lead's) |
| 9896 | Call 226. Loose ends at the command line: a try made quick, lengths said as a builder says them, the database's reply, a cabinet rule taken away, heights flagged (2026-11-26; the lead's design, not built) |
| 9925 | Call 215, as built: a cabinet by rule (2026-11-26) |
| 9964 | Call 227. The marks on a marked-up PDF read as quantities; a TIFF taken in as a plan (2026-11-26; the lead's design, not built) |
| 10015 | Call 228. Clashes handed over as an issue list another program opens: BCF (2026-11-26; the lead's design, not built) |
| 10054 | Call 226, as built: loose ends at the command line (2026-11-26) |
| 10087 | Call 229. The house written out as glTF, to be looked at anywhere (2026-11-26; the lead's design, not built) |
| 10117 | Call 230. A scanner's own files taken in: LAS and E57 (2026-11-26; the lead's design, not built) |
| 10141 | Call 231. What reading the marks turned up, ruled (2026-11-26; the lead's) |
| 10177 | Call 227, as built: marks on a PDF read as quantities; a TIFF as a plan (2026-11-26) |
| 10206 | Call 232. Size tags after call 218: three rulings from two tags that went missing (2026-11-26; the lead's) |
| 10246 | Call 228, as built: clashes handed over as an issue list, BCF 2.1 (2026-11-26) |
| 10280 | Call 233. What a house has that the tool chest draws as a plain box: starter shapes and items for appliances and small devices (2026-11-26; the lead's design, not built) |
| 10318 | Call 229, as built: the house written out as glTF (2026-11-26) |
| 10350 | Call 234. The field's measurements brought back by file: a sheet to fill, and the loop closed without retyping (2026-11-26; the lead's design, not built) |
| 10395 | Call 230, as built: a scanner's own files taken in, LAS, LAZ and E57 (2026-11-26) |
| 10428 | Call 222, part C, as built; calls 224 and 232 as built; the north arrow's double-click (2026-11-26) |
| 10460 | Call 235. Patrick, 2026-11-26: "Do you own the codebase ?"; and call 233's held work ruled and taken up |
| 10509 | Call 234, as built: the field's measurements brought back by file; and call 236, what is ruled on it (2026-11-26) |
| 10554 | Call 233, as built: starter shapes and items for appliances and small devices (2026-11-26) |
| 10585 | Call 237. Patrick, 2026-11-26: the repo is not acceptable as it is; clean it, then run; one manual in place of piling pictures |
| 10657 | Call 238. How the records are kept from 2026-11-26 (the lead; call 237, step 4) |
| 10707 | Call 239. Patrick, 2026-11-26: the window is still busy and its tools cramped; design review goes on the list and is run |
| 10767 | Call 240. The design judgment ruled: the numbers for quiet and room, and the window laid out again in two parts (2026-11-26) |
| 10845 | Call 241. Two window demonstrations failed in the full run on their own stale expectations; each is put right, with its reason (2026-11-27) |
| 10883 | Call 242. Part B of the layout, and every kind of file through one list: what a file is, where the reading lives, how a moved control is still found (2026-11-27; the lead's design, not built) |
