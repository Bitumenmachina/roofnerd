# SPEC: the world data model

What D1 built, as amended by the owner's ruling on units (DECISIONS.md call 16), and
what D3 added to the cost line (sections 4 and 5.4). Later deliverables add components
and views; they do not change these rules.

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
8. Arithmetic that overflows gives Unknown, not infinity. A figure that is not zero but
   would be written as 0.0000 is Unknown with the reason, not shown as zero.

**What counts as a value.** Dimensions, pitches, heights, stretch-outs, the other values
a line's formula names (a spacing, a number of sides), waste, order factors, outlines,
line ends, assigned conditions, cost codes, unit costs and schedule codes carry a status
and a source. A number written into a formula (`* 2`) is part of the formula, which is on
the line to be read.
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
  `yd`); a length with no unit, typed (`30`) or in the file; metric, which is called
  metric in the refusal (`mm`, `cm`, `m` and their spellings; PROPOSALS.md item 5); a
  pitch in degrees or percent, or on a run other than 12 (PROPOSALS.md item 6); a number
  written with an exponent or a thousands comma at the keyboard.
* **In the file a unit is spelled exactly** `ft`, `in`, `ft-in` or `in/12`; `FT` or
  `" ft"` is refused. At the keyboard capitals do not matter: `25 FT` is 25 ft.
* **A bare fraction is not a pitch.** `3/4` is a rise of 3/4 in 12 to one estimator and
  3 in 4 to another, so it is refused; `3/4:12` is read. Only `a/12` is read without more,
  as a in 12.
* **A minus goes in front of the whole length** (`-2' 6"`). `0' -6"` is refused, not read
  as 6 in. The dash in `25'-6"` or `25' - 6"` is a separator.
* A number too small for a 64-bit number to hold (it would be stored as zero) is refused.
* A pitch typed as a bare number is rise in 12. A pitch has one unit, so there is nothing
  else it could be read as.
* Marks and fractions pasted from a drawing are read: ′ ″ ’ ” and ½ ¼ ¾ ⅛ ⅜ ⅝ ⅞. Two
  apostrophes are an inch mark (`6''`). A pitch is also read as `1/4"/ft`, `1/4":12` and
  `6 in./12`.

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
| An attribute that is a mistyping of one of this program's: under its prefix however the prefix and colons are spelled (`pl::Pitch`, `PL::pitch`, `pl;;pitch`, ` pl::pitch`, `pl::pich`), or with a prefix one letter off and one of its names exactly (`pI::pitch`). Another program's key that merely resembles one (`ex::cost`) is kept | **Refused**, naming the key and the one it resembles |
| A field inside one of this program's own components (`pl::pitch`, a cost line, ...) | **Refused**, naming the field |
| The same key twice in one object, anywhere in the file | **Refused**, naming the key |

The fourth row is deliberate. A cost line with an extra `extended_total` is a price with no
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
  on anything but a parapet, a recipe on anything but a Condition); a Condition with no
  recipe.
* *Outlines:* fewer than three points; a repeated point; crossing or touching edges; no
  area; a coordinate beyond 1,000,000 ft.
