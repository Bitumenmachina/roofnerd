//! Call 180: what the second judge found in the moved wall, what a wall carries and the figures.
//! Everything is made up; the figures are worked by hand beside each test.

use plumbline_world::editing as ed;
use plumbline_world::file::save;
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn measure(job: &mut Job, path: &str, name: &str) -> Option<f64> {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).and_then(|m| m.0.get(name).and_then(|d| d.worked()))
}
fn roof_figures(job: &mut Job, roof: &str) -> (f64, f64) {
    let planes: Vec<String> = job.children().get(roof).map(|c| c.values().cloned().collect()).unwrap_or_default();
    let (mut plan, mut surface) = (0.0, 0.0);
    for p in planes {
        plan += measure(job, &p, "plan_area").unwrap_or(0.0);
        surface += measure(job, &p, "surface_area").unwrap_or(0.0);
    }
    (plan, surface)
}
fn roof(job: &mut Job, name: &str, ring: &[[f64; 2]], eave: &str) -> String {
    let rise = units::parse_pitch("6:12").unwrap();
    let (len, unit) = units::parse_length(eave).unwrap();
    let v = |x| Val { value: Some(x), unit: units::PITCH.into(), status: Status::Known, source: "made-up survey".into() };
    let footprint = Footprint {
        edges: ring.iter().map(|_| v(Rise::Pitch(rise))).collect(),
        points: Val { value: Some(ring.to_vec()), unit: units::FT.into(), status: Status::Known, source: "made-up survey".into() },
        eave_height: Val { value: Some(len), unit: unit.into(), status: Status::Known, source: "made-up survey".into() },
    };
    solve::new_roof(job, name, footprint).unwrap().0
}
fn said(m: &ed::Moved) -> String {
    m.said().join("\n")
}

struct Box {
    job: Job,
    level: String,
    walls: [String; 4],
    floor: String,
    roof: String,
}

