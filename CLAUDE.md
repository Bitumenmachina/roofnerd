# Working on this repository

This codebase is yours. You own it, you will maintain it, and its design is your call.
Product scope and the fixed rules belong to Patrick.

Read this first, every session. A session does not remember the last one; the repository
does. Each rule is here once. Patrick's rulings of 2026-11-11 are folded in and override
anything older that conflicts (`plumbline/DECISIONS.md` call 42 has them as he gave them);
so are his of 2026-11-12 (call 57) and of later that day (call 62, which replaced the
look and asked for this file's last section). His of 2026-11-13 (calls 75 and 76: a
harness for each division) are folded in below. His of 2026-11-25 are folded in too:
call 217 (the lead makes the calls and runs to the finish without him; **two windows
stay, and the second can show plans**; the order of work) and call 220 (the main thing
to remember). His of 2026-11-26 are folded in (call 239: the look is reviewed as the
function is, and by other eyes than the builder's; call 237: the repository was not
acceptable as it stood; it is cleaned before anything new is built; **one manual,
changed in place, takes the place of pictures piling up**).

## What this is

**BuildOS** (Patrick's name for it, 2026-11-17; its working name was plumbline, and the folder `plumbline/`, the crates and the two commands are still called that: `plumbline/DECISIONS.md` call 115): a 3D estimating program for the whole house, every trade.
The job is one world; the 3D view, the estimate sheet, the reports and the lists are
views of it. `plumbline/STATUS.md` says where the build is (short, and rewritten in
place: call 238), `plumbline/SPEC.md` what the model is, `plumbline/DECISIONS.md` what
was decided and why (`DECISIONS-index.md` finds a call). `plumbline/STATUS-history.md`
is everything STATUS.md held before 2026-11-26, under the headings it had: a script,
a contract or a decision of before that day that names STATUS.md and a heading means
that file.

**The focus is a full 16-division building platform.** The whole building is estimated,
every division from General Requirements to Electrical: each cost line carries its
division, the estimate and its recap read division by division, and each division has
assemblies behind it. Clash detection matters and is one of its tools; it is not the
centre. (The sixteen are MasterFormat 1995's; `plumbline/DECISIONS.md` call 57 has the
list and its source.)

**The mission.** Trace the drawings we are given, build the 3D house from the tracing,
price it division by division, and show the clashes the drawings hid. A veteran with
simple field tools then confirms or clears each one. Things are put in the house from
their makers' sheets (mechanicals, fixtures, furniture), picked up and moved, and the
clashes light and clear as they move: "the whole point of clash is also moving a
bathroom vanity around". The building goes out as a BIM file (IFC).

**The main thing to remember** (Patrick, 2026-11-25, `plumbline/DECISIONS.md` call
220): "user will be able to bring any of their file types that are common to AEC in,
export out similar, they own their database and it has the depth of a full high end
home custom build but without overwhelming field guys and new users, clean code clean
screens". Four tests for anything built: does a file a builder already has come in,
and go out again in a form another program takes; is every table of the program (items,
shapes, assemblies, prices, rules) theirs, in the open, written out and brought back
whole; is the depth there for the estimator and out of the way of the man in the
field and the first-time user; is the screen clean and the code clean.

**How it should feel** (Patrick, call 62): "editing software for construction that
feels like playing a hyper realistic video game"; "more like an Apple product with
windows depth"; "think Google Earth feel".

**The loop.** Traced values start as Assumed. A clash between Assumed elements is a
field question; a clash between Known elements is a finding. The program produces a
verify-in-field (VIF) list: each question, where it is, and the dollars riding on it,
sorted by dollars. A field measurement turns Assumed into Known.

**Each division will have its own AI harness** (Patrick, calls 75 and 76): its own
knowledge base, pricing and tweaking, run locally beside the program. The program is
built so that a division can be handed to one and taken back whole
(`plumbline/PROPOSALS.md` 21; not built yet). `plumbline/divisions/README.md` is what
the program has for each of the sixteen today.

**The earlier program** (roofnerd, roofing estimating in a Tauri shell) is the lineage
this came from. It is not being built on and it is not on this branch: it is on the
branch `main`, whole, and left this branch on 2026-11-26 (call 237).

## Patrick's fixed rules

- **Open source, permissive licences only.** GPL, AGPL and LGPL code is reference only
  and is never linked. `plumbline/tools/licenses.sh` checks it.
- **Real job, client or supplier data never enters the repository.** Nothing out of a
  real report: no bid names, job numbers, home paths, quantities, prices, hours or
  margins, in code, tests, comments, demonstrations or commit messages. Invent the
  number; it reads the same. `refs/`, `fixtures/` and `evidence/` are ignored by git.
  Anything placed in `refs/` is sanitized first: the model this is built with keeps data
  for 30 days.
- **Local-only is about job data and costs when people use the program.** The program
  sends no job data and no costs anywhere. A harness is a thing beside the program;
  the program itself never opens a connection, and gives a harness its own division
  and nothing else. The code lives in the cloud, on this branch.
- **Flag, never fabricate.** Every value is Known, Assumed or Unknown and says where it
  came from. What a division's harness writes arrives Assumed, with the harness
  named as its source, and is never Known on a harness's say-so (call 76). Nothing is silently zero, silently dropped, or shown as Known when it rests
  on something that is not. A total that leaves something out says what it left out.
  Prove it at every layer a number can collapse.
- **Imports matter as much as exports.** People bring in what they already have: PDF
  plans, vector and scanned; PNG and JPEG plans and aerial images; CSV and XLSX supplier
  price lists, prior estimates and assembly libraries; IFC models from other trades and
  designers; manufacturers' sheets, each made into an item to place (its sizes Assumed
  and sourced to the line of the sheet, what the sheet does not say Unknown); the
  program's own job file. A file is picked from a list; nobody types where a file lies.
  Everything that can be written out can be written out from the window. The job file and CSV round-trip without loss.
  For any other format, loss is reported, never silent.
