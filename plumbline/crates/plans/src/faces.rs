//! The faces a scan's sheet shows (DECISIONS.md call 190): the straight lines the scan's points make
//! where it is cut, read from the sheet's own picture (so they are there again when a job is opened
//! without the scan's file), each with the side its room is on and the other face of its wall where
//! the sheet shows both. The Wall tool snaps to these as it does to a drawn plan's lines, and a wall
//! traced over a scan is set by the face. Nothing here lays a wall or marks anything Known.
//!
//! The sheet is the picture `Scan::cut` writes: each point of the scan a gray dot four across
//! (drawn from -1 to +2 dots about the point's own dot, so the dot's middle is one whole dot, 1/2 in
//! at 24 to the foot, up and east of the point), on white paper. What is returned is in the
//! page's own units, dots from its lower-left corner, as `Page::lines` is.
//!
//! How it is read. The dots are thinned by one dot all round (a pixel stays when its eight
//! neighbours are ink): a lone dot is left as its own middle two by two, and a band of dots as the
//! band without its edge, so that what is fitted is the centres of the points, not the paper they
//! blacken. A pixel of what is left is then taken one dot down and west of where it stands, which
//! puts back the half dot of the drawing: a face the scan puts at a whole foot is read at that foot.
//! The lines are found as a Hough transform finds them (the best line through the most points, its
//! points taken away, the next), each then fitted straight through its points' centres by total
//! least squares, cut into the stretches where the points run on (a gap past `GAP_FT` ends one),
//! and parallel stretches that lie in one band of points are made one face.

/// The shortest a face is, in feet. The lead's (call 190) and Assumed: moves only by a written call.
pub const FACE_LEAST_FT: f64 = 2.0;
/// Points of one line farther apart than this, in feet, are two faces (an opening, a cabinet in front).
/// Assumed.
const GAP_FT: f64 = 1.5;
/// Two faces are one wall's pair when parallel within this many degrees ...
const PAIR_ANGLE_DEG: f64 = 2.0;
/// ... from this many inches apart ...
const PAIR_LEAST_IN: f64 = 2.0;
/// ... to this many ...
const PAIR_MOST_IN: f64 = 14.0;
/// ... overlapping over this many feet ...
const PAIR_OVERLAP_FT: f64 = 2.0;
/// ... with next to no points between them: under this part of the fill of the faces themselves.
const BETWEEN_PART: f64 = 0.15;
/// A line this many degrees off another is not parallel to it when parallel stretches are joined.
const JOIN_ANGLE_DEG: f64 = 3.0;
/// How far from a face, in feet, its room is looked for.
const ROOM_REACH_FT: f64 = 8.0;
/// The two sides are alike (the room is not told) when they differ by under this part of the greater.
const ROOM_TIE: f64 = 0.1;
/// The most points of a sheet kept to work on: a sheet of more has every so-many-th.
const MOST_POINTS: usize = 150_000;
/// A pixel this dark or darker is a point's ink (a dot is 100, paper 255).
const INK: u8 = 180;

/// A face of the sheet.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    /// Its two ends, in page units from the lower-left corner. It runs from the west end to the east
    /// (or, where it is upright, from the south end to the north).
    pub line: [[f64; 2]; 2],
    /// Some(true): the room is to the left of `line[0]` to `line[1]`; Some(false): to its right;
    /// None: the sheet cannot tell. The room is the side with more of the sheet's points within 8 ft
    /// of the face (beyond 14 in of it, so that the other face of its own wall is not counted). Where
    /// the two sides are within a tenth of one another, None.
    pub room: Option<bool>,
    /// The other face of the same wall: its place in the list and how far apart the two are, in page
    /// units. Two faces parallel within 2 degrees, 2 in to 14 in apart, overlapping over at least 2 ft,
    /// with next to no points between them.
    pub pair: Option<(usize, f64)>,
}

