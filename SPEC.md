# SPEC: the world data model

What D1 built, as amended by the owner's ruling on units (DECISIONS.md call 16). Later
deliverables add components and views; they do not change these rules.

## 1. The job is one world

A job is a single entity-component world (`bevy_ecs`). Every building element is an
entity. An element is whatever an estimator would point at and name: a roof plane, an
eave, a hip, a parapet. What an element *is* comes from the components it carries. A hip
has a run and names the planes it sits between. A parapet has a run and a height. Either
can carry a condition, a cost code with lines, and a schedule code.

Nothing is stored twice. A quantity is not typed into the sheet and again into the model;
it is worked out from the geometry on the element and read from there by every view.

## 2. Every value says how sure it is and where it came from

| Status | Meaning | Carries a value | Flag |
|---|---|---|---|
| Known | Read off a plan, measured, or typed from a quote | yes | none |
| Assumed | A working figure chosen because the real one is not in hand | yes | `~` |
| Unknown | Nobody has it | **no** | `?` |

Rules. The same checks (`check.rs`) run when a job is loaded and when a value is set.

1. Every entered value has a non-empty `source`.
2. Unknown carries no value. Known and Assumed carry one.
3. Every entered value carries its unit, and keeps the unit it was entered in (section
   2.1). It is converted only inside arithmetic. A unit this program does not know is
   refused by name; so is a length with no unit. Neither is ever read as feet.
4. A derived value takes the **worst** status among its inputs. An Assumed pitch makes an
   Assumed area. An Unknown height leaves the wall area with no number at all.
5. A derived value shows its `formula` and lists its `inputs` (`path#component`).
6. **A quantity never disappears.** Every element of a geometric kind lists every one of
   its quantities. If an input is missing, the quantity is there as Unknown with the
   reason ("no pitch entered").
7. When inputs do not fit together, the derived value is Unknown and its `note` gives the
   figures. See 5.3.
8. Arithmetic that overflows gives Unknown, not infinity.

**What counts as a value.** Dimensions, pitches, heights, stretch-outs, outlines, line
ends, assigned conditions, cost codes, unit costs and schedule codes carry a status and a
source.
References and labels do not: the planes a line names, an element's name, a cost line's
description and the measure it is priced by. Those are checked for sense instead.

### 2.1 Units

US measurements, entered the way the drawing or the tape gives them. An estimator converts
nothing; the program converts only inside arithmetic, and says where it did.

| Unit in the file | What it is | `value` in the file | Typed as | Shown as |
|---|---|---|---|---|
| `ft` | decimal feet | a number | `25.5 ft`, `25.5'`, `25.5 feet` | `25.5 ft` |
| `in` | inches | a number | `306 in`, `306"`, `19 1/2 in`, `19-1/2"` | `306 in`, `19 1/2 in` |
| `ft-in` | feet and inches | `[feet, inches]` | `25' 6 1/2"`, `25'-6 1/2"`, `25 ft 6 in`, `25' 6` | `25' 6 1/2"` |
| `in/12` | pitch: inches of rise for 12 of run | a number | `6`, `6:12`, `6/12`, `6 in/12`, `1/4:12`, `1/4 in/ft` | `6 in/12` |

* **A single dimension** (a parapet height, an eave height, a stretch-out) takes `ft`, `in`
  or `ft-in`. **Plan coordinates** (outline points, the eave line, line ends) are plain
  numbers and take `ft` or `in`. **A pitch** takes `in/12` only.
* **Held as entered.** `30 in` is saved as the number 30 with the unit `in`, with its
  status and source, and shown as `30 in`. Nothing rewrites it as 2.5 ft. Changing the
  unit a value is written in changes that value in the file and the figures derived from
  it (their formula, and their number if the length itself changed), and nothing else.
* **Exact to the sixty-fourth.** Fractions are halves down to sixty-fourths, which a
  64-bit number holds exactly. `25' 6 1/2"` is `[25, 6.5]`, and it is the same number of
  feet, to the last bit, as `306.5 in`. Any other fraction (`1/3`) is refused; write it as
  a decimal.
* **Feet and inches.** The feet are a whole number, the inches are less than 12, and a
  length below zero carries the minus on both. `25' 14"` and `25.5' 6"` are refused.
  In `25'-6"` the dash is a separator, not a minus.
