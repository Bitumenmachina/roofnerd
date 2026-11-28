use super::*;

pub(super) fn find_core(scan: &Scan) -> Found {
    let mut out = Found::default();
    let (floor, scan_ceiling) = scan.floor_and_ceiling();
    out.floor = floor;
    out.ceiling = scan_ceiling;
    let Some(floor) = floor else {
        out.not_likely = unlikely(scan, None);
        out.left_out.push("no floor shows in the scan, so what stands on it cannot be told".into());
        return out;
    };
    // How thick the scan is, and the measures that depend on it (call 179, A1).
    let rho = density(scan, floor);
    out.density = rho;
    if let Some(r) = rho.filter(|r| *r < LEAST_DENSITY) {
        out.not_likely = unlikely(scan, scan_ceiling.map(|c| (c - floor, c - floor)));
        out.too_thin = Some(format!("the scan is too thin to tell a wall from furniture: about {} to the square foot, and it takes {} or more", figure(r), figure(LEAST_DENSITY)));
        out.left_out.push(out.too_thin.clone().unwrap_or_default());
        return out;
    }
    let k = rho.map_or(1.0, |r| (REFERENCE_DENSITY / r).sqrt().max(1.0));
    let (sq, cell) = (SQUARE * k, CELL * k);
    if rho.is_some_and(|r| r < THIN_NOTE_BELOW * REFERENCE_DENSITY) {
        out.thin_note = Some(format!("this scan is thinner than the measures here were made for ({} to the square foot, against {} and more): a wall is told to a few inches, an opening only to about {} in, and one may be missed or split: check each", figure(rho.unwrap_or(0.0)), figure(REFERENCE_DENSITY), (sq * 12.0).round()));
    }
    // The ceiling over the plan: one height, two, or a slope (call 179, A4).
    let (ceil, ceil_note) = ceiling_of(scan, floor, scan_ceiling, rho, CEIL_CELL * k);
    out.ceiling_note = ceil_note;
    out.not_likely = unlikely(scan, ceil.as_ref().map(|c| (c.lowest, c.highest)));
    let Some(ceil) = ceil else {
        out.no_ceiling = true;
        out.left_out.push("no ceiling shows in the scan: without one, a wall cannot be told from a tall cabinet standing in the room, so no wall is taken".into());
        return out;
    };
    if scan_ceiling.is_none() || ceil.flat.is_none() {
        out.ceiling = Some(floor + ceil.middle);
    }
    let high = ceil.highest - SET_ASIDE;
    let min_votes = ((f64::from(MIN_VOTES) / k).round() as u32).max(MOST_THIN_VOTES);

    // The points that can be a wall's: [east, north, up above the floor].
    let band: Vec<[f64; 3]> = scan.points.iter().filter_map(|p| {
        let z = p[2] - floor;
        (z >= SET_ASIDE && z <= ceil.at([p[0], p[1]]) - SET_ASIDE).then_some([p[0], p[1], z])
    }).collect();
    let key = |x: f64, y: f64| ((x / cell).floor() as i64, (y / cell).floor() as i64);
    // Upright cells: points at several heights in one cell of plan.
    struct Cell {
        n: u32,
        heights: u64,
        /// The points in the top band (1 ft 6 in under the ceiling, less the 3 in set aside), and the
        /// quarter-foot tiers of height they are at: what shows of a wall above what stands before it.
        top: u32,
        top_tiers: u64,
        all: u32,
        sx: f64,
        sy: f64,
    }
    let mut cells: HashMap<(i64, i64), Cell> = HashMap::new();
    for p in &band {
        let here = ceil.at([p[0], p[1]]);
        let mid = p[2] >= 0.5 && p[2] <= here - 0.5;
        let top = p[2] >= here - TOP_BAND;
        if !mid && !top {
            continue;
        }
        let c = cells.entry(key(p[0], p[1])).or_insert(Cell { n: 0, heights: 0, top: 0, top_tiers: 0, all: 0, sx: 0.0, sy: 0.0 });
        if mid {
            c.n += 1;
            c.heights |= 1u64 << (((p[2] - 0.5) / 0.5) as u32).min(63);
        }
        if top {
            c.top += 1;
            c.top_tiers |= 1u64 << (((p[2] - (here - TOP_BAND)) / TOP_TIER) as u32).min(63);
        }
        c.all += 1;
        c.sx += p[0];
        c.sy += p[1];
    }
    let upright: HashSet<(i64, i64)> = cells.iter().filter(|(_, c)| (c.n >= CELL_POINTS && c.heights.count_ones() >= UPRIGHT_HEIGHTS) || (c.top >= CELL_POINTS && c.top_tiers.count_ones() >= TOP_TIERS)).map(|(k, _)| *k).collect();
    let mut votes: Vec<P2> = cells.iter().filter(|(k, _)| upright.contains(*k)).map(|(_, c)| [c.sx / f64::from(c.all), c.sy / f64::from(c.all)]).collect();
    votes.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    let pool: Vec<P2> = band.iter().filter(|p| upright.contains(&key(p[0], p[1]))).map(|p| [p[0], p[1]]).collect();
    // Where the scan saw the floor: cells a foot square with floor points in them.
    let mut floor_cells: HashMap<(i64, i64), u32> = HashMap::new();
    for p in &scan.points {
        if (p[2] - floor).abs() <= SET_ASIDE {
            *floor_cells.entry(((p[0] / FLOOR_CELL).floor() as i64, (p[1] / FLOOR_CELL).floor() as i64)).or_default() += 1;
        }
    }
    // The cells that are floor and not the foot of a wall (which holds points near the floor too, but a fraction of what a floor does):
    // those holding at least FLOOR_LIKE of what the middling floor cell holds (call 183, 2).
    let floor_dense: Vec<P2> = {
        let m = median(floor_cells.values().map(|n| f64::from(*n)).collect());
        floor_cells.iter().filter(|(_, n)| f64::from(**n) >= (FLOOR_LIKE * m).max(3.0)).map(|((a, b), _)| [(*a as f64 + 0.5) * FLOOR_CELL, (*b as f64 + 0.5) * FLOOR_CELL]).collect()
    };
    let floor_cells: Vec<P2> = floor_cells.into_iter().filter(|(_, n)| *n >= 3).map(|((a, b), _)| [(a as f64 + 0.5) * FLOOR_CELL, (b as f64 + 0.5) * FLOOR_CELL]).collect();

    // ---- lines, strongest first; each one's stretches are faces ----
    let steps = (180.0 / VOTE_DEGREES) as usize;
    let (cos, sin): (Vec<f64>, Vec<f64>) = (0..steps).map(|k| { let n = normal((k as f64 * VOTE_DEGREES).to_radians()); (n[0], n[1]) }).unzip();
    // How rough this scan's walls are, from its strongest lines (call 188, 1): the band a face's points lie in, and the reach of what is
    // taken off the vote, are as much wider than the made-up scans' as its walls scatter more.
    let (scatter, kr) = {
        let mut pv = votes.clone();
        let mut lines: Vec<(f64, f64)> = Vec::new();
        for _ in 0..PILOT_LINES {
            let Some((k, r0, _)) = strongest(&pv, &cos, &sin, min_votes) else { break };
            let (t0, r0) = seed_line(&pool, (k as f64 * VOTE_DEGREES).to_radians(), r0, SEED_BAND);
            let n0 = normal(t0);
            pv.retain(|p| (dot(*p, n0) - r0).abs() > SEED_BAND);
            lines.push((t0, r0));
        }
        let walls_say = crate::found_188::wall_scatter(&band, &lines, PILOT_WINDOW);
        let floor_says = crate::found_188::floor_scatter(&scan.points, floor, FLOOR_REACH);
        let sc = match (walls_say, floor_says) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        (sc, crate::found_188::band_scale(sc))
    };
    let mut wide_used = false;
    // A face is first fitted in the band the made-up scans were (so that two faces a thin wall apart are two); only where it is not straight
    // so, is it fitted again in the band this scan's own roughness asks (call 188, 1).
    let (tol, tol_wide, seed_band) = (BAND, BAND * kr, SEED_BAND);
    let mut faces: Vec<Face> = Vec::new();
    // How closely the points of a face keep to its line, as the best faces found so far keep (call 179, A5).
    let mut ref_rms: Option<f64> = None;
    let mut left: Vec<Left> = Vec::new();
    let mut bits = 0usize;
    let big = scan.points.len() >= crate::found_188::PROGRESS_FROM;
    for line_no in 0..MOST_LINES {
        if big && line_no % 10 == 0 {
            crate::found_188::progress(&format!("looking for walls: line {} of at most {MOST_LINES}, {} faces so far", line_no + 1, faces.len()));
        }
        let Some((k, rho0, _)) = strongest(&votes, &cos, &sin, min_votes) else { break };
        // The vote's line, as coarse as the vote: the points within SEED_BAND of it, all
        // heights, along it. Its stretches are looked at one at a time: two things on one
        // line with a gap between them (a sofa and a cabinet) are two, and each is fitted by
        // itself, so that a line through both does not lean across the room.
        // The vote told the line to half a degree and its place to 0.3 ft (three bins are added so that a line
        // between two is not split): two faces nearer than that to one another, a wall 3 1/2 in thick, are one
        // peak, and a line fitted to the points of both leans across them and then across the next (call 179,
        // A5). The line is told again from the points themselves, its direction from all that lie near it and its
        // place from the fullest quarter-inch of them: one face, the fuller; the other is the next line's.
        let (theta0, rho0) = seed_line(&pool, (k as f64 * VOTE_DEGREES).to_radians(), rho0, seed_band);
        let (d0, n0) = (dir(theta0), normal(theta0));
        let mut on: Vec<f64> = band.iter().filter(|p| (dot([p[0], p[1]], n0) - rho0).abs() <= seed_band && !crosses_face([p[0], p[1]], theta0, &faces)).map(|p| dot([p[0], p[1]], d0)).collect();
        on.sort_by(f64::total_cmp);
        let mut taken_out: Vec<(f64, f64)> = Vec::new();
        for (sa, sb) in runs_of(&on, sq) {
            taken_out.push((sa, sb));
            if sb - sa < MIN_FACE {
                if sb - sa >= SHORT_FACE {
                    left.push(Left { from: [sa * d0[0] + rho0 * n0[0], sa * d0[1] + rho0 * n0[1]], to: [sb * d0[0] + rho0 * n0[0], sb * d0[1] + rho0 * n0[1]], why: format!("under {MIN_FACE} ft long (a wall is at least {MIN_FACE} ft long)") });
                } else {
                    bits += 1;
                }
                continue;
            }
            // Fitted to the upright points of the stretch, nearer each time.
            let (mut theta, mut rho) = (theta0, rho0);
            for _ in 0..3 {
                let (d, n) = (dir(theta), normal(theta));
                let near: Vec<P2> = pool.iter().copied().filter(|p| (dot(*p, n) - rho).abs() <= if theta == theta0 { seed_band } else { tol } && dot(*p, d) >= sa - 0.2 && dot(*p, d) <= sb + 0.2 && !crosses_face(*p, theta, &faces)).collect();
                let Some((t, m)) = long_way(&near) else { break };
                // `long_way` gives an angle within 0 to pi; keep the one nearest the vote's.
                theta = if (t - theta0).abs() > std::f64::consts::FRAC_PI_2 { if t > theta0 { t - std::f64::consts::PI } else { t + std::f64::consts::PI } } else { t };
                rho = dot(m, normal(theta));
            }
            // The face is fitted as the wall stands in a real scan (call 188, 1): the points within the band of a plane that leans a
            // little and may bow, the band as wide as the scan's own scatter says.
            let skip = |q: P2, t: f64| crosses_face(q, t, &faces);
            let Some(fit) = crate::found_188::fit_face(&band, theta, rho, sa, sb, tol, sb - sa >= crate::found_188::FREE_FROM, &skip) else { continue };
            let (mut theta, mut rho, mut pts, mut rms) = (fit.theta, fit.rho, fit.pts.clone(), fit.rms);
            let mut used = tol;
            let mut bow = (fit.bow, fit.bow_len);
            // A face is fitted along its whole length or it is not found (call 179, A5): its points keep to
            // its line as closely as the other faces' keep to theirs. A line that leans across two walls near
            // one another (a closet's, 3 1/2 in thick and a few feet long) has points all across its band. It is
            // tried again square to a face already found, the way the walls of a house lie; if its points do not
            // keep to that either, it is not a face, and that is said.
            if let Some(r) = ref_rms.filter(|_| pts.len() >= 20) {
                if rms > STRAIGHT * r {
                    let square: Option<f64> = faces.iter().flat_map(|f| [f.theta, f.theta + std::f64::consts::FRAC_PI_2, f.theta - std::f64::consts::FRAC_PI_2]).filter(|t| (t - theta).abs().to_degrees() <= LEAN_MOST).min_by(|a, b| (a - theta).abs().total_cmp(&(b - theta).abs()));
                    let again = square.and_then(|t| {
                        let n = normal(t);
                        let d = dir(t);
                        let offs: Vec<f64> = pool.iter().filter(|p| dot(**p, d) >= sa - 0.2 && dot(**p, d) <= sb + 0.2 && (dot(**p, n) - rho).abs() <= 0.4).map(|p| dot(*p, n)).collect();
                        let best = offs.iter().copied().max_by_key(|o| offs.iter().filter(|q| (**q - *o).abs() <= 0.075).count())?;
                        let near: Vec<f64> = offs.into_iter().filter(|o| (o - best).abs() <= 0.075).collect();
                        let r2 = near.iter().sum::<f64>() / near.len() as f64;
                        let f2 = crate::found_188::fit_face(&band, t, r2, sa, sb, tol, false, &skip)?;
                        (f2.pts.len() >= 20 && f2.rms <= STRAIGHT * r).then_some(f2)
                    });
                    // Rougher than the made-up scans: the same line in the wider band, held to as many times the bar (call 188, 1).
                    let again = again.map(|f| (f, tol)).or_else(|| {
                        if kr <= 1.0 {
                            return None;
                        }
                        let f3 = crate::found_188::fit_face(&band, theta, rho, sa, sb, tol_wide, true, &skip)?;
                        (f3.pts.len() >= 20 && f3.rms <= STRAIGHT * r * kr).then_some((f3, tol_wide))
                    });
                    match again {
                        Some((f2, band_used)) => {
                            theta = f2.theta;
                            rho = f2.rho;
                            rms = f2.rms;
                            bow = (f2.bow, f2.bow_len);
                            pts = f2.pts;
                            used = band_used;
                            wide_used |= band_used > tol;
                        }
                        None => {
                            left.push(Left { from: [sa * d0[0] + rho0 * n0[0], sa * d0[1] + rho0 * n0[1]], to: [sb * d0[0] + rho0 * n0[0], sb * d0[1] + rho0 * n0[1]], why: "not straight along the whole length (furniture, or two walls close together)".into() });
                            continue;
                        }
                    }
                }
            }
            let (d, n) = (dir(theta), normal(theta));
            if pts.len() < 20 {
                continue;
            }
            // Offset against height: the line's place at the floor (z = 0), unless the face
            // leans so far that it is not a plumb face (more than 2 degrees), when its middle.
            let mean_d = pts.iter().map(|p| p[1]).sum::<f64>() / pts.len() as f64;
            let mean_z = pts.iter().map(|p| p[2]).sum::<f64>() / pts.len() as f64;
            let (mut szz, mut szd) = (0.0, 0.0);
            for p in &pts {
                szz += (p[2] - mean_z) * (p[2] - mean_z);
                szd += (p[2] - mean_z) * (p[1] - mean_d);
            }
            let slope = if szz > 1e-9 { szd / szz } else { 0.0 };
            let at_floor = if slope.abs() <= crate::found_188::TILT_MOST { mean_d - slope * mean_z } else { mean_d };
            pts.sort_by(|p, q| p[0].total_cmp(&q[0]));
            // Its ends, to the point, as an edge (what is beyond is empty or another wall).
            let s: Vec<f64> = pts.iter().map(|p| p[0]).collect();
            let (first_s, last_s) = (s[0], s[s.len() - 1]);
            let s0 = edge(&s, first_s + EDGE_BACK, true, last_s - first_s - EDGE_BACK, f64::MIN).unwrap_or(first_s);
            let s1 = edge(&s, last_s - EDGE_BACK, false, last_s - first_s - EDGE_BACK, f64::MAX).unwrap_or(last_s);
            if s1 - s0 < MIN_FACE {
                if s1 - s0 >= SHORT_FACE {
                    left.push(Left { from: [s0 * d[0] + rho * n[0], s0 * d[1] + rho * n[1]], to: [s1 * d[0] + rho * n[0], s1 * d[1] + rho * n[1]], why: format!("under {MIN_FACE} ft long (a wall is at least {MIN_FACE} ft long)") });
                }
                continue;
            }
            // A face stands: its points are at several heights (a table top seen edge on is a
            // line of points all at one).
            let mut tiers: HashMap<usize, usize> = HashMap::new();
            for p in &pts {
                *tiers.entry((p[2] / 0.5) as usize).or_default() += 1;
            }
            let mut top_tiers: HashMap<usize, usize> = HashMap::new();
            let here = ceil.over([(sa + sb) / 2.0 * d[0] + rho * n[0], (sa + sb) / 2.0 * d[1] + rho * n[1]], n);
            for p in pts.iter().filter(|p| p[2] >= here - TOP_BAND) {
                *top_tiers.entry(((p[2] - (here - TOP_BAND)) / TOP_TIER) as usize).or_default() += 1;
            }
            if tiers.values().filter(|n| **n >= 10).count() < UPRIGHT_HEIGHTS as usize && top_tiers.values().filter(|n| **n >= 10).count() < TOP_TIERS as usize {
                continue;
            }
            // The same face found twice (a second vote that fell on the first's edge): once.
            let twice = faces.iter().any(|f| {
                let apart = (f.theta - theta).abs();
                let across = (dot(f.at(f.mid()), n) - rho).abs();
                let along = f.s1.min(s1) - f.s0.max(s0);
                apart.to_degrees() < 2.0 && across < used && along > 0.5 * (s1 - s0).min(f.s1 - f.s0)
            });
            if twice {
                continue;
            }
            if pts.len() >= 500 {
                ref_rms = Some(ref_rms.map_or(rms, |q: f64| q.min(rms)));
            }
            let mut face = Face { theta, rho: at_floor, s0, s1, pts, wall: false, squared: false, seen: [false, false], height: 0.0, tall: false, floor_cols: Vec::new(), ceil_cols: Vec::new(), partial: false, hung: false, cols_from: 0.0, ceil: 0.0, sq, seen_area: 0.0, tol: used, bow, joined: 0, gone: false, seen_sure: false };
            classify(&mut face, &ceil, sq);
            faces.push(face);
            // Its cells are out of the next vote.
            // (as far as the seed band: cells at a corner are drawn a little off their line by the wall that crosses it)
            votes.retain(|p| !((dot(*p, n) - rho).abs() <= seed_band && dot(*p, d) >= sa - 0.3 && dot(*p, d) <= sb + 0.3));
        }
        // Whatever the vote's own cells were, they are out of the next vote: the ones near
        // its line, where its stretches ran.
        let before = votes.len();
        votes.retain(|p| !((dot(*p, n0) - rho0).abs() <= seed_band && taken_out.iter().any(|(a, b)| dot(*p, d0) >= a - 0.3 && dot(*p, d0) <= b + 0.3)));
        if votes.len() == before {
            votes.retain(|p| (dot(*p, n0) - rho0).abs() > seed_band);
        }
    }

    if big {
        crate::found_188::progress(&format!("{} faces found; telling which are walls", faces.len()));
    }
    out.scatter = if wide_used { scatter } else { None };
    // Every stretch left out is said, with where it is and how long (call 183, 4).
    left.sort_by(|a, b| a.from[0].total_cmp(&b.from[0]).then(a.from[1].total_cmp(&b.from[1])));
    left.dedup_by(|a, b| (a.from[0] - b.from[0]).abs() < 0.5 && (a.from[1] - b.from[1]).abs() < 0.5 && (a.to[0] - b.to[0]).abs() < 0.5 && (a.to[1] - b.to[1]).abs() < 0.5);
    for l in &left {
        out.left_out.push(format!("a stretch of upright points {} long, from {} east, {} north to {} east, {} north, not taken as walls: {}", plumbline_units((l.to[0] - l.from[0]).hypot(l.to[1] - l.from[1])), plumbline_units(l.from[0]), plumbline_units(l.from[1]), plumbline_units(l.to[0]), plumbline_units(l.to[1]), l.why));
    }
    if bits > 0 {
        out.left_out.push(format!("{} of upright points, each under {SHORT_FACE} ft long, not taken as walls (too short to place)", plumbline_units_count(bits, "stretch", "stretches")));
    }
    out.left = left;
    // A line drawn through the points of other things (a sofa's front and back, a table's
    // corner) has a few points of its own: a face with under a quarter of the usual
    // number of points to the square foot of face is not one.
    // (a face that reaches the ceiling and the floor over half its length is measured over its whole face,
    // one that does not over its length and its height)
    let floor_reach = |f: &Face| f.floor_cols.iter().filter(|x| **x).count() as f64 / f.floor_cols.len().max(1) as f64;
    let per_foot: Vec<f64> = faces.iter().map(|f| f.pts.len() as f64 / ((f.s1 - f.s0) * if f.tall && floor_reach(f) >= WALL_REACHES_FLOOR { f.ceil } else { f.height.max(0.5) })).collect();
    // The usual is what a face of the scan shows, but not more than the floor shows to the square foot (call 188, 1): the walls of a phone's
    // scan are seen at 10 times the density of one another (near and square on, or far and at a slant), and the median of three walls
    // is then four times what the fourth shows.
    let usual = median(per_foot.clone());
    let usual = rho.map_or(usual, |r| usual.min(r));
    // A face that reaches the ceiling is also kept if what the scan shows of it is dense: a wall seen only
    // above what stands before it has few points to its whole face and as many to what it shows as any (call
    // 179, A2). A face that stops short is held to the whole of what it spans: a line through other things'
    // points shows little and spans much.
    let keep: Vec<bool> = faces.iter().zip(&per_foot).map(|(f, d)| *d >= usual / SPARSE || (f.tall && f.pts.len() as f64 / f.seen_area.max(sq * sq) >= usual / SPARSE)).collect();
    let mut keep = keep.into_iter();
    faces.retain(|_| keep.next().unwrap_or(true));
    // Each face's ends again, now that every face is known: where another face crosses it, that face's
    // points near the corner are not this face's (call 179, A2: a wall seen only above what stands before
    // it has few points of its own, and a corner's worth of the crossing wall's would carry its end far past
    // the corner). Its points are taken off, and its ends and its reach to ceiling and floor looked at again.
    for i in 0..faces.len() {
        let f = &faces[i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        let kept: Vec<[f64; 3]> = f.pts.iter().copied().filter(|p| !crosses_other([p[0] * d[0] + p[1] * n[0], p[0] * d[1] + p[1] * n[1]], f.theta, &faces, i)).collect();
        if kept.len() < 20 || kept.len() == f.pts.len() {
            continue;
        }
        let mut s: Vec<f64> = kept.iter().map(|p| p[0]).collect();
        s.sort_by(f64::total_cmp);
        // From where points run on, not from a stray one a little way off the end (a column of 3 in needs three).
        let runs = runs_of(&s, sq);
        let (Some(first), Some(last)) = (runs.first(), runs.last()) else { continue };
        let (first_s, last_s) = (s.iter().copied().find(|x| *x >= first.0).unwrap_or(s[0]), s.iter().rev().copied().find(|x| *x <= last.1).unwrap_or(s[s.len() - 1]));
        let s0 = edge(&s, first_s + EDGE_BACK, true, last_s - first_s - EDGE_BACK, f64::MIN).unwrap_or(first_s);
        let s1 = edge(&s, last_s - EDGE_BACK, false, last_s - first_s - EDGE_BACK, f64::MAX).unwrap_or(last_s);
        if s1 - s0 < MIN_FACE {
            continue;
        }
        let f = &mut faces[i];
        f.s0 = s0;
        f.s1 = s1;
        classify(f, &ceil, sq);
    }
    // ---- square to the scan's main directions ----
    let wall_ids: Vec<usize> = (0..faces.len()).filter(|i| faces[*i].wall).collect();
    out.squared = (0, wall_ids.len());
    if !wall_ids.is_empty() {
        // The main direction: the length-weighted mean of the faces' directions taken four
        // times round (so that a face and the one square to it count together).
        let (mut c, mut s) = (0.0, 0.0);
        for i in &wall_ids {
            let w = faces[*i].s1 - faces[*i].s0;
            c += w * (4.0 * faces[*i].theta).cos();
            s += w * (4.0 * faces[*i].theta).sin();
        }
        let main = (s.atan2(c) / 4.0).rem_euclid(std::f64::consts::FRAC_PI_2);
        out.main = Some(main.to_degrees());
        for i in &wall_ids {
            let f = &mut faces[*i];
            // Nearest of main, main + 90, main + 180, ...
            let quarter = std::f64::consts::FRAC_PI_2;
            let best = main + ((f.theta - main) / quarter).round() * quarter;
            let off = (f.theta - best).abs();
            if off.to_degrees() <= SQUARE_DEGREES {
                // Turned about its middle: the same line through the same middle.
                let mid = f.at(f.mid());
                let shift_from = dot(mid, dir(f.theta));
                f.theta = best;
                f.rho = dot(mid, normal(best));
                let shift = dot(mid, dir(best)) - shift_from;
                f.s0 += shift;
                f.s1 += shift;
                for p in &mut f.pts {
                    p[0] += shift;
                }
                f.squared = true;
                out.squared.0 += 1;
            }
        }
    }

    // ---- a wall face's end that comes up to another wall's face is carried to it ----
    // (the faces found run a little past, or a little short of, where they meet, by what
    // the crossing wall's own points did to them)
    let lines: Vec<(f64, f64, f64, f64)> = wall_ids.iter().map(|i| (faces[*i].theta, faces[*i].rho, faces[*i].s0, faces[*i].s1)).collect();
    for i in &wall_ids {
        let f = &mut faces[*i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        for end in 0..2 {
            let here = if end == 0 { f.s0 } else { f.s1 };
            let mut best: Option<(f64, f64)> = None;
            for (gt, gr, g0, g1) in &lines {
                let cross = (f.theta - gt).sin();
                if cross.abs() < NOT_PARALLEL {
                    continue;
                }
                // Where this line meets the other's: s along this one with s d + rho n on gr = p . n_g.
                let (gd, gn) = (dir(*gt), normal(*gt));
                let denom = dot(d, gn);
                if denom.abs() < 1e-9 {
                    continue;
                }
                let s = (gr - f.rho * dot(n, gn)) / denom;
                let q = [s * d[0] + f.rho * n[0], s * d[1] + f.rho * n[1]];
                let along_g = dot(q, gd);
                if (s - here).abs() <= SNAP && along_g >= g0 - SNAP && along_g <= g1 + SNAP && best.is_none_or(|b| (s - here).abs() < (b.0 - here).abs()) {
                    best = Some((s, 0.0));
                }
            }
            if let Some((s, _)) = best {
                if end == 0 { f.s0 = s } else { f.s1 = s }
            }
        }
    }

    // ---- which side the scan saw each wall face from ----
    for i in &wall_ids {
        let f = &mut faces[*i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        let mut sides = [0usize; 2];
        for c in &floor_cells {
            let (s, a) = (dot(*c, d), dot(*c, n) - f.rho);
            if s >= f.s0 && s <= f.s1 && a.abs() > FLOOR_CELL && a.abs() <= SEEN_REACH {
                sides[usize::from(a < 0.0)] += 1;
            }
        }
        f.seen = [sides[0] >= SEEN_FLOOR_CELLS && sides[0] * SEEN_SHARE >= sides[1], sides[1] >= SEEN_FLOOR_CELLS && sides[1] * SEEN_SHARE >= sides[0]];
        f.seen_sure = f.seen[0] != f.seen[1];
        if !f.seen[0] && !f.seen[1] {
            f.seen = [sides[0] >= sides[1], sides[0] < sides[1]];
        }
    }

    // ---- a wall scanned in passes is one face (call 188, 2) ----
    // Faces that run alongside one another within a wall's thickness and were seen from the same side are the layers of one wall, not
    // two walls (the faces of one wall seen from both sides face away from each other, and are paired below, as they were).
    let mut wall_ids = wall_ids;
    {
        let mut again = true;
        while again {
            again = false;
            'passes: for x in 0..wall_ids.len() {
                for y in x + 1..wall_ids.len() {
                    let (a, b) = (wall_ids[x], wall_ids[y]);
                    let (fa, fb) = (&faces[a], &faces[b]);
                    let (na, nb) = (normal(fa.theta), normal(fb.theta));
                    let way = |f: &Face, n: P2| if f.seen[0] == f.seen[1] { None } else if f.seen[0] { Some(n) } else { Some([-n[0], -n[1]]) };
                    if !fa.seen_sure || !fb.seen_sure {
                        continue;
                    }
                    let (Some(wa), Some(wb)) = (way(fa, na), way(fb, nb)) else { continue };
                    let mut dt = (fa.theta - fb.theta).abs();
                    dt = dt.min(std::f64::consts::PI - dt);
                    if dt.to_degrees() > PAIR_DEGREES || dot(wa, wb) <= 0.0 {
                        continue;
                    }
                    let across = (dot(fb.at(fb.mid()), na) - fa.rho).abs();
                    let (b0, b1) = along_span(fb, dir(fa.theta));
                    let overlap = fa.s1.min(b1) - fa.s0.max(b0);
                    if across > PASS_APART || overlap < PASS_ALONGSIDE * (fa.s1 - fa.s0).min(fb.s1 - fb.s0) {
                        continue;
                    }
                    // The one with more points keeps; the other is joined into it.
                    let (k, o) = if fa.pts.len() >= fb.pts.len() { (a, b) } else { (b, a) };
                    let (d, n) = (dir(faces[k].theta), normal(faces[k].theta));
                    let (o0, o1) = along_span(&faces[o], d);
                    let (do_, no_) = (dir(faces[o].theta), normal(faces[o].theta));
                    let moved: Vec<[f64; 3]> = faces[o].pts.iter().map(|p| { let q = [p[0] * do_[0] + p[1] * no_[0], p[0] * do_[1] + p[1] * no_[1]]; [dot(q, d), dot(q, n), p[2]] }).collect();
                    let more = faces[o].joined + 1;
                    faces[o].gone = true;
                    faces[o].wall = false;
                    let fk = &mut faces[k];
                    fk.pts.extend(moved);
                    fk.s0 = fk.s0.min(o0);
                    fk.s1 = fk.s1.max(o1);
                    fk.joined += more;
                    classify(fk, &ceil, sq);
                    wall_ids.retain(|j| *j != o);
                    if !faces[k].wall {
                        wall_ids.retain(|j| *j != k);
                    }
                    again = true;
                    break 'passes;
                }
            }
        }
    }

    // ---- a face that reaches the ceiling and is not shown to be a wall (call 179, A3) ----
    let mut doubts = tall_at_a_wall(&faces, &wall_ids);
    for (k, v) in holed(&faces, &wall_ids, &band, rho.unwrap_or(REFERENCE_DENSITY)) {
        doubts.entry(k).or_insert(v);
    }

    // ---- curved faces (call 183, 5): said and not laid; the chords the finder took of one are in doubt ----
    out.curves = curved(&pool, &faces, &wall_ids, &doubts);
    for c in &out.curves {
        out.left_out.push(format!("a curved face of radius {}, about {} long, from {} east, {} north to {} east, {} north: curved walls are not found, so it is not laid", plumbline_units(c.radius), plumbline_units(c.length), plumbline_units(c.from[0]), plumbline_units(c.from[1]), plumbline_units(c.to[0]), plumbline_units(c.to[1])));
        for &i in &wall_ids {
            let f = &faces[i];
            // On the curve: near its circle (within a wall's thickness and the fit's own error) and within the arc it runs over (and a little more).
            let on = |p: P2| {
                let mut a = (p[1] - c.centre[1]).atan2(p[0] - c.centre[0]) - c.mid;
                while a > std::f64::consts::PI { a -= std::f64::consts::TAU; }
                while a < -std::f64::consts::PI { a += std::f64::consts::TAU; }
                ((p[0] - c.centre[0]).hypot(p[1] - c.centre[1]) - c.radius).abs() <= CURVE_ON && a >= c.span.0 - CURVE_ARC_SLACK && a <= c.span.1 + CURVE_ARC_SLACK
            };
            if on(f.at(f.s0)) && on(f.at(f.s1)) && on(f.at(f.mid())) {
                doubts.insert(i, format!("part of a curved face of radius {}: curved walls are not found, so a chord of one is not laid", plumbline_units(c.radius)));
            }
        }
    }
    // ---- what a wall is hidden by: the ceiling or the floor not reached where something stands before it ----
    // (call 179, A2) Where something stands: the cells of the plan the points of what stops short, and of
    // what is in doubt, are in. A wall that something tall stands before is not "stood short", nor is one that
    // something low stands before: where it is not seen at the floor or the ceiling and something stands within
    // 3 ft before it, on the side the scan saw it from, that is why, and the column counts as reached.
    let stand_of = |ids: &[usize], faces: &[Face]| -> Vec<P2> {
        let mut stands: HashSet<(i64, i64)> = HashSet::new();
        let mut stand_at: Vec<P2> = Vec::new();
        for i in ids {
            let f = &faces[*i];
            let (d, n) = (dir(f.theta), normal(f.theta));
            for p in &f.pts {
                let q = [p[0] * d[0] + p[1] * n[0], p[0] * d[1] + p[1] * n[1]];
                if stands.insert(((q[0] / 0.25).floor() as i64, (q[1] / 0.25).floor() as i64)) {
                    stand_at.push(q);
                }
            }
        }
        stand_at
    };
    let stoppers: Vec<usize> = (0..faces.len()).filter(|i| !faces[*i].gone && (!faces[*i].wall || doubts.contains_key(i))).collect();
    let stand_at = stand_of(&stoppers, &faces);
    let mut wall_ids = wall_ids;
    let mut demoted: Vec<(usize, bool)> = Vec::new();
    for i in wall_ids.clone() {
        if doubts.contains_key(&i) {
            continue;
        }
        let f = &faces[i];
        let (d, n) = (dir(f.theta), normal(f.theta));
        // Which sides something may stand on to hide it: the side the scan saw it from, or either.
        let sides: Vec<f64> = if f.seen[0] != f.seen[1] { vec![if f.seen[0] { 1.0 } else { -1.0 }] } else { vec![1.0, -1.0] };
        let hidden_column = |c: usize| {
            let (a, b) = (f.cols_from + c as f64 * f.sq, f.cols_from + (c + 1) as f64 * f.sq);
            stand_at.iter().any(|q| {
                let (s, across) = (dot(*q, d), dot(*q, n) - f.rho);
                s >= a && s <= b && sides.iter().any(|side| across * side > IN_FRONT_LEAST && across * side <= IN_FRONT)
            })
        };
        let hidden: Vec<bool> = (0..f.floor_cols.len()).map(hidden_column).collect();
        // A column inside the body of a wall that crosses this one is that wall's, not this one's: it is judged neither way (call
        // 183, 4: a short wall's ends are all corner, and the points taken off there are not a lack of wall).
        let corner: Vec<bool> = (0..f.floor_cols.len())
            .map(|c| {
                let q = f.at(f.cols_from + (c as f64 + 0.5) * f.sq);
                wall_ids.iter().filter(|j| **j != i).any(|&j| {
                    let g = &faces[j];
                    (f.theta - g.theta).sin().abs() >= CROSSES && (dot(q, normal(g.theta)) - g.rho).abs() <= CORNER_BODY && dot(q, dir(g.theta)) >= g.s0 - SNAP && dot(q, dir(g.theta)) <= g.s1 + SNAP
                })
            })
            .collect();
        let cols = (f.floor_cols.len() - corner.iter().filter(|x| **x).count()).max(1) as f64;
        let up = (0..f.ceil_cols.len()).filter(|c| !corner[*c] && (f.ceil_cols[*c] || hidden[*c])).count() as f64;
        let mut reach: Vec<bool> = (0..f.floor_cols.len()).map(|c| corner[c] || f.floor_cols[c] || hidden[c]).collect();
        // Openings: stretches at least 1 ft 6 in long that do not reach the floor but are closed over their whole length by a head that
        // reaches the ceiling, with a piece that reaches the floor at each end of the wall (its jambs): a wall with a door in it, or
        // a wall that is mostly opening (a garage's). A face that reaches the floor nowhere, or at one end only, is not one (call 183, 4 and 5).
        let wide = (MIN_OPENING / f.sq).round() as usize;
        let jamb = (END_PIECE / f.sq).ceil() as usize;
        {
            let n_cols = reach.len();
            let (mut c, mut runs) = (0usize, Vec::new());
            while c < n_cols {
                if !reach[c] {
                    let b = c;
                    while c < n_cols && !reach[c] {
                        c += 1;
                    }
                    runs.push((b, c));
                } else {
                    c += 1;
                }
            }
            // A piece reaches the floor at each end: of the columns at an end (twice a jamb's), at least a jamb's worth do.
            let at_end = |range: std::ops::Range<usize>| range.clone().filter(|c| reach[*c]).count() >= jamb;
            let jambs_at_ends = n_cols >= 4 * jamb && at_end(0..2 * jamb) && at_end(n_cols - 2 * jamb..n_cols);
            // A stretch is an opening where it is wide enough and closed over by a head that reaches the ceiling over most of its length.
            let headed = |(b, e): (usize, usize)| (b..e).filter(|c| f.ceil_cols[*c] || corner[*c]).count() as f64 >= WALL_REACHES_CEILING * (e - b) as f64;
            if jambs_at_ends {
                for (b, e) in runs.into_iter().filter(|r| r.1 - r.0 >= wide && headed(*r)) {
                    for c in b..e {
                        reach[c] = true;
                    }
                }
            }
        }
        let down = reach.iter().filter(|x| **x).count() as f64;
        let cols_all = f.floor_cols.len().max(1) as f64;
        // What is left out and is 6 ft or longer is offered by its number as "likely a wall", and is laid only on the person's word (call
        // 188, 1): a face that reaches the ceiling over some of its length, though not over most of it (a wall whose top was not seen over the
        // rest, or lowered), and one that reaches the ceiling and whose foot was not seen (a low thing stands before it that no face was
        // found for, or it hangs from the ceiling). A face that stands short of the ceiling over all its length is a cabinet, and is not offered.
        let long_enough = f.s1 - f.s0 >= LIKELY_LEAST;
        if up / cols < WALL_REACHES_CEILING {
            if long_enough {
                doubts.insert(i, format!("likely a wall: a face {} long whose top reaches the ceiling over {} in 100 of its length and not over the rest (a wall whose top was not seen there, or was lowered, or something tall)", plumbline_units(f.s1 - f.s0), (100.0 * up / cols).round()));
            } else {
                demoted.push((i, false));
            }
        } else if down / cols_all < WALL_REACHES_FLOOR {
            if long_enough {
                doubts.insert(i, format!("likely a wall: a face {} long that reaches the ceiling, its foot not seen over {} of its length (something low stood before it, or it hangs from the ceiling)", plumbline_units(f.s1 - f.s0), plumbline_units((1.0 - down / cols_all) * (f.s1 - f.s0))));
            } else {
                out.left_out.push(format!("a face {} long at {} east, {} north reaches the ceiling and not the floor, and nothing stands before it: it is not taken as a wall (something hung from the ceiling, or a wall seen only above)", plumbline_units(f.s1 - f.s0), plumbline_units(f.at(f.mid())[0]), plumbline_units(f.at(f.mid())[1])));
                demoted.push((i, true));
            }
        }
    }
    for (i, _) in &demoted {
        let f = &faces[*i];
        out.stood.push(Left { from: f.at(f.s0), to: f.at(f.s1), why: "stands short of the ceiling over some of its length (a cabinet, furniture, or a wall whose top was not seen)".into() });
    }
    for (i, hung) in demoted {
        faces[i].wall = false;
        faces[i].hung = hung;
        wall_ids.retain(|j| *j != i);
    }
    let standing: Vec<usize> = (0..faces.len()).filter(|i| !faces[*i].wall && !faces[*i].hung && !faces[*i].gone).collect();
    // ---- one wall seen in two parts, with something standing along the gap between them ----
    // (call 179, A2) A wall's face that stops where a cabinet bank begins and starts again where it
    // ends is one wall, not two with the room open between: not seen is not open. The gap is said
    // where the openings are, as not seen.
    {
        let mut again = true;
        while again {
            again = false;
            'pairs: for &a in &wall_ids {
                for &b in &wall_ids {
                    // `a` is the longer of the two: its line is the one the other is to lie on (a stub of
                    // 2 ft is told to a degree or two at best, and 10 ft of wall to a tenth).
                    if a == b || doubts.contains_key(&a) || doubts.contains_key(&b) || faces[a].s1 - faces[a].s0 < faces[b].s1 - faces[b].s0 {
                        continue;
                    }
                    let (fa, fb) = (&faces[a], &faces[b]);
                    let (d, n) = (dir(fa.theta), normal(fa.theta));
                    let ends = [fb.at(fb.s0), fb.at(fb.s1)];
                    if ends.iter().any(|e| (dot(*e, n) - fa.rho).abs() > BAND + 0.05) {
                        continue;
                    }
                    let (o0, o1) = (dot(ends[0], d).min(dot(ends[1], d)), dot(ends[0], d).max(dot(ends[1], d)));
                    let gap = if o0 >= fa.s1 - 0.5 { (fa.s1, o0) } else if o1 <= fa.s0 + 0.5 { (o1, fa.s0) } else { continue };
                    // Something stands before the whole gap, on the side the scan saw the wall from.
                    let sides: Vec<f64> = if fa.seen[0] != fa.seen[1] { vec![if fa.seen[0] { 1.0 } else { -1.0 }] } else { vec![1.0, -1.0] };
                    let steps = (((gap.1 - gap.0) / sq).ceil() as usize).max(1);
                    let covered = (0..steps).filter(|k| {
                        let (lo, hi) = (gap.0 + *k as f64 * sq, gap.0 + (*k + 1) as f64 * sq);
                        stand_at.iter().any(|q| {
                            let (sq, ac) = (dot(*q, d), dot(*q, n) - fa.rho);
                            sq >= lo && sq <= hi && sides.iter().any(|side| ac * side > IN_FRONT_LEAST && ac * side <= IN_FRONT)
                        })
                    }).count();
                    if covered as f64 >= STANDS_ALONG * steps as f64 {
                        let (db, nb) = (dir(fb.theta), normal(fb.theta));
                        let moved: Vec<[f64; 3]> = fb.pts.iter().map(|p| { let q = [p[0] * db[0] + p[1] * nb[0], p[0] * db[1] + p[1] * nb[1]]; [dot(q, d), dot(q, n), p[2]] }).collect();
                        let seen = [fa.seen[0] || fb.seen[0], fa.seen[1] || fb.seen[1]];
                        let (s0, s1) = (fa.s0.min(o0), fa.s1.max(o1));
                        let fa = &mut faces[a];
                        fa.pts.extend(moved);
                        fa.s0 = s0;
                        fa.s1 = s1;
                        fa.seen = seen;
                        wall_ids.retain(|j| *j != b);
                        again = true;
                        break 'pairs;
                    }
                }
            }
        }
    }

    // ---- what stands short: parts joined at their ends into things ----
    let mut group: Vec<usize> = (0..standing.len()).collect();
    fn root(g: &mut Vec<usize>, i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        g[i] = r;
        r
    }
    for a in 0..standing.len() {
        for b in a + 1..standing.len() {
            let (fa, fb) = (&faces[standing[a]], &faces[standing[b]]);
            let near = [fa.at(fa.s0), fa.at(fa.s1)].iter().any(|p| [fb.at(fb.s0), fb.at(fb.s1)].iter().any(|q| (p[0] - q[0]).hypot(p[1] - q[1]) <= THING_JOIN));
            if near {
                let (ra, rb) = (root(&mut group, a), root(&mut group, b));
                group[ra] = rb;
            }
        }
    }
    let mut things: HashMap<usize, Thing> = HashMap::new();
    for a in 0..standing.len() {
        let r = root(&mut group, a);
        let f = &faces[standing[a]];
        let m = f.at(f.mid());
        let t = things.entry(r).or_insert(Thing { height: 0.0, at: m });
        t.height = t.height.max(f.height);
    }
    let mut things: Vec<Thing> = things.into_values().collect();
    things.sort_by(|a, b| a.at[0].total_cmp(&b.at[0]).then(a.at[1].total_cmp(&b.at[1])));
    out.things = things;
    // ---- pair faces into walls ----
    let mut taken: HashSet<usize> = HashSet::new();
    let mut pairs: Vec<(usize, usize, f64)> = Vec::new();
    for (x, a) in wall_ids.iter().enumerate() {
        for b in &wall_ids[x + 1..] {
            if doubts.contains_key(a) || doubts.contains_key(b) {
                continue;
            }
            let (fa, fb) = (&faces[*a], &faces[*b]);
            let mut dt = (fa.theta - fb.theta).abs();
            dt = dt.min(std::f64::consts::PI - dt);
            if dt.to_degrees() > PAIR_DEGREES {
                continue;
            }
            // Which side of one the other's middle is on, and how far.
            let across = dot(fb.at(fb.mid()), normal(fa.theta)) - fa.rho;
            if across.abs() < PAIR_LEAST || across.abs() > PAIR_MOST_BARE {
                continue;
            }
            // Side by side along the first's own measure of the way along.
            let (b0, b1) = along_span(fb, dir(fa.theta));
            let overlap = fa.s1.min(b1) - fa.s0.max(b0);
            if overlap < PAIR_ALONGSIDE {
                continue;
            }
            // Past 16 in, two faces are one wall (a thick wall of masonry, a chase) only where no floor is seen
            // between them, and where they end together: a cabinet bank 2 ft deep before a wall, or two walls
            // with a closet between, are not (call 179, A6).
            if across.abs() > PAIR_MOST {
                let (lo, hi) = (fa.s0.max(b0), fa.s1.min(b1));
                let (na, da) = (normal(fa.theta), dir(fa.theta));
                let (ra, rb) = (fa.rho.min(fa.rho + across), fa.rho.max(fa.rho + across));
                let width = (rb - ra - 2.0 * FLOOR_MARGIN).max(0.0);
                let seen = scan.points.iter().filter(|p| {
                    let q = [p[0], p[1]];
                    (p[2] - floor).abs() <= SET_ASIDE && dot(q, na) > ra + FLOOR_MARGIN && dot(q, na) < rb - FLOOR_MARGIN && dot(q, da) > lo + FLOOR_MARGIN && dot(q, da) < hi - FLOOR_MARGIN
                }).count() as f64;
                let would = rho.unwrap_or(REFERENCE_DENSITY) * width * (hi - lo - 2.0 * FLOOR_MARGIN).max(0.0);
                let ends_agree = (fa.s0 - b0).abs() <= CORNER && (fa.s1 - b1).abs() <= CORNER;
                if seen > FLOOR_BETWEEN * would || !ends_agree {
                    continue;
                }
            }
            pairs.push((*a, *b, overlap));
        }
    }
    pairs.sort_by(|p, q| q.2.total_cmp(&p.2));
    let mut walls: Vec<(Wall, WallBuild)> = Vec::new();
    for (a, b, _) in pairs {
        if taken.contains(&a) || taken.contains(&b) {
            continue;
        }
        taken.insert(a);
        taken.insert(b);
        let (fa, fb) = (&faces[a], &faces[b]);
        let theta = fa.theta;
        let (d, n) = (dir(theta), normal(theta));
        // Both in the first's frame.
        let rb = dot(fb.at(fb.mid()), n);
        let (b0, b1) = along_span(fb, d);
        let centre = (fa.rho + rb) / 2.0;
        // Where the two faces end differently by no more than a wall is thick, they meet a wall across them: the
        // wall ends at its near face, the shorter of the two, and not at its far one (call 179, A9). Where they
        // end far apart one of them is hidden, and the wall is as long as both show.
        let s0 = { let a0 = fa.s0; if (a0 - b0).abs() <= PAIR_MOST { a0.max(b0) } else { a0.min(b0) } };
        let s1 = { let a1 = fa.s1; if (a1 - b1).abs() <= PAIR_MOST { a1.min(b1) } else { a1.max(b1) } };
        let mut pts: Vec<[f64; 2]> = fa.pts.iter().map(|p| [p[0], p[2]]).collect();
        // fb's points in fa's frame: where they lie along fa's own line (fb's line may run the other way: both are the same line to a degree).
        let (db, nb) = (dir(fb.theta), normal(fb.theta));
        pts.extend(fb.pts.iter().map(|p| [(p[0] * db[0] + p[1] * nb[0]) * d[0] + (p[0] * db[1] + p[1] * nb[1]) * d[1], p[2]]));
        // Seen from the side each face faces away from the other: both, so none of the one-sided.
        let (mut w, flip) = wall_of(theta, centre, s0, s1, Some((rb - fa.rho).abs()), None);
        w.height = Some(fa.ceil.min(fb.ceil));
        let deeper = if fa.bow.0 >= fb.bow.0 { fa.bow } else { fb.bow };
        w.bow = (deeper.0 >= crate::found_188::BOW_SAY).then_some(deeper);
        w.joined = fa.joined + fb.joined;
        walls.push((w, WallBuild { pts, theta, rho: centre, s0, flip }));
    }
    for i in &wall_ids {
        if taken.contains(i) {
            continue;
        }
        let f = &faces[*i];
        // The side it was seen from: the normal's, or against it.
        let toward = if f.seen[1] && !f.seen[0] { -1.0 } else { 1.0 };
        let n = normal(f.theta);
        let seen_from = Some([n[0] * toward, n[1] * toward]);
        let pts: Vec<[f64; 2]> = f.pts.iter().map(|p| [p[0], p[2]]).collect();
        let (mut w, flip) = wall_of(f.theta, f.rho, f.s0, f.s1, None, seen_from);
        w.doubt = doubts.get(i).cloned();
        w.height = Some(f.ceil);
        w.bow = (f.bow.0 >= crate::found_188::BOW_SAY).then_some(f.bow);
        w.joined = f.joined;
        walls.push((w, WallBuild { pts, theta: f.theta, rho: f.rho, s0: f.s0, flip }));
    }

    if big {
        crate::found_188::progress("looking for openings in the walls");
    }
    // ---- the openings, and what is not seen ----
    for (w, b) in &mut walls {
        let (d, n) = (dir(b.theta), normal(b.theta));
        let length = w.length();
        let tops = |s: f64| ceil.over([s * d[0] + b.rho * n[0], s * d[1] + b.rho * n[1]], n);
        let g = Grid::new(&b.pts.iter().map(|p| [p[0], p[1]]).collect::<Vec<_>>(), b.s0, b.s0 + length, high, sq, &tops);
        // Empty squares, joined; the wide ones are holes. A square empty by the scatter's chance (a
        // lone one, or a thin string of them through the solid) is not part of a hole: only squares
        // with at least three empty neighbours are joined, so that noise does not carry a hole out
        // to the wall's end (call 179, A2).
        let empty = |c: usize, r: usize| !g.full(c, r);
        let core = |c: usize, r: usize| {
            empty(c, r) && [(c.wrapping_sub(1), r), (c + 1, r), (c, r.wrapping_sub(1)), (c, r + 1)].iter().filter(|(nc, nr)| *nc < g.cols && *nr < g.rows && empty(*nc, *nr)).count() >= 3
        };
        let mut seen = vec![false; g.cols * g.rows];
        let mut holes: Vec<(usize, usize, usize, usize, usize, bool)> = Vec::new();
        for r in 0..g.rows {
            for c in 0..g.cols {
                if seen[r * g.cols + c] || !core(c, r) {
                    continue;
                }
                let (mut stack, mut cells) = (vec![(c, r)], Vec::new());
                seen[r * g.cols + c] = true;
                while let Some((cc, rr)) = stack.pop() {
                    cells.push((cc, rr));
                    let near = [(cc.wrapping_sub(1), rr), (cc + 1, rr), (cc, rr.wrapping_sub(1)), (cc, rr + 1)];
                    for (nc, nr) in near {
                        if nc < g.cols && nr < g.rows && !seen[nr * g.cols + nc] && core(nc, nr) {
                            seen[nr * g.cols + nc] = true;
                            stack.push((nc, nr));
                        }
                    }
                }
                let (c0, c1) = (cells.iter().map(|x| x.0).min().unwrap_or(0), cells.iter().map(|x| x.0).max().unwrap_or(0));
                let (r0, r1) = (cells.iter().map(|x| x.1).min().unwrap_or(0), cells.iter().map(|x| x.1).max().unwrap_or(0));
                let box_cells = (c1 - c0 + 1) * (r1 - r0 + 1);
                // A hole is a clean block of empty squares, wide enough, not at the wall's ends.
                // One that runs to an end of the wall is no opening (a wall does not end in a doorway)
                // but may be what stands before the wall's end hiding it (call 179, A2): kept to be said so.
                if (c1 - c0 + 1) as f64 * sq >= MIN_OPENING - sq && cells.len() as f64 >= 0.8 * box_cells as f64 {
                    holes.push((c0, c1, r0, r1, cells.len(), c0 == 0 || c1 + 1 == g.cols));
                }
            }
        }
        holes.sort();
        for (c0, c1, r0, r1, _, at_end) in holes {
            let (ls, rs) = (g.col_range(c0), g.col_range(c1 + 1));
            let (zb, zt) = (SET_ASIDE + r0 as f64 * sq, SET_ASIDE + (r1 + 1) as f64 * sq);
            if at_end {
                let (el, er) = (ls.max(b.s0), rs.min(b.s0 + length));
                let blocked = |side: f64| -> bool {
                    stand_at.iter().any(|q| {
                        let (s, a) = (dot(*q, d), (dot(*q, n) - b.rho) * side);
                        let half = w.thickness.map_or(0.0, |t| t / 2.0);
                        s >= el && s <= er && a > half + IN_FRONT_LEAST && a <= half + IN_FRONT
                    })
                };
                let hidden = if w.thickness.is_some() { blocked(1.0) && blocked(-1.0) } else { blocked(w.seen_from.map_or(1.0, |sf| if dot(sf, n) >= 0.0 { 1.0 } else { -1.0 })) };
                if hidden {
                    let (from, to) = (el - b.s0, er - b.s0);
                    w.hidden.push(if b.flip { Stretch { from: length - to, to: length - from } } else { Stretch { from, to } });
                }
                continue;
            }
            if r1 + 1 >= g.rows {
                // Nothing at all from floor to ceiling over this stretch: where something stands before it
                // (a bank of cabinets to the ceiling, with the wall behind it unseen) it is not seen, and is
                // taken as wall and said so (call 179, A2); where nothing does, it is said as it was.
                let (el, er) = (ls.max(b.s0), rs.min(b.s0 + length));
                let blocked = |side: f64| -> bool {
                    stand_at.iter().any(|q| {
                        let (sq, a) = (dot(*q, d), (dot(*q, n) - b.rho) * side);
                        let half = w.thickness.map_or(0.0, |t| t / 2.0);
                        sq >= el && sq <= er && a > half + IN_FRONT_LEAST && a <= half + IN_FRONT
                    })
                };
                let hidden = if w.thickness.is_some() { blocked(1.0) && blocked(-1.0) } else { blocked(w.seen_from.map_or(1.0, |sf| if dot(sf, n) >= 0.0 { 1.0 } else { -1.0 })) };
                if hidden {
                    let (from, to) = (el - b.s0, er - b.s0);
                    w.hidden.push(if b.flip { Stretch { from: length - to, to: length - from } } else { Stretch { from, to } });
                } else {
                    out.left_out.push(format!("a gap {} wide in a wall, {} from its {} end, runs from the floor to the ceiling: it is not taken as an opening, and the wall is laid across it", plumbline_units(rs - ls), plumbline_units(if b.flip { length - (rs - b.s0) } else { ls - b.s0 }), end_word(w)));
                }
                continue;
            }
            // Along: between the points of the strip inside the hole's own height.
            let strip_lo = if r0 <= 1 { SET_ASIDE } else { zb + 0.3 };
            let strip_hi = (zt - 0.3).max(strip_lo + 0.1);
            let in_strip: Vec<f64> = b.pts.iter().filter(|p| p[1] >= strip_lo && p[1] <= strip_hi).map(|p| p[0]).collect();
            let hole_mid = (ls + rs) / 2.0;
            let el = edge(&in_strip, ls - EDGE_BACK, false, ls - EDGE_BACK - b.s0 - NEAR_END, hole_mid).unwrap_or(ls);
            let er = edge(&in_strip, rs + EDGE_BACK, true, b.s0 + length - rs - EDGE_BACK - NEAR_END, hole_mid).unwrap_or(rs);
            if er - el < MIN_OPENING - MIN_OPENING_SLACK {
                continue;
            }
            // Up: between the points of the strip inside the hole's own width.
            let in_up: Vec<f64> = b.pts.iter().filter(|p| p[0] >= el + 0.3 && p[0] <= er - 0.3).map(|p| p[1]).collect();
            let head = edge(&in_up, zt + EDGE_BACK, true, tops((el + er) / 2.0) - SET_ASIDE - zt - EDGE_BACK, zb).unwrap_or(zt);
            let window = r0 > 1;
            let sill = if window { edge(&in_up, zb - EDGE_BACK, false, zb - EDGE_BACK - SET_ASIDE, zt).unwrap_or(zb) } else { 0.0 };
            let (from, to) = (el - b.s0, er - b.s0);
            let opening = if b.flip { Opening { window, from: length - to, to: length - from, sill, head } } else { Opening { window, from, to, sill, head } };
            // Something standing in front, on the side the scan saw the wall from, hides it.
            let blocked = |side: f64| -> bool {
                stand_at.iter().any(|q| {
                    let (s, a) = (dot(*q, d), (dot(*q, n) - b.rho) * side);
                    // On the scanned side the thickness of a wall is its half either way; across both faces from the centre.
                    let half = w.thickness.map_or(0.0, |t| t / 2.0);
                    s >= el && s <= er && a > half + IN_FRONT_LEAST && a <= half + IN_FRONT
                })
            };
            let hidden = if w.thickness.is_some() { blocked(1.0) && blocked(-1.0) } else {
                let toward = w.seen_from.map_or(1.0, |s| if dot(s, n) >= 0.0 { 1.0 } else { -1.0 });
                blocked(toward)
            };
            if hidden {
                w.hidden.push(Stretch { from: opening.from, to: opening.to });
            } else {
                w.openings.push(opening);
            }
        }
    }

    // ---- what lies beyond a wall with no way to it (call 183, 2): in doubt, said ----
    let walls_only: Vec<Wall> = walls.into_iter().map(|(w, _)| w).collect();
    let (pieces, gone, note) = crate::found_183::beyond(walls_only, &mut out.things, &floor_dense);
    out.beyond_note = note;
    // ---- first end first: west to east, or south to north ----
    let mut pieces = pieces;
    // Where the ceiling was not read over a wall (call 183, 6): said.
    for (w, _) in pieces.iter_mut() {
        let (d, n) = (unit2(sub2(w.to, w.from)), [0.0, 0.0]);
        let _ = n;
        let mid = [(w.from[0] + w.to[0]) / 2.0, (w.from[1] + w.to[1]) / 2.0];
        let nn = [-d[1], d[0]];
        w.ceiling_not_read = !(ceil.reads([mid[0] + nn[0] * CEIL_OUT, mid[1] + nn[1] * CEIL_OUT]) || ceil.reads([mid[0] - nn[0] * CEIL_OUT, mid[1] - nn[1] * CEIL_OUT]));
    }
    pieces.sort_by(|a, b| a.0.from[0].total_cmp(&b.0.from[0]).then(a.0.from[1].total_cmp(&b.0.from[1])));
    // The number each wall had before they were put in order, to the number it has now (of its piece that is not in doubt, where it was cut).
    let place_of = |orig: usize| pieces.iter().position(|(w, o)| *o == orig && !w.doubt.as_deref().is_some_and(|d| d.starts_with("in doubt: beyond"))).or_else(|| pieces.iter().position(|(_, o)| *o == orig)).unwrap_or(0);
    out.beyond_things = gone.into_iter().map(|(_, s)| crate::found_183::resolve(&s, &place_of)).collect();
    let numbered: Vec<(Wall, usize)> = pieces.iter().map(|(w, o)| (Wall { doubt: w.doubt.as_ref().map(|d| crate::found_183::resolve(d, &place_of)), ..w.clone() }, *o)).collect();
    out.walls = numbered.into_iter().map(|(w, _)| w).collect();
    // A face belongs to one wall (call 183, 1).
    crate::found_183::one_face_one_wall(&mut out.walls);
    out.open_ends = open_ends(&out.walls);
    out
}
