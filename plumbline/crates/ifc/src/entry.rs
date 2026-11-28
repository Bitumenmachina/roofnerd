use super::*;

/// A thing brought in is called what the file calls it where nothing else in its place goes by
/// that, else that with its number in the file (call 197, point 5).
pub(super) fn keep_name(job: &mut Job, path: &str, called: &str, id: u64) {
    let called = called.trim();
    let Some(e) = job.find(path) else { return };
    let (Some(parent), now) = (job.world.get::<plumbline_world::Node>(e).and_then(|n| n.parent.clone()), job.world.get::<plumbline_world::Node>(e).map(|n| n.name.clone()).unwrap_or_default()) else { return };
    if called.is_empty() || now == called {
        return;
    }
    let name = if job.children().get(&parent).is_some_and(|c| c.contains_key(called)) { format!("{called} {id}") } else { called.to_string() };
    if let Some(mut node) = job.world.get_mut::<plumbline_world::Node>(e) {
        node.name = name;
    }
}

/// What a thing is as the model this program wrote says it, less the mark of how sure it is:
/// `Ceiling register, 4 x 10 (Known; sample plan)` is `Ceiling register, 4 x 10`.
pub(super) fn said_less_its_mark(text: &str) -> Option<String> {
    let at = text.rfind(" (")?;
    let inside = &text[at + 2..];
    ["Known;", "Assumed;", "Unknown;"].iter().any(|s| inside.starts_with(s)).then(|| text[..at].trim().to_string()).filter(|t| !t.is_empty())
}

/// What the model calls a kind of thing, as words: `IFCBEAM` is a beam. The model writes
/// every such name as one run of capitals, so the words of the commoner ones are given
/// here; any other is its name less the "IFC", as the model has it.
pub(super) fn said(class: &str) -> String {
    const WORDS: [(&str, &str); 45] = [
        ("IFCBEAM", "beam"), ("IFCCOLUMN", "column"), ("IFCSTAIR", "stair"), ("IFCSTAIRFLIGHT", "stair flight"), ("IFCROOF", "roof"), ("IFCCOVERING", "covering"), ("IFCFURNITURE", "furniture"),
        ("IFCFURNISHINGELEMENT", "furnishing"), ("IFCBUILDINGELEMENTPROXY", "thing of no stated kind"), ("IFCMEMBER", "framing member"), ("IFCPLATE", "plate"), ("IFCRAILING", "railing"), ("IFCRAMP", "ramp"),
        ("IFCRAMPFLIGHT", "ramp flight"), ("IFCCURTAINWALL", "curtain wall"), ("IFCFOOTING", "footing"), ("IFCPILE", "pile"), ("IFCFLOWTERMINAL", "fixture or outlet"), ("IFCFLOWFITTING", "fitting"),
        ("IFCDUCTFITTING", "duct fitting"), ("IFCPIPEFITTING", "pipe fitting"), ("IFCCABLECARRIERFITTING", "tray fitting"), ("IFCCABLEFITTING", "cable fitting"), ("IFCFLOWSEGMENT", "run"),
        ("IFCDUCTSEGMENT", "duct"), ("IFCPIPESEGMENT", "pipe"), ("IFCCABLECARRIERSEGMENT", "cable tray"), ("IFCCABLESEGMENT", "cable"), ("IFCFLOWCONTROLLER", "valve or damper"),
        ("IFCVALVE", "valve"), ("IFCDAMPER", "damper"), ("IFCAIRTERMINAL", "grille or diffuser"), ("IFCSANITARYTERMINAL", "plumbing fixture"), ("IFCLIGHTFIXTURE", "light fixture"),
        ("IFCOUTLET", "outlet"), ("IFCWALL", "wall"), ("IFCSLAB", "floor slab"), ("IFCROOFSLAB", "roof slab"), ("IFCDOOR", "door"), ("IFCWINDOW", "window"), ("IFCSPACE", "space"), ("IFCOPENINGELEMENT", "opening"),
        ("IFCDISTRIBUTIONPORT", "connection point"), ("IFCGRID", "grid"), ("IFCANNOTATION", "note or dimension"),
    ];
    let base = class.trim_end_matches("STANDARDCASE").trim_end_matches("ELEMENTEDCASE");
    match WORDS.iter().find(|(c, _)| *c == base) {
        Some((_, words)) => words.to_string(),
        None => format!("\"{}\" (as the model calls it)", class.strip_prefix("IFC").unwrap_or(class).to_lowercase()),
    }
}

