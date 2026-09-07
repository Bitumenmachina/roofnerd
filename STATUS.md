# Status

Current as of 2026-09-07. One file, short. Patrick accepts against this whenever he chooses;
acceptance does not pause the build.

## Where the build is

**Sections 1 and 2 are done, and Addendum 1 is in.** Section 3 — the money model: cost
codes, classes, adders, labor shapes, scenarios, waste, order units, the recap — is next,
and its done-check will run against the shipped runtime rather than against Chrome.

Section 0 (the skeleton) was built and pushed earlier the same day.

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
| Real-runtime check (`tools/probe/runtime-check.mjs`) | **PASS 8/8** — the release binary under `tauri-driver`: two OS windows on one document, a change in one reaching the other, and the CSP refusing the network |
| Engine tests | **68/68** |
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

Twenty decisions in `DECISIONS.md` (D3–D9 section 1, D10–D14 section 2, D15–D20 the
addendum). **`QUESTIONS.md` has nothing open** — Q1 was answered from the Edge reports and
the answer is recorded there with the four rows that settle it.

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
