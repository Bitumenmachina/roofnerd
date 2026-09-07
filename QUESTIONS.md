# Open questions

Things where a wrong call would waste work. Each one names the smallest version built so
the rest could proceed. Not a stop — the build continues past every entry here.

_(none open)_

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
