# Contract 213M: the second command-line judge's findings fixed (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 213"
(every numbered point is yours), `research/judgment-cli-1125b.md` (the judge's
notes: each finding has the commands that reproduce it from an empty job),
plumbline/STATUS.md "The command line judged a second time" (the demonstration and
its figures), and the decisions the findings touch: "### Call 211" with its "as
built", "### Call 210" with its "as built", "### Call 205", "### Call 212", and
"### Call 197" point 9 with "### Call 202" point 7 (the same size typed again).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-213M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call213M.patch`.

## Before any code, in your report
1. **Reproduce every finding first**, F1 to F22, on the tree as it is, with the
   judge's commands, and quote what the program says. One you cannot reproduce is
   said so, with what you tried; it is not fixed by guess.
2. For F16: say why the job cannot be read after the rename (what the rename did
   to the tree of the job), list every kind of thing that can be renamed and which
   of them hold the one-name rule today, and where bringing-in holds it.
3. For F2's first case (the item's depth typed 24 in with pipes ending 3 in short
   of the old back): say exactly where each pipe ends and where each connection
   stands, and so whether each is joined by its end or by passing over, and why
   `clashes` says 3 in. If the program is right and the judge's reading is not,
   say so.
4. For point 6 (F5): `grep -rn` every test and demonstration that types the same
   size twice at the command line, or through the model's `item_size`, to make it
   Known. LIST each with its file and line and what it will become before
   changing one. The window's own calls and the window's demonstrations do not
   change: say how the one function is told who is asking.
5. For point 8: say exactly what the figure reader takes today for a length, what
   it will take, and show that nothing which was a figure before reads otherwise
   (run every shape, every connection row and every cost line formula in
   `library/` and the tests through both).
6. For point 15: run the new check over the thirteen starter shapes at the sizes
   a shape is tried at and say what it finds. If a starter block is flat or
   backwards at a tried size, STOP on that point and say which; change no starter
   row.

## What is built
Every numbered point of call 213, each with a test written first and run red
(quoted). `demos/judged-1125b.sh` (new) to the fourteen steps in STATUS.md, every
step ending the script when the program does not say what the step expects; its row
in `README.md`.

## Rules
- Files you may touch: `crates/world/src/` and `crates/world/tests/`,
  `crates/cli/src/`, `crates/ifc/src/` and `crates/ifc/tests/`,
  `demos/judged-1125b.sh`, `README.md` (a row), `SPEC.md` (only if a key changes;
  none should), `PORTMAP.md` (new public items). Nothing in `crates/app/src`; if a
  model function the window calls must change its arguments, STOP and say which:
  the window is being built on by another builder and must still compile unchanged
  (`grep -rn "<the function>" crates/app/src` before changing any public function).
- **Existing expectations**: each one that moves is listed BEFORE it is changed,
  with its file and line, the words or figure it held and the words or figure it
  will hold, and the point of call 213 that moves it. Only words named in call 213
  and, under point 6, the listed twice-typed sizes may move. No quantity, price,
  rounding rule or status rule changes otherwise. Anything else that fails is
  handed back with what was seen, not changed.
- The saved job's format does not change. A job written before reads the same.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates (`-p plumbline-world -p plumbline -p
  plumbline-plans -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing`), one
  crate's tests at a time. NEVER run a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. Watch `df -h /`: under 4 GB
  free, stop and delete your own old test programs (literal absolute paths).
- Search each demonstration's output for FAIL as well as its exit.
- The three root gates, `tools/licenses.sh` and
  `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` pass.
- Every word a person sees is a builder's: a connection, never a port or a
  connector; inches, not feet with decimals, in a refusal about an item.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the six "before any code" answers; every point has its test (red
run quoted) and passes; `demos/judged-1125b.sh` passes; every command-line crate's
tests and every command-line demonstration pass, with every moved expectation on
the list; the gates pass; the report says what is not right and which findings, if
any, are not fixed and why.
