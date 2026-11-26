# BuildOS manual

This is the one page that says how each task is done in BuildOS and what state the
program is in. When a task changes, its section and its picture are replaced. Nothing
is added beside them. A picture has the commit it was taken on below it. A task with
no picture of the present build says so.

Each task below names the demonstration that proves it. A demonstration is a script
that operates the real program with a pointer and a keyboard, and checks the result.

## 1. Start

1. Install the program. `README.md` has the three steps.
2. Make a sample house: `cargo run -p plumbline -- sixteen house.json`
3. Open it: `cargo run -p plumbline-app -- house.json`

Two windows open. The **model window** shows the house. The **estimate window** shows
the price. Each change in one window shows in the other.

To start an empty job, give the name of a file that does not exist. The panel
**Next steps** tells you what to do first.

[picture: two-windows | The model window and the estimate window, open on the sample house.]

Proven by `demos/dressw.sh` and `demos/pricew.sh`.

## 2. The model window

- The **view bar** is at the top: **3D**, **Plan**, **Walk**, **Frame**, **Sun**.
- The **Job** panel is at the left: Next steps, Levels, Show, Drawings, Bring in,
  Write out.
- The **Details** panel is at the right. It shows the thing that you selected.
- The **Tools** bar is at the bottom: Building, then Mechanical, Electrical and
  Plumbing, then Edit.
- Three switches at the lower right put each panel away and bring it back.

On a small screen the panels fold. The size of the lettering has four steps. The
program keeps the place and the size of each window for the next start.

If the program stops before you save, it keeps a recovery copy. At the next start it
offers the copy to you.

The picture in section 1 shows each of them.

Proven by `demos/glassw.sh` and `demos/screensw.sh`.

## 3. Move the view

- **Wheel**: the view dives at the place under the pointer. One wheel goes from the
  whole lot to one hinge.
- **Drag**: the ground moves with your hand. A flick continues and slows down.
- **Right drag**, or **Shift and drag**, or the **arrow keys**: tilt and turn.
- **Double-click** a thing: the view flies to it. **F** frames the selected thing.
- **North arrow**: one click turns the view north-up. A double-click looks straight
  down.
- **Page Up** and **Page Down** go from level to level. **T** goes between plan and 3D.
- **Tilt as I dive** is a switch on the view bar.

No view jumps. Each move starts slowly and stops slowly.

[picture: view-dive | A strip of pictures across one dive of the wheel, from the lot to a fixture.]

Proven by `demos/navw.sh`. The script reads the path of the view frame by frame.

## 4. Bring in a plan and set its scale

1. On the Job panel, select **Add drawing**. Pick the file from the list. Pick the page.
2. Select **Set scale**. Click the two ends of one dimension on the drawing. Type the
   length that the dimension shows.
3. To put a second drawing in line with the first, select **Line up**.

The program takes a PDF (drawn or scanned), a PNG, a JPEG and a scan (PLY) from the
window. Where a PDF has drawn lines, the pointer catches them.

[picture: plan-scale | A plan under the tracing, with its scale set from one dimension.]

Proven by `demos/planw.sh` and `demos/firstw.sh` (a plan in the public domain).

## 5. Trace the house

1. Select **Add level**. Give the height to the next level.
2. Select the **Wall** tool. The tool shows the sizes that it will make. Accept them or
   type other sizes. Click each corner. To make a wall of an exact length, type the
   length and press Enter.
3. Select **Door** or **Window**. Click on a wall.
4. Select **Floor**, then **Room**. Click in a closed room and give it a name.
5. Select **Roof**. Click the corners of the footprint. Set the pitch.
6. Select **3D** on the view bar. The house stands on the tracing.

Each traced value is **Assumed** and says where it came from. **Undo** takes back one
step.

[picture: trace | Walls traced over a plan, in Plan view.]
[picture: house-3d | The 3D house made from the tracing.]

Proven by `demos/draww.sh`, `demos/planw.sh` and `demos/firstw.sh`.

## 6. Enter a field measurement

1. Select the thing. Its values are in the Details panel.
2. Select **Measure** beside a value. Type the measured length, or shoot it with a
   laser in keyboard mode. The laser types the number at the cursor.
3. The value is now **Known**. Its source is your measurement.

A clash that depended on the value confirms or clears. The price in the estimate
window changes at the same time.

To take the list to the site on paper, write out the field sheet. Fill it in. Bring it
back as a file (section 12).

[picture: field | A measured height typed for a level. The value becomes Known.]

Proven by `demos/fieldw.sh` and `demos/field-sheet.sh`.

## 7. Clashes and the verify-in-field list

- A clash between **Assumed** things is a **question for the field**.
- A clash between **Known** things is a **finding**.
- In the estimate window, the tab **Clashes** lists them. The tab **Verify in field**
  lists each question, where it is, and the dollars that depend on it. The largest
  dollars are first.
- Pick up a thing and move it (section 9). Its clashes light and clear while it moves.

[picture: clash | Clashes marked on the house, and the list of them in the estimate window.]

Proven by `demos/clashw.sh`, `demos/tradesw.sh`, `demos/clash.sh` and `demos/vif.sh`.

## 8. The estimate window

- The tab **Estimate** shows the recap and the 16 divisions. Other tabs show one
  division, the budget, the clashes and the field list.
