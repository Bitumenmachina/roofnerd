# Contract 208: four things with two readings, brought to one (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 208" (every
point is yours), "Call 207, the model's half, as built" and "### Call 108" (what a
price list brought in does with a supplier's price), "Call 205, the model's half, as
built", "### Call 206" point 13; `crates/world/src/pricelist.rs` (search
`stores_more_than_a_plain_price`: the three cases), `crates/world/src/tables.rs`
(`prices_in`), `crates/world/src/view.rs` (`pricing`), and
plumbline/research/worker-report-call207M.md ("Second round").
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-208-report.md. Commit nothing, push nothing.
When done: `git add -N . && git diff --binary -- plumbline > SCRATCH/review/call208.patch`.

## Point 1, first and by itself
- **Before any code**: for each of the three cases write the smallest job that
  shows it, what the older road stores and says, and what the list's road stores
  and says. Then list EVERY test and demonstration that sets a material's price or
  a supplier's quote by the older road (`grep -rn "quote\|set .* price" demos/*.sh crates/*/tests | grep -v prices`),
  and for each say whether one of the three cases is met in it and, if so, the
  figures it holds now and will hold, with the arithmetic.
- Then make the older road the list's road in all three cases and take
  `stores_more_than_a_plain_price` away. Move only the expectations on your list,
  each with its arithmetic beside it in the test or script. **A figure that moves
  and is not on your list is NOT edited**: stop, and hand back with it named.
- Tests first, run red, quoted: one for each case.

## Points 2, 3 and 4
Each: the finding shown on your build, a test run red, the fix. For point 2 the
test that tied the two copies becomes a test of the one function. For point 4 list
every kind of length a line can enter and refuse each below nothing, with the
reply in a builder's words.

## Rules
- Files you may touch: `crates/world/src/`, `crates/world/tests/`, `crates/cli/src/`,
  `demos/pricelist.sh` and `demos/judged-1125.sh` (a step each for what you change),
  other existing tests and demonstrations only under point 1's rule, `README.md`,
  `PORTMAP.md`. Nothing in `crates/app/src`.
- The saved job's format does not change. No rounding rule and no status rule
  changes. No price from a real job or a real supplier.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates (`-p plumbline-world -p plumbline
  -p plumbline-plans -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing`),
  one crate's tests at a time. NEVER run a demonstration without `PLUMBLINE_BIN`
  set: without it the script builds the whole workspace, the window too, and
  fills the disk. Check `df -h /` before each build; under 3 GB free, stop.
- Everything in the foreground; no timers or background runs left when you hand
  back. At the end run every command-line demonstration and search their words
  for FAIL and FAILED as well as reading how each ended.
- The three root gates and `plumbline/tools/licenses.sh` pass;
  `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` passes.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has point 1's list, complete, before any change; each point has its
red run; every command-line crate's tests and every command-line demonstration
pass or are on the list of expectations moved, each with its arithmetic; no FAIL
in their words; the gates pass.
