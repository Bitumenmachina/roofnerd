# The pictures

Each folder here is written by one demonstration, which drives the real program on a
virtual display and saves a picture of both windows after each step (`NN-model.png`,
`NN-sheet.png`). The comment beside each `shot` in the script says what that picture
is to show. A script that passes has not looked at its pictures: a person, or a
worker told to, reads them (`../../research/pictures-read-5be55e3.md` is the last
such reading, and STATUS.md says where each thing it found stands).

| Folder | Written by |
|---|---|
| `bring` | `demos/bringw.sh` |
| `build` | `demos/buildw.sh` |
| `carry` | `demos/carryw.sh` |
| `clash` | `demos/clashw.sh` |
| `d2` | `demos/d2.sh` |
| `d4` | `demos/d4w.sh` |
| `d5` | `demos/d5w.sh` |
| `draw` | `demos/draww.sh` |
| `dress` (`one-each`, `priced`, `bare`) | `demos/dressw.sh` |
| `field` | `demos/fieldw.sh` |
| `first` | `demos/firstw.sh` |
| `gablew` | `demos/gablew.sh` |
| `foundw` | `demos/foundw.sh` |
| `glass` (`01` to `09`) | `demos/glassw.sh` |
| `shinglew` | `demos/shinglew.sh` |
| `house` | `demos/house.sh` |
| `itemlib` | `demos/itemlibw.sh` |
| `itemnew` | `demos/itemneww.sh` |
| `cabinetsw` (`01` to `11`) | `demos/cabinetsw.sh` |
| `items` | `demos/itemsw.sh` |
| `layoutsw` (`01` to `10`) | `demos/layoutsw.sh` |
| `layers` (`01` to `05`; `06` to `17` are the trades' buttons, call 177) | `demos/tradesw.sh` |
| `money` | `demos/moneyw.sh` |
| `movew` (`01` to `07`: a wall moved in the window, its floor, footing and roof with it, one Undo, a move refused) | `demos/movew.sh` |
| `light` (`<n>-before.png` and `<n>-after.png`: the same views before and after the light of call 184, step 2) | `demos/lightw.sh` |
| `pagew` | `demos/pagew.sh` |
| `plan` | `demos/planw.sh` |
| `price` | `demos/pricew.sh` |
| `quantityw` | `demos/quantityw.sh` |
| `renow` (`01` to `10`; `multi/` `01` to `14`: several at once, a wall the work opens, a line taken off, a measure typed) | `demos/renow.sh` |
| `roofw` | `demos/roofw.sh` |
| `roomlinew` (`01` to `06`) | `demos/roomlinew.sh` |
| `scanw` (`01` to `15`) | `demos/scanw.sh` |
| `tracew` (`01` to `08`) | `demos/tracew.sh` |
| `trimw` (`01` to `10`) | `demos/trimw.sh` |
| `gableendw` (`01` to `08`, and the build before) | `demos/gableendw.sh` |
| `tools` | `demos/toolsw.sh` |
| `turnw` | `demos/turnw.sh` |

**Taken by hand, and not retaken by any script.** They are the record of a step as
it looked on the day, and are stale in everything else: the name on screen (the
program was called plumbline until 2026-11-17), the panels, the words and the price.

- `glass/step1-floating-panels.png` to `step6-estimate-window.png`: the window being
  rebuilt to call 62, step by step (2026-11-13).
- `layers/plan-panels.png`, `layers/estimate-window.png`: the same rebuild.
- `light/`: the house before and after it was given a sun and a sky (call 68).

**`trades/`** is written by `demos/tradesw.sh` and was retaken on the frozen build of call 177 (2026-11-22). `1-foundation` to `7-waterproofing` and `first-floor-alone` are the layers of the made-up whole house (the ground away, the walls as their frames, electrical, plumbing, HVAC, the inside looked into, the finish off, the first floor alone). `button-<trade>` is each trade's button pressed from the same house opened cold, with a beam, two posts, two pad footings, and joists and rafters named, added to it; `button-all` puts the switches back, and the plumbing, HVAC, electrical and inside ones are the same pictures as `4-plumbing`, `5-hvac`, `3-electrical` and `6-interior-woodwork`. `far-plumbing` is the plumbing seen from far off, its runs no thinner than two dots; `clash-rooms-model` and `clash-rooms-sheet` are the clashes marked and listed, each saying its room.
