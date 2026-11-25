//! Call 204, points 1 to 3: a corner board is as tall as its wall is at that corner. Everything is made up. The hand figures
//! are those of STATUS.md ("A corner board as tall as its wall at that corner: a demonstration").
//!
//! The house of call 192: one level 9 ft high; walls on the lines of a box 24 by 16 ft, 6 in thick, south, east, north, west; a
//! roof drawn 1 ft outside the walls' lines all round, eave height 9 ft, 6 in 12: 9.5 ft over the wall lines at an eave.

use plumbline_world::editing as ed;
use plumbline_world::trim::{self, PieceKind as PK};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up tracing" }
}
const LINE: &str = "Corner boards";
const RING: [[f64; 2]; 4] = [[-1.0, -1.0], [25.0, -1.0], [25.0, 17.0], [-1.0, 17.0]];

fn roof(job: &mut Job, edges: &str, status: Status) {
    let rise = units::parse_pitch("6:12").unwrap();
    let (len, unit) = units::parse_length("9 ft").unwrap();
    let v = |x| Val { value: Some(x), unit: units::PITCH.into(), status, source: "made-up tracing".into() };
    let gable = Val { value: Some(Rise::Gable), unit: String::new(), status, source: "made-up tracing".into() };
    let footprint = Footprint {
        edges: edges.chars().map(|c| if c == 'G' { gable.clone() } else { v(Rise::Pitch(rise)) }).collect(),
        points: Val { value: Some(RING.to_vec()), unit: units::FT.into(), status, source: "made-up tracing".into() },
        eave_height: Val { value: Some(len), unit: unit.into(), status, source: "made-up tracing".into() },
    };
    solve::new_roof(job, "R", footprint).unwrap();
}

/// The box, its roof (None: no roof), and a corner-board line on each wall. South, east, north, west.
fn house(edges: Option<&str>, walls_are: ed::Entered<'static>, roof_status: Status) -> (Job, Vec<String>) {
    let mut job = Job::new("Box (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", walls_are).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", walls_are).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), walls_are).unwrap();
    if let Some(e) = edges {
        roof(&mut job, e, roof_status);
    }
    for w in &walls {
        job.add_line(w, LINE, "corner_board_length", Some(Class::Material), false).unwrap();
    }
    job.derive();
    (job, walls)
}

fn quantity(job: &mut Job, path: &str) -> Derived {
    job.derive();
    let e = job.find(path).unwrap();
    job.world.get::<LineQuantities>(e).unwrap().0[LINE].quantity.clone()
}
fn figure(job: &mut Job, path: &str, name: &str) -> Derived {
    trim::figures(job).into_iter().find(|f| f.path == path && f.name == name).unwrap_or_else(|| panic!("no {name} on {path}")).figure
}
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3
}
fn len3(p: &trim::Piece) -> f64 {
    ((p.to[0] - p.from[0]).powi(2) + (p.to[1] - p.from[1]).powi(2) + (p.to[2] - p.from[2]).powi(2)).sqrt()
}
fn lengths(job: &mut Job, walls: &[String]) -> Vec<f64> {
    walls.iter().map(|w| quantity(job, w).value.unwrap()).collect()
}
fn said(v: &[f64]) -> String {
    format!("{v:?}")
}

// By hand: south and north are under the eaves, 2 x 9 = 18.00 LF each; the east and the west rise to 9.5 ft over their
// ends (top (0, 9.5) (8, 13.5) (16, 9.5)): 2 x 9.5 = 19.00 LF each. The house 18 + 19 + 18 + 19 = 74.00 LF (72.00 before).
#[test]
fn a_gable_roofs_end_walls_take_a_board_as_tall_as_their_top_at_each_corner() {
    let (mut job, walls) = house(Some("PGPG"), assumed(), Status::Assumed);
    let got = lengths(&mut job, &walls);
    assert!(got.iter().zip([18.0, 19.0, 18.0, 19.0]).all(|(a, b)| near(*a, b)), "{}", said(&got));
    assert!(near(got.iter().sum::<f64>(), 74.0));
    // The figure itself, and the two corners it counts.
    for w in &walls {
        assert_eq!(figure(&mut job, w, "outside_corners").value, Some(2.0));
    }
    let east = figure(&mut job, &walls[1], "corner_board_length");
    assert_eq!((east.value, east.unit.as_str(), east.status), (Some(19.0), "LF", Status::Assumed), "{east:?}");
    assert!(east.inputs.iter().any(|i| i.starts_with("plane-")) && east.inputs.iter().any(|i| i.starts_with("rake-")), "it says the roof it rests on: {:?}", east.inputs);
}

