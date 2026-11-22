# Try it: one renovation, on your own desktop

This is the one page for Patrick's first hour with BuildOS on his Linux desktop. Nobody
has yet used the window on a real machine with a real graphics card: this hour is that
test. Write down everything that is wrong, slow, confusing or missing; that list is what
gets built next.

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
inside it is built at full speed either way.) The package list was checked against what the window actually loads
(`tools/installs.sh`: PASS, 2026-11-22). The full install from a clean machine has not
been re-run in this session (the cloud machine is out of disk for a second build), so
if a step fails, the message it prints is the first thing to send back.

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
   file**, pick your file. Its outline lays under the plan. Read the message: it says how
   big it took the house to be and which way it took as up. *If the size is wrong, that
   is the most useful thing you can report* (Scaniverse does not publish its file's
   units or which way is up, so the program reads them from the file and says what it
   assumed).
2. **Trace the house as it stands.** Wall tool (key 1): accept or type the thickness and
   height, click each corner over the scan. Door and Window on the walls. Room in each
   room. Then select the walls and press **Existing** in the panel's WORK row.
3. **Take down and add on.** Select a wall, **To demolish**. Draw the addition's walls.
   Where the plan is open, the **Room line** tool parts two rooms with no wall. Watch
   the rooms' areas follow. The **Demolition** switch under Show hides what comes down.
4. **Set a kitchen.** BRING IN, Starter items. Then the **Cabinets** tool: click a wall's
   face on the room's side, type the boxes (`B18 SB36 B24 DB18 B36`; the codes list as
   you type), Enter. A run off a corner another run fills starts 24 in from it.
5. **Try it two ways.** Save (Ctrl+S). Copy `job.json` to `job-b.json`, open the copy,
   change the addition, save. In the sheet window, **Layouts**, Set beside another
   layout, pick the other file: rooms, walls and cost side by side.

## What is known not to be there yet

So you do not spend your hour looking for it: walls are not found from the scan (you
trace them); no beams, joists or rafters; no island or corner fillers worked out for
you; no printed sheets; prices are yours to type (the starter library carries none);
the takeoff and lists still count existing and demolished work with new; demolition is
a line you type. Each is on the plan in `STATUS.md`.

## What to send back

The scan file's message from step 1; anything that crashed or hung and what you had just
done; a phone picture of anything that looked wrong; and the list of what you reached
for and did not find, in the order you reached for it.
