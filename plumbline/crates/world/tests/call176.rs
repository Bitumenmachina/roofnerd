//! Call 176: what a wall carries and what it stands on, worked from what is drawn above and
//! below it. Every house here is made up; every figure is worked by hand beside its assert.

use plumbline_world::carries::{self, JoistWay};
use plumbline_world::editing as ed;
use plumbline_world::file::{load, save};
use plumbline_world::solve;
use plumbline_world::value::Status;
use plumbline_world::vif::vif;
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}

fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "traced off a made-up plan" }
}

struct House {
    job: Job,
    ground: String,
    second: String,
    w6: String,
}

/// Two levels, 9 ft each, and a 6 in wall type.
fn house() -> House {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-22");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let second = ed::storey(&mut job, "second", "9 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    House { job, ground, second, w6 }
}

fn wall(h: &mut House, level: &str, a: P2, b: P2) -> String {
    let w6 = h.w6.clone();
    ed::walls(&mut h.job, level, &w6, &[a, b], false, ed::Top::Height("9 ft"), known()).unwrap().remove(0)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1.0e-6
}

fn of(job: &mut Job, path: &str) -> carries::Carry {
    carries::of(job, path).unwrap()
}

#[test]
fn a_wall_above_within_the_thickness_stands_on_the_one_below_and_one_farther_off_does_not() {
    // The wall below, 20 ft along the south; 6 in thick, so a wall above whose centre line is
    // within 6 in of it stands on it, and one 7.2 in off does not.
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let below = wall(&mut h, &g, [0.0, 0.0], [20.0, 0.0]);
    let near_by = wall(&mut h, &s, [0.0, 0.5], [20.0, 0.5]);
    let c = of(&mut h.job, &below);
    assert_eq!(c.carries.len(), 1, "the wall 6 in off is over it: {c:?}");
    assert_eq!(c.carries[0].path, near_by);
    assert!(near(c.carries[0].shared, 20.0));
    assert!(c.stands_on.is_empty() && near(c.nothing_under, 20.0) && c.lowest);
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let below = wall(&mut h, &g, [0.0, 0.0], [20.0, 0.0]);
    let far = wall(&mut h, &s, [0.0, 0.6], [20.0, 0.6]);
    assert!(of(&mut h.job, &below).carries.is_empty(), "7.2 in off is not over it");
    let above = of(&mut h.job, &far);
    assert!(above.stands_on.is_empty() && near(above.nothing_under, 20.0), "7.2 in off is not over it: {above:?}");
    assert!(!above.lowest, "it is on an upper level, and the words say it stands on the floor alone");
    assert!(above.stands_on_said().iter().any(|l| l.contains("it stands on the floor alone there, which is ordinary for a partition")), "{:?}", above.stands_on_said());
}

#[test]
fn a_wall_above_at_an_angle_stands_on_the_one_below_only_within_one_degree() {
    // 6 ft long: at 0.5 degree its far end is 0.052 ft off the line, at 2 degrees 0.21 ft:
    // both are inside the thickness, so only the direction tells them apart.
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let below = wall(&mut h, &g, [0.0, 0.0], [6.0, 0.0]);
    let at_half = wall(&mut h, &s, [0.0, 0.0], [6.0, 6.0 * 0.5f64.to_radians().tan()]);
    let c = of(&mut h.job, &below);
    assert_eq!(c.carries.iter().map(|o| o.path.clone()).collect::<Vec<_>>(), vec![at_half]);
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let below = wall(&mut h, &g, [0.0, 0.0], [6.0, 0.0]);
    let _at_two = wall(&mut h, &s, [0.0, 0.0], [6.0, 6.0 * 2.0f64.to_radians().tan()]);
    assert!(of(&mut h.job, &below).carries.is_empty(), "2 degrees is not the same direction");
}

#[test]
fn a_wall_above_only_partly_over_shares_only_the_part_and_the_rest_is_found_to_have_nothing_under_it() {
    // Below 0 to 10; above 6 to 16 on the same line: 4 ft shared, and 12 ft of the wall above has nothing under it.
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let below = wall(&mut h, &g, [0.0, 0.0], [10.0, 0.0]);
    let above = wall(&mut h, &s, [6.0, 0.0], [16.0, 0.0]);
    let c = of(&mut h.job, &below);
    assert!(near(c.carries[0].shared, 4.0), "{c:?}");
    let a = of(&mut h.job, &above);
    assert!(near(a.stands_on[0].shared, 4.0) && near(a.nothing_under, 6.0), "10 ft long, 4 ft over a wall, 6 ft over nothing: {a:?}");
}