- **Speak construction.** Every word on screen is a builder's or estimator's word. Units
  are written the way trades write them: SF, LF, EA, SQ, LS, LB, sheets, rolls, boxes, M.
  No software term appears anywhere a user sees. **US construction language is a known
  language and the builder of this program chooses the words.** A wording question is
  never put to Patrick: decide it, write the call down, and go on.
- **Teach by doing.** A new user completes a first takeoff from a PDF without a manual.
  Every number can show what it is made of and where it came from. Depth is on demand:
  macro to micro means zooming into the same job, never changing screens.
- **The window is glass over the model, with depth.** The model fills the window. What
  is on screen with it is a layer of frosted, translucent, rounded panels floating over
  it, each with a soft shadow, each able to be put away; nearer things read nearer. It
  moves like Google Earth: the wheel dives at what is under the pointer, a drag slides
  the ground with the hand, everything eases and carries momentum, a double-click flies
  to a thing, and no view ever jumps. A panel scrolls smoothly under the wheel and
  shows where it is. Every layer of the job, and every panel, has a switch that reads
  on or off at a glance. (This replaces "the look is an instrument": flat, dense and
  matte was shown to Patrick and read to him as no better. On a plan, edges are still
  drawn by line type and the plan still reads like a drawing.)
- **Quiet, with room** (call 239). The window is not busy and its tools are not
  cramped: the model keeps most of the window, a first screen shows the few things a
  person needs first and the rest is one step away, and a tool can be read at a
  glance. `plumbline/VISUAL.md` is to hold the numbers, written from the review call
  239 orders and not before it. Function does not buy the look and
  the look does not buy function: both are owed, and a window job is not accepted
  until an agent that did not build it has read its pictures against the standard.
- **It looks real.** Physically based materials, soft shadows, light that reads as a
  place. Whether there is a sky is `plumbline/QUESTIONS.md` Q22; the working answer is
  a soft sky and horizon, and no grass, trees or scenery.
- **The 3D house looks like the house.** It is dressed as it will be built, to the
  standard of high-end residential work: siding, masonry and stone, roofing, windows
  with their frames, sashes and glass, doors, trim. What it is dressed in is what the
  estimate prices.
- **See it as a person would.** Walk the program cold, as someone opening it for the
  first time, and read every screen as they would. A figure proven by a script is not
  a screen a builder can use.
- **New features go to `plumbline/PROPOSALS.md`.** Product scope is Patrick's.
- **Judges and every other subagent run on Opus or Sonnet**, never on the model that
  builds.

## What needs no approval, and what does

Run autonomously. Search the web whenever it helps. These need no approval:

- anything in the repository or the cloud workspace;
- installing packages and toolchains there;
- pushing to the branch;
- starting judges.

