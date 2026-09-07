# Open questions

Things where a wrong call would waste work. Each one names the smallest version built so
the rest could proceed. Not a stop — the build continues past every entry here.

## Q2 — Visual Specification v1 is not on this machine

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

**Why it can wait:** every one of those is a stylesheet value or a small component. None of
them is load-bearing on the document model, and replacing a palette or a type scale is an
afternoon, not a rebuild.

## Closed

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
