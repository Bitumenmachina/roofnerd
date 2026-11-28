//! Cut from `editing.rs`. PORTED CODE in part, as `editing.rs` says: the rules of the verbs are BIMCraft's,
//! `crates/core/src/commands.rs` at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (MIT); `PORTMAP.md` has each item.

use super::*;

/// An item: a thing that can be placed any number of times, as its maker's sheet
/// describes it. It is kept with the job's types, under the name it goes by.
pub fn item(job: &mut Job, name: &str, item: Item) -> Result<String, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        if name.trim().is_empty() {
            return Err("an item needs a name: what the thing is called".into());
        }
        let path = job.add(&root, Kind::Type, name.trim())?;
        building::check_item(&path, Kind::Type, &item)?;
        if let Some(said) = &item.form {
            building::shape::check_drawn_as(&job.shapes, said).map_err(|why| format!("{path} item: drawn as: {why}"))?;
        }
        shapes::item_fits(&job.shapes, name.trim(), &item)?;
        job.put(&path, item)?;
        Ok(path)
    })
}

/// What an item is drawn as, changed: a form by its name, "box" for the box of its
/// sizes, or nothing, which leaves it to what its name says. Every one of it already
/// placed is drawn the new way: what it is drawn as is the item's, not the thing's.
pub fn item_drawn_as(job: &mut Job, item: &str, said: Option<&str>) -> Result<(), String> {
    all_or_nothing(job, |job| {
        let e = job.find(item).ok_or_else(|| format!("{item} is not in the job"))?;
        let mut it = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{item} is not an item"))?;
        it.form = match said.map(str::trim).filter(|s| !s.is_empty()) {
            Some(said) => {
                building::shape::check_drawn_as(&job.shapes, said)?;
                Some(said.to_lowercase())
            }
            None => None,
        };
        building::check_item(item, Kind::Type, &it)?;
        let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| item.to_string());
        shapes::item_fits(&job.shapes, &name, &it)?;
        job.put(item, it)
    })
}

/// Which trade's item it is, changed: one of the sixteen divisions, or none. The lines on what is set down from it that took
/// its trade follow it (call 206 point 6).
pub fn item_trade(job: &mut Job, item: &str, division: Option<crate::components::Division>) -> Result<(), String> {
    item_trade_followed(job, item, division).map(|_| ())
}

/// What an item's trade changed did to what is set down from it: how many lines moved, on how many things.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TradeFollowed {
    pub lines: usize,
    pub things: usize,
    /// How many are set down from the item in all.
    pub set_down: usize,
}

impl TradeFollowed {
    /// "3 lines on 2 set down from it moved to Division 12 Furnishings"; empty where nothing moved.
    pub fn said(&self, now: Option<crate::components::Division>) -> String {
        if self.lines == 0 {
            return String::new();
        }
        format!(
            "{} on {} set down from it {} to {}",
            if self.lines == 1 { "1 line".to_string() } else { format!("{} lines", self.lines) },
            self.things,
            "moved",
            now.map(|d| format!("Division {} {}", d.number(), d.title())).unwrap_or_else(|| "the division their assembly gives them".to_string())
        )
    }
}

/// As `item_trade`, saying how many lines followed. A line took the item's trade where the program put it there (call 201
/// point 3): a line in the item's old trade is taken to be one, and goes to the new; where the trade is taken away, to the
/// division its assembly gives it. A line in another division was put there by a person, and stays.
pub fn item_trade_followed(job: &mut Job, item: &str, division: Option<crate::components::Division>) -> Result<TradeFollowed, String> {
    all_or_nothing(job, |job| {
        let e = job.find(item).ok_or_else(|| format!("{item} is not in the job"))?;
        let mut it = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{item} is not an item"))?;
        let old = it.division;
        it.division = division;
        job.put(item, it)?;
        trade_follow(job, item, old, division)
    })
}

/// What the lines on what is set down from an item do when the item's trade has gone from `old` to `new` (the item already
/// says `new`): the lines in `old` follow.
pub fn trade_follow(job: &mut Job, item: &str, old: Option<crate::components::Division>, division: Option<crate::components::Division>) -> Result<TradeFollowed, String> {
    all_or_nothing(job, |job| {
        let mut out = TradeFollowed::default();
        let things: Vec<String> = {
            let mut q = job.world.query::<(&Node, &Placed)>();
            let mut v: Vec<String> = q.iter(&job.world).filter(|(_, p)| p.item.as_deref() == Some(item)).map(|(n, _)| n.path.clone()).collect();
            v.sort();
            v
        };
        out.set_down = things.len();
        let Some(old) = old.filter(|o| Some(*o) != division) else { return Ok(out) };
        for thing in things {
            let te = job.find(&thing).ok_or_else(|| format!("no element {thing}"))?;
            let Some(mut cost) = job.world.get::<Cost>(te).cloned() else { continue };
            // The assembly's own division for a line, where the trade is taken away.
            let recipe = job.world.get::<Condition>(te).and_then(|c| c.0.value.clone()).and_then(|have| {
                let mut q = job.world.query::<(&Node, &Recipe)>();
                q.iter(&job.world).find(|(n, _)| n.name == have || n.path == have).map(|(_, r)| r.clone())
            });
            let mut moved = 0;
            for l in cost.lines.iter_mut().filter(|l| l.division == Some(old)) {
                l.division = match division {
                    Some(d) => Some(d),
                    None => recipe.as_ref().and_then(|r| r.cost.lines.iter().find(|o| o.description == l.description || (l.of_material && o.of_material)).and_then(|o| o.division)),
                };
                moved += 1;
            }
            if moved > 0 {
                job.world.entity_mut(te).insert(cost);
                out.lines += moved;
                out.things += 1;
            }
        }
        job.derive();
        Ok(out)
    })
}

/// One of an item, set down on a level: its middle at `at` on the plan, its underside
/// `offset` above the floor. It is as big as its item says and as sure as that; where
/// the item's size is not all known it is placed with no size, has no shape, and says
/// which of its sizes is missing. Where it stands is as sure as `how`.
///
/// Where the item is "priced as" an assembly the job has, the thing takes that assembly, its lines saying
/// they came by the item; where it is priced as nothing, the thing has no line (call 196).
pub fn place_item(job: &mut Job, storey: &str, item: &str, at: P2, offset: &str, how: Entered) -> Result<String, String> {
    place_item_said(job, storey, item, at, offset, how).map(|(path, _)| path)
}

