use super::*;

/// One line of a system's takeoff: the runs of one kind and one size.
#[derive(Clone, Debug)]
pub struct TakeoffLine {
    pub kind: Kind,
    /// The size, said in inches whatever unit each run's was entered in, so that one
    /// size is one line.
    pub size: String,
    /// The worst status among the sizes of the runs on this line.
    pub size_status: Status,
    pub length: Sum,
    /// The length given three ways where a run on this line is not new work (call 158).
    pub length_by_work: Option<WorkSplit>,
}

#[derive(Clone, Debug)]
pub struct SystemTakeoff {
    /// The system's path. Empty for the two groups that are not systems: what is on no
    /// system, and what is on a system nobody has named.
    pub path: String,
    pub name: String,
    pub discipline: Discipline,
    pub classification: String,
    /// How sure it is that what is listed here belongs here. Known for a system and for
    /// what is on none; Unknown for what is on a system that is not known.
    pub flag: Status,
    /// All of its runs, whatever their size: how many and how far.
    pub runs: Sum,
    /// All of its runs given three ways, where one is not new work (call 158).
    pub runs_by_work: Option<WorkSplit>,
    pub lines: Vec<TakeoffLine>,
    /// Equipment and fixtures on the system: what each is, how many, and the worst status
    /// among what they are said to be and whether they are on this system.
    pub placed: Vec<(Kind, String, usize, Status)>,
    /// How many of each `placed` entry are to build, existing to remain and to demolish,
    /// in step with `placed` (call 158).
    pub placed_by_work: Vec<[usize; 3]>,
    /// The fittings on the system, counted by kind and by what each is said to be, with
    /// the worst status among those. A fitting with no kind entered is counted under None.
    pub fittings: Vec<(Option<FittingKind>, String, usize, Status)>,
    /// The same, in step with `fittings` (call 158).
    pub fittings_by_work: Vec<[usize; 3]>,
    /// How its runs hang together: what is joined, what is open, what has no fitting.
    pub joined: Continuity,
}

/// One end of a run, or one connector, that meets nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenEnd {
    /// The run or the placed thing, by path.
    pub element: String,
    /// Said for a person: "hot riser: its second end", "lavatory 1: its Waste connector".
    pub said: String,
    pub at: P3,
}

/// A run that passes over a thing's connection: where, and what it says.
#[derive(Clone, Debug, PartialEq)]
pub struct OverConnection {
    pub at: P3,
    /// "pipe 1 passes over Lavatory's Waste connection".
    pub said: String,
}

/// How a system's runs hang together, read off the job's network (`network.rs`). Nothing
/// here stops a job from loading or a length from being counted: a job being entered has
/// open ends. It is what an estimator checks before trusting a count of fittings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Continuity {
    /// How many separate networks its runs fall into. One, when every run on the system
    /// is joined to every other.
    pub networks: usize,
    pub open: Vec<OpenEnd>,
    /// Where a run or a connector meets a run along its length with no fitting entered:
    /// each is a tee, or a mistake.
    pub no_fitting: Vec<P3>,
    /// Where a run passes over a thing's connection with no fitting: joined to it, and said as that, by the run, the thing and the
    /// connection (call 213 point 3).
    pub over_connection: Vec<OverConnection>,
    /// What else looks wrong, said in a sentence each.
    pub warnings: Vec<String>,
    /// What could not be placed, and so was not checked, each with why.
    pub left_out: Vec<(String, String)>,
}

/// A size in feet, said in inches: `6 in`, `3/4 in`.
pub(super) fn in_inches(feet: f64) -> String {
    units::show_length(&Len::One((feet * 12.0 * 1.0e6).round() / 1.0e6), units::IN)
}

