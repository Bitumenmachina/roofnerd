# W-207M report (contract 207M; second working copy wtB at 64b5800)

## Before any code

1. BIMCraft carries no money. One grep over its Rust source for price|\$/|unit_cost|supplier found only a
   test that renames a user parameter "Cost" to "Price" (crates/core/src/tests.rs:1041) and one "Supplier"
   parameter in import.rs tests. Nothing to port. PORTMAP.md gets its rows for new public items; portmap to be run.

2. Every way a price reaches a material or an item today (read, then run on a job with "Water heater, 50 gal" twice, priced as "Bought and set"):
   - `import prices` (`tables::prices_in`): by name (plain-name match) over materials AND items; an item with no
     price yet gets a Supply on its first price (that is the only road that gives an item a Supply other than the
     chest's Unit cost column); supplier by --supplier or a column; the estimator's price or one supplier's;
     status/source from the list or from --status/--source; clears (empty price + Unknown); notes $0.00; a supplier's
     price becomes the job's own where the job has none (call 108); says "prices nothing on the job yet" for an entry
     nothing is made of; lines named as the item (not made of it) take the price; all or nothing. It does not:
     say how many lines/things it priced as a whole, nor the job cost before and after; it takes a FILE.
   - the chest's table (`items_in`, "Unit cost" columns, tables.rs ~660-680, 1165): sets an item's own price at
     bringing-in (and the item's Supply). Not for a typed single price, not for materials, no suppliers.
   - `set <material-PATH> price|order` (`Job::set_on_line` "price") and `set <material-PATH> quote --supplier`
     (`Job::set_quote`): by PATH only, on a material only; both fail on an item ("not a material" / "no element <name>"),
     since an item with no price has no Supply; a name is "no element"; quote refuses a supplier written another way;
     reply is one line, no lines/things/job cost. A supplier's quote does NOT become the job's price (differs from prices_in).
   - `price-item` (`editing::price_item`): a unit cost on the cost line NAMED for an item, on each thing set down. It
     refuses lines made of the item ("its price is the item's ... give it in a price list").
   - `set <line> unit_cost --line`: a line's own price; nothing to do with the list.
   What none does: set one price by NAME for a material or an item, typed, with its follow-through said. None tells a
   cleared price from a never-set one (see below).
   Can one be the change? `prices_in` is the one road already (items, suppliers, clearing, $0.00, 108, all-or-nothing),
   but it takes a table. Smallest new change: `pricelist::set_price`, which builds ONE ROW and calls `prices_in` itself
   (so the same road, literally, no second arithmetic), then reads the estimate before and after for the words. It touches
   `tables.rs` not at all.
   Limits found by running (the lead's design said otherwise or was silent):
   - Suppliers: the model has no limit of three. `view::estimate().suppliers` is every supplier that quoted any material
     (sorted by name); the sheet's export writes all. "Three side by side" is the window's, not the model's. The rows give
     every supplier; the window picks its three. (Tested: four.)
   - A price CLEARED vs NEVER SET: both are `Val{value:None, status:Unknown}`; only the source differs ("no price
     entered" at birth; whatever was typed on clearing). So the model can say "no price" for both, and cannot say
     "was cleared". The smallest true thing: a cleared price is no price, which is what call 207 point 2 asks; the row
     says "no price" and keeps the source the clearing gave. Not distinguished on purpose.
   - `$0.00` IS distinguishable (value Some(0.0)): rows carry `at_nothing`.
   - An item nobody has priced has NO Supply, so it is in `prices_out`/`export prices` only once priced. The rows are
     built from the lines (made of the item) as well, so such an item is listed with no price.
   - An item's unit: with no Supply and no unit typed, an item is priced by EA (a thing set down is one EA).

3. (rows: see below, filled in after the build.)
4. (verb: see below.)

## Tests first, run red (plumbline/crates/world/tests/call207m.rs, 12 tests, against a stub of the API)
`cargo test -p plumbline-world --test call207m`: **0 passed; 12 failed** (steps 1-8, material nothing is made of,
four suppliers, same price again, rename, import-over). Quoted: `called Result::unwrap() on an Err value: "not built"`.

3. Rows (`pricelist::price_list`, `PriceList{suppliers, rows, not_found}`; row = PriceRow{name, path, item, what, unit, price: PriceCell,
   suppliers: Vec<Option<PriceCell>> aligned to PriceList.suppliers (every supplier of the job; the window shows its three), lines, lines_off,
   waiting, things}; PriceCell{value, status, source, at_nothing, riding, riding_status, lines_at_it}). Every figure is read off
   `view::estimate`'s own lines (lines made of the entry, counted; dollars = sum of those lines' own amounts/quotes, to the cent), no
   second arithmetic. Order: no estimator's price first (most lines first), then dollars riding (most first), then name.
   Heater job after step 3 reads: Water heater, 50 gal | Item, Mechanical | EA | estimator ~$1,850.00 Assumed "made-up quote", riding ~$3,700.00 on 2 of 2 lines |
   Northside Supply ~$1,790.00 riding ~$3,580.00 | lines 2, things 2, waiting 0.
4. Verb: extended two, added none. `set <file> <item or material by name> price <v|unknown> [--supplier N] --status --source` (item by name or path, material by
   name; a material by PATH keeps the older `set_on_line` road and its reply untouched so no expectation moves; `quote` too) and `price <file> list`
   (`price` printed the job's price only; "prices" did not exist). Why not `price-item`: that sets a unit cost on the lines NAMED for an item, and refuses lines made of the item.

## Built / run
- crates/world/src/pricelist.rs (new, `pub mod pricelist` in lib.rs), tests crates/world/tests/call207m.rs (15 tests, all pass), crates/cli/src/main.rs (set route, `price <file> list`, usage), demos/pricelist.sh, README row, PORTMAP.md note (portmap passes, 380 items, unchanged: pricelist.rs is outside the building model files it reads).
- Tests, one crate at a time: world 44 runs 623 passed; plumbline(cli) 5 runs 22; sheets 6; ifc 53; plans 172; roofing 122: 0 failed.
- 54 command-line demonstrations run (every demos/*.sh without plumbline-app/open_window/xvfb/xdotool, less window.sh, field-job.sh; includes the new pricelist.sh): all exit 0, searched for FAIL/FAILED: none.
- Gates: gate-language, gate-egress, gate-clientdata --all PASS; plumbline/tools/licenses.sh ran (exit 0).
- No expectation moved. Patch: SCRATCH/review/call207M.patch.

## Second round (lead's rulings 1 and 2)
Ruling 1. Held the old reply of `set <material PATH> price|quote`: only demos/d5.sh:202 (no other test, demo or crates/cli test holds either reply;
window demos were not run). Words moved, only the words: "a price does not carry from one unit to another" -> "it is bought by the lb and priced by the SF"
(the refusal now comes from the price list's road; the job is as it was, the demo still checks it). No figure moved.
Routing: `set` on a material by path (or name) or an item goes through `set_price`, EXCEPT where the price list would store more than the older road:
`pricelist::stores_more_than_a_plain_price` names three cases, and there `set` keeps the older road (reply and stored result as before), so I stopped on them
as ruled: (a) a supplier's price on a material the job has no own price for: `prices_in` (call 108) also makes it the job's own price; `quote` stores the quote alone;
(b) a supplier written another way from the job's: `quote` refuses it, `set_price` takes the job's spelling; (c) a cost line of the same name, not made of the material,
with no price: `prices_in` gives it the price; a typed material price does not. No difference found in status, source or unit stored otherwise (figure, unit, status, source
compared in a test, plain `set_on_line` against `set_price`). Unit match is now case-insensitive (lb = LB), stored unit unchanged. Tests: 16 in call207m.rs; demo step 13.
Ruling 2 taken as built. Results: world 624 passed (44 runs), cli 22, sheets 6, ifc 53, plans 172, roofing 122, 0 failed; 54 command-line demos all exit 0, no FAIL/FAILED;
gates PASS, licenses exit 0, portmap PASS. Patch rewritten.
Incident: my d5.sh run without PLUMBLINE_BIN started a workspace `cargo build --quiet` in wtB (it builds the window crate); I killed my own orphan by pid after a pkill was refused.
