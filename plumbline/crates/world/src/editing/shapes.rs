//! A shape changed by hand (call 205, points 1 to 5): the model's changes, for the bench and the command line
//! to say alike. A starter shape is never changed (the thirteen are in the program and an old job means what it
//! meant); an item's shape is kept as a new one under a name, and that copy, the job's own, is changed a block at
//! a time: a block added, any of its six figures or its form or its material or its "There when" set, a block taken
//! away. Each change is one change, all of it or none (`all_or_nothing`, as `item_size`): the shape as it will be
//! is checked as a shape is on the way in (`Shape::check`: every figure a formula in width, depth and height, a
//! length, inside the box of the item's sizes at the three sizes), and one that fails is refused with its block,
//! the size and the reason, the job left as it was. Each says what follows: the items drawn as the shape and the
//! things set down from them, which are drawn from the shape the next time they are drawn. The saved job carries
//! the shapes (`Job::shapes`), so the undo history, which keeps the saved text, takes each change back and does it
//! again without a word more.
//!
//! Pattern taken from BIMCraft's `family` command (`crates/core/src/commands.rs:10588`): change a copy, check the
//! whole result, refuse with the stored family untouched, and say what changed (`family_changes`, 10852). BIMCraft
//! addresses a form by its id or GUID; a shape has no ids, so a block is addressed by its number from 1, as the
//! table's rows and `plumbline items <file> <item>` print it. There is no duplicate-a-family command there (a copy
//! is a family read and sent again under another name); "Keep as a new shape" is that, with the one place that says
//! which names are taken.

use crate::building::forms::{Block, Made, BOX};
use crate::building::shape::{self, Piece, Shape, CHECK_SIZES, FIGURES, MOST_SHAPES};
use crate::building::P3;
use crate::editing::all_or_nothing;
use crate::job::Job;
use crate::{Item, Node, Placed};

/// What one block's setting changes.
#[derive(Clone, Debug, PartialEq)]
pub enum BlockSet {
    /// One of the six figures, by its place in `FIGURES` (0 to 5), written as a formula in width, depth and height.
    Figure(usize, String),
    /// A box (false) or an upright round that fills its box (true).
    Round(bool),
    /// What it is made of.
    Made(Made),
    /// "There when": one comparison of two formulas; none is there always.
    When(Option<String>),
}

/// What a change to a shape did, for a window and a command line to say alike.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeChanged {
    /// The shape's name, as the job carries it.
    pub shape: String,
    /// What was done, in a few words ("block 4 added", "block 4, Up to, set to height - 6 in").
    pub did: String,
    /// How many blocks it has now.
    pub blocks: usize,
    /// The block the change was to (from 1), where there is one that is still there.
    pub block: Option<usize>,
    /// That block worked out for a size, said as `plumbline items` says a block, with the size it was worked at.
    pub block_said: Option<String>,
    /// The item (and its sizes) the shape was worked at: the first drawn as it that has a size, else the typical.
    pub worked_at: String,
    /// The shape it was kept from, where it was kept as a new one.
    pub kept_from: Option<String>,
    /// The name of each item drawn as it now.
    pub items: Vec<String>,
    /// The path of each thing set down from those items.
    pub set_down: Vec<String>,
}

impl ShapeChanged {
    /// What follows, in the words a window and the command line both use.
    pub fn follows(&self) -> String {
        let items = match self.items.len() {
            0 => return "No item is drawn as it, so nothing set down follows.".to_string(),
            1 => "1 item is drawn as it".to_string(),
            n => format!("{n} items are drawn as it"),
        };
        let them = if self.items.len() == 1 { "it" } else { "them" };
        let things = match self.set_down.len() {
            0 => format!("nothing is set down from {them}"),
            1 => format!("1 thing set down from {them} follows"),
            n => format!("{n} things set down from {them} follow"),
        };
        format!("{items}, and {things}.")
    }

