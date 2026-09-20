# D5 — where the tests' expected values come from (auditor, opus)

(pulled verbatim from the agent transcript; the agent ran read-only and could not save it)

# D5 — Where the tests' expected values come from

**Note on the deliverable:** I run read-only (no file creation, including `/home/user/roofnerd/review-shots/`). `review-shots/` exists and is empty. The full document is below verbatim — please persist it as `review-shots/D5-oracles.md` yourself, or re-dispatch with write permission.

Classes: **(1)** hand arithmetic by the same author · **(2)** vendor table on disk (`data/*.json`) · **(3)** NRCA/published figure cited by page or figure · **(4)** artifact made outside the author's sessions · **(5)** self-consistency only (code against code, snapshot, read-back).

## Table 1 — suite by suite

| suite | tests | class(es) | evidence (file:line, ≤20 words) |
|---|---|---|---|
| `packages/engine/test/by-letter.test.mjs` | 5 | 2, 5 | `by-letter.test.mjs:27` reads `data/profiles/versico-versicore-mp-h.json`; letters never typed; pricing asserts scope built in-test |
| `packages/engine/test/engine.test.mjs` | 13 | 1, 5 | `engine.test.mjs:22` `sfToSq(sf(2350))===23.5`; `:38` write-then-read round-trip is pure self-consistency |
| `packages/engine/test/geometry.test.mjs` | 11 | 1 | `geometry.test.mjs:39` bow-tie area 0 on a 10×6 shape, worked in comment |
| `packages/engine/test/line.test.mjs` | 22 | 1 | `line.test.mjs:112` `16 = ceil(400×14/12/30)`, arithmetic written out above it |
| `packages/engine/test/money.test.mjs` | 50 | 1, 5 | `money.test.mjs:2` "shapes read off real Edge reports; the numbers are invented, real ones stay in `fixtures/`" |
| — notable: `a recap that reproduces a printed one` | 1 | 5 | `money.test.mjs:430` `compareRecap` fed figures hand-derived to match the recap it checks |
| — notable: `the profit rate is derived from the dollars` | 1 | 1 | `money.test.mjs:458` 123456/1000000 = 12.3456%, and 12.35% misses by a dollar |
| `packages/engine/test/roof.test.mjs` | 19 | 3, 1 | `roof.test.mjs:22,74,80,99,127` cite NRCA p.166-168, p.79, Fig. 4-13, p.164 by page |
| — notable: `one board layer does not stop at four inches` | 1 | 3 (uncited) | `roof.test.mjs:114` "Carlisle InsulBase runs to 4.5 in; GAF to 4.6" — no page given |
| — notable: `thickness is a magnitude` | 1 | 3/4 | `roof.test.mjs:153` cites `IfcNonNegativeLengthMeasure` and FreeCAD's clamping behaviour |
| `packages/engine/test/takeoff.test.mjs` | 63 | 1, 5 | `takeoff.test.mjs:241` "numbers below are invented; only the rule came from anywhere" |
| — notable: `twelve rectangular curbs read 48 EA` | 1 | 4 (rule only) | `takeoff.test.mjs:251` rule read off real Edge drawing reports; the 48 is hand arithmetic |
| — notable: `readTaperAt` block (4 tests) | 4 | 1, 5 | `takeoff.test.mjs:466` mirrors Rust `read_at`; 9 in vs the cone's 10.3615 worked by hand |
| — notable: `every derived name has a trade label` | 1 | 5 | `takeoff.test.mjs:654` asserts labels against the labeller's own output shape |
| `packages/engine/test/vendor-data.test.mjs` | 8 | 5 over 2 | `vendor-data.test.mjs:5` "says nothing about whether the numbers themselves are right" |
| — notable: `transcribed positions add up to printed count` | 1 | 2 | `vendor-data.test.mjs:105` sums layout positions against the transcribed `fasteners` figure |
| `packages/app/test/ifc-ifcx.test.mjs` | 6 | 5 | `ifc-ifcx.test.mjs:79` every value read back off `expectedOf(plan)` — same plan that wrote it |
| — notable: header key set | 1 | 4 (claimed) | `ifc-ifcx.test.mjs:33` cites `hello-wall.ifcx`'s five header keys; that file is **not in the tree** |
| `packages/app/test/ifc-profile-data.test.mjs` | 5 | 2 | `ifc-profile-data.test.mjs:19,28` every asserted edge read out of the profile JSON on disk |
| `packages/app/test/ifc-triangulate.test.mjs` | 7 | 1 | `ifc-triangulate.test.mjs:44` `10×10 − 6×6` written in the test as the true area |
| `packages/app/test/ifc-write.test.mjs` | 23 | 5, 4, 2 | `ifc-write.test.mjs:11` "every number asserted is read back OFF that same made-up solve" |
| — notable: all `readBack` assertions | ~15 | 4 (format only) | `ifc-write.test.mjs:27,39` web-ifc 0.0.77 re-parses the bytes; quantities still come from the solve |
| — notable: `covering carries the letter's own printed edge` | 1 | 2 | `ifc-write.test.mjs:215` reads `data/profiles/…json` and compares X's thin/thick edge |
| — notable: `expectedOf matches the file, entry by entry` | 1 | 5 | `ifc-write.test.mjs:135` both sides derive from one `plan`; web-ifc only supplies the read |
| `src-tauri/core/tests/nrca_figures.rs` | 7 | 3, 1 | `nrca_figures.rs:8-12` Fig 4-9 p.164, 4-10/4-11 p.165, 4-15 p.167, 4-17 p.168 |
| — notable: `the_core_and_the_engine_state_the_same_conventions` | 1 | 5 | `nrca_figures.rs:357` parses `packages/engine/src/roof.ts` at test time — code against code |
| `src-tauri/core/tests/the_bill.rs` | 10 | 1, 2, 3 | `the_bill.rs:80-106` whole bill worked by hand; `:13-19` NRCA p.166/p.208/p.164 cited |
| — notable: `the_hand_checked_roof_comes_to_the_bill` | 1 | 1 + 2 | `the_bill.rs:148,157` letters X/Y/Z/ZZ and `document_id "VE-7830"` come from `data/profiles` |
| — notable: `the_core_and_the_engine_state_the_same_sump_rule` | 1 | 5 | `the_bill.rs:541` greps `sumpSide` out of `roof.ts` and compares to the Rust copy |
| `src-tauri/core/tests/the_drawings.rs` | 7 | 1, 5 | `the_drawings.rs:11-22` cap/axes/cuts worked by hand off `jobs/demo-job` |
| — notable: `the_layout_draws_the_bill_and_nothing_else` | 1 | 5 | `the_drawings.rs:60` drawing held to the bill the same solve produced |
| — notable: `the_top_of_every_panel_is_the_roof_the_layout_solved` | 1 | 5 | `the_drawings.rs:326` drawing corners compared to `taper::read_at` — same solver |
| — notable: `the_core_and_the_engine_stack_a_roof_in_the_same_order` | 1 | 5 | `the_drawings.rs:414` parses `LAYER_FUNCTIONS` out of `model.ts` |
| `src-tauri/core/tests/the_fastening.rs` | 10 | 5, 2 | `the_fastening.rs:88` count asserted as `zone.boards × pattern_for(...)` — solver's own lookup |
| — notable: `the_adhesive_follows_the_zone` | 1 | 5 | `the_fastening.rs:170` recomputes sets from the same three constants the solver used |
| — notable: `the_standard_the_basis_and_the_document_all_travel` | 1 | 2 | `the_fastening.rs:212` `printed.document_id == "DR-05"`, from `data/fastening/dr-05-2024-07.json` |
| `src-tauri/core/tests/the_verbs.rs` | 8 | 1 | `the_verbs.rs:92,141` 20 ft run and drain-to-drain distance recomputed by hand in the test |
| `src-tauri/core/tests/the_walls.rs` | 7 | 1, 3 | `the_walls.rs:47-50` `1172.46875/12` trapezoid sum worked out; `:98` NRCA 8 in, p.285 |
| — notable: `the_offset_is_proved_on_an_l_shaped_ring` | 1 | 1 | `the_walls.rs:240` 28.5 SF by hand shoelace, checking third-party `cavalier_contours` |
| `src-tauri/core/tests/the_zones.rs` | 8 | 1, 5, 3 | `the_zones.rs:171` recomputes `design × ASD_FACTOR × SAFETY_FACTOR` with the solver's constants |
| — notable: `a_pressure_typed_at_working_load` | 1 | 3 | `the_zones.rs:262` "p.565: an assembly rated FM 1-90 covers 45 psf" |
| — notable: `the_plan_says_what_it_was_drawn_to` | 1 | 5 | `the_zones.rs:196` asserts the strings "ASCE 7-22"/"Appendix 1" the code itself emits |
| `src/coping.rs` `#[cfg(test)]` | 5 | 1, 3 | `coping.rs:275` "Fig. 2-3, p.58: the step is exactly at 8 and at 10 inches" |
| `src/cricket.rs` `#[cfg(test)]` | 3 | 5, 3 | `cricket.rs:516` `cricket_slope(None,0.25)==(0.5,false)` — the 2× rule is NRCA, asserted against itself |
| `src/drawing.rs` `#[cfg(test)]` | 2 | 1 | `drawing.rs:1477` `feet_written(21.9999)=="22'-0\""` — formatting worked by hand |
| `src/fastening.rs` `#[cfg(test)]` | 9 | 2, 3, 5 | `fastening.rs:~700` Versico Table VIII (Manual PDF p.14) vs Carlisle DR-05: two covers, one count |
| — notable: `the_worked_example_in_the_book_is_in_the_table_on_disk` | 1 | 3 | NRCA p.207's 6/10/16 per 4×8 at FM 1-90 — nearest thing to an independent oracle here |
| — notable: `tightening_the_ribbons_to_a_third…` | 1 | 5 | asserts `at_four×3 == at_twelve`, arithmetic on the module's own two constants |
| `src/geom.rs` `#[cfg(test)]` | 5 | 1 | `geom.rs:~688` a point in the bounding box but in the notch is off the roof |
| `src/lib.rs` `#[cfg(test)]` | 10 | 1, 5 | `lib.rs:36-57` THE PIN: cap 14 in, (16,132)→9 in, all worked by hand in the doc comment |
| — notable: `the_cricket_behind_the_curb_stands_on_the_roof` | 1 | 5 | `lib.rs:385-392` a 36-float `positions` array pinned verbatim — a snapshot of the code's output |
| — notable: `the_payload_speaks_camel_case` | 1 | 5 | `lib.rs:427` serializes the solve and asserts its own field names |
| `src/model.rs` `#[cfg(test)]` | 4 | 1, 5 | `model.rs:485` `feetPerUnit 2.0` × (9,4) → (18,8); `:492` pins the serializer's own JSON string |
| `src/profile.rs` `#[cfg(test)]` | 7 | 2, 1 | `profile.rs:502-523` every letter, edge and fill step read from the chart compiled in by `include_str!` |
| — notable: `a_thickness_on_a_shared_edge_takes_the_panel_above_it` | 1 | 5 | `profile.rs:542` "The table prints no rule, so this is ours" — house rule asserted against itself |
| `src/scope.rs` `#[cfg(test)]` | 4 | 1 | `scope.rs:~227` 32×32 roof: 81 cells, X 33−1, Y 8, Z 16, ZZ 24, worked in the doc comment |
| `src/sump.rs` `#[cfg(test)]` | 1 | 3, 1 | `sump.rs:108` `sump_side_inches(12)==36` — NRCA bowl + 24 in |
| `src/verbs.rs` `#[cfg(test)]` | 9 | 1 | `verbs.rs:~500` "arithmetic on a forty-foot square anybody can do in their head" |
| `src/wind.rs` `#[cfg(test)]` | 6 | 1, 3 | `wind.rs:~455` "p.565: FM 1-90 covers 45 psf"; zone widths recomputed by hand |

