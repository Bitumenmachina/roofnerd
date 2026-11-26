# Design judgment, 2026-11-26 (notes written as I go; final sections below)

## Raw notes from the pictures

P1 (1600x960, 3D, nothing selected):
- Top-left bar 10..690 x 10..52: logo, title, Save/Undo/Redo each with shortcut chip. Top-right bar 866..1590: PRICE, figure, "92 to verify, $17,060.62 riding" boxed, Saved (green), 3 counts (ink/amber/red).
- View bar 940..1236 x 68..100: 3D, Plan T, Walk V, Frame F.
- Job panel 10..262 x 64..860 (252 wide): Next steps, Levels (3 rows + switch), Show: 8 pill buttons + 12 switch rows w/ colour dots, Drawings, Bring in (cut off), "more below".
- Details panel 1246..1590 x 64..908 (344 wide): job, price, 16 divisions, markup, level quantities, "more below".
- Tools bar 272..1236 x 810..912: help line, 3 headings, 18 tools ~48x44, label ~10px, labels collide ("Room line|Cabinets", "Fitting|Equipment|Fixture").
- Bottom: status chip left, Job/Details/Tools switches + 3D right.
- House occupies roughly x 410..1185, y 165..780.
- Colour dots: Ground grey, Finishes brown, Framing tan, Architecture grey-blue, Roof grey, Mechanical light blue, Electrical amber, Plumbing cyan, See through blue, Sizes dark, KAU brown, Clashes red. Plus blue switches, green Saved, amber assumed, red unknown, blue selected tab. Amber used for both Electrical and Assumed: collision.
P2 (plan, Cabinets tool in hand): message card over plan top; a "BOXES / CORNER BOXES / APPLIANCES / MARKS / KEY" legend block 272..1236 x 675..800 — a reference sheet floating over the drawing; tool fields row "Cabinets [switch] COUNTERTOP" + help text; compass card 270..370 x 64..180 large; Lettering Smaller/Larger on status bar; coordinates. Details shows "no figure" in red at 28px — alarm as headline. Open items list.
P3 (wall selected): Details: wall 2, 5 figures in amber; WORK: 3 segmented; 6 action buttons w/ shortcut chips (Assembly C, Allowance B, Unit cost U, Add cost line, Material N, Labor on or off L) — 7 buttons + 3 segmented + Select all + Take off + 3 Check/Measure links in one panel. Message card covers top of plan with a "More" button. Model visible area tiny.
P4 estimate window 880x960: tabs run off the right edge (Plumbing 94 clipped, no wrap visible: one line cut, no indicator). Recap card good: plain rows. By division good. Counts header. Status bar at foot. Fine structure; too much card chrome is light. Overall this window is the cleanest one.
P5 two plans: each with floor + Layers buttons; no labels on which plan; plans cropped at right (house cut at the divider); trade colours in pipes; coordinates line "On an end".
Note: P1 price $463,527.62 vs P4 $468,050.22; known counts 3,427 vs 3,575 — probably different captures; if same job and same moment, it is a money defect. Lead to check.

