use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn bring(job: &mut Job, r: &Reader, name: &str, how: Entered, unit_name: String, k: f64, only_runs: bool, extras: bool) -> Result<Brought, String> {
    let mut out = Brought { schema: r.file.schema.clone(), unit: unit_name, feet_per_unit: k, ..Brought::default() };
    // What was not asked for, when only the model's runs and what they serve were (call 210 point 1).
    let mut na: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut left: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut leave = |class: &str, why: &str| *left.entry((said(class), why.to_string())).or_insert(0) += 1;
    let mut made = |what: &str| *out.made.entry(what.to_string()).or_insert(0) += 1;
    // Every value says the thing in the file it was read off.
    let from = |e: &Entity, id: u64| -> String {
        let called = r.name(e);
        // The model's own word for the thing, less the "IFC" every one of them starts
        // with, and its number in the file, so it can be found there again.
        // (In the trade's word for it, and "#113", as the file itself writes its
        // numbers: "wall 44 in the file" read as forty-four inches, J17B-19.)
        let its = said(&e.name);
        format!("{}: {name}, {}{its} #{id} in the file", how.source, if called.is_empty() { String::new() } else { format!("\"{called}\", ") })
    };
    let val = |source: &str, len: f64| Val { value: Some(Len::One(tidy(len))), unit: "ft".to_string(), status: how.status, source: source.to_string() };
    let path_val = |source: &str, path: &str| Val { value: Some(path.to_string()), unit: String::new(), status: how.status, source: source.to_string() };
    let mut ids: Vec<u64> = r.file.entities.keys().copied().collect();
    ids.sort_unstable();
    let of = |class: &[&str]| -> Vec<(u64, &Entity)> { ids.iter().filter_map(|id| r.file.entities.get(id).map(|e| (*id, e))).filter(|(_, e)| class.contains(&e.name.as_str())).collect() };

    // ---- What the model says of each thing in its property sets: the work mark and whether a
    // wall carries (`Status` and `LoadBearing` in the common property set IFC4 gives its kind),
    // and what this program wrote its type's name as (call 175). Each comes in as the mark of
    // the thing, Assumed, and says the file it came from.
    #[derive(Default)]
    struct OfIt {
        status: Option<String>,
        carries: Option<bool>,
        type_name: Option<String>,
        what: Option<String>,
    }
    let mut said_of: HashMap<u64, OfIt> = HashMap::new();
    for (_, rel) in of(&["IFCRELDEFINESBYPROPERTIES"]) {
        let Some((_, set)) = r.get(r.arg(rel, 5)) else { continue };
        if set.name != "IFCPROPERTYSET" {
            continue;
        }
        let set_name = Reader::text(r.arg(set, 2)).unwrap_or("");
        let mut it = OfIt::default();
        for prop in Reader::list(r.arg(set, 4)) {
            let Some((_, p)) = r.get(prop) else { continue };
            if p.name != "IFCPROPERTYSINGLEVALUE" {
                continue;
            }
            let (called, value) = (Reader::text(r.arg(p, 0)).unwrap_or(""), r.arg(p, 2));
            match called {
                "Status" if set_name.starts_with("Pset_") && set_name.ends_with("Common") => it.status = Reader::text(value).map(str::to_string),
                "LoadBearing" if set_name.starts_with("Pset_") && set_name.ends_with("Common") => {
                    if let V::Typed(_, inner) = value {
                        it.carries = match inner.as_ref() {
                            V::Enum(x) if x == "T" => Some(true),
                            V::Enum(x) if x == "F" => Some(false),
                            _ => None,
                        };
                    }
                }
                "what it is" if set_name == "Plumbline_Values" => it.what = Reader::text(value).and_then(said_less_its_mark),
                "type" if set_name == "Plumbline_Values" => it.type_name = Reader::text(value).and_then(|t| t.find(" (type-").map(|at| t[..at].to_string())).filter(|n| !n.is_empty()),
                _ => {}
            }
        }
        for obj in Reader::list(r.arg(rel, 4)) {
            if let V::Ref(id) = obj {
                let at = said_of.entry(*id).or_default();
                at.status = at.status.take().or(it.status.clone());
                at.carries = at.carries.or(it.carries);
                at.type_name = at.type_name.take().or(it.type_name.clone());
                at.what = at.what.take().or(it.what.clone());
            }
        }
    }
    // The mark a thing comes in with, from what the model says of it; the file's own word
    // for the work, never Known on its say-so.
    let marks = |job: &mut Job, path: &str, id: u64, source: &str, kind: Kind| -> Result<(), String> {
        use plumbline_world::components::{Bearing, Hidden, Work, WorkStatus};
        let Some(it) = said_of.get(&id) else { return Ok(()) };
        if let Some(word) = &it.status {
            let mark = match word.as_str() {
                "NEW" => Some(WorkStatus::New),
                "EXISTING" => Some(WorkStatus::Existing),
                "DEMOLISH" => Some(WorkStatus::Demolish),
                "TEMPORARY" => Some(WorkStatus::Temporary),
                _ => None,
            };
            if let Some(mark) = mark {
                job.put(path, Work { status: Val { value: Some(mark), unit: String::new(), status: Status::Assumed, source: source.to_string() } })?;
            }
        }
        if let (Kind::Wall, Some(carries)) = (kind, it.carries) {
            let bearing = if carries { Bearing::Bearing } else { Bearing::NotBearing };
            job.put(path, Hidden { bearing: Some(Val { value: Some(bearing), unit: String::new(), status: Status::Assumed, source: source.to_string() }), in_the_wall: None })?;
        }
        Ok(())
    };

    // ---- Storeys, lowest first. Each is as tall as it is to the next one up.
    // A storey says how high its floor is (its elevation), and it is also placed. The
    // two differ when the building itself is placed at a height: the floor's height is
    // what the storey says, and everything on the storey is measured from where the
    // storey is placed, so a building 50 m up does not lift its walls 50 m off their floor.
    let mut storeys: Vec<(u64, String, f64, f64)> = of(&["IFCBUILDINGSTOREY"])
        .into_iter()
        .map(|(id, e)| {
            let placed = r.get(r.arg(e, 5)).map(|_| r.placed(r.arg(e, 5)).o[2] * k);
            let stated = Reader::number(r.arg(e, 9)).map(|z| z * k);
            (id, r.name(e), stated.or(placed).unwrap_or(0.0), placed.or(stated).unwrap_or(0.0))
        })
        .collect();
    storeys.sort_by(|a, b| a.2.total_cmp(&b.2));
    if storeys.is_empty() {
        return Err("the model has no building level: there is nothing to stand its walls and floors on".into());
    }
    // Only its runs: no level is added, so each run and thing stands on a level the job has (call 210 point 2).
    let job_levels: Vec<(String, String, f64)> = {
        let mut q = job.world.query::<(&plumbline_world::Node, &building::Level)>();
        let mut v: Vec<(String, String, f64)> = q.iter(&job.world).filter_map(|(n, l)| l.elevation.feet().map(|e| (n.name.clone(), n.path.clone(), e))).collect();
        v.sort_by(|a, b| a.2.total_cmp(&b.2));
        v
    };
    // How high each of the job's levels reaches, by path: its floor and its height (call 213 point 11).
    let job_tops: HashMap<String, f64> = {
        let mut q = job.world.query::<(&plumbline_world::Node, &building::Level)>();
        q.iter(&job.world).filter_map(|(n, l)| Some((n.path.clone(), l.elevation.feet()? + l.height.feet()?))).collect()
    };
    if only_runs && job_levels.is_empty() {
        return Err("the job has no level with a height to put the model's runs on: bringing in only its runs adds no level. Add the building's levels first, or bring the whole model into a job of its own".into());
    }
    // The model's levels at a height the job does not have, by their number in the file: its name, what it says its floor is at, and where it is placed.
    let mut loose: HashMap<u64, (String, f64, f64)> = HashMap::new();
    let lifted: Vec<f64> = storeys.iter().map(|s| s.3 - s.2).filter(|d| d.abs() > 0.001).collect();
    if !lifted.is_empty() {
        out.notes.push(format!(
            "{} of the model's levels {} placed {} from where {} its floor is (the building, or the site, is placed at a height): each floor's height is what the level says, and what stands on a level is measured from where the level is placed",
            lifted.len(),
            if lifted.len() == 1 { "is" } else { "are" },
            feet(lifted[0]),
            if lifted.len() == 1 { "it says" } else { "each says" }
        ));
    }
    // The storeys the job already has, by name: another trade's model of the same
    // building stands on the same floors.
    let mut have: Vec<(String, String, Option<f64>)> = {
        let mut q = job.world.query::<(&plumbline_world::Node, &building::Level)>();
        q.iter(&job.world).map(|(n, l)| (n.name.clone(), n.path.clone(), l.elevation.feet())).collect()
    };
    let heights = storey_heights(r);
    let mut storey_path: HashMap<u64, (String, f64)> = HashMap::new();
    let mut shared: Vec<String> = Vec::new();
    // The levels the job had before this model: a level of the model at the height of one of them is that level, whatever
    // the two are called (call 206 point 3).
    let had: Vec<(String, String, Option<f64>)> = have.clone();
    let mut by_height: Vec<String> = Vec::new();
    for (i, (id, called, elevation, placed)) in storeys.iter().enumerate() {
        let source = from(&r.file.entities[id], *id);
        let entered = Entered { status: how.status, source: &source };
        let mut called = if called.is_empty() { format!("level at {}", feet(*elevation)) } else { called.clone() };
        // A level the job had at this height is that level, under the job's name, within an eighth of an inch.
        let same_height = |(_, _, at): &&(String, String, Option<f64>)| at.is_some_and(|a| (a - elevation).abs() <= SAME_HEIGHT_FT);
        if let Some((jname, path, _)) = had.iter().filter(same_height).find(|(n, _, _)| *n == called).or_else(|| had.iter().find(same_height)) {
            if *jname == called {
                shared.push(called.clone());
            } else {
                let theirs = heights.get(id).map(|h| h * k);
                let ours = job.find(path).and_then(|e| job.world.get::<building::Level>(e)).and_then(|l| l.height.feet());
                let differ = match (theirs, ours) {
                    (Some(t), Some(o)) if (t - o).abs() > SAME_HEIGHT_FT => format!("; the model gives it as {} high and the job has it {} high, and the job's stays", feet(t), feet(o)),
                    _ => String::new(),
                };
                by_height.push(format!("the model's level \"{called}\" has its floor at {}, the height of the job's \"{jname}\": it is that level, and what the model has on it was put on the job's own{differ}", feet(*elevation)));
            }
            storey_path.insert(*id, (path.clone(), *placed));
            continue;
        }
        // Only its runs: a level at a height the job has not is not added; what stands on it stands on the job's level it is within.
        if only_runs {
            loose.insert(*id, (called.clone(), *elevation, *placed));
            continue;
        }
        // A storey of this name already in the job, at this height: it is that storey.
        // At another height the two cannot both be right, and the model is refused.
        if let Some((_, path, at)) = have.iter().find(|(n, _, _)| *n == called).filter(|_| !storey_path.values().any(|(p, _)| have.iter().any(|(n, hp, _)| *n == called && hp == p))) {
            match at {
                Some(at) if (at - elevation).abs() < 0.01 => {
                    shared.push(called.clone());
                    storey_path.insert(*id, (path.clone(), *placed));
                    continue;
                }
                _ => {
                    return Err(format!(
                        "the job already has a level called \"{called}\" with its floor at {}, and the model's level of that name has its floor at {}: they cannot both be right. Give one of them another name, or set the job's level to the model's height, and bring the model in again",
                        at.map(feet).unwrap_or_else(|| "a height nobody has entered".to_string()),
                        feet(*elevation)
                    ));
                }
            }
        }
        // Two storeys of one name in the model (two buildings, each with a first floor):
        // the later says which it is by its height.
        if have.iter().any(|(n, _, _)| *n == called) {
            let renamed = format!("{called}, at {}", feet(*elevation));
            out.notes.push(format!("the model has more than one level called \"{called}\": the one with its floor at {} was brought in as \"{renamed}\"", feet(*elevation)));
            called = renamed;
        }
        // The next level whose floor is higher than this one's: two levels at one height
        // (a mezzanine named apart, a model with a level drawn twice) both take their
        // height from the level above them, where there is one (R21-30: the lower of
        // the two was told "the model has no level above this one", which was untrue).
        let above = storeys[i + 1..].iter().find(|next| next.2 - elevation > 0.01);
        let path = match (heights.get(id), above) {
            // The height the model gives the level, where it gives one (a model this program wrote does).
            (Some(h), _) => ed::storey(job, &called, &feet(*elevation), &format!("{} ft", (h * k * 1.0e6).round() / 1.0e6 + 0.0), entered)?,
            (None, Some(next)) => {
                out.notes.push(format!("the model does not give the height of \"{called}\": it was taken as the distance to the floor of the level above, {}, which is the height and the floor's depth", feet(next.2 - elevation)));
                ed::storey(job, &called, &feet(*elevation), &feet(next.2 - elevation), entered)?
            }
            (None, None) => {
                // The top storey: how tall it is is in nothing the file says.
                let path = ed::storey(job, &called, &feet(*elevation), "1 ft", entered)?;
                let e = job.find(&path).ok_or("the new level is not there")?;
                let mut level = job.world.get::<building::Level>(e).cloned().ok_or("the new level has no level")?;
                level.height = Val::unknown("ft", "the model has no level above this one to say how tall it is");
                job.put(&path, level)?;
                path
            }
        };
        if !had.is_empty() {
            out.notes.push(format!("the model's level \"{called}\" has its floor at {}, which is not the height of any level the job has: it was added as a new level", feet(*elevation)));
        }
        have.push((called, path.clone(), Some(*elevation)));
        storey_path.insert(*id, (path, *placed));
        made("levels");
    }
    out.notes.extend(by_height);
    if !shared.is_empty() {
        out.notes.push(format!("the job already has {}: what the model has on {} was put on the job's own, and no second level was made", shared.iter().map(|n| format!("\"{n}\"")).collect::<Vec<_>>().join(", "), if shared.len() == 1 { "it" } else { "them" }));
    }
    // ---- What stands on which storey.
    let mut on: HashMap<u64, u64> = HashMap::new();
    for (_, rel) in of(&["IFCRELCONTAINEDINSPATIALSTRUCTURE"]) {
        if let V::Ref(storey) = r.arg(rel, 5) {
            for thing in Reader::list(r.arg(rel, 4)) {
                if let V::Ref(id) = thing {
                    on.insert(*id, *storey);
                }
            }
        }
    }
    for (_, rel) in of(&["IFCRELAGGREGATES"]) {
        if let V::Ref(whole) = r.arg(rel, 4) {
            for part in Reader::list(r.arg(rel, 5)) {
                if let V::Ref(id) = part {
                    on.entry(*id).or_insert(*whole);
                }
            }
        }
    }
    // The storey a thing is on: the one the file puts it on, or the highest whose floor
    // is not above where the thing is.
    let mut guessed = 0usize;
    // Where a thing was put on a level of the job that the model's own level is not at: the thing, the model's level, the job's level, its
    // height above the datum and how far above the job's floor it is (call 210 point 2).
    let mut put_on: Vec<(u64, u64, String, f64, f64)> = Vec::new();
    let mut storey_of = |id: u64, z: f64| -> (String, f64) {
        let given = on.get(&id).copied().filter(|s| storey_path.contains_key(s) || loose.contains_key(s));
        let sid = match given {
            Some(s) => s,
            None => {
                guessed += 1;
                storeys.iter().rev().find(|s| s.3 <= z + 0.01).unwrap_or(&storeys[0]).0
            }
        };
        if let Some((_, stated, placed)) = loose.get(&sid) {
            // Its height above the datum: the level's floor as it says, and what stands on it measured from where the level is placed. The
            // job's highest level whose floor is not above that, or its lowest where it is below them all.
            let height = stated + z - placed;
            let (_, path, floor) = job_levels.iter().rev().find(|l| l.2 <= height + 0.01).or_else(|| job_levels.first()).cloned().unwrap_or_default();
            put_on.push((id, sid, path.clone(), height, floor));
            return (path, floor + (placed - stated));
        }
        storey_path[&sid].clone()
    };
    // The things that were made, by their number in the file, for what is said of where they stand.
    let mut made_ids: HashSet<u64> = HashSet::new();

    // ---- Walls.
    let mut wall_of: HashMap<u64, (String, P3, P3, f64)> = HashMap::new();
    let mut wall_marks: Vec<(String, u64, String)> = Vec::new();
    let mut told_apart: Vec<String> = Vec::new();
    for (id, e) in of(&["IFCWALL", "IFCWALLSTANDARDCASE", "IFCWALLELEMENTEDCASE"]) {
        if only_runs {
            *na.entry("walls").or_insert(0) += 1;
            continue;
        }
        let body = match r.upright(e) {
            Ok(body) => body,
            Err(why) => {
                leave(&e.name, why);
                continue;
            }
        };
        let (a, b, length, way, thickness, base) = (body.a, body.b, body.length, body.way, body.thickness, body.base);
        let (storey, floor) = storey_of(id, base * k);
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let made_wall = (|| -> Result<String, String> {
            let size = format!("{} in", tidy(thickness * k * 12.0));
            let wall_type = match said_of.get(&id).and_then(|it| it.type_name.as_deref()) {
                Some(called) => ed::wall_type_named(job, called, &size, entered)?,
                None => ed::wall_type_for(job, &size, entered)?,
            };
            if let Some(called) = said_of.get(&id).and_then(|it| it.type_name.as_deref()) {
                let got = job.find(&wall_type).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone());
                if let Some(got) = got.filter(|g| g != called) {
                    let said = format!("\"{called}\" as \"{got}\"");
                    if !told_apart.contains(&said) {
                        told_apart.push(said);
                    }
                }
            }
            let made = ed::walls(job, &storey, &wall_type, &[[tidy(a[0] * k), tidy(a[1] * k)], [tidy(b[0] * k), tidy(b[1] * k)]], false, ed::Top::Height(&feet(body.depth * k)), entered)?;
            let path = made.into_iter().next().ok_or("no wall was made")?;
            // (A wall's mark is put on after its doors and windows are in: a door is not set in a wall that is
            // coming down, and the model has the door in that wall: call 179, C6.)
            wall_marks.push((path.clone(), id, source.clone()));
            // The model's own name for the wall, where it has one nothing else on the
            // storey goes by.
            let called = r.name(e);
            if !called.is_empty() && !job.children().get(&storey).is_some_and(|c| c.contains_key(&called)) {
                let w = job.find(&path).ok_or("the new wall is not there")?;
                if let Some(mut node) = job.world.get_mut::<plumbline_world::Node>(w) {
                    node.name = called;
                }
            }
            if (base * k - floor).abs() > 0.001 {
                let w = job.find(&path).ok_or("the new wall is not there")?;
                let mut wall = job.world.get::<Wall>(w).cloned().ok_or("the new wall has no line")?;
                wall.base_offset = Some(val(&source, base * k - floor));
                job.put(&path, wall)?;
            }
            Ok(path)
        })();
        match made_wall {
            Ok(path) => {
                wall_of.insert(id, (path, [a[0], a[1], base], [way[0], way[1], 0.0], length));
                made("walls");
            }
            Err(why) => leave(&e.name, &why),
        }
    }
    if !told_apart.is_empty() {
        out.notes.push(format!("the model names a wall type the job already has at another thickness: {} came in with a number after the name, so that no wall was left out for its type's name", told_apart.join(", ")));
    }
    // ---- Doors and windows: each fills a hole, and each hole is in a wall.
    let mut hole_in: HashMap<u64, u64> = HashMap::new();
    for (_, rel) in of(&["IFCRELVOIDSELEMENT"]) {
        if let (V::Ref(wall), V::Ref(hole)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            hole_in.insert(*hole, *wall);
        }
    }
    let mut fills: HashMap<u64, u64> = HashMap::new();
    for (_, rel) in of(&["IFCRELFILLSELEMENT"]) {
        if let (V::Ref(hole), V::Ref(filling)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            fills.insert(*filling, *hole);
        }
    }
    for (id, e) in of(&["IFCDOOR", "IFCDOORSTANDARDCASE", "IFCWINDOW", "IFCWINDOWSTANDARDCASE"]) {
        let kind = if e.name.starts_with("IFCDOOR") { Kind::Door } else { Kind::Window };
        if only_runs {
            *na.entry(if kind == Kind::Door { "doors" } else { "windows" }).or_insert(0) += 1;
            continue;
        }
        let Some((wall, start, way, _)) = fills.get(&id).and_then(|hole| hole_in.get(hole)).and_then(|w| wall_of.get(w)) else {
            leave(&e.name, "it fills no hole in a wall that was brought in");
            continue;
        };
        let (Some(height), Some(width)) = (Reader::number(r.arg(e, 8)), Reader::number(r.arg(e, 9))) else {
            leave(&e.name, "the model does not give its width and its height");
            continue;
        };
        // It is placed by a lower corner, its own x along its width.
        let frame = r.placed(r.arg(e, 5));
        let along = dot([frame.o[0] - start[0], frame.o[1] - start[1], 0.0], *way) + dot(frame.x, *way) * width / 2.0;
        let sill = frame.o[2] - start[2];
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let size = |x: f64| format!("{} in", tidy(x * k * 12.0));
        let type_for = match said_of.get(&id).and_then(|it| it.type_name.as_deref()) {
            Some(called) => ed::opening_type_named(job, kind, called, &size(width), &size(height), entered),
            None => ed::opening_type_for(job, kind, &size(width), &size(height), entered),
        };
        let done = type_for.and_then(|t| ed::opening(job, wall, kind, &t, ed::OnWall::Offset(&feet(along * k)), &feet(sill * k), entered)).and_then(|path| marks(job, &path, id, &source, kind).map(|()| path));
        match done {
            Ok(path) => {
                keep_name(job, &path, &r.name(e), id);
                made(if kind == Kind::Door { "doors" } else { "windows" })
            }
            Err(why) => leave(&e.name, &why),
        }
    }
    for (path, id, source) in &wall_marks {
        if let Err(why) = marks(job, path, *id, source, Kind::Wall) {
            out.notes.push(format!("a mark on {path} was not put on it: {why}"));
        }
    }
    for (id, e) in of(&["IFCOPENINGELEMENT", "IFCOPENINGSTANDARDCASE"]) {
        if !fills.values().any(|hole| *hole == id) {
            if only_runs {
                *na.entry("holes in walls").or_insert(0) += 1;
                continue;
            }
            leave(&e.name, "a hole with no door or window in it: this program cuts a wall only for one of those, and a floor for nothing yet");
        }
    }
    // ---- The program's own roof (call 197, point 4). A roof this program wrote carries a list of
    // what is needed to make it again, `Plumbline_Roof`: its outline round the eaves, what each edge
    // does (a pitch, or a gable) and how high the eaves are. It is made by the change the Roof tool
    // makes (`solve::new_roof`), from that, Assumed and sourced to the file. A roof with no such list
    // is a roof from another program, and is left out as it was.
    let mut stood_on: Vec<(String, [f64; 3], [f64; 3])> = Vec::new();
    let mut done: HashSet<u64> = HashSet::new();
    // Where each run brought in lies, in feet and as the file places it: its ends.
    let mut run_ends: Vec<(String, P3, P3)> = Vec::new();
    // The same, by the run's number in the file, for what the file's ports say is joined to it.
    let mut run_by_id: HashMap<u64, (String, P3, P3)> = HashMap::new();
    // ---- Ports (call 200): what the file says is connected. A port is on what it is nested in
    // (`IfcRelNests`, in order; IFC2x3 and older files tie a port to its element by
    // `IfcRelConnectsPortToElement`), and two ports are tied by `IfcRelConnectsPorts`. A connection point is never a
    // thing of its own here.
    let port_ids: HashSet<u64> = of(&["IFCDISTRIBUTIONPORT", "IFCPORT"]).into_iter().map(|(id, _)| id).collect();
    let mut owner_of: HashMap<u64, u64> = HashMap::new();
    let mut ports_of: HashMap<u64, Vec<u64>> = HashMap::new();
    for (_, rel) in of(&["IFCRELNESTS"]) {
        let V::Ref(whole) = r.arg(rel, 4) else { continue };
        for part in Reader::list(r.arg(rel, 5)) {
            if let V::Ref(p) = part {
                if port_ids.contains(p) {
                    owner_of.insert(*p, *whole);
                    ports_of.entry(*whole).or_default().push(*p);
                }
            }
        }
    }
    for (_, rel) in of(&["IFCRELCONNECTSPORTTOELEMENT"]) {
        if let (V::Ref(p), V::Ref(whole)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            if port_ids.contains(p) && !owner_of.contains_key(p) {
                owner_of.insert(*p, *whole);
                ports_of.entry(*whole).or_default().push(*p);
            }
        }
    }
    let mut tied_to: HashMap<u64, Vec<u64>> = HashMap::new();
    for (_, rel) in of(&["IFCRELCONNECTSPORTS"]) {
        if let (V::Ref(a), V::Ref(b)) = (r.arg(rel, 4), r.arg(rel, 5)) {
            tied_to.entry(*a).or_default().push(*b);
            tied_to.entry(*b).or_default().push(*a);
        }
    }
    let file_has_ports = !port_ids.is_empty();
    for id in &port_ids {
        done.insert(*id);
    }
    let mut runs_unsized = 0usize;
    let mut roof_slabs: HashSet<u64> = HashSet::new();
    {
        let mut sets: HashMap<u64, &Entity> = HashMap::new();
        for (_, rel) in of(&["IFCRELDEFINESBYPROPERTIES"]) {
            let Some((_, set)) = r.get(r.arg(rel, 5)) else { continue };
            if set.name == "IFCPROPERTYSET" && Reader::text(r.arg(set, 2)) == Some("Plumbline_Roof") {
                for obj in Reader::list(r.arg(rel, 4)) {
                    if let V::Ref(id) = obj {
                        sets.insert(*id, set);
                    }
                }
            }
        }
        let mut parts_of: HashMap<u64, Vec<u64>> = HashMap::new();
        for (_, rel) in of(&["IFCRELAGGREGATES"]) {
            if let V::Ref(whole) = r.arg(rel, 4) {
                for part in Reader::list(r.arg(rel, 5)) {
                    if let V::Ref(id) = part {
                        parts_of.entry(*whole).or_default().push(*id);
                    }
                }
            }
        }
        for (id, e) in of(&["IFCROOF"]) {
            if only_runs {
                // A roof is not a run: it is counted, with the slabs that are its parts, and not read.
                *na.entry("roofs").or_insert(0) += 1;
                done.insert(id);
                roof_slabs.extend(parts_of.get(&id).cloned().unwrap_or_default());
                continue;
            }
            let Some(set) = sets.get(&id) else { continue };
            let prop = |name: &str| -> Option<&Entity> { Reader::list(r.arg(set, 4)).iter().filter_map(|p| r.get(p)).map(|(_, p)| p).find(|p| Reader::text(r.arg(p, 0)) == Some(name)) };
            let parsed = (|| -> Result<(Vec<[f64; 2]>, Vec<Option<f64>>, f64), String> {
                let outline = prop("Outline").ok_or("it has no outline")?;
                let numbers: Vec<f64> = Reader::list(r.arg(outline, 2)).iter().filter_map(Reader::number).collect();
                if numbers.len() < 6 || numbers.len() % 2 != 0 {
                    return Err("its outline is not three or more corners".into());
                }
                let corners: Vec<[f64; 2]> = numbers.chunks(2).map(|c| [tidy(c[0] * k), tidy(c[1] * k)]).collect();
                let rises = prop("EdgeRise").ok_or("it does not say what each edge does")?;
                let edges = Reader::list(r.arg(rises, 2))
                    .iter()
                    .map(|v| {
                        let said = Reader::text(v).ok_or("an edge's rise is not written as a pitch or as a gable")?.trim().to_string();
                        if said.eq_ignore_ascii_case("gable") {
                            return Ok(None);
                        }
                        said.strip_suffix(":12").and_then(|n| n.trim().parse::<f64>().ok()).filter(|n| n.is_finite()).map(Some).ok_or(format!("\"{said}\" is not a pitch"))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                if edges.len() != corners.len() {
                    return Err(format!("it has {} corners and {} edges", corners.len(), edges.len()));
                }
                let height = prop("EaveHeight").and_then(|p| Reader::number(r.arg(p, 2))).ok_or("it does not say how high its eaves are")?;
                Ok((corners, edges, tidy(height * k)))
            })();
            let source = from(e, id);
            let made_roof = parsed.and_then(|(mut corners, mut edges, height)| {
                // Drawn anticlockwise, as the tool turns what is picked the other way about.
                let twice: f64 = (0..corners.len()).map(|i| corners[i][0] * corners[(i + 1) % corners.len()][1] - corners[(i + 1) % corners.len()][0] * corners[i][1]).sum();
                if twice < 0.0 {
                    let n = corners.len();
                    corners.reverse();
                    edges = (0..n).map(|j| edges[(2 * n - 2 - j) % n]).collect();
                }
                let sure = |x| Val { value: Some(x), unit: units::PITCH.to_string(), status: how.status, source: source.clone() };
                let footprint = Footprint {
                    edges: edges.iter().map(|x| match x { Some(p) => sure(Rise::Pitch(*p)), None => Val { value: Some(Rise::Gable), unit: String::new(), status: how.status, source: source.clone() } }).collect(),
                    points: Val { value: Some(corners), unit: units::FT.to_string(), status: how.status, source: source.clone() },
                    eave_height: Val { value: Some(Len::One(height)), unit: units::FT.to_string(), status: how.status, source: source.clone() },
                };
                // A roof with this very outline and these edges already in the job would be counted twice.
                let there = {
                    let mut q = job.world.query::<&Footprint>();
                    q.iter(&job.world).any(|f| f.points_ft().is_some_and(|p| p.len() == footprint.points.value.as_ref().map_or(0, |c| c.len()) && p.iter().zip(footprint.points.value.iter().flatten()).all(|(a, b)| (a[0] - b[0]).abs() < 0.001 && (a[1] - b[1]).abs() < 0.001)) && f.edges.iter().map(|e| e.value).eq(footprint.edges.iter().map(|e| e.value)))
                };
                if there {
                    return Err("a roof with this very outline and these edges is already in the job: a second would be counted twice".to_string());
                }
                let taken: Vec<String> = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &Kind)>();
                    q.iter(&job.world).filter(|(_, kind)| **kind == Kind::Roof).map(|(n, _)| n.name.clone()).collect()
                };
                let called = r.name(e);
                let called = if called.is_empty() || taken.contains(&called) { (1..).map(|n| if n == 1 { "roof".to_string() } else { format!("roof {n}") }).find(|n| !taken.contains(n)).unwrap_or_default() } else { called };
                solve::new_roof(job, &called, footprint)
            });
            let slabs = parts_of.get(&id).cloned().unwrap_or_default();
            match made_roof {
                Ok((path, report)) => {
                    done.insert(id);
                    roof_slabs.extend(slabs.iter().copied());
                    if let Err(why) = marks(job, &path, id, &source, Kind::Roof) {
                        out.notes.push(format!("a mark on {path} was not put on it: {why}"));
                    }
                    made("roofs");
                    for _ in 0..report.planes {
                        made("roof planes");
                    }
                    // The planes made are set beside the roof slabs in the file.
                    job.derive();
                    let mine: Vec<f64> = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &plumbline_world::Measure)>();
                        q.iter(&job.world).filter(|(n, kind, _)| **kind == Kind::RoofPlane && n.parent.as_deref() == Some(path.as_str())).filter_map(|(_, _, m)| m.0.get("surface_area").and_then(|d| d.value)).collect()
                    };
                    let theirs: Vec<f64> = slabs.iter().filter_map(|s| r.file.entities.get(s)).filter_map(|s| r.face_area(s)).map(|a| a * k * k).collect();
                    let (mut a, mut b) = (mine.clone(), theirs.clone());
                    a.sort_by(f64::total_cmp);
                    b.sort_by(f64::total_cmp);
                    let agree = a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 0.01);
                    if !agree {
                        let total = |v: &[f64]| (v.iter().sum::<f64>() * 100.0).round() / 100.0;
                        out.notes.push(format!(
                            "the roof's planes made here are {} and come to {} SF, and the roof slabs in the file are {} and come to {} SF: they differ, so the roof here is not the roof in the file (made from the roof's outline, its pitches and its eave height, which the file gives)",
                            a.len(),
                            total(&a),
                            b.len(),
                            total(&b)
                        ));
                    }
                }
                Err(why) => {
                    done.insert(id);
                    // Its slabs are part of it: the roof says why it was left out, and they are not left out again for a reason
                    // that is not so (they have the outline).
                    roof_slabs.extend(slabs.iter().copied());
                    if let Some(reach) = r.reach(e) {
                        stood_on.push((said(&e.name), [reach.0[0] * k, reach.0[1] * k, reach.0[2] * k], [reach.1[0] * k, reach.1[1] * k, reach.1[2] * k]));
                    }
                    leave(&e.name, &why);
                }
            }
        }
    }
    // ---- Floor slabs.
    let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
    let mut floor_types: HashMap<i64, String> = HashMap::new();
    let slab_ids: Vec<u64> = of(&["IFCSLAB", "IFCSLABSTANDARDCASE", "IFCSLABELEMENTEDCASE"]).into_iter().map(|(id, _)| id).collect();
    let cut_floors: Vec<u64> = hole_in.values().copied().filter(|host| slab_ids.contains(host)).collect();
    let mut floors_cut = 0usize;
    for (id, e) in of(&["IFCSLAB", "IFCSLABSTANDARDCASE", "IFCSLABELEMENTEDCASE"]) {
        // A slab the model says is a roof is a roof, not a floor (J17B-18).
        if matches!(r.arg(e, 8), V::Enum(k) if k == "ROOF") {
            if roof_slabs.contains(&id) {
                continue;
            }
            if only_runs {
                *na.entry("roofs").or_insert(0) += 1;
                continue;
            }
            leave("IFCROOFSLAB", NO_ROOF_OUTLINE);
            continue;
        }
        if only_runs {
            *na.entry("floors").or_insert(0) += 1;
            continue;
        }
        if r.pieces(e) > 1 {
            leave(&e.name, "its body is made of more than one piece: only a body of one piece is read");
            continue;
        }
        if r.holed(e) {
            leave(&e.name, "its outline has a hole in it: this program has no floor with a hole yet, and would count the hole as floor");
            continue;
        }
        let Some(body) = r.pushed(e) else {
            leave(&e.name, "its body could not be read as an outline pushed straight: it may slope, be a mesh, or name a part that is not in the file");
            continue;
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let way = frame.turn(body.way);
        if way[2].abs() < UPRIGHT || frame.z[2].abs() < UPRIGHT {
            leave(&e.name, "it is not level");
            continue;
        }
        let corners: Vec<[f64; 2]> = match &body.profile {
            Profile::Outline(points) => points.clone(),
            Profile::Rect { x, y, at, along } => {
                let across = [-along[1], along[0]];
                [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)].iter().map(|(u, v)| [at[0] + along[0] * x * u + across[0] * y * v, at[1] + along[1] * x * u + across[1] * y * v]).collect()
            }
            Profile::Circle { .. } => {
                leave(&e.name, "it is round");
                continue;
            }
        };
        let boundary: Vec<[f64; 2]> = corners.iter().map(|p| frame.place([p[0], p[1], 0.0])).map(|p| [tidy(p[0] * k), tidy(p[1] * k)]).collect();
        let top = if way[2] > 0.0 { frame.o[2] + body.depth } else { frame.o[2] };
        let (storey, floor) = storey_of(id, top * k);
        let source = from(e, id);
        let thickness = tidy(body.depth * k * 12.0);
        let done = (|| -> Result<(), String> {
            let key = (thickness * 1.0e4).round() as i64;
            let floor_type = match floor_types.get(&key) {
                Some(path) => path.clone(),
                None => {
                    // A floor type of this name and thickness the job already has (from
                    // a model brought in before) is this one.
                    let named = format!("floor {thickness} in");
                    let has = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &FloorType)>();
                        q.iter(&job.world).find(|(n, t)| n.name == named && t.thickness.feet().is_some_and(|f| (f * 12.0 - thickness).abs() < 1.0e-6)).map(|(n, _)| n.path.clone())
                    };
                    let path = match has {
                        Some(path) => path,
                        None => {
                            let path = job.add(&root, Kind::Type, &named)?;
                            let t = FloorType { thickness: Val { value: Some(Len::One(thickness)), unit: "in".into(), status: how.status, source: source.clone() }, layers: Vec::new() };
                            building::check_floor_type(&path, Kind::Type, &t)?;
                            job.put(&path, t)?;
                            path
                        }
                    };
                    floor_types.insert(key, path.clone());
                    path
                }
            };
            // A floor with this very outline already on the storey (the model brought in
            // a second time, or two trades' models that both carry the slab) would be
            // counted twice: it is not made again.
            let there = {
                let mut q = job.world.query::<(&plumbline_world::Node, &Floor)>();
                q.iter(&job.world).any(|(n, f)| n.parent.as_deref() == Some(storey.as_str()) && f.boundary_ft().is_some_and(|b| b.len() == boundary.len() && b.iter().zip(&boundary).all(|(p, q)| (p[0] - q[0]).abs() < 0.001 && (p[1] - q[1]).abs() < 0.001)))
            };
            if there {
                return Err("a floor with this very outline is already on this level: a second would be counted twice".into());
            }
            let called = free_name(job, &storey, &r.name(e), "floor", id);
            let path = job.add(&storey, Kind::Floor, &called)?;
            let offset = top * k - floor;
            // A floor the model cuts an opening in (a stair, a shaft): the opening is
            // not cut here, so its outline is more than the floor. That is said in the
            // outline's source, and the outline is no surer than Assumed.
            let cut = cut_floors.contains(&id);
            let outline = if cut { Val { value: Some(boundary.clone()), unit: "ft".into(), status: how.status.max(plumbline_world::Status::Assumed), source: format!("{source}; the model cuts an opening in this floor that is not cut here, so this outline is more than the floor") } } else { Val { value: Some(boundary.clone()), unit: "ft".into(), status: how.status, source: source.clone() } };
            let f = Floor { type_name: path_val(&source, &floor_type), boundary: outline, offset: (offset.abs() > 0.001).then(|| val(&source, offset)) };
            building::check_floor(&path, Kind::Floor, &f)?;
            job.put(&path, f)
        })();
        match done {
            Ok(()) => {
                made("floors");
                if cut_floors.contains(&id) {
                    floors_cut += 1;
                }
            }
            Err(why) => leave(&e.name, &why),
        }
    }
    if floors_cut > 0 {
        out.notes.push(format!("{floors_cut} floor{} an opening cut in {} in the model (a stair, a shaft): the opening is not cut here, so {} area is more than the floor. Each is flagged Assumed and says so", if floors_cut == 1 { " has" } else { "s have" }, if floors_cut == 1 { "it" } else { "them" }, if floors_cut == 1 { "its" } else { "each one's" }));
    }
    // ---- Spaces, as rooms: a point inside each.
    for (id, e) in of(&["IFCSPACE"]) {
        if only_runs {
            *na.entry("rooms").or_insert(0) += 1;
            continue;
        }
        let Some(body) = r.pushed(e) else {
            leave(&e.name, "its shape is not an outline pushed straight, so no point inside it could be found");
            continue;
        };
        let frame = r.placed(r.arg(e, 5)).then(&body.frame);
        let middle = match &body.profile {
            Profile::Outline(points) => point_inside(points),
            Profile::Rect { at, .. } | Profile::Circle { at, .. } => *at,
        };
        let at = frame.place([middle[0], middle[1], 0.0]);
        let (storey, _) = storey_of(id, at[2] * k);
        let source = from(e, id);
        let (number, long) = (r.name(e), Reader::text(r.arg(e, 7)).unwrap_or("").trim().to_string());
        let called = match (long.is_empty(), number.is_empty()) {
            (false, false) => format!("{long} {number}"),
            (false, true) => long,
            (true, false) => format!("room {number}"),
            (true, true) => format!("room {id}"),
        };
        let done = (|| -> Result<(), String> {
            let path = job.add(&storey, Kind::Room, &called)?;
            let room = Room { point: Val { value: Some([tidy(at[0] * k), tidy(at[1] * k)]), unit: "ft".into(), status: how.status, source: source.clone() }, number: None };
            building::check_room(&path, Kind::Room, &room)?;
            job.put(&path, room)
        })();
        match done {
            Ok(()) => made("rooms"),
            Err(why) => leave(&e.name, &why),
        }
    }
    // ---- Systems the file names, where they are ones this program has.
    let mut system_of: HashMap<u64, String> = HashMap::new();
    let mut systems_not_known = 0usize;
    for (_, rel) in of(&["IFCRELASSIGNSTOGROUP"]) {
        let Some((_, group)) = r.get(r.arg(rel, 6)) else { continue };
        if !matches!(group.name.as_str(), "IFCDISTRIBUTIONSYSTEM" | "IFCSYSTEM" | "IFCDISTRIBUTIONCIRCUIT") {
            continue;
        }
        // A kind IFC has no word for is written by its name (the object type and the long name): a
        // name this program has for a system is that system.
        let by_name = || -> Option<&'static str> {
            [r.arg(group, 4), r.arg(group, 5)].into_iter().filter_map(Reader::text).find_map(|t| building::CLASSIFICATIONS.iter().find(|(c, _)| c.eq_ignore_ascii_case(t.trim())).map(|(c, _)| *c))
        };
        let class = match r.arg(group, 6) {
            V::Enum(t) => match t.as_str() {
                "USERDEFINED" => by_name(),
                "AIRCONDITIONING" | "VENTILATION" | "HEATING" => Some("Supply Air"),
                "EXHAUST" => Some("Exhaust Air"),
                "DOMESTICCOLDWATER" => Some("Domestic Cold Water"),
                "DOMESTICHOTWATER" => Some("Domestic Hot Water"),
                "SEWAGE" | "WASTEWATER" | "DRAINAGE" => Some("Sanitary"),
                "VENT" => Some("Vent"),
                "REFRIGERATION" => Some("Refrigerant"),
                "ELECTRICAL" => Some("Power"),
                "LIGHTING" => Some("Lighting"),
                _ => None,
            },
            _ => None,
        };
        let Some(class) = class else {
            systems_not_known += 1;
            continue;
        };
        // The job's own system of that kind where it has one; else a new one called what the file
        // calls it, with its capitals (call 197, point 5).
        let have: Option<String> = {
            let mut q = job.world.query::<(&plumbline_world::Node, &building::SystemInfo)>();
            let mut all: Vec<String> = q.iter(&job.world).filter(|(_, s)| s.classification.eq_ignore_ascii_case(class)).map(|(n, _)| n.path.clone()).collect();
            all.sort();
            all.into_iter().next()
        };
        let path = match have {
            Some(path) => path,
            None => {
                let called = r.name(group);
                let taken = job.children().get(&root).is_some_and(|c| c.contains_key(called.as_str()));
                if called.is_empty() || taken { ed::system_of_class(job, class)? } else { ed::system(job, &called, class)? }
            }
        };
        for thing in Reader::list(r.arg(rel, 4)) {
            if let V::Ref(id) = thing {
                system_of.insert(*id, path.clone());
            }
        }
    }
    // ---- Ducts, pipes, cable trays and cables.
    let kind_of_type: HashMap<u64, Kind> = {
        let mut found = HashMap::new();
        for (_, rel) in of(&["IFCRELDEFINESBYTYPE"]) {
            let kind = r.get(r.arg(rel, 5)).and_then(|(_, t)| match t.name.as_str() {
                "IFCDUCTSEGMENTTYPE" => Some(Kind::Duct),
                "IFCPIPESEGMENTTYPE" => Some(Kind::Pipe),
                "IFCCABLECARRIERSEGMENTTYPE" => Some(Kind::Conduit),
                "IFCCABLESEGMENTTYPE" => Some(Kind::Cable),
                _ => None,
            });
            if let Some(kind) = kind {
                for thing in Reader::list(r.arg(rel, 4)) {
                    if let V::Ref(id) = thing {
                        found.insert(*id, kind);
                    }
                }
            }
        }
        found
    };
    for (id, e) in of(&["IFCDUCTSEGMENT", "IFCPIPESEGMENT", "IFCCABLECARRIERSEGMENT", "IFCCABLESEGMENT", "IFCFLOWSEGMENT"]) {
        let kind = match e.name.as_str() {
            "IFCDUCTSEGMENT" => Some(Kind::Duct),
            "IFCPIPESEGMENT" => Some(Kind::Pipe),
            "IFCCABLECARRIERSEGMENT" => Some(Kind::Conduit),
            "IFCCABLESEGMENT" => Some(Kind::Cable),
            _ => kind_of_type.get(&id).copied(),
        };
        let Some(kind) = kind else {
            leave(&e.name, "the model does not say whether it is a duct, a pipe, a tray or a cable");
            continue;
        };
        if r.pieces(e) > 1 {
            leave(&e.name, "its body is made of more than one piece: only a body of one piece is read");
            continue;
        }
        // A run with no size is drawn as its line alone: it comes in as the run it is, its size Unknown (call 200).
        let line_only = if r.pushed(e).is_none() && r.pieces(e) == 0 { r.line_only(e) } else { None };
        let (start, end, across): (P3, P3, Option<(f64, Option<f64>)>) = if let Some((p, q)) = line_only {
            (p, q, None)
        } else {
            let Some(body) = r.pushed(e) else {
                leave(&e.name, "its body is not a circle or a rectangle pushed along its run");
                continue;
            };
            let frame = r.placed(r.arg(e, 5)).then(&body.frame);
            let way = frame.turn(body.way);
            let (start, across): (P3, (f64, Option<f64>)) = match &body.profile {
                Profile::Circle { radius, at } => (frame.place([at[0], at[1], 0.0]), (radius * 2.0, None)),
                Profile::Rect { x, y, at, along } => {
                    // Its width is whichever side lies level; its depth the other.
                    let level = frame.turn([along[0], along[1], 0.0])[2].abs() < 0.5;
                    (frame.place([at[0], at[1], 0.0]), if level { (*x, Some(*y)) } else { (*y, Some(*x)) })
                }
                Profile::Outline(_) => {
                    leave(&e.name, "its body is not a circle or a rectangle pushed along its run");
                    continue;
                }
            };
            let end = [start[0] + way[0] * body.depth, start[1] + way[1] * body.depth, start[2] + way[2] * body.depth];
            (start, end, Some(across))
        };
        let (storey, floor) = storey_of(id, start[2].min(end[2]) * k);
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let inches = |x: f64| format!("{} in", tidy(x * k * 12.0));
        let done = (|| -> Result<String, String> {
            let run_type: Option<String> = match across {
                None => None,
                Some((width, None)) => Some(ed::segment_type_for(job, kind, &inches(width), entered)?),
                Some((width, Some(depth))) => {
                    let called = format!("{} by {}", inches(width), inches(depth));
                    let known = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &building::SegmentType)>();
                        q.iter(&job.world).find(|(n, t)| n.name == called && t.category == kind).map(|(n, _)| n.path.clone())
                    };
                    Some(match known {
                        Some(path) => path,
                        None => ed::segment_type(job, &called, kind, &inches(width), Some(&inches(depth)), entered)?,
                    })
                }
            };
            let (z0, z1) = (start[2] * k - floor, end[2] * k - floor);
            let line = [[tidy(start[0] * k), tidy(start[1] * k)], [tidy(end[0] * k), tidy(end[1] * k)]];
            // A run of this kind in this very place already on the storey would be
            // counted twice: it is not made again.
            let there = {
                let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &Segment)>();
                q.iter(&job.world).any(|(n, other, seg)| {
                    n.parent.as_deref() == Some(storey.as_str()) && *other == kind && seg.line_ft().is_some_and(|l| (0..2).all(|i| (0..2).all(|j| (l[i][j] - line[i][j]).abs() < 0.001))) && seg.offsets_ft().is_some_and(|(a, b)| (a - z0).abs() < 0.001 && (b - z1).abs() < 0.001)
                })
            };
            if there {
                return Err(format!("{} in this very place is already on this level: a second would be counted twice", a_kind(kind)));
            }
            let called = free_name(job, &storey, &r.name(e), kind.stem(), id);
            let path = job.add(&storey, kind, &called)?;
            let segment = Segment {
                type_name: match &run_type {
                    Some(t) => path_val(&source, t),
                    None => Val::unknown("", "the model gives this run no size: it is drawn as its line alone"),
                },
                line: Val { value: Some(line), unit: "ft".into(), status: how.status, source: source.clone() },
                offset: val(&source, z0),
                end_offset: ((z1 - z0).abs() > 0.001).then(|| val(&source, z1)),
                justify: Default::default(),
            };
            building::check_segment(&path, kind, &segment)?;
            job.put(&path, segment)?;
            if let Some(system) = system_of.get(&id) {
                let on = InSystem(path_val(&source, system));
                building::check_in_system(&path, kind, &on)?;
                job.put(&path, on)?;
            }
            Ok(path)
        })();
        match done {
            Ok(path) => {
                made_ids.insert(id);
                if across.is_none() {
                    runs_unsized += 1;
                }
                run_by_id.insert(id, (path.clone(), [start[0] * k, start[1] * k, start[2] * k], [end[0] * k, end[1] * k, end[2] * k]));
                run_ends.push((path, [start[0] * k, start[1] * k, start[2] * k], [end[0] * k, end[1] * k, end[2] * k]));
                made(match kind {
                    Kind::Duct => "ducts",
                    Kind::Pipe => "pipes",
                    Kind::Conduit => "conduits",
                    _ => "cables",
                })
            }
            Err(why) => leave(&e.name, &why),
        }
    }
    // The file's units to feet, for what stood where a left-out thing did (call 197, point 6).
    let reach_of = |e: &Entity| -> Option<([f64; 3], [f64; 3])> { r.reach(e).map(|(lo, hi)| ([lo[0] * k, lo[1] * k, lo[2] * k], [hi[0] * k, hi[1] * k, hi[2] * k])) };
    let has_place = |e: &Entity| r.get(r.arg(e, 5)).is_some_and(|(_, p)| p.name == "IFCLOCALPLACEMENT");
    // ---- Footings (call 197, point 3): a strip footing as a footing along its line, a pad footing as a
    // pad, each by the change that makes one. A footing of another sort, or whose body is not a straight
    // box, is left out with the reason.
    for (id, e) in of(&["IFCFOOTING"]) {
        if !has_place(e) {
            continue;
        }
        done.insert(id);
        if only_runs {
            *na.entry("footings").or_insert(0) += 1;
            continue;
        }
        let sort = match r.arg(e, 8) {
            V::Enum(t) => t.as_str(),
            _ => "",
        };
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let outcome: Result<String, String> = (|| {
            let kept = said_of.get(&id).and_then(|it| it.type_name.clone());
            match sort {
                "STRIP_FOOTING" => {
                    let body = r.upright(e).map_err(|why| why.to_string())?;
                    let (storey, floor) = storey_of(id, body.base * k);
                    if ((body.base + body.depth) * k - floor).abs() > 0.001 {
                        return Err("its top is not at its level's floor: a footing here has its top at the floor of its level".to_string());
                    }
                    let width = format!("{} in", tidy(body.thickness * k * 12.0));
                    let wanted = {
                        let (len, unit) = units::parse_length(&width)?;
                        len.feet(unit)?
                    };
                    let existing: Option<String> = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::WallType)>();
                        let mut all: Vec<String> = q.iter(&job.world).filter(|(_, t)| t.footing && t.layers.is_empty() && t.thickness.feet().is_some_and(|x| (x - wanted).abs() < 1e-9)).map(|(n, _)| n.path.clone()).collect();
                        all.sort();
                        all.into_iter().next()
                    };
                    let type_path = match existing {
                        Some(path) => path,
                        None => {
                            let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
                            let wanted_name = kept.clone().unwrap_or_else(|| format!("footing {width}"));
                            let name = if job.children().get(&root).is_some_and(|c| c.contains_key(wanted_name.as_str())) { format!("{wanted_name} {id}") } else { wanted_name };
                            ed::footing_type(job, &name, &width, entered)?
                        }
                    };
                    let line = [[tidy(body.a[0] * k), tidy(body.a[1] * k)], [tidy(body.b[0] * k), tidy(body.b[1] * k)]];
                    // A footing along this very line already on the level (the model brought in a second time)
                    // would be counted twice: it is not made again.
                    let near = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).abs() < 0.001 && (p[1] - q[1]).abs() < 0.001;
                    let there = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &Wall)>();
                        q.iter(&job.world).any(|(n, kind, w)| *kind == Kind::Footing && n.parent.as_deref() == Some(storey.as_str()) && w.line_ft().is_some_and(|l| (near(l[0], line[0]) && near(l[1], line[1])) || (near(l[0], line[1]) && near(l[1], line[0]))))
                    };
                    if there {
                        return Err("it is already on this level: a footing along this very line would be counted twice".to_string());
                    }
                    let made = ed::footings(job, &storey, &type_path, &line, false, &feet(body.depth * k), entered)?;
                    made.into_iter().next().ok_or_else(|| "no footing was made".to_string())
                }
                "PAD_FOOTING" => {
                    let b = r.boxed(e).ok_or("its body is not a straight box")?;
                    if b.angle.abs() > 1.0e-6 {
                        return Err("it is turned on the plan: a pad footing here stands square to it".to_string());
                    }
                    let (storey, floor) = storey_of(id, b.bottom * k);
                    let (across, deep) = (format!("{} in", tidy(b.size[0] * k * 12.0)), format!("{} in", tidy(b.size[1] * k * 12.0)));
                    let ft = |typed: &str| -> Result<f64, String> {
                        let (len, unit) = units::parse_length(typed)?;
                        len.feet(unit)
                    };
                    let wanted = (ft(&across)?, ft(&deep)?);
                    let existing: Option<String> = {
                        let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::SegmentType)>();
                        let mut all: Vec<String> = q
                            .iter(&job.world)
                            .filter(|(_, t)| t.category == Kind::PadFooting && t.round == b.round && t.width.feet().is_some_and(|x| (x - wanted.0).abs() < 1e-9) && (b.round || t.depth.as_ref().and_then(|d| d.feet()).is_some_and(|x| (x - wanted.1).abs() < 1e-9)))
                            .map(|(n, _)| n.path.clone())
                            .collect();
                        all.sort();
                        all.into_iter().next()
                    };
                    let type_path = match existing {
                        Some(path) => path,
                        None => {
                            let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
                            let wanted_name = kept.clone().unwrap_or_else(|| if b.round { format!("pad footing {across} round") } else { format!("pad footing {across} by {deep}") });
                            let name = if job.children().get(&root).is_some_and(|c| c.contains_key(wanted_name.as_str())) { format!("{wanted_name} {id}") } else { wanted_name };
                            ed::framing_type(job, &name, Kind::PadFooting, &across, if b.round { None } else { Some(deep.as_str()) }, None, entered)?
                        }
                    };
                    ed::pad(job, &storey, &type_path, [tidy(b.middle[0] * k), tidy(b.middle[1] * k)], &format!("{} in", tidy(b.size[2] * k * 12.0)), &feet((b.bottom + b.size[2]) * k - floor), entered)
                }
                "" | "NOTDEFINED" => Err("the model does not say whether it is a strip footing or a pad footing".to_string()),
                other => Err(format!("its sort is a {}, which is neither a strip footing nor a pad footing: only those two are brought in", other.to_lowercase().replace('_', " "))),
            }
        })();
        match outcome {
            Ok(path) => {
                keep_name(job, &path, &r.name(e), id);
                if let Err(why) = marks(job, &path, id, &source, Kind::Footing) {
                    out.notes.push(format!("a mark on {path} was not put on it: {why}"));
                }
                made(if sort == "PAD_FOOTING" { "pad footings" } else { "footings" });
            }
            Err(why) => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, &why);
            }
        }
    }
    // ---- Beams and columns (call 204, point 5). A beam is a rectangle pushed level along its line, a column one pushed upright:
    // each comes in by the change that makes one (a beam along its line with its top, a post at its place, standing so tall), its
    // section as a type of its kind, sized from the body, Assumed and sourced to the file. A body that is not a straight box is
    // left out with the reason that is true of it.
    for (id, e) in of(&["IFCBEAM", "IFCCOLUMN"]) {
        if !has_place(e) {
            continue;
        }
        done.insert(id);
        let is_beam = e.name == "IFCBEAM";
        if only_runs {
            *na.entry(if is_beam { "beams" } else { "posts" }).or_insert(0) += 1;
            continue;
        }
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let outcome: Result<String, String> = (|| {
            let kept = said_of.get(&id).and_then(|it| it.type_name.clone());
            let root = job.world.get::<plumbline_world::Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
            let ft = |typed: &str| -> Result<f64, String> {
                let (len, unit) = units::parse_length(typed)?;
                len.feet(unit)
            };
            if is_beam {
                let b = r.beam(e).map_err(|why| why.to_string())?;
                let (storey, floor) = storey_of(id, b.bottom * k);
                let (across, deep) = (format!("{} in", tidy(b.width * k * 12.0)), format!("{} in", tidy(b.depth * k * 12.0)));
                let wanted = (ft(&across)?, ft(&deep)?);
                let existing: Option<String> = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::SegmentType)>();
                    let mut all: Vec<String> = q
                        .iter(&job.world)
                        .filter(|(_, t)| t.category == Kind::Beam && !t.round && t.width.feet().is_some_and(|x| (x - wanted.0).abs() < 1e-9) && t.depth.as_ref().and_then(|d| d.feet()).is_some_and(|x| (x - wanted.1).abs() < 1e-9))
                        .map(|(n, _)| n.path.clone())
                        .collect();
                    all.sort();
                    all.into_iter().next()
                };
                let type_path = match existing {
                    Some(path) => path,
                    None => {
                        let wanted_name = kept.clone().unwrap_or_else(|| format!("beam {across} by {deep}"));
                        let name = if job.children().get(&root).is_some_and(|c| c.contains_key(wanted_name.as_str())) { format!("{wanted_name} {id}") } else { wanted_name };
                        ed::framing_type(job, &name, Kind::Beam, &across, Some(deep.as_str()), None, entered)?
                    }
                };
                let line = [[tidy(b.a[0] * k), tidy(b.a[1] * k)], [tidy(b.b[0] * k), tidy(b.b[1] * k)]];
                // A beam along this very line already on the level (the model brought in a second time) would be counted twice.
                let near = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).abs() < 0.001 && (p[1] - q[1]).abs() < 0.001;
                let there = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &Segment)>();
                    q.iter(&job.world).any(|(n, kind, s)| *kind == Kind::Beam && n.parent.as_deref() == Some(storey.as_str()) && s.line_ft().is_some_and(|l| (near(l[0], line[0]) && near(l[1], line[1])) || (near(l[0], line[1]) && near(l[1], line[0]))))
                };
                if there {
                    return Err("it is already on this level: a beam along this very line would be counted twice".to_string());
                }
                // Held to its top, as a beam here is: its middle and half its depth, above the floor.
                let top = (b.middle_z + b.depth / 2.0) * k - floor;
                let made = ed::beams(job, &storey, &type_path, &line, &feet(top), entered)?;
                made.into_iter().next().ok_or_else(|| "no beam was made".to_string())
            } else {
                let b = r.boxed(e).ok_or("its body is not a straight box, standing upright, as far as this can read it")?;
                if b.angle.abs() > 1.0e-6 {
                    return Err("it is turned on the plan: a post here stands square to it".to_string());
                }
                let (storey, floor) = storey_of(id, b.bottom * k);
                let (across, deep) = (format!("{} in", tidy(b.size[0] * k * 12.0)), format!("{} in", tidy(b.size[1] * k * 12.0)));
                let wanted = (ft(&across)?, ft(&deep)?);
                let existing: Option<String> = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &plumbline_world::SegmentType)>();
                    let mut all: Vec<String> = q
                        .iter(&job.world)
                        .filter(|(_, t)| t.category == Kind::Post && t.round == b.round && t.width.feet().is_some_and(|x| (x - wanted.0).abs() < 1e-9) && (b.round || t.depth.as_ref().and_then(|d| d.feet()).is_some_and(|x| (x - wanted.1).abs() < 1e-9)))
                        .map(|(n, _)| n.path.clone())
                        .collect();
                    all.sort();
                    all.into_iter().next()
                };
                let type_path = match existing {
                    Some(path) => path,
                    None => {
                        let wanted_name = kept.clone().unwrap_or_else(|| if b.round { format!("post {across} round") } else { format!("post {across} by {deep}") });
                        let name = if job.children().get(&root).is_some_and(|c| c.contains_key(wanted_name.as_str())) { format!("{wanted_name} {id}") } else { wanted_name };
                        ed::framing_type(job, &name, Kind::Post, &across, if b.round { None } else { Some(deep.as_str()) }, None, entered)?
                    }
                };
                let at = [tidy(b.middle[0] * k), tidy(b.middle[1] * k)];
                let there = {
                    let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &plumbline_world::Stood)>();
                    q.iter(&job.world).any(|(n, kind, c)| *kind == Kind::Post && n.parent.as_deref() == Some(storey.as_str()) && c.at_ft().is_some_and(|p| (p[0] - at[0]).abs() < 0.001 && (p[1] - at[1]).abs() < 0.001) && c.base_offset.feet().is_some_and(|x| (x - ((b.bottom * k) - floor)).abs() < 0.001))
                };
                if there {
                    return Err("it is already on this level: a post at this very place would be counted twice".to_string());
                }
                ed::post(job, &storey, &type_path, at, Some(&feet(b.size[2] * k)), Some(&feet(b.bottom * k - floor)), entered)
            }
        })();
        match outcome {
            Ok(path) => {
                keep_name(job, &path, &r.name(e), id);
                if let Err(why) = marks(job, &path, id, &source, if is_beam { Kind::Beam } else { Kind::Post }) {
                    out.notes.push(format!("a mark on {path} was not put on it: {why}"));
                }
                made(if is_beam { "beams" } else { "posts" });
            }
            Err(why) => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, &why);
            }
        }
    }
    // ---- Things set down: fixtures, equipment and furnishings (call 197, point 1). Each is a box the
    // size of its body in the file, set down by the change that sets one down, Assumed, and called what
    // the file calls it. A body this does not read leaves its size unknown, and says so.
    let mut set_down: Vec<(String, Kind, P3, [f64; 3])> = Vec::new();
    let mut thing_by_id: BTreeMap<u64, String> = BTreeMap::new();
    let every_class: Vec<&str> = FIXTURES.iter().chain(&FURNISHINGS).chain(&EQUIPMENT).copied().collect();
    for (id, e) in of(&every_class) {
        if !has_place(e) {
            continue;
        }
        done.insert(id);
        let kind = if FIXTURES.contains(&e.name.as_str()) {
            Kind::Fixture
        } else if FURNISHINGS.contains(&e.name.as_str()) {
            Kind::Furnishing
        } else {
            Kind::Equipment
        };
        if only_runs && kind == Kind::Furnishing {
            *na.entry("furnishings").or_insert(0) += 1;
            continue;
        }
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        let body = r.boxed(e);
        let origin = r.placed(r.arg(e, 5)).o;
        let (middle, bottom) = match &body {
            Some(b) => (b.middle, b.bottom),
            None => ([origin[0], origin[1]], origin[2]),
        };
        let (storey, floor) = storey_of(id, bottom * k);
        let called = r.name(e);
        let what = said_of.get(&id).and_then(|it| it.what.clone()).unwrap_or_else(|| if called.is_empty() { kind.stem().to_string() } else { called.clone() });
        let at = [tidy(middle[0] * k), tidy(middle[1] * k)];
        let offset = feet(bottom * k - floor);
        let outcome = (|| -> Result<String, String> {
            // A thing of this kind in this very place already on the level (the model brought in a second
            // time) would be counted twice: it is not set down again.
            let there = {
                let mut q = job.world.query::<(&plumbline_world::Node, &Kind, &Placed)>();
                q.iter(&job.world).any(|(n, other, p)| {
                    n.parent.as_deref() == Some(storey.as_str())
                        && *other == kind
                        && p.at_ft().is_some_and(|c| (c[0] - at[0]).abs() < 0.001 && (c[1] - at[1]).abs() < 0.001)
                        && p.offset.feet().is_some_and(|z| (z - (bottom * k - floor)).abs() < 0.001)
                        && (called.is_empty() || n.name == called || n.name.starts_with(&format!("{called} ")))
                })
            };
            if there {
                return Err(format!("{} in this very place is already on this level: a second would be counted twice", a_kind(kind)));
            }
            let size = body.as_ref().map(|b| [tidy(b.size[0] * k), tidy(b.size[1] * k), tidy(b.size[2] * k)]);
            let path = ed::place(job, &storey, kind, &what, at, &offset, size, system_of.get(&id).map(String::as_str), entered)?;
            if size.is_none() {
                let at_it = job.find(&path).ok_or("the new thing is not there")?;
                let mut placed = job.world.get::<Placed>(at_it).cloned().ok_or("the new thing has no place")?;
                placed.size = Val::unknown("ft", "the model's body for it is not a shape this reads, so how big it is is not known");
                job.put(&path, placed)?;
            }
            if let Some(b) = &body {
                if b.angle.abs() > 1.0e-6 {
                    let now = (b.angle * 1.0e6).round() / 1.0e6 + 0.0;
                    ed::turn(job, &path, now, entered)?;
                }
            }
            keep_name(job, &path, &called, id);
            Ok(path)
        })();
        match outcome {
            Ok(path) => {
                made_ids.insert(id);
                thing_by_id.insert(id, path.clone());
                set_down.push((path, kind, [at[0], at[1], bottom * k], body.as_ref().map(|b| [b.size[0] * k, b.size[1] * k, b.size[2] * k]).unwrap_or([0.0; 3])));
                made(match kind {
                    Kind::Fixture => "fixtures",
                    Kind::Furnishing => "furnishings",
                    _ => "pieces of equipment",
                });
            }
            Err(why) => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, &why);
            }
        }
    }
    // ---- Fittings, on the runs they join (point 2). A fitting in the file names no run: it has no
    // ports and no relation, only its place. It is put where runs' ends meet at that place, by the
    // change that puts a fitting there, which works out its kind from the runs.
    for (id, e) in of(&["IFCDUCTFITTING", "IFCPIPEFITTING", "IFCCABLECARRIERFITTING", "IFCCABLEFITTING", "IFCFLOWFITTING"]) {
        if !has_place(e) {
            continue;
        }
        done.insert(id);
        let source = from(e, id);
        let entered = Entered { status: how.status, source: &source };
        // A fitting with no body at all (call 204, point 6: one the runs it joins give no size) is where its place is: the joint of the runs
        // there, and it comes in with its size Unknown, as the runs there are.
        let (middle, bottom, centre) = match r.boxed(e) {
            Some(b) => ([b.middle[0], b.middle[1]], b.bottom, b.bottom + b.size[2] / 2.0),
            None if r.pushed(e).is_none() && r.pieces(e) == 0 => {
                let o = r.placed(r.arg(e, 5)).o;
                ([o[0], o[1]], o[2], o[2])
            }
            None => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, "its body is not a box this reads, so where the runs it joins meet cannot be told");
                continue;
            }
        };
        let (storey, floor) = storey_of(id, bottom * k);
        let middle_z = centre * k - floor;
        let outcome = ed::fitting(job, &storey, [tidy(middle[0] * k), tidy(middle[1] * k)], Some(&feet(middle_z)), None, entered);
        match outcome {
            Ok(fitted) => {
                made_ids.insert(id);
                keep_name(job, &fitted.path, &r.name(e), id);
                // What the file calls it, against what the runs that meet there make it (call 200).
                let (said_kind, ok) = match r.arg(e, 8) {
                    V::Enum(w) => match w.as_str() {
                        "BEND" => (Some("bend"), matches!(fitted.kind, FittingKind::Bend)),
                        "JUNCTION" => (Some("junction"), matches!(fitted.kind, FittingKind::Tee | FittingKind::Cross)),
                        "TEE" => (Some("tee"), matches!(fitted.kind, FittingKind::Tee)),
                        "CROSS" => (Some("cross"), matches!(fitted.kind, FittingKind::Cross)),
                        "TRANSITION" | "REDUCER" => (Some("transition"), matches!(fitted.kind, FittingKind::Transition)),
                        "CONNECTOR" => (Some("coupling"), matches!(fitted.kind, FittingKind::Coupling)),
                        _ => (None, true),
                    },
                    _ => (None, true),
                };
                if let (Some(word), false) = (said_kind, ok) {
                    let made_kind = match fitted.kind {
                        FittingKind::Bend => "bend",
                        FittingKind::Tee => "tee",
                        FittingKind::Cross => "cross",
                        FittingKind::Transition => "transition",
                        FittingKind::Coupling => "coupling",
                    };
                    out.notes.push(format!(
                        "the fitting at ({}, {}, {}) is not what the file calls it: the file calls it a {word}, the runs there make a {made_kind}, and it is brought in as a {made_kind}",
                        tidy(middle[0] * k),
                        tidy(middle[1] * k),
                        tidy(middle_z)
                    ));
                }
                made("fittings");
            }
            Err(why) => {
                if let Some(reach) = reach_of(e) {
                    stood_on.push((said(&e.name), reach.0, reach.1));
                }
                leave(&e.name, &why);
            }
        }
    }
    // ---- What is connected to a fixture or a piece of equipment (call 200). A file with ports says it: each
    // port on a thing is its connection there, put where the file puts it by the change that makes a
    // connection; where the file ties it to the end of a run that stands at it, that run is joined to it, by
    // the change the Connect tool makes, and no other end is joined for being near. A file with no ports
    // says nothing, and reads as it always did: a run's end inside a thing's box is joined to it.
    if file_has_ports {
        let (mut connections, mut joined, mut no_sort) = (0usize, 0usize, 0usize);
        let mut apart: Vec<String> = Vec::new();
        let mut refused: Vec<String> = Vec::new();
        // The connections made that are not tied to the end of a run in the file, each by its thing and where (call 206 point 10).
        let mut loose: Vec<String> = Vec::new();
        for (id, thing) in &thing_by_id {
            let Some(ports) = ports_of.get(id) else { continue };
            let kind = job.kind(thing);
            if !matches!(kind, Some(Kind::Fixture | Kind::Equipment)) {
                continue;
            }
            for pid in ports {
                let Some(port) = r.file.entities.get(pid) else { continue };
                let place = r.placed(r.arg(port, 5)).o;
                let at = [place[0] * k, place[1] * k, place[2] * k];
                let sort = match r.arg(port, 8) {
                    V::Enum(w) => match w.as_str() {
                        "DUCT" => Some(plumbline_world::ConnectorKind::Duct),
                        "PIPE" => Some(plumbline_world::ConnectorKind::Pipe),
                        "CABLECARRIER" => Some(plumbline_world::ConnectorKind::Conduit),
                        "CABLE" => Some(plumbline_world::ConnectorKind::Power),
                        _ => None,
                    },
                    _ => None,
                };
                let domain = match r.arg(port, 9) {
                    V::Enum(w) => match w.as_str() {
                        "DOMESTICCOLDWATER" | "WATERSUPPLY" => Some(plumbline_world::Domain::Cold),
                        "DOMESTICHOTWATER" => Some(plumbline_world::Domain::Hot),
                        "SEWAGE" | "DRAINAGE" | "WASTEWATER" => Some(plumbline_world::Domain::Waste),
                        "VENT" => Some(plumbline_world::Domain::Vent),
                        "STORMWATER" | "RAINWATER" => Some(plumbline_world::Domain::Storm),
                        "GAS" => Some(plumbline_world::Domain::Gas),
                        _ => None,
                    },
                    _ => None,
                };
                let flow = match r.arg(port, 7) {
                    V::Enum(w) if w == "SOURCE" => plumbline_world::Flow::Out,
                    V::Enum(w) if w == "SINK" => plumbline_world::Flow::In,
                    _ => plumbline_world::Flow::Both,
                };
                // The end of a run that the file ties it to, where that run is one brought in.
                let mut tie: Option<(String, usize)> = None;
                for peer in tied_to.get(pid).into_iter().flatten() {
                    let Some(whole) = owner_of.get(peer) else { continue };
                    let Some((run, a, b)) = run_by_id.get(whole) else { continue };
                    let (da, db) = (dist3(*a, at), dist3(*b, at));
                    let (end, d) = if da <= db { (0, da) } else { (1, db) };
                    if d <= 0.05 {
                        tie = Some((run.clone(), end));
                    } else {
                        {
                        let mut called = |path: &str| job.find(path).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string());
                        apart.push(format!("a connection of {} is tied to the end of {}, which is {} ft from it", called(thing), called(run), tidy(d)));
                    }
                    }
                    break;
                }
                let sort_or_run = sort.or_else(|| tie.as_ref().and_then(|(run, _)| job.kind(run)).and_then(plumbline_world::ConnectorKind::for_run));
                let Some(sort) = sort_or_run else {
                    no_sort += 1;
                    continue;
                };
                let source = from(port, *pid);
                let entered = Entered { status: how.status, source: &source };
                let facing = r.file.entities.get(id).map(|t| r.placed(r.arg(t, 5))).and_then(|frame| match maker::way_of(&r, port, &frame) {
                    maker::Way::Is(f) => Some(f),
                    _ => None,
                });
                let said = ed::PortSaid { name: r.name(port), kind: sort, domain, flow, facing };
                match ed::connect_port(job, thing, tie.as_ref().map(|(run, end)| (run.as_str(), *end)), at, &said, entered) {
                    Ok(_) => {
                        connections += 1;
                        if tie.is_some() {
                            joined += 1;
                        } else {
                            let name = job.find(thing).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| thing.clone());
                            let port_name = r.name(port);
                            loose.push(format!("{name}{} at ({}, {}, {})", if port_name.is_empty() { String::new() } else { format!(", {port_name}") }, tidy(at[0]), tidy(at[1]), tidy(at[2])));
                        }
                    }
                    Err(why) if why.contains("already has a connection") => {}
                    Err(why) => refused.push(format!("{thing}: {why}")),
                }
            }
        }
        if connections > 0 {
            out.notes.push(format!(
                "{connections} connection{} on fixtures and equipment {} brought in from the file's connection points, each Assumed; {joined} {} tied to the end of a run and {} joined to it, and no other end was joined for being near",
                if connections == 1 { "" } else { "s" },
                if connections == 1 { "was" } else { "were" },
                if joined == 1 { "was" } else { "were" },
                if joined == 1 { "is" } else { "are" }
            ));
        }
        if !loose.is_empty() {
            let shown: Vec<String> = loose.iter().take(5).cloned().collect();
            out.notes.push(format!(
                "{} of the {connections} connection{} {} not tied to the end of a run in the file, and {} left loose: {}{}",
                loose.len(),
                if connections == 1 { "" } else { "s" },
                if loose.len() == 1 { "is" } else { "are" },
                if loose.len() == 1 { "is" } else { "are" },
                shown.join("; "),
                if loose.len() > 5 { format!("; and {} more", loose.len() - 5) } else { String::new() }
            ));
        }
        if !apart.is_empty() {
            out.notes.push(format!("{} connection point{} tied in the file to the end of a run that is not at it, and not joined: {}", apart.len(), if apart.len() == 1 { " is" } else { "s are" }, apart.join("; ")));
        }
        if no_sort > 0 {
            out.notes.push(format!("{no_sort} connection point{} on a fixture or a piece of equipment {} no sort (duct, pipe, cable) and no run to take it from: not brought in", if no_sort == 1 { "" } else { "s" }, if no_sort == 1 { "says" } else { "say" }));
        }
        if !refused.is_empty() {
            out.notes.push(format!("{} connection{} this program would not take: {}", refused.len(), if refused.len() == 1 { "" } else { "s" }, refused.join("; ")));
        }
    } else {
        let runs = run_ends.clone();
        let mut not_joined: Vec<String> = Vec::new();
        let mut joined = 0usize;
        let joining = format!("{}: the run ends at it, in the model brought in", how.source);
        for (thing, kind, at, size) in set_down.clone() {
            if !matches!(kind, Kind::Fixture | Kind::Equipment) || size == [0.0; 3] {
                continue;
            }
            let reach = size[0].hypot(size[1]) / 2.0 + 0.1;
            let near = |p: P3| (p[0] - at[0]).abs() <= reach && (p[1] - at[1]).abs() <= reach && p[2] >= at[2] - 0.1 && p[2] <= at[2] + size[2] + 0.1;
            for (run, a, b) in &runs {
                if !(near(*a) || near(*b)) {
                    continue;
                }
                match ed::connect(job, &thing, run, Entered { status: how.status, source: &joining }) {
                    Ok(_) => joined += 1,
                    Err(why) if why.contains("already has a connection") || why.starts_with("neither end of") => {}
                    Err(why) => not_joined.push(format!("{run} at {thing}: {why}")),
                }
            }
        }
        if joined > 0 {
            out.notes.push(format!("{joined} run{} ending at a fixture or a piece of equipment {} joined to it, where it stood", if joined == 1 { "" } else { "s" }, if joined == 1 { "was" } else { "were" }));
        }
        if !not_joined.is_empty() {
            out.notes.push(format!("{} run{} end{} at a fixture or a piece of equipment and could not be joined to it: {}", not_joined.len(), if not_joined.len() == 1 { "" } else { "s" }, if not_joined.len() == 1 { "s" } else { "" }, not_joined.join("; ")));
        }
    }
    if runs_unsized > 0 {
        out.notes.push(format!("{runs_unsized} run{} {} no size: {} drawn as a line alone and comes in with its size Unknown", if runs_unsized == 1 { "" } else { "s" }, if runs_unsized == 1 { "has" } else { "have" }, if runs_unsized == 1 { "it is" } else { "each is" }));
    }
    // Run ends that stand at a thing but are joined to nothing, said (call 200): nothing is joined for being
    // near, so where an end stands within a foot of a fixture or a piece of equipment and nothing joins it,
    // that is said, with how far.
    {
        job.derive();
        let net = building::network::network(job, None, None, building::network::TOLERANCE_FT)?;
        let mut near_but_open: Vec<String> = Vec::new();
        for node in net.open.iter().filter_map(|i| net.nodes.get(*i)) {
            let Some(building::network::Joint::RunEnd { run, .. }) = node.joints.first() else { continue };
            for (thing, kind, at, size) in &set_down {
                if !matches!(kind, Kind::Fixture | Kind::Equipment) || *size == [0.0; 3] {
                    continue;
                }
                let angle = job.find(thing).and_then(|e| job.world.get::<Placed>(e)).and_then(|p| p.angle_deg()).unwrap_or(0.0);
                let (sin, cos) = angle.to_radians().sin_cos();
                let (dx, dy, dz) = (node.at[0] - at[0], node.at[1] - at[1], node.at[2] - at[2]);
                let local = [dx * cos + dy * sin, -dx * sin + dy * cos, dz];
                let out_by = [(local[0].abs() - size[0] / 2.0).max(0.0), (local[1].abs() - size[1] / 2.0).max(0.0), (-local[2]).max(local[2] - size[2]).max(0.0)];
                let d = (out_by[0] * out_by[0] + out_by[1] * out_by[1] + out_by[2] * out_by[2]).sqrt();
                if d <= 1.0 {
                    let mut name = |path: &str| job.find(path).and_then(|e| job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string());
                    near_but_open.push(format!("{} {:.2} ft from {}", name(run), d, name(thing)));
                }
            }
        }
        if !near_but_open.is_empty() {
            near_but_open.sort();
            out.notes.push(format!(
                "{} run end{} stand{} within a foot of a fixture or a piece of equipment that the file gives no connection there, and read{} open: {}",
                near_but_open.len(),
                if near_but_open.len() == 1 { "" } else { "s" },
                if near_but_open.len() == 1 { "s" } else { "" },
                if near_but_open.len() == 1 { "s" } else { "" },
                near_but_open.join("; ")
            ));
        }
    }
    // ---- Everything else that is a thing with a place and a shape: listed.
    const READ: [&str; 22] = [
        "IFCBUILDINGSTOREY", "IFCWALL", "IFCWALLSTANDARDCASE", "IFCWALLELEMENTEDCASE", "IFCDOOR", "IFCDOORSTANDARDCASE", "IFCWINDOW", "IFCWINDOWSTANDARDCASE", "IFCOPENINGELEMENT", "IFCOPENINGSTANDARDCASE", "IFCSLAB",
        "IFCSLABSTANDARDCASE", "IFCSLABELEMENTEDCASE", "IFCSPACE", "IFCDUCTSEGMENT", "IFCPIPESEGMENT", "IFCCABLECARRIERSEGMENT", "IFCCABLESEGMENT", "IFCFLOWSEGMENT", "IFCSITE", "IFCBUILDING", "IFCPROJECT",
    ];
    // A thing of the building with no place: the model names it (it has the model's
    // own 22-letter mark, and is not a tie between things, a type or a list of values)
    // and says nowhere it is.
    const NOT_THINGS: [&str; 14] = ["IFCPROPERTYSET", "IFCELEMENTQUANTITY", "IFCGROUP", "IFCSYSTEM", "IFCDISTRIBUTIONSYSTEM", "IFCDISTRIBUTIONCIRCUIT", "IFCZONE", "IFCACTOR", "IFCTASK", "IFCWORKPLAN", "IFCWORKSCHEDULE", "IFCCOSTITEM", "IFCCOSTSCHEDULE", "IFCPROJECTLIBRARY"];
    for id in &ids {
        let e = &r.file.entities[id];
        let is_placed = r.get(r.arg(e, 5)).is_some_and(|(_, p)| p.name == "IFCLOCALPLACEMENT" || p.name == "IFCGRIDPLACEMENT");
        if READ.contains(&e.name.as_str()) || done.contains(id) {
            continue;
        }
        if is_placed {
            if let Some(reach) = reach_of(e) {
                stood_on.push((said(&e.name), reach.0, reach.1));
            }
            if only_runs {
                // Not a run, and nothing a run serves: not asked for.
                *na.entry("other things").or_insert(0) += 1;
                continue;
            }
            leave(&e.name, if e.name == "IFCROOF" { NO_ROOF_OUTLINE } else { "nothing here to bring it in as" });
        } else if Reader::text(r.arg(e, 0)).is_some_and(|mark| mark.len() == 22) && e.args.len() >= 7 && !e.name.starts_with("IFCREL") && !e.name.ends_with("TYPE") && !e.name.ends_with("STYLE") && !e.name.ends_with("TEMPLATE") && !NOT_THINGS.contains(&e.name.as_str()) {
            if only_runs {
                *na.entry("other things").or_insert(0) += 1;
            } else {
                leave(&e.name, "the model gives it no place, and there is nothing here to bring it in as");
            }
        }
    }
    if guessed > 0 {
        out.notes.push(format!("{guessed} thing{} the model puts on no level: each was put on the highest level whose floor is not above it", if guessed == 1 { " is one" } else { "s are ones" }));
    }
    if systems_not_known > 0 {
        out.notes.push(format!("{systems_not_known} system{} in the model {} of a sort this program has no name for: what is on {} was brought in on no system", if systems_not_known == 1 { "" } else { "s" }, if systems_not_known == 1 { "is" } else { "are" }, if systems_not_known == 1 { "it" } else { "them" }));
    }
    // What follows from what was left out (call 197, point 6): the runs' ends that stood on a thing not
    // brought in read open here, where they read joined in the model.
    if !stood_on.is_empty() {
        job.derive();
        let net = building::network::network(job, None, None, building::network::TOLERANCE_FT)?;
        let mut by_what: BTreeMap<String, usize> = BTreeMap::new();
        for at in net.open.iter().filter_map(|i| net.nodes.get(*i)).map(|n| n.at) {
            let reach = 0.1;
            if let Some((what, _, _)) = stood_on.iter().find(|(_, lo, hi)| (0..3).all(|i| at[i] >= lo[i] - reach && at[i] <= hi[i] + reach)) {
                *by_what.entry(what.clone()).or_insert(0) += 1;
            }
        }
        let total: usize = by_what.values().sum();
        if total > 0 {
            out.notes.push(format!(
                "{total} run end{} read open here: {} stood on {} that {} not brought in ({})",
                if total == 1 { "" } else { "s" },
                if total == 1 { "it" } else { "each" },
                if by_what.len() == 1 && total == 1 { "a thing" } else { "things" },
                if by_what.len() == 1 && total == 1 { "was" } else { "were" },
                by_what.iter().map(|(what, n)| format!("{n} on {what}")).collect::<Vec<_>>().join(", ")
            ));
        }
    }
    // Where the model's levels at a height the job has not were put (call 210 point 2): on the job's level each thing stands within, and
    // no level was added. Said for each of the model's levels that had something made on it, of the things; and, where a thing stands above
    // the top of the job's highest level or below its lowest, which and how far (call 213 point 11).
    {
        let mut by_level: BTreeMap<u64, Vec<&(u64, u64, String, f64, f64)>> = BTreeMap::new();
        for p in put_on.iter().filter(|p| made_ids.contains(&p.0)) {
            by_level.entry(p.1).or_default().push(p);
        }
        let mut said_levels: Vec<(f64, String)> = Vec::new();
        let (mut above, mut below): (Vec<(String, f64)>, Vec<(String, f64)>) = (Vec::new(), Vec::new());
        let highest = job_levels.last().cloned();
        let lowest = job_levels.first().cloned();
        for (sid, things) in &by_level {
            let Some((called, stated, _)) = loose.get(sid) else { continue };
            let name_of = |path: &str| job_levels.iter().find(|l| l.1 == path).map(|l| l.0.clone()).unwrap_or_else(|| path.to_string());
            let mut targets: BTreeMap<String, usize> = BTreeMap::new();
            for t in things {
                *targets.entry(name_of(&t.2)).or_insert(0) += 1;
                let thing_name = r.file.entities.get(&t.0).map(|e| r.name(e)).filter(|n| !n.is_empty()).unwrap_or_else(|| "a thing".to_string());
                if let Some((_, hp, _)) = &highest {
                    if t.2 == *hp {
                        if let Some(top) = job_tops.get(hp).filter(|top| t.3 > **top + 0.01) {
                            above.push((thing_name.clone(), t.3 - top));
                        }
                    }
                }
                if let Some((_, lp, floor)) = &lowest {
                    if t.2 == *lp && t.3 < *floor - 0.01 {
                        below.push((thing_name, floor - t.3));
                    }
                }
            }
            let where_to = format!("(the job's {})", targets.iter().map(|(n, c)| if targets.len() == 1 { format!("\"{n}\"") } else { format!("\"{n}\": {c}") }).collect::<Vec<_>>().join(", "));
            said_levels.push((*stated, format!("the model's level \"{called}\" has its floor at {}, which is not the height of any level the job has: it holds {}, and each was put on the level it stands within {where_to}; no level was added", feet(*stated), if things.len() == 1 { "1 thing".to_string() } else { format!("{} things", things.len()) })));
        }
        said_levels.sort_by(|a, b| a.0.total_cmp(&b.0));
        out.notes.extend(said_levels.into_iter().map(|(_, n)| n));
        let list = |v: &[(String, f64)], way: &str| -> String {
            let shown: Vec<String> = v.iter().take(5).map(|(n, by)| format!("{n} is {} {way} it", feet(*by))).collect();
            let more = if v.len() > 5 { format!(" and {} more", v.len() - 5) } else { String::new() };
            format!("{}{more}", shown.join(", "))
        };
        if let (false, Some((name, path, _))) = (above.is_empty(), highest) {
            let top = job_tops.get(&path).copied().unwrap_or(0.0);
            out.notes.push(format!("{} above the top of the job's highest level, \"{name}\" ({}): {}; check the model's heights against the job's", if above.len() == 1 { "1 thing stands".to_string() } else { format!("{} things stand", above.len()) }, feet(top), list(&above, "above")));
        }
        if let (false, Some((name, _, floor))) = (below.is_empty(), lowest) {
            out.notes.push(format!("{} below the job's lowest level, \"{name}\" (its floor at {}): {}; check the model's heights against the job's", if below.len() == 1 { "1 thing stands".to_string() } else { format!("{} things stand", below.len()) }, feet(floor), list(&below, "below")));
        }
    }
    out.notes.push(format!("the model is in {}: every length was turned into feet and kept to a ten-thousandth of a foot", out.unit));
    out.left_out = left.into_iter().map(|((class, why), n)| (class, n, why)).collect();
    out.not_asked = na.into_iter().map(|(what, n)| (what.to_string(), n)).collect();
    if only_runs {
        if out.made.is_empty() {
            let total: usize = out.not_asked.iter().map(|(_, n)| n).sum();
            let asked = if total == 0 { String::new() } else { format!(". It holds {}, which {} not asked for", not_asked_list(&out.not_asked), if total == 1 { "was" } else { "were" }) };
            if out.left_out.is_empty() {
                return Err(format!("this model has no runs and no equipment: nothing was brought in{asked}"));
            }
            if let Some(said) = already_in_the_job(&out.left_out) {
                return Err(format!("{said}{asked}"));
            }
            return Err(format!("none of the model's runs and equipment could be brought in: {}{asked}", out.left_out.iter().map(|(c, n, why)| format!("{n} {c} ({why})")).collect::<Vec<_>>().join("; ")));
        }
        return Ok(out);
    }
    if out.made.iter().all(|(what, _)| what == "levels") {
        if let Some(said) = already_in_the_job(&out.left_out) {
            return Err(said);
        }
        return Err(format!("nothing in the model but its levels could be brought in: {}", out.left_out.iter().map(|(c, n, why)| format!("{n} {c} ({why})")).collect::<Vec<_>>().join("; ")));
    }
    // Walls and floors of the model that lie over the job's own (call 210 point 1): left out, and the way not to be asked for them.
    if extras {
        let over = |class: &str| out.left_out.iter().filter(|(c, _, why)| c == class && why.contains("would be counted twice")).map(|(_, n, _)| n).sum::<usize>();
        let (walls, floors) = (over("wall"), over("floor slab"));
        if walls + floors > 0 {
            let mut each = Vec::new();
            if walls > 0 {
                each.push(format!("{walls} wall{}", if walls == 1 { "" } else { "s" }));
            }
            if floors > 0 {
                each.push(format!("{floors} floor{}", if floors == 1 { "" } else { "s" }));
            }
            out.notes.push(format!(
                "{} of the model lie over what the job has and were left out, so as not to be counted twice; to bring a model like this in without being asked for them, choose only its runs and the equipment they serve",
                list_said(&each)
            ));
        }
    }
    Ok(out)
}
