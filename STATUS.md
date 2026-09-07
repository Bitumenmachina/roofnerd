# Status

Current as of 2026-09-07. One file, short. Patrick accepts against this whenever he chooses;
acceptance does not pause the build.

## Where the build is

**Sections 1, 2, 3 and 7 are done, and the front door now works.** Section 4 — the library —
is next, and starts now that the front-door probe passes.

## The one that mattered: the program could not be opened

Patrick opened the built application and could not load anything. He was right, and every
check I had was green.

*File → Open a job* called `window.prompt`. This webview does not implement it: it returns
nothing, the handler treats that as "cancelled", and no job opens. Setting a scale by two
points had the same defect. Every probe passed because every probe called `doc_open` through
the bridge — **not one of them ever touched the door a person uses.** "8/8" was true and
useless in the same breath.

Fixed, and fixed so it cannot come back:

- Every `prompt`, `alert` and `confirm` is gone. Opening uses the dialog plugin's folder
  picker; the scale is a field under the drawing where the instruction already is; naming a
  property is a field in the panel. The egress gate refuses all three, proven red on each.
- **A check that bypasses the path a person uses is not a check.** It is a rule in
  `CLAUDE.md` now. No probe calls `doc_open`; they go in through the start screen with
  `openDemoJob()`, and `tools/probe/front-door.mjs` walks the whole path — start screen,
  button, drawing, scale field, tear-off, second window.
- The start screen offers the demo job, and the demo job now opens to a roof with work on it.

**And one thing the handover itself turned up.** The first drawing opened through the
program was a scan from a real office, and *Add a drawing* copies what it is given into the
job's folder — which for the demo job is inside this repository. It was caught before it was
staged, the copy is gone, the original is untouched, and drawings inside any job folder are
gitignored now, with only the demo's own synthetic sheet tracked. Proven by dropping a file
in and watching git refuse it.

Section 7 was the restyle, against Visual Specification v2. All twelve defects it names are
closed, with a before/after pair from the Tauri window for each.

**Appendix A then restored the sections v2 pointed at, and the reconciliation is done.** Five
interim calls met it and were kept with their mapping written down; eight were overruled and
the section applied — type scale, density and corners, panel widths, the status bar's
contents, no second chrome in a torn-off window, an editor picker per area, the scale badge
and visibility toggle, and a generated price set. `QUESTIONS.md` Q2 is closed; the full
mapping is at the end of `DECISIONS.md`.

**Two things from the appendix are deliberately not done, and are not oversights:**

- **A4's Estimate Sheet columns contradict v2 §0.** §0 says the formula visible on the line
  must not be lost; A4's column set has no Formula column and reveals it on expansion. Both
  cannot hold, and guessing would be the expensive kind of wrong. `QUESTIONS.md` Q3 sets out
  the choice; the sheet is unchanged meanwhile.
- **A8 wants help as markdown files in the repo**, one per editor, so a roofer can correct
  it. It is still strings in `help.ts`. Noted rather than quietly dropped.

Section 3's acceptance, in two halves, both passing:

- **The ladder** reproduces both real recaps from their class subtotals — every line within
  one hundredth of one percent, worst difference two cents on a three-million-dollar
  contract amount.
- **The class subtotals themselves** are now rebuilt from the item lines underneath them,
  priced through the engine rather than read off the report's own total column. Both jobs:
  every cost code and every class within tolerance.

Section 0 (the skeleton) was built and pushed earlier the same day.

## What section 7 changed

Every defect in v2 §1, in the order it lists them:

| | Defect | Now |
|---|---|---|
| 1 | Dark warm-grey desk dominating every plan view | Light neutral desk; the paper has a hairline border and a 1 px shadow |
| 2 | "Pages" listing conditions, no hierarchy | A real tree: Job → Page → Condition → Item. Selecting a node selects it in every editor |
| 3 | Probe controls in the product chrome | File menu and a start screen; the path in the status bar; tear-off on every area's own header |
| 4 | A brown active tool and unrelated trace colours | One accent — mid blue — for active tool, selection and focus. Condition colours from a twelve-hue palette, never for interface state |
| 5 | Two rows of text buttons, "Set scale" and "Rescale" | One row, 16 px icons with their words, one **Scale** action whose label reads Rescale by state, zoom grouped right |
| 6 | `PLAN_SF`, `SIDES 4`, `trace(s)`, red `pending` | Trade words throughout: "4:12 pitch", "1.5 ft high", "1 trace". A measure with no number is an em dash with the reason in its tooltip |
| 7 | Rail heading clipped under the panel | One scroll region: list above, selected condition below, hairline between |
| 8 | Eight equal fields, live measures smallest and last | Measures first and large in tabular figures; properties grouped under Geometry and Metal; hints unchanged |
| 9 | Truncated headers, actions clipping off the edge, a bare "%" | Fixed column widths, horizontal scroll inside the area, actions pinned to a column that never scrolls out, waste reads 0 |
| 10 | An explanatory paragraph on every open | The editor's help page, behind the "?" in its header |
| 11 | No row separators, groups marked only by bold | Hairline separators, tinted condition rows, monospace formulas, tabular numbers, bold extended |
| 12 | "Total" alone | Class subtotals and Selling Price pinned in the footer |