/// 24 by 16, 6 in walls 9 ft tall; a floor drawn 6 in outside the walls' lines all round (25 by 17
/// = 425.00 SF) and a hip roof (6:12, eave 9 ft) drawn a foot outside them all round (26 by 18 =
/// 468.00 SF on plan, 468 x 1.118034 = 523.24 SF of surface).
fn boxed(floor_out: f64, roof_out: f64) -> Box {
    let mut job = Job::new("Box (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let slab = ed::floor_type(&mut job, "slab", "10 in", known()).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), known()).unwrap();
    let f = floor_out;
    let floor = ed::floor(&mut job, &level, &slab, ed::FloorBy::Points(&[[-f, -f], [24.0 + f, -f], [24.0 + f, 16.0 + f], [-f, 16.0 + f]]), known()).unwrap().path;
    let r = roof_out;
    let roof = roof(&mut job, "R", &[[-r, -r], [24.0 + r, -r], [24.0 + r, 16.0 + r], [-r, 16.0 + r]], "9 ft");
    job.derive();
    Box { job, level, walls: [walls[0].clone(), walls[1].clone(), walls[2].clone(), walls[3].clone()], floor, roof }
}

#[test]
fn a_floor_and_a_roof_drawn_outside_the_walls_go_with_the_wall_and_come_back() {
    let mut b = boxed(0.5, 1.0);
    assert_eq!(measure(&mut b.job, &b.floor, "plan_area"), Some(425.0));
    let (plan, surface) = roof_figures(&mut b.job, &b.roof);
    assert!((plan - 468.0).abs() < 1e-6 && (surface - 468.0 * 1.118034).abs() < 0.01, "{plan} {surface}");
    let before = save(&mut b.job);
    let east = b.walls[1].clone();
    let moved = ed::move_wall(&mut b.job, &east, 4.0, 0.0, known(), false).unwrap();
    b.job.derive();
    // By hand: floor 29 x 17 = 493.00 SF; roof plan 30 x 18 = 540.00 SF, surface 540 x 1.118034 = 603.74 SF.
    assert_eq!(measure(&mut b.job, &b.floor, "plan_area"), Some(493.0), "{}", said(&moved));
    let (plan, surface) = roof_figures(&mut b.job, &b.roof);
    assert!((plan - 540.0).abs() < 1e-6, "{plan}: {}", said(&moved));
    assert!((surface - 540.0 * 1.118034).abs() < 0.01, "{surface}");
    // Nothing is left behind unsaid: nothing was left.
    assert!(!said(&moved).contains("left where it was"), "{}", said(&moved));
    // Moved back: both as before, every figure.
    let back = ed::move_wall(&mut b.job, &east, -4.0, 0.0, known(), false).unwrap();
    b.job.derive();
    assert_eq!(measure(&mut b.job, &b.floor, "plan_area"), Some(425.0), "{}", said(&back));
    let (plan, surface) = roof_figures(&mut b.job, &b.roof);
    assert!((plan - 468.0).abs() < 1e-6 && (surface - 468.0 * 1.118034).abs() < 0.01);
    let _ = before;
}

/// Two more inputs of the kind the judge did not try: the south wall moved 2 ft north (the floor 25 x 15
/// = 375 SF, the roof 26 x 16 = 416 SF), and a roof 3 ft out and a floor 1 ft out (which is not
/// the thickness) on the west wall moved 3 ft west: floor 27 x 18 = 486 SF, roof 32 x 22 = 704 SF plan... by hand below.
#[test]
fn the_same_by_other_walls_and_other_overhangs() {
    let mut b = boxed(0.5, 1.0);
    let south = b.walls[0].clone();
    let moved = ed::move_wall(&mut b.job, &south, 0.0, 2.0, known(), false).unwrap();
    b.job.derive();
    assert_eq!(measure(&mut b.job, &b.floor, "plan_area"), Some(25.0 * 15.0), "{}", said(&moved));
    let (plan, _) = roof_figures(&mut b.job, &b.roof);
    assert!((plan - 26.0 * 16.0).abs() < 1e-6, "{plan}: {}", said(&moved));
    // Floor 1 ft out and roof 3 ft out (the eave of a porch roof), west wall 3 ft west.
    let mut c = boxed(1.0, 3.0);
    let west = c.walls[3].clone();
    let moved = ed::move_wall(&mut c.job, &west, -3.0, 0.0, known(), false).unwrap();
    c.job.derive();
    // Floor was 26 x 18; now 29 x 18 = 522. Roof was 30 x 22; now 33 x 22 = 726.
    assert_eq!(measure(&mut c.job, &c.floor, "plan_area"), Some(522.0), "{}", said(&moved));
    let (plan, _) = roof_figures(&mut c.job, &c.roof);
    assert!((plan - 726.0).abs() < 1e-6, "{plan}: {}", said(&moved));
}

/// What is not carried is said by name: a roof 6 ft out is farther than 4 ft and stays; it is named.
#[test]
fn what_does_not_go_with_the_wall_is_said_by_name() {
    let mut b = boxed(0.5, 6.0);
    let east = b.walls[1].clone();
    let moved = ed::move_wall(&mut b.job, &east, 4.0, 0.0, known(), false).unwrap();
    b.job.derive();
    let (plan, _) = roof_figures(&mut b.job, &b.roof);
    assert!((plan - 36.0 * 28.0).abs() < 1e-6, "{plan} (left where it was): {}", said(&moved));
    assert!(said(&moved).contains("R was left where it was"), "{}", said(&moved));
    let _ = b.level;
}

/// `check` says what is drawn out of reach of the walls under it: a wall moved alone leaves the floor and
/// the roof short of it; a roof drawn 6 ft out reaches past the wall by 6 ft. And a box drawn true says nothing.
#[test]
fn check_says_a_roof_or_floor_that_no_longer_reaches_the_walls_or_reaches_past_them() {
    let mut b = boxed(0.5, 1.0);
    assert!(building::reach_cautions(&mut b.job).is_empty(), "{:?}", building::reach_cautions(&mut b.job));
    let east = b.walls[1].clone();
    ed::move_wall(&mut b.job, &east, 4.0, 0.0, known(), true).unwrap();
    b.job.derive();
    let said: Vec<String> = building::reach_cautions(&mut b.job).into_iter().map(|c| c.1).collect();
    // By hand: the east wall's line is at x = 28; the floor's edge at 24.5 is 3.5 ft short (3 ft 6 in), the roof's at 25 is 3 ft short.
    assert!(said.iter().any(|s| s.contains("wall 2") && s.contains("3 ft 6 in outside floor 1") && s.contains("no longer reaches")), "{said:#?}");
    assert!(said.iter().any(|s| s.contains("wall 2") && s.contains("3 ft outside R")), "{said:#?}");
    let mut c = boxed(0.5, 6.0);
    let said: Vec<String> = building::reach_cautions(&mut c.job).into_iter().map(|c| c.1).collect();
    assert!(said.iter().any(|s| s.contains("R reaches 6 ft past")), "{said:#?}");
}

/// B2. A follower that turns can run through a wall it does not meet. 24 by 16 with a partition on x = 18:
/// the south wall moved 10 ft west carries the east wall's south end from (24, 0) to (14, 0), so the east
/// wall now runs (14, 0) to (24, 16), and crosses x = 18 at y = 16 x (18 - 14) / 10 = 6.4. Refused,
/// naming the east wall and the partition, and nothing changed.
#[test]
fn a_follower_that_turns_into_a_wall_it_does_not_meet_refuses_the_move() {
    let mut b = boxed(0.0, 1.0);
    let w6 = { let mut q = b.job.world.query::<(&Node, &Kind)>(); q.iter(&b.job.world).find(|(n, k)| **k == Kind::Type && n.name == "w6").map(|(n, _)| n.path.clone()).unwrap() };
    let part = ed::walls(&mut b.job, &b.level, &w6, &[[18.0, 0.0], [18.0, 16.0]], false, ed::Top::Height("9 ft"), known()).unwrap()[0].clone();
    b.job.derive();
    let before = save(&mut b.job);
    let south = b.walls[0].clone();
    let why = ed::move_wall(&mut b.job, &south, -10.0, 0.0, known(), false).unwrap_err();
    assert!(why.contains("wall 2") && why.contains("wall 5") && why.contains("does not meet"), "{why}");
    assert_eq!(save(&mut b.job), before, "nothing changed");
    // The partition itself: the east wall by hand also turns when the north wall is moved the same way.
    let north = b.walls[2].clone();
    let why = ed::move_wall(&mut b.job, &north, -10.0, 0.0, known(), false).unwrap_err();
    assert!(why.contains("wall 2") && why.contains("wall 5"), "{why}");
    let _ = part;
    // A move that turns nothing across a wall is not refused: the partition slid 2 ft east.
    assert!(ed::move_wall(&mut b.job, &b.walls[1].clone(), 0.0, 0.0, known(), false).is_err()); // by nothing
}

// ------------------------------------------------------------------ A. what a wall carries

use plumbline_world::carries;
use plumbline_world::vif::vif;

struct Levels {
    job: Job,
    ground: String,
    second: String,
    w6: String,
}

/// Two levels 9 ft each, 6 in walls.
fn levels() -> Levels {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-23");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let second = ed::storey(&mut job, "second", "9 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    Levels { job, ground, second, w6 }
}
fn wall(l: &mut Levels, level: &str, a: [f64; 2], b: [f64; 2]) -> String {
    let w6 = l.w6.clone();
    ed::walls(&mut l.job, level, &w6, &[a, b], false, ed::Top::Height("9 ft"), known()).unwrap().remove(0)
}
fn carry(job: &mut Job, path: &str) -> carries::Carry {
    carries::of(job, path).unwrap()
}

/// Two walls 24 ft long, both 6 in thick, the upper one's line offset from the lower's by 0, 2, 3, 5, 6 and
/// 12 in. Half the upper wall's thickness is 3 in; it lies over the lower wall's body (3 in either side of
/// the lower line) when the lines are at most 3 in apart. By hand, from the bodies: the upper body spans
/// (offset - 3 in, offset + 3 in); the lower (-3 in, 3 in); the overlap is 6 in - offset: 6, 4, 3 in at 0, 2, 3 in
/// (stands: at 3 in the overlap is exactly half the thickness, and "at least half" stands: the boundary
/// falls on the standing side), and 1, 0, 0 in at 5, 6 and 12 in (beside it, not on it: 5 and 6 touch
/// or overlap by a hair, and 12 in is 6 in clear of the face).
#[test]
fn a_wall_above_stands_on_the_one_below_for_as_long_as_half_its_thickness_lies_over_it() {
    for (inches, stands) in [(0.0, true), (2.0, true), (3.0, true), (5.0, false), (6.0, false), (12.0, false)] {
        let mut l = levels();
        let (g, s) = (l.ground.clone(), l.second.clone());
        let below = wall(&mut l, &g, [0.0, 0.0], [24.0, 0.0]);
        let above = wall(&mut l, &s, [0.0, inches / 12.0], [24.0, inches / 12.0]);
        let up = carry(&mut l.job, &above);
        let down = carry(&mut l.job, &below);
        let (up_said, down_said) = (up.stands_on_said().join("; "), down.carries_said().join("; "));
        if stands {
            assert!(up_said.contains("for 24.00 LF") && !up_said.contains("beside it, not on it") && !up_said.contains("nothing found"), "{inches} in: {up_said}");
            assert!(down_said.contains("for 24.00 LF") && !down_said.contains("beside it, not on it"), "{inches} in: {down_said}");
        } else {
            assert!(up_said.contains("beside it, not on it") && up_said.contains("nothing found under it for 24.00 LF"), "{inches} in: {up_said}");
            assert!(down_said.contains("beside it, not on it") && !down_said.contains("nothing found standing on it"), "{inches} in: {down_said}");
        }
    }
}

/// The upper wall turned 2 degrees about its middle: its line is d = s sin(2 deg) off the lower's, s
/// from its middle, in feet. Half its thickness (3 in = 0.25 ft) lies over the lower body while
/// d <= 0.25 ft, that is |s| <= 0.25 / sin(2 deg) = 7.16349 ft: 14.327 LF of its 24 ft. Not none, as it was; not 24.
#[test]
fn a_wall_turned_two_degrees_stands_on_the_part_of_the_wall_below_it_is_over() {
    let mut l = levels();
    let (g, s) = (l.ground.clone(), l.second.clone());
    let below = wall(&mut l, &g, [0.0, 0.0], [24.0, 0.0]);
    let (c, t) = (2f64.to_radians().cos(), 2f64.to_radians().sin());
    let above = wall(&mut l, &s, [12.0 - 12.0 * c, -12.0 * t], [12.0 + 12.0 * c, 12.0 * t]);
    let up = carry(&mut l.job, &above);
    let on = up.stands_on.iter().find(|u| u.path == below).expect("it stands on it for part of its length");
    assert!((on.shared - 14.327).abs() < 0.005, "{}", on.shared);
    assert!((up.nothing_under - (24.0 - 14.327)).abs() < 0.01, "{}", up.nothing_under);
    let down = carry(&mut l.job, &below);
    assert!((down.carries[0].shared - 14.327).abs() < 0.005, "{}", down.carries[0].shared);
    // 1 degree: |s| <= 0.25 / sin(1 deg) = 14.32 ft, past its half length of 12 ft: all of it, 24.00 LF.
    let mut l = levels();
    let (g, s) = (l.ground.clone(), l.second.clone());
    let below = wall(&mut l, &g, [0.0, 0.0], [24.0, 0.0]);
    let (c, t) = (1f64.to_radians().cos(), 1f64.to_radians().sin());
    let above = wall(&mut l, &s, [12.0 - 12.0 * c, -12.0 * t], [12.0 + 12.0 * c, 12.0 * t]);
    let up = carry(&mut l.job, &above);
    assert!((up.stands_on.iter().find(|u| u.path == below).unwrap().shared - 24.0).abs() < 0.01);
}

/// A beam in the wall's line whose top is within 2 ft below the wall's base is under it: the wall above stands
/// at 9 ft; a beam top at 7 ft 6 in (18 in below) is under it, one at 6 ft 6 in (2 ft 6 in below) is not, and one
/// at 10 ft (a foot through the floor) is not.
#[test]
fn a_beam_within_two_feet_below_the_wall_is_under_it_and_one_farther_below_or_above_is_not() {
    for (top, under) in [("9 ft", true), ("7 ft 6 in", true), ("7 ft", true), ("6 ft 6 in", false), ("10 ft", false)] {
        let mut l = levels();
        let (g, s) = (l.ground.clone(), l.second.clone());
        let above = wall(&mut l, &s, [0.0, 8.0], [24.0, 8.0]);
        let ty = ed::framing_type(&mut l.job, "lvl", Kind::Beam, "3 1/2 in", Some("11 7/8 in"), None, known()).unwrap();
        ed::beams(&mut l.job, &g, &ty, &[[0.0, 8.0], [24.0, 8.0]], top, known()).unwrap();
        let up = carry(&mut l.job, &above);
        assert_eq!(up.stands_on.iter().any(|u| u.kind == Kind::Beam), under, "beam top {top}: {:?}", up.stands_on_said());
    }
}

/// A roof is over a wall by the plan alone: a one-storey box, 9 ft walls, and a roof whose eave is 11 ft 6 in up
/// (2 ft 6 in over the wall tops) drawn over them. Every wall under an eave says so, 24, 16, 24 and 16 LF.
#[test]
fn a_raised_eave_is_over_the_walls_under_it() {
    let mut l = levels();
    let g = l.ground.clone();
    let w6 = l.w6.clone();
    let ring = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
    let walls = ed::walls(&mut l.job, &g, &w6, &ring, true, ed::Top::Height("9 ft"), known()).unwrap();
    roof(&mut l.job, "R", &ring, "11 ft 6 in");
    for (w, long) in walls.iter().zip([24.0, 16.0, 24.0, 16.0]) {
        let c = carry(&mut l.job, w);
        assert!(c.carries.iter().any(|o| o.kind == Kind::Eave && (o.shared - long).abs() < 0.01), "{:?}", c.carries_said());
    }
}

/// Ground, a middle level with nothing drawn on it, and a top: a wall on the top over a wall on the ground comes
/// down with it: "wall 1 on top, two levels up, lies over this line with nothing drawn between them".
#[test]
fn a_wall_two_levels_up_with_nothing_between_is_said_as_what_it_is_and_cautioned() {
    let mut job = Job::new("A house (not a real job)", "test", "2026-11-23");
    let g = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    ed::storey(&mut job, "middle", "9 ft", "9 ft", known()).unwrap();
    let t = ed::storey(&mut job, "top", "18 ft", "9 ft", known()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", known()).unwrap();
    let low = ed::walls(&mut job, &g, &w6, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), known()).unwrap().remove(0);
    let high = ed::walls(&mut job, &t, &w6, &[[0.0, 0.0], [20.0, 0.0]], false, ed::Top::Height("9 ft"), known()).unwrap().remove(0);
    let said = carry(&mut job, &low).carries_said().join("; ");
    assert!(said.contains("two levels up, lies over this line with nothing drawn between them") && said.contains("20.00 LF"), "{said}");
    let up = carry(&mut job, &high).stands_on_said().join("; ");
    assert!(up.contains("two levels down") && !up.contains("nothing found under"), "{up}");
    ed::mark(&mut job, &[low.clone()], WorkStatus::Demolish, ed::ItsLines::Keep, known()).unwrap();
    let cautions = carries::cautions(&mut job);
    assert_eq!(cautions.len(), 1, "{cautions:?}");
    assert!(cautions[0].1.contains("two levels up") && cautions[0].1.contains("20.00 LF") && cautions[0].1.contains(carries::SAID), "{cautions:?}");
}

/// A roof over a wall that is not under its eave: an interior wall under a hip roof's ridge says that a roof is
/// over it, and that whether it bears is not worked; it does not say "nothing found standing on it".
#[test]
fn a_wall_under_a_roof_but_not_under_its_eave_says_a_roof_is_over_it() {
    let mut l = levels();
    let g = l.ground.clone();
    let w6 = l.w6.clone();
    let ring = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
    ed::walls(&mut l.job, &g, &w6, &ring, true, ed::Top::Height("9 ft"), known()).unwrap();
    let part = wall(&mut l, &g, [12.0, 0.0], [12.0, 16.0]);
    roof(&mut l.job, "R", &ring, "9 ft");
    let said = carry(&mut l.job, &part).carries_said().join("; ");
    assert!(said.contains("a roof is over this wall: whether it bears here is not worked (only eaves are)"), "{said}");
    assert!(!said.contains("nothing found standing on it"), "{said}");
    // Taken down, it is a question: nothing is silent about a wall under a ridge.
    ed::mark(&mut l.job, &[part.clone()], WorkStatus::Demolish, ed::ItsLines::Keep, known()).unwrap();
    assert!(carries::asked(&mut l.job)[&part].iter().any(|s| s.to_lowercase().contains("a roof is over it")), "{:?}", carries::asked(&mut l.job));
}

/// A wall said bearing that stands on nothing is a caution and a field question, for as long as nothing is under it.
#[test]
fn a_wall_said_bearing_with_nothing_under_it_is_a_caution_and_a_question() {
    let mut l = levels();
    let (g, s) = (l.ground.clone(), l.second.clone());
    let low = wall(&mut l, &g, [0.0, 0.0], [10.0, 0.0]);
    let high = wall(&mut l, &s, [0.0, 4.0], [10.0, 4.0]);
    l.job.set(&high, "bearing", Some("bearing"), Status::Known, "seen").unwrap();
    let cautions = carries::cautions(&mut l.job);
    assert!(cautions.iter().any(|c| c.0 == high && c.1.contains("said bearing, and nothing is found under it for 10.00 LF")), "{cautions:?}");
    assert!(carries::asked(&mut l.job)[&high].iter().any(|s| s.contains("said bearing, and nothing is found under it for 10.00 LF")), "{:?}", carries::asked(&mut l.job));
    // Put the wall below under it: the caution goes.
    let _ = low;
    let mut l = levels();
    let (g, s) = (l.ground.clone(), l.second.clone());
    wall(&mut l, &g, [0.0, 0.0], [10.0, 0.0]);
    let high = wall(&mut l, &s, [0.0, 0.0], [10.0, 0.0]);
    l.job.set(&high, "bearing", Some("bearing"), Status::Known, "seen").unwrap();
    assert!(carries::cautions(&mut l.job).is_empty(), "{:?}", carries::cautions(&mut l.job));
}

/// Two walls named alike are named with their levels; and what rides on a wall to demolish is what is priced on
/// it and on its openings.
#[test]
fn walls_named_alike_carry_their_levels_and_the_dollars_riding_include_the_openings() {
    let mut l = levels();
    let (g, s) = (l.ground.clone(), l.second.clone());
    let low = wall(&mut l, &g, [0.0, 0.0], [20.0, 0.0]);
    wall(&mut l, &s, [0.0, 0.0], [20.0, 0.0]);
    let wt = ed::opening_type(&mut l.job, "window 3 ft", Kind::Window, "3 ft", "4 ft", known()).unwrap();
    let win = ed::opening(&mut l.job, &low, Kind::Window, &wt, ed::OnWall::Offset("10 ft"), "3 ft", known()).unwrap();
    ed::mark(&mut l.job, &[low.clone()], WorkStatus::Demolish, ed::ItsLines::Keep, known()).unwrap();
    let cautions = carries::cautions(&mut l.job);
    assert!(cautions[0].1.starts_with("Wall 1 on ground is to demolish, and wall 1 on second stands on it"), "{cautions:?}");
    // $10.00/SF on the wall's face (20 x 9 = 180 SF = $1,800.00) and $300.00 each on the window.
    l.job.add_line(&low, "Siding", "face_area", None, false).unwrap();
    l.job.set_on_line(&low, "unit_cost", Some("Siding"), Some("$10/SF"), Status::Known, "made-up price").unwrap();
    l.job.add_line(&win, "Window", "1", None, false).unwrap();
    l.job.set_on_line(&win, "unit_cost", Some("Window"), Some("$300/EA"), Status::Known, "made-up price").unwrap();
    l.job.derive();
    let q = vif(&mut l.job).questions.into_iter().find(|q| q.path == low).expect("on the field list");
    assert_eq!(q.dollars, Some(2100.0), "{q:?}");
}

// ------------------------------------------------------------------ C1 and C2: joists

fn floor_job(points: &[[f64; 2]]) -> (Job, String) {
    let mut job = Job::new("Joists (not a real job)", "test", "2026-11-23");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", known()).unwrap();
    let ft = ed::floor_type(&mut job, "floor 10 in", "10 in", known()).unwrap();
    let floor = ed::floor(&mut job, &level, &ft, ed::FloorBy::Points(points), known()).unwrap().path;
    job.add_line(&floor, "Floor joists, 2x10 at 16 in", "joists", None, false).unwrap();
    job.derive();
    (job, floor)
}
fn joists(job: &mut Job, floor: &str) -> (f64, f64, f64) {
    let e = job.find(floor).unwrap();
    let m = &job.world.get::<Measure>(e).unwrap().0;
    (m["joists"].value.unwrap(), m["joist_length"].value.unwrap(), m["rim_length"].value.unwrap())
}

/// C1. A floor 24 by 16 has 24 / 1.3333 + 1 = 19 joists of 16 ft, 304.00 LF, however it is written: a south side
/// 0.0001 ft out of square (1/80 in over 24 ft), and the same floor turned and its corners written to four places.
#[test]
fn a_floor_a_hair_out_of_square_or_turned_and_written_to_four_places_has_the_same_joists() {
    let (mut job, floor) = floor_job(&[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]]);
    let (n, lf, _) = joists(&mut job, &floor);
    assert_eq!((n, (lf * 100.0).round() / 100.0), (19.0, 304.0));
    for nudge in [[24.0, 0.0001], [24.0001, 0.0], [23.9999, 0.0], [24.0, 0.001]] {
        let (mut job, floor) = floor_job(&[[0.0, 0.0], nudge, [24.0, 16.0], [0.0, 16.0]]);
        let (n, lf, _) = joists(&mut job, &floor);
        assert_eq!(n, 19.0, "corner {nudge:?}: {lf} LF");
        assert!((lf - 304.0).abs() < 0.2, "corner {nudge:?}: {lf} LF");
    }
    for degrees in [1.0f64, 3.0, 5.0, 7.0, 45.0, 83.0] {
        let (c, s) = (degrees.to_radians().cos(), degrees.to_radians().sin());
        let turn = |p: [f64; 2]| [((p[0] * c - p[1] * s) * 1.0e4).round() / 1.0e4, ((p[0] * s + p[1] * c) * 1.0e4).round() / 1.0e4];
        let (mut job, floor) = floor_job(&[turn([0.0, 0.0]), turn([24.0, 0.0]), turn([24.0, 16.0]), turn([0.0, 16.0])]);
        let (n, lf, _) = joists(&mut job, &floor);
        assert_eq!(n, 19.0, "turned {degrees} degrees: {lf} LF");
        assert!((lf - 304.0).abs() < 0.2, "turned {degrees} degrees: {lf} LF");
    }
}

