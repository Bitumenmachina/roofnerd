# Handoff: fittings, connections and clash detection (the owner's loop, stretch c)

Written 2026-11-11, as the drawing stretch (`HANDOFF_DRAW.md`) went to its judge. Read
`CLAUDE.md` at the repository root first, then this.

## 1. The task

The owner's order of stretches puts this third: "fittings, connections and clash
detection". After it, from an empty job and without editing the file, a person can:

1. Put a fitting where runs meet, and have the takeoff count it by kind.
2. Join a run to a fixture or a piece of equipment, so the network says the system is
   joined and what is still open.
3. Ask what clashes: which solid things in the job occupy the same space, where, by how
   much, and how sure that is.

## 2. The fence

**In:** a verb, a command and a window tool for a fitting at a joint; the same for a
connector that joins a run's end to a placed thing; a split of a run where a branch
meets it along its length (a tee needs three ends); clash detection between everything
`building::placements` gives a shape, at the command line and shown in the window.

**Out, and to PROPOSALS.md if wanted:** choosing which elbow is bought (long radius,
gauge, a manufacturer's part); fitting losses and sizing; routing a run round a clash;
clash rules by discipline pair, clash groups, issue tracking and BCF export; clashes
against the roof's planes (they are not in `placements`; the report says they are left
out); clearance zones round equipment for access (a clearance asked for at the command
line is in; a per-type service zone is not).

## 3. What was checked before this was written

On the machine:

* The model already has `Fitting {kind}` with `FittingKind` Bend, Tee, Cross,
  Transition, Coupling (`crates/world/src/building/model.rs:431`), `Connector` and
  `Connectors` (`model.rs:518`, `:543`), and the network ported from BIMCraft
  (`building/network.rs`): points within the fit tolerance are one node, a fitting's
  connectors are joined through it, a fixture's are not, and a run met along its length
  is the warning `joined_mid_run` whether or not a fitting stands there
  (`network.rs:374`). So a tee is three run ends: the through run has to be two runs.
* Nothing places a fitting or a connector today except `sample.rs` (the house's elbows,
  `sample.rs:402`). `editing::place` refuses a Fitting (`editing.rs:274`).
* BIMCraft places a fitting as a component of a family, by hand, like a fixture
  (`fixtures.rs:203`, `family.rs:132`); it has no automatic fitting and no clash
  detection (searched: `clash`, `interfer` in `crates/core/src`; the only hits are a
  comment in `inquiry.rs` and one in `link.rs`). Its `split` (`commands.rs:12534`) is of
  a wall or a detail line, not a run: the first piece keeps the id, the second is new,
  and what hangs on the wall goes with the piece it is in.
* A pipe connector's domain and a system's classification are already matched in one
  place (`building.rs:1484`).
* `building::placements` (`building.rs:1151`) gives every wall (with its openings cut),
  floor, run and placed thing as a shape in feet with the worst status of what it is
  drawn from, and lists what has no shape and why. The window draws from it. Clash
  detection is built on it and on nothing else: one solve.

On the web (2026-11-11):

* The trade's words: a **hard clash** is two things occupying the same space; a **soft**
  or **clearance clash** is two things closer than a distance asked for. (A third,
  the workflow or 4D clash, is about time and is not this.)
* `parry3d-f64` 0.31.1 is on crates.io, licensed Apache-2.0: shapes (cuboid, cylinder,
  convex hull), `query::intersection_test`, `query::contact` (a signed distance: how far
  apart, or how deep), `query::distance`. Its transitive licences have to pass
  `tools/licenses.sh` before a line of it is called. Its API is read from the crate's
  own source once it is in the registry, not from memory.

## 4. Calls already made (each goes in DECISIONS.md when its part is built)

* **A fitting's kind is worked out from what meets, not typed.** Two ends at an angle
  are a bend; two in line of one size a coupling; two in line of two sizes a transition;
  three a tee; four a cross. A kind given is checked against that and refused if it
  disagrees. Anything else meeting (one end, five, two kinds of run) is refused.
* **A fitting is as sure as the runs it joins**, never surer, and says that which part
  is bought is not selected.
* **A branch on a run's length is made a tee by splitting the run there**: the first
  piece keeps its path, the second is new with the same type, system, heights, status
  and source (the rule of BIMCraft's wall split).
* **Fittings are not placed by themselves as runs are drawn.** A verb places one where
  it is asked, or at every joint of a system that has none (`--all`), and says how many.
* **Every intersection is reported.** A run through a wall or a floor is listed apart,
  as passing through it (it is a hole to cut and sleeve, which is a count an estimator
  wants), not mixed with a duct through a pipe.
* **What is joined is not a clash**: a run and the fitting or the thing it is joined to
  at that end; two runs joined at a node; a door or a window and its wall; walls that
  meet. Two runs of one system that cross without being joined are.
* **A clash is as sure as the two shapes**: Assumed if either is. A clearance is only
  tested when one is asked for; there is no default distance.

## 5. Gates

`tools/portmap.sh` covers `crates/world/src/building*.rs` and `editing*.rs`. The clash
file goes under `building/`, so every public item in it is `ours` with the search that
shows BIMCraft has none. Run the gate red on the new file before giving it rows.
`tools/licenses.sh` must pass with the new crate. The repository's three gates as ever.

## 6. Order of work

1. `editing::split` (a run), `editing::fitting` (one joint, and all of a system's),
   `editing::connect` (a run's end to a placed thing). Tests with hand figures: a tee's
   three connectors, the network going from two groups to one, open ends closing.
2. The same at the command line; `demos/fit.sh` on the drawn house of `demos/draw.sh`:
   elbow at the duct's corner, the pipe joined to the lavatory, a branch and its tee;
   the takeoff's fittings by kind and its "joined" line before and after.
3. `building/clash.rs` on `parry3d-f64`: solids from `placements`, pairs, what is
   excluded, hard and clearance. Tests: boxes and cylinders placed by hand with the
   overlap worked by hand; a pipe through a wall; a pipe joined to its fitting (not a
   clash); an Assumed shape flags its clash.
4. `plumbline clashes <file> [--clearance "2 in"]`; in the window, the clashes listed on
   the sheet and marked on the model.
5. Demonstrations at the command line and in the window, pictures read.
6. The judge.

## 7. Done when

A person can do section 1's three things from an empty job; every figure in the
demonstrations is worked by hand; the gates pass; a judge has tried to break it and each
finding is fixed with a test or left and said; the documents are true.
