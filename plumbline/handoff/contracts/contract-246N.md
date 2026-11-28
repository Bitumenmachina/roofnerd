# Contract 246N: the name BuildOS through the code (worker; mechanical)

**Not issued until the layout (240B1, 240B2, 240C) and the cut of the window's three
long files are accepted, and no other working copy is open; the lead names the
commit.** You work alone in the copy the lead names. Read `CLAUDE.md` whole, then
`DECISIONS.md` calls 115, 174 (point 4) and **246 (every point: it is the whole of
what you do)**. Notes as you go to `SCRATCH/review/W-246N-report.md`, after every
run. Commit nothing, push nothing.

## Before any change, in your report

1. **Every place the word is, in any case**, by `git grep -il`, sorted into: changes
   (call 246 point 1); the file format's words, never changed (point 2), each with
   file and line; records left as written (point 3). A place that fits none of the
   three is listed for the lead and not touched.
2. **The "before" set**, taken on the head and kept in `SCRATCH/review/246N/before/`:
   the names of every test (`cargo test --workspace -- --list`); `tools/check.sh`'s
   whole output; what each command-line demonstration prints; what the command with
   no verb prints; the sample whole house saved, and its checksum; one job saved by
   the window's `house` demonstration.
3. Every sentence the program says that names a command, by file and line: the list
   whose expectations will move by the substitution.

## What is done

1. **One script, `SCRATCH/review/246N/rename.sh`, makes the whole change from the
   head**: `git mv plumbline buildos`, then the substitutions in order (crate names
   with a hyphen and with an underscore, the two commands, the environment names,
   the paths in scripts, hook, gates, `.gitignore` and documents), then
   `Cargo.lock` brought to the new names with no version moved, then `LICENSES.md`
   and `DECISIONS-index.md` and the manual's page by their tools. The substitution
   never touches a place of point 2 or point 3: the script names them and skips
   them. Run twice from the head it gives the same tree.
2. **`tools/gate-name.sh`** and its line in `.githooks/pre-commit`: the word, in any
   case, is nowhere in the tree but in the listed places, each with its reason in
   the gate. Prove it red (the word in a source file; the word in a new document)
   and green, and keep both outputs.
3. The one line each at the head of `DECISIONS.md`, in `DECISIONS-index.md` (by
   `tools/index.sh`) and in `handoff/README.md` that says what `plumbline/` means
   in a record written before this change.

## What must not change

- **No byte of anything the program writes**: the sample whole house is
  `880d9fd3c364b5f7`; a job saved before opens after and saves byte for byte; a BIM
  file, a workbook, a CSV, a PDF, a field sheet written after are byte for byte
  what they were before.
- No number, no meaning, no test's name but for its crate's name, no count, no
  expectation but by the one substitution in a sentence that names a command (your
  list 3, line for line). No test or demonstration is skipped, deleted or weakened.
- Every control's name on the map. A demonstration's file name.
- The records of point 3 are not edited. `CLAUDE.md` and the two READMEs get the
  mechanical substitution only: the sentences that explain the working name are
  the lead's to rewrite, and you list them.
- Files you may touch: every file the script touches under the rule above, and
  nothing by hand but what your report lists with its reason.

## How it is run

Every cargo command through `tools/quiet.sh` with `CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 -j2`; the window demonstrations through
`tools/window-run.sh all` on one frozen build, nothing beside them; long commands
in the background, nothing left running. If a tool asks a permission you do not
have, stop and say so in the report; do not ask the lead to do it.

## Done when

`tools/check.sh` with the port map prints `check: PASS` with every count as in the
"before" set; the "after" set is the "before" set with the substitution applied,
line for line (the comparison's own output in the report, and proved red on one
changed line); all window demonstrations pass on the renamed build; the three
gates, `tools/gate-name.sh`, the licences and the port map pass; the report has
the three "before any change" answers, the script, the hand edits with reasons,
and what is not right.
