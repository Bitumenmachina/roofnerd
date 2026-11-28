use super::*;

pub(super) fn dir(theta: f64) -> P2 {
    [theta.cos(), theta.sin()]
}
pub(super) fn normal(theta: f64) -> P2 {
    [-theta.sin(), theta.cos()]
}
pub(crate) fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// A face: a stretch of a line, with the points on it as (along, offset from the line, up).
pub(super) struct Face {
    pub(super) theta: f64,
    /// Offset of its line from the origin along its normal, measured at the floor.
    pub(super) rho: f64,
    pub(super) s0: f64,
    pub(super) s1: f64,
    /// (along, across, height above the floor), along and across as the line was when its
    /// points were taken.
    pub(super) pts: Vec<[f64; 3]>,
    pub(super) wall: bool,
    pub(super) squared: bool,
    /// Which side, +1 along the normal, -1 against, the scan saw it from; 0 both or unknown.
    pub(super) seen: [bool; 2],
    pub(super) height: f64,
    /// Its points reach the ceiling over most of its length (a wall, or what a wall's lower
    /// part is hidden from: whether the floor is reached too is settled once what stands in
    /// front of it is known).
    pub(super) tall: bool,
    /// For each column of 3 in along it from `s0` at the time it was looked at: whether its points
    /// reach down to within 6 in of the floor.
    pub(super) floor_cols: Vec<bool>,
    /// And whether they reach up to within 6 in of the ceiling.
    pub(super) ceil_cols: Vec<bool>,
    /// Its points reach the ceiling over less than most of its length but over a good part of it: a wall
    /// that something tall stands before, or a thing that stops short (settled once what stands is known).
    pub(super) partial: bool,
    /// It reaches the ceiling and not the floor and nothing stands before it: not a wall, not a thing.
    pub(super) hung: bool,
    pub(super) cols_from: f64,
    /// The ceiling's height over it, above the floor (the lower side's, where the two differ).
    pub(super) ceil: f64,
    /// The side of the squares its columns were looked at in.
    pub(super) sq: f64,
    /// How much of it the scan shows: the squares of 3 in along and up that hold points, in square feet.
    pub(super) seen_area: f64,
    /// The band its points were taken in, across it (wider in a scan that scatters more), and how deep it bows over what length (call 188, 1).
    pub(super) tol: f64,
    pub(super) bow: (f64, f64),
    /// How many faces were joined into this one (a wall scanned in passes), and whether this one was itself joined into another (call 188, 2).
    pub(super) joined: usize,
    pub(super) gone: bool,
    /// Which side the scan saw it from was told by floor in front of it (not guessed where neither side showed enough floor).
    pub(super) seen_sure: bool,
}

impl Face {
    pub(super) fn at(&self, s: f64) -> P2 {
        let (d, n) = (dir(self.theta), normal(self.theta));
        [s * d[0] + self.rho * n[0], s * d[1] + self.rho * n[1]]
    }
    pub(super) fn mid(&self) -> f64 {
        (self.s0 + self.s1) / 2.0
    }
}

/// Where the two ends of `f` lie along the direction `d` (a line's own way, which may be the other way round from `f`'s: both are one line
/// to a degree), the lesser first (call 183: two faces of one wall found running opposite ways were laid over one another mirrored, and an opening
/// in the wall was filled by the other face's points).
pub(super) fn along_span(f: &Face, d: P2) -> (f64, f64) {
    let (a, b) = (dot(f.at(f.s0), d), dot(f.at(f.s1), d));
    (a.min(b), a.max(b))
}

