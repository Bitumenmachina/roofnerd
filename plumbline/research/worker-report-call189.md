# W-189 report: the scan's wall finder rebuilt on the published method (command line only)
Working copy: .../scratchpad/wtP. My scratch: .../scratchpad/w189. Nothing committed or pushed. Written as I go.

## 1. What I read at the source (2026-11-23)
- Paper: https://arxiv.org/html/2503.11498v2 (fetched, 81k characters of text; the parameter table read). Code: https://raw.githubusercontent.com/VaclavNezerka/Cloud2BIM/master/
  LICENSE (MIT, "Copyright (c) 2025 Vaclav Nezerka": read by me, whole file), README.md, config.yaml, cloud2entities.py, aux_functions.py (1,418 lines) fetched into w189/src and read
  where it matters (`identify_walls`, `merge_collinear_segments`, `group_segments`, `calculate_wall_axis`, `adjust_intersections`, `identify_openings`). The repository tree listing was refused
  (GitHub API not enabled for the session), so files I did not name were not seen (space_generator.py, generate_ifc.py).
- The paper's table (stated): z_section_boundaries [0.9, 1.0] of the storey's height; histogram threshold 0.01 ("all nonzero values contribute"); square(5) closing; epsilon 0.02 m/px; angle_tolerance 3 degrees;
  max10 = 10 for openings. The code at master differs from the paper's table: z band [0.85, 1.2] of the storey's point extent; Douglas-Peucker tolerance 0.04 m; pixel = pc_resolution (0.002 m) x grid_coefficient (5) = 1 cm;
  min_wall_length 0.10 m, min_wall_thickness 0.05, max_wall_thickness 0.75 (config.yaml); merge collinear if within min_thickness/2 across and ends within max thickness; group parallel (5 degrees, ends within
  max thickness, overlap more than max thickness); axis from the longer of two; endpoints snap to an intersection within half the max thickness; openings: x-histogram gap under 0.7 of the tenth fullest bin, width over 0.4 m,
  height over 0.6 m, height/width under 4, top over 1.6 m, door if it reaches the floor (0.1 m) and is over 1.6 m tall, else window; zones snap 0.8 m. So the survey's "paper gives no values for min wall length" is true of the paper and
  false of its config (0.10 m, 0.05 m, 0.75 m): those are the code's.
- Not read: space_generator.py, generate_ifc.py (not needed), the paper's last 5 kB.

