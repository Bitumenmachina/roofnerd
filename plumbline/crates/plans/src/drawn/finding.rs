use super::*;

/// A length as a tape says it, to the quarter inch.
pub fn inch_said(feet: f64) -> String {
    let q = (feet.abs() * 12.0 * 4.0).round() as i64;
    let (ft, rem) = (q / 48, q % 48);
    let (whole, frac) = (rem / 4, rem % 4);
    let f = match frac {
        1 => " 1/4",
        2 => " 1/2",
        3 => " 3/4",
        _ => "",
    };
    format!("{}{}' {}{}\"", if feet < 0.0 && q > 0 { "-" } else { "" }, ft, whole, f)
}

pub(super) fn shift(p: P2, by: P2) -> P2 {
    [p[0] + by[0], p[1] + by[1]]
}

pub(super) fn place_said(p: P2) -> String {
    format!("{} east, {} north", inch_said(p[0]), inch_said(p[1]))
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Why {
    Dimension,
    Border,
    Boxed,
    Set,
    Tag,
}

/// The walls a drawn page holds, found from its lines. `feet_per_unit` is the sheet's scale (None:
/// it has none, and nothing is found), `page` its size in its own units, `origin` where the page's
/// lower-left corner lies on the plan, in feet: every place that comes back is a place on the plan.
pub fn find(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2) -> Drawn {
    find_on(lines, feet_per_unit, page, origin, &Extra::default())
}

/// `find`, told which of the lines are dashed and where the page cut lines off.
pub fn find_on(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2, extra: &Extra) -> Drawn {
    find_all(lines, feet_per_unit, page, origin, extra)
}

/// `find_on`, with the part of the sheet the person says is the plan: a box round it (`within`, corners in feet on the plan), a
/// drawing by its number (`plan`, counted from 1 as `Drawn::drawings` lists them, the largest first), or both: the number then
/// counts the drawings found inside the box. A number the sheet does not have is refused, with how many it has.
pub fn find_in(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2, extra: &Extra, pick: &Pick) -> Result<Drawn, String> {
    let Some(k) = feet_per_unit.filter(|k| k.is_finite() && *k > 0.0) else {
        return Ok(find_all(lines, feet_per_unit, page, origin, extra));
    };
    let Some(bx) = pick.within else { return find_core(lines, feet_per_unit, page, origin, extra, pick.plan) };
    // Lines kept to a box, with the dashed marks and the cuts that go with them.
    let (lo, hi) = ([bx[0][0].min(bx[1][0]), bx[0][1].min(bx[1][1])], [bx[0][0].max(bx[1][0]), bx[0][1].max(bx[1][1])]);
    let slack = INCH / 12.0;
    let inside = |p: P2| {
        let (x, y) = (p[0] * k + origin[0], p[1] * k + origin[1]);
        x >= lo[0] - slack && x <= hi[0] + slack && y >= lo[1] - slack && y <= hi[1] + slack
    };
    let dashed_at: std::collections::HashSet<usize> = extra.dashed.iter().copied().collect();
    let (mut ls, mut ds) = (Vec::new(), Vec::new());
    let mut straddling = 0usize;
    for (i, l) in lines.iter().enumerate() {
        let (a, b) = (inside(l[0]), inside(l[1]));
        if a && b {
            if dashed_at.contains(&i) {
                ds.push(*l);
            } else {
                ls.push(*l);
            }
        } else if a || b {
            straddling += 1;
        }
    }
    let outside = lines.len() - ls.len() - ds.len();
    // The solid lines first, the dashed ones after them, marked.
    let first_dashed = ls.len();
    let marks: Vec<usize> = (first_dashed..first_dashed + ds.len()).collect();
    ls.extend(ds);
    let cuts: Vec<P2> = extra.cuts.iter().copied().filter(|p| inside(*p)).collect();
    let mut d = find_core(&ls, feet_per_unit, page, origin, &Extra { dashed: &marks, cuts: &cuts }, pick.plan)?;
    d.counts.outside = outside;
    d.within = Some(bx);
    // A line that crosses the box's edge is left out whole, and a wall drawn with it is not found, or is found short: the walls of the
    // whole sheet that lie partly in the box and partly out of it are named (call 191 item 5).
    if straddling > 0 {
        let full = find_all(lines, feet_per_unit, page, origin, extra);
        let at = |p: P2| p[0] >= lo[0] - slack && p[0] <= hi[0] + slack && p[1] >= lo[1] - slack && p[1] <= hi[1] + slack;
        for w in &full.walls {
            if at(w.from) != at(w.to) {
                d.cut_by_box.push([w.from, w.to]);
            }
        }
    }
    Ok(d)
}

pub(super) fn find_all(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2, extra: &Extra) -> Drawn {
    find_core(lines, feet_per_unit, page, origin, extra, None).unwrap_or_default()
}

pub(super) fn find_core(lines: &[[[f64; 2]; 2]], feet_per_unit: Option<f64>, page: [f64; 2], origin: P2, extra: &Extra, plan: Option<usize>) -> Result<Drawn, String> {
    let mut out = Drawn::default();
    let Some(k) = feet_per_unit.filter(|k| k.is_finite() && *k > 0.0) else {
        out.no_scale = true;
        return Ok(out);
    };
    let min_len = MIN_FACE.max(LETTER_PT * k);
    let dashed_at: std::collections::HashSet<usize> = extra.dashed.iter().copied().collect();
    let mut segs: Vec<Seg> = Vec::new();
    let mut dsegs: Vec<Seg> = Vec::new();
    for (li, l) in lines.iter().enumerate() {
        let a = [l[0][0] * k, l[0][1] * k];
        let b = [l[1][0] * k, l[1][1] * k];
        let len = norm(sub(b, a));
        if a.iter().chain(b.iter()).all(|v| v.is_finite()) && len > 1e-9 {
            // A wall's lines are solid (call 186): a dashed line is looked at for what it may be and for nothing else.
            if dashed_at.contains(&li) {
                dsegs.push(Seg { a, b, len });
            } else {
                segs.push(Seg { a, b, len });
            }
        }
    }
    out.counts.lines = segs.len() + dsegs.len();
    out.counts.dashed = dsegs.len();
    out.overhead = overhead_of(&dsegs, k, origin);
    if segs.len() > MOST_SEGS {
        let before = segs.len();
        segs.retain(|s| s.len >= SHORT_KEPT);
        out.counts.left_out_short_lines = before - segs.len();
        out.notes.push(format!("the sheet draws {before} lines, more than can all be looked at: the {} under 4 in long at scale were left out", before - segs.len()));
    }
    let n_segs = segs.len();
    let mut why: Vec<Option<Why>> = vec![None; n_segs];

    // 1. Dimension lines, with their ticks and extension lines.
    for (i, d) in dimension_lines(&segs, k).iter().enumerate() {
        if *d {
            why[i] = Some(Why::Dimension);
        }
    }

    // 1b. Tags: a window's or a door's mark is no wall, no line across one and no glass (call 188 item 8).
    {
        let skip: Vec<bool> = why.iter().map(|w| w.is_some()).collect();
        for tag in tags_of(&segs, &skip) {
            for i in tag {
                why[i] = Some(Why::Tag);
            }
            out.counts.tags += 1;
        }
    }

    // 2. Boxes: the border, and title blocks, keys and schedules. A box of walls is not one: a ring of
    // walls is two boxes one thickness apart, and a box with lines beside its sides inside it is walls.
    let alive: Vec<usize> = (0..n_segs).filter(|&i| why[i].is_none()).collect();
    let (faces0, of0) = faces_of(&segs, &alive, LETTER_PT * k);
    let long0: Vec<bool> = faces0.iter().map(|f| f.longest >= min_len).collect();
    let near0 = neighbours(&faces0, &long0, min_len);
    let rects = rects_of(&faces0, k);
    let page_area = page[0] * k * page[1] * k;
    let centre_of = |r: &Rect| at(r.u, r.n, (r.t.0 + r.t.1) / 2.0, (r.s.0 + r.s.1) / 2.0);
    for r in &rects {
        let (w, h) = (r.t.1 - r.t.0, r.s.1 - r.s.0);
        let is_border = page_area > 0.0 && w * h >= BORDER_SHARE * page_area;
        if !is_border {
            if w.min(h) <= THICK_MOST {
                continue;
            }
            let c = centre_of(r);
            let ring = rects.iter().any(|q| {
                let along = dot(q.u, r.u);
                if along.abs() < 0.999 && along.abs() > 0.0175 {
                    return false;
                }
                let (qa, qb) = if along.abs() > 0.5 { (q.t.1 - q.t.0, q.s.1 - q.s.0) } else { (q.s.1 - q.s.0, q.t.1 - q.t.0) };
                let d = (w - qa) / 2.0;
                norm(sub(centre_of(q), c)) <= CAP_TOL && (THIN_LEAST - 1e-3..=THICK_MOST + 1e-3).contains(&d) && ((h - qb) / 2.0 - d).abs() <= INCH / 4.0
            });
            // Sides with a line beside them (a wall's other face): a room's inner faces have a wall round
            // them on every side and a ring of walls has one inside; a title block has them at two sides at most.
            let inner = r.sides.iter().filter(|&&s| near0[s].iter().any(|(b, _, _)| !r.sides.contains(b))).count();
            if ring || inner >= 3 {
                continue;
            }
        }
        let slack = TOUCH_PT * k;
        let mut count = 0;
        for &i in &alive {
            if why[i].is_some() {
                continue;
            }
            let s = &segs[i];
            let inside = |p: P2| dot(r.u, p) >= r.t.0 - slack && dot(r.u, p) <= r.t.1 + slack && dot(r.n, p) >= r.s.0 - slack && dot(r.n, p) <= r.s.1 + slack;
            if r.sides.contains(&of0[&i]) {
                why[i] = Some(if is_border { Why::Border } else { Why::Boxed });
                count += usize::from(!is_border);
            } else if !is_border && inside(s.a) && inside(s.b) {
                why[i] = Some(Why::Boxed);
                count += 1;
            }
        }
        if !is_border {
            let corner = at(r.u, r.n, r.t.0, r.s.0);
            out.boxes.push(Boxed { wide: w, high: h, at: corner, lines: count });
        }
    }

    // 3. The faces of what is left.
    let alive: Vec<usize> = (0..n_segs).filter(|&i| why[i].is_none()).collect();
    let (faces, of) = faces_of(&segs, &alive, LETTER_PT * k);
    let nf = faces.len();
    let long: Vec<bool> = faces.iter().map(|f| f.longest >= min_len).collect();
    let near = neighbours(&faces, &long, min_len);

    // Lines inside a wall: a shorter line between two that are a wall's faces (glass, a sash, hatching).
    let mut inside = vec![false; nf];
    for f in 0..nf {
        if !long[f] {
            continue;
        }
        'f: for (p, dp, _) in near[f].iter().filter(|e| e.1 < 0.0) {
            for (q, dq, _) in near[f].iter().filter(|e| e.1 > 0.0) {
                if dq - dp <= THICK_MOST + 1e-6 && faces[f].longest < INSIDE_SHARE * faces[*p].longest.min(faces[*q].longest) {
                    inside[f] = true;
                    break 'f;
                }
            }
        }
    }
    // Evenly spaced runs of three or more are not walls.
    let mut in_set = vec![false; nf];
    {
        let mut order: Vec<usize> = (0..nf).filter(|&i| long[i] && !inside[i]).collect();
        order.sort_by(|&x, &y| ((angle_of(faces[x].u) / ANGLE_PAIR) as i64).cmp(&((angle_of(faces[y].u) / ANGLE_PAIR) as i64)).then(faces[x].c.total_cmp(&faces[y].c)));
        for &start in &order {
            if in_set[start] {
                continue;
            }
            let mut run = vec![start];
            let mut spacing: Option<f64> = None;
            loop {
                let last = *run.last().unwrap();
                let mut next: Option<(usize, f64)> = None;
                for (b, d, _) in &near[last] {
                    if *d <= 0.0 || inside[*b] || in_set[*b] || run.contains(b) {
                        continue;
                    }
                    let ok = spacing.map_or(true, |s| (d - s).abs() <= (EVEN_SHARE * s).max(EVEN_ABS));
                    if ok && next.map_or(true, |(_, nd)| *d < nd) {
                        next = Some((*b, *d));
                    }
                }
                let Some((b, d)) = next else { break };
                spacing.get_or_insert(d);
                run.push(b);
            }
            if run.len() >= 3 {
                let mut centre = [0.0, 0.0];
                for &f in &run {
                    in_set[f] = true;
                    let (p, q) = faces[f].ends(faces[f].iv[0].0, faces[f].iv.last().unwrap().1);
                    centre = [centre[0] + (p[0] + q[0]) / 2.0, centre[1] + (p[1] + q[1]) / 2.0];
                }
                for &i in &alive {
                    if run.contains(&of[&i]) {
                        why[i] = Some(Why::Set);
                    }
                }
                let cnt = run.len() as f64;
                out.sets.push(Set { lines: run.len(), spacing: spacing.unwrap_or(0.0), at: [centre[0] / cnt, centre[1] / cnt] });
            }
        }
    }

    // 4. Pairs and the walls they make.
    let usable = |f: usize| long[f] && !inside[f] && !in_set[f];
    let mut pairs: Vec<(usize, usize, f64, Iv)> = Vec::new();
    for a in 0..nf {
        if usable(a) {
            for (b, d, iv) in &near[a] {
                if *b > a && usable(*b) {
                    pairs.push((a, *b, *d, iv.clone()));
                }
            }
        }
    }
    // A face in two pairs over the same stretch is not a clean pair.
    let np = pairs.len();
    let mut conflict: Vec<Option<String>> = vec![None; np];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); np];
    for i in 0..np {
        for j in (i + 1)..np {
            let Some(f) = [pairs[i].0, pairs[i].1].into_iter().find(|f| *f == pairs[j].0 || *f == pairs[j].1) else { continue };
            let on_f = |p: &(usize, usize, f64, Iv)| if p.0 == f { p.3.clone() } else { faces[p.0].along(&faces[f], &p.3) };
            let shared: f64 = intersect(&on_f(&pairs[i]), &on_f(&pairs[j])).iter().map(|(a, b)| b - a).sum();
            if shared >= 1.0 {
                let said = format!("another line lies beside it ({} and {} from the same line, not evenly spaced): it may be a wall and a line drawn near it", inch_said(pairs[i].2.abs()).replace("0' ", ""), inch_said(pairs[j].2.abs()).replace("0' ", ""));
                conflict[i].get_or_insert(said.clone());
                conflict[j].get_or_insert(said);
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }
    // Pairs that share lines are a stack. A line is the face of one wall only, and a wall drawn in its layers is one wall
    // (call 186). A stack's outermost two lines, if they are a wall's thickness apart and the sheet has other walls of
    // that thickness (or other stacks that outer), are one wall with the lines between them its layers. Else the pair of
    // the thickness most of the sheet's clean walls have claims the lines it uses, and a pair that would use one again
    // is no wall in the way: in doubt, said so. Else it cannot be told, and all of them are in doubt, as they were.
    let mut loser = vec![false; np];
    let mut dropped = vec![false; np];
    let mut layered = vec![0usize; np];
    let mut joined = vec![1usize; np];
    {
        let mut comp = vec![usize::MAX; np];
        let mut comps: Vec<Vec<usize>> = Vec::new();
        for i in 0..np {
            if comp[i] != usize::MAX {
                continue;
            }
            let c = comps.len();
            let mut stack = vec![i];
            comp[i] = c;
            let mut members = Vec::new();
            while let Some(x) = stack.pop() {
                members.push(x);
                for &y in &adj[x] {
                    if comp[y] == usize::MAX {
                        comp[y] = c;
                        stack.push(y);
                    }
                }
            }
            members.sort_unstable();
            comps.push(members);
        }
        // Each component's outermost pair, where it has one, and what it holds.
        struct Stack {
            outer: Option<usize>,
            faces: usize,
            /// Each face's place across the stack, in feet from the first face, and where the outermost two are.
            offs: Vec<f64>,
            span: (f64, f64),
        }
        let mut stacks: Vec<Stack> = Vec::new();
        for m in &comps {
            if m.len() == 1 {
                stacks.push(Stack { outer: Some(m[0]), faces: 2, offs: Vec::new(), span: (0.0, 0.0) });
                continue;
            }
            let mut fs: Vec<usize> = m.iter().flat_map(|&i| [pairs[i].0, pairs[i].1]).collect();
            fs.sort_unstable();
            fs.dedup();
            let r = pairs[m[0]].0;
            let t_mid = pairs[m[0]].3.first().map(|x| (x.0 + x.1) / 2.0).unwrap_or(0.0);
            let off = |f: usize| if f == r { 0.0 } else { faces[f].offset_from(&faces[r], t_mid) };
            let lo = *fs.iter().min_by(|&&x, &&y| off(x).total_cmp(&off(y))).unwrap();
            let hi = *fs.iter().max_by(|&&x, &&y| off(x).total_cmp(&off(y))).unwrap();
            let outer = m.iter().copied().find(|&i| (pairs[i].0 == lo && pairs[i].1 == hi) || (pairs[i].0 == hi && pairs[i].1 == lo));
            let offs: Vec<f64> = fs.iter().map(|&f| off(f)).collect();
            stacks.push(Stack { outer, faces: fs.len(), offs, span: (off(lo), off(hi)) });
        }
        let span = |c: usize| stacks[c].outer.map(|i| pairs[i].2.abs());
        for (c, m) in comps.iter().enumerate() {
            if m.len() == 1 {
                continue;
            }
            // 1. Layers: the outer pair's thickness is one other walls or stacks have.
            if let (Some(o), Some(sp)) = (stacks[c].outer, span(c)) {
                let others = (0..comps.len()).filter(|&c2| c2 != c && span(c2).is_some_and(|s2| (s2 - sp).abs() <= THICK_SAME)).count();
                if others >= 1 && stacks[c].faces >= 3 {
                    for &i in m {
                        dropped[i] = i != o;
                        conflict[i] = None;
                    }
                    layered[o] = stacks[c].faces;
                    joined[o] = m.len();
                    continue;
                }
                // 1b. A wall whose faces are each drawn twice (call 188 item 7): every line of the stack is within a finish's
                // depth of one of its two outermost lines. It is one wall, whether or not another wall has its span.
                if stacks[c].faces >= 3 && stacks[c].offs.iter().all(|&x| (x - stacks[c].span.0).abs() <= DOUBLE_MAX + 1e-9 || (x - stacks[c].span.1).abs() <= DOUBLE_MAX + 1e-9) {
                    for &i in m {
                        dropped[i] = i != o;
                        conflict[i] = None;
                    }
                    layered[o] = stacks[c].faces;
                    joined[o] = m.len();
                    continue;
                }
            }
            // 2. The pair of the thickness most clean walls have claims its lines.
            let strength = |i: usize| (0..comps.len()).filter(|&c2| comps[c2].len() == 1 && (pairs[comps[c2][0]].2.abs() - pairs[i].2.abs()).abs() <= THICK_SAME).count();
            let mut ranked: Vec<(usize, usize)> = m.iter().map(|&i| (strength(i), i)).filter(|(n, _)| *n >= 1).collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));
            let mut resolved = false;
            if let Some(&(top, w)) = ranked.first() {
                // A tie between two pairs that do not share a line is no tie: the claim is on the lines of one pair.
                let tie = ranked.iter().skip(1).any(|&(n, i)| n == top && adj[w].contains(&i));
                if !tie {
                    resolved = true;
                    // A pair of the stack that shares no line with it is the same wall over the same stretch, and joined (call 188).
                    for &i in m {
                        if i != w && !adj[w].contains(&i) {
                            dropped[i] = true;
                            conflict[i] = None;
                            joined[w] += 1;
                        }
                    }
                    let said = format!("another line lies beside it: one of its lines is already the face of a wall {} thick, so it is not a second wall (a counter's edge, a ledge, a line drawn near a wall)", inch_said(pairs[w].2.abs()).replace("0' ", ""));
                    conflict[w] = None;
                    // The line beside it is said once, as the nearest pair it would make; the others over the same line are
                    // the same thing.
                    let mut by_line: BTreeMap<usize, usize> = BTreeMap::new();
                    for &i in m {
                        if i != w && adj[w].contains(&i) {
                            let line = [pairs[i].0, pairs[i].1].into_iter().find(|f| *f != pairs[w].0 && *f != pairs[w].1).unwrap_or(pairs[i].1);
                            match by_line.get(&line) {
                                Some(&keep) if pairs[keep].2.abs() <= pairs[i].2.abs() => dropped[i] = true,
                                Some(&keep) => {
                                    dropped[keep] = true;
                                    loser[keep] = false;
                                    by_line.insert(line, i);
                                }
                                None => {
                                    by_line.insert(line, i);
                                }
                            }
                        }
                    }
                    for &i in by_line.values() {
                        conflict[i] = Some(said.clone());
                        loser[i] = true;
                    }
                }
            }
            // 3. It cannot be told which pair is the wall: the pairs over one stretch are one thing, from the outermost line to the
            // outermost, in doubt (call 188 item 7): found once, with how many pairs it was made of.
            if !resolved {
                let o = stacks[c].outer.unwrap_or_else(|| *m.iter().max_by(|&&x, &&y| pairs[x].2.abs().total_cmp(&pairs[y].2.abs())).unwrap());
                for &i in m {
                    dropped[i] = i != o;
                }
                conflict[o] = conflict[o].take().or_else(|| m.iter().find_map(|&i| conflict[i].clone()));
                joined[o] = m.len();
            }
        }
    }

    let mut cands: Vec<Cand> = Vec::new();
    for (pi, (a, b, d, ov)) in pairs.iter().enumerate() {
        if dropped[pi] {
            continue;
        }
        let (fa, fb) = (&faces[*a], &faces[*b]);
        let cc = fa.c + d / 2.0;
        let thick = d.abs();
        let b_in_a = fb.along(fa, &fb.iv);
        let both: Iv = intersect(&fa.iv, &b_in_a).into_iter().filter(|&(lo, hi)| ov.iter().any(|&(o0, o1)| lo < o1 && hi > o0)).collect();
        if both.is_empty() {
            continue;
        }
        let union = merged(fa.iv.iter().chain(b_in_a.iter()).copied().collect(), 0.0);
        // Lines across the pair: square to it, with an end on each of its lines. A line drawn in pieces
        // (a cap drawn as two halves) is one line.
        let mut crosses: Vec<f64> = Vec::new();
        for g in &faces {
            if dot(g.u, fa.u).abs() > CROSS_ANGLE {
                continue;
            }
            // The pieces of this line that lie between the pair's two lines, none reaching past either, joined:
            // a cap drawn in halves is one cap, and a room line or a grid line that runs on through the wall
            // (and a dashed one) is none, by itself or in company with the cap it crosses.
            // A line across may run past the faces by less than the wall's thickness and be a line across all the same
            // (call 186): a cap drawn a little long. Past that it runs on through the wall (a room line, a grid line).
            let (lo, hi) = (0.0_f64.min(*d) - thick - CAP_TOL, 0.0_f64.max(*d) + thick + CAP_TOL);
            let mut inside: Vec<(f64, f64)> = Vec::new();
            let mut at_t = 0.0;
            for &(x, y) in &g.raw {
                let (p, q) = g.ends(x, y);
                let (e0, e1) = (dot(fa.n, p) - fa.c, dot(fa.n, q) - fa.c);
                if e0 >= lo && e0 <= hi && e1 >= lo && e1 <= hi {
                    inside.push((e0.min(e1), e0.max(e1)));
                    at_t = (dot(fa.u, p) + dot(fa.u, q)) / 2.0;
                }
            }
            let across = merged(inside, CAP_TOL);
            if across.iter().any(|&(e0, e1)| e0 <= 0.0_f64.min(*d) + CAP_TOL && e1 >= 0.0_f64.max(*d) - CAP_TOL) {
                crosses.push(at_t);
            }
        }
        crosses.sort_by(|x, y| x.total_cmp(y));
        crosses.dedup_by(|x, y| (*x - *y).abs() < CAP_TOL);
        // Whether something is drawn in a gap of this wall between `g0` and `g1` along it: a line (a leaf, a piece of a swing, a line across)
        // whose middle lies inside the gap and whose ends lie no further out than the gap and its width to either side. The jambs and
        // the faces of the walls on either side stand at the gap's ends and are not inside it. A line of this pair's own faces is none.
        let drawn_in_gap = |g0: f64, g1: f64| -> bool {
            let width = g1 - g0;
            alive.iter().any(|&i| {
                let sg = &segs[i];
                if why[i].is_some() || of.get(&i).is_some_and(|f| f == a || f == b) {
                    return false;
                }
                let (t0, t1) = (dot(fa.u, sg.a), dot(fa.u, sg.b));
                let mid = (t0 + t1) / 2.0;
                let (o0, o1) = (dot(fa.n, sg.a) - cc, dot(fa.n, sg.b) - cc);
                mid > g0 + CAP_TOL && mid < g1 - CAP_TOL && t0.min(t1) >= g0 - thick && t0.max(t1) <= g1 + thick && o0.abs() <= width && o1.abs() <= width
            })
        };
        // Groups of stretches: joints join, openings are kept, a long break in one line splits.
        let mut groups: Vec<(Iv, Vec<(f64, f64)>)> = Vec::new();
        let mut cur: (Iv, Vec<(f64, f64)>) = (vec![both[0]], Vec::new());
        for w in both.windows(2) {
            let (g0, g1) = (w[0].1, w[1].0);
            // The widest stretch of the break where neither line is drawn.
            let mut none = (0.0, 0.0);
            let mut from = g0;
            for &(x, y) in union.iter().filter(|&&(x, y)| y > g0 && x < g1) {
                if x > from && x.min(g1) - from > none.1 - none.0 {
                    none = (from, x.min(g1));
                }
                from = from.max(y);
            }
            if from < g1 && g1 - from > none.1 - none.0 {
                none = (from, g1);
            }
            if none.1 - none.0 > OPENING_MOST {
                groups.push(std::mem::replace(&mut cur, (vec![w[1]], Vec::new())));
                out.notes.push(format!("both lines of a wall stop for {} at {}, wider than any opening: taken as one wall ending and another beginning", inch_said(none.1 - none.0), place_said(shift(at(fa.u, fa.n, none.0, cc), origin))));
            } else if none.1 - none.0 > BARE_GAP_MOST && !drawn_in_gap(none.0, none.1) {
                // Wider than a door and nothing drawn in it (call 191 item 2): bare paper. No wall is laid across it.
                groups.push(std::mem::replace(&mut cur, (vec![w[1]], Vec::new())));
                out.notes.push(format!("both lines of a wall stop for {} at {}, wider than 8 ft with nothing drawn in it from jamb to jamb (no leaf, no swing, no line across between them): taken as one wall ending and another beginning, and no door is found there", inch_said(none.1 - none.0), place_said(shift(at(fa.u, fa.n, none.0, cc), origin))));
            } else if none.1 - none.0 > JOINT_MOST {
                cur.1.push(none);
                cur.0.push(w[1]);
            } else if g1 - g0 <= JOINT_MOST {
                if g1 - g0 > MERGE_GAP {
                    out.counts.joints += 1;
                }
                cur.0.push(w[1]);
            } else {
                groups.push(std::mem::replace(&mut cur, (vec![w[1]], Vec::new())));
                out.notes.push(format!("one line of a wall goes on for {} past the other, at {}: the wall is taken to end there", inch_said(g1 - g0), place_said(shift(at(fa.u, fa.n, g0, cc), origin))));
            }
        }
        groups.push(cur);
        let (lo, hi) = (0.0_f64.min(*d), 0.0_f64.max(*d));
        // Lines across a stretch, inside the wall's thickness: lines between the two faces, and a face
        // drawn again across the stretch, as a line of its own from jamb to jamb.
        let across = |s0: f64, s1: f64| -> usize {
            let inner = faces
                .iter()
                .enumerate()
                .filter(|(gi, g)| {
                    if gi == a || gi == b || cross(g.u, fa.u).abs() > ANGLE_PAIR {
                        return false;
                    }
                    let off = g.offset_from(fa, (s0 + s1) / 2.0);
                    off > lo + INCH / 16.0 && off < hi - INCH / 16.0 && covered(&g.along(fa, &g.iv), s0, s1) >= GLASS_SHARE * (s1 - s0)
                })
                .count();
            // A face drawn again across the stretch: a line of its own from one jamb to the other.
            let redrawn = [fa, fb].iter().filter(|f| f.along(fa, &f.raw).iter().any(|&(x, y)| (x - s0).abs() <= CAP_TOL && (y - s1).abs() <= CAP_TOL)).count();
            inner + redrawn
        };
        for (pieces, mut opens) in groups {
            let (g_lo, g_hi) = (pieces[0].0, pieces.last().unwrap().1);
            let cont = |t: f64| pieces.iter().any(|&(x, y)| t >= x + CAP_TOL && t <= y - CAP_TOL);
            // A line across within a thickness of the end of a piece, where the next piece or the opening begins, is the
            // jamb of that opening with the faces run on past it, as drafters draw them (call 186): it ends no wall.
            let jamb = |t: f64| pieces.iter().any(|&(x, y)| (x > g_lo + 1e-9 && (t - x).abs() < thick) || (y < g_hi - 1e-9 && (t - y).abs() < thick));
            let mine: Vec<f64> = crosses.iter().copied().filter(|&t| t > g_lo && t < g_hi && cont(t) && !jamb(t)).collect();
            // An opening's sides are its jambs where the faces run on past them: the nearest line across within a thickness
            // of where the faces stop.
            for o in opens.iter_mut() {
                let near_to = |at: f64| crosses.iter().copied().filter(|&t| (t - at).abs() > CAP_TOL && (t - at).abs() < thick).min_by(|p, q| (p - at).abs().total_cmp(&(q - at).abs()));
                if let Some(t) = near_to(o.0).filter(|&t| t < o.0) {
                    o.0 = t;
                }
                if let Some(t) = near_to(o.1).filter(|&t| t > o.1) {
                    o.1 = t;
                }
            }
            // Lines across where both lines go on: a window's jambs, or the end of a wall. (What is in `opens` so far is cut clean
            // through both lines; what is added below has its faces running on.)
            let n_cut = opens.len();
            let mut ends: Vec<f64> = Vec::new();
            let mut i = 0;
            while i < mine.len() {
                if i + 1 < mine.len() {
                    let (t0, t1) = (mine[i], mine[i + 1]);
                    if pieces.iter().any(|&(x, y)| t0 >= x && t1 <= y) && t1 - t0 > JOINT_MOST && t1 - t0 <= OPENING_MOST && across(t0, t1) >= 1 {
                        opens.push((t0, t1));
                        i += 2;
                        continue;
                    }
                }
                ends.push(mine[i]);
                i += 1;
            }
            // A line across ends a wall only if it stands alone: both walls it makes are long enough to be
            // walls and no other line across is nearer than that (hatching across a wall is a row of them).
            let mut bounds = vec![g_lo];
            for (ix, &t) in ends.iter().enumerate() {
                let before = if ix == 0 { g_lo } else { ends[ix - 1] };
                let after = ends.get(ix + 1).copied().unwrap_or(g_hi);
                if t - before >= min_len && after - t >= min_len {
                    bounds.push(t);
                } else {
                    out.counts.crosses_ignored += 1;
                }
            }
            bounds.push(g_hi);
            for w in bounds.windows(2) {
                let (s0, s1) = (w[0], w[1]);
                // Its whole line, not a piece of it between openings (call 186).
                let inside_it: Vec<&(f64, f64)> = pieces.iter().filter(|&&(x, y)| y > s0 + 1e-9 && x < s1 - 1e-9).collect();
                let whole = inside_it.iter().map(|p| p.1.min(s1)).fold(0.0, f64::max) - inside_it.iter().map(|p| p.0.max(s0)).fold(f64::MAX, f64::min);
                if whole < min_len {
                    out.counts.short_stretches += 1;
                    continue;
                }
                let mut ops: Vec<(f64, f64, bool, bool)> = opens
                    .iter()
                    .enumerate()
                    .filter(|&(_, &(x, y))| x >= s0 - 1e-9 && y <= s1 + 1e-9)
                    .map(|(ix, &(x, y))| (x, y, across(x, y) >= 2, ix < n_cut))
                    .collect();
                ops.sort_by(|p, q| p.0.total_cmp(&q.0));
                cands.push(Cand { a: *a, cc, thick, t0: s0, t1: s1, openings: ops, doubt: conflict[pi].clone(), loser: loser[pi], lines: layered[pi], joined: joined[pi] });
            }
        }
    }

    // 5. Carry ends to meet; which walls meet nothing.
    let nw = cands.len();
    let line_of = |c: &Cand| -> (P2, P2, P2) {
        let fa = &faces[c.a];
        (at(fa.u, fa.n, c.t0, c.cc), at(fa.u, fa.n, c.t1, c.cc), fa.u)
    };
    let mut ends: Vec<[P2; 2]> = Vec::with_capacity(nw);
    let mut t_ends: Vec<[f64; 2]> = Vec::with_capacity(nw);
    let mut meets: Vec<[bool; 2]> = vec![[false; 2]; nw];
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for i in 0..nw {
        let (p, q, ui) = line_of(&cands[i]);
        let (mut e, mut te) = ([p, q], [cands[i].t0, cands[i].t1]);
        for end in 0..2 {
            let me = if end == 0 { p } else { q };
            let mut best: Option<(f64, P2, usize)> = None;
            let mut touch: Option<usize> = None;
            for j in 0..nw {
                if j == i || cands[j].loser {
                    continue;
                }
                let (pj, qj, uj) = line_of(&cands[j]);
                let reach = cands[i].thick.max(cands[j].thick);
                let sinv = cross(ui, uj);
                if sinv.abs() < 0.087 {
                    // Going on in line: its end is at this one.
                    if dot(perp(uj), sub(me, pj)).abs() <= reach / 2.0 + CAP_TOL && norm(sub(me, pj)).min(norm(sub(me, qj))) <= reach {
                        touch = Some(j);
                    }
                    continue;
                }
                // Where this wall's line meets that one's.
                let s = cross(sub(pj, me), uj) / sinv;
                let x = [me[0] + ui[0] * s, me[1] + ui[1] * s];
                let along_j = dot(uj, sub(x, pj));
                if s.abs() <= reach && along_j >= -cands[i].thick && along_j <= norm(sub(qj, pj)) + cands[i].thick && best.map_or(true, |(bd, _, _)| s.abs() < bd) {
                    best = Some((s.abs(), x, j));
                }
            }
            if let Some((_, x, j)) = best {
                e[end] = x;
                te[end] = dot(ui, x);
                meets[i][end] = true;
                edges.push((i, j));
            } else if let Some(j) = touch {
                meets[i][end] = true;
                edges.push((i, j));
            }
        }
        ends.push(e);
        t_ends.push(te);
    }
    for i in 0..nw {
        if cands[i].doubt.is_none() && !meets[i][0] && !meets[i][1] {
            let len = norm(sub(ends[i][1], ends[i][0]));
            cands[i].doubt = Some(format!("two lines {} apart and {} long that meet no other wall at either end (a counter's edge, a beam shown, a stair's side)", inch_said(cands[i].thick).replace("0' ", ""), inch_said(len)));
        }
    }
    // The drawings of the sheet (call 188 item 6, call 191 items 1, 3 and 4), told from EVERY wall found, in doubt or not, so a plan whose
    // walls are all in doubt is still a drawing. Walls lying within `DRAWING_GAP` (8 ft: what could be an opening) of one another, by
    // the nearest points of their centre lines, are one drawing: a plan's pieces need not meet to belong together, and a roof plan stands
    // apart from the floor plan. The size of a drawing is all the wall found in it (a loser, a line already a wall's face, is not wall).
    fn root_c(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        r
    }
    let mut cl: Vec<usize> = (0..nw).collect();
    {
        let seg_gap = |i: usize, j: usize| -> f64 {
            let (a, b, c, d) = (ends[i][0], ends[i][1], ends[j][0], ends[j][1]);
            let to_seg = |p: P2, q0: P2, q1: P2| {
                let v = sub(q1, q0);
                let t = (dot(sub(p, q0), v) / dot(v, v).max(1e-300)).clamp(0.0, 1.0);
                norm(sub(p, [q0[0] + v[0] * t, q0[1] + v[1] * t]))
            };
            let crossing = {
                let (d1, d2) = (cross(sub(b, a), sub(c, a)), cross(sub(b, a), sub(d, a)));
                let (d3, d4) = (cross(sub(d, c), sub(a, c)), cross(sub(d, c), sub(b, c)));
                d1 * d2 < 0.0 && d3 * d4 < 0.0
            };
            if crossing {
                0.0
            } else {
                to_seg(a, c, d).min(to_seg(b, c, d)).min(to_seg(c, a, b)).min(to_seg(d, a, b))
            }
        };
        for i in 0..nw {
            for j in (i + 1)..nw {
                if seg_gap(i, j) <= DRAWING_GAP {
                    let (ra, rb) = (root_c(&mut cl, i), root_c(&mut cl, j));
                    cl[ra] = rb;
                }
            }
        }
    }
    // Each drawing: how much wall it holds and where its west edge lies; largest first, ties the westmost first.
    let mut by_size: Vec<(usize, f64, f64)> = {
        let mut sizes: BTreeMap<usize, (f64, f64)> = BTreeMap::new();
        for i in 0..nw {
            let r = root_c(&mut cl, i);
            let e = sizes.entry(r).or_insert((0.0, f64::MAX));
            if !cands[i].loser {
                e.0 += norm(sub(ends[i][1], ends[i][0]));
            }
            e.1 = e.1.min(ends[i][0][0]).min(ends[i][1][0]);
        }
        sizes.into_iter().filter(|(_, v)| v.0 > 0.0).map(|(r, v)| (r, v.0, v.1)).collect()
    };
    by_size.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.2.total_cmp(&b.2)));
    let largest = by_size.first().map_or(0.0, |v| v.1);
    let like_n = by_size.iter().filter(|v| v.1 >= LIKE_SIZE * largest - SHARE_EPS).count();
    // Wall found outside the largest drawing (set apart, or in other drawings), together at least half of it: nothing is laid unasked
    // (call 191 item 4). Where drawings of like size are listed that is said already; here every drawing is listed so each can be named.
    let outside_wall: f64 = by_size.iter().skip(1).map(|v| v.1).sum();
    let unsettled = by_size.len() >= 2 && like_n < 2 && outside_wall >= APART_SHARE * largest - SHARE_EPS;
    let listed: Vec<usize> = if like_n >= 2 { by_size.iter().take(like_n).map(|v| v.0).collect() } else if unsettled { by_size.iter().map(|v| v.0).collect() } else { Vec::new() };
    if unsettled {
        out.unsettled = Some((outside_wall, largest));
    }
    // The drawing the person took: the walls of that drawing and no others (call 191 item 1).
    let chosen_root: Option<usize> = match plan {
        None => None,
        Some(n) => {
            let have = listed.len().max(1);
            if n == 0 || n > have {
                return Err(format!("--plan {n}: this sheet has {have} {}, numbered 1 to {have}", if have == 1 { "drawing" } else { "drawings" }));
            }
            out.chosen = Some(n);
            listed.get(n - 1).copied()
        }
    };
    let scope: Vec<bool> = (0..nw).map(|i| chosen_root.is_none_or(|c| root_c(&mut cl, i) == c)).collect();
    // The plan is the largest group of walls that meet one another; a group far smaller than it, apart from it, is another plan, a
    // detail or a section (call 186, item 3). Where a drawing was taken, it is told within that drawing alone, and the rest by itself.
    let spheres: Vec<Vec<usize>> = if chosen_root.is_some() { vec![(0..nw).filter(|&i| scope[i]).collect(), (0..nw).filter(|&i| !scope[i]).collect()] } else { vec![(0..nw).collect()] };
    let mut group: Vec<usize> = (0..nw).collect();
    fn root_g(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        r
    }
    for &(a, b) in &edges {
        let (ra, rb) = (root_g(&mut group, a), root_g(&mut group, b));
        group[ra] = rb;
    }
    for members in &spheres {
        let mut total: BTreeMap<usize, f64> = BTreeMap::new();
        for &i in members {
            if !cands[i].loser {
                *total.entry(root_g(&mut group, i)).or_default() += norm(sub(ends[i][1], ends[i][0]));
            }
        }
        if let Some((&plan_root, &plan_len)) = total.iter().max_by(|a, b| a.1.total_cmp(b.1)) {
            for &i in members {
                let r = root_g(&mut group, i);
                let mine = total.get(&r).copied().unwrap_or(0.0);
                if r != plan_root && cands[i].doubt.is_none() && mine < APART_SHARE * plan_len - SHARE_EPS {
                    cands[i].doubt = Some(format!("apart from the plan: it meets no wall of the largest group of walls on the sheet ({} of wall), and the group it is in is {} long: another plan, a detail or a section", inch_said(plan_len), inch_said(mine)));
                }
            }
        }
    }
    // A pair thicker than every other wall found, and alone at that thickness, is in doubt (call 186, narrowed): a closet
    // shelf, a bench, a ledge. A thin partition is not, nor two thick walls, nor a sheet of one wall.
    for members in &spheres {
        for &i in members {
            if cands[i].doubt.is_some() || cands[i].loser {
                continue;
            }
            let others: Vec<f64> = members.iter().copied().filter(|&j| j != i && !cands[j].loser).map(|j| cands[j].thick).collect();
            if !others.is_empty() && others.iter().all(|&t| cands[i].thick >= t + THICK_MORE) {
                cands[i].doubt = Some(format!("two lines {} apart, thicker than every other wall found on the sheet ({} at most) and the only one that thick (a closet shelf, a bench, a ledge)", inch_said(cands[i].thick).replace("0' ", ""), inch_said(others.iter().cloned().fold(0.0, f64::max)).replace("0' ", "")));
            }
        }
    }

    // A window drawn by lines across a stretch of a wall whose two faces run on past it, in a wall with a room on both sides, is no
    // window (call 188 item 8): a shower's glass, a pocket or sliding door's leaf drawn in a partition. A window is in a wall with a
    // face to the outside, or one cut clean through. The outside is told by looking straight out from the stretch on both sides: where
    // one side meets no wall of the plan, the wall has a face to the outside.
    let roots: Vec<usize> = (0..nw).map(|i| root_g(&mut group, i)).collect();
    for i in 0..nw {
        if cands[i].openings.iter().all(|o| !o.2 || o.3) {
            continue;
        }
        let fa = &faces[cands[i].a];
        let hit = |p: P2, dir: P2| -> bool {
            (0..nw).any(|j| {
                if j == i || cands[j].loser || cands[j].doubt.is_some() || roots[j] != roots[i] {
                    return false;
                }
                let (q0, q1) = (ends[j][0], ends[j][1]);
                let d = sub(q1, q0);
                let len = norm(d);
                if len < 1e-9 {
                    return false;
                }
                let uj = [d[0] / len, d[1] / len];
                let den = cross(dir, uj);
                if den.abs() < 0.087 {
                    return false;
                }
                let w = sub(q0, p);
                let (sd, td) = (cross(w, uj) / den, cross(w, dir) / den);
                sd > cands[i].thick / 2.0 && td >= -cands[j].thick && td <= len + cands[j].thick
            })
        };
        let mut kept = Vec::new();
        for o in cands[i].openings.clone() {
            if o.2 && !o.3 {
                let p = at(fa.u, fa.n, (o.0 + o.1) / 2.0, cands[i].cc);
                let outside = !hit(p, fa.n) || !hit(p, [-fa.n[0], -fa.n[1]]);
                if !outside {
                    out.counts.glass_inside += 1;
                    out.notes.push(format!("lines across a stretch {} wide of a wall at {}, and lines inside it, are not read as a window: both its faces run on past it and it has a room on each side (a shower's glass, or a sliding or pocket door's leaf drawn in the wall); a window is in a wall with a face to the outside, or one cut clean through", inch_said(o.1 - o.0), place_said(shift(p, origin))));
                    continue;
                }
            }
            kept.push(o);
        }
        cands[i].openings = kept;
    }

    // 6. The walls, each from its first end: the west end of one that runs east and west, else the south.
    let mid_of = |i: usize| [(ends[i][0][0] + ends[i][1][0]) / 2.0, (ends[i][0][1] + ends[i][1][1]) / 2.0];
    let mut order: Vec<usize> = (0..nw).collect();
    order.sort_by(|&x, &y| {
        let (mx, my) = (mid_of(x), mid_of(y));
        (mx[1] * 100.0).round().total_cmp(&(my[1] * 100.0).round()).then((mx[0] * 100.0).round().total_cmp(&(my[0] * 100.0).round()))
    });
    let mut new_index = vec![0usize; nw];
    for (rank, &i) in order.iter().enumerate() {
        new_index[i] = rank;
        let c = &cands[i];
        let (mut p, mut q) = (ends[i][0], ends[i][1]);
        let [t0, t1] = t_ends[i];
        let d = sub(q, p);
        let flip = if d[0].abs() >= d[1].abs() { d[0] < 0.0 } else { d[1] < 0.0 };
        // `t` grows from the unflipped first end; a flipped wall is measured from the other end.
        let mut ops: Vec<Opening> = c.openings.iter().map(|&(x, y, w, _)| if flip { Opening { window: w, from: t1 - y, to: t1 - x } } else { Opening { window: w, from: x - t0, to: y - t0 } }).collect();
        if flip {
            std::mem::swap(&mut p, &mut q);
        }
        ops.sort_by(|x, y| x.from.total_cmp(&y.from));
        out.walls.push(Wall { from: p, to: q, thickness: c.thick, openings: ops, doubt: c.doubt.clone(), lines: c.lines, joined: c.joined });
    }
    // The drawings listed (the clusters above), each with its walls, its size and how much of it is in doubt.
    if !listed.is_empty() {
        for &root in &listed {
            let mut members: Vec<usize> = (0..nw).filter(|&i| root_c(&mut cl, i) == root).collect();
            members.sort_by_key(|&i| new_index[i]);
            let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
            let (mut length, mut in_doubt) = (0.0, 0.0);
            for &i in &members {
                for e in ends[i] {
                    for d in 0..2 {
                        lo[d] = lo[d].min(e[d]);
                        hi[d] = hi[d].max(e[d]);
                    }
                }
                if !cands[i].loser {
                    let l = norm(sub(ends[i][1], ends[i][0]));
                    length += l;
                    if cands[i].doubt.is_some() {
                        in_doubt += l;
                    }
                }
            }
            out.drawings.push(Plan { walls: members.iter().map(|&i| new_index[i]).collect(), length, in_doubt, corners: [lo, hi] });
        }
    }
    // An end that meets nothing is open, unless the page cut the lines off there: where a clip or the page's own edge ends
    // what shows, the wall may go on beyond it (call 186).
    let cuts: Vec<P2> = extra.cuts.iter().map(|p| [p[0] * k, p[1] * k]).collect();
    // (A wall in doubt that runs to the edge of what shows is said to be cut off as well: what is said of the walls closing
    // rests on what they meet.)
    for i in 0..nw {
        for end in 0..2 {
            if !meets[i][end] {
                let at_cut = cuts.iter().any(|c| norm(sub(*c, ends[i][end])) <= cands[i].thick + CAP_TOL);
                if at_cut {
                    out.cut_ends.push((new_index[i], ends[i][end]));
                } else if cands[i].doubt.is_none() {
                    out.open_ends.push((new_index[i], ends[i][end]));
                }
            }
        }
    }
    out.open_ends.sort_by(|x, y| x.0.cmp(&y.0));
    out.cut_ends.sort_by(|x, y| x.0.cmp(&y.0));

    // 7. What each line was, counted by kind.
    for i in 0..n_segs {
        match why[i] {
            Some(Why::Dimension) => out.counts.dimension += 1,
            Some(Why::Border) => out.counts.border += 1,
            Some(Why::Boxed) => out.counts.boxed += 1,
            Some(Why::Set) => out.counts.set += 1,
            Some(Why::Tag) => {}
            None => {
                let s = &segs[i];
                let m = [(s.a[0] + s.b[0]) / 2.0, (s.a[1] + s.b[1]) / 2.0];
                let in_wall = out.walls.iter().any(|w| {
                    let u = unit(sub(w.to, w.from));
                    let (t, o) = (dot(u, sub(m, w.from)), dot(perp(u), sub(m, w.from)));
                    t >= -w.thickness && t <= w.length() + w.thickness && o.abs() <= w.thickness / 2.0 + CAP_TOL
                });
                if in_wall {
                    out.counts.in_walls += 1;
                } else if s.len < min_len {
                    out.counts.small += 1;
                } else {
                    out.counts.single += 1;
                }
            }
        }
    }
    // Long lines left over that meet end to end in an outline (a house or a room drawn with one line a wall): a thickness
    // cannot be read from one line, and it is said (call 186).
    {
        let single: Vec<usize> = (0..n_segs)
            .filter(|&i| why[i].is_none() && segs[i].len >= min_len && {
                let m = [(segs[i].a[0] + segs[i].b[0]) / 2.0, (segs[i].a[1] + segs[i].b[1]) / 2.0];
                !out.walls.iter().any(|w| {
                    let u = unit(sub(w.to, w.from));
                    let (t, o) = (dot(u, sub(m, w.from)), dot(perp(u), sub(m, w.from)));
                    t >= -w.thickness && t <= w.length() + w.thickness && o.abs() <= w.thickness / 2.0 + CAP_TOL
                })
            })
            .collect();
        let cell = CAP_TOL * 2.0;
        let key = |p: P2| ((p[0] / cell).round() as i64, (p[1] / cell).round() as i64);
        let mut at_key: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for (n, &i) in single.iter().enumerate() {
            for e in [segs[i].a, segs[i].b] {
                let (kx, ky) = key(e);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        at_key.entry((kx + dx, ky + dy)).or_default().push(n);
                    }
                }
            }
        }
        let mut group: Vec<usize> = (0..single.len()).collect();
        fn root_of(g: &mut [usize], i: usize) -> usize {
            let mut r = i;
            while g[r] != r {
                r = g[r];
            }
            r
        }
        for (n, &i) in single.iter().enumerate() {
            for e in [segs[i].a, segs[i].b] {
                for &m in at_key.get(&key(e)).map(|v| v.as_slice()).unwrap_or(&[]) {
                    let j = single[m];
                    if m != n && [segs[j].a, segs[j].b].iter().any(|f| norm(sub(*f, e)) <= CAP_TOL) {
                        let (ra, rb) = (root_of(&mut group, n), root_of(&mut group, m));
                        group[ra] = rb;
                    }
                }
            }
        }
        let mut comps: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for n in 0..single.len() {
            let r = root_of(&mut group, n);
            comps.entry(r).or_default().push(single[n]);
        }
        for members in comps.values().filter(|m| m.len() >= 3) {
            let c = members.iter().fold([0.0, 0.0], |m, &j| [m[0] + (segs[j].a[0] + segs[j].b[0]) / 2.0, m[1] + (segs[j].a[1] + segs[j].b[1]) / 2.0]);
            out.single_outlines.push((members.len(), shift([c[0] / members.len() as f64, c[1] / members.len() as f64], origin)));
        }
        // Said only of a sheet that found no wall: where walls were found, a north arrow or a symbol is an outline of lines
        // end to end too, and the walls are not drawn as single lines.
        if out.walls.iter().any(|w| w.doubt.is_none()) {
            out.single_outlines.clear();
        }
        out.single_outlines.sort_by(|a, b| a.1[1].total_cmp(&b.1[1]).then(a.1[0].total_cmp(&b.1[0])));
    }
    // Short pieces that run on end to end and turn a little at each joint: a curve drawn in straight pieces.
    {
        let small: Vec<usize> = (0..n_segs)
            .filter(|&i| why[i].is_none() && segs[i].len < min_len && {
                let m = [(segs[i].a[0] + segs[i].b[0]) / 2.0, (segs[i].a[1] + segs[i].b[1]) / 2.0];
                !out.walls.iter().any(|w| {
                    let u = unit(sub(w.to, w.from));
                    let (t, o) = (dot(u, sub(m, w.from)), dot(perp(u), sub(m, w.from)));
                    t >= -w.thickness && t <= w.length() + w.thickness && o.abs() <= w.thickness / 2.0 + CAP_TOL
                })
            })
            .collect();
        let cell = INCH / 2.0;
        let key = |p: P2| ((p[0] / cell).round() as i64, (p[1] / cell).round() as i64);
        let mut at_key: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for &i in &small {
            at_key.entry(key(segs[i].a)).or_default().push(i);
            at_key.entry(key(segs[i].b)).or_default().push(i);
        }
        let mut seen: std::collections::HashSet<usize> = Default::default();
        for &i in &small {
            if seen.contains(&i) {
                continue;
            }
            // The run through this piece, followed out from it both ways while each joint has exactly two pieces.
            let mut run: Vec<usize> = vec![i];
            seen.insert(i);
            let mut smooth = true;
            for end in 0..2 {
                let (mut cur, mut p) = (i, if end == 0 { segs[i].a } else { segs[i].b });
                loop {
                    let here: Vec<usize> = at_key.get(&key(p)).map(|v| v.iter().copied().filter(|&j| j != cur).collect()).unwrap_or_default();
                    if here.len() != 1 || seen.contains(&here[0]) {
                        break;
                    }
                    let n = here[0];
                    let (da, db) = (unit(sub(segs[cur].b, segs[cur].a)), unit(sub(segs[n].b, segs[n].a)));
                    let turn = dot(da, db).abs().clamp(0.0, 1.0).acos().to_degrees();
                    if turn > CURVE_TURN {
                        smooth = false;
                        break;
                    }
                    seen.insert(n);
                    run.push(n);
                    let far = if norm(sub(segs[n].a, p)) <= norm(sub(segs[n].b, p)) { segs[n].b } else { segs[n].a };
                    p = far;
                    cur = n;
                }
            }
            let total: f64 = run.iter().map(|&j| segs[j].len).sum();
            if smooth && run.len() >= CURVE_PIECES && total >= CURVE_LONG {
                let c = run.iter().fold([0.0, 0.0], |m, &j| [m[0] + (segs[j].a[0] + segs[j].b[0]) / 2.0, m[1] + (segs[j].a[1] + segs[j].b[1]) / 2.0]);
                out.curved.push((run.len(), total, shift([c[0] / run.len() as f64, c[1] / run.len() as f64], origin)));
            }
        }
    }
    // A drawing taken: its walls and no others (call 191 item 1). The rest of the sheet was told as it was, so what is counted is the same;
    // the walls of the other drawings go, and how many is said.
    if chosen_root.is_some() {
        let mut renum: Vec<Option<usize>> = vec![None; nw];
        let mut kept: Vec<Wall> = Vec::new();
        for (rank, w) in std::mem::take(&mut out.walls).into_iter().enumerate() {
            if scope[order[rank]] {
                renum[rank] = Some(kept.len());
                kept.push(w);
            } else {
                out.other_walls.0 += 1;
                out.other_walls.1 += w.length();
            }
        }
        out.walls = kept;
        for e in out.open_ends.iter_mut().chain(out.cut_ends.iter_mut()) {
            e.0 = renum[e.0].unwrap_or(usize::MAX);
        }
        out.open_ends.retain(|e| e.0 != usize::MAX);
        out.cut_ends.retain(|e| e.0 != usize::MAX);
    }
    // Places on the plan.
    for w in out.walls.iter_mut() {
        w.from = shift(w.from, origin);
        w.to = shift(w.to, origin);
    }
    for e in out.open_ends.iter_mut().chain(out.cut_ends.iter_mut()) {
        e.1 = shift(e.1, origin);
    }
    for st in out.sets.iter_mut() {
        st.at = shift(st.at, origin);
    }
    for p in out.drawings.iter_mut() {
        p.corners = [shift(p.corners[0], origin), shift(p.corners[1], origin)];
    }
    for b in out.boxes.iter_mut() {
        b.at = shift(b.at, origin);
    }
    Ok(out)
}

// ---- the sheet's own dimension strings (call 188 item 9) ----