/// What each system comes to: its runs by kind and size with their length, its fittings
/// counted by kind, and its equipment and fixtures counted by what they are.
///
/// A run said to be on a system with less than certainty (Assumed) carries that onto its
/// line: the line's length is no surer than that the run is on this system. Two groups
/// come after the systems, when there is anything in them, so nothing is left off for
/// want of a system: what is on no system, and what is on a system that is not known.
/// Not knowing the system is not the same as having none.
///
/// A run's size is its type's. Runs are grouped by their size as a size, not as it was
/// typed: 6 in, 0.5 ft and 0' 6" are one line. Lines are in order of size, smallest
/// first, with no size last: a run whose type or whose type's size nobody has is not
/// dropped and not guessed.
pub fn takeoff(job: &mut Job) -> Vec<SystemTakeoff> {
    let order = job.order.clone();
    let net = network::network(job, None, None, network::TOLERANCE_FT);
    let tree = Tree::of(&mut job.world);
    let w = &job.world;
    let blank = |name: &str, flag: Status| SystemTakeoff { path: String::new(), name: name.into(), discipline: Discipline::Architecture, classification: String::new(), flag, runs: view::sum_of("LF", &[]), runs_by_work: None, lines: Vec::new(), placed: Vec::new(), placed_by_work: Vec::new(), fittings: Vec::new(), fittings_by_work: Vec::new(), joined: Continuity::default() };
    let mut systems: Vec<SystemTakeoff> = Vec::new();
    for &e in &order {
        if let (Some(n), Some(info)) = (w.get::<Node>(e), w.get::<SystemInfo>(e)) {
            systems.push(SystemTakeoff { path: n.path.clone(), name: n.name.clone(), discipline: info.discipline, classification: info.classification.clone(), ..blank("", Status::Known) });
        }
    }
    let named = systems.len();
    systems.push(blank("On no system", Status::Known));
    systems.push(blank("System not known", Status::Unknown));
    type Figures = Vec<(String, Option<f64>, Status)>;
    // By system, then kind, then size: round before rectangular, small before large, and
    // a run with no size last. Beside each figure, where it is: to build, existing or to
    // demolish (call 158).
    let mut runs: BTreeMap<(usize, u8, u8, i64, i64), (Kind, String, Status, Figures, Vec<usize>)> = BTreeMap::new();
    for &e in &order {
        let (Some(n), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e).copied()) else { continue };
        if !(kind.is_segment() || kind.is_placed()) {
            continue;
        }
        // Which group it goes in, and how sure that is.
        let (at, sure) = match w.get::<InSystem>(e) {
            None => (named, Status::Known),
            Some(s) => match s.0.value.as_ref().and_then(|p| systems[..named].iter().position(|x| x.path == *p)) {
                Some(i) if s.0.status != Status::Unknown => (i, s.0.status),
                _ => (named + 1, Status::Known),
            },
        };
        let place = view::work_place(w, e);
        if kind.is_placed() {
            let what = w.get::<Placed>(e).map(|p| (p.what.value.clone().unwrap_or_else(|| "not said".into()), p.what.status)).unwrap_or(("not said".into(), Status::Unknown));
            if kind == Kind::Fitting {
                let fitting = w.get::<Fitting>(e).map(|f| f.kind);
                let status = what.1.max(sure).max(if fitting.is_some() { Status::Known } else { Status::Unknown });
                let list = &mut systems[at].fittings;
                match list.iter().position(|x| x.0 == fitting && x.1 == what.0) {
                    Some(i) => {
                        list[i].2 += 1;
                        list[i].3 = list[i].3.max(status);
                        systems[at].fittings_by_work[i][place] += 1;
                    }
                    None => {
                        list.push((fitting, what.0, 1, status));
                        let mut n = [0; 3];
                        n[place] = 1;
                        systems[at].fittings_by_work.push(n);
                    }
                }
                continue;
            }
            let list = &mut systems[at].placed;
            match list.iter().position(|x| x.0 == kind && x.1 == what.0) {
                Some(i) => {
                    list[i].2 += 1;
                    list[i].3 = list[i].3.max(what.1).max(sure);
                    systems[at].placed_by_work[i][place] += 1;
                }
                None => {
                    list.push((kind, what.0, 1, what.1.max(sure)));
                    let mut n = [0; 3];
                    n[place] = 1;
                    systems[at].placed_by_work.push(n);
                }
            }
            continue;
        }
        // Its size is its type's. No type, a type nobody knows, or a type with no size:
        // each is a run of no known size.
        let sized = w.get::<Segment>(e).and_then(|r| typed(&tree.segment_types, &r.type_name, "a run").ok().map(|(_, t)| (t, r.type_name.status)));
        let micro = |feet: f64| (feet * 12.0 * 1.0e6).round() as i64;
        let (shape, a, b, said) = match sized.map(|(t, _)| (t.round_ft(), t.rect_ft())) {
            Some((Some(d), _)) => (0, micro(d), 0, format!("{} round", in_inches(d))),
            Some((None, Some((width, depth)))) => (1, micro(width), micro(depth), format!("{} × {}", in_inches(width), in_inches(depth))),
            _ => (2, 0, 0, "size unknown".to_string()),
        };
        let rank = match kind {
            Kind::Duct => 0,
            Kind::Pipe => 1,
            Kind::Conduit => 2,
            _ => 3,
        };
        let figure = |name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (d.value, d.status)).unwrap_or((None, Status::Unknown));
        let entry = runs.entry((at, rank, shape, a, b)).or_insert((kind, said, Status::Known, Vec::new(), Vec::new()));
        entry.2 = entry.2.max(if shape == 2 { Status::Unknown } else { sized.map(|(t, sure)| t.status().max(sure)).unwrap_or(Status::Unknown) });
        // A figure that is there is no surer than that the run is on this system.
        let (l, ls) = figure("true_length");
        entry.3.push((n.path.clone(), l, if l.is_some() { ls.max(sure) } else { ls }));
        entry.4.push(place);
    }
    let mut all: Vec<Figures> = vec![Vec::new(); systems.len()];
    let mut all_places: Vec<Vec<usize>> = vec![Vec::new(); systems.len()];
    for ((at, ..), (_, _, _, lengths, places)) in &runs {
        all[*at].extend(lengths.iter().cloned());
        all_places[*at].extend(places.iter().copied());
    }
    for (i, figures) in all.iter().enumerate() {
        systems[i].runs = view::sum_of("LF", figures);
        systems[i].runs_by_work = view::split_by_work("LF", figures, &all_places[i]);
    }
    for ((at, ..), (kind, size, size_status, lengths, places)) in runs {
        systems[at].lines.push(TakeoffLine { kind, size, size_status, length: view::sum_of("LF", &lengths), length_by_work: view::split_by_work("LF", &lengths, &places) });
    }
    for t in &mut systems {
        // Sorted with the figures of where each is (call 158), so that the two stay in step.
        let mut both: Vec<_> = std::mem::take(&mut t.fittings).into_iter().zip(std::mem::take(&mut t.fittings_by_work)).collect();
        both.sort_by(|(x, _), (y, _)| (x.0.is_none(), x.0, &x.1).cmp(&(y.0.is_none(), y.0, &y.1)));
        (t.fittings, t.fittings_by_work) = both.into_iter().unzip();
    }
    // How each hangs together. An element is checked with the group its system puts it in.
    let group_of = |path: &str| -> usize {
        let e = order.iter().find(|e| w.get::<Node>(**e).is_some_and(|n| n.path == path));
        match e.and_then(|e| w.get::<InSystem>(*e)) {
            None => named,
            Some(s) => match s.0.value.as_ref().and_then(|p| systems[..named].iter().position(|x| x.path == *p)) {
                Some(i) if s.0.status != Status::Unknown => i,
                _ => named + 1,
            },
        }
    };
    match net {
        Err(why) => {
            for t in &mut systems {
                t.joined.warnings.push(format!("what is joined to what was not worked out: {why}"));
            }
        }
        Ok(net) => {
            let groups_of = |elements: &[String]| -> std::collections::BTreeSet<usize> { elements.iter().map(|p| group_of(p)).collect() };
            let mut found: Vec<Continuity> = vec![Continuity::default(); systems.len()];
            for g in &net.groups {
                // A network counts for each system that has a run in it.
                let runs: Vec<String> = g.elements.iter().filter(|p| tree.kinds.get(*p).is_some_and(|k| k.is_segment())).cloned().collect();
                for i in groups_of(&runs) {
                    found[i].networks += 1;
                }
            }
            for &n in &net.open {
                let node = &net.nodes[n];
                let Some(joint) = node.joints.first() else { continue };
                let name = tree.called(joint.element());
                let said = match joint {
                    network::Joint::RunEnd { end, .. } => format!("{name}: its {} end", if *end { "second" } else { "first" }),
                    network::Joint::RunSide { .. } => format!("{name}: along its length"),
                    network::Joint::Connector { element, index, .. } => {
                        let e = order.iter().find(|e| w.get::<Node>(**e).is_some_and(|n| n.path == *element));
                        let label = e.and_then(|e| w.get::<Connectors>(*e)).and_then(|c| c.0.get(*index)).map(|k| k.name.clone()).filter(|s| !s.is_empty());
                        // By its name where it has one ("its Waste connection"); else
                        // by its number ("its connection 2").
                        match label {
                            Some(label) => format!("{name}: its {label} connection"),
                            None => format!("{name}: its connection {}", index + 1),
                        }
                    }
                };
                // An open end of a run is its system's. A connector that meets nothing has
                // no run to say whose it is: a pipe connector that says what it carries
                // is listed on the systems that carry that (a water closet's open waste
                // outlet is the drain's to deal with, though the fixture is on the cold
                // water), and any other on the system the thing itself is on.
                let mut where_to: Vec<usize> = Vec::new();
                if let network::Joint::Connector { domain: Some(domain), .. } = joint {
                    let carried = domain.system_classification();
                    where_to = (0..named).filter(|&i| carried == Some(systems[i].classification.as_str())).collect();
                }
                if where_to.is_empty() {
                    where_to.push(group_of(joint.element()));
                }
                for i in where_to {
                    found[i].open.push(OpenEnd { element: joint.element().to_string(), said: said.clone(), at: node.at });
                }
            }
            for warning in &net.warnings {
                // A warning is said on the systems of the runs it is about. (A water closet
                // is on the cold water system and its waste connector is in the drain's
                // network: a fault in the drain is not the cold water's.)
                let runs: Vec<String> = warning.elements.iter().filter(|p| tree.kinds.get(*p).is_some_and(|k| k.is_segment())).cloned().collect();
                for i in groups_of(if runs.is_empty() { &warning.elements } else { &runs }) {
                    match (warning.kind, warning.at) {
                        ("joined_mid_run", Some(at)) => found[i].no_fitting.push(at),
                        ("over_connection", Some(at)) => {
                            let said = warning.connector.as_ref().map(|(thing, index)| {
                                let label = order.iter().find(|e| w.get::<Node>(**e).is_some_and(|n| n.path == *thing)).and_then(|e| w.get::<Connectors>(*e)).and_then(|c| c.0.get(*index)).map(|k| k.name.clone()).filter(|s| !s.is_empty());
                                let connection = match label {
                                    Some(label) => format!("{label} connection"),
                                    None => format!("connection {}", index + 1),
                                };
                                format!("{} passes over {}'s {connection}", runs.first().map(|r| tree.called(r)).unwrap_or_else(|| "a run".to_string()), tree.called(thing))
                            });
                            found[i].over_connection.push(OverConnection { at, said: said.unwrap_or_else(|| warning.message.clone()) });
                        }
                        _ => found[i].warnings.push(warning.message.clone()),
                    }
                }
            }
            for (path, why) in &net.left_out {
                found[group_of(path)].left_out.push((path.clone(), why.clone()));
            }
            for (t, joined) in systems.iter_mut().zip(found) {
                t.joined = joined;
            }
        }
    }
    systems.into_iter().enumerate().filter(|(i, t)| *i < named || !t.lines.is_empty() || !t.placed.is_empty() || !t.fittings.is_empty()).map(|(_, t)| t).collect()
}

