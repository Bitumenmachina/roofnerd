# Questions for the owner

Each question states the assumption in use until it is answered. No assumption here
enters the money path.

## Q1. Primary cost code: CSI MasterFormat or UniFormat?
**Answered by Patrick, 2026-11-12 (DECISIONS.md call 57):** "the focus is a full 16
division building platform". The primary cost code is the division, one of the sixteen
of MasterFormat 1995. What follows is as it stood before.
**Was open.** MasterFormat codes work results (07 61 00 Sheet Metal Roofing); UniFormat codes
elements (B3010 Roof Coverings). Design-phase pricing by $/SF reads naturally in
UniFormat; a detailed sheet-metal estimate reads naturally in MasterFormat. D4 (macro to
micro) is where the choice bites.
**Assumption in use:** every cost code stores its scheme beside the code, so either can be
primary and both can coexist. The sample job's codes are MasterFormat and marked Assumed.
No total is grouped by code yet.

## Q2. Where does this repository live?
**Ruled, 2026-11-11 (DECISIONS.md call 42, ruling 9):** stay on the branch. Code in the cloud is fine; local-only covers job data and costs when people use the program.
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
**Ruled, 2026-11-11 (call 42, rulings 11 and 12):** installing it in the workspace needs no approval, and the release candidate's IFC4 export is to be validated by it.
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
**Ruled, 2026-11-11 (call 42, ruling 1):** the mission is the whole house, every trade. The earlier program is not built on; its solvers are ported if a step needs them.
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
**Ruled, 2026-11-11 (call 42, rulings 1, 6 and 12):** call 26 stands, the order of work is his and is in CLAUDE.md, and the thirteen items of the release candidate replace the brief's D0 to D7 as what is owed.
You widened the work from the envelope to the whole house and its services (DECISIONS.md
call 26). The model, the takeoff, and a window to see and walk it are built. It is outside
the brief's numbered deliverables, and those (D4 macro to micro, D5 the estimate sheet and
money, D6 interchange) are still where they were.
**Done, 2026-11-11 (DECISIONS.md call 29):** the building model is re-based on BIMCraft's
shapes, with fittings as elements, connectors, a network that says what is joined and
what is open, and rooms found from their walls. So (2) of the list below is built, on
BIMCraft's model and not from nothing.
**Assumption in use:** the next stretch of the whole building is, in this order, (1)
drawing and editing it in the window instead of in a file, (3) clashes. The brief's D4 to
D6 wait behind those unless you say otherwise. If you would rather money (D5) came to the
building before any of that, say so: a pipe, a fitting and a room can each carry a cost
line (a room's floor area and perimeter are figures a flooring or a paint line can use),
and no recipe for one is written.
**New, for you to rule on when you reach them** (each is in PROPOSALS.md, none is built):
whether equipment joins the runs that meet it into one network (11); whether three run
ends meeting with no fitting should be warned of (8).

## Q14. The estimate sheet: what are its columns, and what does a line's labor switch do?
The brief says "Material, labor, and general columns. Labor toggles per line." Your
whole-job estimate is not in `refs/` (Q5), so the sheet is built from those words.
**Assumption in use (DECISIONS.md call 32):** each line is one kind of cost and its
amount sits in one column: material; labor (with fabrication and supervision); general
(subcontract and general conditions). A labor line can be switched off and stays on the
sheet, out of every total. If your sheet has one row carrying both a material amount and
a labor amount, with the switch on the labor half, say so: the arithmetic is the same and
the rows would be paired.

## Q15. Which plan set is traced for the release candidate?
**Answered by a search, 2026-11-11 (DECISIONS.md call 43):** USDA Plan No. 7213, Two-Bedroom Farmhouse, two sheets, public domain by the National Agricultural Library's own statement. It is a scan. If you would rather a set of yours, place a sanitized one in `refs/`.
Item 3 wants a real residential plan PDF: a public set whose licence allows it, or a
sanitized set you place in `refs/`. `refs/` has none.
**Assumption in use:** find a public set on the web whose licence allows it to be used
and kept in the repository (public domain, or a Creative Commons licence without a
no-derivatives term), record where it came from and its licence beside it, and trace
that. If none can be found with a licence that is certain, the tracing is demonstrated
on a plan drawn for the purpose, marked as made up, and item 3 is reported as shown on
a made-up plan and still owed on a real one.

## Q16. What does M mean on screen?
Ruling 4 lists M among the units the trades write.
**Assumption in use:** M is a thousand, as lumber, fasteners and brick are priced (per M,
MBF). It is not metres: metric is still refused, not converted. If you meant something
else by it, say so.

## Q17. What is in each lens: supplier, subcontractor and PM?
Item 8 says three lenses come out of one estimate.
**Assumption in use:** each is a view of the one estimate that leaves out what its
reader is not shown. Supplier: the materials to quote, with quantities and order units
and no prices of ours. Subcontractor: the scope and quantities of the lines classed
Subcontract for that trade, with no prices of ours. PM: every line with its hours,
order quantities and cost codes, the budget by class, and the VIF list; no markup.

## Q18. Scanned plans and aerial images: how far does "snap" go?
Ruling 6 says snap to vector lines where they exist.
**Assumption in use:** a vector PDF's own lines are read out and snapped to. A scan, a
PNG, a JPEG or an aerial image is a picture under the tracing: it is calibrated from one
dimension and traced over by eye, with the program's own snaps (ends, square, angle)
and none to the picture. Finding lines in a picture is not built.

## Q19. A laser set to metres, and the day of a measurement

Two things field entry (order step 4) had to assume.

1. A Leica DISTO set to metres types `7.123m`. Metric is proposal 5 and is refused by
   name everywhere; **working assumption: it is refused here too**, and the refusal says
   to set the laser to feet and inches. If a lasered metre reading should be taken and
   shown in feet, say so; the arithmetic is exact and the source would keep what was
   typed.
2. A value measured in the window is sourced "measured on site" plus whatever the person
   types after it. **Working assumption: the program does not add the day**, because the
   only day it has is the machine's clock. If the record should carry the day (or who
   measured), say what it must hold.

