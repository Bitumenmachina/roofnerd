# Real files through the two wall-finders, for the first time (2026-11-23)

An agent (Sonnet) found public files, ran them through the program as committed at
0677db0, and compared what it said with the sheet or the scan as a person reads it. The
files stayed in the session's scratch area and are **not in the repository** (two of the
sources are not openly licensed). It could not write its own report file; this is the
lead's record of what it handed back, the figures as it gave them.

## Drawn plans (vector PDFs)

| Sheet | Source and terms | What the program said | Against the sheet | Verdict |
|---|---|---|---|---|
| A city's pre-approved accessory dwelling, 1 bedroom, 499 SF; one page with a floor plan AND a roof plan | lagunahillsca.gov DocumentCenter/View/9133 (30 sheets, 36 x 24 in); no terms stated, sheets marked for use in that city | 9,572 lines read; "walls found: 56, 37 of them in doubt"; "do not close: 11 ends meet nothing"; 2 openings; the scale cautioned, rightly (set from a dimension line that overshoots its ticks: 1 percent long) | about 14 wall runs, 5 windows, 3 to 4 doors. West wall right in 3 runs (28 ft 10 in); outside walls 7 in, partitions 4 1/2 to 5 1/2 in; several windows and doors right | **mostly right on the floor plan; wrong and silent on the roof plan**: its lines were found as walls 12 3/4 to 14 in thick, most not in doubt, and laid: takeoff 181.33 LF where the floor plan has about 125 |
| A city's pre-approved accessory dwelling, 491 SF, one cover sheet with a plan at 3/8 in | cityofboise.org media/21148 (the city's page says the plans are public domain, not to be altered) | 1,260 lines; "walls found: 24, 4 of them in doubt"; 3 ends meet nothing; 10 openings; 3/8 in = 1 ft, no caution | overall 24 ft 2 in, 20 ft 2 in, 22 ft 2 in all matched the sheet's own strings; walls 8 1/2 in; partitions 4 1/2 in; windows 3 ft and 5 ft, doors 3 ft right | **mostly right**: a dimension string and a stray in an elevation found but put in doubt; a shower's glass lines read as two windows, not in doubt |
| The same city's 695 SF, 2 bedroom | cityofboise.org media/21149 (same terms) | "walls found: 23, 15 of them in doubt"; 1 end meets nothing; 9 openings | outside walls and the three main partitions right to the inch against 30 ft 2 in by 24 ft 2 in; entry doors 3 ft; two 6 ft windows | **right** on outside walls and main partitions; laid 9 walls, 3 doors, 7 windows, 131.15 LF; a second laying refused |

## Scans

| Scan | Source and terms | What the program said | Against the scan | Verdict |
|---|---|---|---|---|
| A phone LiDAR scan of a small room, 3.6 by 4.8 m, 196,676 mesh vertices | Apple's ARKitScenes (github.com/apple/ARKitScenes; a non-commercial licence) | ceiling 8 ft 0 in and a lower zone; 21 stretches "left out ... not straight along the whole length"; "no wall found"; walls found: 0 | a closed room with four clear walls | **wrong, but said** |
| The same source, a long room 6.6 by 7.5 m, 1.0 million vertices | as above | "walls found: 11, 11 of them seen from one side only"; 2 ends meet nothing | about 4 walls: the two long walls are found, in place (22 ft 3 in, 14 ft 6 in) | **mostly right on the long walls; wrong on the count**: one wall found twice 10 in apart, another three times; the end walls missing (their stretches listed as left out) |
| The same source, a large notched room 8.3 by 9.9 m, 0.8 million vertices | as above | 39.9 s; "walls found: 5 ... 1 in doubt" (pieces 3 ft 6 in to 9 ft 7 in); 6 ends meet nothing; the level's 9 ft against the scan's 7 ft 9 in said | walls on every side, about 17 by 21 ft | **wrong, but said** |
| An office conference room, 3.2 million points, with no ceiling in the file | a research set linked from a public code repository; no licence stated | "no ceiling shows in the scan ... so no wall is taken"; nothing found | no ceiling in the file | **right** to stop |

Also: the wrong measure (`--units ft` on a scan in meters) was caught ("not likely the
house"); **the wrong way up (`--up y`) was not**: it read a plausible ceiling and went on.
A wrong-axis run on the 3.2 million point file ran 4 min 44 s with no word until it was
stopped.

## What worked on real files
Both finders ran on every file without failing, in 0.6 to 40 s, in a builder's words.
Scaling from one dimension gave the sheet's true scale and cautioned on the one that was
off. On clean single-plan sheets the outside walls, thicknesses, partitions, 3 ft doors
and 3, 5 and 6 ft windows were right and overall lengths matched the sheets' own strings.
Strays were put in doubt and nothing in doubt was laid. A second laying was refused. A
scan with no ceiling was refused with the reason.

## What failed, and the agent's reading of why
1. **More than one drawing on a page.** The whole page is searched; a roof plan beside
   the floor plan was laid as walls. Nothing lets a person say which part of the sheet
   is the plan.
2. **One wall found two to four times** (the lines of its layers and the edges of its
   fill): 56 for about 14 on a plan; 11 for about 4 in a scan.
3. **Openings**: window tags and sash lines missed; a shower's glass read as windows; a
   recessed entry read as a 6 ft 10 in doorway.
4. **A real phone scan's walls are thrown out as "not straight"**: real meshes bow and
   are rough; the finder was built on made-up flat walls. Zero of four walls in a small
   room.
5. **The wrong way up is not caught.**
6. **Speed**: 40 s for 0.8 million points; minutes with no word on a larger wrong-axis
   file.

## Not obtained
No phone scan under a public-domain or open licence was found (one public-domain set's
host refused the download for quota; others need a form). Three more plan sets were
fetched and not run; four cities' files could not be fetched. No scanned sheet was tried.
