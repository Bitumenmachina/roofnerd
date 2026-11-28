use super::*;

/// The ceiling over the plan, as its height above the floor.
pub(crate) struct Ceil {
    /// One height over the whole house: the usual, and what the scan's own floor and ceiling say.
    pub(crate) flat: Option<f64>,
    pub(crate) cells: HashMap<(i64, i64), f64>,
    pub(crate) side: f64,
    /// Where no cell is known near.
    pub(crate) middle: f64,
    pub(crate) lowest: f64,
    pub(crate) highest: f64,
}

impl Ceil {
    pub(crate) fn at(&self, p: P2) -> f64 {
        if let Some(h) = self.flat {
            return h;
        }
        let (cx, cy) = ((p[0] / self.side).floor() as i64, (p[1] / self.side).floor() as i64);
        for ring in 0..=3i64 {
            let mut best: Option<(i64, f64)> = None;
            for dx in -ring..=ring {
                for dy in -ring..=ring {
                    if dx.abs().max(dy.abs()) != ring {
                        continue;
                    }
                    if let Some(h) = self.cells.get(&(cx + dx, cy + dy)) {
                        if best.is_none_or(|b| dx * dx + dy * dy < b.0) {
                            best = Some((dx * dx + dy * dy, *h));
                        }
                    }
                }
            }
            if let Some((_, h)) = best {
                return h;
            }
        }
        self.middle
    }
    /// Whether the ceiling was read near a place (a cell of it within a cell either way), or is one height everywhere.
    pub(crate) fn reads(&self, p: P2) -> bool {
        if self.flat.is_some() {
            return true;
        }
        let (cx, cy) = ((p[0] / self.side).floor() as i64, (p[1] / self.side).floor() as i64);
        (-1..=1).any(|dx| (-1..=1).any(|dy| self.cells.contains_key(&(cx + dx, cy + dy))))
    }
    /// The ceiling over a wall at `p` whose normal is `n`: the lower of what is over either side of it.
    pub(crate) fn over(&self, p: P2, n: P2) -> f64 {
        if let Some(h) = self.flat {
            return h;
        }
        self.at([p[0] + n[0] * CEIL_OUT, p[1] + n[1] * CEIL_OUT]).min(self.at([p[0] - n[0] * CEIL_OUT, p[1] - n[1] * CEIL_OUT]))
    }
}

