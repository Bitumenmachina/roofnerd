# W-184T notes (written as I go)

## Before building: what the model holds (read from solve.rs, roof.rs, derive.rs, trim.rs)
- A roof edge's HEIGHT: a roof (footprint) holds ONE `eave_height`, in feet above the job's datum (the same datum as wall
  base/top; it is NOT relative to a level). A plane's height at a plan point = eave_height + (distance from the plane's
  eave line) x pitch/12 (`SolidPlane::height_at`). So an eave's edge is at eave_height everywhere (9 ft in the demo);
  a rake's two ends are at eave_height at the eave end and eave_height + run x pitch/12 at the ridge end (9 and 13.5 ft).
  The model has no fascia depth, no rafter, no heel: the edge height is the height of the roof SURFACE at the edge line.
  Consequence: in the demo the roof's edge (1 ft outside the wall lines) is at 9 ft, the same as the wall top at the line;
  the roof surface at the wall's outside face is 9.375 ft, i.e. above the wall top. That is the model's, not bent.
- Hand figures of the lead contradicted by the model's own rule / arithmetic:
  1. "23 pieces in all": 2+4+2+4+8 = 20, + window 4 + door 3 = 27. Hand sum was wrong.
  (more below as found)

## Status at 21:40 UTC (cargo not yet run; waiting for 22:30)
- Design: new child module `crates/world/src/trim/lay.rs` (pub mod lay; re-exports `trim::{lay, pieces, Laid, Piece, PieceKind}`;
  my `Kind` is `trim::lay::Kind` / `trim::PieceKind` because `trim` already glob-imports `components::Kind`).
  `lay(job) -> Laid {pieces, left_out, priced}`; `pieces(job)` = `lay(job).pieces`.
- Tests written: crates/world/tests/call184t.rs (13 tests). Stub lay.rs in tree for the red run; real code waits in
  scratchpad/lay_real.rs and is copied over after the red run.
- CLI: `trim <job> --pieces` (crates/cli/src/main.rs `trim_pieces_said`), usage line, README line. demos/trim.sh step 6 added.
RED RUN ok: 0 passed; 12 failed (stub), 22:31 UTC

(The lead: these are the worker's running notes, which it did not finish. Its full report came back as a message; what it built, the table of pieces, the hand figures it set right and what it called doubtful are in STATUS.md, "Where each piece of outside trim lies: built".)
