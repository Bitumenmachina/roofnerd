//! Call 191: what the judge of the sheet's scale, two drawings and the trim pieces found. Everything is made up; the hand figures are
//! STATUS.md's ("The judge's findings fixed: a demonstration"), worked beside each test.

use plumbline_world::editing as ed;
use plumbline_world::plans::{self, FoundOpening, FoundWall, Read};
use plumbline_world::trim::{self, PieceKind as PK};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up tracing" }
}

/// A roof over a 24 by 16 ft box of 6 in walls, a hip roof (every edge at 6:12) 1 ft outside the walls' lines: 26 by 18 ft on plan, an
/// overhang of 9 in beyond the walls' faces (1 ft less the half of the 6 in wall).
fn hip_house(soffit: bool, walls: bool) -> (Job, Vec<String>) {
    let mut job = Job::new("Hip box (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    if walls {
        let w6 = ed::wall_type(&mut job, "w6", "6 in", assumed()).unwrap();
        ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), assumed()).unwrap();
    }
    let rise = units::parse_pitch("6:12").unwrap();
    let (len, unit) = units::parse_length("9 ft").unwrap();
    let v = || Val { value: Some(Rise::Pitch(rise)), unit: units::PITCH.into(), status: Status::Assumed, source: "made-up tracing".into() };
    let ring = [[-1.0, -1.0], [25.0, -1.0], [25.0, 17.0], [-1.0, 17.0]];
    let footprint = Footprint {
        edges: (0..4).map(|_| v()).collect(),
        points: Val { value: Some(ring.to_vec()), unit: units::FT.into(), status: Status::Assumed, source: "made-up tracing".into() },
        eave_height: Val { value: Some(len), unit: unit.into(), status: Status::Assumed, source: "made-up tracing".into() },
    };
    solve::new_roof(&mut job, "R", footprint).unwrap();
    let mut q = job.world.query::<(&Node, &Kind)>();
    let mut eaves: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == Kind::Eave).map(|(n, _)| n.path.clone()).collect();
    eaves.sort();
    if soffit {
        for e in &eaves {
            job.add_line(e, "Soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
        }
    }
    (job, eaves)
}

// C10. By hand: four eaves, 26, 18, 26 and 18 ft: 88 LF of fascia priced. The first eave's fascia is priced on its plan length (26 LF),
// the other three on their true length (62 LF): 3 pieces and 62 LF drawn, 88 LF priced, 26 LF priced and not drawn.
#[test]
fn a_fascia_line_priced_on_another_figure_is_said_and_set_against_the_total() {
    let (mut job, eaves) = hip_house(false, true);
    for (i, e) in eaves.iter().enumerate() {
        job.add_line(e, "Fascia", if i == 0 { "plan_length" } else { "true_length" }, Some(Class::Material), false).unwrap();
    }
    job.derive();
    let laid = trim::lay(&mut job);
    let n = laid.pieces.iter().filter(|p| p.kind == PK::Fascia).count();
    let drawn: f64 = laid.pieces.iter().filter(|p| p.kind == PK::Fascia).map(|p| ((p.to[0] - p.from[0]).powi(2) + (p.to[1] - p.from[1]).powi(2) + (p.to[2] - p.from[2]).powi(2)).sqrt()).sum();
    assert_eq!(n, 3, "{:?}", laid.pieces);
    assert!((drawn - 62.0).abs() < 1e-3, "{drawn}");
    assert!((laid.priced_of(PK::Fascia).unwrap() - 88.0).abs() < 1e-3, "{:?}", laid.priced);
    assert!(laid.left_out.iter().any(|l| l.contains(&eaves[0]) && l.contains("Fascia") && l.contains("plan_length") && l.contains("26.00 LF")), "{:?}", laid.left_out);
    // A typed number is a quantity in another unit: said, and set against no total.
    let (mut job, eaves) = hip_house(false, true);
    job.add_line(&eaves[0], "Fascia", "true_length", Some(Class::Material), false).unwrap();
    job.add_line(&eaves[1], "Fascia", "26", Some(Class::Material), false).unwrap();
    job.derive();
    let laid = trim::lay(&mut job);
    assert!(laid.left_out.iter().any(|l| l.contains(&eaves[1]) && l.contains("\"26\"")), "{:?}", laid.left_out);
}

