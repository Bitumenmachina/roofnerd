//! A phone's scan is not a made-up one (DECISIONS.md call 188, 1 to 4): its walls are rough, bowed and a little off
//! plumb, a wall is scanned in more than one pass, and it holds a million points. What the finder does about it is here.
//!
//! The measures were taken from three real scans (a small room, a long one, a large notched one; none of them is in the
//! repository, and no point of them is copied here): their walls scatter 2.4 to 4.8 cm about a local plane against the
//! 1.5 cm of the made-up scans, lean up to 2 degrees over a room's height, bow up to 8 cm over a wall's length, and
//! show two to four layers an inch apart where a wall was passed over more than once. `research/worker-report-call188.md`
//! has the figures; each constant below says which of them it rests on.

use crate::found::P2;

fn dir(theta: f64) -> P2 {
    [theta.cos(), theta.sin()]
}
fn normal(theta: f64) -> P2 {
    [-theta.sin(), theta.cos()]
}
fn dot(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// The scatter of the made-up scans the finder was built on: 1.5 cm, in feet. The band a face's points lie in (`BAND`, 3 sigma of
/// it) is this much scatter and no more; a scan that scatters more has a wider band in proportion.
pub(crate) const SCATTER_MADE_UP: f64 = 0.0492;
/// A scan scatters more than the made-up ones only where its walls' own scatter is this many times theirs or more (the measure of a
/// clean scan is itself good to a few in a hundred; this much is not noise).
const NOTICE: f64 = 1.3;
/// The widest the band grows: 3 times the made-up scatter (4.5 cm of scatter about a wall's plane; the roughest real wall measured
/// was 4.8 cm, and a band wider than this takes in the next wall's points of a closet).
const MOST_WIDER: f64 = 3.0;
/// How far a wall may lean, as a rise over its height (3 1/2 degrees); real walls lean to 2 degrees in a scan from drift.
pub(crate) const TILT_MOST: f64 = 0.06;
/// A face turns to what its points say only from this long (a stub of 3 ft is told to a degree at best, and keeps the line of the vote it came from).
pub(crate) const FREE_FROM: f64 = 6.0;
/// A bow is told, and the face is fitted to it, from this depth (a quarter inch is the scatter's own); and a bow deeper than the most
/// is not a bow but a jog or two walls.
pub(crate) const BOW_FROM: f64 = 1.0 / 12.0 / 2.0;
pub(crate) const BOW_MOST: f64 = 0.5;
/// ... and a bow is shallow for its length: no deeper than this share of it (1 in 30: a wall 12 ft long bows 4 in at most; the real scans' walls bowed
/// 1 in 25 at worst, and an arc of 5 ft radius over a chord of 4 ft is 1 in 10, a curved wall and not a bowed one).
pub(crate) const BOW_RATIO: f64 = 1.0 / 30.0;
/// A bow is told only of a face this long or longer (6 ft): over less, a bow and a little scatter are not told apart.
pub(crate) const BOW_LEAST_LEN: f64 = 6.0;
/// The bow is said when it is this deep (an inch) or more.
pub(crate) const BOW_SAY: f64 = 1.0 / 12.0;
/// A bow is taken into the fit where what it takes off what is left across the face, per point left, is this many times what chance takes off.
const BOW_F: f64 = 60.0;

/// How rough a scan's walls are, from the strongest lines: the median of the robust spread (1.4826 times the median absolute deviation) of
/// the points about each line's own plane, in feet. `lines` are (angle, offset) of the lines the cells voted for.
pub(crate) fn wall_scatter(band: &[[f64; 3]], lines: &[(f64, f64)], win: f64) -> Option<f64> {
    let mut spreads: Vec<f64> = Vec::new();
    for (theta, rho) in lines {
        let n = normal(*theta);
        let mut m: Vec<[f64; 2]> = band.iter().filter(|p| (dot([p[0], p[1]], n) - rho).abs() <= win).map(|p| [dot([p[0], p[1]], n), p[2]]).collect();
        if m.len() < 200 {
            continue;
        }
        // The plane through them, of the points nearer and nearer it.
        let (mut c, mut t) = (rho / 1.0, 0.0);
        let mut w = win;
        for _ in 0..4 {
            let sel: Vec<&[f64; 2]> = m.iter().filter(|p| (p[0] - (c + t * p[1])).abs() <= w).collect();
            if sel.len() < 100 {
                break;
            }
            let n_ = sel.len() as f64;
            let (mz, ma) = (sel.iter().map(|p| p[1]).sum::<f64>() / n_, sel.iter().map(|p| p[0]).sum::<f64>() / n_);
            let (mut szz, mut sza) = (0.0, 0.0);
            for p in &sel {
                szz += (p[1] - mz) * (p[1] - mz);
                sza += (p[1] - mz) * (p[0] - ma);
            }
            t = if szz > 1e-9 { (sza / szz).clamp(-TILT_MOST, TILT_MOST) } else { 0.0 };
            c = ma - t * mz;
            w = (w * 0.75).max(win * 0.5);
        }
        let mut r: Vec<f64> = m.drain(..).map(|p| (p[0] - (c + t * p[1])).abs()).filter(|x| *x <= win).collect();
        if r.len() < 100 {
            continue;
        }
        r.sort_by(f64::total_cmp);
        spreads.push(1.4826 * r[r.len() / 2]);
    }
    if spreads.is_empty() {
        return None;
    }
    // The cleanest of them: a line through clutter or two walls close together is rougher than any wall, never smoother.
    spreads.sort_by(f64::total_cmp);
    Some(spreads[0])
}

/// How rough the floor is: the robust spread (1.4826 times the median absolute deviation) of the points within `reach` of it about the
/// plane through them, in feet. The floor of a room is flat to a quarter inch, so what it spreads is the scan's own roughness (call 188, 1).
pub(crate) fn floor_scatter(points: &[[f64; 3]], floor: f64, reach: f64) -> Option<f64> {
    let stride = (points.len() / 300_000).max(1);
    let near: Vec<[f64; 3]> = points.iter().step_by(stride).filter(|p| (p[2] - floor).abs() <= reach).copied().collect();
    if near.len() < 500 {
        return None;
    }
    let (mut a, mut b, mut c) = (0.0, 0.0, floor);
    let mut w = reach;
    for _ in 0..4 {
        // z = c + a x + b y over the points within w of it
        let sel: Vec<&[f64; 3]> = near.iter().filter(|p| (p[2] - (c + a * p[0] + b * p[1])).abs() <= w).collect();
        if sel.len() < 200 {
            return None;
        }
        let n = sel.len() as f64;
        let (mx, my, mz) = (sel.iter().map(|p| p[0]).sum::<f64>() / n, sel.iter().map(|p| p[1]).sum::<f64>() / n, sel.iter().map(|p| p[2]).sum::<f64>() / n);
        let (mut sxx, mut sxy, mut syy, mut sxz, mut syz) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for p in &sel {
            let (x, y, z) = (p[0] - mx, p[1] - my, p[2] - mz);
            sxx += x * x;
            sxy += x * y;
            syy += y * y;
            sxz += x * z;
            syz += y * z;
        }
        let det = sxx * syy - sxy * sxy;
        if det.abs() > 1e-9 {
            a = (sxz * syy - syz * sxy) / det;
            b = (syz * sxx - sxz * sxy) / det;
        }
        c = mz - a * mx - b * my;
        w = (w * 0.6).max(reach * 0.5);
    }
    let mut r: Vec<f64> = near.iter().map(|p| (p[2] - (c + a * p[0] + b * p[1])).abs()).filter(|x| *x <= reach).collect();
    if r.len() < 500 {
        return None;
    }
    r.sort_by(f64::total_cmp);
    Some(1.4826 * r[r.len() / 2])
}

/// How many times wider than the made-up scans' band this scan's is, from the scatter of its walls: 1 for a clean scan.
pub(crate) fn band_scale(scatter: Option<f64>) -> f64 {
    match scatter {
        Some(s) if s >= NOTICE * SCATTER_MADE_UP => (s / SCATTER_MADE_UP).min(MOST_WIDER),
        _ => 1.0,
    }
}

/// What a face is fitted to: its points as (along, across, up), the line, how it leans, and how it bows.
#[derive(Clone, Debug)]
pub(crate) struct Fit {
    pub theta: f64,
    /// Where the face is across its line, at the floor (a bowed face: the middle of the bow's own average).
    pub rho: f64,
    pub pts: Vec<[f64; 3]>,
    /// How far across the points lie from the face fitted (leaning and bowed as it is), root mean square.
    pub rms: f64,
    /// How deep the bow is, middle against the ends, in feet; zero where there is none worth telling; and the length it runs over.
    pub bow: f64,
    pub bow_len: f64,
}

/// The least squares of (across - base) on the columns `active` of `row(p)` (0 the offset, 1 the lean, 2 the slope along, 3 the bow); the rest are
/// zero. None where the points do not fix it.
fn solve(pts: &[[f64; 3]], base: f64, active: &[usize], row: &dyn Fn(&[f64; 3]) -> [f64; 4]) -> Option<[f64; 4]> {
    let m = active.len();
    let mut a = [[0.0f64; 5]; 4];
    for p in pts {
        let r = row(p);
        let y = p[1] - base;
        for (i, ci) in active.iter().enumerate() {
            for (j, cj) in active.iter().enumerate() {
                a[i][j] += r[*ci] * r[*cj];
            }
            a[i][4] += r[*ci] * y;
        }
    }
    for i in 0..m {
        let piv = (i..m).max_by(|x, y| a[*x][i].abs().total_cmp(&a[*y][i].abs()))?;
        if a[piv][i].abs() < 1e-9 {
            return None;
        }
        a.swap(i, piv);
        for k in i + 1..m {
            let f = a[k][i] / a[i][i];
            for j in i..=4 {
                a[k][j] -= f * a[i][j];
            }
        }
    }
    let mut sol = [0.0f64; 4];
    for i in (0..m).rev() {
        let mut s = a[i][4];
        for j in i + 1..m {
            s -= a[i][j] * sol[active[j]];
        }
        sol[active[i]] = s / a[i][i];
    }
    Some(sol)
}

/// How far up a face's points run: the 10th to the 90th in 100 of their heights. A lean is told only over this much height (a patch of wall seen
/// through a window, 3 ft to 7 ft up, and a stray or two near the floor, would otherwise tilt the plane about the stray).
fn height_spread(pts: &[[f64; 3]]) -> f64 {
    let mut z: Vec<f64> = pts.iter().map(|p| p[2]).collect();
    if z.len() < 10 {
        return 0.0;
    }
    z.sort_by(f64::total_cmp);
    z[z.len() * 9 / 10] - z[z.len() / 10]
}

/// A lean is told only of a face whose points run this high (4 ft: half a room's height).
const LEAN_NEEDS: f64 = 4.0;

fn rms_of(pts: &[[f64; 3]], model: impl Fn(&[f64; 3]) -> f64) -> f64 {
    (pts.iter().map(|p| (p[1] - model(p)).powi(2)).sum::<f64>() / pts.len().max(1) as f64).sqrt()
}

/// A face fitted to the points of `band` near the line (`theta`, `rho`) between `sa` and `sb` along it: across = c + tilt * up, and
/// where a bow shows, + bow * u^2 (u running from -1 to 1 along it); the points within `tol` of it, nearer each time. `skip(p, theta)`
/// says a point is another face's. `free` lets the direction turn to what the points say. None where too few points lie on it.
pub(crate) fn fit_face(band: &[[f64; 3]], theta0: f64, rho0: f64, sa: f64, sb: f64, tol: f64, free: bool, skip: &dyn Fn(P2, f64) -> bool) -> Option<Fit> {
    let (mut theta, mut rho, mut tilt) = (theta0, rho0, 0.0f64);
    let (mid, half) = ((sa + sb) / 2.0, ((sb - sa) / 2.0).max(0.5));
    let top = band.iter().map(|p| p[2]).fold(0.0, f64::max);
    // The points near the line, once, wide enough for the lean, the bow and the turn the fit may make; each pass looks at these only.
    let wide = tol + 0.3 + TILT_MOST * top + BOW_MOST + 0.05 * (sb - sa);
    let cand: Vec<[f64; 3]> = {
        let (d, n) = (dir(theta0), normal(theta0));
        band.iter()
            .filter(|p| {
                let q = [p[0], p[1]];
                let (a, s_) = (dot(q, n), dot(q, d));
                (a - rho0).abs() <= wide + 0.0 && s_ >= sa - 1.0 && s_ <= sb + 1.0 && !skip(q, theta0)
            })
            .copied()
            .collect()
    };
    // Those of them near the line as it now is, in its frame: [along, across, up].
    let gather = |theta: f64, reach: &dyn Fn(f64, f64) -> bool| -> Vec<[f64; 3]> {
        let (d, n) = (dir(theta), normal(theta));
        cand.iter()
            .filter_map(|p| {
                let q = [p[0], p[1]];
                let t = [dot(q, d), dot(q, n), p[2]];
                (t[0] >= sa - 0.2 && t[0] <= sb + 0.2 && reach(t[1], t[2])).then_some(t)
            })
            .collect()
    };
    let plane = |c: f64, t: f64| move |p: &[f64; 3]| c + t * p[2];
    // The plane: the line, how it leans, and (where the direction is free) which way it turns.
    for (it, w) in [tol, tol, tol, tol].iter().enumerate() {
        let (c, t) = (rho, tilt);
        let slack = 0.0;
        let _ = it;
        let all = gather(theta, &|a, z| (a - (c + t * z)).abs() <= w + slack);
        let members: Vec<[f64; 3]> = all.into_iter().filter(|p| (p[1] - (c + t * p[2])).abs() <= *w + slack).collect();
        if members.len() < 20 {
            return None;
        }
        let row = |p: &[f64; 3]| -> [f64; 4] { [1.0, p[2], (p[0] - mid) / half, 0.0] };
        let leans = height_spread(&members) >= LEAN_NEEDS;
        let active: &[usize] = match (leans, free) {
            (true, true) => &[0, 1, 2],
            (true, false) => &[0, 1],
            (false, true) => &[0, 2],
            (false, false) => &[0],
        };
        let sol = solve(&members, 0.0, active, &row)?;
        tilt = sol[1].clamp(-TILT_MOST, TILT_MOST);
        rho = sol[0];
        if free {
            // The slope along it turns the line, through the same middle.
            let (d, n) = (dir(theta), normal(theta));
            let pivot = [mid * d[0] + rho * n[0], mid * d[1] + rho * n[1]];
            let turn = (sol[2] / half).clamp(-0.05, 0.05);
            theta += turn;
            rho = dot(pivot, normal(theta));
        }
    }
    let flat = gather(theta, &|a, z| (a - (rho + tilt * z)).abs() <= tol);
    if flat.len() < 20 {
        return None;
    }
    let flat_rms = rms_of(&flat, plane(rho, tilt));
    let mut best = Fit { theta, rho, pts: flat, rms: flat_rms, bow: 0.0, bow_len: 2.0 * half };
    // A bow: the plane's points looked at again with a bow in the model, the reach wide enough for a bow of the most.
    let (mut c, mut t, mut b, mut sl) = (rho, tilt, 0.0f64, 0.0f64);
    let mut mem: Vec<[f64; 3]> = Vec::new();
    let row = |p: &[f64; 3]| -> [f64; 4] {
        let u = (p[0] - mid) / half;
        [1.0, p[2], u, u * u]
    };
    for w in [tol, tol, tol, tol] {
        let model = |p: &[f64; 3]| c + t * p[2] + sl * ((p[0] - mid) / half) + b * ((p[0] - mid) / half).powi(2);
        mem = gather(theta, &|a, z| (a - (c + t * z)).abs() <= w + BOW_MOST * 0.5).into_iter().filter(|p| (p[1] - model(p)).abs() <= w).collect();
        let Some(sol) = (mem.len() >= 100).then(|| solve(&mem, 0.0, if height_spread(&mem) >= LEAN_NEEDS { &[0, 1, 2, 3][..] } else { &[0, 2, 3][..] }, &row)).flatten() else { return Some(best) };
        c = sol[0];
        t = sol[1].clamp(-TILT_MOST, TILT_MOST);
        sl = sol[2];
        b = sol[3];
    }
    if b.abs() >= BOW_FROM && b.abs() <= BOW_MOST && b.abs() <= BOW_RATIO * 2.0 * half && 2.0 * half >= BOW_LEAST_LEN {
        let rss1: f64 = mem.iter().map(|p| (p[1] - (c + t * p[2] + sl * ((p[0] - mid) / half) + b * ((p[0] - mid) / half).powi(2))).powi(2)).sum();
        // What the same points leave across a plane that leans and runs along the line, fitted to them and no bow.
        let rss0 = solve(&mem, 0.0, if height_spread(&mem) >= LEAN_NEEDS { &[0, 1, 2][..] } else { &[0, 2][..] }, &row).map_or(f64::MAX, |q| mem.iter().map(|p| (p[1] - (q[0] + q[1] * p[2] + q[2] * ((p[0] - mid) / half))).powi(2)).sum::<f64>());
        // The bow is there where taking it into the model takes off much more than chance does: the ratio of what it takes off to what is
        // left, per point left (an F ratio); chance alone gives about 1, and a bow of an inch on a wall 4 cm rough over 100 points gives over 100.
        let f_ratio = (rss0 - rss1) / (rss1 / (mem.len() as f64 - 4.0).max(1.0)).max(1e-12);
        if f_ratio >= BOW_F {
            let bowed_rms = (rss1 / mem.len() as f64).sqrt();
            best = Fit { theta, rho: c + b / 3.0, pts: mem, rms: bowed_rms, bow: b.abs(), bow_len: 2.0 * half };
        }
    }
    Some(best)
}

// ---- a large scan: how it is told of, and thinned (call 188, 4) ----

/// The most points the finder reads whole: about 20 seconds on the machine it was built on (the three real scans ran 15,000 to 20,000
/// points a second; one of 197,000 points took 2 to 7 seconds, one of 818,000 took 54). A scan with more is thinned to about this many.
pub const MOST_POINTS: usize = 300_000;
/// A scan of this many points or more says how it is going, line by line.
pub const PROGRESS_FROM: usize = 150_000;

static PROGRESS: std::sync::Mutex<Option<fn(&str)>> = std::sync::Mutex::new(None);

/// Where the finder says how it is going (a line at a time); None says nothing.
pub fn set_progress(f: Option<fn(&str)>) {
    if let Ok(mut g) = PROGRESS.lock() {
        *g = f;
    }
}

pub(crate) fn progress(line: &str) {
    if let Ok(g) = PROGRESS.lock() {
        if let Some(f) = *g {
            f(line);
        }
    }
}

/// One point in `k` of the scan, chosen by a fixed scramble of its place in the file (never every k-th, which a scan made in rows of one length
/// would lose whole walls to): the same points every time. Returns the thinned scan and how many it kept.
pub(crate) fn thin(scan: &crate::scan::Scan) -> (crate::scan::Scan, usize) {
    let k = scan.points.len().div_ceil(MOST_POINTS).max(2) as u64;
    let mix = |mut x: u64| {
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    };
    let points: Vec<[f64; 3]> = scan.points.iter().enumerate().filter(|(i, _)| mix(*i as u64) % k == 0).map(|(_, p)| *p).collect();
    let kept = points.len();
    (crate::scan::Scan { points, lost: scan.lost.clone() }, kept)
}

// ---- which way is up, told by shape (call 188, 3) ----

/// The floor and the ceiling are the two largest flat level things in a room: the largest in AREA, not in points (a wall scanned close and square on shows
/// ten times the points to the foot that the floor does). Along each of the scan's three directions the points are counted in bins of this many feet;
/// a surface square to the direction (a floor, a ceiling, or a wall that happens to face it) piles up in one bin or two, and its area is the squares of
/// `UP_CELL` feet of the other two directions that have points in the pile.
const UP_BIN: f64 = 0.1;
const UP_CELL: f64 = 1.0;
/// A square of a pile has at least this many points (a stray or two are not a surface).
const UP_POINTS: u32 = 3;
/// The two piles are at least this far apart (the least a room is high, as `LEAST_HIGH`) and at most this (`MOST_HIGH`).
const UP_APART: (f64, f64) = (6.0, 15.0);
/// Another direction is the likelier up where its two largest piles cover this many times the area the direction taken as up has in its own two (the
/// walls of a narrow room can out-weigh its floor by a little; one and a half times a floor's does not happen to a room).
const UP_BETTER: f64 = 1.5;

/// The two largest piles of the points along axis `a`: (their area in square feet, how far apart they are): the most area in two groups of three bins
/// of `UP_BIN` that lie at least `UP_APART.0` apart and at most `UP_APART.1`.
fn two_piles(points: &[[f64; 3]], a: usize) -> Option<(f64, f64)> {
    let (area, centres) = two_piles_at(points, a)?;
    // Where each pile is, from the points near it (the middle of them), not from the bins.
    let middle = |c: f64| -> f64 {
        let mut near: Vec<f64> = points.iter().map(|p| p[a]).filter(|x| (x - c).abs() <= 0.4).collect();
        near.sort_by(f64::total_cmp);
        near[near.len() / 2]
    };
    Some((area, (middle(centres.1) - middle(centres.0)).abs()))
}

fn two_piles_at(points: &[[f64; 3]], a: usize) -> Option<(f64, (f64, f64))> {
    let (b, c) = match a {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let (lo, hi) = points.iter().fold((f64::MAX, f64::MIN), |m, p| (m.0.min(p[a]), m.1.max(p[a])));
    if !(hi - lo).is_finite() || hi - lo < UP_APART.0 {
        return None;
    }
    // Per bin: the squares of the other two directions that hold points, counted.
    let mut squares: std::collections::HashMap<(i64, i64, i64), u32> = std::collections::HashMap::new();
    for p in points {
        let k = (((p[a] - lo) / UP_BIN) as i64, (p[b] / UP_CELL).floor() as i64, (p[c] / UP_CELL).floor() as i64);
        *squares.entry(k).or_default() += 1;
    }
    let mut by_bin: std::collections::HashMap<i64, std::collections::HashSet<(i64, i64)>> = std::collections::HashMap::new();
    for ((bin, x, y), n) in &squares {
        if *n >= UP_POINTS {
            by_bin.entry(*bin).or_default().insert((*x, *y));
        }
    }
    // The pile at a bin: the squares held by it and its two neighbours.
    let area = |bin: i64| -> f64 {
        let mut u: std::collections::HashSet<(i64, i64)> = std::collections::HashSet::new();
        for k in bin - 1..=bin + 1 {
            if let Some(s) = by_bin.get(&k) {
                u.extend(s.iter().copied());
            }
        }
        u.len() as f64 * UP_CELL * UP_CELL
    };
    let mut bins: Vec<i64> = by_bin.keys().copied().collect();
    bins.sort_by_key(|k| std::cmp::Reverse(by_bin[k].len()));
    bins.truncate(300);
    let apart = (UP_APART.0 / UP_BIN) as i64;
    let most = (UP_APART.1 / UP_BIN) as i64;
    let mut best: Option<(f64, i64, i64)> = None;
    for (i, x) in bins.iter().enumerate() {
        let ax = area(*x);
        for y in &bins[i + 1..] {
            let d = (y - x).abs();
            if d >= apart && d <= most {
                let t = ax + area(*y);
                if best.is_none_or(|b| t > b.0) {
                    best = Some((t, *x, *y));
                }
            }
        }
    }
    best.map(|(t, x, y)| (t, (lo + (x.min(y) as f64 + 0.5) * UP_BIN, lo + (x.max(y) as f64 + 0.5) * UP_BIN)))
}

/// Whether another direction than the one taken as up fits it better: the sentence to say first when it does (nothing is laid without the
/// person's word), None when the direction taken as up is the likelier or the scan is too small to tell.
pub(crate) fn which_way_up(points: &[[f64; 3]]) -> Option<String> {
    if points.len() < 5_000 {
        return None;
    }
    let stride = (points.len() / 200_000).max(1);
    let sample: Vec<[f64; 3]> = points.iter().step_by(stride).copied().collect();
    // A scan lower than any room has the measure wrong, not the way up: that is said by the size of it.
    let up = two_piles(&sample, 2)?;
    let mut best: Option<(usize, f64, f64)> = None;
    for a in 0..2 {
        if let Some((area, apart)) = two_piles(&sample, a) {
            if best.is_none_or(|b| area > b.1) {
                best = Some((a, area, apart));
            }
        }
    }
    let (a, area, apart) = best?;
    if area < UP_BETTER * up.0.max(1.0) {
        return None;
    }
    let way = if a == 0 { "east and west" } else { "north and south" };
    Some(format!(
        "this is not likely the house: check the measure and which way is up (the two largest flat level things in a room, its floor and its ceiling, lie {} across, facing {way}, and cover {} sq ft; across the direction taken as up the two largest cover {} sq ft: say --up z or --up y, whichever was not taken)",
        crate::found::plumbline_units(apart),
        area.round(),
        up.0.round()
    ))
}