## Source notes (as fetched)
- Apple HIG Accessibility (https://developer.apple.com/design/human-interface-guidelines/accessibility, read via its JSON data): text default/minimum: iOS 17/11 pt, macOS 13/10 pt, visionOS 17/12. Control default/minimum: iOS 44x44/28x28, macOS 28x28/20x20, visionOS 60x60/28x28. "add about 12 points of padding around elements that include a bezel ... For elements without a bezel, about 24 points". Contrast per WCAG AA.
- Apple HIG Toolbars (https://developer.apple.com/design/human-interface-guidelines/toolbars): "Choose items deliberately to avoid overcrowding"; "Minimize the number of groups ... aim for a maximum of three"; "Prefer simple, recognizable symbols for items instead of text"; "Prefer system-provided symbols without borders ... Borders ... aren't necessary because the section provides a visible container"; "Keep actions with text labels separate"; "Only specify one primary action"; "Reduce the use of toolbar backgrounds and tinted controls"; "Don't title windows with your app name"; "keep the title under 15 characters"; "Consider temporarily hiding toolbars".
- Apple HIG Sidebars (https://developer.apple.com/design/human-interface-guidelines/sidebars): "show no more than two levels of hierarchy"; "Group hierarchy with disclosure controls"; "Avoid putting critical information or actions at the bottom of a sidebar"; "Consider automatically hiding ... when its container window resizes"; sidebars can float above content in Liquid Glass.
- Apple HIG Materials (https://developer.apple.com/design/human-interface-guidelines/materials): Liquid Glass is a functional layer for controls and navigation floating above content; "Don't use Liquid Glass in the content layer"; "Use Liquid Glass effects sparingly"; "Thicker materials ... better contrast for text"; clear variant over bright content: "consider adding a dark dimming layer of 35% opacity".
- Apple HIG Panels (https://developer.apple.com/design/human-interface-guidelines/panels): inspector shows details of the current selection; HUD panels: "Use color sparingly"; "Keep HUDs small ... Don't let a HUD obscure the content it adjusts".
- Google Earth (web), Google's own help, now at https://developers.google.com/maps/documentation/earth/discover-places-change-view : on a computer, zoom +/-, compass, 3D/2D and My Location are all "At the bottom right"; Settings "In the upper-right corner". Earth's own help names no other always-on control. Google Earth projects: "In the Map Contents panel on the left, select the data layer" opens "an inspector panel with details and options for that layer" (https://developers.google.com/maps/documentation/earth/style-data-layers). So: a contents list on the left, an inspector only on selection, four view controls in one corner. No numbers given.
- Figma, "Our approach to designing UI3" (https://www.figma.com/blog/our-approach-to-designing-ui3/): goal "placing their work center stage"; "Toolbars will float at the bottom"; floating panels reversed: users said they "cramped the canvas, especially on smaller screens", "designs ... peek out from behind them, which was distracting", and "they slowed people down"; "panels are fixed, but still resizable"; panels float only in Minimize UI. An early minimal (label-less) approach was dropped for accessibility: "This was a deal breaker." No type sizes given.
- Shapr3D adaptive UI (support page https://support.shapr3d.com/hc/en-us/articles/7873882619548 returned 403 to the fetcher; search snippet): tools are suggested from the selection (face -> Offset Face, profile -> Extrude, body -> Move/Rotate), "More" reaches the rest. Shapr3D UI refresh blog (https://shapr3d.com/blog/behind-the-shapr3d-user-interface-refresh): "reinforced our Adaptive UI by forming new logical groups and a better hierarchy"; "Labels are larger and offer better readability"; "Buttons are rectangular and use space more efficiently"; filled icons with thicker lines; fixed elements as "anchor-like points"; Section View / Isolate / Measure collected into one bottom-left section that shows on/off. No numbers given.
- SketchUp Toolbars (https://help.sketchup.com/en/sketchup/toolbars): the default Getting Started toolbar names 21 distinct tools; everything else (Large Tool Set, Construction, Drawing...) is turned on from View > Toolbars; Large Tool Set docks only on the left; Options let you hide tooltips and pick large or small icons. Page does not say whether icons carry labels.
- Procreate Handbook, Interface (https://help.procreate.com/procreate/handbook/interface-gestures/interface): 14 controls in all: top left 5 (Gallery, Actions, Adjustments, Selections, Transform), top right 5 (Paint, Smudge, Erase, Layers, Color), left sidebar 4 (size, modify, opacity, undo/redo). Four-finger tap: "The interface will slide away to give you a clean view of your canvas."
- Cities: Skylines (first game) user manual, Steam (https://cdn.akamai.steamstatic.com/steam/apps/255710/manuals/CitiesSkylines-UserManual_EN.pdf): the "City Service Panels" are one row of categories (Roads, Zoning, Districts, Electricity, Water & Sewage, Garbage, Healthcare, Fire, Police, Education, Public Transport, Decoration, ...); "select a service from the city service panel. This will open more options related to the selected city service"; "Some of the city services are unlocked as your city grows." Cities: Skylines II City Corner #1 (https://www.paradoxinteractive.com/games/cities-skylines-ii/news/upcoming-visual-updates): the toolbar "had the most changes", "more round shapes", icons changed "to be more expressive". Neither gives control counts or sizes.
- NN/g Progressive disclosure (https://www.nngroup.com/articles/progressive-disclosure/): defer advanced or rarely used features to a secondary screen; "designs that go beyond 2 disclosure levels typically have low usability"; put "everything that users frequently need up front".
- NN/g Icon usability (https://www.nngroup.com/articles/icon-usability/): "a text label must be present alongside an icon"; labels visible at all times, not on hover; few icons are universal (home, print, search); 5-second rule.
- WCAG 2.2 SC 2.5.8 (https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html): target at least 24 by 24 CSS px, or spaced so 24 px circles do not meet; 2.5.5 Enhanced uses 44 by 44. SC 1.4.3 (https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html): 4.5:1 text, 3:1 for large text (18 pt / 14 pt bold, about 24 px / 18.67 px).

## Measurements (mine)
- theme.rs: MIN_TYPE 11; tool_button 52x44, flex-shrinks to min 44 wide; tool label 11 px Medium; gaps in picture 1 about 3-4 px between tools. Text sizes in code (calls of text()/typed()): 11.0 x101, 12.5 x99, 12.0 x66, 11.5 x43, then 13, 13.5, 14, 14.5, 15, 17, 22, 23, 26, 96, 110: four sizes within 1.5 px carry 309 of ~330 uses, so size gives no hierarchy.
- Contrast (computed, WCAG formula) of Assumed amber #A9690A: 4.46 on white, 4.09 on #F4F5F7, 3.73 on #E9EBEE: below 4.5:1 for the 11-12 px text it is used on. #8A5508 gives 6.21 / 5.69 / 5.20. Red #B93227 5.92/5.43/4.96 passes. ink_faint #5C6570 5.91/5.42/4.95 passes. Electrical dot #EAB308 1.92 on white (a swatch, not text, but it fails 1.4.11's 3:1 for a meaningful graphic).
- Picture 1 controls in view (each a separate press target with its own outline or switch): top-left 3, top-right 1, view bar 4, Job panel 30 (Show link, All levels, 3 level switches, Add level, 8 Show buttons, 12 layer switches, Add drawing, 3 Bring in buttons), tools 18, Details 2 (Set, Set), bottom 4 = 62. Plus 7 keycaps drawn like buttons (Ctrl+S, Ctrl+Z, Ctrl+Y, T, V, F, Del) and 18 tool numbers, and 20 clickable rows in Details. About 62 controls and over 100 bordered or boxed shapes.
- Picture 1 coverage: Job 252x796, Details 344x844, Tools 964x102, top bars 680x44 + 724x44, view bar 296x32, bottom chips: about 673,000 of 1,536,000 px = 44 % covered; the model keeps 56 %; the largest clear rectangle (x 272-1236, y 64-800) is 964x736 = 46 % of the window. At 1366 (screen-1366-model.png, Details off) the clear rectangle is about 1084x440 = 50 %.
- Distinct hues carrying meaning at once in picture 1: accent blue, green (Saved), amber (assumed), red (unknown, clashes), and the layer dots (grey, brown, tan, slate, light blue, yellow, cyan, blue, near-black, brown): 10 or more.
- Price shown three times on the model window's screen at once in picture 1 (top bar; Details in 28-30 px; and again on the estimate window). Known/assumed/unknown counts shown on both windows.
- 1366 (screen-1366-model.png): the top-right bar runs off the window: the unknown count is cut to "■ 2". That is a figure hidden without saying so.
- Possible money defect, for the lead to check, not a design finding: picture 1 says $463,527.62 and 3,427 known; picture 4 says $468,050.22 and 3,575 known for the same named job. Probably two captures at different moments; if not, the two windows disagree.
- price/06-sheet.png (estimate window with lines): the same twelve lines are listed twice, "The estimate, by item" and "The sheet"; every unpriced row says "no price" and "no figure", both red, in two columns: 24 red words in 12 rows. VISUAL A6's "not a column of red sentences" is broken here and in picture 2's Open items.

---

# THE JUDGMENT

Verdict in one line: the owner is right. The window is a control panel with a house in the middle of it. At rest it shows about 62 controls, 10 or more meaning-colours and the price three times, and it keeps 56 % of the window for the model (46 % as a clear rectangle). Google Earth keeps four view controls in one corner; Procreate shows 14 controls in all. "Cramped" is measurable: 11 px labels in 48 px buttons with 3-4 px between them, where a label is wider than its button. None of this is a renderer limit; all of it is layout and restraint.

## 1. Findings, in order of harm

**D-1. Everything is on screen at rest; nothing is one step away.** Picture 1, whole window. 62 separate controls (count in the measurements above), plus 7 keycaps drawn like buttons and 18 tool numbers; two full-height panels; the model keeps 56 % of the window, 46 % as a clear rectangle (964x736 of 1600x960). This is the single cause of "busy". Google Earth on a computer puts zoom, compass, 3D/2D and My Location "At the bottom right" and Settings "In the upper-right corner", and an inspector opens only when a thing is chosen (https://developers.google.com/maps/documentation/earth/discover-places-change-view ; https://developers.google.com/maps/documentation/earth/style-data-layers). Procreate's whole interface is 14 controls (https://help.procreate.com/procreate/handbook/interface-gestures/interface). NN/g: put up front only "everything that users frequently need", defer the rest (https://www.nngroup.com/articles/progressive-disclosure/). Fix: sections 2-5: at rest about 20 controls and at least 90 % of the window to the model.

**D-2. The tools are cramped, measurably.** Pictures 1-3, the tool bar at x 278-1230, y 868-912. Buttons 52x44 in code, shrunk to about 48 wide on screen (flex_shrink 1.0, min 44); label 11 px Medium; 3-4 px between buttons. "Equipment" at 11 px Inter Medium is about 54 px wide in a 48 px button, so it runs to its neighbours' edges: "Room line|Cabinets" and "Fitting|Equipment|Fixture" read as one word each. The number key sits in the top right corner at 11 px, and "Del" on Delete is written over the glyph's corner. Every button has its own white fill and outline, 18 identical boxes in a row: nothing leads the eye and the row reads as a keyboard. Apple: "add about 12 points of padding around elements that include a bezel", and toolbar symbols need no border "because the section provides a visible container"; "aim for a maximum of three" groups; "Keep actions with text labels separate" (https://developer.apple.com/design/human-interface-guidelines/accessibility ; https://developer.apple.com/design/human-interface-guidelines/toolbars). Shapr3D's refresh: "Labels are larger and offer better readability"; groups by logic and hierarchy (https://shapr3d.com/blog/behind-the-shapr3d-user-interface-refresh). Fix: section 4.

**D-3. The Job panel is six unlike things in one scrolling stack, and the most important ones are below the fold.** Pictures 1-3, x 10-262, y 64-860. Guide, levels, layers (two systems), drawings, nine Bring in buttons, the tool chest, six Write out buttons. Bring in is cut off at the panel's foot in picture 1; Tool chest and Write out are never on screen without scrolling. A panel you must scroll to find a command is a list of commands nobody finds. Apple: "show no more than two levels of hierarchy in a sidebar", "Group hierarchy with disclosure controls", and "Avoid putting critical information or actions at the bottom of a sidebar"; document-wide commands (Export, Rename...) belong in "a document menu" next to the title (https://developer.apple.com/design/human-interface-guidelines/sidebars ; .../toolbars). Fix: remove the Job panel; section 5.

**D-4. Show has two systems for one idea, and its colours fight the certainty colours.** Picture 1, x 18-250, y 316-690: eight preset buttons (All, Foundation, Framing, Outside, Plumbing, HVAC, Electrical, Inside) above twelve switches (thirteen at 1366: Trim). Two of the preset words (Framing, Electrical, Plumbing) also name switches beneath them, so a person cannot tell whether a preset is a switch or sets switches. The switches carry twelve dots in ten hues; the Electrical dot (#EAB308) sits a hue away from Assumed amber (#A9690A), and the "Known, Assumed, Unknown" dot is brown, like Finishes. Off rows are greyed as if disabled (Ground, Framing in picture 1). Fix: one Layers sheet, presets as one row at its head (section 5); trade colours only in the model and in the sheet's swatches (section 6).

**D-5. With nothing selected the Details panel is the estimate again.** Picture 1, x 1246-1590, y 64-908: price in 28-30 px, job cost, overhead, profit, 16 divisions, markup, the level's quantities: 30 rows of figures beside a window (the estimate window) that shows the same, and under a top bar that shows the price too. The price is on screen three times. Apple panels: an inspector "displays the details of the currently selected item" (https://developer.apple.com/design/human-interface-guidelines/panels); Google Earth opens its inspector on selecting (style-data-layers page above). Fix: no inspector without a selection. Remove outright.

**D-6. The inspector for a selected thing is a control panel, not a reading of the thing.** Picture 3, x 1246-1590. Five figures in 17-18 px amber (Net face, Length, Gable, Cost, All in), then Select all on this level, a three-way Work control, six action buttons each with a keycap (Assembly C, Allowance $/SF B, Unit cost U, Add cost line, Material N, Labor on or off L), a cost-line card with Take off, then As drawn with three Check/Measure links: 16 controls and 6 keycaps, no primary action, equal weights throughout. The amber on every figure makes "assumed" the panel's dominant colour, so a fully assumed new job looks like a page of warnings. Shapr3D shows the tools that fit the selection and puts the rest behind More (https://support.shapr3d.com/hc/en-us/articles/7873882619548, fetcher refused 403; read from the search snippet). Fix: section 2 and 7: the figures, one primary action (Measure, for a field person), the rest behind three disclosure rows.

**D-7. Alarm as the headline.** Picture 2, Details: "no figure" set in red at about 28 px is the largest type on the screen, followed by "Unknown no figure" twice, "Unknown not set" twice, and four red "Unknown" headers in Open items. price/06-sheet.png: 24 red words in 12 rows ("no price", "no figure" on every line). The project's own VISUAL.md A6 says "A panel is not a column of red sentences: what is not yet priced ... is said once" and these screens pass a standard that says otherwise. For a first-time user on an empty job, every screen is red. Fix: say it once per block ("12 lines have no price yet. Price them") and leave the cells blank with a red mark; red text only on totals.

**D-8. Messages are cards dropped on the work.** Picture 2: the cabinet-run message card (x 445-1072, y 120-188) sits over the plan; the Cabinets card (x 272-1236, y 675-800), a 125 px reference sheet of box codes, covers the bottom fifth of the plan; and the tool-in-hand row says "Cabinets [switch] COUNTERTOP" with the switch's label above it and a two-line hint beside. Picture 3: "Moved wall 2 4 ft east" (x 497-1012, y 120-240) covers the selected wall's top edge, with a More button. Apple on HUDs: "Don't let a HUD obscure the content it adjusts" (https://developer.apple.com/design/human-interface-guidelines/panels). Fix: one toast place, bottom centre above the tool fields, never over the selection, fades after 6 s; the box-code key goes in a "Codes" disclosure inside the fields strip.

**D-9. Type has no scale, and the smallest size is the commonest.** theme.rs: 15 sizes in use; 11, 11.5, 12 and 12.5 px carry 309 of about 330 text calls; MIN_TYPE 11. Hierarchy is made with capitals, grey and colour instead of size, which is why every panel reads as one grey texture. All-caps 11 px headings (NEXT STEPS, LEVELS, SHOW, BUILDING, MECHANICAL, ELECTRICAL, PLUMBING, EDIT...) are 15 in picture 1. Apple's default body on macOS is 13 pt, minimum 10; iOS 17 and 11 (https://developer.apple.com/design/human-interface-guidelines/accessibility). Fix: four sizes, 11 / 13 / 15 / 20 (section 6), body 13.

**D-10. Too many meaning-colours at once, and two collisions.** Picture 1: accent blue, green, amber, red, and ten layer hues: 10 or more. Red means Unknown, a Known clash, demolition, hot water (#EF4444) and the Clashes dot. Amber means Assumed, a clash to verify (#F2990D), Electrical (#EAB308) and Lighting (#F59E0B). Assumed amber #A9690A is 4.46:1 on white and 4.09:1 on the glass: under WCAG's 4.5:1 for the 11-12 px words it is used on (https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html). Apple: "Use color sparingly in HUDs" (panels page). Fix: section 6.

**D-11. Keycaps everywhere.** Pictures 1-3: Ctrl+S, Ctrl+Z, Ctrl+Y, T, V, F, eighteen tool numbers, Del, C, B, U, N, L: up to 25 small boxes, each with its own outline, each competing with the control it labels. Shortcuts belong in a tooltip on hover and in a one-page Keys sheet; a first-time user does not need them in his face, and an estimator learns them once.

**D-12. The top bars are long rows of equal things.** Picture 1: top left "BuildOS", the job, Save, Undo, Redo with three keycaps (680 px); top right PRICE, the figure, a boxed "92 to verify, $17,060.62 riding", Saved, three counts (724 px). Apple: "Don't title windows with your app name"; keep the title short; "Only specify one primary action" (toolbars page). At 1366 the top-right bar runs off the window and cuts the unknown count to "■ 2" (screen-1366-model.png): a figure hidden without a word.

**D-13. The window's own housekeeping is on the window.** Pictures 2-3, bottom right: Job/Details/Tools switches, Lettering Smaller/Larger, the view name, coordinates: 7 controls and a readout on the glass all the time. These are settings. Procreate hides its whole interface with one gesture (handbook above); Figma's Minimize UI keeps "tools one step away" (https://www.figma.com/blog/our-approach-to-designing-ui3/).

**D-14. The view bar and the north card.** Picture 2-3: 3D, Plan T, Walk V, Frame F, Tilt as I dive, Sun (six text buttons, three with keycaps) at the top right of the model, and a 100x115 north arrow card top left of the model with "Home" under it. The north card is the second largest glass object after the panels and carries one function. Google Earth's compass is a small control in the bottom-right cluster.

**D-15. Estimate window: tabs run off the window, and the sheet repeats itself.** Picture 4: ten tabs on one line, "Plumbing 94" cut at the right edge with no sign more exist (the inventory says they wrap; in this picture they do not). Tabs mix kinds: views of the estimate (Estimate, Budget), lists (Verify in field, Clashes), an output (Layouts 0) and trade filters (Building, Roof, Mechanical, Electrical, Plumbing). price/06-sheet.png lists the same lines twice. Apple: "avoid layouts that cause toolbar items to overflow by default" (toolbars page). Fix: section 7.

**D-16. Two plans are cut at the divider.** Picture 5: both plans are cropped at x 440 and x 880; the right plan's house runs off the window; "First floor" and "Layers" are buttons that look the same; a coordinate line "On an end" floats mid-window. Each plan should open fitted to its frame.

**D-17 (standard). VISUAL.md let all of this through.** It has no number in it: no share of the window for the model, no most controls at rest, no least size of type or of a tool or of the gap, no most colours at once. A1 ("the model is seen in the gaps round every panel") rewards floating panels of any size, so two full-height panels pass. A3 ("a control on a panel is a little whiter than the panel") makes every control a box, which is the clutter. A5 asks "nothing cramped" with no measure, so 3 px gaps pass. A6 is right and was failed by the screens it passed. A7 "One accent" is right for the glass and silent on the ten layer hues. What is right and should stay: A2 frosted glass; A4 round corners; A6's rule on red; A7; A8 words; A9 says what happened; B (movement) entire; E1-E3; F1-F2. Section 8 has the lines to add.

## 2. What each person needs first, and what is one step away

One layout; what grows is what the person has chosen.

| | On screen at rest | One step away (one press) | Two steps (never more) |
|---|---|---|---|
| First-time user | the model; the next step of the first job, one line; the shell tools (Wall, Door, Window, Floor, Room, Roof) labelled; 3D/Plan; the price | the ten steps of the guide; Bring in a plan (from the job menu); a tool's fields (they come with the tool) | Layers, Levels, other tools |
| Field person checking one measurement | the model; the thing he taps; its inspector with its figures and one primary button, **Measure**, that takes a typed or lasered reading; the "N to verify" count | the Verify in field list (from the count); Mark known | where the value came from; what it costs |
| Estimator | the model; the price; the count to verify; the tools; the estimate window on the second screen | the inspector's Price section (assembly, cost lines, formula); Layers; Levels; Bring in / Write out | each line's source and formula (in the estimate window, on the line) |

Rules that make this one layout: (1) nothing appears without a selection or a press except the five clusters in section 3; (2) the inspector opens on a selection and closes on Esc or clicking empty ground; (3) each inspector has exactly one primary button, chosen by the thing's state: Measure when its key figure is Assumed or Unknown, Price when it has no assembly, nothing when Known and priced; (4) depth is disclosure rows inside the inspector (Price, Where it came from, Work), closed by default and remembered per person; (5) the guide shows until its ten steps are done or it is put away, then lives under Help in the job menu. NN/g: no more than two levels (progressive-disclosure page).

## 3. Layout specification

**Float with gaps, not on the edges, but only small things float.** Figma reversed floating panels because users said they "cramped the canvas, especially on smaller screens", "designs ... peek out from behind them", and "they slowed people down" (UI3 blog). All three come from full-height panels that are always there. Here, at rest, nothing taller than 64 px floats, so the gaps cost nothing, and the owner's "Google Earth feel" and "glassy overlays" are kept. The inspector, the only tall thing, floats on the right and is as tall as its content, like Earth's inspector and Apple's Liquid Glass sidebars, which "float above content" (sidebars page). The estimate window, which people read for hours, is not glass (keep VISUAL F4).

### 1600 x 960, nothing selected

| Region | Place (x, y, w, h) | What is in it | Collapses how |
|---|---|---|---|
| Model | 0, 0, 1600, 960 | the house | never |
| A. Job cluster | 16, 16, auto (max 440), 40 | [job name ▾] (opens the job menu: Bring in, Write out, Save a copy, Messages, Keys, Help, Settings), Undo and Redo as two 32x32 icon buttons, "Saved" in 11 px text (no box) | job name ends at 28 characters with an ellipsis, full name in the menu header |
| B. Price cluster | right 16, 16, auto (max 400), 40 | "$463,528" 15 px semibold (cents in the estimate window only); a chip "92 to verify" in amber ink, press opens Verify in field in the estimate window; hover on the price shows Known / Assumed / Unknown counts | below 1440 wide, "riding" dollars move into the hover |
| C. Next step (new jobs only) | 16, 68, 320, 44 | "Next: Add doors and windows · 3 of 10", press to open the ten steps (to 360 tall); × puts it away | gone when done |
| D. Tool dock | centred, bottom 16, about 640, 64 | section 4 | Tab hides all glass but the model (Procreate) |
| E. Tool fields (tool in hand) | centred, 8 above D, auto (max 960), 44 | the tool's fields inline, its one-line hint, Codes ▾ for Cabinets | appear with the tool, go with it |
| F. View cluster | right 16, bottom 16, about 420, 40 | [Level: First floor ▾] [Layers ▾ · 2 off] [3D │ Plan] segmented [Walk] [fit icon] [compass 32 px, press = north up] | Walk and fit become icons below 1440 wide |
| G. Toast | centred, 8 above E (or D), max 560 x auto | the last message, one or two lines, a More link; fades after 6 s; never over the selection | history under the job menu |
| H. Coordinates | left 16, bottom 16, text only, 11 px, no card | E / N / H | only while a tool is in hand or in Plan |

Covered at rest: A about 400x40, B 380x40, C 320x44, D 640x64, F 420x40 = about 102,000 px of 1,536,000: **the model keeps about 93 %**; the clear band from y 68 to y 880 across the full width is 85 % of the window.

### 1600 x 960, a wall selected and a tool in hand
Add **I. Inspector**: right 16, top 68, width 320, height = content, max to 12 px above F (960-68-68 = 824). Header (kind 11 px, name 15 px semibold, icon buttons Move, Copy, Delete at the right, 28x28). Key figures in a 2-column grid (label 11, figure 20 for the first, 15 for the rest; ink, with the certainty mark). Primary button full-width 36 tall (Measure). Then three disclosure rows 36 tall each: Price ($432.00 shown in the row), Where it came from, Work (Existing / Demolish / New). Opened Price shows the assembly, the cost lines and the actions now shown as six buttons (Assembly, Allowance, Unit cost, Add cost line, Material, Labor), as a list of rows, not buttons. Covered with the inspector open: + about 320x420 = 134,000; the model still keeps about 85 %.

### 1366 x 768
Same regions. Dock tools 56 wide (labels still whole at 12 px for every current tool name; "Equipment" moves into the Systems flyout so it is not in the dock). Inspector 300 wide, max height 768-68-68 = 632. View cluster about 320 (Walk and fit as icons; level name to 12 characters). Dock centred at x 403-963; view cluster from x 1030: they do not meet. If they would (lettering one step larger), the dock moves left to sit 16 px from the view cluster, never under it. Price cluster drops "riding". The model keeps about 89 % at rest.

## 4. The tools

- **Two levels, like Cities: Skylines' service panels**: the row shows groups, and a group opens its options (Steam manual above). With 18 tools today and more coming, the row shows the six shell tools by name, because a first takeoff is those six, and two group buttons that open a flyout above the dock:
  - in the row: **Select** (pointer icon, no label needed: one of the few universal icons), **Wall 1, Door 2, Window 3, Floor 4, Room 5, Roof 6**, then a 16 px gap, then **Cabinets & items ▴** (Cabinets, Fixture, Equipment, Furnishing as each comes, the tool chest), **Systems ▴** (Duct 7, Pipe 8, Conduit 9, Cable 0, Fitting, Connect).
  - **Room line** is an option of Room (a two-way choice in Room's fields: "Room │ Room line"), not a tool. **Fitting** and **Connect** are options in a run's fields (they only make sense with a pipe, duct, conduit or cable in hand) and also stay in the Systems flyout for direct use.
  - **Move** and **Delete** leave the dock: they act on a selection, so they are icon buttons in the inspector's header and keys (M, Del). Shapr3D: "selecting a body makes Move/Rotate active" (support page snippet).
  - A group button shows the last tool used from it, so an electrician's second day has Conduit in the row (SketchUp-style flyouts keep the last one; this is reasoning, not a cited SketchUp line).
- **Size**: each tool 60 wide (min; grows to label + 16), 52 tall; glyph 20 px; label 12 px Medium, one line, never shrunk; 6 px between tools in a group, 16 px between groups, 8 px padding in the dock. No outline or fill on a tool at rest; hover is a 6 % ink wash; in hand is the accent wash plus a 2 px accent bar under the label (accent never alone: VISUAL A7). Apple: no borders on toolbar items because the bar is the container; at most three groups (toolbars page). Target 60x52 is above Apple macOS 28x28, iOS 44x44 and WCAG 2.5.5's 44x44.
- **Icon alone** only for: Select (arrow), Undo, Redo, the fit-to-view and compass, and the inspector's Move/Copy/Delete with a tooltip. Everything else carries its word (NN/g: "a text label must be present alongside an icon").
- **Number keys** stay; they show in the tooltip ("Wall · 1") and on the Keys sheet, not on the button.
- **Fields of the tool in hand**: region E, a glass strip 44 tall directly above the dock, centred on it. Fields inline: label 11 px left of a 28 px field ("Thick 6"", "High 9' 0"", "Long" with the cursor), then the hint in 13 px ink-soft, then Esc and Enter as words at the right. Never in the inspector, never at the top of the window: the eye is at the pointer and the dock.

## 5. Where everything else goes

| Thing | Today | Goes to | Form |
|---|---|---|---|
| Levels | Job panel rows with switches | View cluster, [Level: First floor ▾] | a popover listing levels top-down (second floor, first floor, foundation), each with its switch and height; "All levels" first; Add level at its foot. The chip says which level the tools draw on. |
| Show (layers) | 8 presets + 12 switches in the Job panel | View cluster, [Layers ▾ · N off] | a popover 280 wide: one row of presets at the head as a segmented control (All, Shell, Frame, Systems, Inside); then three groups with headings: Building (Ground, Foundation, Framing, Finishes, Trim, Roof), Systems (Mechanical, Electrical, Plumbing), On the model (See through, Sizes, How sure, Clashes). Each row: swatch, word, count, switch. The chip's "2 off" keeps every layer's state readable at a glance without the sheet open. **This needs the lead's ruling** against "every layer has a switch that reads on or off at a glance": I read it as satisfied by the chip plus the sheet, as Earth and Apple Maps do; if not, the popover can pin open. |
| Bring in | 9 buttons | Job menu > Bring in... | one sheet: a file list (the program's own, as today: no system dialog) with the kind of each file recognised from the file and shown as a column ("Price list", "Plan, 12 sheets", "Designer's model"). One button, Bring in. When the kind is ambiguous (a CSV that could be prices or a prior estimate), a two-choice question in the sheet. Ten more kinds add rows to a recogniser, not buttons. Starter assemblies and the starter tool chest are offered by the new-job flow and the guide, and listed in the sheet under "Starters". |
| Write out | 6 buttons | Job menu > Write out... | one sheet: a list of outputs, each a row with its name, who it is for and its file kind (Estimate workbook · for you · .xlsx; For suppliers · .xlsx; BIM model · IFC...), grouped by who it is for (You, Suppliers, Subcontractors, PM, Other programs). One Write out button. Ten more kinds are ten more rows in a list that scrolls, and a search field when it passes 12 rows. |
| Drawings (plans under the tracing) | Job panel | Layers popover gets "Plans" rows (each plan a switch); Add drawing, Set scale, Line up live under Job menu > Bring in (a plan) and in the guide's step 2 | |
| Tool chest (items) | Job panel | the "Cabinets & items" flyout ends in "All items...", which opens a 320-wide library sheet on the left (search, by trade, New item) | like SketchUp's trays, opened on demand |
| First-job guide | top of Job panel | region C, then Job menu > Help | |
| Job / Details / Tools switches, Lettering | bottom right | Job menu > Settings (Lettering) and one key, Tab, to hide all glass; the inspector closes with Esc | |
| Tilt as I dive, Sun | view bar | View cluster, a "..." at its end > View settings | |

## 6. Type, spacing, colour

**Type (Inter, at 1x; the four lettering steps scale all of it):**
| Token | px / line | Weight | Use |
|---|---|---|---|
| caption | 11 / 14 | Medium | units, counts, field labels, coordinates. Never a sentence, never a control's only label. |
| body | 13 / 18 | Regular, Medium for controls | every control label, every row, messages, the inspector's rows. (Apple macOS default 13 pt.) |
| title | 15 / 20 | Semibold | a thing's name, a sheet's heading, the price chip |
| figure | 20 / 24 | Semibold, tabular | the one key figure in the inspector; the estimate's Price |
Four sizes. Delete 11.5, 12, 12.5, 13.5, 14, 14.5, 17, 22, 23, 26. Headings: 13 Semibold, sentence case, no capitals (capitals at 11 px are 15 of the busiest words in picture 1). Tool labels are 12 Medium (the one exception, inside the dock only). 96 and 110 px (the start screen, I assume) are outside this scale.

**Spacing:** 4, 8, 12, 16, 24. Window margin 16. Gap between floating clusters 12. Inside a panel: 12 padding, 8 between rows of controls, 16 between groups. Row height 32 (inspector, popovers), 32 (estimate rows). Radii: panel 14, control 8, field 6 (14 = 8 + 6 padding: Apple asks for concentric corners, toolbars page).

**Colour tokens (light scene):**
| Token | Hex | Use |
|---|---|---|
| glass | #FAFBFC at 78 % over the blur | every floating thing |
| glass-edge | #FFFFFF at 70 % | 1 px edge |
| ink | #16191D | text, Known figures |
| ink-2 | #4E5661 | secondary text (7.4:1 on white) |
| ink-3 | #5C6570 | captions (5.4:1 on #F4F5F7) |
| rule | #16191D at 8 % | hairlines |
| hover | #16191D at 6 % | hover wash |
| accent | #1D4ED8 | selected, in hand, switch on. Nothing else. |
| accent-wash | #1D4ED8 at 14 % | |
| assumed | #8A5508 (text, 5.7:1 on the glass); mark fill #E3B04B | Assumed; a clash between unmeasured things |
| unknown | #B93227 (text 5.4:1); mark fill #D6584C | Unknown; a clash between Known things |
| done | #1F7A4D | only the one "Saved" word and a passed check, 11 px |
**Known / Assumed / Unknown**: the figure stays in ink at every size; its certainty is a mark of shape and colour before it: no mark for Known, a half-filled amber circle for Assumed, an open red circle for Unknown (shape and colour, so colour is never alone; WCAG 1.4.1). Colour the figure itself only in the estimate sheet's quantity and unit-cost cells, where the column makes it read as one. This replaces "figure's colour and mark agree" (VISUAL A6) and is a change the lead must rule on. **Trade colours** live only in the model and as 10 px swatches in the Layers sheet; never as text, never on a figure, never on a panel at rest. Move them off the certainty hues: Electrical from #EAB308 to violet #7C3AED, Lighting to #A78BFA, Refrigerant to #DB2777, Domestic Hot Water from #EF4444 to deep pink #BE185D (not orange, which would meet amber). Clashes in the model get a form as well as a colour (a pulsing outline and a numbered pin), so a red hot-water pipe is never read as a clash. **Most meaning-colours on the glass at once: 3** (accent, assumed, unknown).

## 7. The estimate window

- **Header** (one card, 48 tall): job name 15 semibold; the price 15 semibold tabular; "92 to verify" chip; [Estimate │ Plan │ Two plans] segmented; no "BuildOS", no counts (counts on hover over the price, as in the model window).
- **Tabs: four**: Estimate · Verify in field 85 · Clashes 11 · Lists ▾. Budget becomes a two-way switch at the head of Estimate ("Estimate │ Budget", the design-phase price per SF or LF and its detail are one table at two depths, item 6 of the deliverable). Building, Roof, Mechanical, Electrical, Plumbing are filters, and the By division rows already are the filter: pressing a division filters the lines to it, with a "Showing Division 6 × " chip. Layouts moves to Write out. Tabs never wrap and never clip; if they ever outgrow the width, the last ones go into "More ▾" (Apple: avoid overflow by default).
- **Recap**: keep as it is in picture 4: it is the best screen in the program. Drop its card shadow and outline (rows on the page with one hairline rule), figures right-aligned tabular 13, Price row 15 semibold with a 2 px rule above. "none" stays ink-3; "no figure" red once per block.
- **By division**: 16 rows, number, name, lines (ink-3), figure. A division with no price shows a red mark and a blank cell, not "no figure" in red.
- **Lines**: one listing, not two (remove "The estimate, by item" or make it the collapsed state of a division). Row 32 tall; columns Line (flex), Qty (with unit), Unit cost, Material, Labor, Sub/General, Extension; fields drawn as fields only on hover and focus (a 1 px rule under the cell), not as 3 boxes per row; the formula and source open in a row drawer under the line (one press), not as text on every line. Unpriced lines: an empty unit-cost cell with a red mark; one sentence at the division's head: "4 lines have no price yet".
- **Two plans** (picture 5): each opens fitted to its half; the plan's level and Layers sit as one chip at its top left ("First floor · Layers ▾"); the divider is a 1 px rule; the coordinates line goes to the bottom-left of its own plan, 11 px, no card.

## 8. Proposed lines for the visual standard (each judged from one picture, with a number)

| # | Line | Number | Source or reasoning |
|---|---|---|---|
| V1 | The model keeps at least this share of the model window at rest (nothing selected, no tool in hand), measured as the window less every floating thing's box. | **≥ 90 %** at 1600x960 and ≥ 85 % at 1366x768; with a selection and a tool in hand **≥ 75 %** | Today 56 %. Earth: four controls in one corner (Google help); Procreate's 14 controls; Figma's "work center stage". 90 % is what the layout in section 3 reaches (93 %) with a margin. |
| V2 | Most separate press targets in view on the model window at rest. | **≤ 24** (and ≤ 16 on a new, empty job) | Procreate 14, SketchUp's default toolbar 21 (their own pages). Section 3 reaches about 20. |
| V3 | Smallest lettering anywhere; nothing that is a sentence or a control's only label below body. | **11 px** for captions (unit, count, field label); **13 px** for body and controls; tool labels **12 px** | Apple: iOS minimum 11 pt, macOS default 13 pt (accessibility page). |
| V4 | Number of text sizes on one screen. | **≤ 4** | Today 15 in code. Reasoning: four is the fewest that gives caption, body, title, figure. |
| V5 | A tool's press target, and its label whole on one line inside it. | **≥ 56 x 52 px**, label never touches the button's edge (≥ 6 px each side) | WCAG 2.5.5 44x44; Apple iOS default 44x44. Today 48x44 with labels over the edge. |
| V6 | Gap between two press targets that have no edge of their own (tools, icon buttons). | **≥ 6 px**, and **≥ 16 px** between groups | Apple: about 12 pt padding round bezelled controls, 24 round unbezelled ones; with no bezel and a hover wash, 6 px between targets of 56 px still gives each a 62 px pitch. Today 3-4 px. |
| V7 | Groups in the tool dock. | **≤ 3** | Apple toolbars: "aim for a maximum of three". |
| V8 | Colours carrying meaning on the glass at once (not counting ink and greys). | **≤ 3** (accent, assumed, unknown); trade colours only in the model and as swatches inside an open Layers sheet | Apple panels: "Use color sparingly"; today 10+. |
| V9 | Text contrast against the glass under it, at its least (sample the panel's lightest point). | **≥ 4.5 : 1** for text under 18.67 px bold / 24 px; ≥ 3 : 1 above | WCAG 1.4.3. Today Assumed amber is 4.09:1 on the glass. |
| V10 | No panel stands taller than its content, and no panel is present at rest that is taller than this. | **64 px** at rest; the inspector, when open, ≤ the window less 136 px | Figma's floating panels cramped the canvas because they were tall and always there. |
| V11 | A figure is shown once per window. | the price **1 time** on the model window | Today 2 on the model window (3 with the estimate). |
| V12 | Red words in any one panel or card. | **≤ 2** (a total and one sentence) | VISUAL A6, made countable. Today 9 (picture 2) and 24 (price/06). |
| V13 | Keyboard keys drawn on screen at rest. | **0** (in tooltips and the Keys sheet only) | Today up to 25. |
| V14 | Nothing floating covers the selected thing or the point under a tool. | **0 px** of overlap | Apple panels: don't obscure the content a HUD adjusts. |
| V15 | Nothing is cut at any of the four window sizes (1366x768, 1600x960, 1920x1080, 2560x1440) at every lettering step; a figure is never cut. | **0** clipped controls or figures | Today the unknown count is cut at 1366. |
| V16 | Tabs on the estimate window fit one line at 880 wide at the largest lettering step. | **≤ 5 tabs**, 0 clipped | Apple: avoid overflow by default. |

Lines to change in VISUAL.md: A1 add V1 and V10; A3 replace "a control on a panel is a little whiter than the panel" with "a control at rest has no box of its own unless it is a field; its group is the box"; A5 add V5, V6, V15; A6 add V12 and the shape marks; A7 add V8; new A10 = V2, V4, V3, V13.

## 9. Remove outright, then rank

**Remove**: the Details panel with nothing selected; the Job panel as a panel; the eight Show presets as buttons (they become one segmented row in the Layers sheet); every keycap at rest; "BuildOS" in both windows' bars; the PRICE word before the price; the help line "Pick a tool below to draw..." at rest (keep it only on an empty job); the Cabinets reference card as a standing overlay; Room line, Fitting, Connect, Move, Delete as dock tools; Lettering, Job/Details/Tools switches on the glass; the north card (a 32 px compass in the view cluster); the duplicated "by item" listing in the estimate; outlines and fills on every control at rest.

**Ranked by effect on "busy" and "cramped" against cost** (cost: S = a day or less of layout in panels.rs/theme.rs, M = a few days, L = a week or more; every control keeps its `Named` name for the scripts, as CLAUDE.md asks):

| Rank | Change | Effect | Cost |
|---|---|---|---|
| 1 | No inspector without a selection (D-5) | large: removes 30 rows and a 344 px panel at rest | S |
| 2 | Tools: 60x52, 12 px labels, no box at rest, 6/16 gaps, Move/Delete/Room line/Fitting/Connect out of the dock, Systems and Items flyouts (D-2) | large, the "cramped" complaint itself | M |
| 3 | Job panel removed: Levels and Layers into the view cluster popovers; Bring in, Write out into the job menu sheets (D-3, D-4) | large | M-L (the two sheets are new; the file list exists) |
| 4 | Keycaps off the glass (D-11) | medium-large, 25 boxes | S |
| 5 | Type scale to four sizes, body 13 (D-9) | medium-large, every panel calms | S-M (a sweep of 330 calls; check the map script) |
| 6 | Certainty as a mark, figures in ink; assumed #8A5508; one red sentence per block (D-6, D-7, D-10) | large on a new job, which is every first impression | M (a rule change: lead's call) |
| 7 | Messages to one toast place (D-8) | medium | S |
| 8 | Inspector: one primary action, three disclosure rows (D-6) | medium-large for the field person | M |
| 9 | Top bars compressed into the job cluster and price cluster (D-12) | medium | S |
| 10 | Housekeeping off the glass, Tab hides all (D-13) | small-medium | S |
| 11 | Trade colours moved off amber and red; only in the model (D-10) | medium | S (theme.rs constants) |
| 12 | Estimate window: four tabs, one listing, rows without boxes (D-15) | medium on the second window | M |
| 13 | Two plans fitted, chips merged (D-16) | small | S |
| 14 | VISUAL.md rewritten with section 8's numbers (D-17) | prevents the next round | S |

## 10. Sources
- Apple HIG, Accessibility (text and control sizes, padding): https://developer.apple.com/design/human-interface-guidelines/accessibility (read through its JSON at https://developer.apple.com/tutorials/data/design/human-interface-guidelines/accessibility.json)
- Apple HIG, Toolbars: https://developer.apple.com/design/human-interface-guidelines/toolbars
- Apple HIG, Sidebars: https://developer.apple.com/design/human-interface-guidelines/sidebars
- Apple HIG, Materials: https://developer.apple.com/design/human-interface-guidelines/materials
- Apple HIG, Panels (inspectors, HUDs): https://developer.apple.com/design/human-interface-guidelines/panels
- Google Earth help, Discover places and change your view: https://developers.google.com/maps/documentation/earth/discover-places-change-view
- Google Earth help, Navigate the globe: https://developers.google.com/maps/documentation/earth/navigate-the-globe
- Google Earth help, Style data layers (Map Contents panel, inspector): https://developers.google.com/maps/documentation/earth/style-data-layers
- Figma, Our approach to designing UI3: https://www.figma.com/blog/our-approach-to-designing-ui3/
- Shapr3D, Behind the Shapr3D user interface refresh: https://shapr3d.com/blog/behind-the-shapr3d-user-interface-refresh
- Shapr3D support, Adaptive user interface: https://support.shapr3d.com/hc/en-us/articles/7873882619548 (403 to the fetcher; content from the search result's summary only)
- SketchUp Help, Toolbars: https://help.sketchup.com/en/sketchup/toolbars
- Procreate Handbook, Interface: https://help.procreate.com/procreate/handbook/interface-gestures/interface
- Cities: Skylines user manual (Steam): https://cdn.akamai.steamstatic.com/steam/apps/255710/manuals/CitiesSkylines-UserManual_EN.pdf
- Cities: Skylines II, City Corner #1, Upcoming visual updates: https://www.paradoxinteractive.com/games/cities-skylines-ii/news/upcoming-visual-updates
- NN/g, Progressive disclosure: https://www.nngroup.com/articles/progressive-disclosure/
- NN/g, Icon usability: https://www.nngroup.com/articles/icon-usability/
- WCAG 2.2 Understanding 2.5.8 Target Size (Minimum): https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
- WCAG 2.2 Understanding 1.4.3 Contrast (Minimum): https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
- Not found from a primary source, so not used: a Google Earth design team's own account of its layout (the Medium post returned 403); any vendor's figures for share of screen kept for content; Cities: Skylines II's control counts or sizes. Every percentage of window above is my own measurement of the pictures.

Not done: the three HTML/SVG mockups asked of the original prompt were not asked of me here; section 3's tables are the build specification.
