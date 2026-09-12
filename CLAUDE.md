# Working on roofnerd

Read this first, every session. A session does not remember the last one; the repository
does. What follows is the part that is expensive to rediscover.

## What this is

Roofing estimating software. Trace the roof on one screen, watch the money move on the
other. `README.md` explains it the way a roofer would want it explained; `STATUS.md` says
where the build actually is; `DECISIONS.md` says what was decided and why.

The ruling documents — the handoff and its addenda — are in `refs/`, which is local-only.
If `refs/` is empty on this clone, ask before assuming what the project wants.

`refs/carry-forward-register.md` is the first thing to read there. It is the list of things
that have been re-derived more than once or lost at least once across this product's
iterations — every entry was paid for, and the register exists because the standing complaint
about this work is that each pass drops earned context. It rules nothing; where it disagrees
with a ruling document, the ruling document wins.

## Search before asking — three sources, in order

This rule exists because it was broken. A session reported sheet-metal stretch-out, dimensioned
details, gauge tables, the pitch factor and assemblies as "gaps that must be authored from
nothing." All five were on this machine, and `~/KNOWLEDGE` has a machine-readable index of itself
that was never opened. Nine questions went to Patrick that a search would have answered.

1. **The box** — the repo, `refs/`, `DECISIONS.md`, `QUESTIONS.md`, the carry-forward register,
   `~/KNOWLEDGE`, the prior lineage's trees. **Where a tree has an index or a manifest, that is the
   first read, not the last.** `~/KNOWLEDGE/index/index_audit.md` states what the index covers and
   what it misses — one read instead of a crawl, and the manifest is CSV and JSON, so it needs no
   model and no retrieval service. A gap claim made without reading an available index is a defect
   in the same class as a claim made with no search at all.
2. **The prior sweeps** — scored landscapes that already exist. A scored list that exists and goes
   unread is worse than no list.
3. **The open internet** — this is an open-source program built against current public work, not
   against a model's training weights. The seat has web search. Not using it is a defect, not a
   style.

**Do not invent what already has a name.** Roofing is a solved trade with published standards, a
settled vocabulary and thirty years of geometry literature. Girth, tapered layout, hip-and-valley
generation, provenance on a measurement — none are novel, and none need a roofnerd-original word,
table or algorithm. Coining one is drift, and it is the expensive kind because it looks like
progress.

**A question to Patrick is a last resort with a receipt.** Before an entry reaches `QUESTIONS.md`,
all three sources are searched and the search came back empty. If it came back with something
merely inconvenient, that is a **decision with a reason** in `DECISIONS.md` — not a question handed
back to him.

**Anything meant to survive is written to disk in the turn it is produced.** A paste cache is not
storage. Two documents have already been lost this way.

## Never

- **No client data in the public tree.** Nothing out of `fixtures/`, `refs/` or `evidence/`.
  No bid names, no job numbers, no home paths, and no quantities, prices, hours or margins
  lifted from a real report — not in tests, not in comments, not in a commit message. This
  has gone wrong twice, both times because a real figure was in front of me while writing a
  test and it made the test feel real. Invent the number. It reads the same.
  `tools/gate-clientdata.sh` runs as a pre-commit hook; turn it on with
  `git config core.hooksPath .githooks`.
- **Never loosen a gate.** If a gate fires on legitimate code, fix the code or narrow the
  pattern with a comment saying why. The three gates are the vocabulary gate, the egress
  detector and the client-data gate, and each was proven to fail before it was trusted.
- **No network call in the product.** The content security policy keeps `connect-src` at
  `'self'`, there is no HTTP plugin, and `tools/probe/runtime-check.mjs` tests it in the
  shipped runtime rather than asserting it.
- **No feature nobody asked for.** Unrequested features are the named defect this project
  was started to escape. If it is not in the handoff or an addendum, it is not built.
- **Never force-push.** History is public.
- **Write only inside this repository.**

## Always

- **The vocabulary in `README.md` is the only vocabulary.** Job, Scenario, Page, Condition,
  Item, Formula, Order unit, Assembly, Library, Cost code, Class, Recap, Lens. Units are SF,
  LF, EA, SQ. No synonyms in the UI, the file format, the types or the comments.
