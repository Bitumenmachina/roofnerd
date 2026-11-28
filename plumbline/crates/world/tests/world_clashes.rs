use plumbline_world::file::{load, save};
use plumbline_world::sample::sample;
use plumbline_world::value::Status;
use plumbline_world::*;

#[allow(dead_code)]
mod common;
use common::{measure_of, path_named, drawn, ducted, clashes_of, about};

#[test]
fn two_things_in_one_space_clash_and_two_things_close_clash_only_when_a_clearance_is_asked() {
    use plumbline_world::building::clash::clashes;
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, ..) = drawn();
    // Two boxes 2 ft each way on the floor, their middles 1.5 ft apart: they overlap by
    // 2 − 1.5 = 0.5 ft, and that is how far one has to move to clear the other.
    let a = ed::place(&mut job, &s, Kind::Equipment, "Boiler", [0.0, 0.0], "0 ft", Some([2.0, 2.0, 2.0]), None, how).unwrap();
    let b = ed::place(&mut job, &s, Kind::Equipment, "Tank", [1.5, 0.0], "0 ft", Some([2.0, 2.0, 2.0]), None, how).unwrap();
    assert_eq!(clashes_of(&mut job, None), [("Boiler".to_string(), "Tank".to_string(), true, false, Some(0.5), Status::Known)]);
    let found = clashes(&mut job, None).unwrap();
    assert_eq!((found.tested, found.left_out.len()), (2, 0));
    assert_eq!((found.found[0].a.as_str(), found.found[0].b.as_str()), (a.as_str(), b.as_str()));
    // Where: between the two faces that are inside each other, east 0.5 to 1.
    assert!(found.found[0].at[0] >= 0.5 - 1.0e-9 && found.found[0].at[0] <= 1.0 + 1.0e-9, "{:?}", found.found[0].at);
    // Moved by someone looking at the window until they just touch (middles 2 ft apart):
    // no clash. Asked for a clearance, they are closer than it by all of it, and the
    // answer is no surer than the move.
    let seen = ed::Entered { status: Status::Assumed, source: "set in the 3D view" };
    ed::move_by(&mut job, &b, 0.5, 0.0, seen).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());
    assert_eq!(clashes_of(&mut job, Some(0.5)), [("Boiler".to_string(), "Tank".to_string(), false, false, Some(0.0), Status::Assumed)]);
    // A quarter foot apart: nothing, unless a clearance of more than that is asked for.
    ed::move_by(&mut job, &b, 0.25, 0.0, seen).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());
    assert!(clashes_of(&mut job, Some(2.0 / 12.0)).is_empty());
    assert_eq!(clashes_of(&mut job, Some(0.5)), [("Boiler".to_string(), "Tank".to_string(), false, false, Some(0.25), Status::Assumed)]);
    // A clearance of nothing, or of a room's width, is refused.
    assert!(clashes(&mut job, Some(0.0)).is_err() && clashes(&mut job, Some(11.0)).is_err() && clashes(&mut job, Some(f64::NAN)).is_err());
    // A thing with no size has no shape: it is not tested, and that is said.
    let c = ed::place(&mut job, &s, Kind::Fixture, "Something", [0.0, 0.0], "0 ft", None, None, how).unwrap();
    let found = clashes(&mut job, None).unwrap();
    assert_eq!(found.tested, 2);
    assert!(found.left_out.iter().any(|(path, why)| *path == c && why.starts_with("it has no shape")), "{:?}", found.left_out);
}

#[test]
fn runs_clash_by_how_far_their_surfaces_are_in_each_other() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, ..) = drawn();
    let duct = ed::segment_type(&mut job, "6 in round", Kind::Duct, "6 in", None, how).unwrap();
    let wide = ed::segment_type(&mut job, "12 by 8", Kind::Duct, "12 in", Some("8 in"), how).unwrap();
    let pipe = ed::segment_type(&mut job, "3/4 in", Kind::Pipe, "3/4 in", None, how).unwrap();
    let name = |job: &mut Job, path: &str| job.find(path).and_then(|e| job.world.get::<Node>(e).map(|n| n.name.clone())).unwrap();
    // A 6 in duct east along y = 5 and a 3/4 in pipe north along x = 5, both 8 ft up:
    // their centre lines cross. To clear, one moves by both radii: 0.25 + 0.03125 ft.
    let d = ed::runs(&mut job, &s, Kind::Duct, &duct, &[[0.0, 5.0], [10.0, 5.0]], "8 ft", None, how).unwrap();
    let p = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, 0.0], [5.0, 10.0]], "8 ft", None, how).unwrap();
    let (dn, pn) = (name(&mut job, &d[0]), name(&mut job, &p[0]));
    let got = clashes_of(&mut job, None);
    assert_eq!(got.len(), 1);
    assert_eq!((got[0].0.as_str(), got[0].1.as_str(), got[0].2, got[0].3, got[0].5), (dn.as_str(), pn.as_str(), true, false, Status::Known));
    assert!(about(got[0].4, 0.2813), "{:?}", got[0].4);
    // The pipe raised to 8.2 ft: the centre lines are 0.2 apart, the surfaces 0.08125 in
    // each other. Raised to 8.5 ft: 0.21875 clear, a clash only for a clearance over that.
    ed::remove(&mut job, &p[0]).unwrap();
    let p = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, 0.0], [5.0, 10.0]], "8.2 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 0.0813), "{got:?}");
    ed::remove(&mut job, &p[0]).unwrap();
    let seen = ed::Entered { status: Status::Assumed, source: "set in the 3D view" };
    let p = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, 0.0], [5.0, 10.0]], "8.5 ft", None, seen).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());
    let got = clashes_of(&mut job, Some(0.25));
    assert!(got.len() == 1 && !got[0].2 && about(got[0].4, 0.2188) && got[0].5 == Status::Assumed, "{got:?}");
    assert!(clashes_of(&mut job, Some(0.2)).is_empty());
    ed::remove(&mut job, &p[0]).unwrap();
    ed::remove(&mut job, &d[0]).unwrap();
    // A duct 12 in wide and 8 in deep east along y = 5, its middle 8 ft up: its top is at
    // 8 ft 4 in and its sides at y = 4.5 and 5.5. A 3/4 in pipe over it at 8.35 ft is in
    // it by 0.3333 + 0.0313 − 0.35 = 0.0146 ft; at 8.4 ft it clears by 0.0354.
    let d = ed::runs(&mut job, &s, Kind::Duct, &wide, &[[0.0, 5.0], [10.0, 5.0]], "8 ft", None, how).unwrap();
    let p = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, 0.0], [5.0, 10.0]], "8.35 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 0.0146), "{got:?}");
    ed::remove(&mut job, &p[0]).unwrap();
    let p = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, 0.0], [5.0, 10.0]], "8.4 ft", None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());
    ed::remove(&mut job, &p[0]).unwrap();
    // A pipe beside it at the same height, along y = 5.52: its side is at 5.5 and the
    // pipe's surface at 5.4888, in it by 0.0113. Along y = 5.55 it clears.
    let p = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 5.52], [10.0, 5.52]], "8 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 0.0113), "{got:?}");
    ed::remove(&mut job, &p[0]).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 5.55], [10.0, 5.55]], "8 ft", None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());
    let _ = d;
}

