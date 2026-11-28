use super::*;

// ---------------------------------------------------------------- reading it back

/// The rows a flat reading of the job shows for this module's components on one element.
pub fn rows(w: &World, e: Entity, path: &str, names: &HashMap<String, String>) -> Vec<Row> {
    let mut rows = Vec::new();
    let pair = |[a, b]: &[P2; 2]| format!("({}, {}) to ({}, {})", view::num(a[0]), view::num(a[1]), view::num(b[0]), view::num(b[1]));
    let plain = |label: &str, text: String, basis: &str| Row { path: path.into(), label: label.into(), text, status: Status::Known, basis: basis.into(), note: String::new(), derived: false, number: None, unit: String::new() };
    let named = |p: &String| names.get(p).map(|n| format!("{n} ({p})")).unwrap_or_else(|| p.clone());
    let layers = |rows: &mut Vec<Row>, list: &[Layer]| {
        for (i, l) in list.iter().enumerate() {
            rows.push(view::entered(path, &format!("layer {}, {:?}", i + 1, l.function).to_lowercase(), &l.material, |m| m.clone()));
            rows.push(view::length(path, &format!("layer {} thickness", i + 1), &l.thickness));
        }
    };
    if let Some(c) = w.get::<Level>(e) {
        rows.push(view::length(path, "elevation", &c.elevation));
        rows.push(view::length(path, "level height", &c.height));
        if let Some(h) = &c.computation_height {
            rows.push(view::length(path, "rooms find their walls at", h));
        }
    }
    if let Some(c) = w.get::<Room>(e) {
        rows.push(view::entered(path, "a point inside it", &c.point, |p| format!("({}, {})", view::num(p[0]), view::num(p[1]))));
        if let Some(n) = &c.number {
            rows.push(view::entered(path, "number", n, |s| s.clone()));
        }
    }
    if let Some(c) = w.get::<RoomLine>(e) {
        rows.push(view::entered(path, "line", &c.line, pair));
    }
    if let Some(c) = w.get::<WallType>(e) {
        // A footing's type gives a width; a wall's, a thickness. It is a footing's where
        // it was made as one (call 87), or, from a file written before types said so,
        // where footings are drawn on it.
        let footings = c.footing || w.iter_entities().any(|x| x.get::<Kind>() == Some(&Kind::Footing) && x.get::<Wall>().is_some_and(|c| c.type_name.value.as_deref() == Some(path)));
        rows.push(view::length(path, if footings { "footing width" } else { "wall thickness" }, &c.thickness));
        layers(&mut rows, &c.layers);
    }
    if let Some(c) = w.get::<FloorType>(e) {
        rows.push(view::length(path, "floor thickness", &c.thickness));
        layers(&mut rows, &c.layers);
    }
    if let Some(c) = w.get::<Item>(e) {
        rows.push(plain("an item, placed as", c.category.a(), "what kind of thing it is"));
        if !c.sheet.is_empty() {
            rows.push(plain("made from", c.sheet.clone(), "the maker's sheet it was read from"));
        }
        // Which trade's library it is kept in, and what one of it is drawn as.
        rows.push(plain("trade", c.division.map(|d| format!("Division {} {}", d.number(), d.title())).unwrap_or_else(|| "not said".to_string()), "which trade's item it is"));
        let drawn = match c.form.as_deref() {
            Some(said) => said.to_string(),
            None => forms::form_of(w.get::<Node>(e).map(|n| n.name.as_str()).unwrap_or("")).map(|f| format!("{}, by its name", f.name())).unwrap_or_else(|| format!("{}, its name saying nothing else", forms::BOX)),
        };
        rows.push(plain("drawn as", drawn, "what one of it is drawn as, inside the box of its sizes"));
        if let Some(a) = &c.priced_as {
            rows.push(plain("priced as", a.clone(), "the assembly each of it set down takes"));
        }
        rows.push(view::entered(path, "maker", &c.maker, |s| s.clone()));
        rows.push(view::entered(path, "model", &c.model, |s| s.clone()));
        rows.push(view::length(path, "width", &c.width));
        rows.push(view::length(path, "depth", &c.depth));
        rows.push(view::length(path, "height", &c.height));
        if let Some(weight) = &c.weight {
            rows.push(view::entered(path, "weight", weight, |x| view::num(*x)));
        }
    }
    if let Some(c) = w.get::<OpeningType>(e) {
        rows.push(plain("a type of", format!("{:?}", c.category).to_lowercase(), "what the type is for"));
        rows.push(view::length(path, "width", &c.width));
        rows.push(view::length(path, "height", &c.height));
    }
    // A footing is held as a wall is, and is listed in a footing's words: its line, how
    // deep it is, and where its top is.
    if let Some(c) = w.get::<Wall>(e).filter(|_| w.get::<Kind>(e) == Some(&Kind::Footing)) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "footing line", &c.line, pair));
        if let Some(h) = &c.height {
            rows.push(view::length(path, "depth", h));
        }
        if let Some(o) = &c.base_offset {
            rows.push(view::length(path, "top, above its level's floor", o));
        }
    } else if let Some(c) = w.get::<Wall>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "wall line", &c.line, pair));
        if let Some(t) = &c.top_level {
            rows.push(view::entered(path, "top level", t, named));
        }
        if let Some(h) = &c.height {
            rows.push(view::length(path, "height", h));
        }
        if let Some(o) = &c.base_offset {
            rows.push(view::length(path, "base offset", o));
        }
        if let Some(o) = &c.top_offset {
            rows.push(view::length(path, "top offset", o));
        }
        if c.location != Location::Centre {
            rows.push(plain("its line is its", c.location.label().into(), "which line of the wall was drawn"));
        }
        if !c.room_bounding {
            rows.push(plain("bounds rooms", "no".into(), "a room runs past it"));
        }
        if !c.join {
            rows.push(plain("joins its neighbours", "no".into(), "its ends stay square"));
        }
    }
    if let Some(c) = w.get::<Opening>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::length(path, "center, along the wall", &c.offset));
        rows.push(view::length(path, "sill", &c.sill));
    }
    if let Some(c) = w.get::<JoistsSpan>(e) {
        rows.push(view::entered(path, "joists span", &c.0, |s| s.clone()));
    }
    if let Some(c) = w.get::<Floor>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "boundary", &c.boundary, |pts| format!("{} points", pts.len())));
        if let Some(o) = &c.offset {
            rows.push(view::length(path, "offset", o));
        }
    }
    if let Some(c) = w.get::<SegmentType>(e) {
        rows.push(plain("a type of", format!("{:?}", c.category).to_lowercase(), "what the type is for"));
        rows.push(view::length(path, if c.round { "diameter" } else { "width" }, &c.width));
        if let Some(d) = &c.depth {
            rows.push(view::length(path, "depth", d));
        }
        if let Some(per) = &c.weight_per_foot {
            rows.push(view::entered(path, "weight per foot", per, |x| view::num(*x)));
        }
    }
    if let Some(c) = w.get::<Stood>(e) {
        let pad = w.get::<Kind>(e) == Some(&Kind::PadFooting);
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "position", &c.at, |p| format!("({}, {})", view::num(p[0]), view::num(p[1]))));
        rows.push(view::length(path, "base, above its level's floor", &c.base_offset));
        rows.push(view::length(path, if pad { "thickness" } else { "height" }, &c.height));
        // A pad's top is where it was said to be; it is held as its base and its thickness.
        if let (true, Some(b), Some(h)) = (pad, c.base_offset.feet(), c.height.feet()) {
            let mut top = plain("top", crate::structure::at_height(b + h), "its base and its thickness");
            top.status = c.base_offset.status.max(c.height.status);
            rows.push(top);
        }
    }
    if let Some(c) = w.get::<Segment>(e) {
        rows.push(view::entered(path, "type", &c.type_name, named));
        rows.push(view::entered(path, "line on the plan", &c.line, pair));
        rows.push(view::length(path, if c.end_offset.is_some() { "height above its level, first end" } else { "height above its level" }, &c.offset));
        if let Some(o) = &c.end_offset {
            rows.push(view::length(path, "height above its level, second end", o));
        }
        if c.justify != Justify::Centre {
            rows.push(plain("height is to its", format!("{:?}", c.justify).to_lowercase(), "the bottom, the center or the top of its section: where its heights are taken to"));
        }
    }
    if let Some(c) = w.get::<Fitting>(e) {
        rows.push(plain("fitting", c.kind.title().to_lowercase(), "what joins the runs here"));
    }
    if let Some(c) = w.get::<Placed>(e) {
        rows.push(view::entered(path, "what it is", &c.what, |s| s.clone()));
        rows.push(view::entered(path, "position", &c.at, |p| format!("({}, {})", view::num(p[0]), view::num(p[1]))));
        rows.push(view::length(path, "height above its level", &c.offset));
        if let Some(a) = &c.angle {
            // `entered` writes the unit after the figure: "180 deg".
            rows.push(view::entered(path, "turned", a, |x| view::num(crate::editing::turned_said(*x))));
        }
        // Its size as a tape reads it, wide by deep by high: the same way an item's is said.
        let mut size = view::entered(path, "size", &c.size, |s| format!("{} × {} × {}", view::num(s[0]), view::num(s[1]), view::num(s[2])));
        if let Some([wide, deep, high]) = c.size_ft() {
            size.text = units::size_said([wide, deep, high]);
        }
        rows.push(size);
    }
    if let Some(c) = w.get::<Connectors>(e) {
        for (i, k) in c.0.iter().enumerate() {
            let what = [Some(format!("{:?}", k.kind).to_lowercase()), k.domain.map(|d| format!("{d:?}").to_lowercase()), (k.flow != Flow::Both).then(|| format!("{:?}", k.flow).to_lowercase()), k.facing.map(|f| format!("faces {}", f.word()))].into_iter().flatten().collect::<Vec<_>>().join(", ");
            let label = if k.name.is_empty() { format!("connection {} ({what})", i + 1) } else { format!("connection {}: {} ({what})", i + 1, k.name) };
            rows.push(view::entered(path, &label, &k.at, |p| format!("({}, {}, {})", view::num(p[0]), view::num(p[1]), view::num(p[2]))));
            if let Some(size) = &k.size {
                rows.push(view::length(path, &format!("connection {} size", i + 1), size));
            }
        }
    }
    if let Some(c) = w.get::<SystemInfo>(e) {
        rows.push(Row { path: path.into(), label: "classification".into(), text: format!("{} ({:?})", c.classification, c.discipline), status: Status::Known, basis: "what the system is".into(), note: String::new(), derived: false, number: None, unit: String::new() });
    }
    if let Some(c) = w.get::<InSystem>(e) {
        rows.push(view::entered(path, "system", &c.0, |p| names.get(p).cloned().unwrap_or_else(|| p.clone())));
    }
    rows
}

