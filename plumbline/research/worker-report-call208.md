# W-208 report (contract 208, worker on Sonnet 5.5), written as I go

## Point 1, BEFORE any change (list made 2026-11-25, build at 51f56da, nothing edited yet)

### The three cases, smallest job each (the job: one ground level, one 20 ft x 8 ft wall, material "Siding (made up)" bought by SF, one line "Siding (made up)" = face_area 160 SF made of it; all numbers made up; commands in review/w208/mk.sh)
(a) A supplier's price on a material the job has no price for.
  older road: `set material-0005 quote '$5.00/SF' --supplier "Supply One"` stores the quote alone and says "set material-0005 quote from Supply One = $5.00/SF (Assumed)";
              estimator price stays "no price entered"; the line has no figure; job cost "none".
  list road : (`import prices ... --supplier "Supply One"`): "which Supply One had not quoted; the job had no price for it and takes Supply One's";
              estimator price becomes $5.00/SF (Assumed, "Supply One's price list: ..."); line 160 SF x $5.00 = $800.00; job cost $800.00.
(b) A supplier's name written another way ("supply  one" for "Supply One", which the job has).
  older road: refuses: `"supply  one" is "Supply One" written another way; a supplier has one name in a job, so use "Supply One"`, job unchanged.
  list road : takes the job's spelling: Supply One's $5.00 becomes $4.00/SF (a second supplier is NOT added).
(c) An unpriced cost line of the same name, not made of the material (a second wall with a hand-made line "Siding (made up)").
  older road: `set material-0005 price '$6.00/SF'` prices the made-of line only (160 SF x $6.00 = $960.00); the second line stays "no price"; job cost $960.00, 1 line left out.
  list road : `"Siding (made up)" on wall 2: priced at $6.00/SF, the list's`; both lines 160 x 6.00 = $960.00; job cost $1,920.00.

### Every test and demonstration that sets a material's price or a supplier's quote by the older road
Found by the contract's grep, a wider one for set_on_line/set_quote/"price"/"quote" in tests and src, and then MEASURED: I put a temporary line in the command line's `set` price branch writing which case (if any) each call met, and ran `cargo test -p plumbline` and all 54 command-line demonstrations (baseline log: review/caselog.txt; the temporary line is removed again before the end).
Command-line routes (where the three cases decide the road):
 1. crates/cli/tests/cli.rs:123 `set material-0005 price $6.25/SF` (Siding, line added AFTER, of-material): case none. No figure moves.
 2. crates/cli/tests/cli.rs:311 `set <Gyp board> price 12`: line "Board" is not named "Gyp board": none. No figure moves ($48.00).
 3. demos/d5.sh:202 `set MAT quote $9.40/SF --supplier "Tinners Yard"` (refused, unit): the measure shows case none; reply unchanged by this point; no figure.
 4. demos/pricelist.sh 55-174 (HEATER an item: always by the list's road), 127/131 `Sheathing`, 140 `Zinc sheet`, 198 `$MAT price $20.00`: measured, case none; nothing moves.
 5. demos/pricelist.sh:203 `set $LEAD quote '$30.00' --supplier "Northside Supply"` (step 13): case (a) MET, the only one in the whole command line corpus.
    Now: the quote alone; says "set material-0099 quote from Northside Supply = $30.00 (Assumed)".
    Will be: the job's own price for "Lead sheet (made up)" is also $30.00/sheet (Assumed) and Northside Supply's; reply is the list road's. Money: no cost line is made of or named for the lead sheet in that job (it is a bare material), so 0 lines priced and the job cost stays $3,820.00 - NO figure of money moves; only the reply's words (the demonstration's expectation line 204 changes to the list road's wording, which I will read from the run).
No other `set`/`quote` call in the command-line demonstrations or the command line's tests reaches the price branch (judged-1125.sh, itemcost.sh, imports.sh, bringw.sh use `import prices`, the list's own road).
Window demonstrations (they mention xvfb/open_window; not run, not in my files): grep shows none sets a price by `set`.
Library routes (not reached by the command line; no case decides them): crates/world/tests/world.rs:4057,4061 (`Job::set` price), 4089-4099 (`Job::set_quote`), 4928-4929 (`Job::set_quote` refuses another spelling: this IS case (b) at the library level). They call Job functions that stay as they are; no figure moves. HELD for the lead: `Job::set_quote` remains a second road in the library that only the "not a material" refusal of the command line still reaches; I do not delete or weaken its tests.