pub(crate) fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// The edge of a solid, measured to the point. A 1.5 cm scatter blurs an edge; the
/// nearest point on the hollow side lies a little inside it. What does not blur is the
/// count: from `x0`, a place well inside the solid, the points between there and the
/// hollow number so many, and the solid's density (points to the foot) is counted from
/// a stretch beyond `x0` that is all solid. The edge is where that many points, at that
/// density, begin. `up` says the solid lies toward larger values. `v` are the places of
/// the points in the strip, `room` how far the solid runs beyond `x0`, and `hollow_end`
/// how far into the hollow the counting goes (1 ft at most). Where the solid is too short
/// to count the density in, `None`.
pub(crate) fn edge(v: &[f64], x0: f64, up: bool, room: f64, hollow_end: f64) -> Option<f64> {
    let look = EDGE_LOOK.min(room);
    if look < 0.2 {
        return None;
    }
    let (dense, between, reach) = if up {
        let back = (x0 - 1.0).max(hollow_end);
        let between: Vec<f64> = v.iter().copied().filter(|x| *x >= back && *x < x0).collect();
        (v.iter().filter(|x| **x >= x0 && **x < x0 + look).count(), between.len(), between.iter().copied().fold(f64::MAX, f64::min))
    } else {
        let back = (x0 + 1.0).min(hollow_end);
        let between: Vec<f64> = v.iter().copied().filter(|x| *x <= back && *x > x0).collect();
        (v.iter().filter(|x| **x <= x0 && **x > x0 - look).count(), between.len(), between.iter().copied().fold(f64::MIN, f64::max))
    };
    if dense < 5 {
        return None;
    }
    let density = dense as f64 / look;
    // The edge is not beyond the farthest point of what is counted: where the solid a little further in is thicker than where the density was
    // counted (the jamb of a door beside its head, at a wall's end), the count would carry the edge feet past the last point (call 183, 4).
    let (counted, extent) = (between as f64 / density, (reach - x0).abs());
    let covered = if between > 0 && counted > EDGE_THICKER * extent + EDGE_OVERSHOOT { extent } else { counted };
    Some(if up { x0 - covered } else { x0 + covered })
}

/// A grid of 3 in squares along a wall and up it, with the count of points in each.
pub(crate) struct Grid {
    pub(crate) cols: usize,
    pub(crate) rows: usize,
    pub(super) n: Vec<u32>,
    /// The count a square needs to be there: a quarter of what the middling square that
    /// has any holds, at least 2.
    pub(super) need: u32,
    pub(super) s0: f64,
    /// The side of a square, in feet: 3 in on a dense scan, larger on a thin one (`Measures`).
    pub(super) sq: f64,
}

impl Grid {
    /// `tops(s)` is the ceiling's height over the way along `s`: squares from there up are not part of the wall
    /// (they are marked full, so that no hole is found above a lower ceiling).
    pub(crate) fn new(pts: &[[f64; 2]], s0: f64, s1: f64, high: f64, sq: f64, tops: &dyn Fn(f64) -> f64) -> Grid {
        let cols = (((s1 - s0) / sq).ceil() as usize).max(1);
        let rows = (((high - SET_ASIDE) / sq + 1e-9).floor() as usize).max(1);
        let mut n = vec![0u32; cols * rows];
        for p in pts {
            let (c, r) = (((p[0] - s0) / sq).floor(), ((p[1] - SET_ASIDE) / sq).floor());
            if c >= 0.0 && r >= 0.0 && (c as usize) < cols && (r as usize) < rows {
                n[r as usize * cols + c as usize] += 1;
            }
        }
        let held = median(n.iter().filter(|x| **x > 0).map(|x| f64::from(*x)).collect());
        let need = ((held / 4.0).ceil() as u32).max(2);
        for c in 0..cols {
            let top = tops(s0 + (c as f64 + 0.5) * sq) - SET_ASIDE;
            for r in 0..rows {
                if SET_ASIDE + r as f64 * sq >= top {
                    n[r * cols + c] = u32::MAX;
                }
            }
        }
        Grid { cols, rows, n, need, s0, sq }
    }
    pub(crate) fn full(&self, c: usize, r: usize) -> bool {
        self.n[r * self.cols + c] >= self.need
    }
    pub(crate) fn col_range(&self, c: usize) -> f64 {
        self.s0 + c as f64 * self.sq
    }
}

