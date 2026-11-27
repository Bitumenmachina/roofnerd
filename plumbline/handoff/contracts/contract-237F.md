# Contract 237F: every kind of file, in and out, from the window (window builder)

**Not issued as written. Call 242 replaces it: the file kinds come in through the Bring in and Write out sheets (contract 240B2, over `crates/files` from contract 242M). Kept as the record of what was first meant.**

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read `CLAUDE.md` whole ("The main thing to
remember"; "Imports matter as much as exports"; "Speak construction"), then
`plumbline/DECISIONS.md` "### Call 237", `plumbline/STATUS.md`, and
**`demos/filesw.md`: it is your demonstration, written first, with the words on every
button already decided.** Then in `crates/app/src`: `bring.rs` and `plans.rs` (how a
file is picked from a list and brought in today), `panels.rs` (the Bring in and Write
out parts of the Job panel; a drawing's panel), `map.rs`; and in `crates/cli/src` how
each command of `demos/filesw.md` calls the model (`grep -n` the command's word): the
window calls the same function of `plumbline_world`, `plumbline_plans` or
`plumbline_ifc`, with the same arguments. SCRATCH is named in the message that starts
you. Notes as you go to `SCRATCH/review/W-237F-report.md`. Commit nothing, push nothing.

## Before any code, in your report

For each of the ten steps of `demos/filesw.md`: the command line's function and where
it is called (file and line); the function the window will call; where the sentence the
window says comes from (it must be the model's own, not written again); what, if
anything, the command line does in its own file that the window cannot reach, and so
must first be moved into a crate both link (say which, and that nothing it does changes).

## What is built

The ten rows of the table in `demos/filesw.md`, each by its button, and
`demos/filesw.sh` to its ten steps, in sessions of at most ten minutes, each check a
checksum of the window's saved job (or written file) against the command line's from the
same file. Tests first where a thing can be tested without a screen. Its row in
`README.md`.

## Rules

- **No second arithmetic.** If the window's saved job differs from the command line's by
  a byte, the window is wrong; do not change the check.
- Files you may touch: `crates/app/src/*.rs`, `demos/filesw.sh`, `demos/window.sh` (only
  to add a helper), `README.md` (a row). A function moved out of `crates/cli/src` into a
  crate the window links is a pure move, said in the report with the proof that the
  command line's tests and demonstrations pass unchanged (`tools/check.sh`).
- No number, no meaning, no file format, no existing expectation changes. Every word on
  screen is in the table; `tools/gate-language.sh` passes. Every control keeps its name
  on the map; new ones are on it by the name in the table.
- Every cargo command through `tools/quiet.sh`; every window session through
  `tools/window-run.sh`. Long commands with `run_in_background`; wait for the notice.
- Look at every picture, as a person opening the program for the first time. Say what
  is under a panel, what cannot be read, what is not right.
- If a tool asks a permission you do not have, stop and say so in the report; do not
  ask the lead to do it.

## Done when

`cargo test -p plumbline-app` passes (the count); `filesw`, `bringw`, `scanw`, `planw`
and `firstw` pass by `tools/window-run.sh` on your last frozen build; `tools/check.sh`
ends `check: PASS`; the three gates pass; the report has the "before any code" table,
`target/window-run/results.txt`, what each picture shows and what is not right.
