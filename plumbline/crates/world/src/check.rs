//! The rules an entered component has to meet before the world will hold it. The same
//! checks run when a job is loaded and when a value is set, so there is one standard
//! and no way round it. Each returns a sentence an estimator can act on.

use crate::components::*;
use crate::formula;
use crate::geom;
use crate::lines;
use crate::units;
use crate::value::Val;

// Every limit and tolerance here is in feet. Entered values are converted to feet before
// they are tested, so 0.01 ft is the same closeness whether the outline was entered in
// feet or in inches.

/// The eave has to sit on an outline edge to within this many feet (about 1/8 inch).
pub const FIT_TOLERANCE_FT: f64 = 0.01;
/// Plan coordinates beyond this are refused: far enough for any site, near enough that
/// a sixteenth of an inch is still exact in a 64-bit number.
pub const MAX_COORDINATE_FT: f64 = 1.0e6;
/// Steeper than this (87 degrees) is a wall, not a roof plane.
pub const MAX_PITCH: f64 = 240.0;
/// A footprint edge shorter than this (about 1/4 in) is a slip of the trace.
pub const MIN_FOOTPRINT_EDGE_FT: f64 = 0.02;
/// A footprint edge flatter than this is not a slope the roof can be solved from.
pub const MIN_FOOTPRINT_PITCH: f64 = 0.01;
/// A unit cost beyond this is a typing error, not a price.
pub const MAX_UNIT_COST: f64 = 1.0e9;

/// A set of plan coordinates: the basics, and a unit coordinates can be in.
fn coords_ok<T>(what: &str, v: &Val<T>) -> Result<(), String> {
    v.check_basics(what)?;
    units::coord_divisor(&v.unit).map(|_| ()).map_err(|why| format!("{what}: {why}"))
}

/// `p` is in feet.
fn point_ok(what: &str, p: P2) -> Result<(), String> {
    if p.iter().any(|x| !x.is_finite() || x.abs() > MAX_COORDINATE_FT) {
        return Err(format!("{what}: a coordinate is not a usable number (limit {MAX_COORDINATE_FT:e} ft either way)"));
    }
    Ok(())
}

fn text_ok(what: &str, v: &Val<String>) -> Result<(), String> {
    v.check(what, "")?;
    if v.value.as_ref().is_some_and(|s| s.trim().is_empty()) {
        return Err(format!("{what}: is blank; mark it Unknown if nobody has it"));
    }
    Ok(())
}

pub fn outline(path: &str, kind: Kind, o: &Outline) -> Result<(), String> {
    if !kind.is_area() {
        return Err(format!("{path}: a {kind:?} cannot carry an outline; only a roof plane or a low-slope area can"));
    }
    coords_ok(&format!("{path} outline points"), &o.points)?;
    coords_ok(&format!("{path} eave line"), &o.eave)?;
    o.eave_height.check_length(&format!("{path} eave height"))?;
    let pts = o.points_ft();
    if let Some(pts) = &pts {
        for p in pts {
            point_ok(&format!("{path} outline"), *p)?;
        }
        if let Some(fault) = geom::outline_fault(pts) {
            return Err(format!("{path} outline: {fault}"));
        }
    }
    if let Some([a, b]) = o.eave_ft() {
        point_ok(&format!("{path} eave line"), a)?;
        point_ok(&format!("{path} eave line"), b)?;
        if let Some(pts) = &pts {
            eave_edge(pts, [a, b]).map_err(|why| format!("{path} eave line: {why}"))?;
        }
    }
    if o.eave_height.feet().is_some_and(|h| h.abs() > MAX_COORDINATE_FT) {
        return Err(format!("{path} eave height: is not a usable number (limit {MAX_COORDINATE_FT:e} ft either way)"));
    }
    Ok(())
}

