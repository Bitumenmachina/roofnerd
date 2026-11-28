use super::*;

/// What a run of cabinet boxes came to (call 155).
#[derive(Clone, Debug, PartialEq)]
pub struct Cabinets {
    /// Each box set: its path, its code, the middle of its base on the plan, and how wide it is.
    pub boxes: Vec<(String, String, P2, f64)>,
    /// The countertop over the base boxes, with the middle of its base on the plan, if one was asked for.
    pub countertop: Option<(String, P2)>,
    /// Where the run starts along the wall's line from its first end, how long it is, and
    /// how much of the wall's face is left beyond it to its corner; all in feet.
    pub from: f64,
    pub run: f64,
    pub left: f64,
    /// The boxes hang on the wall (they are wall boxes), this far up in feet.
    pub hung: Option<f64>,
    /// Each box a rule made because the tool chest had not its code (call 215), said in full:
    /// which rule, its sizes, and a width the trade does not stock.
    pub made: Vec<String>,
}

/// The box codes a job's tool chest has: its furnishings of Division 12 named by one word of
/// letters then figures (B18, SB36, W3030, T1884, F3).
pub fn box_codes(job: &mut Job) -> Vec<(String, String)> {
    let mut q = job.world.query::<(&Node, &Item)>();
    let mut found: Vec<(u64, String, String)> = q
        .iter(&job.world)
        .filter(|(n, it)| {
            let (letters, rest) = n.name.split_at(n.name.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(n.name.len()));
            it.category == Kind::Furnishing && it.division == crate::components::Division::of_number(12) && !letters.is_empty() && !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
        })
        .map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone(), n.name.clone()))
        .collect();
    found.sort();
    found.into_iter().map(|(_, path, name)| (path, name)).collect()
}

/// Boxes named in a list: words separated by spaces, each an item's code (B18, SB36) or,
/// in square brackets, any item of the job by its name, which may have a space or a comma
/// in it: `B18 [Dishwasher] SB36 [Range, 30 in]` is four boxes (call 157).
pub fn box_list(text: &str) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = text.trim_start();
    while !rest.is_empty() {
        if let Some(inner) = rest.strip_prefix('[') {
            let Some(close) = inner.find(']') else {
                return Err(format!("a name in square brackets is not closed: [{}", inner.trim_end()));
            };
            let name = inner[..close].trim();
            if name.is_empty() {
                return Err("a pair of square brackets has no name in it".into());
            }
            out.push(name.to_string());
            rest = inner[close + 1..].trim_start();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            if rest[..end].contains('[') || rest[..end].contains(']') {
                return Err(format!("{:?} has a square bracket in the middle of it: a name with a space or a comma goes whole in brackets, like [Range, 30 in]", &rest[..end]));
            }
            out.push(rest[..end].to_string());
            rest = rest[end..].trim_start();
        }
    }
    Ok(out)
}

/// The most a thing can stand and still have a countertop laid over it: a base box's height.
pub(super) const COUNTER_HEIGHT_FT: f64 = 34.5 / 12.0;

