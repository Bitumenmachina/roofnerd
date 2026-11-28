use super::*;

// ---------------------------------------------------------------- what is worked out

pub(super) const NOT_CONVERTED: &str = "is in a unit this program does not convert";

pub(super) fn feet_text(name: &str, v: &Val<Len>) -> String {
    units::to_feet_text(name, &v.unit)
}

/// One opening as its wall sees it: where it starts and ends along the wall, its sill and
/// its head, in feet, with the worst status among them. None when a value is missing.
pub(super) struct Hole {
    pub(super) path: String,
    pub(super) span: Option<[f64; 4]>,
    pub(super) area: Option<f64>,
    /// The worst status among everything about it: its type, its size, where it is.
    pub(super) status: Status,
    /// The worst status among what its area alone rests on: its type and its size.
    pub(super) area_status: Status,
    /// What its area was worked out from.
    pub(super) inputs: Vec<String>,
}

/// A door or window worked out against its type: the hole it makes, and its type's width
/// and height, or why it has none.
pub(super) fn hole_of(tree: &Tree, me: &str, o: &Opening) -> (Hole, Result<(Val<Len>, Val<Len>), String>) {
    let mut inputs = vec![format!("{me}#{K_OPENING}")];
    match typed(&tree.opening_types, &o.type_name, "a door or window") {
        Err(why) => (Hole { path: me.into(), span: None, area: None, status: Status::Unknown, area_status: Status::Unknown, inputs }, Err(why)),
        Ok((type_path, t)) => {
            inputs.push(format!("{type_path}#{K_OPENING_TYPE}"));
            let (w, h) = (t.width.feet(), t.height.feet());
            let span = o.offset.feet().zip(w).zip(o.sill.feet().zip(h)).map(|((at, w), (s, h))| [at - w / 2.0, at + w / 2.0, s, s + h]);
            let status = [o.type_name.status, t.width.status, t.height.status, o.offset.status, o.sill.status].into_iter().max().unwrap_or(Status::Unknown);
            let area = w.zip(h).map(|(w, h)| w * h);
            let area_status = o.type_name.status.max(t.width.status).max(t.height.status);
            (Hole { path: me.into(), span, area, status, area_status, inputs }, Ok((t.width.clone(), t.height.clone())))
        }
    }
}

/// How wide each door and window is, in feet, by path: its type's width. One whose type
/// or width is not known is left out (call 170: a wall moved asks whether its openings
/// still lie within it).
pub(crate) fn opening_widths(job: &mut Job) -> HashMap<String, f64> {
    let tree = Tree::of(&mut job.world);
    let mut out = HashMap::new();
    let mut q = job.world.query::<(&Node, &Kind, &Opening)>();
    for (n, k, o) in q.iter(&job.world) {
        if k.is_opening() {
            if let Some(w) = typed(&tree.opening_types, &o.type_name, "a door or window").ok().and_then(|(_, t)| t.width.feet()) {
                out.insert(n.path.clone(), w);
            }
        }
    }
    out
}

/// A wall's base and its height, worked out from its storey, its top level and its
/// offsets, each with why it has none when it has none.
pub(crate) struct WallHeights {
    /// Its base above the datum.
    pub(crate) base: Result<f64, String>,
    pub(crate) height: Result<f64, String>,
    /// The worst status among what the height was worked out from.
    pub(crate) status: Status,
    /// The worst status among everything its base and its top rest on: its storey's
    /// elevation and its base offset as well as what its height does. It is what "does
    /// this wall stand at such a height?" is no surer than.
    pub(crate) span: Status,
    /// What its base alone rests on: its storey's elevation and its base offset. A wall
    /// whose base is Known and whose top is not may reach higher or lower, and starts
    /// where it starts.
    pub(crate) base_status: Status,
    pub(crate) formula: String,
    pub(crate) inputs: Vec<String>,
}

/// BIMCraft's `own_base` and `own_top` (`model.rs:3890`, `3896`): a wall's base is its
/// level plus its base offset; its top is its top level plus its top offset, or its base
/// plus its height.
pub(crate) fn wall_heights(tree: &Tree, me: &str, w: &Wall) -> WallHeights {
    let mut inputs = vec![format!("{me}#{K_WALL}")];
    let storey = tree.storey_of(me);
    let own = storey.and_then(|s| tree.levels.get(s));
    let own_elevation: Result<f64, String> = match (storey, own) {
        (None, _) => Err("it is under no level".into()),
        (Some(_), None) => Err("its level has no level entered".into()),
        (Some(_), Some(l)) => l.elevation.feet().ok_or_else(|| "its level's elevation is not known".to_string()),
    };
    let (base_offset, base_status): (Result<f64, String>, Status) = match &w.base_offset {
        None => (Ok(0.0), Status::Known),
        Some(o) => (o.feet().ok_or_else(|| "its base offset is not known".to_string()), o.status),
    };
    let base = own_elevation.clone().and_then(|e| base_offset.clone().map(|o| e + o));
    let (height, status, formula) = match (&w.top_level, &w.height) {
        (None, Some(h)) => (h.feet().ok_or_else(|| "its height is not known".to_string()), h.status, units::to_feet_text("height", &h.unit)),
        (Some(t), _) => {
            let top = t.value.as_ref().ok_or_else(|| "its top level is not known".to_string()).and_then(|p| {
                inputs.push(format!("{p}#{K_LEVEL}"));
                let l = tree.levels.get(p).ok_or_else(|| format!("its top level, {p}, is not a level with a level in this job"))?;
                let e = l.elevation.feet().ok_or_else(|| format!("the elevation of {}, its top level, is not known", tree.called(p)))?;
                Ok((e, l.elevation.status))
            });
            if let Some(s) = storey {
                inputs.push(format!("{s}#{K_LEVEL}"));
            }
            let (top_offset, top_offset_status) = match &w.top_offset {
                None => (Ok(0.0), Status::Known),
                Some(o) => (o.feet().ok_or_else(|| "its top offset is not known".to_string()), o.status),
            };
            let own_status = own.map(|l| l.elevation.status).unwrap_or(Status::Unknown);
            let status = [t.status, top.as_ref().map(|x| x.1).unwrap_or(Status::Unknown), own_status, base_status, top_offset_status].into_iter().max().unwrap_or(Status::Unknown);
            let height = top.and_then(|(e, _)| top_offset.map(|o| e + o)).and_then(|top| base.clone().map(|b| top - b));
            let mut f = "elevation of its top level".to_string();
            if w.top_offset.is_some() {
                f.push_str(" + top_offset");
            }
            f.push_str(" − elevation of its level");
            if w.base_offset.is_some() {
                f.push_str(" − base_offset");
            }
            (height, status, f)
        }
        (None, None) => (Err("it has no top level and no height".into()), Status::Unknown, String::new()),
    };
    let height = height.and_then(|h| if h > 0.0 { Ok(h) } else { Err(format!("its top is {} above its base; a wall's top is above its base", crate::units::length_said(h))) });
    let own_status = own.map(|l| l.elevation.status).unwrap_or(Status::Unknown);
    let span = status.max(own_status).max(base_status);
    WallHeights { base, height, status, span, base_status: own_status.max(base_status), formula, inputs }
}