/// The edge of the outline an eave line lies along. The plane's slope is measured from
/// this edge's own line, not from the eave line as typed, so a short or slightly crooked
/// eave entry cannot tilt the plane. Both arguments are in feet. The eave has to be the plane's low edge: no part of
/// the outline may lie on the other side of it, or the plane would fold.
pub fn eave_edge(pts: &[P2], eave: [P2; 2]) -> Result<[P2; 2], String> {
    let [a, b] = eave;
    if geom::dist(a, b) <= FIT_TOLERANCE_FT {
        return Err("has no length".into());
    }
    // The outline edges each end of the eave sits on. If one edge holds both, that is the
    // eave's edge. If not, the eave may span an edge traced in pieces: a straight run of
    // edges from one holding one end to one holding the other. A run counts as straight
    // only if every corner inside it is within the tolerance of the one line from the
    // run's start to its end, so small bends cannot add up into a curve.
    let n = pts.len();
    let on = |x: P2, i: usize| geom::point_to_segment(x, pts[i], pts[(i + 1) % n]) <= FIT_TOLERANCE_FT;
    let a_edges: Vec<usize> = (0..n).filter(|&i| on(a, i)).collect();
    let b_edges: Vec<usize> = (0..n).filter(|&i| on(b, i)).collect();
    let mut candidates: Vec<[P2; 2]> = a_edges.iter().filter(|i| b_edges.contains(i)).map(|&i| [pts[i], pts[(i + 1) % n]]).collect();
    for &i in &a_edges {
        for &j in &b_edges {
            for (start, end) in [(i, j), (j, i)] {
                let steps = (end + n - start) % n;
                if steps == 0 || steps >= n - 1 {
                    continue;
                }
                let (p, q) = (pts[start], pts[(end + 1) % n]);
                let straight = (1..=steps).all(|k| geom::point_to_segment(pts[(start + k) % n], p, q) <= FIT_TOLERANCE_FT);
                if straight && geom::point_to_segment(a, p, q) <= FIT_TOLERANCE_FT && geom::point_to_segment(b, p, q) <= FIT_TOLERANCE_FT {
                    candidates.push([p, q]);
                }
            }
        }
    }
    candidates.sort_by(|x, y| geom::dist(y[0], y[1]).total_cmp(&geom::dist(x[0], x[1])));
    let Some(&edge) = candidates.first() else {
        return Err("does not lie along any one edge of the outline".into());
    };
    // Several candidates are fine if they are pieces of the same straight line; the
    // longest stands for them. If they are different lines, the eave is too short to say.
    let same_line = |c: &[P2; 2]| c.iter().all(|x| geom::point_to_line(*x, edge[0], edge[1]).is_some_and(|d| d <= FIT_TOLERANCE_FT));
    if !candidates.iter().all(same_line) {
        return Err("is too short to tell which edge of the outline it lies along".into());
    }
    let all = pts;
    let offsets: Vec<f64> = all.iter().filter_map(|p| geom::signed_offset(*p, edge[0], edge[1])).collect();
    let (lo, hi) = offsets.iter().fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(*x), hi.max(*x)));
    if lo < -FIT_TOLERANCE_FT && hi > FIT_TOLERANCE_FT {
        return Err("part of the outline lies on the low side of it; the eave is the plane's lowest edge, and the plane rises one way from it".into());
    }
    Ok(edge)
}

