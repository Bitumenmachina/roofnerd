# Contract 193W: the window judge's findings fixed (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 193" (points A to H: every one is yours),
plumbline/research/judgment-window-1124.md (the judge's notes: each finding names
the script and step that shows it; its scripts, jobs and pictures are in
SCRATCH/judges/window-1124/, READ ONLY), and plumbline/STATUS.md "The window judged
(2026-11-24), and its findings fixed: a demonstration" (the steps your work comes
to). SCRATCH is SCRATCH.
Notes as you go to SCRATCH/review/W-193W-report.md. Commit nothing, push nothing.

## Part 0, first, before any code
The tree was merged at 51cf498 and SCRATCH/binM3 holds its two programs, frozen. The
judge drove that build for thirteen runs of its own; the repository's own
demonstrations have not been run on it. With `PLUMBLINE_BIN=SCRATCH/binM3`, nothing
else running, run `demos/trimw.sh`, `tracew.sh`, `movew.sh` and `tradesw.sh`; read
their pictures; write each result to your report at once. A failure is run once more
before it is believed; a real one is reported and no expectation is edited for it.

## The order, and how each is done
For each point: reproduce the judge's finding first on SCRATCH/binM3 and say what the
window said; then the fix; then the step of the demonstration in STATUS.md (extend
`tracew.sh` for B, C, D, F, G; `movew.sh` for E; `trimw.sh` for A; `lightw.sh` for H).
A unit test first wherever the thing can be tested without the window.

Do B, C, D, E, F, G, H first: they move no existing picture. Then A, last and by
itself, because it changes the picture of every house with openings or siding:
- Before changing what is drawn, list in your report every window demonstration
  whose house has openings or siding with no trim priced, and what each measures
  near an opening, a corner or an eave.
- Make the change. Run those demonstrations. **Where a check fails because trim is
  no longer drawn where it sampled, do NOT edit it: record which check, the colour
  or count it wanted, what it now gets, and a picture before and after, and go on
  to the next.** The lead rules on each; you then make the moves it rules.
- The roof's own edge stays as the roof (rename what the code calls it; it is not
  fascia and is not counted as trim).

## Rules
- Files you may touch: `crates/app/src/*.rs`; `demos/tracew.sh`, `movew.sh`,
  `trimw.sh`, `lightw.sh`; `demos/captured/` for those four; `plumbline/README.md`
  (a row, if a demonstration is added). Nothing in `crates/world` or `crates/plans`
  except this: if point B, C or D cannot be done without a function the model lacks,
  say what is needed and stop on that point.
- Keep every existing control's name. No number, meaning or file format changes.
  No existing expectation is edited except by the lead's ruling under point A.
- Every word a person sees is a builder's word. Point C's, D's, F's and H's
  sentences are yours to word; say them in your report.
- Freeze a copy of the two programs to a scratch folder before window runs
  (`PLUMBLINE_BIN`); delete old frozen copies; no cargo beside a timed run; a
  command-line worker shares the machine at low priority from the time the lead
  gives: a demonstration that fails is run once more before it is believed. Watch
  the disk (`df -h /`; under 4 GB free, delete your own old test programs by
  literal path).
- Look at every picture yourself before you call anything shown.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop and
  say so in the report; do not ask the lead to do it.

## Done when
`cargo test --workspace` passes (count); `tracew`, `movew`, `trimw`, `lightw`,
`scanw`, `planw` and `tradesw` pass on your frozen build (or stand recorded under
point A for the lead's ruling); the three root gates, `licenses.sh` and
`BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` pass; the report goes point
by point (reproduced, fixed, shown), lists point A's failed checks with their
pictures, and says plainly what is not right.