/// The faces of a sheet of gray, rows from the top, one byte to the dot. `dots_per_foot` is the sheet's
/// own (24 as `Scan::cut` makes it).
pub fn faces(gray: &[u8], width: u32, height: u32, dots_per_foot: f64) -> Vec<Face> {
    let (w, h) = (width as usize, height as usize);
    if w < 3 || h < 3 || gray.len() < w * h || !(dots_per_foot > 0.0) {
        return Vec::new();
    }
    let dpf = dots_per_foot;
    let all = centres(gray, w, h);
    if all.len() < 2 {
        return Vec::new();
    }
    let mut cands = candidates(&all, w, h, dpf);
    join(&mut cands, &all, dpf);
    slivers(&mut cands, dpf);
    let mut out: Vec<Cand> = cands.into_iter().filter(|c| c.length() >= FACE_LEAST_FT * dpf).collect();
    out.sort_by(|a, b| b.length().total_cmp(&a.length()).then(a.c[0].total_cmp(&b.c[0])).then(a.c[1].total_cmp(&b.c[1])));
    let mut list: Vec<Face> = out.iter().map(|c| Face { line: c.ends(), room: room(c, &all, dpf), pair: None }).collect();
    for (i, j, g) in pairs(&out, &all, dpf) {
        list[i].pair = Some((j, g));
    }
    list
}

/// The same for a picture as the page reader hands it over: four bytes to the dot, red, green, blue and
/// opacity (a dot with no opacity is paper).
pub fn faces_rgba(rgba: &[u8], width: u32, height: u32, dots_per_foot: f64) -> Vec<Face> {
    let n = width as usize * height as usize;
    if rgba.len() < n * 4 {
        return Vec::new();
    }
    let gray: Vec<u8> = (0..n).map(|i| { let p = &rgba[i * 4..i * 4 + 4]; if p[3] < 128 { 255 } else { ((u16::from(p[0]) + u16::from(p[1]) + u16::from(p[2])) / 3) as u8 } }).collect();
    faces(&gray, width, height, dots_per_foot)
}

/// The middles of the sheet's points, in page units (see the head of this file for the half dot).
fn centres(gray: &[u8], w: usize, h: usize) -> Vec<[f64; 2]> {
    let ink = |x: usize, y: usize| gray[y * w + x] < INK;
    let mut out = Vec::new();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if ink(x, y) && ink(x - 1, y) && ink(x + 1, y) && ink(x, y - 1) && ink(x - 1, y - 1) && ink(x + 1, y - 1) && ink(x, y + 1) && ink(x - 1, y + 1) && ink(x + 1, y + 1) {
                // The pixel's middle (x + 1/2, h - y - 1/2 from the bottom), one dot down and west.
                out.push([x as f64 - 0.5, (h - y) as f64 - 1.5]);
            }
        }
    }
    if out.len() > MOST_POINTS {
        let step = out.len().div_ceil(MOST_POINTS);
        out = out.into_iter().step_by(step).collect();
    }
    out
}

/// A straight stretch of points: where it is and the points it is fitted through.
#[derive(Clone, Debug)]
struct Cand {
    /// The middle of its points and the way it runs (unit).
    c: [f64; 2],
    d: [f64; 2],
    /// How far its points run along it from the middle, least and most.
    lo: f64,
    hi: f64,
    pts: Vec<[f64; 2]>,
}

impl Cand {
    /// Fitted through `pts` by total least squares.
    fn fit(pts: Vec<[f64; 2]>) -> Cand {
        let n = pts.len() as f64;
        let (mx, my) = (pts.iter().map(|p| p[0]).sum::<f64>() / n, pts.iter().map(|p| p[1]).sum::<f64>() / n);
        let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
        for p in &pts {
            let (x, y) = (p[0] - mx, p[1] - my);
            sxx += x * x;
            sxy += x * y;
            syy += y * y;
        }
        let a = 0.5 * (2.0 * sxy).atan2(sxx - syy);
        let mut d = [a.cos(), a.sin()];
        // Which way it is written: east, or north where it is upright.
        if d[0] < -1e-12 || (d[0].abs() <= 1e-12 && d[1] < 0.0) {
            d = [-d[0], -d[1]];
        }
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for p in &pts {
            let s = (p[0] - mx) * d[0] + (p[1] - my) * d[1];
            lo = lo.min(s);
            hi = hi.max(s);
        }
        Cand { c: [mx, my], d, lo, hi, pts }
    }

