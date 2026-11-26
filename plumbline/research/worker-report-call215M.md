# W-215M report (a cabinet by rule), notes as I go

Scratch folder of mine: SCRATCH/w215M.e0fU

## Before any code

### 1. BIMCraft (/home/user/ipedrax/bimcraft, pinned f202abd8...)
- `crates/core/src/family.rs`: a Family has `parameters` (Parameter: name, kind, instance, value, formula), `types`
  (BTreeMap type name -> BTreeMap parameter name -> Value: only the parameters the type changes) and `forms`.
  `Family::values(type, instance)` -> `values_in` -> `resolve(name,...)`: a parameter's value is the formula worked
  (if it has one) else the instance's value (if the parameter is an instance one) else the type's value else the
  parameter's own default. So a size comes from NAMED PARAMETERS set per type, never from a name.
  The starter "Base Cabinet" (family.rs:2115): parameters Width 1800, Depth 600, Height 900 (mm) plus two materials;
  types "1800 x 600" {}, "1200 x 600" {Width:1200}, "600 x 600" {Width:600}. The type's NAME ("1200 x 600") is only a
  label: nothing reads it (grep for split(' x ') / split_once(" x ") / parse over family.rs, import.rs: nothing).
- Ported: nothing new. Our rule table is OURS: a code that carries its size in its name and a rule that reads it
  (BIMCraft has the types-of-a-family idea: a width per type; ours is a width read from the code, on demand).
  New public items go in PORTMAP.md as "ours" with a grep over BIMCraft that finds nothing; NOTICE untouched.

