# Working on roofnerd

Read this first, every session. A session does not remember the last one; the repository
does. What follows is the part that is expensive to rediscover.

## What this is

Roofing estimating software. Trace the roof on one screen, watch the money move on the
other. `README.md` explains it the way a roofer would want it explained; `STATUS.md` says
where the build actually is; `DECISIONS.md` says what was decided and why.

The ruling documents — the handoff and its addenda — are in `refs/`, which is local-only.
If `refs/` is empty on this clone, ask before assuming what the project wants.

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
  with no price says so. A total that leaves something out says what it left out.
- **A section is done when its check passes against the shipped runtime**, not against a
  browser with the shell stubbed. `pnpm build:app` then
  `node tools/probe/runtime-check.mjs`. Evidence goes in `evidence/` named with the section
  and the commit.
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
    node tools/probe/runtime-check.mjs          # the shipped runtime
    node tools/compare-fixtures.mjs             # needs fixtures/, which is local only

`packages/engine` is pure arithmetic with no window anywhere near it — if a number is wrong,
it is wrong there, and there is a test there that should have caught it. `src-tauri` holds
the one open job and hands the same copy to every window. `packages/app` are the editors.