    /// Its length: the points' middles run a dot (one pixel, the thinned dot's own width) further than
    /// the face they stand on.
    fn length(&self) -> f64 {
        (self.hi - self.lo - 1.0).max(0.0)
    }

    fn ends(&self) -> [[f64; 2]; 2] {
        let (a, b) = (self.lo + 0.5, (self.hi - 0.5).max(self.lo + 0.5));
        [[self.c[0] + self.d[0] * a, self.c[1] + self.d[1] * a], [self.c[0] + self.d[0] * b, self.c[1] + self.d[1] * b]]
    }

    /// How far a point is from it, to its left.
    fn across(&self, p: [f64; 2]) -> f64 {
        (p[0] - self.c[0]) * -self.d[1] + (p[1] - self.c[1]) * self.d[0]
    }

    fn along(&self, p: [f64; 2]) -> f64 {
        (p[0] - self.c[0]) * self.d[0] + (p[1] - self.c[1]) * self.d[1]
    }
}

/// The straight stretches of the points: the line through the most points, cut into the stretches
/// where its points run on, then its points taken away and the next.
fn candidates(all: &[[f64; 2]], w: usize, h: usize, dpf: f64) -> Vec<Cand> {
    const STEPS: usize = 360; // a half degree each
    let tables: Vec<(f64, f64)> = (0..STEPS).map(|i| { let a = std::f64::consts::PI * i as f64 / STEPS as f64; (a.cos(), a.sin()) }).collect();
    // rho runs from -w (a point at the east edge, the line turned to the west) to w + h.
    let (shift, bins) = (w as f64 + 2.0, 2 * w + h + 6);
    let mut votes = vec![0u32; STEPS * bins];
    let bin = |p: [f64; 2], t: (f64, f64)| ((p[0] * t.0 + p[1] * t.1 + shift).round().max(0.0) as usize).min(bins - 1);
    for p in all {
        for (i, t) in tables.iter().enumerate() {
            votes[i * bins + bin(*p, *t)] += 1;
        }
    }
    let least = (0.75 * FACE_LEAST_FT * dpf) as u32;
    let mut alive = vec![true; all.len()];
    let mut out: Vec<Cand> = Vec::new();
    for _ in 0..400 {
        // The best of the lines: the most points in a bin and the two beside it.
        let (mut best, mut at) = (0u32, (0usize, 0usize));
        for i in 0..STEPS {
            let row = &votes[i * bins..(i + 1) * bins];
            for r in 1..bins - 1 {
                if row[r] == 0 {
                    continue;
                }
                let v = row[r - 1] + row[r] + row[r + 1];
                if v > best {
                    best = v;
                    at = (i, r);
                }
            }
        }
        if best < least {
            break;
        }
        let t = tables[at.0];
        let rho = at.1 as f64 - shift;
        // Fitted again and again through the points within a few dots of it.
        let mut line = (([rho * t.0, rho * t.1]), [-t.1, t.0]);
        let mut band = 4.0;
        let mut inl: Vec<usize> = Vec::new();
        for _ in 0..4 {
            let n = [-line.1[1], line.1[0]];
            inl = (0..all.len()).filter(|i| alive[*i] && ((all[*i][0] - line.0[0]) * n[0] + (all[*i][1] - line.0[1]) * n[1]).abs() <= band).collect();
            if inl.len() < 2 {
                break;
            }
            let c = Cand::fit(inl.iter().map(|i| all[*i]).collect());
            line = (c.c, c.d);
            band = 3.0;
        }
        if inl.len() < 2 {
            // Nothing there to take: put the bin out so that it is not found again.
            for k in 0..STEPS {
                votes[k * bins + at.1] = 0;
            }
            votes[at.0 * bins + at.1] = 0;
            continue;
        }
        // The points of the line, wider where the points are scattered wider, are taken away.
        let n = [-line.1[1], line.1[0]];
        let dist = |p: [f64; 2]| (p[0] - line.0[0]) * n[0] + (p[1] - line.0[1]) * n[1];
        let rms = (inl.iter().map(|i| dist(all[*i]).powi(2)).sum::<f64>() / inl.len() as f64).sqrt();
        let width = (2.5 * rms).clamp(3.0, 5.0);
        let taken: Vec<usize> = (0..all.len()).filter(|i| alive[*i] && dist(all[*i]).abs() <= width).collect();
        for i in &taken {
            alive[*i] = false;
            for (k, tt) in tables.iter().enumerate() {
                let v = &mut votes[k * bins + bin(all[*i], *tt)];
                *v = v.saturating_sub(1);
            }
        }
        // Cut where the points stop: the stretches where they run on.
        let mut along: Vec<(f64, usize)> = taken.iter().map(|i| ((all[*i][0] - line.0[0]) * line.1[0] + (all[*i][1] - line.0[1]) * line.1[1], *i)).collect();
        along.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut run: Vec<[f64; 2]> = Vec::new();
        let mut last = f64::MIN;
        let close = |run: &mut Vec<[f64; 2]>, out: &mut Vec<Cand>| {
            if run.len() >= 2 {
                let c = Cand::fit(std::mem::take(run));
                // A fragment shorter than a foot is not kept to join.
                if c.length() >= dpf {
                    out.push(c);
                }
            }
            run.clear();
        };
        for (s, i) in along {
            if !run.is_empty() && s - last > GAP_FT * dpf {
                close(&mut run, &mut out);
            }
            run.push(all[i]);
            last = s;
        }
        close(&mut run, &mut out);
    }
    out
}

