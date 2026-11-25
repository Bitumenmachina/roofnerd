//! Call 185: the figures that price a house's outside trim (an eave's and a rake's overhang, a
//! wall's outside corners and corner-board length, an opening's casing length) and the starter
//! lines that use them. Everything is made up. The hand figures are those of STATUS.md ("The
//! outside trim priced: a demonstration, written before it is built"), worked beside each test.

use plumbline_world::editing as ed;
use plumbline_world::file;
use plumbline_world::tables;
use plumbline_world::trim;
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up tracing" }
}
fn rows(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .map(|line| {
            let (mut out, mut cell, mut quoted) = (Vec::new(), String::new(), false);
            for c in line.chars() {
                match (c, quoted) {
                    ('"', _) => quoted = !quoted,
                    (',', false) => out.push(std::mem::take(&mut cell)),
                    (c, _) => cell.push(c),
                }
            }
            out.push(cell);
            out
        })
        .collect()
}
/// A roof drawn from its footprint, pitch 6 in 12, eave 9 ft; edges 2 and 4 of a four-sided one are gables (the ridge runs east-west).
fn roof(job: &mut Job, name: &str, ring: &[[f64; 2]], status: Status) -> String {
    roof_with(job, name, ring, status, true)
}
fn roof_with(job: &mut Job, name: &str, ring: &[[f64; 2]], status: Status, gables: bool) -> String {
    let rise = units::parse_pitch("6:12").unwrap();
    let (len, unit) = units::parse_length("9 ft").unwrap();
    let v = |x| Val { value: Some(x), unit: units::PITCH.into(), status, source: "made-up tracing".into() };
    let gable = Val { value: Some(Rise::Gable), unit: String::new(), status, source: "made-up tracing".into() };
    let footprint = Footprint {
        edges: (0..ring.len()).map(|i| if gables && ring.len() == 4 && i % 2 == 1 { gable.clone() } else { v(Rise::Pitch(rise)) }).collect(),
        points: Val { value: Some(ring.to_vec()), unit: units::FT.into(), status, source: "made-up tracing".into() },
        eave_height: Val { value: Some(len), unit: unit.into(), status, source: "made-up tracing".into() },
    };
    solve::new_roof(job, name, footprint).unwrap().0
}
fn of_kind(job: &mut Job, kind: Kind) -> Vec<String> {
    let mut q = job.world.query::<(&Node, &Kind)>();
    let mut all: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == kind).map(|(n, _)| n.path.clone()).collect();
    all.sort();
    all
}
fn figure(job: &mut Job, path: &str, name: &str) -> Derived {
    trim::figures(job).into_iter().find(|f| f.path == path && f.name == name).unwrap_or_else(|| panic!("no figure {name} of {path}")).figure
}
fn quantity(job: &mut Job, path: &str, line: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<LineQuantities>(e).unwrap().0[line].quantity.clone()
}
fn measure_names(job: &mut Job, path: &str) -> Vec<String> {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).map(|m| m.0.keys().cloned().collect()).unwrap_or_default()
}

struct House {
    job: Job,
    walls: Vec<String>,
}

/// One level 9 ft high; a ring of walls on the lines of a box 24 by 16, 6 in thick; a gable roof drawn 1 ft
/// outside the walls' lines all round (26 by 18 on plan), its ridge running east-west, pitch 6 in 12.
fn box_house(status: ed::Entered<'static>, roof_status: Status) -> (House, String) {
    let mut job = Job::new("Box (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", status).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", status).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), status).unwrap();
    let _r = roof(&mut job, "R", &[[-1.0, -1.0], [25.0, -1.0], [25.0, 17.0], [-1.0, 17.0]], roof_status);
    job.derive();
    (House { job, walls }, level)
}

// By hand: eaves north and south 26 LF each; rakes four, each 9 x 1.118034 = 10.0623 LF; the roof's edge is 1 ft outside
// the walls' lines and the walls' outside faces are 3 in outside them: 12 - 3 = 9 in = 0.75 ft at every eave and rake.
#[test]
fn the_box_has_an_overhang_of_nine_inches_at_every_eave_and_rake_assumed_as_its_tracing_is() {
    let (mut h, _) = box_house(assumed(), Status::Assumed);
    let (eaves, rakes) = (of_kind(&mut h.job, Kind::Eave), of_kind(&mut h.job, Kind::Rake));
    assert_eq!((eaves.len(), rakes.len()), (2, 4));
    for path in eaves.iter().chain(rakes.iter()) {
        let d = figure(&mut h.job, path, "overhang");
        assert_eq!((d.value, d.unit.as_str(), d.status), (Some(0.75), "ft", Status::Assumed), "{path}: {d:?}");
        assert!(d.inputs.iter().any(|i| i.starts_with("wall-")), "it says which wall it rests on: {:?}", d.inputs);
    }
    // Drawn Known, it is Known.
    let (mut k, _) = box_house(known(), Status::Known);
    let eave = of_kind(&mut k.job, Kind::Eave)[0].clone();
    assert_eq!(figure(&mut k.job, &eave, "overhang").status, Status::Known);
}