**Ask Patrick before** rewriting branch history or touching any other branch. No pull
request unless he asks for one.

Log each call in `plumbline/DECISIONS.md`. A question goes to `plumbline/QUESTIONS.md`
with the working assumption, and the build keeps going.

## Order of work

The order is Patrick's; how each step is done is yours.

1. Finish fittings and clashes, and have them judged.
2. PDF underlay: calibrate from one dimension, trace, and snap to vector lines where they
   exist.
3. The remaining imports, plus a language pass over every screen.
4. Field entry. Any field that takes a length accepts a laser in keyboard mode (a Leica
   DISTO types the reading at the cursor, with units). Marking a value Known works in the
   window. (Bosch GLM over Bluetooth is in PROPOSALS.md.)
5. Floors, rooms and roofs drawn in the window.
6. The VIF list and the teach-by-doing first job.
7. The visual standard and a visual judge.
8. IFC export, xlsx export and publishing.
9. The window rebuilt to call 62: glass with depth, Google Earth navigation, switches,
   smooth scrolling, a thing picked up and moved with its clashes live, items from
   makers' sheets, everything exported from the window. `plumbline/STATUS-history.md` ("The
   rebuild after call 62") has the plan, part by part, and where it stands. This comes
   before the release candidate: items 9 to 12 of the deliverable are judged on the
   rebuilt window.

## The deliverable: a release candidate

It is done when `plumbline/STATUS.md` shows each item below with evidence (screenshots or
a recording, plus command output). Then tag the release candidate on the branch, stop,
and ask Patrick for the production ruling.

1. A clean clone installs on Linux by following the README in one pass.
2. Every import in the fixed rules works as stated.
3. Starting from an empty job, it opens a real residential plan PDF (a public set whose
   licence allows it, or a sanitized set Patrick places in `refs/`), calibrates it, and
   traces walls, openings, floors, rooms and the roof. The 3D house builds from the
   tracing.
4. Clashes show their certainty, and the VIF list is sorted by dollars.
5. A typed or lasered measurement makes a value Known. The clash confirms or clears, and
   the price changes live in the second window.
6. A design-phase price per SF or LF grows into line items for the same condition, and
   the detail rolls back up.
7. The estimate sheet has stretch-out editable on the line, labor switchable per line,
   editable overhead and profit, and three suppliers side by side.
8. Exports: xlsx with live formulas, CSV, and IFC4 validated by IfcOpenShell. Supplier,
   subcontractor and PM lenses come out of one estimate.
9. A judge lists every word on every screen, and none is a software term.
10. A fresh judge, briefed as a veteran estimator with no software background, completes
    a whole-house takeoff from a PDF using only the in-app help.