/// A footprint goes on a Roof: an outline that makes sense, an eave height, and a rise for
/// every edge.
pub fn footprint(path: &str, kind: Kind, f: &Footprint) -> Result<(), String> {
    if kind != Kind::Roof {
        return Err(format!("{path}: a {kind:?} cannot carry a footprint; only a Roof can"));
    }
    coords_ok(&format!("{path} footprint points"), &f.points)?;
    f.eave_height.check_length(&format!("{path} eave height"))?;
    if f.eave_height.feet().is_some_and(|h| h.abs() > MAX_COORDINATE_FT) {
        return Err(format!("{path} eave height: is not a usable number (limit {MAX_COORDINATE_FT:e} ft either way)"));
    }
    if let Some(pts) = f.points_ft() {
        for p in &pts {
            point_ok(&format!("{path} footprint"), *p)?;
        }
        if let Some(fault) = geom::outline_fault(&pts) {
            return Err(format!("{path} footprint: {fault}"));
        }
        if f.edges.len() != pts.len() {
            return Err(format!("{path} footprint: has {} points and {} edge rises; each edge, from one point to the next, has one", pts.len(), f.edges.len()));
        }
        // A footprint the roof is solved from has to be clean: an edge too short to be a
        // wall, or a corner a hair off straight, each makes planes and hips of its own.
        let n = pts.len();
        // Short edges first: a short edge also puts its ends a hair off the line through
        // their neighbours, and the short edge is the cause.
        for i in 0..n {
            let len = geom::dist(pts[i], pts[(i + 1) % n]);
            if len < MIN_FOOTPRINT_EDGE_FT {
                return Err(format!("{path} footprint: edge {} is {:.4} ft long; an edge under {MIN_FOOTPRINT_EDGE_FT} ft (about 1/4 in) is a slip of the trace, not a wall. Take one of its points out", i + 1, len));
            }
        }
        for i in 0..n {
            let (a, b, c) = (pts[i], pts[(i + 1) % n], pts[(i + 2) % n]);
            let off = geom::point_to_segment(b, a, c);
            if off > 1e-9 && off <= FIT_TOLERANCE_FT {
                return Err(format!("{path} footprint: point {} is {:.4} ft off the straight line between its neighbours; a kink that small would get a hip or a valley of its own. Put it on the line, or take it out", (i + 1) % n + 1, off));
            }
        }
    }
    for (i, e) in f.edges.iter().enumerate() {
        let what = format!("{path} footprint edge {}", i + 1);
        match e.value {
            Some(Rise::Gable) => e.check(&what, "")?,
            Some(Rise::Pitch(x)) => {
                e.check(&what, U_PITCH)?;
                if !x.is_finite() || x <= 0.0 {
                    return Err(format!("{what}: a sloping edge has a pitch above zero; an edge that does not slope is a gable"));
                }
                if x > MAX_PITCH {
                    return Err(format!("{what}: {x} in 12 is steeper than {MAX_PITCH} in 12, which is a wall; make it a gable"));
                }
                if x < MIN_FOOTPRINT_PITCH {
                    return Err(format!("{what}: {x} in 12 is flat; a roof is solved from slopes of {MIN_FOOTPRINT_PITCH} in 12 or more, and a flat roof is a low-slope area"));
                }
            }
            None => {
                e.check_basics(&what)?;
                if e.unit != U_PITCH && !e.unit.is_empty() {
                    return Err(format!("{what}: its unit is {:?}; a pitch is in {U_PITCH:?} and a gable has none", e.unit));
                }
            }
        }
    }
    Ok(())
}

pub fn solved(path: &str, kind: Kind, s: &Solved) -> Result<(), String> {
    if !(kind.is_area() || kind.is_run()) {
        return Err(format!("{path}: a {kind:?} is not something the solver makes"));
    }
    if s.from.trim().is_empty() {
        return Err(format!("{path}: is marked as solved but does not say from which roof"));
    }
    Ok(())
}

pub fn pitch(path: &str, kind: Kind, p: &Pitch) -> Result<(), String> {
    if !kind.is_area() {
        return Err(format!("{path}: a {kind:?} cannot carry a pitch; only a roof plane or a low-slope area can"));
    }
    p.0.check(&format!("{path} pitch"), U_PITCH)?;
    if let Some(x) = p.0.value {
        if !x.is_finite() || x < 0.0 {
            return Err(format!("{path} pitch: must be zero or more"));
        }
        if x > MAX_PITCH {
            let shown = if x > 1.0e6 { format!("{x:e}") } else { format!("{x}") };
            return Err(format!("{path} pitch: {shown} in 12 is steeper than {MAX_PITCH} in 12, which is a wall, not a roof plane"));
        }
    }
    Ok(())
}