// A shed roof (its eave on the south, rising north): south 2 x 9 = 18; east 9.5 + 17.5 = 27; north 2 x 17.5 = 35; west 17.5 + 9.5 = 27.
// The house 107.00 LF (72.00 before).
#[test]
fn a_shed_roofs_walls_take_the_boards_of_their_height_at_each_end() {
    let (mut job, walls) = house(Some("PGGG"), assumed(), Status::Assumed);
    let got = lengths(&mut job, &walls);
    assert!(got.iter().zip([18.0, 27.0, 35.0, 27.0]).all(|(a, b)| near(*a, b)), "{}", said(&got));
    assert!(near(got.iter().sum::<f64>(), 107.0));
}

// A hip roof: eaves all round, every wall 18.00 LF, the house 72.00 LF: nothing moves. No roof: the same.
#[test]
fn a_hip_roof_and_no_roof_leave_every_board_as_it_was() {
    for edges in [Some("PPPP"), None] {
        let (mut job, walls) = house(edges, assumed(), Status::Assumed);
        let got = lengths(&mut job, &walls);
        assert!(got.iter().all(|g| near(*g, 18.0)), "{edges:?}: {}", said(&got));
    }
}

// The pieces laid are the same lengths: one arithmetic. On the gable's end walls two of 9.5 ft; on the shed's east wall one of 9.5 ft
// at the south corner and one of 17.5 ft at the north corner, each from the foot to the wall's top at that corner.
#[test]
fn the_pieces_laid_are_the_figures_lengths_and_the_shed_walls_north_board_is_seventeen_and_a_half() {
    let (mut job, walls) = house(Some("PGPG"), assumed(), Status::Assumed);
    let ps = trim::pieces(&mut job);
    for (w, each) in walls.iter().zip([9.0, 9.5, 9.0, 9.5]) {
        let mine: Vec<&trim::Piece> = ps.iter().filter(|p| &p.of == w && p.kind == PK::CornerBoard).collect();
        assert_eq!(mine.len(), 2);
        assert!(mine.iter().all(|p| near(len3(p), each) && near(p.from[2], 0.0) && near(p.to[2], each)), "{w}: {mine:?}");
    }
    let (mut job, walls) = house(Some("PGGG"), assumed(), Status::Assumed);
    let laid = trim::lay(&mut job);
    assert!(near(laid.priced_of(PK::CornerBoard).unwrap(), 107.0), "{:?}", laid.priced_of(PK::CornerBoard));
    assert!(laid.left_out.iter().all(|s| !s.contains("corner board")), "nothing of the boards is left out: {:?}", laid.left_out);
    for w in &walls {
        let q = quantity(&mut job, w).value.unwrap();
        let got: f64 = laid.pieces.iter().filter(|p| &p.of == w && p.kind == PK::CornerBoard).map(len3).sum();
        assert!(near(got, q), "{w}: the pieces lie {got} LF and the line prices {q}");
    }
    let east: Vec<&trim::Piece> = laid.pieces.iter().filter(|p| p.of == walls[1] && p.kind == PK::CornerBoard).collect();
    let north_corner = east.iter().find(|p| p.from[1] > 8.0).expect("a board at the north end of the east wall");
    assert!(near(len3(north_corner), 17.5) && near(north_corner.to[2], 17.5), "{north_corner:?}");
    let south_corner = east.iter().find(|p| p.from[1] < 8.0).expect("a board at the south end of the east wall");
    assert!(near(len3(south_corner), 9.5), "{south_corner:?}");
}