11. The visual judge passes every screen against `plumbline/VISUAL.md` (rewritten to
    call 62's standard; it names the lines the window is known to fail today).
12. A fresh judge walks the whole path cold and cannot break the money path.
13. A hand estimate in `refs/` is reproduced, with every difference explained line by
    line. If `refs/` has none, this item is marked pending.

## How the work is done

Each of these was paid for at least once.

- **Search before asking: the machine, then what was already surveyed, then the web.**
  Where a tree has an index or a manifest, that is the first read. A claim that
  something is missing, made without that search, is a defect. A claim about another
  product or a standard is checked against its source, never written from memory.
- **Do not invent what already has a name.** Construction is a solved trade with
  published standards and a settled vocabulary. Coining a word, a table or an algorithm
  for something that has one is drift that looks like progress.
- **Port from source that is open in front of you.** What comes from BIMCraft is listed
  in `plumbline/PORTMAP.md`, item by item, and `plumbline/tools/portmap.sh` checks it.
  Owning the code includes keeping that map true when the code is moved.
- **Write each demonstration before building what it demonstrates**, in
  `demos/<name>.md` beside its script, with the figures worked by hand.
- **One arithmetic for one number.** A quantity that can be worked out from the model is
  worked out in the core; no window recomputes it. A formula is visible and editable on
  the line. The stretch-out belongs to the detail, not to the material.
- **A check that bypasses the path a person uses is not a check.** A demonstration
  drives the real program from an empty job or the start of the path, through the
  buttons and the fields.
- **Look at every picture, as a person, before anything is called shown.** A check that
  passes while the window is wrong is a defect in the check. Evidence is the program as
  committed; pictures taken before a change to what they show are retaken or said to be
  stale.
- **Nothing is done until an agent that did not build it has tried to break it.** A fix
  is not judged until it has been. The look is judged the same way as the money.
- **Never loosen a gate, and prove a gate red before trusting it.** If one fires on
  legitimate work, fix the work or narrow the pattern and say why. A test is not
  deleted, skipped or weakened to get green; an expectation changes only after the
  decision behind the change is written down.
- **Fix what you find, and say what you changed.** A known defect is not left as a
  report. Every change to an existing behaviour or number is named in the call's
  "as built" entry in DECISIONS.md.
- **Anything meant to survive is written to disk in the turn it is produced.**
- **Report by updating STATUS.md** at each stopping point: what is so now, what was
  run on which commit and what it showed, what is next, what is not right. It is
  rewritten, not added to: what is no longer so comes out, and the story of how it
  got there is the calls in DECISIONS.md. Reporting is not stopping.

## Running plumbline

    cd plumbline
    cargo test --workspace                      # the model, the command line, the window's crate
    tools/check.sh                              # the whole check before work is accepted: tests one program at a time, every command-line demonstration, gates, licences, the sample house
    tools/window-run.sh all                     # window demonstrations, one at a time, on a frozen copy, nothing beside them
    tools/quiet.sh cargo build                  # any cargo command while window runs may be going: it waits for them
    tools/portmap.sh                            # every public item against BIMCraft's source
    tools/licenses.sh                           # every linked crate's licence
    tools/committed.sh                          # the last commit builds and passes from what is in it; before a push
    demos/draw.sh                               # each demonstration is a script; README.md lists them
    cargo run -p plumbline -- house house.json  # a made-up whole house
    cargo run -p plumbline-app -- house.json    # the model window and the sheet window

From the repository root, the three gates that cover the whole tree:

    tools/gate-language.sh
    tools/gate-egress.sh
    tools/gate-clientdata.sh --all              # the three are the pre-commit hook: git config core.hooksPath .githooks

The window demonstrations run on a virtual display (`xvfb-run`, `xdotool`, Mesa's
software Vulkan). On that renderer a key or a click that begins and ends inside one slow
frame can be lost; the scripts hold each one longer than a frame.

`plumbline/refs/` holds Patrick's brief and the first handoff. It is local-only; if it is
empty on a clone, the rules above still stand.

## Who holds what (Patrick asked, 2026-11-17, as the model that built this runs out)

Four seats. One mind does not sit in two of them on the same piece of work.

**Patrick: the owner.** Scope, rulings, the name, real data, and the look. He reads
the pictures and has all along: when he says he does not see it, it is not there.
**What he is shown is the manual** (call 237): one page, `plumbline/manual/`, that
says how each thing is done in the program with one current picture of it and the
state of each part, published to him as one artifact at one address. When a thing
changes, its section and its picture are replaced in place; nothing is added beside
them, and `demos/captured/` is not kept in the repository (a demonstration writes
its pictures there for whoever runs it to read). When a question is his
(`plumbline/QUESTIONS.md`) a ruling from him is a few words and ends days of a
working assumption. Put the question to him early and plainly; do not build round it.

**The lead: one session at a time, the strongest model on hand.** It holds what
breaks everything else when it is wrong, and it is never delegated:
- what a number means: the model's rules, one arithmetic for one number, Known,
  Assumed and Unknown, how a quantity is kept and rounded, what is in the price;
- the saved job and every file written out (the format is a promise to old jobs);
- every decision in `plumbline/DECISIONS.md`, written before an expectation moves;
- the gates and the expectations of tests and demonstrations (never loosened);
- reading what a judge found and deciding what is so;
- what goes in one commit, and in what order, when two risks could meet;
- the truth of `plumbline/STATUS.md`: what was run, on which commit, and what was
  not. Nobody else says "done".

**Workers: any capable model, given a contract.** A piece of work can be handed off
when it has all four: the files it may touch; a check that already exists or is
written first (a test, a demonstration with its figures worked by hand, a gate); a
rule that it changes no number, no meaning, no file format and no expectation; and a
result the lead can accept by reading a diff and its pictures. That covers most of
what is owed: a division's assemblies, items and prices as tables (they arrive
Assumed); a shape for a thing or a detail of the house, one file each; the wording
pass against a list the lead has decided; running the window demonstrations and
reading their pictures; mechanical changes (the rename to BuildOS, splitting a long
file); an installer; a reader or a writer for a file behind a table the lead has
fixed; a survey of what others do, from their own pages.

