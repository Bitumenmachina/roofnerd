//! Call 184 step 1, the model's half: where each piece of the outside trim lies (fascia, rake fascia, soffit, rake soffit,
//! corner boards, casing). Everything is made up. The hand figures are those of STATUS.md ("Where each piece of outside trim
//! lies"), worked beside each test; where the model's own rule gives another figure the test says so.

use plumbline_world::editing as ed;
use plumbline_world::trim::{self, PieceKind as PK};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up tracing" }
}
fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn roof(job: &mut Job, ring: &[[f64; 2]], status: Status) -> String {
    let rise = units::parse_pitch("6:12").unwrap();
    let (len, unit) = units::parse_length("9 ft").unwrap();
    let v = |x| Val { value: Some(x), unit: units::PITCH.into(), status, source: "made-up tracing".into() };
    let gable = Val { value: Some(Rise::Gable), unit: String::new(), status, source: "made-up tracing".into() };
    let footprint = Footprint {
        edges: (0..ring.len()).map(|i| if ring.len() == 4 && i % 2 == 1 { gable.clone() } else { v(Rise::Pitch(rise)) }).collect(),
        points: Val { value: Some(ring.to_vec()), unit: units::FT.into(), status, source: "made-up tracing".into() },
        eave_height: Val { value: Some(len), unit: unit.into(), status, source: "made-up tracing".into() },
    };
    solve::new_roof(job, "R", footprint).unwrap().0
}
fn of_kind(job: &mut Job, kind: Kind) -> Vec<String> {
    let mut q = job.world.query::<(&Node, &Kind)>();
    let mut all: Vec<String> = q.iter(&job.world).filter(|(_, k, )| **k == kind).map(|(n, _)| n.path.clone()).collect();
    all.sort();
    all
}
fn quantity(job: &mut Job, path: &str, line: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<LineQuantities>(e).unwrap().0[line].quantity.clone()
}
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3
}
fn at(p: [f64; 3], x: f64, y: f64, z: f64) -> bool {
    near(p[0], x) && near(p[1], y) && near(p[2], z)
}
fn len3(p: &trim::Piece) -> f64 {
    ((p.to[0] - p.from[0]).powi(2) + (p.to[1] - p.from[1]).powi(2) + (p.to[2] - p.from[2]).powi(2)).sqrt()
}

struct House {
    job: Job,
    walls: Vec<String>,
    eaves: Vec<String>,
    rakes: Vec<String>,
    win: String,
    door: String,
}

/// The demonstration's house: a box 24 by 16 on the walls' lines, walls 6 in, 9 ft; a gable roof 1 ft outside the lines all round,
/// ridge east-west, 6 in 12; a window 3 by 5 (sill 2.5 ft, centre 18 ft along) and a door 3 by 6 ft 8 in (centre 6 ft along) in the
/// south wall; the starter-like lines given to every eave, rake, wall and opening.
fn house(status: ed::Entered<'static>, roof_status: Status, lines: bool) -> House {
    let mut job = Job::new("Box (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", status).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", status).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), status).unwrap();
    let wt = ed::opening_type(&mut job, "window 3 ft", Kind::Window, "3 ft", "5 ft", status).unwrap();
    let dt = ed::opening_type(&mut job, "door 3 ft", Kind::Door, "3 ft", "6 ft 8 in", status).unwrap();
    let win = ed::opening(&mut job, &walls[0], Kind::Window, &wt, ed::OnWall::Offset("18 ft"), "2.5 ft", status).unwrap();
    let door = ed::opening(&mut job, &walls[0], Kind::Door, &dt, ed::OnWall::Offset("6 ft"), "0 ft", status).unwrap();
    roof(&mut job, &[[-1.0, -1.0], [25.0, -1.0], [25.0, 17.0], [-1.0, 17.0]], roof_status);
    let (eaves, rakes) = (of_kind(&mut job, Kind::Eave), of_kind(&mut job, Kind::Rake));
    if lines {
        for e in &eaves {
            job.add_line(e, "Fascia", "true_length", Some(Class::Material), false).unwrap();
            job.add_line(e, "Soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
            job.add_line(e, "Gutter", "true_length", Some(Class::Material), false).unwrap();
        }
        for r in &rakes {
            job.add_line(r, "Rake fascia", "true_length", Some(Class::Material), false).unwrap();
            job.add_line(r, "Rake soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
        }
        for w in &walls {
            job.add_line(w, "Corner boards", "corner_board_length", Some(Class::Material), false).unwrap();
        }
        for o in [&win, &door] {
            job.add_line(o, "Exterior casing", "casing_length", Some(Class::Material), false).unwrap();
        }
    }
    job.derive();
    House { job, walls, eaves, rakes, win, door }
}
fn of<'a>(ps: &'a [trim::Piece], kind: PK) -> Vec<&'a trim::Piece> {
    ps.iter().filter(|p| p.kind == kind).collect()
}

