use super::*;

/// One level of a scan: its floor and the ceiling over it, as heights in the scan (call 183, 8).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScanLevel {
    pub floor: f64,
    pub ceiling: f64,
}

/// The floors a level has between it and the next: a plate (the floor's depth and the ceiling's below it) is no thicker than this, 3 ft 6 in.
pub(super) const MOST_PLATE: f64 = 3.5;
/// An inch of height is a surface where it holds more than a third of what the fullest inch holds (the scan's own measure of a floor or a ceiling).
pub(super) const SURFACE_SHARE: f64 = 3.0;

/// The levels a scan holds, from the lowest: a floor and the ceiling over it (a surface 5 ft or more above it, and not over 15 ft), the next floor
/// within 3 ft 6 in above that ceiling, and its ceiling. Fewer than two complete levels is one level, and the scan's own floor and ceiling are its (call 183, 8).
pub fn levels(scan: &Scan) -> Vec<ScanLevel> {
    let Some(lo) = scan.points.iter().map(|p| p[2]).reduce(f64::min) else { return Vec::new() };
    let hi = scan.points.iter().map(|p| p[2]).fold(lo, f64::max);
    let inch = 1.0 / 12.0;
    let bins = (((hi - lo) / inch).ceil() as usize + 1).min(100_000);
    let mut count = vec![0usize; bins];
    for p in &scan.points {
        count[(((p[2] - lo) / inch).round() as usize).min(bins - 1)] += 1;
    }
    let most = count.iter().copied().max().unwrap_or(0);
    // Surfaces: runs of full inches, 3 in apart or less, each as the height of its fullest inch.
    let mut surfaces: Vec<f64> = Vec::new();
    let mut run: Option<(usize, usize, usize)> = None; // (last full inch, fullest inch, its count)
    for (i, n) in count.iter().enumerate() {
        if *n * SURFACE_SHARE as usize > most && *n > 0 {
            run = Some(match run {
                Some((last, best, c)) if i - last <= 3 => (i, if *n > c { i } else { best }, c.max(*n)),
                Some((_, best, _)) => {
                    surfaces.push(lo + best as f64 * inch);
                    (i, i, *n)
                }
                None => (i, i, *n),
            });
        }
    }
    if let Some((_, best, _)) = run {
        surfaces.push(lo + best as f64 * inch);
    }
    let mut out: Vec<ScanLevel> = Vec::new();
    let mut k = 0;
    while k < surfaces.len() {
        let floor = surfaces[k];
        let Some(c) = (k + 1..surfaces.len()).find(|j| surfaces[*j] - floor >= LEAST_CEILING) else { break };
        let ceiling = surfaces[c];
        if ceiling - floor > MOST_HIGH {
            break;
        }
        out.push(ScanLevel { floor, ceiling });
        // The next level's floor: a surface within a plate above this ceiling.
        match (c + 1..surfaces.len()).find(|j| surfaces[*j] - ceiling <= MOST_PLATE) {
            Some(n) => k = n,
            None => break,
        }
    }
    if out.len() < 2 {
        Vec::new()
    } else {
        out
    }
}

/// Find the walls of a scan that has been levelled by its floor. Where the scan holds more than one level, the lowest is found and the others are said;
/// `find_level` finds another.
pub fn find(scan: &Scan) -> Found {
    find_level(scan, None)
}

/// Find the walls of one level of a scan (call 183, 8): the one whose floor is nearest `wanted` feet above the scan's lowest floor, or the lowest where none
/// is asked for. A scan of one level is found as it is. More than one is said, "the scan holds two levels: floors at 0' 0" and 9' 3"", and the points of the
/// others are not looked at: each level is found alone, so that the walls of one are not found twice and a door of the upper floor is not a window of the lower.
pub fn find_level(scan: &Scan, wanted: Option<f64>) -> Found {
    find_level_by(scan, wanted, Method::Lines)
}

/// Which finder reads a scan (call 189): the first, built on lines voted for by the points the whole height of the room, or the one built on the
/// published method, the points near the ceiling laid on the plan (`found_189`). Everything said and laid after is the same.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Lines,
    Ceiling,
}

/// The walls of a scan by the points near its ceiling (call 189).
pub fn find_by_ceiling(scan: &Scan) -> Found {
    find_level_by(scan, None, Method::Ceiling)
}

/// `find_level`, by the finder named.
pub fn find_level_by(scan: &Scan, wanted: Option<f64>, method: Method) -> Found {
    // A scan too large to be read in good time is thinned first, and that is said (call 188, 4).
    if scan.points.len() > crate::found_188::MOST_POINTS {
        let (thin, kept) = crate::found_188::thin(scan);
        let mut out = find_level_by(&thin, wanted, method);
        out.thinned = Some((scan.points.len(), kept));
        return out;
    }
    // Which way is up, told by shape, before anything is found (call 188, 3): said first, and nothing is laid without the person's word.
    let mut out = find_level_inner(scan, wanted, method);
    if let Some(up) = crate::found_188::which_way_up(&scan.points) {
        // The shape says it first; what the reading of the floor to the ceiling says is kept, as what it read.
        out.not_likely = Some(match out.not_likely.take() {
            Some(read) => match (read.find('('), read.rfind(')')) {
                (Some(a), Some(b)) if b > a => format!("{up}; {}", &read[a + 1..b]),
                _ => format!("{up}; {read}"),
            },
            None => up,
        });
    }
    out
}

