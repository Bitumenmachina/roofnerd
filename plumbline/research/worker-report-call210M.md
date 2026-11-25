# W-210M report (worker, contract 210M). Written as I go. Second working copy: .../scratchpad/wtB. Nothing committed.

## Before any code

1. BIMCraft (`/home/user/ipedrax/bimcraft`): `grep -rn "filter|only|categories|disciplines" crates/core/src/import.rs crates/core/src/ifc.rs`
   finds nothing that chooses what an import brings in. Its `import::ifc(bytes, name)` (import.rs:1664) reads the whole file
   into a model; the "categories" hits (import.rs:165, 3271-3303) are project-parameter categories, not an import filter.
   `grep -rln "IfcOptions\|ImportOptions\|ImportFilter" crates` finds nothing. Nothing to port. PORTMAP gets one line (ours).
2. What the reader does today with the demonstration's model brought into the sample house (run, not read): the lead's step 1
   ("refused as today") is NOT what happens. The job is not refused whole: `import_whole` brings in what it can and
   refuses per thing. 1 duct and 1 piece of equipment come in; 1 floor is left out ("a floor with this very outline is already
   on this level: a second would be counted twice", the check is `there` in the floor block, lib.rs); 4 walls left out ("the new
   wall: lies along wall-0010 for 40.00 ft: one wall drawn over another would be counted twice", the check is inside `ed::walls`).
   The model's level "Level 1" at 0 ft is taken as the job's "first floor" (call 206, by height). The whole is refused only when
   nothing but levels came in (`out.made` all levels: `already_in_the_job` / "nothing in the model but its levels"), which is not
   the case when the file has a duct. If the lies-over checks were passed over, 4 walls (136 LF) and 1 floor (1,120 SF) would be
   made on top of the sample's own: counted twice.
   So, smallest true thing: step 1 is "brought in whole: the duct and the equipment come in, the 4 walls and the floor are left
   out as lying over the job's, and the reply says the runs and equipment can be brought in alone". I build that (a hint in the
   reply when walls or floors were left out for lying over what the job has) and demonstrate it as it is.
   The refusal that does exist (nothing but levels made) gains the words on the same point.

## Red runs (tests first; the API was stubbed so they fail on what they assert, not on compiling)
`cargo test -p plumbline-ifc --test call210`: 10 tests, 2 pass (the sample's figures and price are unchanged by a whole bringing-in;
two systems, which the whole reading already does), 8 FAILED on their assertions, e.g.
- step_2: `assertion left == right failed: []  right: [("floors", 1), ("walls", 4)]` (nothing is counted as not asked)
- step_4: `the reply says what it was done for` (no clash count)
- walls alone: `called Result::unwrap_err() on an Ok value: Brought { made: {"walls": 2} ...` (as runs only the walls still came in)
- mezzanine: `no level is added: left ["Loft","Mezzanine","first floor","second floor"] right ["first floor","second floor"]`
- step_5: the refusal says `7 of its 7 things ... (4 wall, 1 duct, 1 floor slab, 1 other things)`, not 2 of 2
- fixture connected: `{"cables": 16, ... "walls": ...}` (the whole house's walls came in)
(Test 6 first failed on a wording of mine in the test: the note for a model this program wrote reads "so everything from it was
brought in Assumed, not Known as was said of it"; the test now asserts that, which is today's text.)

3. The word and the function. Command line: `plumbline import ifc <job> <model.ifc> --status ... --source ... --only runs` (no `--only`
   is the whole, as before; `--only walls` etc. is refused: "the choice is runs"). Window (call 210.6 will call it):
   `plumbline_ifc::import_choosing(job: &mut Job, bytes: &[u8], name: &str, how: Entered, bring: Bring) -> Result<Brought, String>`,
   `Bring::{Whole, RunsOnly}`. `import()` is untouched (what the window calls today). `Brought` gains `not_asked: Vec<(String, usize)>`
   (+ `not_asked_said() -> Option<String>`, "not asked for: 1 floor and 4 walls") and `clashes: Option<ClashChange>` {before, after, made:
   `ClashCounts{same_space,set_against,holes,laid_in}`, listed: Vec<String>} (+ `clashes_said() -> Vec<String>`; its last line is
   "they are in the clash list, with the job's own": the CLI turns it into `plumbline clashes <file>`, the window words it itself).
   `plumbline_ifc::one_of("pieces of equipment") -> "piece of equipment"` is new too. The "flag words" `--only runs` appear only in the CLI.
4. The rule on one case by hand. Job levels: first floor at 0 ft, second floor at 10 ft. Model level "Mezzanine", floor at 4 ft (no
   job level at 4 ft), a duct 3 ft above it (7 ft above the datum). The job's highest level whose floor is not above 7 ft is the
   first floor (0 ft): the duct goes on the first floor, 7 - 0 = 7 ft above its floor, and a note says so; no level is added. A
   thing at "Loft" 13 ft, 2 ft up (15 ft) goes on the second floor, 15 - 10 = 5 ft above it. A thing below the job's lowest level
   goes on the lowest, with a negative offset, and the note says how many. The code: `storey_of` in `bring` (lib.rs):
   height = stated + z - placed; job level = last of the job's levels with elevation <= height + 1/100 ft, else the first; the thing's
   floor = that level's elevation + (placed - stated). Tests: `a_run_on_a_level_at_a_height_the_job_lacks...` (7 ft and 5 ft),
   `no_level_to_stand_on...` (-7 ft); demo step 7.
5. Clash kinds and the count. Kinds are the model's: in the same space (Hard, not through, not set against), set against (a thing
   against its wall, within 2 in), a hole to cut (through a wall or floor, once per hole), laid in a floor or wall, taken from
   `Clashes::in_the_same_space / set_against / hole_rows / laid_in`, the same accessors `plumbline clashes`' counts equal (checked: the test
   compares the reply's "after" with a second call of the same function, and the demo compares the reply with `plumbline clashes` before and
   after, and the diff of the two lists is exactly the one new line). `clash::clashes(job, None)` is called once before and once after
   inside the bringing-in (inside `all_or_nothing`). "Made by what was brought in" = a clash with a path that was not in the job before.
   If the after-minus-before does not equal the made, the reply says the job's own count moved by that much and the things brought in
   account for the rest.