/// C2. The rim is the true length of every edge the joists' ends meet. A 24 by 16 floor with a 10 ft chamfer at the
/// north-west corner: 0,0 24,0 24,16 10,16 0,6. Joists run north; their ends meet the south edge (24), the north edge
/// (24 - 10 = 14) and the chamfer, which is 10 ft over and 10 ft up: sqrt(200) = 14.1421. By hand 24 + 14 + 14.1421 =
/// 52.1421 LF; the judge is right (it said 48: the chamfer measured square to the joists, 10). The square floor stays 48.
#[test]
fn the_rim_along_a_chamfer_is_its_true_length() {
    let (mut job, floor) = floor_job(&[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [10.0, 16.0], [0.0, 6.0]]);
    let (_, _, rim) = joists(&mut job, &floor);
    assert!((rim - 52.1421).abs() < 0.001, "{rim}");
    // The 6 ft chamfer: 0,0 24,0 24,16 6,16 0,10: 24 + 18 + sqrt(72) = 24 + 18 + 8.4853 = 50.4853 LF.
    let (mut job, floor) = floor_job(&[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [6.0, 16.0], [0.0, 10.0]]);
    let (_, _, rim) = joists(&mut job, &floor);
    assert!((rim - 50.4853).abs() < 0.001, "{rim}");
    let (mut job, floor) = floor_job(&[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]]);
    assert_eq!(joists(&mut job, &floor).2, 48.0);
}