/// How full of points a strip of the sheet is: the part of its area that is points (a point is a
/// pixel of what the thinning left).
fn fill(all: &[[f64; 2]], a: &Cand, across: (f64, f64), along: (f64, f64)) -> f64 {
    let area = (across.1 - across.0).abs() * (along.1 - along.0).max(1.0);
    if area <= 0.0 {
        return 0.0;
    }
    let (ac0, ac1) = (across.0.min(across.1), across.0.max(across.1));
    let n = all.iter().filter(|p| { let (u, s) = (a.across(**p), a.along(**p)); u >= ac0 && u <= ac1 && s >= along.0 && s <= along.1 }).count();
    n as f64 / area
}

/// How far b lies from a (to a's left), how far along a they overlap, and whether next to no points lie
/// between them (the two are the faces of one wall, not one face seen wide). None when they are not
/// parallel and near enough to be asked.
fn between(a: &Cand, b: &Cand, all: &[[f64; 2]], dpf: f64, angle: f64, least: f64) -> Option<(f64, f64, bool)> {
    let cos = (a.d[0] * b.d[0] + a.d[1] * b.d[1]).abs().min(1.0);
    if cos.acos().to_degrees() > angle {
        return None;
    }
    let g = a.across(b.c);
    let mid = |c: &Cand| c.along(a.c);
    // b's run along a's direction.
    let (b_lo, b_hi) = (mid(b) + if a.d[0] * b.d[0] + a.d[1] * b.d[1] >= 0.0 { b.lo } else { -b.hi }, mid(b) + if a.d[0] * b.d[0] + a.d[1] * b.d[1] >= 0.0 { b.hi } else { -b.lo });
    let (o_lo, o_hi) = (a.lo.max(b_lo), a.hi.min(b_hi));
    let overlap = o_hi - o_lo;
    if overlap < least || g.abs() > PAIR_MOST_IN / 12.0 * dpf {
        return None;
    }
    // The strip along the middle of the gap, a dot and a half either side at most (nearer the two faces
    // the strip would hold their own scatter), against the fill of the two faces' own bands.
    let half = (g.abs() / 4.0).min(1.5);
    let mid_fill = fill(all, a, (0.5 * g - half, 0.5 * g + half), (o_lo, o_hi));
    let own = fill(all, a, (-1.5, 1.5), (o_lo, o_hi)).max(fill(all, a, (g - 1.5, g + 1.5), (o_lo, o_hi)));
    Some((g, overlap, mid_fill < BETWEEN_PART * own))
}