Totals: 5 distinct places in tests/demonstrations set a price by `set`/`quote` on a material (cli.rs x2, d5.sh x1, pricelist.sh step 13 x2 lines plus the pricelist.sh HEATER/SHEATH/ZINC lines counted as the list's own road); 1 meets one of the three cases (case a, pricelist.sh:203); 0 money figures move.

### Point 1 as done
Red run (before the change, `cargo test -p plumbline --test call208`, 3 failed):
 (a) panicked at call208.rs:51: `set material-0005 quote from Supply One = $5.00/SF (Assumed)` (the reply lacked "the job had no price for it and takes Supply One's")
 (b) panicked at call208.rs:66: `plumbline: "supply  one" is "Supply One" written another way; a supplier has one name in a job, so use "Supply One"`
 (c) panicked at call208.rs:85: `set material-0005 price = $6.00/SF (Assumed)` + "figures worked out again on: wall-0004" (the line on wall 2 not named)
Green after: 3 passed. Change: `stores_more_than_a_plain_price` removed (pricelist.rs, cli main.rs); the spelling case now says which (a note in `set_price`, because the list road's own reply did not say it: `"northside  supply" is "Northside Supply" written another way ...`); crates/cli/tests/call208.rs is NEW (a new test file beside the cli's others; the contract's file list names only existing places for tests: said here).
Test moved: crates/world/tests/call207m.rs `where_the_price_list_would_store_more_than_a_plain_price_it_says_so` lost its first half (calls to the removed function) and keeps the second (plain price and list store the same), renamed `a_plain_price_and_the_price_list_store_the_same`. No figure.
Moved expectation: demos/pricelist.sh step 13 only the words (the quote's reply) - arithmetic: no line is made of the lead sheet, job cost $3,820.00 before and after; new steps 14 (b) and 15 (c): 4 x $12.00 = $48.00, $3,820.00 + $48.00 = $3,868.00 (run: matches).
Money moved in tests/demos beyond the list: none found yet (to be confirmed by the full runs below).
Not right: in (c) the reply says "Pitch pan (made up) prices nothing on the job yet: no line is made of it" in the same breath as pricing a line named for it (the count is of lines made of it). HELD for the lead's ruling (wording of PriceSet::said when `done` names lines priced by name).
Held: `Job::set_quote` (job.rs) is still the library's older road for quotes (its refusal of another spelling is tested at world.rs:4928-4929); nothing a person can reach uses it for a material any more. Not touched.

## Point 2 ("priced on nothing" worked in one place)
Finding on the build: `view::pricing` still carried its own inline copy of the rule (no allowance, no line, not existing) beside `view::nothing_priced_on` that the level's count uses. Red: call205b.rs now asks the one function (`nothing_priced_on`, made `pub`) of each thing set down and in a case-by-case test: `error[E0603]: function nothing_priced_on is private` (the test of two copies, `all == d.not_priced()`, is kept AND now asks the one function). Fix: `pricing` calls `nothing_priced_on`; the only remaining inline test is "has no allowance and no lines" for existing-to-remain, which is the same function's other half. 7 tests in call205b pass; no figure moved.

## Point 3 (`check` of a shape under a starter shape's name)
Shown on the build: a job with a shape renamed "Bathtub" (as a pre-call-205 file could hold) -> `check` said only "reads, its figures agree..." exit 0. Red: world test compile error (no `shapes_under_starter_names`). Fix: `editing::shapes::shapes_under_starter_names`; `check` prints `! "Bathtub" is a starter shape's name, and this job holds a shape of its own under it, which is drawn ahead of the starter's: keep it under another name (...)`. PORTMAP.md line added (ours; portmap PASS).

## Point 4 (every length a line can enter is refused below nothing)
List of the kinds of length a line enters (measured on the build, review/w208/p4.sh): (1) stretch-out: refused already, as "must be more than zero" (now as "stretch out is below nothing (-19 in)" when negative; zero keeps the old words); (2) a height (name has "height", wainscot, or a formula's `..._below_x`/`..._above_x`): refused since call 206; (3) ANY OTHER named value typed with a length unit, e.g. spacing, turn_up, lap in inches: ACCEPTED below nothing (`set wall-0004 spacing = -12 in (Known)`; `--with spacing=-16 in`) = the finding; (4) a measure typed outright ("-60 LF", "-60 SF"): refused already ("below zero; a measure is zero or more"); (5) a plain number input (lap=-2): not a length, taken as before; (6) waste: percent, refused already.
Red: `["spacing", "-12 in", "--line", "Studs"]: set wall-0004 spacing = -12 in (Assumed)` (call208.rs:137). Fix: job.rs `below_nothing` refuses any value with a length unit below nothing; reply `spacing is below nothing (-12 in): a length a line enters is nothing or more` (a height keeps "a height ...").
Expectation moved (NOT a money figure, and not on point 1's list; done under point 4, FLAGGED for the lead): crates/world/tests/world.rs `a_count_below_zero_is_refused_not_rounded_up_to_nothing` entered spacing -12 in on purpose to see the quantity refuse -40; the entry is now refused first, so the first block asserts that refusal; the quantity's own "below zero" refusal is still held by the formula block that follows (a gap of 17 ft). No figure.
Also the cli `usage` text no longer says a material by its path "keeps the older road".

## What was run (after the change, in wtB, command-line crates only, one at a time)
cargo test: plumbline-world 626 passed 0 failed; plumbline 27 (cli: 3 new in call208.rs) passed; plans 172; sheets 6; ifc 53; roofing 122; 0 failed anywhere.
54 command-line demonstrations (every demos/*.sh without plumbline-app/open_window/xvfb/xdotool, except window.sh and field-job.sh), PLUMBLINE_BIN set: all exit 0 (judged-1125 exited 1 once on MY first step 16 - a job file name I reused - fixed, then exit 0); words searched for FAIL and FAILED: none. Compared with the baseline run before the change (temp names normalised): only pricelist.out and judged-1125.out differ, both the new steps/words; no other demo's output moved, so no other figure moved.
Gates: gate-language PASS, gate-egress PASS, gate-clientdata --all PASS; tools/licenses.sh exit 0; BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh PASS.
Patch: review/call208.patch. Nothing committed or pushed. A permission was refused once (a python edit combined with a run of demos/pricelist.sh): I did that edit with the Edit tool and ran the demo separately; nothing else needed it.

## Second round (the lead's rulings)
Ruled as done: the moved test `a_count_below_zero_is_refused_not_rounded_up_to_nothing` already carries a comment beside it saying call 208 point 4 moved it and that the quantity's own refusal is still held by the formula block after it.
1. Case (c) reply: `PriceSet` gained `named_lines`; `said()` now says "no line is made of X, and N lines of that name are priced by it: <names>" and "prices nothing on the job yet" only when nothing at all is priced. Red: cli test `a_price_for_a_name_no_line_is_made_of_but_lines_are_named_says_which_it_prices` failed on the old words (panicked at call208.rs:161, the reply listed the two lines and said it priced nothing); green after. Demo pricelist.sh step 15 asserts the new words and that the old are absent. No money moved.
2. `Job::set_quote`: callers were the command line's last branch (reachable only for a thing that is neither material nor item, which it refused with "not a material") and tests in world.rs. Removed the function. The CLI branch now refuses in builder's words: "<x> is neither a material nor an item: a supplier quotes a material or an item". world.rs: the four calls in the Tinners Yard test now go through `pricelist::set_price` (same figures held: 8,159.63 and 6,714.91; the two refusals' words are the list road's: "priced by the SF", "neither a material nor an item"). Deleted the two assertions of item 12 of `j11_10_to_14_the_smaller_findings` ("harbor metals" and "Ridge  Supply" refused with "written another way"): call 208 point 1(b) is why - the decision replaces that refusal with taking the job's spelling and saying so, held in crates/cli/tests/call208.rs `a_suppliers_name_written_another_way_is_taken_as_the_jobs_and_said`. No money figure moved.
Run: world 626, cli 28, plans 172, sheets 6, ifc 53, roofing 122, 0 failed. 54 command-line demos with PLUMBLINE_BIN: all exit 0, no FAIL/FAILED; only pricelist.out differs from the first round (the new step 15 lines). Gates, licenses, portmap PASS. Patch rewritten: review/call208.patch.
