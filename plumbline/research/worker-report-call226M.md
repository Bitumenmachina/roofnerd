# W-226M report (notes as I go)

## Run 1 (before any code)
- Debug build, sample whole house, `try wall` (a 10 ft partition, level-0007, type-0003) with a new `--timed 5` flag I added (measuring aid, crates/cli/src/feel.rs): 0.290, 0.321, 0.355, 0.333, 0.309 s; least 0.290 s.
- Release build started at 7.0 GB free; shared disk fell to 2.8 GB while it compiled plumbline_world; per the under-3-GB rule I stopped it and deleted target/release. No release figure yet; retry if space returns.
- Baseline copies of the sample house text (estimate, takeoff, vif, clashes) made BEFORE any change by running the unmodified debug binary; `sixteen` sha 880d9fd3c364b5f7 ok.

## Run 2: release build (before; world code unchanged, only `try --timed`)
Release (cargo build --release -p plumbline, built 6m38 cold): five tries of a 10 ft partition on the sample house, least of five: 0.066, 0.077, 0.073, 0.075 s over runs (machine loaded, idle class, load avg 4.5 on 4 cores). Where it goes (release): writing the whole job out 0.013 s, reading it back 0.028-0.046 s (together ~55-60% of the try); the clash test ~25 ms. A wall move (roof, floors followed): least 0.070 s. Debug build: least 0.290 s.
=> UNDER a tenth of a second in the build a person is given. Per call 226 point 1 (the put-back changes only if over a tenth) no re-engineering is required; figure before = after.

## Run 3: RED (tests first): crates/world/tests/call226m.rs (new, 11 tests)
`cargo test -p plumbline-world --test call226m` -> does not compile: E0425 cannot find function `length_said` in module `units` (x9, lines 34-42); E0425 cannot find function `cabinet_rule_away` in module `tables` (x4); E0609 no field `stocked_heights` on type `&Rule` (x2+). Red by compile; they turn into behaviour reds once stubs exist (shown after).

## Run 4: code in, tests green
world 723 (712+11 new), cli 69 (64+5 new), ifc 111, sheets 7, plans 187, roofing see below; 0 failed after moving the listed expectations. cabinets' "Stocked heights" column placed LAST (so every older table/test reading 11 columns still reads): call215m tests pass untouched.

## Done. Patch: review/call226M.patch. Full report handed back through the hand-back message (same content).
Release try figures: before 0.066-0.077 s (least of five), after 0.074-0.078 s: unchanged, under a tenth; put-back left as it is. Demos: 61 of 62 pass; cabinet-rules.sh:129 expects the old 11-heading line (handed back).
