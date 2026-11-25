# Contract 211W: where an item connects, on the bench and in the house (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 211" and "Call 211, the model's side, as built",
"### Call 195" (step D), "### Call 205" (how a shape's blocks are shown and changed
on the bench: this job follows that pattern, row for row), plumbline/STATUS.md
"Where an item connects, in the window: a demonstration" (your demonstration, with
its figures), `demos/connections.sh` (the same things at the command line),
`demos/shapebenchw.sh` (how the bench's panel is driven), and in `crates/app/src`:
`bench.rs`, `panels.rs` (the bench's Shape section; the Connections list), `tools.rs`
(the run tools and the Join tool; how a run's end catches a point: search
`SnapKind`), `scene.rs` (what is drawn while a tool is in hand).
In the model: `crates/world/src/editing/connections.rs` (`item_rows`,
`item_connections_worked`, `connection_add`, `connection_set`, `connection_take`,
`ConnectionDone::said`, `connections_standing`, `run_end_near_connection`) and
`crates/world/src/building/connections.rs` (`ItemConnection::said_at`, `Facing`).
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-211W-report.md. Commit nothing, push nothing.

## Before any code, in your report
1. What each of those model functions gives, quoted. The window says the model's
   sentences and shows the model's places; it works no place out, counts nothing,
   and words no refusal. If something the window needs is missing (for example a
   connection's place on a thing in the house, ready to draw; a word for a snap on
   a connection), say what and stop on that point; nothing in `crates/world` is
   yours to change.
2. How a run tool takes its height today, and what happens to a run's height when
   its first point catches something. Say how clause 5 below will be done without
   the window working out a height.
3. The colours: which colour each system's runs are drawn in today, so a
   connection is drawn in the colour of what it carries and no new colour is made.

## What is built
1. **On the bench, an item's connections are drawn on it**: each a small mark where
   the model puts it, with a short stub the way it faces, in the colour of what it
   carries. They are drawn by the model's worked places, the same for a thing in
   the house.
2. **The panel has "Connections"** under the Shape section: a row for each
   ("Cold water: pipe, cold, in" and where it is in the model's own words), the
   picked one outlined on the item, its figures shown as a block's are (Across,
   Back, Up, Faces, Size, what it takes, what it carries, which way it goes), each
   typed the way a block's figure is. What is typed goes to the model's change and
   the message is the model's. A refusal is the model's words and changes nothing.
3. **"Add connection" and "Take connection away"**, as the Shape section has "Add
   block" and "Take block away". A new one is asked its name first.
4. **Bring in has the connections table**, in the pattern of the entry for the
   item library (a file picked from the list, or the rows the program comes with),
   and Write out writes it. Nothing else Bring in or Write out does changes.
5. **In the house, with a run tool or the Join tool in hand, the connection points
   of what is in view are drawn**, and at no other time. A run's first point
   caught on a connection starts the run there at the connection's height; a run's
   last point caught on a connection ends on it when the run is at the
   connection's height, and when it is not the window says both heights and that
   the run is not joined (the model's `run_end_near_connection` gives how far).
   Only a connection of the tool's own sort catches (a pipe tool: pipe
   connections). The catch names what was caught, as other catches do.
6. The Connections list and the Clashes tab are the model's and change by
   themselves; show that they do.
7. The saved job after the demonstration is byte for byte the job the command line
   makes by the same changes (the demonstration checks it).
8. Every message is a builder's words (`tools/gate-language.sh` passes): a
   connection, never a port or a connector. The new controls are on the map by
   name; every existing control keeps its name.
9. `demos/connectw.sh` (new) to the numbered points in STATUS.md, its pictures in
   `demos/captured/connectw/`, its row in `plumbline/README.md`. The pictures a
   person will look at: the lavatory on the bench with its three connections; the
   cold one picked and moved; the house with the pipe tool in hand and the
   connections showing; the pipe drawn from the connection.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/connectw.sh`,
  `demos/captured/connectw/`, `demos/window.sh` (only to add a helper, changing
  none), `plumbline/README.md` (a row). Nothing in any other crate.
- No number, meaning or file format changes, and no existing expectation. Tests
  first where a thing can be tested without a screen, run red, quoted.
- Window sessions run in the FOREGROUND (a timeout of at most ten minutes each; a
  long demonstration is cut into sessions); never hand back while one is running;
  leave no timers behind. Freeze a copy of the two programs before window runs
  (`PLUMBLINE_BIN`); delete old frozen copies by literal absolute path, keeping
  one; no cargo beside a timed run; a failure is run once more before it is
  believed. Watch the disk (`df -h /`; under 4 GB free delete your own old test
  programs by literal path).
- Look at every picture yourself, as a person opening the program for the first
  time. Say plainly whether a connection reads as a place a pipe goes, whether
  the marks can be told apart, whether anything is under a panel, and what is not
  right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop
  and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
`cargo test --workspace` passes (count); `connectw`, `shapebenchw`, `benchw`,
`itemlibw` and `moneyw` pass on your frozen build, or stand recorded with what was
seen; the three root gates and `tools/licenses.sh` pass; the report has the three
"before any code" answers, says what each picture shows and what is not right.