/// As `place_item`, and saying what is not as the item says: its assembly is not in the job, so the thing is
/// set down and not priced (call 196).
pub fn place_item_said(job: &mut Job, storey: &str, item: &str, at: P2, offset: &str, how: Entered) -> Result<(String, Vec<String>), String> {
    all_or_nothing(job, |job| {
        let offset = how.length(offset)?;
        let item = item_by_name(job, item)?;
        let (path, connections) = place_item_in(job, storey, &item, at, offset, how)?;
        let mut notes: Vec<String> = price_as_its_item(job, &path, &item)?.into_iter().collect();
        // Where the item says where it connects, the thing has its connections from now (call 211); where its sizes are not all
        // known it has none, and this says so.
        notes.extend(connections);
        Ok((path, notes))
    })
}

/// What an item is called in the price list (call 196): its own name. An item that is priced as something is its own entry
/// in the price list, holding what a material holds for it (a `Supply`: the estimator's price and each supplier's), so a
/// price is worked one way whether what is bought is a stud or a heater. No material is made for it. This is the one place
/// that says the entry's name.
pub fn item_material_name(item: &str) -> String {
    item.trim().to_string()
}

/// The assembly the job has by this name (its path or its name, then any capitals and spacing), as the job names
/// it, if it goes on `kind`.
pub(crate) fn assembly_named(job: &mut Job, typed: &str, kind: Kind) -> Result<(String, Recipe), String> {
    let mut q = job.world.query::<(&Node, &Recipe)>();
    let all: Vec<(String, String, Recipe)> = q.iter(&job.world).map(|(n, r)| (n.path.clone(), n.name.clone(), r.clone())).collect();
    let exact: Vec<&(String, String, Recipe)> = all.iter().filter(|a| a.0 == typed || a.1 == typed).collect();
    let found: Vec<&(String, String, Recipe)> = if exact.is_empty() { all.iter().filter(|a| check::same_name(&a.1, typed)).collect() } else { exact };
    match found.as_slice() {
        [] => Err(format!("no assembly {typed:?} in this job")),
        [one] => {
            if !one.2.applies_to.contains(&kind) {
                return Err(format!("{:?} goes on {}, and an item of this kind is {}", one.1, one.2.applies_to.iter().map(|k| format!("a {}", k.said())).collect::<Vec<_>>().join(" or "), kind.said()));
            }
            Ok((one.1.clone(), one.2.clone()))
        }
        many => Err(format!("{typed:?} names {} assemblies ({}); use the path", many.len(), many.iter().map(|f| f.0.clone()).collect::<Vec<_>>().join(", "))),
    }
}

/// The one line of an assembly that is made of the element's material, by its word on the assembly: the line a thing
/// set down from an item names for the item. None where the assembly has none or more than one.
pub(super) fn item_line_of(recipe: &Recipe) -> Option<String> {
    let mut made: Vec<&crate::components::CostLine> = recipe.cost.lines.iter().filter(|l| l.of_material).collect();
    (made.len() == 1).then(|| made.remove(0).description.clone())
}

/// A thing set down from an item, its assembly's item line named for the item and its lines in the item's trade
/// where they say none. `from` is the word the assembly gives the item line.
pub(super) fn name_item_lines(job: &mut Job, thing: &str, recipe: &Recipe, item_name: &str, division: Option<Division>) -> Result<(), String> {
    let e = job.find(thing).ok_or_else(|| format!("no element {thing}"))?;
    let kind = *job.world.get::<Kind>(e).ok_or_else(|| format!("{thing} has no kind"))?;
    let Some(mut cost) = job.world.get::<Cost>(e).cloned() else { return Ok(()) };
    let theirs: Vec<String> = recipe.cost.lines.iter().map(|l| l.description.clone()).collect();
    if let Some(word) = item_line_of(recipe) {
        let taken = cost.lines.iter().any(|l| l.description != word && check::same_name(&l.description, item_name));
        if !taken {
            if let Some(l) = cost.lines.iter_mut().find(|l| l.of_material && l.description == word) {
                l.description = item_name.to_string();
            }
        }
    }
    if let Some(d) = division {
        // A line the assembly brought onto a thing set down from an item is in the item's trade: a heater's labor is
        // Mechanical and a vanity's is Furnishings, whatever trade the assembly's own line says (call 196).
        for l in cost.lines.iter_mut() {
            if theirs.contains(&l.description) || (l.of_material && check::same_name(&l.description, item_name)) {
                l.division = Some(d);
            }
        }
    }
    check::cost(thing, kind, &cost)?;
    job.world.entity_mut(e).insert(cost);
    Ok(())
}

/// A thing just set down, given the assembly its item is priced as. Says (and does nothing more) where the item's
/// assembly is not in the job.
pub(super) fn price_as_its_item(job: &mut Job, thing: &str, item_path: &str) -> Result<Option<String>, String> {
    let e = job.find(item_path).ok_or_else(|| format!("no element {item_path}"))?;
    let (Some(node), Some(it)) = (job.world.get::<Node>(e), job.world.get::<Item>(e)) else { return Ok(None) };
    let (name, it) = (node.name.clone(), it.clone());
    let Some(typed) = it.priced_as.clone() else { return Ok(None) };
    let (assembly, recipe) = match assembly_named(job, &typed, it.category) {
        Ok(found) => found,
        Err(_) => return Ok(Some(format!("{name} is priced as {typed:?}, and this job has no such assembly that goes on it: the {} is set down and not priced", it.category.said()))),
    };
    job.apply_as(thing, &assembly, Status::Known, &format!("the item {name}: priced as {assembly}"))?;
    name_item_lines(job, thing, &recipe, &name, it.division)?;
    Ok(None)
}

