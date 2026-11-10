//! The rules an entered component has to meet before the world will hold it. The same
//! checks run when a job is loaded and when a value is set, so there is one standard
//! and no way round it. Each returns a sentence an estimator can act on.

use crate::components::*;
use crate::geom;
use crate::value::Val;

/// The eave has to sit on an outline edge to within this many feet (about 1/8 inch).
pub const FIT_TOLERANCE_FT: f64 = 0.01;
/// Plan coordinates beyond this are refused: far enough for any site, near enough that
/// a sixteenth of an inch is still exact in a 64-bit number.
pub const MAX_COORDINATE_FT: f64 = 1.0e6;
/// Steeper than this (87 degrees) is a wall, not a roof plane.
pub const MAX_PITCH: f64 = 240.0;
/// A unit cost beyond this is a typing error, not a price.
pub const MAX_UNIT_COST: f64 = 1.0e9;

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
    o.points.check(&format!("{path} outline points"), U_LENGTH)?;
    o.eave.check(&format!("{path} eave line"), U_LENGTH)?;
    o.eave_height.check(&format!("{path} eave height"), U_LENGTH)?;
    if let Some(pts) = &o.points.value {
        for p in pts {
            point_ok(&format!("{path} outline"), *p)?;
        }
        if let Some(fault) = geom::outline_fault(pts) {
            return Err(format!("{path} outline: {fault}"));
        }
    }
    if let Some([a, b]) = o.eave.value {
        point_ok(&format!("{path} eave line"), a)?;
        point_ok(&format!("{path} eave line"), b)?;
        if let Some(pts) = &o.points.value {
            eave_edge(pts, [a, b]).map_err(|why| format!("{path} eave line: {why}"))?;
        }
    }
    if o.eave_height.value.is_some_and(|h| !h.is_finite() || h.abs() > MAX_COORDINATE_FT) {
        return Err(format!("{path} eave height: is not a usable number"));
    }
    Ok(())
}

/// The edge of the outline an eave line lies along. The plane's slope is measured from
/// this edge's own line, not from the eave line as typed, so a short or slightly crooked
/// eave entry cannot tilt the plane. The eave has to be the plane's low edge: no part of
/// the outline may lie on the other side of it, or the plane would fold.
pub fn eave_edge(pts: &[P2], eave: [P2; 2]) -> Result<[P2; 2], String> {
    let [a, b] = eave;
    if geom::dist(a, b) <= FIT_TOLERANCE_FT {
        return Err("has no length".into());
    }
    let n = pts.len();
    let holding: Vec<usize> = (0..n)
        .filter(|&i| {
            let (p, q) = (pts[i], pts[(i + 1) % n]);
            geom::point_to_segment(a, p, q) <= FIT_TOLERANCE_FT && geom::point_to_segment(b, p, q) <= FIT_TOLERANCE_FT
        })
        .collect();
    let edge = match holding.as_slice() {
        [] => return Err("does not lie along any one edge of the outline".into()),
        [i] => [pts[*i], pts[(*i + 1) % n]],
        _ => return Err("is too short to tell which edge of the outline it lies along".into()),
    };
    let offsets: Vec<f64> = pts.iter().filter_map(|p| geom::signed_offset(*p, edge[0], edge[1])).collect();
    let (lo, hi) = offsets.iter().fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(*x), hi.max(*x)));
    if lo < -FIT_TOLERANCE_FT && hi > FIT_TOLERANCE_FT {
        return Err("part of the outline lies on the low side of it; the eave is the plane's lowest edge, and the plane rises one way from it".into());
    }
    Ok(edge)
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
    r.ends.check(&format!("{path} line ends"), U_LENGTH)?;
    if let Some([a, b]) = r.ends.value {
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
    h.0.check(&format!("{path} height"), U_LENGTH)?;
    if h.0.value.is_some_and(|x| !x.is_finite() || x < 0.0) {
        return Err(format!("{path} height: must be zero or more"));
    }
    if h.0.value.is_some_and(|x| x > MAX_COORDINATE_FT) {
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

/// A cost line has to say what it is, be priced by a quantity this kind of element
/// actually has, in that quantity's unit, at a price that is not negative.
pub fn cost(path: &str, kind: Kind, c: &Cost) -> Result<(), String> {
    c.code.check(&format!("{path} cost code"), "")?;
    if let Some(code) = &c.code.value {
        if code.scheme.trim().is_empty() || code.code.trim().is_empty() {
            return Err(format!("{path} cost code: needs both a scheme and a code; mark it Unknown if it has not been coded"));
        }
    }
    for line in &c.lines {
        if line.description.trim().is_empty() {
            return Err(format!("{path} cost line: has no description"));
        }
        let what = format!("{path} cost line {:?}", line.description);
        let Some((_, unit)) = kind.measures().iter().find(|(name, _)| *name == line.quantity) else {
            let have: Vec<&str> = kind.measures().iter().map(|(n, _)| *n).collect();
            return Err(format!("{what}: is priced by {:?}, which a {kind:?} does not have (it has: {})", line.quantity, have.join(", ")));
        };
        if unit.is_empty() {
            return Err(format!("{what}: is priced by {:?}, which is a factor, not a quantity", line.quantity));
        }
        line.unit_cost.check(&format!("{what} unit cost"), &format!("$/{unit}"))?;
        if line.unit_cost.value.is_some_and(|x| !x.is_finite() || x < 0.0) {
            return Err(format!("{what} unit cost: must be zero or more"));
        }
        if line.unit_cost.value.is_some_and(|x| x > MAX_UNIT_COST) {
            return Err(format!("{what} unit cost: is beyond {MAX_UNIT_COST:e} per unit, which is not a usable price"));
        }
    }
    Ok(())
}

pub fn job_info(path: &str, j: &JobInfo) -> Result<(), String> {
    if j.length_unit != U_LENGTH {
        return Err(format!("{path}: the job's length unit is {:?}; this program works in {U_LENGTH:?} only", j.length_unit));
    }
    Ok(())
}