/// A wall's thickness and its layers (function and thickness, exterior first) in feet, or
/// why they are not to be had. The layers are empty when a layer's thickness is not known
/// and the wall's line does not need them.
pub(crate) fn wall_section(tree: &Tree, w: &Wall) -> Result<(f64, Vec<(Function, f64)>, Status), String> {
    let (_, t) = typed(&tree.wall_types, &w.type_name, "a wall")?;
    let thickness = t.thickness.feet().ok_or("its type's thickness is not known")?;
    let layers: Option<Vec<(Function, f64)>> = t.layers.iter().map(|l| l.thickness.feet().map(|x| (l.function, x))).collect();
    let needs_core = matches!(w.location, Location::CoreCentre | Location::ExteriorCore | Location::InteriorCore);
    let layers = match layers {
        Some(l) => l,
        None if needs_core => return Err("a layer of its type has no thickness, so its core line cannot be placed".into()),
        None => Vec::new(),
    };
    let status = t.layers.iter().map(|l| l.thickness.status).fold(w.type_name.status.max(t.thickness.status), Status::max);
    Ok((thickness, layers, status))
}

/// The figures of every element this module knows, by path. `derive` adds them to the
/// element's measure beside whatever else it carries.
pub fn measures(world: &mut World) -> HashMap<String, BTreeMap<String, Derived>> {
    let mut out: HashMap<String, BTreeMap<String, Derived>> = HashMap::new();
    // A door or window to demolish in a wall that stays is filled in: it is no hole in
    // its wall when the work is done (call 150).
    let gone = crate::components::demolished(world);
    let tree = Tree::of(world);

    // Openings first: each one's size and area, and each wall's list of them.
    let mut holes: HashMap<String, Vec<Hole>> = HashMap::new();
    // In the order of their paths, whatever order the world holds them in: a wall's
    // list of openings, and so what its figures cite, is the same every time.
    let mut q = world.query::<(&Node, &Kind, Option<&Opening>, Option<&Cost>)>();
    let mut openings: Vec<(&Node, &Kind, Option<&Opening>, Option<&Cost>)> = q.iter(world).collect();
    openings.sort_by_key(|(n, ..)| path_number(&n.path));
    for (node, kind, opening, cost) in openings {
        if !kind.is_opening() {
            continue;
        }
        let me = &node.path;
        let mut m = BTreeMap::new();
        let hole = match opening {
            None => {
                let input = [format!("{me}#{K_OPENING}")];
                for (name, unit) in kind.measures().iter().filter(|(n, _)| !crate::trim::GIVEN_WHERE_NAMED.contains(n)) {
                    m.insert(name.to_string(), Derived::refused(unit, "", &input, "no opening entered"));
                }
                Hole { path: me.clone(), span: None, area: None, status: Status::Unknown, area_status: Status::Unknown, inputs: input.to_vec() }
            }
            Some(o) => {
                let (hole, sizes) = hole_of(&tree, me, o);
                match sizes {
                    Err(why) => {
                        m.insert("width".into(), Derived::refused("ft", "its type's width", &hole.inputs, &why));
                        m.insert("height".into(), Derived::refused("ft", "its type's height", &hole.inputs, &why));
                        m.insert("opening_area".into(), Derived::refused("SF", "width × height", &hole.inputs, &why));
                    }
                    Ok((width, height)) => {
                        let sure = o.type_name.status;
                        m.insert("width".into(), Derived::new(width.feet(), "ft", &[sure, width.status], &units::to_feet_text("its type's width", &width.unit), &hole.inputs));
                        m.insert("height".into(), Derived::new(height.feet(), "ft", &[sure, height.status], &units::to_feet_text("its type's height", &height.unit), &hole.inputs));
                        m.insert("opening_area".into(), Derived::new(hole.area, "SF", &[sure, width.status, height.status], "width × height", &hole.inputs));
                    }
                }
                hole
            }
        };
        // Its casing, where a line of it names it (call 185).
        if crate::trim::names(cost, crate::trim::CASING_LENGTH) {
            let casing = crate::trim::casing(*kind, m.get("width"), m.get("height"), &hole.inputs);
            m.insert(crate::trim::CASING_LENGTH.into(), casing);
        }
        out.entry(me.clone()).or_default().extend(m);
        if let Some(parent) = node.parent.as_ref().filter(|_| !gone.contains(me)) {
            holes.entry(parent.clone()).or_default().push(hole);
        }
    }

    // Every strip footing's level, line and width: where one stops on the run of
    // another (a T), the part of it inside the other is that other's concrete.
    let strips: Vec<footings::Strip> = {
        let mut fq = world.query::<(&Node, &Kind, &Wall)>();
        fq.iter(world)
            .filter(|(_, k, _)| **k == Kind::Footing)
            .filter_map(|(n, _, w)| {
                let line = w.line_ft()?;
                let width = typed(&tree.wall_types, &w.type_name, "a footing").ok().and_then(|(_, t)| t.thickness.feet())?;
                let order = n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u32::MAX);
                Some(footings::Strip { path: n.path.clone(), level: n.parent.clone(), line, width, depth: w.height.as_ref().and_then(|h| h.feet()), top: w.base_offset.as_ref().and_then(|o| o.feet()).unwrap_or(0.0), order })
            })
            .collect()
    };
    // Every thing set in place, for a countertop to see what stands under it (call 162).
    let set_in_place: Vec<(String, String, Option<String>, Placed)> = {
        let mut sq = world.query::<(&Node, &Kind, &Placed)>();
        sq.iter(world).filter(|(_, k, _)| matches!(**k, Kind::Furnishing | Kind::Equipment | Kind::Fixture)).map(|(n, _, p)| (n.path.clone(), p.what.value.clone().unwrap_or_else(|| n.name.clone()), tree.storey_of(&n.path).map(str::to_string), p.clone())).collect()
    };
    // Each wall's outside corners, where a line of it names them: worked from every wall of its level (call 185).
    let corner_figures = {
        let wanted = crate::trim::naming(world, &[Kind::Wall], &[crate::trim::OUTSIDE_CORNERS, crate::trim::CORNER_BOARD_LENGTH]);
        if wanted.is_empty() { HashMap::new() } else { crate::trim::corners(world, Some(&wanted)) }
    };
    // Each wall's gable (call 192), where a line of it names the area or where its frame is counted, so that the frame can say the
    // gable's is not. The area is given on the wall only where a line names it.
    let (gable_named, gable_framed) = crate::gable::asked_of(world);
    let gable_figures = {
        let wanted: HashSet<String> = gable_named.union(&gable_framed).cloned().collect();
        if wanted.is_empty() { HashMap::new() } else { crate::gable::gables(world, Some(&wanted)) }
    };
    let mut q = world.query::<(&Node, &Kind, Option<&Wall>, Option<&Floor>, Option<&Segment>, Option<&Cost>, Option<&MadeOf>, Option<&Condition>, Option<&Placed>, Option<&Stood>, Option<&JoistsSpan>)>();
    for (node, kind, wall, floor, segment, cost, made_of, condition, placed, column, joists_span) in q.iter(world) {
        let me = &node.path;
        let mut m = BTreeMap::new();
        match kind {
            Kind::Wall => {
                let input = [format!("{me}#{K_WALL}")];
                let f_len = format!("length of the wall's line on the plan{}", wall.filter(|w| w.line.unit == units::IN).map(|_| " ÷ 12 (entered in inches)").unwrap_or_default());
                let Some(w) = wall else {
                    for (name, unit) in kind.measures().iter().filter(|(n, _)| !crate::trim::GIVEN_WHERE_NAMED.contains(n)) {
                        m.insert(name.to_string(), Derived::refused(unit, "", &input, "no wall line entered"));
                    }
                    out.entry(me.clone()).or_default().extend(m);
                    continue;
                };
                let line = w.line_ft();
                let length = line.map(|[a, b]| geom::dist(a, b));
                let unit_fault = w.line.value.is_some() && line.is_none();
                let plan = if unit_fault { Derived::refused("LF", &f_len, &input, &format!("the wall line {NOT_CONVERTED}")) } else { Derived::new(length, "LF", &[w.line.status], &f_len, &input) };
                // Its height: entered, or from its storey up to its top level.
                let heights = wall_heights(&tree, me, w);
                let tall = match &heights.height {
                    Ok(h) => Derived::new(Some(*h), "ft", &[heights.status], &heights.formula, &heights.inputs),
                    Err(why) => Derived::refused("ft", &heights.formula, &heights.inputs, why),
                };
                let f_face = "plan_length × height";
                let face = match (&heights.height, unit_fault) {
                    (_, true) => Derived::refused("SF", f_face, &heights.inputs, &format!("the wall line {NOT_CONVERTED}")),
                    (Err(why), _) => Derived::refused("SF", f_face, &heights.inputs, why),
                    (Ok(h), _) => Derived::new(length.map(|l| l * h), "SF", &[w.line.status, heights.status], f_face, &heights.inputs),
                };
                // Its openings: every one has to have an area, lie within the wall, and
                // not lie over another, or the wall has no net area.
                let mine = holes.get(me).map(Vec::as_slice).unwrap_or(&[]);
                let mut inputs: Vec<String> = mine.iter().flat_map(|h| h.inputs.iter().cloned()).collect();
                inputs.dedup();
                let f_open = if mine.is_empty() { "the openings in this wall (it has none)".to_string() } else { if mine.len() == 1 { "the opening_area of the one opening in this wall".to_string() } else { format!("the opening_area of each of the {} openings in this wall, added up", mine.len()) } };
                let total = mine.iter().map(|h| h.area).sum::<Option<f64>>();
                // What the openings come to is as sure as their sizes. Where each one is
                // matters to the net, not to this.
                let mut statuses: Vec<Status> = mine.iter().map(|h| if h.area.is_some() { h.area_status } else { Status::Unknown }).collect();
                statuses.push(Status::Known);
                let open = match mine.iter().find(|h| h.area.is_none()) {
                    Some(h) => Derived::refused("SF", &f_open, &inputs, &format!("{} has no area", h.path)),
                    None => Derived::new(total, "SF", &statuses, &f_open, &inputs),
                };
                for i in heights.inputs.iter().rev() {
                    if !inputs.contains(i) {
                        inputs.insert(0, i.clone());
                    }
                }
                // Every opening that does not fit is named, not the first alone.
                let mut faults: Vec<String> = Vec::new();
                // An opening that has a size and no place cannot be checked for fit.
                for h in mine.iter().filter(|h| h.area.is_some() && h.span.is_none()) {
                    faults.push(format!("where {} is in the wall is not known, so whether it fits cannot be checked", h.path));
                }
                if let (Some(l), Ok(h)) = (length, &heights.height) {
                    for (i, hole) in mine.iter().enumerate() {
                        let Some([from, to, sill, head]) = hole.span else { continue };
                        // It fits its wall from end to end (BIMCraft `model.rs:6678`).
                        if from < -crate::check::FIT_TOLERANCE_FT {
                            faults.push(format!("{} starts {} before the start of its wall", hole.path, crate::units::length_said(-from)));
                        } else if to > l + crate::check::FIT_TOLERANCE_FT {
                            faults.push(format!("{} ends {} along a wall {} long", hole.path, crate::units::length_said(to), crate::units::length_said(l)));
                        } else if head > h + crate::check::FIT_TOLERANCE_FT {
                            faults.push(format!("{} reaches {} up a wall {} tall", hole.path, crate::units::length_said(head), crate::units::length_said(*h)));
                        } else if let Some(other) = mine[..i].iter().find(|o| o.span.is_some_and(|s| from < s[1] && s[0] < to && sill < s[3] && s[2] < head)) {
                            faults.push(format!("{} and {} lie over each other", other.path, hole.path));
                        }
                    }
                }
                // An opening whose place is not known still has its size: the openings' area
                // stands, and it is the net that cannot be had. One that is known not to fit
                // (past an end, over the top, over another) is another matter: the sum of
                // the openings is then not what is cut out of this wall, and is not given.
                let misfit: Vec<&String> = faults.iter().filter(|f| !f.starts_with("where ")).collect();
                let open = match (misfit.is_empty(), open) {
                    (false, o) if o.value.is_some() => Derived::refused("SF", &f_open, &o.inputs, &misfit.iter().map(|f| f.as_str()).collect::<Vec<_>>().join("; ")),
                    (_, o) => o,
                };
                let fault = (!faults.is_empty()).then(|| faults.join("; "));
                // Its length less what comes down to its floor (call 175): a door, and a window
                // whose sill is at the floor. Given only where a line of this wall's names it,
                // as the frame's figures are: a wall that prices nothing on it has no new figure.
                let names_net_length = cost.is_some_and(|c| c.lines.iter().any(|l| !l.off && crate::formula::parse(&l.quantity).is_ok_and(|e| crate::formula::names(&e).iter().any(|n| n == "net_length"))));
                let net_length = if !names_net_length {
                    None
                } else {
                    let f_nl = "plan_length − the widths of the doors and of the windows whose sill is at the floor";
                    Some(match (&fault, length) {
                        _ if unit_fault => Derived::refused("LF", f_nl, &inputs, &format!("the wall line {NOT_CONVERTED}")),
                        (Some(why), _) => Derived::refused("LF", f_nl, &inputs, why),
                        (None, None) => Derived::refused("LF", f_nl, &inputs, "the wall has no length"),
                        (None, Some(l)) => match mine.iter().find(|h| h.span.is_none()) {
                            Some(h) => Derived::refused("LF", f_nl, &inputs, &format!("{} has no size, so how much of the wall it takes is not known", h.path)),
                            None => {
                                let cut: f64 = mine.iter().filter_map(|h| h.span).filter(|s| s[2] <= crate::check::FIT_TOLERANCE_FT).map(|s| s[1] - s[0]).sum();
                                let mut s: Vec<Status> = mine.iter().map(|h| h.status).collect();
                                s.push(w.line.status);
                                Derived::new(Some(l - cut), "LF", &s, f_nl, &inputs)
                            }
                        },
                    })
                };
                // The net area of the wall below a height a line of it names (call 179, C2): its face up
                // to that height, less the parts of its openings that lie below it. A stone wainscot's
                // 3 ft is such a height, and the siding is the wall's net area less this. The height is
                // a length entered on the line (`net_area_below_wainscot` takes the line's `wainscot`);
                // it is as sure as that entry. Given only where a line names it.
                let mut below: Vec<(String, Derived)> = Vec::new();
                for l in cost.iter().flat_map(|c| c.lines.iter()).filter(|l| !l.off) {
                    let Ok(e) = crate::formula::parse(&l.quantity) else { continue };
                    for name in crate::formula::names(&e).into_iter().filter(|n| n.starts_with(crate::formula::BELOW)) {
                        if below.iter().any(|(n, _)| *n == name) {
                            // A second line with the same name: the two have to give one height.
                            let first = below.iter().position(|(n, _)| *n == name).unwrap_or(0);
                            let height_of = |x: &crate::components::CostLine| x.inputs.get(&name[crate::formula::BELOW.len()..]).map(|v| v.feet());
                            let one = cost.iter().flat_map(|c| c.lines.iter()).filter(|x| !x.off && crate::formula::parse(&x.quantity).is_ok_and(|e| crate::formula::names(&e).contains(&name))).map(height_of).collect::<Vec<_>>();
                            if one.windows(2).any(|w| w[0] != w[1]) {
                                let f_below = "the face up to that height less the parts of its openings below it";
                                below[first].1 = Derived::refused("SF", f_below, &inputs, &format!("two lines of this wall give {:?} two heights", &name[crate::formula::BELOW.len()..]));
                            }
                            continue;
                        }
                        let input_name = &name[crate::formula::BELOW.len()..];
                        let f_below = "the face up to that height less the parts of its openings below it";
                        let figure = match l.inputs.get(input_name) {
                            None => Derived::refused("SF", f_below, &inputs, &format!("{:?} is not a height entered on the line", crate::view::words(input_name))),
                            Some(hv) => match (&fault, length, hv.feet()) {
                                _ if unit_fault => Derived::refused("SF", f_below, &inputs, &format!("the wall line {NOT_CONVERTED}")),
                                (Some(why), _, _) => Derived::refused("SF", f_below, &inputs, why),
                                (None, None, _) => Derived::refused("SF", f_below, &inputs, "the wall has no length"),
                                (None, _, None) => Derived::refused("SF", f_below, &inputs, &format!("the height {:?} is not known", crate::view::words(input_name))),
                                (None, Some(l_ft), Some(h_ft)) => match mine.iter().find(|h| h.span.is_none()) {
                                    Some(h) => Derived::refused("SF", f_below, &inputs, &format!("{} has no size, so how much of the wall it takes is not known", h.path)),
                                    None => {
                                        let cut: f64 = mine.iter().filter_map(|h| h.span).map(|[from, to, sill, head]| (to - from) * (head.clamp(0.0, h_ft) - sill.clamp(0.0, h_ft))).sum();
                                        let mut st: Vec<Status> = mine.iter().map(|h| h.status).collect();
                                        st.extend([w.line.status, hv.status]);
                                        Derived::new(Some(l_ft * h_ft - cut), "SF", &st, f_below, &inputs)
                                    }
                                },
                            },
                        };
                        below.push((name, figure));
                    }
                }
                // The length of the wall's corner boards above a height a line of it names (call 194): at each outside
                // corner the board's length less that height, never less than nothing, added up. A stone wainscot that turns
                // the corner takes the boards up from its top. The height is a length entered on the line
                // (`corner_board_length_above_wainscot` takes the line's `wainscot`), and the figure is as sure as that entry
                // and as the corner boards' own length. Given only where a line names it.
                let mut above: Vec<(String, Derived)> = Vec::new();
                for l in cost.iter().flat_map(|c| c.lines.iter()).filter(|l| !l.off) {
                    let Ok(e) = crate::formula::parse(&l.quantity) else { continue };
                    for name in crate::formula::names(&e).into_iter().filter(|n| n.starts_with(crate::formula::ABOVE) && n.len() > crate::formula::ABOVE.len()) {
                        let f_above = "for each outside corner, the corner board's length less the height the line enters, never less than nothing, added up";
                        let input_name = &name[crate::formula::ABOVE.len()..];
                        if let Some(first) = above.iter().position(|(n, _)| *n == name) {
                            // A second line with the same name: the two have to give one height.
                            let height_of = |x: &crate::components::CostLine| x.inputs.get(input_name).map(|v| v.feet());
                            let one = cost.iter().flat_map(|c| c.lines.iter()).filter(|x| !x.off && crate::formula::parse(&x.quantity).is_ok_and(|e| crate::formula::names(&e).contains(&name))).map(height_of).collect::<Vec<_>>();
                            if one.windows(2).any(|w| w[0] != w[1]) {
                                above[first].1 = Derived::refused("LF", f_above, &inputs, &format!("two lines of this wall give {:?} two heights", crate::view::words(input_name)));
                            }
                            continue;
                        }
                        let figure = match (l.inputs.get(input_name), corner_figures.get(me)) {
                            (None, _) => Derived::refused("LF", f_above, &inputs, &format!("{:?} is not a height entered on the line", crate::view::words(input_name))),
                            _ if unit_fault => Derived::refused("LF", f_above, &inputs, &format!("the wall line {NOT_CONVERTED}")),
                            (Some(_), None) => Derived::refused("LF", f_above, &inputs, "the wall's outside corners are not worked"),
                            (Some(hv), Some(corner)) => match (hv.feet(), corner.count.worked(), corner.board.worked()) {
                                (None, _, _) => Derived::refused("LF", f_above, &inputs, &format!("the height {:?} is not known", crate::view::words(input_name))),
                                (_, None, _) => Derived::refused("LF", f_above, &inputs, if corner.count.note.is_empty() { "the wall's outside corners are not worked" } else { &corner.count.note }),
                                (_, _, None) => Derived::refused("LF", f_above, &inputs, if corner.board.note.is_empty() { "the wall's corner boards are not worked" } else { &corner.board.note }),
                                (Some(h_ft), Some(_), Some(_)) => {
                                    let mut rests = inputs.clone();
                                    rests.extend(corner.board.inputs.iter().filter(|i| !inputs.contains(i)).cloned());
                                    // Each corner's board is as tall as the wall is there (call 204); the entry is held between nothing and
                                    // that height, corner by corner.
                                    let up: f64 = corner.up.iter().map(|(_, h)| h - h_ft.clamp(0.0, *h)).sum();
                                    Derived::new(Some(up), "LF", &[corner.board.status, hv.status], f_above, &rests)
                                }
                            },
                        };
                        above.push((name, figure));
                    }
                }
                let f_net = "face_area − opening_area";
                let net = match (fault, face.value, open.value) {
                    (Some(why), _, _) => Derived::refused("SF", f_net, &inputs, &why),
                    (None, Some(a), Some(b)) => {
                        // The net rests on where each opening is as well as on its size.
                        let mut s: Vec<Status> = mine.iter().map(|h| h.status).collect();
                        s.extend([face.status, open.status]);
                        Derived::new(Some(a - b), "SF", &s, f_net, &inputs)
                    }
                    (None, None, _) => Derived::refused("SF", f_net, &inputs, if face.note.is_empty() { "the wall has no face area" } else { &face.note }),
                    (None, _, None) => Derived::refused("SF", f_net, &inputs, "an opening in this wall has no area"),
                };
                m.insert("plan_length".into(), plan);
                m.insert("height".into(), tall);
                m.insert("face_area".into(), face);
                m.insert("opening_area".into(), open);
                m.insert("net_area".into(), net);
                if let Some(nl) = net_length {
                    m.insert("net_length".into(), nl);
                }
                for (name, figure) in below.into_iter().chain(above) {
                    m.insert(name, figure);
                }
                // How thick it is: its type's thickness, as sure as the type and the
                // wall's choice of it. A foundation wall's concrete is net_area ×
                // thickness (call 130), worked here and not typed beside the wall.
                let f = "the thickness its type gives";
                let mut t_inputs = input.to_vec();
                let thick = match typed(&tree.wall_types, &w.type_name, "a wall") {
                    Ok((type_path, t)) => {
                        t_inputs.push(format!("{type_path}#{K_WALL_TYPE}"));
                        match t.thickness.feet() {
                            Some(x) => Derived::new(Some(x), "ft", &[w.type_name.status, t.thickness.status], f, &t_inputs),
                            None => Derived::refused("ft", f, &t_inputs, "its type's thickness is not known"),
                        }
                    }
                    Err(why) => Derived::refused("ft", f, &t_inputs, &why),
                };
                m.insert("thickness".into(), thick);
                // Its outside corners and the corner board they take, where a line of it names them (call 185).
                if let Some(c) = corner_figures.get(me) {
                    m.insert(crate::trim::OUTSIDE_CORNERS.into(), c.count.clone());
                    m.insert(crate::trim::CORNER_BOARD_LENGTH.into(), c.board.clone());
                }
                // Its gable, where a line of it names the area (call 192).
                if gable_named.contains(me) {
                    if let Some(g) = gable_figures.get(me) {
                        m.insert(crate::gable::GABLE_AREA.into(), g.area.clone());
                    }
                }
                // Its frame, counted, where what is priced on it names the lumber and the
                // spacing it is framed with (call 137): the frame the window draws, laid out
                // by the same rule (`framing::frame`), from the same line (`finish::framing_of`).
                // Not given at all where nothing names a framing: a wall framed by nobody is
                // not a wall with no studs.
                if let Some((_, spacing)) = crate::finish::framing_of(crate::finish::wall_texts(cost, made_of, condition).iter().map(String::as_str)) {
                    let mut f_inputs = inputs.clone();
                    f_inputs.push(format!("{me}#{K_COST}"));
                    let names = [("studs", "EA", "every stud, king, jack and cripple of its frame, one stud each; a corner's and an intersection's extra studs not counted"), ("plate_length", "LF", "its bottom plate and two top plates"), ("header_length", "LF", "a header over each opening, jack to jack"), ("sill_length", "LF", "a rough sill under each window")];
                    // An opening that does not fit its wall (past an end, over the top, over
                    // another) is not framed as if it were cut to fit: the frame is refused,
                    // as the wall's net area is, with the same reason (judged 2026-11-18). Nor
                    // is one whose head is in the top plates: it has no room for a header.
                    let misfits: Vec<&String> = faults.iter().filter(|f| !f.starts_with("where ")).collect();
                    let plates = 2.0 * 1.5 / 12.0;
                    let why = match (length, &heights.height) {
                        _ if unit_fault => Some(format!("the wall line {NOT_CONVERTED}")),
                        (None, _) => Some("the wall has no length".to_string()),
                        (_, Err(why)) => Some(why.clone()),
                        _ if !misfits.is_empty() => Some(misfits.iter().map(|f| f.as_str()).collect::<Vec<_>>().join("; ")),
                        (_, Ok(h)) => mine
                            .iter()
                            .find(|o| o.span.is_none())
                            .map(|o| format!("where {} is in the wall is not known", o.path))
                            .or_else(|| mine.iter().find(|o| o.span.is_some_and(|sp| sp[3] > h - plates + 1e-9)).map(|o| format!("the head of {} is in the top plates: there is no room for its header", o.path))),
                    };
                    match (why, length, &heights.height) {
                        (None, Some(l), Ok(h)) => {
                            let spans: Vec<[f64; 4]> = mine.iter().filter_map(|h| h.span).collect();
                            let tally = framing::tally(&framing::frame(l, *h, &spans, spacing));
                            let of = |pieces: &[framing::Piece], count: bool| -> f64 { tally.iter().filter(|(p, _, _)| pieces.contains(p)).map(|(_, n, len)| if count { *n as f64 } else { *len }).sum() };
                            use framing::Piece::*;
                            let figures = [of(&[Stud, King, Jack, Cripple], true), of(&[BottomPlate, TopPlate], false), of(&[Header], false), of(&[Sill], false)];
                            let mut statuses: Vec<Status> = mine.iter().map(|h| h.status).collect();
                            statuses.extend([w.line.status, heights.status]);
                            for ((name, unit, f), v) in names.iter().zip(figures) {
                                // The count rests on a rule as well as on the wall: studs on
                                // the spacing the line names, the corners left to the
                                // allowance. Assumed, and its formula says what is not in it.
                                let mut st = statuses.clone();
                                if *name == "studs" {
                                    st.push(Status::Assumed);
                                }
                                let mut d = Derived::new(Some(v), unit, &st, f, &f_inputs);
                                // A gable's frame is not counted, and the wall says so (call 192).
                                if gable_figures.get(me).is_some_and(|g| g.area.value.is_some_and(|a| a > 0.0)) {
                                    d.note = crate::gable::FRAME_NOT_COUNTED.to_string();
                                }
                                m.insert(name.to_string(), d);
                            }
                        }
                        (why, _, _) => {
                            let why = why.unwrap_or_else(|| "the frame cannot be laid out".to_string());
                            for (name, unit, f) in names {
                                m.insert(name.to_string(), Derived::refused(unit, f, &f_inputs, &why));
                            }
                        }
                    }
                }
            }
            // A strip footing: its length on plan, its width from its type, how deep it
            // is, and the concrete in it. Length times width times depth is the volume of
            // a ring of footings exactly: where two meet at a corner point they overlap
            // in a square and leave the same square open outside it.
            Kind::Footing => {
                let input = [format!("{me}#{K_WALL}")];
                let Some(w) = wall else {
                    for (name, unit) in kind.measures() {
                        m.insert(name.to_string(), Derived::refused(unit, "", &input, "no footing line entered"));
                    }
                    out.entry(me.clone()).or_default().extend(m);
                    continue;
                };
                let line = w.line_ft();
                let length = line.map(|[a, b]| geom::dist(a, b));
                let f_len = "length of the footing's line on the plan";
                let plan = if w.line.value.is_some() && line.is_none() { Derived::refused("LF", f_len, &input, &format!("the footing's line {NOT_CONVERTED}")) } else { Derived::new(length, "LF", &[w.line.status], f_len, &input) };
                let deep = match w.height.as_ref().map(|h| (h.feet(), h.status)) {
                    Some((Some(d), status)) => Derived::new(Some(d), "ft", &[status], "its depth, as entered", &input),
                    Some((None, _)) => Derived::refused("ft", "its depth, as entered", &input, "its depth is not known"),
                    None => Derived::refused("ft", "its depth, as entered", &input, "no depth entered"),
                };
                let mut inputs = input.to_vec();
                let wide = match typed(&tree.wall_types, &w.type_name, "a footing") {
                    Ok((type_path, t)) => {
                        inputs.push(format!("{type_path}#{K_WALL_TYPE}"));
                        match t.thickness.feet() {
                            Some(x) => Derived::new(Some(x), "ft", &[w.type_name.status, t.thickness.status], "the width its type gives", &inputs),
                            None => Derived::refused("ft", "the width its type gives", &inputs, "its type's width is not known"),
                        }
                    }
                    Err(why) => Derived::refused("ft", "the width its type gives", &inputs, &why),
                };
                // What of it lies in a footing drawn before it is that footing's concrete
                // and is taken off this one's, and the formula says so: worked exactly
                // from the two rectangles on plan (`footings`). A plain corner takes
                // nothing off: there the square the two share is the square left open.
                // (J20-1: a 20 ft stem on a 40 ft run, 24 by 12, is 118 CF by hand and
                // was 120. J21-1, J21-3, J21-4: a cross, three at a point, a stem part
                // way across and a stem deeper than its run were still counted twice.)
                let (taken, within) = match strips.iter().find(|s| s.path == *me) {
                    Some(mine) => footings::counted_elsewhere(mine, &strips),
                    None => (0.0, Vec::new()),
                };
                for other in &within {
                    inputs.push(format!("{other}#{K_WALL}"));
                }
                let f_vol = if within.is_empty() { "plan_length × width × depth ÷ 27".to_string() } else { format!("(plan_length × width × depth − {} CF that lies in {} and is counted there) ÷ 27", view::figure_shown(taken, 4), within.join(" and ")) };
                let f_vol = f_vol.as_str();
                // From the three as they were worked, not as they are written (call 133):
                // 20 in by 10 in is 20/12 by 10/12, not 1.6667 by 0.8333.
                let volume = match (plan.worked(), wide.worked(), deep.worked()) {
                    (Some(l), Some(x), Some(d)) => Derived::new(Some((l * x * d - taken).max(0.0) / 27.0), "CY", &[plan.status, wide.status, deep.status], f_vol, &inputs),
                    _ => {
                        let why = [&plan, &wide, &deep].iter().find(|d| d.value.is_none()).map(|d| d.note.clone()).unwrap_or_default();
                        Derived::refused("CY", f_vol, &inputs, &why)
                    }
                };
                m.insert("plan_length".into(), plan);
                m.insert("width".into(), wide);
                m.insert("depth".into(), deep);
                m.insert("volume".into(), volume);
            }
            Kind::Floor => {
                let input = [format!("{me}#{K_FLOOR}")];
                let f = format!("area of the boundary on the plan{}", floor.filter(|f| f.boundary.unit == units::IN).map(|_| " ÷ 144 (entered in inches)").unwrap_or_default());
                m.insert(
                    "plan_area".into(),
                    match floor {
                        None => Derived::refused("SF", &f, &input, "no boundary entered"),
                        Some(fl) if fl.boundary.value.is_some() && fl.boundary_ft().is_none() => Derived::refused("SF", &f, &input, &format!("the boundary {NOT_CONVERTED}")),
                        Some(fl) => Derived::new(fl.boundary_ft().map(|p| geom::plan_area(&p)), "SF", &[fl.boundary.status], &f, &input),
                    },
                );
                // The distance round it on the plan: a slab's edge form, its thickened
                // edge, the sill seal on it (call 130).
                let f = format!("the distance round the boundary on the plan{}", floor.filter(|f| f.boundary.unit == units::IN).map(|_| " ÷ 12 (entered in inches)").unwrap_or_default());
                m.insert(
                    "perimeter".into(),
                    match floor {
                        None => Derived::refused("LF", &f, &input, "no boundary entered"),
                        Some(fl) if fl.boundary.value.is_some() && fl.boundary_ft().is_none() => Derived::refused("LF", &f, &input, &format!("the boundary {NOT_CONVERTED}")),
                        Some(fl) => Derived::new(fl.boundary_ft().map(|p| geom::perimeter(&p)), "LF", &[fl.boundary.status], &f, &input),
                    },
                );
                // How thick it is: its type's thickness, as sure as the type and the
                // floor's choice of it. A slab's concrete is plan_area × thickness (call 130).
                let f = "the thickness its type gives";
                let mut inputs = input.to_vec();
                let thick = match floor.map(|fl| (fl, typed(&tree.floor_types, &fl.type_name, "a floor"))) {
                    None => Derived::refused("ft", f, &inputs, "no boundary entered"),
                    Some((_, Err(why))) => Derived::refused("ft", f, &inputs, &why),
                    Some((fl, Ok((type_path, t)))) => {
                        inputs.push(format!("{type_path}#{K_FLOOR_TYPE}"));
                        match t.thickness.feet() {
                            Some(x) => Derived::new(Some(x), "ft", &[fl.type_name.status, t.thickness.status], f, &inputs),
                            None => Derived::refused("ft", f, &inputs, "its type's thickness is not known"),
                        }
                    }
                };
                m.insert("thickness".into(), thick);
                // Its joists, counted where what is priced on it names the lumber and the
                // spacing they are framed with (call 165), by the rule a wall's studs are
                // (`finish::members_named`); not given at all where nothing names joists.
                if let Some((_, spacing)) = crate::finish::members_named(crate::finish::wall_texts(cost, made_of, condition).iter().map(String::as_str), &["joist"]) {
                    let mut f_inputs = input.to_vec();
                    f_inputs.push(format!("{me}#{K_COST}"));
                    if joists_span.is_some() {
                        f_inputs.push(format!("{me}#{K_JOISTS_SPAN}"));
                    }
                    m.extend(joist_figures(floor, joists_span, spacing, &f_inputs));
                }
            }
            k if k.is_segment() || *k == Kind::Beam => {
                let input = [format!("{me}#{K_SEGMENT}")];
                let Some(r) = segment else {
                    for (name, unit) in kind.measures() {
                        m.insert(name.to_string(), Derived::refused(unit, "", &input, "no line entered"));
                    }
                    out.entry(me.clone()).or_default().extend(m);
                    continue;
                };
                // Its true length: along its line on the plan when it is level, and
                // through its rise when its two ends are at different heights
                // (BIMCraft `runs.rs:107`).
                let line = r.line_ft();
                let plan = line.map(|[a, b]| geom::dist(a, b));
                let inches = if r.line.unit == units::IN { " ÷ 12 (entered in inches)" } else { "" };
                let (length, statuses, f_len) = match &r.end_offset {
                    None => (plan, vec![r.line.status], format!("length of its line on the plan{inches}")),
                    Some(e) => (
                        plan.zip(r.offsets_ft()).map(|(p, (z0, z1))| p.hypot(z1 - z0)),
                        vec![r.line.status, r.offset.status, e.status],
                        format!("√(length on the plan{inches}² + ({} − {})²)", feet_text("end_offset", e), feet_text("offset", &r.offset)),
                    ),
                };
                let len_d = if r.line.value.is_some() && line.is_none() { Derived::refused("LF", &f_len, &input, &format!("its line {NOT_CONVERTED}")) } else { Derived::new(length, "LF", &statuses, &f_len, &input) };
                if *k == Kind::Duct {
                    let skin = match typed(&tree.segment_types, &r.type_name, "a run") {
                        Err(why) => Derived::refused("SF", "true_length × the distance round the duct", &input, &why),
                        Ok((type_path, t)) => {
                            let inputs = [input[0].clone(), format!("{type_path}#{K_SEGMENT_TYPE}")];
                            let f = match (t.round, &t.depth) {
                                (true, _) => format!("true_length × π × {}", feet_text("its type's diameter", &t.width)),
                                (false, Some(d)) => format!("true_length × 2 × ({} + {})", feet_text("its type's width", &t.width), feet_text("its type's depth", d)),
                                (false, None) => "true_length × the distance round the duct".to_string(),
                            };
                            match len_d.value {
                                None => Derived::refused("SF", &f, &inputs, "the duct has no length"),
                                Some(l) => Derived::new(t.perimeter_ft().map(|p| l * p), "SF", &[len_d.status, r.type_name.status, t.status()], &f, &inputs),
                            }
                        }
                    };
                    m.insert("surface_area".into(), skin);
                }
                // A beam's figures (call 164): its length along it, its section from its
                // type, and what it weighs where its type says what a foot of it weighs.
                if *k == Kind::Beam {
                    let formula = "length × its type's weight per foot";
                    match typed(&tree.segment_types, &r.type_name, "a beam") {
                        Err(why) => {
                            for (name, unit, f) in [("width", "ft", "its type's width"), ("depth", "ft", "its type's depth"), ("weight", "LB", formula), ("section_area", "SF", "width × depth"), ("volume", "CY", "section_area × length ÷ 27")] {
                                m.insert(name.into(), Derived::refused(unit, f, &input, &why));
                            }
                        }
                        Ok((type_path, t)) => {
                            let inputs = [input[0].clone(), format!("{type_path}#{K_SEGMENT_TYPE}")];
                            let sure = r.type_name.status;
                            let size = |v: Option<&Val<Len>>, what: &str| match v {
                                Some(v) => match v.feet() {
                                    Some(x) => Derived::new(Some(x), "ft", &[sure, v.status], &units::to_feet_text(what, &v.unit), &inputs),
                                    None => Derived::refused("ft", what, &inputs, &format!("{what} is not known")),
                                },
                                None => Derived::refused("ft", what, &inputs, &format!("{what} is not known")),
                            };
                            let wide = size(Some(&t.width), if t.round { "its type's diameter" } else { "its type's width" });
                            let deep = if t.round { size(Some(&t.width), "its type's diameter") } else { size(t.depth.as_ref(), "its type's depth") };
                            // What the BIM file's base quantities are (BIMCraft `member_quantities`).
                            let (area, volume) = section_figures(&wide, &deep, t.round, &len_d, &inputs);
                            m.insert("width".into(), wide);
                            m.insert("depth".into(), deep);
                            m.insert("section_area".into(), area);
                            m.insert("volume".into(), volume);
                            let weight = match (&t.weight_per_foot, len_d.worked()) {
                                (None, _) => Derived::refused("LB", formula, &inputs, "its type gives no weight per foot: type what a foot of it weighs, from its shape's name or its maker's sheet"),
                                (Some(w), _) if w.value.is_none() => Derived::refused("LB", formula, &inputs, "its type's weight per foot is not known"),
                                (Some(_), None) => Derived::refused("LB", formula, &inputs, &len_d.note),
                                (Some(w), Some(l)) => Derived::new(w.value.map(|per| l * per), "LB", &[len_d.status, sure, w.status], formula, &inputs),
                            };
                            m.insert("weight".into(), weight);
                        }
                    }
                    m.insert("length".into(), len_d);
                } else {
                    m.insert("true_length".into(), len_d);
                }
            }
            // A post, and a pad footing, which is a post laid flat (call 164): where it
            // stands, how tall (how thick), and its section from its type.
            Kind::Post | Kind::PadFooting => {
                let input = [format!("{me}#{K_COLUMN}")];
                let post = *kind == Kind::Post;
                let Some(c) = column else {
                    for (name, unit) in kind.measures() {
                        m.insert(name.to_string(), Derived::refused(unit, "", &input, "no position entered"));
                    }
                    out.entry(me.clone()).or_default().extend(m);
                    continue;
                };
                let tall = match c.height.feet() {
                    Some(h) => Derived::new(Some(h), if post { "LF" } else { "ft" }, &[c.height.status], if post { "its height, as entered" } else { "its thickness, as entered" }, &input),
                    None => Derived::refused(if post { "LF" } else { "ft" }, if post { "its height, as entered" } else { "its thickness, as entered" }, &input, if post { "its height is not known" } else { "its thickness is not known" }),
                };
                let mut inputs = input.to_vec();
                // The section its type gives: across and deep (a round one is as deep as it is wide).
                let section: Result<(Derived, Derived), String> = match typed(&tree.segment_types, &c.type_name, if post { "a post" } else { "a pad footing" }) {
                    Err(why) => Err(why),
                    Ok((type_path, t)) => {
                        inputs.push(format!("{type_path}#{K_SEGMENT_TYPE}"));
                        let sure = c.type_name.status;
                        let one = |v: Option<&Val<Len>>, what: &str, inputs: &[String]| match v.and_then(|v| v.feet().map(|x| (x, v.status))) {
                            Some((x, status)) => Derived::new(Some(x), "ft", &[sure, status], &format!("the {what} its type gives"), inputs),
                            None => Derived::refused("ft", &format!("the {what} its type gives"), inputs, &format!("its type's {what} is not known")),
                        };
                        if t.round {
                            Ok((one(Some(&t.width), "diameter", &inputs), one(Some(&t.width), "diameter", &inputs)))
                        } else {
                            Ok((one(Some(&t.width), "width", &inputs), one(t.depth.as_ref(), "depth", &inputs)))
                        }
                    }
                };
                match (section, post) {
                    (Err(why), true) => {
                        m.insert("width".into(), Derived::refused("ft", "the width its type gives", &inputs, &why));
                        m.insert("depth".into(), Derived::refused("ft", "the depth its type gives", &inputs, &why));
                        m.insert("section_area".into(), Derived::refused("SF", "width × depth", &inputs, &why));
                        m.insert("volume".into(), Derived::refused("CY", "section_area × height ÷ 27", &inputs, &why));
                        m.insert("height".into(), tall);
                    }
                    (Ok((w, d)), true) => {
                        let round = typed(&tree.segment_types, &c.type_name, "a post").ok().is_some_and(|(_, t)| t.round);
                        let (area, volume) = section_figures(&w, &d, round, &tall, &inputs);
                        m.insert("width".into(), w);
                        m.insert("depth".into(), d);
                        m.insert("section_area".into(), area);
                        m.insert("volume".into(), volume);
                        m.insert("height".into(), tall);
                    }
                    (section, false) => {
                        // A pad: its sides are 2 × (width + length) × thickness of form, its
                        // concrete the three multiplied, in cubic yards. Each from the sizes as
                        // they were worked, so 36 in by 36 in by 18 in is 3 by 3 by 1.5.
                        let (w, l) = match section {
                            Ok(x) => x,
                            Err(why) => (Derived::refused("ft", "the width its type gives", &inputs, &why), Derived::refused("ft", "the depth its type gives", &inputs, &why)),
                        };
                        let figure = |name: &str, unit: &str, f: &str, list: &[&Derived], by: &dyn Fn(f64, f64, f64) -> f64| match (w.worked(), l.worked(), tall.worked()) {
                            (Some(a), Some(b), Some(t)) => Derived::new(Some(by(a, b, t)), unit, &list.iter().map(|d| d.status).collect::<Vec<_>>(), f, &inputs),
                            _ => {
                                let why = list.iter().find(|d| d.value.is_none()).map(|d| d.note.clone()).unwrap_or_default();
                                let _ = name;
                                Derived::refused(unit, f, &inputs, &why)
                            }
                        };
                        let area = figure("plan_area", "SF", "width × length", &[&w, &l], &|a, b, _| a * b);
                        let form = figure("form_area", "SF", "2 × (width + length) × thickness", &[&w, &l, &tall], &|a, b, t| 2.0 * (a + b) * t);
                        let volume = figure("volume", "CY", "width × length × thickness ÷ 27", &[&w, &l, &tall], &|a, b, t| a * b * t / 27.0);
                        m.insert("width".into(), w);
                        m.insert("length".into(), l);
                        m.insert("thickness".into(), tall);
                        m.insert("plan_area".into(), area);
                        m.insert("form_area".into(), form);
                        m.insert("volume".into(), volume);
                    }
                }
            }
            // A thing set in place: the figures of its size, as sure as its size (call 155).
            Kind::Equipment | Kind::Fixture | Kind::Furnishing => {
                let input = [format!("{me}#{K_PLACED}")];
                let (f_w, f_d, f_h, f_a) = ("its width, along its own x before it is turned", "its depth, from its front to its back", "its height, from its underside", "width × depth");
                match placed {
                    None => {
                        for (name, unit) in kind.measures() {
                            m.insert(name.to_string(), Derived::refused(unit, "", &input, "no position and size entered"));
                        }
                    }
                    Some(p) => match p.size_ft() {
                        Some([w, d, h]) => {
                            let st = [p.size.status];
                            let width = Derived::new(Some(w), "ft", &st, f_w, &input);
                            let depth = Derived::new(Some(d), "ft", &st, f_d, &input);
                            m.insert("height".into(), Derived::new(Some(h), "ft", &st, f_h, &input));
                            // From the sizes as they were worked, so 24 in by 25 1/2 in is 2 by 2.125.
                            let mut area = Derived::new(Some(w * d), "SF", &[width.status, depth.status], f_a, &input);
                            // A countertop over a box that turns a corner does not have the leg
                            // that turns it: the area does not move, and says so (call 162).
                            if p.what.value.as_deref().is_some_and(|what| what.eq_ignore_ascii_case("Countertop")) {
                                area.note = counter_legs_said(me, p, [w, d, h], &set_in_place).join("; ");
                            }
                            m.insert("plan_area".into(), area);
                            m.insert("width".into(), width);
                            m.insert("depth".into(), depth);
                        }
                        None => {
                            let why = if p.size.value.is_some() { format!("its size {NOT_CONVERTED}") } else { format!("it has no size: {}", p.size.source) };
                            for (name, unit) in kind.measures() {
                                m.insert(name.to_string(), Derived::refused(unit, "", &input, &why));
                            }
                        }
                    },
                }
            }
            _ => {}
        }
        if !m.is_empty() {
            out.entry(me.clone()).or_default().extend(m);
        }
    }
    room_measures(world, &tree, &mut out, &mut HashMap::new());
    out
}