## Q20. Which English, and what is a line's "class" called?

From the fifteenth judgment's list of words on screen.

1. The program writes "storey" and "centre". Its units are US, and US drawings say
   "story" and "center". **Working assumption: left as written**, because a name a
   person typed would be respelled with it if the screen changed these wholesale. If
   the screen should be in US spelling, say so and each word is changed where it is
   written.
2. A cost line's "class" is material, labor, fabrication, supervision, subcontract or
   general conditions. A judge says estimating programs call this "cost type" or
   "category". **Working assumption: "class" stays** until you say which word your
   estimators use.

## Q21. What are "the dollars riding on" a field question?

The verify-in-field list is sorted by them. **Working assumption (DECISIONS.md call
54):** for a value that is not Known, the sum of the estimate's lines whose quantity
is worked from it (a storey height scaled off a plan carries every wall face priced
up to it); for a clash, the sum of the lines priced on the two things in it. The cost
of fixing a clash is not something the program knows, and it does not guess one. If
"riding" should mean something else (an allowance per clash, a percentage, the cost of
the rework), say what.

**Patrick, 2026-11-12, on Q20 and on the screens as a whole (his words):** "I do not see
how the interface is showing a more manageable and Construcion oriented screen and using
English storey and the way things are worded does not seem to fit with good UI and
software design protocols". Work stopped for his review at this point; nothing further
is built until he rules on the screen and its words.

**Answered, 2026-11-12, on his word to "make the call" (DECISIONS.md call 55):**

1. **US English on screen, and a floor of the building is a "level".** Not "story":
   a US drawing set's own sheets and sections say Level 1, Level 2 and "T.O. level",
   and "story" is a count ("two-story"). The screen says level, center, color, meter.
   A name a person typed is not respelled: what is inside quotation marks, and a name
   shown by itself, is shown as typed. The job file and the command line keep the
   model's word (storey), which is IFC's (`IfcBuildingStorey`). **Still his to
   overrule**: if the command line should say level too, say so.
2. "Class" stays until he says which word his estimators use (unchanged).

**Closed, 2026-11-12 (DECISIONS.md call 57).** Patrick: "Why are you asking me on
language when I said US construction it's a known language and you are the driver of
this act like it." Both parts of Q20 are decided by the builder: a level everywhere a
person reads, the command line included; and a line's "class" is its cost type. No
wording question is put to him again.