/// J17A-20. Cutting a run in two moves nothing: what crosses it at the cut is as deep in
/// it as before, in each of the two pieces, at the same place.
#[test]
fn a_run_cut_where_another_crosses_it_still_clashes_by_as_much() {
    use plumbline_world::building::clash::clashes;
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, ..) = drawn();
    let duct = ed::segment_type(&mut job, "8 in round", Kind::Duct, "8 in", None, how).unwrap();
    let wide = ed::segment_type(&mut job, "12 by 8", Kind::Duct, "12 in", Some("8 in"), how).unwrap();
    let pipe = ed::segment_type(&mut job, "2 in", Kind::Pipe, "2 in", None, how).unwrap();
    // An 8 in duct east along y = 5 and a 2 in pipe north along x = 10, both 8 ft up: the
    // pipe is through the middle of the duct. To clear, one moves by both radii:
    // 4 in + 1 in = 5 in = 0.4167 ft.
    let d = ed::runs(&mut job, &s, Kind::Duct, &duct, &[[0.0, 5.0], [20.0, 5.0]], "8 ft", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[10.0, 0.0], [10.0, 10.0]], "8 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 0.4167), "{got:?}");
    // Cut 6 ft from the crossing: still one clash, with the piece the pipe is in.
    let far = ed::split(&mut job, &d[0], [16.0, 5.0], how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 0.4167), "{got:?}");
    let _ = far;
    // Cut at the crossing: the pipe is in the end of each piece, and each piece carries
    // on into the other. Two clashes, each by the same 5 in, both where the pipe is.
    ed::split(&mut job, &d[0], [10.0, 5.0], how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 2 && got.iter().all(|c| c.2 && !c.3 && about(c.4, 0.4167)), "{got:?}");
    for c in clashes(&mut job, None).unwrap().found {
        assert!((c.at[0] - 10.0).abs() < 0.01 && (c.at[1] - 5.0).abs() < 0.01 && (c.at[2] - 8.0).abs() < 0.01, "{:?}", c.at);
    }
    // The same of a rectangular duct, 12 in wide and 8 in deep along y = 15 with a 2 in
    // pipe over it at 8.35 ft: in it by 0.3333 + 0.0833 - 0.35 = 0.0667 ft, cut or not.
    let w = ed::runs(&mut job, &s, Kind::Duct, &wide, &[[0.0, 15.0], [20.0, 15.0]], "8 ft", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[10.0, 12.0], [10.0, 18.0]], "8.35 ft", None, how).unwrap();
    let over: Vec<_> = clashes_of(&mut job, None).into_iter().skip(2).collect();
    assert!(over.len() == 1 && about(over[0].4, 0.0667), "{over:?}");
    ed::split(&mut job, &w[0], [10.0, 15.0], how).unwrap();
    let got = clashes_of(&mut job, None);
    assert_eq!(got.len(), 4, "{got:?}");
    assert_eq!(got.iter().filter(|c| about(c.4, 0.4167)).count(), 2, "{got:?}");
    assert_eq!(got.iter().filter(|c| about(c.4, 0.0667)).count(), 2, "{got:?}");
    // Two runs joined at a corner are not one length: a duct that turns north at
    // (20, 5) is not carried on round the corner.
    ed::runs(&mut job, &s, Kind::Duct, &duct, &[[20.0, 5.0], [20.0, 9.0]], "8 ft", None, how).unwrap();
    assert_eq!(clashes_of(&mut job, None).len(), 4);
}

