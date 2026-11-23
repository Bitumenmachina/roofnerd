# Worker report: call 176, what a wall carries and what it stands on

Working copy: `/tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/wt176` (nothing committed or pushed).
Patch: `/tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/review/call176.patch`.

## What was built

- `crates/world/src/carries.rs` (new, ours): `all`, `of` (the two figures of every wall / one wall),
  `cautions` (for `check`), `asked` (for the field list), `headers_asked` (call 175 item 6),
  `unsaid` (count of walls whose bearing is not said with something standing on them, by level and for the job).
  Nothing is saved; the figures are worked each time.
- `building.rs`: `plan_wall` made `pub(crate)`; the joists' direction rule of `joist_figures` factored into
  `joists_run` (same arithmetic, one place; the joist tests and `joists.sh` pass untouched).
- `building/framing.rs`: `HEADER_DEPTH_IS`, the sentence every place a header's depth is shown carries. No lumber count changed.
- `vif.rs`: `Question` gains `also: Vec<String>` (the program's own sentences on a wall the work opens).
  A wall the work opens with something standing on it, or a cut opening whose header is to be sized, is a question
  even if its bearing and its contents are said, with everything priced on the wall riding.
- `crates/cli/src/main.rs`: `check` prints the cautions; `vif` prints the sentences; new verb `carries <file> [<level or wall>]`
  (every wall by level, or a level's, or one wall; ends with the count of walls whose bearing is not said with something on them);
  `show <file> <thing>` prints that thing's block and, for a wall, its two figures (plain `show <file>` is byte for byte what it was).
  "Carries" is the builder's word chosen for the verb.
- `demos/carries.sh` (new), `crates/world/tests/call176.rs` (18 tests), README list, SPEC row, PORTMAP row (`HEADER_DEPTH_IS`, ours, search done in BIMCraft `crates/`).

## How the two figures are worked, in plain words

- The level above is the storey with the next higher elevation; the level below the next lower.
- **A wall (or footing, beam, eave) lies over/under another** when it runs within 1 degree of the same direction and
  both its ends are within this wall's thickness of this wall's centre line (centre to centre, so a wall 6 in off a 6 in wall counts, 7.2 in does not).
  The shared length is how far along this wall that stretch reaches.
- **Carries**: walls of the level above so placed; eaves (not rakes) so placed whose plane's eave height is within 2 ft of this wall's top;
  the joists of the floor above where this wall lies across the inside of that floor's outline (stretches along an edge of the floor,
  within the thickness, are left out): joists said and across it (more than 1 degree from its direction) -> "cross it";
  alongside -> nothing; not said -> "which way ... is not said".
  Walls above marked to demolish are listed and not counted.
- **Stands on**: walls of the level below; footings drawn on the wall's own level; beams (any level) whose top is within 1 ft of the wall's base.
  Walls below marked to demolish are listed and not counted. The rest of the length: "nothing found under it for N LF"
  ("(no footing)" on the lowest level; on an upper level "it stands on the floor alone there, which is ordinary for a partition").
- **Status**: the worst status of the positions of everything used; those not Known are named. Every place shown says the call's sentence.
- **Where it speaks up**: a wall the work opens (demolish, or an old wall cut for a new opening) with a wall/eave/crossing joists on it
  -> caution in `check` and a sentence on its field question. For a wall that comes down, only what would be left with nothing under it
  counts (a beam in the line, or anything else drawn under the wall above, answers it; a half-length beam leaves "for 4.00 LF").
  For a wall that is cut the sentence says the header carries it; a beam does not answer it. A wall said not bearing with a wall or eave on it -> caution.
- **Header**: for each new opening in an old wall said bearing or not said: "What size is the header over window 1? It is drawn 5 1/2 in deep (Assumed: a rule of thumb used to lay out the cripples, not a size: an engineer or a span table sizes a header)." Not asked where the wall is said not bearing.

## Hand figures against what the program printed (demos/carries.sh passes)

| Figure (STATUS.md) | By hand | Printed |
|---|---|---|
| P1 carries P2 | 12.00 LF | wall 5 on second [wall-0016] for 12.00 LF |
| P1 joists, direction not said | said | which way the joists of the floor above run is not said |
| P1 stands on | nothing, 16.00 LF (no footing) | nothing found under it for 16.00 LF (no footing) |
| first-floor south carries / stands on | 24.00 LF / footing 24.00 LF | wall 1 on second ... for 24.00 LF / footing 1 (footing) for 24.00 LF |
| second-floor south carries / stands on | eave 24.00 LF / first floor 24.00 LF | the roof's eave for 24.00 LF / wall 1 on ground for 24.00 LF |
| second-floor east carries / stands on | nothing (rake) / 16.00 LF | nothing found standing on it / wall 2 on ground for 16.00 LF |
| P2 carries / stands on | nothing / P1 12.00 LF | nothing found standing on it / wall 5 on ground for 12.00 LF |
| joists north-south: P1 | P2 12.00 LF and no joists | no joist line |
| joists east-west: P1 | they cross it | the joists of the floor above cross it |
| P1 demolished: caution | "P2 stands on it for 12 LF: once it is down, nothing is found under that wall" (+ joists cross) | both, with 12.00 LF and 16.00 LF and the call's sentence |
| P1's dollars riding | $1,440.00 (16 x 9 x $10) | wall 5 [wall-0015] (wall), on ground: $1,440.00, 1 line |
| P2 then | nothing found for 12.00 LF | nothing found under it for 12.00 LF (P1 "which comes down: not counted") |
| beam under P2 | P2 on beam 12.00 LF; caution gone; question gone | beam 1 (beam) for 12.00 LF; no caution; no "stands on it" on the list |
| said not bearing | caution "said not bearing, and P2 stands on it for 12.00 LF" | "is said not bearing, and wall 1 on second stands on it for 12.00 LF" |
| nothing else moves | byte for byte | estimate, clashes, takeoff, job file: cmp equal |

Every hand figure came out. I believe none of them is wrong. Two places where the demonstration's wording had to be chosen, not a figure:
the field list's other questions on P1 (bearing, in the wall) stay after the beam (they are call 160's and the beam does not answer them);
the demonstration checks the new sentence is gone, not that the wall leaves the list. The partition is priced by a direct line at $10.00/SF of face, not by the starter assembly.
The beam is drawn on the second level, top 0 ft (it also works on the ground level at top 9 ft).

