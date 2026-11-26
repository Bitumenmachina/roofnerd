//! The clashes handed over as an issue list another program opens (call 228): buildingSMART's BCF, version 2.1.
//!
//! One issue for each clash in the same space, in the order of the clash list, and, when asked, one for each hole to cut. Each
//! issue names its two things by the identifier the BIM file (IFC) gives them, made by the very function that makes it for that
//! file (`write::guid`), so a program that opens the model and the issues selects the right things. Each has a view: where to
//! stand and what to look at, worked from the clash's place and the two things' boxes (`clash::extent`). What the issue says is
//! what the clash list says: the certainty is the clash's own (a finding where everything it rests on is Known, a question for
//! the field where not) and is never raised; the dollars are the verify-in-field list's own.
//!
//! The format (`research/bcf.md`): a zip holding `bcf.version`, and for each issue a folder, named by the issue's identifier, with
//! `markup.bcf` (the issue) and `viewpoint.bcfv` (the view). Lengths in the view are metres, the axes the model's own as the BIM
//! file has them (east, north, up), the field of view 60 degrees. No picture of the place is made (the format's snapshot is
//! optional); the author is the program's name and the time the time of writing, since the program knows no person.

use std::io::Write;

use plumbline_world::building::clash::{self, Clash, Clashes};
use plumbline_world::building::{self, Placement};
use plumbline_world::value::Status;
use plumbline_world::{units, view, vif, Job, P3};

use crate::write;

/// Who is named as the author of every issue.
pub const AUTHOR: &str = "BuildOS";
/// The metres in a foot: the format's length is metres, the model's feet.
const METRE: f64 = 0.3048;
/// The field of view the views are given, in degrees (the format's own limit is 45 to 60).
const FIELD_OF_VIEW: f64 = 60.0;
/// How much of a large thing (a wall, a floor) the view is made to hold: so far each way of the place, in feet.
const WINDOW_FT: f64 = 5.0;

/// What sort of issue it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sort {
    /// Two things in the same space.
    Clash,
    /// A run through a wall or a floor: work to do, not a clash.
    Hole,
}

/// One of the things an issue names.
#[derive(Clone, Debug, PartialEq)]
pub struct Thing {
    /// Where it is in the job.
    pub path: String,
    pub name: String,
    /// Its identifier in the BIM file.
    pub id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    /// The issue's own identifier (the folder it is in).
    pub topic: String,
    pub sort: Sort,
    /// Everything it rests on is Known: a finding. Otherwise a question for the field. A hole is a finding where its place is Known.
    pub finding: bool,
    pub title: String,
    pub description: String,
    /// The things it names, in the order the clash list gives them (for a hole: the pieces of the run, then what they pass through).
    pub things: Vec<Thing>,
    /// Where, in feet: east, north, height above the datum: the clash's own.
    pub at: P3,
    /// The view, in metres and the model's axes: where to stand, the way to look (unit length) and which way is up (unit length).
    pub eye: [f64; 3],
    pub direction: [f64; 3],
    pub up: [f64; 3],
}

#[derive(Clone, Debug)]
pub struct Written {
    /// The file.
    pub bytes: Vec<u8>,
    pub issues: Vec<Issue>,
    /// Of the clashes in the same space handed over: how many are findings and how many are questions for the field.
    pub findings: usize,
    pub questions: usize,
    /// How many holes were handed over (0 unless asked for).
    pub holes: usize,
    /// How many things set against a wall (not a clash) were left out, as the clash list counts them.
    pub set_against: usize,
    /// How many holes there are and were not handed over, when they were not asked for.
    pub holes_left: usize,
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / n, v[1] / n, v[2] / n]
}

