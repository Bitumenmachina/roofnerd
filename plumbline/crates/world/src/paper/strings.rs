//! Where a plan's dimension strings run and what they say, in feet, before any paper (call 182).
//! Ours: BIMCraft's plan pass dimensions nothing along a wall; what is followed here is
//! ordinary drafting practice and the US National CAD Standard's sheet conventions: on each
//! side of the plan, nearest the house the openings in that outside wall to their centers
//! from the wall's ends, next the walls that meet it to their lines, outermost the overall;
//! every string a chain whose parts add to the string outside it; a wall off square dimensioned
//! along itself, its angle lettered at its corner; a partition that meets no outside wall
//! located by a string inside the plan from the nearest wall parallel to it.
//!
//! Every figure is written to the sixteenth of an inch, and a chain is made of whole
//! sixteenths measured from its own first station, so its parts add to its whole exactly:
//! the sums are made here, not left to rounding each part on its own.

use crate::skeleton::P2 as V;
use plumbline_roofing::drawing::feet_written;

/// Sixteenths of an inch in a foot.
const PER_FOOT: f64 = 192.0;

/// A length in feet, as whole sixteenths of an inch.
pub fn sixteenths(ft: f64) -> i64 {
    (ft * PER_FOOT).round() as i64
}

/// Whole sixteenths as plans write a length: `12'-0"`, `8'-0 1/2"`.
pub fn written(n: i64) -> String {
    feet_written(n as f64 / PER_FOOT)
}

/// Which way a wall runs on the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// East to west (a horizontal wall); its strings count x.
    Across,
    /// North to south (a vertical wall); its strings count y.
    Up,
    /// Neither: a wall off square.
    Angled,
}

/// How a wall from `a` to `b` runs: level, plumb, or off square (by more than a hair).
pub fn axis_of(a: V, b: V) -> Axis {
    let d = b - a;
    let len = d.0.hypot(d.1);
    if len < 1e-9 {
        Axis::Angled
    } else if d.1.abs() / len < 1e-4 {
        Axis::Across
    } else if d.0.abs() / len < 1e-4 {
        Axis::Up
    } else {
        Axis::Angled
    }
}

/// A side of the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    South,
    North,
    West,
    East,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::South, Side::North, Side::West, Side::East];
    pub fn name(self) -> &'static str {
        match self {
            Side::South => "south",
            Side::North => "north",
            Side::West => "west",
            Side::East => "east",
        }
    }
    /// The axis of the walls that face this side, which is the axis its strings count along.
    pub fn axis(self) -> Axis {
        match self {
            Side::South | Side::North => Axis::Across,
            Side::West | Side::East => Axis::Up,
        }
    }
    /// The unit vector pointing out of the plan on this side.
    pub fn out(self) -> V {
        match self {
            Side::South => V(0.0, -1.0),
            Side::North => V(0.0, 1.0),
            Side::West => V(-1.0, 0.0),
            Side::East => V(1.0, 0.0),
        }
    }
}

/// The coordinate of `p` that a string along `axis` counts.
pub fn along(p: V, axis: Axis) -> f64 {
    if axis == Axis::Up { p.1 } else { p.0 }
}

/// The coordinate square to a string along `axis`.
pub fn across(p: V, axis: Axis) -> f64 {
    if axis == Axis::Up { p.0 } else { p.1 }
}

/// A wall as the strings see it.
#[derive(Clone, Debug)]
pub struct W {
    pub start: V,
    pub end: V,
    /// Offsets of its two faces to its left (the model's `ext` and `int`).
    pub ext: f64,
    pub int: f64,
    /// Whether it is on the outside of the plan (one side indoors) and which way is out.
    pub edge: bool,
    pub out: V,
    /// Whether its length is Known.
    pub sure: bool,
}

impl W {
    pub fn length(&self) -> f64 {
        self.start.dist(self.end)
    }
    /// How far its outer face lies from its line, along `out`.
    pub fn face(&self) -> f64 {
        let t = (self.end - self.start).unit();
        if self.out.dot(t.perp()) > 0.0 {
            self.ext.max(self.int)
        } else {
            -self.ext.min(self.int)
        }
    }
    /// Half its thickness, at the most.
    pub fn half(&self) -> f64 {
        (self.ext - self.int).abs() / 2.0
    }
    pub fn axis(&self) -> Axis {
        axis_of(self.start, self.end)
    }
}

