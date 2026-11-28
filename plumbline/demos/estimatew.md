# The estimate at three depths: a demonstration, written before it is built

Call 243. The estimate window shows the sixteen divisions, their items, and an
item's places as one table that opens, where today it writes about 520 rows with
every line twice. It is `demos/estimatew.sh`, by `tools/window-run.sh`.

## The job

The sample whole house (`plumbline sixteen`): 432 lines, 56 items, sixteen
divisions, job cost $393,981.66, overhead at 10 % $39,398.17, profit at 8 %
$34,670.39, price $468,050.22. The estimate window at 880 by 960.

## What is checked, with the figures

1. **Four tabs on one line**, none cut, at the largest lettering: Estimate, Verify
   in field (85), Clashes (11), Lists. Lists opened has six rows: Building, Roof,
   Mechanical, Electrical, Plumbing, Layouts. `sheet "Building"` with Lists closed
   still reaches Building (the way on the map), and what it shows has the rows it
   has today (the script counts them before and after from a build of the commit
   before).
2. **The head**: the price is written once in the estimate window, $468,050.22;
   "85 to verify" pressed shows Verify in field.
3. **At rest**: sixteen division rows, their figures as below, adding to
   $393,981.66; 56 item rows; no row of a place. The division rows and the items
   under each:

   | Division | Figure | Items | Lines |
   |---|---|---|---|
   | 1 General Requirements | $48,050.00 | 4 | 4 |
   | 2 Site Construction | $18,372.00 | 3 | 3 |
   | 3 Concrete | $32,485.40 | 3 | 9 |
   | 4 Masonry | $12,635.00 | 1 | 4 |
   | 5 Metals | $504.00 | 1 | 1 |
   | 6 Wood and Plastics | $54,795.62 | 11 | 74 |
   | 7 Thermal and Moisture Protection | $51,488.19 | 8 | 54 |
   | 8 Doors and Windows | $28,735.00 | 4 | 35 |
   | 9 Finishes | $48,955.12 | 7 | 52 |
   | 10 Specialties | $1,550.00 | 2 | 3 |
   | 11 Equipment | $10,900.00 | 2 | 2 |
   | 12 Furnishings | $6,460.00 | 1 | 17 |
   | 13 Special Construction | $9,200.00 | 1 | 1 |
   | 14 Conveying Systems | $38,000.00 | 1 | 1 |
   | 15 Mechanical | $27,182.24 | 4 | 148 |
   | 16 Electrical | $4,669.09 | 3 | 24 |
   | | $393,981.66 | 56 | 432 |

4. **A division closed and opened**: Division 6 pressed, 45 item rows; pressed
   again, 56. "Close all": 16 rows and no item; "Open all": 16, 56 and 432.
5. **An item opened to its places**: "Concrete footing, in place" (Division 3; 4
   places, 10.0742 CY, $450.00/CY, $4,533.40) opens to four lines: 2.9630 CY
   $1,333.35; 2.0741 CY $933.35; 2.9630 CY $1,333.35; 2.0741 CY $933.35. They add to
   the item's row: 10.0742 CY and $4,533.40. "Wall framing, 2x6 at 16 in" opens to 8
   lines adding to 2,584.00 SF and $10,852.80.
6. **A price typed on a place moves every depth above it, in both windows.** Footing
   1's unit cost typed as 500: its line 2.9630 x 500 = $1,481.50; the item $4,533.40
   - 1,333.35 + 1,481.50 = $4,681.55, and its unit cost says the prices vary;
   Division 3 $32,485.40 + 148.15 = $32,633.55; job cost $394,129.81; overhead
   $39,412.98; profit $34,683.42; price $468,226.21, in the estimate window's head
   and in the model window. Undo: every figure as it was. The saved job after Undo
   is byte for byte the job before.
7. **An item in one place is priced on its own row**: "Steel ridge beam" (Division
   5; 12.00 LF at $42.00/LF, $504.00) typed as 50: $600.00; Division 5 $600.00; job
   cost $394,077.66. Undo.
8. **Budget is the other side of the switch**: "Budget" pressed from Estimate shows
   what the Budget tab shows today (the script counts its rows against the build of
   the commit before); "Estimate" comes back with what was open still open.
9. **Not priced is said once.** A new job with one wall drawn and nothing priced: in
   the estimate window no block has more than two red words; a division with lines
   not priced says how many on its own row; the cells are empty with the mark.
10. **Nothing cut** at 880 by 960 and at 640 by 720, at each step of lettering: no
    control or figure whose box runs past its row or the window. Four sizes of
    lettering and no more.
11. **Every demonstration that types on the estimate's lines still passes
    unchanged**: `d4w`, `d5w`, `pricew`, `moneyw`, `fieldw`, `carryw`, `layoutsw`,
    `fixes197w`, `fixes209w`, `pagew`, `twoplansw`.

## Proved red first

Run on the window as it is before the work: line 1 gives ten tabs on two lines; line
3 gives 56 item rows and 432 rows of places at rest; line 4 has nothing to press.
