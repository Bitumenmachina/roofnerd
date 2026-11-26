# Every kind of file, in and out, from the window: a demonstration, written before it is built

Call 237, step 6. Call 220's first test: a file a builder already has comes in, and goes
out again in a form another program takes. The command line passes it for every kind
below; the window takes seven kinds. This is the window's half. Nothing new is worked
out: each entry calls the function the command line calls, and **the check is that the
job the window saves is byte for byte the job the command line saves from the same
file** (the header's time set aside), and that a file the window writes is byte for byte
the file the command line writes. One arithmetic for one number, shown by checksum.

The script is `demos/filesw.sh`. Every file in it is made up and already in `demos/`.

## The words on the buttons (the lead's call; no software term)

A file is picked from a list; nobody types where it lies. No new button is made where
an existing one already means the thing:

| What comes in or goes out | Where in the window |
|---|---|
| A CAD drawing (DXF) or a TIFF as a plan | **Add drawing**: the list shows them with the PDFs and pictures. A CAD drawing is listed as "CAD drawing", a TIFF as "picture". A TIFF of several pages asks for its page as a PDF does. |
| The layers of a CAD drawing switched off | On the drawing's panel: one switch to a layer, by the file's own names. |
| A scanner's own file (LAS, LAZ, E57) | **Scan file**: the list shows them with the PLY scans, each as "scan". |
| The measured marks on a PDF | When a PDF added as a drawing has measured marks, the drawing's panel says how many and offers **Bring in its marks**. Counts are brought with a second switch, **Counts too**. |
| A maker's model (IFC of one product) as an item | Under Bring in: **Maker's model**, beside Maker's sheet. |
| Every table of the job in one workbook | Under Bring in: **My database**. Under Write out: **My database**. |
| The field's sheet | Under Write out: **Field sheet**. Under Bring in: **Field sheet**. |
| The clashes as an issue list another program opens | Under Write out: **Issue list (BCF)**. |
| The house as one file to look at anywhere | Under Write out: **3D file (glTF)**. |
| The plan of the level in view as a CAD drawing | Under Write out: **Plan as CAD (DXF)**. |

What the window says after each is the sentence the command line says, taken from the
model (not written a second time in the window): what was read, what was left out and
why. A file that cannot be read is refused with the reason, and the job is as it was.

## The steps, each with its check

Each step starts from a job made at the command line and saved; the same job is copied,
the command line does the thing to one copy and the window to the other.

1. **A CAD drawing as a plan.** Job: one level. Window: Add drawing, pick
   `plan-inches.dxf`. Command line: `plumbline drawing <job> ground "plan-inches"
   --file demos/dxf/plan-inches.dxf`. Check: the two saved jobs are the same; the window
   shows the drawing's lines under the level (a picture); the scale needs no setting (the
   file says inches) and the window says so in the model's words.
2. **A layer switched off.** On `plan-with-extras.dxf`: one layer off in the window, the
   same by `plumbline layer <job> <drawing> "<layer>" off`. Check: the saved jobs are the same
   (`layers_off` holds the one name); the picture shows the layer gone.
3. **A TIFF of two pages.** Add drawing, pick `two-pages.tif`, page 2. Check: the saved
   job is the command line's with `--page 2`.
4. **A scanner's file.** Scan file, pick a made-up LAS, a LAZ and an E57 in turn (made by
   `demos/scan-files/make.py`, as `demos/scan-files.sh` makes them). Check: each saved
   job is the command line's `import scan`; the sheet to trace shows under the level.
5. **Marks on a PDF.** Add drawing, pick `marked-plan.pdf`: the drawing's panel says the
   count of measured marks the command line's `plumbline marks` gives for that page.
   Bring in its marks. Check: the saved job is the one `import marks` gives; the estimate
   window shows the lines under the file's name, each Assumed. `unreadable.pdf` is
   refused in the command line's words and the job does not change.
6. **A maker's model.** Maker's model, pick `nl-2018.ifc`. Check: the saved job is
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
10. **Nothing is lost silently.** For each file brought in, the window's message
    (`<map>.said`) is the command line's reply, sentence for sentence.

## What is not in this demonstration

A cabinet made by a rule saying so in the window, and the cabinet rules as their own
entry (they travel with My database), stay with contract 219WA. An issue list read
back, and a mesh brought in, are not built at the command line either.
