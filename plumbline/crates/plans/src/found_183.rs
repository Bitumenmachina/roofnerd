//! What call 183 decided about the scan's walls, after the second judge: what is done to a find once
//! its walls are made, each pass a function of the walls (and, where it needs the floor, of the places
//! the floor was seen). Said in `Wall::doubt`, `Found::beyond`, `Found::gaps`; laid only on the person's word.

use crate::found::*;

/// A face is within this of another's, on the same side, for it to be the same face seen twice: the least
/// a wall is thick (call 173's 2 in): two faces nearer than that are one.
pub(crate) const SAME_FACE: f64 = PAIR_LEAST;
/// Walls share a length when they lie alongside one another for more than this (a corner's worth, 1 ft).
pub(crate) const SHARE_ALONG: f64 = CORNER;

/// How the line of `b` lies against the line of `a`: (offset of b's middle from a's line, how far
/// along they overlap), where they are parallel within a degree; None where they are not.
fn alongside(a: &Wall, b: &Wall) -> Option<(f64, f64)> {
    let (da, db) = (unit(sub(a.to, a.from)), unit(sub(b.to, b.from)));
    if (da[0] * db[1] - da[1] * db[0]).abs().asin().to_degrees() > PAIR_DEGREES {
        return None;
    }
    let na = [-da[1], da[0]];
    let mid = [(b.from[0] + b.to[0]) / 2.0, (b.from[1] + b.to[1]) / 2.0];
    let off = dot(sub(mid, a.from), na);
    let (a0, a1): (f64, f64) = (0.0, a.length());
    let (b0, b1) = (dot(sub(b.from, a.from), da), dot(sub(b.to, a.from), da));
    let overlap = a1.min(b0.max(b1)) - a0.max(b0.min(b1));
    Some((off, overlap))
}

fn middle(w: &Wall) -> P2 {
    [(w.from[0] + w.to[0]) / 2.0, (w.from[1] + w.to[1]) / 2.0]
}
fn unit(a: P2) -> P2 {
    let l = a[0].hypot(a[1]).max(1e-12);
    [a[0] / l, a[1] / l]
}
fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}

/// A face belongs to one wall (call 183, item 1). Two walls whose bodies share a length are one wall or one of
/// them is in doubt: never both laid. A wall's body is its two faces and what is between them; a wall seen from one side
/// has one face, and the place where its other face would be is not known.
///
/// - a wall of one face that lies on a face of a wall of two (within the scatter, an inch) is that face again: in doubt;
/// - a wall of one face within 16 in (the most a wall of two faces is thick, call 173) of a wall of two, outside its
///   faces and alongside it, is a third face where there are two: in doubt (it is that wall's, or something against it);
/// - two walls of one face, on the same side within 2 in, are one face found twice; on opposite sides, each seen from
///   the side away from the other and within 24 in (the thickest a wall is, call 179 A6), they are the two faces of one wall
///   that were not paired: the shorter is in doubt;
/// - two walls of two faces whose bodies overlap: the shorter is in doubt.
///
/// What is put in doubt says which wall it lies by, how far, and along how much of it.
pub(crate) fn one_face_one_wall(walls: &mut [Wall]) {
    let n = walls.len();
    let mut doubts: Vec<(usize, String)> = Vec::new();
    for i in 0..n {
        for j in i + 1..n {
            let (a, b) = (&walls[i], &walls[j]);
            if a.doubt.is_some() || b.doubt.is_some() {
                continue;
            }
            let Some((off, overlap)) = alongside(a, b) else { continue };
            if overlap <= SHARE_ALONG {
                continue;
            }
            // Which is in doubt, and why: (the one in doubt, the one it lies by, how far).
            let verdict: Option<(usize, usize, f64)> = match (a.thickness, b.thickness) {
                (Some(ta), None) => single_by_double(j, i, off.abs(), ta, b, a),
                (None, Some(tb)) => single_by_double(i, j, off.abs(), tb, a, b),
                (Some(ta), Some(tb)) => (off.abs() < (ta + tb) / 2.0).then(|| if a.length() >= b.length() { (j, i, off.abs()) } else { (i, j, off.abs()) }),
                (None, None) => {
                    let (sa, sb) = (a.seen_from.unwrap_or([0.0, 0.0]), b.seen_from.unwrap_or([0.0, 0.0]));
                    let d = off.abs();
                    let shorter = if a.length() >= b.length() { (j, i, d) } else { (i, j, d) };
                    if sa[0] * sb[0] + sa[1] * sb[1] > 0.0 {
                        (d <= SAME_FACE).then_some(shorter)
                    } else {
                        // Seen from opposite sides, each from the side away from the other: the two faces of one wall.
                        let (ma, mb) = (middle(a), middle(b));
                        let away = dot(sub(mb, ma), sa) < 0.0 && dot(sub(ma, mb), sb) < 0.0;
                        (d >= SAME_FACE && d <= PAIR_MOST_BARE && away).then_some(shorter)
                    }
                }
            };
            if let Some((doubted, by, d)) = verdict {
                let along = alongside(&walls[by], &walls[doubted]).map_or(0.0, |x| x.1);
                doubts.push((doubted, format!("its face lies {} from wall {} along {} of it, where it already has its faces: the same face seen again, or something against it", plumbline_units(d), by + 1, plumbline_units(along))));
            }
        }
    }
    for (i, why) in doubts {
        if walls[i].doubt.is_none() {
            walls[i].doubt = Some(why);
        }
    }
}

