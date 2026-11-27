# Every kind of file, in and out, from the window: a demonstration, written before it is built

Call 237, step 6. Call 220's first test: a file a builder already has comes in, and goes
out again in a form another program takes. The command line passes it for every kind
below; the window takes seven kinds. This is the window's half. Nothing new is worked
out: each entry calls the function the command line calls, and **the check is that the
job the window saves is byte for byte the job the command line saves from the same
file** (the header's time set aside), and that a file the window writes is byte for byte
the file the command line writes. One arithmetic for one number, shown by checksum.

The script is `demos/filesw.sh`. Every file in it is made up and already in `demos/`.

## Where each thing is in the window (calls 240 and 242: two sheets, not a button to a kind)

A file is picked from a list; nobody types where it lies. The job's menu has **Bring
in** and **Write out**; each opens one sheet.

**Bring in** lists the files of the job's folder, each with what it is, in the words
`plumbline kind` says (`demos/kind.md`): the same function. One button, **Bring in**.
Where a file could be two things ("a price list, or a prior estimate") the sheet asks
which, once, before anything is read. A kind that needs something more asks for it in
the sheet: a plan its level and its page; a scan its level; prices how sure they are
and where they came from. Starters (the starter assemblies, the starter tool chest)
are rows under their own heading.

| What comes in | How it is listed |
|---|---|
| A PDF, a picture, a TIFF, a CAD drawing (DXF) as a plan | "Plan", "Plan (a picture)", "Plan (a CAD drawing)"; a TIFF or PDF of several pages asks for its page |
| The layers of a CAD drawing switched off | In Layers, under the plan's own row: one switch to a layer, by the file's own names |
| A scan (PLY, LAS, LAZ, E57) | "Scan" |
| The measured marks on a PDF | The row says "with N measured marks"; brought in as a plan, its panel offers **Bring in its marks**, and **Counts too** |
| A designer's or another trade's model; a maker's model | "Designer's model"; "Maker's model" |
| A maker's sheet | "Maker's sheet" (a PDF said to be a plan, or a maker's sheet: the sheet asks) |
| A price list, assemblies, a prior estimate, items, shapes, connections, cabinet rules | by their headings |
| Every table of the job in one workbook | "My database" |
| The field's sheet, filled | "Field sheet" |

**Write out** lists what can be written, grouped by who it is for (call 242, point
5); a row is a name and the file it makes. One button, **Write out**.

Every control that was a button under Bring in or Write out keeps its name on the
map and is reached through the sheet (call 242, point 3).

What the window says after each is the sentence the command line says, from the same
function in `crates/files` (not written a second time in the window): what was read,
what was left out and why. A file that cannot be read is refused with the reason,
and the job is as it was.

## The steps, each with its check

Each step starts from a job made at the command line and saved; the same job is copied,
the command line does the thing to one copy and the window to the other.

1. **A CAD drawing as a plan.** Job: one level. Window: Bring in, pick
   `plan-inches.dxf` (listed as a plan, a CAD drawing). Command line: `plumbline drawing <job> ground "plan-inches"
   --file demos/dxf/plan-inches.dxf`. Check: the two saved jobs are the same; the window
   shows the drawing's lines under the level (a picture); the scale needs no setting (the
   file says inches) and the window says so in the model's words.
2. **A layer switched off.** On `plan-with-extras.dxf`: one layer off in the window, the
   same by `plumbline layer <job> <drawing> "<layer>" off`. Check: the saved jobs are the same
   (`layers_off` holds the one name); the picture shows the layer gone.
3. **A TIFF of two pages.** Bring in, pick `two-pages.tif`, page 2. Check: the saved
   job is the command line's with `--page 2`.
4. **A scanner's file.** Bring in, pick a made-up LAS, a LAZ and an E57 in turn (made by
   `demos/scan-files/make.py`, as `demos/scan-files.sh` makes them). Check: each saved
   job is the command line's `import scan`; the sheet to trace shows under the level.
5. **Marks on a PDF.** Bring in, pick `marked-plan.pdf` as a plan: its row and the drawing's panel say the
   count of measured marks the command line's `plumbline marks` gives for that page.
   Bring in its marks. Check: the saved job is the one `import marks` gives; the estimate
   window shows the lines under the file's name, each Assumed. `unreadable.pdf` is
   refused in the command line's words and the job does not change.
6. **A maker's model.** Bring in, pick `nl-2018.ifc` (listed as a maker's model). Check: the saved job is
   `import maker-model`'s; the item is in the tool chest with its sizes Assumed and its
   connections; `nl-2018-no-maker.ifc` says what the command line says of it.
7. **The database out and back.** The sample whole house. Write out My database. Check:
   the workbook is byte for byte `export database`'s. Change one price in it with
   `tools/workbook.py` as `demos/database.sh` does, Bring in My database. Check: the
   saved job is `import database`'s, and the price in the estimate window is the new one.
8. **The field's sheet out and back.** The sample house. Write out Field sheet. Check:
   byte for byte `export field-sheet`'s. Fill three rows as `demos/field-sheet.sh` does,
   Bring in Field sheet. Check: the saved job is `import field-sheet`'s; three values are
   Known, and the count of questions on the Verify in field tab falls by what the
   command line says.
9. **The issue list, the 3D file and the plan as CAD.** The sample house. Write out
   each. Check: each file is byte for byte `export bcf`, `export gltf` and `export dxf
   --level` for the level in view; `tools/bcf-check.py`, `tools/gltf-check.sh` pass on
   them.
10. **A file that could be two things is asked about, once.** Bring in, pick
    `tinners-yard.csv`: the sheet asks "a price list, or a prior estimate"; Price
    list, with how sure and where from. Check: the saved job is `import prices`'s.
11. **Ten kinds are rows, not buttons.** `demos/quietw.sh` counts what can be pressed
    at rest with the sheets closed: it is what it was before the kinds were added.
12. **Nothing is lost silently.** For each file brought in, the window's message
    (`<map>.said`) is the command line's reply, sentence for sentence.

## What is not in this demonstration

A cabinet made by a rule saying so in the window, and the cabinet rules as their own
entry (they travel with My database), stay with contract 219WA. An issue list read
back, and a mesh brought in, are not built at the command line either.