/// A door or window: its wall, where along it, from the wall's first end, and how sure.
#[derive(Clone, Copy, Debug)]
pub struct Op {
    pub wall: usize,
    pub s0: f64,
    pub s1: f64,
    pub sure: bool,
}

/// One place a string measures to or from.
#[derive(Clone, Copy, Debug)]
pub struct Station {
    /// Its coordinate along the string, feet.
    pub c: f64,
    /// Where on the plan its extension line begins: the outer face of the wall it is on.
    pub anchor: V,
    /// Whether what fixes it is not Known.
    pub unsure: bool,
}

/// What a chain measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Openings,
    Partitions,
    Overall,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Openings => "openings",
            Kind::Partitions => "partitions",
            Kind::Overall => "overall",
        }
    }
}

/// A string of stations, each part the distance from one to the next, whole sixteenths from
/// the first.
#[derive(Clone, Debug)]
pub struct Chain {
    pub side: Side,
    pub kind: Kind,
    pub stations: Vec<Station>,
    /// Sixteenths from the first station to each station (the first is 0).
    pub steps: Vec<i64>,
    /// For each part, whether it rests on something not Known.
    pub vif: Vec<bool>,
}

impl Chain {
    /// The parts, as written.
    pub fn parts(&self) -> Vec<String> {
        self.steps.windows(2).map(|w| written(w[1] - w[0])).collect()
    }
    /// The whole, as written.
    pub fn whole(&self) -> String {
        written(self.steps.last().copied().unwrap_or(0))
    }
    /// The parts in whole sixteenths.
    pub fn sixteenths(&self) -> Vec<i64> {
        self.steps.windows(2).map(|w| w[1] - w[0]).collect()
    }
}

/// Build a chain from raw stations (merging those that fall in the same sixteenth).
fn chain(side: Side, kind: Kind, mut raw: Vec<Station>, covering: &dyn Fn(f64) -> bool) -> Chain {
    raw.sort_by(|a, b| a.c.partial_cmp(&b.c).unwrap_or(std::cmp::Ordering::Equal));
    let lo = raw.first().map_or(0.0, |s| s.c);
    let mut stations: Vec<Station> = Vec::new();
    let mut steps: Vec<i64> = Vec::new();
    for s in raw {
        let n = sixteenths(s.c - lo);
        match steps.last() {
            Some(&last) if last == n => {
                if let Some(prev) = stations.last_mut() {
                    prev.unsure |= s.unsure;
                }
            }
            _ => {
                steps.push(n);
                stations.push(s);
            }
        }
    }
    let vif = (0..stations.len().saturating_sub(1)).map(|i| stations[i].unsure || stations[i + 1].unsure || covering((stations[i].c + stations[i + 1].c) / 2.0)).collect();
    Chain { side, kind, stations, steps, vif }
}

/// The strings on the outside of the plan, side by side, and what could not be drawn.
#[derive(Clone, Debug, Default)]
pub struct Sides {
    /// Each side's chains, nearest the house first.
    pub chains: Vec<Chain>,
    /// Sides where no overall is drawn, each with why.
    pub no_overall: Vec<(Side, String)>,
    /// Walls that meet an outside wall of a side (their place is in that side's partitions string).
    pub located: Vec<bool>,
}