pub(super) fn find_level_inner(scan: &Scan, wanted: Option<f64>, method: Method) -> Found {
    if scan.points.len() >= crate::found_188::PROGRESS_FROM {
        crate::found_188::progress(&format!("reading {} points for walls", plumbline_world_figure(scan.points.len())));
    }
    let lv = levels(scan);
    if lv.len() < 2 {
        return find_one(scan, method);
    }
    let base = lv[0].floor;
    let at: Vec<f64> = lv.iter().map(|l| l.floor - base).collect();
    let k = wanted.map_or(0, |h| (0..lv.len()).min_by(|a, b| (at[*a] - h).abs().total_cmp(&(at[*b] - h).abs())).unwrap_or(0));
    let lo_cut = if k == 0 { f64::MIN } else { (lv[k - 1].ceiling + lv[k].floor) / 2.0 };
    let hi_cut = if k + 1 == lv.len() { f64::MAX } else { (lv[k].ceiling + lv[k + 1].floor) / 2.0 };
    let shift = lv[k].floor - base;
    let sub = Scan { points: scan.points.iter().filter(|p| p[2] > lo_cut && p[2] < hi_cut).map(|p| [p[0], p[1], p[2] - shift]).collect(), lost: Vec::new() };
    let mut out = find_one(&sub, method);
    let list = at.iter().map(|h| plumbline_units(*h)).collect::<Vec<_>>();
    let list = match list.as_slice() {
        [a, b] => format!("{a} and {b}"),
        many => format!("{} and {}", many[..many.len() - 1].join(", "), many[many.len() - 1]),
    };
    let asked = match wanted {
        Some(h) if (at[k] - h).abs() > LEVEL_NEAR => format!(", and the level asked for is {} above the lowest floor: the nearest is found", plumbline_units(h)),
        _ => String::new(),
    };
    out.levels = at;
    out.level_found = Some(k);
    out.level_note = Some(format!("the scan holds {} levels: floors at {list}; the level found is the one whose floor is at {}, alone (find another by naming its level){asked}", lv.len(), plumbline_units(out.levels[k])));
    out
}

/// A level asked for is this near (2 ft) a floor of the scan's for that to be the one found without saying so.
pub(super) const LEVEL_NEAR: f64 = 2.0;

/// Set where progress is said (a line at a time), for a scan large enough to take a while; None says nothing (call 188, 4).
pub fn set_progress(f: Option<fn(&str)>) {
    crate::found_188::set_progress(f);
}

/// Find the walls of a scan of one level (levelled by its floor), by how thick the scan is where: where one part is thinner than another by 2 1/2 times or
/// more, each part is read by its own thickness and what each finds is one find, and the parts are said, with where (call 183, 7).
pub(super) fn find_one(scan: &Scan, method: Method) -> Found {
    let core: fn(&Scan) -> Found = match method {
        Method::Lines => find_core,
        Method::Ceiling => crate::found_189::find_core,
    };
    let Some((floor, _)) = scan.floor_and_ceiling().0.map(|f| (f, ())) else { return core(scan) };
    let Some(z) = crate::found_183::zones(scan, floor) else { return core(scan) };
    let (thick_scan, thin_scan) = (crate::found_183::part_of(scan, &z.thick, &z.thin), crate::found_183::part_of(scan, &z.thin, &z.thick));
    let thick = core(&thick_scan);
    let thin = core(&thin_scan);
    let describe = |blocks: &HashSet<(i64, i64)>| {
        let (x0, x1) = (blocks.iter().map(|b| b.0).min().unwrap_or(0), blocks.iter().map(|b| b.0).max().unwrap_or(0) + 1);
        let (y0, y1) = (blocks.iter().map(|b| b.1).min().unwrap_or(0), blocks.iter().map(|b| b.1).max().unwrap_or(0) + 1);
        format!("from {} east, {} north to {} east, {} north", plumbline_units(x0 as f64 * 8.0), plumbline_units(y0 as f64 * 8.0), plumbline_units(x1 as f64 * 8.0), plumbline_units(y1 as f64 * 8.0))
    };
    let mut note = format!("the scan is not as thick everywhere: about {} to the square foot {}, and about {} to the square foot {}; each part is read by its own thickness and what each finds is joined", figure(z.thick_rho), describe(&z.thick), figure(z.thin_rho), describe(&z.thin));
    if let Some(why) = &thin.too_thin {
        note.push_str(&format!("; the thinner part {} is too thin to tell a wall: nothing is found there ({why})", describe(&z.thin)));
    }
    let mut out = crate::found_183::merge(thick, thin, note);
    out.walls.sort_by(|a, b| a.from[0].total_cmp(&b.from[0]).then(a.from[1].total_cmp(&b.from[1])));
    out.open_ends = open_ends(&out.walls);
    out
}