// By hand: fascia 2 pieces, each 26 ft, from (-1,-1) to (25,-1) looking south and from (-1,17) to (25,17) looking north.
// The height is the model's: the roof's eave height, 9 ft, the height of the roof surface at the edge; the board hangs from it,
// so the middle of its back is half its width (5 1/2 in drawn) below: 9 - 0.2292 = 8.7708. 2 x 26 = 52.00 LF, the priced 52.00.
#[test]
fn fascia_lies_on_each_eave_at_the_roofs_edge_and_adds_to_the_priced_line() {
    let mut h = house(assumed(), Status::Assumed, true);
    let ps = trim::pieces(&mut h.job);
    let f = of(&ps, PK::Fascia);
    assert_eq!(f.len(), 2, "{f:?}");
    let z = 9.0 - 5.5 / 12.0 / 2.0;
    let south = f.iter().find(|p| near(p.from[1], -1.0)).expect("a fascia on the south eave");
    let north = f.iter().find(|p| near(p.from[1], 17.0)).expect("a fascia on the north eave");
    assert!(at(south.from, -1.0, -1.0, z) && at(south.to, 25.0, -1.0, z), "{south:?}");
    assert!(at(north.from, -1.0, 17.0, z) && at(north.to, 25.0, 17.0, z), "{north:?}");
    assert!(at(south.face, 0.0, -1.0, 0.0) && at(north.face, 0.0, 1.0, 0.0), "{:?} {:?}", south.face, north.face);
    for p in &f {
        assert!(h.eaves.contains(&p.of), "it says which eave it trims: {}", p.of);
        assert_eq!(p.line, "Fascia");
        assert!(near(p.width, 5.5 / 12.0) && near(p.thickness, 0.75 / 12.0), "{p:?}");
        assert!(p.sized_from.contains("the line names no size") && p.sized_from.contains("drawn at"), "{}", p.sized_from);
        assert_eq!(p.status, Status::Assumed);
    }
    assert!(near(f.iter().map(|p| len3(p)).sum::<f64>(), 52.0));
    let priced: f64 = h.eaves.clone().iter().map(|e| quantity(&mut h.job, e, "Fascia").value.unwrap()).sum();
    assert!(near(priced, 52.0));
}

// Rake fascia: 4 pieces, each from an eave's end up to the ridge's end: on plan from north -1 to 8 at east -1 and 25 and from 17 to 8;
// rising from 9 to 13.5 ft (6 in 12 over 9 ft); each 9 x 1.118034 = 10.0623 ft along the slope. 4 x 10.0623 = 40.25 LF.
#[test]
fn rake_fascia_climbs_from_each_eaves_end_to_the_ridge_and_adds_to_the_priced_line() {
    let mut h = house(assumed(), Status::Assumed, true);
    let ps = trim::pieces(&mut h.job);
    let r = of(&ps, PK::RakeFascia);
    assert_eq!(r.len(), 4, "{r:?}");
    let drop = 5.5 / 12.0 / 2.0;
    let mut ends: Vec<(f64, f64, f64)> = r.iter().map(|p| (p.from[0], p.from[1], p.to[1])).collect();
    ends.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let want = [(-1.0, -1.0, 8.0), (-1.0, 17.0, 8.0), (25.0, -1.0, 8.0), (25.0, 17.0, 8.0)];
    for (got, want) in ends.iter().zip(want) {
        assert!(near(got.0, want.0) && near(got.1, want.1) && near(got.2, want.2), "{ends:?}");
    }
    for p in &r {
        assert!(near(p.from[2], 9.0 - drop) && near(p.to[2], 13.5 - drop), "low end first, rising 4.5 ft: {p:?}");
        assert!(near(len3(p), 10.0623), "{}", len3(p));
        assert!(h.rakes.contains(&p.of));
        // It looks out of the gable: east at the east end, west at the west end, level.
        assert!(near(p.face[2], 0.0) && near(p.face[0].abs(), 1.0), "{:?}", p.face);
        assert_eq!(p.face[0] > 0.0, p.from[0] > 0.0);
    }
    assert!(near(r.iter().map(|p| len3(p)).sum::<f64>(), 40.2492));
}

