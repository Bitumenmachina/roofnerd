# Contract 192W: the gable drawn (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 192" and "Call 192, the model's half, as built",
plumbline/STATUS.md "A wall under a rake rises to the roof: the model's half built"
and "The gable drawn: a demonstration" (your demonstration), and
`crates/world/src/gable.rs`. SCRATCH is
SCRATCH.
Notes as you go to SCRATCH/review/W-192W-report.md. Commit nothing, push nothing.

## Part 0, first, before any code
The starter outside-wall lines now price the gable (`net_area + gable_area`), and
the window demonstrations that give a gabled house those lines have not been run
since: with `PLUMBLINE_BIN` set to the frozen merged build the lead names, nothing
else running, run `demos/trimw.sh`, `pricew.sh`, `gablew.sh`, `tracew.sh` and
`moneyw.sh`; read their pictures; write each result at once. **Where a figure a
script expects has moved because a gable is now priced, do NOT edit it: record the
check, the figure it wanted, the figure it gets, and the hand arithmetic (old plus
gable), and go on.** The lead rules on each.

## What is built
1. A wall that has a gable is drawn up to its top line: the model's
   `gable::figures(job)` gives each such wall's top as points (feet along the wall's
   line, feet above the datum). The wall's body, and everything the wall is dressed
   in on its outside (cladding, sheathing and weather barrier as the Finishes switch
   shows them today), runs to that line. **The window works nothing out**: no
   height, no area. Inside finishes stop at the wall's own top, as the estimate has
   them. The gable's frame is not counted and is not drawn; the framing view shows
   the wall's frame as it is today, and says nothing it did not say.
2. The wall's panel shows the gable's area where the model gives one, in the words
   "Gable" and its SF with its status, among the wall's figures; a wall with none
   shows nothing new.
3. A wall with no gable is drawn dot for dot as before (a hip-roofed house does not
   change).
4. `demos/gableendw.sh` (new) to the figures in STATUS.md, and
   `demos/captured/gableendw/`. The pictures a person will look at: the gable end of
   the trim's house from the street, closed and sided, beside the same view on the
   build before; the shed-roofed house with its tall wall; the weather barrier on
   the gable with Finishes off.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/gableendw.sh`,
  `demos/captured/gableendw/`, `plumbline/README.md` (a row). Nothing in
  `crates/world` or `crates/plans`: if the model's top line is not enough to draw
  from (an opening that reaches into the gable, two stretches), say what is needed
  and stop on that.
- Keep every existing control's name. No number, meaning or file format changes. No
  existing expectation is edited except by the lead's ruling under Part 0.
- Freeze a copy of the two programs before window runs (`PLUMBLINE_BIN`); delete old
  frozen copies; no cargo beside a timed run; a failure is run once more before it
  is believed. Watch the disk (`df -h /`; under 4 GB free delete your own old test
  programs by literal path).
- Look at every picture yourself. Say plainly whether the house now reads as a
  house from the gable end, and what still does not.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop and
  say so in the report; do not ask the lead to do it.

## Done when
`cargo test --workspace` passes (count); `gableendw`, `trimw`, `tradesw`, `dressw`,
`house`, `roofw` and `glassw` pass on your frozen build (or stand recorded for the
lead's ruling); the three root gates, `licenses.sh` and
`BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` pass; the report says what
each picture shows and what is not right.