// ------------------------------------------------------------------ C4 and C5: what may not be drawn

/// C4. Beams that share a line for any length are refused as walls on one line are: two beams in one place
/// are counted twice. A beam from 0 to 10 and one from 10 to 20 on the same line and top are fine (they butt); one
/// from 5 to 15 shares 5 ft with each; one 1 in off the line (3 1/2 in wide, so they overlap) shares its whole length;
/// one 1 ft off, one across at right angles, and one with its top 2 ft lower are not in the same place.
#[test]
fn beams_sharing_a_line_are_refused_and_beams_that_do_not_are_not() {
    let mut l = levels();
    let g = l.ground.clone();
    let ty = ed::framing_type(&mut l.job, "lvl", Kind::Beam, "3 1/2 in", Some("11 7/8 in"), None, known()).unwrap();
    ed::beams(&mut l.job, &g, &ty, &[[0.0, 8.0], [10.0, 8.0]], "9 ft", known()).unwrap();
    ed::beams(&mut l.job, &g, &ty, &[[10.0, 8.0], [20.0, 8.0]], "9 ft", known()).unwrap();
    let why = ed::beams(&mut l.job, &g, &ty, &[[5.0, 8.0], [15.0, 8.0]], "9 ft", known()).unwrap_err();
    assert!(why.contains("two beams") && why.contains("5 ft"), "{why}");
    let why = ed::beams(&mut l.job, &g, &ty, &[[0.0, 8.0 + 1.0 / 12.0], [10.0, 8.0 + 1.0 / 12.0]], "9 ft", known()).unwrap_err();
    assert!(why.contains("two beams") && why.contains("10 ft"), "{why}");
    ed::beams(&mut l.job, &g, &ty, &[[0.0, 9.0], [10.0, 9.0]], "9 ft", known()).unwrap();
    ed::beams(&mut l.job, &g, &ty, &[[5.0, 0.0], [5.0, 16.0]], "9 ft", known()).unwrap();
    ed::beams(&mut l.job, &g, &ty, &[[5.0, 8.0], [15.0, 8.0]], "7 ft", known()).unwrap();
}