/// The three strings on each side of the plan.
pub fn sides(walls: &[W], ops: &[Op], tol: f64) -> Sides {
    let mut out = Sides { located: vec![false; walls.len()], ..Default::default() };
    // The page's own extent, east to west and north to south, from the walls' lines.
    let ends: Vec<V> = walls.iter().flat_map(|w| [w.start, w.end]).collect();
    let extent = |axis: Axis| {
        let (lo, hi) = ends.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, e| (a.0.min(along(*e, axis)), a.1.max(along(*e, axis))));
        (lo, hi)
    };
    for side in Side::ALL {
        let axis = side.axis();
        let cand: Vec<usize> = (0..walls.len()).filter(|&i| walls[i].edge && walls[i].axis() == axis && walls[i].out.dot(side.out()) > 0.5).collect();
        if cand.is_empty() {
            continue;
        }
        let span = |i: usize| {
            let (a, b) = (along(walls[i].start, axis), along(walls[i].end, axis));
            (a.min(b), a.max(b))
        };
        // The station at `c`: its extension line begins on the most outward of the outside
        // walls that have `c` in their reach.
        let anchor_at = |c: f64| -> V {
            let mut best: Option<(f64, V)> = None;
            for &i in &cand {
                let (lo, hi) = span(i);
                if c < lo - tol || c > hi + tol {
                    continue;
                }
                let w = &walls[i];
                let cross = across(w.start, axis) + across(w.out, axis) * w.face();
                let reach = cross * across(side.out(), axis);
                let p = if axis == Axis::Up { V(cross, c) } else { V(c, cross) };
                if best.map_or(true, |b| reach > b.0) {
                    best = Some((reach, p));
                }
            }
            best.map_or(V(c, c), |b| b.1)
        };
        let covering_unsure = |c: f64| cand.iter().any(|&i| !walls[i].sure && span(i).0 - tol <= c && c <= span(i).1 + tol);
        // The walls that meet an outside wall of this side: their ends within its thickness.
        let mut meets: Vec<(f64, bool)> = Vec::new();
        for (j, w) in walls.iter().enumerate() {
            if cand.contains(&j) {
                continue;
            }
            for e in [w.start, w.end] {
                let on = cand.iter().any(|&i| {
                    let (lo, hi) = span(i);
                    let c = along(e, axis);
                    c >= lo - tol && c <= hi + tol && (across(e, axis) - across(walls[i].start, axis)).abs() <= walls[i].half() + tol.max(0.05)
                });
                if on {
                    meets.push((along(e, axis), !w.sure));
                    out.located[j] = true;
                }
            }
        }
        let mut middle: Vec<Station> = Vec::new();
        for &i in &cand {
            let (lo, hi) = span(i);
            for c in [lo, hi] {
                middle.push(Station { c, anchor: anchor_at(c), unsure: false });
            }
        }
        for (c, unsure) in meets {
            middle.push(Station { c, anchor: anchor_at(c), unsure });
        }
        let (c_lo, c_hi) = middle.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, s| (a.0.min(s.c), a.1.max(s.c)));
        // The openings: each wall's ends and each opening's center, from the extremes.
        let mut opening_stations: Vec<Station> = Vec::new();
        for o in ops.iter().filter(|o| cand.contains(&o.wall)) {
            let w = &walls[o.wall];
            let t = (w.end - w.start).unit();
            let mid = w.start + t * ((o.s0 + o.s1) / 2.0);
            let c = along(mid, axis);
            opening_stations.push(Station { c, anchor: anchor_at(c), unsure: !o.sure });
            for c in [span(o.wall).0, span(o.wall).1] {
                opening_stations.push(Station { c, anchor: anchor_at(c), unsure: false });
            }
        }
        if !opening_stations.is_empty() {
            for c in [c_lo, c_hi] {
                opening_stations.push(Station { c, anchor: anchor_at(c), unsure: false });
            }
            out.chains.push(chain(side, Kind::Openings, opening_stations, &covering_unsure));
        }
        let (p_lo, p_hi) = extent(axis);
        let additive = (c_lo - p_lo).abs() <= 0.005 && (c_hi - p_hi).abs() <= 0.005;
        let partitions = chain(side, Kind::Partitions, middle, &covering_unsure);
        if additive {
            // A string that says what the overall says, in one part, is not drawn twice.
            if partitions.stations.len() >= 3 {
                out.chains.push(partitions);
            }
            let mut overall = chain(side, Kind::Overall, vec![Station { c: c_lo, anchor: anchor_at(c_lo), unsure: false }, Station { c: c_hi, anchor: anchor_at(c_hi), unsure: false }], &covering_unsure);
            overall.vif = vec![cand.iter().any(|&i| !walls[i].sure) || partitions_unsure(&out, side)];
            out.chains.push(overall);
        } else {
            out.chains.push(partitions);
            let off_square = walls.iter().any(|w| w.axis() == Axis::Angled);
            out.no_overall.push((side, if off_square { format!("a wall off square makes the plan run past the walls of the {} side, so the strings there do not add to its extent", side.name()) } else { format!("the walls of the {} side do not reach across the whole plan, so the strings there do not add to its extent", side.name()) }));
        }
    }
    out
}

