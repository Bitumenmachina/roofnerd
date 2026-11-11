//! The undo history: each step that changed the job, and the job as it was before it.
//!
//! PORTED CODE. This file is ported from BIMCraft, `crates/core/src/engine.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//! `PORTMAP.md` gives the line each item came from.
//!
//! What is taken: an undo stack and a redo stack of steps, each a label and the whole
//! model as it was before the step (`engine.rs:160`, `161`); a step is recorded only when
//! the model changed, the stack keeps the last 200 and drops the oldest, and a new step
//! empties the redo stack (`engine.rs:497` to `503`); stepping back pops one stack, pushes
//! what stands now onto the other, and names the step (`engine.rs:728` to `749`).
//!
//! What was changed in the port, and nothing else:
//! * The model kept with a step is the saved job's text, not a shared copy of the model
//!   in memory. A job here is an entity-component world, which is not copied; and the
//!   saved text is the job (it loads back to the same bytes), so a step undone is the
//!   file as it was, byte for byte.
//! * A step does not count what it made and removed by category (`Step::made`,
//!   `Step::removed`): the label is what is shown.
//! * Not taken: phases and design options, read-only sessions, dry runs, transactions
//!   and merged steps, and the record of whether a model had passed its check (a job is
//!   checked whenever its text is loaded).

use crate::file;
use crate::job::Job;

/// How many steps are kept (BIMCraft `engine.rs:103`).
pub const UNDO_LIMIT: usize = 200;

/// The steps that can be undone and redone, the latest last.
#[derive(Default)]
pub struct History {
    undo: Vec<(String, String)>,
    redo: Vec<(String, String)>,
}

impl History {
    /// The job's text now, to hand to `record` once a change has been made.
    pub fn before(job: &mut Job) -> String {
        file::save(job)
    }

    /// Record a step: `label` is what was done, `before` the job's text before it. A step
    /// that changed nothing is not recorded. Returns whether one was.
    pub fn record(&mut self, label: &str, before: String, job: &mut Job) -> bool {
        if file::save(job) == before {
            return false;
        }
        self.undo.push((label.to_string(), before));
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        true
    }

    /// Take the last step back. The job becomes what it was before it; the label of the
    /// step is returned.
    pub fn undo(&mut self, job: &mut Job) -> Result<String, String> {
        self.step_back(false, job)
    }

    /// Do again the step last undone.
    pub fn redo(&mut self, job: &mut Job) -> Result<String, String> {
        self.step_back(true, job)
    }

    fn step_back(&mut self, redo: bool, job: &mut Job) -> Result<String, String> {
        let (from, to) = if redo { (&mut self.redo, &mut self.undo) } else { (&mut self.undo, &mut self.redo) };
        let Some((label, text)) = from.pop() else {
            return Err(format!("nothing to {}", if redo { "redo" } else { "undo" }));
        };
        // The text was this program's own save of a job that loaded: it loads again.
        let (back, _) = match file::load(&text) {
            Ok(loaded) => loaded,
            Err(why) => {
                from.push((label, text));
                return Err(format!("the step cannot be taken back: the job as it was does not load ({why})"));
            }
        };
        let now = file::save(job);
        *job = back;
        to.push((label.clone(), now));
        Ok(label)
    }

    /// The labels of the steps that can be undone, oldest first; or with `redo`, of those
    /// that can be redone.
    pub fn labels(&self, redo: bool) -> Vec<&str> {
        let from = if redo { &self.redo } else { &self.undo };
        from.iter().map(|(label, _)| label.as_str()).collect()
    }
}