/// Where to stand and what to look at: the eye is outside both things' boxes, and the line of sight goes through the clash's place.
/// All in feet here; the caller turns them to metres. The view is made to hold the part of each thing near the place (so far each
/// way of it), looked at from the south west and above.
fn view_of(at: P3, boxes: &[(P3, P3)]) -> (P3, [f64; 3], [f64; 3]) {
    let (mut lo, mut hi) = (at, at);
    for (a, b) in boxes {
        for k in 0..3 {
            let (l, h) = (a[k].max(at[k] - WINDOW_FT), b[k].min(at[k] + WINDOW_FT));
            // A thing whose box does not come within the window of the place holds the place itself.
            let (l, h) = if l > h { (at[k], at[k]) } else { (l, h) };
            lo[k] = lo[k].min(l);
            hi[k] = hi[k].max(h);
        }
    }
    let middle = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    let half = ((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2) + (hi[2] - lo[2]).powi(2)).sqrt() / 2.0;
    let way = unit([-1.0, -1.0, 0.6]);
    let mut far = 4.0 * half.max(0.5) + 6.0;
    let mut eye = middle;
    for _ in 0..24 {
        eye = [middle[0] + way[0] * far, middle[1] + way[1] * far, middle[2] + way[2] * far];
        if !boxes.iter().any(|(a, b)| (0..3).all(|k| eye[k] >= a[k] && eye[k] <= b[k])) {
            break;
        }
        far *= 1.5;
    }
    let direction = unit([at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]]);
    // Up is the model's up, turned to be square to the way looked.
    let d = direction[2];
    let up = unit([-d * direction[0], -d * direction[1], 1.0 - d * d]);
    (eye, direction, up)
}

fn dollars_said(q: Option<&vif::Question>) -> String {
    let Some(q) = q else { return String::new() };
    let lines = |n: usize| view::so_many(n, "line");
    let have = |n: usize| if n == 1 { "line that rides on it has" } else { "lines that ride on it have" };
    // How sure the figure is is said only where it is not Known (the verify-in-field list marks it the same way).
    let sure = if q.dollars_status == Status::Known { String::new() } else { format!("; that figure is {:?}", q.dollars_status) };
    match (q.dollars, q.unpriced) {
        (Some(x), 0) => format!("${} rides on it ({}){sure}.", view::figure(x, 2), lines(q.lines)),
        (Some(x), u) => format!("${} rides on it ({}){sure}. Of what else rides on it, {} {} no price yet.", view::figure(x, 2), lines(q.lines), u, have(u)),
        (None, 0) => "Nothing priced rides on it.".to_string(),
        (None, u) => format!("No dollars yet: {u} {} no price yet.", have(u)),
    }
}

/// Where the place is, in a builder's words: the level, the room and the place in feet and inches.
fn where_said(level: Option<&str>, room: &str, at: P3) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(level) = level {
        parts.push(level.to_string());
    }
    parts.push(if room == "outside any room" { room.to_string() } else { format!("in {room}") });
    format!("Where: {}, {} east, {} north and {} above the datum.", parts.join(", "), units::length_said(at[0]), units::length_said(at[1]), units::length_said(at[2]))
}

fn markup(issue: &Issue, project: &str, when: &str, index: usize, viewpoint: &str) -> String {
    let (type_, label) = match (&issue.sort, issue.finding) {
        (Sort::Hole, _) => ("Hole to cut", "Hole to cut"),
        (Sort::Clash, true) => ("Clash", "Finding"),
        (Sort::Clash, false) => ("Clash", "To verify in the field"),
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Markup>\n  <Header>\n    <File IfcProject=\"{project}\" />\n  </Header>\n  <Topic Guid=\"{topic}\" TopicType=\"{type_}\" TopicStatus=\"Open\">\n    <Title>{title}</Title>\n    <Index>{index}</Index>\n    <Labels>{label}</Labels>\n    <CreationDate>{when}</CreationDate>\n    <CreationAuthor>{AUTHOR}</CreationAuthor>\n    <Description>{description}</Description>\n  </Topic>\n  <Viewpoints Guid=\"{viewpoint}\">\n    <Viewpoint>viewpoint.bcfv</Viewpoint>\n    <Index>0</Index>\n  </Viewpoints>\n</Markup>\n",
        topic = issue.topic,
        title = escape(&issue.title),
        description = escape(&issue.description),
    )
}