/// A point inside an outline: the middle of its corners where that is inside it, else
/// the middle of the widest stretch of the outline on a line across it at that height.
/// (The middle of an L-shaped room's corners can lie outside the room.)
pub(super) fn point_inside(points: &[[f64; 2]]) -> [f64; 2] {
    let n = points.len() as f64;
    let mid = [points.iter().map(|p| p[0]).sum::<f64>() / n, points.iter().map(|p| p[1]).sum::<f64>() / n];
    // Where the outline crosses the line across it at `y`, in order.
    let crossings = |y: f64| -> Vec<f64> {
        let mut xs: Vec<f64> = (0..points.len())
            .filter_map(|i| {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                ((a[1] > y) != (b[1] > y)).then(|| a[0] + (y - a[1]) / (b[1] - a[1]) * (b[0] - a[0]))
            })
            .collect();
        xs.sort_by(f64::total_cmp);
        xs
    };
    let xs = crossings(mid[1]);
    let inside: Vec<(f64, f64)> = xs.chunks_exact(2).map(|c| (c[0], c[1])).collect();
    if inside.iter().any(|(a, b)| *a < mid[0] && mid[0] < *b) {
        return mid;
    }
    match inside.iter().max_by(|p, q| (p.1 - p.0).total_cmp(&(q.1 - q.0))) {
        Some((a, b)) => [(a + b) / 2.0, mid[1]],
        None => mid,
    }
}

/// How many things in the file are written under a number another thing already has.
/// The reader keeps the later of each; this is how many earlier ones it lost.
pub(super) fn numbers_used_twice(bytes: &[u8], kept: usize) -> usize {
    // Each thing is written `#12= NAME(...);`. Text in quotes can hold anything, so it
    // is stepped over.
    let (mut written, mut i, mut fresh) = (0usize, 0usize, true);
    while i < bytes.len() {
        match bytes[i] {
            b'\'' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
            }
            b';' => fresh = true,
            b'#' if fresh => {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                let digits = j > i + 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if digits && bytes.get(j) == Some(&b'=') {
                    written += 1;
                }
                fresh = false;
            }
            c if !c.is_ascii_whitespace() => fresh = false,
            _ => {}
        }
        i += 1;
    }
    written.saturating_sub(kept)
}