/// What a window needs to draw an element: its shape in feet, in space.
#[derive(Clone, Debug)]
pub enum Shape {
    /// A wall from `a` to `b` on plan, standing from `base` to `base + height`, with the
    /// openings cut in it: each as [from, to] along the wall and [sill, head] above `base`.
    Wall { a: P2, b: P2, base: f64, height: f64, thickness: f64, holes: Vec<(String, Kind, [f64; 4])> },
    Floor { boundary: Vec<P2>, top: f64, thickness: f64 },
    /// One straight piece from `a` to `b` along its centre line, and its section: round by
    /// a diameter, rectangular by width and depth, or neither when its size is not known.
    Run { a: P3, b: P3, round: Option<f64>, rect: Option<(f64, f64)> },
    /// A box by the middle of its base and its size, turned `angle` degrees
    /// counter-clockwise from east about that point.
    Box { at: P3, size: P3, angle: f64 },
}

#[derive(Clone, Debug)]
pub struct Placement {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    /// The storey it is under, by path.
    pub storey: Option<String>,
    /// The system it belongs to, by path.
    pub system: Option<String>,
    pub discipline: Discipline,
    pub shape: Shape,
    /// The worst status among the values its shape is drawn from.
    pub status: Status,
    /// What a placed thing is drawn as, inside its box: what its item says, else what
    /// its name says. None: the box, and always for anything that is not placed.
    pub form: Option<forms::Form>,
    /// What it is called by its size, in the words the trade uses (call 177): "3 in waste",
    /// "3/4 in cold water", "16 x 8 supply" for a run on a system; its type's own name for a
    /// run that is on none, a beam, a post, a pad footing, a footing or a wall. None for
    /// what has no type to name. Worked here, once; a window only writes it.
    pub tag: Option<String>,
    /// A vertical drain of 3 in or more: a stack (call 177). Its tag says so.
    pub stack: bool,
}