**Two product bugs the restyle surfaced**, both found by the probes rather than by eye:

- Three quick clicks with the Count tool started **three separate conditions**. Each click
  read the job before the one before it had finished writing, saw nothing selected that took
  a count, and made its own. Traces are recorded one at a time now.
- A run was showing "— SF", which suggests it might have a surface if you scaled the sheet.
  It has none by design; each kind now shows the measures it actually has.

## What section 3 built

The money model, and the check that it is the right one.

- **Three units on every item**, not two: what it is estimated in, what it is bought in, and
  what it is priced against. Membrane is estimated in squares, bought by the roll and quoted
  by the square foot. Each step has its own conversion and its own rounding, and both are on
  the line where they can be seen.
- **The rounding rule belongs to the item and the step.** Rolls and sheets round up;
  fastener plates order at a fraction, because that is what the supplier bills. The program
  does not decide this on an estimator's behalf.
- **Cost codes, classes and the adder ladder.** Each adder sits on its class subtotal and
  they are summed — tax and escalation side by side on material, never compounding.
  Supervision carries its own burden rate. Then profit, then contract amount, then bond,
  then selling price. Cost per square on every class.
- **Labor is not a special case.** A labor line measures in its own unit and *orders* in
  hours, with the production rate as the conversion and no package rounding. Fabrication and
  installation are separate lines with separate rates. Crew days come from hours.
- **Hours keep every digit until they are displayed.** A line shows fewer digits than it
  holds; summing what is shown is how a labor total ends up dollars adrift.
- **A comparison tool** that takes a recap printed by the program this one replaces and
  reproduces it — deriving the profit rate from the printed dollars rather than trusting the
  printed rate. A report prints its rate to two places and its dollars to the cent, and on
  one of the two jobs two places is not enough to reproduce the dollars. The dollars are
  what happened.

## The fixture comparison — section 3's acceptance

Both jobs, every line, within one hundredth of one percent:

```
Line                Result
Material total      within 0.01%
Labor total         within 0.01%
Sub total           within 0.01%
Equipment total     within 0.01%
Other total         within 0.01%
Supervision total   within 0.01%
Job cost            within 0.01%
Profit              within 0.01%
Contract amount     within 0.01%
Bond                within 0.01%
Selling price       within 0.01%
Total SQ            within 0.01%
Total hours         within 0.01%

13 of 13, on each of the two jobs.
```

The figures behind those rows are a client's and stay in `fixtures/`, which is not in this
repository. `node tools/compare-fixtures.mjs` prints the full table on the estimator's own
machine; `--shape` prints exactly what is above.

### The subtotals, rebuilt from the lines (Addendum 3 §1)

Both jobs' consolidated reports were parsed into item lines — description, cost code,
estimating quantity and unit, order quantity and unit, price unit and unit price — and every
line was priced **through the engine**, not read off the report's own net-cost column.

```
                                    job one     job two
item lines                              107         100
  chain fully stated by the report       99          96
  conversion recovered from the money     8           4
  money the engine did not reproduce      0           0
cost codes within tolerance         13 / 13     15 / 15
classes within tolerance              6 / 6       6 / 6
```

Two things that reading is careful about:

- A line whose price unit differs from its order unit — membrane bought by the roll and
  quoted by the square foot — does not have its conversion printed anywhere in the report.
  That conversion is recovered from the money, which means the line cannot also be evidence
  that the money is right. Those lines are counted separately and never allowed to look like
  a pass.
- The per-line tolerance is **derived, not flat**. A report displays an order quantity to two
  decimal places and computed the money on the unrounded one, so feeding the displayed figure
  back in can be out by up to half a hundredth of a unit's price. That is Addendum 2 §3's rule
  about hours — and it turns out not to be only hours: it applies to boxes and cartons too.

The entered lines are the library's seed for section 4. Prices become invented values before
any of it leaves `fixtures/`; product names are public and may stay.