#[test]
fn two_walls_above_are_both_listed_and_a_footing_under_part_of_a_wall_leaves_the_rest_bare() {
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let below = wall(&mut h, &g, [0.0, 0.0], [10.0, 0.0]);
    let (one, two) = (wall(&mut h, &s, [0.0, 0.0], [4.0, 0.0]), wall(&mut h, &s, [6.0, 0.0], [10.0, 0.0]));
    // A footing 24 in wide under the first 6 ft of it.
    let ft = ed::footing_type(&mut h.job, "f24", "24 in", known()).unwrap();
    let foot = ed::footings(&mut h.job, &g, &ft, &[[0.0, 0.0], [6.0, 0.0]], false, "12 in", known()).unwrap().remove(0);
    let c = of(&mut h.job, &below);
    assert_eq!(c.carries.iter().map(|o| (o.path.clone(), o.shared)).collect::<Vec<_>>(), vec![(one, 4.0), (two, 4.0)]);
    assert_eq!(c.stands_on.len(), 1);
    assert!(c.stands_on[0].path == foot && near(c.stands_on[0].shared, 6.0));
    assert!(near(c.nothing_under, 4.0), "10 ft, 6 ft of it on a footing: {c:?}");
    assert!(c.stands_on_said().last().unwrap().contains("nothing found under it for 4.00 LF (no footing)"));
}

#[test]
fn a_wall_of_the_level_below_that_comes_down_is_listed_and_not_counted() {
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let below = wall(&mut h, &g, [0.0, 0.0], [10.0, 0.0]);
    let above = wall(&mut h, &s, [0.0, 0.0], [10.0, 0.0]);
    ed::mark(&mut h.job, &[below.clone()], WorkStatus::Demolish, ed::ItsLines::Keep, known()).unwrap();
    let a = of(&mut h.job, &above);
    assert_eq!(a.stands_on.len(), 1);
    assert!(a.stands_on[0].comes_down && near(a.nothing_under, 10.0), "{a:?}");
    // And the wall that comes down still says what stands on it, as it was drawn.
    let b = of(&mut h.job, &below);
    assert!(b.carries[0].shared == 10.0 && !b.carries[0].comes_down);
}

fn roofed(eave: &str, edges: [Rise; 4]) -> (House, String, String) {
    // A 24 by 16 box on the second level, walls to 18 ft; a roof over it at `eave`.
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let w6 = h.w6.clone();
    let ring = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
    let first = ed::walls(&mut h.job, &g, &w6, &ring, true, ed::Top::Height("9 ft"), known()).unwrap();
    let second = ed::walls(&mut h.job, &s, &w6, &ring, true, ed::Top::Height("9 ft"), known()).unwrap();
    let val = |r: Rise| Val { value: Some(r), unit: if matches!(r, Rise::Gable) { String::new() } else { units::PITCH.into() }, status: Status::Known, source: "made-up survey".into() };
    let (len, unit) = units::parse_length(eave).unwrap();
    let footprint = Footprint {
        points: Val { value: Some(ring.to_vec()), unit: units::FT.into(), status: Status::Known, source: "made-up survey".into() },
        eave_height: Val { value: Some(len), unit: unit.into(), status: Status::Known, source: "made-up survey".into() },
        edges: edges.into_iter().map(val).collect(),
    };
    solve::new_roof(&mut h.job, "Main roof", footprint).unwrap();
    (h, first[0].clone(), second[0].clone())
}

#[test]
fn an_eave_stands_on_the_wall_under_it_and_a_rake_does_not() {
    // The ridge runs east-west: the south and north edges are eaves, the east and west gables.
    let (mut h, _, south2) = roofed("18 ft", [Rise::Pitch(6.0), Rise::Gable, Rise::Pitch(6.0), Rise::Gable]);
    let c = of(&mut h.job, &south2);
    assert_eq!(c.carries.len(), 1);
    assert!(c.carries[0].kind == Kind::Eave && near(c.carries[0].shared, 24.0), "{c:?}");
    // The east wall has a rake over it, and a rake is not listed.
    let east2 = carries::all(&mut h.job).into_iter().filter(|c| c.storey_name == "second" && near(c.length, 16.0)).collect::<Vec<_>>();
    assert_eq!(east2.len(), 2);
    assert!(east2.iter().all(|c| c.carries.is_empty()), "{east2:?}");
}

