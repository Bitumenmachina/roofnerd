# W-179B report (call 179 part B items 1 to 4, command line only)

Working copy: .../scratchpad/wtB (plumbline/). Not committed. Patch: .../scratchpad/review/call179B.patch (7 files).
Disk: the build folder was cleared once by the coordinator when the machine filled (not by me); rebuilt, no `cargo clean` needed.

## Built
- carries.rs: `over` pub(crate); `Body`, `bodies()`, `GoesWith`, `goes_with()` (walls above, footings, floors, roof edges that lie on a wall's line, by `over`).
- editing/stretch.rs: `move_wall` = `step` (call 170 rules, unchanged) + `bring_along` (B1, B2) + `refuse_crossing` (B3) + `read_move`/`way_said` (B4).
- building.rs: `reach_cautions` (room past its level's floor; wall off a footing that has no wall on it, nearest parallel wall within 10 ft).
- cli main.rs: `move --by` uses `read_move`; `check` prints `reach_cautions`.
- demos/stretch.sh: steps 11 to 18 (existing 1 to 10 untouched, pass as written). PORTMAP.md: 5 rows.

## Items
- B3 done. Refusal names the wall in the way and how far it can go (faces touching, to the 1/16 in). Bodies are compared less half a thickness at each end, so a corner/tee joint is not a crossing. Walls it meets (ends touching/on span) are exempt. It runs AFTER the older refusals (shrink to nothing, ends held, opening outside), so those keep their words.
- B4 done. `--by` reads "4,0" (feet, as before), lengths with units ("4 ft, 2' 6\""), directions ("4 ft west", "3' 6\" east and 2 ft north", "6 in south"); a bare "4" is refused; east and west together refused. Said as "moved 4 ft west", lengths as "11 ft 6 in". A wall is named with its level ("wall 2 on second") whenever more than one level has walls (also floors, footings, room lines), else plain so old steps read the same.
- B2 done. Room line ends at the moved wall's ends go to its new ends; ends on its span by the square part (same rule as a butting wall). Said.
- B1 done. Followed and said: walls above (chain upward, with their neighbours), floor edge (area before/after), footings (the one under it plus those meeting its ends or lying against it), roof footprint edge (plan and surface before/after, re-solved), room lines. Left behind, said: anything lying along the wall that runs past its end (would turn). `--alone`: moves the wall only, says "it left where they were: ...". `check` cautions as above.

## Hand figures against the program
| figure | by hand | program |
|---|---|---|
| floor before / after | 384.00 / 448.00 SF | 384 / 448 |
| footings before / after | 80 / 88 LF | 80 / 88 |
| room before / after | 364.25 / 426.25 SF | 364.25 / 426.25 |
| roof plan before / after | 384 / 448 SF (+64) | 384.00 / 448.00 |
| roof surface (x1.118034) | 429.33 / 500.88 SF | 429.33 / 500.88 |
| price rise (floor $10/SF, footing $20/LF, wall face $3/SF) | 640 + 160 + 216 = $1,016.00 ($800.00 + walls $216.00) | 7,600.00 to 8,616.00 |
| moved back 4 ft west | all as before | floor 384, footings 80, room 364.25, price 7,600.00 |
| 30 ft west | refused, file same; 23 ft 6 in | refused, 23 ft 6 in, byte for byte |
| partition at 12, 14 ft west | 11 ft 6 in (23.75 - 12.25) | 11 ft 6 in; 11 ft 6 in then taken |
| second level | upper east wall + neighbours follow | said, lines (28,0)-(28,16) on both |

Extra houses (not in the contract): L (30x24 notch): east wall 4 ft east 528 -> 576 SF; inner wall 3 east 528 -> 564; step wall 3 north 528 -> 576; 5 ft west inward 528 -> 468: all equal hand figures (+/- 12/16 x distance). House turned 30 degrees, east wall moved square to itself by 4 ft: 528 -> 576. Collinear two-wall south side (floor edge runs past one wall): floor left behind and said; check says nothing more there.

## Tests / gates
`cargo test -q -j2 --no-fail-fast -p plumbline-world -p plumbline`: 433 passed, 0 failed. All plumbline-only demonstrations pass except `field-job.sh` (a helper that needs an argument; not a demo). Gates (language, egress, clientdata --all) PASS; portmap PASS (320 items).
No new Rust tests were written (the demonstration carries the figures); a world test for B3/B1 would be a good addition.

## Expectations moved
None. My first build moved two (call170.rs:229 and :279, wording of two refusals); I instead ordered B3 after the older refusals, so both stand unchanged. (Message wording "wall 2 followed: 4 ft longer" etc. unchanged; `feet_said` in the existing refusals now writes "11 ft 6 in" instead of "11' 6\"" - no test or demo asserted the old form.)

## Choices
- Wall names carry their level only when more than one level has that kind of thing.
- Anything lying on a wall's line goes with it for every wall, not only outside walls (an upper partition over a lower one follows).
- Roof attaches via the wall whose top is within 2 ft of the eave (carries' EAVE_REACH).
- Footings that meet the moved wall's ends or lie against it follow like walls do.
- Status/source of every moved floor, footing, roof, room line: worse of old and move; source gains "moved with wall N (how)".

## Does not follow yet / not right (blunt)
- Contract hand claim wrong for --alone: `check` cannot say the room "reaches past its floor" after the east wall is moved alone, because the north and south walls then end 4 ft short of it, the room's walls no longer close and it has no outline. The caution works where the room does close (floor drawn short; floor left behind): tested by hand (3 ft 9 in past floor 1) but not in stretch.sh.
- Not followed and not said: a wall that follows and turns may cross some third wall (only the moved wall is tested for crossing). Doors/windows in the upper walls of the chain are handled by the same call 170 rule; an upper wall that refuses refuses the whole move.
- A footing/floor/roof on a level above or below the moved wall's own level is not found except through the chain (footings are only those of the wall's own level).
- "wall off footing" caution is a heuristic (parallel, <= 10 ft, footing with no wall on it); could fire on a footing drawn before its wall.
- Red proof of the new demo steps against the old program was not run; they obviously could not pass on it (the old reply had no floor/footing/roof lines) but I did not run it.
- crates/app calls `move_by`/`move_wall`: its window moves now take floors/footings/roofs along too; I did not look at how the window words it.