/// Which way a floor's joists run with what was said of it, for what a wall carries
/// (call 176): the same rule as `joist_run`, read from it, so that a count of joists, the
/// frame as drawn and what crosses a wall can never disagree. `said` is None where the
/// way was not said (the short way is then taken).
pub(crate) fn joists_run(ring: &[P2], span: Option<&JoistsSpan>) -> Option<(P2, Option<(bool, Status)>)> {
    joist_run(ring, span).map(|(run, said, status)| (run, said.map(|long| (long, status))))
}

/// What is not in a count of joists or rafters, said on every figure of it (call 165).
pub const NOT_COUNTED: &str = "blocking, hangers and doubled members are not in this count";

/// Which way a floor's joists run, from its boundary and what is said of it: the way (a
/// direction on the plan), whether anything was said, and how sure that is. Square to the
/// longest edge (the short way) unless the floor says the long way; not said, the short way,
/// Assumed. One rule, for what is counted and for what is drawn (call 177).
pub(crate) fn joist_run(ring: &[P2], span: Option<&JoistsSpan>) -> Option<(P2, Option<bool>, Status)> {
    let longest = (0..ring.len()).map(|i| (ring[i], ring[(i + 1) % ring.len()])).max_by(|a, b| geom::dist(a.0, a.1).total_cmp(&geom::dist(b.0, b.1)));
    let (p, q) = longest.filter(|(p, q)| geom::dist(*p, *q) > 1.0e-9)?;
    let along = [q[0] - p[0], q[1] - p[1]];
    let said = span.and_then(|s| s.0.value.as_deref().map(|w| (w == LONG_WAY, s.0.status)));
    let (long_way, way_status) = said.unwrap_or((false, Status::Assumed));
    let run = if long_way { along } else { [-along[1], along[0]] };
    Some((run, said.map(|(long, _)| long), way_status))
}

