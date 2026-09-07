# Open questions

Things where a wrong call would waste work. Each one names the smallest version built so
the rest could proceed. Not a stop — the build continues past every entry here.

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