#[test]
fn a_run_through_a_wall_is_a_hole_to_cut_and_what_is_joined_is_not_a_clash() {
    use plumbline_world::building::clash::clashes;
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, _, nt) = drawn();
    let pipe = ed::segment_type(&mut job, "3/4 in", Kind::Pipe, "3/4 in", None, how).unwrap();
    // A wall east along y = 0, 10 ft long, 9 ft tall, with a window 4 ft by 3 ft in its
    // middle: from 3 ft to 7 ft along it and from 3 ft to 6 ft up.
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    ed::opening(&mut job, &wall[0], Kind::Window, &nt, ed::OnWall::Offset("5 ft"), "3 ft", how).unwrap();
    // A pipe north across the wall at x = 5, 4 ft up: through the window, so through
    // nothing. At 7 ft up: through the wall over the window's head.
    let low = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, -3.0], [5.0, 3.0]], "4 ft", None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());
    let high = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, -3.0], [5.0, 3.0]], "7 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert_eq!(got.len(), 1);
    // Hard, through a wall, and with no "how far to move": it is meant to go through.
    assert!(got[0].2 && got[0].3 && got[0].4.is_none(), "{got:?}");
    let found = clashes(&mut job, None).unwrap();
    assert_eq!((found.found[0].a.as_str(), found.found[0].b.as_str()), (wall[0].as_str(), high[0].as_str()));
    // A second wall crossing the first is built into it: walls are not tested against walls.
    ed::walls(&mut job, &s, &wt, &[[8.0, -4.0], [8.0, 4.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    assert_eq!(clashes_of(&mut job, None).len(), 1);
    ed::remove(&mut job, &high[0]).unwrap();
    ed::remove(&mut job, &low[0]).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());

    // A duct with a corner: its two pieces are in each other at the corner, and they are
    // joined there, so that is no clash. Nor is the elbow put on the joint.
    let (mut job, s, _, air, pieces) = ducted();
    assert!(clashes_of(&mut job, None).is_empty());
    ed::fitting(&mut job, &s, [10.0, 2.0], None, None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());
    // An air handler stood over the duct's first end has the duct in it: a clash, until
    // the duct is joined to it.
    let ahu = ed::place(&mut job, &s, Kind::Equipment, "Air handler", [2.0, 2.0], "6 ft", Some([2.0, 2.0, 4.0]), Some(&air), how).unwrap();
    let got = clashes_of(&mut job, None);
    // The handler is from 1 ft to 3 ft east and the duct, going east, starts on its middle at
    // 2 ft: 1 ft of the duct is in it, and it clears by moving that 1 ft.
    assert!(got.len() == 1 && got[0].2 && !got[0].3 && about(got[0].4, 1.0) && got[0].1 == "Air handler", "{got:?}");
    ed::connect(&mut job, &ahu, &pieces[0], how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty());
}

#[test]
fn j12_1_a_wall_drawn_over_another_is_refused_by_the_verbs_and_on_load() {
    use plumbline_world::editing as ed;
    use plumbline_world::Wall;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, ..) = drawn();
    // 20 ft by 10 ft, closed: 60 LF of wall.
    let made = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]], true, ed::Top::Height("9 ft"), how).unwrap();
    let before = save(&mut job);
    let total = |job: &mut Job| plumbline_world::building::storey_totals(job)[0].walls.value;
    assert_eq!(total(&mut job), Some(60.0));
    // The judge's three: a wall on the south wall, a wall on part of the north wall, and
    // two points closed back on themselves.
    let why = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap_err();
    assert!(why.contains(&format!("lies along {} for 20 ft", made[0])) && why.contains("counted twice"), "{why}");  // call 226: a length in a sentence is said as a builder says it
    let why = ed::walls(&mut job, &s, &wt, &[[5.0, 10.0], [15.0, 10.0]], false, ed::Top::Height("9 ft"), how).unwrap_err();
    assert!(why.contains("for 10 ft"), "{why}");  // call 226: a length in a sentence is said as a builder says it
    assert!(ed::walls(&mut job, &s, &wt, &[[30.0, 0.0], [40.0, 0.0]], true, ed::Top::Height("9 ft"), how).unwrap_err().contains("counted twice"));
    // A chain that goes back over itself, and a wall moved onto another.
    assert!(ed::walls(&mut job, &s, &wt, &[[30.0, 0.0], [40.0, 0.0], [35.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap_err().contains("for 5 ft"));  // call 226: a length in a sentence is said as a builder says it
    assert!(ed::move_alone(&mut job, &made[2], 0.0, -10.0, how).unwrap_err().contains("counted twice"));
    assert_eq!(save(&mut job), before);
    assert_eq!(total(&mut job), Some(60.0));
    // What is not one wall over another: a wall that carries on in line from a corner, a
    // wall that crosses, and a wall that stands on top of another (its own height above it).
    ed::walls(&mut job, &s, &wt, &[[20.0, 0.0], [30.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    ed::walls(&mut job, &s, &wt, &[[10.0, -5.0], [10.0, 5.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let upper = Wall { base_offset: Some(Val::known(Len::One(9.0), "ft", "made-up plan")), ..Wall::tall(Val::known(wt.clone(), "", "made-up plan"), Val::known([[0.0, 0.0], [20.0, 0.0]], "ft", "made-up plan"), Val::known(Len::One(3.0), "ft", "made-up plan")) };
    let over = job.add(&s, Kind::Wall, "gable above").unwrap();
    job.put(&over, upper.clone()).unwrap();
    plumbline_world::building::check_whole(&mut job).unwrap();
    // The same wall with no base of its own is on the south wall: a file that holds it is refused.
    job.put(&over, Wall { base_offset: None, ..upper }).unwrap();
    assert!(load(&save(&mut job)).err().unwrap().contains("counted twice"));
}

#[test]
fn j12_2_3_14_what_something_still_rests_on_is_not_removed() {
    use plumbline_world::editing as ed;
    // What a roof's footprint solves would come back: the house's roof planes and hips.
    let mut job = plumbline_world::sample::house();
    let solved: Vec<(String, Kind)> = {
        let mut q = job.world.query::<(&Node, &Kind, &Solved)>();
        q.iter(&job.world).map(|(n, k, _)| (n.path.clone(), *k)).collect()
    };
    assert!(solved.iter().any(|(_, k)| *k == Kind::RoofPlane) && solved.iter().any(|(_, k)| *k == Kind::Hip));
    let before = save(&mut job);
    for (path, _) in &solved {
        assert!(ed::remove(&mut job, path).unwrap_err().contains("is solved from"), "{path}");
    }
    assert_eq!(save(&mut job), before);
    // A plane entered by hand, with lines that lie on it.
    let mut job = sample();
    let (plane, lines): (String, Vec<String>) = {
        let mut q = job.world.query::<(&Node, &Run)>();
        let named: Vec<(String, Vec<String>)> = q.iter(&job.world).map(|(n, r)| (n.path.clone(), r.planes.clone())).collect();
        let plane = named.iter().flat_map(|(_, p)| p.iter().cloned()).next().unwrap();
        let mut on: Vec<String> = named.iter().filter(|(_, p)| p.contains(&plane)).map(|(n, _)| n.clone()).collect();
        on.sort();
        (plane, on)
    };
    let before = save(&mut job);
    let why = ed::remove(&mut job, &plane).unwrap_err();
    assert!(why.contains(&format!("{plane} is the plane")) && lines.iter().take(3).all(|l| why.contains(l.as_str())), "{why}");
    assert_eq!(save(&mut job), before);
    // A material something is made of.
    let mut job = plumbline_world::sample::priced();
    let material = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let all: Vec<(String, String, Kind)> = q.iter(&job.world).map(|(n, k)| (n.path.clone(), n.name.clone(), *k)).collect();
        let mut made = job.world.query::<&MadeOf>();
        let used: Vec<String> = made.iter(&job.world).filter_map(|m| m.0.value.clone()).collect();
        all.iter().find(|(p, n, k)| *k == Kind::Material && used.iter().any(|u| u == p || u == n)).unwrap().0.clone()
    };
    let before = save(&mut job);
    assert!(ed::remove(&mut job, &material).unwrap_err().contains("made of"));
    assert_eq!(save(&mut job), before);
    // A condition something has: the roof recipe, once it is applied to a plane. Before
    // that nothing has it, and it can go; so the removal is tried on a copy first.
    let mut job = sample();
    let condition = path_named(&mut job, "Standing seam copper roof");
    let (mut spare, _) = load(&save(&mut job)).ok().unwrap();
    ed::remove(&mut spare, &condition).unwrap();
    job.set(&plane, "condition", None, Status::Unknown, "to be given the recipe").unwrap();
    job.apply(&plane, "Standing seam copper roof", "test").unwrap();
    job.derive();
    let before = save(&mut job);
    assert_eq!(ed::remove(&mut job, &condition).unwrap_err(), format!("{condition} is the assembly of {plane}: give it another first"));
    assert_eq!(save(&mut job), before);
}

#[test]
fn j12_4_5_12_a_run_on_the_wrong_system_an_opening_off_its_wall_and_a_top_held_to_nothing_above() {
    use plumbline_world::editing as ed;
    use plumbline_world::InSystem;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, dt, _) = drawn();
    let duct = ed::segment_type(&mut job, "6 in round", Kind::Duct, "6 in", None, how).unwrap();
    let pipe = ed::segment_type(&mut job, "3/4 in", Kind::Pipe, "3/4 in", None, how).unwrap();
    let cold = ed::system(&mut job, "Cold water", "Domestic Cold Water").unwrap();
    let cooling = ed::system(&mut job, "Line set", "Refrigerant").unwrap();
    let power = ed::system(&mut job, "Power", "Power").unwrap();
    // A duct does not go on the cold water, nor a pipe on the power. A pipe goes on the
    // refrigerant, though that is a mechanical system: it is what the pipe carries that counts.
    let before = save(&mut job);
    let why = ed::runs(&mut job, &s, Kind::Duct, &duct, &[[0.0, 60.0], [10.0, 60.0]], "8 ft", Some(&cold), how).unwrap_err();
    assert!(why.contains("a duct goes on: Supply Air, Return Air, Exhaust Air") && why.contains("Domestic Cold Water system"), "{why}");
    assert!(ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 60.0], [10.0, 60.0]], "8 ft", Some(&power), how).unwrap_err().contains("a pipe goes on"));
    assert_eq!(save(&mut job), before);
    let made = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 60.0], [10.0, 60.0]], "8 ft", Some(&cooling), how).unwrap();
    // A fixture can be on any system; and a file with a pipe on the power is refused.
    ed::place(&mut job, &s, Kind::Fixture, "Receptacle", [3.0, 3.0], "1 ft", Some([0.3, 0.1, 0.4]), Some(&power), how).unwrap();
    job.put(&made[0], InSystem(Val::known(power.clone(), "", "made-up plan"))).unwrap();
    assert!(load(&save(&mut job)).err().unwrap().contains("a pipe goes on"));
    job.put(&made[0], InSystem(Val::known(cooling, "", "made-up plan"))).unwrap();

    // A 10 ft wall east along y = 0, 6 in thick. A door by a point 500 ft off it is not on
    // it; nor one 4 in off (half the wall is 3 in). One 2 in off is.
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let before = save(&mut job);
    let why = ed::opening(&mut job, &wall[0], Kind::Door, &dt, ed::OnWall::Point([5.0, 500.0]), "0 ft", how).unwrap_err();
    assert_eq!(why, format!("the point (5, 500) is 500 ft off the middle of {}, which is 6 in thick: it is not on the wall", wall[0]));  // call 226: a length in a sentence is said as a builder says it
    assert!(ed::opening(&mut job, &wall[0], Kind::Door, &dt, ed::OnWall::Point([5.0, 4.0 / 12.0]), "0 ft", how).is_err());
    assert_eq!(save(&mut job), before);
    let door = ed::opening(&mut job, &wall[0], Kind::Door, &dt, ed::OnWall::Point([5.0, -2.0 / 12.0]), "0 ft", how).unwrap();
    assert_eq!(job.find(&door).and_then(|e| job.world.get::<plumbline_world::Opening>(e).map(|o| o.offset.feet())).unwrap(), Some(5.0));

    // A wall's top held to its own storey, or to the one below, can never have a height.
    let upper = ed::storey(&mut job, "upper", "9 ft", "9 ft", how).unwrap();
    let before = save(&mut job);
    assert!(ed::walls(&mut job, &s, &wt, &[[0.0, 30.0], [10.0, 30.0]], false, ed::Top::Level(&s), how).unwrap_err().contains("cannot have its top held to it"));
    assert!(ed::walls(&mut job, &upper, &wt, &[[0.0, 30.0], [10.0, 30.0]], false, ed::Top::Level(&s), how).unwrap_err().contains("is not above"));
    assert_eq!(save(&mut job), before);
    // Held to the storey above, it is that storey's floor less its own: 9 ft.
    let held = ed::walls(&mut job, &s, &wt, &[[0.0, 30.0], [10.0, 30.0]], false, ed::Top::Level(&upper), how).unwrap();
    assert_eq!(measure_of(&mut job, &held[0], "height").value, Some(9.0));
}