* *Eaves:* an eave line with no length; one that does not lie along one edge of its
  outline, or is too short to tell which edge; one with part of the outline on its low
  side (the eave is the plane's lowest edge); an eave height beyond 1,000,000 ft.
* *Pitches and heights:* a pitch negative or above 240 in 12; a parapet height negative or
  beyond 1,000,000 ft.
* *Lines:* two ends at the same point; a blank plane path; a parapet that names a plane.
* *Cost:* a code lacking a scheme or a code; a line with no description, or one that
  starts or ends with a space or holds a character nobody can see (a zero-width space, a
  tab); two lines on one element whose descriptions differ only in
  capitals or spacing; a quantity formula that cannot be read, names something the
  element does not have and the line does not carry, adds a length to an area, comes out
  as a volume or as one over a length, divides by a zero, or is a factor (`slope_factor`)
  standing alone; a stretch-out of zero or less, or beyond 1,000,000 ft; a stretch-out or
  any other value on the line that the formula does not use; a value named like a measure,
  or not in lower-case letters, digits and underscores; waste outside 0 to 1000 percent; an
  order unit that is blank or the unit the line already comes out in; an order factor of
  zero or less, or not in the line's unit per order unit or the other way up; a unit cost
  in the wrong unit, negative, or beyond 1e9.
* *Recipes:* one that names no kind to apply to, or a Job, Roof or Condition, or a kind
  twice; one with no lines; one whose lines do not pass every check above on every kind it
  applies to.
* *Structure:* the same key twice in an object; an unknown field inside one of this
  program's components; an attribute under `pl::` that is not one of them; no elements; a blank or repeated path; a blank child name; a
  child that points at nothing; an element with two parents; a tree that does not lead
  back to one job, first in the file; a `next_id` not higher than every path number in use.

An eave height may be negative: it is a level relative to the job's datum.

Measures and line quantities in the file are **never trusted**. They are recomputed from
the geometry on every load, the recomputed figures are the ones used, and any element
whose stored figures do not match is reported by `show`, `check` and `set`.

## 4. Components

| Key in the file | On | Holds | Unit |
|---|---|---|---|
| `pl::class` | every element | The kind: Job, Roof, RoofPlane, LowSlopeArea, Eave, Rake, Ridge, Hip, Valley, Headwall, PitchBreak, Parapet, Condition; and for the rest of the building (section 5.6) Storey, Wall, Door, Window, Floor, Room, System, Duct, Pipe, Conduit, Cable, Fitting, Equipment, Fixture, Type | |
| `pl::job` | the job | Title, the unit derived lengths are worked out in (`ft`), the next path number | |
| `pl::footprint` | a Roof | The outline the roof stands on (`points`), its `eave_height`, and how each edge rises (`edges`, edge i from point i to the next): a pitch, or `"gable"` for an edge that stands upright. The roof is solved from these (section 5.5). | `points`: `ft` or `in`. `eave_height`: `ft`, `in` or `ft-in`. An edge: `in/12`, or none for a gable |
| `pl::solve` | a Roof with a footprint | What the last solve came to: `not_solved`, with the reason, when the footprint cannot be solved; `notes` on how the planes meet. Worked out, never typed. | |
| `pl::solved` | planes and lines the solver made | `from`: the roof it was solved from. `edge`: for a plane, the footprint edge it rises from, counted from 1. | |
| `pl::outline` | areas | `points`: the footprint traced on the plan, either way round. `eave`: the low edge the pitch rises from, as its own line along one edge of the outline (an edge traced in pieces is one edge when every corner inside it is within 0.01 ft of the one line from its start to its end), with no part of the outline on its other side. The slope is measured from that edge's line. `eave_height`. | `points`, `eave`: `ft` or `in`. `eave_height`: `ft`, `in` or `ft-in` |
| `pl::pitch` | areas | Rise in inches per 12 inches of run | `in/12` |
| `pl::run` | lines | `ends`: the two ends on the plan. `planes`: paths of the planes the line lies on. | `ft` or `in` |
| `pl::height` | parapets | Height | `ft`, `in` or `ft-in` |
| `pl::condition` | any | The assigned condition, by name | none |
| `pl::cost` | any | `code` (scheme, code, title) and `lines`. Each line is described below. | see below |
| `pl::schedule` | any | `wbs`: the work breakdown code | none |
| `pl::recipe` | a Condition | `applies_to`: the kinds of element the condition goes on. `cost`: the cost code and lines an element gets when it is applied. | as `pl::cost` |
| `pl::measure` | any with geometry | Derived quantities by name. Never typed. | |
| `pl::line_quantities` | any with cost lines | What each cost line works out to, by the line's description: `quantity`; `with_waste` and `order` when the line has them; `extension`, what it comes to in dollars; and for a line made of the element's material, `quotes`: what it comes to at each supplier's price. Never typed. | |
| `pl::allowance` | an element with one figure it is bought by | `rate`: a design-phase price for the whole element (section 5.7). | `$/SF` or `$/LF` |
| `pl::priced` | any with an allowance or cost lines | What the element comes to both ways, by name: `allowance`, `detail`, `detail_rate`, `difference`, `difference_percent`. Never typed. | |
| `pl::material` | an element that is built | What it is made of: the name of a Material element of the job. | none |
| `pl::supply` | a Material | How it is bought and what it costs. `order`, as a cost line's: the unit it is bought by and the factor (`1.25 lb/SF`, `30 SF/sheet`). `price`: the generic price. `quotes`: the same from suppliers, by the supplier's name. | `$/` + the unit it is bought by |
| `pl::markup` | the job | `overhead` and `profit`. Neither has a default. | `%` |
| `pl::level` | a Storey | `elevation`: how high its floor is above the job's datum. `height`: how far up to the next floor. `computation_height`, when it is not the usual 4 ft: how far above its floor its rooms find the walls that bound them. | each `ft`, `in` or `ft-in` |
| `pl::wall_type` | a Type | `thickness`. `layers`, when it is built up: each a `function` (structure, substrate, thermal, membrane, finish), a `material` and a `thickness`, from the exterior in. They add up to the thickness. | each `ft`, `in` or `ft-in` |
| `pl::floor_type` | a Type | `thickness`, and `layers` from the top down, as a wall type's. | each `ft`, `in` or `ft-in` |
| `pl::opening_type` | a Type | `category`: Door or Window. `width`, `height`. | each `ft`, `in` or `ft-in` |
| `pl::segment_type` | a Type | `category`: Duct, Pipe, Conduit or Cable. `round` and `width` (the diameter), or `width` and `depth`. | each `ft`, `in` or `ft-in` |
| `pl::wall` | a Wall, under a Storey | `type`: the path of a Type with a `pl::wall_type`. `line`: its location line on the plan, first end to second; its exterior side is to the left of that. `location`, when the line is not its centre line: `core-centre`, `exterior-finish`, `interior-finish`, `exterior-core`, `interior-core`. `top_level` (the path of the Storey its top is held to) with `top_offset` if any, or `height`: one or the other. `base_offset`, when it does not stand on its storey's floor. `room_bounding: false` and `join: false` when it does not bound rooms or join its neighbours. | `line`: `ft` or `in`. The others: `ft`, `in` or `ft-in` |
| `pl::opening` | a Door or Window, under its Wall | `type`: the path of a Type with a `pl::opening_type` of its kind. `offset`: how far its **centre** is along the wall's line from the line's first end. `sill`: how high its bottom is above the wall's base. | each `ft`, `in` or `ft-in` |
| `pl::floor` | a Floor, under a Storey | `type`: the path of a Type with a `pl::floor_type`. `boundary` on the plan. `offset`, when its top is not at its storey's floor level. | `boundary`: `ft` or `in` |
| `pl::room` | a Room, under a Storey | `point`: a point inside it. `number`: what the plan calls it. Its boundary is not stored: it is found from the walls. | `point`: `ft` or `in` |
| `pl::segment` | a Duct, Pipe, Conduit or Cable, under a Storey | One straight piece. `type`: the path of a Type with a `pl::segment_type` of its kind. `line`: its two ends on the plan. `offset`: how high it is above its storey's floor at the first end; `end_offset` at the second, when it is not level. `justify`, when its heights are those of its `top` or `bottom` and not its centre. | `line`: `ft` or `in`. The others: `ft`, `in` or `ft-in` |
| `pl::placed` | Equipment, a Fixture or a Fitting, under a Storey | `what` it is. `at`: the middle of its base on the plan. `offset`: how high that is above its storey's floor. `angle`, when it is turned: degrees counter-clockwise from east. `size`: along its own x, along its own y, tall. | `at`, `size`: `ft` or `in`. `offset`: `ft`, `in` or `ft-in`. `angle`: `deg` |
| `pl::fitting` | a Fitting | `kind`: bend, tee, cross, transition or coupling. | |
| `pl::connectors` | Equipment, a Fixture or a Fitting | A list. Each: `name`; `kind` (pipe, duct, conduit, power); for a pipe, `domain` (cold, hot, waste, vent, storm, gas); `flow` (in, out) when it is not both; `at`, in the thing's own axes (x along its angle, y to its left, z up from its base); `size`. | `at`: `ft` or `in` |
| `pl::system` | a System | `discipline` (Mechanical, Electrical, Plumbing) and `classification`: Supply Air, Return Air, Exhaust Air, Refrigerant, Domestic Cold Water, Domestic Hot Water, Sanitary, Vent, Power, Lighting. | |
| `pl::in_system` | a run, Equipment, a Fixture or a Fitting | The system it belongs to, by the system's path. It has to be a System in the job. | none |

A file saved before the building model was re-based on BIMCraft's shapes (DECISIONS.md
call 29) does not load: its `pl::route` and `pl::size` are not keys this program knows,
and its walls and openings lack a type. No job had been saved with them but the samples.