/// What an item's "priced as" came to for what is set down from it (call 196).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemFollowed {
    /// The item's name.
    pub item: String,
    /// What it was priced as, and is.
    pub was: Option<String>,
    pub now: Option<String>,
    /// Each thing set down from it that now has what the item says.
    pub followed: Vec<String>,
    /// How many are set down from it in all.
    pub set_down: usize,
    /// Each line that came off, on which thing, with what had been entered on it and what it came to (call 206 point 1).
    pub took: Vec<(String, crate::entered::Took)>,
    /// Things left as they were, each with its reason.
    pub left: Vec<(String, String)>,
    /// Lines a person entered on a thing itself, which stay: the thing and each line.
    pub own_lines: Vec<(String, Vec<String>)>,
}

impl ItemFollowed {
    /// The sentences that say it, in a builder's words.
    pub fn said(&self) -> Vec<String> {
        let mut out = Vec::new();
        // The same again: no change, and how many are set down from it (call 206 point 4).
        if self.was == self.now {
            let is = match &self.now {
                Some(a) => format!("{} is already priced as {a:?}", self.item),
                None => format!("{} is already priced as nothing", self.item),
            };
            return vec![match self.set_down {
                0 => format!("{is}; nothing changes, and none is set down yet"),
                1 => format!("{is}; nothing changes, and the 1 set down from it keeps it"),
                n => format!("{is}; nothing changes, and the {n} set down from it keep it"),
            }];
        }
        let by = match &self.now {
            Some(a) => format!("{} is now priced as {a:?}", self.item),
            None => format!("{} is now priced as nothing", self.item),
        };
        let n = self.followed.len();
        out.push(match (self.set_down, n) {
            (0, _) => format!("{by}; none is set down yet, and each set down after takes it"),
            (_, 0) => format!("{by}; none of the {} set down from it changed", self.set_down),
            (_, 1) => format!("{by}; 1 set down from it follows ({})", self.followed[0]),
            _ => format!("{by}; {n} set down from it follow"),
        });
        for (thing, took) in &self.took {
            out.push(format!("  {:?} came off {thing}{}", took.line, took.tail()));
        }
        for (thing, why) in &self.left {
            out.push(format!("  {thing} is left as it was: {why}"));
        }
        for (thing, lines) in &self.own_lines {
            out.push(format!("  {thing} keeps {} entered on it: {}", if lines.len() == 1 { "a line" } else { "lines" }, lines.iter().map(|l| format!("{l:?}")).collect::<Vec<_>>().join(", ")));
        }
        out
    }
}

