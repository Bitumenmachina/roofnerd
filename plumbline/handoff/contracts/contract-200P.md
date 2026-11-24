# Contract 200P: the file says what is connected (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 200"
(points 1 to 3 are yours) and "### Call 197" before it, plumbline/STATUS.md "The file
says what is connected: a demonstration", plumbline/research/worker-report-call197I.md
(what the last worker found, and where: answers 2 and 4, "Held for a ruling"),
`crates/ifc/src/write.rs` and `lib.rs`, `crates/world/src/sample.rs` (lines near
512 to 565: the connections the sample puts on its fixtures), `crates/world/src/editing.rs`
(`connect`, `fitting`), and `crates/ifc/tests/whole.rs` (the test marked as
waiting: `p2_each_system_reads_as_closed_as_it_was`).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-200P-report.md. Commit nothing, push nothing.
When done: `git add -N . && git diff --binary -- plumbline > SCRATCH/review/call200P.patch`.

## Before any code, in your report
1. Search ALL of BIMCraft at /home/user/ipedrax/bimcraft for ports and connectors
   written to or read from a model (`grep -rln -i "port\|connector\|RelConnects\|RelNests" crates apps`
   and read the whole list). Port what is there; record in `plumbline/PORTMAP.md`
   either way; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
2. From buildingSMART's own pages for IFC4 (say which you read): how a port is put
   on an element (`IfcDistributionPort`, `IfcRelNests` in IFC4; what IFC2x3 used),
   how two ports are tied (`IfcRelConnectsPorts`), and what `FlowDirection`,
   `PredefinedType` and `SystemType` may say. Write nothing from memory.
3. What the model holds for a connection today: on a thing set down, on a run's
   end, between two runs; what `show` counts as an open end and by which rule.
   Run that count on the first job and give each system's figure: **those are the
   figures the demonstration is held to**, and where one differs from the lead's
   (STATUS.md), say which is right.
4. One fixture with two connections and the two runs that reach it, as it will
   read in the file. The lead reads this when you hand back; go on.

## What is built (call 200, points 1 to 3)
- **Written**: each connection of a thing set down as a port on it, each end of a
  run as a port, two that the model holds as joined tied to one another; a run
  with no size as the run it is, with its line and no body. The file stays valid
  IFC4 by IfcOpenShell (`demos/export.sh`).
- **Read**: what the file's ports say is joined is joined, by the change the
  Connect tool makes, and nothing else is. A connection on a thing comes back as
  the thing's connection at its place. A file with no ports reads as it does
  today, with today's note on open ends. An unsized run comes back with its size
  Unknown and is counted.
- **A fitting whose kind the runs make is not what the file calls it** is brought
  in as what the runs make, and the reply names the place and both words.
- Everything brought in stays Assumed and sourced to the file. Nothing is joined
  because it is near.
- The test marked as waiting is made to run and pass at the figures of answer 3.
- **Every existing expectation that moves is listed before one is changed** (the
  counts of what a written file holds will: ports are new things in it; say each).
- Tests FIRST, run red, quoted. `demos/ifc-whole.sh` gains the open ends by system
  and the hot-water runs; its row in `README.md` says so.

## Rules
- Files you may touch: `crates/ifc/src/`, `crates/ifc/tests/`, `crates/world/src/`
  only where the reader needs a change a tool already makes opened to it (say
  which; no figure's meaning changes), `crates/cli/src/` (what `import ifc` and
  `export ifc` print), `demos/ifc-whole.sh`, existing tests and demonstrations only
  under the rule above, `README.md`, `PORTMAP.md`, `NOTICE` (only if something is
  ported). Nothing in `crates/app/src`.
- The saved job's format does not change. IfcOpenShell is LGPL: run it, never link
  it. No number from a real job.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Watch
  `df -h /`. Demonstrations with `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has the four "before any code" answers; the new tests pass (red runs
quoted) and the waiting test runs; `ifc-whole.sh`, `ifc.sh`, `export.sh` and `model.sh`
pass; every command-line crate's tests and every command-line demonstration pass or
are on the list of expectations moved, each with its reason; the gates pass.
