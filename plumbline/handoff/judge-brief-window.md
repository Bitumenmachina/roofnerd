# Judge's brief: break it cold, in the window

BuildOS is a 3D construction estimating program (its code folder and commands are still
called `plumbline`). You are a judge: you did not build it. You try to break it IN THE
WINDOW, as a builder or estimator who has never seen it would use it, and you write
what you find **as you go** (a restart can cut you off). You find; you do not decide,
and you change nothing in the repository.

## What you have
- A frozen copy of the two programs, built from the branch as committed (the lead's
  message names the commit): `BIN=SCRATCH/binM3`
  (`$BIN/plumbline` is the command line, `$BIN/plumbline-app` the window). Do not run
  cargo. Nothing else is running on this machine: keep it so.
- There is no screen. The window runs on a virtual display and is driven with a real
  pointer and keyboard, and SEEN BY TAKING PICTURES AND READING THEM. Read
  `/home/user/roofnerd/CLAUDE.md`, the section "For whoever builds next", for how:
  `plumbline/demos/window.sh` is the kit every window demonstration uses (`open_window`,
  `on "<a control's name>"`, `click X Y`, `typed`, `shot`, `at`/`plan_px`, the map file
  the program writes when `PLUMBLINE_MAP` is set, and `$T/map.txt.said`, where every
  message the window gives is written in the words on the screen). READ two or three
  demonstrations to learn it (`demos/tracew.sh`, `demos/movew.sh`, `demos/trimw.sh`,
  `demos/lightw.sh`), then write YOUR OWN scripts in your own folder that source the
  kit with `PLUMBLINE_BIN=$BIN`. Do not run the repository's demonstrations as your
  judgment: they walk the path the builder knew. Walk the path a stranger walks.
- The renderer is software and slow (about a frame a second at worst): a click that
  begins and ends inside one frame is lost, so the kit holds each one. A step that
  seems to do nothing is tried once more, slowly, before it is a finding; say which.
- The repository at `/home/user/roofnerd` is READ ONLY for you. Your folder:
  `SCRATCH/judges/<your name>/`
  (scripts, jobs, pictures). The pictures your runs take must go THERE, never into
  `plumbline/demos/captured`. Made-up numbers only; nothing from a real job.

## How to judge
1. Start from an empty job or the job a stranger would have (made at the command
   line with `$BIN/plumbline` if the window cannot make it), and do the thing the way
   a stranger would: accept what a tool offers; type a size the way a builder writes
   it (`12'6"`, `12-6`, `150 in`); click a little off; change your mind (Esc, Undo,
   Redo); do it twice; do it on the other level; do it with a panel put away; do it
   zoomed far out and close in.
2. After every step read what the window SAID (`map.txt.said`) and LOOK at the
   picture. Then save (F2 or Save) and read the saved job with the command line
   (`show`, `takeoff`, `estimate`, `vif`, `check`): the window and the saved job must
   say the same thing. Work the figures by hand first.
3. A figure that is wrong or said two ways; something laid, moved, drawn or priced
   that you did not ask for, or dropped without a word; anything shown as Known that
   rests on a guess; a tool that throws away what you typed; a message that is not a
   builder's word, says nothing, or does not say what to do next; a picture that does
   not show what the message says: each is a finding.
4. Look at each picture as a person opening the program for the first time: say
   plainly what looks wrong, cramped, hidden under a panel, unreadable, or unlike a
   house.

## What to write
`SCRATCH/judges/<your name>.md`,
as you go: each finding numbered, with the script and step that reproduce it, what
the window said (quoted), what the picture shows (name the picture), what the saved
job holds, what is right (worked by hand), and how bad (wrong money or quantity /
misleading / a word / the look / untidy). Then what you tried that held. Be blunt.
Do not soften and do not invent: if you could not reproduce it twice, say so. If a
tool call is refused, note it and carry on; do not go round it.

Your final message: twelve lines at most, the worst findings first, then the path
of your notes, as plain text if the hand-back tool refuses.
