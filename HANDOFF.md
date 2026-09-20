# Handoff — the state of roofnerd, in full

Written 2026-09-20 from a fresh clone of `main` at `2bd6894` (2026-09-13 17:02 -0400). This is a
reading of the repository for a session that has never seen it. It rules nothing: `CLAUDE.md`
governs the work, `STATUS.md` is the build log, `DECISIONS.md` says why, `QUESTIONS.md` says what
is open, `README.md` is the vocabulary. Where this file and those disagree, those win.

Read this file, then `CLAUDE.md`, then the "Where the build is" head of `STATUS.md`. Everything
else is reference.

---

## 1. The repository, as facts

| | |
|---|---|
| Remote | `https://github.com/Bitumenmachina/roofnerd` |
| Default branch | `main`, the only branch on the remote; head `2bd6894` |
| Open pull requests / issues | none, none |
| Licence | MIT, "roofnerd contributors" |
| Authors in the reachable log | Patrick Moriarty (44 commits), Bitumenmachina (6) |
| Commit style | one-sentence subjects written as prose, no prefixes, no ticket numbers |
| Pre-commit hook | `.githooks/pre-commit` — `pnpm -r build`, `pnpm test`, then the three gates. Enabled per clone with `git config core.hooksPath .githooks`. It is NOT enabled on a fresh clone |
| Package manager | pnpm 10.33.0 (pinned in `package.json`), Node 20+, Rust 1.77+ (Tauri 2's floor) |

**This clone is shallow.** `git rev-list --count HEAD` is 50 and `.git/shallow` exists. Every
commit visible is dated 2026-09-13. `STATUS.md` refers to earlier hashes (`fffc4f5`, `c893211`,
`7cb2559`, `28b7adf`, and a push of "116 commits") that are not reachable here. Anything that
needs older history needs `git fetch --unshallow`.

**Local-only trees are absent here, by design.** `.gitignore` excludes `refs/` (the ruling
documents: the handoff and Addenda 1–7, the carry-forward register, the proposal, the audit
notes), `fixtures/` (two real bids and `.clientdata-terms`), `evidence/` (every screenshot and
`csp-<commit>.txt`), `work/`, `out/`, `.scratch/` (the persona pass `c8-persona.md` and
`c9-notes.md`). Also referenced and not in the repository at all: `~/KNOWLEDGE` (the manual and
vendor corpus with its own manifest), `~/venvs/ifc` (IfcOpenShell 0.8.4), the plan file
`~/.claude/plans/handoff-is-complete-and-foamy-ember.md`, and the design bundle zip. A session
without these can build and test the program but cannot run the fixture comparison, cannot read
a ruling document, and should ask before assuming what an addendum says (`CLAUDE.md`, "If `refs/`
is empty on this clone, ask").

---

## 2. What the program is

Roofing estimating software for commercial low-slope work. Trace the roof on a PDF or image on
one screen; the estimate moves on the other. A desktop program (Tauri 2, WebKitGTK on Linux)
because the second window has to be a real OS window on a second monitor. No server, no network
call, no accounts. A job is a folder of plain JSON. Exports are CSV and IFC4.

**The ruling architecture (Patrick, 2026-09-12; D146):** the 3D model is the function, not a view.
What has to be built — the tapered package laid out the trade's way, walls and curbs with their
flashing heights, the fastening pattern by wind zone — is solved in `src-tauri/core`
(`roofnerd-core`, Rust). The 2D outputs are produced FROM that solve: the tapered layout drawing,
the fastening plan, the section details, the estimate lines (derived, shown with ◈), the IFC file.
The core owns solvers only: no window, no disk, no network, no money. Money is `packages/engine`,
plain TypeScript arithmetic. `src-tauri/src/lib.rs` holds the one open job behind a lock and hands
the same copy to every window. `packages/app` is the editors.

**The demo job** is `jobs/demo-job`, "Northgate Retail Center — Roof Retrofit": 9,600 SF field,
400 LF parapet at 42 in, three drains, tapered ¼:12, 4 ft sumps, an RTU curb with a cricket behind
it, a General Conditions group. Six conditions, 17 cost codes, two scenarios, a synthetic roof
plan drawn by `tools/make-demo-sheet.mjs`. Every figure in it is invented (D50, D135). At the
current head it bids at **$228,099.66 and 8.30 hours a square** (STATUS, C8).

---

## 3. Where the build is

**Plan v3 is complete** (STATUS, 2026-09-13). The sequence that landed, in order, each with its
D-number:

| Batch | What | Decisions |
|---|---|---|
| Gate 0 | Skeleton: engine, Tauri shell with one document, two windows, demo job, gates | D1–D9 |
| Sections 1–2 | Trace on PDF, scale, area/line/count tools, condition panel, estimate sheet, formula on the line, `from:` inheritance | D3–D14 |
| Section 3 | The money: three units per item, rounding per step, cost codes, classes, adder ladder, recap; verified against two real bids within 0.01% | D21–D32 |
| Section 4 | Library: assemblies, profiles, provenance, firmness (Q9) | D75–D82 |
| Section 5 | Lenses: five reports as column filters, concealed costs absent from the page | — |
| Section 6, 6b | The Model (three.js), roof standing up, taper, cricket, no-fall, panel beside every editor, selection across windows | D57, D71–D74, D83–D86, D121–D130 |
| Section 7 | Legibility restyle against Visual Specification v2 and Appendix A | D34–D56 |
| Section 9a | CSV export into the job's `exports/` folder | D106–D116 |
| Addendum 7 / 10a | The handoff's window anatomy: document bar, tool settings strip, tool rail, one surface (Plan / Model / Section), cost panel with detach, status strip; 1440×900 | D131–D140 |
| 10b | 3D toolkit: @thatopen/components + web-ifc self-hosted, CSP gains `wasm-unsafe-eval` | D141–D145 |
| **C0** | `roofnerd-core` exists; the seam (`doc:changed` + `solve_get`); the cone reproduced bit for bit; window size fix | D146–D148 |
| **C1** | Drainage regions, four-way planes, 45° valleys, plateau; cone retired | D149 |
| **C2** | Boards by letter and fill off Versico's printed table, sumps, cricket on the surface, the bill on the sheet (`TAPER_<letter>_SF` etc.) | D150–D152 |
| **C5** | One `Drawing` model in the core, one renderer; Section view (Frame 3); layout and fastening overlays on the Plan; the look matched to Frame 2 | D155–D156 |
| **C3** | Walls and curbs as true polyline offsets; `FLASH_SF` exact; coping gauge, sections, corners, cleat | D153–D154 |
| **C4** | Wind zones per NRCA Appendix 1, FM class per zone, fasteners and plates board by board, adhesive by zone | D157 |
| **C6** | The verbs: set high and low, box out a curb, measure, cut a section — two picks, the core answers, then the document is written | D158 |
| **C7** | File → Export IFC (IFC4, feet) from the solve; IfcOpenShell verifies; alpha IFCX beside it | D159 |
| **C9** | Labor basis: crew and rate on the line the way a foreman states it, hours derived, kind derived (labor / fabrication / sub), OFF lines struck and kept, Hours/SQ on the recap; tapered lines generated per letter | D161–D162 |
| **C8** | Persona pass: a roofer opens it cold for twenty minutes; five money defects and twelve misleading behaviours fixed; the demo opens on a drawn roof plan | D160 |

**Selling price on the demo, by batch:** $170,517.77 (B1) → $165,328.20 (C2) → $168,956.54 (C3)
→ $212,153.98 (C4, adhesive was missing) → $230,496.85 (C9, labor restated) → $228,099.66 (C8,
corners priced at corners). Every move is explained in its STATUS entry.

**What the last session left as the next session's first act** (STATUS head, verbatim in
intent): build the AppImage in the foreground, open the demo job, and leave it open for Patrick's
review; read the plan file's handoff sections, the C8 entry, `.scratch/c8-persona.md` and
`.scratch/c9-notes.md`; **do not start a batch until Patrick has looked.** Three of those four
reads are not in this clone.

---

## 4. What is owed, open, or parked

**Patrick's open calls** (STATUS head, C2–C9 entries):
- `BOARDS_MAX` = 4 courses against the 3 typed on the demo (D150, D152) — a fourth layer or a
  shallower slope.
- The demo's field and perimeter wind pressures are an Exposure C floor from the brief; the
  building is Exposure D. Retype from the ASCE 7-22 figure when it is in the box (D157).
- The 130 mph adhesive rate uses the 120 mph row; the NEMO evaluation report is not in the box.
- A six-man crew at 8.30 hours a square: is that the crew he would carry (D161).
- Whether the permit belongs in General Conditions or on the field.
- Whether a struck alternate kept on the sheet reads right.
- Whether the two pushed commits carrying real figures (`fffc4f5`, `4ba8909`, plus `7eba043`,
  `f46aca1` per D88) get history rewritten. The executor never force-pushes; that is his call.

**Not in plan v3 and still owed from the handoff:** the Schedule (§4.12, section 8; D103–D104
already say how), Undo (D136), IFC import, the "Start a roof" first-open frame, help as markdown
files per editor (A8), a dark theme token set (D59 rules a toggle; not built), per-condition colour
editing, collapsible condition groups on the sheet, Material/Labor split per line (Q3 tail).

**Parked as taste by the persona pass** (listed, not built): the Condition panel offers every
property to every condition; "9,600.00 SF" where a bid says 9,600 SF; selection floods the whole
field; "Taper" against "Slope"; the help sheet covers the cost panel; 697 board letters illegible
at the opening camera; title says "42 lines" against 45 rows; no recent-jobs list; the 3D cut
pane's size.

**Older seen-not-fixed items still standing** (STATUS, 2026-09-12 list): the cricket is not
legible as two planes at the default angle; a torn-off Model may not fit on open; the scale badge
shows the method rather than the ratio; library rows show `VERTICES → EA`.

**`QUESTIONS.md` has nothing open.** Q1–Q9 are all answered or closed; Q4 was re-answered by
Patrick on 2026-09-12 (the model feeds the estimate), which is what C0–C9 built.

**Recognised and declined** (D68, D77 superseded by C4, D105 superseded by C7, D112): drain
sizing, a supplier cut sheet, bid comparison against a previous job, ponding simulation over time.

---

## 5. The code, inventoried

Line counts are from this clone at `2bd6894`.

### `packages/engine` — the money (4,326 lines TS, 191 tests)

Pure arithmetic, no window, no disk beyond `job.ts`. Exposed as `@roofnerd/engine` and as a
command line, `packages/engine/bin/roofnerd.mjs` (`recap`, `check`, `normalize`; run
`node packages/engine/bin/roofnerd.mjs recap jobs/demo-job`).

| File | Lines | What |
|---|---|---|
| `model.ts` | 701 | The vocabulary as types: Job, Scenario, Page, Condition, Item, Assembly, Class, `LAYER_FUNCTIONS` (deck up) |
| `solved.ts` | 939 | The one TypeScript copy of the solved-model types the core emits (D148); `readTaperAt`, `readFacetAt` |
| `recap.ts` | 479 | Classes, adders on the subtotal (never compounding, D26), overhead, profit, bond, Hours/SQ, "not in this total" |
| `line.ts` | 368 | Pricing one line: formula → quantity → waste → order step → price step; `basis`, `off`, crew and rate |
| `measures.ts` | 297 | SF, LF, EA, SQ, PLAN_SF, VERTICES, SEGMENTS off a trace; pending, never zero |
| `roof.ts` | 297 | Trade conventions as tested arithmetic (board module, sump rule, cricket slope, flashing minimum); the core's tests read this file |
| `formula.ts` | 271 | The formula language: parsed to a tree and walked, never `eval`; names, `ceil`/`floor`/`round` |
| `geometry.ts` | 264 | Polygon area, pitch factor, self-intersection, page bounds |
| `compare.ts` | 163 | Reproducing a printed recap (fixture ladder) |
| `lens.ts` | 135 | The five lenses as column filters |
| `dimension.ts` | 98 | Does the unit follow from the formula ("check the unit") |
| `job.ts` | 99 | The job folder: `JOB_PARTS`, canonical sorted-key JSON |
| `takeoff.ts` | 82 | Measuring a whole job |
| `units.ts`, `rounding.ts`, `words.ts`, `index.ts` | 63 / 34 / 14 / 22 | Units, `SETTLE_PLACES` (D13, D16), `plural()`, exports |

Tests: `money.test.mjs` 50, `takeoff.test.mjs` 63, `line.test.mjs` 22, `roof.test.mjs` 19,
`engine.test.mjs` 13, `geometry.test.mjs` 11, `vendor-data.test.mjs` 8, `by-letter.test.mjs` 5.

### `packages/app` — the editors (12,165 lines TS, 1,454 CSS, 41 tests)

Vanilla TypeScript and Vite. No UI framework (D-Gate 0). Dependencies: `@tauri-apps/api` 2.9,
`@tauri-apps/plugin-dialog`, `pdfjs-dist` 6.3, `three` 0.185, `@thatopen/components` 3.4.8,
`@thatopen/components-front`, `@thatopen/fragments`, `camera-controls`, `web-ifc` 0.0.77. The
wasm and the fragments worker are copied out of the installed packages at build time by
`vite.config.ts` (D142); nothing is fetched.

| File | Lines | What |
|---|---|---|
| `editors/model.ts` | 2,683 | The Model: fixed isometric with preset corners, fit, vertical exaggeration; draws facets, walls, boards, crickets, words on the roof from `solve_get` |
| `ifc/write.ts` | 1,134 | The solved roof as IFC4 (`roofPlan`, `expectedOf`) |
| `editors/estimate.ts` | 919 | The Estimate tab: line per item, formula editable on the line, ◈ derived, struck OFF lines, generated per-letter lines |
| `editors/plan.ts` | 847 | The Plan: PDF under an SVG trace surface; layout and fastening overlays |
| `verbs.ts` | 727 | The four acts fed by Plan and Model: set high and low, box out a curb, measure, cut a section |
| `chrome.ts` | 600 | The window anatomy: document bar, tool settings strip, tool rail, surface, cost panel, status strip |
| `main.ts` | 578 | Booting a window; start screen; File menu; window label decides torn-off (D74) |
| `editors/reports.ts` | 511 | The lenses, Copy as CSV, Export CSV |
| `editors/condition-panel.ts` | 450 | Measures first, then properties by group (Geometry, Metal, Wind); ◈ names the solve carries |
| `cost-panel.ts` | 420 | Estimate / Condition / Recap tabs; detach to its own window |
| `viewer/drawing.ts` | 349 | The one renderer for the core's `Drawing` (layout, fastening plan, sections) |
| `viewer/tools.ts`, `viewer/surface.ts` | 337 / 323 | Area, line, count; snapping; Enter / Escape / Backspace / Shift |
| `editors/section.ts` | 305 | Section view: clipped three.js pane plus the drawing at an architect's scale |
| `doc.ts` | 253 | The document as a window sees it: `doc_get`, `doc_set`, subscribe to `doc:changed`, pull `solve_get` when the solve id moves |
| `editors/library.ts` | 200 | The Library |
| `labels.ts`, `selection.ts`, `help.ts`, `caret.ts`, `theme.ts`, `icons.ts`, `cut.ts`, `scale.ts` | 193 / 138 / 133 / 86 / 85 / 80 / 70 / 34 | Words on screen, cross-window selection (D124), help behind "?", focus survives rebuild (D139), theme, inline Lucide, the cut on screen, feet from page units |
| `ifc/triangulate.ts`, `ifc/ifcx.ts`, `ifc/export.ts`, `ifc/profile-data.ts`, `ifc/runtime.ts` | 169 / 141 / 65 / 61 / 38 | Ear clipping with holes, the IFCX alpha, File → Export IFC, the chart's printed edges, booting web-ifc |

Tests (`test/`): `ifc-write`, `ifc-ifcx`, `ifc-triangulate`, `ifc-profile-data`, `typescript`
(a `tsc --noEmit` pass, D119); `ifc-fixture.mjs` hands the writer a made-up solve of `solved.ts`'s
shape.

Shell commands the app calls: `doc_get`, `doc_set`, `doc_open`, `doc_save`, `doc_folder`,
`demo_folder`, `solve_get` (six call sites), `add_page_source`, `read_page_source`,
`export_write`, `open_editor`, `fit_window`.

### `src-tauri` — the shell (861 lines Rust) and `core` — the solvers (8,816 lines Rust, 122 tests)

`src-tauri/src/lib.rs`: the one document behind a lock, keyed by part name (D4). `doc_set` and
`doc_open` mutate under the lock, solve the whole model in the core, clone, drop the guard, then
emit `doc:changed` with `{document, solveId, scopes}` (the per-condition map of named numbers).
`solve_get(part)` answers `taper`, `walls`, `fastening`, `drawing`, and `ask <name> <json>` for the
verbs (D158). `read_page_source` and `export_write` are confined to the open job (D8, D114).
Windows are named after their editor; the capabilities file lists `main, plan, estimate, model,
recap, library, reports, cost` and grants `core:default`, window focus and close, webview create,
`dialog:allow-open` — nothing else. CSP: `default-src 'self'; connect-src 'self' ipc:
http://ipc.localhost; worker-src 'self' blob:; img-src 'self' data: blob:; style-src 'self';
script-src 'self' 'wasm-unsafe-eval'; object-src 'none'`. Sets
`WEBKIT_DISABLE_DMABUF_RENDERER=1` when `/dev/nvidiactl` exists (NOTES.md).

`src-tauri/core/src` (`roofnerd-core`, deps `geo` 0.29.3, `cavalier_contours` 0.4.0, serde):

| File | Lines | What |
|---|---|---|
| `drawing.rs` | 1,487 | The `Drawing` model in feet: tapered layout, fastening plan, one section per drain; `section_at`, `wall_at` |
| `taper.rs` | 1,075 | Drainage regions by perpendicular bisector, four planes per region, valleys at 45°, plateau, `depth_inches_at`, `folds_along` (D149) |
| `verbs.rs` | 756 | What two picks would mean: `high_and_low` (slope rounded to 1/16), `box_out`, `measure`, `cut_at` |
| `fastening.rs` | 749 | Pressures → working load → ×2 → FM rating; pattern by board size and rating from DR-05; plates, adhesive sets, bonding gallons (D157) |
| `geom.rs` | 705 | Polygons in feet: `on_or_in` with boundary fallback, `distance_to_segment`, offsets, clipping |
| `model.rs` | 600 | The types on the wire (mirrors `solved.ts`) |
| `lib.rs` | 529 | `solve(document)`: runs the solvers, assembles the scopes |
| `wind.rs` | 527 | Zone width `a`, corner regions, perimeter band, field; `WIND_STANDARD` |
| `cricket.rs` | 522 | Crickets behind curbs, two-way saddles, Fig 4-13 length-to-width check |
| `profile.rs` | 428 | The manufacturer's chart compiled in with `include_str!` from `data/profiles/` |
| `walls.rs` | 423 | Wall and curb solids by true offset; flashing ceiling; `FLASH_SF` as exact trapezoids (D153) |
| `scope.rs` | 338 | The named numbers a condition carries (`TAPER_X_SF`, `FLASH_SF`, `COPING_PCS_EA`, `FASTENERS_Z1_EA`, …) |
| `coping.rs` | 320 | Cap width, gauge off face height, sections, corners inside/outside, cleat (D154) |
| `boards.rs` | 239 | The 4 ft grid anchored at the drain's cell middle (D150) |
| `sump.rs` | 118 | 4 ft or 8 ft sump off thickness at the drain |

Tests: 65 unit tests in `lib.rs` and the modules, plus `tests/`: `nrca_figures.rs` 7 (holds the
solver to NRCA Figs 4-9, 4-10, 4-11, 4-15 and reads `roof.ts` at test time), `the_bill.rs` 10,
`the_fastening.rs` 10, `the_verbs.rs` 8, `the_zones.rs` 8, `the_drawings.rs` 7, `the_walls.rs` 7.
The shell has 14 tests of its own (STATUS), which need GTK to compile.

### `data/` — vendor data the solvers read

`profiles/versico-versicore-mp-h.json` (Versico VE-7830 tapered polyiso bulletin, 2019-12-16,
fetched 2026-09-12) and `fastening/dr-05-2024-07.json` (Carlisle DR-05 insulation fastening
patterns, 2024-07). Transcribed from page images, product names only, shape-checked by
`vendor-data.test.mjs`. `data/README.md` says plainly: this data is not authority.

### `tools/` — checks and generators (2,031 lines) and `tools/probe/` (12,516 lines)

Gates: `gate-language.sh` (governance words, OBS/INF/STD markers, programmer's plural in
product source), `gate-egress.sh` (fetch/XHR/WebSocket/sendBeacon, prompt/alert/confirm, the two
CDNs, multi-thread wasm, the CSP shape, no HTTP plugin), `gate-clientdata.sh` (numbers and terms
from `fixtures/` and `refs/` refused in the public tree; passes trivially where those trees are
absent). Generators: `make-demo-prices.mjs` (779 lines, every demo price and line), `make-demo-sheet.mjs`
(the roof plan via ImageMagick MVG). Fixture tools needing `fixtures/`: `compare-fixtures.mjs`,
`rebuild-subtotals.mjs`, `parse-consolidated.mjs`. `verify-ifc.py` needs IfcOpenShell.

Probes all drive the release binary through `tauri-driver` and WebKitWebDriver, in through the
start screen with `openDemoJob(session)` from `tauri-harness.mjs` (569 lines, a hand-written
WebDriver client, D18). None calls `doc_open`. Each stamps evidence with the commit and refuses a
dirty tree (D98). The set, with the counts recorded green at the last head: front-door 18,
runtime-check 9, vocabulary-check 38, section1-trace 12, section2-sheet 17, section4-library 16,
section5-lenses 18, section6-model 37, section6b-panel 39, section7-legibility 28,
section9a-export 64, section10a-chrome 35, section-core 8, section-taper 9, section-model 23,
section-bill 22, section-walls 7, section-fastening 18, section-section 16, section-verbs 20,
section-labor 18, section-ifc 19, section-persona 28, real-job 13 (needs `fixtures/`).
`evidence-shots.mjs` takes the standing pictures.

---

## 6. What passes, and where

**In this container (2026-09-20, fresh clone, `pnpm install --frozen-lockfile`):**

| Check | Result |
|---|---|
| `pnpm -r build` | green (engine `tsc`; app `vite build`, 4.76 MB main chunk, wasm and worker copied) |
| `packages/engine` tests | 191 / 191 |
| `packages/app` tests (with `tsc --noEmit`) | 41 / 41 |
| `cargo test -p roofnerd-core` | 122 / 122 across nine suites |
| `pnpm gates` | vocabulary PASS, egress PASS, client-data PASS (nothing local to protect) |
| `cargo test --workspace` (the shell) | **cannot compile**: `gdk-3.0` not found by pkg-config. Needs webkit2gtk4.1, libsoup3, gtk3, javascriptcoregtk4.1, librsvg2 |
| `pnpm build:app`, every probe, `compare-fixtures` | **not runnable here**: no GTK, no display, no `fixtures/` |

**On Patrick's machine at `2bd6894` (STATUS, C8 landing):** every probe above green on the
shipped AppImage; real-job 13 / 13; fixtures within a hundredth of a percent; IFC export
reopened by IfcOpenShell with every count and quantity equal to the solve's, 21 schema findings
all the one known web-ifc limit (`IfcDimensionalExponents` written as floats, D159).

So the arithmetic and the solvers are verifiable anywhere. The claim that the *program* works is
verifiable only where a window can open, and the rule is that a section is done only there.

---

## 7. The rules that bind every session

All in `CLAUDE.md`; the ones that were paid for most:

- No client data in the public tree. Invent every number. The gate is a hook, not a habit (D29).
- Never loosen a gate. Never force-push. Write only inside this repository.
- No network call, no HTTP plugin, `connect-src 'self'`, tested in the shipped runtime (D19).
- No feature nobody asked for. If it is not in the handoff or an addendum, it is not built.
- The README vocabulary is the only vocabulary. Units are SF, LF, EA, SQ.
- A derived quantity is derived in the core and shows ◈; the front end never recomputes.
- Nothing is ever silently zero. Pending, no price, not in this total — each says so.
- The formula is editable on the line and no validation pass strips it.
- The stretch-out belongs to the detail, not the material (register #3; D134 makes it the screen word).
- No `prompt`, `alert`, `confirm` — the webview does not implement them.
- A check that bypasses the front door is not a check. Every probe starts at the start screen.
- Look at every image as a person before anything is called green (D91).
- Search before asking: the box, the prior sweeps, the open internet — and read the index first
  (D63). A question to Patrick is a last resort with a receipt.
- Decide, record, continue: a one-line D-entry, or a Q-entry with the smallest version built.
- Report at the end of every session in `STATUS.md`. Reporting is not stopping.

**Two traps recorded at the top of STATUS:** the client-data gate refuses any four-digit decimal
and any two numbers joined by a slash that happen to match a real report — write figures in prose
as whole numbers and ranges with "and"; and Patrick's harness kills background jobs during a Rust
compile or a long probe run — build and probe in the foreground.

---

## 8. The decisions, as a map

`DECISIONS.md` holds D1–D162 (some numbers unused; the file is a table, one line each, plus the
Appendix A reconciliation of 2026-09-07). By theme:

- **Public/private boundary and the gates:** D1, D2, D28–D30, D33, D50, D55, D56, D88, D89, D95, D98, D111, D115.
- **The document and the shell:** D4, D5, D8, D74, D107, D114, D117, D124, D127, D137.
- **Money:** D3, D11–D13, D16, D21–D27, D31, D32, D87, D109, D123, D128, D138, D161, D162.
- **Trace and measures:** D7, D10, D15, D41, D42, D93.
- **Chrome and legibility:** D14, D34–D40, D43–D49, D51–D54, D94, D96, D110, D120–D122, D129, D131–D134, D139, D140, D156.
- **The Model and geometry (superseded chain):** D57, D65, D71, D72, D73, D83–D86 → **D146–D149** (the core, the seam, regions and planes) → D150–D158 (bill, drawings, walls, coping, wind, verbs).
- **3D toolkit and IFC:** D141–D145, D159.
- **Library and provenance:** D60, D66, D67, D70, D75–D82, D80.
- **Sources and search discipline:** D61–D64, D69, D79, D90, D62.
- **Order of work and what is owed:** D99–D106, D112, D136.
- **Acceptance:** D91, D92, D97, D118, D119, D125, D126, D160.

The three to read whole before touching geometry or money: **D146** (Rust owns solvers, engine
owns money, estimate produced from the model), **D149** (piecewise-planar drainage regions; the
cone is retired), **D150** (the bill off the manufacturer's table, and `BOARDS_MAX` against `BOARDS`).

---

## 9. For a session that has this clone and nothing else

You can: read everything above, build both packages, run 354 tests and the three gates, price the
demo from the command line, change the engine or the core with tests to hold it, and write the
STATUS / DECISIONS / QUESTIONS records.

You cannot: open a window, take evidence, certify a section, run the fixture comparison, read a
ruling document, or see the persona notes. A change to what a person sees is not done until a
machine with WebKitGTK runs `pnpm build:app` and the probe for it, and a person opens the pictures.

The last session's instruction stands: nothing new is started until Patrick has looked at the
shipped C8 build. The open calls in §4 are his, with the receipts in STATUS.
