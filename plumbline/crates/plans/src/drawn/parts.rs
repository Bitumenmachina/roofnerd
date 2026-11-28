use super::*;

pub(super) fn sub(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
pub(super) fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
pub(super) fn cross(a: P2, b: P2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
pub(super) fn norm(a: P2) -> f64 {
    dot(a, a).sqrt()
}
pub(super) fn unit(a: P2) -> P2 {
    let n = norm(a).max(1e-300);
    [a[0] / n, a[1] / n]
}
pub(super) fn perp(u: P2) -> P2 {
    [-u[1], u[0]]
}
pub(super) fn at(u: P2, n: P2, t: f64, c: f64) -> P2 {
    [u[0] * t + n[0] * c, u[1] * t + n[1] * c]
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Seg {
    pub(super) a: P2,
    pub(super) b: P2,
    pub(super) len: f64,
}

pub(super) type Iv = Vec<(f64, f64)>;

pub(super) fn merged(mut v: Iv, gap: f64) -> Iv {
    v.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut out: Iv = Vec::new();
    for (a, b) in v {
        match out.last_mut() {
            Some(l) if a <= l.1 + gap => l.1 = l.1.max(b),
            _ => out.push((a, b)),
        }
    }
    out
}

pub(super) fn intersect(a: &Iv, b: &Iv) -> Iv {
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < a.len() && j < b.len() {
        let (lo, hi) = (a[i].0.max(b[j].0), a[i].1.min(b[j].1));
        if hi > lo {
            out.push((lo, hi));
        }
        if a[i].1 < b[j].1 {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

pub(super) fn covered(iv: &Iv, lo: f64, hi: f64) -> f64 {
    iv.iter().map(|(a, b)| (b.min(hi) - a.max(lo)).max(0.0)).sum()
}

// ---- faces: the lines that lie on one line, with where along it they are drawn ----

#[derive(Clone, Debug)]
pub(super) struct Face {
    pub(super) u: P2,
    pub(super) n: P2,
    pub(super) c: f64,
    pub(super) iv: Iv,
    /// The same, bridged across joints; a bridged stretch is kept only if some piece of it is as long as
    /// a letter is tall on the paper (a row of lettering is no line, though its strokes line up).
    pub(super) ivb: Iv,
    pub(super) longest: f64,
    /// Each line's stretch as drawn, not merged: how many times over the face is drawn.
    pub(super) raw: Iv,
}

impl Face {
    pub(super) fn ends(&self, t0: f64, t1: f64) -> (P2, P2) {
        (at(self.u, self.n, t0, self.c), at(self.u, self.n, t1, self.c))
    }
    /// This face's stretches, as they lie along `other`'s direction.
    pub(super) fn along(&self, other: &Face, iv: &Iv) -> Iv {
        let mut out: Iv = iv
            .iter()
            .map(|&(a, b)| {
                let (p, q) = self.ends(a, b);
                let (x, y) = (dot(other.u, p), dot(other.u, q));
                (x.min(y), x.max(y))
            })
            .collect();
        out.sort_by(|x, y| x.0.total_cmp(&y.0));
        out
    }
    /// How far this face lies from `other`'s line, along `other`'s normal, where `other` is at `t`.
    pub(super) fn offset_from(&self, other: &Face, t: f64) -> f64 {
        let p = at(other.u, other.n, t, other.c);
        (self.c - dot(self.n, p)) * if dot(self.n, other.n) >= 0.0 { 1.0 } else { -1.0 }
    }
}

pub(super) fn angle_of(u: P2) -> f64 {
    let mut t = u[1].atan2(u[0]);
    if t < 0.0 {
        t += std::f64::consts::PI;
    }
    if t >= std::f64::consts::PI {
        t -= std::f64::consts::PI;
    }
    t
}

/// The faces the segments `ids` make: lines on one line (both ends within 1/8 in of it, directions
/// within 0.11 degree) are one face. Each line is looked for among those near it by direction and
/// by where it lies, so a hatched sheet is no slower than a bare one. Returns the faces and, for
/// each segment, the face it is in.
pub(super) fn faces_of(segs: &[Seg], ids: &[usize], letter: f64) -> (Vec<Face>, HashMap<usize, usize>) {
    let nb = (std::f64::consts::PI / ANGLE_FACE).ceil() as usize + 1;
    let frame = |b: usize| {
        let th = (b as f64 + 0.5) * ANGLE_FACE;
        let u = [th.cos(), th.sin()];
        (u, perp(u))
    };
    let mut buckets: BTreeMap<usize, Vec<(f64, usize)>> = BTreeMap::new();
    let mut dirs: HashMap<usize, (P2, usize)> = HashMap::new();
    for &i in ids {
        let s = &segs[i];
        let mut u = unit(sub(s.b, s.a));
        if u[1] < 0.0 || (u[1] == 0.0 && u[0] < 0.0) {
            u = [-u[0], -u[1]];
        }
        let b = ((angle_of(u) / ANGLE_FACE) as usize).min(nb - 1);
        let (_, n) = frame(b);
        let mid = [(s.a[0] + s.b[0]) / 2.0, (s.a[1] + s.b[1]) / 2.0];
        buckets.entry(b).or_default().push((dot(n, mid), i));
        dirs.insert(i, (u, b));
    }
    for v in buckets.values_mut() {
        v.sort_by(|x, y| x.0.total_cmp(&y.0));
    }
    let mut order: Vec<usize> = ids.to_vec();
    order.sort_by(|&x, &y| segs[y].len.total_cmp(&segs[x].len).then(x.cmp(&y)));
    let mut taken: HashMap<usize, usize> = HashMap::new();
    let mut faces: Vec<Face> = Vec::new();
    for &s in &order {
        if taken.contains_key(&s) {
            continue;
        }
        let (u, b) = dirs[&s];
        let n = perp(u);
        let c = dot(n, segs[s].a);
        let mid = [(segs[s].a[0] + segs[s].b[0]) / 2.0, (segs[s].a[1] + segs[s].b[1]) / 2.0];
        let id = faces.len();
        let mut spans: Iv = Vec::new();
        for nbk in [(b + nb - 1) % nb, b, (b + 1) % nb] {
            let Some(v) = buckets.get(&nbk) else { continue };
            let (_, nn) = frame(nbk);
            let target = dot(nn, mid);
            let lo = v.partition_point(|e| e.0 < target - 1.0);
            for &(_, i) in &v[lo..] {
                let (cc, _) = (0, 0);
                let _ = cc;
                // Entries are sorted by where they lie; stop past the window.
                let m = [(segs[i].a[0] + segs[i].b[0]) / 2.0, (segs[i].a[1] + segs[i].b[1]) / 2.0];
                if dot(nn, m) > target + 1.0 {
                    break;
                }
                if taken.contains_key(&i) {
                    continue;
                }
                let (ea, eb) = (dot(n, segs[i].a) - c, dot(n, segs[i].b) - c);
                if ea.abs() <= LINE_TOL && eb.abs() <= LINE_TOL {
                    taken.insert(i, id);
                    let (x, y) = (dot(u, segs[i].a), dot(u, segs[i].b));
                    spans.push((x.min(y), x.max(y)));
                }
            }
        }
        let iv = merged(spans.clone(), MERGE_GAP);
        let mut ivb: Iv = merged(iv.clone(), JOINT_MOST).into_iter().filter(|&(a, b)| iv.iter().any(|&(x, y)| x >= a - 1e-9 && y <= b + 1e-9 && y - x >= letter)).collect();
        // A pier (call 186): a piece of the line too short to count alone, on the same line as a stretch that does and no
        // further from it than the widest opening, is part of the wall that stretch is the face of, however short: it is
        // what lies between two openings, or between an opening and the wall's end.
        loop {
            let more: Vec<(f64, f64)> = iv.iter().copied().filter(|&(a, b)| !ivb.iter().any(|&(x, y)| x <= a + 1e-9 && y >= b - 1e-9) && ivb.iter().any(|&(x, y)| (a >= y && a - y <= OPENING_MOST) || (x >= b && x - b <= OPENING_MOST))).collect();
            if more.is_empty() {
                break;
            }
            ivb.extend(more);
            ivb.sort_by(|x, y| x.0.total_cmp(&y.0));
        }
        let longest = ivb.iter().map(|(a, b)| b - a).fold(0.0, f64::max);
        faces.push(Face { u, n, c, iv, ivb, longest, raw: spans });
    }
    (faces, taken)
}

// ---- dimension lines ----

/// The lines that are a dimension: a line with a tick or an arrowhead at each end, and the ticks,
/// arrowheads and extension lines of it. A tick crosses its line or two arrowhead lines lie either
/// side of it; one short oblique line on one side only is a wall's mitred end and is not a tick.
pub(super) fn dimension_lines(segs: &[Seg], k: f64) -> Vec<bool> {
    dimension_scan(segs, k).0
}

/// A dimension line found: the line, and the two places it measures between (where its ticks cross it or its arrowheads point),
/// which are not always where the line itself ends.
#[derive(Clone, Copy, Debug)]
pub(super) struct DimLine {
    pub(super) between: [P2; 2],
}

pub(super) fn dimension_scan(segs: &[Seg], k: f64) -> (Vec<bool>, Vec<DimLine>) {
    let mut dim = vec![false; segs.len()];
    let tick_max = TICK_MAX_PT * k;
    let near = TERM_NEAR_PT * k;
    let long_least = MIN_FACE.max(LETTER_PT * k);
    let cell = (tick_max * 2.0).max(1e-6);
    let key = |p: P2| ((p[0] / cell).floor() as i64, (p[1] / cell).floor() as i64);
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, s) in segs.iter().enumerate() {
        if s.len <= tick_max {
            grid.entry(key(s.a)).or_default().push(i);
            let kb = key(s.b);
            if kb != key(s.a) {
                grid.entry(kb).or_default().push(i);
            }
        }
    }
    let dist_to = |p: P2, s: &Seg| -> f64 {
        let d = sub(s.b, s.a);
        let t = (dot(sub(p, s.a), d) / dot(d, d).max(1e-300)).clamp(0.0, 1.0);
        norm(sub(p, [s.a[0] + d[0] * t, s.a[1] + d[1] * t]))
    };
    let nearby = |p: P2| -> Vec<usize> {
        let (cx, cy) = key(p);
        let mut v = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(l) = grid.get(&(cx + dx, cy + dy)) {
                    v.extend(l.iter().copied());
                }
            }
        }
        v.sort_unstable();
        v.dedup();
        v
    };
    let mut ends_of: Vec<(usize, P2, Vec<usize>, [P2; 2])> = Vec::new();
    for (i, s) in segs.iter().enumerate() {
        if s.len < long_least {
            continue;
        }
        let u = unit(sub(s.b, s.a));
        let n = perp(u);
        let mut all_ticks: Vec<usize> = Vec::new();
        let mut both = true;
        let mut between: Vec<P2> = Vec::new();
        for e in [s.a, s.b] {
            let (mut crossing, mut plus, mut minus) = (false, false, false);
            let mut here: Vec<usize> = Vec::new();
            let (mut mids, mut apexes): (Vec<P2>, Vec<P2>) = (Vec::new(), Vec::new());
            for j in nearby(e) {
                let t = &segs[j];
                if j == i || dist_to(e, t) > near {
                    continue;
                }
                let tu = unit(sub(t.b, t.a));
                let sin = cross(u, tu).abs();
                if !(0.34..=0.94).contains(&sin) {
                    continue;
                }
                let (s0, s1) = (dot(n, sub(t.a, e)), dot(n, sub(t.b, e)));
                here.push(j);
                let apex = if norm(sub(t.a, e)) <= norm(sub(t.b, e)) { t.a } else { t.b };
                apexes.push(apex);
                if s0 * s1 < 0.0 && s0.abs().min(s1.abs()) >= 0.15 * t.len {
                    crossing = true;
                    mids.push([(t.a[0] + t.b[0]) / 2.0, (t.a[1] + t.b[1]) / 2.0]);
                } else if s0 + s1 > 0.0 {
                    plus = true;
                } else {
                    minus = true;
                }
            }
            if crossing || (plus && minus) {
                all_ticks.extend(here);
                let pts = if crossing { &mids } else { &apexes };
                let c = pts.iter().fold([0.0, 0.0], |m, p| [m[0] + p[0], m[1] + p[1]]);
                let c = [c[0] / pts.len() as f64, c[1] / pts.len() as f64];
                // Where it is along the line, on the line.
                let t = dot(u, sub(c, s.a));
                between.push([s.a[0] + u[0] * t, s.a[1] + u[1] * t]);
            } else {
                both = false;
                break;
            }
        }
        if both {
            ends_of.push((i, s.a, all_ticks, [between[0], between[1]]));
        }
    }
    let found: Vec<DimLine> = ends_of.iter().map(|e| DimLine { between: e.3 }).collect();
    for (i, _, ticks, _) in &ends_of {
        dim[*i] = true;
        for t in ticks {
            dim[*t] = true;
        }
    }
    // Extension lines: square to a dimension line and through its end.
    let dims: Vec<usize> = ends_of.iter().map(|e| e.0).collect();
    if !dims.is_empty() {
        let cell2 = (long_least * 4.0).max(1e-6);
        let _ = cell2;
        for (j, x) in segs.iter().enumerate() {
            if dim[j] || x.len <= tick_max {
                continue;
            }
            let xu = unit(sub(x.b, x.a));
            for &i in &dims {
                let s = &segs[i];
                let u = unit(sub(s.b, s.a));
                if dot(u, xu).abs() > CROSS_ANGLE {
                    continue;
                }
                if [s.a, s.b].iter().any(|e| dist_to(*e, x) <= near) {
                    dim[j] = true;
                    break;
                }
            }
        }
    }
    (dim, found)
}

// ---- boxes: the sheet's border, title block, key, schedules ----

#[derive(Clone, Debug)]
pub(super) struct Rect {
    pub(super) u: P2,
    pub(super) n: P2,
    /// Low and high along `u`, low and high along `n`.
    pub(super) t: (f64, f64),
    pub(super) s: (f64, f64),
    pub(super) sides: Vec<usize>,
}

pub(super) fn rects_of(faces: &[Face], k: f64) -> Vec<Rect> {
    // Pieces long enough to be the side of a box.
    struct Piece {
        face: usize,
        a: P2,
        b: P2,
    }
    let least = LETTER_PT * k;
    let touch = TOUCH_PT * k;
    let mut pieces: Vec<Piece> = Vec::new();
    for (fi, f) in faces.iter().enumerate() {
        for &(t0, t1) in &f.iv {
            if t1 - t0 >= least {
                let (a, b) = f.ends(t0, t1);
                pieces.push(Piece { face: fi, a, b });
            }
        }
    }
    let cell = (touch * 4.0).max(1e-6);
    let key = |p: P2| ((p[0] / cell).floor() as i64, (p[1] / cell).floor() as i64);
    let mut grid: HashMap<(i64, i64), Vec<(usize, bool)>> = HashMap::new();
    for (i, p) in pieces.iter().enumerate() {
        grid.entry(key(p.a)).or_default().push((i, false));
        grid.entry(key(p.b)).or_default().push((i, true));
    }
    let touching = |q: P2| -> Vec<(usize, bool)> {
        let (cx, cy) = key(q);
        let mut v = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(l) = grid.get(&(cx + dx, cy + dy)) {
                    for &(i, end) in l {
                        let e = if end { pieces[i].b } else { pieces[i].a };
                        if norm(sub(e, q)) <= touch {
                            v.push((i, end));
                        }
                    }
                }
            }
        }
        v
    };
    let mut seen: std::collections::BTreeSet<[usize; 4]> = Default::default();
    let mut out = Vec::new();
    for (ai, a) in pieces.iter().enumerate() {
        let ua = unit(sub(a.b, a.a));
        let la = norm(sub(a.b, a.a));
        let at_a = touching(a.a);
        let at_b = touching(a.b);
        for &(bi, bend) in at_a.iter().take(6) {
            if bi == ai {
                continue;
            }
            let b = &pieces[bi];
            let fb = if bend { b.a } else { b.b };
            let ub = unit(sub(fb, a.a));
            if dot(ua, ub).abs() > 0.0175 {
                continue;
            }
            for &(ci, cend) in at_b.iter().take(6) {
                if ci == ai || ci == bi {
                    continue;
                }
                let c = &pieces[ci];
                let fc = if cend { c.a } else { c.b };
                let uc = unit(sub(fc, a.b));
                if dot(ub, uc) < 0.999 || (norm(sub(fb, a.a)) - norm(sub(fc, a.b))).abs() > 2.0 * touch {
                    continue;
                }
                for (di, _) in touching(fb) {
                    if [ai, bi, ci].contains(&di) {
                        continue;
                    }
                    let d = &pieces[di];
                    let far = if norm(sub(d.a, fb)) <= touch { d.b } else { d.a };
                    if norm(sub(far, fc)) > touch {
                        continue;
                    }
                    let mut key4 = [ai, bi, ci, di];
                    key4.sort_unstable();
                    if !seen.insert(key4) {
                        continue;
                    }
                    let lb = norm(sub(fb, a.a));
                    let n = perp(ua);
                    let sgn = if dot(n, ub) >= 0.0 { 1.0 } else { -1.0 };
                    let (t0, t1) = (dot(ua, a.a), dot(ua, a.b));
                    let (s0, s1) = (dot(n, a.a), dot(n, a.a) + sgn * lb);
                    out.push(Rect { u: ua, n, t: (t0.min(t1), t0.max(t1)), s: (s0.min(s1), s0.max(s1)), sides: vec![pieces[ai].face, pieces[bi].face, pieces[ci].face, pieces[di].face] });
                    let _ = la;
                }
            }
        }
    }
    out
}

// ---- two faces side by side ----

/// Two faces side by side: the stretches where both are drawn, long enough, and making up enough
/// of the longer one's, as they lie along `a`'s direction; and how far `b` lies from `a` (signed
/// along `a`'s normal). None if they are not parallel, not alongside, or not parallel enough to
/// have one thickness along the stretch.
pub(super) fn alongside(faces: &[Face], a: usize, b: usize, min_len: f64) -> Option<(Iv, f64)> {
    let (fa, fb) = (&faces[a], &faces[b]);
    if cross(fa.u, fb.u).abs() > ANGLE_PAIR {
        return None;
    }
    let ivb_b = fb.along(fa, &fb.ivb);
    let mut keep: Iv = Vec::new();
    for (lo, hi) in intersect(&fa.ivb, &ivb_b) {
        let piece = |iv: &Iv| iv.iter().find(|(x, y)| *x <= lo + 1e-9 && *y >= hi - 1e-9).map(|(x, y)| y - x).unwrap_or(0.0);
        if hi - lo >= OVERLAP_SHARE * piece(&fa.ivb).max(piece(&ivb_b)) {
            keep.push((lo, hi));
        }
    }
    // The least length is of the wall's whole line, not of a piece between two openings (call 186): pieces no
    // further apart than the widest opening are one wall, and it is as long as its first piece's start to its last
    // piece's end. A piece alone is a wall only if it is long enough alone.
    let mut kept: Iv = Vec::new();
    let mut run: Iv = Vec::new();
    for &(lo, hi) in &keep {
        if run.last().is_some_and(|l| lo - l.1 > OPENING_MOST) {
            if run[run.len() - 1].1 - run[0].0 >= min_len {
                kept.append(&mut run);
            }
            run.clear();
        }
        run.push((lo, hi));
    }
    if !run.is_empty() && run[run.len() - 1].1 - run[0].0 >= min_len {
        kept.append(&mut run);
    }
    let keep = kept;
    if keep.is_empty() {
        return None;
    }
    let mid = (keep[0].0 + keep[0].1) / 2.0;
    let d = fb.offset_from(fa, mid);
    let (e0, e1) = (fb.offset_from(fa, keep[0].0), fb.offset_from(fa, keep[keep.len() - 1].1));
    if (e0 - e1).abs() > INCH {
        return None;
    }
    Some((keep, d))
}

/// For each long face, the faces alongside it within a wall's thickness: which, how far (signed along
/// its own normal) and the stretches, along its own direction.
pub(super) fn neighbours(faces: &[Face], long: &[bool], min_len: f64) -> Vec<Vec<(usize, f64, Iv)>> {
    let nf = faces.len();
    let mut near: Vec<Vec<(usize, f64, Iv)>> = vec![Vec::new(); nf];
    let mut by_angle: Vec<usize> = (0..nf).filter(|&i| long[i]).collect();
    by_angle.sort_by(|&x, &y| angle_of(faces[x].u).total_cmp(&angle_of(faces[y].u)));
    let m = by_angle.len();
    for ix in 0..m {
        let a = by_angle[ix];
        let ta = angle_of(faces[a].u);
        for off in 1..m {
            let b = by_angle[(ix + off) % m];
            let diff = (angle_of(faces[b].u) - ta).rem_euclid(std::f64::consts::PI);
            if diff > ANGLE_PAIR {
                break;
            }
            if let Some((iv, d)) = alongside(faces, a, b, min_len) {
                if d.abs() >= THIN_LEAST - 1e-6 && d.abs() <= THICK_MOST + 1e-6 {
                    let back = faces[a].along(&faces[b], &iv);
                    near[a].push((b, d, iv));
                    near[b].push((a, -d, back));
                }
            }
        }
    }
    // A sheet whose long faces are all level (or all upright) has the loop above come round to the faces behind it, and each pair
    // is found from both its faces: a pair is listed once (call 188).
    for list in near.iter_mut() {
        let mut seen = std::collections::HashSet::new();
        list.retain(|(b, _, _)| seen.insert(*b));
    }
    near
}

/// The spans a page's dimension lines measure, line by line: along each line that is drawn (in pieces or whole, with gaps where a
/// label is), the places a tick or an arrowhead stands (a short oblique stroke touching the line, or one on each side of it), and each
/// run of two ticks next to one another is a span (a chain of dimensions on one line is several). Units are the page's own.
pub(super) fn dimension_spans(segs: &[Seg]) -> Vec<DimLine> {
    let ids: Vec<usize> = (0..segs.len()).collect();
    let (faces, _) = faces_of(segs, &ids, LETTER_PT);
    let ticks: Vec<usize> = (0..segs.len()).filter(|&i| segs[i].len <= TICK_MAX_PT).collect();
    let mut out = Vec::new();
    for f in &faces {
        let (Some(first), Some(last)) = (f.iv.first(), f.iv.last()) else { continue };
        if f.longest < LETTER_PT && last.1 - first.0 < 2.0 * LETTER_PT {
            continue;
        }
        // Each tick's place along the line and which sides of it its strokes lie on.
        let mut marks: Vec<(f64, bool, bool)> = Vec::new();
        for &i in &ticks {
            let s = &segs[i];
            let tu = unit(sub(s.b, s.a));
            let sin = cross(f.u, tu).abs();
            if !(0.34..=0.94).contains(&sin) {
                continue;
            }
            let (da, db) = (dot(f.n, s.a) - f.c, dot(f.n, s.b) - f.c);
            // It touches the line: one end on it, or it crosses it.
            let touches = da.abs().min(db.abs()) <= TERM_NEAR_PT || da * db < 0.0;
            if !touches {
                continue;
            }
            let t = if da * db < 0.0 {
                let w = da.abs() / (da.abs() + db.abs());
                dot(f.u, s.a) + (dot(f.u, s.b) - dot(f.u, s.a)) * w
            } else if da.abs() <= db.abs() {
                dot(f.u, s.a)
            } else {
                dot(f.u, s.b)
            };
            if t < first.0 - TERM_NEAR_PT * 3.0 || t > last.1 + TERM_NEAR_PT * 3.0 {
                continue;
            }
            let (plus, minus) = if da * db < 0.0 { (true, true) } else { (da + db > 0.0, da + db < 0.0) };
            marks.push((t, plus, minus));
        }
        marks.sort_by(|a, b| a.0.total_cmp(&b.0));
        // Marks within 1.5 pt of one another are one tick; it is a tick if it has a stroke on both sides or one across.
        let mut places: Vec<f64> = Vec::new();
        let mut i = 0;
        while i < marks.len() {
            let mut j = i;
            let (mut plus, mut minus, mut sum) = (false, false, 0.0);
            while j < marks.len() && marks[j].0 - marks[i].0 <= TERM_NEAR_PT * 2.0 {
                plus |= marks[j].1;
                minus |= marks[j].2;
                sum += marks[j].0;
                j += 1;
            }
            if plus && minus {
                places.push(sum / (j - i) as f64);
            }
            i = j;
        }
        for w in places.windows(2) {
            if w[1] - w[0] >= LETTER_PT {
                out.push(DimLine { between: [at(f.u, f.n, w[0], f.c), at(f.u, f.n, w[1], f.c)] });
            }
        }
    }
    out
}

// ---- tags ----

/// The closed shapes of short straight pieces a plan marks its windows and doors with (a hexagon, a diamond): at least `TAG_PIECES`
/// pieces, none longer than `TAG_PIECE`, end to end in a loop, the whole no more than `TAG_SPAN` across. Each comes back as the
/// places of the lines in it. A tag is not a wall, and its lines are not lines across a wall or inside one (call 188 item 8).
pub(super) fn tags_of(segs: &[Seg], skip: &[bool]) -> Vec<Vec<usize>> {
    let short: Vec<usize> = (0..segs.len()).filter(|&i| !skip[i] && segs[i].len <= TAG_PIECE).collect();
    let cell = CAP_TOL * 2.0;
    let key = |p: P2| ((p[0] / cell).round() as i64, (p[1] / cell).round() as i64);
    let mut at_key: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (n, &i) in short.iter().enumerate() {
        for e in [segs[i].a, segs[i].b] {
            let (kx, ky) = key(e);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    at_key.entry((kx + dx, ky + dy)).or_default().push(n);
                }
            }
        }
    }
    let mut group: Vec<usize> = (0..short.len()).collect();
    fn root_of(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        r
    }
    for (n, &i) in short.iter().enumerate() {
        for e in [segs[i].a, segs[i].b] {
            for &m in at_key.get(&key(e)).map(|v| v.as_slice()).unwrap_or(&[]) {
                let j = short[m];
                if m != n && [segs[j].a, segs[j].b].iter().any(|f| norm(sub(*f, e)) <= CAP_TOL) {
                    let (ra, rb) = (root_of(&mut group, n), root_of(&mut group, m));
                    group[ra] = rb;
                }
            }
        }
    }
    let mut comps: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for n in 0..short.len() {
        let r = root_of(&mut group, n);
        comps.entry(r).or_default().push(short[n]);
    }
    let mut out = Vec::new();
    for members in comps.into_values() {
        // The distinct pieces (a line drawn over and over is one piece), and each corner's count of piece ends: a loop has every
        // corner twice (an even number, for pieces drawn twice over).
        let mut pieces: Vec<(i64, i64, i64, i64)> = Vec::new();
        let mut ends: HashMap<(i64, i64), usize> = HashMap::new();
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for &i in &members {
            let (a, b) = (segs[i].a, segs[i].b);
            let (ka, kb) = (key(a), key(b));
            let id = if ka <= kb { (ka.0, ka.1, kb.0, kb.1) } else { (kb.0, kb.1, ka.0, ka.1) };
            if !pieces.contains(&id) {
                pieces.push(id);
                *ends.entry(ka).or_default() += 1;
                *ends.entry(kb).or_default() += 1;
            }
            for p in [a, b] {
                for d in 0..2 {
                    lo[d] = lo[d].min(p[d]);
                    hi[d] = hi[d].max(p[d]);
                }
            }
        }
        if pieces.len() >= TAG_PIECES && ends.values().all(|&n| n == 2) && (hi[0] - lo[0]).max(hi[1] - lo[1]) <= TAG_SPAN {
            out.push(members);
        }
    }
    out
}