* **Converted only for arithmetic.** Every tolerance and limit in this document is in
  feet and is applied after conversion. A derived value's `formula` shows the conversion:
  `plan_length × height ÷ 12` for a height in inches, `plan_length × height` for one in
  feet (section 5).
* **Refused by name, never read as feet:** a unit this program does not know (`cubits`,
  `yd`, `FT`); a length with no unit, typed (`30`) or in the file; metric, which is called
  metric in the refusal (`mm`, `cm`, `m` and their spellings; PROPOSALS.md item 5); a
  pitch in degrees or percent, or on a run other than 12 (PROPOSALS.md item 6); a number
  written with an exponent or a thousands comma at the keyboard.
* A pitch typed as a bare number is rise in 12. A pitch has one unit, so there is nothing
  else it could be read as.
* Marks and fractions pasted from a drawing are read: ′ ″ ’ ” and ½ ¼ ¾ ⅛ ⅜ ⅝ ⅞.

## 3. The saved job

The saved job is the world written out as JSON in IFC5's entity-and-component layout.
IFC5 is the pattern only; no IFC5 schema is imported and the file is not claimed to be
valid IFCX.

```
{
  "header":  { "id", "ifcxVersion": "ifcx_alpha", "dataVersion", "author", "timestamp" },
  "imports": [],
  "schemas": { "pl::pitch": { "value": { "dataType": "Object" } }, ... },
  "data": [
    { "path": "plane-0017",
      "children":   { "<name>": "<path>" },
      "attributes": { "pl::class": "RoofPlane", "pl::pitch": { ... }, ... } }
  ]
}
```

* **One entry per element.** A path appears once. A file with a repeated path is refused.
* **Paths are permanent.** A new element's path is its kind and the job's next number
  (`plane-0017`). Renaming an element changes its name under its parent, never its path.
* **The tree** is carried by `children`: each parent lists its children by name. The first
  entry is the job. It alone has no parent, it alone is of kind Job and carries `pl::job`,
  and every other element leads back up to it. A loop of children is refused.
* **Entries are in the order elements were added.**
* **Saving is repeatable.** The same world writes the same bytes. Nothing is stamped at
  save time; `timestamp` is when the job was started. So a diff between two saves shows
  what was edited and what followed from it, and nothing else.
* **Readable.** One key per line; a point or a row of points stays on one line.
* **Numbers come back as the same numbers.** A coordinate read from a file is written back
  exactly. (A file written by another tool may still differ in spacing or in how a number
  is spelled, `40` against `40.0`; `plumbline check` reports that as a difference.)
* **Numbers are 64-bit.** Each entered length is in the unit it carries (section 2.1).
  Derived lengths and areas are in LF, SF and SQ.

### Nothing is dropped quietly

| Where an unknown key is | What happens |
|---|---|
| Top level of the file, the header, a `schemas` entry, a data entry, IFC5's `inherits` | Kept and written back unchanged |
| An attribute whose key this program does not know | Kept and written back unchanged |
| A field inside one of this program's own components (`pl::pitch`, a cost line, ...) | **Refused**, naming the field |
| The same key twice in one object, anywhere in the file | **Refused**, naming the key |

The third row is deliberate. A cost line with an extra `extended_total` is a price with no
status and no source; it is refused, not carried.

### Loading checks the file

A file is refused if:

* *Values:* a value breaks the rules in section 2; a condition or schedule code is blank
  text; a number is not finite.
* *Units:* a length in a unit this program does not know, in metric, or with no unit; a
  number that does not suit its unit (a `[feet, inches]` pair marked `ft`, a single number
  marked `ft-in`, feet that are not whole, inches of 12 or more, feet and inches of
  opposite signs); plan coordinates in `ft-in`; a pitch not in `in/12`; a unit cost not in
  `$/` and the unit its line comes out in; a job whose `length_unit` is not `ft`.
* *Kinds:* a component sits on a kind that cannot carry it (an outline on a hip, a height
  on anything but a parapet).
* *Outlines:* fewer than three points; a repeated point; crossing or touching edges; no
  area; a coordinate beyond 1,000,000 ft.