#[test]
fn an_eave_two_floors_up_is_not_over_the_first_floors_wall() {
    // The roof's eave is at 18 ft: the first floor's walls top out at 9 ft, 9 ft below it.
    let (mut h, south1, _) = roofed("18 ft", [Rise::Pitch(6.0), Rise::Gable, Rise::Pitch(6.0), Rise::Gable]);
    let c = of(&mut h.job, &south1);
    assert!(c.carries.iter().all(|o| o.kind != Kind::Eave), "{c:?}");
    // A roof whose eave is at 9 ft is over it.
    let (mut h, south1, _) = roofed("9 ft", [Rise::Pitch(6.0), Rise::Gable, Rise::Pitch(6.0), Rise::Gable]);
    assert!(of(&mut h.job, &south1).carries.iter().any(|o| o.kind == Kind::Eave));
}

fn floored(span: Option<&str>) -> (House, String, String, String) {
    // A 24 by 16 floor on the second level; a partition across it north-south at east 12 and one east-west at north 8, on the first.
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let w6 = h.w6.clone();
    let ring = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
    let outer = ed::walls(&mut h.job, &g, &w6, &ring, true, ed::Top::Height("9 ft"), known()).unwrap();
    let ns = wall(&mut h, &g, [12.0, 0.0], [12.0, 16.0]);
    let ew = wall(&mut h, &g, [0.0, 8.0], [24.0, 8.0]);
    let ft = ed::floor_type(&mut h.job, "f10", "10 in", known()).unwrap();
    let made = ed::floor(&mut h.job, &s, &ft, ed::FloorBy::Points(&ring), known()).unwrap();
    let floor = made.path.clone();
    if let Some(way) = span {
        h.job.set(&floor, "joists_span", Some(way), Status::Known, "made-up survey").unwrap();
    }
    (h, ns, ew, outer[0].clone())
}

#[test]
fn the_joists_of_the_floor_above_cross_a_wall_or_run_alongside_it_or_are_not_said() {
    // The short way: joists run north-south, the 16 ft; the long way: east-west, the 24 ft.
    let (mut h, ns, ew, _) = floored(Some("the short way"));
    assert_eq!(of(&mut h.job, &ns).joists.map(|j| j.way), Some(JoistWay::Alongside));
    assert_eq!(of(&mut h.job, &ew).joists.map(|j| j.way), Some(JoistWay::Cross));
    let (mut h, ns, ew, _) = floored(Some("the long way"));
    assert_eq!(of(&mut h.job, &ns).joists.map(|j| j.way), Some(JoistWay::Cross));
    assert_eq!(of(&mut h.job, &ew).joists.map(|j| j.way), Some(JoistWay::Alongside));
    let (mut h, ns, ew, _) = floored(None);
    for p in [&ns, &ew] {
        let c = of(&mut h.job, p);
        assert_eq!(c.joists.as_ref().map(|j| j.way), Some(JoistWay::NotSaid));
        assert!(c.carries_said().iter().any(|l| l == "which way the joists of the floor above run is not said"));
        assert!(near(c.joists.unwrap().inside, if p == &ns { 16.0 } else { 24.0 }));
    }
    // Said, they are in the words only when they cross.
    let (mut h, ns, _, _) = floored(Some("the short way"));
    assert!(!of(&mut h.job, &ns).carries_said().iter().any(|l| l.contains("joists")));
}

#[test]
fn a_wall_along_the_edge_of_the_floor_above_is_said_nothing_of_joists() {
    let (mut h, _, _, south) = floored(None);
    assert!(of(&mut h.job, &south).joists.is_none(), "the floor's edge rests on it whichever way the joists run");
}

/// The first floor's partition at east 12, 9 ft tall, with one above it from north 4 to 16 on the second: both made.
fn partitions() -> (House, String, String) {
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let p1 = wall(&mut h, &g, [12.0, 0.0], [12.0, 16.0]);
    let p2 = wall(&mut h, &s, [12.0, 4.0], [12.0, 16.0]);
    (h, p1, p2)
}

