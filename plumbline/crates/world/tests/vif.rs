//! The verify-in-field list: what is still a question, what to measure to settle it,
//! and the dollars riding on it. The job and every figure are made up, and worked by
//! hand in STATUS.md ("The verify-in-field list") before they are compared here.

use plumbline_world::editing::{self as ed, Entered, Reading, Top};
use plumbline_world::value::Status;
use plumbline_world::vif::vif;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };
const TRACED: Entered = Entered { status: Status::Assumed, source: "scaled off a made-up plan" };

/// The field job: two levels scaled off a plan; four Known walls from the ground's
/// floor to the upper's, the south one carrying gypsum board on its net face at
/// $2.40/SF; a traced duct; two Known pipes across it.
struct Made {
    job: Job,
    ground: String,
    upper: String,
    wall: String,
    duct: String,
    pipe_a: String,
    pipe_b: String,
}

fn made() -> Made {
    let mut job = Job::new("Verified in the field (not a real job)", "test", "test");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", TRACED).unwrap();
    let upper = ed::storey(&mut job, "upper", "9 ft", "8 ft", TRACED).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let duct_type = ed::segment_type(&mut job, "12 x 8", Kind::Duct, "12 in", Some("8 in"), KNOWN).unwrap();
    let pipe_type = ed::segment_type(&mut job, "2 in", Kind::Pipe, "2 in", None, KNOWN).unwrap();
    let walls = ed::walls(&mut job, &ground, &wall_type, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, Top::Level(&upper), KNOWN).unwrap();
    let duct = ed::runs(&mut job, &ground, Kind::Duct, &duct_type, &[[2.0, 8.0], [22.0, 8.0]], "8 ft", None, TRACED).unwrap().remove(0);
    let pipe_a = ed::runs(&mut job, &ground, Kind::Pipe, &pipe_type, &[[12.0, 2.0], [12.0, 14.0]], "7 ft 9 in", None, KNOWN).unwrap().remove(0);
    let pipe_b = ed::runs(&mut job, &ground, Kind::Pipe, &pipe_type, &[[18.0, 2.0], [18.0, 14.0]], "8 ft", None, KNOWN).unwrap().remove(0);
    let wall = walls[0].clone();
    job.add_line(&wall, "Gypsum board, hung and finished", "net_area", Some(Class::Material), false).unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Gypsum board, hung and finished"), Some("$2.40/SF"), Status::Known, "made-up plan").unwrap();
    job.derive();
    Made { job, ground, upper, wall, duct, pipe_a, pipe_b }
}

/// A row as the test reads it: the thing, what it is in a clash with, its dollars, how
/// many lines, and what to measure or check.
fn rows(job: &mut Job) -> Vec<(String, Option<String>, Option<f64>, usize, Vec<String>)> {
    vif(job).questions.into_iter().map(|q| (q.path, q.clash.map(|c| c.other), q.dollars, q.lines, q.values.into_iter().map(|v| v.0).collect())).collect()
}

#[test]
fn as_traced_two_levels_carry_the_wall_and_the_duct_and_its_clashes_carry_nothing() {
    let mut m = made();
    let list = vif(&mut m.job);
    // 24 ft by 9 ft is 216 SF, at $2.40: $518.40, one line, counted once in the total.
    assert_eq!((list.riding, list.riding_status, list.riding_lines), (Some(518.40), Status::Assumed, 1));
    let found = rows(&mut m.job);
    let levels = vec!["elevation".to_string(), "level height".to_string()];
    assert_eq!(found.len(), 5, "{found:#?}");
    assert_eq!(found[0], (m.ground.clone(), None, Some(518.40), 1, levels.clone()));
    assert_eq!(found[1], (m.upper.clone(), None, Some(518.40), 1, levels));
    // The two clashes, with no dollars, before the duct itself.
    assert_eq!((found[2].0.as_str(), found[2].1.as_deref(), found[2].2, found[2].3), (m.duct.as_str(), Some(m.pipe_a.as_str()), None, 0));
    assert_eq!((found[3].0.as_str(), found[3].1.as_deref(), found[3].2, found[3].3), (m.duct.as_str(), Some(m.pipe_b.as_str()), None, 0));
    assert_eq!((found[4].0.as_str(), found[4].1.as_deref(), found[4].2), (m.duct.as_str(), None, None));
    for value in ["type", "line on the plan", "height above its level"] {
        assert!(found[4].4.iter().any(|v| v == value), "the duct's {value} is to be verified: {:?}", found[4].4);
    }
    // What is Known asks nothing: no wall, no pipe, no type is on the list.
    for known in [&m.wall, &m.pipe_a, &m.pipe_b] {
        assert!(!found.iter().any(|r| r.0 == **known && r.1.is_none()), "{known} is Known and is on the list");
    }
}