fn partitions_unsure(out: &Sides, side: Side) -> bool {
    out.chains.iter().any(|c| c.side == side && c.kind == Kind::Partitions && c.vif.iter().any(|v| *v))
}

/// A string along one wall: its ends and what stands in it, from its first end.
#[derive(Clone, Debug)]
pub struct Own {
    pub wall: usize,
    /// Distances along the wall to each station: the first end, each opening's center, the last end.
    pub at: Vec<f64>,
    pub steps: Vec<i64>,
    pub vif: Vec<bool>,
}

impl Own {
    pub fn parts(&self) -> Vec<String> {
        self.steps.windows(2).map(|w| written(w[1] - w[0])).collect()
    }
    pub fn whole(&self) -> String {
        written(self.steps.last().copied().unwrap_or(0))
    }
}

/// A string along `wall` whose whole is `length` (the model's own figure for the wall) and
/// whose parts are the openings' centers from its first end. With none it is the one figure.
pub fn own(wall: usize, length: f64, sure: bool, ops: &[Op]) -> Own {
    let total = sixteenths(length);
    let mut centres: Vec<(i64, f64, bool)> = ops.iter().filter(|o| o.wall == wall).map(|o| (sixteenths((o.s0 + o.s1) / 2.0).clamp(0, total), (o.s0 + o.s1) / 2.0, !o.sure)).collect();
    centres.sort_by(|a, b| a.0.cmp(&b.0));
    let mut steps = vec![0i64];
    let mut at = vec![0.0];
    let mut unsure = vec![false];
    for (n, s, u) in centres {
        if n > *steps.last().unwrap_or(&0) && n < total {
            steps.push(n);
            at.push(s);
            unsure.push(u);
        }
    }
    steps.push(total);
    at.push(length);
    unsure.push(false);
    let vif = (0..steps.len() - 1).map(|i| !sure || unsure[i] || unsure[i + 1]).collect();
    Own { wall, at, steps, vif }
}

/// An interior wall that meets no outside wall, and the nearest wall parallel to it: a string
/// square to both, from that wall's line to this one's.
#[derive(Clone, Debug)]
pub struct Locate {
    pub wall: usize,
    pub from: usize,
    /// Where along the walls the string lies (the middle of the part they share).
    pub along: f64,
    /// The cross coordinates of the two lines.
    pub from_cross: f64,
    pub to_cross: f64,
    pub steps: [i64; 2],
    pub vif: bool,
    pub axis: Axis,
}

/// Each interior wall (not an outside wall, not meeting one) that runs level or plumb, with the
/// wall parallel to it that lies nearest and shares some of its length.
pub fn locate(walls: &[W], located: &[bool], tol: f64) -> Vec<Locate> {
    let mut out = Vec::new();
    for (j, w) in walls.iter().enumerate() {
        let axis = w.axis();
        if w.edge || located.get(j).copied().unwrap_or(false) || axis == Axis::Angled {
            continue;
        }
        let span = |i: usize| {
            let (a, b) = (along(walls[i].start, axis), along(walls[i].end, axis));
            (a.min(b), a.max(b))
        };
        let (lo, hi) = span(j);
        let mut best: Option<(f64, usize, f64)> = None;
        for (i, o) in walls.iter().enumerate() {
            if i == j || o.axis() != axis {
                continue;
            }
            let (olo, ohi) = span(i);
            let (s, e) = (lo.max(olo), hi.min(ohi));
            if e - s < 0.5 {
                continue;
            }
            let d = (across(o.start, axis) - across(w.start, axis)).abs();
            if d <= tol {
                continue;
            }
            if best.map_or(true, |b| d < b.0) {
                best = Some((d, i, (s + e) / 2.0));
            }
        }
        if let Some((d, i, mid)) = best {
            out.push(Locate { wall: j, from: i, along: mid, from_cross: across(walls[i].start, axis), to_cross: across(w.start, axis), steps: [0, sixteenths(d)], vif: !w.sure || !walls[i].sure, axis });
        }
    }
    out
}

