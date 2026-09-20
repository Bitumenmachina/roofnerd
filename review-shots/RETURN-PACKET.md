# Return packet — cloud executor to the web seat

Against `main` at `2bd6894`, 2026-09-20. Branch `review/web-seat-2bd6894`. Review only: no file
under `packages/`, `src-tauri/`, `data/`, `jobs/` or `tools/` was changed. The eight audit tables,
the twelve pictures, the auditors' readings and the capture script are beside this file.

Marks: `[record]` read or run here, place named; `[inference]` reasoning from records.

---

## 1. Gate A — models pinned and proven

- `claude --version`: **2.1.278 (Claude Code)**.
- The two environment boxes were **not filled**: none of `CLAUDE_CODE_SUBAGENT_MODEL`,
  `CLAUDE_CODE_FORK_SUBAGENT`, `CLAUDE_CODE_DISABLE_EXPLORE_PLAN_AGENTS` was set, and GTK, WebKit,
  WebKitWebDriver and ImageMagick were absent at session start. Rule A2 carried the pin: every agent
  call named `sonnet` or `opus` explicitly. The packages were installed mid-session by the executor
  (apt, exit 0; WebKitGTK 2.52.6, GTK 3.24.41).
- The roster is on the branch: `.claude/agents/{scout,runner,auditor}.md` and `.claude/settings.json`
  with the deny list. It loads at the next session start, not this one; in this session the
  read-only `Explore` type stood in for all three roles, always with a per-call model. The built-in
  `general-purpose` type was refused by the session's permission classifier on both attempts, so no
  agent in this session ever had a write tool. That is why four scouts and both auditors handed
  their tables back in chat and the executor pulled them from the transcripts verbatim (each file
  says so in its header).
- Transcript proof, `grep -h -m1 -o '"model":"[^"]*"' ~/.claude/projects/*/*/subagents/agent-*.jsonl | sort | uniq -c`:

```
     14 "model":"claude-opus-5"
      6 "model":"claude-sonnet-5"
```

  Twenty agents ran in this session; **none on a Fable model**. The two proof reads were
  `agent-a9efb28f…` (sonnet, `wc -l CLAUDE.md` → 159) and `agent-a2b59d4…` (opus, `wc -l README.md`
  → 171). Two surveys started before the operator's instruction arrived also show `claude-opus-5`:
  `Explore` is capped at Opus, as the documentation says. Gate A passes.

## 2. Gate B — a window opens in the container

| Step | Result `[record]` |
|---|---|
| `git config core.hooksPath .githooks` | set; the hook then ran on every review commit (build, engine 191, app 41, shell **14/14** now that GTK is present, core 122, three gates PASS) |
| `pnpm install --frozen-lockfile` | done, 93 packages |
| `git fetch --unshallow` | done: 161 commits, `.git/shallow` gone |
| `cargo install tauri-driver --locked` | 2.0.6 at `~/.cargo/bin/tauri-driver` |
| `pnpm tauri build --no-bundle` | first run **failed**: `generate_context!` could not read `packages/app/dist/assets/pdf.worker.min-….js` because the pre-commit hook's own `pnpm -r build` rewrote `dist/` with new hashes while cargo was compiling. Second run, nothing else touching `dist/`: **built**, `src-tauri/target/release/roofnerd`, 14.3 MB |
| `xvfb-run -a -s "-screen 0 1440x900x24" node tools/probe/front-door.mjs` | **PASS 18/18** |
| `… node tools/probe/runtime-check.mjs` | **PASS 9/9** (including "the webview cannot reach the network") |
| Model pane under the virtual display | rendered (S6); `LIBGL_ALWAYS_SOFTWARE` not needed |

Gate B passes. The container is a place where a window opens; a change to a screen can be looked
at here.

## 3. Gate C — the pictures

