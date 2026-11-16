# Twenty-third judgment: the money path at the command line, after calls 91 and 92

You did not build this program. Your job is to try to break it, and to say plainly
what you find. You are judging a construction estimating program called plumbline
(`/home/user/roofnerd/plumbline`). Read `/home/user/roofnerd/CLAUDE.md` first for
its fixed rules (above all: "Flag, never fabricate", "One arithmetic for one
number", and that every value is Known, Assumed or Unknown and says where it came
from). Then read calls 91 and 92 at the end of
`/home/user/roofnerd/plumbline/DECISIONS.md`: they are what was just changed.

## Hard limits (a timed run is going on beside you on this machine)

- **Do not run cargo, rustc, or anything that compiles.** Do not start the window
  program (`plumbline-app`), `xvfb-run`, or any script in `demos/` whose name ends
  in `w.sh`, nor `demos/d2.sh` or `demos/house.sh`.
- Use only the program already built:
  `/tmp/claude-0/-home-user-roofnerd/ae04f28e-dd5a-5c73-a560-15cd8a86f760/scratchpad/bin30/plumbline`
  (run it with no arguments to list every verb). It is the build of the commit you
  are judging.
- Work only in
  `/tmp/claude-0/-home-user-roofnerd/ae04f28e-dd5a-5c73-a560-15cd8a86f760/scratchpad/j23/`.
  Change nothing under `/home/user/roofnerd`. Do not commit, push or touch git.
- Every job, name and price you make is made up. Use no real job data.
- **Write your notes as you go** to `j23/NOTES.md` (append after every finding): a
  usage limit can cut you off, and what is not written down is lost.

## What to try to break

Work every figure by hand before you compare it. A finding is: a wrong number; a
number shown as surer than what it rests on; something silently zero, silently
dropped, or left out of a total without the total saying so; a refusal that leaves
the job changed; a sentence a US builder or estimator would not understand, or a
software word in what the program prints.

1. **The verify-in-field list (`plumbline vif`) with a roof.** Make a small house
   (level, wall type, walls, a roof with `plumbline roof ... --status Assumed`),
   bring in the starter assemblies (`plumbline import assemblies <job> starter`),
   apply "Asphalt shingle roof" to a plane, price its lines. Is the roof on the
   list with the right dollars? Is each line counted once in the total at the top?
   Now `plumbline measure` the eave height at a plane, the pitch at a plane, the
   footprint and each edge at the roof ("same" or a length). Does each do what call
   91 says? Does the question leave the list exactly when everything it rests on is
   Known, and not before? Try a gable edge, an L-shaped roof, two roofs, a plane
   with nothing priced, a measurement that changes the eave height (do areas and
   dollars follow? they should not change with eave height alone; do they?). Try
   nonsense readings. Save, `plumbline check`, and read it back.
2. **A drawing's scale on the list.** `plumbline drawing` / `calibrate` (see the
   verbs; `demos/plan.sh` shows how a made-up plan PDF is made with
   `tools/made-up-plan.py` and traced; you may run `demos/plan.sh` itself with
   `PLUMBLINE_BIN=<the bin30 folder>` set, which builds nothing). Is a drawing whose
   scale is Assumed a question, with what was traced over it riding? Does checking
   it take it off and leave the traced walls as questions?
3. **Deleting.** `plumbline remove`: a priced wall with a door in it, a roof drawn
   from its footprint with priced planes, a thing with a line switched off, a thing
   with a line that has no price. Is what is said true to the cent? Is the estimate
   after it what the sentence says?
4. **Labor from the starter library (call 92).** Apply assemblies with labor lines.
   With no production rate: is the line in no total, and does every total that
   leaves it out say so? Give a rate both ways ("40 SF/hr", "0.025 hr/SF") and a
   wage: hours and dollars by hand. A rate of 0, a negative rate, "fast", a rate in
   the wrong unit (LF/hr on an area). Switch the line off and on. Change the wall's
   assembly for another (`apply ... --change`): what happens to a labor line the two
   share, and to its rate and wage? Export the estimate (`plumbline export estimate
   <job> out.xlsx` and `.csv`) and the assemblies: are the hours and the rate there,
   and does a labor line with no rate export honestly?
5. **Overhead and profit**: out of range, over 100, and that the price everywhere
   agrees.
6. Anything else on this path that looks wrong to you.

## What to hand back

A list of findings, each with: a number (J23-1, ...), what you did (the exact
commands), what you expected and why (your hand arithmetic), what the program said,
and how bad it is (wrong money / shown surer than it is / silent / words / small).
Then one paragraph: could you make the money path say something false without it
flagging it? Say what you tried that held, too. Keep the final report under 900
words; the detail belongs in NOTES.md.