pub fn run(path: &str, kind: Kind, r: &Run) -> Result<(), String> {
    if !kind.is_run() {
        return Err(format!("{path}: a {kind:?} cannot carry a line"));
    }
    coords_ok(&format!("{path} line ends"), &r.ends)?;
    if let Some([a, b]) = r.ends_ft() {
        point_ok(&format!("{path} line ends"), a)?;
        point_ok(&format!("{path} line ends"), b)?;
        if geom::dist(a, b) == 0.0 {
            return Err(format!("{path} line ends: the two ends are the same point"));
        }
    }
    if r.planes.iter().any(|p| p.trim().is_empty()) {
        return Err(format!("{path} line: names a plane with a blank path"));
    }
    if kind == Kind::Parapet && !r.planes.is_empty() {
        return Err(format!("{path} line: a parapet lies on no plane, but this one names {}", r.planes.join(", ")));
    }
    Ok(())
}

pub fn height(path: &str, kind: Kind, h: &Height) -> Result<(), String> {
    if kind != Kind::Parapet {
        return Err(format!("{path}: a {kind:?} cannot carry a height; only a parapet can"));
    }
    h.0.check_length(&format!("{path} height"))?;
    if h.0.feet().is_some_and(|x| x < 0.0) {
        return Err(format!("{path} height: must be zero or more"));
    }
    if h.0.feet().is_some_and(|x| x > MAX_COORDINATE_FT) {
        return Err(format!("{path} height: is not a usable number (limit {MAX_COORDINATE_FT:e} ft)"));
    }
    Ok(())
}

pub fn condition(path: &str, c: &Condition) -> Result<(), String> {
    text_ok(&format!("{path} condition"), &c.0)
}

pub fn work_code(path: &str, w: &WorkCode) -> Result<(), String> {
    text_ok(&format!("{path} schedule code"), &w.wbs)
}

/// A cost line has to say what it is (and no two lines on one element say the same
/// thing), carry a formula that makes sense on this kind of element, use every value
/// entered on it, and be priced in the unit it comes out in, at a price that is not
/// negative.
/// An allowance is a rate, in dollars per SF or per LF, on the one figure a thing of its
/// kind is bought by.
pub fn allowance(path: &str, kind: Kind, a: &Allowance) -> Result<(), String> {
    let Some((figure, unit)) = kind.priced_by() else {
        return Err(format!("{path}: a {kind:?} cannot carry an allowance; it has no area or length a rate could be on"));
    };
    let what = format!("{path} allowance");
    a.rate.check(&what, &format!("$/{unit}")).map_err(|why| format!("{why} (a {kind:?} is priced by its {figure}, in {unit})"))?;
    if a.rate.value.is_some_and(|x| !x.is_finite() || x < 0.0) {
        return Err(format!("{what}: must be zero or more"));
    }
    if a.rate.value.is_some_and(|x| x > MAX_UNIT_COST) {
        return Err(format!("{what}: is beyond {MAX_UNIT_COST:e} per unit, which is not a usable price"));
    }
    Ok(())
}