/// A face reaches the ceiling or stops short of it: its points run to within 6 in of the ceiling
/// over most of its length, or they do not (and then how tall it is). Whether it also reaches the
/// floor is kept for the columns it is looked at in: where it does not, what stands in front of it
/// may be why (call 179, A2), which is settled once what stands is known.
pub(super) fn classify(f: &mut Face, ceil: &Ceil, sq: f64) {
    // The ceiling over each column of it: where it is not one height, the lower of the two sides' (a wall
    // between a low room and a high one reaches the ceiling if it reaches the low one's: call 179, A4).
    let n_of = normal(f.theta);
    let cols_n = (((f.s1 - f.s0) / sq).ceil() as usize).max(1);
    let heights: Vec<f64> = (0..cols_n).map(|c| ceil.over(f.at(f.s0 + (c as f64 + 0.5) * sq), n_of)).collect();
    f.ceil = median(heights.clone());
    let high = heights.iter().copied().fold(0.0, f64::max) - SET_ASIDE;
    let tops = |s: f64| heights[(((s - f.s0) / sq).floor().max(0.0) as usize).min(cols_n - 1)];
    let g = Grid::new(&f.pts.iter().map(|p| [p[0], p[2]]).collect::<Vec<_>>(), f.s0, f.s1, high, sq, &tops);
    let cols = g.cols as f64;
    // Within 6 in of the ceiling: the strip from 6 in under it to the 3 in set aside under it, each column
    // of it with as many points as a square of the grid needs, scaled to the strip's height (it is a quarter
    // of a foot, and a square on a thin scan is more). Not the squares whose tops are above it: how many
    // squares fit under the ceiling depends on the inch the floor and the ceiling were read to, and on how
    // large a square is, and a scan turned and leaned read a hair over.
    // On a thin scan the strip is as much taller as the squares are larger (`Measures`): a strip of 3 in holds
    // two or three points to a column and chance alone empties a third of them.
    let strip_high = SET_ASIDE * sq / SQUARE;
    let mut strip = vec![0u32; g.cols];
    for p in &f.pts {
        let c = ((p[0] - f.s0) / sq).floor();
        if c >= 0.0 && (c as usize) < g.cols && p[2] >= heights[c as usize] - SET_ASIDE - strip_high {
            strip[c as usize] += 1;
        }
    }
    let need_top = ((f64::from(g.need) * strip_high / sq).ceil() as u32).max(2);
    f.ceil_cols = strip.iter().map(|n| *n >= need_top).collect();
    let ceiling_cols = f.ceil_cols.iter().filter(|x| **x).count() as f64;
    f.floor_cols = (0..g.cols).map(|c| g.n[c] >= g.need).collect();
    f.cols_from = f.s0;
    f.sq = sq;
    f.seen_area = (0..g.rows).map(|r| (0..g.cols).filter(|c| g.n[r * g.cols + *c] != u32::MAX && g.full(*c, r)).count()).sum::<usize>() as f64 * sq * sq;
    f.tall = ceiling_cols / cols >= WALL_REACHES_CEILING;
    f.partial = !f.tall && ceiling_cols / cols >= SOME_REACHES_CEILING;
    f.hung = false;
    f.wall = f.tall || f.partial;
    f.height = if f.tall { 0.0 } else { top_of(&f.pts) };
}

/// Where, along `f`, the line of `h` crosses `f`'s own line, and where along `h` that is.
pub(super) fn cross_at(f: &Face, h: &Face) -> Option<(f64, f64)> {
    let (d, n) = (dir(f.theta), normal(f.theta));
    let (hd, hn) = (dir(h.theta), normal(h.theta));
    let denom = dot(d, hn);
    if (f.theta - h.theta).sin().abs() < NOT_PARALLEL || denom.abs() < 1e-9 {
        return None;
    }
    let s = (h.rho - f.rho * dot(n, hn)) / denom;
    let q = [s * d[0] + f.rho * n[0], s * d[1] + f.rho * n[1]];
    Some((s, dot(q, hd)))
}