### 4.1 A cost line

A line is worked in steps that each stay visible: formula, then quantity, then waste, then
order units.

| Field | Holds | Unit |
|---|---|---|
| `description` | What the line is. No two alike on one element, ignoring capitals and spacing. | |
| `quantity` | The line's formula, as the estimator writes it. | |
| `stretch_out` | Optional. The width of the flat sheet the profile is bent from, as the detail gives it. | `ft`, `in` or `ft-in` |
| `inputs` | Optional. Any other values the formula names, by name: a `spacing`, a number of `sides`. | a length, or none for a plain number |
| `waste` | Optional. Percent added to the quantity. | `%` |
| `order` | Optional. `unit`: what is bought, one word (`sheet`, `lb`). `factor`: the conversion. `round_up`: buy whole ones. Typed as `30 SF/sheet` or `1.25 lb/SF`, with `whole` or `exact` after it to say which; unsaid, a package (the divide form) is whole and a weight is exact. | `factor`: `SF/sheet` (divide) or `lb/SF` (multiply) |
| `unit_cost` | The price. Absent on a line made of the element's material. | `$/` + the order unit, or the unit the quantity comes out in |
| `class` | Optional. Which row of the recap the line lands in: `Material`, `Labor`, `Fabrication`, `Supervision`, `Subcontract` or `General`. | |
| `off` | Optional, on a Labor line: switched off. It stays on the sheet and is out of every total. | |
| `of_material` | Optional. The line is made of the element's material: it is ordered and priced as that material is, and carries no `order` or `unit_cost` of its own. Its class is Material. | |

**The formula.** Names, numbers, `+ - * /` (or `×`, `÷`) and brackets. A name is one of:
the element's measures (`plan_length`, `true_length`, `face_area`, `plan_area`,
`surface_area`, `slope_factor`); a parapet's `height`; `stretch_out`; a name in `inputs`.
`squares` can stand alone as a quantity and cannot be mixed into a formula.

