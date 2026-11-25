# Contract 209M: the third window judge's findings that are the model's (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 209" (points
2, 3, 4, 5 and 8 are yours, the model's and the command line's side of each),
plumbline/research/judgment-window-1125.md (the judge's notes; its scripts and jobs
are in SCRATCH/judges/window-1125/, READ ONLY), and the calls each names.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-209M-report.md. Commit nothing, push nothing.
When done: `git add -N . && git diff --binary -- plumbline > SCRATCH/review/call209M.patch`.

For each point: the judge's finding shown on your own build at the command line
(the window says the model's words: find where the words are made), a test run
red and quoted, the fix, a step of `demos/judged-1125.sh`.
- **Point 2**: `rename_item`: every thing set down is renamed, each keeping its
  number; what it gives back is the count renamed. Tests: one, two and five set
  down; a thing a person renamed by hand since (say what the rule is and why: the
  lead's working answer is that a name a person gave is left, and counted as left).
- **Point 3**: every phrase the judge quotes, and any other in the same replies
  that is the program's word and not a builder's: list them all before changing
  one, with what each becomes. The reply the window shows must not name a
  command; if the command line needs the command named, the command line adds it
  to the model's sentence, the model does not carry it.
- **Point 4**: the two refusals say what stands in the way and what a person can
  do about it today. Do not build point 11 (bringing in part of a model).
- **Point 5**: where a length is read (`units::parse_length` and whoever calls it):
  say what is read today for `12-6`, `12'-6"`, `12 6`, `12-6 1/2`, `6-0`, `-6`, and
  what will be; `12-6` is twelve feet six inches; a bare `-6` is still below
  nothing. **This changes what a typed size means: list every test and
  demonstration that types a length with a hyphen, and what each reads before and
  after, before changing anything; a figure that moves and is not on the list is
  not edited.** Then the two wordings.
- **Point 8**: the model gives the counts of known, assumed and unknown values for
  the building and for the tool chest apart (find where the top bar's count is
  worked; if it is the model's, split it there and keep the old total as their
  sum; if it is the window's own, say so and give the model's function for the
  window to call). `plumbline show` prints both. Do not edit `crates/app`.

## Rules
- Files you may touch: `crates/world/src/`, `crates/world/tests/`, `crates/ifc/src/`,
  `crates/ifc/tests/`, `crates/cli/src/`, `crates/cli/tests/`, `demos/judged-1125.sh`,
  existing tests and demonstrations only where a reply gained or changed the
  words a point asks for (list each first), `README.md`, `PORTMAP.md`. Nothing in
  `crates/app/src`: for each point say what the window will now say, or what call
  it must make.
- The saved job's format does not change. No rounding rule and no status rule
  changes. No number from a real job.
- Build ONLY the command-line crates (`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
  nice -n 19 cargo ... -p plumbline-world -p plumbline -p plumbline-plans
  -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing -j2`), tests one crate
  at a time, never with no `-p`. NEVER run a demonstration without `PLUMBLINE_BIN`
  set. Check `df -h /` before each build; under 3 GB free, stop.
- Everything in the foreground; no timers or background runs left when you hand
  back. At the end run every command-line demonstration and search their words
  for FAIL and FAILED as well as reading how each ended.
- The three root gates, `plumbline/tools/licenses.sh` and
  `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
Each point has its finding shown, its red run and its step; the lists asked for
under points 3 and 5 are in the report, complete, before any change; every
command-line crate's tests and every command-line demonstration pass or are on a
list of expectations moved, each with its reason; no FAIL in their words; the
gates pass.
