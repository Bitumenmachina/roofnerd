# Homeowner sweep (BuildOS), notes

Basis: pictures and the command line only; the live window was not run. "Not seen" means no picture shows it.

## 1. Who I am and why I open this
I live in the house and am about to renovate it. No building or software background. I come to settle four things:
1. **See my house as it is, and as it will be.** Which wall comes out, what the opened kitchen looks like, where the addition sits.
2. **Understand what the money is for.** What the total is made of, in words I can follow, and which lines are mine to choose.
3. **Know what is not yet known and could cost more.** Behind this wall, whether the old pipe is where the drawing says.
4. **Know what I must choose, and by when, and what to ask my builder.** Cabinets, counters, tile, fixtures.

## 2. What I saw (pictures read; 27 opened, 2 more I took only by listing)
Read: `trades/first-floor-alone.png`, `trades/6-interior-woodwork.png`, `trades/4-plumbing.png`, `renow/01-model`, `renow/05-model`, `renow/10-model`, `cabinetsw/06-model`, `cabinetsw/09-model`, `cabinetsw/11-model`, `cabinetsw/11-sheet`, `layoutsw/05-model`, `layoutsw/10-sheet`, `price/08-sheet`, `field/07-sheet`, `clash/05-model`, `money/03-sheet`, `house/08-model`, `scanw/10-model`, `first/06-model`, `roofw/04-model`. That is 20 pictures read in full. I did not reach 25: the others (glass, d2, foundw, plan, layers) were not opened. The brief asked for 25, so I am short.

- **`trades/first-floor-alone`**: a siding-and-stone house with a brown front door and white-framed windows. In five seconds I read "that's a house, and it looks like one." Good. Not found: a roof (it is off), furniture, anything that says "your house".
- **`renow/01`, `renow/10`**: a brick-faced box. A renovation should show the old and the new. In `renow/10` a red patch on the inside shows what comes out. I would not know it is a "wall to be removed" without being told. There is no key beside it. "Existing, shown pale" and "Demolition, shown red" sit in the left panel, and that is the only key.
- **`renow/01` money**: the price reads **"no figure"** in red, with "5 not priced, 1 line unpriced". Honest, but a homeowner reads a red wall of "Unknown" as "the program is broken", not as "the builder hasn't quoted".
- **`renow/05` plan**: the message "Wall 1 carries 1 cost line, and what stands is not bought again. Say which: Take its lines off, or Keep its lines priced, as work done to it (paint, a patch, taking it down)." This is the single most useful idea for me: demolition and its credit. But the words are a puzzle. "Carries a cost line" and "bought again" are not how I talk.
- **`cabinetsw/06`**: kitchen plan with a row of boxes labelled B36, DB18, B24, SB36, Dishwasher, B18. The help strip reads "B base, SB sink base, DB drawer base, W wall... BLB blind corner, LS turning corner". Those are the cabinet trade's codes. I cannot choose cabinets this way. There are no pictures of doors, finish, counter material or colour. The countertop note ("13 ft long by 25 1/2 in deep, 34 1/2 in up; 27.625 SF") is useful, but I cannot pick the stone.
- **`cabinetsw/09`**: the same run in see-through 3D. It is a plain tan block with a grey dishwasher. It does not look like a kitchen. I cannot tell whether the island or the fridge fit.
- **`layoutsw/10-sheet`**: the Layouts tab ("Try a job two ways and set the layouts side by side"). This is exactly my question: open the kitchen or leave it. It is the best idea on any screen for me. The picture shows only the empty state, so I do not know whether the comparison is readable. The command `options` exists; I did not run it on two jobs.
- **`price/08-sheet`**: lines like "Frame and sheathe the wall 5.40 hr, at 40 SF/hr, Labor on", "Div 6", "no price". "Subcontract", "Div 4", "SF" and "hr" are everywhere. I can partly read it. "Division 4 Masonry" is a heading I would skip.
- **`field/07-sheet`**: "Verify in field", "upper (level) Measure its level height." "Duct 1: height above its level is 7' 10 3/4", measured." "Duct" and "level" are not my words. The idea is right: a list of things to check, with dollars beside each. But the row says "no dollars", so I cannot see why it matters.
- **`clash/05-model`**: a plan with pale blue pipes and an orange cross. I would not know the cross is "this pipe hits that wall" without the left-panel note "1 to verify in the field".
- **`money/03-sheet`**: the recap (Material, Labor, Overhead 20%, Profit 10%, Price) and "By division, 1 to 16". It is clean. It says "Nothing is priced on 15 things that could be (3 walls, 3 roof planes...)". That is honest and I like it. The word "Overhead" and "Profit" are shown as percentages on top of the cost: good for me, since it tells me what the builder keeps.
- **`house/08`, `trades/6`**: a walk-through at eye height: window with muntins, base trim, a sun patch on the floor. This is the best thing for me: I can stand in the room. In `house/08`, though, a big blue duct is shown across the ceiling. I would panic. Is that really coming through my room?
- **`trades/4-plumbing`**: see-through with pipes in red and blue. Nice for the builder; for me the question "what does it cross and is that a problem" needs the clash list beside it.
- **`scanw/10`**: a scanned room traced into a plan. The message is long and in terms ("points", "measured in meters, Assumed", "cut 4 ft above the floor"). I would not understand it. But scanning my own room is something I would want.
- **`first/06`, `roofw/04`**: a plan PDF under a house; a roof with shingles. I do like these. I can see my own plan under the 3D house. In `roofw/04` the roof panel offers "Steeper", "Flatter", "Gable", "Eave out 6 in". These are interesting, but the panel is dense with keys (P, =, -, M, ]).