#[test]
fn a_wall_that_comes_down_with_a_wall_standing_on_it_is_a_caution_a_question_and_a_beam_answers_it() {
    let (mut h, p1, p2) = partitions();
    ed::mark(&mut h.job, &[p1.clone()], WorkStatus::Demolish, ed::ItsLines::Keep, known()).unwrap();
    let said = carries::asked(&mut h.job);
    let lines = &said[&p1];
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("stands on it for 12.00 LF: once it is down, nothing is found under that wall for 12.00 LF"), "{lines:?}");
    assert!(lines[0].contains(carries::SAID));
    let cautions = carries::cautions(&mut h.job);
    assert_eq!(cautions.len(), 1);
    assert!(cautions[0].1.starts_with("Wall 1 is to demolish, and wall 1 on second stands on it for 12.00 LF: once it is down"), "{:?}", cautions[0].1);
    // The field list asks it, whatever else it asks.
    let q = vif(&mut h.job).questions.into_iter().find(|q| q.path == p1).expect("the partition is on the field list");
    assert!(q.also.iter().any(|a| a.contains("stands on it for 12.00 LF")), "{q:?}");
    assert!(q.to_do().contains("once it is down"), "{}", q.to_do());
    // A beam under the wall above, partly: 8 of its 12 ft, from north 4 to 12. 4 ft are left.
    let ty = ed::framing_type(&mut h.job, "lvl", Kind::Beam, "3 1/2 in", Some("11 7/8 in"), None, assumed()).unwrap();
    let second = h.second.clone();
    let beam = ed::beams(&mut h.job, &second, &ty, &[[12.0, 4.0], [12.0, 12.0]], "0 ft", known()).unwrap().remove(0);
    assert!(near(of(&mut h.job, &p2).stands_on.iter().find(|u| u.path == beam).unwrap().shared, 8.0));
    let said = carries::asked(&mut h.job);
    assert!(said[&p1][0].contains("nothing is found under that wall for 4.00 LF"), "{:?}", said[&p1]);
    // The whole of it: answered, and nothing is said.
    ed::remove(&mut h.job, &beam).unwrap();
    ed::beams(&mut h.job, &second, &ty, &[[12.0, 4.0], [12.0, 16.0]], "0 ft", known()).unwrap();
    assert!(carries::asked(&mut h.job).is_empty());
    assert!(carries::cautions(&mut h.job).is_empty());
    // A beam well below, at the first floor's floor, is not under the wall above.
    let (mut h, p1, p2) = partitions();
    ed::mark(&mut h.job, &[p1.clone()], WorkStatus::Demolish, ed::ItsLines::Keep, known()).unwrap();
    let ty = ed::framing_type(&mut h.job, "lvl", Kind::Beam, "3 1/2 in", Some("11 7/8 in"), None, assumed()).unwrap();
    let ground = h.ground.clone();
    ed::beams(&mut h.job, &ground, &ty, &[[12.0, 4.0], [12.0, 16.0]], "3 ft", known()).unwrap();
    assert!(of(&mut h.job, &p2).stands_on.iter().all(|u| u.kind != Kind::Beam), "a beam 6 ft under it is not under it");
    assert!(!carries::asked(&mut h.job).is_empty());
}

#[test]
fn a_wall_the_work_cuts_with_a_wall_on_it_asks_for_the_header_and_a_beam_does_not_answer_it() {
    let (mut h, p1, _) = partitions();
    ed::mark(&mut h.job, &[p1.clone()], WorkStatus::Existing, ed::ItsLines::Keep, known()).unwrap();
    let wt = ed::opening_type(&mut h.job, "w3x5", Kind::Window, "3 ft", "5 ft", known()).unwrap();
    ed::opening(&mut h.job, &p1, Kind::Window, &wt, ed::OnWall::Offset("8 ft"), "2.5 ft", known()).unwrap();
    let said = carries::asked(&mut h.job);
    assert!(said[&p1][0].contains("stands on it for 12.00 LF and the work cuts it for window 1"), "{:?}", said[&p1]);
    let ty = ed::framing_type(&mut h.job, "lvl", Kind::Beam, "3 1/2 in", Some("11 7/8 in"), None, assumed()).unwrap();
    let second = h.second.clone();
    ed::beams(&mut h.job, &second, &ty, &[[12.0, 4.0], [12.0, 16.0]], "0 ft", known()).unwrap();
    assert!(!carries::asked(&mut h.job).is_empty(), "a beam does not answer an opening cut in the wall");
}