// Fascia on the eaves 2 x 26 = 52.00 LF; soffit 2 x 26 x 0.75 = 39.00 SF; rake fascia 4 x 10.0623 = 40.2492 LF; rake soffit
// 40.2492 x 0.75 = 30.1869 SF.
#[test]
fn fascia_and_soffit_on_the_eaves_and_the_rakes_come_to_the_hand_figures() {
    let (mut h, _) = box_house(assumed(), Status::Assumed);
    let (eaves, rakes) = (of_kind(&mut h.job, Kind::Eave), of_kind(&mut h.job, Kind::Rake));
    for e in &eaves {
        h.job.add_line(e, "Fascia", "true_length", Some(Class::Material), false).unwrap();
        h.job.add_line(e, "Soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
    }
    for r in &rakes {
        h.job.add_line(r, "Rake fascia", "true_length", Some(Class::Material), false).unwrap();
        h.job.add_line(r, "Rake soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
    }
    h.job.derive();
    let sum = |job: &mut Job, paths: &[String], line: &str| -> f64 { paths.iter().map(|p| quantity(job, p, line).value.unwrap()).sum() };
    assert_eq!(sum(&mut h.job, &eaves, "Fascia"), 52.0);
    assert_eq!(sum(&mut h.job, &eaves, "Soffit"), 39.0);
    assert!((sum(&mut h.job, &rakes, "Rake fascia") - 40.2492).abs() < 0.001);
    assert!((sum(&mut h.job, &rakes, "Rake soffit") - 30.1869).abs() < 0.001);
    let soffit = quantity(&mut h.job, &eaves[0], "Soffit");
    assert_eq!((soffit.unit.as_str(), soffit.status), ("SF", Status::Assumed));
}

// Each wall has 2 outside corners: 2 x 9 = 18.00 LF on the south and the north wall (under an eave), 2 x 9.5 = 19.00 LF on the east and
// the west (under a rake, call 204); four walls 18 + 19 + 18 + 19 = 74.00 LF (72.00 before call 204).
#[test]
fn each_wall_of_the_box_has_two_outside_corners_and_eighteen_feet_of_corner_board() {
    let (mut h, _) = box_house(assumed(), Status::Assumed);
    for w in h.walls.clone() {
        h.job.add_line(&w, "Corner boards", "corner_board_length", Some(Class::Material), false).unwrap();
    }
    h.job.derive();
    for (k, w) in h.walls.clone().into_iter().enumerate() {
        let (n, l) = (figure(&mut h.job, &w, "outside_corners"), figure(&mut h.job, &w, "corner_board_length"));
        assert_eq!((n.value, n.unit.as_str(), n.status), (Some(2.0), "EA", Status::Assumed), "{w}: {n:?}");
        let each = if k == 1 || k == 3 { 19.0 } else { 18.0 };
        assert_eq!((l.value, l.unit.as_str(), l.status), (Some(each), "LF", Status::Assumed), "{w}: {l:?}");
    }
    let total: f64 = h.walls.clone().iter().map(|w| quantity(&mut h.job, w, "Corner boards").value.unwrap()).sum();
    assert_eq!(total, 74.0);
}

// A window 3 by 5: 2 x (3 + 5) = 16.00 LF. A door 3 by 6 ft 8 in: 3 + 2 x 6.6667 = 16.33 LF.
#[test]
fn casing_is_head_and_legs_and_sill_on_a_window_and_head_and_legs_on_a_door() {
    let (mut h, _) = box_house(assumed(), Status::Assumed);
    let wt = ed::opening_type(&mut h.job, "window 3 ft", Kind::Window, "3 ft", "5 ft", assumed()).unwrap();
    let dt = ed::opening_type(&mut h.job, "door 3 ft", Kind::Door, "3 ft", "6 ft 8 in", assumed()).unwrap();
    let win = ed::opening(&mut h.job, &h.walls[0], Kind::Window, &wt, ed::OnWall::Offset("18 ft"), "2.5 ft", assumed()).unwrap();
    let door = ed::opening(&mut h.job, &h.walls[0], Kind::Door, &dt, ed::OnWall::Offset("6 ft"), "0 ft", assumed()).unwrap();
    for o in [&win, &door] {
        h.job.add_line(o, "Exterior casing", "casing_length", Some(Class::Material), false).unwrap();
    }
    h.job.derive();
    let (w, d) = (figure(&mut h.job, &win, "casing_length"), figure(&mut h.job, &door, "casing_length"));
    assert_eq!((w.value, w.unit.as_str(), w.status), (Some(16.0), "LF", Status::Assumed), "{w:?}");
    assert_eq!((d.value, d.status), (Some(16.3333), Status::Assumed), "{d:?}");
    assert_eq!(quantity(&mut h.job, &win, "Exterior casing").value, Some(16.0));
    assert_eq!(quantity(&mut h.job, &door, "Exterior casing").value, Some(16.3333));
}

// An L: a 24 by 16 box with a 12 by 10 wing off its north side (the wing's east face 12 ft in from the box's east end): six walls,
// five outside corners and one inside. Boards 10 x 9 = 90.00 LF; the inside corner has none, and the two walls that meet there
// have one outside corner each.
#[test]
fn an_l_has_five_outside_corners_and_the_inside_corner_has_no_board() {
    let mut job = Job::new("L (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", assumed()).unwrap();
    // South, east, back along the box's north to the wing's east face, up the wing, across its top, down its west face, and home.
    let ring = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [12.0, 16.0], [12.0, 26.0], [0.0, 26.0]];
    let walls = ed::walls(&mut job, &level, &w6, &ring, true, ed::Top::Height("9 ft"), assumed()).unwrap();
    assert_eq!(walls.len(), 6);
    for w in &walls {
        job.add_line(w, "Corner boards", "corner_board_length", Some(Class::Material), false).unwrap();
    }
    job.derive();
    let counts: Vec<f64> = walls.iter().map(|w| figure(&mut job, w, "outside_corners").value.unwrap()).collect();
    // The inside corner is at (12, 16): walls 3 (north of the box, ends there) and 4 (the wing's east face, starts there).
    assert_eq!(counts, vec![2.0, 2.0, 1.0, 1.0, 2.0, 2.0], "{counts:?}");
    let total: f64 = walls.iter().map(|w| quantity(&mut job, w, "Corner boards").value.unwrap()).sum();
    assert_eq!(total, 90.0);
}

// The roof drawn alone: no wall under any edge. Its overhang is Unknown and says why; the soffit has no figure.
#[test]
fn a_roof_with_no_wall_under_its_edge_has_no_overhang_and_says_why() {
    let mut job = Job::new("Roof alone (not a real job)", "test", "2026-11-23");
    let _r = roof(&mut job, "R", &[[-1.0, -1.0], [25.0, -1.0], [25.0, 17.0], [-1.0, 17.0]], Status::Assumed);
    let eave = of_kind(&mut job, Kind::Eave)[0].clone();
    job.add_line(&eave, "Soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
    job.derive();
    let d = figure(&mut job, &eave, "overhang");
    assert_eq!((d.value, d.status), (None, Status::Unknown));
    assert!(d.note.contains("no wall is found under this edge"), "{}", d.note);
    let s = quantity(&mut job, &eave, "Soffit");
    assert_eq!(s.value, None, "{s:?}");
}

// A roof edge that lies inside the wall's thickness is short of the wall's outside face: it has no overhang, and says so.
#[test]
fn an_edge_inside_the_walls_thickness_has_no_overhang_and_says_so() {
    let (mut h, _) = box_house(assumed(), Status::Assumed);
    // Move nothing: draw a second roof whose south edge is on the wall's line (0.25 ft inside its outside face).
    let r2 = roof_with(&mut h.job, "R2", &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], Status::Assumed, false);
    let _ = r2;
    h.job.derive();
    let eaves = of_kind(&mut h.job, Kind::Eave);
    let on_line: Vec<&String> = eaves.iter().filter(|e| figure(&mut h.job, e, "overhang").value.is_none()).collect();
    assert!(!on_line.is_empty());
    let d = figure(&mut h.job, on_line[0], "overhang");
    assert!(d.note.contains("inside the wall's outside face"), "{}", d.note);
}

// Where three walls end at one point, which of its corners is an outside corner is not worked, and the walls say so;
// a ring that is not closed has no inside and no outside.
#[test]
fn a_corner_that_cannot_be_told_is_said_not_counted() {
    let mut job = Job::new("Open (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", assumed()).unwrap();
    // Three sides of a box, not closed.
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    job.add_line(&walls[1], "Corner boards", "corner_board_length", Some(Class::Material), false).unwrap();
    job.derive();
    let d = figure(&mut job, &walls[1], "outside_corners");
    assert_eq!(d.value, None);
    assert!(d.note.contains("not closed"), "{}", d.note);
    let l = figure(&mut job, &walls[1], "corner_board_length");
    assert_eq!(l.value, None);
    assert_eq!(quantity(&mut job, &walls[1], "Corner boards").value, None);
    // A partition's end on the middle of a wall is a tee, not a corner: the box still has its four.
    let (mut h, level) = box_house(assumed(), Status::Assumed);
    let w6 = ed::wall_type_for(&mut h.job, "6 in", assumed()).unwrap();
    let part = ed::walls(&mut h.job, &level, &w6, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    for w in h.walls.iter().chain(part.iter()) {
        h.job.add_line(w, "Corner boards", "corner_board_length", Some(Class::Material), false).unwrap();
    }
    h.job.derive();
    for w in &h.walls {
        assert_eq!(figure(&mut h.job, w, "outside_corners").value, Some(2.0), "{w}");
    }
    assert_eq!(figure(&mut h.job, &part[0], "outside_corners").value, Some(0.0));
}

// Nothing existing moves: a thing that no line asks the figure of does not carry it, so a job saved before opens and saves the same.
#[test]
fn a_figure_nobody_names_is_not_given_and_a_job_with_the_lines_saves_and_opens_the_same() {
    let (mut h, _) = box_house(assumed(), Status::Assumed);
    let wall = h.walls[0].clone();
    let eave = of_kind(&mut h.job, Kind::Eave)[0].clone();
    for name in ["outside_corners", "corner_board_length"] {
        assert!(!measure_names(&mut h.job, &wall).contains(&name.to_string()), "{name}");
    }
    assert!(!measure_names(&mut h.job, &eave).contains(&"overhang".to_string()));
    let before = file::save(&mut h.job);
    let (mut again, report) = file::load(&before).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(file::save(&mut again), before);
    // With the lines on, saved and opened again: the same bytes, and nothing stale.
    h.job.add_line(&wall, "Corner boards", "corner_board_length", Some(Class::Material), false).unwrap();
    h.job.add_line(&eave, "Soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
    h.job.derive();
    let text = file::save(&mut h.job);
    let (mut back, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(file::save(&mut back), text);
    assert!(text.contains("corner_board_length") && text.contains("overhang"));
}

// The starter lines, no price on any; "Downspouts" has no quantity until a person types how many (call 159's way).
#[test]
fn the_starter_trim_lines_have_no_price_and_downspouts_waits_to_be_typed() {
    let (mut h, _) = box_house(assumed(), Status::Assumed);
    tables::assemblies_in(&mut h.job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    let (eaves, rakes) = (of_kind(&mut h.job, Kind::Eave), of_kind(&mut h.job, Kind::Rake));
    for e in &eaves {
        h.job.apply(e, "Gutter", "the starter library").unwrap();
    }
    for r in &rakes {
        h.job.apply(r, "Rake trim", "the starter library").unwrap();
    }
    for w in h.walls.clone() {
        h.job.apply(&w, "Exterior wall, 2x6, lap siding", "the starter library").unwrap();
    }
    h.job.derive();
    let e = &eaves[0];
    let (fascia, soffit, gutter) = (quantity(&mut h.job, e, "Fascia"), quantity(&mut h.job, e, "Soffit"), quantity(&mut h.job, e, "Gutter"));
    assert_eq!((fascia.value, soffit.value, gutter.value), (Some(26.0), Some(19.5), Some(26.0)));
    // Not a price among them.
    let ext = |job: &mut Job, p: &str, l: &str| {
        let e = job.find(p).unwrap();
        job.world.get::<LineQuantities>(e).unwrap().0[l].extension.clone().map(|x| x.value)
    };
    assert_eq!(ext(&mut h.job, e, "Fascia"), Some(None));
    // Downspouts: no figure, and it says a person is to say how many.
    let ds = quantity(&mut h.job, e, "Downspouts");
    assert_eq!(ds.value, None, "{ds:?}");
    // It says a person is to say how many: the count is held on the line as a value that is Unknown, and says why.
    let e_id = h.job.find(e).unwrap();
    let held = h.job.world.get::<Cost>(e_id).unwrap().lines.iter().find(|l| l.description == "Downspouts").unwrap().inputs["downspouts"].clone();
    assert_eq!((held.value.is_none(), held.status, held.source.as_str()), (true, Status::Unknown, "how many is the job's to say: give the count"));
    h.job.set_on_line(e, "downspouts", Some("Downspouts"), Some("4 EA"), Status::Assumed, "one at each end of the run, walked").unwrap();
    h.job.derive();
    let ds = quantity(&mut h.job, e, "Downspouts");
    assert_eq!((ds.value, ds.unit.as_str(), ds.status), (Some(4.0), "EA", Status::Assumed), "{ds:?}");
    let rake = &rakes[0];
    assert_eq!(quantity(&mut h.job, rake, "Rake fascia").value, Some(10.0623));
    assert_eq!(quantity(&mut h.job, &h.walls[0].clone(), "Corner boards").value, Some(18.0));
}