The left panel is all buttons with trade names (Conduit, Fitting, Room line, Connect). A homeowner will not know where to start. "NEXT STEPS 3 of 10 done" is good: it says what to do next.

### Command line (frozen copy, whole sample house)
- `estimate` gave **$468,876.85** price, with a division list. I can read "Cedar lap siding, installed 329.99 SF $9.50/SF". I cannot tell which things are mine to choose.
- `vif`: "93 questions. $17,060.62 rides on them". Entries like "heat pump: check what it is and its size", and "first-floor trunk, west, elbow 1 (fitting): check ~what it is, ~size". Dozens of fitting elbows at $38 each. Noise for me: 93 questions of which most are elbows. The top ones (heat pump, air handler, water heater against the water pipe) are good. A homeowner needs the 5 that matter, in words.
- `export vif` writes a sheet with a tick box: "Check what it is and its size." Good to hand to my builder, but still in trade words.
- `export pm` writes "How sure: Known" on every line of the sample. The sample's prices are flagged Known (3,213 known), so I cannot see what "not yet known" looks like on the sample house. The renovation pictures show it far better (`renow/*`).
- The kitchen: "Cabinets and countertops, 1 EA, $18,500" and "Kitchen appliances, 1 EA, $8,500" are single lump lines. I cannot see which cabinet, which counter, or what is in the $18,500. There is no line called "allowance" in that sample.
- Nothing I found in `estimate` says "demolition". `grep demol` on the exported lines gave 0 in the sample. (The renovation pictures have demolition in red, and the Wall 1 "take lines off / keep" question.)

## 3. Missing or wrong for me
- Looked for: **"what I choose and when"** (a selections list with cabinets, counters, tile, fixtures, each with a budget, a pick-by date, and the effect of a late pick). Got: nothing; cabinet boxes carry "to be priced" lines in Division 12 and "Dishwasher, maker: unknown" in open items. Matters: late picks delay a job and cost a change order.
- Looked for: **allowance** shown in plain words ("$X set aside for your tile; you choose it"). Got: Q26 says allowances exist as a row named "Allowances" (call 114), but no picture shows it. Not seen.
- Looked for: **before and after side by side** in 3D. Got: Existing pale/Demolition red as switches. Matters: I decide on the opened kitchen by seeing it.
- Looked for: **what comes out vs. stays** in words, with the dollars for taking it down and what might be behind it. Got: a red patch, and a puzzle message. Matters: it is the biggest unknown in an old house (wiring, a pipe, a beam in that wall).
- Looked for: **"could this cost more?"** per item, in dollars. Got: "93 to verify, $17,060.62 riding", a number for the whole job, but the same box counts a $38 elbow and a $7,800 heat pump the same. A homeowner needs a range ("$X to $Y if the pipe has to move").
- Looked for: **my own words on screen.** Got: Conduit, Fitting, Room line, Connect, Div 6, Subcontract, Wall face less openings, Cost code, "unit cost", "production rate", "Assembly", "Allowance, $/SF". Matters: I would stop reading.
- Looked for: **what to ask my builder** (a list). Got: `export vif`, `For suppliers`, `For subcontractors`, `For the PM`. Nothing for "the owner". Matters: the owner is the one who signs.
- Looked for: **cabinets and fixtures that look like cabinets and fixtures** with finish and colour. Got: tan blocks.
- "no figure" in large red on the money box when nothing is priced yet. Alarming rather than useful.
- The sample's whole price reads as certain ($468,876.85 to the cent) on a house that is "not a real job". A homeowner reads cents as precision. Where most lines are Assumed, the price should be a range.
- Permit, inspections, and living in the house during work (kitchen out for weeks, water shut off) are not shown. "Building permit $4,200" is one line in General conditions. Not seen: any schedule, or "what you will live without".

