# D3 — who asked for each batch (auditor, opus)

(pulled verbatim from the agent transcript; the agent ran read-only and could not save it)

I have everything. One constraint to flag first.

**I could not save the file.** This role is read-only with no write tools, so `/home/user/roofnerd/review-shots/D3-requests.md` was not created. The directory exists and is empty. The full table is below, ready to paste.

# D3 — who asked for each batch

| batch | tracked sentence (quoted, ≤30 words) | file:line | verdict |
|---|---|---|---|
| 9a (CSV export) | "Addendum 6: Patrick's 2026-09-12 frame names estimating, project management and design in one 3D tool; export is Gate 3's debt and A3's panel-in-every-editor is unmet." | /home/user/roofnerd/DECISIONS.md:128 (D99) | tracked request |
| 10a (window chrome to the handoff's anatomy) | "Addendum 7 adopts the handoff's anatomy (document bar · tool settings strip · tool rail of roofer's verbs · one surface with Plan / Model / Section switched in place…)" | /home/user/roofnerd/STATUS.md:793 | tracked request |
| 10b (@thatopen/components, web-ifc) | "The 3D toolkit (section 10b, added 2026-09-12 — Addendum 7 / Patrick's redirection: use what exists)" | /home/user/roofnerd/DEPENDENCIES.md:64 | tracked request |
| C0 | "Patrick ruled that the 3D model is the function, not a view." | /home/user/roofnerd/STATUS.md:656 | tracked request |
| C1 | "C1 the field on drainage regions with four-way planes, valleys at 45°, the cone retired (D149)" — a record of what landed, naming no requester | /home/user/roofnerd/STATUS.md:16 | no request found |
| C2 | "This closes the B1 pushback the plan names at S1 step 4." — not in the tracked tree — cites the plan file | /home/user/roofnerd/STATUS.md:544 | cites untracked refs only |
| C3 | "`coping.rs` land the S2 solver the plan named" — not in the tracked tree — cites the plan file | /home/user/roofnerd/STATUS.md:391 | cites untracked refs only |
| C4 | "The plan's own sketch figure for this step … is wrong against the transcribed sheet" — not in the tracked tree — cites the plan file | /home/user/roofnerd/DECISIONS.md:186 (D157) | cites untracked refs only |
| C5 | "Patrick, 2026-09-13 02:30: the look is acceptance for C5." | /home/user/roofnerd/DECISIONS.md:183 (D156) | tracked request |
| C6 | "The tool rail offers all eight of the handoff's acts, and the four that were missing now do something." | /home/user/roofnerd/STATUS.md:286 | tracked request |
| C7 | "Patrick 2026-09-12: 'a working 3D software, not more gates or more mess … use all your resources … Unix principle … IFC4 or IFC5.'" | /home/user/roofnerd/DECISIONS.md:170 (D141) | tracked request |
| C8 | "Order: 6c → 10a chrome → 10b renderers → 10c → 8 Schedule → Undo → IFCX → persona." | /home/user/roofnerd/STATUS.md:800 | tracked request (see note) |
| C9 | "Patrick's ruling for this batch (2026-09-13 09:30): the test of every line is whether a commercial roofing estimator reads it as their own." | /home/user/roofnerd/DECISIONS.md:190 (D161) | tracked request |

## Notes and corroboration

- **10a** — corroborated by /home/user/roofnerd/DECISIONS.md:166 (D137, "The handoff's frame; supersedes D121"), :161 (D132, "Addendum 7; the handoff has no tree"), :160 (D131, "Addendum 7"), and /home/user/roofnerd/STATUS.md:800, which names "10a chrome" in Addendum 7's order.
- **10b** — corroborated by the D141 Patrick quote at /home/user/roofnerd/DECISIONS.md:170, which is the same ruling C7 rests on.
- **C0** — corroborated by /home/user/roofnerd/DECISIONS.md:175 (D146): "Patrick, 2026-09-12: the 3D model is the function, not a view; it is what has to get built…".
- **C1** — no quoted ruling, no addendum, no handoff section anywhere in the tracked tree. Its cited authority is a book (NRCA *Membrane Roof Systems* 2019, ch.4 pp.163–168, /home/user/roofnerd/STATUS.md:586–587) and D71's correction (/home/user/roofnerd/DECISIONS.md:178) — both reasons, not requests. The nearest thing to a request is the previous batch's own "Next: C1 — regions and planes" at /home/user/roofnerd/STATUS.md:723, which is the build seat choosing its own next step.
- **C2 / C3 / C4** — every request citation points at the plan file, named at /home/user/roofnerd/STATUS.md:32 as `~/.claude/plans/handoff-is-complete-and-foamy-ember.md`, which is outside the repository. Further plan-only cites: /home/user/roofnerd/STATUS.md:538 and :556 (C2). D150–D152 and D153–D154 carry no Patrick quote, addendum number or handoff section; D151 mentions the handoff only to say its tapered figures "were a sketch".
- **C6** — the handoff reference is explicit on both sides of the seam: D158 at /home/user/roofnerd/DECISIONS.md:187 names "The four acts in Frame 1's rail that work ON the roof — Set high and low, Box out a curb, Measure, Cut a section".
- **C7** — also /home/user/roofnerd/STATUS.md:799, "IFCX export is re-opened after the chrome", inside the Addendum 7 paragraph. Worth flagging for the reviewer: D105 at /home/user/roofnerd/DECISIONS.md:134 had recorded the opposite — "No IFC export and no glTF export in this run … Nobody asked (register #11)" — so D141 is a reversal with a receipt.
- **C8 — caveat, the weakest cell in the table.** No tracked sentence names C8 or asks for a persona pass by that name. The single word "persona" at the end of Addendum 7's order (/home/user/roofnerd/STATUS.md:800) is the only tracked authority, and it was written 2026-09-12 against the 6c–10c sequence, not against the plan-v3 C-batches of 2026-09-13. D160 (/home/user/roofnerd/DECISIONS.md:189) makes the persona pass the acceptance standard *after the fact* and its reason column is a justification, not a request. The pass itself lives at `.scratch/c8-persona.md`, which `.gitignore` excludes. Read strictly, C8 is closer to "no request found".
- **Collective framing.** "Plan v3 is built" at /home/user/roofnerd/STATUS.md:8 covers C0–C9 as a set, but the plan it names is the untracked file, so it cannot stand as a tracked request for any individual C batch.

**Tally:** 8 tracked request (one of them, C8, weak) · 3 cites untracked refs only (C2, C3, C4 — all the plan file) · 1 no request found (C1).
