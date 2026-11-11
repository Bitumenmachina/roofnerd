# Working on this repository

This codebase is yours. You own it, you will maintain it, and its design is your call.
Product scope and the fixed rules belong to Patrick.

Read this first, every session. A session does not remember the last one; the repository
does. Each rule is here once. Patrick's rulings of 2026-11-11 are folded in and override
anything older that conflicts (`plumbline/DECISIONS.md` call 42 has them as he gave them).

## What this is

**plumbline** (`plumbline/`): a 3D estimating program for the whole house, every trade.
The job is one world; the 3D view, the estimate sheet, the reports and the lists are
views of it. `plumbline/STATUS.md` says where the build is, `plumbline/SPEC.md` what the
model is, `plumbline/DECISIONS.md` what was decided and why.

**The mission.** Trace the drawings we are given, build the 3D house from the tracing,
and show the clashes the drawings hid. A veteran with simple field tools then confirms or
clears each one.

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
  designers; the program's own job file. The job file and CSV round-trip without loss.
  For any other format, loss is reported, never silent.
- **Speak construction.** Every word on screen is a builder's or estimator's word. Units
  are written the way trades write them: SF, LF, EA, SQ, LS, LB, sheets, rolls, boxes, M.
  No software term appears anywhere a user sees.
- **Teach by doing.** A new user completes a first takeoff from a PDF without a manual.
  Every number can show what it is made of and where it came from. Depth is on demand:
  macro to micro means zooming into the same job, never changing screens.
- **The look is an instrument.** Edges are drawn by line type, surfaces are matte,
  layouts are dense, and the plan and the sheet read like drawings. 3D stays shaded, with
  studio light and a neutral ground. No sky, sun or grass.
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
11. The visual judge passes every screen against the instrument standard.
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