/// Faces that reach the ceiling and are "a wall or something tall standing at the wall" (call
/// 179, A3): seen from one side only, both ends meeting no wall, with a wall face behind them
/// within 3 ft that goes on past an end of theirs. A cabinet bank within 6 in of the ceiling is
/// that: its back is not seen, the wall behind it shows either side. What it is that stands there
/// is the person's to say. (A wall of two faces that stops in the room has a face behind each
/// face too, but that face goes no further than the other: it is the wall's other side.)
pub(super) fn tall_at_a_wall(faces: &[Face], wall_ids: &[usize]) -> HashMap<usize, String> {
    let mut candidates: HashMap<usize, f64> = HashMap::new();
    for &i in wall_ids {
        let f = &faces[i];
        if !f.tall || f.seen[0] == f.seen[1] {
            continue;
        }
        let side = if f.seen[0] { 1.0 } else { -1.0 };
        let n = normal(f.theta);
        let behind: Vec<(f64, f64, f64)> = wall_ids
            .iter()
            .filter(|&&j| j != i)
            .filter_map(|&j| {
                let g = &faces[j];
                let mut dt = (f.theta - g.theta).abs();
                dt = dt.min(std::f64::consts::PI - dt);
                let across = (dot(g.at(g.mid()), n) - f.rho) * -side;
                if dt.to_degrees() > PAIR_DEGREES || across <= 0.0 || across > BEHIND {
                    return None;
                }
                let (b0, b1) = along_span(g, dir(f.theta));
                Some((b0, b1, across))
            })
            .collect();
        let near = |e: f64| behind.iter().any(|b| e >= b.0 - BEHIND && e <= b.1 + BEHIND);
        let goes_on = behind.iter().any(|b| b.0 < f.s0 - CORNER || b.1 > f.s1 + CORNER);
        if near(f.s0) && near(f.s1) && goes_on {
            candidates.insert(i, behind.iter().map(|b| b.2).fold(f64::MAX, f64::min));
        }
    }
    let mut out = HashMap::new();
    for (&i, &across) in &candidates {
        let f = &faces[i];
        let meets = |e: f64| {
            wall_ids.iter().filter(|j| **j != i && !candidates.contains_key(j)).any(|&j| {
                let h = &faces[j];
                cross_at(f, h).is_some_and(|(s, along)| (s - e).abs() <= CORNER && along >= h.s0 - CORNER && along <= h.s1 + CORNER)
            })
        };
        if !meets(f.s0) && !meets(f.s1) {
            out.insert(i, format!("a wall or something tall standing at the wall: it reaches the ceiling, but neither end meets a wall and a wall face stands {} behind it", plumbline_units(across)));
        }
    }
    // Any face whose two ends meet no wall is in doubt, whatever is behind it (call 183, 3): a wall of one face that stops in the room, or the front of
    // something tall. A face with another beside it that it could be paired with (a wall of two faces that stops in the room) is not asked about.
    for &i in wall_ids {
        let f = &faces[i];
        if out.contains_key(&i) || candidates.contains_key(&i) || !f.tall {
            continue;
        }
        let has_partner = wall_ids.iter().filter(|&&j| j != i).any(|&j| {
            let g = &faces[j];
            let mut dt = (f.theta - g.theta).abs();
            dt = dt.min(std::f64::consts::PI - dt);
            let across = (dot(g.at(g.mid()), normal(f.theta)) - f.rho).abs();
            let (b0, b1) = along_span(g, dir(f.theta));
            dt.to_degrees() <= PAIR_DEGREES && across >= PAIR_LEAST && across <= PAIR_MOST_BARE && f.s1.min(b1) - f.s0.max(b0) >= MIN_FACE
        });
        if has_partner {
            continue;
        }
        let meets = |e: f64| {
            wall_ids.iter().filter(|j| **j != i && !candidates.contains_key(j) && !out.contains_key(j)).any(|&j| {
                let h = &faces[j];
                cross_at(f, h).is_some_and(|(s, along)| (s - e).abs() <= CORNER && along >= h.s0 - CORNER && along <= h.s1 + CORNER)
            })
        };
        if !meets(f.s0) && !meets(f.s1) {
            out.insert(i, "a face that reaches the ceiling and meets no wall at either end: a wall seen from one side that stops in the room, or the front of something tall standing in it".to_string());
        }
    }
    out
}

