# Working on this repository

This codebase is yours. You own it, you will maintain it, and its design is your call.
Product scope and the fixed rules belong to Patrick.

Read this first, every session. A session does not remember the last one; the repository
does. Each rule is here once. Patrick's rulings of 2026-11-11 are folded in and override
anything older that conflicts (`plumbline/DECISIONS.md` call 42 has them as he gave them);
so are his of 2026-11-12 (call 57) and of later that day (call 62, which replaced the
look and asked for this file's last section).

## What this is

**plumbline** (`plumbline/`): a 3D estimating program for the whole house, every trade.
The job is one world; the 3D view, the estimate sheet, the reports and the lists are
views of it. `plumbline/STATUS.md` says where the build is, `plumbline/SPEC.md` what the
model is, `plumbline/DECISIONS.md` what was decided and why.

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

**How it should feel** (Patrick, call 62): "editing software for construction that
feels like playing a hyper realistic video game"; "more like an Apple product with
windows depth"; "think Google Earth feel".

**The loop.** Traced values start as Assumed. A clash between Assumed elements is a
field question; a clash between Known elements is a finding. The program produces a
verify-in-field (VIF) list: each question, where it is, and the dollars riding on it,
sorted by dollars. A field measurement turns Assumed into Known.

**The earlier program** (roofnerd: `src-tauri/`, `packages/`, the files at the root) is
the lineage this came from. It is not being built on. Its own notes are at the end.

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
  sends no job data and no costs anywhere. The code lives in the cloud, on this branch.
- **Flag, never fabricate.** Every value is Known, Assumed or Unknown and says where it
  came from. Nothing is silently zero, silently dropped, or shown as Known when it rests
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
   makers' sheets, everything exported from the window. `plumbline/STATUS.md` ("The
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
11. The visual judge passes every screen against `plumbline/VISUAL.md` (to be rewritten
    to call 62's standard before it is judged; the file as it stands is the old one).
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
- **Write each demonstration before building what it demonstrates**, in STATUS.md, with
  the figures worked by hand.
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
  report. Every change to an existing behaviour or number is named in STATUS.md.
- **Anything meant to survive is written to disk in the turn it is produced.**
- **Report by updating STATUS.md** at each stopping point: what was built, what was run
  and what it showed, what is next, what is not right. Reporting is not stopping.

## Running plumbline

    cd plumbline
    cargo test --workspace                      # the model, the command line, the window's crate
    tools/portmap.sh                            # every public item against BIMCraft's source
    tools/licenses.sh                           # every linked crate's licence
    tools/committed.sh                          # the last commit builds and passes from what is in it; before a push
    demos/draw.sh                               # each demonstration is a script; README.md lists them
    cargo run -p plumbline -- house house.json  # a made-up whole house
    cargo run -p plumbline-app -- house.json    # the model window and the sheet window

From the repository root, the three gates that cover the whole tree:

    tools/gate-language.sh
    tools/gate-egress.sh
    tools/gate-clientdata.sh --all              # also a pre-commit hook: git config core.hooksPath .githooks

The window demonstrations run on a virtual display (`xvfb-run`, `xdotool`, Mesa's
software Vulkan). On that renderer a key or a click that begins and ends inside one slow
frame can be lost; the scripts hold each one longer than a frame.

`plumbline/refs/` holds Patrick's brief and the first handoff. It is local-only; if it is
empty on a clone, the rules above still stand.

## The earlier program, at the repository root

roofnerd is roofing estimating software in a Tauri shell: `src-tauri/core` solves,
`packages/engine` prices, `packages/app` is the editors. Its own `README.md`,
`STATUS.md` and `DECISIONS.md` are at the root. Work on it only if Patrick asks. If he
does, three things there are not obvious: its webview has no `prompt`, `alert` or
`confirm` (they return nothing and the feature silently does not exist); its runtime
probes go through the start screen with `openDemoJob(session)` from
`tools/probe/tauri-harness.mjs` and never call `doc_open`; and its screen vocabulary is
the list in its `README.md`.

    pnpm install && pnpm test && pnpm gates
    cargo test --workspace --manifest-path src-tauri/Cargo.toml
    pnpm build:app && node tools/probe/front-door.mjs && node tools/probe/runtime-check.mjs

## For whoever builds next

Patrick asked for this section and named Opus 5.5 as the model that builds next. It is
how to use the program and how to change it without breaking what it promises.

**Read, in this order, before touching anything.** This file. Then the end of
`plumbline/STATUS.md`: "Self-review, written cold", "Eighteenth judgment: what it
found" and "The rebuild after call 62" say what is wrong, what an estimator hit, and
what is half built. Then `plumbline/DECISIONS.md` call 62 (Patrick's words and how
each clause was read) and `plumbline/QUESTIONS.md` Q22. The two surveys the rebuild
rests on (what Google Earth, Cesium and building tools do, with numbers; what the
engine gives, with file and line) are summarised in call 63 of DECISIONS.md when it
exists; until then STATUS.md says where they are.

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
plan to a place on screen). Controls are found by name through a map the program
writes when `PLUMBLINE_MAP` is set (`crates/app/src/map.rs`): **keep every control's
name when restyling it, or every script that clicks it breaks.** The renderer is
software and slow (about a frame a second at worst): a click that begins and ends
inside one frame is lost, so the kit holds each one. F2 saves. Do not run cargo in
the tree while a timed window demonstration is running; build in a scratch worktree.

**Where each thing is changed.**

| To change | Go to |
|---|---|
| a colour, a size of type, a corner, a shadow, a word on a button | `crates/app/src/theme.rs` |
| what panels there are and what is in them | `crates/app/src/panels.rs` (model window), `crates/app/src/sheet.rs` (estimate window) |
| the camera, the light, how the house is drawn, what a click on the model does | `crates/app/src/scene.rs` |
| a drawing tool (wall, door, run, move), its fields, what typing does | `crates/app/src/tools.rs`, and the entry state in `crates/app/src/main.rs` |
| the patterns the house is dressed in (brick, lap siding, shingles) | `crates/app/src/dress.rs`; what finish a priced line names: `crates/world/src/finish.rs` |
| bringing a file in from the window | `crates/app/src/bring.rs`, `crates/app/src/plans.rs` |
| what the model holds and its rules | `crates/world/src/building/model.rs`, `components.rs`, `check.rs`; the saved job: `file.rs`, and `SPEC.md` |
| a quantity, a formula on a line, a price | `crates/world/src/lines.rs`, `formula.rs`, `derive.rs`, `view.rs` |
| clashes; the verify-in-field list | `crates/world/src/building/clash.rs`; `vif.rs` |
| a maker's sheet read into an item | `crates/plans/src/lib.rs` (`words`), `crates/world/src/spec.rs` |
| the workbook, CSV, the three lenses; IFC in and out | `crates/world/src/export.rs`, `crates/sheets`; `crates/ifc` |
| a command-line verb and what it prints | `crates/cli/src/main.rs` |

**Change it the way that has held.** In this order, every time:

1. Write the demonstration first, in STATUS.md, with the figures worked by hand.
2. If the change alters something a person sees or a number, write the decision in
   DECISIONS.md before changing any expectation. A wording question is yours; decide
   it and write it down.
3. Build it. One arithmetic for one number: the model works a figure out, a window
   only shows it. Every value is Known, Assumed or Unknown and says where it came from.
4. Run `cargo test --workspace`, the demonstrations, and the gates listed under
   "Running plumbline" (`tools/portmap.sh` after any new public item in
   `crates/world/src/building*` or `editing*`: say where in BIMCraft you looked).
5. Look at every picture yourself, as a person opening it for the first time.
6. Have an agent that did not build it try to break it, and record what it finds in
   STATUS.md the turn it arrives. Tell a judge to write its notes as it goes: a usage
   limit can cut one off mid-run, and the eighteenth was cut off once.
7. Say in STATUS.md what changed and what is not right. `tools/committed.sh`, then
   commit and push to the branch.

**What the last builder got wrong, so you do not.** Seventeen judgments went to the
command line and the arithmetic and one, cut short, to the window a person uses. The
window demonstrations typed sizes the one way that worked, so they never met what the
first estimator met in his first hour: a tool that throws typing away and places a
different size without a word (J18-1, J18-2). Walk the path a stranger walks, not the
path you know. And the look was built to a written rule and judged against that rule
rather than against whether anyone would want to open it. Patrick's test is his own
eyes: when he says he does not see it, it is not there.

**Small things that cost time.** Disk is a fixed allowance: `target/` grows past 12
GB; delete `target/debug/incremental` and scratch targets when a link fails. The
harness refuses `rm -rf` on a shell variable: use the literal path. IfcOpenShell is
LGPL and is only ever run as an outside checker (`tools/ifc-check.py`), never linked.
The default branch has none of this; the README's clone line names this branch.