* **Units are carried through it.** Every length is brought to feet, whatever it was
  entered in. Length × length is an area; length ÷ length is a count. The unit the line
  comes out in follows from the formula: a count (EA), a length (LF) or an area (SF).
  Nobody types `/ 12`.
* **A count is whole.** A formula that comes out as a count is rounded up to the next
  whole number, and the worked formula says so; so is a count with waste added. A count
  that comes out below zero is refused before it is rounded, so it is never rounded up to
  nothing, and one above zero but too small to tell from zero (under a billionth) is
  refused as that.
* **A formula is something read on a line.** It holds at most 200 names, numbers, signs
  and brackets. A formula made only of a factor and plain numbers (`slope_factor * 1`) is
  a factor, not a quantity, and is refused.
* **Status is carried through it.** The result is as good as its worst operand. An Unknown
  operand leaves no result at all, never a zero.
* **Stretch-out belongs to the line, not to the material.** The same copper is 12 in at an
  eave and 36 in in a valley. A recipe cannot carry one: a recipe line with a stretch-out
  filled in is refused.

**A condition as a recipe.** A Condition is an element of the job. Applying it to an
element of a kind it is for gives that element the condition's name, its cost code, and a
copy of its lines. The copies are then edited on the element. What the recipe leaves blank
is Unknown on the element until someone enters it. Anything else a recipe states (a number
of sides, a waste, a price) is copied with the recipe's status and source, and `show`
lists it where the recipe is read. Applying the same recipe twice is refused (its lines
would be there twice), and so is applying a recipe to an element that already has another
condition's name: an element is one condition.

How many planes a line names: a hip, valley or pitch break, two; an eave, rake or headwall, one; a
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

The formula saved beside each figure is the one that was used, conversion included.
Figures are worked from unrounded inputs and written to four decimals, so multiplying the
rounded figures in the file can differ from the figure shown in its last place. A `true_length` that leans on
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
* the line does not run along the outline of a plane it names: the outline edges within
  0.01 ft of the line are laid along it and have to cover it end to end. What is not
  covered is added up, and the total has to stay within 0.01 ft: a line that crosses a
  notch, or a hundred hairline notches, or cuts across the plane, fails;
* the planes put either end more than 0.01 ft (about 1/8 in) apart in height;
* the line's kind and its slope disagree: an eave, ridge or headwall that is not level on
  its plane, or a rake, hip or valley that is level on a plane with enough pitch to lift
  it more than 0.1 ft over its length;
* another line on one of its planes fails the test just above on that plane. That line
  is evidence the plane's eave is entered on the wrong edge, and a rake can climb under
  both readings of a plane, so every line on that plane loses its length until the eave is
  settled. The test is made on every plane a line names, before the planes are compared
  with each other, and nothing about the line is decided until every plane it names has
  been looked at, so the evidence counts even when the line is refused for something else
  (another plane it names is missing, lacks a pitch, or is not where the line runs), and
  counts against each plane it shows wrong.

Example: steepen one plane of a hip roof while the hips stay where they were traced, and
those hips and the ridge are flagged with both heights. The roof as drawn no longer closes.

### 5.4 Line quantities

Each step is saved with the formula that was worked, its unit and its status.

| Step | Formula as saved | Unit |
|---|---|---|
| `quantity` | The line's formula with each conversion written in: `plan_length × stretch_out ÷ 12`, `plan_length × sides ÷ (spacing ÷ 12), rounded up to a whole number`, `plan_length × (height ÷ 12 + turn_out ÷ 12)` | EA, LF or SF by the formula; a measure standing alone keeps its own |
| `with_waste` | `quantity × (1 + waste ÷ 100)` | the quantity's |
| `order` | `with_waste ÷ factor, rounded up to a whole sheet`, or `with_waste × factor` (from `quantity` when the line has no waste) | the order unit |

Each step is worked from the step before as the file shows it (four decimals), and the
quantity from the element's measures as the file shows them, so a line can be followed by
hand from the page: 20 LF × 19 1/2 in ÷ 12 = 32.5 SF; plus 10 percent, 35.75 SF; at 30 SF
to the sheet, 2 sheets; or at 1.25 lb to the SF, 44.6875 lb. Rounding up allows for the
crumbs binary arithmetic leaves, so 60 SF at 30 to the sheet is 2 sheets, not 3.

A step whose input is Unknown has no number. A formula that comes out below zero has no
number, with the reason.

### 5.5 A roof solved from its footprint

A Roof that carries a `pl::footprint` is solved: each sloping edge moves in at 12 ÷ pitch
for every foot of height, a gable edge stays where it is, and where the moving edges meet
is where the slopes meet (a weighted straight skeleton; `skeleton.rs`, ported from
BIMCraft). That gives one plane to each sloping edge. The planes give the lines, read off
them the way `plumbline lines` reads them (DECISIONS.md call 17): eaves, rakes, ridges,
hips, valleys.
Planes and lines are written into the job as elements under the roof, each marked
`pl::solved`, and from there they are ordinary elements: derived, checked, priced and
saved like any other.