/// Curved faces (call 183, 5). The upright points that no wall found accounts for (those near a wall face not in doubt are its own) are gathered into
/// pieces that run on from one to the next; a piece at least 5 ft long whose points lie off a straight line by 0.2 ft or more (a chord of an arc of 8 ft
/// radius, 4 ft long, is 0.25 ft off) and are fitted by a circle at least three times as closely, of a radius a room can have (2 ft 6 in to 80 ft)
/// and an arc of 30 degrees or more, is a curved face. Polygons (a bay of 45 degree walls) are not: their walls are found as walls, and their points
/// are accounted for.
pub(super) fn curved(pool: &[P2], faces: &[Face], wall_ids: &[usize], doubts: &HashMap<usize, String>) -> Vec<Curve> {
    let near_a_wall = |p: P2| {
        wall_ids.iter().filter(|i| !doubts.contains_key(i)).any(|&i| {
            let f = &faces[i];
            (dot(p, normal(f.theta)) - f.rho).abs() <= BAND + 0.1 && dot(p, dir(f.theta)) >= f.s0 - 0.2 && dot(p, dir(f.theta)) <= f.s1 + 0.2
        })
    };
    let mut cells: HashMap<(i64, i64), Vec<P2>> = HashMap::new();
    for p in pool.iter().filter(|p| !near_a_wall(**p)) {
        cells.entry(((p[0] / CURVE_CELL).floor() as i64, (p[1] / CURVE_CELL).floor() as i64)).or_default().push(*p);
    }
    cells.retain(|_, v| v.len() >= 3);
    let mut seen: HashSet<(i64, i64)> = HashSet::new();
    let mut keys: Vec<(i64, i64)> = cells.keys().copied().collect();
    keys.sort();
    let mut out = Vec::new();
    for k in keys {
        if !seen.insert(k) {
            continue;
        }
        let (mut stack, mut pts) = (vec![k], Vec::new());
        while let Some((ci, cj)) = stack.pop() {
            pts.extend(cells[&(ci, cj)].iter().copied());
            for di in -1..=1 {
                for dj in -1..=1 {
                    let n = (ci + di, cj + dj);
                    if cells.contains_key(&n) && seen.insert(n) {
                        stack.push(n);
                    }
                }
            }
        }
        if pts.len() < CURVE_LEFTOVER {
            continue;
        }
        // The short walls the finder took of the curve are part of it: those with an end or their middle against the pieces left over (a chord of an arc is
        // found straight where it is short, and the bulge between its end and the next is what is left over). A bay has nothing left over to begin with.
        let reached: HashSet<(i64, i64)> = pts.iter().map(|p| ((p[0] / CURVE_CELL).floor() as i64, (p[1] / CURVE_CELL).floor() as i64)).collect();
        for &i in wall_ids {
            let f = &faces[i];
            if f.s1 - f.s0 >= CURVE_CHORD {
                continue;
            }
            let touches = |p: P2| {
                let (ci, cj) = ((p[0] / CURVE_CELL).floor() as i64, (p[1] / CURVE_CELL).floor() as i64);
                (-2..=2).any(|di| (-2..=2).any(|dj| reached.contains(&(ci + di, cj + dj))))
            };
            if touches(f.at(f.s0)) || touches(f.at(f.s1)) || touches(f.at(f.mid())) {
                let (d, n) = (dir(f.theta), normal(f.theta));
                pts.extend(f.pts.iter().map(|p| [p[0] * d[0] + p[1] * n[0], p[0] * d[1] + p[1] * n[1]]));
            }
        }
        let Some((theta, mean)) = long_way(&pts) else { continue };
        let (d, n) = (dir(theta), normal(theta));
        let along: Vec<f64> = pts.iter().map(|p| dot(sub2(*p, mean), d)).collect();
        let across: Vec<f64> = pts.iter().map(|p| dot(sub2(*p, mean), n)).collect();
        let extent = along.iter().copied().fold(f64::MIN, f64::max) - along.iter().copied().fold(f64::MAX, f64::min);
        let line_rms = (across.iter().map(|a| a * a).sum::<f64>() / across.len() as f64).sqrt();
        if extent < CURVE_LEAST || line_rms < CURVE_OFF {
            continue;
        }
        // The circle (Kasa's fit): x^2 + y^2 + a x + b y + c = 0 in the least squares sense, about the mean.
        let m = pts.len() as f64;
        let (mut sx, mut sy, mut sxx, mut syy, mut sxy, mut sz, mut sxz, mut syz) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for p in &pts {
            let (x, y) = (p[0] - mean[0], p[1] - mean[1]);
            let z = x * x + y * y;
            sx += x;
            sy += y;
            sxx += x * x;
            syy += y * y;
            sxy += x * y;
            sz += z;
            sxz += x * z;
            syz += y * z;
        }
        // Normal equations for (a, b, c): [sxx sxy sx; sxy syy sy; sx sy m] [a b c]^T = -[sxz syz sz]^T.
        let det = |a: [[f64; 3]; 3]| a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1]) - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0]) + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
        let a = [[sxx, sxy, sx], [sxy, syy, sy], [sx, sy, m]];
        let d0 = det(a);
        if d0.abs() < 1e-9 {
            continue;
        }
        let rhs = [-sxz, -syz, -sz];
        let solve = |col: usize| {
            let mut t = a;
            for r in 0..3 {
                t[r][col] = rhs[r];
            }
            det(t) / d0
        };
        let (ca, cb, cc) = (solve(0), solve(1), solve(2));
        let centre = [-ca / 2.0, -cb / 2.0];
        let radius2 = centre[0] * centre[0] + centre[1] * centre[1] - cc;
        if radius2 <= 0.0 {
            continue;
        }
        let radius = radius2.sqrt();
        let rms = (pts.iter().map(|p| { let r = (p[0] - mean[0] - centre[0]).hypot(p[1] - mean[1] - centre[1]) - radius; r * r }).sum::<f64>() / m).sqrt();
        let angles: Vec<f64> = pts.iter().map(|p| (p[1] - mean[1] - centre[1]).atan2(p[0] - mean[0] - centre[0])).collect();
        // The arc's span: the angles about the middle one (the middle of the points' own way round), so that a seam at 180 degrees does not split it.
        let mid = angles.iter().map(|a| a.sin()).sum::<f64>().atan2(angles.iter().map(|a| a.cos()).sum::<f64>());
        let rel: Vec<f64> = angles.iter().map(|a| { let mut r = a - mid; while r > std::f64::consts::PI { r -= std::f64::consts::TAU; } while r < -std::f64::consts::PI { r += std::f64::consts::TAU; } r }).collect();
        let (lo, hi) = (rel.iter().copied().fold(f64::MAX, f64::min), rel.iter().copied().fold(f64::MIN, f64::max));
        let span = hi - lo;
        if rms > CURVE_FIT * line_rms || radius < CURVE_RADIUS.0 || radius > CURVE_RADIUS.1 || span.to_degrees() < CURVE_ARC.0 || span.to_degrees() > CURVE_ARC.1 {
            continue;
        }
        let c = [mean[0] + centre[0], mean[1] + centre[1]];
        let at = |a: f64| [c[0] + radius * (mid + a).cos(), c[1] + radius * (mid + a).sin()];
        out.push(Curve { from: at(lo), to: at(hi), centre: c, radius, length: radius * span, mid, span: (lo, hi) });
    }
    out
}