## What is built
- `crates/ifc/src/lib.rs`: `Bring`, `ClashCounts`, `ClashChange`, `import_choosing`, `Brought::{not_asked, clashes, not_asked_said,
  clashes_said}`, `one_of`; `bring` takes `only_runs` and `extras`. Only its runs: walls, doors, windows, openings, roofs (and roof
  slabs), floors, rooms, footings, beams, posts, furnishings and other things are counted in `not_asked` and not read; ducts, pipes,
  conduit, cable, their fittings, fixtures, equipment, systems and ports are read as before (so "they come in connected"). No level is
  added; a model level at a job level's height is that level (call 206, as before); at another height its things go on the level they
  stand within. A model with no run and no equipment brought in as runs only is refused, saying what it holds. A job with no level is refused.
  Where the whole was refused, a trial of runs-only is worked and taken back (nested `all_or_nothing`), and the refusal says what
  could be brought in alone only if the trial succeeded. Where walls/floors lie over the job's and the whole came in, a note says so and
  says to choose only its runs. All of it only on `import_choosing`; `import()` (the window today) says nothing new.
- `crates/cli/src/main.rs`: `--only runs`; the reply prints not-asked, the notes, and "clashes:" (for both choices); the usage line;
  "1 pieces of equipment" is now "1 piece of equipment" (`one_of`).