/// A wall of one face (`single`, index `si`) beside a wall of two faces (`double`, `di`, thickness `t`), `off` from the
/// double's centre line: in doubt if it lies on a face of the double (an inch), or outside it by no more than 16 in.
fn single_by_double(si: usize, di: usize, off: f64, t: f64, _single: &Wall, _double: &Wall) -> Option<(usize, usize, f64)> {
    let out = off - t / 2.0;
    (out >= -BODY_SLACK && out <= PAIR_MOST).then_some((si, di, out.max(0.0)))
}

/// An inch: how far a face is from where it is said to lie, for the scatter of the points.
pub(crate) const BODY_SLACK: f64 = 1.0 / 12.0;

// ---- item 2: what is found beyond a wall with no way to it ----

use std::collections::HashMap;

/// Floor cells are 1 ft square, as `find` made them.
const CELL: f64 = 1.0;
/// Floor is joined across cells this far apart (so that a table's shadow does not part a room): two cells.
const JOIN: i64 = 2;
/// How far from a wall's face the floor beside it is looked for.
const BESIDE: f64 = 0.75;
/// Where there is no floor beside a wall, that is how far out the nearest floor cell may lie and still be "beside" it.
const NEAR_FLOOR: f64 = 2.0;
/// Two regions of floor are alike in size, so that the one the scan began on is the house, where the smaller is at least this
/// share of the larger.
const ALIKE: f64 = 0.8;
/// Whether a wall of two faces (a wall with a body, measured) bars the way as a wall of one face does. Call 183 says "a wall with no opening leading to
/// it"; read to the letter every room behind a partition whose doorway the scan did not show is beyond, and the made-up houses the finder was built on
/// (steps 1 to 12 of demos/findwalls.sh) all have a partition with no doorway in it, and would lose the rooms behind them. So a wall that was seen to have a body
/// (both its faces found) is a wall that can be gone round or through where the scan did not see it; a wall of one face (a mirror, glass, the outside
/// seen from within) bars the way. Said in the report as a place where the call and the existing steps pull against one another.
const TWO_FACED_WALL_BARS_THE_WAY: bool = false;
/// A stretch of wall beyond is only carved from the rest where it runs at least this long (a wall is not under 2 ft).
const CARVE_LEAST: f64 = MIN_FACE;

/// A placeholder in a doubt for the number of the wall that stands in the way: the wall's place before the walls are numbered.
fn mark(k: usize) -> String {
    format!("\u{1}{k}\u{1}")
}

struct Plan<'a> {
    walls: &'a [Wall],
    /// The cell of floor (index) -> its region.
    region: HashMap<(i64, i64), usize>,
    anchored: usize,
    cells: Vec<P2>,
}