pub(super) fn sub2(a: P2, b: P2) -> P2 {
    [a[0] - b[0], a[1] - b[1]]
}
pub(super) fn unit2(a: P2) -> P2 {
    let l = a[0].hypot(a[1]).max(1e-12);
    [a[0] / l, a[1] / l]
}

/// A tall face with nothing between its lower part and its upper part over most of its length, with a wall face standing behind that hole within 3 ft
/// (call 183, 3): the front of something standing at the wall with something over it in the same plane (base cabinets and a soffit; a bar and a bulkhead),
/// and not a wall: the lower part and the upper part are things in one plane, and what shows between is the wall behind them. Where nothing stands
/// behind the hole it is a window along most of the wall, and is measured as one. Said as the other faces in doubt are, and laid only on the person's word.
pub(super) fn holed(faces: &[Face], wall_ids: &[usize], band: &[[f64; 3]], density: f64) -> HashMap<usize, String> {
    let mut out = HashMap::new();
    for &i in wall_ids {
        let f = &faces[i];
        if !f.tall || f.floor_cols.is_empty() {
            continue;
        }
        let cols = f.floor_cols.len();
        let tiers = ((f.ceil / HOLE_TIER).ceil() as usize).max(1);
        let mut seen = vec![vec![0u32; tiers]; cols];
        for p in &f.pts {
            let c = ((p[0] - f.cols_from) / f.sq).floor();
            if c >= 0.0 && (c as usize) < cols {
                seen[c as usize][((p[2] / HOLE_TIER) as usize).min(tiers - 1)] += 1;
            }
        }
        // For each column the widest stretch of empty tiers between points above and points below it.
        let mut holes: Vec<(f64, f64)> = Vec::new();
        for col in &seen {
            let occupied: Vec<usize> = (0..tiers).filter(|t| col[*t] > 0).collect();
            let gap = occupied.windows(2).max_by_key(|w| w[1] - w[0]);
            if let Some(w) = gap {
                let high = (w[1] - w[0] - 1) as f64 * HOLE_TIER;
                if high >= HOLE_HIGH {
                    holes.push(((w[0] + 1) as f64 * HOLE_TIER, w[1] as f64 * HOLE_TIER));
                }
            }
        }
        if (holes.len() as f64) < HOLE_SHARE * cols as f64 {
            continue;
        }
        let (lo, hi) = (median(holes.iter().map(|h| h.0).collect()), median(holes.iter().map(|h| h.1).collect()));
        // What stands behind the hole, on the side the scan did not see the face from (either side where it is not known).
        let (d, n) = (dir(f.theta), normal(f.theta));
        let sides: Vec<f64> = if f.seen[0] != f.seen[1] { vec![if f.seen[0] { -1.0 } else { 1.0 }] } else { vec![1.0, -1.0] };
        let behind = band
            .iter()
            .filter(|p| p[2] > lo + 0.3 && p[2] < hi - 0.3 && dot([p[0], p[1]], d) >= f.s0 && dot([p[0], p[1]], d) <= f.s1)
            .filter(|p| {
                let a = dot([p[0], p[1]], n) - f.rho;
                sides.iter().any(|sg| a * sg > IN_FRONT_LEAST && a * sg <= BEHIND)
            })
            .count() as f64;
        let would = density * (f.s1 - f.s0) * (hi - lo - 0.6).max(0.0);
        if behind >= HOLE_BACKED * would {
            out.insert(i, format!("a face with nothing between its part from the floor and its part under the ceiling ({} to {} up) along most of its length, and a wall standing behind that: the front of something at the wall with something over it, such as base cabinets and a soffit, and not a wall", plumbline_units(lo), plumbline_units(hi)));
        }
    }
    out
}