Twelve files in `review-shots/`, stamped `7e6f3a5` (the review branch's head when taken; the
product tree is `2bd6894`'s, unchanged). Each auditor was an `opus` agent handed one PNG and one
line; the full three-line readings are in `auditor-readings.md`. Condensed here, with the
executor's own reading only where it differs.

| Shot | Auditor's reading (condensed) | Executor differs |
|---|---|---|
| S1 start screen | Two buttons, "Open a job" and "Open the demo job"; no way to start a new job from a plan; scale never mentioned; four stacked "nothing yet" messages; status strip shows "Flow —" lit blue and a trailing bare "—" with no label; no recent jobs | Agree. The bare "—" is the fifth status field with no name. |
| S2 File menu | **The menu paints as an empty sliver** under the File button; no item legible; no "New job" | Confirmed twice (500 ms and 1,500 ms waits). Cause found in the tree, see §4. |
| S3 Plan + Estimate | Derived lines with ◈ and "buy 11.00 ROLL"; item names truncated mid-word; assembly header quantity clipped to "9,60…"; no unit price column; "42 lines" against "35 more below" | Agree on the clipping. The auditor's 168′×92′ = 15,456 SF objection is wrong: the bottom dimension reads 108′, not 168′, and 108′ × 92′ less the notch is close to the 9,600 SF traced; the auditor misread the string. |
| S4 Recap | Classes each tie; **Job cost prints $194,458.36 while the six classes and the footer sum to $194,458.37**, and Contract amount $223,627.12 is built on the .37; Overhead $0.00 rides silently; Bond row clipped by the pinned footer | Agree. Hours/SQ, which C9 put on the recap, is below the fold and not in the picture. |
| S5 Condition, parapet | "19.5 in girth" on screen; fields without units ("Fastener spacing 12", "Flash up 24"); "elevation 0 ft" on a wall; the picked condition's RUN/COUNT figures **clipped under the money footer**; no per-condition cost visible | Agree. D134 rules "stretch-out" the screen word; **"girth" is on the Condition list and on the Section drawing** (S7). |
| S6 Model | Water direction readable (arrows, valleys, named drains, "no fall · 132 SF"); thickness only as a ramp; "Boards 4 courses" in the strip against "Boards 3" in the list; "Insulation here —" blank until hover; RUN/COUNT clipped again | Agree. The cricket reads as a small orange patch, not two planes (standing item). |
| S7 Section | **Two scales on one screen**: the strip says "Horizontal 1″ = 4′-0″", the drawing's own note says "horizontal 1″ = 5′-4″"; letters overlap "6″ at the wall" and "1/2″ at the drain"; "girth" again; Drain 1 "drawn as a symbol; nothing states the bowl" | Agree. 1″ = 5′-4″ is 3/16″ = 1′-0″, an architect's scale, so the drawing may be right and the strip stale, or the reverse; the executor did not change code to find out. |
| S8a line open | The chain reads well and is editable; the title renders "TPO membrane , 60 mil" with an orphaned comma; header "42 lines" against "44 more lines below"; sibling lines named but not linked | Agree. The orphaned comma is the C8 fix ("the tail after the last comma no longer gives way") drawn as two spans. |
| S8b struck line | The struck alternate is visible with its note; no restore control and no alternate subtotal; four descriptions truncated in the middle ("Shop-formed …, mitre welded"); "buy 0.80 BOX" on fasteners | "buy 0.80 BOX" is D24's rule (plates order at a fraction, as the supplier bills), a decision on the record, not a defect; the auditor was not shown D24. The corners now price 8 EA (C8's fix, confirmed). |
| S9 two windows | Two OS windows, the cost window over the left third of the plan; two labels for one act ("dock it back" / "bring the cost panel back"); the status strip spans the desktop | The windows are chromeless because xvfb has no window manager; on a desktop they carry title bars. Otherwise agree. |

## 4. The blank-job walk (S2)

Steps reached, through the front door with nothing typed: **one**. The start screen offers "Open a
job" (the operating-system folder picker, which WebDriver cannot drive) and "Open the demo job".
The File menu's DOM holds four items — `File`, `Open a job…`, `Save`, `Export IFC` — and no
"New job", no "Recent". The walk stopped at the picker.

