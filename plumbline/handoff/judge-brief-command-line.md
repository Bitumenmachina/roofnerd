# Judge's brief: break it cold (command line)

BuildOS is a 3D construction estimating program (its code folder and commands are still
called `plumbline`). You are a judge: you did not build it. Try to break one piece of it
at the command line, as a builder or estimator who has never seen it would use it, and
write what you find **as you go** (a restart can cut you off). You find; you do not
decide, and you change nothing in the repository.

## What you have
- A frozen copy of the program, built from the branch as committed at b207e7b:
  `B=SCRATCH/review/bin33/plumbline`.
  `$B` alone lists every verb with its words. Do NOT start `plumbline-app` (the window),
  do not run cargo, do not run any script in `plumbline/demos/` (another agent's timed
  runs share this machine); you may READ the demonstration scripts to see how a verb is
  used, and copy their commands into a folder of your own.
- The repository at `/home/user/roofnerd`, READ ONLY for you: `CLAUDE.md` (what the
  program must be: the fixed rules), `plumbline/DECISIONS.md` (search `### Call N` for
  the decisions named in your prompt), `plumbline/STATUS.md` (search the sections named
  in your prompt for the hand figures), `plumbline/tools/made-up-scan.py` (makes made-up
  scans; you may run it, writing into your own folder).
- A folder of your own:
  `SCRATCH/judges/<your name>/`.
  Work only there. Made-up numbers only; nothing from a real job.

## How to judge
1. Start from an empty job (`$B new`), not from a demonstration's finished job, and do
   the thing the way a stranger would: guess the verb's words from its usage line, make
   the mistakes a person makes (a wrong unit, a size typed as `3'6"`, a missing flag, a
   name with a space, the same thing done twice, things done in another order).
2. Work every figure the program gives you by hand and compare. A figure that is wrong,
   that is said two ways in two places, that is silently zero or silently dropped, or
   that reads as Known when it rests on something Assumed or Unknown, is a finding.
3. Save the job, read it back (`$B show`, `$B check`, `$B estimate`, `$B takeoff`,
   `$B vif`, `$B clashes`), write it out (`$B export ...`) and read what was written.
4. Read every sentence the program prints as a builder would: a software word (a file
   ending, "path", "null", an identifier shown raw where a name is wanted), a sentence
   that says nothing, a refusal that does not say what to do instead, is a finding.
5. Try to make it lie: find an input where it says something confident and wrong.

## What to write
`SCRATCH/judges/<your name>.md`,
as you go: each finding numbered, with the exact commands that reproduce it from an
empty job, what the program said (quoted), what is right (worked by hand), and how bad
(wrong money or quantity / misleading / a word / untidy). Then what you tried that held.
Be blunt. Do not soften, and do not invent: if you could not reproduce it twice, say so.
If a tool call is refused, note it and carry on; do not go round it.

Your final message: ten lines at most, the worst findings first, then the path of your
notes, as plain text if the hand-back tool refuses.