#[test]
fn a_header_is_asked_for_each_opening_cut_in_a_wall_bearing_or_not_said_and_its_depth_is_a_rule_of_thumb() {
    let mut h = house();
    let g = h.ground.clone();
    let p1 = wall(&mut h, &g, [0.0, 0.0], [20.0, 0.0]);
    ed::mark(&mut h.job, &[p1.clone()], WorkStatus::Existing, ed::ItsLines::Keep, known()).unwrap();
    let w3 = ed::opening_type(&mut h.job, "w3x5", Kind::Window, "3 ft", "5 ft", known()).unwrap();
    let w6 = ed::opening_type(&mut h.job, "w6x5", Kind::Window, "6 ft", "5 ft", known()).unwrap();
    ed::opening(&mut h.job, &p1, Kind::Window, &w3, ed::OnWall::Offset("4 ft"), "2.5 ft", known()).unwrap();
    ed::opening(&mut h.job, &p1, Kind::Window, &w6, ed::OnWall::Offset("12 ft"), "2.5 ft", known()).unwrap();
    // Bearing not said: asked, for each. 3 ft: a 2x6, 5 1/2 in; 6 ft: a 2x8, 7 1/4 in.
    let asked = carries::headers_asked(&mut h.job);
    assert_eq!(asked[&p1].len(), 2);
    assert!(asked[&p1][0].contains("header over window 1? It is drawn 5 1/2 in deep (Assumed: a rule of thumb used to lay out the cripples, not a size: an engineer or a span table sizes a header)"), "{:?}", asked[&p1]);
    assert!(asked[&p1][1].contains("over window 2? It is drawn 7 1/4 in deep"), "{:?}", asked[&p1]);
    let q = vif(&mut h.job).questions.into_iter().find(|q| q.path == p1).unwrap();
    assert_eq!(q.also.len(), 2);
    // Said bearing: asked still; said not bearing: not asked.
    h.job.set(&p1, "bearing", Some("bearing"), Status::Known, "seen").unwrap();
    assert_eq!(carries::headers_asked(&mut h.job)[&p1].len(), 2);
    h.job.set(&p1, "bearing", Some("not bearing"), Status::Known, "seen").unwrap();
    assert!(carries::headers_asked(&mut h.job).is_empty());
    assert!(vif(&mut h.job).questions.iter().all(|q| q.also.is_empty()));
    // A new wall is not cut: no header is asked of it.
    let mut h = house();
    let g = h.ground.clone();
    let new = wall(&mut h, &g, [0.0, 0.0], [20.0, 0.0]);
    let w3 = ed::opening_type(&mut h.job, "w3x5", Kind::Window, "3 ft", "5 ft", known()).unwrap();
    ed::opening(&mut h.job, &new, Kind::Window, &w3, ed::OnWall::Offset("4 ft"), "2.5 ft", known()).unwrap();
    assert!(carries::headers_asked(&mut h.job).is_empty());
}

#[test]
fn a_wall_said_not_bearing_with_something_on_it_is_a_caution_and_one_said_bearing_is_not() {
    let (mut h, p1, _) = partitions();
    assert!(carries::cautions(&mut h.job).is_empty(), "nothing is said of it");
    h.job.set(&p1, "bearing", Some("not bearing"), Status::Assumed, "looks like a partition").unwrap();
    let cautions = carries::cautions(&mut h.job);
    assert_eq!(cautions.len(), 1);
    assert!(cautions[0].1.contains("is said not bearing, and wall 1 on second stands on it for 12.00 LF"), "{:?}", cautions);
    assert!(cautions[0].1.contains(carries::SAID));
    h.job.set(&p1, "bearing", Some("bearing"), Status::Known, "seen").unwrap();
    assert!(carries::cautions(&mut h.job).is_empty());
    // Said not bearing with nothing on it: nothing is said.
    let mut h = house();
    let g = h.ground.clone();
    let alone = wall(&mut h, &g, [0.0, 0.0], [10.0, 0.0]);
    h.job.set(&alone, "bearing", Some("not bearing"), Status::Known, "seen").unwrap();
    assert!(carries::cautions(&mut h.job).is_empty());
}