* **Solving again keeps what it can, and goes by where things are.** A plane stays with
  the edge its eave lies along: same path, same condition, same cost lines. If a point is
  added to the footprint and the edges after it take new numbers, the plane keeps its
  slope and takes the new number, and the solve says so. An edge moved square to itself
  lies along nothing it did before and keeps its plane by its number. A line is known by
  its kind and the edges of its planes (`hip 1-2`, `ridge 1-3`); a line on one plane also
  says which footprint edge it stands over (`rake 1 at 4`), so a plane's two rakes are
  told apart by where they are. What no longer exists is removed and reported.
* **What is solved is never trusted from the file.** Every roof with a footprint is solved
  again on load. Planes and lines that no longer match the footprint are put right and
  reported, as stale measures are. If the footprint cannot be solved (an edge with no
  pitch), the planes and lines of an earlier solve lose their geometry and show Unknown,
  and the roof shows `not solved` with the reason.
* **A change of pitch is a line.** Where two slopes of different pitch meet and both fall
  the same way, the line between them is a `PitchBreak`. It has a length like any line and
  no rule about being level or climbing.
* **A wall between two planes.** Where two planes share an edge on plan at different
  heights (a gable beside a lower roof), each has an edge of its own there with its own
  line, and the roof carries a note saying so.
* **Only what the solver made is replaced.** Elements under the roof without `pl::solved`
  are left alone.
* **Status.** A solved plane's outline takes the worst status among the footprint's
  points and every edge's rise; its pitch takes its own edge's; its eave height the
  footprint's.
* **Coordinates** are written to a millionth of a foot.
* **Refused:** a footprint on anything but a Roof; an outline that is not a simple loop;
  a number of rises other than the number of points; an edge under 0.02 ft long (about
  1/4 in); a point more than nothing and up to 0.01 ft off the straight line between its
  neighbours (it would get a hip of its own); a pitch under 0.01 or above 240 in 12; every
  edge a gable; gables that fence in part of the roof; a slope that closes round a hole
  (courtyards) or comes back to touch itself at a point (not solved yet); a move of an
  edge that would take it past the other side of the footprint.
* **Edits** (`set <roof> edge:<n>`, `set <roof> move:<n>`, `set <roof> eave_height`, and
  the 3D window): the footprint is changed and the roof solved again; if the changed
  footprint is refused or cannot be solved, the job is put back exactly as it was.

### 5.6 The rest of the building

Worked out in `building.rs`, under the same three rules as the roof (section 5). The
shapes are BIMCraft's, ported (`building/model.rs` and beside it; `PORTMAP.md`).

**Types.** A Type is an element directly under the job. It carries one of the four type
components, and a wall, floor, door, window or run names its type by path. A size is
entered once. A type that is not known, or is not in the job, leaves what depends on it
with no figure and the reason; nothing falls back to a number.

**Heights.** A wall's base is its storey's elevation plus its `base_offset`. Its top is its
`top_level`'s elevation plus its `top_offset`, or its base plus its `height`. A run's and
a placed thing's heights are above the storey they are under.

| Kind | Quantity | How |
|---|---|---|
| Wall | `plan_length` LF | length of its line on the plan |
| Wall | `height` ft | its `height`, or from its base up to its top level. Refused when the top is not above the base, or an elevation it needs is not known. |
| Wall | `face_area` SF | `plan_length × height`: one face, with nothing taken out |
| Wall | `opening_area` SF | the `opening_area` of each door and window under it, added up. A wall with none has 0, and that zero is Known. It is as sure as the openings' sizes; where each one is does not come into it. |
| Wall | `net_area` SF | `face_area − opening_area`. Refused, with the reason, when an opening starts before the wall does, runs past its end, stands taller than it, or lies over another, when an opening has no area, or when where an opening is in the wall is not known. It is as sure as where each opening is as well as its size. The gross face still stands. |
| Door, Window | `width`, `height` ft | its type's |
| Door, Window | `opening_area` SF | `width × height` |
| Floor | `plan_area` SF | area of its boundary on the plan |
| Room | `floor_area` SF | the floor inside the faces of the walls round its point, less any island of walls standing in it |
| Room | `gross_area` SF | the same to those walls' centre lines |
| Room | `perimeter` LF | the distance round the room at the faces of its walls, an island's faces included |
| Duct, Pipe, Conduit, Cable | `true_length` LF | the length of its line on the plan when it is level; through its rise, `√(plan² + rise²)`, when its two ends are at different heights. A riser is its rise. |
| Duct | `surface_area` SF | `true_length ×` the distance round it: `π × diameter`, or `2 × (width + depth)`, its type's. Refused when its type or its size is not known. |