/// Say what an item is priced as, or that it is priced as nothing (`None`), and let each thing already set down from
/// it follow, as it follows its sizes: it changes assembly the way one assembly is changed for another, so a line the
/// two share keeps what was entered on it. A line that came off says what had been entered on it. A thing a person
/// gave another assembly, a thing set in a run (it carries its own line named for the item) and a line entered on a
/// thing itself are left, and named. The assembly has to be one the job has that goes on the item's kind.
pub fn item_priced_as(job: &mut Job, item: &str, assembly: Option<&str>) -> Result<ItemFollowed, String> {
    all_or_nothing(job, |job| {
        let path = item_by_name(job, item)?;
        let e = job.find(&path).ok_or_else(|| format!("{item} is not in the job"))?;
        let (name, mut it) = match (job.world.get::<Node>(e), job.world.get::<Item>(e)) {
            (Some(n), Some(it)) => (n.name.clone(), it.clone()),
            _ => return Err(format!("{item} is not an item")),
        };
        let was = it.priced_as.clone();
        let new = match assembly {
            Some(typed) => Some(assembly_named(job, typed, it.category)?),
            None => None,
        };
        let now_name = new.as_ref().map(|n| n.0.clone());
        let mut out = ItemFollowed { item: name.clone(), was: was.clone(), now: now_name.clone(), ..ItemFollowed::default() };
        if was == now_name {
            // Nothing to change; how many are set down from it is still said (call 206 point 4).
            out.set_down = job.world.query::<&Placed>().iter(&job.world).filter(|p| p.item.as_deref() == Some(path.as_str())).count();
            return Ok(out);
        }
        // What each line came to before, for the lines that come off to say it (call 206 point 1).
        let before_estimate = crate::view::estimate(job);
        it.priced_as = now_name.clone();
        building::check_item(&path, Kind::Type, &it)?;
        job.put(&path, it.clone())?;
        let old = match &was {
            Some(w) => assembly_named(job, w, it.category).ok(),
            None => None,
        };
        let mut things: Vec<(u64, String)> = {
            let mut q = job.world.query::<(&Node, &Placed)>();
            q.iter(&job.world).filter(|(_, p)| p.item.as_deref() == Some(path.as_str())).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone())).collect()
        };
        things.sort();
        out.set_down = things.len();
        for (_, thing) in things {
            let te = job.find(&thing).ok_or_else(|| format!("no element {thing}"))?;
            let cond = job.world.get::<Condition>(te).and_then(|c| c.0.value.clone());
            let had: Vec<crate::components::CostLine> = job.world.get::<Cost>(te).map(|c| c.lines.clone()).unwrap_or_default();
            // Set in a run: it carries its own line named for the item, and is priced by it.
            if cond.is_none() && had.iter().any(|l| !l.of_material && check::same_name(&l.description, &name)) {
                out.left.push((thing.clone(), "it was set in a run and carries its own line named for the item".to_string()));
                continue;
            }
            // Another assembly, chosen on the thing itself.
            let own = cond.as_ref().filter(|c| !was.as_deref().is_some_and(|w| check::same_name(c, w)) && !now_name.as_deref().is_some_and(|n| check::same_name(c, n)));
            if let Some(c) = own {
                out.left.push((thing.clone(), format!("it has an assembly of its own, {c:?}")));
                continue;
            }
            // Its lines as the old assembly wrote them, for the change to compare by.
            let before_cost = job.world.get::<Cost>(te).cloned();
            let before_cond = job.world.get::<Condition>(te).cloned();
            let restore = |job: &mut Job| {
                let mut ent = job.world.entity_mut(te);
                match &before_cost {
                    Some(c) => {
                        ent.insert(c.clone());
                    }
                    None => {
                        ent.remove::<Cost>();
                    }
                }
                match &before_cond {
                    Some(c) => {
                        ent.insert(c.clone());
                    }
                    None => {
                        ent.remove::<Condition>();
                    }
                }
            };
            let mut step = || -> Result<Vec<crate::entered::Took>, String> {
                // Back to the word the old assembly gave the item line, so that the lines it shares are found.
                if let Some(word) = old.as_ref().and_then(|(_, r)| item_line_of(r)) {
                    if let Some(mut cost) = job.world.get::<Cost>(te).cloned() {
                        if !cost.lines.iter().any(|l| l.description == word) {
                            if let Some(l) = cost.lines.iter_mut().find(|l| l.of_material && check::same_name(&l.description, &name)) {
                                l.description = word;
                                job.world.entity_mut(te).insert(cost);
                            }
                        }
                    }
                }
                let source = format!("the item {name}: priced as {}", now_name.as_deref().unwrap_or("nothing"));
                let taken: Vec<crate::entered::Took> = match &new {
                    // It has the assembly already, by hand: only the item's line is named.
                    Some((assembly, _)) if cond.as_deref() == Some(assembly.as_str()) => Vec::new(),
                    Some((assembly, _)) => job.change_assembly_in(&thing, assembly, Status::Known, &source, it.division)?.took,
                    None => {
                        // Priced as nothing: what the old assembly brought comes off, and nothing else.
                        let Some((_, r)) = &old else { return Ok(Vec::new()) };
                        let theirs: Vec<String> = r.cost.lines.iter().map(|l| l.description.clone()).collect();
                        let mut cost = job.world.get::<Cost>(te).cloned().unwrap_or(Cost { code: r.cost.code.clone(), lines: Vec::new() });
                        let goes: Vec<crate::components::CostLine> = cost.lines.iter().filter(|l| theirs.contains(&l.description) || (l.of_material && check::same_name(&l.description, &name))).cloned().collect();
                        let mut gone: Vec<crate::entered::Took> = Vec::new();
                        for l in &goes {
                            // Against the line as the old assembly wrote it; the item line is the assembly's one made of the material.
                            let theirs_line = if l.of_material && check::same_name(&l.description, &name) { item_line_of(r).and_then(|w| r.cost.lines.iter().find(|o| o.description == w)) } else { r.cost.lines.iter().find(|o| o.description == l.description) };
                            let fig = job.line_figure(&thing, l);
                            gone.push(crate::entered::Took { line: l.description.clone(), entered: crate::entered::entered_on_item(l, theirs_line, fig.as_deref(), it.division), amount: None });
                        }
                        cost.lines.retain(|l| !goes.iter().any(|g| g.description == l.description));
                        let mut ent = job.world.entity_mut(te);
                        if cost.lines.is_empty() {
                            ent.remove::<Cost>();
                        } else {
                            ent.insert(cost);
                        }
                        ent.remove::<Condition>();
                        gone
                    }
                };
                if let Some((_, recipe)) = &new {
                    name_item_lines(job, &thing, recipe, &name, it.division)?;
                }
                Ok(taken)
            };
            match step() {
                Ok(taken) => {
                    let word = old.as_ref().and_then(|(_, r)| item_line_of(r));
                    for mut took in taken {
                        // What was on it, by the line as it was before; the item line is called by the item's name.
                        let is_item_line = word.as_deref() == Some(took.line.as_str()) || (name == took.line);
                        let before_name = if is_item_line { name.clone() } else { took.line.clone() };
                        took.amount = before_estimate.lines.iter().find(|l| l.path == thing && l.description == before_name && !l.off).and_then(|l| l.amount.value);
                        took.line = before_name;
                        out.took.push((thing.clone(), took));
                    }
                    out.followed.push(thing.clone());
                }
                Err(why) => {
                    restore(job);
                    out.left.push((thing.clone(), why));
                    continue;
                }
            }
            // Lines entered on the thing itself stay: named.
            let theirs: Vec<String> = [old.as_ref(), new.as_ref()].into_iter().flatten().flat_map(|(_, r)| r.cost.lines.iter().map(|l| l.description.clone())).collect();
            let own_lines: Vec<String> = job.world.get::<Cost>(te).map(|c| c.lines.iter().filter(|l| !theirs.contains(&l.description) && !(l.of_material && check::same_name(&l.description, &name))).map(|l| l.description.clone()).collect()).unwrap_or_default();
            if !own_lines.is_empty() {
                out.own_lines.push((thing.clone(), own_lines));
            }
        }
        Ok(out)
    })
}

/// An item by its number, or by its name in any capitals (call 167). Two items of one name are
/// refused with both; what names no item is left as typed, for the caller to refuse.
pub(super) fn item_by_name(job: &mut Job, typed: &str) -> Result<String, String> {
    let mut q = job.world.query::<(&Node, &Item)>();
    let mut named: Vec<(String, String)> = q.iter(&job.world).filter(|(n, _)| n.path == typed || n.name.eq_ignore_ascii_case(typed.trim())).map(|(n, _)| (n.path.clone(), n.name.clone())).collect();
    named.sort();
    if let Some(one) = named.iter().find(|(path, _)| path == typed) {
        return Ok(one.0.clone());
    }
    match named.as_slice() {
        [] => Ok(typed.to_string()),
        [(path, _)] => Ok(path.clone()),
        many => Err(format!("{} items are called {:?}: {}; give the one to set down by its number", many.len(), typed.trim(), many.iter().map(|(path, name)| format!("{name} ({path})")).collect::<Vec<_>>().join(", "))),
    }
}

/// One of an item set down, within a change that is all or nothing already.
pub(super) fn place_item_in(job: &mut Job, storey: &str, item: &str, at: P2, offset: Val<Len>, how: Entered) -> Result<(String, Option<String>), String> {
    storey_of(job, storey)?;
    let e = job.find(item).ok_or_else(|| format!("{item} is not in the job"))?;
    let (name, it) = match (job.world.get::<Node>(e), job.world.get::<Item>(e)) {
        (Some(n), Some(it)) => (n.name.clone(), it.clone()),
        _ => return Err(format!("{item} is not an item: an item is made from a maker's sheet, or entered by its sizes")),
    };
    let size = match it.size_ft() {
        Ok((feet, status, source)) => Val { value: Some(feet), unit: "ft".into(), status, source: format!("{name}: {source}") },
        Err(why) => Val::unknown("ft", &format!("{name}: {why}")),
    };
    let placed = Placed { what: Val { value: Some(name.clone()), unit: String::new(), status: Status::Known, source: format!("the item {item}") }, at: how.val(at, "ft"), offset, angle: None, size, item: Some(item.to_string()) };
    let path = add_called(job, storey, it.category, &name, 2, true)?;
    building::check_placed(&path, it.category, &placed)?;
    job.put(&path, placed)?;
    let note = connections::set_down(job, &path, item)?;
    Ok((path, note))
}