### 2. The table of rules, and the 27 (not 35) starter codes
FINDING: the contract and call 215 say "thirty-five cabinet rows"; the starter chest (library/items.csv, 48 items)
has **27** box codes (B x9, SB x3, DB x2, BLB, LS, W x6 by 30 high, W3018, W3612, T x2, F3 = 27). `box_codes` returns 27
and the existing tests assert 27 (world.rs:6644, call157.rs:155,276). The other 21 rows are not codes (Range,
Refrigerator, Base cabinet 36 in, Vanity ...). So 35 is the lead's figure and is wrong; 27 is right.
Headings (a builder's words): Code | What it is | Number reads | Width | Depth | Height | Stands | Drawn as |
Stocked widths | How sure | Where from
Starter rows (8): see library/cabinet-rules.csv (written below). Run through by hand: all 27 come out as their row.

### 3. Baseline (before any change), the build that is there
Script scratch/w215M.e0fU/baseline.sh (new job, ground level, 24x16 closed wall, Kitchen, starter items,
`cabinets ... --boxes "B15 B30 SB36" --countertop` along the east wall from 1 ft).
Saved job: scratch/w215M.e0fU/bl/base.json  sha256 8e62faf3f50663c1b91fdc3066ff946427a529e3cfb9fb112a255c15aa6a5e16
(run twice: same bytes.)

### 4. Where the program asks "is it a wall box" or reads a cabinet's name
Model: editing.rs `box_codes` (1144), `box_list` (1161), `cabinets` (1203), `starts_with('W')` at 1242 (the only place a
letter decides anything); the refusal "X is not in the tool chest. It has: ..." (1227); view.rs/sample.rs only say the
word Cabinets as a line name. building/planset.rs:68 says "wall box" in a comment about what hangs above the cut (it
reads each thing's own height/offset, not its name). cli main.rs:3732-3760 calls `ed::box_list` and `ed::cabinets` and
reads `made.boxes/hung/countertop/run/from/left`.
Window's crate (not mine): tools.rs:1462 `ed::box_list`, :1472 `ed::cabinets(&mut job,&storey,&wall,&room,&from,&codes,top,how)`
reading made.boxes/run/from/left/hung/countertop; it splits a refusal on ". It has: " (tools.rs ~1491); model.rs:903
`editing::box_codes(&mut job)` -> Vec<(String,String)>.
How I keep them: `box_list`, `box_codes` and `cabinets` keep their arguments and return types; `Cabinets` gains one
field (`made`, a list of sentences) which the window may ignore (it never builds a Cabinets); the refusal keeps the
". It has: " shape (what a rule reads goes before it).
EXISTING EXPECTATIONS THAT CALL 215 MOVES (to hand back, not edited): world.rs:6650 and demos/cabinets.sh:100 expect
B99 to be refused "not in the tool chest. It has:"; by points 3/4 B99 is a base 99 in wide, made and flagged as not
stocked. Also demos/cabinetsw.sh:146 (window; I cannot run it).
Baseline note: the job's header carries today's date, so the demonstration compares the job with its "timestamp" line
set to "-": sha256 of base.json so normalised is recorded in demos/cabinet-rules.sh (value below once written).

## Red run 1 (tests written first: crates/world/tests/call215m.rs; wiring of the table, job key and database in, `cabinets` not yet changed)
First attempt did not compile (E0609: no field `made` on type `Cabinets`, 12 errors). With the field added empty and
`cabinets` unchanged: 11 tests, 2 passed (the ones of the table alone / starter), 9 FAILED, for instance
`W3342 is not in the tool chest. It has: B12 B15 ... F3` (a_wall_box_and_a_tall_box_are_read_from_their_numbers) and
`B42 is not in the tool chest. It has: ...` (a_code_the_chest_lacks_is_made_by_its_rule, made_boxes_..., the_rules_are_...).
Correction to red run 1: 11 tests, 2 passed (a_job_that_never_touched..., every_starter_box_is_what_its_rule_makes: the
latter shows the 27 are exactly their rules, no STOP), 9 FAILED.
## Green run 1: call215m 11/11 after `cabinets` reads the rules (Cabinets.made, per-box hang from rule, ask/make/refusals).
## Run 2: whole model crate, `cargo test -p plumbline-world --no-fail-fast`: 706 passed, 5 failed (700 before + 11 new = 711)
Existing expectations that fail BY THE DESIGN (none edited; handed back):
 - crates/world/tests/world.rs:6649-6650 (a_run_of_cabinet_boxes_stands_along_a_wall_to_the_figures_worked_by_hand):
   `ed::cabinets(... ["B99"] ...).unwrap_err()` expected a refusal "B99 is not in the tool chest. It has: ..."; got Ok: B99 is now
   made by the rule for B (99 in wide, 24 deep, 34 1/2 high, not a stocked width). (Checked with B99 -> Q99 and the sentence
   loosened in a throwaway edit, reverted: the REST of that test passes untouched.)
 - crates/world/tests/call223m.rs:68 (point_1), :85 (points_2_and_3), :128 (point_2), :257 (a_part_of_a_database...): they name
   the FIVE tabs; the database now has six ("Cabinet codes" after Items): got ["Shapes","Items","Cabinet codes","Connections",
   "Assemblies","Prices"] where five were expected; at :257 the tabs said absent are ["Shapes","Cabinet codes","Connections",
   "Assemblies","Prices"] where four were expected.  The starter-database test (:270) PASSES: I left the cabinet codes out of the
   starter database (the program's own rules are always in force; a `starter` bring-in would otherwise write the job's changed rules back).
## Run 3: crates
 plumbline-world 706+5 (711 = 700 + 11 new; 5 fail by design, listed above); plumbline (cli) 62 passed + 2 failed (64 as before);
 sheets 7; ifc 111; roofing 122; plans 187. Failing in the cli crate: crates/cli/tests/call223m.rs:63 (expects "a tab for each
 table: Shapes 50, Items 49, Connections 5, Assemblies 202, Prices 8"? got "...Items 49, Cabinet codes 8, Connections 5, ...") and :88
 (expects "(a workbook, 5 tabs): brought in whole", got "my.xlsx (a workbook, 6 tabs): brought in whole").
## Run 4: demonstrations (command line), idle class, PLUMBLINE_BIN set
 cabinet-rules.sh passes (exit 0, 0 FAIL lines); against the OLD program (baseline binary) it exits 1 at step 1
 ("B12 is not in the tool chest, and it has no boxes: bring the starter items in first").
 Hand-backs: demos/cabinets.sh:100 expects "B99 is not in the tool chest. It has: "; got "plumbline: B99 would stand in B18
 (furnishing-0062) by 18 in: set the run clear of it..." (B99 is made now, 99 in wide, and the run stands in B18).
 demos/database.sh:71 and :80 expect "...: Shapes 50, Items 49, Connections 5, Assemblies 202, Prices 8", got "...Items 49, Cabinet codes 8,
 Connections 5...". With the two lines and the two counts ("(a workbook, 5 tabs)" -> 6, "(a folder, 5 files)" -> 6) set right in a
 throwaway copy (deleted), the rest of database.sh passes: nothing else of it moves.
## Run 5 (final)
 Demonstrations: 60 existing command-line demonstrations run, 58 pass with no FAIL (cabinets.sh and database.sh are the two
 handed back); cabinet-rules.sh passes (59 passing of 61 in all). Gates: language, egress, clientdata --all PASS; licenses.sh exit 0
 (no new crate); portmap PASS (472 public items, 328 ours). `plumbline sixteen`: sha256 880d9fd3c364b5f7..., byte for byte the old
 program's; estimate, vif, clashes, items of it unchanged.
 Patch: review/call215M.patch (14 files).

## Round 2 (call 225)
Done: B99->Q99 in world.rs and demos/cabinets.sh (comments added); the six-tab moves listed (call223m.rs world 68/85/128/257, cli 63/88, database.sh 71/80/91/104); 96 in width cap (Rule::number, WIDEST, test no_box_is_made_wider_than_96_in red first: panicked at call215m.rs:382, then green; PORTMAP row).
My own test that used B99 for a past-the-corner run now uses B96.
Not done, STOPPED per rule "no other figure moves": call223m.rs world:72-74 (all[2..4] index the connections/assemblies/prices tables, now one place on: all[2]=cabinet codes) and the row-count list [50,49,5,202,8] -> [50,49,8,5,202,8]; cli call223m.rs:66 (tab names list five), ~:102 "(a folder, 5 files)", and TABLES at :33 (five names; the cabinet tab's verb is cabinet-rules but its file is "cabinet codes.csv"). In a throwaway copy the world edits made all 14 tests pass; cli not tried (TABLES needs a verb/file pair).
Round 2, part 2 (ruling): world call223m point_1 (positions, counts [50,49,8,5,202,8]) and cli call223m (six-tab list, TABLES with cabinet-rules + folder_file helper, "(a folder, 6 files)") moved. All six crates pass: world 712, cli 64, sheets 7, ifc 111, roofing 122, plans 187, none failing. database.sh, cabinet-rules.sh, cabinets.sh pass; gates pass; patch rewritten.