impl Plan<'_> {
    fn cell(p: P2) -> (i64, i64) {
        ((p[0] / CELL).floor() as i64, (p[1] / CELL).floor() as i64)
    }

    /// The first wall (but `skip`) that the straight way from `p` to `q` crosses where it has no doorway; walls in doubt are not walls here.
    fn blocker(&self, p: P2, q: P2, skip: Option<usize>) -> Option<usize> {
        let r = sub(q, p);
        let mut best: Option<(f64, usize)> = None;
        for (i, w) in self.walls.iter().enumerate() {
            if Some(i) == skip || w.doubt.is_some() || (w.thickness.is_some() && !TWO_FACED_WALL_BARS_THE_WAY) {
                continue;
            }
            let e = sub(w.to, w.from);
            let denom = r[0] * e[1] - r[1] * e[0];
            if denom.abs() < 1e-12 {
                continue;
            }
            let diff = sub(w.from, p);
            let t = (diff[0] * e[1] - diff[1] * e[0]) / denom;
            let u = (diff[0] * r[1] - diff[1] * r[0]) / denom;
            if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u) {
                continue;
            }
            let along = u * w.length();
            if w.openings.iter().any(|o| !o.window && along >= o.from && along <= o.to) {
                continue;
            }
            if best.is_none_or(|b| t < b.0) {
                best = Some((t, i));
            }
        }
        best.map(|b| b.1)
    }

    fn regions_near(&self, p: P2, rings: i64) -> Vec<usize> {
        let (ci, cj) = Plan::cell(p);
        let mut out = Vec::new();
        for di in -rings..=rings {
            for dj in -rings..=rings {
                if let Some(r) = self.region.get(&(ci + di, cj + dj)) {
                    out.push(*r);
                }
            }
        }
        out
    }

    fn nearest_anchored(&self, p: P2) -> Option<P2> {
        self.cells.iter().copied().filter(|c| self.region.get(&Plan::cell(*c)) == Some(&self.anchored)).min_by(|a, b| distance(*a, p).total_cmp(&distance(*b, p)))
    }

    /// Whether a place is reached from the floor that was walked: None where it is, the wall in the way (or `usize::MAX` where no
    /// wall can be named) where it is not.
    fn beyond(&self, sides: &[P2], skip: Option<usize>) -> Option<usize> {
        let mut blocked: Option<usize> = None;
        let mut any_floor = false;
        for &p in sides {
            let near = self.regions_near(p, 1);
            if near.contains(&self.anchored) {
                return None;
            }
            let wide = if near.is_empty() { self.regions_near(p, NEAR_FLOOR as i64) } else { near };
            if wide.contains(&self.anchored) {
                return None;
            }
            any_floor |= !wide.is_empty();
            // The way to the nearest floor that was walked, as the crow flies: a wall across it with no doorway is what is in the way.
            let Some(a) = self.nearest_anchored(p) else { continue };
            match self.blocker(p, a, skip) {
                Some(k) => blocked = blocked.or(Some(k)),
                None if any_floor => blocked = blocked.or(Some(usize::MAX)),
                None => return None,
            }
        }
        blocked
    }
}