/// What an item's being called something else did to what is set down from it (call 209 point 2).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Renamed {
    /// How many things are set down from the item.
    pub set_down: usize,
    /// How many of them are called by the new name now.
    pub renamed: usize,
    /// The names left as they were: a person gave the thing a name of its own.
    pub kept: Vec<String>,
}

/// Call an item something else, and say what it did to each thing set down from it.
pub fn rename_item_counted(job: &mut Job, path: &str, name: &str) -> Result<Renamed, String> {
    all_or_nothing(job, |job| rename_item_inner(job, path, name))
}

impl Renamed {
    /// What a person is told of the things set down, in a builder's words, and nothing when none is set down. The
    /// command line says what else it likes after it; the window says this as it stands.
    pub fn said(&self) -> String {
        let mut out = match (self.set_down, self.renamed) {
            (0, _) => return String::new(),
            (1, 1) => "The one set down is called that too.".to_string(),
            (n, m) if n == m => format!("The {n} set down are called that too."),
            (n, 0) => format!("None of the {n} set down is called that."),
            (n, m) => format!("{m} of the {n} set down are called that too."),
        };
        match self.kept.len() {
            0 => {}
            1 => out.push_str(&format!(" 1 has a name a person gave it and keeps it: {}.", self.kept[0])),
            n => out.push_str(&format!(" {n} have names a person gave them and keep them: {}.", self.kept.join(", "))),
        }
        out
    }
}

/// Call an item something else. Two items cannot share a name, however it is typed;
/// each thing set down from it is called what the item is. Returns how many were.
pub fn rename_item(job: &mut Job, path: &str, name: &str) -> Result<usize, String> {
    rename_item_counted(job, path, name).map(|r| r.renamed)
}

pub(super) fn rename_item_inner(job: &mut Job, path: &str, name: &str) -> Result<Renamed, String> {
    {
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        if job.world.get::<crate::Item>(e).is_none() {
            return Err(format!("{path} is not an item"));
        }
        let name = name.trim();
        if name.is_empty() {
            return Err("an item needs a name: what the thing is called".into());
        }
        let was = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_default();
        if was == name {
            return Err(format!("it is called {name:?} already"));
        }
        let taken = {
            let mut q = job.world.query::<(&Node, &crate::Item)>();
            q.iter(&job.world).any(|(n, _)| n.path != path && crate::check::same_name(&n.name, name))
        };
        if taken {
            return Err(format!("another item is called {name:?} already"));
        }
        // Nothing else in the item's place goes by that name either (a material, say): the rule bringing a thing in holds (call 213 point 1).
        let parent = job.world.get::<Node>(e).and_then(|n| n.parent.clone()).unwrap_or_default();
        job.name_free(&parent, name, Some(path))?;
        if let Some(mut node) = job.world.get_mut::<Node>(e) {
            node.name = name.to_string();
        }
        let placed: Vec<(String, Kind, Placed)> = {
            let mut q = job.world.query::<(&Node, &Kind, &Placed)>();
            q.iter(&job.world).filter(|(_, _, p)| p.item.as_deref() == Some(path)).map(|(n, k, p)| (n.path.clone(), *k, p.clone())).collect()
        };
        let mut done = Renamed { set_down: placed.len(), renamed: 0, kept: Vec::new() };
        for (at, k, mut p) in placed {
            p.what.value = Some(name.to_string());
            building::check_placed(&at, k, &p)?;
            job.put(&at, p)?;
            // A thing set down is called by its item and, after the first, its number: renamed, it is the new name and the
            // same number (call 209 point 2: the second kept "Vanity, 36 in 2" while the window said it was renamed). A
            // name a person gave it since is theirs: it is left, and counted as left. So is a name that another thing on
            // its level already has.
            let now = job.find(&at).and_then(|e| job.world.get::<Node>(e)).map(|n| n.name.clone()).unwrap_or_default();
            let number = if now == was { Some(None) } else { now.strip_prefix(&format!("{was} ")).filter(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit())).map(|n| Some(n.to_string())) };
            match number {
                Some(number) => {
                    let wanted = match &number {
                        Some(n) => format!("{name} {n}"),
                        None => name.to_string(),
                    };
                    let there = job.find(&at).ok_or("the thing is not there")?;
                    let parent = job.world.get::<Node>(there).and_then(|n| n.parent.clone()).unwrap_or_default();
                    if names_in_scope(job, &parent).contains(&wanted) {
                        done.kept.push(now);
                    } else {
                        if let Some(mut node) = job.find(&at).and_then(|e| job.world.get_mut::<Node>(e)) {
                            node.name = wanted;
                        }
                        done.renamed += 1;
                    }
                }
                None => done.kept.push(now),
            }
            // Its line made of the item is called by the item's new name (call 196).
            if let Some(te) = job.find(&at) {
                if let Some(mut cost) = job.world.get::<Cost>(te).cloned() {
                    let free = !cost.lines.iter().any(|l| !(l.of_material && check::same_name(&l.description, &was)) && check::same_name(&l.description, name));
                    if free {
                        if let Some(l) = cost.lines.iter_mut().find(|l| l.of_material && check::same_name(&l.description, &was)) {
                            l.description = name.to_string();
                            job.world.entity_mut(te).insert(cost);
                        }
                    }
                }
            }
        }
        // Where the item says where it connects, what is set down from it says the item's new name in the source of each (call 211);
        // where each is does not move.
        connections::follow(job, path)?;
        Ok(done)
    }
}

