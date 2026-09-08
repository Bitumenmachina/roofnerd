# roofnerd

Roofing estimating software. You trace the roof on one screen and watch the money
move on the other. Both screens are the same job, open at the same time.

The name is a placeholder.

**Where it is right now:** Gate 0. The program opens, it opens a second window, and a
change you make in one window shows up in the other. That is all it does today. There is
no takeoff yet, no pricing yet, no reports yet. The gates below say what arrives when.

---

## Why this exists

Estimating software makes you flip between the drawing and the sheet. You measure a
parapet, then you leave the drawing to find out what it cost you, then you go back. That
flip is the whole job, done a few hundred times a bid, and it is why takeoff feels like
data entry instead of estimating.

The fix is not a better layout. It is a second window — a real one, that you drag to your
other monitor and work in beside the drawing. A browser tab cannot do that. So this is a
desktop program.

## What it is, and what it is not

**It is** the estimate and the geometry. Trace the roof, price it honestly, send each
party the sheet they need.

**It is not** a scheduling program, an accounting package, or a project management suite.
A Gantt chart is a bonus at the very end, after everything above works.

## Your data is yours

- **Nothing leaves your computer.** The program has no server and makes no network calls.
  The window it runs in is locked down so it *cannot* reach out, and there is a check in
  the build that fails if anyone adds one. The source is public so you can confirm that
  yourself instead of taking our word for it.
- **A job is a folder of plain text.** Open it in Notepad. Grep it. Put it in Dropbox.
  Read it on your phone. Every table exports to CSV, which every spreadsheet on every
  platform opens.
- **MIT licensed.** Use it, sell it, fork it, ship it inside your own product. No fee, no
  permission, no warranty.

## The words, and what each one means

Every screen, every file, and every comment in the code uses these words and no synonyms.
If a screen says something else, that is a defect — report it.

| Word | What it means |
|---|---|
| **Job** | One bid. |
| **Scenario** | One set of prices for that job. A supply house, or a pricing date. You price a job three ways and compare. |
| **Page** | One drawing or aerial photo you are tracing. |
| **Condition** | One thing you traced, with its properties — pitch, height, width, thickness, taper, elevation, sump width, boards, number of sides. A single condition yields square feet, linear feet and a count at the same time, because a parapet run is all three. |
| **Drain** | A traced count that water goes to. The low point of a tapered field. A roof with none has nowhere for the taper to fall to. |
| **Ridge** | The high line a cricket is built along, running between two drains. |
| **Cricket** | The pair of sloped planes built to push water away from something — a curb, a wall — and toward the drains either side. |
| **Taper** | Slope added by the insulation, in inches per foot, on top of whatever the deck already does. Sold as 1/8, 1/4, 1/2. |
| **Elevation** | How high something sits, in feet. Top of the deck on an area; the base of the run on a line. |
| **Sump** | The square cut down around a drain so the tapered field can meet the bowl. Four foot or eight, off the thickness at the drain. |
| **Girth** | The flat width of coil a formed profile eats — the blank strip a piece is rolled from. The shop calls the same measure the *stretch-out*; a quote written either way is read the same. |
| **Item** | One thing a condition consumes: material, labor, equipment, a sub, or something else. |
| **Formula** | The arithmetic that turns what you measured into how much of an item you need. It is printed on the estimate line and you can edit it there. A formula you cannot see is a formula you cannot trust. |
| **Order unit** | What the supply house actually sells — a roll, a sheet, a box, a 5-gallon pail — and the conversion from what you measured to what you order, waste included. |
| **Assembly** | A saved set of items for a condition. Generic, or a specific manufacturer's system. |
| **Library** | The folder of assemblies, profiles, prices and codes. Optional. You can bid a job without ever opening it. |
| **Cost code** | Your own bucket for a cost. Defaults to an 07-xxx pattern; rename them all if you want. |
| **Class** | The recap bucket a cost rolls into, each with its own adders: Material, Labor, Sub, Equipment, Other, Supervision. |
| **Recap** | The roll-up to selling price. Every adder — tax, burden, escalation, overhead, profit, bond — is editable. |
| **Lens** | A report. One estimate, filtered for one party. |