// Soffit: 2 pieces under the eaves, each 26 ft by 0.75 ft (the overhang), looking down: 2 x 26 x 0.75 = 39.00 SF. Rake soffit: 4 pieces,
// each 10.0623 by 0.75: 30.19 SF.
#[test]
fn soffit_lies_under_the_overhang_looks_down_and_its_area_is_the_priced_line() {
    let mut h = house(assumed(), Status::Assumed, true);
    let ps = trim::pieces(&mut h.job);
    let s = of(&ps, PK::Soffit);
    assert_eq!(s.len(), 2);
    for p in &s {
        assert!(near(p.width, 0.75), "its width is the overhang: {p:?}");
        assert!(near(len3(p), 26.0));
        assert!(near(p.face[2], -1.0) && near(p.face[0], 0.0) && near(p.face[1], 0.0), "{:?}", p.face);
        assert!(p.sized_from.contains("overhang") && p.sized_from.contains("the line names no size"), "{}", p.sized_from);
    }
    // The south one's back is half the overhang in from the roof's edge (north of -1): -1 + 0.375.
    let south = s.iter().find(|p| p.from[1] < 8.0).unwrap();
    assert!(near(south.from[1], -1.0 + 0.375) && near(south.to[1], -1.0 + 0.375), "{south:?}");
    assert!(near(s.iter().map(|p| len3(p) * p.width).sum::<f64>(), 39.0));
    let rs = of(&ps, PK::RakeSoffit);
    assert_eq!(rs.len(), 4);
    for p in &rs {
        assert!(near(len3(p), 10.0623) && near(p.width, 0.75));
        // Level across, sloping along: its face looks down and a little out of the slope's way, never up.
        assert!(p.face[2] < -0.8, "{:?}", p.face);
        let n = (p.face[0].powi(2) + p.face[1].powi(2) + p.face[2].powi(2)).sqrt();
        assert!(near(n, 1.0));
    }
    assert!(near(rs.iter().map(|p| len3(p) * p.width).sum::<f64>(), 30.1869));
}

// Corner boards: 8 pieces, 2 at each of the 4 outside corners (one on each face), each from the wall's foot to its top at that corner:
// 9 ft on the south and north walls (under an eave), 9.5 ft on the east and west walls (under a rake; call 204: it was 9 ft on all).
// The model's rule: a board's back lies on its wall's outside face, flush with the corner of the outside faces, so the middle of its
// back is half a board (3 1/2 in) in from the corner along the wall: the south-west corner of the outside faces is (-0.25, -0.25), the
// south wall's board is at east -0.25 + 0.1458, north -0.25 (the lead's hand figure puts the piece AT the corner; it is its outer edge).
#[test]
fn corner_boards_stand_two_at_each_outside_corner_nine_feet_on_their_own_faces() {
    let mut h = house(assumed(), Status::Assumed, true);
    let ps = trim::pieces(&mut h.job);
    let c = of(&ps, PK::CornerBoard);
    assert_eq!(c.len(), 8, "{c:?}");
    let half = 3.5 / 12.0 / 2.0;
    let sw_on_south = c.iter().find(|p| p.of == h.walls[0] && near(p.from[0], -0.25 + half)).expect("south wall, west end");
    assert!(at(sw_on_south.from, -0.25 + half, -0.25, 0.0) && at(sw_on_south.to, -0.25 + half, -0.25, 9.0), "{sw_on_south:?}");
    assert!(at(sw_on_south.face, 0.0, -1.0, 0.0));
    let sw_on_west = c.iter().find(|p| p.of == h.walls[3] && near(p.from[1], -0.25 + half)).expect("west wall, south end");
    assert!(at(sw_on_west.from, -0.25, -0.25 + half, 0.0) && at(sw_on_west.face, -1.0, 0.0, 0.0), "{sw_on_west:?}");
    let ne_on_north = c.iter().find(|p| p.of == h.walls[2] && near(p.from[0], 24.25 - half)).expect("north wall, east end");
    assert!(at(ne_on_north.from, 24.25 - half, 16.25, 0.0) && at(ne_on_north.face, 0.0, 1.0, 0.0), "{ne_on_north:?}");
    for p in &c {
        let rises = p.of == h.walls[1] || p.of == h.walls[3];
        assert!(near(len3(p), if rises { 9.5 } else { 9.0 }) && near(p.from[0], p.to[0]) && near(p.from[1], p.to[1]), "{p:?}");
        assert!(near(p.width, 3.5 / 12.0) && near(p.thickness, 0.75 / 12.0));
        assert_eq!(p.line, "Corner boards");
        assert_eq!(p.status, Status::Assumed);
    }
    for w in &h.walls {
        assert_eq!(c.iter().filter(|p| &p.of == w).count(), 2, "{w}");
    }
    assert!(near(c.iter().map(|p| len3(p)).sum::<f64>(), 74.0), "4 x 9.0 + 4 x 9.5 = 74.00 (72.00 before call 204)");
}