/// Every thing set down from an item is as big as the item says: after the item's
/// sizes change, each follows. Returns the path of each that changed size.
pub(crate) fn placed_follow(job: &mut Job, item_path: &str, item_name: &str, item: &crate::Item) -> Result<Vec<String>, String> {
    let placed: Vec<(String, Kind, Placed)> = {
        let mut q = job.world.query::<(&crate::Node, &Kind, &Placed)>();
        q.iter(&job.world).filter(|(_, _, p)| p.item.as_deref() == Some(item_path)).map(|(n, k, p)| (n.path.clone(), *k, p.clone())).collect()
    };
    let mut followed = Vec::new();
    for (at, k, mut p) in placed {
        let size = match item.size_ft() {
            Ok((feet, status, from)) => Val { value: Some(feet), unit: "ft".into(), status, source: format!("{item_name}: {from}") },
            Err(why) => Val::unknown("ft", &format!("{item_name}: {why}")),
        };
        if p.size == size {
            continue;
        }
        p.size = size;
        building::check_placed(&at, k, &p)?;
        job.put(&at, p)?;
        followed.push(at);
    }
    Ok(followed)
}

/// Where an item's size typed in a window comes from: said once, here, for New item and for a size typed over.
pub const ITEM_SIZE_TYPED: &str = "typed in the window; not yet checked against its maker's sheet";

/// Who is asking for a size to be typed (call 213 point 5): where a size was typed is said by who typed it, and what typing the same size
/// again means depends on it. In the window a person is looking at the item and the message says what happened, so the same size again is
/// the check. At the command line the same line run twice would mark a value Known that nobody measured, so there it changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    Window,
    CommandLine,
}

impl Asked {
    /// Where the size was typed, as the source of a size says it.
    pub fn typed_source(self) -> &'static str {
        match self {
            Asked::Window => ITEM_SIZE_TYPED,
            Asked::CommandLine => "typed at the command line; not yet checked against its maker's sheet",
        }
    }
}

/// What typing a size on an item did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HowSized {
    /// A size that differs from the item's: the item is changed, and is Assumed.
    Changed,
    /// The same size again, or the word "same", in the window: the check, Known, checked on site. At the command line: a size measured, Known,
    /// with where it is from.
    Checked,
    /// The same size again at the command line: nothing changes, and it says so.
    Unchanged,
}

/// What `item_size` did, for a window and a command line to say alike.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemSizeDone {
    /// The item's name.
    pub item: String,
    /// `width`, `depth` or `height`.
    pub what: String,
    pub how: HowSized,
    /// The size it had, in the item's own words ("36 in"); none where it had none.
    pub old: Option<String>,
    /// The size it has.
    pub new: String,
    /// Each thing set down from the item that now follows it, by path.
    pub followed: Vec<String>,
    /// Whether the item's name carries the size it had ("Vanity, 36 in" from 36 in to 48 in), so that its name is
    /// no longer true.
    pub name_carries_old: bool,
    /// What else the measuring change had to say (a run pulled off a joint, say).
    pub also: Vec<String>,
    /// Who asked.
    pub asked: Asked,
    /// Where a measured size is from (a measurement at the command line); empty otherwise.
    pub from: String,
}

impl ItemSizeDone {
    /// What happened, in the words a window and the command line both use.
    pub fn said(&self) -> String {
        let (name, what, now) = (&self.item, &self.what, &self.new);
        let how_to_know = format!("To make it Known, say how sure and where from: plumbline set <file> \"{name}\" {what} \"{now}\" --status Known --source \"<who measured, or with what>\"");
        let mut said = match (self.how, self.asked) {
            (HowSized::Checked, Asked::Window) => format!("{name}: {what} {now} is checked: Known, checked on site."),
            (HowSized::Checked, Asked::CommandLine) => format!("{name}: {what} {now} is Known: {}.", self.from),
            (HowSized::Unchanged, _) => format!("{name}: {what} is {now} already: nothing changed. Typing a size again is not a check. {how_to_know}."),
            (HowSized::Changed, asked) => {
                let follow = match self.followed.len() {
                    0 => "Nothing is set down from it yet.".to_string(),
                    1 => "1 thing set down from it follows.".to_string(),
                    n => format!("{n} things set down from it follow."),
                };
                let note = self.old.as_ref().map(|old| format!(" (it was {old})")).unwrap_or_default();
                let renamed = match (&self.old, self.name_carries_old) {
                    (Some(old), true) => format!(" Its name still reads {old}: Rename item changes it."),
                    _ => String::new(),
                };
                let check = match asked {
                    Asked::Window => format!("To check it, type {now} again on its {what}: that makes it Known, checked on site."),
                    Asked::CommandLine => format!("{how_to_know}."),
                };
                format!("{name}: {what} is now {now}{note}. Assumed: {}. {follow} {check}{renamed}", asked.typed_source())
            }
        };
        for line in &self.also {
            said.push('\n');
            said.push_str(line);
        }
        said
    }
}

/// An item's width, depth or height typed (call 195 step A; contract 200P). One change, all of it or none:
/// - a size that differs from the one the item has is the item changed: the value is Assumed, its source the words
///   New item uses (`ITEM_SIZE_TYPED`: typed, not yet checked against its maker's sheet);
/// - the same size again, or the word "same", is the check: Known, "checked on site" (`who`, where given, goes
///   after it);
/// - every thing set down from the item follows, its size and how sure that is, as sure as the item's three
///   sizes together are (`placed_follow`);
/// - a size that is not a length changes nothing and says why.
/// This is the window's way (`Asked::Window`): a person is looking at the item. The command line asks `item_size_asked`.
pub fn item_size(job: &mut Job, path: &str, what: &str, typed: &str, who: &str) -> Result<ItemSizeDone, String> {
    item_size_asked(job, path, what, typed, who, Asked::Window)
}