**Rooms.** A storey's rooms are found among the walls that bound rooms and stand at the
storey's computation height (4 ft above its floor unless the storey says otherwise),
whatever storey those walls are under. A room is the smallest loop round its point. Where
walls meet they are joined as BIMCraft joins them: end to end at any angle, or one ending
on another; their centre lines run on to meet, so a room closes whatever line of a wall
was drawn. A room has no figure, and says why, when no loop closes round its point, when
its point is inside a wall, or when a room added before it has the space. A wall under
the room's own storey that cannot be placed (no height, no type, no thickness) leaves
every room of that storey with no figure: whether it closes a room off is not known. One
under another storey makes the figure Assumed, and the figure's note says what is assumed.
A figure is no surer than the walls it is measured to, and names them. A bounding wall is
joined to every wall it meets, bounding or not: a partition that ends on a low wall stops
at that wall's face. A Wall element with nothing entered on it counts as a wall that
cannot be placed. Where a wall's base or top rests on anything less than Known, each room
of the storey is worked out again with that wall gone and with it standing full height,
and a room that comes out differently either way is no surer than that wall's heights and
says which wall. A room whose point is inside a wall has no figure at all.

**Runs and fittings.** A run of duct or pipe that turns corners is one element for each
straight piece and a Fitting at each corner. Length is measured through fittings, centre
to centre. A run with no size keeps its length. It has no surface, and on a takeoff it is
on a line of its own, "size unknown". It is never left out and never given a size.

**The network** (`building::network::network`, ported from BIMCraft's `network.rs`) is
worked out when asked and never stored. The ends of runs and the connectors of placed
things that are within 0.01 ft of each other are one node; only points of one kind of run
join. An end on another run's length is a node that splits that run without splitting
the element. A fitting's connectors are joined through it; a fixture's and equipment's are
not. It gives the nodes, the edges, the groups of what is joined, the open ends, and
warnings: two systems in one network, two pipe domains in one, a run met along its length
with no fitting, and, where a drain drops, a pipe leaving the drop narrower than one
coming down to it (a drain that narrows on the level is not looked for). It never refuses a job. What cannot be
placed is listed in `left_out` with why.

`building::takeoff` gives each system's runs by kind and size, with the length of each
line as a sum that says what it could not include (a sum over nothing has no figure; it
is not a Known zero); its fittings counted by kind and by what each is said to be; its
equipment and fixtures counted by what each is said to be; and how it is `joined`: how
many separate networks its runs fall into, each open end by name and place, each place a
run meets another along its length with no fitting, and the other warnings, each said on
the systems of the runs it is about. Runs are grouped by their size as a size, said in
inches, whatever unit each was entered in, and listed smallest first with "size unknown"
last. A line is no surer than that its runs are on the system: an Assumed membership
makes the line Assumed. After the systems come two groups, each only when it has
something in it: "On no system" (no `pl::in_system` at all, which is Known) and "System
not known" (a `pl::in_system` that is Unknown), flagged Unknown. Nothing is off the
takeoff for want of a system. A storey has to be directly under the job, and each wall,
floor and room counts on its nearest storey only. `building::storey_totals` gives each
storey's walls, floors and rooms.
`building::placements` gives every element's shape in feet, in space, for a window to
draw, and beside it the list of elements that have no shape and why. A window has to say
the second list: an element that cannot be drawn does not just go missing. A door or
window that cannot be cut in its wall (it has no size, it runs past the wall's end, it
stands taller than the wall) is on that list too.

Not modelled yet: curved walls; openings in a floor; a wall's top following a roof;
room heights and volumes; families; structure; stairs; sizing and code checks. (Clashes
are section 5.10.)
PROPOSALS.md has each.

### 5.7 Money

Every amount of money is worked out in the world crate (`lines.rs`, `view.rs`) and is
written to the cent. A total is the sum of the amounts as they are shown. Half a cent
rounds up, as on paper. An amount above zero and under half a cent, or over $10 trillion,
is refused. Nothing is $0.00 unless it comes to nothing: a line with no price, no
quantity or no class has no figure and is named by every total that leaves it out; a
line whose quantity is 0 comes to $0.00 and says that is why.

**A line's extension** is what it is priced in (its order quantity, else its quantity
with waste, else its quantity) times its unit cost. It is as sure as the worst of: the
quantity, the price, the order factor, the waste, and whether the element is the
condition its lines came from (an Assumed condition makes assumed money; a condition
that is not known leaves no figure).

**An allowance** is a rate on the one figure a thing of its kind is bought by: a roof
plane's surface area, a wall's net area, a floor's area, a room's floor area, the length
of a parapet, a roof line or a run. **An element's detail** is its lines' extensions
added up, when every one of them has a figure, and none otherwise; lines switched off
are not in it. The detail over the same figure is the rate it rolls up to, and the
difference from the allowance is given in dollars and as a percentage of the allowance.
`view::pricing` adds these up for the job: each total says how much of itself rests on
something assumed and names what it leaves out, and the like-for-like comparison is over
the elements that have both figures and no others.

