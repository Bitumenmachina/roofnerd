# Open questions

Things where a wrong call would waste work. Each one names the smallest version built so
the rest could proceed. Not a stop — the build continues past every entry here.

## Q9 — does an item's price source become a firmness type?

**Built:** nothing. `priceSource` is a string on the item, as it has been since section 3.

**The question:** should it become a small record carrying where a price came from *and how firm it
is* — quoted, budget, estimate, placeholder — rendered as a chip on the line?

**Why it is being asked now rather than later.** The same idea has now been arrived at independently
four times, and one of those arrivals is already shipped code:

- NEXUS types it — `Confidence: firm | budget | estimate | placeholder` — and ships the colour
  language for it (`nexus-phase0/viewer/src/index.css`).
- The prior estimating lineage reached it as a weakest-wins basis chain over a line's inputs.
- The design bundle draws it as a provenance chip on every quantity.
- This program reached it as prose: *nothing is ever silently zero*, a line with no price says so.

Four arrivals is not a coincidence; it is a shape the problem has. **Section 4 is the cheap moment**
— once section 5's lenses render prices, retrofitting firmness into a priced chain is the same class
of change as retrofitting a second waste term, and expensive for the same reason. Section 5 also
needs it: a supplier lens that hides cost still has to decide whether to show a placeholder as a
quantity.

**Narrowed 2026-09-08.** Two thirds of this stopped being a question once the axes were separated
(D66). Authorship — derived or typed — is the design bundle's contract and is settled. Validity —
usable, pending, excluded, no price — is "nothing is ever silently zero" and is settled. Neither
needed Patrick.

**What is left is one axis: firmness.** Does a price carry how good it is — firm, budget, estimate,
placeholder? That is the only part that widens the item, and it is the only part still asked.

**Why it waits:** it widens the item, and a wrong shape there is paid for in every lens. Declining is
a perfectly good answer and costs a string.

## Q7 — orbit camera, or fixed isometric?

**Built:** nothing yet. When section 6 renders, it builds **orbit, per §4.10**, which is the ruling
document.

**The disagreement:** §4.10 says "orbit camera". The design bundle says fixed isometric with preset
corners, fit, and a vertical exaggeration control — *"no free orbit — the user never orbits or drags
vertices"* — reasoning that 3D here is a view of the takeoff, not a modeling environment. The earlier
RAISE work was also isometric.

**Why it waits, and what makes it cheap either way:** a half-inch-over-twenty-two-feet fall is
invisible at true scale, which is an argument for exaggeration over orbit. But §4.10 is the ruling
document and this is not the executor's call. **Build orbit, and keep the projection behind one
function** so switching is a small change rather than a rewrite.

## Q6 — window anatomy: Appendix A3, or the design bundle's?

**Built:** A3, throughout. Sections 1, 2 and 7 are built to it and section 7 closed twelve defects
against it.

**The disagreement:** A3 gives a 260 px tree, a 320 px contextual properties panel, and a 28 px
status bar. The bundle gives a 52 px tool rail, a 364 px cost panel, a document bar and a tool
settings strip. These are not variants of one anatomy — the bundle has no tree and no properties
panel; A3 has no tool rail.

**Three things in the bundle are additive rather than competing, and are worth having whichever
anatomy wins:**

- **The tool settings strip** — everything the active tool needs and nothing else, swapping wholesale
  with the tool. This is "spare surface, depth on demand" made concrete.
- **The live status readout under the cursor** — slope, thickness at that point, flow direction,
  distance to the drain water reaches. That is the water-migration view expressed as text, and it
  costs no 3D at all.
- **One roof-area figure in the document bar that everything reconciles to.**

**Why it waits:** rebuilding the chrome on a reading of a reference bundle would discard a completed
section. The bundle is marked reference, not exact.

## Q4 — does the tapered heightfield produce an estimate measure, or only a picture?

