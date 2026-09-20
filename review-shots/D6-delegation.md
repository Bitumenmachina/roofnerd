# D6 — delegation trace (scout, sonnet)

(pulled verbatim from the agent transcript; the agent ran read-only and could not save it)

READ-ONLY NOTICE: I could not save to /home/user/roofnerd/review-shots/D6-delegation.md — this scout role is configured strictly read-only (no file creation/writes permitted). The full report content is below in full; please save it yourself if a file is needed.

Method: `grep -inE` whole-word, case-insensitive for agent(s), orchestrator, clone, reader, seat(s), sonnet, opus, haiku, fable, and the phrase "model ran" across STATUS.md, NOTES.md, DECISIONS.md. NOTES.md: zero hits on any term. sonnet/opus/haiku: zero hits anywhere. "model ran": zero hits anywhere. fable: one hit (STATUS.md:1210).

---

# D6 — Delegation grep (agent/orchestrator/clone/reader/seat/model-name terms)

## (a) Every hit

| File:Line | Matching phrase |
|---|---|
| STATUS.md:144 | "...the twenty-second was the **clone's** own rewrite of a figure-range comment..." |
| STATUS.md:206 | "this **clone** and `pnpm build:app` is outside this batch" |
| STATUS.md:275 | "Never run here — no binary in this **clone** and a probe opens a window this sandbox has" |
| STATUS.md:276 | "none of. The **orchestrator** runs it, then runs `tools/verify-ifc.py`..." |
| STATUS.md:364 | "...the document the patterns came from, and the two things a **reader** has to be told" |
| STATUS.md:386 | "sheet sends the **reader** to an evaluation report that is not on this machine" |
| STATUS.md:429 | "no binary, no `pnpm build:app`, no window to open in this **clone**" |
| STATUS.md:695 | "The check, to be run by the **orchestrator** (probes open windows; nothing was run from the..." |
| STATUS.md:696 | "build **clone**): `pnpm build:app`, then `node tools/probe/section-core.mjs`" |
| STATUS.md:769 | "A **reader** opened the 29 remaining `b7a4228` images" |
| STATUS.md:790 | "whether the **seat** was driving or following. The answer was: following." |
| STATUS.md:792 | "had never been opened by any **seat**, and this one ruled for Appendix A3..." |
| STATUS.md:810 | "Built by the 9a **agent** in a scratch **clone** (its notes are the record...)" |
| STATUS.md:829 | "The 9a **agent's** findings, on the record: `window.__errors` was set by nothing..." |
| STATUS.md:939 | "...green after). Both rounds by the 9a **agent**; both defects seen in..." |
| STATUS.md:944 | "...reads as a contradiction to a first-time **reader**; the..." |
| STATUS.md:1029 | "Five templates, the **reader** picks one, and a lens leaves things out..." |
| STATUS.md:1191 | header: "## What the **seats** settled, 2026-09-08" |
| STATUS.md:1193 | "The web **seat** answered the handoff Code sent it." |
| STATUS.md:1208 | "...naming one without a path already sent a **seat** to the..." |
| STATUS.md:1210 | "...the source list widened, by both **seats**. It does not override **Fable's**..." |
| DECISIONS.md:51 | "A **clone** had the proposal, the open-ended handoff and the addenda, and no authority document..." |
| DECISIONS.md:54 | "The web **seat** recovered §4.10, §4.11, §4.12, §5, §6 and §7 from conversation history..." |
| DECISIONS.md:62 | "Recognized and declined, so no **seat** builds them as a favour" |
| DECISIONS.md:63 | "...naming one without a path has misdirected a **seat** once already." |
| DECISIONS.md:83 | "Naming the product without the path already sent one **seat** to the wrong artifact..." |
| DECISIONS.md:129 | "...QUESTIONS.md answered it 2026-09-08 and this **seat** does not re-open it." |
| DECISIONS.md:137 | "...a field added to the engine silently does not exist in a **clone**" |
| DECISIONS.md:139 | "...the screen check refuses software and AI words (**agent**, AI, inference, token, prompt...)" |
| DECISIONS.md:140 | "A git hook binds every commit from every **seat**, which a Claude Code hook does not" |
| DECISIONS.md:142 | "...a dated folder, not a '-2' suffix nobody can read (9a **agent**)" |
| DECISIONS.md:167 | "...fixtures still within 0.01% (6b **agent**, batch 6c)" |
| DECISIONS.md:172 | "...something the gate reads rather than something a **reader** assumes..." |
| DECISIONS.md:177 | "...a field added in one place silently does not exist in a **clone**." |

