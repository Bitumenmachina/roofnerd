# W-191 notes
started; first build running (build0.log)

## Reproduction before (main-tree binary, judge's sheets; w191/repro-before.txt)
A1: nested.pdf: --plan 1 --lay => takeoff "~ walls 7  360.00 LF" (listing said "taking drawing 1 of the sheet's 2: 0 lines outside it are left out")
A2: three.pdf: "wall 1: from 0' 0" east ... to 73' 0" east, 73' long; door or open doorway: 30' 3" to 44' 9" from its west end, 14' 6" wide"
A4: section sheet (plan 92 LF + section): the small thing set apart in doubt, plan found (as call 186 has it) -- still the rule for under half.
B6: wrong1.pdf, Known 1/4 scale then --scale-by 1: "scaled by string 1, 30'-0": 1/8" = 1'-0", Assumed ... walls found 60' x 40', 1' 0" thick"; show: "~ scale dimension 30 ft"
B8: laid 4 walls, 100.00 LF; --scale-by 1; takeoff still "~ walls 4 100.00 LF"; show: nothing about "rescaled"
B9: zero.pdf: "string 1: 0'-0" ... inf to the foot" / "only one string: it gives inf to the foot"
## Plans-crate red runs (original drawn.rs + only the new struct fields so the tests compile)
a_plan_taken_by_its_number...: left 7 right 3 (turned 0)
two_houses_in_line...: "no wall runs across the gap"
a_gap_wider...: "no wall across the bare gap, two walls on the line: [50.0]"
walls_that_do_not_meet...: gap 9: left 0 right 2
where_the_wall_found_outside...: "turned 0: not unsettled"
a_box_that_cuts...: left 0 right 2 ("the south and the north wall are cut: []")
a_string_of_no_length...: left 2 right 1 (the 0'-0" string was read as a dimension)
## Plans-crate green with the fix: all 8 new pass; the 157 old ones pass unchanged (run before SHARE_EPS 1e-6 added).

## World-crate red runs (original tree stashed; only the two tests that need no new API)
a_fascia_line_priced_on_another_figure...: priced [(Fascia, Some(62.0)) ...] -- the estimate prices 88 (assert wants 88); original left_out says nothing of eave-0013
walls_laid_from_a_sheet_are_named_when_its_scale_changes: "wall-0005 not named: []"
C11 / C12 before (CLI, original binary w191/orig/plumbline; w191/c-before.txt): "soffit: 0 pieces come to 0 SF; the lines price 0 SF"; fascia "3 pieces come to 62 LF; the lines price 62 LF" and no Left out.
C after (w191/c-after.txt): "fascia: 3 pieces come to 62.00 LF; the lines price 88.00 LF (they differ...): 26.00 LF of what the lines price is not drawn" + Left out names eave 1; soffit "no figure"; "2.25 SF is corner counted in two runs (4 corners)".

## Real sheets (scratch only), A1-A4 + the rest built, orig = program at d575489, new = this tree
boise_c: orig == new byte for byte (find-walls): 23 walls, 3 in doubt; --lay: 18 walls, 131.47 LF (both)
boise_d: orig == new: 20 walls, 12 in doubt; --lay: 8 walls, 120.14 LF (both)
lh499 (floor plan + roof plan): orig == new: "2 drawings on this sheet: about 149 ft 10 in (149 ft 10 in of it in doubt) and about 129 ft 2 in (38 ft of it in doubt)"; --lay refused: "nothing is laid: 2 drawings on this sheet, and a roof plan's lines are not walls..." (lays nothing unasked, as before)
(other pdfs in real/pdf not run: only the three jobs in jobs188 exist)
Existing planwalls.sh steps (1-9) and trim.sh steps 1-7 pass unchanged under A2/A3/A4 (planwalls run1: exit 0). trim.sh expectation strings in steps 6 changed from 4-place to 2-place by item 13 (flagged to the lead).

## Final (23:55)
tests: plans 160 lib + 4 + 7 = 171 pass (7 new); world 488 pass over 29 binaries (4 new, call191.rs); cli 21 pass. demos/planwalls.sh PASS, demos/trim.sh PASS (final runs on target/debug built with CARGO_PROFILE_DEV_DEBUG=0 for lack of disk). gates: language, egress, clientdata --all, licenses.sh PASS; portmap.sh not run (BIMCraft not on this machine).
Patch: review/call191.patch (10 files). Expectation changed: demos/trim.sh step 6, 8 lines 4-place -> 2-place by point 13 (flagged).
Disk fell to 335 MB free once (others' trees); I removed my own binaries/old debug artifacts.
update: points 14, 15, trim.sh comment done; 681 tests pass; planwalls/trim/vif/carries/hidden/partwork/planset/sweep-money demos pass