**Built:** nothing yet — section 6 has not opened. When it does, the heightfield is **view-only**
until this is answered, which is what Addendum 4 §5 instructs.

**The question, in Addendum 4's own words:** §4.10 makes 3D derived and rendered. Whether the
tapered heightfield also produces an estimate measure on the tapered condition — an average
thickness, or a thickness schedule — is Patrick's ruling.

**Why it matters more than it looks.** Tapered insulation is bought by the board and by the
thickness, so a heightfield that knows the depth at every point already holds the quantity. If it
feeds the estimate, the heightfield stops being a view and becomes a measure, and every rule that
governs a measure applies to it: it appears on the condition, a formula can use it by name, and it
is pending rather than zero when the inputs are not there. That is a different piece of work from
drawing slopes, and building the drawing first and retrofitting the measure is the expensive order.

**Why it waits:** view-only is the smaller boundary and the one Addendum 4 names. If the ruling
comes back "it feeds the estimate", the heightfield is already computed and the measure is added to
it; nothing is thrown away. Guessing the other way would put a number on an estimate line that
nobody asked for, which is the named defect this project exists to escape.

**Evidence found since, and it points one way (2026-09-08).** Two prior framings answered this
question before it was asked, and neither said view-only:

- An earlier gate ladder had tapered thickness and cricket quantities as **derived measures**, with
  a pass criterion that a solved iso schedule reproduces a reference tapered system within tolerance
  and that **cricket SF is computed, not entered**.
- The design bundle derives **sump depth** from thickness at drain, slope, sump width and board
  count, and shows it as a worked line beside the four tapered estimate lines a pick changed.

Against that: view-only makes tapered the one condition kind whose geometry cannot pay for itself.
**The ruling is still Patrick's and the boundary holds until he gives it** — but section 6 should
build the heightfield behind a seam that a measure can be attached to without restructuring, because
the evidence says that seam will be opened.

## Q3 — the Estimate Sheet's columns: v2 §0 and A4 disagree

**Built:** the formula stays visible on every line, as it is today.

**The disagreement, inside the specification rather than between it and me:**

- v2 §0 lists what the screenshots show that **stays**, and the first item is "the formula
  visible on the line".
- A4 gives the sheet the columns Condition · Item · Qty · Unit · Material · Labor · Total,
  with Formula revealed only when a line is expanded.

Both cannot hold. A4's column set has no Formula column at all, so adopting it hides on
expansion the one thing v2 §0 says must not be lost — and the thing the handoff itself calls
the reason the program exists ("a library you cannot see into is a library nobody trusts").

**Why it waits:** rebuilding the sheet's columns on a reading of an ambiguity would be the
expensive kind of wrong. The current sheet already does the harder half of A4 — the whole
chain is on the line rather than hidden — so the change, if A4 is meant literally, is to
*remove* columns and put them behind an expander. That is an afternoon either way, and it
should be Patrick's word rather than my guess.

**Also still open from A4, and smaller:** rows grouped by condition are not yet collapsible,
and Material/Labor are not yet split per line. Both are additive to what is there.

## Q2 — Visual Specification v1 is not on this machine — CLOSED by Appendix A

**Built:** the twelve defects in v2 §1, which v2 specifies completely on their own.

**The gap:** v2 §2 carries forward v1 §3 (window anatomy), §4 (editor anatomy and the start
screen), §5 (type), §6 (color), §7 (density), §8 (help), §9 (icons) and §10 (the vocabulary
check) "unchanged" — and `refs/` holds only the proposal, the handoff and addenda 1–3. v2 §1
leans on those sections by name: the twelve-hue palette, the start screen, Lucide icons at
16 px, "fixed anatomy".

**What was decided in the meantime**, all recorded in `DECISIONS.md` and all cheap to
replace when v1 arrives: a twelve-hue condition palette, a type and density scale, the help
affordance, an icon set drawn inline rather than fetched, and the vocabulary check. Where v2
§1 is specific it was followed to the letter; where it referred to v1 the call is mine and
is marked as such.

