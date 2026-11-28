//! Cut from `editing.rs`. PORTED CODE in part, as `editing.rs` says: the rules of the verbs are BIMCraft's,
//! `crates/core/src/commands.rs` at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (MIT); `PORTMAP.md` has each item.

use super::*;

/// How a floor's outline is given.
pub enum FloorBy<'a> {
    /// The outline the storey's walls enclose.
    Walls,
    /// Its corners, in feet on the plan.
    Points(&'a [P2]),
}

/// What `floor` made, and what whoever asked for it should know.
#[derive(Clone, Debug, PartialEq)]
pub struct Floored {
    pub path: String,
    pub notes: Vec<String>,
}

/// A floor of a type on a storey (BIMCraft `floor`, `commands.rs:7556`): by its corners,
/// or by the outline the storey's walls enclose. By its walls it is no surer than the
/// least sure of them, and its source says it is their outline. A floor on an outline a
/// floor of the storey already has is refused: it would be counted twice.
pub fn floor(job: &mut Job, storey: &str, type_path: &str, by: FloorBy, how: Entered) -> Result<Floored, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let called = job.find(storey).and_then(|e| job.world.get::<Node>(e).map(|n| n.name.clone())).unwrap_or_else(|| storey.to_string());
        if job.find(type_path).and_then(|e| job.world.get::<crate::FloorType>(e)).is_none() {
            return Err(format!("{type_path} is not a floor type"));
        }
        let mut notes = Vec::new();
        let boundary = match by {
            FloorBy::Points(points) => {
                if points.len() < 3 {
                    return Err("a floor's outline has at least three corners".into());
                }
                how.val(points.to_vec(), "ft")
            }
            FloorBy::Walls => {
                let found = building::floor_outline(job, storey)?.ok_or_else(|| format!("no walls enclose an area on {called}: draw the walls round the floor first, or give the floor by its corners"))?;
                if found.courtyards > 0 {
                    return Err(format!(
                        "the walls on {called} enclose {} inside the floor's outline, and a floor here has no hole yet: give the floor by its corners, round {}",
                        if found.courtyards == 1 { "a courtyard".to_string() } else { format!("{} courtyards", found.courtyards) },
                        if found.courtyards == 1 { "it" } else { "them" }
                    ));
                }
                if found.others > 0 {
                    notes.push(format!("the walls on {called} enclose {} apart from this one, and this floor is the largest only: each of the others is a floor of its own, given by its corners", if found.others == 1 { "another outline".to_string() } else { format!("{} other outlines", found.others) }));
                }
                Val { value: Some(found.boundary), unit: "ft".into(), status: how.status.max(found.status), source: format!("the outline of the walls on {called} ({})", how.source) }
            }
        };
        // A floor already on this very outline.
        let same = |a: &[P2], b: &[P2]| a.len() == b.len() && (0..a.len()).any(|shift| (0..a.len()).all(|i| (a[i][0] - b[(i + shift) % b.len()][0]).abs() < 1.0e-3 && (a[i][1] - b[(i + shift) % b.len()][1]).abs() < 1.0e-3));
        let mine = boundary.value.clone().unwrap_or_default();
        let there = {
            let mut q = job.world.query::<(&Node, &crate::Floor)>();
            q.iter(&job.world).find(|(n, f)| n.parent.as_deref() == Some(storey) && f.boundary_ft().is_some_and(|b| same(&b, &mine))).map(|(n, _)| n.name.clone())
        };
        if let Some(there) = there {
            return Err(format!("{there} is already on this outline on {called}: a second floor there would be counted twice"));
        }
        let floor = crate::Floor { type_name: how.path(type_path), boundary, offset: None };
        let path = add_numbered(job, storey, Kind::Floor)?;
        building::check_floor(&path, Kind::Floor, &floor)?;
        job.put(&path, floor)?;
        Ok(Floored { path, notes })
    })
}

/// A room: a point inside it (in feet). Its boundary is found from the walls round the
/// point (BIMCraft's `room` command, `commands.rs:7909`: a room is placed by a point).
pub fn room(job: &mut Job, storey: &str, name: &str, at: P2, how: Entered) -> Result<String, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        let room = crate::Room { point: how.val(at, "ft"), number: None };
        let path = job.add(storey, Kind::Room, name)?;
        building::check_room(&path, Kind::Room, &room)?;
        job.put(&path, room)?;
        Ok(path)
    })
}

