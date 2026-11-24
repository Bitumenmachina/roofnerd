# Contract 194: the sample whole house prices its outside trim (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 194" (every
numbered point), plumbline/STATUS.md "The sample whole house prices its outside trim: a
demonstration" (the figures your work comes to), `crates/world/src/sample.rs`
(`house_priced`), `crates/world/src/formula.rs` (search `BELOW`: how
`net_area_below_<entry>` is named and read; follow it everywhere it is handled, with
`grep -rn "BELOW\|net_area_below_" crates`), `crates/world/src/trim.rs` and
`crates/world/src/trim/lay.rs` (the corner boards' figure and their pieces; how a
line is taken as a corner-board line: `priced_lines`, `other_lines`), and
`library/sixteen-divisions.csv` (the wainscot wall's Corner boards line).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-194-report.md. Commit nothing, push nothing.
When done, from the working copy's root:
`git add -N . && git diff --binary -- plumbline > SCRATCH/review/call194.patch`.

## Before any code, in your report
1. Search ALL of BIMCraft at /home/user/ipedrax/bimcraft for a corner board or trim
   that stops at a wainscot or a base course (`grep -rln -i "wainscot\|corner_board\|cornerboard\|water.table" crates apps`
   and read the whole list). Port what is there; record in `plumbline/PORTMAP.md`
   either way; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
2. Run `plumbline sixteen`, `estimate`, `trim` and `trim --pieces` on the tree as it
   is and set what the program says beside the lead's hand table in STATUS.md, line
   by line (the walls' heights, each window's casing, each eave, the overhang, the
   count of windows, the recap and division 6 before). Do not bend the program to a
   hand figure: where they differ, say which is right and why, and stop on that
   point for a ruling.
3. Say how a figure that takes a height from its line is handled today, place by
   place (naming, the names a formula lists, working it, the worked formula shown,
   the refusal when the entry is missing or is not a length, the workbook's live
   formula in `export.rs`, the CSV of assemblies in `tables.rs`), so the new figure
   is handled in every one of them. A place you find and do not handle is named.

## What is built
- The figure `corner_board_length_above_<entry>` on a wall, as call 194 point 2 says.
  One arithmetic: the pieces laid for a line priced by it start at the entered
  height, and their lengths add to the line's quantity (a test holds the two
  together on walls of different heights, an entry taller than the wall, an entry
  of nothing, a wall with one outside corner, a wall with none).
- A line priced by it is a corner-board line wherever the program asks whether a
  wall's corner boards are priced (`trim --pieces`; the function the window calls to
  draw trim only where priced: find it and make it true there by test, without
  touching `crates/app`).
- `house_priced` as call 194 points 1, 3 and 4 say, to the names and prices given.
- The starter wainscot wall's Corner boards line moved to the new figure with its
  `wainscot` entry (point 5). Nothing else in the library changes.
- **Every existing test and demonstration expectation that moves is listed before
  one is changed** (point 6): run every command-line crate's tests with
  `--no-fail-fast` and every command-line demonstration; for each failure give the
  file and line, the figure it expected, the figure it now gets, and the sum that
  takes one to the other. Then change those, and only those, whose movement is
  exactly a figure in the STATUS.md section or follows from one by arithmetic you
  show (for example a price that rises by $4,421.03, a line count by 33, a division
  6 total by $3,721.40). An expectation that moves any other way is NOT edited: it
  is handed back with what was seen. For the window's demonstrations
  (`grep -ln "sixteen\|house_priced" demos/*.sh crates/app`), which you cannot run:
  list each expectation that will move, change it under the same rule, and say
  plainly in the report that none was run.
- Tests FIRST, run red, quoted. A demonstration `demos/sixteen-trim.sh` (new; command
  line only) to the six numbered points in STATUS.md, and its row in `README.md`.

## Rules
- Files you may touch: `crates/world/src/` (`sample.rs`, `formula.rs`, `trim.rs`,
  `trim/lay.rs`, and only what a place found under "before any code" 3 needs),
  `crates/world/tests/` (a new test file; existing ones only under the rule above),
  `crates/cli/src/` (only if a verb must name the figure), `library/sixteen-divisions.csv`
  (the one line), `demos/sixteen-trim.sh`, existing demonstrations only under the
  rule above, `README.md` (a row), `PORTMAP.md`, `NOTICE` (only if something is
  ported). Nothing in `crates/app/src`.
- The saved job's format does not change. No rounding rule, no status rule and no
  other figure's meaning changes. No price or quantity anywhere comes from a real job.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates (`-p plumbline-world -p plumbline -p
  plumbline-plans -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing`), one
  crate's tests at a time. Watch `df -h /`: under 4 GB free, stop and delete your own
  old test programs (literal absolute paths). Do not run cargo before the time the
  starting message gives; do other work until then and do not hand back only to wait.
- Demonstrations with `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has the three "before any code" answers; the new tests pass (red runs
quoted); `demos/sixteen-trim.sh` passes; the list of every expectation that moved is
complete, each with its arithmetic, and every one not covered by the rule is left
unedited and named; every command-line crate's tests and every command-line
demonstration pass or are on that list; the gates pass.
