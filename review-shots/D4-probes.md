# D4 — how each probe gets a job open (scout, sonnet)

| File | Route | Evidence |
|---|---|---|
| evidence-shots.mjs | demo via `openDemoJob(` | :51 |
| front-door.mjs | clicks the start screen's demo button directly (`.start-demo`) | :71 |
| real-job.mjs | **`fixtures/real-job` via a direct `doc_open` invoke** (`window.__TAURI_INTERNALS__.invoke('doc_open', …)`) | :60; comment :53–58 says the native picker cannot be driven |
| runtime-check.mjs | demo via `openDemoJob(` | :46 |
| section-bill.mjs | demo via `openDemoJob(` | :488 |
| section-core.mjs | demo via `openDemoJob(` | :292 |
| section-fastening.mjs | demo via `openDemoJob(` | :383 |
| section-ifc.mjs | demo via `openDemoJob(` | :143 |
| section-labor.mjs | demo via `openDemoJob(` | :261 |
| section-model.mjs | demo via `openDemoJob(` | :253 |
| section-persona.mjs | demo via `openDemoJob(` | :250 |
| section-section.mjs | demo via `openDemoJob(` | :438 |
| section-taper.mjs | demo via `openDemoJob(` | :376 |
| section-verbs.mjs | demo via `openDemoJob(` | :439 |
| section-walls.mjs | demo via `openDemoJob(` | :211 |
| section1-trace.mjs | demo via `openDemoJob(` | :45 |
| section10a-chrome.mjs | demo via `openDemoJob(` | :160 |
| section2-sheet.mjs | demo via `openDemoJob(` | :46 |
| section4-library.mjs | demo via `openDemoJob(` | :56 |
| section5-lenses.mjs | demo via `openDemoJob(` | :53 |
| section6-model.mjs | demo via `openDemoJob(` | :198 |
| section6b-panel.mjs | demo via `openDemoJob(` | :188 |
| section7-legibility.mjs | demo via `openDemoJob(` | :39 |
| section9a-export.mjs | demo via `openDemoJob(` | :277 |
| vocabulary-check.mjs | demo via `openDemoJob(` | :96 |
| tauri-harness.mjs | harness; defines `openDemoJob()` | :195 |

Probes that start from anything other than the demo job: **1 of 25** (`real-job.mjs`), and it does so
through `doc_open`, the bridge call `CLAUDE.md` says no probe makes ("None of them calls `doc_open`").
No probe opens an empty job or a job it created.
