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
**Open.** It was started in a session workspace that has no remote and does not outlast
the session. Pushing to a remote needs your go-ahead.
**Assumption in use:** local commits only. The repository is handed over as a git bundle.

## Q3. Are MPL-2.0 crates allowed?
**Open.** Rule 1 names GPL and AGPL as reference-only. MPL-2.0 (ifc-lite, bimifc,
boolmesh) is file-level copyleft: an unmodified MPL crate can be linked from a permissive
program, but changes to its own files must be published.
**Assumption in use:** not linked. Nothing needs one before D6.

## Q4. Toolchain for the 3D window
**Open.** Bevy 0.20 needs rustc 1.97.1 or newer. This workspace has 1.97.0. Updating the
toolchain changes files outside the repository, which needs your go-ahead.
**Assumption in use:** not updated. The world crate needs only rustc 1.95 and builds here.
The D2 viewport proof (DECISIONS.md, call 2) cannot run in this workspace until this is
ruled on, or the repository is on a machine with a current toolchain.

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
