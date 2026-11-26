# BuildOS

A 3D estimating program for the whole house, every trade. Trace the drawings, build the 3D
house from the tracing, price it division by division, and see the clashes the drawings hid.

The program is in [`plumbline/`](plumbline/README.md) (its working name; the folder and the
commands still carry it). Start there: `plumbline/README.md` says how to install it and run it,
`plumbline/STATUS.md` where the build stands, `CLAUDE.md` how the work is done.

`tools/` holds the three gates every commit passes: no word a builder would not use
(`gate-language.sh`), no connection opened by the program (`gate-egress.sh`), no real job's
figures in the repository (`gate-clientdata.sh`).

The earlier program this grew from, roofnerd (roofing estimating in a Tauri shell), is on the
branch `main`. It stood at the root of this branch until 2026-11-26 and left it whole
(`plumbline/DECISIONS.md`, call 237): nothing of it was changed here.