/// A floor's joists counted across its boundary (call 165): `joists`, `joist_length`,
/// `joist_longest` and `rim_length`. They run square to the longest edge (the short way) unless
/// the floor says the long way; not said, the short way is taken, Assumed, and the figures say so.
pub(crate) fn joist_figures(floor: Option<&Floor>, span: Option<&JoistsSpan>, spacing: f64, inputs: &[String]) -> BTreeMap<String, Derived> {
    const NAMES: [(&str, &str, &str); 4] = [
        ("joists", "EA", "one joist at each multiple of the spacing from the first side, and one inside each edge that runs with them, each cut to the boundary"),
        ("joist_length", "LF", "the lengths of its joists, added"),
        ("joist_longest", "ft", "the longest of its joists: the stock they are bought in is that length or the next up"),
        ("rim_length", "LF", "the boundary's edges measured square to the joists: the band their ends meet"),
    ];
    let refused = |why: &str| NAMES.iter().map(|(n, u, f)| (n.to_string(), Derived::refused(u, f, inputs, why))).collect::<BTreeMap<_, _>>();
    let Some(floor) = floor else { return refused("no boundary entered") };
    let Some(ring) = floor.boundary_ft() else {
        return refused(if floor.boundary.value.is_some() { "the boundary is in a unit this program does not convert" } else { "no boundary entered" });
    };
    // Which way they run: square to the longest edge, or along it.
    let Some((run, said, way_status)) = joist_run(&ring, span) else { return refused("the boundary has no length") };
    let mut note = Vec::new();
    if said.is_none() {
        note.push("joists usually span the short way: say which way these run".to_string());
    }
    let layout = members::members_across(&ring, run, spacing).and_then(|a| members::rim_length(&ring, run).map(|r| (a, r)));
    match layout {
        Err(why) => refused(&why),
        Ok((across, rim)) => {
            if across.left_out > 0 {
                note.push(format!("{} piece{} under 6 in left out", across.left_out, if across.left_out == 1 { "" } else { "s" }));
            }
            note.push(NOT_COUNTED.to_string());
            let status = [floor.boundary.status, way_status];
            let figures = [across.count() as f64, across.total(), across.longest(), rim];
            NAMES
                .iter()
                .zip(figures)
                .map(|((n, u, f), v)| {
                    let mut d = Derived::new(Some(v), u, &status, f, inputs);
                    d.note = note.join("; ");
                    (n.to_string(), d)
                })
                .collect()
        }
    }
}

