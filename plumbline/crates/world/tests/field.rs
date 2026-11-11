//! Field entry: what a laser types read as a length, and a measurement taken on site put
//! on the job. Every figure here is made up, and worked by hand before it is compared.

use plumbline_world::building::clash::clashes;
use plumbline_world::editing::{self as ed, Entered, Reading, Top};
use plumbline_world::file::save;
use plumbline_world::units::parse_length;
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };
const TRACED: Entered = Entered { status: Status::Assumed, source: "scaled off a made-up plan" };

fn feet(typed: &str) -> f64 {
    let (len, unit) = parse_length(typed).unwrap_or_else(|e| panic!("{typed:?} refused: {e}"));
    len.feet(unit).unwrap()
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1.0e-9
}

/// A ground storey and an upper one, both scaled off a plan; four walls to the upper
/// floor; a 12 in by 8 in duct at 8 ft, traced; two 2 in pipes across it, known.
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
    let mut job = Job::new("Measured in the field (not a real job)", "test", "test");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", TRACED).unwrap();
    let upper = ed::storey(&mut job, "upper", "9 ft", "8 ft", TRACED).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let duct_type = ed::segment_type(&mut job, "12 x 8", Kind::Duct, "12 in", Some("8 in"), KNOWN).unwrap();
    let pipe_type = ed::segment_type(&mut job, "2 in", Kind::Pipe, "2 in", None, KNOWN).unwrap();
    let walls = ed::walls(&mut job, &ground, &wall_type, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, Top::Level(&upper), KNOWN).unwrap();
    let duct = ed::runs(&mut job, &ground, Kind::Duct, &duct_type, &[[2.0, 8.0], [22.0, 8.0]], "8 ft", None, TRACED).unwrap().remove(0);
    let pipe_a = ed::runs(&mut job, &ground, Kind::Pipe, &pipe_type, &[[12.0, 2.0], [12.0, 14.0]], "7 ft 9 in", None, KNOWN).unwrap().remove(0);
    let pipe_b = ed::runs(&mut job, &ground, Kind::Pipe, &pipe_type, &[[18.0, 2.0], [18.0, 14.0]], "8 ft", None, KNOWN).unwrap().remove(0);
    Made { job, ground, upper, wall: walls[0].clone(), duct, pipe_a, pipe_b }
}

/// The clashes between things: each as (first, second, depth in inches, status).
fn hard(job: &mut Job) -> Vec<(String, String, f64, Status)> {
    clashes(job, None).unwrap().found.into_iter().filter(|c| !c.through).map(|c| (c.a, c.b, (c.by.unwrap_or(f64::NAN) * 12.0 * 100.0).round() / 100.0, c.status)).collect()
}

/// One component of the element at a path, copied out.
fn got<C: bevy_ecs::component::Component + Clone>(job: &mut Job, path: &str) -> C {
    let e = job.find(path).unwrap_or_else(|| panic!("no {path}"));
    job.world.get::<C>(e).unwrap_or_else(|| panic!("{path} has no such component")).clone()
}

fn face(job: &mut Job, wall: &str) -> Derived {
    let e = job.find(wall).unwrap();
    job.world.get::<Measure>(e).unwrap().0["face_area"].clone()
}