#[test]
fn a_measurement_takes_its_question_off_and_a_line_that_is_known_has_nothing_riding() {
    let mut m = made();
    ed::measure(&mut m.job, &m.ground, "elevation", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.ground, "level height", Reading::Length("8'10\"1/2"), None, "JM with the laser").unwrap();
    // The wall is 24 by 8.875 now, 213 SF, $511.20, and Known: the upper level still has
    // its own height to verify, and no dollars ride on that.
    let found = rows(&mut m.job);
    assert!(!found.iter().any(|r| r.0 == m.ground), "the ground level is measured and is still on the list");
    let upper = found.iter().find(|r| r.0 == m.upper).expect("the upper level's height is still as scaled");
    assert_eq!((upper.2, upper.3, upper.4.clone()), (None, 0, vec!["level height".to_string()]));
    assert_eq!(vif(&mut m.job).riding, None);

    ed::measure(&mut m.job, &m.duct, "line on the plan", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.duct, "type", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.duct, "height above its level", Reading::Length("7'10\"3/4"), Some(Justify::Bottom), "JM with the laser").unwrap();
    // Pipe A is clear; pipe B in the duct is a finding now, and a finding is not a question.
    let found = rows(&mut m.job);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].0, m.upper);

    ed::measure(&mut m.job, &m.upper, "level height", Reading::Same, None, "JM on site").unwrap();
    let list = vif(&mut m.job);
    assert!(list.questions.is_empty(), "{:#?}", list.questions);
    assert_eq!(list.riding, None);
}

#[test]
fn a_line_on_two_things_in_question_is_counted_once_in_the_total() {
    let mut m = made();
    // The duct is priced too: 20 LF at $12.00, $240.00. It rides on the duct, and on
    // each of the duct's two clashes; the total counts it once: $518.40 + $240.00.
    m.job.add_line(&m.duct, "Duct, 12 x 8, hung", "true_length", Some(Class::Material), false).unwrap();
    m.job.set_on_line(&m.duct, "unit_cost", Some("Duct, 12 x 8, hung"), Some("$12.00/LF"), Status::Known, "made-up plan").unwrap();
    m.job.derive();
    let list = vif(&mut m.job);
    assert_eq!((list.riding, list.riding_lines), (Some(758.40), 2));
    let duct_rows: Vec<&plumbline_world::vif::Question> = list.questions.iter().filter(|q| q.path == m.duct).collect();
    assert_eq!(duct_rows.len(), 3);
    assert!(duct_rows.iter().all(|q| q.dollars == Some(240.00) && q.lines == 1), "{duct_rows:#?}");
    // Largest first: the two levels at $518.40, then the three at $240.00.
    let order: Vec<Option<f64>> = list.questions.iter().map(|q| q.dollars).collect();
    assert_eq!(order, vec![Some(518.40), Some(518.40), Some(240.00), Some(240.00), Some(240.00)]);
}

/// Dollars ride on a measurement only where the line's own quantity is in question
/// (call 83; M21-2). A wall 40 ft long, traced; its line and its type then checked on
/// site, its height still as traced. Base along it, 40 LF at an Assumed $5.00: $200.00.
/// The base does not use the height, and no tape makes its 40 LF surer: nothing rides
/// on measuring the height. Gypsum board on its face (40 x 9 = 360 SF at $1.00) does
/// use it: $360.00 rides on it.
#[test]
fn a_line_of_a_known_length_at_an_assumed_price_does_not_ride_on_a_measurement() {
    let mut job = Job::new("What rides on a measurement (not a real job)", "test", "test");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let wall = ed::walls(&mut job, &ground, &wall_type, &[[0.0, 0.0], [40.0, 0.0]], false, Top::Height("9 ft"), TRACED).unwrap().remove(0);
    ed::measure(&mut job, &wall, "wall line", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut job, &wall, "type", Reading::Same, None, "JM on site").unwrap();
    job.add_line(&wall, "Base, along the wall", "plan_length", Some(Class::Material), false).unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Base, along the wall"), Some("$5.00/LF"), Status::Assumed, "made-up allowance").unwrap();
    job.derive();
    let found = rows(&mut job);
    // One question: the wall's height. Nothing priced rides on it.
    assert_eq!(found, vec![(wall.clone(), None, None, 0, vec!["height".to_string()])], "{found:#?}");
    assert_eq!(vif(&mut job).riding, None);
    // A line that does use the height rides on it, and only that line.
    job.add_line(&wall, "Gypsum board, hung and finished", "face_area", Some(Class::Material), false).unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Gypsum board, hung and finished"), Some("$1.00/SF"), Status::Known, "made-up plan").unwrap();
    job.derive();
    let found = rows(&mut job);
    assert_eq!(found, vec![(wall.clone(), None, Some(360.00), 1, vec!["height".to_string()])], "{found:#?}");
}

