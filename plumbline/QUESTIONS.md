# Questions for the owner

Each question states the assumption in use until it is answered. No assumption here
enters the money path.

## Q1. Primary cost code: CSI MasterFormat or UniFormat?
**Open.** MasterFormat codes work results (07 61 00 Sheet Metal Roofing); UniFormat codes
elements (B3010 Roof Coverings). Design-phase pricing by $/SF reads naturally in
UniFormat; a detailed sheet-metal estimate reads naturally in MasterFormat. D4 (macro to
micro) is where the choice bites.
**Assumption in use:** every cost code stores its scheme beside the code, so either can be
primary and both can coexist. The sample job's codes are MasterFormat and marked Assumed.
No total is grouped by code yet.

## Q2. Where does this repository live?
**Answered by the owner, 2026-11-10.** Asked whether to push it to the public repository
of his earlier program, on the working branch of the session that carried this work on,
as a `plumbline/` folder with its history, he said yes. He also said that "local only"
in his rules is about job data, not about the code. It lives there until he says
otherwise. Job, client and supplier data and the ruling documents still never go in:
`refs/` is ignored by git and the sample job is made up.
**Still open:** whether that is its permanent home or it gets a repository of its own,
and whether the working branch is merged (QUESTIONS.md Q11 is the related question).

## Q3. Are MPL-2.0 crates allowed?
**Open.** Rule 1 names GPL and AGPL as reference-only. MPL-2.0 (ifc-lite, bimifc,
boolmesh) is file-level copyleft: an unmodified MPL crate can be linked from a permissive
program, but changes to its own files must be published.
**Assumption in use:** not linked. Nothing needs one before D6.

## Q4. Toolchain for the 3D window
**Decided for you, on your instruction, 2026-11-10** (DECISIONS.md call 23): rustc 1.97.1
is installed and pinned, `libxkbcommon-x11-0` is installed, and the window runs on a
virtual display. Still yours: nothing. What follows is the history.
**Was open.** Bevy 0.20 needs rustc 1.97.1 or newer. This workspace has 1.97.0. Updating the
toolchain changes files outside the repository, which needs your go-ahead.
**Assumption in use:** not updated. The world crate needs only rustc 1.95 and builds here.
The D2 viewport proof (DECISIONS.md, call 2) cannot run in this workspace until this is
ruled on, or the repository is on a machine with a current toolchain.

**Facts gathered since, to help you rule** (second stretch, 2026-11-10; a scratch folder
outside the repository, nothing added to the project, nothing installed):
* `cargo build` of a one-line Bevy 0.20.1 program on rustc 1.97.0 is refused, as expected.
* The same build with cargo's `--ignore-rust-version` **compiles** (6 min 29 s on 4 cores,
  with a 3D feature set and audio and gamepad left off). So the toolchain update may not
  be needed to build; whether to build against a compiler the engine says it does not
  support is yours to say. It was compiled, not run past start-up, so nothing is known
  about whether it behaves.