- `demos/ifc-runs.sh`, its row in README.md (and the command in the README's list), PORTMAP.md paragraph.
- `crates/world`: NOTHING touched. `crates/app`: nothing.

## The model the demonstration makes, and where it differs from the lead's text (all run, not read)
Made by the script with the program itself: level "Level 1" 0 ft / 10 ft; 4 walls on (0,0)-(40,28) 10 ft high; a 4 in floor of the
same outline; ONE DUCT 10 ft LONG (not 12), 14 in by 8 in, from (20,20) to (30,20), 8 ft up, on a supply-air system; one air handler
2 x 2.5 x 5 ft with its middle at (30,8). Why: (a) a 12 ft duct from (20,20) reaches x=32 and meets the sample's own "branch to
office" (a second clash, 2.20 in, in the same space), and from x=18 the sample's west trunk is hit; 20 to 30 at y=20 meets nothing but
the partition at 24 (which is (24,16)-(24,28), 9 ft high, no opening); (b) the air handler at (30,8) is clear of every wall and duct
(the lead's "will not stand clear" worry did not arise there: 0 clashes from it).
By hand: duct 14 in x 8 in centred 8 ft up spans 7 ft 8 in to 8 ft 4 in, below the partition's 9 ft top, so it passes through the
partition at x = 24: one hole, 14.00 in by 8.00 in, at (24, 20, 8). Before: 11 in the same space, 10 set against, 47 holes to cut,
193 tested. After: 11, 10, 48, 195 tested. (Run: exactly that.) Price $468,050.22 before and after, in both choices.

## Where the code differs from the design as written (smallest true things)
1. Design step 1 ("brought in whole: refused as today ... and that the runs and equipment can be brought in alone") is not what
   today's code does. Whole is refused only when nothing but levels could come in; this model brings the duct and the air handler
   in and leaves out the 4 walls and the floor (each saying it would be counted twice). Built and shown as it is: the whole path
   gains a note ("4 walls and 1 floor of the model lie over what the job has and were left out ... choose only its runs and the
   equipment they serve (--only runs)"), and the refusals gain "Its runs and the equipment they serve (1 duct) can be brought in
   alone, without its walls and floors" where a worked trial shows that is true (e.g. a level of the job's own name at another
   height refuses the whole and not the runs: test `a_whole_refusal_says_what_could_be_brought_in_alone_and_it_can`).
   HELD FOR A RULING: whether the whole bringing-in of a model that lies over the job should not simply refuse (it would then bring
   in nothing, not even the runs). I did not change that.
2. Furnishings (IFCFURNITURE etc.) are not "fixtures and equipment they reach": counted as not asked for. A lead who wants them
   with the runs says so.
3. Design point 2 says "the job's level it stands within". Implemented as the highest job level whose floor is not above the
   thing's height (no use of the level's height as a ceiling: a thing above the top of the top level goes on the top level).

## The reply when only runs are brought in (the demonstration's own run, quoted)
```
d.ifc (IFC4): brought in, only its runs and what they serve
    1 duct
    1 piece of equipment
  not brought in: 0 things
  not asked for: 1 floor and 4 walls
  note: the model's level "Level 1" has its floor at 0 ft, the height of the job's "first floor": it is that level, and what the model has on it was put on the job's own
  note: the model is in feet: ...
  clashes:
    the things brought in make 1 hole to cut, 0 in the same space and 0 set against a wall with what the job has
    the job had 47 holes to cut and now has 48
      a hole to cut: duct 1 and partition at 24, 14.00 in by 8.00 in, at (24, 20, 8)
    they are listed with the job's own by `plumbline clashes <file>`
```
Second time: `this model is already in the job: nothing was brought in. 2 of its 2 things are in the job already, in the same place, so a second
would be counted twice (1 duct, 1 other thing). It holds 1 floor and 4 walls, which were not asked for. To bring them in again, take what
they brought in out of this job first. Bringing in only what the job lacks is not there yet` (job byte for byte as it was).
Walls alone, as runs only: `this model has no runs and no equipment: nothing was brought in. It holds 2 walls, which were not asked for`.

## What was run
- Tests, one crate at a time, all pass: world 632, plans 172, ifc 69 (12 new in tests/call210.rs), cli 28, sheets 6, roofing 122 =
  1,029 passed, 0 failed. No existing expectation moved (call206.rs:98's exact refusal and call209m.rs's are on `import()`, which says
  what it said). Red run of the new tests quoted above.
- 55 command-line demonstrations (every demos/*.sh without plumbline-app, open_window, xvfb or xdotool, less window.sh and field-job.sh;
  54 old + the new demos/ifc-runs.sh), all with PLUMBLINE_BIN set: 55 ended rc=0; `grep -c "FAIL"` (which also counts FAILED) was 0 in each
  output; each ended on its own PASS/holds line (d1.sh ends on a list of flags, as it always does). Outputs in
  scratchpad/w210/demo-out/. Windows demos not run (not mine).
- Gates: tools/gate-language.sh PASS, gate-egress.sh PASS, gate-clientdata.sh --all PASS; plumbline/tools/licenses.sh exit 0;
  BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh: PASS (383 public items, 142 ported, 241 ours).
- ifc-runs.sh went red twice on the way (a wrong expected shape of one diff line; a quoting error) and was fixed in the script, not
  loosened; each check that failed was a check of what the program printed.

## Not right / not done
- Whole mode of the CLI now ends every reply with a "clashes:" block (two clash passes per import). Cost ~0.2 s on the sample. No demo
  or test moved by it. `import()` (window) is unchanged and says no clashes: the window's half (call 210.6) will call `import_choosing`.
- "1 other thing" in the already-in-the-job refusal is the file's class "(as the model calls it)" (an air handler is written as a
  distribution flow element): the existing wording, only pluralised right. Not fixed further.
- The new duct's type is "14 in by 8 in" beside the sample's own "Duct 14 in × 8 in, assumed": two types of one size (existing way a
  type is looked up by name). Not touched.
- STATUS.md and DECISIONS.md not written (the lead's). The call to write: runs-only as `--only runs`, levels rule, furnishings not asked.
- Nothing committed or pushed. Patch: scratchpad/review/call210M.patch (git add -N of the two new files only, so node_modules at the
  root is not in the index).
