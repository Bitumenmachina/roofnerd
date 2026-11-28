# Contract 240B2: every kind of file, in and out, from the window (window builder)

**Not issued until part B1 (contract 240B1) and the files crate (contract 242M) are
both accepted; the lead names the commit.** You are the one window builder, in
/home/user/roofnerd. Read `CLAUDE.md` whole, then `plumbline/DECISIONS.md` calls 220,
237 (step 6), 240 and 242 (every point, with the "as built" of 242M and of B1),
**`demos/filesw.md` (your demonstration, written first)**, `demos/kind.md`, and
`crates/files` as it stands: every function you call is there already. Notes as you
go to `SCRATCH/review/W-240B2-report.md`, after every run. Commit nothing, push
nothing.

## Before any code, in your report

1. **What the window spells itself today**: every place in `crates/app/src/bring.rs`
   and `plans.rs` that reads or writes a file by its own lines and not through
   `crates/files`, with the function in `crates/files` that takes its place. Where
   there is none, say so and stop on that kind: the lead rules (you write no reader
   and no writer; the window calls one).
2. **The steps of `demos/filesw.md` run red**: `demos/filesw.sh` written to the
   statement and run on the tree as B1 left it. Steps 1 to 10 must fail for want of
   the entry, not for want of the script; the line each fails on.
3. **What the sheet must ask, kind by kind**: for each kind, what the command line's
   verb takes besides the file (a level, a page, how sure, where from), from the
   verb's own flags. A thing the verb needs and the sheet has no field for is on
   this list, not guessed.
4. Every existing demonstration that brings a file in or writes one out through the
   window (`grep` the scripts for the controls under Bring in and Write out): the
   list, and that each still presses a control by the name it has today.

## What is built

1. **Bring in is one list.** The sheet B1 made lists the files of the job's folder,
   each with what it is in the words of `crates/files`' `kind` (the same function
   `plumbline kind` prints from; no second list of endings or headings in the
   window). One button, **Bring in**. Starters are rows under their own heading. A
   file that is not a kind BuildOS reads is not listed; the foot of the list says
   how many such files there are. A file that does not open is listed with the
   reader's reason and cannot be brought in. A job is listed as a job, to open.
2. **A file that could be two things is asked about once**, in the sheet, before
   anything is read, in the order `kind` gives.
3. **What a kind needs is asked in the sheet**: a plan its level and its page; a
   scan its level; prices how sure they are and where they came from. What is
   offered first is what the command line takes when the flag is left off.
4. **Every kind the command line brings in comes in**: a CAD drawing (DXF) and a
   TIFF as a plan; LAS, LAZ and E57 as a scan; the measured marks on a PDF (the
   row's count, **Bring in its marks** and **Counts too** on the drawing's panel);
   a maker's model; My database; the field's sheet; with the seven it takes today.
5. **A CAD drawing's layers** are rows under the plan's own row in Layers, one
   switch to a layer, by the file's own names.
6. **Write out is one list**, grouped by who it is for, with the rows and the
   words of call 242, point 5: what it writes today, and the issue list (BCF), the
   3D file (glTF), the plan as CAD (DXF) for the level in view, the field's sheet
   and My database (the cabinet rules travel with it). One button, **Write
   out**. The file is written beside the job; the window says where in the words
   the command line says.
7. **What the window says after each is what `crates/files` gives back**: what was
   read, what was left out and why. No sentence is written a second time in the
   window. A file that cannot be read is refused with the reader's reason and the
   job is as it was (Undo has nothing new to undo).
8. `demos/filesw.sh`, with a row in `README.md`.

## What must not change

- **The job the window saves is byte for byte the job the command line saves from
  the same file** (the header's time set aside), and a file the window writes is
  byte for byte the command line's. That is the check of every step; where it does
  not hold, the window is wrong, not the check.
- No number, no meaning, no file format, no sentence of `crates/files`, no
  expectation of an existing test or demonstration. Every control that was a button
  under Bring in or Write out keeps its name on the map and is reached through its
  way. A new word on screen is listed in your report for the lead.
- **Nobody types where a file lies.** No field takes a path.
- The program opens no connection (`tools/gate-egress.sh`).
- Files you may touch: `crates/app/src/*.rs`, `crates/app/Cargo.toml` (the one line
  for `crates/files`), `demos/filesw.sh`, `demos/window.sh` (helpers only),
  `demos/quietw.sh` (step 11's count, if the lead's statement names it), `README.md`
  rows. Not `crates/files`: if it lacks something, say what in the report.

## How it is run

As contract 240B1: every cargo command through `tools/quiet.sh`, every window session
through `tools/window-run.sh`, long commands in the background, nothing left running.
**Look at every picture as a builder with a folder of files would**: can he tell
what each file is before he presses anything; is the sheet quiet with thirty files
in the folder; does the house keep the window when the sheet is closed. Pictures for
the lead in `SCRATCH/review/240B2/`: the Bring in sheet on a folder with one file of
every kind; the question for a file that could be two things; a CAD drawing under a
level with one layer off; the Write out sheet; a refusal. If a tool asks a
permission you do not have, stop and say so in the report; do not ask the lead to
do it.

## Done when

`cargo test -p plumbline-app` passes (the count); `demos/filesw.sh` passes its twelve
steps under `tools/window-run.sh`; `quietw` passes as B1 left it (ten kinds are rows,
not buttons: the count at rest is what it was); every window demonstration that
brings in or writes out has been run on your last frozen build and passes; the three
gates and `tools/licenses.sh --check` pass; the report has the four "before any
code" answers, the results, the pictures and what is not right.
