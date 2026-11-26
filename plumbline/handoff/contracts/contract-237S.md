# Contract 237S: the long files of the command line's side cut (worker)

You are a worker in the working copy the lead names (it is clean, on the commit the
lead names). Read `CLAUDE.md` whole, then `plumbline/DECISIONS.md` "### Call 237" and
"### Call 238", and `plumbline/STATUS.md`. SCRATCH is named in the message that starts
you. Notes as you go to `SCRATCH/review/W-237S-report.md`, after every file. Commit
nothing, push nothing.

## The job

Eight files are too long to read and are cut into smaller files by subject. **Nothing
else changes**: no number, no meaning, no file format, no word a person reads, no
expectation of a test or a demonstration.

| File | Lines |
|---|---:|
| `crates/world/tests/world.rs` | 6,713 |
| `crates/cli/src/main.rs` | 4,310 |
| `crates/world/src/building.rs` | 3,944 |
| `crates/plans/src/found.rs` | 2,930 |
| `crates/ifc/src/lib.rs` | 2,852 |
| `crates/world/src/editing.rs` | 2,666 |
| `crates/world/src/tables.rs` | 2,332 |
| `crates/plans/src/drawn.rs` | 2,330 |

Not yours: anything in `crates/app` (the window builder has it).

## Before any code, in your report

For each file: the subjects in it (a list of its items with their line ranges, grouped),
the files you will make and what goes in each, and what keeps every outside path as it
is. Aim for no file over about 1,200 lines; where a subject is honestly one thing and
longer, say so and leave it whole rather than cut it in an arbitrary place.

## How it is done

- **A pure move.** An item's body is carried over byte for byte. What may change: the
  `mod`, `use` and `pub use` lines, and a visibility word (`pub(crate)`, `pub(super)`)
  where an item now has to be reached from a sibling file. Nothing is renamed. Nothing
  is "tidied while there".
- **Every path that exists today still exists.** `plumbline_world::building::X`,
  `editing::Y`, `tables::Z` and the rest resolve as before (`pub use` from the old
  place). `tools/portmap.sh` reads public items by path against `PORTMAP.md`: it must
  pass, and where `PORTMAP.md` names a file an item is in, the file's new name is put
  there and nothing else in that file is touched.
- **Tests keep their names and their number.** `tests/world.rs` becomes several test
  files by subject; every `#[test]` function is there once, under the name it had.
  Helpers the tests share go in `tests/common/mod.rs`.
- One file at a time. After each: build, run that crate's tests, write the proof
  below into your report, then go on.

## The proof, for each file cut

1. `tools/check.sh` at the end, whole, with `BIMCRAFT=/home/user/ipedrax/bimcraft`:
   it must end `check: PASS`, and the counts of tests must be what they are today (the
   model 759, the command line 94, the workbook 7, the BIM file 131, the roof 122, the
   plans 231), the command-line demonstrations 69, the sample whole house
   880d9fd3c364b5f7.
2. **That it is a move and nothing else**: for each file cut, the sorted non-blank
   lines of the old file (`git show HEAD:<path>`) against the sorted non-blank lines of
   the new files together. Every line that differs is quoted in your report and is a
   `mod`, `use` or `pub use` line, a visibility word, or an indentation change from a
   moved `mod tests` block. Write the small script that does this in SCRATCH, not in
   the repository, and say how to run it.
3. The names of all `#[test]` functions in `crates/world/tests` before and after,
   sorted: the two lists are the same.

## Rules

- Files you may touch: the eight above and the new files beside them; `PORTMAP.md`
  (file names only); `crates/*/src/lib.rs` or `main.rs` for `mod` lines. Nothing else.
  `CLAUDE.md` has a table "Where each thing is changed" that names some of these
  files: do not edit it; list in your report which rows now point at a new file.
- **Every cargo command goes through `tools/quiet.sh`** (`tools/quiet.sh cargo test -p
  plumbline-world ...`), with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0` and
  `-j2`. It waits while a window demonstration holds the machine; that wait can be an
  hour and more. It is not a hang: do not work round it, and use the time to read and
  plan the next file.
- Disk: `df -h /` before every build; under 5 GB free, delete your own old test
  programs (`target/debug/deps`, files over 20 MB that are not libraries, by a
  literal path). `tools/check.sh` deletes each test program after it runs.
- Run long commands with `run_in_background` and wait for the notice; leave no timers
  and no background jobs when you hand back.
- If a tool asks a permission you do not have, stop and say so in the report; do not
  ask the lead to do it. No web search is needed for this job.

## Done when

`tools/check.sh` ends `check: PASS` with the counts above; the three proofs are in the
report; the report lists the files made, the longest file left and why, and the rows
of CLAUDE.md's table that moved.