/// `item_size`, told who is asking (call 213 points 5 and 6). The window's answer is `item_size`'s; at the command line a size that
/// differs is the item changed, Assumed, said to be typed there, and the same size again, or the word "same", changes nothing and says so.
pub fn item_size_asked(job: &mut Job, path: &str, what: &str, typed: &str, who: &str, asked: Asked) -> Result<ItemSizeDone, String> {
    all_or_nothing(job, |job| {
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let item = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{path} is not an item"))?;
        let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string());
        let was = match what {
            "width" => &item.width,
            "depth" => &item.depth,
            "height" => &item.height,
            other => return Err(format!("an item has a width, a depth and a height: {other:?} is none of them")),
        };
        let word_same = typed.trim().eq_ignore_ascii_case("same");
        let parsed = if word_same { None } else { Some(units::parse_length(typed)?) };
        let was_text = was.value.as_ref().map(|len| units::show_length(len, &was.unit));
        // The same size again: the feet are the same to a ten-thousandth of an inch.
        let again = word_same || parsed.zip(was.feet()).is_some_and(|((len, unit), old)| was.status != Status::Unknown && len.feet(unit).is_ok_and(|now| (now - old).abs() < 1.0e-5));
        let text_of = |job: &Job, what: &str| -> String {
            let it = job.world.get::<Item>(e);
            let v = it.map(|it| match what {
                "width" => &it.width,
                "depth" => &it.depth,
                _ => &it.height,
            });
            v.and_then(|v| v.value.as_ref().map(|len| units::show_length(len, &v.unit))).unwrap_or_default()
        };
        let follower_of = |a: &String| a.strip_suffix(" is one of it and is now that size").map(str::to_string);
        if again && asked == Asked::CommandLine {
            // Nothing changes, and nothing is checked by being typed again.
            if was_text.is_none() {
                return Err(format!("{name} has no {what} to be the same as: type the size"));
            }
            let new = was_text.clone().unwrap_or_default();
            return Ok(ItemSizeDone { item: name, what: what.to_string(), how: HowSized::Unchanged, old: was_text, new, followed: Vec::new(), name_carries_old: false, also: Vec::new(), asked, from: String::new() });
        }
        if again {
            let source = if who.is_empty() { "checked on site".to_string() } else { format!("checked on site: {who}") };
            let done = measure(job, path, what, Reading::Same, None, &source)?;
            let followed: Vec<String> = done.also.iter().filter_map(follower_of).collect();
            let also: Vec<String> = done.also.iter().filter(|a| follower_of(a).is_none()).cloned().collect();
            return Ok(ItemSizeDone { item: name, what: what.to_string(), how: HowSized::Checked, old: was_text.clone(), new: was_text.unwrap_or_else(|| text_of(job, what)), followed, name_carries_old: false, also, asked, from: String::new() });
        }
        // At the command line a size typed with where it is from (Assumed) says that; with nothing it says it was typed there.
        let source = if asked == Asked::CommandLine && !who.trim().is_empty() { who.trim().to_string() } else { asked.typed_source().to_string() };
        let done = measure(job, path, what, Reading::Length(typed), None, &source)?;
        // The size is the item changed, and what is not yet checked is Assumed.
        let mut it = job.world.get::<Item>(e).cloned().unwrap_or(item);
        match what {
            "width" => it.width.status = Status::Assumed,
            "depth" => it.depth.status = Status::Assumed,
            _ => it.height.status = Status::Assumed,
        }
        job.put(path, it.clone())?;
        // Its shape held to its box at the size it has now (call 206 point 2): the size is refused, and nothing changes.
        shapes::item_fits(&job.shapes, &name, &it)?;
        // What follows it is as sure as the item's three sizes together are: where the three were all Known, the ones set
        // down said Known a moment ago, and are put right to what the model says of the item's size.
        let followed: Vec<String> = done.also.iter().filter_map(follower_of).collect();
        if let Ok((feet, status, from)) = it.size_ft() {
            for at in &followed {
                let Some(mut placed) = job.find(at).and_then(|f| job.world.get::<Placed>(f)).cloned() else { continue };
                if placed.size.status != status {
                    placed.size = Val { value: Some(feet), unit: "ft".into(), status, source: format!("{name}: {from}") };
                    job.put(at, placed)?;
                }
            }
        }
        job.derive();
        let also: Vec<String> = done.also.iter().filter(|a| follower_of(a).is_none()).cloned().collect();
        let name_carries_old = was_text.as_ref().is_some_and(|old| !old.is_empty() && name.contains(old.as_str()));
        Ok(ItemSizeDone { item: name, what: what.to_string(), how: HowSized::Changed, old: was_text, new: text_of(job, what), followed, name_carries_old, also, asked, from: String::new() })
    })
}

/// An item's size given as measured (call 213 point 6): how sure and where from, as every value is made Known. The size is what the
/// person read, the item changed to it where it differs, Known, its source what they say, and everything set down from the item follows.
/// Where the item has the size already, it is the same size, now Known. Refused with no source: a measurement says who made it.
pub fn item_size_known(job: &mut Job, path: &str, what: &str, typed: &str, who: &str) -> Result<ItemSizeDone, String> {
    all_or_nothing(job, |job| {
        if who.trim().is_empty() {
            return Err(format!("a size is Known where somebody measured it, and a measurement says where it is from: add --source \"<who measured, or with what>\""));
        }
        let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
        let item = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{path} is not an item"))?;
        let name = job.world.get::<Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string());
        let was = match what {
            "width" => &item.width,
            "depth" => &item.depth,
            "height" => &item.height,
            other => return Err(format!("an item has a width, a depth and a height: {other:?} is none of them")),
        };
        let was_text = was.value.as_ref().map(|len| units::show_length(len, &was.unit));
        let same = typed.trim().eq_ignore_ascii_case("same");
        let done = measure(job, path, what, if same { Reading::Same } else { Reading::Length(typed) }, None, who.trim())?;
        let it = job.world.get::<Item>(e).cloned().unwrap_or(item);
        shapes::item_fits(&job.shapes, &name, &it)?;
        let follower_of = |a: &String| a.strip_suffix(" is one of it and is now that size").map(str::to_string);
        let followed: Vec<String> = done.also.iter().filter_map(follower_of).collect();
        let also: Vec<String> = done.also.iter().filter(|a| follower_of(a).is_none()).cloned().collect();
        job.derive();
        let now = {
            let v = match what {
                "width" => &it.width,
                "depth" => &it.depth,
                _ => &it.height,
            };
            v.value.as_ref().map(|len| units::show_length(len, &v.unit)).unwrap_or_default()
        };
        Ok(ItemSizeDone { item: name, what: what.to_string(), how: HowSized::Checked, old: was_text, new: now, followed, name_carries_old: false, also, asked: Asked::CommandLine, from: who.trim().to_string() })
    })
}