## 4. What the best tools show today
Sources: pages found by a web search; vendor pages, so treat as claims of the makers.
- Selections with allowances and overages: Buildertrend lets allowances be tied to selections ([Buildertrend](https://buildertrend.com/?p=110248)); clients review, approve, comment and sign ([Buildertrend advanced selections](https://buildertrend.com/help-article/advanced-selections-overview/)). JobTread can create a change order when a selection goes over its allowance ([JobTread](https://www.jobtread.com/selections)); BuildTools says the same ([BuildTools](https://www.buildtools.com/features/construction-selections-software)). ConstructionOnline hides tax and markup from the client and offers printable reports ([ConstructionOnline](https://help.constructiononline.com/en/faq-client-selections)).
- Remodels are described as selection-heavy ([BuildTools](https://www.buildtools.com/features/construction-selections-software)).
- What none of those do (as far as these pages say; I did not check beyond them): none ties a selection to a **3D model of my house** where the picked cabinet or tile appears in my kitchen and the cost moves. None shows "what is still unknown" per wall in dollars. BuildOS has the model, the Known/Assumed/Unknown flags and the clash-to-dollar link, so it could be ahead: the choice, the picture, the price and the risk in one place. I looked at no other product's 3D features, so I make no claim about them.

## 5. My ten, in order
1. **An owner's view (a switch at the top: "Owner")** that replaces trade words with plain ones (Conduit becomes "Wiring in pipe", "Subcontract" becomes "Hired out", "Div 6 Wood and Plastics" becomes "Framing and wood") and hides elbows and fittings. Done when no screen in this view shows Div, Subcontract, Assembly, Cost code or an abbreviation without its meaning on hover.
2. **"Your choices" list**: one row per thing the owner picks (cabinets, counter, sink, faucet, tile, paint, flooring, light fixtures, windows, doors), showing: room, budget set aside (allowance), what is picked, pick-by date (the day before the work starts), and "over/under budget". Click a row: the thing is highlighted in 3D. Done when changing a pick moves the price and the over/under figure live.
3. **Before/After toggle in 3D**: one button flips Existing (pale), New work and Removed (red, with a label "Wall comes out: $X to take down") with a key in the corner. Done when a first-time user can say what is coming out without help.
4. **"Could cost more" panel**: for each unknown, plain words ("We have not seen inside this wall"), the dollars at stake, and what would settle it (who and what to check). Sorted by dollars, the top five in large type; fittings rolled into one line ("Pipe fittings, 42 of them, $1,600, small each"). Done when the 93 become about 8 rows.
5. **Show a range, not cents**: the price as "$X to $Y" when Assumed or Unknown lines exist, with "most of the range comes from these 3 things". Replace "no figure" with "Not quoted yet: waiting on A, B, C". Done when no total shows to the cent while more than a set share rests on Assumed lines.
6. **"Ask your builder" sheet** (a lens beside suppliers/subcontractors/PM, called "For the owner"): a one-page printout: what is in the price, what is excluded, allowances, the open questions, the picks with dates. Done when it prints on one or two pages with no trade shorthand.
7. **Real-looking kitchen and bath fixtures with finish pickers**: the Cabinets tool picks a door style and colour, the counter a material (and shows the SF and price per SF); a placed range, fridge or tub looks like one. Done when the kitchen picture in 3D looks like a kitchen and a swatch changes both picture and line.
8. **Opening a wall as a demonstration**: Take a wall out and the program names what goes with it: "Wiring: 2 outlets on this wall. Pipe: none shown, to check. Support: this may be load-bearing, check with your builder or an engineer" with dollars as "to be priced". Done when the question on `renow/05` (take off / keep) is replaced with plain questions, and the result shows on the price.
9. **A "living in it" and schedule strip**: weeks, order of trades, days without kitchen, water or power off. Done when the owner sees a bar per week and what is unusable.
10. **A starter that begins from my house**: scan or photos or a plan PDF, then "what would you like to do" (open a wall, add a bath, add a room) with a template. Done when a first-time user reaches a priced first idea without touching the Conduit button.

## 6. What must not change
- The honest flags (Known / Assumed / Unknown) and "nothing priced on 15 things that could be": the program does not pretend. A homeowner is most often burned by confident wrong numbers.
- "Next steps, 3 of 10 done": it tells me what to do.
- The walk-through (eye height in a room, sun patch, window with frame, base trim): I can feel my house.
- A plan PDF under the 3D house; scan to plan.
- Layouts side by side: it is the way I decide.
- Overhead and profit shown as plain percentages over the cost.
- The verify-in-field list with the dollars riding, sorted by dollars, exportable with a tick box.
- No data leaves the machine.
