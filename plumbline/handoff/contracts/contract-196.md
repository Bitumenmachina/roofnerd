# Contract 196: an item carries its cost (the bench, step B; command line first; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 196" (every
numbered point) with "### Call 195" before it (why), "### Call 105" and "### Call 111"
(what the tool chest is), plumbline/STATUS.md "The bench, step B: an item carries its
cost. A demonstration" (the figures your work comes to), and in `crates/world/src`:
`editing.rs` (`place_item`, `placed_follow`), `job.rs` (`apply_as`,
`change_assembly`; call 199 if it is in the tree), `lines.rs` (search `of_material`:
a line made of the element's material), `tables.rs` (`items_out`, `items_in`,
`prices_out`, `prices_in`), `file.rs` and `SPEC.md` (the saved job),
`library/items.csv` and `library/sixteen-divisions.csv`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-196-report.md. Commit nothing, push nothing.
When done, from the working copy's root:
`git add -N . && git diff --binary -- plumbline > SCRATCH/review/call196.patch`.

## Before any code, in your report
1. Search ALL of BIMCraft at /home/user/ipedrax/bimcraft for a type or family that
   carries a cost, a price or a schedule line (`grep -rln -i "cost\|price\|schedule" crates apps`
   and read the whole list). Port what is there; record in `plumbline/PORTMAP.md`
   either way; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
2. **Set the design beside the code, point by point** (call 196, 1 to 8), and say
   for each whether the code can do it as written. In particular: what a line
   "made of the element's material" does today on a thing set down, and whether
   making it "made of its item" touches any wall, floor or roof line (it must
   not); how the price list finds a price by a line's name and by a material's
   name, and which of the two an item's line uses; how `placed_follow` tells a
   thing set down from an item, and whether lines can follow by the same road;
   where the estimate says what a total leaves out. **Where the design and the
   code disagree, stop on that point for a ruling** and finish what does not
   depend on it.
3. The saved job: show the one entry an item gains as it will read in the file,
   that a job saved before reads with every item priced as nothing, and what
   `SPEC.md` gains.
4. The chest's table: the columns as they will read (their names and where they
   sit), and that a table written before this change is read as it was.

## What is built (call 196, every clause)
- Points 1 to 7, to the demonstration's seven steps.
- Tests FIRST, run red, quoted. `demos/itemcost.sh` (new; command line only), and
  its row in `README.md`. The command line gains what the demonstration needs to
  say "Priced as" on an item (find the verb items are changed by today and extend
  it; do not add a verb where one fits).
- **Every existing test and demonstration expectation that moves is listed before
  one is changed.** By the design none should: the starter chest's items name no
  assembly, and a job saved before prices as it did. One that moves is handed back
  unedited, with what was seen.

## Rules
- Files you may touch: `crates/world/src/` (the item, its following, the line made
  of its item, the tables, the saved job), `crates/world/tests/` (a new test
  file), `crates/cli/src/`, `library/sixteen-divisions.csv` (the three new starter
  assemblies only), `library/items.csv` (only if the table's new columns must be
  in the starter file: say why), `SPEC.md`, `demos/itemcost.sh`, `README.md` (a
  row), `PORTMAP.md`, `NOTICE` (only if something is ported). Nothing in
  `crates/app/src`: say in the report which window code sets a thing down from an
  item and what it will now do, unchanged.
- No rounding rule and no status rule changes. A thing with no price is never
  priced at zero. No price or quantity anywhere comes from a real job.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Watch
  `df -h /`. Demonstrations with `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has the four "before any code" answers; the new tests pass (red runs
quoted); `demos/itemcost.sh` passes; every command-line crate's tests and every
command-line demonstration pass with no expectation edited, or the ones that moved
are named and unedited; the gates pass.