/// The area of a beam's or a post's section, and its volume over its length, from the two
/// sizes as they were worked: a round section is a circle across its width, a rectangular
/// one its width by its depth (BIMCraft `member_quantities`, `ifc.rs:707`). `along` is its
/// length (a post's height).
pub(super) fn section_figures(wide: &Derived, deep: &Derived, round: bool, along: &Derived, inputs: &[String]) -> (Derived, Derived) {
    let statuses = [wide.status, deep.status];
    let f_area = if round { "π × diameter² ÷ 4" } else { "width × depth" };
    let area = match (wide.worked(), deep.worked()) {
        (Some(w), Some(d)) => Derived::new(Some(if round { std::f64::consts::PI * w * w / 4.0 } else { w * d }), "SF", &statuses, f_area, inputs),
        _ => Derived::refused("SF", f_area, inputs, &[wide, deep].iter().find(|d| d.value.is_none()).map(|d| d.note.clone()).unwrap_or_default()),
    };
    let f_vol = "section_area × its length ÷ 27";
    let volume = match (area.worked(), along.worked()) {
        (Some(a), Some(l)) => Derived::new(Some(a * l / 27.0), "CY", &[area.status, along.status], f_vol, inputs),
        _ => Derived::refused("CY", f_vol, inputs, &[&area, along].iter().find(|d| d.value.is_none()).map(|d| d.note.clone()).unwrap_or_default()),
    };
    (area, volume)
}