/// A run of things standing against a wall's face, side by side (a run of cabinets, a vanity):
/// where it starts and ends along the wall's line from the wall's first end.
#[derive(Clone, Debug)]
pub struct Run {
    pub wall: usize,
    /// Which side of the wall it stands on: +1 its left, -1 its right.
    pub side: f64,
    pub s0: f64,
    pub s1: f64,
    /// How far it stands out from the wall's face.
    pub depth: f64,
    pub things: Vec<usize>,
}

/// Runs among `things` (their corner rings and whether they hang above the cut): axis-aligned
/// boxes with a side against a wall's face, grouped by wall and side, and joined where one
/// starts within an inch of where the last ends.
pub fn runs(walls: &[W], things: &[([V; 4], bool)]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::new();
    let flush = 0.06;
    for (wi, w) in walls.iter().enumerate() {
        let axis = w.axis();
        if axis == Axis::Angled || w.length() < 0.5 {
            continue;
        }
        let t = (w.end - w.start).unit();
        for side in [1.0, -1.0] {
            let face_off = if side > 0.0 { w.ext.max(w.int) } else { w.ext.min(w.int) };
            let mut found: Vec<(f64, f64, f64, usize)> = Vec::new();
            for (ti, (ring, hung)) in things.iter().enumerate() {
                if *hung {
                    continue;
                }
                // The ring in the wall's own frame: along it, and out from its line to its left.
                let pts: Vec<(f64, f64)> = ring.iter().map(|q| ((*q - w.start).dot(t), (*q - w.start).dot(t.perp()))).collect();
                let (s0, s1) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, p| (a.0.min(p.0), a.1.max(p.0)));
                let (n0, n1) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, p| (a.0.min(p.1), a.1.max(p.1)));
                // Axis-aligned to the wall: its four corners fall on two values each way.
                let square = pts.iter().all(|p| ((p.0 - s0).abs() < 0.02 || (p.0 - s1).abs() < 0.02) && ((p.1 - n0).abs() < 0.02 || (p.1 - n1).abs() < 0.02));
                if !square || s1 < -flush || s0 > w.length() + flush {
                    continue;
                }
                let (near, far) = if side > 0.0 { (n0, n1) } else { (-n1, -n0) };
                if (near - side * face_off).abs() > flush && (near - face_off.abs()).abs() > flush {
                    continue;
                }
                found.push((s0, s1, (far - near).abs(), ti));
            }
            found.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let mut cur: Option<Run> = None;
            for (s0, s1, depth, ti) in found {
                match cur.as_mut() {
                    Some(r) if s0 <= r.s1 + 0.1 => {
                        r.s1 = r.s1.max(s1);
                        r.depth = r.depth.max(depth);
                        r.things.push(ti);
                    }
                    _ => {
                        if let Some(r) = cur.take() {
                            out.push(r);
                        }
                        cur = Some(Run { wall: wi, side, s0: s0.max(0.0), s1, depth, things: vec![ti] });
                    }
                }
            }
            if let Some(r) = cur {
                out.push(r);
            }
        }
    }
    out
}

/// Where two walls meet off square: the corner, the angle between them in degrees, and the
/// direction into the sector the angle is measured in.
#[derive(Clone, Debug)]
pub struct Corner {
    pub at: V,
    pub a: usize,
    pub b: usize,
    pub degrees: f64,
    pub bisector: V,
}