/// The same, against every face but `skip`.
/// How far the points of a face lie from its line, root mean square.

pub(super) fn crosses_other(p: P2, theta: f64, faces: &[Face], skip: usize) -> bool {
    faces.iter().enumerate().any(|(j, g)| {
        j != skip && (theta - g.theta).sin().abs() >= CROSSES && (dot(p, normal(g.theta)) - g.rho).abs() <= g.tol + 0.05 && dot(p, dir(g.theta)) >= g.s0 - SNAP && dot(p, dir(g.theta)) <= g.s1 + SNAP
    })
}

/// Whether a point is on a face already found that this line crosses: at a corner the
/// crossing face's points lie within a band of the line too and would pull it, and its
/// ends, away from where the thing itself is; and a line that only joins the faces of two
/// things (through a sofa's front and back) has nothing of its own.
pub(super) fn crosses_face(p: P2, theta: f64, faces: &[Face]) -> bool {
    faces.iter().any(|g| {
        (theta - g.theta).sin().abs() >= CROSSES && (dot(p, normal(g.theta)) - g.rho).abs() <= g.tol + 0.05 && dot(p, dir(g.theta)) >= g.s0 - 0.2 && dot(p, dir(g.theta)) <= g.s1 + 0.2
    })
}

/// The strongest line the cells vote for: (direction index k, offset along that line's normal, votes): the line runs at k steps from east.
pub(super) fn strongest(votes: &[P2], cos: &[f64], sin: &[f64], least: u32) -> Option<(usize, f64, u32)> {
    let reach = votes.iter().map(|p| p[0].abs() + p[1].abs()).fold(0.0, f64::max);
    let nb = (2.0 * reach / VOTE_FEET).ceil() as usize + 4;
    let mut acc = vec![0u32; cos.len() * nb];
    for p in votes {
        for k in 0..cos.len() {
            let rho = p[0] * cos[k] + p[1] * sin[k];
            acc[k * nb + ((rho + reach) / VOTE_FEET) as usize + 1] += 1;
        }
    }
    let mut best: Option<(usize, usize, u32)> = None;
    for k in 0..cos.len() {
        for b in 1..nb - 1 {
            let score = acc[k * nb + b - 1] + acc[k * nb + b] + acc[k * nb + b + 1];
            if best.is_none_or(|x| score > x.2) {
                best = Some((k, b, score));
            }
        }
    }
    best.filter(|x| x.2 >= least).map(|(k, b, score)| (k, (b as f64 - 1.0 + 0.5) * VOTE_FEET - reach, score))
}

/// The stretches along a line that points run through, as (from, to) along it: columns of
/// 3 in with at least COLUMN_POINTS points each are there, and a gap of GAP or more parts one
/// stretch from the next. `s` is sorted.
pub(super) fn runs_of(s: &[f64], sq: f64) -> Vec<(f64, f64)> {
    let Some((first, last)) = s.first().zip(s.last()) else { return Vec::new() };
    let cols = (((last - first) / sq).floor() as usize) + 1;
    let mut count = vec![0usize; cols];
    for x in s {
        count[(((x - first) / sq).floor() as usize).min(cols - 1)] += 1;
    }
    let gap_cols = (GAP / sq).round() as usize;
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let (mut begun, mut at): (Option<usize>, usize) = (None, 0);
    for (c, n) in count.iter().enumerate() {
        if *n >= COLUMN_POINTS {
            match begun {
                Some(_) if c - at <= gap_cols => {}
                Some(b) => {
                    runs.push((b, at));
                    begun = Some(c);
                }
                None => begun = Some(c),
            }
            at = c;
        }
    }
    if let Some(b) = begun {
        runs.push((b, at));
    }
    runs.into_iter().map(|(a, b)| (first + a as f64 * sq, first + (b + 1) as f64 * sq)).collect()
}