/// Room lines along `points` (in feet, on the plan): each pair of points that follow one
/// another is one room line. A room line bounds the rooms of its level where no wall
/// stands (call 152; BIMCraft's `room_separator` command, `commands.rs:6938`, one line
/// for each pair of points).
pub fn room_lines(job: &mut Job, storey: &str, points: &[P2], how: Entered) -> Result<Vec<String>, String> {
    all_or_nothing(job, |job| {
        storey_of(job, storey)?;
        if points.len() < 2 {
            return Err("a room line needs at least 2 points".into());
        }
        let mut made = Vec::new();
        for pair in points.windows(2) {
            let line = crate::RoomLine { line: how.val([pair[0], pair[1]], "ft") };
            let path = add_called(job, storey, Kind::RoomLine, "room line", 1, false)?;
            building::check_room_line(&path, Kind::RoomLine, &line)?;
            job.put(&path, line)?;
            made.push(path);
        }
        Ok(made)
    })
}

/// What to say of room lines just drawn, each end that meets no wall and no other room
/// line of its level: "its end at x,y meets no wall and no room line: it parts nothing
/// until it does". One sentence for each such end; none when every end meets something.
pub fn room_lines_unreached_said(job: &mut Job, made: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for path in made {
        for p in building::room_line_unreached(job, path) {
            out.push(format!("{path}: its end at {},{} meets no wall and no room line: it parts nothing until it does", crate::view::num(p[0]), crate::view::num(p[1])));
        }
    }
    out
}

/// A room's figures as the model works them out: what a person reads of it in the takeoff (call 206 point 3).
#[derive(Clone, Debug, PartialEq)]
pub struct RoomFigures {
    pub path: String,
    pub name: String,
    /// The name of the level it is on.
    pub level: String,
    /// Its floor area in SF, where it has one.
    pub area: Option<f64>,
    /// Its receptacle outlets, where it has them.
    pub receptacles: Option<f64>,
}

/// Every room's floor area and receptacles now, the figures worked out again first.
pub fn room_figures(job: &mut Job) -> Vec<RoomFigures> {
    job.derive();
    let mut q = job.world.query::<(&Node, &Kind, Option<&Measure>)>();
    let mut rooms: Vec<(String, String, Option<String>, Option<f64>, Option<f64>)> = q
        .iter(&job.world)
        .filter(|(_, k, _)| **k == Kind::Room)
        .map(|(n, _, m)| (n.path.clone(), n.name.clone(), n.parent.clone(), m.and_then(|m| m.0.get("floor_area")).and_then(|d| d.value), m.and_then(|m| m.0.get("receptacles")).and_then(|d| d.value)))
        .collect();
    rooms.sort_by(|a, b| a.0.cmp(&b.0));
    rooms
        .into_iter()
        .map(|(path, name, parent, area, receptacles)| {
            let level = parent.and_then(|p| job.find(&p)).and_then(|e| job.world.get::<Node>(e)).map(|n| n.name.clone()).unwrap_or_default();
            RoomFigures { path, name, level, area, receptacles }
        })
        .collect()
}

/// A room whose figures are not what they were.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomMoved {
    pub name: String,
    pub level: String,
    pub area: (Option<f64>, Option<f64>),
    pub receptacles: (Option<f64>, Option<f64>),
}

impl RoomMoved {
    /// The room, its area before and after and, if they moved, its receptacles: "living room (first floor): floor area 614.72
    /// SF before, 365.72 SF now; receptacles 10 before, 8 now".
    pub fn said(&self) -> String {
        let area = |v: Option<f64>| v.map(|x| format!("{} SF", crate::view::figure2(x))).unwrap_or_else(|| "no figure".to_string());
        let count = |v: Option<f64>| v.map(|x| format!("{}", crate::view::num(x))).unwrap_or_else(|| "no figure".to_string());
        let mut out = format!("{} ({}): floor area {} before, {} now", self.name, self.level, area(self.area.0), area(self.area.1));
        if self.receptacles.0 != self.receptacles.1 {
            out.push_str(&format!("; receptacles {} before, {} now", count(self.receptacles.0), count(self.receptacles.1)));
        }
        out
    }
}

/// The rooms that were in `before` whose figures are not the same in `after`, in the order they were: a figure moves when
/// it differs at the cent (the way a quantity is shown).
pub fn rooms_moved(before: &[RoomFigures], after: &[RoomFigures]) -> Vec<RoomMoved> {
    let cent = |v: Option<f64>| v.map(crate::lines::cents);
    before
        .iter()
        .filter_map(|b| {
            let a = after.iter().find(|a| a.path == b.path);
            let (area, receptacles) = match a {
                Some(a) => ((b.area, a.area), (b.receptacles, a.receptacles)),
                None => ((b.area, None), (b.receptacles, None)),
            };
            (cent(area.0) != cent(area.1) || receptacles.0 != receptacles.1).then(|| RoomMoved { name: b.name.clone(), level: b.level.clone(), area, receptacles })
        })
        .collect()
}