**Judges: never the mind that built it.** They try to break it cold and write as
they go. They find; they do not decide.

**Where the cost went, so it is not paid again.** Most of the lead's effort on this
branch was not judgment: it was waiting on window runs at a frame a second, reading
contact sheets, and keeping the record. All of that is a worker's. **This
environment and its pictures are the evidence** (Patrick, calls 174 and 217: "please
stop saying this needs a machine the environment with screenshots are fine"): another
machine is not spoken of to him, in a reply, a page or a record. Feel is judged here by
measuring the view's path frame by frame and by strips of pictures across a move.

## For whoever builds next

Patrick asked for this section and named Opus 5.5 as the model that builds next. It is
how to use the program and how to change it without breaking what it promises.

**Read, in this order, before touching anything.** This file. Then
`plumbline/STATUS.md`, whole: it is short, and says where the work stands and what is
owed, in order. Then `plumbline/divisions/README.md` and `plumbline/research/README.md`.
In `plumbline/STATUS-history.md`: "Self-review, written cold", "Eighteenth judgment: what it
found" and "The rebuild after call 62" say what was wrong, what an estimator hit, and
what was half built. Then `plumbline/DECISIONS.md` call 62 (Patrick's words and how
each clause was read) and `plumbline/QUESTIONS.md` Q22. The two surveys the rebuild
rests on (what Google Earth, Cesium and building tools do, with numbers; what the
engine gives, with file and line) are summarised in call 63 of DECISIONS.md.

**Use it.**

    cd plumbline
    cargo run -p plumbline -- sixteen /tmp/h.json    # a made-up whole house, priced in all sixteen divisions
    cargo run -p plumbline -- estimate /tmp/h.json   # the estimate, by division; `plumbline` alone lists every verb
    cargo run -p plumbline -- vif /tmp/h.json        # what to verify in the field, by dollars
    cargo run -p plumbline -- clashes /tmp/h.json
    cargo run -p plumbline-app -- /tmp/h.json        # the two windows
    demos/spec.sh                                    # a demonstration: a script that drives the real program and checks it

There is no screen on a cloud machine. The window is driven on a virtual display
with a real pointer and keyboard, and seen by taking pictures and **reading them**:

    xvfb-run -a -s "-screen 0 2500x980x24" demos/pricew.sh    # pictures land in demos/captured/price

`demos/window.sh` is the kit every window demonstration uses: `open_window`, `on
"Wall"` (click the control with that name), `sheet "Estimate"`, `typed`, `click X Y`,
`shot` (F12 saves a picture of both windows), `park`, `at`/`plan_px` (a place on the
plan to a place on screen). Every message the window gives is written, in the words
on the screen, to `$T/map.txt.said`: check what a person was told there
(`demos/moneyw.sh` does), not only what the saved job holds. Controls are found by name through a map the program
writes when `PLUMBLINE_MAP` is set (`crates/app/src/map.rs`): **keep every control's
name when restyling it, or every script that clicks it breaks.** The renderer is
software and slow (about a frame a second at worst): a click that begins and ends
inside one frame is lost, so the kit holds each one. F2 saves. **No build runs on the
machine while a timed window demonstration is running, in any working copy and at
any priority**: on 2026-11-26 builds in the idle class beside them cost three
demonstrations their clicks and a day's window evidence. `tools/window-run.sh` holds
a lock while it runs and waits for cargo to be gone; every other cargo command goes
through `tools/quiet.sh`, which waits for the lock.

**Where each thing is changed.**

| To change | Go to |
|---|---|
| a colour, a size of type, a corner, a shadow, a word on a button | `crates/app/src/theme.rs` |
| what panels there are and what is in them | `crates/app/src/panels.rs` (model window), `crates/app/src/sheet.rs` (estimate window) |
| the camera, the light, how the house is drawn, what a click on the model does | `crates/app/src/scene.rs` |
| a drawing tool (wall, door, run, move), its fields, what typing does | `crates/app/src/tools.rs`, and the entry state in `crates/app/src/main.rs` |
| the patterns the house is dressed in (brick, lap siding, shingles) | `crates/app/src/dress.rs`; what finish a priced line names: `crates/world/src/finish.rs` |
| the sun, the sky's light, how the range of light is folded to the screen | `crates/app/src/scene.rs` (`SKY_LIGHT`, `SUN_LIGHT`, `SUN_COLOUR`, `FOLD`), `crates/app/src/skylight.rs` (the two pictures the sky lights the house from) |
| what a placed thing is drawn as (a tub, a sink, a range, a receptacle: by a word in its name) | `crates/world/src/building/forms.rs` (twenty-seven forms: ten are BIMCraft's, seventeen ours); a shape as rows of a table: `library/shapes.csv`, `crates/world/src/solids.rs`; drawn in `scene.rs`; DECISIONS.md calls 102, 203, 233 |
| how the view moves (the wheel, a drag, a flight, a double-click, when a label shows) | `crates/app/src/nav.rs`; DECISIONS.md call 218 |
| the window's place and size, folding panels, lettering size, the recovery copy, what is kept between runs | `crates/app/src/screen.rs`; call 222 |
| a plan in the second window | `crates/app/src/second.rs`; call 222 part C |
| the bench (an item looked at alone, its shape, its price) | `crates/app/src/bench.rs`; calls 195, 205 |
| the frosted panels themselves; the trades drawn in the model | `crates/app/src/glass.rs`; `crates/app/src/trades.rs` |
| every table of the job in one workbook, out and back | `crates/world/src/database.rs`; call 223 |
| a price list as rows; a price set by name | `crates/world/src/pricelist.rs`; call 207 |
| measured marks on a PDF read as quantities | `crates/plans/src/marks.rs`, `crates/world/src/marks.rs`; call 227 |
| the field's sheet written out and its measurements brought back | `crates/world/src/fieldsheet.rs`; call 234 |
| which panels are on screen, and their switches at the lower right | `crates/app/src/panels.rs` (`panels_on`, `status_line`); a panel's "more below" strip: `more_below` |
| a footing (a strip footing is held as a wall is, and is never counted as one) | `crates/world/src/building.rs` (search `Kind::Footing`), `editing.rs` (`footings`); DECISIONS.md call 70 |
| how a wall's frame is laid out; what a framing line names | `crates/world/src/building/framing.rs`; drawn in `scene.rs` behind the Framing switch; call 71 |
| bringing a file in from the window | `crates/app/src/bring.rs`, `crates/app/src/plans.rs` |
| what the model holds and its rules | `crates/world/src/building/model.rs`, `components.rs`, `check.rs`; the saved job: `file.rs`, and `SPEC.md` |
| a quantity, a formula on a line, a price | `crates/world/src/lines.rs`, `formula.rs`, `derive.rs`, `view.rs` |
| clashes; the verify-in-field list | `crates/world/src/building/clash.rs`; `vif.rs` |
| a maker's sheet read into an item | `crates/plans/src/lib.rs` (`words`), `crates/world/src/spec.rs` |
| the item library: items by trade as a table, brought in and written out; the starter items; a placed thing following its item, and turned | `crates/world/src/tables.rs` (`items_in`, `items_in_missing`, `items_out`), `library/items.csv`, `crates/world/src/editing.rs` (`placed_follow`, `turn`); in the window `crates/app/src/bring.rs`, the Items list and an item's panel in `panels.rs` (New item, Rename item, Delete item: `tools.rs` `new_item`, `rename_item`); DECISIONS.md calls 105, 107, 109, 111 |
| the workbook, CSV, the three lenses; IFC in and out | `crates/world/src/export.rs`, `crates/sheets`; `crates/ifc` |
| a command-line verb and what it prints | `crates/cli/src/main.rs` |

**Change it the way that has held.** In this order, every time:

1. Write the demonstration first, in `demos/<name>.md`, with the figures worked by hand.
2. If the change alters something a person sees or a number, write the decision in
   DECISIONS.md before changing any expectation. A wording question is yours; decide
   it and write it down.
3. Build it. One arithmetic for one number: the model works a figure out, a window
   only shows it. Every value is Known, Assumed or Unknown and says where it came from.
4. Run `cargo test --workspace`, the demonstrations, and the gates listed under
   "Running plumbline" (`tools/portmap.sh` after any new public item in
   `crates/world/src/building*` or `editing*`: say where in BIMCraft you looked).
5. Look at every picture yourself, as a person opening it for the first time.
6. Have an agent that did not build it try to break it; its notes are a file in
   `research/`, written the turn they arrive, and what is decided about them is a call. Tell a judge to write its notes as it goes: a usage
   limit can cut one off mid-run, and the eighteenth was cut off once.
7. Say what changed in the call's "as built" entry, and bring STATUS.md to what is so
   now, with what is not right. `tools/committed.sh`, then commit and push to the branch.

**What the last builder got wrong, so you do not.** Seventeen judgments went to the
command line and the arithmetic and one, cut short, to the window a person uses. The
window demonstrations typed sizes the one way that worked, so they never met what the
first estimator met in his first hour: a tool that throws typing away and places a
different size without a word (J18-1, J18-2). Walk the path a stranger walks, not the
path you know. And the look was built to a written rule and judged against that rule
rather than against whether anyone would want to open it. Patrick's test is his own
eyes: when he says he does not see it, it is not there.

**A control on a bar that does not scroll** (the top bar, the status line) is given
`Named { scrolls: false }`. With `scrolls: true` the map leaves it out as "scrolled
off its panel" and no script can click it (the three panel switches were built that
way first and were on screen and not on the map).

**Running several window demonstrations in a row.** Each one builds the program
before it starts, so work in the tree changes the program under a long run (it broke
three of four in one afternoon). Copy `target/debug/plumbline` and `plumbline-app` to
a scratch folder and run with `PLUMBLINE_BIN=<that folder>`: the demonstration uses
that program as it is, builds nothing, and says so. Even then, do not commit a
demonstration's pictures until its run has ended: a set taken half way was committed
once and had to be set right. A build beside a timed demonstration can still cost it
a click.

**Before saying BIMCraft lacks something, search all of it.** `grep -rln <word>
crates` and read the whole list. The framing layout was first written up as "ours"
on a list of file names cut short by `head`; BIMCraft's `crates/core/src/timber.rs`
has it. `tools/portmap.sh` checks every "finds nothing" claim and caught a second.

**What Patrick asked to see, and what is still not there** (QUESTIONS.md Q23,
DECISIONS.md calls 69 to 74): one snapshot of each of seven trades. All seven now
have something in the model, each drawn only where the estimate prices it, each with
a switch under Show: electrical, plumbing and HVAC as before; **foundation** (strip
footings, a kind of their own, and foundation walls; the Ground switch; the made-up
whole house has one, priced by what is drawn); **framing** (a wall with a framing
line as its frame; the Framing switch); **waterproofing** (the weather barrier and
opening flashing with the Finishes switch off); **interior woodwork** (base in each
room with base priced). The manual has a picture of each. Since then joists and rafters are
counted and priced (call 165) and cabinets are set by code and by rule (calls 157,
215); still lacking are roof underlayment and roof flashing, interior casing, pad
footings, joists and rafters drawn, and a tool to draw a footing in the window. **The four new layers have one window demonstration** (`demos/tradesw.sh`: the
Ground, Finishes and Framing switches and a level looked into, each measured, and
the concrete of the foundation wall with the ground away; it does not yet measure
the flashing, the base or the footing itself), **and
the twenty-first judgment (2026-11-16) walked them to the end**: its findings and
where each stood are in STATUS-history.md. What is owed next is in STATUS.md: the
look and the words were last judged on 2026-11-16 and failed, and its table of
thirteen says where each item is today.

**A tool's sizes accepted, then a length typed** is the path every first wall takes,
and no demonstration walked it until `planw` was repaired: it found the first digit
of the length thrown away (STATUS-history.md, 2026-11-13). When a demonstration is written,
make it accept what is offered at least once, not only type over it.

**Small things that cost time.** Disk is a fixed allowance: `target/` grows past 12
GB; delete `target/debug/incremental` and scratch targets when a link fails. Each
frozen copy of the program kept for `PLUMBLINE_BIN` is half a gigabyte: delete the
old ones (the machine was restarted once at 92 % full). The
harness refuses `rm -rf` on a shell variable: use the literal path. IfcOpenShell is
LGPL and is only ever run as an outside checker (`tools/ifc-check.py`), never linked.
The default branch `main` has none of this (it holds the earlier program). The same
commits are pushed to two branches, `claude/hopeful-cannon-kng8eo` and
`claude/nifty-tesla-qmn44n`; the README's clone line names the second, which is the
one Patrick was first given.