fn viewpoint(issue: &Issue, id: &str) -> String {
    let mut selection = String::new();
    for t in &issue.things {
        selection.push_str(&format!(
            "        <Component IfcGuid=\"{}\">\n          <OriginatingSystem>{AUTHOR}</OriginatingSystem>\n          <AuthoringToolId>{}</AuthoringToolId>\n        </Component>\n",
            t.id,
            escape(&t.path)
        ));
    }
    let n = |x: f64| format!("{:.6}", x + 0.0);
    let point = |tag: &str, v: [f64; 3]| format!("      <{tag}>\n        <X>{}</X>\n        <Y>{}</Y>\n        <Z>{}</Z>\n      </{tag}>\n", n(v[0]), n(v[1]), n(v[2]));
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<VisualizationInfo Guid=\"{id}\">\n  <Components>\n    <Selection>\n{selection}    </Selection>\n    <Visibility DefaultVisibility=\"true\" />\n  </Components>\n  <PerspectiveCamera>\n{}{}{}      <FieldOfView>{}</FieldOfView>\n  </PerspectiveCamera>\n</VisualizationInfo>\n",
        point("CameraViewPoint", issue.eye),
        point("CameraDirection", issue.direction),
        point("CameraUpVector", issue.up),
        n(FIELD_OF_VIEW),
    )
}