/// What the ceiling is, from the points over the floor: the ceiling to use (None where none shows), and a
/// sentence where it is not one height. `side` is the cell's side, `rho` how thick the scan is.
pub(crate) fn ceiling_of(scan: &Scan, floor: f64, scan_ceiling: Option<f64>, rho: Option<f64>, side: f64) -> (Option<Ceil>, Option<String>) {
    let key = |p: &[f64; 3]| ((p[0] / side).floor() as i64, (p[1] / side).floor() as i64);
    // (sorted by cell, then taken cell by cell: a map with a list for each cell costs more than this on a
    // scan of a few hundred thousand points)
    let mut high: Vec<((i64, i64), f32, u8)> = Vec::new();
    let mut low: Vec<(i64, i64)> = Vec::new();
    for p in &scan.points {
        let z = p[2] - floor;
        if z >= LEAST_CEILING && z <= 30.0 {
            // Which ninth of its cell the point is in (3 by 3): a ceiling fills the cell, the foot of two walls at a corner does not.
            let (fx, fy) = (p[0] / side - (p[0] / side).floor(), p[1] / side - (p[1] / side).floor());
            high.push((key(p), z as f32, (((fx * 3.0) as u8).min(2)) * 3 + ((fy * 3.0) as u8).min(2)));
        } else if z.abs() <= SET_ASIDE {
            low.push(key(p));
        }
    }
    high.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    low.sort_unstable();
    low.dedup();
    let under: HashSet<(i64, i64)> = low.into_iter().collect();
    let least = (CEILING_SHARE * rho.unwrap_or(0.0) * side * side).max(6.0);
    let mut cells: HashMap<(i64, i64), f64> = HashMap::new();
    for run in high.chunk_by(|a, b| a.0 == b.0) {
        let mut bins = vec![0u32; 300];
        for (_, z, _) in run {
            bins[(((f64::from(*z) - LEAST_CEILING) / 0.1) as usize).min(299)] += 1;
        }
        // The highest half foot with as many points in it as a surface would hold.
        let window = (0..295).rev().find(|b| f64::from(bins[*b..*b + 5].iter().sum::<u32>()) >= least);
        if let Some(b) = window {
            let lo = LEAST_CEILING + b as f64 * 0.1;
            let mut filled = 0u16;
            let inside: Vec<f64> = run.iter().filter(|(_, z, _)| f64::from(*z) >= lo && f64::from(*z) < lo + 0.5).map(|(_, z, sub)| { filled |= 1 << *sub; f64::from(*z) }).collect();
            // A surface fills its cell (call 183, 6): the points of a wall's top, or two walls' at a corner, in the same half foot of height are not a ceiling.
            if f64::from(filled.count_ones()) < CEIL_FILL * 9.0 {
                continue;
            }
            // Its height is the middle of what is in the window, not the mean: what is above the ceiling or a wall's points below it, in the same window, do not move it.
            cells.insert(run[0].0, median(inside));
        }
    }
    let covered = cells.keys().filter(|k| under.contains(*k)).count();
    if cells.len() < 5 || (covered as f64) < 0.3 * under.len() as f64 {
        return (scan_ceiling.map(|c| Ceil { flat: Some(c - floor), cells: HashMap::new(), side, middle: c - floor, lowest: c - floor, highest: c - floor }), None);
    }
    let mut hs: Vec<f64> = cells.values().copied().collect();
    hs.sort_by(f64::total_cmp);
    let (p5, p95) = (hs[hs.len() / 20], hs[hs.len() - 1 - hs.len() / 20]);
    let middle = hs[hs.len() / 2];
    if p95 - p5 <= CEIL_FLAT {
        let h = scan_ceiling.map_or(middle, |c| c - floor);
        return (Some(Ceil { flat: Some(h), cells: HashMap::new(), side, middle: h, lowest: h, highest: h }), None);
    }
    // Groups of heights a step apart or more.
    let mut groups: Vec<Vec<f64>> = vec![vec![hs[0]]];
    for h in &hs[1..] {
        if h - groups.last().and_then(|g| g.last()).copied().unwrap_or(*h) >= CEIL_STEP {
            groups.push(Vec::new());
        }
        if let Some(g) = groups.last_mut() {
            g.push(*h);
        }
    }
    groups.retain(|g| g.len() as f64 >= CEIL_GROUP * hs.len() as f64);
    let tape = |x: f64| plumbline_units(x);
    let note = if groups.len() >= 2 {
        let names: Vec<String> = groups.iter().map(|g| tape(g[g.len() / 2])).collect();
        format!("the ceiling is not one height: {}; each wall is judged against the ceiling over it (a wall between two rooms, against the lower), and a wall that runs under both is laid at the height of the ceiling over its middle (the step in its top is not drawn)", names.join(" and "))
    } else {
        // A slope is said only where the cells' heights fit a plane better than a flat by more than their scatter (call 183, 6).
        let pts: Vec<(f64, f64, f64)> = cells.iter().map(|(k, h)| ((k.0 as f64 + 0.5) * side, (k.1 as f64 + 0.5) * side, *h)).collect();
        let n = pts.len() as f64;
        let (mx, my, mz) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n, pts.iter().map(|p| p.2).sum::<f64>() / n);
        let (mut sxx, mut sxy, mut syy, mut sxz, mut syz) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for p in &pts {
            let (x, y, z) = (p.0 - mx, p.1 - my, p.2 - mz);
            sxx += x * x;
            sxy += x * y;
            syy += y * y;
            sxz += x * z;
            syz += y * z;
        }
        let det = sxx * syy - sxy * sxy;
        let (a, b) = if det.abs() > 1e-9 { ((sxz * syy - syz * sxy) / det, (syz * sxx - sxz * sxy) / det) } else { (0.0, 0.0) };
        let flat_rms = (pts.iter().map(|p| (p.2 - middle).powi(2)).sum::<f64>() / n).sqrt();
        let plane_rms = (pts.iter().map(|p| (p.2 - mz - a * (p.0 - mx) - b * (p.1 - my)).powi(2)).sum::<f64>() / n).sqrt();
        let rise = pts.iter().map(|p| a * (p.0 - mx) + b * (p.1 - my)).fold(f64::MIN, f64::max) - pts.iter().map(|p| a * (p.0 - mx) + b * (p.1 - my)).fold(f64::MAX, f64::min);
        if plane_rms > SLOPE_BETTER * flat_rms || rise < CEIL_FLAT {
            // Neither one height, nor two, nor a slope the points fit: the ceiling is not told here; the scan's own is used, and that is said.
            return (scan_ceiling.map(|c| Ceil { flat: Some(c - floor), cells: HashMap::new(), side, middle: c - floor, lowest: c - floor, highest: c - floor }), Some("the ceiling's heights vary more than one height, two, or a slope can say: the scan's own floor to ceiling is used for every wall".to_string()));
        }
        format!("the ceiling slopes, from {} to {}; a wall under it is judged against the ceiling over it and laid at the height of the ceiling over its middle (its sloped top is not drawn)", tape(hs[0]), tape(hs[hs.len() - 1]))
    };
    (Some(Ceil { flat: None, cells, side, middle, lowest: hs[0], highest: hs[hs.len() - 1] }), Some(note))
}