## Addendum 1, applied

- **§1 `EA` on a line or an area is the vertex count**, and an arc contributes none. Read
  off the Edge drawing reports rather than guessed: twelve rectangular curbs read 48 EA, and
  a radial counter flashing reads 0 EA against 60 LF. `VERTICES` and `SEGMENTS` are now
  separate measures a formula can use by name. This closes the only entry in `QUESTIONS.md`.
- **§2 A section is no longer done on Chrome.** `tools/probe/runtime-check.mjs` drives the
  release binary through `tauri-driver`: WebKitGTK rendering, the Rust shell holding the
  document, two real OS windows. **PASS 8/8**, including that a change made in the Plan
  window reaches the torn-off Estimate window.
- **§3 The security policy is now tested, not asserted.** In the shipped runtime,
  `fetch('https://example.com')` is refused by policy and `fetch('/index.html')` returns 200.
  Written to `evidence/csp-<commit>.txt`. It had been deferred twice; it is a done-check now.
- **§3 The release bundle builds** — `.deb`, `.rpm` and AppImage, via `pnpm build:app`.
- **§4 Evidence carries its section and its commit** in the filename, and the runtime
  screenshots come from the Tauri window.
- **§5 The Edge reports were read before section 3 opened**, not during. Both recap targets
  and the labor model are now understood: a labor line measures in its own unit and *orders*
  in HOURS, with the production rate as the conversion and the price per hour — so hours are
  an order unit like any other, and they are not rounded up to a package. The figures stay in
  `fixtures/`.
- **§7 The rounding rule is centralised** in `packages/engine/src/rounding.ts` and applied at
  every step that rounds — the order-unit step and `ceil`/`floor`/`round` typed into a
  formula — not only where the defect was found.

## What section 1 built

A drawing goes on screen and gets traced, and what comes off it is square feet, linear feet
and a count at the same time.

- **Drawings.** PDF pages and images (PNG, JPG, WEBP). The estimator picks a file; it is
  copied into the job folder and the job stores the path. The drawing is never copied into
  the job file — a job stays small, readable and greppable.
- **Scale.** Two points and a real dimension, typed the way a drawing writes it — `12'-6"`,
  `6"`, `4'-6 1/2"`, `12.5` all read the same. Or an architectural preset, 1/8" through 1".
  A sheet nobody has scaled says *unscaled*, and everything measured on it reads as pending
  rather than as zero.
- **Tools.** Area, line, count. Click to place, Enter or double-click to finish, Escape to
  drop it, Backspace to take back a corner, Shift to square up. Corners snap to corners
  already on the sheet, which is what stops a roof plan leaking area at every junction.
- **The conditions list**, live: `216.67 SF · 57.73 LF · 4 EA` off one traced area, with its
  pitch shown beside it.
- **Pitch does the work it is supposed to.** `PLAN_SF` is the footprint; `SF` is the sloped
  surface. A 12:12 roof is 41.4% more material than the plan says.
- **The formula language** an item line will carry: parsed to a tree and walked, never
  `eval`. `LF * H`, `ceil(LF * STRETCHOUT / 12 / 30)`, `LF * 2 / 0.5`. A formula that is
  wrong says what is wrong and where. A pending measure makes the answer pending — never
  zero, because a zero prices as a finished line for nothing.
- **`from:` inheritance.** A coping takes the parapet's measures instead of being traced
  twice, keeps its own properties, and the two can never drift. Circles are refused.

## What section 2 built

The other half of the working loop, and the thing the whole program exists for: trace on
one screen, watch the money move on the other.

- **The condition panel** — a name, where its measures come from, and the six properties a
  roofing detail keeps asking for: height, width, thickness, pitch, sides, stretch-out. Any
  other number the estimator needs, they name themselves, and every formula on that
  condition can use it by that name. The panel also shows what the condition measures, so
  the numbers a formula will see are visible beside the formula.
- **The estimate sheet** — a line per item, grouped under the condition it comes off. The
  formula is printed on the line and edited there. Quantity, unit, waste, order units, unit
  cost, extended. A line with no price says *no price*; a line off an unscaled sheet says
  *pending*. Neither is ever counted as zero.
- **A bad formula says what is wrong and where**, on the line, and shows no quantity rather
  than a plausible one.
- **One trace, several units.** A parapet traced once feeds wall flashing in SF, coping in
  LF and corners in EA, and all three move together.

## What the checks say