fn distance(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// The floor seen, in regions: cells joined where no wall with no doorway lies between them. Returns the regions, how many cells each
/// holds, and which is the house (the largest, and where two are alike in size the one the scan begins on: its own zero), with whether
/// that was a tie.
fn regions(cells: &[P2], walls: &[Wall]) -> (HashMap<(i64, i64), usize>, usize, bool) {
    let mut region: HashMap<(i64, i64), usize> = HashMap::new();
    let by_cell: HashMap<(i64, i64), P2> = cells.iter().map(|c| (Plan::cell(*c), *c)).collect();
    let probe = Plan { walls, region: HashMap::new(), anchored: 0, cells: Vec::new() };
    let mut sizes: Vec<usize> = Vec::new();
    let mut keys: Vec<(i64, i64)> = by_cell.keys().copied().collect();
    keys.sort();
    for k in keys {
        if region.contains_key(&k) {
            continue;
        }
        let id = sizes.len();
        sizes.push(0);
        let mut stack = vec![k];
        region.insert(k, id);
        while let Some((ci, cj)) = stack.pop() {
            sizes[id] += 1;
            for di in -JOIN..=JOIN {
                for dj in -JOIN..=JOIN {
                    let n = (ci + di, cj + dj);
                    if (di == 0 && dj == 0) || region.contains_key(&n) {
                        continue;
                    }
                    let (Some(p), Some(q)) = (by_cell.get(&(ci, cj)), by_cell.get(&n)) else { continue };
                    if probe.blocker(*p, *q, None).is_none() {
                        region.insert(n, id);
                        stack.push(n);
                    }
                }
            }
        }
    }
    let Some(largest) = (0..sizes.len()).max_by_key(|i| sizes[*i]) else { return (region, 0, false) };
    let alike: Vec<usize> = (0..sizes.len()).filter(|i| sizes[*i] as f64 >= ALIKE * sizes[largest] as f64).collect();
    if alike.len() > 1 {
        // The one the scan begins on: of the regions alike in size, the one whose floor lies nearest the scan's own zero.
        let nearest = by_cell.iter().filter(|(k, _)| alike.contains(&region[*k])).min_by(|a, b| a.1[0].hypot(a.1[1]).total_cmp(&b.1[0].hypot(b.1[1]))).map(|(k, _)| region[k]);
        if let Some(r) = nearest {
            return (region, r, true);
        }
    }
    (region, largest, false)
}

/// A wall cut at `cut` feet from its `from` end: the two walls, its openings and what is not seen going to the piece they lie in.
fn split_wall(w: &Wall, cut: f64) -> (Wall, Wall) {
    let d = unit(sub(w.to, w.from));
    let p = [w.from[0] + d[0] * cut, w.from[1] + d[1] * cut];
    let mut a = Wall { from: w.from, to: p, openings: Vec::new(), hidden: Vec::new(), ..w.clone() };
    let mut b = Wall { from: p, to: w.to, openings: Vec::new(), hidden: Vec::new(), ..w.clone() };
    for o in &w.openings {
        if (o.from + o.to) / 2.0 < cut {
            a.openings.push(*o);
        } else {
            b.openings.push(Opening { from: o.from - cut, to: o.to - cut, ..*o });
        }
    }
    for h in &w.hidden {
        if (h.from + h.to) / 2.0 < cut {
            a.hidden.push(*h);
        } else {
            b.hidden.push(Stretch { from: h.from - cut, to: h.to - cut });
        }
    }
    (a, b)
}

/// What was found that the scan saw from the floor that was walked, and what was found beyond a wall with no way to it (call 183, item 2):
/// a mirror's room, a neighbour's wall through glass, a room not walked into. A wall found beyond is in doubt and is not laid without the
/// person's word; where a wall runs from the house into what lies beyond, it is cut where it leaves the house and the part beyond is
/// in doubt; a thing beyond is taken out of what stood short and said. Returns the walls (each with the place it had), what was taken
/// out of the things, and what is said of which floor is the house.
pub(crate) fn beyond(walls: Vec<Wall>, things: &mut Vec<Thing>, cells: &[P2]) -> (Vec<(Wall, usize)>, Vec<(Thing, String)>, Option<String>) {
    let plain = |walls: Vec<Wall>| walls.into_iter().enumerate().map(|(i, w)| (w, i)).collect::<Vec<_>>();
    if walls.len() < 2 || cells.is_empty() {
        return (plain(walls), Vec::new(), None);
    }
    let (region, anchored, tie) = regions(cells, &walls);
    let plan = Plan { walls: &walls, region, anchored, cells: cells.to_vec() };
    let why = |k: usize| {
        let by = if k == usize::MAX { "the floor that was walked".to_string() } else { format!("wall {}", mark(k)) };
        format!("in doubt: beyond {by} with no way to it: what the scan saw from the floor it was walked on does not lead here (a mirror, a window, a neighbour's wall, or a room not walked into)")
    };
    let mut out: Vec<(Wall, usize)> = Vec::new();
    for (i, w) in walls.iter().enumerate() {
        if w.doubt.is_some() {
            out.push((w.clone(), i));
            continue;
        }
        let length = w.length();
        let d = unit(sub(w.to, w.from));
        let n = [-d[1], d[0]];
        let steps = (length / CELL).ceil().max(1.0) as usize;
        let mut verdict: Vec<Option<usize>> = Vec::new();
        for k in 0..steps {
            let s = (k as f64 + 0.5) * length / steps as f64;
            let c = [w.from[0] + d[0] * s, w.from[1] + d[1] * s];
            let sides: Vec<P2> = match (w.thickness, w.seen_from) {
                (Some(t), _) => [1.0, -1.0].iter().map(|sg| [c[0] + n[0] * sg * (t / 2.0 + BESIDE), c[1] + n[1] * sg * (t / 2.0 + BESIDE)]).collect(),
                (None, Some(sf)) => vec![[c[0] + sf[0] * BESIDE, c[1] + sf[1] * BESIDE]],
                (None, None) => [1.0, -1.0].iter().map(|sg| [c[0] + n[0] * sg * BESIDE, c[1] + n[1] * sg * BESIDE]).collect(),
            };
            verdict.push(plan.beyond(&sides, Some(i)));
        }
        // Runs of beyond, each at least a wall long.
        let step_len = length / steps as f64;
        let mut runs: Vec<(usize, usize, usize)> = Vec::new();
        let mut k = 0;
        while k < steps {
            if let Some(b) = verdict[k] {
                let mut e = k;
                while e + 1 < steps && verdict[e + 1].is_some() {
                    e += 1;
                }
                if (e - k + 1) as f64 * step_len >= CARVE_LEAST {
                    runs.push((k, e + 1, b));
                }
                k = e + 1;
            } else {
                k += 1;
            }
        }
        if runs.is_empty() {
            out.push((w.clone(), i));
            continue;
        }
        // Where the wall in the way crosses this one, the cut is there (within a step and a half of where the sampling changed).
        let cut_at = |boundary: f64, by: usize| -> f64 {
            if by == usize::MAX {
                return boundary;
            }
            let v = &walls[by];
            let (e, r) = (sub(v.to, v.from), sub(w.to, w.from));
            let denom = r[0] * e[1] - r[1] * e[0];
            if denom.abs() < 1e-12 {
                return boundary;
            }
            let diff = sub(v.from, w.from);
            let t = (diff[0] * e[1] - diff[1] * e[0]) / denom;
            let s = t * length;
            if (s - boundary).abs() <= 1.5 * step_len { s } else { boundary }
        };
        let mut pieces: Vec<(f64, f64, Option<usize>)> = Vec::new();
        let mut at = 0.0;
        for (a, b, by) in runs {
            let (lo, hi) = (cut_at(a as f64 * step_len, by).clamp(0.0, length), if b == steps { length } else { cut_at(b as f64 * step_len, by).clamp(0.0, length) });
            let lo = if a == 0 { 0.0 } else { lo };
            if lo > at + 1e-6 {
                pieces.push((at, lo, None));
            }
            pieces.push((lo, hi, Some(by)));
            at = hi;
        }
        if at < length - 1e-6 {
            pieces.push((at, length, None));
        }
        // Cut the wall into those pieces.
        let mut rest = w.clone();
        let mut done = 0.0;
        for (idx, (_, hi, by)) in pieces.iter().enumerate() {
            let last = idx + 1 == pieces.len();
            let (mut head, tail) = if last { (rest.clone(), rest.clone()) } else { split_wall(&rest, hi - done) };
            if let Some(b) = by {
                head.doubt = Some(why(*b));
            }
            out.push((head, i));
            rest = tail;
            done = *hi;
        }
    }
    // Things beyond: what stands where no floor that was walked is beside it.
    let mut gone: Vec<(Thing, String)> = Vec::new();
    things.retain(|t| match plan.beyond(&[t.at], None) {
        Some(k) => {
            let by = if k == usize::MAX { "the floor that was walked".to_string() } else { format!("wall {}", mark(k)) };
            gone.push((*t, format!("something {}' {}\" high, at {}' {}\" east, {}' {}\" north, is beyond {by} with no way to it: it is not counted among what stood short", (t.height * 12.0).round() as i64 / 12, (t.height * 12.0).round() as i64 % 12, (t.at[0] * 12.0).round() as i64 / 12, (t.at[0] * 12.0).round() as i64 % 12, (t.at[1] * 12.0).round() as i64 / 12, (t.at[1] * 12.0).round() as i64 % 12)));
            false
        }
        None => true,
    });
    let any_beyond = out.iter().any(|(w, _)| w.doubt.as_deref().is_some_and(|d| d.starts_with("in doubt: beyond"))) || !gone.is_empty();
    let note = (any_beyond && tie).then(|| "which side of the wall the house is on is not in the scan (the floor seen is as much on one side as the other): it is taken as the side the scan begins on".to_string());
    (out, gone, note)
}

/// The numbers of the walls that stand in the way are written where the walls are numbered: `marks` put by `beyond` are the walls' places
/// before they were put in order; `order` says where each place went.
pub(crate) fn resolve(text: &str, place_of: &dyn Fn(usize) -> usize) -> String {
    let mut out = String::new();
    let mut parts = text.split('\u{1}');
    out.push_str(parts.next().unwrap_or(""));
    while let (Some(k), Some(rest)) = (parts.next(), parts.next()) {
        out.push_str(&(place_of(k.parse().unwrap_or(0)) + 1).to_string());
        out.push_str(rest);
    }
    out
}

// ---- item 7: how thick the scan is, place by place ----

use crate::scan::Scan;
use std::collections::HashSet;

/// A cell of floor has at most this many points above it, between 6 in and 7 ft up (strays, a lamp).
const ABOVE_MOST: u32 = 3;
/// The scan's floor is looked at in blocks this wide (8 ft) for how thick it is.
const ZONE: f64 = 8.0;
/// Two parts of a scan are of different thickness where the thinner's blocks hold this many times fewer points to the square foot than the thicker's.
const ZONE_RATIO: f64 = 2.5;
/// A block is floor where it holds at least this many of its 64 cells of a foot that have floor points (3 in 8: the foot of a wall that runs along the block's edge,
/// with its spill, has 16 at most; a block outside the house has none).
const ZONE_CELLS: usize = 24;
/// A point over a block with no floor of its own (the edge of the house, where a wall stands) belongs to the part whose blocks it lies nearer, within this (1 ft 6 in).
/// A wall that runs from one part into the next is found in each, cut where they meet, and joined.
const ZONE_EDGE: f64 = 1.5;

/// The parts of the scan of different thickness: the blocks of the thinner and of the thicker, and about how thick each is. None where the scan is of one
/// thickness (the thinnest block is not under half the thickest), so that a scan of even thickness is read as it always was.
pub(crate) struct Zones {
    pub thick: HashSet<(i64, i64)>,
    pub thin: HashSet<(i64, i64)>,
    pub thick_rho: f64,
    pub thin_rho: f64,
}

pub(crate) fn zones(scan: &Scan, floor: f64) -> Option<Zones> {
    let mut cells: HashMap<(i64, i64), u32> = HashMap::new();
    let mut above: HashMap<(i64, i64), u32> = HashMap::new();
    for p in &scan.points {
        let up = p[2] - floor;
        if up.abs() <= 0.25 {
            *cells.entry((p[0].floor() as i64, p[1].floor() as i64)).or_default() += 1;
        } else if (0.5..=7.0).contains(&up) {
            *above.entry((p[0].floor() as i64, p[1].floor() as i64)).or_default() += 1;
        }
    }
    // Floor is where nothing stands above it: the foot of a wall, which holds more points near the floor than floor does (a wall is often scanned from
    // several passes), has the wall's own points above it in the cell.
    cells.retain(|c, _| above.get(c).copied().unwrap_or(0) <= ABOVE_MOST);
    let block_of = |c: (i64, i64)| (c.0.div_euclid(ZONE as i64), c.1.div_euclid(ZONE as i64));
    let mut by_block: HashMap<(i64, i64), Vec<f64>> = HashMap::new();
    for (c, n) in &cells {
        if *n >= 2 {
            by_block.entry(block_of(*c)).or_default().push(f64::from(*n));
        }
    }
    let mut blocks: Vec<((i64, i64), f64)> = by_block.into_iter().filter(|(_, v)| v.len() >= ZONE_CELLS).map(|(b, mut v)| {
        v.sort_by(f64::total_cmp);
        (b, v[v.len() / 2])
    }).collect();
    if blocks.len() < 2 {
        return None;
    }
    blocks.sort_by(|a, b| a.1.total_cmp(&b.1));
    // The widest step between one block's thickness and the next, in order: the parts divide there.
    let (at, ratio) = (0..blocks.len() - 1).map(|i| (i, blocks[i + 1].1 / blocks[i].1.max(1e-9))).max_by(|a, b| a.1.total_cmp(&b.1))?;
    if ratio < ZONE_RATIO {
        return None;
    }
    let thin: HashSet<(i64, i64)> = blocks[..=at].iter().map(|b| b.0).collect();
    let thick: HashSet<(i64, i64)> = blocks[at + 1..].iter().map(|b| b.0).collect();
    let mid = |part: &[((i64, i64), f64)]| part[part.len() / 2].1;
    Some(Zones { thick_rho: mid(&blocks[at + 1..]), thin_rho: mid(&blocks[..=at]), thick, thin })
}

/// The points of one part of the scan: those over its blocks, those over blocks with no floor of their own that lie nearer to this part's than to the other's
/// (a wall at the edge of the floor), and those of the other part within the strip along the boundary.
pub(crate) fn part_of(scan: &Scan, own: &HashSet<(i64, i64)>, other: &HashSet<(i64, i64)>) -> Scan {
    let block = |p: &[f64; 3]| ((p[0] / ZONE).floor() as i64, (p[1] / ZONE).floor() as i64);
    let rect_distance = |p: &[f64; 3], b: (i64, i64)| {
        let (x0, y0) = (b.0 as f64 * ZONE, b.1 as f64 * ZONE);
        let (dx, dy) = ((x0 - p[0]).max(p[0] - (x0 + ZONE)).max(0.0), (y0 - p[1]).max(p[1] - (y0 + ZONE)).max(0.0));
        dx.hypot(dy)
    };
    // How far a point is from the nearest block of a part (a block neighbouring its own).
    let reach = |p: &[f64; 3], set: &HashSet<(i64, i64)>| {
        let b = block(p);
        (-1..=1).flat_map(|di| (-1..=1).map(move |dj| (b.0 + di, b.1 + dj))).filter(|nb| set.contains(nb)).map(|nb| rect_distance(p, nb)).fold(f64::MAX, f64::min)
    };
    let points: Vec<[f64; 3]> = scan
        .points
        .iter()
        .filter(|p| {
            let b = block(p);
            if own.contains(&b) {
                return true;
            }
            if other.contains(&b) {
                return false;
            }
            // A block with no floor of its own: a wall at the foot of the house's edge: it goes with the part it lies nearer to, within ZONE_EDGE of it.
            let (mine, theirs) = (reach(p, own), reach(p, other));
            mine <= ZONE_EDGE && mine <= theirs
        })
        .copied()
        .collect();
    Scan { points, lost: Vec::new() }
}

/// A wall's number in what was said, where a wall is named ("wall 3"), is written again by `map`.
fn renumber(text: &str, map: &dyn Fn(usize) -> usize) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find("wall ") {
        out.push_str(&rest[..i + 5]);
        rest = &rest[i + 5..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = digits.parse::<usize>() {
            out.push_str(&map(n - 1).saturating_add(1).to_string());
            rest = &rest[digits.len()..];
        }
    }
    out.push_str(rest);
    out
}

/// What two parts of the scan found, as one find: the walls joined where one wall runs from one part into the next (the same line, within 3 in, the ends meeting
/// or overlapping), with its openings, and what was left out, said, and stood short, together. `thick` is the thicker part's (its floor, ceiling and measures stand).
pub(crate) fn merge(thick: Found, thin: Found, note: String) -> Found {
    let mut out = thick;
    // The walls: thick's first, then thin's; each thin wall that lies along a thick wall is joined to it.
    let mut merged: Vec<Wall> = out.walls.clone();
    let thick_map: Vec<usize> = (0..merged.len()).collect();
    let mut thin_map: Vec<usize> = Vec::new();
    for w in &thin.walls {
        let mut joined: Option<usize> = None;
        for (k, m) in merged.iter().enumerate() {
            let Some((off, overlap)) = alongside(m, w) else { continue };
            let (da, db) = (unit(sub(m.to, m.from)), unit(sub(w.to, w.from)));
            let gap_ok = overlap > -JOIN_GAP;
            if off.abs() <= JOIN_OFF && gap_ok && m.thickness.is_some() == w.thickness.is_some() && (da[0] * db[0] + da[1] * db[1]) > 0.0 {
                joined = Some(k);
                break;
            }
        }
        match joined {
            Some(k) => {
                let m = merged[k].clone();
                merged[k] = join_walls(&m, w);
                thin_map.push(k);
            }
            None => {
                merged.push(w.clone());
                thin_map.push(merged.len() - 1);
            }
        }
    }
    // The doubts name walls by their place in each part: write them as the places in the whole (before the walls are put in order, which `find` does once the parts are one).
    let (tm, nm) = (thick_map.clone(), thin_map.clone());
    for (k, w) in merged.iter_mut().enumerate() {
        if let Some(d) = w.doubt.clone() {
            let in_thick = k < out.walls.len();
            w.doubt = Some(if in_thick { renumber(&d, &|n| tm.get(n).copied().unwrap_or(n)) } else { renumber(&d, &|n| nm.get(n).copied().unwrap_or(n)) });
        }
    }
    out.walls = merged;
    for t in thin.things {
        if !out.things.iter().any(|u| distance(u.at, t.at) < 1.0) {
            out.things.push(t);
        }
    }
    out.squared = (out.squared.0 + thin.squared.0, out.squared.1 + thin.squared.1);
    for l in thin.left_out {
        if !out.left_out.contains(&l) {
            out.left_out.push(l);
        }
    }
    out.left.extend(thin.left);
    out.stood.extend(thin.stood);
    out.curves.extend(thin.curves);
    out.beyond_things.extend(thin.beyond_things);
    out.zone_note = Some(note);
    out
}

/// Where a wall was cut at the boundary of two parts, each part's end of it falls short of the boundary by what the end's measure loses (an edge is told to
/// the point from the points beyond it, and there are none): the two ends are one wall where they are this near (two corners, 2 ft).
const JOIN_GAP: f64 = 2.0 * CORNER;
/// Two walls lie along one another (the same line within this, 3 in) to be one wall.
const JOIN_OFF: f64 = 0.25;

/// One wall from two that lie along one line: from the lesser end to the greater, with the openings and the stretches not seen of both (the thick's first).
fn join_walls(a: &Wall, b: &Wall) -> Wall {
    let da = unit(sub(a.to, a.from));
    let along = |p: P2| dot(sub(p, a.from), da);
    let (b0, b1) = (along(b.from), along(b.to));
    let (lo, hi) = (0.0f64.min(b0.min(b1)), a.length().max(b0.max(b1)));
    let from = [a.from[0] + da[0] * lo, a.from[1] + da[1] * lo];
    let to = [a.from[0] + da[0] * hi, a.from[1] + da[1] * hi];
    let mut w = Wall { from, to, openings: Vec::new(), hidden: Vec::new(), ..a.clone() };
    w.doubt = a.doubt.clone().or_else(|| b.doubt.clone());
    let forward = b1 >= b0;
    for o in &a.openings {
        w.openings.push(Opening { from: o.from - lo, to: o.to - lo, ..*o });
    }
    for o in &b.openings {
        let (f, t) = if forward { (b0 + o.from, b0 + o.to) } else { (b0 - o.to, b0 - o.from) };
        let (f, t) = (f - lo, t - lo);
        if !w.openings.iter().any(|x| x.to.min(t) - x.from.max(f) > 0.5 * (t - f)) {
            w.openings.push(Opening { from: f, to: t, ..*o });
        }
    }
    for h in &a.hidden {
        w.hidden.push(Stretch { from: h.from - lo, to: h.to - lo });
    }
    for h in &b.hidden {
        let (f, t) = if forward { (b0 + h.from, b0 + h.to) } else { (b0 - h.to, b0 - h.from) };
        let (f, t) = (f - lo, t - lo);
        if !w.hidden.iter().any(|x| x.to.min(t) - x.from.max(f) > 0.5 * (t - f)) {
            w.hidden.push(Stretch { from: f, to: t });
        }
    }
    w.openings.sort_by(|x, y| x.from.total_cmp(&y.from));
    w.hidden.sort_by(|x, y| x.from.total_cmp(&y.from));
    w
}