* *Eaves:* an eave line with no length; one that does not lie along one edge of its
  outline, or is too short to tell which edge; one with part of the outline on its low
  side (the eave is the plane's lowest edge); an eave height beyond 1,000,000 ft.
* *Pitches and heights:* a pitch negative or above 240 in 12; a parapet height negative or
  beyond 1,000,000 ft.
* *Lines:* two ends at the same point; a blank plane path; a parapet that names a plane.
* *Cost:* a code lacking a scheme or a code; a line with no description; two lines on one
  element with the same description; a line priced by a measure its element does not
  have, or by a factor (`slope_factor`) and not a quantity; a stretch-out on a line not
  priced by a length, or of zero or less, or beyond 1,000,000 ft; a unit cost in the wrong
  unit, negative, or beyond 1e9.
* *Structure:* the same key twice in an object; an unknown field inside one of this
  program's components; no elements; a blank or repeated path; a blank child name; a
  child that points at nothing; an element with two parents; a tree that does not lead
  back to one job, first in the file; a `next_id` not higher than every path number in use.

An eave height may be negative: it is a level relative to the job's datum.

Measures and line quantities in the file are **never trusted**. They are recomputed from
the geometry on every load, the recomputed figures are the ones used, and any element
whose stored figures do not match is reported by `show`, `check` and `set`.

## 4. Components

| Key in the file | On | Holds | Unit |
|---|---|---|---|
| `pl::class` | every element | The kind: Job, Roof, RoofPlane, LowSlopeArea, Eave, Rake, Ridge, Hip, Valley, Headwall, Parapet | |
| `pl::job` | the job | Title, the unit derived lengths are worked out in (`ft`), the next path number | |
| `pl::outline` | areas | `points`: the footprint traced on the plan, either way round. `eave`: the low edge the pitch rises from, as its own line along one edge of the outline, with no part of the outline on its other side. The slope is measured from that edge's line. `eave_height`. | `points`, `eave`: `ft` or `in`. `eave_height`: `ft`, `in` or `ft-in` |
| `pl::pitch` | areas | Rise in inches per 12 inches of run | `in/12` |
| `pl::run` | lines | `ends`: the two ends on the plan. `planes`: paths of the planes the line lies on. | `ft` or `in` |
| `pl::height` | parapets | Height | `ft`, `in` or `ft-in` |
| `pl::condition` | any | The assigned condition, by name | none |
| `pl::cost` | any | `code` (scheme, code, title) and `lines`. Each line has a `description` (no two alike on one element), the `quantity` (a measure on this element) it is priced by, an optional `stretch_out`, and a `unit_cost`. | `stretch_out`: `ft`, `in` or `ft-in`. `unit_cost`: `$/` + the unit the line comes out in |
| `pl::schedule` | any | `wbs`: the work breakdown code | none |
| `pl::measure` | any with geometry | Derived quantities by name. Never typed. | |
| `pl::line_quantities` | any with a sheet-metal line | The derived quantity of each cost line that has a stretch-out, by the line's description. Never typed. | SF |

**Stretch-out.** A sheet-metal line carries the width of the flat sheet its profile is
bent from, as the detail gives it. It belongs to the line, not to the material: the same
copper is 12 in at an eave and 36 in in a valley. It goes on a line priced by a length
(LF), and the line then comes out in SF and is priced in `$/SF`. Sheets, pounds and waste
are D3.

How many planes a line names: a hip or valley, two; an eave, rake or headwall, one; a
ridge, one or two; a parapet, none.

An entered value is written `{ "value", "unit", "status", "source" }`.
A derived value is written `{ "value", "unit", "status", "formula", "inputs", "note" }`.

## 5. Derived quantities

Derived figures are written to four decimal places.

### 5.1 Areas (RoofPlane, LowSlopeArea)

| Measure | Formula | Inputs |
|---|---|---|
| `plan_area` (SF) | area of the outline on the plan (÷ 144 if it was entered in inches) | outline |
| `slope_factor` | √(1 + (pitch ÷ 12)²) | pitch |
| `surface_area` (SF) | plan_area × slope_factor | outline, pitch |
| `squares` (SQ) | surface_area ÷ 100 | outline, pitch |

### 5.2 Lines

| Measure | On | Formula | Inputs |
|---|---|---|---|
| `plan_length` (LF) | every line | length of the line on the plan (÷ 12 if it was entered in inches) | run |
| `true_length` (LF) | eave, rake, ridge, hip, valley, headwall | √(plan_length² + rise²) | run, and the outline and pitch of each plane named |
| `face_area` (SF) | parapet | plan_length × height, the height in feet: `height` as entered in `ft`, `height ÷ 12` in `in`, `height (feet + inches ÷ 12)` in `ft-in` | run, height |

The formula saved beside each figure is the one that was used, conversion included, so
the figure can be checked by hand from what the file shows. A `true_length` that leans on
anything entered in inches ends "lengths not entered in feet are converted to feet first
(inches ÷ 12)".

The height of a plane above a plan point is its eave height plus pitch ÷ 12 × the point's
distance from the plane's eave line (the line of the outline edge the eave lies along),
measured square to it. A line's rise is the
difference between the heights of its two ends. Its status is the worst among its own
ends and every entered value on each plane it names (outline, eave line, eave height,
pitch).

### 5.3 A line has to fit its planes

`true_length` is Unknown, with a note saying why, when:

* the line names the wrong number of planes for its kind, the same plane twice, a plane
  that is not in the job, or an element that is not a plane;
* a named plane lacks its outline or its pitch;
* the line does not run along the outline of a plane it names: its ends and three points
  between are each tested, and any more than 0.01 ft off the outline fails it;
* the planes put either end more than 0.01 ft (about 1/8 in) apart in height;
* the line's kind and its slope disagree: an eave, ridge or headwall that is not level on
  its plane, or a rake, hip or valley that is level on a plane with enough pitch to lift
  it more than 0.1 ft over its length.

Example: steepen one plane of a hip roof while the hips stay where they were traced, and
those hips and the ridge are flagged with both heights. The roof as drawn no longer closes.

### 5.4 Line quantities

| Quantity | On | Formula | Inputs |
|---|---|---|---|
| sheet area (SF) | a cost line with a `stretch_out`, priced by a length | *length* × stretch_out, the stretch-out in feet: `true_length × stretch_out ÷ 12` when it was entered in inches | the element's measure, the element's cost lines |

The length is the measure as the file shows it (four decimals), so 12 LF × 24 in ÷ 12 =
24 SF can be checked from the page. Its status is the worst of the length's and the
stretch-out's: an Unknown stretch-out leaves no area, not a zero one.

## 6. Cost code scheme

Which scheme is primary is an open question (QUESTIONS.md Q1). Every code stores its
`scheme` beside it. No total is grouped by code in D1.

## 7. What D1 does not do

* **It does not re-solve the roof.** Ridge, hip and valley positions are entered, not
  worked out from the footprint and pitches. D2 does that with the skeleton (DECISIONS.md
  call 3); until then a pitch change the linework cannot follow is flagged per 5.3.
* **No window.** The world is read and edited through the command line. The same rows the
  command line prints (`view::elements`) are what the sheet and the 3D view will read.
* **No money.** Cost lines exist and carry unit costs with statuses; nothing is extended
  or totalled. Tying a line's status to its quantity's status is D5.
* **Sheet metal stops at area.** A stretch-out turns a line's length into SF. Sheets,
  pounds, waste, and the conditions one parapet produces are D3.
* **Metric, and pitch in degrees or percent, are refused**, not converted (PROPOSALS.md
  items 5 and 6).
* **Adding or removing a cost line, and re-entering outlines and line ends, are done in
  the file.** `set` edits pitch, height, eave height, a line's stretch-out, condition and
  schedule code.
* **No undo, no conditions library, no derived conditions** (D3).
* **Not checked:** outlines that overlap each other or leave gaps; a plane whose eave is
  entered on the edge parallel to the true one (every line still fits and every length is
  right, but its heights run downhill); a line that leaves the outline and returns between
  the five points tested; whether a cost code is a real code in its scheme.
* **A plane with an eave that steps** (around a bay) takes its lowest edge as the eave;
  the stepped-back eave is entered as its own Eave line and comes out level and higher.
* **The 0.01 ft fit tolerance** is tighter than a hand trace of a scanned plan can hold
  (QUESTIONS.md Q9).
* **Paths are counter-based**, so two copies of a job edited apart can mint the same path
  (PROPOSALS.md item 4).
* **`Job::put` is unchecked.** Code that builds a job directly (the sample does) bypasses
  the entry checks; a job built that way is checked when it is next loaded.

## 8. Code layout

| Crate | Depends on | Holds |
|---|---|---|
| `crates/world` (`plumbline-world`) | `bevy_ecs`, `serde`, `serde_json` | The model: values, units, components, checks, plan geometry, derivation, the saved file, the sample job, the flat view |
| `crates/cli` (`plumbline`) | `plumbline-world` | `sample`, `show`, `set`, `check` |

The world crate does not depend on any renderer or window, and must not.