- **Nothing is ever silently zero.** A quantity off an unscaled sheet is pending. A line
  with no price says so. A total that leaves something out says what it left out. This is not
  a principle you prove once — it has been closed at the engine, found again in the formula
  parser, and found again in the library data. Prove it at every layer a number can collapse.
- **Geometry is an imperative, not a feature.** Open source, import/export, geometry — the three
  things this program exists for. *"Builders see in 3D — I cannot stress how imperative that is."*
  It has been slotted late twice and pulled forward twice. It is not a bonus.
- **A formula is editable on the line, and a validation pass must never strip what the sheet can
  author.** A library you cannot see into is a library nobody trusts. This has already been ruled,
  reversed by a sanitizer that deleted line-level formulas on load, and reversed back.
- **The stretch-out belongs to the detail, not to the material.** It is 19½″ at a coping, 8″ at a
  gravel-stop edge, 26″ at an equipment curb cap — so it can never be a stored constant on a library
  item. The sheet-metal case is what the formula language exists for: LF of profile becomes pounds
  through a width that changes per detail. A formula design that cannot say this is wrong however
  clean it looks.
- **A check that bypasses the path a person uses is not a check.** Every runtime probe
  starts at the start screen and goes through the menu, the buttons and the fields. None of
  them calls `doc_open` — or any other command — to get a job open. This is not a
  preference: every probe passed for days while the program could not be given a job at all,
  because File → Open a job called `window.prompt`, which this webview does not implement,
  so it returned nothing and the handler gave up quietly. "8/8" only counts when it went
  through the front door. Use `openDemoJob(session)` from `tools/probe/tauri-harness.mjs`.
- **No `prompt`, `alert` or `confirm`.** This webview does not implement them: they return
  nothing, the caller gives up, and the feature silently does not exist. Anything the program
  asks, it asks in a field in the window. `tools/gate-egress.sh` refuses them.
- **A section is done when its check passes against the shipped runtime**, not against a
  browser with the shell stubbed. `pnpm build:app`, then `node tools/probe/front-door.mjs`
  and `node tools/probe/runtime-check.mjs`. Evidence goes in `evidence/` named with the
  section and the commit.
- **Look at every image, as a person, before anything is called green.** A check that passes
  while the window is wrong is a defect in the check, not a pass. This is the rule that was
  agreed and then left in a gitignored file, so it bound nobody: three sections were certified
  without one of their evidence images being opened, and section 7 — twelve defects entirely
  about how the program looks — was certified with the disproof of its own §1.9 sitting in the
  "after" screenshot nobody opened. A count of green rows is not a review. The review is
  opening the picture and reading it the way the estimator will.
- **Evidence is the product, opened through the front door, and nothing else.** Every capture
  starts at the start screen and opens the demo job the way a person does — `openDemoJob(session)`
  — and is stamped with the commit the section closes at. A probe that dresses a scene by calling
  `doc_open` is photographing a set, not the program. Probe scratch states never land in
  `evidence/`.
- **Prove a gate red before trusting it.**
- **Report at the end of every session** by updating `STATUS.md`: what was built, what check
  passed, what is next, anything new in `DECISIONS.md` and `QUESTIONS.md`, and what Patrick
  should look at if he wants to. Reporting is not stopping — the next session opens at the
  next section without waiting for a reply.
- **Decide, record, continue.** If something is not covered, make the call and write one
  line in `DECISIONS.md` saying what and why. If a wrong call would waste work, write it in
  `QUESTIONS.md` and build the smallest version that lets the rest proceed. Neither is a
  stop.

## Running it

    pnpm install
    pnpm dev                                    # the application
    pnpm -r test                                # the engine
    pnpm gates                                  # vocabulary, egress, client data
    pnpm build:app                              # deb, rpm, AppImage
    node tools/probe/front-door.mjs             # start screen → buttons → fields
    node tools/probe/runtime-check.mjs          # the shipped runtime, two windows
    node tools/compare-fixtures.mjs             # needs fixtures/, which is local only

`packages/engine` is pure arithmetic with no window anywhere near it — if a number is wrong,
it is wrong there, and there is a test there that should have caught it. `src-tauri` holds
the one open job and hands the same copy to every window. `packages/app` are the editors.