// Casing: the window 4 pieces (head 3, sill 3, two sides 5: 16.00 LF); the door 3 pieces (head 3, two sides 6 ft 8 in: 16.33 LF, none
// across its foot), each on the south wall's outside face (north -0.25), looking south. The model's rule: the head lies over the
// opening, the sill under it, a side to each side of it; each piece is the priced length (the head is the width of the opening).
#[test]
fn casing_goes_round_a_window_on_four_sides_and_a_door_on_three_on_the_outside_face() {
    let mut h = house(assumed(), Status::Assumed, true);
    let ps = trim::pieces(&mut h.job);
    let c = of(&ps, PK::Casing);
    let (win, door): (Vec<&trim::Piece>, Vec<&trim::Piece>) = c.iter().copied().partition(|p| p.of == h.win);
    assert_eq!((win.len(), door.len()), (4, 3));
    assert_eq!(c.len(), 7);
    for p in &c {
        assert!(near(p.from[1], -0.25) && near(p.to[1], -0.25) && at(p.face, 0.0, -1.0, 0.0), "{p:?}");
        assert_eq!(p.line, "Exterior casing");
    }
    assert!(near(win.iter().map(|p| len3(p)).sum::<f64>(), 16.0));
    assert!(near(door.iter().map(|p| len3(p)).sum::<f64>(), 16.0 + 1.0 / 3.0));
    // The window: 3 ft wide centred 18 ft along, sill 2.5 ft, 5 ft high: head and sill 3 ft long, level; the sides 5 ft, plumb.
    let mut level: Vec<f64> = win.iter().filter(|p| near(p.from[2], p.to[2])).map(|p| len3(p)).collect();
    level.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(level.len() == 2 && near(level[0], 3.0) && near(level[1], 3.0), "{level:?}");
    let plumb: Vec<&&trim::Piece> = win.iter().filter(|p| near(p.from[0], p.to[0])).collect();
    assert_eq!(plumb.len(), 2);
    assert!(plumb.iter().all(|p| near(len3(p), 5.0) && near(p.from[2].min(p.to[2]), 2.5)));
    // The door: nothing lies across its foot (the floor): its level pieces lie above its head's line, none at height 0.
    assert!(door.iter().filter(|p| near(p.from[2], p.to[2])).all(|p| p.from[2] > 6.0), "{door:?}");
    assert_eq!(door.iter().filter(|p| near(p.from[2], p.to[2])).count(), 1);
    assert!(door.iter().all(|p| p.from[2].min(p.to[2]) > -1e-9));
    // Sizes: not named by the line, so drawn: 3 1/2 in by 3/4 in, said.
    assert!(c.iter().all(|p| near(p.width, 3.5 / 12.0) && near(p.thickness, 0.75 / 12.0) && p.sized_from.contains("the line names no size")));
}

