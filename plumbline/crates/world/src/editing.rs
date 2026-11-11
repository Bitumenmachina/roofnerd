//! Drawing and editing the building: the verbs a window or a command line uses to put a
//! wall, an opening, a run or a fixture into the job, to move and remove them, and to
//! take any of it back.
//!
//! This file is this program's own. What is ported from BIMCraft is in `editing/`:
//! each file there says so in its header, and `PORTMAP.md` gives the line each item came
//! from. `tools/portmap.sh` checks every public item here and there.

pub mod history;

pub use history::{History, UNDO_LIMIT};