pub fn cost(path: &str, kind: Kind, c: &Cost) -> Result<(), String> {
    // Cost lines are on what is built and on the job. A condition's lines are its recipe;
    // a material's cost is its price; a type is not a thing that is bought.
    if matches!(kind, Kind::Condition | Kind::Material | Kind::Type | Kind::System | Kind::Storey | Kind::Drawing | Kind::Prior) && !c.lines.is_empty() {
        return Err(format!("{path}: a {kind:?} cannot carry cost lines; they go on what is built, or on the job for general conditions{}", if kind == Kind::Condition { " (a condition's own lines are its recipe)" } else { "" }));
    }
    c.code.check(&format!("{path} cost code"), "")?;
    if let Some(code) = &c.code.value {
        if code.scheme.trim().is_empty() || code.code.trim().is_empty() {
            return Err(format!("{path} cost code: needs both a scheme and a code; mark it Unknown if it has not been coded"));
        }
    }
    for (i, line) in c.lines.iter().enumerate() {
        if line.description.trim().is_empty() {
            return Err(format!("{path} cost line: has no description"));
        }
        if line.description != line.description.trim() {
            return Err(format!("{path} cost line {:?}: its description starts or ends with a space; a line is known by its description, and a space nobody can see makes two lines out of one", line.description));
        }
        if let Some(c) = line.description.chars().find(|c| c.is_control() || matches!(*c, '\u{00AD}' | '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202F}' | '\u{2060}'..='\u{2064}' | '\u{FEFF}')) {
            return Err(format!("{path} cost line {:?}: its description holds a character nobody can see (U+{:04X}); two lines could look the same and be different", line.description, c as u32));
        }
        let same = |a: &str, b: &str| a.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase() == b.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
        if let Some(other) = c.lines[..i].iter().find(|other| same(&other.description, &line.description)) {
            return Err(format!("{path}: has two cost lines described as {:?} and {:?}; a line is known by its description, so each one on an element is different in more than capitals and spacing", other.description, line.description));
        }
        let what = format!("{path} cost line {:?}", line.description);

        if let Some(so) = &line.stretch_out {
            so.check_length(&format!("{what} stretch-out"))?;
            if so.feet().is_some_and(|x| x <= 0.0) {
                return Err(format!("{what} stretch-out: must be more than zero; it is the width of the flat sheet. Mark it Unknown if nobody has it"));
            }
            if so.feet().is_some_and(|x| x > MAX_COORDINATE_FT) {
                return Err(format!("{what} stretch-out: is not a usable number (limit {MAX_COORDINATE_FT:e} ft)"));
            }
        }
        for (name, v) in &line.inputs {
            if let Some(fault) = lines::input_name_fault(kind, name) {
                return Err(format!("{what}: its input {fault}"));
            }
            let named = format!("{what} {name}");
            if v.unit.is_empty() {
                // A plain number: a count, a number of sides.
                v.check_basics(&named)?;
                match v.value {
                    None => {}
                    Some(units::Len::One(x)) if x.is_finite() && x.abs() <= MAX_UNIT_COST => {}
                    Some(units::Len::One(_)) => return Err(format!("{named}: is not a usable number")),
                    Some(units::Len::FtIn(_)) => return Err(format!("{named}: is a [feet, inches] pair with no unit; the unit for a pair is \"ft-in\"")),
                }
            } else {
                v.check_length(&named)?;
                if v.feet().is_some_and(|x| x.abs() > MAX_COORDINATE_FT) {
                    return Err(format!("{named}: is not a usable number (limit {MAX_COORDINATE_FT:e} ft)"));
                }
            }
        }

        // The formula, read against this kind of element with no figures in it.
        let on = lines::On { kind, measure: None, height: None };
        let read = lines::read(&on, line).map_err(|why| format!("{what}: its quantity {:?} {why}", line.quantity))?;
        let used = formula::names(&read.expr);
        if line.stretch_out.is_some() && !used.iter().any(|n| n == lines::STRETCH_OUT) {
            return Err(format!("{what}: carries a stretch-out that its quantity, {:?}, does not use. A stretch-out turns a length (LF) into an area: write the quantity as a length times stretch_out", line.quantity));
        }
        if let Some(unused) = line.inputs.keys().find(|k| !used.contains(k)) {
            return Err(format!("{what}: carries {unused:?}, which its quantity, {:?}, does not use; a value the formula does not read would sit on the line unused", line.quantity));
        }

        if let Some(w) = &line.waste {
            w.check(&format!("{what} waste"), "%")?;
            if w.value.is_some_and(|x| !x.is_finite() || !(0.0..=1000.0).contains(&x)) {
                return Err(format!("{what} waste: is a percentage from 0 to 1000"));
            }
        }
        if let Some(o) = &line.order {
            if o.unit.is_empty() || o.unit.chars().any(char::is_whitespace) || o.unit.contains('/') || o.unit.contains('$') {
                return Err(format!("{what}: its order unit {:?} is not a usable name for what is bought (sheet, lb, box)", o.unit));
            }
            lines::order_multiplies(o, read.unit).map_err(|why| format!("{what}: {why}"))?;
            if o.unit == HOURS && o.round_up {
                return Err(format!("{what}: its hours are rounded up to whole hours; hours are not bought whole"));
            }
            o.factor.check_basics(&format!("{what} order factor"))?;
            if o.factor.value.is_some_and(|x| !x.is_finite() || x <= 0.0 || x > MAX_UNIT_COST) {
                return Err(format!("{what} order factor: must be more than zero (and not beyond {MAX_UNIT_COST:e})"));
            }
        }

        if line.of_material {
            // Ordered and priced as the element's material is: it carries neither itself.
            if line.order.is_some() {
                return Err(format!("{what}: is made of the element's material and also carries an order factor; how it is ordered is the material's"));
            }
            if !CostLine::is_no_price_of_its_own(&line.unit_cost) {
                return Err(format!("{what}: is made of the element's material and also carries a unit cost; its price is the material's"));
            }
            if line.class != Some(Class::Material) {
                return Err(format!("{what}: is made of the element's material, so its class is Material"));
            }
        } else {
            if CostLine::is_no_price_of_its_own(&line.unit_cost) {
                return Err(format!("{what}: has no unit cost; a line carries one (marked Unknown until it is entered) unless it is made of the element's material"));
            }
            let unit = lines::priced_in(line, read.unit);
            line.unit_cost.check(&format!("{what} unit cost"), &format!("$/{unit}"))?;
            if line.unit_cost.value.is_some_and(|x| !x.is_finite() || x < 0.0) {
                return Err(format!("{what} unit cost: must be zero or more"));
            }
            if line.unit_cost.value.is_some_and(|x| x > MAX_UNIT_COST) {
                return Err(format!("{what} unit cost: is beyond {MAX_UNIT_COST:e} per unit, which is not a usable price"));
            }
        }
        // A class says what the line is, and the line has to be that. In-house work is in
        // hours, by a production rate; nothing else is.
        if let Some(class) = line.class {
            match (class.in_hours(), line.in_hours()) {
                (true, false) => return Err(format!("{what}: is {}, which is in-house work, and is not in hours; give it a production rate as its order factor (\"40 SF/{HOURS}\" or \"0.025 {HOURS}/SF\") and dollars an hour as its unit cost", class.title())),
                (false, true) if class == Class::Subcontract => return Err(format!("{what}: is Subcontract and carries a production rate; a subcontractor's work is a quantity and a price, and in-house work is Labor, Fabrication or Supervision")),
                (false, true) => return Err(format!("{what}: is {} and is in hours; a line in hours is Labor, Fabrication or Supervision", class.title())),
                _ => {}
            }
        }
        if line.off && line.class != Some(Class::Labor) {
            return Err(format!("{what}: is switched off, and only a Labor line can be"));
        }
    }
    Ok(())
}

