# Contract 207W: the price list in the window (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 207" (points 1 to 3 are yours) with "Call 207, the
model's half, as built", "### Call 201" and "### Call 206" point 13;
plumbline/STATUS.md "The price list in the window: a demonstration";
plumbline/research/worker-report-call207M.md (the model's `set_price` and
`price_list`, and the three calls the window makes); `crates/world/src/pricelist.rs`;
and in `crates/app/src`: `sheet.rs` (the estimate window: its tabs, the Estimate tab,
how a unit cost is typed on a line and how sure it is afterwards), the item's panel
and the bench in `panels.rs` and `bench.rs`.
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-207W-report.md. Commit nothing, push nothing.

## Before any code, in your report
1. How a unit cost typed on a line of the Estimate tab is taken today: what the
   person does, how sure the value is afterwards and where it says it is from.
   A price typed on the Prices tab is taken the same way and is as sure.
2. The tabs of the estimate window, when each is offered, and where "Prices"
   goes (beside Estimate; offered whenever the job has a row for it).
3. How the sheet shows three suppliers side by side today, so the Prices tab's
   three are the same three, in the same order, under the same names.

## What is built (call 207, points 1 to 3)
1. **"Prices"**, a tab of the estimate window: the model's `price_list(job)` as
   rows, nothing counted or summed in the window. Each row: the name, what it is
   (a material, or an item and its kind), its unit, the estimator's price, the
   first three suppliers' beside it, each in the colour of how sure it is with
   where it is from on demand, the lines it prices and on how many things, and
   the dollars riding. Rows with no price are at the head and read "no price",
   never $0.00. A price entered at nothing reads $0.00 and is marked.
2. **A price is typed on its cell**, the estimator's or a supplier's: the model's
   `set_price` is called and its words are said (the lines priced, the things,
   the job cost before and after). Cleared with the field left empty: no price.
   A refusal is said as the model says it and the cell is as it was. One step
   for Undo. The estimate beside it and the price at the top follow at once.
3. **A supplier added**: a way to name a supplier not yet in the job from this
   tab (a fourth and later ones are kept by the model and not shown: the tab says
   how many are not shown).
4. **On an item's panel and on the bench**, under "Priced as": the item's price,
   typed the same way, the same call. An item priced as nothing shows no price
   row and says why.
5. Scrolling, "more below", the wheel and the look are the estimate tab's.
6. Every word is a builder's (`tools/gate-language.sh`); every new control is on
   the map by name; every existing control keeps its name.
7. `demos/pricesw.sh` (new) to the steps in STATUS.md, its pictures in
   `demos/captured/pricesw/`, its row in `plumbline/README.md`.

## Part 2, after the above is whole: the window says the model's sentences (call 209, the model's side as built)
Read plumbline/DECISIONS.md "Call 209, the model's side, as built" and
plumbline/research/worker-report-call209M.md (search "The window must"). One step
each in `demos/pricesw.sh` or a session of its own:
- **A rename** calls `rename_item_counted` and shows what it says (how many were
  renamed, how many kept and why), in place of the window's own sentence.
- **A file written out or brought in**: the window shows `Written::not_written_said()`
  and `Brought::left_out_said()` in place of its own list; for the estimate it adds
  where (Estimate workbook, under Write out).
- **The top bar** shows `view::tallies(...).building`; the tool chest's values are
  not in it (point 8 of call 209, left by the last window job).

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/pricesw.sh`,
  `demos/captured/pricesw/`, `demos/window.sh` (only to add a helper, changing
  none), `plumbline/README.md` (a row). Nothing in `crates/world`: if the model
  does not give something the tab needs, say what and stop on that point.
- No number, meaning or file format changes; no existing expectation is edited.
  Tests first where a thing can be tested without a screen, run red, quoted.
- Every window run in the FOREGROUND with the longest timeout, a session at a
  time; before each, `pgrep -x rustc` and wait if busy; a failure is run once
  more before it is believed. Freeze your build (`PLUMBLINE_BIN`); delete your own
  old frozen builds by literal absolute path. Check `df -h /` before each build.
- Look at every picture yourself, as an estimator opening the program for the
  first time: can they find where a price goes without being told? Say plainly
  what is not right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop and
  say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it. No timers left running when you hand back.

## Done when
`cargo test` passes for every crate, one at a time (count); `pricesw`, `pricew`,
`moneyw`, `benchw` and `itemlibw` pass on your frozen build, or stand recorded with
what was seen; their output holds no FAIL; the three root gates and
`tools/licenses.sh` pass; the report has the three "before any code" answers, says
what each picture shows and what is not right.