// C11. A roof drawn alone, a soffit line on each eave: no wall under it, so no overhang. The kind's total is no figure, not nothing.
#[test]
fn a_soffit_whose_overhang_is_unknown_is_no_figure_and_not_nothing() {
    let (mut job, _) = hip_house(true, false);
    job.derive();
    let laid = trim::lay(&mut job);
    assert!(laid.pieces.iter().all(|p| p.kind != PK::Soffit));
    assert_eq!(laid.priced_of(PK::Soffit), None, "{:?}", laid.priced);
    assert_eq!(laid.no_figure_of(PK::Soffit), 4, "{:?}", laid.no_figure);
}

// C12. By hand: 2 x (26 + 18) x 0.75 = 66.00 SF priced; each of the four corners is a 0.75 by 0.75 ft square in two runs: 0.5625 SF each,
// 2.25 SF in all. The priced figure does not change.
#[test]
fn the_corners_of_a_hip_roofs_soffit_are_said_in_two_runs_and_the_price_is_unchanged() {
    let (mut job, _) = hip_house(true, true);
    job.derive();
    let laid = trim::lay(&mut job);
    assert!((laid.priced_of(PK::Soffit).unwrap() - 66.0).abs() < 1e-3, "{:?}", laid.priced);
    let corners = trim::soffit_corners(&mut job.world);
    assert_eq!(corners.len(), 4, "{corners:?}");
    assert!(corners.iter().all(|c| (c.area - 0.5625).abs() < 1e-6), "{corners:?}");
    let all: f64 = corners.iter().map(|c| c.area).sum();
    assert!((all - 2.25).abs() < 1e-6, "{all}");
}

// J191-8. Walls laid from a sheet stand where the sheet's scale put them: a scale set again names them, as traced walls are named.
#[test]
fn walls_laid_from_a_sheet_are_named_when_its_scale_changes() {
    let mut job = Job::new("A sheet (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    let page = Read { file: "plans/made-up-plan.pdf", page: 1, bytes: 758, checksum: "0123456789abcdef", size: [792.0, 612.0] };
    let d = plans::add(&mut job, &level, "A-1", page, assumed()).unwrap();
    plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30 ft", assumed()).unwrap();
    let walls = vec![
        FoundWall { from: [0.0, 0.0], to: [30.0, 0.0], thickness: 0.5, measured: true, height: None, openings: vec![] },
        FoundWall { from: [30.0, 0.0], to: [30.0, 20.0], thickness: 0.5, measured: true, height: None, openings: vec![FoundOpening { window: false, from: 4.0, to: 7.0, sill: 0.0, head: 6.67 }] },
    ];
    let made = plans::lay_found_as(&mut job, &level, "found on sheet A-1", None, 9.0, None, &walls).unwrap();
    assert_eq!(made.walls.len(), 2);
    // Scaled again, 1/8 in to the foot: what was laid stays where the old scale put it, and is named.
    let done = plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "60 ft", assumed()).unwrap();
    for w in &made.walls {
        assert!(done.left_at_earlier_scale.contains(w), "{w} not named: {:?}", done.left_at_earlier_scale);
    }
    assert!(plans::laid_from(&mut job, &d).unwrap().len() >= 2);
    // A second laying onto the same level says what to do about it.
    let again = plans::lay_found_as(&mut job, &level, "found on sheet A-1", None, 9.0, None, &walls).unwrap_err();
    assert!(again.contains("already has 2 walls found on sheet A-1") && again.contains("level of its own"), "{again}");
}

