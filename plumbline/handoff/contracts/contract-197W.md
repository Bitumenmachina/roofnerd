# Contract 197W: the second window judge's findings fixed (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 197" (points 10 to 15 are yours; 16 is held: change
nothing for it), plumbline/research/judgment-window-1124b.md (the judge's notes: each
finding names the run that shows it; its scripts, jobs and pictures are in
SCRATCH/judges/window-1124b/, READ ONLY), and plumbline/STATUS.md "The window's six
points of call 197: a demonstration" (the steps your work comes to).
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-197W-report.md. Commit nothing, push nothing.

## The order, and how each is done
For each point: reproduce the judge's finding first on the frozen build the lead
names and say what the window said; then the fix; then its step of the
demonstration. A unit test first wherever the thing can be tested without the
window. Do 11, 12, 13 and 15 first (words and a card: they move no picture of the
house), then 10, then 14 last and by itself: it changes how every view is turned.

- **10. Things set down are listed in the sheet window.** The model already counts
  them (the top bar's "not priced", `plumbline show`): find the figure, do not
  count in the window.
- **11. The Cabinets card.** With the tool in hand, at the view the tool opens in,
  no wall of the room is under the card and no word of the card runs into another.
  Boxes typed before the run is started say what to do in the trade's words.
- **12. A corner refused in a run** says where the run still stands.
- **13. A mark asked for and left** says nothing was marked.
- **14. The view swung round the house** (call 197 point 14, both halves). Read how
  the swing picks what it turns about today (`scene.rs`), say it in your report
  before changing it, and say what Google Earth does (CLAUDE.md, "The window is
  glass over the model": the rule is its). The wheel, the slide, the fly-to and
  the home view do not change. **Before changing it, list every window
  demonstration that swings the view** (`grep -ln "orbit\|swing\|mousedown 3\|click 3" demos/*.sh`
  and read `window.sh` for the helper) and what each measures after; run them
  after; a check that fails because the view now turns about another point is NOT
  edited: record what it wanted, what it gets, and a picture before and after.
- **15. A maker's sheet made into an item** says its trade is not said yet.
- **16 (call 200, point 5). An assembly changed for another says what it rewrote.**
  `crates/app/src/main.rs` calls `change_assembly` in two places; use
  `Job::change_assembly_said` and say, in the words `plumbline apply --change`
  uses, each line whose quantity now reads as the new assembly's (old and new) and
  each left as entered. Do it with 11 to 13.

## Rules
- Files you may touch: `crates/app/src/*.rs`; `demos/fixes197w.sh` (new) and
  `demos/captured/fixes197w/`; `demos/window.sh` (only to add a helper, changing
  none); `plumbline/README.md` (a row). Nothing in `crates/world`, `crates/plans`
  or `crates/ifc`: if the model does not give what a point needs, say what is
  needed and stop on that point.
- Keep every existing control's name. No number, meaning or file format changes. No
  existing expectation is edited except by the lead's ruling.
- Freeze a copy of the two programs before window runs (`PLUMBLINE_BIN`); delete old
  frozen copies by literal absolute path; no cargo beside a timed run; a failure is
  run once more before it is believed. Watch the disk (`df -h /`).
- Look at every picture yourself, as a person opening the program for the first
  time, and say plainly what is not right.
- Every message is a builder's words; `tools/gate-language.sh` passes.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop and
  say so in the report; do not ask the lead to do it.

## Done when
`cargo test --workspace` passes (count); `fixes197w`, `cabinetsw` (or whatever the
cabinet run's demonstration is called), `movew`, `tracew`, `glassw`, `house`,
`tradesw` and every demonstration on your list for point 14 pass on your frozen
build, or stand recorded for the lead's ruling; the three root gates and
`tools/licenses.sh` pass; the report says what each picture shows and what is not
right.