#[test]
fn j12_6_an_opening_that_does_not_fit_leaves_no_opening_area_and_each_one_is_named() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, dt, nt) = drawn();
    // A wall 10 ft long and 9 ft tall. A door (3 by 7) 2 ft along and a window (4 by 3)
    // 7 ft along with its sill at 3 ft: both fit. 90 − 21 − 12 = 57 SF.
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let door = ed::opening(&mut job, &wall[0], Kind::Door, &dt, ed::OnWall::Offset("2 ft"), "0 ft", how).unwrap();
    let window = ed::opening(&mut job, &wall[0], Kind::Window, &nt, ed::OnWall::Offset("7 ft"), "3 ft", how).unwrap();
    assert_eq!((measure_of(&mut job, &wall[0], "opening_area").value, measure_of(&mut job, &wall[0], "net_area").value), (Some(33.0), Some(57.0)));
    // The window moved to 9.5 ft along runs 1.5 ft past the wall's end, and a second door
    // put over the first: two openings that do not fit. Neither the openings' area nor the
    // net is given, and both are named in each.
    let e = job.find(&window).unwrap();
    let mut o = job.world.get::<plumbline_world::Opening>(e).cloned().unwrap();
    o.offset = Val::known(Len::One(9.5), "ft", "made-up plan");
    job.put(&window, o).unwrap();
    // (Call 179, B5: a door drawn over another is refused now. The second is put in by hand, as a job saved
    // before holds it, and the wall's figures are still not given and still name both.)
    let refused = ed::opening(&mut job, &wall[0], Kind::Door, &dt, ed::OnWall::Offset("2.5 ft"), "0 ft", how).unwrap_err();
    assert!(refused.contains("lie over each other"), "{refused}");
    let second = job.add(&wall[0], Kind::Door, "door 2").unwrap();
    let held = plumbline_world::Opening { type_name: Val::known(dt.clone(), "", "made-up plan"), offset: Val::known(Len::One(2.5), "ft", "made-up plan"), sill: Val::known(Len::One(0.0), "ft", "made-up plan") };
    job.put(&second, held).unwrap();
    job.derive();
    for figure in ["opening_area", "net_area"] {
        let d = measure_of(&mut job, &wall[0], figure);
        assert_eq!((d.value, d.status), (None, Status::Unknown), "{figure}");
        assert!(d.note.contains(&format!("{window} ends 11 ft 6 in along a wall 10 ft long")) && d.note.contains(&format!("{door} and {second} lie over each other")), "{figure}: {}", d.note);  // call 226: a length in a sentence is said as a builder says it
    }
}

#[test]
fn j12_10_11_13_17_18_sources_names_and_what_goes_where() {
    use plumbline_world::check::same_name;
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "plan A" };
    let (mut job, s, wt, ..) = drawn();
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let source = |job: &mut Job| job.find(&wall[0]).and_then(|e| job.world.get::<plumbline_world::Wall>(e).map(|w| (w.line.status, w.line.source.clone()))).unwrap();
    // Moved four times on a guess: it is Assumed, it still says it came from plan A, and
    // its source has not grown with each move.
    let guess = ed::Entered { status: Status::Assumed, source: "guess" };
    for _ in 0..4 {
        ed::move_by(&mut job, &wall[0], 1.0, 0.0, guess).unwrap();
    }
    assert_eq!(source(&mut job), (Status::Assumed, "plan A; moved (guess)".to_string()));
    ed::move_by(&mut job, &wall[0], 1.0, 0.0, ed::Entered { status: Status::Known, source: "survey" }).unwrap();
    // (Call 46: a source loses nothing; each change of hand adds its note.)
    assert_eq!(source(&mut job), (Status::Assumed, "plan A; moved (guess); moved (survey)".to_string()));
    // A move by nothing is not a move.
    let before = save(&mut job);
    assert_eq!(ed::move_by(&mut job, &wall[0], 0.0, 0.0, guess).unwrap_err(), format!("{} is moved by nothing", wall[0]));
    assert_eq!(save(&mut job), before);

    // One name, however it is typed.
    assert!(same_name("Copper, 16 oz", "copper,16 OZ") && same_name("Copper, 16 oz", "Copper, 16 oz.") && !same_name("Copper, 16 oz", "Copper, 20 oz"));
    let mut job = plumbline_world::sample::priced();
    assert!(job.add_material("Copper,16 oz", "lb").is_err() && job.add_material("Copper, 16 oz.", "lb").is_err());
    let made: String = {
        let mut q = job.world.query::<(&Node, &MadeOf)>();
        q.iter(&job.world).map(|(n, _)| n.path.clone()).next().unwrap()
    };
    job.set(&made, "material", Some("copper, 16 OZ"), Status::Known, "test").unwrap();
    let named = job.find(&made).and_then(|e| job.world.get::<MadeOf>(e).and_then(|m| m.0.value.clone())).unwrap();
    assert_eq!(named, "Copper, 16 oz");

    // Cost lines go on what is built and on the job: not on a system or a storey.
    let mut job = plumbline_world::sample::house();
    let (system, storey) = {
        let mut q = job.world.query::<(&Node, &Kind)>();
        let all: Vec<(String, Kind)> = q.iter(&job.world).map(|(n, k)| (n.path.clone(), *k)).collect();
        (all.iter().find(|(_, k)| *k == Kind::System).unwrap().0.clone(), all.iter().find(|(_, k)| *k == Kind::Storey).unwrap().0.clone())
    };
    assert!(job.add_line(&system, "X", "5", Some(Class::General), false).unwrap_err().contains("cannot carry cost lines"));
    assert!(job.add_line(&storey, "X", "5", Some(Class::General), false).unwrap_err().contains("cannot carry cost lines"));
    // A material is not ordered by the hour.
    let mut job = plumbline_world::sample::priced();
    let material: String = {
        let mut q = job.world.query::<(&Node, &Supply)>();
        q.iter(&job.world).map(|(n, _)| n.path.clone()).next().unwrap()
    };
    assert!(job.set(&material, "order", Some("2 lb/hr"), Status::Known, "test").unwrap_err().contains("not ordered by the hour"));
}