/// The storey each element stands on, by path: the nearest storey above it in the tree.
/// An element under no storey is not in the map.
pub fn storeys_of(job: &mut Job) -> HashMap<String, String> {
    let tree = Tree::of(&mut job.world);
    tree.parents.keys().filter_map(|p| tree.storey_of(p).map(|s| (p.clone(), s.to_string()))).collect()
}

/// What a storey's walls, floors and rooms come to.
#[derive(Clone, Debug)]
pub struct StoreyTotals {
    pub path: String,
    pub name: String,
    pub elevation: Option<f64>,
    pub walls: Sum,
    pub wall_face: Sum,
    pub wall_net: Sum,
    pub floors: Sum,
    /// Its rooms: the floor inside their walls, added up.
    pub rooms: Sum,
    pub doors: usize,
    pub windows: usize,
    /// Its walls again, by the type of wall, in the order each type first appears: a
    /// 6 in outside wall and a 4 in partition are not one number to an estimator.
    pub walls_by_type: Vec<WallsOfType>,
    /// Its strip footings: their length, and the concrete in them. Never in the walls.
    pub footings: Sum,
    pub footing_volume: Sum,
    /// What of the above is not new work, given three ways (call 158): to build,
    /// existing to remain, to demolish. Empty where nothing on the level is marked.
    pub by_work: StoreyByWork,
    /// What is set down on it: equipment, fixtures and furnishings, by kind, counted here (call 197 point 10).
    pub set_down: SetDown,
}