Units are **SF**, **LF**, **EA** and **SQ**. Those four, spelled that way, everywhere.

### Why the recap has classes

A job showing fifteen thousand dollars of "material" is not fifteen thousand dollars of
material. It is roofing material, plus a dumpster, plus a permit, plus equipment rental.
Lumping those together produces a number that looks honest and isn't. So every cost
carries a code, every code sits in a class, and each class carries its own adders —
material gets sales tax and escalation, labor gets burden, supervision gets burden at its
own rate, subs get general liability. The recap adds up what is actually there.

### One estimate, several audiences

The same finished estimate prints differently depending on who is receiving it:

- **Supply house** — the item list in order units. No costs, no waste.
- **Subcontractor** — the measurements and the takeoff. No internal assumptions.
- **Project manager** — their slice.
- **Accounting** — theirs.

Nothing is retyped, and nothing gets sent to someone who should not see it. This is a
print filter, not a login system: there are no accounts, no permissions and no users.

---

## The gates

Each one is finished when it does the thing next to it. No dates.

| | | Finished when |
|---|---|---|
| **Gate 0** | Skeleton | Two windows, one job. A change in one shows in the other. **← here** |
| **Gate 1** | The working loop | Trace a parapet on a PDF and watch wall flashing SF, coping LF and corner count appear, with the total moving in the other window. |
| **Gate 2** | The money | Cost codes, classes, labor from production rates, waste, order units, the recap, the library. Checked by reproducing two real Edge bids from their own quantities and prices. |
| **Gate 3** | Lenses | The five reports as templates you can edit. CSV, XLSX and PDF out. |
| **Gate 4** | 3D | Facets, parapets, tapered insulation and crickets as real slopes — and a low spot with no drain on it, visible. |
| Bonus | Gantt | Only after Gate 4. |

Gate 2 gets checked against real bids that live on the estimator's own machine and are not
in this repository. The public build checks the *structure* of the recap against a made-up
demo job. Both halves have to pass.

---

## Running it

You need [Rust](https://rustup.rs), [Node](https://nodejs.org) 20 or newer, and pnpm. On
Fedora you also need the webview libraries:

    sudo dnf install webkit2gtk4.1-devel libsoup3-devel gtk3-devel \
                     javascriptcoregtk4.1-devel librsvg2-devel openssl-devel

Then:

    pnpm install
    pnpm dev            # opens the program

The estimating math is a separate package that runs without opening a window at all:

    node packages/engine/bin/roofnerd.mjs recap jobs/demo-job

That is deliberate. The arithmetic is testable on its own, it can run from a script or a
scheduled job, and nothing about a window is allowed to change a number.

---

## How the code is laid out, in roofing terms

Think of the program as a commercial roof.

- **`packages/engine` is the deck.** Everything bears on it. It is pure arithmetic — the
  vocabulary above as types, the conversions, the labor, the recap. It knows nothing about
  windows, drawings or files. If a number is wrong, it is wrong here, and there is a test
  for it here.
- **`src-tauri` is the structure.** It holds the one open job and hands the same copy to
  every window. Windows do not own the job; they subscribe to it. That is why two windows
  can never disagree.
- **`packages/app` are the systems on the deck** — the drawing editor, the estimate sheet,
  the recap, the library. Each is one editor, each can be torn off into its own window, and
  any of them can sit in any area of the screen.
- **The formula on the estimate line is the flashing.** It is the joint where the geometry
  meets the money, it is the part that leaks when it is hidden, and so it is the part kept
  visible and editable at the point of use.

Comments in the source explain structure the same way.

## Contributing

Two rules that matter more than style:

1. **Nothing gets added that nobody asked for.** Features appearing unbidden is the defect
   this project was started to escape.
2. **The words in the table above are the only words.** No synonyms in the UI, the file
   format, the types or the comments.