NOTES.md: no hits for any term.

## (b) Per-batch delegation table

| Batch | Delegated to an agent (per STATUS) | What orchestrator/main seat did itself (per STATUS) | Model named for any agent? |
|---|---|---|---|
| 6b | Not named directly in STATUS's own §6b text. DECISIONS.md D138 (money/rounding fix that lands in 6c) is credited "(6b agent, batch 6c)" — i.e. STATUS attributes that fix to a "6b agent" only via the DECISIONS.md cross-reference, not within STATUS's §6b prose itself. | STATUS's §6b prose doesn't name who ran the checks/fixes; no "orchestrator" language in this section. | No |
| 6c | STATUS's own §6c text names no agent directly, but the section states the rounding fix (D138, "6b agent") "landed" as part of this batch. | "A reader opened the 29 remaining `b7a4228` images... nothing new that blocks." (a reader, not the orchestrator, did the image review) | No |
| 9a | Explicit: "Built by the **9a agent** in a scratch clone (its notes are the record: what it reused, what it could not run, seven findings)." The 9a agent built the CSV-export feature and left findings (window.__errors set by nothing; `vite build` doesn't type-check) — "The 9a agent's findings, on the record..." | No "orchestrator" language used in this section; who ran the follow-up probe/checks/commits after the agent's build is not stated. | No |
| 10a | No standalone batch report exists. Only appears as a planned step in the Addendum-7 order list ("6c → **10a chrome** → 10b renderers → ...", line 800) and as probe/check names (`section10a`, `section10a-chrome`) folded into B1/C0/C8's check tables. | Not found. | No |
| 10b | Same as 10a — only named as a planned future step ("10b renderers", line 800); no batch write-up exists. | Not found. | No |
| B1 | Not named. | Not named — the section states check results (red→green) impersonally, no orchestrator/agent attribution. | No |
| C0 | Not named. | Explicit: "**The check, to be run by the orchestrator** (probes open windows; nothing was run from the build clone): `pnpm build:app`, then `node tools/probe/section-core.mjs`." | No |
| C1 | Not named. | Not named — "Not run here: the probe... has never executed," no attribution of who would run it. | No |
| C2 | Not named. | Not named — probe "written, landed first and red by construction, and has never executed"; no orchestrator/agent language. | No |
| C3 | Not named. | Not named — "no binary, no `pnpm build:app`, no window to open in this clone" states a limitation but assigns no actor. | No |
| C4 | Not named. | Not named — section contains no agent/orchestrator/clone/reader/seat term at all. | No |
| C5 | Not named. | Not named — no such terms appear in this section. | No |
| C6 | Not named. | Not named — no such terms appear in this section. | No |
| C7 | Not named. | Explicit: "Never run here — no binary in this clone... **The orchestrator runs it**, then runs `tools/verify-ifc.py` against the file it writes and records its rows here." | No |
| C8 | Not explicitly named as an agent doing delegated work, but the landing paragraph notes "the twenty-second [of 22 commits] was **the clone's** own rewrite of a figure-range comment... so it had nothing left to commit" — implying a separate clone/session independently made (a now-redundant) commit during landing. | Not named as "orchestrator" — landing described impersonally ("Landed... through the hook"). | No |
| C9 | Not named. | Not named — no such terms appear in this section. | No |

## Notes / caveats
- No batch text names a specific Claude model (sonnet/opus/haiku) for any agent, anywhere in the three files.
- "Fable" appears once, STATUS.md:1210, as a proper noun ("does not override Fable's..."), not one of the batches in scope.
- "seat(s)" is STATUS/DECISIONS' general term for a work session/contributor (e.g., "the web seat," "this seat," "every seat") — it is never paired with a model name.
- The C0 and C7 "orchestrator" lines are the only two places in scope where STATUS explicitly states the orchestrator itself executed a check/probe (both because the sandboxed build clone cannot open windows).
- 9a is the only batch in scope where STATUS explicitly describes work built by a named agent ("the 9a agent") rather than by the orchestrator/main seat.

