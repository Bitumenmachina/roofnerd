# Contract 190P: the two worked-out parts of call 190 (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd (the window builder has it and is running timed window runs).
Read CLAUDE.md, then plumbline/DECISIONS.md "Call 190" and plumbline/STATUS.md "A wall
traced over a scan, set by its face". The window half is another builder's, after
you: you build the two parts the window will ask, each tested at its own layer, so
that the window only asks and works nothing out itself. Notes as you go to
SCRATCH/review/W-190P-report.md (SCRATCH is
SCRATCH).
Commit nothing, push nothing. When done, from the working copy's root:
`git add -N . && git diff --binary -- plumbline > SCRATCH/review/call190P.patch`.

## Part 1, `crates/plans`: the faces a scan's sheet shows
The sheet of a scan is the picture `Scan::cut` writes (`crates/plans/src/scan.rs`:
gray dots four across at 24 to the foot, white paper). Read how the window gets that
picture back when a job is opened (`crates/plans/src/lib.rs`, the page reader for a
PNG, and `crates/app/src/plans.rs` `read`: READ ONLY, do not edit the app) so that
what you return is in the same units and corner as `Page.lines`.

    pub struct Face {
        pub line: [[f64; 2]; 2],        // page units, from the lower-left corner
        pub room: Option<bool>,         // Some(true): the room is to the left of line[0] -> line[1]; None: cannot tell
        pub pair: Option<(usize, f64)>, // the other face of the same wall (its index) and how far apart, page units
    }
    pub fn faces(gray: &[u8], width: u32, height: u32, dots_per_foot: f64) -> Vec<Face>

(Put it in a new file beside scan.rs; take RGBA too if that is what the page reader
hands over; the names above are the contract, the argument types are yours to fit.)
- A face is a straight line of the sheet's points at least 2 ft long (a named
  constant `FACE_LEAST_FT`, the lead's, Assumed). Fitted through the points' centres:
  a face the made-up scan puts at a whole foot is read within 1/4 in of it (mind
  that `cut` draws each dot from -1 to +2 about its point: say in a comment how the
  half-dot is accounted for, and test it).
- The room's side: the side with more of the sheet's points within 8 ft of the line.
  Where the two sides are within a tenth of one another, None.
- A pair: two faces parallel within 2 degrees, 2 in to 14 in apart, overlapping over
  at least 2 ft, with (nearly) no points between them.
- Tests FIRST, run red, the red run quoted in your report. Scans made by
  `tools/made-up-scan.py --scene house` (30 by 16 ft, a 4 1/2 in partition with both
  faces at 18 ft) or built in the test as points and cut with `Scan::cut`: 6 faces,
  each within 1/4 in; the two partition faces a pair 4 1/2 in apart; each outside
  face's room on the inside; the same faces with the house turned 17, 33 and 45
  degrees; a 1 ft 6 in run of points (furniture) is not a face; the phone scene
  (`--scene phone`: rough, bowed, in passes) gives one face a wall, not three.
- The Wall tool will snap to these as it does to a drawn plan's lines. Nothing here
  lays a wall or marks anything Known.

## Part 2, `crates/world`: a line of points carried out by half a thickness
    pub fn carried_out(points: &[[f64; 2]], by: f64, left: bool, closed: bool) -> Vec<[f64; 2]>
in `crates/world/src/editing` (a new file or an existing one, yours to choose): each
stretch moved `by` to its left (or right) and the corners carried both ways, so a
closed line of the inside corners (0,0) (30,0) (30,16) (0,16) carried out 0.25 to
the outside gives (-0.25,-0.25) (30.25,-0.25) (30.25,16.25) (-0.25,16.25). An open
line's ends move square to their one stretch. Two stretches in one line stay in
line. A corner sharper than 15 degrees is not carried to a point miles away: it is
cut square (say how in a comment and test it).
BEFORE writing it: search ALL of BIMCraft at /home/user/ipedrax/bimcraft
(`grep -rln -i "offset\|location.line\|justif" crates apps` and read the whole list)
for an offset of a line of points or a wall's location line. Port it if it is there;
record it in `plumbline/PORTMAP.md` either way with where you looked;
`BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
Tests first, run red, quoted.

## The real scans (scratch only; they never enter the repository)
SCRATCH/real/ply/ark/ holds three real phone scans; SCRATCH/w189/truth_*.txt says
what a person reads as walls in each. For each: cut it as the program does (the
`scan` verb or `Scan::cut` in a scratch example that you delete after), run `faces`
on the cut, and draw the faces over the cut in a picture in SCRATCH/w190P/ (python
and PIL). Report for each scan: faces found, how many lie on a wall of the truth
file (within 3 in across), how many are furniture or ghosts, which walls of the
truth have no face. Look at each picture yourself. Say plainly where it is wrong.
No test or demonstration reads a real scan.

## Rules
- Files you may touch: a new file in `crates/plans/src/` and two lines of its
  `lib.rs`; a new or existing file under `crates/world/src/editing*` and its tests;
  `plumbline/PORTMAP.md`. Nothing in `crates/app`. No existing test, demonstration,
  number, meaning or file format changes. If an existing test fails, stop and report.
- Build and test with `CARGO_INCREMENTAL=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only `-p plumbline-plans -p plumbline-world`. Watch `df -h /`: under
  4 GB free, stop and delete your own old test programs (literal absolute paths).
- The three root gates and `plumbline/tools/licenses.sh` pass (no new crate unless
  its licence is permissive and you say why it is needed; prefer none).
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
Both crates' tests pass (counts, and the red runs quoted); portmap, licences and the
gates pass; the report has the real scans' figures and pictures and says what is
still wrong.