/// A vote's line told again from the points near it (see where it is used): its direction from all of them, and
/// its place from the fullest 3 in across it of them.
pub(super) fn seed_line(pool: &[P2], theta: f64, rho: f64, seed_band: f64) -> (f64, f64) {
    let n = normal(theta);
    let near: Vec<P2> = pool.iter().copied().filter(|p| (dot(*p, n) - rho).abs() <= seed_band + 0.05).collect();
    if near.len() < 20 {
        return (theta, rho);
    }
    // Its own direction, if that is within a degree and a half of the vote's (more is another wall's points
    // crossing it).
    let theta = match long_way(&near) {
        Some((t, _)) => {
            let t = if (t - theta).abs() > std::f64::consts::FRAC_PI_2 { if t > theta { t - std::f64::consts::PI } else { t + std::f64::consts::PI } } else { t };
            if (t - theta).abs().to_degrees() <= 1.5 { t } else { theta }
        }
        None => theta,
    };
    let n = normal(theta);
    let offs: Vec<f64> = near.iter().map(|p| dot(*p, n)).collect();
    let bins = (2.0 * (seed_band + 0.05) / FINE_BIN).ceil() as usize + 1;
    let mut count = vec![0u32; bins];
    for o in &offs {
        let b = ((o - (rho - seed_band - 0.05)) / FINE_BIN).floor();
        if b >= 0.0 && (b as usize) < bins {
            count[b as usize] += 1;
        }
    }
    // The fullest three bins together (a face's scatter is a few bins wide).
    let best = (0..bins).max_by_key(|b| (b.saturating_sub(1)..=(*b + 1).min(bins - 1)).map(|i| count[i]).sum::<u32>()).unwrap_or(0);
    let centre = rho - seed_band - 0.05 + (best as f64 + 0.5) * FINE_BIN;
    let mine: Vec<f64> = offs.into_iter().filter(|o| (o - centre).abs() <= 0.1).collect();
    if mine.is_empty() {
        return (theta, rho);
    }
    (theta, mine.iter().sum::<f64>() / mine.len() as f64)
}

/// The direction (radians, 0 to pi) of the points' own long way, and their mean.
pub(super) fn long_way(pts: &[P2]) -> Option<(f64, P2)> {
    if pts.len() < 3 {
        return None;
    }
    let n = pts.len() as f64;
    let (mx, my) = (pts.iter().map(|p| p[0]).sum::<f64>() / n, pts.iter().map(|p| p[1]).sum::<f64>() / n);
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for p in pts {
        let (x, y) = (p[0] - mx, p[1] - my);
        sxx += x * x;
        sxy += x * y;
        syy += y * y;
    }
    let mut t = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    if t < 0.0 {
        t += std::f64::consts::PI;
    }
    Some((t, [mx, my]))
}

/// How long a scan is on the plan, east to west and north to south: the middle 95 in 100 of its
/// points, so that the strays outside the house (one in fifty) do not make it long.
pub(super) fn span(scan: &Scan, axis: usize) -> f64 {
    let mut v: Vec<f64> = scan.points.iter().map(|p| p[axis]).collect();
    if v.len() < 40 {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() - 1 - v.len() / 40] - v[v.len() / 40]
}

pub(super) fn plan_size(scan: &Scan) -> (f64, f64) {
    (span(scan, 0), span(scan, 1))
}

/// Whether this is likely a house at all, from the scan's own size (call 179, A7): the sentence
/// when it is not.
pub(crate) fn unlikely(scan: &Scan, heights: Option<(f64, f64)>) -> Option<String> {
    let ft_in = |x: f64| plumbline_units(x);
    let check = "this is not likely the house: check the measure and which way is up";
    if let Some((lowest, highest)) = heights {
        if lowest < LEAST_HIGH {
            return Some(format!("{check} (floor to ceiling reads {}, under {})", ft_in(lowest), ft_in(LEAST_HIGH)));
        }
        if highest > MOST_HIGH {
            return Some(format!("{check} (floor to ceiling reads {}, over {})", ft_in(highest), ft_in(MOST_HIGH)));
        }
    } else {
        // No ceiling shows: where the whole scan is lower than a room, that is why.
        let high = span(scan, 2);
        if high > 0.0 && high < LEAST_HIGH {
            return Some(format!("{check} (the whole scan is only {} high, and {} by {} across; a room is at least {} high)", ft_in(high), ft_in(plan_size(scan).0), ft_in(plan_size(scan).1), ft_in(LEAST_HIGH)));
        }
    }
    let (east, north) = plan_size(scan);
    if east.max(north) > MOST_LONG {
        return Some(format!("{check} (the scan is {} across, and a house is not over {})", ft_in(east.max(north)), ft_in(MOST_LONG)));
    }
    None
}

/// A count with its thousands set off, as the program writes them.
pub(crate) fn plumbline_world_figure(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

pub(super) fn plumbline_units_count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// A length as a tape reads it, feet and inches to the inch.
pub(crate) fn plumbline_units(feet: f64) -> String {
    let inches = (feet * 12.0).round() as i64;
    format!("{}' {}\"", inches.div_euclid(12), inches.rem_euclid(12))
}