/// A length in feet as inches the way a pipe is sized: to the sixteenth as a fraction
/// where it is one ("3/4", "1 1/2"), else as a decimal ("0.4").
pub(super) fn inches_said(feet: f64) -> String {
    let inches = feet * 12.0;
    let sixteenths = inches * 16.0;
    if (sixteenths - sixteenths.round()).abs() > 0.02 {
        let d = format!("{:.2}", inches);
        return d.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    let parts = sixteenths.round() as u64;
    let (whole, mut top, mut bottom) = (parts / 16, parts % 16, 16);
    while top > 0 && top % 2 == 0 {
        top /= 2;
        bottom /= 2;
    }
    match (whole, top) {
        (w, 0) => format!("{w}"),
        (0, t) => format!("{t}/{bottom}"),
        (w, t) => format!("{w} {t}/{bottom}"),
    }
}

/// What a classification of system is called in the trade, in a size: a drain carries
/// "waste", a supply duct "supply".
pub fn trade_word(classification: &str) -> Option<&'static str> {
    Some(match classification {
        "Supply Air" => "supply",
        "Return Air" => "return",
        "Exhaust Air" => "exhaust",
        "Refrigerant" => "refrigerant",
        "Domestic Cold Water" => "cold water",
        "Domestic Hot Water" => "hot water",
        "Sanitary" => "waste",
        "Vent" => "vent",
        "Power" => "power",
        "Lighting" => "lighting",
        _ => return None,
    })
}