#[test]
fn a_drawing_gets_its_scale_from_one_dimension_and_what_is_traced_over_it_says_so() {
    use plumbline_world::editing as ed;
    use plumbline_world::plans::{self, Drawing, Read};
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, ..) = drawn();
    let page = || Read { file: "plans/made-up-plan.pdf", page: 1, bytes: 758, checksum: "0123456789abcdef", size: [792.0, 612.0] };
    let d = plans::add(&mut job, &s, "A-1", page(), how).unwrap();
    let drawing = |job: &mut Job| job.find(&d).and_then(|e| job.world.get::<Drawing>(e).cloned()).unwrap();
    // No scale yet: it is not known where anything on it is, and nothing is traced over it.
    assert_eq!((drawing(&mut job).status(), drawing(&mut job).to_plan([126.0, 126.0])), (Status::Unknown, None));
    assert_eq!(plans::traced_from(&mut job, &d).unwrap_err(), format!("{d} has no scale: calibrate it from one dimension before tracing over it"));
    // Its 30 ft dimension line runs from (126, 90) to (666, 90): 540 points. 30 / 540 is
    // 1/18 ft to the point. The dimension is read off the drawing, so it is Assumed.
    let read = ed::Entered { status: Status::Assumed, source: "read off the drawing" };
    let k = plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30 ft", read).unwrap().feet_per_unit;
    assert!((k - 1.0 / 18.0).abs() < 1.0e-12, "{k}");
    let near = |a: Option<[f64; 2]>, b: [f64; 2]| a.is_some_and(|a| (a[0] - b[0]).abs() < 1.0e-9 && (a[1] - b[1]).abs() < 1.0e-9);
    // The page's lower-left corner is the plan's corner until it is lined up: the house's
    // corner, 126 points in and up, is 7 ft in and up. And back again.
    assert!(near(drawing(&mut job).to_plan([126.0, 126.0]), [7.0, 7.0]));
    assert!(near(drawing(&mut job).to_page([7.0, 7.0]), [126.0, 126.0]));
    // Lined up so the house's corner is the plan's (0, 0): its far corner is at (30, 20).
    plans::line_up(&mut job, &d, [126.0, 126.0], [0.0, 0.0], read).unwrap();
    assert!(near(drawing(&mut job).to_plan([666.0, 486.0]), [30.0, 20.0]));
    assert_eq!(drawing(&mut job).status(), Status::Assumed);
    assert_eq!(plans::traced_from(&mut job, &d).unwrap(), "traced from A-1 (plans/made-up-plan.pdf, page 1)");
    // What cannot be a scale is refused and leaves the job as it was.
    let before = save(&mut job);
    assert!(plans::calibrate(&mut job, &d, [126.0, 90.0], [126.0, 90.0], "30 ft", read).unwrap_err().contains("its two points are one point"));
    assert!(plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "0 ft", read).unwrap_err().contains("more than nothing"));
    assert!(plans::calibrate(&mut job, &d, [126.0, 90.0], [666.0, 90.0], "30", read).unwrap_err().contains("no unit"));
    assert!(plans::calibrate(&mut job, &d, [126.0, f64::NAN], [666.0, 90.0], "30 ft", read).is_err());
    assert!(plans::add(&mut job, &wt, "on a type", page(), how).unwrap_err().contains("not a level"));
    assert!(plans::line_up(&mut job, &s, [0.0, 0.0], [0.0, 0.0], read).unwrap_err().contains("is not a drawing"));
    assert_eq!(save(&mut job), before);
    // It saves and loads to the same bytes, with its scale.
    let text = save(&mut job);
    let (mut again, report) = load(&text).ok().unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut again), text);
    assert!(text.contains("\"pl::drawing\""));
    // A drawing that is not under a storey, or has a page 0, is refused on load.
    assert!(load(&text.replace("\"page\": 1", "\"page\": 0")).err().unwrap().contains("pages are counted from 1"));
    let under_job = job.add("job-0001", Kind::Drawing, "loose").unwrap();
    let moved = drawing(&mut job);
    job.put(&under_job, moved).unwrap();
    assert!(load(&save(&mut job)).err().unwrap().contains("a drawing is a plan of a level"));
}

// ---------------------------------------------------------------- the thirteenth judgment

#[test]
fn j13_1_what_is_joined_is_excused_at_the_joint_and_nowhere_else() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, ..) = drawn();
    let pipe = ed::segment_type(&mut job, "1 in", Kind::Pipe, "1 in", None, how).unwrap();
    // Two 1 in pipes drawn on one line, (0, 0) to (10, 0), 8 ft up. They meet at both
    // ends and lie in each other all the way: by both radii, 1 in, 0.0833 ft.
    let first = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 0.0], [10.0, 0.0]], "8 ft", None, how).unwrap();
    let second = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 0.0], [10.0, 0.0]], "8 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && !got[0].3 && about(got[0].4, 0.0833), "{got:?}");
    ed::remove(&mut job, &second[0]).unwrap();
    // A pipe drawn back along the first from its far end, 8 ft of it: the same.
    let back = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[10.0, 0.0], [2.0, 0.0]], "8 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 0.0833), "{got:?}");
    ed::remove(&mut job, &back[0]).unwrap();
    // A pipe that turns the corner at the first one's end, and a branch off its side:
    // joints, and nothing more.
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[10.0, 0.0], [10.0, 6.0]], "8 ft", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, 0.0], [5.0, 4.0]], "8 ft", None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty(), "{:?}", clashes_of(&mut job, None));
    let _ = first;

    // A tank 4 ft each way from 6 ft up, its west face at x = 18. A pipe starts a tenth of
    // a foot inside that face and runs east through the tank and out: 3.9 ft of it in the
    // tank, which clears by moving half the tank's width and the pipe's radius, 2.0417 ft.
    // Joined to the tank at its end, it is still through the tank.
    let tank = ed::place(&mut job, &s, Kind::Equipment, "Tank", [20.0, 20.0], "6 ft", Some([4.0, 4.0, 4.0]), None, how).unwrap();
    let through = ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[18.1, 20.0], [30.0, 20.0]], "8 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 2.0417), "{got:?}");
    ed::connect(&mut job, &tank, &through[0], how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 2.0417), "{got:?}");

    // Three ends at one place, two of them leaving it the same way: no tee is made.
    let (mut job, s, ..) = drawn();
    let pipe = ed::segment_type(&mut job, "1 in", Kind::Pipe, "1 in", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[0.0, 0.0], [5.0, 0.0]], "8 ft", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, 0.0], [5.0, 5.0]], "8 ft", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, 0.0], [5.0, 3.0]], "8 ft", None, how).unwrap();
    let before = save(&mut job);
    let why = ed::fitting(&mut job, &s, [5.0, 0.0], None, None, how).unwrap_err();
    assert!(why.contains("the same way, one over the other"), "{why}");
    assert_eq!(save(&mut job), before);
    // And the two that lie over each other are a clash: 3 ft of one in the other.
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 0.0833), "{got:?}");
}