#[test]
fn what_a_laser_types_is_read() {
    // Leica's text setting: the display as it stands, the fraction after the inch mark.
    assert!(near(feet("2'2\"13/32"), 2.0 + (2.0 + 13.0 / 32.0) / 12.0));
    assert_eq!(parse_length("2'2\"13/32").unwrap(), (Len::FtIn([2.0, 2.40625]), "ft-in"));
    assert_eq!(parse_length("2' 2\" 13/32").unwrap(), (Len::FtIn([2.0, 2.40625]), "ft-in"));
    assert_eq!(parse_length("26\"13/32").unwrap(), (Len::One(26.40625), "in"));
    assert_eq!(parse_length("8'10\"1/2").unwrap(), (Len::FtIn([8.0, 10.5]), "ft-in"));
    assert_eq!(parse_length("0'10\"1/2").unwrap(), (Len::FtIn([0.0, 10.5]), "ft-in"));
    assert_eq!(parse_length("23.46ft").unwrap(), (Len::One(23.46), "ft"));
    assert_eq!(parse_length("281.5in").unwrap(), (Len::One(281.5), "in"));
    // What was read before is read as before.
    assert_eq!(parse_length("25' 6 1/2\"").unwrap(), (Len::FtIn([25.0, 6.5]), "ft-in"));
    assert_eq!(parse_length("19 1/2\"").unwrap(), (Len::One(19.5), "in"));
    // A fraction after a fraction, a fraction after the word, a third: refused.
    for (typed, why) in [
        ("2' 6 1/2\" 1/4", "one or the other"),
        ("6.5\" 1/4", "one or the other"),
        ("2 ft 6 in 1/2", "something after the inches"),
        ("6 in 1/2", "something after the inches"),
        ("2'6\"1/3", "halves, quarters"),
        ("2'6\"1/2 x", "something after the inches"),
        ("2'6\"x", "something after the inches"),
        ("2'11\"3/2", "less than one"),
    ] {
        let e = parse_length(typed).unwrap_err();
        assert!(e.contains(why), "{typed:?}: {e}");
    }
    // 11 63/64 in is under a foot; 12 in after the feet is still refused.
    assert_eq!(parse_length("2'11\"63/64").unwrap(), (Len::FtIn([2.0, 11.984375]), "ft-in"));
    assert!(parse_length("2'12\"1/2").unwrap_err().contains("less than 12"));
    // The spreadsheet setting sends no unit, a laser set to metres sends metres, and one
    // set to a comma sends a comma: each refused, and each says what to do.
    let e = parse_length("7.123").unwrap_err();
    assert!(e.contains("has no unit") && e.contains("never read as feet") && e.contains("laser"), "{e}");
    let e = parse_length("7.123m").unwrap_err();
    assert!(e.contains("metric"), "{e}");
    let e = parse_length("7,123 ft").unwrap_err();
    assert!(e.contains("comma") && e.contains("point"), "{e}");
    let e = parse_length("1,234.5 ft").unwrap_err();
    assert!(e.contains("comma"), "{e}");
}

#[test]
fn before_any_measurement_two_questions_and_a_flagged_wall() {
    let mut m = made();
    // The duct is 8 in deep about 8 ft: 7 ft 8 in to 8 ft 4 in. Pipe A, 2 in about
    // 7 ft 9 in, is 7 ft 8 in to 7 ft 10 in: in it by 2 in. Pipe B, about 8 ft, is wholly
    // inside the duct's depth: 5 in out, up or down.
    assert_eq!(hard(&mut m.job), [(m.duct.clone(), m.pipe_a.clone(), 2.0, Status::Assumed), (m.duct.clone(), m.pipe_b.clone(), 5.0, Status::Assumed)]);
    // 24 ft by 9 ft.
    let f = face(&mut m.job, &m.wall);
    assert_eq!((f.value, f.status), (Some(216.0), Status::Assumed));
}