## 2. Truth first (before any code)
Three files, written 2026-11-23 before any finder code: `.../scratchpad/w189/truth_small.txt`, `truth_long.txt`, `truth_large.txt` (pictures: pic.py, `mid_*`, `cut4_*`, `top_*` PNGs there; each wall: ends in feet in
the program's frame, how sure, faces seen, a note). In short:
- small: a narrow furnished room about 4.5 by 8 ft turned about 15 degrees, NOT four clear walls at the sizes the first agents said: south wall (1.7,-1.6)-(5.7,-0.5) and north wall (-0.9,6.1)-(3.6,7.4) sure; east wall
  (3.8,7.4)-(5.7,-0.4) medium (the ceiling over it is 6 ft 11 in, the top band is faint there); west wall low (three pieces, a jog of about 1.2 ft that I cannot read). All one-sided. Closes. No opening I can read.
- long: a 12 by 24 ft room, a big north part and a narrower south alcove, a step on the west and a jog on the east: 6 sure walls (N 10.8 ft, W1 12.8, E1 14.5, W2 5.8, S 8.3, E2 6.0), STEP (2.4 ft) medium, JOG low;
  another room's wall far east (x 4.4 to 6.5) is not this room's.
- large: a notched room turned 25 degrees; sure only NA (6.7 ft), NB (4.2 ft) and the 10 ft wall EW (9.7 ft); W (11 ft, a closet front to 6 ft) medium; the north-east and south parts low. Does not close.
Red run (the stub finder, tests first): `w189/red.log`: "15 failed; 0 passed" (the first compile was red as well: the pure rules did not exist).

## 3. Progress notes (written as I go)
- Ported and written: `crates/plans/src/found_189.rs` (new finder), `Method`/`find_by_ceiling`/`find_level_by` in `found.rs` (visibility of a few items made `pub(crate)`; the first finder's logic untouched),
  `--by-ceiling` on the verb (`crates/cli/src/found.rs`, usage in `main.rs`), NOTICE and LICENSES.md (Cloud2BIM, MIT, method ported, no code copied).
- Tests: `found_189_tests.rs`, 15 tests, written first; red run (stub) quoted: 15 failed, 0 passed (`w189/red.log`); green now: 15 passed.
- Found by running on the real scans (not by tests): (a) the first design paired two faces by which side their outline's mass was on; a wall of two faces is two separate thin outlines in a phone's or a tripod's scan
  (nothing between its faces), so the pairing is now by parallel faces within 2 ft with a trough of points between, as the paper's `group_segments` does; (b) a ceiling lower by a few inches over part of a room
  put the ceiling's own points in the band as a patch (the first reading of the ceiling takes under 3 in as one height): the ceiling over each foot square is now told again from its own points (LOCAL_*);
  (c) tried and dropped: a test of "level" pixels (a phone's mesh has vertices in rows, a wall's top is as level as a ceiling) and of "fat" patches (it cut a real wall that lies against a patch).

## 4. What was ported, what was written here, and every measure (constant, value, where it is stated)
Ported from Cloud2BIM (read open; MIT; no line copied; nothing linked): the order of work (top band, density picture, closing, outlines, straightening, collinear merge, pairing of opposite faces, longest wall as axis, ends carried to meet, each wall's
face read for openings) and the measures its paper's table states: BAND_SHARE 0.10 (z_section_boundaries [0.9, 1.0]); closing one pixel (square(5) at 1 cm); EPS_PAPER 0.02 m (epsilon); ANGLE_TOL 3 degrees (angle_tolerance);
MERGE_GAP 2.46 ft (the code's max_wall_thickness 0.75 m: ends within it); the longest wall's direction as the axis; snapping to an intersection (the code: half the greatest thickness; here CORNER 1 ft, call 173, which is 0.3 ft less).
Written here from the published papers: the border following (Suzuki and Abe 1985; mine follows the pixels' edges, finds the same outer and hole borders, joins eight neighbours: `Mask::borders`, tests `a_block_has_one_outline...`,
`two_blocks_touching_at_a_corner...`), and Douglas-Peucker (1973; `straighten`, `straighten_open`, with the tolerance no less than 1/20 of the chord: a bowed wall is one wall).
This project's own (not in the paper): the density threshold relative to what a wall of this scan shows (MASK_SHARE 0.25 of the floor's own density, call 188's quarter, at least 2 points; the paper's is "any point"); the pixel
no finer than twice the floor's scatter (PIXEL_ROUGH 2, at most 0.2 ft); the ceiling over each foot square told again from its own points (LOCAL_*: bins 0.05 ft, 8 points, from 3/4 of the height: found necessary on the long scan, whose
ceiling is a few inches lower over part of the room: the first reading takes under 3 in as one height, and those ceiling points formed patches with edges that looked like walls); the thin outline of a wall seen from one side told from the two faces of a
wall seen from both (the two sides of a thin outline are one face: nearer than max(2 in, 2.5 scatters); two faces only where the points across the gap are two layers with a trough between (TROUGH 0.5) AND the floor was seen on both outer sides: else one
face, "passes of one wall" joined and said); the lean told from the points the whole height (Theil-Sen over slices of 1 ft, at least 4 slices agreeing within 0.1 ft: call 188's lean) and the wall laid where it stands at the floor; the bow (a parabola fitted to the
face's points, said at 1 in or more: call 188); what stands in the room (cells of 0.5 ft with 4 or more points between 0.8 ft up and 0.8 ft under the ceiling, no wall's face accounting for them, three or more touching: STAND_*); the foot test, the three doubts
("likely a wall... hangs from the ceiling", "meets no wall at either end", "a wall or something tall standing at the wall"), the beyond-a-wall rule, one face one wall, whether the walls close, what is left out and said, the progress lines, the thinning,
the up check, the unlikely-house words: all the first finder's own code or its words, reused (`found_183::beyond`, `one_face_one_wall`, `Grid`, `edge`, the sentences) or ported from it. Openings are read by the first finder's rules (grid of 3 in squares, holes of 1 ft 6 in or more, hidden where
something stands in front), NOT by the paper's (x-histogram gaps under 0.7 of the tenth fullest bin; width over 0.4 m, height over 0.6 m, aspect under 4: not used, since the first finder's rules say "not seen" from "open" and the paper's do not).
Where each would hold on a scan not seen: ratios to the scan's own density and scatter (MASK_SHARE, PIXEL_ROUGH, SEPARATE, tol), physical sizes of a house (2 ft, 16 in, 1 ft, 3 degrees), or the paper's/calls' figures. Honest limits: LOCAL_* and the choice not to filter "level" or
"fat" patches were found on the real scans themselves, so those three scans are no longer a clean test of the finder; MERGE_GAP and the pairing's floor-share (SEEN_SHARE 0.3) are mine and were set on the made-up phone scene.

## 5. The real scans, finder against finder (my truth files; `w189/tables.txt`, `w189/score.py`)
(Matching: a found wall matches a truth wall when it is within 10 degrees, 1 ft across on average, and runs side by side for 40% of the shorter; "off" is the mean distance across in feet; "ends" are how far its two ends lie from the truth's,
along the wall, in feet; "(doubt)" is in doubt and not laid by itself. Truth ends are by hand, to about half a foot: do not read a figure under that as exact.)
Run time: present 10.2 s, 6.5 s, 6.2 s; by ceiling 1.6 s, 2.3 s, 2.0 s (the three scans, the long and the large thinned first 1 in 5 and 1 in 3).

small (a narrow furnished room about 4.5 by 8 ft; truth: S, N sure; E medium; W low):
| truth wall | present finder | by ceiling |
|---|---|---|
| S 4.1 ft (high) | found once, in doubt, 9.6 ft long (runs 2.7 ft past each end), off 0.15 | found once, 4.2 ft, ends within 0.0 and 0.0, off 0.02, not in doubt |
| N 4.7 ft (high) | NOT FOUND | found once, 4.5 ft, off 0.06, not in doubt |
| E 8.0 ft (medium) | found twice, both in doubt, 8.3 ft and 9.2 ft (a wall and a ghost) | found once, in doubt, 2.2 ft (27% of it), off 0.67 |
| W 8.1 ft (low) | NOT FOUND | found twice, 2.2 ft each, not in doubt, 54% of it, off 0.15 and 0.09 |
| not in the truth | none | none |
| laid by itself | 0 walls | 4 walls (S, N, two pieces of W) |
| closes? truth: yes | said nothing of closing (all in doubt) | said: do not close, 4 ends meet nothing (true: E and W are broken) |

long (a 12 by 24 ft room: big north part, south alcove; truth: N, W1, E1, W2, S, E2 high; STEP medium; JOG low):
| truth wall | present finder | by ceiling |
|---|---|---|
| N 10.8 ft | found once, 14.0 ft (3.3 ft past the west end), off 0.12 | found once, 12.1 ft (1.4 ft past), off 0.11 |
| W1 12.8 ft | found once, 13.7 ft, off 0.50 | found once, 13.1 ft, off 0.57 |
| E1 14.5 ft | found twice: 14.5 ft not in doubt, and a 10.1 ft one in doubt | found once, 14.5 ft, off 0.36, ends within 0.0 |
| STEP 2.4 ft | NOT FOUND | NOT FOUND |
| W2 5.8 ft | found as a 21.5 ft wall, off 0.85 (the W line run on into W1: wrong length) | one piece 2.8 ft in doubt (42% of it) |
| S 8.3 ft | found once, 9.7 ft, off 0.34 | found in 3 pieces of 2 to 3 ft, all in doubt (67% covered) |
| E2 6.0 ft | found twice, 5.4 ft and 5.1 ft (a wall twice) | NOT FOUND |
| JOG | NOT FOUND | NOT FOUND |
| not in the truth | 4: a 22.9 ft wall (-5.8,-12.2 to -11.1,10.2: the west line again), 7.0 ft, 3.2 ft (none in doubt), 6.7 ft (in doubt) | 3, all in doubt: 2.3 ft, 6.0 ft, 4.5 ft (the last is the ghost room's wall beyond the east) |
| closes? | 6 ends meet nothing, said | 3 ends meet nothing, said |
| laid by itself | 10 walls, 3 of them not in the truth | 3 walls, all in the truth |

large (a notched room about 17 by 21 ft; truth: NA, NB, EW high; W medium, STEPA medium; NC, NE, SW low; nothing else readable):
| truth wall | present finder | by ceiling |
|---|---|---|
| NA 6.7 ft | NOT FOUND | found once, in doubt, 4.5 ft (67% of it), off 0.13 |
| NB 4.2 ft | found twice as 14.7 ft long walls (NB, NC and more as one line), 0.5 to 0.8 off | found once, in doubt, 4.3 ft, off 0.14 |
| EW 9.7 ft | found once, 10.6 ft, off 0.02 | found once, 9.1 ft, off 0.09, not in doubt |
| W 11.0 ft | NOT FOUND | NOT FOUND (its top is not there: a closet front to 6 ft) |
| SW 3.2 ft (low) | found, 6.1 ft | found, 2.7 ft, off 0.03 |
| not in the truth | 4: 10.7 ft (in doubt), 3.1 ft (in doubt), 11.6 ft, 2.9 ft (the last two not in doubt) | 2, both in doubt (4.7 ft, 2.5 ft) |
| laid by itself | 5 walls, 2 not in the truth | 2 walls, both in the truth |
| closes? truth: no | 6 ends meet nothing, said | 2 ends meet nothing, said |

(The figures above are with the final code: the band is the top tenth where the scan is dense, as it is on all three; the real scans' rows did not change when the band was made to deepen on thin scans.)
Caveats on my own truth: I wrote it before any code, from three cuts of the points; the long scan's north wall I ended at the west wall's corner, and both finders run it 1.4 to 3.3 ft on (the picture does show a line going on: I took it for another room's);
the large scan's east and south I could not read at all, and the "low" walls are guesses. I used the three real scans to find faults while building (the local ceiling, the pairing, what not to filter), so they are not an untouched test of the new finder; the present finder was not tuned by me.

## 6. Which finder the verb uses, and why
**The present finder stays the verb's.** `--by-ceiling` reaches the new one (`found::find_level_by(.., Method::Ceiling)`, `found::find_by_ceiling`). The contract's condition, better on all three real scans, is not met:
- small: the new finder is better (it lays the two sure walls by itself, at the right size and place, 4.2 ft and 4.5 ft, where the present finder finds one wall 5 ft too long, in doubt, and misses the other).
- long: the new finder is safer and not better: it lays 3 walls by itself and all three are in the truth (N, W1, E1); the present finder lays 10, 3 of them not in the truth (a 22.9 ft wall, a 7.0 ft one, a 3.2 ft one). But the new finder breaks the alcove's south wall into 3 pieces of 2 to 3 ft, finds the east alcove wall not at all, and the present finder finds both (twice, once).
- large: the new finder is safer: it lays 2 walls, both in the truth; the present finder lays 5, 2 not in the truth, and runs the north wall 14.7 ft across the notch. It finds NA and NB only as short pieces in doubt and the present finder finds NB (wrongly joined) and EW at 10.6 ft against 9.1 ft.
So: the new finder wins on never laying a wall that is not there (on all three) and on the small room; the present finder wins on completeness on the long room, and on every made-up house (below). Why it might still be better on a fourth real scan: it does not depend on a face reaching the floor, so a wall behind furniture is found from its top; it takes a tenth of the time (1.6 to 2.3 s against 6 to 10 s); it never invents a thickness. Why it might not: it needs the top of a wall to be in the scan (a closet front to 6 ft, a soffit, a
lowered ceiling over a wall's top, and the room's ceiling not read, all lose a wall), and it sees a wall's top at a tenth of its points.

## 7. The made-up houses (the 65 of the earlier builders' table, `w189/tab/`, scripts there; `present.txt`, `ceiling2.txt`, `final-compare.txt`)
The present finder's answers on them are unchanged by this work (the default path is the same code; `demos/findwalls.sh` steps 1 to 24 passed on it, below). With `--by-ceiling`: 28 of 65 are the same in walls, thickness, doors, windows and closing; 37 differ. By cause:
1. Extra walls, mostly pieces of one wall, where the top of a wall is broken in the picture (a part of its top not dense enough, or a partition's top joined to the wall's): S10 (does not close), HID60, HID90, TWOCEIL (7 for 5), N2masonry (7 for 6), N3bank, N8hid (6 for 4), N9bank (8 for 6), J-big (11 for 9), J-soffit (8 for 5, closes), J-fire, J-dens (7 for 5), N-dens25, N-storey2 (7 for 5).
2. A partition not paired and no thickness told, because a wall's two faces were not both seen in the top band or the pairing was rejected: N1cabin and N10cabin (the closet's 3 1/2 in wall), J-stub, N-stub41 (the 4 1/2 in stub, 1 ft 6 in doorway), J-curved (found a 4 in wall in a curve that is not one).
3. Said not to close where the present finder says it does (walls found right but an end not meeting): T33, N3bank, N6rooms, J-neighbor, J-garage, J-storey, N-soffit3, N-garage2: the same four walls found, an end short of its neighbour by more than a foot.
4. Better than the present finder: J-stub, N-hall4, N-curved5 (4 walls laid, the curve not made into walls where the present finder laid 6) and N10cabin (open either way).
The deeper band (top quarter) on thin scans took the thin houses (S20 to S60, LONG40, L40s, T60d) from different to the same; before it, 22 of 65 were the same.
A made-up house is not the test (call 189 point 3) and none of these is an expectation moved: the present finder is still the verb's, so no expectation (a test, a demonstration step) moved; the one demonstration step added (step 25) is the new finder's own.
Eight angles (my own run on the made-up house at 0, 17, 33, 45, 60, 77, 90 and 123 degrees, before the band deepened, density 100 and so unchanged by it): the present finder 5 walls each time, partition 4 1/2 in, closes; by ceiling 5 walls at 0, 17, 33, 123; 6 at 60 and 77; 7 at 45 and 90 (extra pieces, and no partition thickness at 90): worse, and it depends on the angle (the pixel grid), which the present finder does not.

## 8. Where it still fails on a real scan, bluntly
- It breaks a wall whose top is not dense in the picture into pieces (the long scan's south wall, 3 pieces of 2 to 3 ft), and a wall found in pieces is then "meets no wall at either end" and in doubt, so little is laid: small lays 4 walls but the room does not close (E and W are pieces of 2 ft), large lays 2, long lays 3.
- It finds no wall whose top is not in the scan: the large room's west side (a closet front to 6 ft), the long room's step.
- It misses what is not a wall's top but a person would call a wall by the 4 ft cut: the small room's east (a wardrobe, ceiling 6 ft 11 in there), the jog.
- It is wrong by up to 0.85 ft across on the long room's west alcove wall and its S wall at an angle of 18 degrees where my truth has 10 (my truth is only good to about a half foot).
- On made-up houses it is worse on 37 of 65 and depends on the angle the house lies at.
- Nothing about openings was tested on a real scan: none is readable in the three; the made-up room's doorway and window are read right (unit test, step 25's sibling, and the first finder's rules).
- Honest limit of what I tried: I did not get to a version better on all three; two ideas dropped with the reason (level and fat patches), the local ceiling kept.

## 9. Done list (everything below was run on the final code unless it says otherwise)
- Tests: `cargo test -q -j2 -p plumbline-plans`: 140 passed (124 before; 16 new in `found_189_tests.rs`: 15 written first and run red, quoted above, and one, `a_thin_scan_is_read_by_a_deeper_band_and_says_so`, written AFTER the band was made to deepen, said in its comment). `-p plumbline-world --lib`: 76 passed.
  `-p plumbline --bins`: 0 tests (the verb has none of its own). NOT RUN: the 31 integration test programs of `plumbline-world` and of the cli: the disk (3 GB free) cannot hold their links (each 120 MB; the first attempt ended "ld terminated with signal 7 [Bus error]",
  disk 100% full); I ran `cargo clean -q -p plumbline-plans -p plumbline-world -p plumbline` in my own workspace and deleted two old `plumbline_app` test programs there (900 MB), and ran the plans, world lib and cli sets. My change touches no world code and no number any world test checks.
- Demonstrations (PLUMBLINE_BIN=wtP/plumbline/target/debug; nothing built by them): `demos/scan.sh` PASS; `demos/planwalls.sh` PASS (drawn.rs untouched); `demos/findwalls.sh`: one full run, 222 checks ok and none FAIL, steps 1 to 24 complete, on the present finder; it
  was then cut at step 25 because I edited step 25's text while the run was inside it (bash reads a script as it runs: the run read the old text and its old bow assertion failed: my edit, not a break); step 25 as committed passes alone (`w189/step25.sh`: steps 1 to 5 and 25 on the final binary: PASS).
  The full script was not run again from the top: it takes over an hour on this machine. Step 25: the phone scan of step 21 by `--by-ceiling`: 4 walls, none thick, none in doubt, the west wall's two passes joined and said, 3 walls about 12 ft and 1 of 14 ft, the open north-west end said, all four laid Assumed, a second laying refused.
- Gates from the working copy's root: `tools/gate-language.sh` PASS, `tools/gate-egress.sh` PASS, `tools/gate-clientdata.sh --all` PASS; `plumbline/tools/licenses.sh` exit 0 (no new crate: nothing linked).
- Patch: `.../scratchpad/review/call189.patch` (10 files, 1,728 added, 62 removed; `git add -N . && git diff --binary -- plumbline`). It holds: `crates/plans/src/found_189.rs` and `found_189_tests.rs` (new), `lib.rs` (two lines), `found.rs`
  (`Method`, `find_by_ceiling`, `find_level_by`, `find_one` taking the finder; and `pub(crate)` on `Ceil` with its fields and methods, `Grid`, `ceiling_of`, `density`, `unlikely`, `median`, `open_ends`, `edge`, `figure`, `end_word` and the constants the new finder reads: no logic of the first
  finder changed), `crates/cli/src/found.rs` and `main.rs` (`--by-ceiling`), `demos/findwalls.sh` (step 25), `README.md`, `NOTICE`, `LICENSES.md`. `PORTMAP.md` unchanged (no public item in `building*` or `editing*` changed).
- Scratch (not in the patch): truth files `w189/truth_*.txt`; pictures `w189/*.png`; scripts `w189/pic.py elev.py ov.py score.py run.sh`; tables `w189/tables.txt`, `w189/tab/*.txt`; the Cloud2BIM files read `w189/src/`; red run `w189/red.log`.
- Nothing is Known anywhere: every wall, opening and thing the new finder hands the verb is Assumed, sourced to the scan, and laid only with the person's word where it is in doubt; a second laying is refused; the wrong measure and the wrong way up are said first (the up check and the unlikely-house check wrap both finders).
