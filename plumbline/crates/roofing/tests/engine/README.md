# The earlier engine's statement of the roofing conventions

`roof.ts` and `model.ts` are two files of the earlier program (roofnerd), byte for byte as
they stood at the root of this branch (`packages/engine/src/`) until 2026-11-26, when that
program left the branch (DECISIONS.md call 237; it is on the branch `main`).

This crate's figures were ported against them, and three tests read them at test time
rather than restating their numbers: `nrca_figures.rs` (the taper and coping conventions),
`the_bill.rs` (the sump rule) and `the_drawings.rs` (the order of a roof's layers). The
tests are as they were; only where they find the two files changed. Nothing here is built
or run, and nothing else reads it.
