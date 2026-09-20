# Return — round 2, cloud executor to the web seat

Branch `repair/hands` from `chore/roster` from `main` at `2bd6894`. Pull request "repair/hands → main", not
merged by the executor. Roster PR #2 (`chore/roster`) is separate and first. Review branch of round 1 is PR #1.

## Gate A, again

Transcripts this session, all rounds: 29 agent transcripts, 22 on claude-opus-5 and 7 on claude-sonnet-5. None on a Fable model. This time the roster loaded
mid-session, so `scout`, `runner`, `auditor` and `builder` were all callable; the executor used
`scout`/`auditor` for proof reads and `auditor` for every cold picture, and built the items at the main
seat with the patches staged in a scratch worktree (`.scratch/repair-wt`, gitignored, a throwaway branch
`repair/hands-wt` pushed once under the stop hook and deleted at the end). Environment pins were not
set; rule A2 carried the pin.

## The items

| # | Item | Commit(s) | Cold auditor's reading (condensed) | Capture |
|---|---|---|---|---|
| R1 | File menu paints, reachable | 230cb31 (check red), 56d3803 | Three items visible, legible, inside the viewport; "clickable-looking" not demonstrable from a still; Save looks enabled with no file open | `R1-menu-56d3803.png` |
| R2 | Every probe click proves reach; D140 reversed | 911270a, 6c8f7a8, 8f88e92, 36aefb6 | No picture: a harness rule. Sweeps 1–3 below | — |
| R3 | New job… on the start screen and in File; empty job says what first | df47f47, 7961527 (hint regression fixed) | "New job…" first in reading order, primary weight, demo demoted to a second row; what the ellipsis opens is not determinable from a still | `R3-start-df47f47.png`, `R3-menu-df47f47.png` |
| R4 | Recap foots by hand | 2c8961e | Auditor re-added every printed figure: classes, job cost 194,458.37, ladder to 228,099.67 all tie; notes Overhead $0.00 unlabeled and Equipment/Other with no lines row | `R4-R5-recap-ff2b7e8.png` |
| R5 | Closed run counts corners | ff2b7e8 | Parapet 400.00 LF · 8 EA, curb 24.00 LF · 4 EA, field 8 sides; (then) RUN/COUNT clipped under the footer — R9 | `R4-R5-condition-parapet-ff2b7e8.png` |
| R6 | Substitutions on the line; Wind, plan, Section say their rule | c6193cb | Fastening plan: the words are on the window's caption, "borne out"; corner FM rating under the RD-2 symbol; Section: "under review" on the footnote, letters themselves unmarked; Wind note: first capture was below the fold, retaken scrolled — on screen | `final-fastening-75b1468.png`, `final-section-75b1468.png`, `R6-wind-75b1468.png`, `final-estimate-75b1468.png` (adhesive line open) |
| R7 | One scale on the Section | 5364ade | Strip and footnote both 1″ = 5′-4″ and ×4, verified at magnification; correctness of the scale not checkable from a still | `final-section-75b1468.png` |
| R8 | Header counts rows | 1232221 | (executor) header reads 45 lines over 45 rows | `final-estimate-75b1468.png` |
| R9 | Condition figures whole | 78dbb3b | AREA, RUN, COUNT, SQUARES whole and clear of the footer; the selected list row's summary line is cut mid-glyph by the list's own edge (new, listed) | `final-condition-75b1468.png` |
| R10 | One-piece names | 093157d | (auditor cut off by rate limit; executor) "Barbed plates, 3 in", "Insulation adhesive, dual ta…" one span, ellipsis at the end | `final-estimate-75b1468.png` |
| R11 | Stretch-out everywhere | 600a660, c83251a, 40d7acf | Coping label "19 1/2″ stretch-out"; no "girth" anywhere in three captures | `final-section-75b1468.png`, `final-condition-75b1468.png` |
| R12 | Hours once; counts whole | e29b3dd, refined 36aefb6 | (executor) "4,498 EA", "10 EA"; the dumpster keeps 6.86 measured beside 7 bought | `final-estimate-75b1468.png` |
| R13 | ◈ only where the core solved | 1346636 | (executor) hours unmarked; traced-LF lines unmarked; COPING_LF lines marked | `final-estimate-75b1468.png` |
| R14 | Whole boxes on the demo | c945627 | (executor) "buy 5.00 BOX", "buy 2.00 BOX"; selling price 228,894.06 | `final-estimate-75b1468.png` |
| R15 | CLAUDE.md names real-job.mjs | 7c0904c | No picture | — |

## R2's new reds, listed

Sweep 1 (reach gate, no scroll): section2-sheet, section4-library, section5-lenses (14 rows),
section6-model (7), section6b-panel (2), section7-legibility (1), section9a-export, section10a-chrome
(2), section-persona (1) — every one a click on a row below the fold or on a menu row read in the
same tick it was opened. The gate learned what a hand does (scroll once to the nearest edge; open,
wait, then click). Sweep 2 (with that): everything green except **section7 §1.9, one row 51 px
against 50** — a name wrapped in the two-span title. R10 took it. Sweep 3 (all items in):
every probe green after six older probes were taught the repaired behaviours (vocabulary-check, section2-sheet, section4-library expect stretch-out; section2-sheet, section-labor, section-persona read the mark as D175 defines it), plus one row of the buy span's line box that had made a 51 px row among 50s in this container's fonts. The one probe that had to change for a product word was `vocabulary-check`, which
expected "Girth" on screen; it expects "Stretch-out" now, per D134.

## Money on the demo

$228,099.66 at `2bd6894` → $228,099.67 after R4 (the cent the footing found) → **$228,894.06** after
R14 (whole boxes on five fastener and plate lines; no unit price moved). The fixture comparison
cannot run on this clone and wants running on Patrick's machine.

## What the executor got wrong on the way, on the record

- Edited tracked files while a sweep was running, twice; the stamp refused every later probe and the
  sweep had to be rerun. Rule kept from then on: nothing in the tree moves while a sweep runs.
- The reach gate first packed a script's return before the driver saw it, so a promise came back as
  an empty object and two probes read "[object Object]". Fixed under R2.
- R3's no-drawing hint stayed on the demo's Plan under a drawn plan, seen in R4's own picture and by
  both auditors of that picture. Fixed under R3.
- R8's first patch redeclared a name in the same function; the hook refused it.
- R6's fastening-plan title ran past the core test's 140-character limit for a title line; shortened.
- The vocabulary probe expected the old word; changed with the receipt.

## Not done, on purpose

B1 (four-zone wind geometry) and B2 (course lettering) are labelled on the plan, the panel, the
Section and the tapered lines, and not changed: each waits for an outside oracle (round-2 §6). The
first-open frame is Q10. `compare-fixtures` and `real-job` need `fixtures/`.

## Delegation ledger, this round

| Agent | Model (transcript) | Task |
|---|---|---|
| scout, auditor | sonnet ×1, opus ×1 | gate A proof reads |
| auditor ×11 | claude-opus-5 | cold readings: R1, R3, R4, R5, R6 ×3, R7/R11, R9/R11, estimate rows (three cut short by the session's rate limit after reporting or mid-read) |
| builder | — | loaded mid-session, not used: the patches were staged by the main seat in the scratch worktree before it appeared, and said so |
| main seat | Fable | every edit, commit, build, probe run and capture, as logged shell commands |

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01WQsmiydMGxXkd2Seuvitsu
