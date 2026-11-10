# SPEC: the world data model

What D1 built. Later deliverables add components and views; they do not change these rules.

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
3. Every entered value is in the one unit its field takes (section 4). The program
   converts nothing: a height in inches or an outline in metres is refused.
4. A derived value takes the **worst** status among its inputs. An Assumed pitch makes an
   Assumed area. An Unknown height leaves the wall area with no number at all.
5. A derived value shows its `formula` and lists its `inputs` (`path#component`).
6. **A quantity never disappears.** Every element of a geometric kind lists every one of
   its quantities. If an input is missing, the quantity is there as Unknown with the
   reason ("no pitch entered").
7. When inputs do not fit together, the derived value is Unknown and its `note` gives the
   figures. See 5.3.
8. Arithmetic that overflows gives Unknown, not infinity.

**What counts as a value.** Dimensions, pitches, heights, outlines, line ends, assigned
conditions, cost codes, unit costs and schedule codes carry a status and a source.
References and labels do not: the planes a line names, an element's name, a cost line's
description and the measure it is priced by. Those are checked for sense instead.

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
* **Lengths and plan coordinates are in feet**, 64-bit.

### Nothing is dropped quietly

| Where an unknown key is | What happens |
|---|---|
| Top level of the file, the header, a `schemas` entry, a data entry, IFC5's `inherits` | Kept and written back unchanged |
| An attribute whose key this program does not know | Kept and written back unchanged |
| A field inside one of this program's own components (`pl::pitch`, a cost line, ...) | **Refused**, naming the field |

The last row is deliberate. A cost line with an extra `extended_total` is a price with no
status and no source; it is refused, not carried.

### Loading checks the file

A file is refused if: a value breaks the rules in section 2; a component sits on a kind
that cannot carry it (an outline on a hip); an outline has fewer than three points, a
repeated point, crossing or touching edges, or no area; an eave line has no length or
does not lie along one edge of its outline; a pitch is negative or above 240 in 12; a
parapet height is negative; a line's two ends are the same point; a coordinate is not
finite or is beyond 1,000,000 ft; a cost line has no description, is priced by a measure
its element does not have, is priced in the wrong unit, or has a negative unit cost; a
cost code lacks a scheme or a code; a path is blank or repeats; a child name is blank; a
child points at nothing; an element has two parents; the tree does not lead back to one
job. An eave height may be negative: it is a level relative to the job's datum.

Measures in the file are **never trusted**. They are recomputed from the geometry on every
load, the recomputed figures are the ones used, and any element whose stored measure does
not match is reported by `show`, `check` and `set`.

## 4. Components

| Key in the file | On | Holds | Unit |
|---|---|---|---|
| `pl::class` | every element | The kind: Job, Roof, RoofPlane, LowSlopeArea, Eave, Rake, Ridge, Hip, Valley, Headwall, Parapet | |
| `pl::job` | the job | Title, length unit (`ft`), the next path number | |
| `pl::outline` | areas | `points`: the footprint traced on the plan, either way round. `eave`: the low edge the pitch rises from, as its own line along one edge of the outline. `eave_height`. | ft |
| `pl::pitch` | areas | Rise in inches per 12 inches of run | in/12 |
| `pl::run` | lines | `ends`: the two ends on the plan. `planes`: paths of the planes the line lies on. | ft |
| `pl::height` | parapets | Height | ft |
| `pl::condition` | any | The assigned condition, by name | none |
| `pl::cost` | any | `code` (scheme, code, title) and `lines`. Each line has a `description`, the `quantity` (a measure on this element) it is priced by, and a `unit_cost`. | `$/` + the measure's unit |
| `pl::schedule` | any | `wbs`: the work breakdown code | none |
| `pl::measure` | any with geometry | Derived quantities by name. Never typed. | |

How many planes a line names: a hip or valley, two; an eave, rake or headwall, one; a
ridge, one or two; a parapet, none.

An entered value is written `{ "value", "unit", "status", "source" }`.
A derived value is written `{ "value", "unit", "status", "formula", "inputs", "note" }`.

## 5. Derived quantities

Derived figures are written to four decimal places.

### 5.1 Areas (RoofPlane, LowSlopeArea)

| Measure | Formula | Inputs |
|---|---|---|
| `plan_area` (SF) | area of the outline on the plan | outline |
| `slope_factor` | √(1 + (pitch ÷ 12)²) | pitch |
| `surface_area` (SF) | plan_area × slope_factor | outline, pitch |
| `squares` (SQ) | surface_area ÷ 100 | outline, pitch |

### 5.2 Lines

| Measure | On | Formula | Inputs |
|---|---|---|---|
| `plan_length` (LF) | every line | length of the line on the plan | run |
| `true_length` (LF) | eave, rake, ridge, hip, valley, headwall | √(plan_length² + rise²) | run, and the outline and pitch of each plane named |
| `face_area` (SF) | parapet | plan_length × height | run, height |

The height of a plane above a plan point is its eave height plus pitch ÷ 12 × the point's
distance from the plane's eave line, measured square to the eave. A line's rise is the
difference between the heights of its two ends. Its status is the worst among its own
ends and every entered value on each plane it names (outline, eave line, eave height,
pitch).

### 5.3 A line has to fit its planes

`true_length` is Unknown, with a note saying why, when:

* the line names the wrong number of planes for its kind, the same plane twice, a plane
  that is not in the job, or an element that is not a plane;
* a named plane lacks its outline or its pitch;
* either end of the line is more than 0.01 ft off the outline of a plane it names;
* the planes put either end more than 0.01 ft (about 1/8 in) apart in height;
* the line is an eave and its plane does not make it level.

Example: steepen one plane of a hip roof while the hips stay where they were traced, and
those hips and the ridge are flagged with both heights. The roof as drawn no longer closes.

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
* **No undo, no conditions library, no derived conditions** (D3).
* **Not checked:** outlines that overlap each other; a line that touches a plane's outline
  only at a point but is drawn across its interior; whether a rake, ridge or headwall is
  the kind of edge its name says (only an eave is checked for being level); whether a
  cost code is a real code in its scheme.
* **The 0.01 ft fit tolerance** is tighter than a hand trace of a scanned plan can hold
  (QUESTIONS.md Q9).
* **Paths are counter-based**, so two copies of a job edited apart can mint the same path
  (PROPOSALS.md item 4).
* **`Job::put` is unchecked.** Code that builds a job directly (the sample does) bypasses
  the entry checks; a job built that way is checked when it is next loaded.

## 8. Code layout

| Crate | Depends on | Holds |
|---|---|---|
| `crates/world` (`plumbline-world`) | `bevy_ecs`, `serde`, `serde_json` | The model: values, components, checks, plan geometry, derivation, the saved file, the sample job, the flat view |
| `crates/cli` (`plumbline`) | `plumbline-world` | `sample`, `show`, `set`, `check` |

The world crate does not depend on any renderer or window, and must not.