/// How far a thing is turned, said as a builder says it, from 0 up to 360 degrees counter-clockwise from east (call 213 point 7). The model
/// keeps it between -180 and 180; a turn is said one way everywhere, and this is it.
pub fn turned_said(degrees: f64) -> f64 {
    let d = (degrees.rem_euclid(360.0) * 1.0e6).round() / 1.0e6;
    if d >= 360.0 { 0.0 } else { d + 0.0 }
}

/// A source after something was done to the value it stands on: where the value came
/// from, then each thing done to it since and on whose word. It loses nothing.
pub(crate) fn after(source: &str, done: &str, by: &str) -> String {
    // Nothing a source says is taken out of it: what a person wrote and a note this
    // wrote look alike, and a guess at which is which would cut a person's words. The
    // one thing not done is saying the same thing twice running: a wall moved four
    // times on one person's word says so once.
    let note = format!("; {done} ({by})");
    if source.ends_with(&note) {
        source.to_string()
    } else {
        format!("{source}{note}")
    }
}

/// What `price_item` priced.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemPriced {
    /// The item's name.
    pub item: String,
    /// Each thing set down from the item whose line named for it was priced.
    pub priced: Vec<String>,
    /// Each that has no line of that name, and so took nothing.
    pub without: Vec<String>,
}

/// A unit cost on the cost line named for an item, on every one of the item set in the job
/// (a box set in a run carries one line named for its item): one price for one code
/// (call 157). `item` is the item's name or its path. Things set from it that have no line
/// of that name take nothing, and are listed. Nothing is set where none of the item has
/// been set down yet; that is for the caller to say.
pub fn price_item(job: &mut Job, item: &str, price: &str, status: Status, source: &str) -> Result<ItemPriced, String> {
    all_or_nothing(job, |job| {
        let found: Vec<(u64, String, String)> = {
            let mut q = job.world.query::<(&Node, &Item)>();
            q.iter(&job.world).filter(|(n, _)| n.path == item || n.name.eq_ignore_ascii_case(item.trim())).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone(), n.name.clone())).collect()
        };
        // What is cut to its run and is no item of the tool chest (a countertop) is found by
        // what it is: every thing set in place of that name that is of no item (call 162).
        let by_what: Vec<(u64, String)> = {
            let mut q = job.world.query::<(&Node, &Placed)>();
            q.iter(&job.world).filter(|(_, p)| p.item.is_none() && p.what.value.as_deref().is_some_and(|w| w.eq_ignore_ascii_case(item.trim()))).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone())).collect()
        };
        let found_by_what = found.is_empty() && !by_what.is_empty();
        let (name, set): (String, Vec<String>) = match found.into_iter().min() {
            Some((_, path, name)) => {
                let mut q = job.world.query::<(&Node, &Placed)>();
                let mut all: Vec<(u64, String)> = q.iter(&job.world).filter(|(_, p)| p.item.as_deref() == Some(path.as_str())).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone())).collect();
                all.sort();
                (name, all.into_iter().map(|(_, p)| p).collect())
            }
            None if !by_what.is_empty() => {
                let mut all = by_what;
                all.sort();
                let name = job.world.query::<&Placed>().iter(&job.world).find_map(|p| p.what.value.clone().filter(|w| w.eq_ignore_ascii_case(item.trim()))).unwrap_or_else(|| item.trim().to_string());
                (name, all.into_iter().map(|(_, p)| p).collect())
            }
            None => return Err(format!("{item} is not an item of this job: price takes an item's name, like B18")),
        };
        // A countertop is bought by the LF of its front edge (its width) or by the SF of its
        // top (its plan area): a price that says which puts the line on that figure; one that
        // says another unit is refused with the two it takes (call 167).
        let countertop = found_by_what;
        let figure: Option<&str> = match (countertop, price.rsplit_once('/')) {
            (true, Some((_, per))) => match per.trim().to_uppercase().as_str() {
                "LF" => Some("width"),
                "SF" => Some("plan_area"),
                _ => return Err(format!("{price:?} is a price per {}, and a countertop is priced by the LF of its front edge or by the SF of its top: write it like $80/LF or $62/SF", per.trim())),
            },
            _ => None,
        };
        let (mut priced, mut without) = (Vec::new(), Vec::new());
        for thing in set {
            let has = job.find(&thing).and_then(|e| job.world.get::<Cost>(e)).is_some_and(|c| c.lines.iter().any(|l| l.description == name));
            // A line made of its item has no price of its own: the item's is in the price list (call 206 point 7).
            let made_of_item = job.find(&thing).and_then(|e| job.world.get::<Cost>(e)).is_some_and(|c| c.lines.iter().any(|l| l.description == name && l.of_material));
            if made_of_item {
                return Err(format!("{thing} cost line {name:?} is made of the item itself, so its price is the item's and not the line's: give it in a price list (plumbline import prices, a row named {name:?}) or in the item table's Unit cost column, and each of it set down takes it"));
            }
            if has {
                if let Some(figure) = figure {
                    let on = job.find(&thing).and_then(|e| job.world.get::<Cost>(e)).and_then(|c| c.lines.iter().find(|l| l.description == name)).map(|l| l.quantity.clone());
                    if on.as_deref() != Some(figure) {
                        // A price does not carry from one unit to another: it is cleared, the
                        // line is put on the figure, and the price is entered in its unit.
                        job.set_on_line(&thing, "unit_cost", Some(&name), None, Status::Unknown, "the line changed units")?;
                        job.set_formula(&thing, Some(&name), figure, &[], status, source)?;
                    }
                }
                job.set_on_line(&thing, "unit_cost", Some(&name), Some(price), status, source)?;
                priced.push(thing);
            } else {
                without.push(thing);
            }
        }
        Ok(ItemPriced { item: name, priced, without })
    })
}