#[test]
fn a_storey_height_measured_moves_the_floor_above_and_the_walls_follow() {
    let mut m = made();
    // The floor everything is measured from: as drawn.
    let done = ed::measure(&mut m.job, &m.ground, "elevation", Reading::Same, None, "JM on site").unwrap();
    assert_eq!(done.said, "ground: elevation is 0 ft, found as drawn");
    let l = got::<Level>(&mut m.job, &m.ground);
    assert_eq!((l.elevation.status, l.elevation.source.as_str()), (Status::Known, "scaled off a made-up plan; found as drawn (JM on site)"));
    // Floor to floor, as the laser types it: 8 ft 10 1/2 in is 8.875 ft.
    let done = ed::measure(&mut m.job, &m.ground, "storey height", Reading::Length("8'10\"1/2"), None, "JM with the laser").unwrap();
    assert_eq!(done.said, "ground: storey height is 8' 10 1/2\", measured (it was 9 ft, Assumed: scaled off a made-up plan)");
    assert_eq!(done.also, ["upper's floor is now at 8' 10 1/2\", Known; it was at 9' 0\""]);
    let l = got::<Level>(&mut m.job, &m.ground);
    assert_eq!((l.height.feet(), l.height.status, l.height.source.as_str()), (Some(8.875), Status::Known, "JM with the laser"));
    let up = got::<Level>(&mut m.job, &m.upper);
    assert_eq!((up.elevation.feet(), up.elevation.status), (Some(8.875), Status::Known));
    assert_eq!(up.elevation.source, "ground's floor and its storey height (JM with the laser)");
    // The upper storey's own height was not measured and is as it was.
    assert_eq!((up.height.feet(), up.height.status), (Some(8.0), Status::Assumed));
    // 24 ft by 8.875 ft, and nothing it rests on is assumed any more.
    let f = face(&mut m.job, &m.wall);
    assert_eq!((f.value, f.status), (Some(213.0), Status::Known));
    // Saved and read back, the same.
    let text = save(&mut m.job);
    let (mut back, _) = file::load(&text).unwrap();
    assert_eq!(save(&mut back), text);
}

#[test]
fn a_floor_only_as_sure_as_the_floor_below_it() {
    let mut m = made();
    // The ground floor itself still Assumed: the floor above is where the measurement
    // puts it, and no surer than the floor it was measured from.
    let done = ed::measure(&mut m.job, &m.ground, "storey height", Reading::Length("8' 10 1/2\""), None, "JM").unwrap();
    assert_eq!(done.also, ["upper's floor is now at 8' 10 1/2\", Assumed; it was at 9' 0\""]);
    let f = face(&mut m.job, &m.wall);
    assert_eq!((f.value, f.status), (Some(213.0), Status::Assumed));
}

#[test]
fn a_measurement_against_a_known_floor_above_is_refused_and_says_which_three() {
    let mut m = made();
    ed::measure(&mut m.job, &m.upper, "elevation", Reading::Same, None, "the surveyor").unwrap();
    let before = save(&mut m.job);
    let e = ed::measure(&mut m.job, &m.ground, "storey height", Reading::Length("8' 10 1/2\""), None, "JM").unwrap_err();
    assert!(e.contains("ground's floor is at 0' 0\"") && e.contains("upper's floor is Known at 9' 0\"") && e.contains("this measurement says 8' 10 1/2\"") && e.contains("does not pick"), "{e}");
    assert_eq!(save(&mut m.job), before);
    // The same measurement that agrees with the known floor is taken.
    let done = ed::measure(&mut m.job, &m.ground, "storey height", Reading::Length("108 in"), None, "JM").unwrap();
    assert!(done.also.is_empty(), "{:?}", done.also);
}

#[test]
fn floors_further_up_move_by_the_same_and_stay_as_sure_as_they_were() {
    let mut m = made();
    let attic = ed::storey(&mut m.job, "attic", "17 ft", "7 ft", TRACED).unwrap();
    let roof = ed::storey(&mut m.job, "roof deck", "24 ft", "1 ft", KNOWN).unwrap();
    let above = ed::storey(&mut m.job, "above the deck", "30 ft", "1 ft", TRACED).unwrap();
    let done = ed::measure(&mut m.job, &m.ground, "storey height", Reading::Length("9' 3\""), None, "JM").unwrap();
    assert_eq!(
        done.also,
        [
            "upper's floor is now at 9' 3\", Assumed; it was at 9' 0\"",
            "attic's floor is now at 17' 3\"; it was at 17' 0\"",
            "roof deck's floor is Known at 24' 0\" and was left there, with every floor above it",
        ]
    );
    let z = |job: &mut Job, p: &str| got::<Level>(job, p).elevation;
    let a = z(&mut m.job, &attic);
    assert_eq!((a.feet(), a.status, a.source.as_str()), (Some(17.25), Status::Assumed, "scaled off a made-up plan; moved up 0' 3\" with the floor below (JM)"));
    assert_eq!(z(&mut m.job, &roof).feet(), Some(24.0));
    assert_eq!(z(&mut m.job, &above).feet(), Some(30.0));
    // A measurement that would push an assumed floor through a known one above it.
    let before = save(&mut m.job);
    let e = ed::measure(&mut m.job, &m.ground, "storey height", Reading::Length("17 ft"), None, "JM").unwrap_err();
    assert!(e.contains("roof deck's floor is Known at 24' 0\""), "{e}");
    assert_eq!(save(&mut m.job), before);
}