| | |
|---|---|
| Section 1 done-check (`tools/probe/section1-trace.mjs`) | **PASS 12/12** — opens a PDF, scales it, traces an area with pitch, a run and a count, reads the measures off the list, confirms traces are stored in page units and the drawing is only referenced |
| Section 2 done-check (`tools/probe/section2-sheet.mjs`) | **PASS 16/16** — traces a parapet, types a height in the panel, tears the sheet into a second window, adds three items in three units, then changes the drawing and the height and watches the money move in the OTHER window. Two windows, one job, throughout |
| Front door (`tools/probe/front-door.mjs`) | **PASS 17/17** — start screen → button → job open → drawing → scale typed into a field → tear-off → second window. Never once through the bridge |
| Real-runtime check (`tools/probe/runtime-check.mjs`) | **PASS 9/9** — opens through the front door, then two OS windows on one document, a property typed in one moving the money in the other, and the CSP refusing the network |
| Fixture ladder (`tools/compare-fixtures.mjs`) | **PASS 26/26** — both real recaps, every line within 0.01% |
| Subtotals rebuilt (`tools/rebuild-subtotals.mjs`) | **PASS** — 207 item lines priced through the engine; 28/28 cost codes, 12/12 classes |
| Client-data gate | **PASS**, and proven red on a figure, a bid name and a home path |
| Vocabulary check (`tools/probe/vocabulary-check.mjs`) | **PASS 23/23** — reads what is rendered in the shipped window, not the source; proven red on `PLAN_SF` |
| Synthetic prices (`tools/make-demo-prices.mjs`) | 18 seeded invented prices in `jobs/demo-job/prices.json`; every screenshot and probe draws from them |
| Engine tests | **95/95** |
| Shell tests | **5/5**, including that a job saved by the application is byte-for-byte one saved by the command line |
| Vocabulary gate | **PASS** |
| Egress gate | **PASS** |
| Demo job round-trip | clean, 4 files |

Screenshot in `evidence/` (local only).

## A rule I broke, and what was done about it

The handoff bars client data from the public tree. While building sections 1 and 2 I used
**real quantities out of the Edge reports as test values** — a run length here, a stretch-out
case there — because they were in front of me and they made the tests feel real. They are a
client's bid figures and they had no business in a public repository.

Every one is now replaced with an invented number, and the reasoning that needed those
reports is written down without repeating any of them. `QUESTIONS.md` explains the vertex
rule through the *patterns* in the reports and quotes no quantity at all.

**Two commits already pushed still contain them** — `fffc4f5` and `4ba8909`, in
`QUESTIONS.md` and three test files. Figures with no job name attached, so the exposure is
small, but the rule is the rule. Taking them out of history means rewriting two public
commits, and the executor does not force-push. That is Patrick's call; the working tree is
clean either way.

## Defects found and fixed in flight

- **The document was keyed by file path.** A JSON Pointer splits on `/`, so `pages/pages.json`
  read as two steps and nothing could address it. It was latent in the Rust side too. The
  document is now keyed by part name. Found by the probe, which is what the probe is for.
- **pdf.js was asked to paint one canvas twice at once** — what leaning on the zoom key
  does. Paints are now cancellable and serialized, and the overlay is sized before the
  paint rather than after, so a trace sits in the right place while a big sheet is still
  drawing.
- **A drawing that failed to open said nothing.** It now says what went wrong.
- **Floating-point crumbs bought an extra roll.** `100` with 10% waste is
  `110.00000000000001`, and a bare `ceil` made that twelve rolls instead of eleven. An
  estimator would have found it by counting the pallet. Quantities now settle before they
  meet a rounding step, and a test pins it.
- **The formula column clipped its own contents** — the one thing on the sheet that must
  never be cut off.

## Since the last entry

Twenty-eight decisions in `DECISIONS.md` (D3–D9 section 1, D10–D14 section 2, D15–D20
addendum 1, D21–D28 section 3 and addendum 2). **`QUESTIONS.md` has nothing open.**

## If Patrick wants to look

    cd ~/projects/roofnerd && pnpm dev

Add a drawing, set its scale, pick Line, trace a parapet. Click it in the list and type a
height into the panel. Tear off the Estimate Sheet, put it on the other monitor, and add a
line with the formula `LF * H`. Then go back to the drawing and trace another run of the
same parapet — the money moves while you are still on the first screen.

To run what the executor ran:

    node tools/probe/section1-trace.mjs
    node tools/probe/section2-sheet.mjs
    pnpm build:app && node tools/probe/runtime-check.mjs
    node tools/compare-fixtures.mjs           # needs fixtures/, which is local only
    node tools/rebuild-subtotals.mjs          # likewise