#[test]
fn j13_2_4_a_hole_is_a_hole_whatever_stands_over_it_and_a_door_that_is_not_cut_flags_its_wall() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, dt, nt) = drawn();
    let pipe = ed::segment_type(&mut job, "3/4 in", Kind::Pipe, "3/4 in", None, how).unwrap();
    // A wall 10 ft long and 9 ft tall with two windows 4 ft by 3 ft, one over the other:
    // both from 3 ft to 7 ft along it; one from 6 in to 3 ft 6 in up, one from 5 ft to 8 ft.
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    ed::opening(&mut job, &wall[0], Kind::Window, &nt, ed::OnWall::Offset("5 ft"), "6 in", how).unwrap();
    ed::opening(&mut job, &wall[0], Kind::Window, &nt, ed::OnWall::Offset("5 ft"), "5 ft", how).unwrap();
    // The pieces of its face: 3 ft each side whole, and between them 0 to 0.5, 3.5 to 5
    // and 8 to 9.
    let pieces = plumbline_world::geom::face_less_holes(10.0, 9.0, &[[3.0, 7.0, 0.5, 3.5], [3.0, 7.0, 5.0, 8.0]]);
    assert_eq!(pieces, [[0.0, 3.0, 0.0, 9.0], [3.0, 7.0, 0.0, 0.5], [3.0, 7.0, 3.5, 5.0], [3.0, 7.0, 8.0, 9.0], [7.0, 10.0, 0.0, 9.0]]);
    // A pipe through the middle of each window is through nothing; one between them, at
    // 4 ft 3 in, is through the wall.
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, -3.0], [5.0, 3.0]], "2 ft", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, -3.0], [5.0, 3.0]], "6 ft", None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty(), "{:?}", clashes_of(&mut job, None));
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[6.0, -3.0], [6.0, 3.0]], "4.25 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && got[0].3 && got[0].5 == Status::Known, "{got:?}");
    // A door over a window's corner (the door from 2.5 to 5.5 along and 7 ft up, a window
    // from 4 to 8 along and 4 ft to 7 ft up): what the door cuts the window does not fill.
    assert_eq!(
        plumbline_world::geom::face_less_holes(10.0, 9.0, &[[2.5, 5.5, 0.0, 7.0], [4.0, 8.0, 4.0, 7.0]]),
        [[0.0, 2.5, 0.0, 9.0], [2.5, 5.5, 7.0, 9.0], [5.5, 8.0, 0.0, 4.0], [5.5, 8.0, 7.0, 9.0], [8.0, 10.0, 0.0, 9.0]]
    );

    // A wall 6 ft 11 in tall with a 7 ft door: the door cannot be cut in it, the wall is
    // drawn solid there, and a pipe through where the door would be is a question for the
    // field, not a finding.
    let (mut job, s, wt, dt2, _) = drawn();
    let _ = dt;
    let pipe = ed::segment_type(&mut job, "3/4 in", Kind::Pipe, "3/4 in", None, how).unwrap();
    // (Call 180, C5: a door taller than its wall is refused when it is drawn, so the wall is cut down to 6 ft 11 in
    // after the door is in it, as a job saved before has it.)
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    ed::opening(&mut job, &wall[0], Kind::Door, &dt2, ed::OnWall::Offset("5 ft"), "0 ft", how).unwrap();
    let low = ed::walls(&mut job, &s, &wt, &[[0.0, 40.0], [10.0, 40.0]], false, ed::Top::Height("6 ft 11 in"), how).unwrap();
    {
        let (here, there) = (job.find(&wall[0]).unwrap(), job.find(&low[0]).unwrap());
        let height = job.world.get::<Wall>(there).unwrap().height.clone();
        let mut w = job.world.get::<Wall>(here).cloned().unwrap();
        w.height = height;
        job.put(&wall[0], w).unwrap();
    }
    ed::remove(&mut job, &low[0]).unwrap();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, -3.0], [5.0, 3.0]], "3 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].3 && got[0].5 == Status::Assumed, "{got:?}");
}

#[test]
fn j13_3_a_clash_is_no_surer_than_the_floor_heights_and_the_wall_base_it_rests_on() {
    use plumbline_world::building::Wall;
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let seen = ed::Entered { status: Status::Assumed, source: "scaled off the section" };
    let (mut job, s, wt, ..) = drawn();
    // The upper floor is 10 ft up, as far as anyone has scaled it. A tall thing on the
    // ground floor reaches from 8 ft to 11 ft; a thing on the upper floor stands on it,
    // from 10 ft: they are in each other by 1 ft only if the 10 ft is right.
    let upper = ed::storey(&mut job, "upper", "10 ft", "9 ft", seen).unwrap();
    ed::place(&mut job, &s, Kind::Equipment, "Tall", [0.0, 0.0], "8 ft", Some([2.0, 2.0, 3.0]), None, how).unwrap();
    ed::place(&mut job, &upper, Kind::Equipment, "Low", [0.0, 0.0], "0 ft", Some([2.0, 2.0, 2.0]), None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && about(got[0].4, 1.0) && got[0].5 == Status::Assumed, "{got:?}");

    // A wall 4 ft tall whose base is 2 ft up, as far as anyone knows: it stands from 2 ft
    // to 6 ft. A pipe through it at 5 ft is through it only if the 2 ft is right.
    let (mut job, s, wt2, ..) = drawn();
    let _ = wt;
    let pipe = ed::segment_type(&mut job, "3/4 in", Kind::Pipe, "3/4 in", None, how).unwrap();
    let wall = ed::walls(&mut job, &s, &wt2, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("4 ft"), how).unwrap();
    let e = job.find(&wall[0]).unwrap();
    let mut w = job.world.get::<Wall>(e).cloned().unwrap();
    w.base_offset = Some(Val::assumed(Len::One(2.0), "ft", "scaled off the section"));
    job.put(&wall[0], w).unwrap();
    job.derive();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[5.0, -3.0], [5.0, 3.0]], "5 ft", None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].3 && got[0].5 == Status::Assumed, "{got:?}");
    // A fitting is no surer than the floor height its runs are measured from.
    let (mut job, ..) = drawn();
    let high = ed::storey(&mut job, "upper", "10 ft", "9 ft", seen).unwrap();
    let duct = ed::segment_type(&mut job, "6 in round", Kind::Duct, "6 in", None, how).unwrap();
    ed::runs(&mut job, &high, Kind::Duct, &duct, &[[0.0, 0.0], [5.0, 0.0], [5.0, 5.0]], "8 ft", None, how).unwrap();
    let made = ed::fitting(&mut job, &high, [5.0, 0.0], None, None, how).unwrap();
    let e = job.find(&made.path).unwrap();
    assert_eq!(job.world.get::<Placed>(e).unwrap().at.status, Status::Assumed);
}