// A board is as sure as what it rests on: under an Assumed roof the walls that rise are Assumed (a Known wall, an Assumed roof);
// the walls under an eave are as they were, Known, resting on their own height. Known walls under a Known roof are Known.
#[test]
fn a_board_under_an_assumed_roof_is_assumed_and_one_under_an_eave_is_as_it_was() {
    let (mut job, walls) = house(Some("PGPG"), known(), Status::Assumed);
    let s: Vec<Status> = walls.iter().map(|w| quantity(&mut job, w).status).collect();
    assert_eq!(s, [Status::Known, Status::Assumed, Status::Known, Status::Assumed]);
    let ps = trim::pieces(&mut job);
    for p in ps.iter().filter(|p| p.kind == PK::CornerBoard) {
        let want = if p.of == walls[1] || p.of == walls[3] { Status::Assumed } else { Status::Known };
        assert_eq!(p.status, want, "{p:?}");
    }
    let (mut job, walls) = house(Some("PGPG"), known(), Status::Known);
    for w in &walls {
        assert_eq!(quantity(&mut job, w).status, Status::Known, "{w}");
    }
}

// A line priced above the stone (call 194) takes the stone off each corner's own height: the shed's east wall with 3 ft:
// (9.5 - 3) + (17.5 - 3) = 21.00 LF; with 10 ft: nothing at the 9.5 ft corner (never less than nothing), 17.5 - 10 = 7.50 LF at the other.
#[test]
fn a_wainscot_is_taken_off_each_corners_own_height() {
    for (entry, want) in [("3 ft", 21.0), ("10 ft", 7.5)] {
        let (mut job, walls) = house(Some("PGGG"), assumed(), Status::Assumed);
        let w = &walls[1];
        job.remove_line(w, LINE).unwrap();
        job.add_line(w, "Boards above", "corner_board_length", Some(Class::Material), false).unwrap();
        job.set_formula(w, Some("Boards above"), "corner_board_length_above_wainscot", &[("wainscot".to_string(), entry.to_string())], Status::Assumed, "made-up").unwrap();
        let above = {
            job.derive();
            let e = job.find(w).unwrap();
            job.world.get::<Measure>(e).and_then(|m| m.0.get("corner_board_length_above_wainscot")).cloned().expect("the figure is on the wall")
        };
        assert!(near(above.value.unwrap(), want), "{entry}: {above:?}");
        let e = job.find(w).unwrap();
        let q = job.world.get::<LineQuantities>(e).unwrap().0["Boards above"].quantity.clone();
        assert!(near(q.value.unwrap(), want), "{entry}: {q:?}");
        let ps = trim::pieces(&mut job);
        let drawn: Vec<&trim::Piece> = ps.iter().filter(|p| &p.of == w && p.line == "Boards above").collect();
        let got: f64 = drawn.iter().map(|p| len3(p)).sum();
        assert!(near(got, want), "{entry}: the pieces lie {got} LF: {drawn:?}");
        assert_eq!(drawn.len(), if entry == "3 ft" { 2 } else { 1 });
    }
}

// A roof whose height over the wall cannot be worked leaves the board with no figure, and says why; never the wall's height as if
// there were no roof. The line that prices it has no quantity, and no piece is drawn.
#[test]
fn a_roof_that_cannot_be_placed_leaves_the_boards_of_the_wall_under_it_with_no_figure_and_says_why() {
    let (mut job, walls) = house(Some("PGPG"), assumed(), Status::Assumed);
    let plane = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        q.iter(&job.world).find(|(_, k)| **k == Kind::RoofPlane).map(|(n, _)| n.path.clone()).unwrap()
    };
    let e = job.find(&plane).unwrap();
    job.world.get_mut::<Pitch>(e).unwrap().0.value = None;
    job.world.get_mut::<Pitch>(e).unwrap().0.status = Status::Unknown;
    let d = figure(&mut job, &walls[3], "corner_board_length");
    assert_eq!(d.value, None, "{d:?}");
    assert!(!d.note.is_empty(), "it says why");
    let q = quantity(&mut job, &walls[3]);
    assert_eq!(q.value, None, "{q:?}");
    let laid = trim::lay(&mut job);
    assert!(laid.pieces.iter().all(|p| !(p.of == walls[3] && p.kind == PK::CornerBoard)));
    assert!(laid.left_out.iter().any(|s| s.contains("has no figure")), "{:?}", laid.left_out);
}