/// Every place two walls meet other than square or straight: end to end at a corner, or one
/// ending on the other's side (a tee), the acute angle then.
pub fn corners(walls: &[W], tol: f64) -> Vec<Corner> {
    let mut out: Vec<Corner> = Vec::new();
    let square = |deg: f64| (deg - 90.0).abs() < 0.05 || (deg - 180.0).abs() < 0.05 || deg < 0.05;
    for a in 0..walls.len() {
        for b in a + 1..walls.len() {
            // End to end.
            let mut found = false;
            for (pa, da) in [(walls[a].start, (walls[a].end - walls[a].start).unit()), (walls[a].end, (walls[a].start - walls[a].end).unit())] {
                for (pb, db) in [(walls[b].start, (walls[b].end - walls[b].start).unit()), (walls[b].end, (walls[b].start - walls[b].end).unit())] {
                    if pa.dist(pb) <= tol.max(0.02) {
                        found = true;
                        let deg = da.dot(db).clamp(-1.0, 1.0).acos().to_degrees();
                        if !square(deg) {
                            out.push(Corner { at: pa, a, b, degrees: deg, bisector: (da + db).unit() });
                        }
                    }
                }
            }
            if found {
                continue;
            }
            // One ending on the other's side.
            for (x, y) in [(a, b), (b, a)] {
                for (p, d) in [(walls[x].start, (walls[x].end - walls[x].start).unit()), (walls[x].end, (walls[x].start - walls[x].end).unit())] {
                    let o = &walls[y];
                    let u = (o.end - o.start).unit();
                    let s = (p - o.start).dot(u);
                    let off = (p - o.start).dot(u.perp()).abs();
                    if s > tol && s < o.length() - tol && off <= o.half() + tol.max(0.05) {
                        let deg = d.dot(u).clamp(-1.0, 1.0).acos().to_degrees();
                        let deg = deg.min(180.0 - deg);
                        if !square(deg) {
                            let toward = if d.dot(u) >= 0.0 { u } else { u * -1.0 };
                            out.push(Corner { at: V(p.0 - 0.0, p.1), a: x, b: y, degrees: deg, bisector: (d + toward).unit() });
                        }
                    }
                }
            }
        }
    }
    out
}

