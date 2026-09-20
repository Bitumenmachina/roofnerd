# D7 and D8 — record drift and domain prints (scout/runner, sonnet)

(pulled verbatim from the agent transcript; the agent ran read-only and could not save it)

NOTE ON DELIVERY: I run under a strict read-only mandate (no file creation/modification of any kind, even inside the repo's own review-shots/ folder). I could not write review-shots/D7-D8-drift-and-domain.md. The full content that was to be saved there is below in full — please have a write-capable agent persist it if the file is still wanted on disk.

# D7-D8-drift-and-domain.md
(intended path: /home/user/roofnerd/review-shots/D7-D8-drift-and-domain.md — NOT written, see note above)

## D7 — record drift

### (1) README "Where it is right now" vs STATUS.md "Where the build is" (lines 6–30)

**README quote (README.md:8-12):**
> **Where it is right now:** sections 1–7, 9a (CSV export into the job folder) and 6b (the Model as a working surface) are built and checked on the shipped program — trace, conditions and the estimate sheet, the money model against two real jobs, the library, the lenses, the Model standing the roof up, and the legibility pass. `STATUS.md` says what each check proved and what was seen in the pictures that no check caught. Next: CSV export to the job folder, the Model as a working surface, then the Schedule.

**STATUS.md "Where the build is" head (STATUS.md:6-30), key sentences:**
> **Plan v3 is built: the model is what has to get built, and everything a person reads comes out of it.** Pushed 2026-09-13 as one line of 116 commits on `28b7adf`. On the shipped program every probe is green... (STATUS.md:8-13)
> **What was done, in the order it landed (the D-numbers say why):** C0 the Rust core and the seam (D146–D148) · C1 the field on drainage regions... · C2 boards... · C5 the Model matched to the handoff's Frame 2, the Section to Frame 3, one drawing model and one renderer for the layout and fastening overlays (D155–D156) · C3 walls and curbs... · C4 wind zones... · C6 the verbs... · C7 IFC4... · C9 the labor basis... · C8 a roofer opened it cold and used it... (STATUS.md:15-26)
> **For the next session, first act, before anything else is built:** `cd ~/projects/roofnerd && CARGO_BUILD_JOBS=4 pnpm build:app`... open the AppImage... press "Open the demo job"... (STATUS.md:28-34)

| # | README claim | STATUS contradiction |
|---|---|---|
| 1 | 9a "CSV export into the job folder" is listed as **built and checked** | The same README sentence's closing clause lists "CSV export to the job folder" again under **Next** — self-contradictory on its face — and STATUS's built-list (C0–C9, STATUS.md:16-26) has no entry that names CSV export at all, so nothing in STATUS confirms it as built. |
| 2 | 6b "the Model as a working surface" is listed as **built and checked** | Same sentence's **Next** clause repeats "the Model as a working surface" as upcoming work. STATUS's closest entry, C5 ("the Model matched to the handoff's Frame 2, the Section to Frame 3, one drawing model and one renderer for the layout and fastening overlays," STATUS.md:18-19), never uses the phrase "working surface," and STATUS's forward-looking "For the next session" block (STATUS.md:28-34) is about opening the demo for Patrick's review, not building this feature — consistent with it still being open, not "built and checked." |
| 3 | README's progress is keyed to numbered "sections" (1–7, 9a, 6b) | STATUS's entire head (lines 6–30) names progress exclusively by `C0`–`C9` phase codes tied to `D`-numbers (D146–D162) and never once uses the word "section" or the labels "9a"/"6b". The two files use incommensurable taxonomies with no stated crosswalk — a reader cannot tell which C-phase(s) correspond to README's "sections 1–7." |
| 4 | README's "Next" list ends with "then the Schedule" | STATUS's built-list (C0–C9) and its "What needs doing next" open items (STATUS.md:30 onward, Patrick's open calls on boards/wind pressures/adhesive rate/crew size) never mention "the Schedule" anywhere in the lines-6–30 window. |
| 5 | README implies only "sections 1–7, 9a, 6b" of the product are done, with more (CSV export, Model as working surface, Schedule) still ahead | STATUS opens by declaring **all** of Plan v3 built and shipped ("Plan v3 is built... Pushed 2026-09-13 as one line of 116 commits," STATUS.md:8-9) with every probe green and a live dollar bid — i.e., STATUS describes a fully shipped, reviewed build, not a partially-built one with three features still pending as README's paragraph implies. |

### (2) README "How the code is laid out" — src-tauri/core mention

Full quote (README.md:144-162):
> ## How the code is laid out, in roofing terms
>
> Think of the program as a commercial roof.
>
> - **`packages/engine` is the deck.** Everything bears on it. It is pure arithmetic — the vocabulary above as types, the conversions, the labor, the recap. It knows nothing about windows, drawings or files. If a number is wrong, it is wrong here, and there is a test for it here.
> - **`src-tauri` is the structure.** It holds the one open job and hands the same copy to every window. Windows do not own the job; they subscribe to it. That is why two windows can never disagree.
> - **`packages/app` are the systems on the deck** — the drawing editor, the estimate sheet, the recap, the library. Each is one editor, each can be torn off into its own window, and any of them can sit in any area of the screen.
> - **The formula on the estimate line is the flashing.** It is the joint where the geometry meets the money, it is the part that leaks when it is hidden, and so it is the part kept visible and editable at the point of use.

Grep confirms: `src-tauri` appears exactly once in README.md (line 152); `src-tauri/core` and `roofnerd-core` appear **zero** times.

**What's missing, against CLAUDE.md:12-19 ("The 3D model is the function, not a view"):**
> What has to get built — the tapered package laid out the trade's way, the walls and curbs with their flashing heights, the fastening pattern by wind zone — is solved in `src-tauri/core` (`roofnerd-core`), and the 2D outputs are produced FROM it... The core owns the solvers and nothing else: no window, no disk, no network, and no money. What a solved number costs is `packages/engine`, which is plain arithmetic.

README's "How the code is laid out" section describes `src-tauri` only as the thing that "holds the one open job and hands the same copy to every window" (the shell/sync layer) and describes `packages/engine` as pure arithmetic for cost. It never names `src-tauri/core`/`roofnerd-core` as the place where the actual geometry solving happens (tapered layout, walls/curbs, wind-zone fastening). Per CLAUDE.md that solver code is a distinct, load-bearing module — "the core owns the solvers and nothing else" — but README's roofing-metaphor tour of the codebase omits it entirely, leaving a reader with no idea where the 3D-model math actually lives.

### (3) .github directory / CI workflow at HEAD

`git ls-files .github` returns **nothing** (empty output, exit 0) — no `.github/` directory is tracked at HEAD, hence no CI workflow file exists in the tree.

### (4) .claude/ at HEAD

`git ls-files .claude` returns **nothing** (empty output, exit 0) — nothing under `.claude/` is tracked at HEAD. Any `.claude/` content currently in the working tree is untracked and was added by this review, per instructions.

### (5) Hook state on a fresh clone

On a fresh clone `git config --get core.hooksPath` returns nothing (hooks are off by default); this review's working copy already has it set to `.githooks` (confirmed: `git config --get core.hooksPath` → `.githooks`).

`.githooks/pre-commit` line 9:
> `# Turn it on once per clone:   git config core.hooksPath .githooks`

So the hook file itself documents that a fresh clone must manually run `git config core.hooksPath .githooks` to activate it — it is not enabled by anything in the repo automatically.

---

## D8 — domain prints

### (1) src-tauri/core/src/wind.rs — WIND_STANDARD and zone-width rule

Doc comment above the zone logic (wind.rs:1-31):
> ```
> 1  // ── Wind zones, and the uplift rating each one asks for ────────────────────
> 2  // Where the corners and the edges of a roof are, how hard the wind pulls on
> 3  // each, and what rating of assembly that makes the roof need. Everything a
> 4  // fastening pattern is chosen against, and nothing about the pattern itself —
> 5  // that is `fastening.rs`.
> 6  //
> 7  // THE ZONES (Part 1). NRCA *Membrane Roof Systems* 2019, Appendix 1, p.553:
> 8  // "Dimension 'a' is 10 percent of the least horizontal dimension of the building
> 9  // or 0.4 times the mean building height, whichever is smaller; but not less than
> 10 // either 4 percent of the least horizontal dimension or 3 feet." Figure 1-9 on
> 11 // the facing page draws what that dimension sets out: a corner region `a` by `a`
> 12 // at each outside corner, a perimeter band `a` wide along every edge, and the
> 13 // field of the roof inside that. The three tile the roof exactly, and
> 14 // `tests/the_zones.rs` says so to a hundredth of a square foot.
> ...
> 25 // ZONE 1'. The appendix names four zones, not three — Zone 1' at GCp 0.9 beside
> 26 // Zone 1 at 1.7 (Fig 1-2, p.546) — and then never draws the boundary. Figure 1-9
> 27 // is the only zone plan printed, for h ≤ 60 ft and for 60 < h ≤ 160 ft, and it
> 28 // draws 1, 2 and 3 and no 1'. Where Zone 1' begins comes from the ASCE figure
> 29 // itself, which is not in the box. So three zones are solved and the fourth is
> 30 // reported as missing rather than guessed at, and the solve says which document
> 31 // would close it.
> ```

`WIND_STANDARD` (wind.rs:74):
> `pub const WIND_STANDARD: &str = "ASCE 7-22 (Georgia 2024)";`

`dimension_a` function and its doc comment (wind.rs:162-185):
> ```
> 162 /// The zone width, and which term of the rule set it.
> 163 ///
> 164 /// `min(0.1 x least side, 0.4 x mean height)`, never under `max(0.04 x least
> 165 /// side, 3 ft)`.
> 166 pub fn dimension_a(least_dimension_feet: f64, height_feet: f64) -> (f64, &'static str) {
> 167     let tenth = least_dimension_feet * 0.1;
> 168     let by_height = height_feet * 0.4;
> 169     let floor = (least_dimension_feet * 0.04).max(LEAST_ZONE_FEET);
> 170     let (smaller, word) = if by_height < tenth {
> 171         (by_height, "0.4 of the mean roof height")
> 172     } else {
> 173         (tenth, "a tenth of the least side")
> 174     };
> 175     if smaller < floor {
> 176         let word = if least_dimension_feet * 0.04 >= LEAST_ZONE_FEET {
> 177             "the floor at 4 per cent of the least side"
> 178         } else {
> 179             "the floor at 3 ft"
> 180         };
> 181         (floor, word)
> 182     } else {
> 183         (smaller, word)
> 184     }
> 185 }
> ```
Also relevant, line 27: `// is the only zone plan printed, for h ≤ 60 ft and for 60 < h ≤ 160 ft, and it`

**Mapping:** This is the `a = min(0.1·least, 0.4·h)`, floored at `max(0.04·least, 3 ft)`, **three-zone** layout (corner/perimeter/field), sourced to NRCA *Membrane Roof Systems* 2019 Appendix 1 — the ASCE 7-10-style rule. It is explicitly not the ASCE 7-16/22 low-slope four-zone (0.6h / 0.2h) layout: the code's own comment (line 27) references "h ≤ 60 ft and 60 < h ≤ 160 ft" as the two Figure 1-9 zone plans it draws from, and lines 25-31 state that the appendix's fourth zone (Zone 1') is *not* solved — "three zones are solved and the fourth is reported as missing rather than guessed at." Quoting only, per the task's instruction not to judge beyond this mapping.

### (2) Insulation-adhesive and bonding-adhesive lines

**tools/make-demo-prices.mjs (catalogue, lines 138-147):**
> ```
> { id: 'iso-adhesive-field', description: 'Insulation adhesive, dual tank set, field',
>   costCode: '07-100-100', unit: 'EA', layer: 'adhesive', priceAs: 'iso-adhesive', band: [1050, 1320] },
> { id: 'iso-adhesive-perimeter', description: 'Insulation adhesive, dual tank set, perimeter',
>   costCode: '07-100-100', unit: 'EA', layer: 'adhesive', priceAs: 'iso-adhesive', band: [1050, 1320] },
> { id: 'iso-adhesive-corner', description: 'Insulation adhesive, dual tank set, corner',
>   costCode: '07-100-100', unit: 'EA', layer: 'adhesive', priceAs: 'iso-adhesive', band: [1050, 1320] },
> { id: 'bonding-adhesive', description: 'Bonding adhesive, 5 gallon pail',
>   costCode: '07-100-100', unit: 'EA', layer: 'adhesive', band: [205, 255] },
> ```
> DERIVED formula for bonding adhesive (line 494), with notes:
> ```
> { condition: 'c-field', id: 'bonding-adhesive', formula: 'BONDING_GAL / 5',
>   notes: 'Gallons into five gallon pails. The coverage rate the manual prints '
>     + 'already carries both faces, so it is not doubled here.', },
> ```

**jobs/demo-job/conditions.json (lines 115-152):**
> ```
> "description": "Insulation adhesive, dual tank set, field",   formula: "ADHESIVE_SETS_Z1_EA"   (no note)
> "description": "Insulation adhesive, dual tank set, perimeter", formula: "ADHESIVE_SETS_Z2_EA"  (no note)
> "description": "Insulation adhesive, dual tank set, corner",  formula: "ADHESIVE_SETS_Z3_EA",
>   "notes": "Every board course above the fastened one, and the cover board over the lot, at the
>    ribbon spacing the manufacturer's warranty table gives for this wind speed. The table stops
>    below this coast; the top row it prints is what this rests on, and the fastening plan says so
>    on its face."
> "description": "Bonding adhesive, 5 gallon pail", formula: "BONDING_GAL / 5",
>   "notes": "Gallons into five gallon pails. The coverage rate the manual prints already carries
>    both faces, so it is not doubled here."
> ```

**Does any on-screen field say the 120 mph row is used for a 130 mph roof?** No field says "120 mph" or "130 mph" literally. The closest on-screen disclosure is the `iso-adhesive-corner` note in conditions.json: "the ribbon spacing the manufacturer's warranty table gives for this wind speed. The table stops below this coast; the top row it prints is what this rests on, and the fastening plan says so on its face." This is a vaguer, un-numbered version of the gap — it says the manufacturer's table stops short of this coast's wind speed and the top (highest) row printed is used instead, but it never states the actual numbers (120 mph vs. 130 mph). The explicit "130 mph" figure and the "120 mph row" framing appear only in a source-code comment in tools/make-demo-prices.mjs (around line 525, in a comment block describing where `WIND_PSF_*` demo values come from — "pressures for this coast at 130 mph") and in STATUS.md's open-calls list ("the 130 mph adhesive rate uses the 120 mph row until the NEMO report is in the box") — neither of which is an on-screen `description`/`note` field a user would see in the app.

### (3) Cricket line(s)

**tools/make-demo-prices.mjs, catalogue entry (lines 379-387):**
> ```
> CATALOGUE.push({
>   // Cricket stock is its own lettered series — Carlisle prints two charts side by
>   // side, headed Cricket and Primary Slope — and the bulletin on disk carries only
>   // the primary one. So this is priced by the square foot of footprint and not by
>   // letter, and that gap is on the record rather than papered over with the
>   // field's own table.
>   id: 'cricket-stock', description: 'Cricket stock, cut to the saddle',
>   costCode: '07-100-100', unit: 'SF', band: [2.15, 2.6], waste: 10,
> });
> ```
DERIVED formula (line 406): `{ condition: 'c-cricket', id: 'cricket-stock', formula: 'CRICKET_SF' },` — no `notes` field.

**jobs/demo-job/conditions.json, `c-cricket` condition (lines 689-700):**
> ```
> "id": "c-cricket",
> "items": [
>   {
>     "costCode": "07-100-100",
>     "description": "Cricket stock, cut to the saddle",
>     "formula": "CRICKET_SF",
>     "id": "cricket-stock",
>     "unit": "SF",
>     "unitCost": 2.5,
>     "waste": 10
>   }
> ],
> ```
No `notes`/`note` field present.

**Does any field name that the cricket is priced by area from the primary-slope chart rather than cricket stock?** No. The explanation ("Cricket stock is its own lettered series... the bulletin on disk carries only the primary one. So this is priced by the square foot of footprint and not by letter") exists **only** as a source-code comment (lines 380-384 of tools/make-demo-prices.mjs) — it never appears in any `description` or `note`/`notes` field in either the catalogue item or the job's `conditions.json`. On screen the item just reads "Cricket stock, cut to the saddle," priced per SF, with no disclosure that it's riding the primary-slope chart rather than a cricket-specific one.

### (4) `node packages/engine/bin/roofnerd.mjs recap jobs/demo-job` — full output

```
Northgate Retail Center — Roof Retrofit — Scenario 1
================================================
MATERIAL                              $26,153.66
  Sales tax @ 7%                       $1,830.76
  Escalation @ 3%                        $784.61
  Total Material                      $28,769.03   $299.68/SQ
LABOR                                 $30,722.16   584.21 hrs
  Burden @ 10%                         $3,072.22
  Total Labor                         $33,794.38   $352.02/SQ
SUB                                    $3,053.00
  General liability @ 5%                 $152.65
  Total Sub                            $3,205.65   $33.39/SQ
EQUIPMENT                              $6,806.25
                                                   $70.90/SQ
OTHER                                 $12,013.25
                                                   $125.14/SQ
SUPERVISION                            $8,340.00   120.00 hrs
  Burden @ 50%                         $4,170.00
  Total Supervision                   $12,510.00   $130.31/SQ
----------------------------------------------------
Job cost                              $97,098.55
Profit                                $14,564.78
CONTRACT AMOUNT                      $111,663.33
Bond                                   $2,233.27
----------------------------------------------------
SELLING PRICE                        $113,896.60

Total SQ    96.00
Total hours 704.21
Hours/SQ    7.34

By what each line is:
  Material            $26,153.66
  Labor               $39,062.16   704.21 hrs
  Subcontract          $3,053.00
  Equipment            $6,806.25
  Other               $12,013.25

1 line on the sheet and out of the total:
  Parapet Wall Flashing → Form and set the metal: alternate — the shop is fabricating this ($3,300.00)

24 lines not in the total:
  Main Roof Field → Tapered *: the model found nothing of this kind on this roof (TAPER_*_SF)
  Main Roof Field → Flat polyiso fill board: nothing here is called "FILL_SF"
  Main Roof Field → Base layer fasteners, field: nothing here is called "FASTENERS_Z1_EA"
  Main Roof Field → Base layer fasteners, perimeter: nothing here is called "FASTENERS_Z2_EA"
  Main Roof Field → Base layer fasteners, corner: nothing here is called "FASTENERS_Z3_EA"
  Main Roof Field → Barbed plates, 3 in: nothing here is called "PLATES_EA"
  Main Roof Field → Insulation adhesive, dual tank set, field: nothing here is called "ADHESIVE_SETS_Z1_EA"
  Main Roof Field → Insulation adhesive, dual tank set, perimeter: nothing here is called "ADHESIVE_SETS_Z2_EA"
  Main Roof Field → Insulation adhesive, dual tank set, corner: nothing here is called "ADHESIVE_SETS_Z3_EA"
  Main Roof Field → Bonding adhesive, 5 gallon pail: nothing here is called "BONDING_GAL"
  Parapet Wall Flashing → TPO wall flashing, 60 mil: nothing here is called "FLASH_SF"
  Parapet Wall Flashing → Coping cap, 24 ga galvanized: nothing here is called "COPING_LF"
  Parapet Wall Flashing → Continuous cleat, 20 ga: nothing here is called "CLEAT_LF"
  Parapet Wall Flashing → Shop-formed corner, mitre welded: nothing here is called "COPING_CORNERS_EA"
  Parapet Wall Flashing → Fasteners, cleat and termination bar: nothing here is called "CLEAT_LF"
  Parapet Wall Flashing → Fabricate coping cap, 24 ga: nothing here is called "COPING_LF"
  Parapet Wall Flashing → Install coping cap, 24 ga: nothing here is called "COPING_LF"
  Parapet Wall Flashing → Fabricate continuous cleat, 20 ga: nothing here is called "CLEAT_LF"
  Parapet Wall Flashing → Install continuous cleat, 20 ga: nothing here is called "CLEAT_LF"
  Parapet Wall Flashing → Coping length, shop-formed: nothing here is called "COPING_PCS_EA"
  Roof Drains → Tapered sump panel, 4 ft square: nothing here is called "SUMP_PANELS_EA"
  RTU-1 Curb → TPO curb flashing, 60 mil: nothing here is called "FLASH_SF"
  RTU-1 Curb → Curb cap, 24 ga galvanized: nothing here is called "COPING_LF"
  Cricket behind RTU-1 → Cricket stock, cut to the saddle: nothing here is called "CRICKET_SF"
```

Note: this recap output shows 24 formula-driven lines unresolved ("nothing here is called ...") against the demo job — a discrepancy worth flagging to the parent review, distinct from D7/D8's specific asks but visible in the raw output pasted above.

---

**Files referenced (absolute paths):**
- /home/user/roofnerd/README.md (lines 8-12, 144-162)
- /home/user/roofnerd/STATUS.md (lines 1-34)
- /home/user/roofnerd/CLAUDE.md (lines 10-19)
- /home/user/roofnerd/.githooks/pre-commit (lines 1-15)
- /home/user/roofnerd/src-tauri/core/src/wind.rs (lines 1-31, 74, 90, 162-185)
- /home/user/roofnerd/tools/make-demo-prices.mjs (lines 130-160, 375-410, 480-498, 505-544)
- /home/user/roofnerd/jobs/demo-job/conditions.json (lines 100-152, 686-720)
</message>
</invoke>