#[test]
fn j13_5_how_deep_a_thing_is_in_a_wall_or_a_floor_is_out_of_all_of_it() {
    use plumbline_world::building::{Floor, FloorType};
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, _, nt) = drawn();
    // A floor 20 ft square and 12 in thick, its top at the storey's floor. It is cut
    // corner to corner into two solids. A box 0.4 ft square and 2 ft tall is sunk 0.9 ft
    // into it: once well inside one of the two, once on the cut between them. Either way
    // it comes out by rising 0.9 ft.
    let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).unwrap();
    let ft = job.add(&root, Kind::Type, "slab 12 in").unwrap();
    job.put(&ft, FloorType { thickness: Val::known(Len::One(12.0), "in", "made-up plan"), layers: Vec::new() }).unwrap();
    let floor = job.add(&s, Kind::Floor, "slab").unwrap();
    job.put(&floor, Floor { type_name: Val::known(ft.clone(), "", "made-up plan"), boundary: Val::known(vec![[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]], "ft", "made-up plan"), offset: None }).unwrap();
    job.derive();
    let inside = ed::place(&mut job, &s, Kind::Equipment, "Pit pump", [4.0, 10.0], "-0.9 ft", Some([0.4, 0.4, 2.0]), None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && !got[0].3 && about(got[0].4, 0.9), "{got:?}");
    ed::remove(&mut job, &inside).unwrap();
    ed::place(&mut job, &s, Kind::Equipment, "Pit pump", [10.0, 10.0], "-0.9 ft", Some([0.4, 0.4, 2.0]), None, how).unwrap();
    let got = clashes_of(&mut job, None);
    assert!(got.len() == 1 && got[0].2 && about(got[0].4, 0.9), "{got:?}");

    // A 6 in wall along y = 30 with a window from 3 ft to 7 ft along it and 3 ft to 6 ft
    // up. A box 0.4 ft along the wall, 0.2 ft through it and 1 ft tall stands in the
    // wall's middle at 3 ft along, from 1 ft to 2 ft up, where the wall is in two pieces.
    // It comes out through the wall's face: half the wall and half the box, 0.35 ft.
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 30.0], [10.0, 30.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    ed::opening(&mut job, &wall[0], Kind::Window, &nt, ed::OnWall::Offset("5 ft"), "3 ft", how).unwrap();
    ed::place(&mut job, &s, Kind::Fixture, "Box in the wall", [3.0, 30.0], "1 ft", Some([0.4, 0.2, 1.0]), None, how).unwrap();
    let got = clashes_of(&mut job, None);
    let in_wall = got.iter().find(|c| c.1.starts_with("Box in the wall")).unwrap();
    assert!(in_wall.2 && !in_wall.3 && about(in_wall.4, 0.35), "{got:?}");
}

#[test]
fn j13_6_7_a_fitting_lies_along_its_run_and_a_sum_of_typed_numbers_is_a_typed_number() {
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, ..) = drawn();
    // A duct 24 in by 6 in running north-east, with a coupling half way: the coupling is
    // drawn at the duct's size, 2 ft square on the plan, and lies along the duct. A 1 in
    // pipe beside the duct, 1.2 ft from its centre line, is 0.1583 ft clear of the duct
    // and of the coupling.
    let duct = ed::segment_type(&mut job, "24 by 6", Kind::Duct, "24 in", Some("6 in"), how).unwrap();
    let pipe = ed::segment_type(&mut job, "1 in", Kind::Pipe, "1 in", None, how).unwrap();
    ed::runs(&mut job, &s, Kind::Duct, &duct, &[[0.0, 0.0], [10.0, 10.0], [20.0, 20.0]], "8 ft", None, how).unwrap();
    let made = ed::fitting(&mut job, &s, [10.0, 10.0], None, None, how).unwrap();
    let e = job.find(&made.path).unwrap();
    let turned = job.world.get::<Placed>(e).unwrap().angle.as_ref().and_then(|a| a.value).unwrap();
    assert!((turned.abs() - 135.0).abs() < 1.0e-6 || (turned.abs() - 45.0).abs() < 1.0e-6, "{turned}");
    let off = 1.2 / 2.0_f64.sqrt();
    ed::runs(&mut job, &s, Kind::Pipe, &pipe, &[[-off, off], [20.0 - off, 20.0 + off]], "8 ft", None, how).unwrap();
    assert!(clashes_of(&mut job, None).is_empty(), "{:?}", clashes_of(&mut job, None));
    // Asked for a clearance under that, nothing; over it, the pipe is that close to both
    // pieces of the duct and to the coupling between them.
    assert!(clashes_of(&mut job, Some(0.15)).is_empty());
    let near = clashes_of(&mut job, Some(0.17));
    assert!(near.len() == 3 && near.iter().all(|c| !c.2 && about(c.4, 0.1583)), "{near:?}");

    // Four hangers written as a sum are four hangers typed: cut in two, the run would
    // carry four on each piece. Refused however it is written.
    for typed in ["2 * 2", "-(-4)", "4"] {
        let (mut job, _, _, _, pieces) = ducted();
        job.add_line(&pieces[0], "hangers", typed, None, false).unwrap();
        job.derive();
        let before = save(&mut job);
        let why = ed::split(&mut job, &pieces[0], [6.0, 2.0], how).unwrap_err();
        assert!(why.contains("\"hangers\"") && why.contains("typed number") && why.contains("counted on both pieces"), "{why}");
        assert_eq!(save(&mut job), before);
    }
}

