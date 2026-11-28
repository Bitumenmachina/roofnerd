# Quiet, with room: a demonstration, written before it is built

Calls 239 and 240. Patrick: "it still seems quite busy and the tools look a bit
cramped". `VISUAL.md` section G has sixteen lines with a number each. This
demonstration measures them on the real window, from what the program writes of
itself (the map: every control's place and size, every card's box, every piece of
text), so that "busy" and "cramped" are figures and not an opinion. It is `demos/quietw.sh`.

It is proved red first: run on the window as it is today, before anything is changed,
it must fail the lines below at the figures the design judge measured by hand from
the pictures (`research/judgment-design-1126.md`, "Measurements").

## The job and the views

The sample whole house (`plumbline sixteen`), opened cold.

- **At rest**: 3D, nothing selected, no tool in hand. At 1600 by 960 and at 1366 by 768.
- **At work**: a wall of the first floor selected and the Wall tool in hand. At 1600 by 960.
- **A new job**: a job whose file is not there yet. At 1600 by 960.
- **A job entered Assumed** (call 248): the box house `demos/movew.sh` makes (24 by
  16 ft, walls 9 ft, every value entered Assumed from a made-up survey, no overhead
  or profit set), its east wall selected, on plan. At 1600 by 960.

## What is measured, the figure today, and the figure it must reach

"Today" is the judge's measurement of the pictures; the script must agree with it to
within what is said, or the script is wrong and is put right before it is trusted.

| G | What the script works out | Today | Must be |
|---|---|---|---|
| 1 | At rest, the window's area less the area covered by every floating thing's box (boxes that overlap counted once), over the window's area | 56 % (give or take 3) at 1600 by 960 | at least 90 % at 1600 by 960, 85 % at 1366 by 768; at work at least 75 % |
| 2 | At rest, the count of things that can be pressed and are in view (a control on the map, not scrolled off) | 62 (give or take 4), with 18 tools among them | at most 24; on the new job at most 16 |
| 3 | The smallest lettering in view, and the smallest that is a sentence or a control's only word | about 10 to 11 px; controls at 11 to 12.5 | 11 at the least; a control's word 13 (a tool's 12) |
| 4 | The count of different sizes of lettering in view at rest | 6 or more in view (15 in the code) | at most 4, and the tools' 12 |
| 5 | Each tool's box, and its word's width against it | 48 by 44; "Equipment" wider than its box | at least 56 by 52, the word inside by 6 px each side |
| 6 | The gap between each tool and the next in its group, and between groups | 3 to 4 px | at least 6; at least 16 between groups |
| 7 | The count of groups in the tools | 3 | at most 3 |
| 10 | The tallest floating thing at rest | about 845 px (the two side panels) | at most 64 |
| 11 | The count of places the price's figure is written on the model window | 2 | 1 |
| 12 | The most red words in any one panel or card, on the new job | 9 or more | at most 2 |
| 13 | The count of key labels drawn at rest (a key's name beside or on a control) | 25 (7 as buttons, 18 on tools) | 0 |
| 14 | At work, the area of any floating thing over the selected wall's box on screen | more than 0 (a message lies on it) | 0 |
| 15 | At each of 1366 by 768, 1600 by 960, 1920 by 1080, 2560 by 1440 and each step of lettering: controls or figures whose box runs past their card or the window | at 1366 the count of unknowns is cut | 0 |
| 16 | The estimate window at 880 wide, largest lettering: the count of tabs and of lines they take | 10 tabs on 2 lines, one cut | at most 5, on 1 line |

## What call 248 added, after part A's first hand-back (figures by hand)

| | What the script works out | Must be |
|---|---|---|
| M1 | On the job entered Assumed: the east wall is 16 ft long and 9 ft tall, so its face is 16 x 9 = 144.00 SF, its length 16.00 LF, its cost 144 x $3 = $432.00. Each is Assumed in the model. The marks the map says stand before those three figures in the wall's panel | 3 at the least, each the half-filled amber circle, each within 16 px to the left of its figure; the builder lists the model's status of every figure the panel shows, and every Assumed or Unknown one has its mark |
| M2 | In the picture of that screen, inside each mark's box: dots of the mark's colour; in each figure's own lettering: dots of amber or red | more than 0 in every mark; 0 in every figure |
| M3 | On the same job the price has no figure (no overhead or profit is set): what stands before "no figure" in the top bar | the open red circle of Unknown |
| M4 | On the sample house at rest: what the top bar says beside the price | "85 to verify, $17,060.62 riding" (the figures `plumbline vif` gives), and the word Price before the price |
| M5 | The count of places a message stands, at rest, at work and on the new job; and any run of four words or more written in two places in view | 1 place at the most; 0 runs written twice |
| M6 | At work at 1366 by 768 as at 1600 by 960: line 14 | 0 |
| M7 | The job's name in the top bar | whole (no ellipsis) where it is narrower than two fifths of the window |
| M8 | In the pictures at rest and on the new job: the contrast of each piece of lettering against the glass beside it, by its own dots | 4.5 to 1 at the least (3 to 1 at 24 px or more); 0 dots of green in any lettering |

Part A owes M1 to M8.

Lines 8 and 9 are not measured on the screen but in the colours themselves, by a test
in `crates/app/src/theme.rs`, written first: every colour used for lettering on the
glass, against the glass at its lightest, gives at least 4.5 to 1 (today Assumed
amber `#A9690A` gives 4.09 to 1 and fails; `#8A5508` gives 5.7 to 1); and no trade's
colour is within the hue of assumed or of unknown.

## In two parts

The layout is built in two parts (call 240, point 6), and the script says which lines
each part owes. A line not yet owed is printed with its figure and the words "not
yet: part B"; it is never printed as passed.

- **After part A**: lines 3, 4, 5, 6, 7, 8, 9, 11, 12, 13 and 14 pass. Line 1 reaches
  at least 74 % at 1600 by 960 (the panel for a thing is gone at rest and the two top
  bars are small; the Job panel still stands). Line 15 passes for what part A touches.
- **After part B**: every line passes at the figure in the table.

## What the program must say for this to be measured

New lines on the map only; nothing already there changes: the size of the lettering
of each piece of text in view; which controls are key labels; which control is a
tool, and its group; whether a box floats. The builder says in its report what it
added.