/// What an element is made of is a material's name.
pub fn made_of(path: &str, kind: Kind, m: &MadeOf) -> Result<(), String> {
    if matches!(kind, Kind::Job | Kind::Condition | Kind::Material | Kind::Type | Kind::System | Kind::Storey | Kind::Drawing | Kind::Prior) {
        return Err(format!("{path}: a {kind:?} is not made of a material"));
    }
    text_ok(&format!("{path} material"), &m.0)
}

/// How a material is bought and what it costs: an order factor that is a usable number
/// in a unit like `lb/SF`, and prices, each in dollars for the unit it is bought in.
pub fn supply(path: &str, kind: Kind, s: &Supply) -> Result<(), String> {
    if kind != Kind::Material {
        return Err(format!("{path}: a {kind:?} cannot carry a material's supply; only a Material element can"));
    }
    let mut per = None;
    if let Some(o) = &s.order {
        if o.unit.is_empty() || o.unit.chars().any(char::is_whitespace) || o.unit.contains('/') || o.unit.contains('$') {
            return Err(format!("{path}: its order unit {:?} is not a usable name for what is bought (sheet, lb, box)", o.unit));
        }
        if is_hours(&o.unit) {
            return Err(format!("{path}: a material is not bought by the hour"));
        }
        let (a, b) = o.factor.unit.split_once('/').unwrap_or(("", ""));
        if is_hours(a) || is_hours(b) {
            return Err(format!("{path}: its order factor is in {:?}; a material is not ordered by the hour", o.factor.unit));
        }
        if !((a == o.unit && !b.is_empty() && b != o.unit) || (b == o.unit && !a.is_empty() && a != o.unit)) {
            return Err(format!("{path}: its order factor is in {:?}; for a material bought by the {} it is so many {} to the unit it is measured in (\"{}/SF\") or the other way up (\"SF/{}\")", o.factor.unit, o.unit, o.unit, o.unit, o.unit));
        }
        o.factor.check_basics(&format!("{path} order factor"))?;
        if o.factor.value.is_some_and(|x| !x.is_finite() || x <= 0.0 || x > MAX_UNIT_COST) {
            return Err(format!("{path} order factor: must be more than zero (and not beyond {MAX_UNIT_COST:e})"));
        }
        per = Some(format!("$/{}", o.unit));
    }
    let price = |what: &str, v: &Val<f64>, per: &mut Option<String>| -> Result<(), String> {
        let ok = v.unit.strip_prefix("$/").is_some_and(|u| !u.is_empty() && !u.contains('/') && !u.contains('$') && !u.chars().any(char::is_whitespace));
        if !ok {
            return Err(format!("{what}: its unit is {:?}; a price is dollars for a unit, like \"$/lb\" or \"$/SF\"", v.unit));
        }
        if v.unit.strip_prefix("$/").is_some_and(is_hours) {
            return Err(format!("{what}: a material is not bought by the hour"));
        }
        let wanted = per.get_or_insert_with(|| v.unit.clone()).clone();
        v.check(what, &wanted)?;
        if v.value.is_some_and(|x| !x.is_finite() || x < 0.0) {
            return Err(format!("{what}: must be zero or more"));
        }
        if v.value.is_some_and(|x| x > MAX_UNIT_COST) {
            return Err(format!("{what}: is beyond {MAX_UNIT_COST:e} per unit, which is not a usable price"));
        }
        Ok(())
    };
    price(&format!("{path} price"), &s.price, &mut per)?;
    for (i, (supplier, quote)) in s.quotes.iter().enumerate() {
        if supplier.trim().is_empty() || supplier != supplier.trim() || supplier.chars().any(char::is_control) {
            return Err(format!("{path}: a quote is from {supplier:?}, which is not a usable name for a supplier"));
        }
        // One supplier is one name: two that differ only in capitals or spacing are one
        // supplier written two ways, and would stand in two columns.
        if let Some((other, _)) = s.quotes.iter().take(i).find(|(other, _)| same_name(other, supplier)) {
            return Err(format!("{path}: has quotes from {other:?} and {supplier:?}; a supplier is known by its name, so two are different in more than capitals and spacing"));
        }
        price(&format!("{path} quote from {supplier:?}"), quote, &mut per)?;
    }
    Ok(())
}