**Classes.** A line with a class lands in that row of the recap and in no other. In-house
work (Labor, Fabrication, Supervision) is a line in hours: its order unit is `hr`, its
order factor is the production rate (`25 SF/hr`, or `0.25 hr/LF`) and its unit cost is
dollars an hour. No other line is in hours, and a Subcontract line is refused a
production rate. The estimate sheet has three money columns: Material; Labor (with
Fabrication and Supervision); General (Subcontract and General conditions). General
conditions are cost lines on the job element itself.

**A material** is an element. A line made of its element's material takes the material's
order factor and price, and is extended again at each supplier's quote. Naming another
material on the element reprices those lines and no others. A material the job does not
have, or one priced in a unit the line does not come out in, leaves the line with no
figure and the reason.

**The recap** (`view::estimate`): each class's total and, for in-house work, its hours;
the job cost, their sum; overhead, a percentage of the job cost; profit, a percentage of
the job cost and the overhead together; the price, the three added. With overhead or
profit not entered there is no price. A supplier's total is over the lines it has quoted
and names the rest.

### 5.8 Drawing and editing

`editing.rs` holds the verbs a window or a command line uses to build a job without
editing its file: `storey`, `wall_type`, `opening_type`, `segment_type`, `system`,
`walls`, `opening`, `room`, `runs`, `place`, `move_by` and `remove`. The command line has
one command for each (`plumbline new`, `storey`, `type`, `system`, `wall`, `door`,
`window`, `room`, `run`, `place`, `move`, `remove`), and the window's tools call the same
verbs (`crates/app/src/tools.rs`). The rules, each ported from BIMCraft's `commands.rs`
(PORTMAP.md has the lines):

* **Every value a verb writes has a status and a source**, given by whoever calls it.
  The command line refuses a verb without `--status` and `--source`. The window writes
  Assumed, "set in the 3D view; not yet confirmed against a drawing".
* **Nothing is given a value nobody entered.** A wall has no default height, an opening
  no default sill or place, a run no default height above the floor.
* **A verb is all of its change or none of it.** Three walls of which the second cannot
  be drawn leave the job as it was.
* **A wall is one element for each pair of points**; `--closed` joins the last to the
  first. **An opening is placed by its centre along its wall**, from a distance or from
  a point on the plan; a point past either end of the wall is refused.
* **A wall moved takes its doors and windows; a wall removed takes them too.** What is
  moved is no surer than the move. An element with elements under it is not removed.
* **A room is a point**; its boundary is found from the walls round it (section 5.6).