// J191-14. By hand: the 30 by 20 ft house laid from its sheet, 4 walls (100 LF) 9 ft high, each priced by gypsum board on its face at a
// made-up $3.00/SF: 100 x 9 = 900 SF of gross face, $2,700.00 (no door or window in these walls to take out). All of it rests on the sheet's
// scale, which was read off the sheet: the list asks after the scale and puts $2,700.00 on it.
#[test]
fn walls_laid_from_a_sheet_ride_on_that_sheets_scale_in_the_list_of_what_to_verify() {
    let mut job = Job::new("A sheet (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    let page = Read { file: "plans/made-up-plan.pdf", page: 1, bytes: 758, checksum: "0123456789abcdef", size: [792.0, 612.0] };
    let d = plans::add(&mut job, &level, "A-1", page, assumed()).unwrap();
    plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30 ft", assumed()).unwrap();
    let w = |a: [f64; 2], b: [f64; 2]| FoundWall { from: a, to: b, thickness: 0.5, measured: true, height: None, openings: vec![] };
    let walls = vec![w([0.0, 0.0], [30.0, 0.0]), w([30.0, 0.0], [30.0, 20.0]), w([30.0, 20.0], [0.0, 20.0]), w([0.0, 20.0], [0.0, 0.0])];
    let made = plans::lay_found_as(&mut job, &level, "found on sheet A-1", None, 9.0, None, &walls).unwrap();
    for p in &made.walls {
        job.add_line(p, "Gypsum board, hung and finished", "face_area", Some(Class::Material), false).unwrap();
        job.set_on_line(p, "unit_cost", Some("Gypsum board, hung and finished"), Some("$3.00/SF"), Status::Known, "made-up price").unwrap();
    }
    job.derive();
    let list = plumbline_world::vif::vif(&mut job);
    let on_scale = list.questions.iter().find(|q| q.path == d).and_then(|q| q.dollars);
    assert_eq!(on_scale, Some(2700.0), "{:#?}", list.questions.iter().map(|q| (q.path.clone(), q.dollars)).collect::<Vec<_>>());
}

// J191-16. By hand: a 40 by 20 ft house laid from its sheet, 120 LF of wall 9 ft high, an opening 8 ft wide laid as a door 7 ft high in the
// south wall: gross face 1,080 SF, less the door's 56 SF, 1,024 SF net at a made-up $3.00/SF = $3,072.00 (4 lines); the door at $500.00 EA:
// $3,572.00 riding on the scale of its sheet, 5 lines. A door priced alone: $500.00 riding.
fn house_with_a_door_laid_from_its_sheet(price_walls: bool) -> (Job, String) {
    let mut job = Job::new("A sheet (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    let page = Read { file: "plans/made-up-plan.pdf", page: 1, bytes: 758, checksum: "0123456789abcdef", size: [792.0, 612.0] };
    let d = plans::add(&mut job, &level, "A-1", page, assumed()).unwrap();
    plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30 ft", assumed()).unwrap();
    let w = |a: [f64; 2], b: [f64; 2], ops: Vec<FoundOpening>| FoundWall { from: a, to: b, thickness: 0.5, measured: true, height: None, openings: ops };
    let door = FoundOpening { window: false, from: 16.0, to: 24.0, sill: 0.0, head: 7.0 };
    let walls = vec![w([0.0, 0.0], [40.0, 0.0], vec![door]), w([40.0, 0.0], [40.0, 20.0], vec![]), w([40.0, 20.0], [0.0, 20.0], vec![]), w([0.0, 20.0], [0.0, 0.0], vec![])];
    let made = plans::lay_found_as(&mut job, &level, "found on sheet A-1", Some("said when the walls were found"), 9.0, None, &walls).unwrap();
    assert_eq!(made.doors.len(), 1);
    if price_walls {
        for p in &made.walls {
            job.add_line(p, "Lap siding", "net_area", Some(Class::Material), false).unwrap();
            job.set_on_line(p, "unit_cost", Some("Lap siding"), Some("$3.00/SF"), Status::Known, "made-up price").unwrap();
        }
    }
    let door = made.doors[0].clone();
    job.add_line(&door, "Entry door, prehung", "1", Some(Class::Material), false).unwrap();
    job.set_on_line(&door, "unit_cost", Some("Entry door, prehung"), Some("$500.00/EA"), Status::Known, "made-up price").unwrap();
    job.derive();
    (job, d)
}

#[test]
fn a_door_laid_from_a_sheet_rides_on_that_sheets_scale_in_the_list_of_what_to_verify() {
    let (mut job, d) = house_with_a_door_laid_from_its_sheet(true);
    let list = plumbline_world::vif::vif(&mut job);
    let q = list.questions.iter().find(|q| q.path == d).unwrap_or_else(|| panic!("no question on the drawing"));
    assert_eq!((q.dollars, q.lines), (Some(3572.0), 5), "{q:?}");
    // A door priced alone: $500.00 riding on the scale.
    let (mut job, d) = house_with_a_door_laid_from_its_sheet(false);
    let list = plumbline_world::vif::vif(&mut job);
    let q = list.questions.iter().find(|q| q.path == d).unwrap_or_else(|| panic!("no question on the drawing: the door's $500.00 does not ride on its scale"));
    assert_eq!(q.dollars, Some(500.0), "{q:?}");
}

// J191-17. By hand: an L-shaped house (the walls' lines (0,0) (24,0) (24,16) (12,16) (12,26) (0,26)), a hip roof 1 ft outside them, an overhang
// of 9 in. An L has six corners: five outside, each a 0.75 by 0.75 ft square in two runs, 5 x 0.5625 = 2.8125 SF; one inside, a square of
// 0.5625 SF in neither run. The formula gives 2.8125 - 0.5625 = 2.25 SF more than the roof has (4 x 0.5625: outside less inside corners is 4 for
// any closed run of right angles). (Call 191 item 17's first figures, 6 corners and 3.38 SF, count seven corners, which an L has not.)
#[test]
fn an_l_roofs_soffit_says_the_corners_in_two_runs_and_the_inside_corner_in_neither() {
    let mut job = Job::new("L box (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", assumed()).unwrap();
    ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [12.0, 16.0], [12.0, 26.0], [0.0, 26.0]], true, ed::Top::Height("9 ft"), assumed()).unwrap();
    let rise = units::parse_pitch("6:12").unwrap();
    let (len, unit) = units::parse_length("9 ft").unwrap();
    let v = || Val { value: Some(Rise::Pitch(rise)), unit: units::PITCH.into(), status: Status::Assumed, source: "made-up tracing".into() };
    let ring = [[-1.0, -1.0], [25.0, -1.0], [25.0, 17.0], [13.0, 17.0], [13.0, 27.0], [-1.0, 27.0]];
    let footprint = Footprint {
        edges: (0..6).map(|_| v()).collect(),
        points: Val { value: Some(ring.to_vec()), unit: units::FT.into(), status: Status::Assumed, source: "made-up tracing".into() },
        eave_height: Val { value: Some(len), unit: unit.into(), status: Status::Assumed, source: "made-up tracing".into() },
    };
    solve::new_roof(&mut job, "R", footprint).unwrap();
    let mut q = job.world.query::<(&Node, &Kind)>();
    let eaves: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == Kind::Eave).map(|(n, _)| n.path.clone()).collect();
    for e in &eaves {
        job.add_line(e, "Soffit", "true_length * overhang", Some(Class::Material), false).unwrap();
    }
    job.derive();
    let corners = trim::soffit_corners(&mut job.world);
    let twice: f64 = corners.iter().filter(|c| !c.inside).map(|c| c.area).sum();
    let neither: f64 = corners.iter().filter(|c| c.inside).map(|c| c.area).sum();
    assert_eq!((corners.iter().filter(|c| !c.inside).count(), corners.iter().filter(|c| c.inside).count()), (5, 1), "{corners:?}");
    assert!((twice - 2.8125).abs() < 1e-6 && (neither - 0.5625).abs() < 1e-6, "{twice} {neither}");
}

// J191-18. A figure to two places is rounded as the estimate rounds it: 1.125 reads 1.13, 2.8125 reads 2.81, 0.5625 reads 0.56.
#[test]
fn a_figure_to_two_places_is_rounded_as_the_estimate_rounds() {
    assert_eq!(view::figure2(1.125), "1.13");
    assert_eq!(view::figure2(2.8125), "2.81");
    assert_eq!(view::figure2(0.5625), "0.56");
    assert_eq!(view::figure2(3.375), "3.38");
    assert_eq!(view::figure2(1234.5), "1,234.50");
}