What a roofer would see is worse than the stop: **the File menu is invisible.** `.doc-bar` carries
`overflow: hidden` (`packages/app/src/app.css:255`, commit `cab2a465`, 2026-09-12, the 10a chrome)
and `.menu-list` is `position: absolute; top: 100%` inside it (`app.css:234-237`), so the dropdown
is clipped to a 12 px sliver below the button. Confirmed with the menu open for 1,500 ms
(`S2b-file-menu-1500ms-7e6f3a5.png`). Every probe that uses the menu — `section-persona.mjs:222-225`,
`section-ifc.mjs`, `section9a-export.mjs` — clicks `.menu-item` through the DOM, where a clipped
element still clicks. So File → Save and File → Export IFC have been green and unreachable by a
hand since 2026-09-12, through C0–C9 and the persona pass. `[record]` This is ledger row 1's
mechanism again: the check took a path a person cannot. Whether a person's WebKitGTK paints it the
same way is a CSS fact, not a container fact, but Patrick's own `evidence/` at `2bd6894` would
settle it in one look.

Behind the picker, `read_folder` (`src-tauri/src/lib.rs:85-92`) skips a missing part
(`NotFound => continue`), so an empty folder would open as an empty job — that is the only
"new job" the program has, and nothing on screen says so. `[record + inference]`

## 5. The audit tables, D1–D8

| # | File | What it found, in one line |
|---|---|---|
| D1 | `D1-history.md` | 161 commits on four dates: 9 · 15 · 41 · **96** (2026-09-13). 19 record-only, 139 product. 61 subjects say fix/round/red/check/probe. STATUS grew 20 → 46 → 70 → 121 KB; DECISIONS 19 → 46 → 66 → 106 KB |
| D2 | `D2-decisions.md` | All 162 D-numbers classed. **35 decisions that move money or change a screen cite no ruling** — the executor's own call — including every C-batch money decision: D149, D150, D151, D152, D153, D154, D157, D162 |
| D3 | `D3-requests.md` | Tracked request found for 9a, 10a, 10b, C0, C5, C6, C7, C9 and (weakly, one word) C8; **C2, C3, C4 cite only the untracked plan file; C1 has no request in the tracked tree** |
| D4 | `D4-probes.md` | 24 of 25 probes open the demo job; `real-job.mjs` opens `fixtures/` **through a direct `doc_open` invoke**, which `CLAUDE.md` says no probe does. None opens an empty job |
| D5 | `D5-oracles.md` | **Eleven of twelve solver outputs have no outside oracle in the tree**; the IFC writer's oracle (web-ifc, IfcOpenShell) checks the file's shape, not any quantity. `taper.rs`, `walls.rs`, `boards.rs` carry no unit tests of their own |
| D6 | `D6-delegation.md` | No model is named for any agent anywhere. 9a is the one batch STATUS says an agent built; C0 and C7 say the orchestrator ran the probe; the rest name no actor |
| D7 | `D7-D8-drift-and-domain.md` | README's status paragraph lists 9a and 6b as both built and next, names no C-batch, and its layout section never mentions `src-tauri/core`; no `.github/`, no `.claude/` at HEAD; hook off on a fresh clone |
| D8 | same file | `WIND_STANDARD = "ASCE 7-22 (Georgia 2024)"` while `dimension_a` is `min(0.1·least, 0.4·h)` floored at `max(0.04·least, 3 ft)` — the three-zone, 7-10-shaped rule, cited to NRCA 2019 Appendix 1. The 120-for-130 mph substitution is named only in a source comment and, vaguely, in one line's note; the cricket's primary-slope pricing only in a source comment. **The engine CLI recap on the demo prices $113,896.60 with 24 lines "not in the total"** because the command line has no solve |

## 6. Every `[check]` in §5 and §6, resolved