/// Whether two names are the same but for capitals, spacing and a full stop at the end:
/// "Copper, 16 oz", "copper,16 OZ" and "Copper, 16 oz." are one name.
pub fn same_name(a: &str, b: &str) -> bool {
    // One space where there is any, none beside a comma, and no full stop that is not a
    // decimal point: "No. 1" is "No 1", and "1.5 in" keeps its point. Spaces between
    // figures stay: "1 1/2 in" is not "11/2 in".
    let plain = |s: &str| {
        let spaced = s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ").replace(", ", ",").replace(" ,", ",");
        let chars: Vec<char> = spaced.chars().collect();
        let kept: String = chars.iter().enumerate().filter(|(i, c)| **c != '.' || (*i > 0 && chars[*i - 1].is_ascii_digit() && chars.get(*i + 1).is_some_and(char::is_ascii_digit))).map(|(_, c)| *c).collect();
        kept.split_whitespace().collect::<Vec<_>>().join(" ")
    };
    plain(a) == plain(b)
}

/// Whether a unit is the hour, however it is written.
pub fn is_hours(unit: &str) -> bool {
    matches!(unit.to_ascii_lowercase().as_str(), "hr" | "hrs" | "h" | "hour" | "hours")
}

/// Overhead and profit are percentages, on the job.
pub fn markup(path: &str, kind: Kind, m: &Markup) -> Result<(), String> {
    if kind != Kind::Job {
        return Err(format!("{path}: overhead and profit are the job's; a {kind:?} cannot carry them"));
    }
    for (what, v) in [("overhead", &m.overhead), ("profit", &m.profit)] {
        let what = format!("{path} {what}");
        v.check(&what, "%")?;
        if v.value.is_some_and(|x| !x.is_finite() || !(0.0..=1000.0).contains(&x)) {
            return Err(format!("{what}: is a percentage from 0 to 1000"));
        }
    }
    Ok(())
}