/// What is set down on one level: equipment, fixtures and furnishings, by kind in the order each kind first
/// appears on it, counted here and nowhere else (call 197 point 10; contract 205M part 2). A fitting is joined to a
/// run, not set down, and is in its system's list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SetDown {
    pub kinds: Vec<SetDownKind>,
}

/// One kind of thing set down on one level.
#[derive(Clone, Debug, PartialEq)]
pub struct SetDownKind {
    pub kind: Kind,
    pub count: usize,
    /// Each thing's name, in the order they stand in the job.
    pub names: Vec<String>,
    pub paths: Vec<String>,
    /// As sure as the worst of them: what each is drawn from (`Placement::status`); a thing with no size is Unknown.
    pub status: Status,
    /// How many of them have nothing priced on them (the rule `view::pricing` counts by).
    pub not_priced: usize,
    /// How many have no shape, so are not drawn: their size is not known.
    pub not_drawn: usize,
}

impl SetDown {
    /// How many things are set down on the level.
    pub fn count(&self) -> usize {
        self.kinds.iter().map(|k| k.count).sum()
    }
    /// How many of them have nothing priced on them.
    pub fn not_priced(&self) -> usize {
        self.kinds.iter().map(|k| k.not_priced).sum()
    }
    /// How many of them cannot be drawn.
    pub fn not_drawn(&self) -> usize {
        self.kinds.iter().map(|k| k.not_drawn).sum()
    }
    /// What the sheet window says under "Set down on this level", line by line: each kind with its names, then what is
    /// not priced, then what cannot be drawn. A level with nothing set down says so.
    pub fn said(&self) -> Vec<String> {
        if self.kinds.is_empty() {
            return vec!["Nothing is set down on this level.".to_string()];
        }
        let mut out: Vec<String> = self.kinds.iter().map(|k| format!("{}: {}", k.kind.so_many(k.count), k.names.join(", "))).collect();
        let bare: Vec<&SetDownKind> = self.kinds.iter().filter(|k| k.not_priced > 0).collect();
        if !bare.is_empty() {
            let n = self.not_priced();
            let kinds: Vec<String> = bare.iter().map(|k| k.kind.so_many(k.not_priced)).collect();
            out.push(format!("{} {} set down and not priced.", kinds.join(", "), if n == 1 { "is" } else { "are" }));
        }
        let undrawn: Vec<&SetDownKind> = self.kinds.iter().filter(|k| k.not_drawn > 0).collect();
        if !undrawn.is_empty() {
            let kinds: Vec<String> = undrawn.iter().map(|k| k.kind.so_many(k.not_drawn)).collect();
            out.push(format!("{} cannot be drawn: its size is not known.", kinds.join(", ")));
        }
        out
    }
}

