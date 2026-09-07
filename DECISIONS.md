# Decisions

One line each: what was decided, and why. Calls made during the build that the handoff
did not cover. Newest last.

| # | Decision | Why |
|---|---|---|
| D1 | The build handoff lives in `refs/` with the other two ruling documents | `refs/` is gitignored and the handoff is the third document of its kind. It carries no client data, but the smaller boundary is the right default for anything that rules the build. |
| D2 | `STATUS.md`, `DECISIONS.md`, `QUESTIONS.md`, `DEPENDENCIES.md` are public | They carry no client data, and a contributor reading the repo cold should see what was decided and what is open. Client-identifiable material never enters them. |
| D3 | `SF` is a real number only for an **area** condition. A run becomes an area through a formula — `LF * H` — never behind the estimator's back | The handoff's own worked example writes it that way. Two routes to the same number is how a line gets counted twice, and the height stays visible on the line instead of buried in a function. |
| D4 | The open document is keyed by PART NAME — `job`, `pages`, `conditions`, `costCodes` — not by file path | A JSON Pointer splits on `/`, so a key like `pages/pages.json` reads as two steps and can never be addressed. Windows now say `/pages/0/feetPerUnit`, which is what it should have looked like anyway. Found by the section 1 probe; it was latent in the Rust side too. |
| D5 | No Vite dev server. The front end builds to static files and the shell serves them over its own protocol in development exactly as in a release | It makes the content security policy live while working. A dev server would leave the local-only claim untested until a release build, which is the wrong way round for the one property the program is publishing its source to prove. |
| D6 | Probes drive the real built front end in headless Chrome (`puppeteer-core`), with only the shell stubbed | A done-check the executor can run itself, rather than a description of one. Everything above the shell boundary — editors, surface, tools, engine — is the shipped code unmodified. |
| D7 | `page.render()` is cancellable and paints are serialized; the overlay is sized before the paint, not after | pdf.js refuses two paints on one canvas, which is exactly what leaning on the zoom key asks for. Sizing the overlay first also means a trace is correctly placed while a big sheet is still drawing underneath it. |
| D8 | The dialog plugin is in; there is no file-system plugin | Picking a drawing needs the operating system. Reading one does not: `read_page_source` resolves inside the open job and refuses anything outside it, so a window cannot name a file anywhere on the machine and have this program read it out. |
| D9 | A drawing that fails to open says so on the hint line | A blank sheet with no explanation is the worst thing the editor could do — the estimator would trace nothing and never learn the file was the problem. |