**Gap worth flagging:** `src-tauri/core/src/taper.rs` (1,075 lines, the tapered-layout solver), `walls.rs` (423) and `boards.rs` (239) carry **no `#[cfg(test)]` module at all** — they are exercised only through the integration tests above.

## Table 2 — outside oracle (class 4) per solver output

| solver output | class-4 oracle in the tree? | which / why not |
|---|---|---|
| Tapered layout | **no** | NRCA figures are cited prose (class 3); every figure re-derived by hand on invented roofs (`nrca_figures.rs:8-12`) |
| Board bill | **no** | Hand arithmetic (`the_bill.rs:80-106`) over the author-transcribed vendor chart (class 2) |
| Sumps | **no** | NRCA p.164 bowl + 24 in, cited only (`the_bill.rs:317-327`, `sump.rs:108`) |
| Crickets | **no** | NRCA Fig 4-13 p.166 ratio, cited; 7.5:1 refusal worked by hand (`the_bill.rs:446-468`) |
| Walls / flashing | **no** | `1172.46875/12` and the 28.5 SF offset are the author's own shoelace (`the_walls.rs:47,240`) |
| Coping | **no** | Corner counts hand-worked off the demo ring; gauge steps from Fig. 2-3 p.58, cited (`coping.rs:230-247,275`) |
| Wind zones | **no** | ASCE 7-22 App.1 cited as strings; the ASD→safety→rating chain re-run with the solver's own constants (`the_zones.rs:171`) |
| Fastening counts | **no** | Counts asserted as `boards × pattern_for(...)` — the solver's own lookup (`the_fastening.rs:88`). Closest independents are still class 2/3: Versico Table VIII vs Carlisle DR-05, and NRCA p.207's 6/10/16 (`fastening.rs` tests) |
| Adhesive | **no** | `at_four×3 == at_twelve` is arithmetic on the module's own constants; coverage rate never checked against anything (`fastening.rs`, `the_fastening.rs:158-178`) |
| Drawings / sections | **no** | Held to the solve that drew them (`the_drawings.rs:60,326`); the 22/19/10 ft cuts are hand arithmetic off the demo job |
| **IFC writer** | **yes — partial** | **web-ifc 0.0.77** re-parses the written bytes (`ifc-write.test.mjs:27,39`); **IfcOpenShell** in `tools/verify-ifc.py` (not a test). Format/parseability/schema only — every *quantity* compared comes from `expectedOf` off the same solve, so no outside number ever meets a solver number |
| Money recap | **no** | `compareRecap` has the shape but the real Edge reports live in gitignored `fixtures/` (`.gitignore:17`, `money.test.mjs:2-3`, `tools/compare-fixtures.mjs:5-6`) — nothing in-tree supplies a printed bid to reproduce |

**Bottom line.** Eleven of the twelve solver outputs have no class-4 oracle anywhere in the repository. The only outside artifact that actually runs in the test tree is web-ifc, and it validates *serialization*, not *arithmetic*. The two class-4 artifacts the project genuinely depends on — a real Edge recap and a real job — are deliberately excluded (`fixtures/`, `refs/`, `evidence/` are gitignored, by a stated client-data policy), so the public tree cannot demonstrate that any quantity this program produces matches a number produced by anyone else. The strongest substitutes present are: NRCA p.207's worked fastener example being *present in* the transcribed table (`fastening.rs`), the Versico-vs-Carlisle two-document agreement on 8 fasteners per 4×8 at FM 1-90, and the hand-shoelace check of `cavalier_contours` (`the_walls.rs:240`).