/// How thick a scan is, and what is worked from it (call 179, A1). Every count in this file that depends on
/// how many points there are was tuned on scans of about this many points to the square foot of floor (a
/// phone's scan of a room is 100 or more): on a scan as thick or thicker the measures are as they were; on a
/// thinner one the squares and the cells a face is looked for in are made larger by the square root of how
/// much thinner it is, so that each holds as many points as it did, and the least a face is voted for by is
/// made fewer. Under the least that has been tried and found to tell a wall (the thinnest scan of a house
/// this found every wall of, tried at several sizes and angles) nothing is found, and that is said.
pub(crate) const REFERENCE_DENSITY: f64 = 80.0;
pub(crate) const LEAST_DENSITY: f64 = 8.0;
/// A scan under this share of the reference thickness is said to be thin, and what that does to what it finds.
pub(crate) const THIN_NOTE_BELOW: f64 = 0.7;
/// A line is voted for by at least this many upright cells however thin the scan.
pub(super) const MOST_THIN_VOTES: u32 = 8;

/// A number of points, in words.
pub(crate) fn figure(x: f64) -> String {
    format!("{} points", x.round() as i64)
}

/// Points to the square foot of the scan's floor: of the 1 ft squares that hold two or more points within 3 in
/// of the floor, the middle one's count. (The floor shows much as a wall does, at the same spacing; a square
/// at a wall's foot or under furniture holds fewer and does not set it.)
pub(crate) fn density(scan: &Scan, floor: f64) -> Option<f64> {
    let mut at: Vec<(i64, i64)> = scan.points.iter().filter(|p| (p[2] - floor).abs() <= SET_ASIDE).map(|p| (p[0].floor() as i64, p[1].floor() as i64)).collect();
    at.sort_unstable();
    let held: Vec<f64> = at.chunk_by(|a, b| a == b).map(|r| r.len()).filter(|n| *n >= 2).map(|n| n as f64).collect();
    (held.len() >= 10).then(|| median(held))
}