// One arithmetic: for every line, the pieces on an element add up to that element's priced quantity (LF; SF for a soffit).
#[test]
fn the_pieces_of_a_kind_on_an_element_add_up_to_its_priced_line() {
    let mut h = house(assumed(), Status::Assumed, true);
    let ps = trim::pieces(&mut h.job);
    let cases: [(PK, &str, bool, Vec<String>); 8] = [
        (PK::Fascia, "Fascia", false, h.eaves.clone()),
        (PK::Soffit, "Soffit", true, h.eaves.clone()),
        (PK::RakeFascia, "Rake fascia", false, h.rakes.clone()),
        (PK::RakeSoffit, "Rake soffit", true, h.rakes.clone()),
        (PK::CornerBoard, "Corner boards", false, h.walls.clone()),
        (PK::Casing, "Exterior casing", false, vec![h.win.clone(), h.door.clone()]),
        (PK::Fascia, "Fascia", false, vec![]),
        (PK::Casing, "Exterior casing", false, vec![]),
    ];
    let mut checked = 0;
    for (kind, line, area, paths) in cases {
        for path in paths {
            let q = quantity(&mut h.job, &path, line);
            let got: f64 = ps.iter().filter(|p| p.kind == kind && p.of == path).map(|p| len3(p) * if area { p.width } else { 1.0 }).sum();
            assert!((got - q.worked().unwrap()).abs() < 1e-3, "{path} {line}: pieces {got}, priced {:?}", q.value);
            checked += 1;
        }
    }
    assert_eq!(checked, 2 + 2 + 4 + 4 + 4 + 2);
}

// A piece exists only where the estimate prices it: an eave with a fascia line has a fascia, one without has none; the soffit
// and the gutter alone make no fascia.
#[test]
fn a_piece_exists_only_where_a_line_prices_it() {
    let mut h = house(assumed(), Status::Assumed, false);
    assert!(trim::pieces(&mut h.job).is_empty(), "no line, no piece");
    // Only the south eave (the first, edge 1) is given a fascia, and the other a gutter, which is not trim.
    let (a, b) = (h.eaves[0].clone(), h.eaves[1].clone());
    h.job.add_line(&a, "Fascia", "true_length", Some(Class::Material), false).unwrap();
    h.job.add_line(&b, "Gutter", "true_length", Some(Class::Material), false).unwrap();
    h.job.derive();
    let ps = trim::pieces(&mut h.job);
    assert_eq!(ps.len(), 1, "{ps:?}");
    assert_eq!((ps[0].kind, ps[0].of.as_str()), (PK::Fascia, a.as_str()));
}

// A piece is never Known unless everything it rests on is: Known drawing, Known piece; one Assumed thing under it makes it Assumed.
#[test]
fn a_piece_is_known_only_when_everything_it_rests_on_is() {
    let mut k = house(known(), Status::Known, true);
    let ps = trim::pieces(&mut k.job);
    assert!(!ps.is_empty());
    assert!(ps.iter().all(|p| p.status == Status::Known), "{:?}", ps.iter().filter(|p| p.status != Status::Known).map(|p| (p.kind, p.status)).collect::<Vec<_>>());
    // The same house with the roof only Assumed: what rests on the roof (fascia, soffit, rakes) is Assumed; the boards of the walls under an eave stay Known, and those of the
    // east and west walls, which rise to the roof, are Assumed with it (call 204 point 2).
    let mut m = house(known(), Status::Assumed, true);
    let ps = trim::pieces(&mut m.job);
    for p in &ps {
        let on_roof = matches!(p.kind, PK::Fascia | PK::RakeFascia | PK::Soffit | PK::RakeSoffit) || (p.kind == PK::CornerBoard && (p.of == m.walls[1] || p.of == m.walls[3]));
        assert_eq!(p.status == Status::Assumed, on_roof, "{:?} {:?}", p.kind, p.status);
    }
}

// A size the line names is the piece's size, sourced to the line: "1x6" is 3/4 in by 5 1/2 in, "5/4x4" 1 in by 3 1/2 in (the softwood
// standard's actual sizes). A soffit's width is still its overhang.
#[test]
fn a_size_the_line_names_is_the_pieces_size_and_says_so() {
    let mut h = house(assumed(), Status::Assumed, false);
    let (e, w) = (h.eaves[0].clone(), h.walls[1].clone());
    h.job.add_line(&e, "Fascia, 1x8 primed pine", "true_length", Some(Class::Material), false).unwrap();
    h.job.add_line(&w, "Corner boards, 5/4x4 PVC", "corner_board_length", Some(Class::Material), false).unwrap();
    h.job.derive();
    let ps = trim::pieces(&mut h.job);
    let f = ps.iter().find(|p| p.kind == PK::Fascia).unwrap();
    assert!(near(f.width, 7.25 / 12.0) && near(f.thickness, 0.75 / 12.0), "{f:?}");
    assert!(f.sized_from.contains("1x8") && !f.sized_from.contains("names no size"), "{}", f.sized_from);
    let c = ps.iter().find(|p| p.kind == PK::CornerBoard).unwrap();
    assert!(near(c.width, 3.5 / 12.0) && near(c.thickness, 1.0 / 12.0), "{c:?}");
    assert!(c.sized_from.contains("5/4x4"));
}