/// What the file's lengths are in: the name as the file has it, and feet to one of it.
pub(super) fn length_unit(r: &Reader) -> (String, f64) {
    const FT_PER_M: f64 = 1.0 / 0.3048;
    let si = |e: &Entity| -> Option<(String, f64)> {
        if !matches!(r.arg(e, 1), V::Enum(k) if k == "LENGTHUNIT") || !matches!(r.arg(e, 3), V::Enum(n) if n == "METRE") {
            return None;
        }
        Some(match r.arg(e, 2) {
            V::Enum(p) if p == "MILLI" => ("millimeters".to_string(), 0.001 * FT_PER_M),
            V::Enum(p) if p == "CENTI" => ("centimeters".to_string(), 0.01 * FT_PER_M),
            V::Enum(p) if p == "DECI" => ("decimeters".to_string(), 0.1 * FT_PER_M),
            _ => ("meters".to_string(), FT_PER_M),
        })
    };
    for e in r.file.entities.values().filter(|e| e.name == "IFCUNITASSIGNMENT") {
        for u in Reader::list(r.arg(e, 0)) {
            let Some((_, u)) = r.get(u) else { continue };
            match u.name.as_str() {
                "IFCSIUNIT" => {
                    if let Some(found) = si(u) {
                        return found;
                    }
                }
                // A foot or an inch, said as so many of an SI unit.
                "IFCCONVERSIONBASEDUNIT" if matches!(r.arg(u, 1), V::Enum(k) if k == "LENGTHUNIT") => {
                    let name = Reader::text(r.arg(u, 2)).unwrap_or("").to_lowercase();
                    if let Some((_, measure)) = r.get(r.arg(u, 3)) {
                        let (factor, base) = (Reader::number(r.arg(measure, 0)), r.get(r.arg(measure, 1)).and_then(|(_, b)| si(b)));
                        if let (Some(factor), Some((_, base))) = (factor, base) {
                            // "the model is in feet", not "in foot".
                            let name = match name.as_str() {
                                "foot" => "feet".to_string(),
                                "inch" => "inches".to_string(),
                                _ => name,
                            };
                            return (name, factor * base);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    ("meters (the file does not say; an IFC file with no length unit is in meters)".to_string(), FT_PER_M)
}

/// A name for a thing brought in: what the model calls it where nothing else in that
/// place goes by that, else that with its number in the file; a thing the model does not
/// name is its kind and its number.
pub(super) fn free_name(job: &mut Job, parent: &str, called: &str, kind: &str, id: u64) -> String {
    if called.is_empty() {
        format!("{kind} {id}")
    } else if job.children().get(parent).is_some_and(|c| c.contains_key(called)) {
        format!("{called} {id}")
    } else {
        called.to_string()
    }
}

pub(super) fn dist3(a: P3, b: P3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

pub(super) fn tidy(x: f64) -> f64 {
    (x * 1.0e4).round() / 1.0e4 + 0.0
}

pub(super) fn feet(x: f64) -> String {
    format!("{} ft", tidy(x))
}

/// Bring a model in. `name` is what the file is called, for the sources of what is made.
/// All of it or none: if the job would not check afterwards, it is as it was.
pub fn import(job: &mut Job, bytes: &[u8], name: &str, how: Entered) -> Result<Brought, String> {
    import_whole(job, bytes, name, how, Bring::Whole, false)
}

/// Bring a model in, choosing what of it (call 210). `Bring::Whole` is `import` and says in addition, in the reply, what the
/// things brought in do to the job's clashes and, where walls or floors of the model were left out for lying over the job's own,
/// that only its runs can be chosen; `Bring::RunsOnly` brings only its runs and what they serve (see `Bring`), and counts the rest
/// as not asked for. What is brought in is Assumed and sourced to the file, either way.
pub fn import_choosing(job: &mut Job, bytes: &[u8], name: &str, how: Entered, bring: Bring) -> Result<Brought, String> {
    import_whole(job, bytes, name, how, bring, true)
}

/// What stands in the way of a model being brought in, and what a person can do about it today (call 209 point 4, call 210). A model
/// comes in whole or only its runs and what they serve: bringing in only what the job lacks is not there yet. `alone` is what a
/// trial of bringing in only its runs found would come in (the choice worked, and taken back), said, where the whole was refused.
pub(super) fn with_a_way_on(why: String, bring: Bring, alone: Option<String>) -> String {
    let alone = alone.map(|a| format!(". Its runs and the equipment they serve ({a}) can be brought in alone, without its walls and floors")).unwrap_or_default();
    if why.starts_with("this model is already in the job") {
        if bring == Bring::RunsOnly {
            return format!("{why}. To bring them in again, take what they brought in out of this job first. Bringing in only what the job lacks is not there yet");
        }
        return format!("{why}. To bring it in again, bring it into a job of its own, or take what it brought in out of this job first. A model comes in whole: bringing in only what the job lacks is not there yet{alone}");
    }
    if why.contains("would be counted twice") {
        let said = if why.contains("nothing was brought in") { why } else { format!("{why}. Nothing was brought in") };
        return format!("{said}. To bring this model in, take out of the job what is already there on that ground (here, the floor), or bring the model into a job of its own. A model comes in whole: bringing in only what the job lacks is not there yet{alone}");
    }
    format!("{why}{alone}")
}

/// What not asked for is called, as a builder says it, for one and for more: ("walls", "wall").
pub(super) const NOT_ASKED_WORDS: [(&str, &str); 11] = [
    ("walls", "wall"), ("floors", "floor"), ("roofs", "roof"), ("footings", "footing"), ("doors", "door"), ("windows", "window"), ("rooms", "room"),
    ("beams", "beam"), ("posts", "post"), ("furnishings", "furnishing"), ("holes in walls", "hole in a wall"),
];

/// What a kind of thing brought in is called for one of it: "ducts" is "duct", "pieces of equipment" is "piece of equipment".
pub fn one_of(what: &str) -> &str {
    if what == "pieces of equipment" {
        return "piece of equipment";
    }
    what.strip_suffix('s').unwrap_or(what)
}

/// "a", "a and b", "a, b and c".
pub(super) fn list_said(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// What was not asked for, as words: "1 floor and 4 walls".
pub(super) fn not_asked_list(not_asked: &[(String, usize)]) -> String {
    let each: Vec<String> = not_asked
        .iter()
        .map(|(what, n)| {
            let word = if *n == 1 { NOT_ASKED_WORDS.iter().find(|(many, _)| many == what).map(|(_, one)| one.to_string()).unwrap_or_else(|| what.strip_suffix('s').unwrap_or(what).to_string()) } else { what.clone() };
            format!("{n} {word}")
        })
        .collect();
    list_said(&each)
}

/// Each clash of the job, by the model's own clash list, with the kind it counts as and the two things it is between: what
/// `plumbline clashes` lists, and counted by the same accessors (call 210 point 4).
pub(super) struct ClashRow {
    pub(super) kind: &'static str,
    pub(super) a: String,
    pub(super) b: String,
    pub(super) said: String,
}

pub(super) fn clash_state(job: &mut Job) -> Result<(ClashCounts, Vec<ClashRow>), String> {
    use plumbline_world::building::clash;
    let found = clash::clashes(job, None)?;
    let inches = |feet: f64| format!("{:.2} in", feet * 12.0 + 0.0);
    let place = |at: [f64; 3]| format!("({}, {}, {})", tidy(at[0]), tidy(at[1]), tidy(at[2]));
    let counts = ClashCounts { same_space: found.in_the_same_space().len(), set_against: found.set_against().len(), holes: found.hole_rows().len(), laid_in: found.laid_in().len() };
    let mut rows = Vec::new();
    for c in found.in_the_same_space() {
        let by = c.by.map(|b| format!("in each other by {}", inches(b))).unwrap_or_else(|| "in each other".to_string());
        rows.push(ClashRow { kind: "in the same space", a: c.a.clone(), b: c.b.clone(), said: format!("{} and {}, {by}, at {}", c.a_name, c.b_name, place(c.at)) });
    }
    for c in found.set_against() {
        rows.push(ClashRow { kind: "set against a wall", a: c.a.clone(), b: c.b.clone(), said: format!("{} and {}, set {} into it, at {}", c.a_name, c.b_name, inches(c.by.unwrap_or(0.0)), place(c.at)) });
    }
    for c in found.hole_rows() {
        let said = match c.hole.and_then(|h| found.holes.get(h)) {
            Some(h) => format!("{} and {}{}, at {}", h.pieces.first().map(|(_, n)| n.as_str()).unwrap_or(&c.a_name), h.through_name, if h.size.is_empty() { String::new() } else { format!(", {}", h.size) }, place(c.at)),
            None => format!("{} and {}, at {}", c.a_name, c.b_name, place(c.at)),
        };
        rows.push(ClashRow { kind: "a hole to cut", a: c.a.clone(), b: c.b.clone(), said });
    }
    for c in found.laid_in() {
        rows.push(ClashRow { kind: "laid in a floor or a wall", a: c.a.clone(), b: c.b.clone(), said: format!("{} and {}, at {}", c.a_name, c.b_name, place(c.at)) });
    }
    Ok((counts, rows))
}

/// Every thing of the job, by path.
pub(super) fn paths_of(job: &mut Job) -> HashSet<String> {
    let mut q = job.world.query::<&plumbline_world::Node>();
    q.iter(&job.world).map(|n| n.path.clone()).collect()
}

pub(super) fn import_whole(job: &mut Job, bytes: &[u8], name: &str, how: Entered, choice: Bring, extras: bool) -> Result<Brought, String> {
    let file = step::parse(bytes).map_err(|why| format!("it is not an IFC file this can read ({why})"))?;
    if !file.schema.starts_with("IFC") {
        return Err(format!("it is a file written the way a model is, but of another kind (\"{}\"): not an IFC model", file.schema));
    }
    if how.source.trim().is_empty() {
        return Err("say where the model came from".into());
    }
    let r = Reader { file: &file };
    let (unit_name, k) = length_unit(&r);
    let twice = numbers_used_twice(bytes, file.entities.len());
    // A model this program wrote says, value by value, how sure each is (in property
    // sets called Plumbline_Values). Which mark belonged to which value is not read
    // back, so where any is Assumed or Unknown nothing from the model comes in as
    // Known, whatever is said of it as a whole, and the report says so (J17B-16).
    let text = String::from_utf8_lossy(bytes);
    let (assumed, unknown) = if text.contains("'Plumbline_Values'") { (text.matches("(Assumed;").count(), text.matches("(Unknown;").count()) } else { (0, 0) };
    let said_known = how.status == Status::Known;
    // Everything read from a model is Assumed (call 180, C3): a model is what its maker drew, not what
    // was measured here, so a model is never Known on the strength of being said to be.
    let how = if said_known { Entered { status: Status::Assumed, source: how.source } } else { how };
    // What the model holds that is not a thing with a shape, and is not read at all.
    let count = |names: &[&str]| file.entities.values().filter(|e| names.contains(&e.name.as_str())).count();
    let (property_sets, materials) = (count(&["IFCPROPERTYSET", "IFCELEMENTQUANTITY"]), count(&["IFCMATERIAL", "IFCMATERIALLAYERSET", "IFCMATERIALCONSTITUENTSET", "IFCMATERIALPROFILESET"]));
    let only_runs = choice == Bring::RunsOnly;
    let done = ed::all_or_nothing(job, |job| {
        // The clashes the job has, and the things it has, before anything comes in (call 210 point 4).
        let before = if extras { Some((paths_of(job), clash_state(job))) } else { None };
        // The rooms as they were, for the rooms the model changes to be named (call 206 point 3); a job with no room has none to change.
        let rooms_before = if !only_runs && job.world.query::<&plumbline_world::Room>().iter(&job.world).next().is_some() { ed::room_figures(job) } else { Vec::new() };
        let mut out = bring(job, &r, name, how, unit_name, k, only_runs, extras)?;
        if !rooms_before.is_empty() {
            let after = ed::room_figures(job);
            out.rooms_moved = ed::rooms_moved(&rooms_before, &after);
            if !out.rooms_moved.is_empty() {
                out.notes.push(format!(
                    "{} of the job {} other figures now, because walls brought in from the model stand in {}: {}",
                    if out.rooms_moved.len() == 1 { "1 room".to_string() } else { format!("{} rooms", out.rooms_moved.len()) },
                    if out.rooms_moved.len() == 1 { "has" } else { "have" },
                    if out.rooms_moved.len() == 1 { "it" } else { "them" },
                    out.rooms_moved.iter().map(|m| m.said()).collect::<Vec<_>>().join("; ")
                ));
            }
        }
        if said_known && assumed + unknown == 0 {
            out.notes.push("everything from this model was brought in Assumed, not Known as was said of it: a model is what its maker drew, not what was measured here. Measure it and mark it Known".to_string());
        }
        if assumed + unknown > 0 {
            out.notes.push(format!(
                "this model was written by BuildOS and marks {assumed} of its values Assumed and {unknown} Unknown: which value each mark was on is not read back, so {}",
                if said_known { "everything from it was brought in Assumed, not Known as was said of it" } else { "nothing from it is surer than was said of it as a whole" }
            ));
        }
        let property_sets = property_sets.saturating_sub(storey_heights(&r).len()).saturating_sub(out.made.get("roofs").copied().unwrap_or(0));
        if property_sets + materials > 0 {
            // What the model keeps on its things besides their shapes, said as a builder would (call 209 point 3): notes, and
            // materials only when there are some.
            let notes = if property_sets == 0 { String::new() } else { format!("the notes the model carries on its things ({property_sets} of them)") };
            let mats = if materials == 0 { String::new() } else { format!("{materials} material{} it names", if materials == 1 { "" } else { "s" }) };
            let both = [notes, mats].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" and ");
            out.notes.push(format!(
                "not read: {both}. A thing brought in has its shape, its place and its name, and, where the model gives them, its work mark (new, existing, to demolish or temporary) and its bearing mark, each Assumed; what the model says it is made of, and anything else it says about each thing, is not brought in"
            ));
        }
        if twice > 0 {
            out.notes.push(format!("{twice} number{} in the file {} used by more than one thing: the later of each was read and the earlier was not, so something may be missing or changed", if twice == 1 { "" } else { "s" }, if twice == 1 { "is" } else { "are" }));
        }
        // What it was done for (call 210 point 4): the clashes the things brought in make with what the job had, by the model's own
        // clash list, worked before and after, and the ones that have a thing brought in as one of the two.
        if let Some((had, state_before)) = before {
            match (state_before, clash_state(job)) {
                (Ok((counts_before, _)), Ok((counts_after, rows))) => {
                    let mine: Vec<&ClashRow> = rows.iter().filter(|c| !had.contains(&c.a) || !had.contains(&c.b)).collect();
                    let by = |kind: &str| mine.iter().filter(|c| c.kind == kind).count();
                    out.clashes = Some(ClashChange {
                        before: counts_before,
                        after: counts_after,
                        made: ClashCounts { same_space: by("in the same space"), set_against: by("set against a wall"), holes: by("a hole to cut"), laid_in: by("laid in a floor or a wall") },
                        listed: mine.iter().map(|c| format!("{}: {}", c.kind, c.said)).collect(),
                    });
                }
                (Err(why), _) | (_, Err(why)) => out.notes.push(format!("the clashes of the job could not be counted before and after: {why}")),
            }
        }
        Ok(out)
    });
    done.map_err(|why| {
        // Where the whole was refused, what a bringing in of its runs alone would bring (worked, and taken back).
        let alone = (extras && !only_runs)
            .then(|| {
                let mut found = None;
                let _ = ed::all_or_nothing(job, |job| {
                    if let Ok(b) = import_whole(job, bytes, name, how.clone(), Bring::RunsOnly, false) {
                        let each: Vec<String> = b.made.iter().map(|(what, n)| format!("{n} {}", if *n == 1 { one_of(what) } else { what })).collect();
                        found = Some(list_said(&each));
                    }
                    Err::<(), String>("a trial, taken back".to_string())
                });
                found
            })
            .flatten();
        with_a_way_on(why, choice, alone)
    })
}

/// The height each building level is given in the base quantities IFC4 gives a storey
/// (`GrossHeight`, in the model's unit), by the level's number in the file (call 180, C3).
pub(super) fn storey_heights(r: &Reader) -> HashMap<u64, f64> {
    let mut out = HashMap::new();
    for rel in r.file.entities.values().filter(|e| e.name == "IFCRELDEFINESBYPROPERTIES") {
        let Some((_, set)) = r.get(r.arg(rel, 5)) else { continue };
        if set.name != "IFCELEMENTQUANTITY" || Reader::text(r.arg(set, 2)) != Some("Qto_BuildingStoreyBaseQuantities") {
            continue;
        }
        let height = Reader::list(r.arg(set, 5)).iter().filter_map(|q| r.get(q)).find(|(_, q)| q.name == "IFCQUANTITYLENGTH" && Reader::text(r.arg(q, 0)) == Some("GrossHeight")).and_then(|(_, q)| Reader::number(r.arg(q, 3)));
        let Some(height) = height else { continue };
        for thing in Reader::list(r.arg(rel, 4)) {
            if let V::Ref(id) = thing {
                out.insert(*id, height);
            }
        }
    }
    out
}