/// Parallel stretches lying in one band of points are one face seen wide: joined, and fitted again.
fn join(cands: &mut Vec<Cand>, all: &[[f64; 2]], dpf: f64) {
    'again: loop {
        for i in 0..cands.len() {
            for j in i + 1..cands.len() {
                let Some((g, _, empty)) = between(&cands[i], &cands[j], all, dpf, JOIN_ANGLE_DEG, 0.5 * dpf) else { continue };
                // Under the least a pair is apart, they are one face; above it, one where points lie between.
                if g.abs() < PAIR_LEAST_IN / 12.0 * dpf || !empty {
                    let b = cands.remove(j);
                    let mut pts = std::mem::take(&mut cands[i].pts);
                    pts.extend(b.pts);
                    cands[i] = Cand::fit(pts);
                    continue 'again;
                }
            }
        }
        break;
    }
}

/// A line that crosses between the two faces of a wall, or lies across one face, is not a face: where
/// every point of a stretch is within 14 in of a longer one and it is not parallel to it, it is dropped.
/// (The points a face's fit left at its edges, two faces 4 1/2 in apart, make a line a few feet long across
/// the pair.)
fn slivers(cands: &mut Vec<Cand>, dpf: f64) {
    let near = PAIR_MOST_IN / 12.0 * dpf;
    let longer: Vec<Cand> = cands.clone();
    cands.retain(|c| {
        !longer.iter().any(|o| {
            o.length() > c.length() && (o.d[0] * c.d[0] + o.d[1] * c.d[1]).abs().min(1.0).acos().to_degrees() > JOIN_ANGLE_DEG && c.pts.iter().all(|p| o.across(*p).abs() <= near && o.along(*p) >= o.lo - near && o.along(*p) <= o.hi + near)
        })
    });
}

/// The side of a face its room is on: that with more of the sheet's points within 8 ft of it, from 14 in
/// away (so the other face of its own wall does not count), along its length and 8 ft past either end.
fn room(f: &Cand, all: &[[f64; 2]], dpf: f64) -> Option<bool> {
    let (near, far) = (PAIR_MOST_IN / 12.0 * dpf, ROOM_REACH_FT * dpf);
    let (mut left, mut right) = (0usize, 0usize);
    for p in all {
        let s = f.along(*p);
        // Along it and 8 ft past either end: the strip an 8 ft reach covers (a partition's rooms are
        // the walls it runs between, which begin where it ends).
        if s < f.lo - far || s > f.hi + far {
            continue;
        }
        let u = f.across(*p);
        if u >= near && u <= far {
            left += 1;
        } else if -u >= near && -u <= far {
            right += 1;
        }
    }
    let most = left.max(right);
    if most == 0 || (left as f64 - right as f64).abs() < ROOM_TIE * most as f64 {
        None
    } else {
        Some(left > right)
    }
}