/// C4. Pad footings whose plan areas overlap are refused: 24 in square pads at 0,0 and 1,0 overlap by half (the
/// areas 2 ft by 2 ft overlap 1 ft by 2 ft = 2 SF); 2 ft apart they touch along an edge, and are fine; 2.5 ft apart are clear.
#[test]
fn pads_overlapping_are_refused() {
    let mut l = levels();
    let g = l.ground.clone();
    let ty = ed::framing_type(&mut l.job, "pad 24 in", Kind::PadFooting, "24 in", Some("24 in"), None, known()).unwrap();
    ed::pad(&mut l.job, &g, &ty, [0.0, 0.0], "10 in", "0 ft", known()).unwrap();
    let why = ed::pad(&mut l.job, &g, &ty, [1.0, 0.0], "10 in", "0 ft", known()).unwrap_err();
    assert!(why.contains("pad footings"), "{why}");
    ed::pad(&mut l.job, &g, &ty, [2.0, 0.0], "10 in", "0 ft", known()).unwrap();
    ed::pad(&mut l.job, &g, &ty, [4.5, 0.0], "10 in", "0 ft", known()).unwrap();
}

/// C5. A door or a window that goes above the top of its wall is refused, saying the wall's height: a 9 ft wall,
/// a 12 ft door, and a window 3 ft by 4 ft at a sill of 8 ft (top 12 ft). A 7 ft door and a window to the very top
/// (sill 5 ft, 4 ft high: 9 ft) are fine.
#[test]
fn an_opening_taller_than_its_wall_is_refused_saying_the_height_of_the_wall() {
    let mut l = levels();
    let g = l.ground.clone();
    let w = wall(&mut l, &g, [0.0, 0.0], [30.0, 0.0]);
    let tall = ed::opening_type(&mut l.job, "door 12 ft", Kind::Door, "3 ft", "12 ft", known()).unwrap();
    let why = ed::opening(&mut l.job, &w, Kind::Door, &tall, ed::OnWall::Offset("5 ft"), "0 ft", known()).unwrap_err();
    assert!(why.contains("9 ft") && why.contains("high"), "{why}");
    let win = ed::opening_type(&mut l.job, "window 3 by 4", Kind::Window, "3 ft", "4 ft", known()).unwrap();
    let why = ed::opening(&mut l.job, &w, Kind::Window, &win, ed::OnWall::Offset("15 ft"), "8 ft", known()).unwrap_err();
    assert!(why.contains("9 ft") && why.contains("high"), "{why}");
    let door = ed::opening_type(&mut l.job, "door 7 ft", Kind::Door, "3 ft", "7 ft", known()).unwrap();
    ed::opening(&mut l.job, &w, Kind::Door, &door, ed::OnWall::Offset("5 ft"), "0 ft", known()).unwrap();
    ed::opening(&mut l.job, &w, Kind::Window, &win, ed::OnWall::Offset("15 ft"), "5 ft", known()).unwrap();
}