#[test]
fn the_walls_whose_bearing_is_not_said_with_something_on_them_are_counted_by_level_and_in_the_job() {
    let (mut h, p1, _) = partitions();
    let u = carries::unsaid(&mut h.job);
    assert_eq!((u.job, u.levels.iter().map(|l| (l.1.clone(), l.2)).collect::<Vec<_>>()), (1, vec![("ground".to_string(), 1), ("second".to_string(), 0)]));
    h.job.set(&p1, "bearing", Some("bearing"), Status::Assumed, "joists lap over it").unwrap();
    assert_eq!(carries::unsaid(&mut h.job).job, 0);
    // Under a wall that comes down it carries nothing, and is not counted.
    let (mut h, p1, p2) = partitions();
    ed::mark(&mut h.job, &[p2], WorkStatus::Demolish, ed::ItsLines::Keep, known()).unwrap();
    let _ = p1;
    assert_eq!(carries::unsaid(&mut h.job).job, 0);
}

#[test]
fn the_figures_rest_on_the_positions_they_were_worked_from() {
    // The wall above traced and Assumed: the wall below's figure is Assumed, and says which.
    let mut h = house();
    let (g, s) = (h.ground.clone(), h.second.clone());
    let w6 = h.w6.clone();
    let below = ed::walls(&mut h.job, &g, &w6, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), known()).unwrap().remove(0);
    ed::walls(&mut h.job, &s, &w6, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    let c = of(&mut h.job, &below);
    assert_eq!(c.status, Status::Assumed);
    assert_eq!(c.rests_on.len(), 1, "{c:?}");
    assert!(c.status_said().starts_with("Assumed: it rests on positions that are not Known: wall 1 on second"), "{}", c.status_said());
    // All Known: Known.
    let (mut h, p1, _) = partitions();
    assert_eq!(of(&mut h.job, &p1).status, Status::Known);
    assert!(of(&mut h.job, &p1).status_said().starts_with("Known"));
}

#[test]
fn only_a_wall_has_figures_and_a_wall_with_no_level_below_is_the_lowest() {
    let (mut h, p1, _) = partitions();
    assert!(carries::of(&mut h.job, "level-9999").is_err());
    let g = h.ground.clone();
    assert!(carries::of(&mut h.job, &g).unwrap_err().contains("not a wall"));
    assert!(of(&mut h.job, &p1).lowest);
}

#[test]
fn asking_for_the_figures_changes_nothing_in_the_job_and_a_job_saved_before_opens_and_saves_the_same() {
    let (mut h, p1, _) = partitions();
    ed::mark(&mut h.job, &[p1.clone()], WorkStatus::Demolish, ed::ItsLines::Keep, known()).unwrap();
    let before = save(&mut h.job);
    let estimate_before = format!("{:?}", view::estimate(&mut h.job).lines.len());
    let _ = carries::all(&mut h.job);
    let _ = carries::cautions(&mut h.job);
    let _ = carries::asked(&mut h.job);
    let _ = carries::headers_asked(&mut h.job);
    let _ = carries::unsaid(&mut h.job);
    let _ = vif(&mut h.job);
    assert_eq!(save(&mut h.job), before, "the file is the same to the byte");
    assert_eq!(format!("{:?}", view::estimate(&mut h.job).lines.len()), estimate_before);
    let (mut back, _) = load(&before).unwrap();
    assert_eq!(save(&mut back), before, "opened and saved, byte for byte");
    assert!(!before.contains("stands_on") && !before.contains("carries"), "nothing of the figures is saved");
    // A made-up whole house, the same way.
    let mut sample = sample::house();
    let text = save(&mut sample);
    let _ = carries::all(&mut sample);
    assert_eq!(save(&mut sample), text);
    let (mut reread, _) = load(&text).unwrap();
    assert_eq!(save(&mut reread), text);
}

#[test]
fn the_whole_made_up_house_is_worked_and_every_wall_is_said_or_has_its_reason() {
    let mut job = sample::house();
    let all = carries::all(&mut job);
    assert!(!all.is_empty());
    for c in &all {
        assert!(!c.carries_said().is_empty() && !c.stands_on_said().is_empty() && !c.status_said().is_empty(), "{c:?}");
        assert!(c.length > 0.0);
    }
}