/// The faces that are two of one wall: (a, b, how far apart in dots), each way round.
fn pairs(list: &[Cand], all: &[[f64; 2]], dpf: f64) -> Vec<(usize, usize, f64)> {
    // For each face its nearest candidate.
    let mut best: Vec<Option<(usize, f64)>> = vec![None; list.len()];
    for i in 0..list.len() {
        for j in 0..list.len() {
            if i == j {
                continue;
            }
            let Some((g, _, empty)) = between(&list[i], &list[j], all, dpf, PAIR_ANGLE_DEG, PAIR_OVERLAP_FT * dpf) else { continue };
            if empty && g.abs() >= PAIR_LEAST_IN / 12.0 * dpf && best[i].is_none_or(|(_, d)| g.abs() < d) {
                best[i] = Some((j, g.abs()));
            }
        }
    }
    (0..list.len()).filter_map(|i| best[i].and_then(|(j, g)| (best[j].map(|b| b.0) == Some(i)).then_some((i, j, g)))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::found_scenes::Scene;
    use crate::scan::Scan;

    const DOTS: f64 = 24.0;
    const QUARTER_INCH: f64 = 1.0 / 48.0;
    const LEAN: f64 = 0.5; // degrees: what `Scene::scan` leans every scan by

    struct Sheet {
        gray: Vec<u8>,
        w: u32,
        h: u32,
        /// The scan's own feet at the sheet's lower-left corner (what `Cut::corner` says).
        corner: [f64; 2],
    }

    /// The scan cut as the program cuts it (`bring_in`): 4 ft above its floor, 2 in either way, 24 dots to the foot, 2 ft of margin.
    fn sheet_of(scan: &Scan) -> Sheet {
        sheet_at(scan, scan.floor_and_ceiling().0.unwrap_or(0.0) + 4.0)
    }

    fn sheet_at(scan: &Scan, height: f64) -> Sheet {
        let cut = scan.cut(height, 2.0 / 12.0, DOTS, 2.0).unwrap();
        let g = image::load_from_memory(&cut.png).unwrap().to_luma8();
        Sheet { w: g.width(), h: g.height(), gray: g.into_raw(), corner: cut.corner }
    }

    fn found(s: &Sheet) -> Vec<Face> {
        faces(&s.gray, s.w, s.h, DOTS)
    }

    /// A face in the scan's feet.
    fn feet(f: &Face, s: &Sheet) -> [[f64; 2]; 2] {
        let one = |p: [f64; 2]| [p[0] / DOTS + s.corner[0], p[1] / DOTS + s.corner[1]];
        [one(f.line[0]), one(f.line[1])]
    }

    /// How far a point is from the infinite line a to b.
    fn off(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / dx.hypot(dy)
    }

    /// Where the made-up house's line (x, y in the house's own feet, at the cut's height) is in the scan after `Scene::scan`
    /// turned it about its southwest corner `turn` degrees and leaned it half a degree (the lean moves everything at a height
    /// 4 ft up by 4 ft times the lean's sine, east).
    fn truth(p: [f64; 2], turn: f64) -> [f64; 2] {
        let (t, l) = (turn.to_radians(), LEAN.to_radians());
        let (x, y) = (p[0] * t.cos() - p[1] * t.sin(), p[0] * t.sin() + p[1] * t.cos());
        [x * l.cos() - 4.0 * l.sin(), y]
    }

    /// Which faces lie on a line of the truth: (their indices).
    fn on(list: &[Face], s: &Sheet, a: [f64; 2], b: [f64; 2], within: f64) -> Vec<usize> {
        (0..list.len()).filter(|i| { let l = feet(&list[*i], s); off(l[0], a, b) <= within && off(l[1], a, b) <= within }).collect()
    }

    /// A 30 by 16 ft room, 8 ft high, one face of each outside wall, and a partition 4 1/2 in thick, both faces, 18 ft from the west wall.
    fn house(turn: f64) -> Scan {
        let mut s = Scene::new(100.0, 7);
        s.room(30.0, 16.0, 8.0);
        s.thick([18.0, 0.0], [18.0, 16.0], 0.375, None, &[], &[]);
        s.scan(turn)
    }

    #[test]
    fn the_house_gives_six_faces_each_within_a_quarter_inch_and_the_partitions_two_are_a_pair_at_any_turn() {
        for turn in [0.0, 17.0, 33.0, 45.0] {
            let sheet = sheet_of(&house(turn));
            let list = found(&sheet);
            assert_eq!(list.len(), 6, "turn {turn}: {list:?}");
            let t = |p: [f64; 2]| truth(p, turn);
            let lines: [([f64; 2], [f64; 2]); 4] = [([0.0, 0.0], [30.0, 0.0]), ([30.0, 0.0], [30.0, 16.0]), ([30.0, 16.0], [0.0, 16.0]), ([0.0, 16.0], [0.0, 0.0])];
            let mut inside: Vec<Option<bool>> = Vec::new();
            for (a, b) in lines {
                let hit = on(&list, &sheet, t(a), t(b), QUARTER_INCH);
                assert_eq!(hit.len(), 1, "turn {turn}: the face on {a:?} to {b:?} is read within 1/4 in once: {hit:?} of {:?}", list.iter().map(|f| feet(f, &sheet)).collect::<Vec<_>>());
                let f = &list[hit[0]];
                // Its room is the house's middle: on the left of its line when the line was drawn counterclockwise (the way `lines` are).
                let l = feet(f, &sheet);
                let mid = t([15.0, 8.0]);
                let left_of_face = (l[1][0] - l[0][0]) * (mid[1] - l[0][1]) - (l[1][1] - l[0][1]) * (mid[0] - l[0][0]) > 0.0;
                assert_eq!(f.room, Some(left_of_face), "turn {turn}: the room of the face on {a:?} to {b:?} is the inside");
                assert_eq!(f.pair, None, "an outside face has no other face");
                inside.push(f.room);
            }
            // The partition's two faces, 4 1/2 in either side of 18 ft.
            for x in [18.0 - 0.1875, 18.0 + 0.1875] {
                let hit = on(&list, &sheet, t([x, 0.0]), t([x, 16.0]), QUARTER_INCH);
                assert_eq!(hit.len(), 1, "turn {turn}: the partition face at {x}: {hit:?}");
                let (me, other) = (hit[0], on(&list, &sheet, t([36.0 - x, 0.0]), t([36.0 - x, 16.0]), QUARTER_INCH)[0]);
                let (j, apart) = list[me].pair.unwrap_or_else(|| panic!("turn {turn}: the face at {x} has no pair: {:?}", list[me]));
                assert_eq!(j, other, "turn {turn}");
                // 4 1/2 in in page units (24 to the foot is 9 dots), within a quarter inch (half a dot).
                assert!((apart - 9.0).abs() <= 0.5, "turn {turn}: {apart} dots apart");
                // Both of its sides hold the same of the sheet's points: it cannot say which is the room.
                assert_eq!(list[me].room, None, "turn {turn}");
            }
        }
    }

    /// Points on four sides of a rectangle (and whatever else the test adds) at one height, every 0.1 ft, no scatter.
    fn rectangle(w: f64, d: f64) -> Vec<[f64; 3]> {
        let mut pts = Vec::new();
        for i in 0..=(w * 10.0) as usize {
            pts.push([i as f64 / 10.0, 0.0, 4.0]);
            pts.push([i as f64 / 10.0, d, 4.0]);
        }
        for i in 0..=(d * 10.0) as usize {
            pts.push([0.0, i as f64 / 10.0, 4.0]);
            pts.push([w, i as f64 / 10.0, 4.0]);
        }
        pts
    }

    #[test]
    fn a_face_at_a_whole_foot_is_read_there_the_half_dot_taken_out() {
        // `cut` draws the dot of a point from -1 to +2 dots about it (four across): the dot's middle is one whole dot (1/2 in) up and
        // east of the point, so a line read from the picture alone lies 1/2 in off. It is read from the dots' middles and put back
        // one dot, in each way.
        let scan = Scan { points: rectangle(30.0, 16.0), lost: Vec::new() };
        let sheet = sheet_at(&scan, 4.0);
        let list = found(&sheet);
        assert_eq!(list.len(), 4, "{list:?}");
        for (a, b) in [([0.0, 0.0], [30.0, 0.0]), ([30.0, 0.0], [30.0, 16.0]), ([30.0, 16.0], [0.0, 16.0]), ([0.0, 16.0], [0.0, 0.0])] {
            let hit = on(&list, &sheet, a, b, QUARTER_INCH / 4.0);
            assert_eq!(hit.len(), 1, "{a:?} to {b:?}: {:?}", list.iter().map(|f| feet(f, &sheet)).collect::<Vec<_>>());
            // Its ends are the corners, to within a dot.
            let l = feet(&list[hit[0]], &sheet);
            let along = |p: [f64; 2]| ((p[0] - a[0]) * (b[0] - a[0]) + (p[1] - a[1]) * (b[1] - a[1])) / (b[0] - a[0]).hypot(b[1] - a[1]);
            let len = (b[0] - a[0]).hypot(b[1] - a[1]);
            let (lo, hi) = (along(l[0]).min(along(l[1])), along(l[0]).max(along(l[1])));
            assert!(lo > -0.2 && lo < 0.2 && hi > len - 0.2 && hi < len + 0.2, "{a:?} to {b:?}: runs {lo} to {hi}");
        }
        // The same picture as RGBA (what the page reader hands over) is read the same.
        let rgba: Vec<u8> = sheet.gray.iter().flat_map(|g| [*g, *g, *g, 255]).collect();
        assert_eq!(faces_rgba(&rgba, sheet.w, sheet.h, DOTS), list);
    }

    #[test]
    fn a_run_of_points_a_foot_and_a_half_long_is_not_a_face_and_a_blank_sheet_has_none() {
        let mut pts = rectangle(30.0, 16.0);
        // A chest 1 ft 6 in long, 6 ft in from the west wall and 8 ft from the south one.
        for i in 0..=15 {
            pts.push([6.0 + i as f64 / 10.0, 8.0, 4.0]);
        }
        let sheet = sheet_at(&Scan { points: pts, lost: Vec::new() }, 4.0);
        let list = found(&sheet);
        assert_eq!(list.len(), 4, "{:?}", list.iter().map(|f| feet(f, &sheet)).collect::<Vec<_>>());
        assert!(list.iter().all(|f| (feet(f, &sheet)[0][1] - 8.0).abs() > 1.0 || (feet(f, &sheet)[1][1] - 8.0).abs() > 1.0));
        // It alone: a sheet that is only the chest, and a sheet of nothing at all.
        let only: Vec<[f64; 3]> = (0..=15).map(|i| [i as f64 / 10.0, 0.0, 4.0]).chain((0..=15).map(|i| [10.0, 5.0 + i as f64 / 10.0, 4.0])).collect();
        let s = sheet_at(&Scan { points: only, lost: Vec::new() }, 4.0);
        assert!(found(&s).is_empty(), "{:?}", found(&s));
        assert!(faces(&vec![255u8; 400 * 300], 400, 300, DOTS).is_empty());
        assert!(faces(&[], 0, 0, DOTS).is_empty());
    }

    /// A 12 by 14 ft room as a phone scans it (`tools/made-up-scan.py --scene phone`): every face rough 2 in across and leaning a degree,
    /// the south one bowed 1 1/2 in, the north and east seen at 8 times the density of the others, the west one passed twice 10 in apart.
    fn phone() -> Scan {
        let (dens, lean, extra, bow) = (300.0, 0.0175, 0.0656, 0.125);
        let mut s = Scene::new(dens, 19);
        s.floor([0.0, 0.0, 12.0, 14.0]);
        s.ceiling([0.0, 0.0, 12.0, 14.0], 8.0, 8.0);
        s.face_rough([0.0, 0.0], [12.0, 0.0], None, bow, lean, extra, 0.0, dens);
        s.face_rough([12.0, 0.0], [12.0, 14.0], None, 0.0, lean, extra, 0.0, dens * 8.0);
        s.face_rough([12.0, 14.0], [0.0, 14.0], None, 0.0, lean, extra, 0.0, dens * 8.0);
        s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, lean, extra, 0.0, dens / 2.0);
        s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, lean, extra, 10.0 / 12.0, dens / 2.0);
        s.scan(0.0)
    }

    #[test]
    fn a_phones_rough_bowed_scan_gives_one_face_a_wall_and_a_pair_where_a_wall_was_passed_twice() {
        let sheet = sheet_of(&phone());
        let list = found(&sheet);
        let say = || list.iter().map(|f| feet(f, &sheet)).collect::<Vec<_>>();
        // The walls: south y 0, east x 12, north y 14 (a face's lean moves it east by the height times the lean: the cut is 4 ft up).
        let at = |a: [f64; 2], b: [f64; 2]| on(&list, &sheet, truth(a, 0.0), truth(b, 0.0), 0.17);
        for (name, a, b) in [("south", [0.0, 0.0], [12.0, 0.0]), ("east", [12.0, 0.0], [12.0, 14.0]), ("north", [12.0, 14.0], [0.0, 14.0])] {
            assert_eq!(at(a, b).len(), 1, "{name}: one face, not three: {:?}", say());
        }
        // The west wall: its two passes, each a face, and they pair; nothing else.
        let west = at([0.0, 14.0], [0.0, 0.0]);
        let shifted = on(&list, &sheet, truth([10.0 / 12.0, 0.0], 0.0), truth([10.0 / 12.0, 14.0], 0.0), 0.17);
        assert!(west.len() + shifted.len() <= 2 && !west.is_empty(), "{:?}", say());
        assert!(list.len() <= 5, "{:?}", say());
    }
}
