# W-177 report (call 177, a trade's view of the building, the window)

All five parts built and proved by `demos/tradesw.sh` on frozen build `scratchpad/w177/bin4` (tree source == that build; final run `w177/run5`, PASS). cargo test --workspace: 582 passed, 0 failed (46 suites). d2.sh and moneyw.sh PASS on bin4 (pictures to scratch). Gates (language, egress, clientdata) and `BIMCRAFT=... tools/portmap.sh` PASS (312 items). Not committed.

## Parts
1. Trades row (All, Foundation, Framing, Outside, Plumbing, HVAC, Electrical, Inside) at the head of Show; sets the existing switches; status line says what was set; All restores the switches from before the first trade. Files: `app/src/trades.rs` (new), `panels.rs`, `model.rs` (Show.sizes, Doc.trade, Doc.before_trade), `main.rs`, `map.rs` (new `state<TAB>Switch<TAB>on|off` lines in the map).
2. Sizes switch + tags: model words in world (`Placement.tag/stack`, `run_tag`, `trade_word`), drawn by `size_tags` (panels.rs); on the map as "Tag <words>".
4. `widen_runs` (scene.rs): runs >= 2 dots on screen; stack at true size and tagged "... stack".
3. Frame: world `building/frames.rs` (`frames()`), `members::members_placed`, shared `joist_run` and `derive::rafter_layout` (the counts use the same code); drawn under Framing (joists, rim, rafters, beams, posts). Floor with no direction said: no joists and tag "joists: direction not said".
5. Clash room: `Clashes::in_the_same_space/set_against/hole_rows`, `clash::rooms_of`; sheet list (room first: "kitchen: 11' 0", ..."), "Set against: not clashes" apart, one row per hole; Show count, marks, carry line use the model's counts; marks wear "in <room>" / "outside any room".
Tests: `crates/world/tests/call177.rs` (6). PORTMAP rows added. Captured README updated.

## New checks, red and green (picture checks)
- pipe/duct/wire colour per trade: green 07 pipe 346, duct 0, wire 53; 08 duct 6193, pipe 0, wire 0; 09 wire 1737, pipe 0, duct 0. Red: the same "pipe present >= 150" applied to the HVAC picture reads 0 (fails).
- tags inside window and written: pill fraction at the tag's map place after 0.59-0.73, on the opened picture 0.00 (fails there). Eight tags checked; no two tags overlap; none as opened / Outside / Sizes alone over the solid house.
- frame: roof box lumber 0.74 (rafters) vs 0.00 on step-03 picture (walls framed, none named) and on the committed 2-framing.png; floor box 0.997 vs 0.368 (03) / 0.35 (committed).
- 2 dots: 2 in vent, far off, min width over rows 2 (11 rows). Red: same program with PLUMBLINE_RUN_DOTS=0 (true size, the draw path before call 177): 2 rows only, check fails (>= 6 rows). Not the literal committed binary: an env hook (PLUMBLINE_RUN_DOTS, default 2) was added for this.
- clash tags: "clash in kitchen / living room / utility", "outside any room" on the map.
- State checks per trade read off the map (see states in run log) and status-line sentences; price unchanged for both houses.

## Choices not made by the decision
- "Roof's covering off so rafters show" = See through on (roof planes glass); no new roof-covering switch. Frame pieces are never glass.
- Foundation: ground off + See through on keeps footings, foundation walls (cast or below ground) solid. Pad footings tagged/drawn as before (under the ground plane).
- Foundation/Framing also turn Mechanical/Electrical/Plumbing off; Outside/Inside leave them on. Outside sets ground, finishes, roof on; Inside picks the lowest above-grade level with a wall.
- Tag wording: round "N in <word>", rectangular "W x D <word>", unknown size "<word>, size not known", others the type's own name; stack: "3 in waste stack". Trade words: waste, cold water, hot water, supply, return, exhaust, refrigerant, vent, power, lighting.
- Pieces end to end: same system and words, ends within a foot, parallel: only the longest wears the tag.
- Tags only appear where the work is seen (see through, frame shown or ground away); Sizes alone over the solid house says so in the status line. Foundation things tag with the ground away, beams/posts with the frame.
- Carry line ("it is in ...") now excludes set-against (call 175 meaning): a change of existing behaviour.
- Beams and posts are no longer drawn without Framing (decision text); the Framing row appears when frames/beams/posts exist; the Ground row also for pad footings.
- Demo house for part 3 is made at the command line from `sixteen` (beam, 2 posts, 2 pads, joists on both floors, direction said only on the second, rafters on the roof): the sixteen house has none.

## Not right (picture by picture)
- 07 plumbing: pipes are 2-3 dots wide but pale grey-pastel against the glass house; hot water barely reads. Tags (about 20) crowd the middle.
- 10 framing: second-floor joists read as a nearly solid tan slab at this distance; the first floor's lack of joists is only the small amber tag. Rim and joists at the same depth look fused. Wall frames with no openings' headers not touched.
- 11 foundation: solid but the strip footings read as pale slabs; "foundation wall, 8 in concrete" repeats on every piece.
- 13 inside: marks/pipes from other trades all on; a duct runs through the cut wall.
- 15 far: house tiny after 5 wheel clicks; runs are visible but only a few tags survive.
- 16 clashes: marks over the solid house float over walls (pre-existing); "outside any room" tags repeat for clashes inside walls.
- 17 sheet: good; "where" column wide, the set-against table lists wall-set items as "outside any room" (they are in a wall).
- Rafter section uses the plane's slope but rafters' tails/plumb cuts and ridge are not modelled; blocking, hangers not drawn (said by the model).
- Window runs were done on one frozen build (bin4); older bins deleted.