/// The clashes of the job as an issue list: one for each clash in the same space, and with `holes` one for each hole to cut. `when`
/// is the moment it is written (`2026-11-26T09:30:00`), the one thing two writings differ in.
pub fn export(job: &mut Job, holes: bool, when: &str) -> Result<Written, String> {
    let found: Clashes = clash::clashes(job, None)?;
    let title = write::job_title(job);
    let elements = view::elements(job);
    let level_name = |path: &str| elements.iter().find(|e| e.path == path).map(|e| e.name.clone());
    let (placed, _) = building::placements(job);
    let (rooms, hole_rooms) = clash::rooms_of(job, &found);
    let asked = vif::vif(job);
    let thing = |path: &str, name: &str| Thing { path: path.to_string(), name: name.to_string(), id: write::guid(&title, path) };
    let boxes = |things: &[Thing]| -> Vec<(P3, P3)> { things.iter().filter_map(|t| placed.iter().find(|p| p.path == t.path)).filter_map(|p: &Placement| clash::extent(p)).collect() };
    let at_time = if when.ends_with('Z') { when.to_string() } else { format!("{when}Z") };

    let mut issues: Vec<Issue> = Vec::new();
    let finish = |sort: Sort, finding: bool, title_text: String, description: String, things: Vec<Thing>, at: P3, key: String, issues: &mut Vec<Issue>| {
        let (eye, direction, up) = view_of(at, &boxes(&things));
        issues.push(Issue {
            topic: write::uuid(&title, &key),
            sort,
            finding,
            title: title_text,
            description,
            things,
            at,
            eye: [eye[0] * METRE, eye[1] * METRE, eye[2] * METRE],
            direction,
            up,
        });
    };

    let same: Vec<&Clash> = found.in_the_same_space();
    let (mut findings, mut questions) = (0, 0);
    for c in &same {
        let i = found.found.iter().position(|x| std::ptr::eq(x, *c)).unwrap_or(0);
        let finding = c.status == Status::Known;
        if finding {
            findings += 1;
        } else {
            questions += 1;
        }
        let level = clash::storey_of_pair(&placed, &c.a, &c.b).and_then(|s| level_name(&s));
        let how = match c.by {
            Some(by) => format!("are in each other by {}", units::length_said(by)),
            None => "are in the same space; by how much could not be worked out".to_string(),
        };
        let mut text = vec![format!("In the same space: {} and {} {how}.", c.a_name, c.b_name), where_said(level.as_deref(), &rooms[i], c.at)];
        if let Some(words) = clash::connection_words(job, c) {
            text.push(format!("Its connections: {words}."));
        }
        if finding {
            text.push("A finding: everything it rests on is Known.".to_string());
        } else {
            text.push("A question for the field: something it rests on is Assumed. Measure it in the field: a measurement turns it into a finding or clears it.".to_string());
            let q = asked.questions.iter().find(|q| q.clash.as_ref().is_some_and(|b| !b.through && q.path == c.a && b.other == c.b && b.at == c.at));
            let said = dollars_said(q);
            if !said.is_empty() {
                text.push(said);
            }
        }
        finish(
            Sort::Clash,
            finding,
            format!("{} and {}: {}", c.a_name, c.b_name, c.depth_listed()),
            text.join("\n"),
            vec![thing(&c.a, &c.a_name), thing(&c.b, &c.b_name)],
            c.at,
            format!("clash {} {} {:?}", c.a, c.b, c.at),
            &mut issues,
        );
    }

    let rows = found.hole_rows();
    let mut handed = 0;
    if holes {
        for c in &rows {
            let i = found.found.iter().position(|x| std::ptr::eq(x, *c)).unwrap_or(0);
            let (names, through, things, at, status, size_said, room, title_text, key) = match c.hole.and_then(|h| found.holes.get(h).map(|x| (h, x))) {
                Some((h, hole)) => {
                    let mut names: Vec<String> = hole.pieces.iter().take(3).map(|(_, n)| n.clone()).collect();
                    if hole.pieces.len() > 3 {
                        names.push(format!("{} more", hole.pieces.len() - 3));
                    }
                    let size = if hole.size.is_empty() { String::new() } else { format!(", {}", hole.size) };
                    let of = if hole.pieces.len() > 1 { format!(", through {} pieces of the run", hole.pieces.len()) } else { String::new() };
                    let mut things: Vec<Thing> = hole.pieces.iter().map(|(p, n)| thing(p, n)).collect();
                    things.push(thing(&hole.through, &hole.through_name));
                    (names.join(", "), hole.through_name.clone(), things, hole.at, hole.status, hole.size_said.clone(), hole_rooms[h].clone(), format!("{} and {}: 1 hole{size}{of}", names.join(", "), hole.through_name), format!("hole {h}"))
                }
                None => (c.a_name.clone(), c.b_name.clone(), vec![thing(&c.a, &c.a_name), thing(&c.b, &c.b_name)], c.at, c.status, String::new(), rooms[i].clone(), format!("{} and {}: a hole", c.a_name, c.b_name), format!("hole {} {} {:?}", c.a, c.b, c.at)),
            };
            let level = clash::storey_of_pair(&placed, &things[0].path, &things[things.len() - 1].path).and_then(|s| level_name(&s));
            let finding = status == Status::Known;
            let size = if size_said.is_empty() { String::new() } else { format!(", {size_said}") };
            let mut text = vec![format!("A hole to cut: {names} goes through {through}{size}."), where_said(level.as_deref(), &room, at)];
            text.push(if finding { "Where it comes is Known.".to_string() } else { "Where it comes rests on something Assumed: verify it in the field before it is cut.".to_string() });
            text.push("Nothing priced rides on the hole itself.".to_string());
            finish(Sort::Hole, finding, title_text, text.join("\n"), things, at, key, &mut issues);
            handed += 1;
        }
    }

    // The zip: the version, then each issue's folder.
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opt = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut put = |name: &str, text: &str| -> Result<(), String> {
        z.start_file(name, opt).map_err(|e| format!("cannot write the file: {e}"))?;
        z.write_all(text.as_bytes()).map_err(|e| format!("cannot write the file: {e}"))
    };
    put("bcf.version", "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Version VersionId=\"2.1\">\n  <DetailedVersion>2.1</DetailedVersion>\n</Version>\n")?;
    let project = write::guid(&title, "project");
    for (index, issue) in issues.iter().enumerate() {
        let vp = write::uuid(&title, &format!("{} viewpoint", issue.topic));
        put(&format!("{}/markup.bcf", issue.topic), &markup(issue, &project, &at_time, index, &vp))?;
        put(&format!("{}/viewpoint.bcfv", issue.topic), &viewpoint(issue, &vp))?;
    }
    let bytes = z.finish().map_err(|e| format!("cannot write the file: {e}"))?.into_inner();
    Ok(Written { bytes, issues, findings, questions, holes: handed, set_against: found.set_against().len(), holes_left: if holes { 0 } else { rows.len() } })
}