| Check | Verdict | How |
|---|---|---|
| §5a label what each suite proves (D5) | **confirmed** | D5 table; no class-4 oracle for any solver quantity |
| §5a wire types kept in step by hand | **confirmed** | `grep -rl solved.ts src-tauri/core/tests` → none; tests read only `roof.ts` (5 refs) and `model.ts`'s `LAYER_FUNCTIONS` (1) |
| §5b wind geometry edition (D8) | **confirmed** three-zone `a` rule under a 7-22 label; whether the typed pressures' edition matches is P9 |
| §5b adhesive substitution on the line (D8) | **refuted**: no line says 120 or 130 mph; the corner line's note says "the table stops below this coast" |
| §5b cricket approximation on the line | **refuted**: source comment only (`make-demo-prices.mjs:380-384`) |
| §5c blank-job path (S2) | **could not determine past the picker**; and the menu is clipped (§4) |
| §6 trace from an empty job (S2) | could not determine — same door |
| §6 labor shapes hourly and per-square | **confirmed on screen**: "2 at 120 a crew-day", "70 a shop hour" (S8b) |
| §6 three supply houses side by side | **refuted**: one dropdown, one scenario at a time (S3, S8) |
| §6 assemblies carry a picture and notes | notes **yes** (`library.json` assemblies: `notes`), picture **no** field at all |
| §6 IFCX alpha, toolkit, property sets unrequested (D3) | 10b and C7 cite Patrick's 2026-09-12 ruling (D141) and Addendum 7's "IFCX export is re-opened" (STATUS:799) — **tracked**; the three property sets cite "the charter" (the plan file) — **not in the tracked tree** |
| §3 row 16 vocabulary-gate exception request | **executor's own call**: D159's reason is its own ("buildingSMART's own word"); D2 classes it so |
| §2 tauri-driver on PATH | was not; `cargo install` put it there |

## 7. What this review got wrong, or did not know

1. **"Expected: none" for D4 was wrong.** One probe leaves the demo, and it does so by the bridge
   call `CLAUDE.md` forbids. Worth its own ledger row.
2. **The cloud "limit" was a two-command gap**, as §4 guessed: apt and `xvfb-run`. Everything the
   record says only Patrick's machine could run, ran here except the fixture comparison and
   IfcOpenShell.
3. **§2's fear about `Explore` was the wrong one.** It ran at Opus with no pin at all; the thing
   that actually blocked was the permission classifier refusing `general-purpose`, which also
   removed every write tool from every agent.
4. **§5a "one push of 116 commits"** understates it: 96 commits on one day out of 161, and the log
   could only be seen after `--unshallow`.
5. **Row 3 of the S3 auditor's reading is a misread** (108′ read as 168′); the executor kept it in
   the readings file rather than editing an auditor's words.
6. **Two findings the work order did not ask for and the pictures gave**: the clipped File menu
   (§4) and the Section's two scales (S7). Both are "green while wrong" and both are in the
   shipped program at `2bd6894`.
7. **The web seat's §4 claim that the C-run agents "never saw a pixel"** stands, and this session
   repeated the shape once: the D-audits ran blind by design. The pictures were the one place an
   agent was shown the product, and it found more than the tables did.

## 8. Delegation ledger for this session

| Agent | Model (transcript) | Task | Handed back |
|---|---|---|---|
| proof read | claude-sonnet-5 | `wc -l CLAUDE.md` | 1 line |
| proof read | claude-opus-5 | `wc -l README.md` | 1 line |
| scout D1 | claude-sonnet-5 | history tables | ~100 lines, wrote its file |
| scout D2 | claude-sonnet-5 | 162 decisions classed | ~200 lines, pulled from transcript |
| auditor D3 | claude-opus-5 | request provenance | ~40 lines, pulled |
| scout D4 | claude-sonnet-5 | probe routes | ~35 lines, wrote a placeholder; table rewritten from its report |
| auditor D5 | claude-opus-5 | test oracles | ~90 lines, pulled |
| scout D6 | claude-sonnet-5 | delegation grep | ~70 lines, pulled |
| scout/runner D7–D8 | claude-sonnet-5 | drift and domain prints | ~200 lines, pulled |
| auditors S1–S9 | claude-opus-5 ×9 | one picture each | 3 lines each (S8: 6) |
| two surveys (before the pin instruction) | claude-opus-5 ×2 | code inventory | stopped; nothing used |

Builds, installs, probes and the capture ran as background shell commands from the main seat with
logs, not through an agent: the roster's `runner` was not loadable this session and a logged
command costs the main seat one `tail`. Every command and its exit code is in §2.

## 9. Where things are

- Branch `review/web-seat-2bd6894` on the remote; draft pull request "Review only — do not merge".
- `review-shots/`: `S1…S9*.png`, `S2b-file-menu-1500ms-*.png`, `capture-notes-*.md`,
  `auditor-readings.md`, `D1…D8` tables, `capture.mjs`, this packet.
- `.claude/`: the roster and the settings pin.
- Nothing under `evidence/` was kept (gitignored; the probes' own shots landed there and stay).
- §9 of the work order is untouched: proposals are Patrick's to rule on.