/// A level's counts given three ways where something on that line is not new work (call
/// 158). Each is None where everything on the line is new, and the line prints as it did.
/// Each is the sum of the same figures the line itself sums.
#[derive(Clone, Debug, Default)]
pub struct StoreyByWork {
    pub walls: Option<WorkSplit>,
    pub wall_face: Option<WorkSplit>,
    pub wall_net: Option<WorkSplit>,
    pub floors: Option<WorkSplit>,
    pub footings: Option<WorkSplit>,
    pub footing_volume: Option<WorkSplit>,
    /// Doors, and windows: how many to build, existing to remain, to demolish.
    pub doors: Option<[usize; 3]>,
    pub windows: Option<[usize; 3]>,
}

/// The walls of one type on one level, added up.
#[derive(Clone, Debug, PartialEq)]
pub struct WallsOfType {
    /// The type, by path and by name; a wall whose type is not known is under "no type".
    pub path: String,
    pub name: String,
    pub length: Sum,
    pub face: Sum,
    pub net: Sum,
}

pub fn storey_totals(job: &mut Job) -> Vec<StoreyTotals> {
    let order = job.order.clone();
    // What is drawn, and how sure each is: a thing set down with no shape is not in this list.
    let drawn_status: HashMap<String, Status> = placements(job).0.into_iter().map(|p| (p.path, p.status)).collect();
    let w = &job.world;
    let parents: HashMap<String, Option<String>> = order.iter().filter_map(|e| w.get::<Node>(*e)).map(|n| (n.path.clone(), n.parent.clone())).collect();
    let storeys: Vec<String> = order.iter().filter(|e| w.get::<Kind>(**e) == Some(&Kind::Storey)).filter_map(|e| w.get::<Node>(*e)).map(|n| n.path.clone()).collect();
    // An element counts on the nearest storey above it in the tree, and on no other.
    let under = |path: &str, storey: &str| {
        let mut at = parents.get(path).cloned().flatten();
        let mut steps = 0;
        while let Some(p) = at {
            if storeys.contains(&p) {
                return p == storey;
            }
            at = parents.get(&p).cloned().flatten();
            steps += 1;
            if steps > parents.len() {
                break;
            }
        }
        false
    };
    let mut out = Vec::new();
    for &s in &order {
        let (Some(sn), Some(Kind::Storey)) = (w.get::<Node>(s), w.get::<Kind>(s)) else { continue };
        let mut figures: [Vec<(String, Option<f64>, Status)>; 7] = Default::default();
        let (mut doors, mut windows) = (0, 0);
        let mut places: [Vec<usize>; 7] = Default::default();
        let (mut door_places, mut window_places) = ([0usize; 3], [0usize; 3]);
        // Each wall type met, in order: its path, and its walls' three figures.
        #[allow(clippy::type_complexity)]
        let mut of_type: Vec<(String, [Vec<(String, Option<f64>, Status)>; 3])> = Vec::new();
        for &e in &order {
            let (Some(n), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e)) else { continue };
            if !under(&n.path, &sn.path) {
                continue;
            }
            let figure = |name: &str| w.get::<Measure>(e).and_then(|m| m.0.get(name)).map(|d| (n.path.clone(), d.value, d.status)).unwrap_or((n.path.clone(), None, Status::Unknown));
            let place = view::work_place(w, e);
            match kind {
                Kind::Wall => {
                    for p in &mut places[..3] {
                        p.push(place);
                    }
                    figures[0].push(figure("plan_length"));
                    figures[1].push(figure("face_area"));
                    figures[2].push(figure("net_area"));
                    let its_type = w.get::<Wall>(e).and_then(|wall| wall.type_name.value.clone()).unwrap_or_default();
                    let at = match of_type.iter().position(|(t, _)| *t == its_type) {
                        Some(at) => at,
                        None => {
                            of_type.push((its_type, Default::default()));
                            of_type.len() - 1
                        }
                    };
                    for (i, name) in ["plan_length", "face_area", "net_area"].into_iter().enumerate() {
                        of_type[at].1[i].push(figure(name));
                    }
                }
                Kind::Footing => {
                    places[5].push(place);
                    places[6].push(place);
                    figures[5].push(figure("plan_length"));
                    figures[6].push(figure("volume"));
                }
                Kind::Floor => {
                    places[3].push(place);
                    figures[3].push(figure("plan_area"))
                }
                Kind::Room => figures[4].push(figure("floor_area")),
                Kind::Door => {
                    doors += 1;
                    door_places[place] += 1;
                }
                Kind::Window => {
                    windows += 1;
                    window_places[place] += 1;
                }
                _ => {}
            }
        }
        let mut set_down = SetDown::default();
        for &e in &order {
            let (Some(n), Some(kind)) = (w.get::<Node>(e), w.get::<Kind>(e).copied()) else { continue };
            if !matches!(kind, Kind::Equipment | Kind::Fixture | Kind::Furnishing) || !under(&n.path, &sn.path) {
                continue;
            }
            let status = drawn_status.get(&n.path).copied();
            let bare = view::nothing_priced_on(w, e);
            let at = match set_down.kinds.iter().position(|k| k.kind == kind) {
                Some(at) => at,
                None => {
                    set_down.kinds.push(SetDownKind { kind, count: 0, names: Vec::new(), paths: Vec::new(), status: Status::Known, not_priced: 0, not_drawn: 0 });
                    set_down.kinds.len() - 1
                }
            };
            let k = &mut set_down.kinds[at];
            k.count += 1;
            k.names.push(n.name.clone());
            k.paths.push(n.path.clone());
            k.status = k.status.max(status.unwrap_or(Status::Unknown));
            k.not_priced += usize::from(bare);
            k.not_drawn += usize::from(status.is_none());
        }
        out.push(StoreyTotals {
            path: sn.path.clone(),
            name: sn.name.clone(),
            elevation: w.get::<Level>(s).and_then(|l| l.elevation.feet()),
            walls: view::sum_of_what_is_there("LF", &figures[0]),
            wall_face: view::sum_of_what_is_there("SF", &figures[1]),
            wall_net: view::sum_of_what_is_there("SF", &figures[2]),
            floors: view::sum_of_what_is_there("SF", &figures[3]),
            rooms: view::sum_of_what_is_there("SF", &figures[4]),
            doors,
            windows,
            footings: view::sum_of_what_is_there("LF", &figures[5]),
            footing_volume: view::sum_of_what_is_there("CY", &figures[6]),
            set_down,
            by_work: StoreyByWork {
                walls: view::split_by_work("LF", &figures[0], &places[0]),
                wall_face: view::split_by_work("SF", &figures[1], &places[1]),
                wall_net: view::split_by_work("SF", &figures[2], &places[2]),
                floors: view::split_by_work("SF", &figures[3], &places[3]),
                footings: view::split_by_work("LF", &figures[5], &places[5]),
                footing_volume: view::split_by_work("CY", &figures[6], &places[6]),
                doors: (door_places[1] + door_places[2] > 0).then_some(door_places),
                windows: (window_places[1] + window_places[2] > 0).then_some(window_places),
            },
            walls_by_type: of_type
                .into_iter()
                .map(|(path, f)| {
                    let name = order.iter().filter_map(|e| w.get::<Node>(*e)).find(|n| n.path == path).map(|n| n.name.clone()).unwrap_or_else(|| "no type".to_string());
                    WallsOfType { path, name, length: view::sum_of("LF", &f[0]), face: view::sum_of("SF", &f[1]), net: view::sum_of("SF", &f[2]) }
                })
                .collect(),
        });
    }
    out
}