/// A hip roof on a 24 ft by 16 ft footprint, 6:12 on every edge, eaves at 9 ft, all of
/// it traced. Underlayment at $0.50/SF on the plane that rises from edge 1. Returns the
/// job, the roof and that plane.
fn roofed() -> (Job, String, String) {
    let mut job = Job::new("A roof to verify (not a real job)", "test", "test");
    fn traced<T>(value: T, unit: &str) -> value::Val<T> {
        value::Val { value: Some(value), unit: unit.to_string(), status: Status::Assumed, source: "scaled off a made-up plan".to_string() }
    }
    let footprint = Footprint {
        points: traced(vec![[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], "ft"),
        eave_height: traced(units::Len::One(9.0), "ft"),
        edges: (0..4).map(|_| traced(Rise::Pitch(6.0), "in/12")).collect(),
    };
    let (roof, _) = solve::new_roof(&mut job, "main roof", footprint).unwrap();
    let plane = {
        let mut q = job.world.query::<(&Node, &Solved)>();
        q.iter(&job.world).find(|(_, s)| s.edge == Some(1)).map(|(n, _)| n.path.clone()).unwrap()
    };
    job.add_line(&plane, "Underlayment", "surface_area", Some(Class::Material), false).unwrap();
    job.set_on_line(&plane, "unit_cost", Some("Underlayment"), Some("$0.50/SF"), Status::Known, "made-up plan").unwrap();
    job.derive();
    (job, roof, plane)
}

#[test]
fn a_roof_drawn_from_its_footprint_is_on_the_list_with_what_is_priced_on_its_planes() {
    // J22A-18: the roof carried the most dollars on a traced house and was not on the
    // list, because nothing `measure` takes was on a roof solved from its footprint.
    // The plane on the 24 ft eave: on plan (24 + 8) / 2 x 8 = 128 SF; at 6:12 the slope
    // factor is the root of 1.25, 1.118034; 143.1084 SF at $0.50 is $71.55.
    let (mut job, roof, plane) = roofed();
    let list = vif(&mut job);
    assert_eq!((list.riding, list.riding_status, list.riding_lines), (Some(71.55), Status::Assumed, 1));
    let found = rows(&mut job);
    let names = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(found[0], (roof.clone(), None, Some(71.55), 1, names(&["footprint", "eave height", "edge 1", "edge 2", "edge 3", "edge 4"])), "{found:#?}");
    assert_eq!(found[1], (plane.clone(), None, Some(71.55), 1, names(&["eave height", "pitch"])), "{found:#?}");
    // The other three planes are questions with nothing riding: nothing is priced on them.
    assert_eq!(found.iter().filter(|r| r.2.is_none()).count(), 3, "{found:#?}");
}

#[test]
fn a_roof_measured_on_site_leaves_the_list_and_takes_its_dollars_with_it() {
    let (mut job, roof, plane) = roofed();
    // The eave is shot at the plane. It is the roof's, so every plane has it.
    let done = ed::measure(&mut job, &plane, "eave height", Reading::Length("9 ft 2 in"), None, "laser").unwrap();
    assert!(done.said.contains("eave height is 9' 2\", measured (it was 9 ft, Assumed"), "{}", done.said);
    assert!(done.also.iter().any(|a| a.contains("every plane of that roof has the one eave height")), "{:?}", done.also);
    let eaves: Vec<(Status, Option<f64>)> = {
        let mut q = job.world.query::<(&Outline, &Solved)>();
        q.iter(&job.world).map(|(o, _)| (o.eave_height.status, o.eave_height.feet())).collect()
    };
    assert_eq!(eaves.len(), 4);
    assert!(eaves.iter().all(|(s, ft)| *s == Status::Known && (ft.unwrap() - (9.0 + 2.0 / 12.0)).abs() < 1e-9), "{eaves:?}");
    // The pitch of that plane is checked at the plane; the rest at the roof. Until the
    // last one is, the plane's outline rests on a pitch that is Assumed and it rides.
    ed::measure(&mut job, &plane, "pitch", Reading::Same, None, "pitch gauge").unwrap();
    ed::measure(&mut job, &roof, "footprint", Reading::Same, None, "tape").unwrap();
    ed::measure(&mut job, &roof, "edge 2", Reading::Same, None, "pitch gauge").unwrap();
    ed::measure(&mut job, &roof, "edge 3", Reading::Same, None, "pitch gauge").unwrap();
    let found = rows(&mut job);
    assert_eq!(found[0], (roof.clone(), None, Some(71.55), 1, vec!["edge 4".to_string()]), "{found:#?}");
    // The plane has nothing left on it that is its own to check, so it is not asked of twice.
    assert!(!found.iter().any(|r| r.0 == plane), "{found:#?}");
    ed::measure(&mut job, &roof, "edge 4", Reading::Same, None, "pitch gauge").unwrap();
    let list = vif(&mut job);
    assert!(list.questions.is_empty(), "{:#?}", list.questions);
    assert_eq!(list.riding, None);
    // What a roof has no value called is refused with what it has.
    let why = ed::measure(&mut job, &roof, "edge 9", Reading::Same, None, "pitch gauge").unwrap_err();
    assert!(why.contains("has no entered value called \"edge 9\""), "{why}");
    // And the job still reads back as it was saved.
    assert!(file::load(&file::save(&mut job)).is_ok());
}

#[test]
fn what_is_priced_on_a_thing_is_said_as_it_is_deleted() {
    // J22A-17: a priced wall went with "Deleted wall 1." and the price fell without a
    // word. The wall carries 216 SF of gypsum board at $2.40: $518.40.
    let mut m = made();
    let before = view::estimate(&mut m.job);
    ed::remove(&mut m.job, &m.pipe_a).unwrap();
    assert_eq!(view::went_with_said(&before, &mut m.job), None, "nothing is priced on the pipe");
    let before = view::estimate(&mut m.job);
    ed::remove(&mut m.job, &m.wall).unwrap();
    assert_eq!(view::went_with_said(&before, &mut m.job).as_deref(), Some("1 cost line went with it: $518.40 is out of the job cost."));
    // A line with no price yet says the price is as it was.
    let mut m = made();
    m.job.add_line(&m.duct, "Duct, hung", "true_length", Some(Class::Material), false).unwrap();
    m.job.derive();
    let before = view::estimate(&mut m.job);
    ed::remove(&mut m.job, &m.duct).unwrap();
    assert_eq!(view::went_with_said(&before, &mut m.job).as_deref(), Some("1 cost line went with it; none of them had come to a figure, so the price is as it was."));
}

#[test]
fn a_markup_out_of_range_is_refused_in_a_whole_sentence() {
    // J22A-19: it read "the job overhead: is a percentage from 0 to 1000".
    let mut m = made();
    let root = {
        let mut q = m.job.world.query::<(&Node, &Kind)>();
        q.iter(&m.job.world).find(|(_, k)| **k == Kind::Job).map(|(n, _)| n.path.clone()).unwrap()
    };
    let why = m.job.set(&root, "overhead", Some("-5"), Status::Assumed, "typed").unwrap_err();
    assert!(why.ends_with("overhead has to be a percentage from 0 to 1000"), "{why}");
}

#[test]
fn a_drawings_scale_is_a_question_with_everything_traced_over_it_riding() {
    // J22A-18, its second part: every traced figure rests on the sheet's scale, which
    // was read off the sheet, and the list did not ask after it. A made-up sheet 800
    // by 600, scaled from a 30 ft dimension; one wall traced over it, 30 ft long and
    // 9 ft tall, with gypsum board on its face at $2.00/SF: 270 SF, $540.00.
    use plumbline_world::plans;
    let mut job = Job::new("Traced (not a real job)", "test", "test");
    let read = Entered { status: Status::Assumed, source: "read off the drawing" };
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", KNOWN).unwrap();
    let drawing = plans::add(&mut job, &ground, "made-up-plan", plans::Read { file: "made-up-plan.pdf", page: 1, bytes: 1000, checksum: "0123456789abcdef", size: [800.0, 600.0] }, KNOWN).unwrap();
    plans::calibrate(&mut job, &drawing, [100.0, 100.0], [700.0, 100.0], "30 ft", read).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let traced = plans::traced_from(&mut job, &drawing).unwrap();
    let wall = ed::walls(&mut job, &ground, &wall_type, &[[0.0, 0.0], [30.0, 0.0]], false, Top::Height("9 ft"), Entered { status: Status::Assumed, source: &traced }).unwrap().remove(0);
    job.add_line(&wall, "Gypsum board, hung and finished", "face_area", Some(Class::Material), false).unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Gypsum board, hung and finished"), Some("$2.00/SF"), Status::Known, "made-up plan").unwrap();
    job.derive();
    let found = rows(&mut job);
    assert_eq!(found[0], (drawing.clone(), None, Some(540.0), 1, vec!["scale dimension".to_string()]), "{found:#?}");
    // A length is not what settles it, and the refusal says what does.
    let why = ed::measure(&mut job, &drawing, "scale dimension", Reading::Length("30 ft"), None, "tape").unwrap_err();
    assert!(why.contains("tape that dimension on site") && why.contains("say \"same\""), "{why}");
    ed::measure(&mut job, &drawing, "scale dimension", Reading::Same, None, "taped on site").unwrap();
    let found = rows(&mut job);
    assert!(!found.iter().any(|r| r.0 == drawing), "{found:#?}");
    // The wall is still as traced: its own line and height are still to be found.
    assert_eq!(found[0].0, wall);
    assert_eq!(found[0].2, Some(540.0));
}

#[test]
fn a_roof_drawn_from_its_footprint_is_deleted_with_its_planes_and_says_what_was_priced_on_them() {
    // Seen in the window on 2026-11-16: the roof could not be deleted (it "has
    // elements under it") and its planes could not either (they "would come back").
    let (mut job, roof, plane) = roofed();
    assert!(ed::remove(&mut job, &plane).unwrap_err().contains("is solved from"));
    let before = view::estimate(&mut job);
    let gone = ed::remove(&mut job, &roof).unwrap();
    // 4 planes, 4 eaves, 4 hips and a ridge, and the roof last.
    assert_eq!((gone.len(), gone.last()), (14, Some(&roof)), "{gone:?}");
    assert!(job.find(&plane).is_none());
    assert_eq!(view::went_with_said(&before, &mut job).as_deref(), Some("1 cost line went with it: $71.55 is out of the job cost."));
    assert!(vif(&mut job).questions.is_empty());
    assert!(file::load(&file::save(&mut job)).is_ok());
}

#[test]
fn a_pitch_set_or_read_at_a_solved_plane_is_put_on_its_roof_and_the_file_still_checks() {
    // J23-1: `set <plane> pitch` was taken, said to be Known, and thrown away at the
    // next solve, leaving a file that failed its own check. J23-2: a pitch read on site
    // was refused as "not one length".
    let (mut job, roof, plane) = roofed();
    job.set(&plane, "pitch", Some("8:12"), Status::Known, "pitch gauge").unwrap();
    job.derive();
    let edge = |job: &mut Job| {
        let e = job.find(&roof).unwrap();
        job.world.get::<Footprint>(e).unwrap().edges[0].clone()
    };
    assert_eq!((edge(&mut job).value, edge(&mut job).status), (Some(Rise::Pitch(8.0)), Status::Known));
    // The plane on the 24 ft eave between 6:12 hips: its ridge is where 8:12 from the
    // south meets 6:12 from the north, 16 x 6 / 14 = 6.8571 ft in. It reads back the same.
    let text = file::save(&mut job);
    let (mut again, report) = file::load(&text).ok().expect("it reads back");
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(file::save(&mut again), text);
    let pitch_of = |job: &mut Job| {
        let e = job.find(&plane).unwrap();
        job.world.get::<Pitch>(e).unwrap().0.clone()
    };
    assert_eq!((pitch_of(&mut again).value, pitch_of(&mut again).status), (Some(8.0), Status::Known));
    // Read on site at the roof's edge, and at the plane as its pitch.
    let done = ed::measure(&mut job, &roof, "edge 2", Reading::Length("7:12"), None, "pitch gauge").unwrap();
    assert!(done.said.contains("edge 2 is 7:12, measured (it was 6:12, Assumed"), "{}", done.said);
    let done = ed::measure(&mut job, &plane, "pitch", Reading::Length("9:12"), None, "pitch gauge").unwrap();
    assert!(done.said.contains("pitch is 9:12, measured (it was 8:12, Known"), "{}", done.said);
    assert_eq!(edge(&mut job).value, Some(Rise::Pitch(9.0)));
    // What is not a pitch is refused, and says how one is typed.
    let why = ed::measure(&mut job, &roof, "edge 3", Reading::Length("steep"), None, "pitch gauge").unwrap_err();
    assert!(why.contains("type it like 7:12"), "{why}");
    assert!(file::load(&file::save(&mut job)).is_ok());
}

#[test]
fn an_eave_at_the_jobs_zero_is_remarked_on_and_a_second_check_is_not_as_drawn() {
    // J23-4.
    let (mut job, roof, _) = roofed();
    let done = ed::measure(&mut job, &roof, "eave height", Reading::Length("0 ft"), None, "laser").unwrap();
    assert!(done.also.iter().any(|a| a.contains("at or below the job's zero")), "{:?}", done.also);
    let done = ed::measure(&mut job, &roof, "eave height", Reading::Same, None, "laser").unwrap();
    assert!(done.said.ends_with("found the same again"), "{}", done.said);
    let done = ed::measure(&mut job, &roof, "eave height", Reading::Length("9 ft"), None, "laser").unwrap();
    assert!(!done.also.iter().any(|a| a.contains("job's zero")), "{:?}", done.also);
}

#[test]
fn lines_that_come_off_or_go_with_a_dropped_plane_are_said_to_the_cent() {
    // J23-3. The plane on edge 1 carries underlayment, $71.55. Its edge made a gable,
    // the plane is gone and so is the line.
    let (mut job, roof, _) = roofed();
    let before = view::estimate(&mut job);
    solve::edit(&mut job, &roof, solve::Change::Rise { edge: 1, rise: value::Val { value: Some(Rise::Gable), unit: String::new(), status: Status::Assumed, source: "the plans".into() } }).unwrap();
    job.derive();
    assert_eq!(view::went_with_said(&before, &mut job).as_deref(), Some("1 cost line went with it: $71.55 is out of the job cost."));
    // A line taken off a thing that stays.
    let mut m = made();
    let before = view::estimate(&mut m.job);
    let e = m.job.find(&m.wall).unwrap();
    m.job.world.entity_mut(e).remove::<Cost>();
    m.job.derive();
    assert_eq!(view::came_off_said(&before, &mut m.job).as_deref(), Some("1 cost line came off: $518.40 is out of the job cost."));
    // Overhead typed ten times too large is answered; an ordinary one is not.
    assert_eq!(view::markup_large_said("250").as_deref(), Some("That is more than what it is a percentage of. If 25.00 % was meant, enter it again."));
    assert_eq!(view::markup_large_said("12"), None);
}

#[test]
fn a_dropped_plane_is_not_the_whole_of_what_a_roof_change_does_to_the_job_cost() {
    // The plane on edge 1 (a 24 ft eave) carries underlayment at $0.50/SF: 143.1084 SF,
    // $71.55. The plane on edge 2 (a 16 ft eave) is given the same: 71.5542 SF, $35.78.
    // Job cost $107.33. Edge 2 made a gable: its plane goes, $35.78 with it; the plane
    // on edge 1 now runs to the gable, (24 + 16) / 2 x 8 = 160 SF on plan, 178.8854 SF
    // on the slope, $89.44. The job cost was $107.33 and is $89.44, not $71.55.
    let (mut job, roof, _) = roofed();
    let east = {
        let mut q = job.world.query::<(&Node, &Solved)>();
        q.iter(&job.world).find(|(_, s)| s.edge == Some(2)).map(|(n, _)| n.path.clone()).unwrap()
    };
    job.add_line(&east, "Underlayment", "surface_area", Some(Class::Material), false).unwrap();
    job.set_on_line(&east, "unit_cost", Some("Underlayment"), Some("$0.50/SF"), Status::Known, "made-up plan").unwrap();
    job.derive();
    let before = view::estimate(&mut job);
    assert_eq!(before.job_cost.value, Some(107.33));
    solve::edit(&mut job, &roof, solve::Change::Rise { edge: 2, rise: value::Val { value: Some(Rise::Gable), unit: String::new(), status: Status::Assumed, source: "the plans".into() } }).unwrap();
    job.derive();
    assert_eq!(view::went_with_said(&before, &mut job).as_deref(), Some("1 cost line went with it: $35.78 is out of the job cost."));
    assert_eq!(view::job_cost_was_and_is_said(&before, &mut job).as_deref(), Some("The planes that stay changed with it: the job cost was $107.33 and is $89.44."));
}
