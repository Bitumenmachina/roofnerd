# Contract 195A: the bench, an item looked at alone (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 195" (step A is yours; B to F are not), "### Call 105"
and "### Call 111" (what the tool chest is), plumbline/STATUS.md "The bench: an item
looked at alone. A demonstration" (your demonstration, with its figures),
`demos/itemlibw.sh` and `demos/itemneww.sh` (how an item's panel is driven), and in
`crates/app/src`: `panels.rs` (the Items list, "Change", an item's panel: search
`Rename item`), `scene.rs` (how a thing set down is drawn from its form; how the view
flies to a thing on a double-click; how a trade's view and the Show switches put
things away), `tools.rs` and `main.rs` (what Esc does, tool by tool).
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-195A-report.md. Commit nothing, push nothing.

## Before any code, in your report
1. Search ALL of BIMCraft at /home/user/ipedrax/bimcraft for a place where one
   family or type is edited alone (`grep -rln -i "family_editor\|family editor\|edit_family\|isolate\|workbench\|bench" crates apps`
   and read the whole list). Port what is there; say in the report what you found
   either way. (`PORTMAP.md` is only for public items in `crates/world`; you add
   none.)
2. Say how the window will know it is on the bench (one piece of state, where it
   lives), what draws on the bench and what is put away, and how the view that was
   left is kept. Say what Esc does today in each state the bench can be entered
   from, so Esc on the bench takes nothing else with it.

## What is built (call 195, step A, every clause)
1. **"Bench"** on an item's panel (and nowhere else). On the bench: nothing of the
   house, its sheets, its runs or its clashes is drawn; the item stands alone at
   the middle of a floor of its own, lined every foot, a little larger than the
   item; it is drawn by the same code that draws it set down (its form, its
   material), not a second drawing of it.
2. Its width, depth and height are dimensioned beside it: a line with ends, and
   the size in the words its panel uses ("36 in", "34 1/2 in"), in the colour of how
   sure it is. The three do not lie over one another or over the item from the
   view the bench opens at.
3. The view flies to the bench and frames the item with room round it; it never
   jumps. Orbit, wheel and drag work as they do on the house. **"Back to the
   house"** (on the item's panel, where "Bench" was) and **Esc** fly back to the
   view that was left.
4. The item's panel stays and works as it does today. A size typed there changes
   the item on the bench when it is entered, and the window says the new size and
   how many set down from the item follow (the model already gives this: find it,
   do not count in the window).
5. No drawing tool is offered on the bench, and none can be started from the
   keyboard there. The Items list stays: picking another item's "Change" on the
   bench puts that item on the bench.
6. An item never set down goes on the bench the same way.
7. **The saved job does not change**: the bench is not saved, and a job saved
   after the bench is, byte for byte, the job saved after the same changes made
   without it. The window works no number out.
8. Every message is a builder's words (`tools/gate-language.sh` passes). The new
   controls are on the map by name ("Bench", "Back to the house"); every existing
   control keeps its name.
9. **Call 197, points 8 and 9** (read them): a size typed on an item's panel that
   differs from the size it has is Assumed, "typed in the window; not yet checked
   against its maker's sheet" (the words `tools.rs` already has for New item);
   the same size typed again is the check and is Known, "checked on site", as
   today. The message says which happened, and, the first time, how it is checked.
   On anything that is not an item a value picked and typed is a field
   measurement, exactly as now. When the item's name carries the old size, the
   message says so and names Rename item. **List every existing expectation this
   moves before changing one** (`grep -n "measured" demos/item*.sh demos/*.sh`,
   the tests of `crates/app`): each moves only from Known "measured on site" to
   Assumed with those words, on an item's size; anything else is handed back.
10. `demos/benchw.sh` (new) to the eight points in STATUS.md, with its pictures in
   `demos/captured/benchw/`, and its row in `plumbline/README.md`. The pictures a
   person will look at: the vanity on the bench at 36 in and at 48 in; the water
   heater on the bench; the house before and after.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/benchw.sh`,
  `demos/captured/benchw/`, `demos/window.sh` (only to add a helper, changing none),
  `plumbline/README.md` (a row). Nothing in `crates/world`, `crates/plans` or any
  other crate: if the model does not give something the bench needs (how many
  follow, a size's words), say what is needed and stop on that point.
- No number, meaning or file format changes, and no existing expectation except under
  clause 9. Tests first,
  where a thing can be tested without a screen (the bench's state, what is drawn),
  run red, quoted.
- Freeze a copy of the two programs before window runs (`PLUMBLINE_BIN`); delete old
  frozen copies by literal absolute path; no cargo beside a timed run; a failure is
  run once more before it is believed. Watch the disk (`df -h /`; under 4 GB free
  delete your own old test programs by literal path).
- Look at every picture yourself, as a person opening the program for the first
  time. Say plainly whether the item on the bench reads as the thing it is, whether
  anything is under a panel, and what is not right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop and
  say so in the report; do not ask the lead to do it. No web search with a person's
  or the project's name in it.

## Done when
`cargo test --workspace` passes (count); `benchw`, `itemlibw`, `itemneww`, `itemsw`
(if there is one: the maker's sheet walk), `movew` and `glassw` pass on your frozen
build, or stand recorded with what was seen; the three root gates and
`tools/licenses.sh` pass; the report has the two "before any code" answers, says
what each picture shows and what is not right.