/// A job saved before with one of these still opens, and `check` says it (`drawn_faults`).
#[test]
fn a_job_with_one_already_drawn_still_opens_and_says_it() {
    let mut l = levels();
    let g = l.ground.clone();
    let w = wall(&mut l, &g, [0.0, 0.0], [30.0, 0.0]);
    let door = ed::opening_type(&mut l.job, "door 7 ft", Kind::Door, "3 ft", "7 ft", known()).unwrap();
    let d = ed::opening(&mut l.job, &w, Kind::Door, &door, ed::OnWall::Offset("5 ft"), "0 ft", known()).unwrap();
    // The type made taller in the file: written straight, not by an edit.
    let text = save(&mut l.job).replace("\"value\": 7.0", "\"value\": 12.0");
    let (mut again, _) = plumbline_world::file::load(&text).expect("it opens");
    let faults = building::drawn_faults(&mut again);
    assert!(faults.iter().any(|f| f.1.contains("9 ft") && f.1.contains(&d)), "{faults:?}");
}

// ------------------------------------------------------------------ B3 and B4

/// B3. A wall moved alone leaves its corners open, and says each one, and how far apart: the east wall of the
/// box moved 4 ft east alone leaves the south and north walls' east ends 4 ft short of it.
#[test]
fn a_wall_moved_alone_says_each_corner_it_opens() {
    let mut b = boxed(0.5, 1.0);
    let east = b.walls[1].clone();
    let moved = ed::move_wall(&mut b.job, &east, 4.0, 0.0, known(), true).unwrap();
    let said = said(&moved);
    assert!(said.contains("the corner of wall 1 and wall 2 is open, 4 ft apart"), "{said}");
    assert!(said.contains("the corner of wall 3 and wall 2 is open, 4 ft apart"), "{said}");
    // Not alone: no corner is opened.
    let mut b = boxed(0.5, 1.0);
    let moved = ed::move_wall(&mut b.job, &east, 4.0, 0.0, known(), false).unwrap();
    assert!(!said_of(&moved).contains("is open"), "{}", said_of(&moved));
}
fn said_of(m: &ed::Moved) -> String {
    m.said().join("\n")
}

/// B4. "4 ft to the east", "toward the east" and "4 feet east" are read, as are the compass words spelled out.
#[test]
fn a_move_is_read_with_to_the_toward_and_eastward() {
    for (typed, want) in [
        ("4 ft to the east", (4.0, 0.0)),
        ("4 feet east", (4.0, 0.0)),
        ("4 ft toward the east", (4.0, 0.0)),
        ("4 ft toward east", (4.0, 0.0)),
        ("4 feet to the east and 2 feet to the north", (4.0, 2.0)),
        ("3 ft to the south", (0.0, -3.0)),
        ("4 ft eastward", (4.0, 0.0)),
        ("2 ft 6 in to the west", (-2.5, 0.0)),
    ] {
        assert_eq!(ed::read_move(typed), Ok(want), "{typed}");
    }
    assert!(ed::read_move("4 ft to the").is_err());
    assert!(ed::read_move("east 4 ft").is_err() || ed::read_move("east 4 ft") == Ok((4.0, 0.0)));
}
