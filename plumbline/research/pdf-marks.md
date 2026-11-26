# How a PDF holds a measured mark (call 227)

**Not confirmed against any file a marking program wrote:** the keys this program reads for a mark's label (`Subj`) and
figure (`Contents`), and the convention by which a count is read (stamps counted by label), have not been checked against a
file written by any marking program; no such file is in the repository. Only the standard's own keys below are sourced.
A count is therefore this program's count of a page's symbols, not a figure the file states, and is brought into an estimate
only when asked for (`import marks --counts`, call 231).

Read for `crates/plans/src/marks.rs`. **No file written by a marking program was fetched, kept or read**: the program's
tests read files the tests write, to the standard (`demos/marks/make.py`). Where this page says "could not be confirmed",
the key is not relied on for a figure.

## Sources reached, and what each gave
- The PDF standard's own text, ISO 32000 (the 2.0 text, as published in an English rendering at
  `hellowac.github.io/pdf-std-zh-cn/iso32000/c12/s5` and `.../c12/s9`): the tables below. The renderings are of the
  standard's text; I did not have the ISO document itself, so table numbers are those of that rendering (32000-2 numbering).
- The marking program maker's own help pages (`support.bluebeam.com`, "Markups List" and "Add captions to markups"): they
  describe what a person sees (columns Subject, Label, Length, Area, Volume, Count, Measurement; "Revu can display multiple
  values within one measurement markup"; custom captions) and say **nothing of the keys the program writes**.
- Third-party library pages (iText, Apryse, Aspose, ABCpdf) for the names of the intent values only; not relied on.

## Each kind, each key read
| a mark | the standard's note | keys read | address | the line quoted |
|---|---|---|---|---|
| a length | Line | `L` (two points), `Measure` | 12.5.6.7, Table 175 | `L`: "specifying the starting and ending coordinates of the line in default user space"; `Measure`: "A measure dictionary (see Table 261) that shall specify the scale and units that apply to the line annotation." |
| a run of lengths | PolyLine | `Vertices`, `Measure` | Table 178 | `Vertices`: "the alternating horizontal and vertical coordinates, respectively, of each vertex, in default user space"; `Measure` as above |
| an area, or a perimeter | Polygon (closed) | `Vertices`, `Measure` | Table 178 | as PolyLine; a polygon that says a length is a perimeter and is read as a run of lengths |
| any | the common keys | `Subtype`, `Rect`, `Contents`, `NM`, `F` | Table 164 | `Contents`: "Text that shall be displayed for the annotation..."; `NM`: "a text string uniquely identifying it among all the annotations on its page"; `F`: "A set of flags specifying various characteristics of the annotation" (hidden and no-view marks are not listed) |
| any marking | the markup keys | `Subj` | Table 170 | "Text representing a short description of the subject being addressed by the annotation." |
| the scale | Measure (rectilinear, `Subtype` RL) | `R`, `X`, `Y`, `D`, `A` | 12.9, Tables 261 to 263 | `R`: "A text string expressing the scale ratio of the drawing"; `X`: "A number format array for measurement of change along the x axis"; `D`: "...for measurement of distance in any direction"; `A`: "...for measurement of area"; each entry `U` (unit label) and `C` ("the conversion factor") |
| the arithmetic | 12.9 | | | "The first element in the array shall specify the conversion to the largest distance unit" (of D, from the x units); "Multiply the value specified previously by the C entry of the first number format dictionary in the array"; area: "the conversion squared" |

How the figure is worked here: length = length of the points in user space x `X[0].C` x `D[0].C`, then to feet by `D[0].U`
(ft, in); area = shoelace area x `X[0].C` x `Y[0].C` (or X) x `A[0].C`, then to SF by `A[0].U` (sq ft, sq in). Only the first
element of each array is used. A `Y` that differs from `X`, a scale that is not RL, a unit not ft/in (sq ft/sq in) is said and
the points are not worked (the mark's own text is still taken).

**The two figures.** The standard's measure dictionary holds a scale, not the figure: the figure a person saw is in the
mark's own text (`Contents`) or its drawn appearance. This program takes the text. Where `Contents` has no figure with a unit
the mark is "not read: it says no figure".

## What could not be confirmed from a source (not relied on, or relied on and flagged here)
1. **What the best-known marking program writes**: its `Subj`, `Contents`, `Label` and measurement text, and how a label is
   stored. Its help pages do not say. **Relied on as a convention, unconfirmed**: label = `Subj` (else the words round the figure in
   `Contents`, else the kind of mark); figure = `Contents`. `T` is **not** used as the label: in the standard it names who made
   the note (the title bar of its pop-up).
2. **A count**: the standard has no mark for one. **Convention, unconfirmed**: a Stamp (a symbol set on the sheet), labelled by
   `Subj` (else `Contents`), counted by label, one count at the first of them. A stamp that is no count of anything (an
   "approved" stamp) would be read as one; it comes in as an Assumed EA line naming itself, and the estimator clears it.
3. **A curved edge**: Circle and Ink (Table 177 gives Circle) with a figure or a measure are "not read". A `Path` key (curved
   polygons) was not found in the text read and is used only to refuse.
4. **A scale for part of a sheet**: a viewport's `Measure` (12.9) applies to the contents within its `BBox`; a mark with no
   measure of its own on a page that has viewports is "not read: its scale is set for a part of the sheet". Reading it is owed.
5. Intent names (`PolygonDimension`, `PolyLineDimension`, `PolygonCloud`) came from library pages only; not read.
6. `UserUnit` (a page whose point is not 1/72 in) is not read; a Measure on a Square is not in the table read and is not used.
7. The ISO document itself, and the exact area step (whether the area array's `C` also takes the squared x factor), were not
   confirmed beyond the quoted sentences.