// ---- dashed lines: what is overhead or hidden ----

/// Dashed lines side by side, a wall's thickness apart and long enough to be a wall's: each such group is one thing
/// overhead or hidden (a beam, a soffit, cabinets above) or, on a renovation plan, a wall to be demolished. Judged by
/// the rules for a wall's two lines, so that what would have been laid as a wall is said as what it may be.
pub(super) fn overhead_of(dsegs: &[Seg], k: f64, origin: P2) -> Vec<Overhead> {
    if dsegs.is_empty() {
        return Vec::new();
    }
    let min_len = MIN_FACE.max(LETTER_PT * k);
    let ids: Vec<usize> = (0..dsegs.len()).collect();
    let (faces, _) = faces_of(dsegs, &ids, LETTER_PT * k);
    let long: Vec<bool> = faces.iter().map(|f| f.longest >= min_len).collect();
    let near = neighbours(&faces, &long, min_len);
    // Faces that lie side by side are one group.
    let mut group: Vec<usize> = (0..faces.len()).collect();
    fn root(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        let mut c = i;
        while g[c] != r {
            let n = g[c];
            g[c] = r;
            c = n;
        }
        r
    }
    for (a, list) in near.iter().enumerate() {
        for (b, _, _) in list {
            let (ra, rb) = (root(&mut group, a), root(&mut group, *b));
            group[ra] = rb;
        }
    }
    let mut by: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (f, l) in near.iter().enumerate() {
        if !l.is_empty() {
            let r = root(&mut group, f);
            by.entry(r).or_default().push(f);
        }
    }
    let mut out: Vec<Overhead> = Vec::new();
    for members in by.values() {
        let mut apart: f64 = 0.0;
        let mut centre = [0.0, 0.0];
        let mut n = 0.0;
        for &a in members {
            for (_, d, iv) in &near[a] {
                apart = apart.max(d.abs());
                let (t0, t1) = (iv[0].0, iv[iv.len() - 1].1);
                let c = at(faces[a].u, faces[a].n, (t0 + t1) / 2.0, faces[a].c + d / 2.0);
                centre = [centre[0] + c[0], centre[1] + c[1]];
                n += 1.0;
            }
        }
        let length = members.iter().map(|&f| faces[f].longest).fold(0.0, f64::max);
        out.push(Overhead { lines: members.len(), apart, length, at: shift([centre[0] / n, centre[1] / n], origin) });
    }
    out.sort_by(|a, b| a.at[1].total_cmp(&b.at[1]).then(a.at[0].total_cmp(&b.at[0])));
    out
}

// ---- the finding ----

#[derive(Clone, Debug)]
pub(super) struct Cand {
    pub(super) a: usize,
    pub(super) cc: f64,
    pub(super) thick: f64,
    pub(super) t0: f64,
    pub(super) t1: f64,
    /// From, to, whether it is a window, and whether its faces are cut clean through (not run on past it).
    pub(super) openings: Vec<(f64, f64, bool, bool)>,
    pub(super) doubt: Option<String>,
    /// Its doubt is that a line of it is already the face of another wall: it is no wall in the way of the others.
    pub(super) loser: bool,
    pub(super) lines: usize,
    /// How many pairs of lines were joined into it (call 188 item 7).
    pub(super) joined: usize,
}