- Each line shows its quantity, its formula and its source. You can change the
  stretch-out on the line.
- You can switch labor on or off for each line.
- **Overhead** and **Profit** are fields. Type a new percent.
- **Price list** (on the Job panel) brings in a supplier's list. The window shows
  three suppliers side by side.
- A price per SF or LF grows into line items for the same condition. The detail rolls
  up again to the same line.

[picture: estimate | The estimate window: the recap and the divisions.]

Proven by `demos/moneyw.sh`, `demos/pricew.sh`, `demos/d4w.sh` and `demos/d5w.sh`.

## 9. Items: place, move, and the bench

1. On the Job panel, select **Starter tool chest**. The items of each trade come in.
   An item from a maker's sheet comes in with **Maker's sheet**.
2. Select **Fixture** or **Equipment**. Pick the item. Click where it goes.
3. Select **Move**. Pick up the thing and put it down in its new place. A wall that
   you move takes the walls that meet it.
4. Open an item on **the bench** to see it alone: its sizes, its shape, where it
   connects, and how it is priced.

Each size of an item from a sheet is Assumed and names the line of the sheet. A size
that the sheet does not give is Unknown.

[picture: items | An item of the tool chest: each size is Assumed and says where it came from.]
[picture: move | A wall moved. The message says what followed it.]
[picture: bench | An item on the bench.]

Proven by `demos/itemlibw.sh`, `demos/itemneww.sh`, `demos/movew.sh`, `demos/benchw.sh`
and `demos/shapebenchw.sh`.

## 10. Kitchens, renovation and layouts

- **Cabinets**: select the Cabinets tool. Click a wall. Type the codes of the boxes
  (for example `B15 B30 SB36`). The run stands along the wall with its countertop.
- **Renovation**: select a wall. Mark it **Existing** or **To demolish**. The lists
  show what to build, what stays and what comes down.
- **Room line**: divides an open plan into two rooms with no wall.
- **Layouts**: keep the job as another layout, change it, and set the two side by side
  with their prices.

[picture: cabinets | A run of cabinet boxes along a kitchen wall.]
[picture: renovation | A renovation: existing walls pale, a wall to demolish marked.]

Proven by `demos/cabinetsw.sh`, `demos/renow.sh`, `demos/roomlinew.sh` and
`demos/layoutsw.sh`.

## 11. See each trade

The **Show** part of the Job panel has one switch for each layer: Foundation, Framing,
Outside, Plumbing, HVAC, Electrical, Inside, Ground, Finishes, Roof, See through,
Sizes. Each switch reads on or off at a glance. A trade is drawn only where the
estimate prices it.

**Sun** on the view bar sets the month and the hour.

[picture: trades-framing | The frame of the walls, with the finishes switched off.]
[picture: trades-hvac | The ducts, with the house see-through.]

Proven by `demos/tradesw.sh`, `demos/dressw.sh`, `demos/trimw.sh` and `demos/lightw.sh`.

## 12. Bring in and write out

The window brings in these files, each picked from a list:

| File | Button on the Job panel |
|---|---|
| Plan: PDF, PNG, JPEG | Add drawing |
| Scan: PLY | Scan file |
| Another trade's or a designer's model: IFC | Designer's model |
| Supplier's price list: CSV, XLSX | Price list |
| Assemblies and a prior estimate: CSV, XLSX | Assemblies, Prior estimate |
| A maker's sheet: PDF | Maker's sheet |
| Items by trade: CSV | Tool chest file |

The window writes out the BIM model (IFC), the estimate as a workbook with live
formulas, the estimate as CSV, and one sheet each for suppliers, subcontractors and
the project manager.

The command line does more today. The window does not have these yet:

| Task | Command |
|---|---|
| A CAD drawing (DXF) or a TIFF as a plan | `plumbline drawing <job> <level> "<name>" --file <file>` |
| The plan of a level as a CAD drawing (DXF) | `plumbline export dxf <job> <out.dxf> --level <level>` |
| A scanner's own file (LAS, LAZ, E57) | `plumbline import scan <job> <file> --level <level>` |
| The measured marks on a PDF as quantities | `plumbline import marks <job> <marked-up PDF>` |
| All tables of the job in one workbook, out and back | `plumbline export database`, `plumbline import database` |
| The field sheet, out and back | `plumbline export field-sheet`, `plumbline import field-sheet` |
| The clashes as an issue list (BCF) | `plumbline export bcf <job> <out.bcfzip>` |
| The house as one 3D file (glTF) | `plumbline export gltf <job> <out.glb>` |

`plumbline` with no words after it lists each command. When a file loses something on
the way in or out, the program says what it lost.

[picture: bring-in | A designer's model brought in from the window. The program says what it read.]

Proven by `demos/bringw.sh`, `demos/scanw.sh` and the command-line demonstrations
`imports`, `export`, `dxf`, `scan-files`, `marks`, `database`, `field-sheet`, `bcf` and
`gltf`.

## 13. Two plans side by side

At the top of the estimate window, select **Plan**. The window shows a plan in place
of the estimate. Select **Two plans** to show two: two levels, or a drawing beside its
tracing. Each plan moves by the same wheel and drag as the model window. A thing that
you select in one window is marked in the other.

[picture: two-plans | The second window with the first floor beside the second floor.]

Proven by `demos/twoplansw.sh`.