    /// What happened.
    pub fn said(&self) -> String {
        let mut said = match &self.kept_from {
            Some(from) => format!("{}: kept as a new shape from {from}: its {}, word for word.", self.shape, so_many(self.blocks, "block")),
            None => format!("{}: {}; it has {} now.", self.shape, self.did, so_many(self.blocks, "block")),
        };
        if let (Some(n), Some(block)) = (self.block, &self.block_said) {
            said.push_str(&format!(" Block {n} reads, at {}: {block}.", self.worked_at));
        }
        said.push(' ');
        said.push_str(&self.follows());
        said
    }
}

fn so_many(n: usize, what: &str) -> String {
    if n == 1 {
        format!("1 {what}")
    } else {
        format!("{n} {what}s")
    }
}

/// A length in feet as inches, the way `plumbline items` says it: to the thousandth of an inch, no trailing zeros.
pub fn inches_said(ft: f64) -> String {
    let t = format!("{:.3}", ft * 12.0 + 0.0);
    let t = t.trim_end_matches('0').trim_end_matches('.');
    format!("{} in", if t == "-0" { "0" } else { t })
}

/// One block as it is said at a size: what it is made of, a round, and its six figures worked, across from the middle,
/// back from the back, up from the foot.
pub fn block_said(b: &Block, size: P3) -> String {
    let (back_from, back_to) = (size[1] / 2.0 - b.hi[1], size[1] / 2.0 - b.lo[1]);
    format!("{}{}: across {} to {}; back {} to {}; up {} to {}", b.made.word(), if b.round { ", round" } else { "" }, inches_said(b.lo[0]), inches_said(b.hi[0]), inches_said(back_from), inches_said(back_to), inches_said(b.lo[2]), inches_said(b.hi[2]))
}

/// The one place that says a starter shape is not changed.
fn starter_not_changed(name: &str) -> String {
    format!("{name:?} is a starter shape, and a starter shape is not changed (every job's items may be drawn as it): keep it as a new shape first, then change that")
}

/// The name a new shape may have, trimmed: not empty, not the plain box, not one the program's thirteen have, not one the
/// job's own shapes have, however it is capitalised. The one place that says so, for Keep and for a table brought in.
pub fn shape_name_free(job: &Job, name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("a shape has to be called something, with no space at either end of its name".into());
    }
    if name.eq_ignore_ascii_case(BOX) {
        return Err(format!("{BOX:?} is the plain box and is not a name for a shape"));
    }
    if shape::starter_named(name).is_some() {
        return Err(format!("{name:?} is a starter shape's name: a job's own shape may not take it; call the shape something else"));
    }
    if let Some(have) = job.shapes.iter().find(|s| s.name.eq_ignore_ascii_case(name)) {
        return Err(format!("the job has a shape called {:?} already; call this one something else", have.name));
    }
    Ok(name.to_string())
}

/// Where the job's own shape of that name is in `Job::shapes`. A starter shape is refused (not changed), a name nothing
/// has is refused with the shapes the job does have.
fn own_shape(job: &Job, name: &str) -> Result<usize, String> {
    let name = name.trim();
    if shape::starter_named(name).is_some() {
        return Err(starter_not_changed(name));
    }
    job.shapes.iter().position(|s| s.name.eq_ignore_ascii_case(name)).ok_or_else(|| {
        let have: Vec<&str> = job.shapes.iter().map(|s| s.name.as_str()).collect();
        if have.is_empty() {
            format!("no shape called {name:?}: the job has no shape of its own yet; keep an item's shape as a new shape first")
        } else {
            format!("no shape called {name:?}: the job's own shapes are {}", have.join(", "))
        }
    })
}