`editing/history.rs` (ported from BIMCraft's `engine.rs`) keeps the job's saved text
before each step, up to 200 steps, so a step back is the job exactly as it was. The
history is the window's, for the time it is open; it is not saved in the job.

`editing/snap.rs` (ported from BIMCraft's `snap.rs`) finds where a pointer on the plan
means: within reach, the nearest of an end, a crossing or a middle (the end when two are
as near); else the foot of the perpendicular from the last point onto a line near the
pointer; else a guide through the last point every 15 degrees, when the pointer is within
4 degrees of one; else the nearest point on a wall's line; else the nearest inch.

Not there: a verb for a floor; arcs; a run that changes height along its length; moving
one end of a wall; moving or removing several things at once.

### 5.9 Fittings and connections

`editing/join.rs` (DECISIONS.md call 39). Three verbs, each all of its change or none:

* **`fitting`** (`plumbline fitting <file> <storey> --at "x,y"`): a fitting where runs'
  ends meet. Its kind is worked out from what meets: two ends at an angle, a bend; two
  in line of one size, a coupling; two in line of two sizes, a transition; three, a tee;
  four, a cross. A run that passes through the point is cut there first. The fitting has
  a connector for each run, takes the runs' system, and is as sure as the least sure of
  them. Refused: one end, five or more, runs of two systems, a fixture's connector at
  the joint, cable, a joint that already has its fitting, a kind asked for that is not
  the kind the joint takes. `--all` does every joint that has none and lists what it
  passed over; it does not put a coupling between two runs of one size in line.
* **`split`** (`plumbline split <file> <run> --at "x,y"`): a run cut in two at a point on
  its line. The first piece keeps the path; the second is new and carries what the first
  carries. A run that climbs is as high at the cut as its line is there. Refused: a
  point off the run or at its end, an upright run, a run with a cost line whose quantity
  is a typed number.
* **`connect`** (`plumbline connect <file> <thing> <run>`): a fixture or a piece of
  equipment given a connector where the run's end is, in the thing's own axes, of the
  run's kind and size. Refused: a run neither of whose ends is in the thing's box, both
  of whose ends are, a thing with no size, a connector already there.

After these the takeoff's "joined" line (section 5.6) says how many networks a system
is in, how many ends are open, and where a run still meets another with no fitting.

### 5.10 Clashes

`building/clash.rs` (DECISIONS.md call 40); `plumbline clashes <file> [--clearance "2 in"]`;
the sheet's Clashes tab. Worked out from the job each time and never stored.

* **What is tested** is every element `building::placements` gives a shape: a wall as
  its solid pieces round its doors and windows, a floor as its outline cut into slabs,
  a run as a cylinder or a box, a fixture, equipment and a fitting as boxes.
* **In the same space** (a hard clash): two solids in each other by more than the fit
  tolerance, 0.01 ft. Reported with how far one would have to move to clear the other,
  where that can be worked out, and where.
* **Through a wall or a floor**: a run or a fitting in a wall or a floor. Listed apart,
  with no depth: a hole to cut.
* **Closer than a clearance** (a clearance clash): only when one is asked for, up to
  10 ft. Reported with how far apart.
* **Never tested:** walls and floors against each other; what is joined (two runs at a
  joint, a run and its fitting, a run and the thing it is connected to); what fills a
  door or a window; the roof's planes.
* **Not tested, and listed with why:** anything with no shape.
* **A clash takes the worse status of its two shapes.**

The solid geometry is `parry3d-f64`'s. Its measure of how deep two solids are in each
other answers zero when one is dead centre in the other, so the yes or no is its
overlap test and the depth is asked for separately (call 40).

## 6. Cost code scheme

Which scheme is primary is an open question (QUESTIONS.md Q1). Every code stores its
`scheme` beside it. No total is grouped by code in D1.

## 7. What D1 does not do

* **It does not re-solve the roof.** Ridge, hip and valley positions are entered, not
  worked out from the footprint and pitches. D2 does that with the skeleton (DECISIONS.md
  call 3); until then a pitch change the linework cannot follow is flagged per 5.3.
* **The window** (`crates/app`) shows the roof, the building and the sheet. It changes a
  solved roof's pitches, eaves and conditions, types prices, and draws, moves and
  removes walls, openings, runs, fixtures and equipment on the plan (section 5.8).
  Everything else is still entered through the command line. Both read the same rows (`view::elements`,
  `view::roof_totals`); the window works out no quantity of its own.
* **Money is section 5.7** (D4 and D5). Not there: tax, burden, escalation, general
  liability and bond (PROPOSALS.md item 12); a production rate by the crew-day (13);
  pricing the job at a chosen supplier (14).
* **A recipe's lines are copied, not linked.** Changing a recipe does not change the
  elements it was applied to before.
* **Waste is one percentage on the line.** There is no separate waste for the material
  and for the condition.
* **An order factor is a number** (30 SF to the sheet). A sheet size is not entered as
  3 ft × 10 ft and multiplied out.
* **A material is one price and one order factor.** There is no library of materials
  beyond the job's own Material elements, and an element is made of one.
* **Metric, and pitch in degrees or percent, are refused**, not converted (PROPOSALS.md
  items 5 and 6).
* **Removing a cost line, writing a recipe, and re-entering outlines and line ends are
  done in the file.** (`line` adds a cost line; `set` enters a unit cost, an allowance, a
  material, a material's price and quotes, and the job's overhead and profit; `class` and
  `labor` set a line's class and switch.) `set` edits pitch, height, eave height, condition
  and schedule code on an element, and on a cost line its stretch-out, its formula (with
  the values the formula brings in), its waste, its order conversion, and any value its
  formula names. `apply` gives an element a condition's lines.
* **No conditions library.** Undo is in the window only (section 5.8); the command line has none.
* **Planes are checked against each other only where they share an edge, and only on
  request.** `plumbline lines` reports two planes that share an edge and disagree about
  its height, and sets the lines the planes imply against the lines entered (DECISIONS.md
  call 17). Loading a job does not run it, and it changes nothing.
* **Not checked:** outlines that overlap each other or leave gaps; a plane whose eave is
  entered on the edge parallel to the true one (every line still fits and every length is
  right, but its heights run downhill); a plane whose eave is entered on the wrong edge
  when no line entered on it is level where it should climb or climbs where it should be
  level (with only a rake entered, nothing contradicts the eave); two cost lines whose
  descriptions look alike but are spelled with different letters (a Cyrillic о, an ﬂ
  ligature); whether a cost code is a real code in its scheme.
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
| `crates/world` (`plumbline-world`) | `bevy_ecs`, `serde`, `serde_json`, `parry3d-f64` (solid geometry, for clashes) | The model: values, units, components, checks, plan geometry, derivation, line formulas and the steps a line is worked in, the lines planes imply, the saved file, the sample job, the flat view |
| `crates/cli` (`plumbline`) | `plumbline-world` | The command line: the samples, `show`, `check`, `lines`, `takeoff`, `price`, `estimate`, the entry commands (`set`, `apply`, `line`, `class`, `labor`, `material`, `roof`) and the drawing verbs of sections 5.8 and 5.9, and `clashes` |
| `crates/app` (`plumbline-app`) | `plumbline-world`, `bevy` | The 3D window and the sheet window |

The world crate does not depend on any renderer or window, and must not. The engine is
linked by the application crate alone.
