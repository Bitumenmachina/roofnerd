# Contract 210W: another trade's model brought in from the window, only its runs and equipment (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 210" (point 6 is yours) and "Call 210, the model's
side, as built", plumbline/STATUS.md "Another trade's model, from the window: a
demonstration" (your demonstration, with its figures), `demos/ifc-runs.sh` (the same
figures at the command line, and how the designer's file is made with the program),
`demos/bringw.sh` (how Bring in is driven), and in `crates/app/src`: `bring.rs` (the
one place a model comes in: search `plumbline_ifc::import`), `model.rs`
(`Bring::Model`), `panels.rs` (how Bring in lists files and how a question with two
answers is asked elsewhere in the window: find one and use its way).
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-210W-report.md. Commit nothing, push nothing.

## Before any code, in your report
1. What `plumbline_ifc::import_choosing`, `Bring`, `Brought::not_asked_said`,
   `Brought::clashes_said` and `Brought::left_out_said` give, quoted from the code.
   The window says the model's sentences: it words nothing of its own about what
   came in, what was not asked for, or how many clashes there are. If a sentence
   the window needs is missing or reads as a command line's (a verb, a flag), say
   which and stop on that point; do not write it in the window.
2. How the window will know the job "already has a level with something on it"
   (ask the model; if it has no such question, say what is needed and stop on that
   point), and where the one piece of state for "a file picked and the question
   not yet answered" lives. What Esc does there: the file is not brought in and
   the window says so.

## What is built (call 210, point 6, every clause)
1. "Designer's model" picked, a file picked, and the job has a level with
   something on it: the window asks, in a panel of the kind it already uses,
   **"Bring in everything"** and **"Only its runs and equipment"**, with a line
   under each saying what it means (everything: walls and floors that lie over the
   job's own are left out and named; only runs: its ducts, pipes, conduit, cable,
   their fittings, the fixtures and equipment they reach; walls, floors, roofs,
   doors, windows and rooms are not asked for). Both are on the map by those names.
   An empty job is not asked: the file comes in whole as today.
2. Either answer calls `import_choosing`; nothing else in the window brings a model
   in. The message is the model's own sentences, whole: what came in by kind, what
   was not asked for or left out, the level it was put on, and the clashes it now
   makes with what the job had, by kind. The last of those says where to see them
   in the window's words (the Clashes tab of the estimate window, by its name on
   screen), not a command.
3. After it: the things brought in are drawn, the Clashes tab lists the new clash
   with the others, and a double-click on it flies to it (as any clash does today).
4. Undo takes the whole bringing-in back in one step, and the job saved after the
   undo is byte for byte the job before.
5. Every message is a builder's words (`tools/gate-language.sh` passes). Every
   existing control keeps its name.
6. `demos/ifcrunsw.sh` (new) to the numbered points in STATUS.md, with its pictures
   in `demos/captured/ifcrunsw/`, and its row in `plumbline/README.md`. The pictures
   a person will look at: the question as asked; the house with the duct and the
   air handler in it and the message; the Clashes tab with the new hole to cut;
   the view flown to it.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/ifcrunsw.sh`,
  `demos/captured/ifcrunsw/`, `demos/window.sh` (only to add a helper, changing
  none), `plumbline/README.md` (a row). Nothing in `crates/world`, `crates/ifc` or
  any other crate.
- No number, meaning or file format changes, and no existing expectation. Tests
  first where a thing can be tested without a screen, run red, quoted.
- Window sessions run in the FOREGROUND (a timeout of at most ten minutes each; a
  long demonstration is cut into sessions); never hand back while one is running;
  leave no timers behind. Freeze a copy of the two programs before window runs
  (`PLUMBLINE_BIN`); delete old frozen copies by literal absolute path; no cargo
  beside a timed run; a failure is run once more before it is believed. Watch the
  disk (`df -h /`; under 4 GB free delete your own old test programs by literal
  path).
- Look at every picture yourself, as a person opening the program for the first
  time. Say plainly whether the question reads as a builder would ask it, whether
  anything is under a panel, and what is not right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop
  and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
`cargo test --workspace` passes (count); `ifcrunsw`, `bringw` and `moneyw` pass on
your frozen build, or stand recorded with what was seen; the three root gates and
`tools/licenses.sh` pass; the report has the two "before any code" answers, says
what each picture shows and what is not right.
