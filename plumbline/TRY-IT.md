# Try it: one renovation, on your own desktop

**Reissued 2026-11-22, and judged.** A second judge, cold, did three of the five things
below from the screen and this page alone and two partly (`STATUS.md`, "Call 161 judged").
What it met is written into the steps below so you do not meet it blind, and is being put
right (DECISIONS.md call 169).

This is the one page for Patrick's first hour with BuildOS on his Linux desktop. Write
down everything that is wrong, slow, confusing or missing; that list is what gets built
next. (The manual, `manual/`, takes this page's place when it is made: DECISIONS.md call
237.)

## 1. Install (once, about 20 minutes, most of it the first build)

```
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl git libx11-dev libxcursor1 libxi6 libxkbcommon-x11-0 mesa-vulkan-drivers
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
git clone -b claude/nifty-tesla-qmn44n https://github.com/Bitumenmachina/roofnerd
cd roofnerd/plumbline
cargo build
```

(`cargo build` is the build every test and demonstration here has run on; the 3D engine
inside it is built at full speed either way.) **These steps were run on a clean clone from
GitHub on 2026-11-22:** the build took 23 minutes 41 seconds on a small cloud machine and
all 478 tests passed on it. The package list was checked against what the window
actually loads (`tools/installs.sh`: PASS). If a step fails on your desktop, the message
it prints is the first thing to send back.

## 2. Start a job in its own folder

```
mkdir -p ~/jobs/addition && cd ~/jobs/addition
~/roofnerd/plumbline/target/debug/plumbline-app job.json
```

Two windows open: the model, and the sheet (estimate, budget, field list, layouts).
Put your scan file in the same folder first: in Scaniverse, Share, Export Model, PLY,
and copy that file into `~/jobs/addition/`.

## 3. Five things to try, in order

1. **Bring your scan in.** Left side: Add level (the first floor). Then BRING IN, **Scan
   file**. At the head of the list choose **Measured in meters** or **Measured in feet**
   (meters to begin with), then click your scan in the list. *If it is refused with
   "nothing to draw", or the house comes in several times too big, it is the other
   measure: choose it and click the scan again* (the window does not yet say so). Its outline lays under the
   plan. Read the message: it says how big it took the house to be. If the house came in
   on its side, the message has a button, **It is lying on its side: stand it up**. *If
   the size is wrong, that is the most useful thing you can report* (Scaniverse does not
   publish its file's units or which way is up, so the program reads them from the file
   and says what it assumed).
2. **Trace the house as it stands.** Wall tool (key 1): accept or type the thickness and
   height, click each corner over the scan. Door and Window on the walls. Room in each
   room. Then mark the house existing: click a wall and press **Existing** in the panel's
   WORK row, **one wall at a time** (several cannot be selected together yet).
3. **Take down and add on.** Select a wall, **To demolish**. Draw the addition's walls.
   Where the plan is open, the **Room line** tool parts two rooms with no wall. Watch
   the rooms' areas follow. The **Demolition** switch under Show hides what comes down.
4. **Set a kitchen.** BRING IN, Starter items. Then the **Cabinets** tool: click a wall's
   face on the room's side, type the boxes (`B18 [Dishwasher] SB36 B24 DB18 B36`: the
   codes, the appliances and a one-line key list above the field), Enter. Each box shows
   its code on the plan. The run starts where you click (you can type another start in Starts at); a run off
   a corner another run fills has to start 24 in from it.
5. **Try it two ways.** Save (Ctrl+S). In the sheet window, **Layouts**, **Keep this as
   another layout**: a title is offered, Enter keeps a copy beside this job and sets it
   beside this one. **Open another layout** opens the copy to change it. *A wall cannot
   be made longer or moved with its neighbours yet:* to change the addition, take its
   walls out with Delete and draw them again; save; open the first again, and **Set
   beside another layout** to see rooms, walls and cost side by side.

## What is known not to be there yet

So you do not spend your hour looking for it: walls are not found from the scan (you
trace them); no beams, joists or rafters; no island or corner fillers worked out for
you; no printed sheets; prices are yours to type (the starter library carries none);
in the window the counts are not yet split into new, existing and to demolish (the
command line's are); demolition is a line you type. Each is on the plan in `STATUS.md`.

## What to send back

The scan file's message from step 1; anything that crashed or hung and what you had just
done; a phone picture of anything that looked wrong; and the list of what you reached
for and did not find, in the order you reached for it.