/// A size and what it carries, said as a tag (call 177): the section by itself, then the
/// system's word. A run on no system, or on one with no trade word, is its type's name. One
/// whose size is not known says so.
pub fn run_tag(type_name: Option<&str>, round: Option<f64>, rect: Option<(f64, f64)>, classification: Option<&str>) -> Option<String> {
    let size = match (round, rect) {
        (Some(d), _) => Some(format!("{} in", inches_said(d))),
        (None, Some((w, d))) => Some(format!("{} x {}", inches_said(w), inches_said(d))),
        _ => None,
    };
    match (size, classification.and_then(trade_word)) {
        (Some(size), Some(word)) => Some(format!("{size} {word}")),
        // Its size is not known: said, and not left off or guessed.
        (None, Some(word)) => Some(format!("{word}, size not known")),
        (Some(size), None) => Some(type_name.map(str::to_string).unwrap_or(size)),
        (None, None) => Some(type_name.map(|t| format!("{t}, size not known")).unwrap_or_else(|| "size not known".to_string())),
    }
}

/// Every element of this module's kinds that has a shape, placed in space, and every one
/// that has none with the reason. A window draws the first list and has to say the second:
/// an element that cannot be drawn does not just go missing from the picture.
pub fn placements(job: &mut Job) -> (Vec<Placement>, Vec<(String, String)>) {
    let order = job.order.clone();
    let tree = Tree::of(&mut job.world);
    let w = &job.world;
    let node = |e: Entity| w.get::<Node>(e);
    let index: HashMap<String, Entity> = order.iter().filter_map(|e| node(*e).map(|n| (n.path.clone(), *e))).collect();
    let mut drawn = Vec::new();
    let mut not_drawn = Vec::new();
    for &e in &order {
        let (Some(n), Some(kind)) = (node(e), w.get::<Kind>(e).copied()) else { continue };
        if !(matches!(kind, Kind::Wall | Kind::Floor | Kind::Footing) || kind.is_segment() || kind.is_placed() || kind.is_framing()) {
            continue;
        }
        let storey = tree.storey_of(&n.path).map(str::to_string);
        let system = w.get::<InSystem>(e).and_then(|s| s.0.value.clone());
        let discipline = system.as_ref().and_then(|p| index.get(p)).and_then(|s| w.get::<SystemInfo>(*s)).map(|s| s.discipline).unwrap_or(kind.discipline());
        // Where its storey's floor is: None when it is under no storey, Some(None) when
        // the storey's elevation is not known.
        let base = storey.as_ref().map(|s| tree.levels.get(s).and_then(|l| l.elevation.feet()));
        // A shape stands no surer than the height of the floor it is measured from.
        let floor_status = storey.as_ref().and_then(|s| tree.levels.get(s)).map(|l| l.elevation.status).unwrap_or(Status::Known);
        // Openings under this wall that cannot be cut in it, each with why.
        let mut uncut: Vec<(String, String)> = Vec::new();
        let shape: Result<(Shape, Status), String> = match kind {
            // A footing is placed as a wall is: its line, its type's width, and from its
            // base up as far as it is deep.
            Kind::Wall | Kind::Footing => (|| {
                let c = w.get::<Wall>(e).ok_or(if kind == Kind::Footing { "it has no line" } else { "it has no wall line" })?;
                let [a, b] = c.line_ft().ok_or("its line is not known")?;
                let heights = wall_heights(&tree, &n.path, c);
                let height = heights.height.clone()?;
                // A footing's top is at its level's floor (or as far over it as it says),
                // and it lies below that by its depth; a wall stands up from its base.
                let base = heights.base.clone()? - if kind == Kind::Footing { height } else { 0.0 };
                let (thickness, layers, section_status) = wall_section(&tree, c)?;
                // Where it stands as well as how tall it is: its storey's floor and its
                // base offset are part of its shape (`span`, not the height's own status).
                let mut status = c.line.status.max(heights.span).max(section_status);
                let length = geom::dist(a, b);
                // The shape is drawn about the wall's centre line, which lies to the left
                // of the line that was drawn by as much as the wall's `location` says.
                let off = walls::centre(thickness, &layers, c.location);
                let left = [-(b[1] - a[1]) / length * off, (b[0] - a[0]) / length * off];
                let (a, b) = ([a[0] + left[0], a[1] + left[1]], [b[0] + left[0], b[1] + left[1]]);
                let mut holes = Vec::new();
                for &child in &order {
                    let Some(cn) = node(child) else { continue };
                    if cn.parent.as_deref() != Some(n.path.as_str()) {
                        continue;
                    }
                    let Some(k) = w.get::<Kind>(child).copied().filter(|k| k.is_opening()) else { continue };
                    let Some(o) = w.get::<Opening>(child) else {
                        uncut.push((cn.path.clone(), "it has no opening entered, so it is not cut in its wall".into()));
                        continue;
                    };
                    // An opening is cut only where it is known and lies in the wall. One
                    // that is not cut is said, with why: it does not just go missing.
                    let (hole, _) = hole_of(&tree, &cn.path, o);
                    match hole.span {
                        None => uncut.push((cn.path.clone(), "where it is or how big it is is not known, so it is not cut in its wall".into())),
                        Some([from, to, _, _]) if from < -crate::check::FIT_TOLERANCE_FT || to > length + crate::check::FIT_TOLERANCE_FT => uncut.push((cn.path.clone(), format!("it runs past the end of {}, so it is not cut in it", n.path))),
                        Some([_, _, _, head]) if head > height + crate::check::FIT_TOLERANCE_FT => uncut.push((cn.path.clone(), format!("it stands taller than {}, so it is not cut in it", n.path))),
                        Some(span) => {
                            holes.push((cn.path.clone(), k, span));
                            status = status.max(hole.status);
                        }
                    }
                }
                // A wall with an opening that could not be cut is drawn solid where the
                // opening is: its shape is not known to be right there.
                if !uncut.is_empty() {
                    status = status.max(Status::Assumed);
                }
                Ok((Shape::Wall { a, b, base, height, thickness, holes }, status))
            })(),
            Kind::Floor => (|| {
                let c = w.get::<Floor>(e).ok_or("it has no boundary")?;
                let boundary = c.boundary_ft().ok_or("its boundary is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let (offset, offset_status) = match &c.offset {
                    None => (0.0, Status::Known),
                    Some(o) => (o.feet().ok_or("its offset is not known")?, o.status),
                };
                let (_, t) = typed(&tree.floor_types, &c.type_name, "a floor")?;
                let thickness = t.thickness.feet().ok_or("its type's thickness is not known")?;
                Ok((Shape::Floor { boundary, top: level + offset, thickness }, c.boundary.status.max(c.type_name.status).max(t.thickness.status).max(offset_status).max(floor_status)))
            })(),
            k if k.is_segment() || k == Kind::Beam => (|| {
                let c = w.get::<Segment>(e).ok_or("it has no line")?;
                let [p, q] = c.line_ft().ok_or("its line is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let (z0, z1) = c.offsets_ft().ok_or("its height above its level is not known")?;
                let t = typed(&tree.segment_types, &c.type_name, "a run").ok().map(|x| x.1);
                let (round, rect) = (t.and_then(|t| t.round_ft()), t.and_then(|t| t.rect_ft()));
                let section = round.map(|d| (d, d)).or(rect);
                if c.justify != Justify::Centre && section.is_none() {
                    return Err("its size is not known and its height is to its top or its bottom, so its center line cannot be placed".into());
                }
                let (width, depth) = section.unwrap_or((0.0, 0.0));
                let g = runs::Geom::of(level, p, q, z0, z1, c.justify, width, depth, round.is_some());
                let sized = if section.is_some() { t.map(|t| t.status().max(c.type_name.status)).unwrap_or(Status::Unknown) } else { Status::Unknown };
                let status = c.line.status.max(c.offset.status).max(c.end_offset.as_ref().map(|e| e.status).unwrap_or(Status::Known)).max(sized).max(floor_status);
                Ok((Shape::Run { a: g.a, b: g.b, round, rect }, status))
            })(),
            // A post stands as a vertical run, its section from its type; a pad footing is a
            // box of its type's width and length and its own thickness, its top where it
            // was said to be (call 164).
            Kind::Post | Kind::PadFooting => (|| {
                let c = w.get::<Stood>(e).ok_or("it has no position")?;
                let at = c.at_ft().ok_or("its position is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let (b, h) = (c.base_offset.feet().ok_or("its base's height above its level is not known")?, c.height.feet().ok_or("its height is not known")?);
                let t = typed(&tree.segment_types, &c.type_name, if kind == Kind::Post { "a post" } else { "a pad footing" }).ok().map(|x| x.1);
                let (round, rect) = (t.and_then(|t| t.round_ft()), t.and_then(|t| t.rect_ft()));
                let sized = if round.is_some() || rect.is_some() { t.map(|t| t.status().max(c.type_name.status)).unwrap_or(Status::Unknown) } else { Status::Unknown };
                let status = c.at.status.max(c.base_offset.status).max(c.height.status).max(sized).max(floor_status);
                if kind == Kind::Post {
                    Ok((Shape::Run { a: [at[0], at[1], level + b], b: [at[0], at[1], level + b + h], round, rect }, status))
                } else {
                    let (wide, long) = rect.ok_or("its type's width and depth are not known")?;
                    Ok((Shape::Box { at: [at[0], at[1], level + b], size: [wide, long, h], angle: 0.0 }, status))
                }
            })(),
            _ => (|| {
                let c = w.get::<Placed>(e).ok_or("it has no position")?;
                let at = c.at_ft().ok_or("its position is not known")?;
                let level = base.ok_or("it is under no level")?.ok_or("its level's elevation is not known")?;
                let offset = c.offset.feet().ok_or("its height above its level is not known")?;
                let size = c.size_ft().ok_or("its size is not known")?;
                let angle = c.angle_deg().ok_or("how far it is turned is not known")?;
                let status = c.at.status.max(c.offset.status).max(c.size.status).max(c.angle.as_ref().map(|a| a.status).unwrap_or(Status::Known)).max(floor_status);
                Ok((Shape::Box { at: [at[0], at[1], level + offset], size, angle }, status))
            })(),
        };
        match shape {
            Ok((shape, status)) => {
                // An item says what it is drawn as, or leaves it to its name.
                let form = match &shape {
                    Shape::Box { .. } if kind.is_framing() => None,
                    Shape::Box { .. } => {
                        let said = w.get::<Placed>(e).and_then(|c| c.item.as_ref()).and_then(|item| index.get(item)).and_then(|t| w.get::<Item>(*t)).and_then(|it| it.form.clone());
                        match said {
                            Some(said) => forms::drawn_as(&said).ok().flatten(),
                            None => forms::form_of(&n.name),
                        }
                    }
                    _ => None,
                };
                // What it is called by its size (call 177): worked here, from its type.
                let type_path = match kind {
                    Kind::Wall | Kind::Footing => w.get::<Wall>(e).and_then(|c| c.type_name.value.clone()),
                    Kind::Post | Kind::PadFooting => w.get::<Stood>(e).and_then(|c| c.type_name.value.clone()),
                    k if k.is_segment() || k == Kind::Beam => w.get::<Segment>(e).and_then(|c| c.type_name.value.clone()),
                    _ => None,
                };
                let type_name = type_path.as_ref().and_then(|t| tree.names.get(t)).map(String::as_str);
                let class = system.as_ref().and_then(|p| index.get(p)).and_then(|s| w.get::<SystemInfo>(*s)).map(|s| s.classification.as_str());
                let (tag, stack) = match &shape {
                    Shape::Run { a, b, round, rect } if kind.is_segment() => {
                        let upright = (a[0] - b[0]).hypot(a[1] - b[1]) < 1.0e-6 && (a[2] - b[2]).abs() > 1.0e-6;
                        let stack = upright && class == Some("Sanitary") && round.is_some_and(|d| d >= 0.25 - 1.0e-6);
                        let tag = run_tag(type_name, *round, *rect, class);
                        (tag.map(|t| if stack { format!("{t} stack") } else { t }), stack)
                    }
                    _ if type_name.is_some() => (type_name.map(str::to_string), false),
                    _ => (None, false),
                };
                drawn.push(Placement { path: n.path.clone(), name: n.name.clone(), kind, storey, system, discipline, shape, status, form, tag, stack })
            }
            Err(why) => not_drawn.push((n.path.clone(), why)),
        }
        not_drawn.extend(uncut);
    }
    (drawn, not_drawn)
}