#[test]
fn a_duct_shot_to_its_underside_clears_one_clash_and_makes_the_other_a_finding() {
    let mut m = made();
    ed::measure(&mut m.job, &m.ground, "elevation", Reading::Same, None, "JM").unwrap();
    for what in ["line on the plan", "type"] {
        ed::measure(&mut m.job, &m.duct, what, Reading::Same, None, "JM").unwrap();
    }
    // Still two questions: the duct's height is the traced one.
    assert_eq!(hard(&mut m.job).iter().map(|c| c.3).collect::<Vec<_>>(), [Status::Assumed, Status::Assumed]);
    let done = ed::measure(&mut m.job, &m.duct, "height above its storey", Reading::Length("7'10\"3/4"), Some(Justify::Bottom), "JM with the laser").unwrap();
    assert_eq!(done.said, "duct 11: height above its storey is 7' 10 3/4\", measured (it was 8 ft, Assumed: scaled off a made-up plan)");
    assert_eq!(done.also, ["duct 11's height is now held to its bottom (it was held to its centre)"]);
    // The duct is now 7 ft 10 3/4 in to 8 ft 6 3/4 in. Pipe A's top is at 7 ft 10 in:
    // 3/4 in clear. Pipe B, 7 ft 11 in to 8 ft 1 in, is still inside: 2 1/4 in down to
    // the duct's underside, 7 3/4 in up. Everything it rests on is Known.
    assert_eq!(hard(&mut m.job), [(m.duct.clone(), m.pipe_b.clone(), 2.25, Status::Known)]);
    // With a clearance asked for, pipe A is 3/4 in off the duct.
    let near_by: Vec<_> = clashes(&mut m.job, Some(2.0 / 12.0)).unwrap().found.into_iter().filter(|c| c.b == m.pipe_a).collect();
    assert_eq!(near_by.len(), 1);
}

#[test]
fn what_a_measurement_will_not_do() {
    let mut m = made();
    let before = save(&mut m.job);
    let refused = |job: &mut Job, path: &str, what: &str, reading: Reading, to: Option<Justify>, source: &str, why: &str| {
        let e = ed::measure(job, path, what, reading, to, source).unwrap_err();
        assert!(e.contains(why), "{what}: {e}");
        assert_eq!(save(job), before, "{what}: a refusal changed the job");
    };
    let (ground, wall, duct) = (m.ground.clone(), m.wall.clone(), m.duct.clone());
    // No unit, a comma, metres, less than nothing.
    refused(&mut m.job, &ground, "storey height", Reading::Length("9"), None, "JM", "has no unit");
    refused(&mut m.job, &ground, "storey height", Reading::Length("8,875 ft"), None, "JM", "comma");
    refused(&mut m.job, &ground, "storey height", Reading::Length("2.7 m"), None, "JM", "metric");
    refused(&mut m.job, &ground, "storey height", Reading::Length("-9 ft"), None, "JM", "storey height");
    // Nobody measured it.
    refused(&mut m.job, &ground, "storey height", Reading::Length("9 ft"), None, " ", "who measured");
    // A line on the plan is not one length; a value that is not there.
    refused(&mut m.job, &wall, "wall line", Reading::Length("24 ft"), None, "JM", "not one length");
    refused(&mut m.job, &wall, "length", Reading::Length("24 ft"), None, "JM", "it has: type, wall line, top level");
    refused(&mut m.job, "wall-9999", "height", Reading::Length("9 ft"), None, "JM", "no element");
    // Only a run's height is taken to its bottom or its top.
    refused(&mut m.job, &wall, "wall line", Reading::Same, Some(Justify::Top), "JM", "not a run");
    refused(&mut m.job, &duct, "line on the plan", Reading::Same, Some(Justify::Bottom), "JM", "only duct 11's height");
    refused(&mut m.job, &duct, "height above its storey", Reading::Same, Some(Justify::Bottom), "JM", "leave the line out");
}

