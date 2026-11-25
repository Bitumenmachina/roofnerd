# Contract 207M: a price set by name, and the price list as rows (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 207" (points
4 and 5 are yours; 1 to 3 are the window's), "### Call 196" and "### Call 201" (an
item is its own entry in the price list), "### Call 206" point 13 ($0.00);
plumbline/STATUS.md "A price typed where it is seen: a demonstration";
`crates/world/src/tables.rs` (`prices_out`, `prices_in`), `crates/world/src/editing.rs`
(`price_item`: what it does and for what, call 157), how a material holds its price
and each supplier's (`Supply`), and how the estimate sets three suppliers side by
side (`crates/world/src/view.rs`).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-207M-report.md. Commit nothing, push nothing.
When done: `git add -N . && git diff --binary -- plumbline > SCRATCH/review/call207M.patch`.

## Before any code, in your report
1. BIMCraft (`/home/user/ipedrax/bimcraft`): it carries no money (the worker of
   call 196 read all of it: `research/worker-report-call196.md`). Confirm with one
   grep and say so; nothing to port. `PORTMAP.md` gets its rows for new public
   items; portmap must pass.
2. Every way a price reaches a material or an item today (`import prices`, the
   chest's table, `price-item`, anything else), what each does, and what none
   does. Say whether one of them can be the change call 207 point 4 asks for, or
   what the smallest new change is. One arithmetic: the new change and
   `prices_in` must set a price by the same road.
3. The rows the model will give for the price list (their fields), with one job's
   list written out as it will read, and where each figure comes from (lines
   counted and dollars summed by the model's own estimate, not again).
4. The verb: extend one that fits (`price`, `price-item`, `set`); add one only if
   none does. Say which and why.

## What is built (call 207, points 4 and 5)
- The change: a price set or cleared, the estimator's or a named supplier's, for a
  material or an item by name, all or nothing, one step for Undo, giving back
  what follows (lines priced, things, the job cost before and after, a supplier
  added) in words both the window and the command line say.
- The price list as rows, worked by the model, for whoever shows it; the command
  line prints it (where `price` or `prices` prints today: say).
- Tests FIRST, run red, quoted: each of the demonstration's eight steps on the
  model, and: a price for a material nothing is made of (taken, says it prices
  nothing yet); two suppliers then a third and a fourth (what the job allows:
  say); the same price again (says nothing changed); a price set, the item
  renamed, the price still its; a price set then `import prices` of a list that
  has the same name (which wins, and that it is said).
- `demos/pricelist.sh` (new) to the eight steps; its row in `README.md`.
- **Every existing expectation that moves is listed before one is changed**; by the
  design none should.

## Rules
- Files you may touch: `crates/world/src/` (the change, the rows; `tables.rs` only
  so both roads are one), `crates/world/tests/` (a new test file), `crates/cli/src/`,
  `demos/pricelist.sh`, `README.md` (a row), `PORTMAP.md`, `SPEC.md` (only if what
  the saved job holds changes: it should not). Nothing in `crates/app/src`: say in
  the report the call the window will make for a price typed on a row, for the
  rows of the tab, and for the item's panel.
- The saved job's format does not change. No rounding rule and no status rule
  changes. No price anywhere comes from a real job or a real supplier; no real
  supplier's name is used.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Check
  `df -h /` before each build. Demonstrations with
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- Everything in the foreground; no timers or background runs left when you hand
  back. At the end run every command-line demonstration and search their words
  for FAIL and FAILED as well as reading how each ended.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has the four "before any code" answers; the new tests pass (red runs
quoted); `demos/pricelist.sh` passes; every command-line crate's tests and every
command-line demonstration pass with no expectation edited, or the ones that moved
are named and unedited; the gates pass.