**Closed 2026-09-07.** Appendix A restored those sections and is now their only source. Five
interim calls met it and were kept with their mapping recorded; eight were overruled and the
section applied. The full reconciliation is at the end of `DECISIONS.md`.

The one thing A8 asks for that is not yet done: help text as markdown files in the repo, one
per editor, so a roofer can correct it. It is currently strings in `help.ts`. Noted rather
than quietly dropped.

## Closed

### Q5 — light or dark — CLOSED 2026-09-08, and it was never a conflict

**Answer: it is a toggle.** Patrick's words: *"it's a toggle like every major construction software
is."* Light stays the default face; dark is an option the estimator picks.

**Why it looked like a collision and was not.** The design bundle is dark throughout — a palette
reasoned from the material world, membrane charcoal through galvanized greys — while visual
specification v2 §1.1 puts a light neutral desk behind the paper, and sections 1, 2 and 7 are built
and restyled to that. But v2 §2 already said, in its own words: *"Dark theme can return later as an
option; it is not the product's face."* v2 set the default and deferred the option. The bundle
supplies the option's values. They were answering different halves of one question.

**So nothing built is wrong and nothing needs restyling.** What it needs is the colours lifted out of
hard-coded values into a token set with two themes, which is a legibility-pass job touching every
editor — not section 6's work, and not urgent. Recorded so the next seat does not read the bundle as
a demand to rebuild.

### Q8 — are the two master templates rival authorities? — CLOSED 2026-09-08, they are not

**Answer: they are complementary, and the question dissolves.** Checked directly:

- `~/KNOWLEDGE/PRICING/EDGE/EDGE MASTER TEMPLATE.xlsx` — **one sheet**, ~2,190 rows, columns
  `Description · Estimated Unit · Cost Type · User Code · Order Formula · Round Order Quantity ·
  Order Unit · Price Formula · Price Unit · Unit Price · Waste Percent · Production Rate · …`.
  A flat **item price book**. This is the library's column shape.
- `~/BUSINESS/WATERTIGHT/TEMPLATES/MASTER_ROOFING_ESTIMATE_TEMPLATE.xlsx` — **six sheets**,
  START · TAKEOFF · MATERIAL · LABOR · EQUIP+MISC · RECAP, a working **estimate workbook** whose
  flow is takeoff into material and labor into a recap.

One is a catalogue of items; the other is the structure that prices a job — and that structure is
what section 3 already built and verified against two real recaps. No ruling is needed because they
never competed. Both are named in Addendum 5's source list with their roles attached, so the
question cannot be re-asked.

### Q1 — what `EA` counts on an area or a run — CLOSED 2026-09-07

**Answer:** `EA` is the number of vertices in the trace, and an arc contributes none.

It was settled by reading real drawing reports rather than by guessing. Four patterns in
them decide it, and none of them needs a number repeating here:

- A closed rectangle traced a dozen times counts four corners each time, and the report's
  `EA` is exactly four times the number of rectangles. So `EA` is not the count of traced
  shapes — it was never one-per-shape.
- A single radial flashing — one arc — carries a real run in `LF` and **zero** `EA`. A
  curve has length but nothing on it gets mitred.
- The same building perimeter traced twice, once roughly and once in detail, reports a
  different `EA` each time while the `LF` barely moves. `EA` follows how many points were
  clicked, not how big the thing is.
- A long straight parapet reports a small `EA`; a short broken one reports a larger `EA`.

So `VERTICES` and `SEGMENTS` are separate measures a formula can use by name, and `EA` on a
line or an area is `VERTICES`. Mitres and corner pieces are bought against `VERTICES`;
pieces between corners against `SEGMENTS`.

Splitting inside from outside corners by signed angle is left until something asks for it.
The section 3 fixture comparison confirms the counts against the real reports, which stay
in `fixtures/` where they belong.