/// An angle as lettered: whole degrees, or tenths when it is not whole.
pub fn degrees_written(deg: f64) -> String {
    if (deg - deg.round()).abs() < 0.05 {
        format!("{}\u{00B0}", deg.round() as i64)
    } else {
        format!("{:.1}\u{00B0}", deg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall(a: (f64, f64), b: (f64, f64), edge: bool, out: (f64, f64)) -> W {
        W { start: V(a.0, a.1), end: V(b.0, b.1), ext: 0.25, int: -0.25, edge, out: V(out.0, out.1), sure: true }
    }

    fn chains_of(c: &Sides, side: Side) -> Vec<(Kind, Vec<String>, String)> {
        c.chains.iter().filter(|c| c.side == side).map(|c| (c.kind, c.parts(), c.whole())).collect()
    }

    /// The hand figures of call 182 item 1: a house 36 ft by 16 ft (the walls' lines), one
    /// partition across it at 24 ft east, a door 3 ft wide in the south wall centered 6 ft from
    /// its west end, a window 5 ft wide in it centered 30 ft from its west end, a door 2 ft 8 in
    /// wide in the partition centered 8 ft from its south end.
    fn house() -> (Vec<W>, Vec<Op>) {
        let walls = vec![
            wall((0.0, 0.0), (36.0, 0.0), true, (0.0, -1.0)),
            wall((36.0, 0.0), (36.0, 16.0), true, (1.0, 0.0)),
            wall((36.0, 16.0), (0.0, 16.0), true, (0.0, 1.0)),
            wall((0.0, 16.0), (0.0, 0.0), true, (-1.0, 0.0)),
            wall((24.0, 0.0), (24.0, 16.0), false, (-1.0, 0.0)),
        ];
        let ops = vec![Op { wall: 0, s0: 4.5, s1: 7.5, sure: true }, Op { wall: 0, s0: 27.5, s1: 32.5, sure: true }, Op { wall: 4, s0: 8.0 - 1.0 - 1.0 / 3.0, s1: 8.0 + 1.0 + 1.0 / 3.0, sure: true }];
        (walls, ops)
    }

    #[test]
    fn the_south_side_of_the_hand_house_reads_openings_partitions_and_the_overall() {
        let (walls, ops) = house();
        let s = sides(&walls, &ops, 0.01);
        assert_eq!(chains_of(&s, Side::South), vec![
            (Kind::Openings, vec!["6'-0\"".into(), "24'-0\"".into(), "6'-0\"".into()], "36'-0\"".into()),
            (Kind::Partitions, vec!["24'-0\"".into(), "12'-0\"".into()], "36'-0\"".into()),
            (Kind::Overall, vec!["36'-0\"".into()], "36'-0\"".into()),
        ]);
        // The north has no opening: the partitions and the overall. The west and the east are one
        // wall each: the overall alone (a string that says what the overall says is not drawn twice).
        assert_eq!(chains_of(&s, Side::North), vec![(Kind::Partitions, vec!["24'-0\"".into(), "12'-0\"".into()], "36'-0\"".into()), (Kind::Overall, vec!["36'-0\"".into()], "36'-0\"".into())]);
        assert_eq!(chains_of(&s, Side::West), vec![(Kind::Overall, vec!["16'-0\"".into()], "16'-0\"".into())]);
        assert_eq!(chains_of(&s, Side::East), vec![(Kind::Overall, vec!["16'-0\"".into()], "16'-0\"".into())]);
        // The partition meets the south and north walls: located by their strings, not by a string of its own inside.
        assert!(s.located[4]);
        assert!(locate(&walls, &s.located, 0.01).is_empty());
        // Every chain adds to the string outside it, whole sixteenths.
        for side in Side::ALL {
            let all: Vec<&Chain> = s.chains.iter().filter(|c| c.side == side).collect();
            for c in &all {
                assert_eq!(c.sixteenths().iter().sum::<i64>(), all.last().unwrap().steps.last().copied().unwrap(), "{side:?} {:?}", c.kind);
            }
        }
        // The partition's door: 8 ft from the south wall's line and 8 ft to the north wall's.
        let own = own(4, walls[4].length(), true, &ops);
        assert_eq!(own.parts(), vec!["8'-0\"", "8'-0\""]);
        assert_eq!(own.whole(), "16'-0\"");
    }

    #[test]
    fn the_l_house_chains_twenty_and_twenty_to_forty_and_sixteen_and_fourteen_to_thirty() {
        // 0,0 40,0 40,16 20,16 20,30 0,30, closed.
        let walls = vec![
            wall((0.0, 0.0), (40.0, 0.0), true, (0.0, -1.0)),
            wall((40.0, 0.0), (40.0, 16.0), true, (1.0, 0.0)),
            wall((40.0, 16.0), (20.0, 16.0), true, (0.0, 1.0)),
            wall((20.0, 16.0), (20.0, 30.0), true, (1.0, 0.0)),
            wall((20.0, 30.0), (0.0, 30.0), true, (0.0, 1.0)),
            wall((0.0, 30.0), (0.0, 0.0), true, (-1.0, 0.0)),
        ];
        let s = sides(&walls, &[], 0.01);
        assert_eq!(chains_of(&s, Side::North), vec![(Kind::Partitions, vec!["20'-0\"".into(), "20'-0\"".into()], "40'-0\"".into()), (Kind::Overall, vec!["40'-0\"".into()], "40'-0\"".into())]);
        assert_eq!(chains_of(&s, Side::East), vec![(Kind::Partitions, vec!["16'-0\"".into(), "14'-0\"".into()], "30'-0\"".into()), (Kind::Overall, vec!["30'-0\"".into()], "30'-0\"".into())]);
        // The notch corner's extension line begins at the wall that is nearest the string.
        let north = s.chains.iter().find(|c| c.side == Side::North && c.kind == Kind::Partitions).unwrap();
        assert!((north.stations[1].anchor.1 - 30.25).abs() < 1e-9, "{:?}", north.stations[1].anchor);
        assert!((north.stations[2].anchor.1 - 16.25).abs() < 1e-9);
        assert!(s.no_overall.is_empty());
    }

    #[test]
    fn a_wing_at_thirty_degrees_has_no_overall_where_its_strings_do_not_add() {
        // A main 30 by 20 and a wing leaving its east wall at 30 degrees: walls 20 ft and 12 ft along.
        let (c, sn) = (30f64.to_radians().cos(), 30f64.to_radians().sin());
        let a = V(30.0, 0.0);
        let b = V(a.0 + 20.0 * c, a.1 + 20.0 * sn);
        let d = V(b.0 - 12.0 * sn, b.1 + 12.0 * c);
        let walls = vec![
            wall((0.0, 0.0), (30.0, 0.0), true, (0.0, -1.0)),
            wall((30.0, 0.0), (b.0, b.1), true, (sn, -c)),
            wall((b.0, b.1), (d.0, d.1), true, (c, sn)),
            wall((d.0, d.1), (30.0, 20.0), true, (-sn, c)),
            wall((30.0, 20.0), (0.0, 20.0), true, (0.0, 1.0)),
            wall((0.0, 20.0), (0.0, 0.0), true, (-1.0, 0.0)),
        ];
        let s = sides(&walls, &[], 0.01);
        // The across extent runs to the wing's far corner; the north and south walls reach only 30 ft.
        assert!(chains_of(&s, Side::South).iter().all(|c| c.0 != Kind::Overall));
        assert!(s.no_overall.iter().any(|(side, _)| *side == Side::South));
        // The angle at the wing's corner is found: 30 degrees off the main's east side.
        let corners = corners(&walls, 0.01);
        assert!(corners.iter().any(|c| (c.degrees - 120.0).abs() < 0.01 || (c.degrees - 60.0).abs() < 0.01 || (c.degrees - 30.0).abs() < 0.01 || (c.degrees - 150.0).abs() < 0.01), "{corners:?}");
        // The wing's own wall is dimensioned along itself: the model's figure.
        assert_eq!(own(1, 20.0, true, &[]).parts(), vec!["20'-0\""]);
    }

    #[test]
    fn a_chain_of_parts_that_do_not_round_alike_still_adds_to_its_whole() {
        // 8.0418 ft is 8'-0 1/2" and 20.0417 is 20'-0 1/2": parts measured from the first station.
        let walls = vec![
            wall((0.0, 0.0), (20.0417, 0.0), true, (0.0, -1.0)),
            wall((20.0417, 0.0), (20.0417, 10.0), true, (1.0, 0.0)),
            wall((20.0417, 10.0), (0.0, 10.0), true, (0.0, 1.0)),
            wall((0.0, 10.0), (0.0, 0.0), true, (-1.0, 0.0)),
            wall((8.0418, 0.0), (8.0418, 10.0), false, (1.0, 0.0)),
        ];
        let s = sides(&walls, &[], 0.01);
        let south = s.chains.iter().find(|c| c.side == Side::South && c.kind == Kind::Partitions).unwrap();
        assert_eq!(south.parts(), vec!["8'-0 1/2\"", "12'-0\""]);
        assert_eq!(south.whole(), "20'-0 1/2\"");
        assert_eq!(south.sixteenths().iter().sum::<i64>(), *south.steps.last().unwrap());
    }

    #[test]
    fn a_partition_in_the_middle_is_located_from_the_nearest_parallel_wall_and_a_run_by_its_start() {
        // A 24 by 16 room, a short partition inside it from 6,8 to 12,8, parallel to the south and north walls.
        let mut walls = vec![
            wall((0.0, 0.0), (24.0, 0.0), true, (0.0, -1.0)),
            wall((24.0, 0.0), (24.0, 16.0), true, (1.0, 0.0)),
            wall((24.0, 16.0), (0.0, 16.0), true, (0.0, 1.0)),
            wall((0.0, 16.0), (0.0, 0.0), true, (-1.0, 0.0)),
            wall((6.0, 5.0), (12.0, 5.0), false, (0.0, 1.0)),
        ];
        let s = sides(&walls, &[], 0.01);
        assert!(!s.located[4]);
        let l = locate(&walls, &s.located, 0.01);
        assert_eq!(l.len(), 1);
        assert_eq!((l[0].wall, l[0].from), (4, 0));
        assert_eq!(written(l[0].steps[1]), "5'-0\"");
        assert!((l[0].along - 9.0).abs() < 1e-9);
        // A run of two boxes against the north wall's inner face (y = 15.75), from 1 ft: 18 in and 3 ft wide, 2 ft deep.
        let rect = |x0: f64, x1: f64| [V(x0, 13.75), V(x1, 13.75), V(x1, 15.75), V(x0, 15.75)];
        walls[2].sure = true;
        let r = runs(&walls, &[(rect(24.0 - 1.0 - 1.5, 24.0 - 1.0), false), (rect(24.0 - 4.5, 24.0 - 2.5), false)]);
        let north: Vec<&Run> = r.iter().filter(|r| r.wall == 2).collect();
        assert_eq!(north.len(), 1, "{r:?}");
        assert!((north[0].s0 - 1.0).abs() < 1e-9 && (north[0].s1 - 4.5).abs() < 1e-9 && (north[0].depth - 2.0).abs() < 1e-9, "{north:?}");
    }
}
