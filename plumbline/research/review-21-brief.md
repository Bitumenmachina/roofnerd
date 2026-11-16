# Brief for a code review by a reviewer that did not build it (2026-11-16)

You are reviewing code you did not write. Find what is wrong in it: defects first,
then what makes it hard to keep. Do not fix anything and do not edit the repository.
Write only inside your own folder, RDIR (given to you when you were started).

Write RDIR/NOTES.md **as you go**, one finding at a time (R21-1, R21-2, ...). A run
can be cut off; notes written as you go survive. Each finding: the file and line,
what the code does, why that is wrong or fragile, how serious (a wrong number or a
lost value; a crash; a wrong word; untidy), and, wherever you can, a reproduction: the
exact commands on a made-up job with the output you got and the output you expected,
worked by hand. A finding you reproduced is worth ten you only suspect; say which is
which.

## What this is

plumbline (repository /home/user/roofnerd, folder plumbline/) is a 3D estimating
program for a whole house: a Rust workspace. Read /home/user/roofnerd/CLAUDE.md first
(the fixed rules: flag never fabricate; every value Known, Assumed or Unknown with
where it came from; one arithmetic for one number, worked in the core and never
recomputed in a window; nothing silently zero or silently dropped; a total that
leaves something out says so; no real job data anywhere). Then plumbline/SPEC.md for
the model. plumbline/DECISIONS.md says why things are as they are: before calling a
behaviour a defect, search it for the decision (grep the word); a decision you think
is wrong is still worth a finding, said as such.

    crates/world   the model and every figure (bevy_ecs)   crates/cli     the command line `plumbline`
    crates/plans   PDFs, pictures, makers' sheets          crates/sheets  xlsx and CSV
    crates/ifc     IFC in and out                          crates/app     the two windows (Bevy)

## Rules of the review

- **Do not run cargo, and do not build.** The builder is building in the same tree.
  A built copy of the command line is given to you as BIN/plumbline; run it alone to
  list every verb. plumbline/demos/*.sh (the ones not ending in w) show how each verb
  is used; plumbline/demos/found.sh and d5.sh are good first reads.
- Made-up jobs only, in RDIR. No real job data of any kind.
- Work every figure by hand before you compare it. Any cent of difference is a finding.
- Read the code itself, whole files, not a search for patterns. Say which files you
  read whole and which you did not get to.

## What to look for, in this order

1. **A wrong number.** Quantities, formulas, waste, order rounding, unit changes,
   stretch-out, labor switched off, allowance against detail, overhead and profit,
   totals by cost type and by division, the verify-in-field dollars, what a total
   leaves out. Rounding done at more than one layer. A figure worked twice in two
   places that can disagree. A status (Known, Assumed, Unknown) that comes out surer
   than what it rests on.
2. **A value lost or made up.** Something saved that does not read back the same;
   something read (CSV, xlsx, IFC, a maker's sheet) and silently dropped or defaulted;
   a zero where there is no figure; a default nobody entered.
3. **A crash or a hang** reachable from what a person can type, click or bring in:
   unwrap, expect, indexing, slicing a string at a byte, a division, a loop that
   depends on its input ending.
4. **A refusal or a message that says something untrue,** or a software word a
   builder would not use, anywhere a user sees.
5. **What makes it hard to keep:** the same logic written twice, a function doing
   several jobs, a name that says the wrong thing, dead code, a comment that no
   longer matches its code, a test that cannot fail.

## How to report

End with RDIR/REPORT.md: every finding, most serious first, each marked reproduced or
suspected; the files you read whole; and the ten changes you would make first.