/// The items drawn as a shape (by name), and the things set down from them.
fn followers(job: &mut Job, shape: &str) -> (Vec<(String, String)>, Vec<String>) {
    let items: Vec<(String, String)> = {
        let mut q = job.world.query::<(&Node, &Item)>();
        q.iter(&job.world).filter(|(_, it)| it.form.as_deref().is_some_and(|f| f.trim().eq_ignore_ascii_case(shape))).map(|(n, _)| (n.path.clone(), n.name.clone())).collect()
    };
    let set_down: Vec<String> = {
        let mut q = job.world.query::<(&Node, &Placed)>();
        q.iter(&job.world).filter(|(_, p)| p.item.as_ref().is_some_and(|i| items.iter().any(|(path, _)| path == i))).map(|(n, _)| n.path.clone()).collect()
    };
    (items, set_down)
}

/// The sizes a shape is worked at to say a block: the first item drawn as it that has a size, else the typical size.
fn worked_for(job: &mut Job, items: &[(String, String)]) -> (P3, String) {
    for (path, name) in items {
        if let Some(size) = job.find(path).and_then(|e| job.world.get::<Item>(e)).and_then(|it| it.size_ft().ok()) {
            let (feet, _, _) = size;
            return (feet, format!("{name}'s sizes ({} x {} x {})", inches_said(feet[0]), inches_said(feet[1]), inches_said(feet[2])));
        }
    }
    let (_, typical) = CHECK_SIZES[1];
    (typical, format!("the typical size ({} x {} x {})", inches_said(typical[0]), inches_said(typical[1]), inches_said(typical[2])))
}

fn said_of(job: &mut Job, shape: &str, did: String, block: Option<usize>, kept_from: Option<String>) -> Result<ShapeChanged, String> {
    let at = job.shapes.iter().position(|s| s.name.eq_ignore_ascii_case(shape)).ok_or("the shape is not in the job")?;
    let (items, set_down) = followers(job, shape);
    let (size, worked_at) = worked_for(job, &items);
    let s = job.shapes[at].clone();
    let block_said = match block {
        Some(n) => Some(match s.worked_block(n - 1, size)? {
            Some(b) => block_said(&b, size),
            None => "not there at these sizes (its \"There when\" does not hold)".to_string(),
        }),
        None => None,
    };
    Ok(ShapeChanged { shape: s.name.clone(), did, blocks: s.blocks.len(), block, block_said, worked_at, kept_from, items: items.into_iter().map(|(_, n)| n).collect(), set_down })
}

/// Change a copy of the job's own shape, check it as a shape is on the way in, and keep it: all of it or none.
fn change(job: &mut Job, shape: &str, f: impl FnOnce(&mut Shape) -> Result<(String, Option<usize>), String>) -> Result<ShapeChanged, String> {
    all_or_nothing(job, |job| {
        let at = own_shape(job, shape)?;
        let mut s = job.shapes[at].clone();
        let (did, block) = f(&mut s)?;
        s.check()?;
        job.shapes[at] = s;
        let name = job.shapes[at].name.clone();
        said_of(job, &name, did, block, None)
    })
}

/// The block to change, by its number from 1, or why there is none.
fn block_index(s: &Shape, n: usize) -> Result<usize, String> {
    if n == 0 || n > s.blocks.len() {
        return Err(format!("block {n}: shape {:?} has {}, numbered from 1", s.name, so_many(s.blocks.len(), "block")));
    }
    Ok(n - 1)
}

/// A block added: a box the whole size of the item, of what the shape's first block is made of (or `made`), until
/// its figures are typed. It is the last block, so a block that is there already keeps its number.
pub fn shape_block_add(job: &mut Job, shape: &str, made: Option<Made>) -> Result<ShapeChanged, String> {
    change(job, shape, |s| {
        if s.blocks.len() >= shape::MOST_BLOCKS {
            return Err(format!("shape {:?} has {} blocks already; a shape has at most {}", s.name, s.blocks.len(), shape::MOST_BLOCKS));
        }
        let made = made.or_else(|| s.blocks.first().map(|b| b.made)).unwrap_or(Made::Timber);
        s.blocks.push(Piece::whole(made));
        let n = s.blocks.len();
        Ok((format!("block {n} added, the whole size of the item, made of {} until its figures are typed", made.word()), Some(n)))
    })
}

