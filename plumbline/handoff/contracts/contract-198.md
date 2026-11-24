# Contract 198: an item's shape as data (the bench, step C; command line first; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 198" (every
numbered point) with "### Call 195" and "### Call 105" before it, plumbline/PROPOSALS.md
28, plumbline/STATUS.md "The bench, step C: an item's shape as data. A demonstration"
(the figures your work comes to), `crates/world/src/building/forms.rs` whole (the
thirteen shapes as code, and where each came from in BIMCraft),
`crates/world/src/formula.rs` (the language a cost line's quantity is written in),
`crates/world/src/tables.rs` (`items_out`, `items_in`: how the chest's table is
written and read), `crates/world/src/file.rs` and `SPEC.md` (the saved job), and
BIMCraft's `/home/user/ipedrax/bimcraft/crates/core/src/family.rs` (parameters,
formulas, forms: what a shape as data is there).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-198-report.md. Commit nothing, push nothing.
When done: `git add -N . && git diff --binary -- plumbline > SCRATCH/review/call198.patch`.

## Before any code, in your report
1. BIMCraft's `Family`: what of it is ported already (`PORTMAP.md`), what a form's
   figures are there (its `Expr`), and what this step takes from it and leaves
   (call 198 point 7 says what is left). Record in `PORTMAP.md`;
   `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
2. **Set the design beside the code, point by point** (call 198, 1 to 5). In
   particular: every operation the thirteen pieces of code use that the formula
   language does not have today (the lesser of two, a share of a size, a size in
   millimetres), and for each how the row will say it; whether `min` and `max`
   can be added to the language without changing how any formula written today
   is read (run every test that reads a formula; say the count); the axes and the
   unit a row's figures are in, and that a fixed size is written with its unit.
   **Where the design and the code disagree, stop on that point for a ruling.**
3. The starter table as it will read: its columns, and the base cabinet's three
   rows written out, with the lead's hand figures for the 36 in vanity beside
   what the rows give.
4. The saved job: what it gains, as it will read in the file; that a job saved
   before opens and every thing set down in it has the blocks it had.

## What is built (call 198, points 1 to 5; point 6 is the window's and is not yours)
- The shape as rows; the starter table `library/shapes.csv` with the thirteen;
  **a test that holds each shape's rows to the code it replaces, block for block,
  at three sizes (one small enough that every "no more than a share" bites), to a
  hundred-millionth of a foot, run and passing BEFORE the code is taken out**;
  then the code taken out, so a shape is worked from its rows and nowhere else.
- `min(a, b)` and `max(a, b)` in the formula language, for cost lines as well.
- The chest written out and brought in as two tables; the saved job carrying the
  shapes its items name; an item naming a shape that is not there refused.
- A verb that prints a shape's blocks at an item's sizes (find where `items`
  prints an item and extend it; add a verb only if none fits).
- `crates/app` draws from `forms::blocks(form, size)` today: keep a function the
  window can call with what it has, so the window builds unchanged, and say in the
  report what it is. Do not edit `crates/app`.
- **Every existing expectation that moves is listed before one is changed.** By the
  design none should: the thirteen shapes give the blocks they gave.
- Tests FIRST, run red, quoted. `demos/shapes.sh` (new; command line only) to the
  six points in STATUS.md, and its row in `README.md`.

## Rules
- Files you may touch: `crates/world/src/` (the forms, the formula language, the
  tables, the saved job, the item), `crates/world/tests/` (a new test file),
  `crates/cli/src/`, `library/shapes.csv` (new), `library/items.csv` (only if the
  items' "Drawn as" words must change: say why), `SPEC.md`, `demos/shapes.sh`,
  `README.md` (a row), `PORTMAP.md`, `NOTICE` (only if something is ported).
  Nothing in `crates/app/src`.
- No figure of any cost line, no rounding rule and no status rule changes. A shape
  carries no Known or Assumed (call 198 point 5). No number from a real job.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Watch
  `df -h /`. Demonstrations with `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- At the end run every command-line demonstration and search their words for
  FAILED and FAIL as well as reading how each ended.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has the four "before any code" answers; the holding test passed before
the code went, and passes after; `demos/shapes.sh` passes; every command-line
crate's tests and every command-line demonstration pass with no expectation
edited, or the ones that moved are named and unedited; the gates pass.