// The L-shaped house: 10 corner boards, 90.00 LF; none at the inside corner (12,16); the two walls that meet there have one each.
#[test]
fn the_l_shaped_house_has_ten_corner_boards_and_none_at_the_inside_corner() {
    let mut job = Job::new("L (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", assumed()).unwrap();
    let ring = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [12.0, 16.0], [12.0, 26.0], [0.0, 26.0]];
    let walls = ed::walls(&mut job, &level, &w6, &ring, true, ed::Top::Height("9 ft"), assumed()).unwrap();
    for w in &walls {
        job.add_line(w, "Corner boards", "corner_board_length", Some(Class::Material), false).unwrap();
    }
    job.derive();
    let ps = trim::pieces(&mut job);
    assert_eq!(ps.len(), 10, "{ps:?}");
    assert!(near(ps.iter().map(|p| len3(p)).sum::<f64>(), 90.0));
    for w in &walls {
        let q = quantity(&mut job, w, "Corner boards").value.unwrap();
        let got: f64 = ps.iter().filter(|p| &p.of == w).map(|p| len3(p)).sum();
        assert!(near(got, q), "{w}: {got} {q}");
    }
    // No board within a board's width of the inside corner's faces: the inside corner of the outside faces is (12.25, 15.75).
    assert!(ps.iter().all(|p| (p.from[0] - 12.0).hypot(p.from[1] - 16.0) > 0.6), "{:?}", ps.iter().map(|p| p.from).collect::<Vec<_>>());
    // The wall whose end is at the inside corner looks north (it is the box's north face east of the wing): its one board is at its far end.
    let on3: Vec<&trim::Piece> = ps.iter().filter(|p| p.of == walls[2]).collect();
    assert_eq!(on3.len(), 1);
    assert!(near(on3[0].from[0], 24.25 - 3.5 / 12.0 / 2.0) && near(on3[0].from[1], 16.25) && at(on3[0].face, 0.0, 1.0, 0.0), "{:?}", on3[0]);
}

// The roof drawn alone: no wall under any edge; its soffit has no piece and the list says why. Its fascia, priced by the length alone,
// still lies on the edge.
#[test]
fn a_roof_with_no_wall_under_its_edge_has_no_soffit_piece_and_the_list_says_why() {
    let mut job = Job::new("Roof alone (not a real job)", "test", "2026-11-23");
    roof(&mut job, &[[-1.0, -1.0], [25.0, -1.0], [25.0, 17.0], [-1.0, 17.0]], Status::Assumed);
    let eave = of_kind(&mut job, Kind::Eave)[0].clone();
    job.add_line(&eave, "Soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
    job.add_line(&eave, "Fascia", "true_length", Some(Class::Material), false).unwrap();
    job.derive();
    let laid = trim::lay(&mut job);
    assert!(laid.pieces.iter().all(|p| p.kind != PK::Soffit), "{:?}", laid.pieces);
    assert_eq!(laid.pieces.iter().filter(|p| p.kind == PK::Fascia).count(), 1);
    assert!(laid.left_out.iter().any(|l| l.contains(&eave) && l.contains("Soffit") && l.contains("no wall is found under this edge")), "{:?}", laid.left_out);
}

// Where a person's quantity is not the figure alone (a waste factor in the formula), the pieces would not add up to it: they are not
// drawn, and the list says so, never silently.
#[test]
fn a_line_whose_quantity_is_not_the_length_alone_is_said_and_not_drawn() {
    let mut h = house(assumed(), Status::Assumed, false);
    let e = h.eaves[0].clone();
    h.job.add_line(&e, "Fascia", "true_length * 1.1", Some(Class::Material), false).unwrap();
    h.job.derive();
    let laid = trim::lay(&mut h.job);
    assert!(laid.pieces.is_empty());
    assert!(laid.left_out.iter().any(|l| l.contains(&e) && l.contains("Fascia") && l.contains("not the length alone")), "{:?}", laid.left_out);
}