## Expectations moved

None. `cargo test` (the six packages the contract names): 580 passed, 0 failed (18 of them new, `crates/world/tests/call176.rs`).
Every command-line demonstration passes with `PLUMBLINE_BIN` set (`demos/field-job.sh` is a helper that needs an argument; it is not a demonstration);
`demos/hidden.sh`, `structure.sh`, `framing.sh`, `joists.sh`, `vif.sh` and `sweep-money.sh` among them untouched.
No existing test or demonstration counts a stacked wall the work opens or a cut opening, so none of the new questions or cautions reached one.

## Choices the decision did not make

- Verb names: `carries <file> [<level or wall>]`; `show <file> <thing>`. Sentences as printed above.
- The level above/below is by elevation, adjacent only.
- "Within the thickness" is centre to centre, up to and including the full thickness.
- An eave is over a wall only within 2 ft of the wall's top (otherwise a second-floor roof's eave would be on the first floor's wall). An eave whose height is not known is taken as over, and its status Unknown.
- Beams: any level, top within 1 ft of the wall's base (mean of the two ends if sloped). Footings: the wall's own level only (see below).
- Joists: only where the wall lies across the inside of the floor's outline; a wall along an edge of the floor is said nothing of joists (the hand table says nothing of joists for the outside walls; this is why). Crossing is "more than 1 degree from parallel".
- The header question is asked only of walls the work opens by cutting a new opening (call 160's "cut" walls), not of every wall with an opening in the job ("walks every opened wall"). If it was meant for every wall with an opening, that is one filter to change and many existing counts would move.
- The caution for a wall that comes down says "once it is down ... for N LF" with N the length left bare, so it differs from the call's sentence by the added length.
- The count of unsaid-bearing walls is printed by `carries` and available as `carries::unsaid`; I did not add it to `structure`.
- Another level's wall is named "wall 1 on second" in these sentences (every level has a "wall 1").

## Where it would mislead on a real house (blunt)

- A wall above offset 3 in, or up to a full thickness, from the one below counts as standing on it though part of it hangs over; one offset 7 in, which still bears on half the wall below, is "nothing found under it".
- A wall above that lands on a floor joist and not a wall says "it stands on the floor alone there, which is ordinary for a partition": true of a partition, but the program does not know a bearing wall over unsupported joists, parallel to them between two, needs a doubled joist or a beam. It says nothing of joists under walls.
- A post is not a support here: a wall (or beam) standing on a post reads "nothing found under it". Beams' ends are not looked at.
- A stair opening or any hole in a floor is ignored: the floor's outline is taken whole, so joists "cross" a wall across the stair well. The floor is not checked to be over the wall at its own height.
- A cantilever (a wall above past the wall below at the floor's edge) reads as the floor alone, "ordinary for a partition", which is wrong in kind for an outside wall.
- A split level or half levels: the level above is simply the next elevation up; walls are compared whatever heights they stand at, so a half-flight up reads as the level above for every wall.
- Footings drawn on a level of their own below (a "footings" level with nothing else, or a basement's) are not found under the ground floor's walls; only footings on the wall's own level are (the hand table needs it that way: a second-floor wall must not stand on the ground floor's footing). A wall on the lowest level with footings drawn elsewhere says "(no footing)".
- The roof: only eaves are looked at. A rake, a gable end's framing, a hip, a ridge beam's wall or struts bearing on an interior wall are not; an interior wall under the ridge says nothing found. Eaves at a different height from the wall's top (more than 2 ft) are missed.
- A beam "answers" a wall that comes down whatever its size, even Unknown or Assumed (its own questions stand); its ends need not bear on anything the program checks.
- A wall marked to demolish above is not counted as standing on anything (right), and a wall below marked to demolish is not counted as a support, even if a new wall is drawn in its place at another height or not in line.
- It is positions only: no load, no span, no species, and never "safe".

## What is not right / not done

- Nothing in the window (told not to touch it). `Question` gained a field (`also`); the window builds against `Question` by field name, not by construction, but I did not build `plumbline-app`.
- `show <file> <thing>` shows the one thing, not its children (openings under a wall).
- The header's depth is not shown anywhere else on the command line (nothing there prints it); the window's drawing of the frame will need `framing::HEADER_DEPTH_IS` where it labels a header.
- The count of unsaid walls is not on `structure` and not shown on the field list.
- Not judged by anyone.

## Gates

`tools/gate-language.sh`, `tools/gate-egress.sh`, `tools/gate-clientdata.sh --all`: PASS. `tools/licenses.sh`: exit 0. `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh`: PASS (304 public items).