#[test]
fn same_on_a_value_nobody_knows_and_a_sloping_run() {
    let mut m = made();
    // The storey's height marked unknown by hand: there is nothing for it to be the same as.
    let mut level = got::<Level>(&mut m.job, &m.ground);
    level.height = Val { value: None, unit: "ft".into(), status: Status::Unknown, source: "nobody has said".into() };
    m.job.put(&m.ground, level).unwrap();
    let before = save(&mut m.job);
    let why = ed::measure(&mut m.job, &m.ground, "storey height", Reading::Same, None, "JM").unwrap_err();
    assert!(why.contains("nothing for it to be the same as"), "{why}");
    assert_eq!(save(&mut m.job), before);
    // Measured, it is Known, and the floor above follows.
    let done = ed::measure(&mut m.job, &m.ground, "storey height", Reading::Length("9 ft"), None, "JM").unwrap();
    assert!(done.said.contains("it was not known: nobody has said"), "{}", done.said);
    // A run whose ends differ, held to its centre: a shot to its underside at one end is
    // refused; a shot to its centre is taken, at either end.
    let mut run = got::<Segment>(&mut m.job, &m.duct);
    run.end_offset = Some(Val { value: Some(Len::One(8.5)), unit: "ft".into(), status: Status::Assumed, source: "scaled off a made-up plan".into() });
    m.job.put(&m.duct, run).unwrap();
    m.job.derive();
    let before = save(&mut m.job);
    let why = ed::measure(&mut m.job, &m.duct, "height above its storey, second end", Reading::Length("8 ft"), Some(Justify::Bottom), "JM").unwrap_err();
    assert!(why.contains("slopes") && why.contains("held to its centre"), "{why}");
    assert_eq!(save(&mut m.job), before);
    // The name a level run's height goes by is not this run's any more.
    assert!(ed::measure(&mut m.job, &m.duct, "height above its storey", Reading::Length("8 ft"), None, "JM").unwrap_err().contains("it has:"));
    ed::measure(&mut m.job, &m.duct, "height above its storey, second end", Reading::Length("8' 3\""), Some(Justify::Centre), "JM").unwrap();
    let run = got::<Segment>(&mut m.job, &m.duct);
    assert_eq!((run.offsets_ft(), run.end_offset.unwrap().status, run.offset.status), (Some((8.0, 8.25)), Status::Known, Status::Assumed));
}

#[test]
fn a_run_with_no_size_cannot_be_shot_to_its_underside() {
    let mut m = made();
    // The pipe's type loses its size.
    let pipe = got::<Segment>(&mut m.job, &m.pipe_a);
    let t = pipe.type_name.value.clone().unwrap();
    let mut ty = got::<SegmentType>(&mut m.job, &t);
    ty.width = Val { value: None, unit: "in".into(), status: Status::Unknown, source: "not read off the plan".into() };
    m.job.put(&t, ty).unwrap();
    m.job.derive();
    let before = save(&mut m.job);
    let why = ed::measure(&mut m.job, &m.pipe_a, "height above its storey", Reading::Length("7 ft"), Some(Justify::Bottom), "JM").unwrap_err();
    assert!(why.contains("size is not known"), "{why}");
    assert_eq!(save(&mut m.job), before);
    // To its centre, it is taken.
    ed::measure(&mut m.job, &m.pipe_a, "height above its storey", Reading::Length("7 ft"), Some(Justify::Centre), "JM").unwrap();
    // The size itself measured: it is a length like any other.
    let done = ed::measure(&mut m.job, &t, "diameter", Reading::Length("2\"1/8"), None, "JM with the calipers").unwrap();
    assert_eq!(done.said, "2 in: diameter is 2 1/8 in, measured (it was not known: not read off the plan)");
}
