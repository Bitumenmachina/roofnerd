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