#[test]
fn j13_8_9_one_wall_over_another_is_judged_by_their_bodies_and_an_opening_by_its_walls_middle() {
    use plumbline_world::building::{Location, Wall};
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    let (mut job, s, wt, dt, _) = drawn();
    // A 6 in wall along y = 0, 10 ft long.
    let first = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let before = save(&mut job);
    // The same wall a quarter inch to one side, and one crossing it at 1 in 100: each is
    // in the first for all of its 10 ft. Refused.
    for line in [[[0.0, 0.02], [10.0, 0.02]], [[0.0, -0.05], [10.0, 0.05]]] {
        let why = ed::walls(&mut job, &s, &wt, &line, false, ed::Top::Height("9 ft"), how).unwrap_err();
        assert!(why.contains(&format!("lies along {}", first[0])) && why.contains("10 ft"), "{why}");  // call 226: a length in a sentence is said as a builder says it
    }
    assert_eq!(save(&mut job), before);
    // A wall square across it is in it for their thickness, as walls that cross are; a
    // wall beside it a foot away is clear of it. Both are walls.
    ed::walls(&mut job, &s, &wt, &[[5.0, -4.0], [5.0, 4.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    ed::walls(&mut job, &s, &wt, &[[0.0, 1.0], [4.0, 1.0]], false, ed::Top::Height("9 ft"), how).unwrap();

    // Two walls back to back on one line. The first is drawn to its outside face along
    // y = 20, so its body is from y = 19.5 to 20. The second is the same line drawn the
    // other way, to its own outside face: its body is from 20 to 20.5. They share a face
    // and nothing else.
    let (mut job, s, wt, dt2, _) = drawn();
    let _ = dt;
    let one = ed::walls(&mut job, &s, &wt, &[[0.0, 20.0], [10.0, 20.0]], false, ed::Top::Height("9 ft"), how).unwrap();
    let e = job.find(&one[0]).unwrap();
    let mut w = job.world.get::<Wall>(e).cloned().unwrap();
    w.location = Location::ExteriorFinish;
    job.put(&one[0], w.clone()).unwrap();
    let other = job.add(&s, Kind::Wall, "the wall behind it").unwrap();
    let mut back = w.clone();
    back.line = Val::known([[10.0, 20.0], [0.0, 20.0]], "ft", "made-up plan");
    job.put(&other, back).unwrap();
    job.derive();
    let text = save(&mut job);
    assert!(load(&text).is_ok(), "{:?}", load(&text).err());
    // A door by a point: in the first wall's body, 0.45 ft behind the line it was drawn
    // on, it is on the wall; 0.26 ft in front of that line it is out in the room.
    let before = save(&mut job);
    let why = ed::opening(&mut job, &one[0], Kind::Door, &dt2, ed::OnWall::Point([8.0, 20.26]), "0 ft", how).unwrap_err();
    assert!(why.contains("6 1/8 in off the middle") && why.contains("it is not on the wall"), "{why}");  // call 226: a length in a sentence is said as a builder says it
    assert_eq!(save(&mut job), before);
    ed::opening(&mut job, &one[0], Kind::Door, &dt2, ed::OnWall::Point([5.0, 19.55]), "0 ft", how).unwrap();

    // A wall on a knee wall. The upper one's base is 4 ft up and its top is the next
    // floor, 10 ft up; the knee wall under it on the same line is 4 ft tall.
    let (mut job, s, wt, ..) = drawn();
    let next = ed::storey(&mut job, "upper", "10 ft", "9 ft", how).unwrap();
    let over = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Level(&next), how).unwrap();
    let e = job.find(&over[0]).unwrap();
    let mut w = job.world.get::<Wall>(e).cloned().unwrap();
    w.base_offset = Some(Val::known(Len::One(4.0), "ft", "made-up plan"));
    job.put(&over[0], w).unwrap();
    job.derive();
    ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("4 ft"), how).unwrap();
    let text = save(&mut job);
    assert!(load(&text).is_ok(), "{:?}", load(&text).err());
    // One 5 ft tall there would stand a foot into the wall over it. Refused.
    assert!(ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("5 ft"), how).unwrap_err().contains("lies along"));
}

#[test]
fn j13_10_11_12_13_18_hours_names_sources_systems_and_a_cut_runs_name() {
    use plumbline_world::check::same_name;
    use plumbline_world::editing as ed;
    let how = ed::Entered { status: Status::Known, source: "made-up plan" };
    // A material is not ordered by the hour, however the hour is spelled.
    let mut job = plumbline_world::sample::priced();
    let material: String = {
        let mut q = job.world.query::<(&Node, &Supply)>();
        q.iter(&job.world).map(|(n, _)| n.path.clone()).next().unwrap()
    };
    for by_the_hour in ["2 lb/hr", "2 lb/hour", "2 lb/h", "2 lb/HR", "0.5 hours/lb", "2 lb/MH", "2 lb/man-hour", "2 lb/manhour", "2 lb/hr.", "2 lb/day", "2 lb/min", "2 lb/shift"] {
        let why = job.set(&material, "order", Some(by_the_hour), Status::Known, "test").unwrap_err();
        assert!(why.contains("by the hour"), "{by_the_hour}: {why}");
    }
    // One name: capitals, spaces run together or beside a comma, a full stop that is
    // not a decimal point. Not one name: figures a space apart, a different figure.
    for (a, b) in [("Copper,16 oz", "Copper, 16 oz."), ("Grade No. 1", "grade no 1"), ("No 1.", "No 1"), ("Supply  air.", "supply air"), ("g", "G"), ("Copper, 16 oz", "Copper 16 oz"), ("Pipe 1-1/2 in", "Pipe 1 1/2 in"), ("Cu 16oz", "Cu 16 oz"), ("No. 1", "No.1"), ("Type X", "Type-X"), ("2x4", "2 x 4")] {
        assert!(same_name(a, b), "{a:?} and {b:?}");
    }
    for (a, b) in [("Pipe 1 1/2 in", "Pipe 11/2 in"), ("Copper 1 6 oz", "Copper 16 oz"), ("Pipe 1.5 in", "Pipe 15 in"), ("Pipe 1.5 in", "Pipe 1 5 in"), ("a.b", "ab"), ("Pipe 1,2", "Pipe 1, 2"), ("Copper, 16 oz", "Copper, 20 oz")] {
        assert!(!same_name(a, b), "{a:?} and {b:?}");
    }
    // What a person wrote as a source stays, whatever words are in it.
    let (mut job, s, wt, ..) = drawn();
    let own = ed::Entered { status: Status::Known, source: "A-101; cut (section 3) shows this wall at grid C, rev B" };
    let wall = ed::walls(&mut job, &s, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), own).unwrap();
    let source = |job: &mut Job| job.find(&wall[0]).and_then(|e| job.world.get::<plumbline_world::Wall>(e).map(|w| w.line.source.clone())).unwrap();
    let by_eye = ed::Entered { status: Status::Assumed, source: "by eye" };
    ed::move_by(&mut job, &wall[0], 1.0, 0.0, by_eye).unwrap();
    assert_eq!(source(&mut job), "A-101; cut (section 3) shows this wall at grid C, rev B; moved (by eye)");
    ed::move_by(&mut job, &wall[0], 1.0, 0.0, ed::Entered { status: Status::Assumed, source: "by tape" }).unwrap();
    assert_eq!(source(&mut job), "A-101; cut (section 3) shows this wall at grid C, rev B; moved (by eye); moved (by tape)");

    // A duct is not joined to a lavatory on the cold water.
    let (mut job, s, _, _, pieces) = ducted();
    let cold = ed::system(&mut job, "Cold water", "domestic cold water").unwrap();
    let lavatory = ed::place(&mut job, &s, Kind::Fixture, "Lavatory", [2.0, 2.0], "7 ft", Some([2.0, 1.5, 3.0]), Some(&cold), how).unwrap();
    let before = save(&mut job);
    let why = ed::connect(&mut job, &lavatory, &pieces[0], how).unwrap_err();
    assert!(why.contains("Domestic Cold Water system: a duct is not joined to it"), "{why}");
    assert_eq!(save(&mut job), before);

    // A run somebody named keeps its name when it is cut; one with only its number gets
    // a number of its own.
    let name = |job: &mut Job, path: &str| job.find(path).and_then(|e| job.world.get::<Node>(e).map(|n| n.name.clone())).unwrap();
    let numbered = ed::split(&mut job, &pieces[0], [6.0, 2.0], how).unwrap();
    assert_eq!(name(&mut job, &numbered), "duct 3", "the two pieces drawn are duct 1 and duct 2");
    if let Some(mut node) = job.find(&pieces[1]).and_then(|e| job.world.get_mut::<Node>(e)) {
        node.name = "trunk to the bedrooms".into();
    }
    let named = ed::split(&mut job, &pieces[1], [10.0, 8.0], how).unwrap();
    assert_eq!(name(&mut job, &named), "trunk to the bedrooms, piece 2");
}