/// A run of cabinet boxes set side by side along the face `wall` has on `room`'s side,
/// backs to the wall, from `from` along the wall's line from its first end, each turned to
/// the wall. Base and tall boxes stand on the floor; wall boxes hang 54 in up (Assumed).
/// An item that is no box (a dishwasher, a range, named in the list by its name) stands on
/// the floor, back to the wall, as a base box does. Each is one of its item, as sure as
/// the item, and carries one cost line (its item's name, 1 EA, no price). With
/// `countertop`, one countertop over the base boxes and whatever else stands no taller than
/// they do: as long as they are, 25 1/2 in deep, 1 1/2 in thick, 34 1/2 in up (Assumed).
/// Refused, the job as it was: a code the tool chest has not, a wall that is to demolish, a
/// run past the room's corner or starting inside it, a run across a door, a box across a
/// window it would stand in front of (a wall box or a tall box; a base box where the sill is
/// below its top), a box that would stand in another thing set in place (call 157).
#[allow(clippy::too_many_arguments)]
pub fn cabinets(job: &mut Job, storey: &str, wall: &str, room: &str, from: &str, codes: &[String], countertop: bool, how: Entered) -> Result<Cabinets, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if codes.is_empty() {
            return Err("a run needs boxes: give their codes, like \"B18 SB36 B24\"".into());
        }
        let tol = crate::check::FIT_TOLERANCE_FT;
        let ft = |x: f64| units::feet_as_ft_in(x, 16);
        let start = how.length(from)?.feet().ok_or("where the run starts is not a length")?;
        if start < 0.0 {
            return Err("a run starts so far along the wall from its first end: that is not less than nothing".into());
        }
        // The boxes, by their codes (or any item by its name), with their sizes.
        struct Boxed {
            path: String,
            name: String,
            size: P3,
            /// A box of the tool chest, named by a code: its rule says where it stands.
            code: bool,
            /// How far up its foot hangs, in feet, by its rule (call 215): None, it stands on the floor.
            hang: Option<f64>,
            division: Option<crate::components::Division>,
        }
        let rules = building::cabinet::effective(&job.cabinet_rules);
        let mut chest = box_codes(job);
        let mut made: Vec<String> = Vec::new();
        let mut boxes: Vec<Boxed> = Vec::new();
        for code in codes {
            let named = chest.iter().find(|(_, name)| name.eq_ignore_ascii_case(code.trim())).map(|(p, n)| (p.clone(), n.clone(), true)).or_else(|| {
                let mut q = job.world.query::<(&Node, &Item)>();
                let mut all: Vec<(u64, String, String)> = q.iter(&job.world).filter(|(n, _)| n.name.eq_ignore_ascii_case(code.trim())).map(|(n, _)| (n.path.rsplit('-').next().and_then(|x| x.parse().ok()).unwrap_or(u64::MAX), n.path.clone(), n.name.clone())).collect();
                all.sort();
                all.into_iter().next().map(|(_, p, n)| (p, n, false))
            });
            let named = match named {
                Some(found) => Some(found),
                // A code the chest has not is made by its rule, when a rule reads it (call 215).
                None => match building::cabinet::ask(&rules, code) {
                    building::cabinet::Asked::Made(m) => {
                        let path = item(job, &m.name, m.item.clone())?;
                        chest.push((path.clone(), m.name.clone()));
                        made.push(m.said.clone());
                        Some((path, m.name, true))
                    }
                    building::cabinet::Asked::Refused(why) => {
                        let have = chest.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>().join(" ");
                        return Err(if have.is_empty() { format!("{why}. It has no boxes: bring the starter items in first") } else { format!("{why}. It has: {have}") });
                    }
                    building::cabinet::Asked::NotACode => None,
                },
            };
            let Some((path, name, code)) = named else {
                let have = chest.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>().join(" ");
                return Err(if have.is_empty() { format!("{code} is not in the tool chest, and it has no boxes: bring the starter items in first") } else { format!("{code} is not in the tool chest. It has: {have}") });
            };
            let e = job.find(&path).ok_or_else(|| format!("{path} is not in the job"))?;
            let it = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{path} is not an item"))?;
            let (size, _, _) = it.size_ft().map_err(|why| format!("{name}: {why}, so it cannot be set"))?;
            // Where a box stands is its rule's, by the letters of its code; a box no rule reads stands on the floor (call 215).
            let hang = if code { building::cabinet::hung_ft(&rules, &name) } else { None };
            boxes.push(Boxed { path, name, size, code, hang, division: it.division });
        }
        let hangs = |b: &Boxed| b.hang.is_some();
        // What a countertop goes over: what is not hung and stands no taller than a base box.
        let covered = |b: &Boxed| !hangs(b) && b.size[2] <= COUNTER_HEIGHT_FT + tol;
        let wall_boxes = boxes.iter().filter(|b| hangs(b)).count();
        if wall_boxes != 0 && wall_boxes != boxes.len() {
            return Err("wall boxes hang over the counter and base boxes stand under it: set them as two runs".into());
        }
        let hung = boxes.iter().find_map(|b| b.hang);
        if let Some(first) = hung {
            if boxes.iter().any(|b| b.hang.is_some_and(|h| (h - first).abs() > tol)) {
                return Err("the wall boxes of one run hang at one height, and these hang at different heights by their rules: set them as two runs".into());
            }
        }
        if countertop && hung.is_some() {
            return Err("a countertop goes over base boxes, and a run of wall boxes has none".into());
        }
        if countertop && !boxes.iter().any(&covered) {
            return Err("a countertop goes over base boxes, and this run has none".into());
        }
        // A wall that is to demolish is no wall to set boxes against.
        if crate::components::demolished(&mut job.world).contains(wall) {
            let called = job.find(wall).and_then(|e| job.world.get::<Node>(e)).map(|n| format!("{} ({wall})", n.name)).unwrap_or_else(|| wall.to_string());
            return Err(format!("{called} is to demolish: boxes set against a wall that is coming down would be left standing with nothing behind them. Set the run along a wall that stays"));
        }
        let face = building::wall_face(job, storey, wall, room)?;
        let run: f64 = boxes.iter().map(|b| b.size[0]).sum();
        let end = start + run;
        let Some(&(_, hi)) = face.clear.iter().find(|(lo, hi)| start >= lo - tol && start <= hi + tol) else {
            let said = face.clear.iter().map(|(lo, hi)| format!("{} to {}", ft(*lo), ft(*hi))).collect::<Vec<_>>().join(", ");
            return Err(format!("{} does not have a clear face at {} along it: its face on {} runs clear from {said}", face.wall_called, ft(start), face.room_called));
        };
        if end > hi + tol {
            return Err(format!("a run of {} from {} reaches {} along {}, {} past the corner of {} at {}", ft(run), ft(start), ft(end), face.wall_called, ft(end - hi), face.room_called, ft(hi)));
        }
        if let Some((door, a, b)) = face.doors.iter().find(|(_, a, b)| start < *b - tol && *a < end - tol) {
            return Err(format!("a run of {} from {} crosses {door}, which is in {} from {} to {}: a door is no place for boxes", ft(run), ft(start), face.wall_called, ft(*a), ft(*b)));
        }
        // A window the box would stand in front of: it reaches from the floor (or from where
        // it hangs) up to its top, and the window from its sill up to its head.
        let mut at = start;
        for b in &boxes {
            let (low, high) = (hung.unwrap_or(0.0), hung.unwrap_or(0.0) + b.size[2]);
            for (window, called, from, to, sill, head) in &face.windows {
                if at < *to - tol && *from < at + b.size[0] - tol && low < *head - tol && *sill < high - tol {
                    let hangs_or_stands = if hung.is_some() { "hangs" } else { "stands" };
                    return Err(format!(
                        "{} {hangs_or_stands} from {} to {} along {} and from {} up to {} high, across {called} ({window}), which is in {} from {} to {} and from {} up to {} high{}",
                        b.name,
                        ft(at),
                        ft(at + b.size[0]),
                        face.wall_called,
                        ft(low),
                        ft(high),
                        face.wall_called,
                        ft(*from),
                        ft(*to),
                        ft(*sill),
                        ft(*head),
                        if hung.is_none() && covered(b) && *sill > low + tol { format!(": a base box fits under a window only where the sill is at or above its top ({}), and this sill is lower", ft(high)) } else { if hung.is_some() { ": a wall box does not hang across a window".to_string() } else { ": a tall box does not stand in front of a window".to_string() } }
                    ));
                }
            }
            at += b.size[0];
        }
        // Each box's middle on the plan: along the wall, then back from the face into the room.
        let (u, into) = (face.along, face.into);
        let spot = |along: f64, out: f64| -> P2 { [face.start[0] + u[0] * along + into[0] * out, face.start[1] + u[1] * along + into[1] * out] };
        // Turned so its back (the +y of its own frame) is against the wall.
        let turned = {
            let mut a = into[0].atan2(-into[1]).to_degrees();
            if a <= -180.0 {
                a += 360.0;
            }
            (a * 1.0e6).round() / 1.0e6
        };
        let sure = how.status.max(face.status);
        let by = format!("{}, {} along {} against {} ({})", "set in a run", ft(start), face.wall_called, face.room_called, how.source);
        let assumed = Entered { status: how.status.max(Status::Assumed), source: "" };
        let to_wall = |job: &mut Job, path: &str| -> Result<(), String> {
            let e = job.find(path).ok_or_else(|| format!("no element {path}"))?;
            let mut p = job.world.get::<Placed>(e).cloned().ok_or_else(|| format!("{path} has no position"))?;
            p.angle = if turned == 0.0 && sure == Status::Known { None } else { Some(Val { value: Some(turned), unit: "deg".into(), status: sure, source: format!("turned to {}, the wall it backs onto", face.wall_called) }) };
            building::check_placed(path, job.kind(path).unwrap_or(Kind::Furnishing), &p)?;
            job.put(path, p)
        };
        let mut set: Vec<(String, String, P2, f64)> = Vec::new();
        let mut along = start;
        let mut base_span: Option<(f64, f64)> = None;
        for b in &boxes {
            let (item, code, size) = (&b.path, &b.name, b.size);
            let mid = spot(along + size[0] / 2.0, face.off + size[1] / 2.0);
            let offset = match hung {
                Some(up) => Val { value: Some(units::Len::One(up)), unit: "ft".into(), status: assumed.status, source: format!("wall boxes hang with their bottoms {} up, {} over a 36 in counter; the maker's catalog decides", units::inches_said(up), units::inches_said(up - 3.0)) },
                None => how.length("0 ft")?,
            };
            let (path, _) = place_item_in(job, storey, item, mid, offset, Entered { status: sure, source: &by })?;
            to_wall(job, &path)?;
            job.add_line(&path, code, "1", Some(crate::Class::Material), false)?;
            job.set_division(&path, Some(code), b.division.or(crate::components::Division::of_number(12)).ok_or("Division 12 is not there")?)?;
            if covered(b) {
                base_span = Some(match base_span {
                    Some((a, _)) => (a, along + size[0]),
                    None => (along, along + size[0]),
                });
            }
            set.push((path, code.clone(), mid, size[0]));
            along += size[0];
        }
        // A box that stands in another thing set in place (a box of another run, an
        // appliance) is refused: the clash test is the one `clashes` reports with, asked of
        // the job as it now stands, and only what a box of this run is in is looked at.
        let mine: std::collections::HashSet<&str> = set.iter().map(|(p, _, _, _)| p.as_str()).collect();
        if let Ok(found) = building::clash::clashes(job, None) {
            let is_set_thing = |k: Kind| matches!(k, Kind::Furnishing | Kind::Equipment | Kind::Fixture);
            for c in found.found.iter().filter(|c| c.how == building::clash::ClashKind::Hard && !c.through) {
                let (this, other_name, other_kind, other_path) = match (mine.contains(c.a.as_str()), mine.contains(c.b.as_str())) {
                    (true, false) => (&c.a, &c.b_name, c.b_kind, &c.b),
                    (false, true) => (&c.b, &c.a_name, c.a_kind, &c.a),
                    _ => continue,
                };
                if !is_set_thing(other_kind) {
                    continue;
                }
                let name = set.iter().find(|(p, _, _, _)| p == this).map(|(_, code, _, _)| code.clone()).unwrap_or_default();
                return Err(format!(
                    "{name} would stand in {other_name} ({other_path}){}: set the run clear of it. A run that turns an inside corner starts off the corner by what the other run fills (24 in for base boxes)",
                    c.by.map(|d| format!(" by {} in", crate::view::num((d * 12.0 * 100.0).round() / 100.0))).unwrap_or_default()
                ));
            }
        }
        // The countertop over the base boxes.
        let top = if countertop {
            let (a, b) = base_span.ok_or("a countertop goes over base boxes, and this run has none")?;
            if boxes.iter().any(|x| !covered(x) && !hangs(x)) {
                let mut at = start;
                let mut between: Option<&Boxed> = None;
                for x in &boxes {
                    if !covered(x) && at > a + tol && at + x.size[0] < b - tol {
                        between = Some(x);
                    }
                    at += x.size[0];
                }
                if let Some(x) = between {
                    return Err(if x.code { "a tall box stands in the middle of the base boxes, and a countertop does not go over it: set the countertop by hand".to_string() } else { format!("{} stands taller than a countertop in the middle of the base boxes, and a countertop does not go over it: set the countertop by hand", x.name) });
                }
            }
            let (long, deep, thick) = (b - a, 25.5 / 12.0, 1.5 / 12.0);
            let mid = spot((a + b) / 2.0, face.off + deep / 2.0);
            let said = Entered { status: assumed.status, source: "a countertop over the base boxes: as long as they are, 25 1/2 in deep (a 1 1/2 in overhang at the front), 1 1/2 in thick, 34 1/2 in up; the maker's catalog decides" };
            let path = add_called(job, storey, Kind::Furnishing, "Countertop", 2, true)?;
            let placed = Placed { what: how.val("Countertop".to_string(), ""), at: Entered { status: sure, source: &by }.val(mid, "ft"), offset: Val { value: Some(units::Len::One(34.5 / 12.0)), unit: "ft".into(), status: said.status, source: said.source.into() }, angle: None, size: said.val([long, deep, thick], "ft"), item: None };
            building::check_placed(&path, Kind::Furnishing, &placed)?;
            job.put(&path, placed)?;
            to_wall(job, &path)?;
            // It carries its own line, by its area, to be priced (call 162); `price-item
            // Countertop` finds countertops by what they are.
            job.add_line(&path, "Countertop", "plan_area", Some(crate::Class::Material), false)?;
            job.set_division(&path, Some("Countertop"), crate::components::Division::of_number(12).ok_or("Division 12 is not there")?)?;
            Some((path, mid))
        } else {
            None
        };
        Ok(Cabinets { boxes: set, countertop: top, from: start, run, left: hi - end, hung, made })
    })
}