* **It cannot open a window in that workspace regardless.** Under a virtual display
  (`Xvfb`, already installed) it stops at start-up: the window library needs
  `libxkbcommon-x11.so.0`, which is not installed. Installing it is a system package.
  A software renderer was present (SwiftShader, shipped with the workspace's browser) but
  was never reached, so rendering is untested.
* So the opening proof needs one of: a machine with a display; or your go-ahead to install
  `libxkbcommon-x11-0` there and try the virtual display, which would prove the window,
  the second window and the readout, and leave "picked with the mouse" to a simulated
  pointer.

## Q5. Reference estimates
`refs/` is empty: no copper sheet-metal estimate and no whole-job estimate were provided.
**Assumption in use:** D3 will be built against the numbers in the brief (eave 12 in, step
flashing 8 in, valley 36 in, headwall 24 in) and D3 and D5's reproduction demonstrations
will be marked pending in STATUS.md until the spreadsheets are in `refs/`.

## Q6. D7 capture
The brief says you rule on capture before work starts. No work has started on it.

## Q7. IfcOpenShell for D6 validation
IfcOpenShell (LGPL-3.0) is the validation oracle for IFC4 export. It is run as a separate
tool and never linked, but running it means installing a package, which needs your
go-ahead. Not needed before D6.

## Q8. Name
"plumbline" is a working name. It appears in the crate names, the binary name, and the
`pl::` prefix on component keys in the saved file. Renaming before real jobs are saved
costs a search-and-replace; after, it costs a file migration.

## Q9. How tightly must a traced line fit its planes?
A hip is refused a length unless both planes put each of its ends within 0.01 ft (1/8 in)
of the same height, and its ends sit within 0.01 ft of both outlines. That is right for
geometry the program solves. On a hand trace of a scanned plan it will flag most hips.
**Assumption in use:** 0.01 ft, because D2 replaces hand-entered hips with solved ones.
If you want hand tracing to be a first-class route, this needs a snap step or a looser
figure, and you would know what tolerance a trace deserves better than a default does.

## Q10. The BIMCraft source for the skeleton port
**Closed, 2026-11-10.** Read access was available after all; the skeleton is ported
(DECISIONS.md calls 3 and 23). What follows is the history.
**Was open; it blocked the roof solver.** DECISIONS.md call 3 is to port BIMCraft's weighted
straight skeleton (`IPedrax/BIMCraft`, `crates/core/src/skeleton.rs`, MIT OR Apache-2.0).
The workspace the second stretch ran in may read one repository only, and the request to
add BIMCraft was declined, so the file was not fetched. Any of these unblocks it: allow
that repository to be read; start the work somewhere that can clone it; or put
`skeleton.rs`, its `geom` module and the commit they came from in `refs/`.
**Assumption in use:** none. Nothing was written in place of the port. The work done
meanwhile is what the port will plug into (DECISIONS.md call 17).

## Q11. Is this project meant to take over the ground of the earlier program?
The earlier estimating program on the same machine already solves the low-slope side:
tapered packages, crickets, copings, parapet flashing heights (DECISIONS.md call 18). D2
asks for "a low-slope area with tapered crickets" and D3 for a parapet's coping and
flashing.
**Assumption in use:** borrow its solvers by porting them when D2 and D3 reach those
steps, with attribution, and leave that program alone. If you would rather the two share
one core crate, or that this project not touch that ground yet, say so before the port.

## Q12. Is the SIL Open Font License acceptable for the program's typeface?
The windows are set in Inter, which is built into `plumbline-app`. Inter is under the SIL
Open Font License 1.1, not MIT or Apache. The OFL allows a font to be bundled, embedded
and redistributed with any software, including sold software; what it restricts is the
font itself (a changed font must stay under the OFL and cannot be sold on its own). It
places nothing on the program. `tools/licenses.sh` checks crates, not fonts, so this is
recorded by hand in LICENSES.md.
**Assumption in use:** yes. If not, the alternative is to use whatever sans-serif the
machine has (`FontSource::sans_serif()`), at the cost of the program looking different on
every machine and having no text at all on one with no fonts.

## Q13. The whole building: what comes next, and where it sits against the brief
You widened the work from the envelope to the whole house and its services (DECISIONS.md
call 26). The first stretch is built: the model, the takeoff, and a window to see and walk
it. It is outside the brief's numbered deliverables, and those (D4 macro to micro, D5 the
estimate sheet and money, D6 interchange) are still where they were.
**Assumption in use:** the next stretch of the whole building is, in this order, (1)
drawing and editing it in the window instead of in a file, (2) fittings and connections,
so a system can be checked and its fittings counted by type, (3) clashes. The brief's D4
to D6 wait behind those unless you say otherwise. If you would rather money (D5) came to
the building before any of that, say so: a pipe can already carry a cost line, and no
recipe for one is written.