/// A condition's recipe has to say what kinds of element it is for, and its lines have
/// to make sense on each of them.
pub fn recipe(path: &str, kind: Kind, t: &Recipe) -> Result<(), String> {
    if kind != Kind::Condition {
        return Err(format!("{path}: a {kind:?} cannot carry a condition's recipe; only a Condition element can"));
    }
    if t.applies_to.is_empty() {
        return Err(format!("{path}: the condition does not say what kind of element it applies to"));
    }
    for (i, k) in t.applies_to.iter().enumerate() {
        if !(k.is_area() || k.is_run()) {
            return Err(format!("{path}: a condition applies to roof planes, low-slope areas and lines, not to a {k:?}"));
        }
        if t.applies_to[..i].contains(k) {
            return Err(format!("{path}: names {k:?} twice in what it applies to"));
        }
        cost(&format!("{path} (applied to a {k:?})"), *k, &t.cost)?;
    }
    // The stretch-out belongs to the detail, so a recipe never carries one. Anything else
    // a recipe states (a number of sides, a waste, a price) goes onto the element with the
    // recipe's status and source, where it can be seen.
    if let Some(line) = t.cost.lines.iter().find(|l| l.stretch_out.as_ref().is_some_and(|so| so.value.is_some())) {
        return Err(format!("{path} recipe line {:?}: carries a stretch-out. A stretch-out belongs to the detail it is bent for, not to a recipe; leave it Unknown here and enter it on the element", line.description));
    }
    if t.cost.lines.is_empty() {
        return Err(format!("{path}: the condition has no lines; applying it would do nothing"));
    }
    Ok(())
}

pub fn job_info(path: &str, j: &JobInfo) -> Result<(), String> {
    if j.length_unit != U_LENGTH {
        if j.length_unit.trim().is_empty() {
            return Err(format!("{path}: the job's length unit is blank; derived lengths are worked out in {U_LENGTH:?}"));
        }
        return Err(format!("{path}: the job's length unit {}; derived lengths are worked out in {U_LENGTH:?} only (LF, SF, SQ), and each entered value carries its own unit", units::refuse_length_unit(&j.length_unit, "\"ft\"")));
    }
    Ok(())
}
