# Open questions

Things where a wrong call would waste work. Each one names the smallest version built so
the rest could proceed. Not a stop — the build continues past every entry here.

## Q1 — what `EA` counts on an area or a run

**Built:** `EA` is the number of corners (vertices). A count condition's `EA` is its markers,
which is not in doubt.

**The doubt:** an Edge report shows `Parapet Wall Flashing A3/1A551 — 1,354.93 LF, 5 EA`.
Five corners on 1,355 feet of parapet is implausible; five traced *runs* is not. So Edge's
`EA` on a line may be the number of separate shapes, not the number of vertices.

**Why it can wait:** the fixture comparison in section 3 runs both jobs' real numbers against
this program's, and a mismatch on a line condition's `EA` will show up there against a real
report rather than being guessed at now. If it turns out to mean runs, it is a one-line
change in `measures.ts` and a test.