/// A block's figure, form, material or condition set.
pub fn shape_block_set(job: &mut Job, shape: &str, block: usize, what: BlockSet) -> Result<ShapeChanged, String> {
    change(job, shape, |s| {
        let i = block_index(s, block)?;
        let b = &mut s.blocks[i];
        let did = match what {
            BlockSet::Figure(n, text) => {
                let text = text.trim().to_string();
                if n >= 6 {
                    return Err(format!("a block has six figures, numbered 0 to 5: there is no figure {n}"));
                }
                let said = format!("block {block}, {}, set to {}", FIGURES[n], if text.is_empty() { "nothing".to_string() } else { text.clone() });
                match n {
                    0 | 1 => b.across[n] = text,
                    2 | 3 => b.back[n - 2] = text,
                    _ => b.up[n - 4] = text,
                }
                said
            }
            BlockSet::Round(round) => {
                b.round = round;
                format!("block {block} is {} now", if round { "an upright round" } else { "a box" })
            }
            BlockSet::Made(m) => {
                b.made = m;
                format!("block {block} is made of {} now", m.word())
            }
            BlockSet::When(when) => {
                let when = when.map(|w| w.trim().to_string()).filter(|w| !w.is_empty());
                let said = match &when {
                    Some(w) => format!("block {block} is there when {w}"),
                    None => format!("block {block} is there at every size"),
                };
                b.when = when;
                said
            }
        };
        Ok((did, Some(block)))
    })
}

/// A block taken away, by its number: the blocks after it are numbered one less. A shape keeps at least one.
pub fn shape_block_take(job: &mut Job, shape: &str, block: usize) -> Result<ShapeChanged, String> {
    change(job, shape, |s| {
        let i = block_index(s, block)?;
        if s.blocks.len() == 1 {
            return Err(format!("shape {:?} would have no blocks: a shape keeps at least one (take the item's shape off instead)", s.name));
        }
        s.blocks.remove(i);
        Ok((format!("block {block} taken away; the blocks after it are numbered one less"), None))
    })
}

/// "Keep as a new shape": the shape an item is drawn as (its own, or one of the thirteen, or what its name says) is copied
/// into the job under a new name, word for word, and the item is drawn as the copy. Items drawn as the old shape stay
/// as they were. The name may not be one the job or the program has (`shape_name_free`).
pub fn keep_shape_as(job: &mut Job, item: &str, name: &str) -> Result<ShapeChanged, String> {
    all_or_nothing(job, |job| {
        let e = job.find(item).ok_or_else(|| format!("{item} is not in the job"))?;
        let mut it = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{item} is not an item"))?;
        let item_name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| item.to_string());
        let name = shape_name_free(job, name)?;
        let (from, blocks) = match it.form.as_deref().map(str::trim) {
            Some(said) if said.eq_ignore_ascii_case(BOX) => return Err(format!("{item_name} is drawn as a box: a box is its sizes and has no shape to keep; draw it as a shape first")),
            Some(said) => match shape::find(&job.shapes, said) {
                Some(s) => (s.name.clone(), s.blocks.clone()),
                None => return Err(format!("{item_name} is drawn as {said:?}, which is not a shape the job has")),
            },
            None => match crate::building::forms::form_of(&item_name).and_then(|f| shape::starter_named(f.name())) {
                Some(s) => (s.name.clone(), s.blocks.clone()),
                None => return Err(format!("{item_name} is drawn as a box, its name saying nothing else: a box has no shape to keep")),
            },
        };
        if job.shapes.len() >= MOST_SHAPES {
            return Err(format!("a job carries at most {MOST_SHAPES} shapes"));
        }
        let copy = Shape { name: name.clone(), blocks };
        copy.check()?;
        job.shapes.push(copy);
        it.form = Some(name.to_lowercase());
        job.put(item, it)?;
        said_of(job, &name, format!("kept from {from}"), None, Some(from))
    })
}
