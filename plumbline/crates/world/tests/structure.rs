//! A beam, its posts and their pad footings (call 164). Every figure is worked by hand in
//! the comments and in STATUS.md ("A beam, its posts and their pad footings").

use plumbline_world::building::{self, clash, Shape};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::file::{load, save};
use plumbline_world::sample::house;
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up survey" };
const ASSUMED: Entered = Entered { status: Status::Assumed, source: "drawn size, not an engineer's" };

fn figure(job: &mut Job, path: &str, name: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).unwrap().0[name].clone()
}

/// The house of the demonstration: a level 9 ft high, and a beam type, a post type and a pad type.
struct Yard {
    job: Job,
    level: String,
    beam: String,
    post: String,
    pad: String,
}

fn yard() -> Yard {
    let mut job = Job::new("Structure (not a real job)", "test", "2026-11-22");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", KNOWN).unwrap();
    let beam = ed::framing_type(&mut job, "LVL 3 1/2 x 11 7/8", Kind::Beam, "3.5 in", Some("11.875 in"), None, ASSUMED).unwrap();
    let post = ed::framing_type(&mut job, "6x6 post", Kind::Post, "5.5 in", Some("5.5 in"), None, ASSUMED).unwrap();
    let pad = ed::framing_type(&mut job, "Pad 36 x 36", Kind::PadFooting, "3 ft", Some("3 ft"), None, ASSUMED).unwrap();
    Yard { job, level, beam, post, pad }
}

#[test]
fn a_beam_is_its_length_and_weighs_what_its_type_says_a_foot_does() {
    let mut y = yard();
    // 16 LF; its type gives no weight per foot, and the figure says why.
    let beam = ed::beams(&mut y.job, &y.level, &y.beam, &[[12.0, 0.0], [12.0, 16.0]], "9 ft", KNOWN).unwrap().remove(0);
    let length = figure(&mut y.job, &beam, "length");
    assert_eq!((length.value, length.unit.as_str(), length.status), (Some(16.0), "LF", Status::Known));
    let weight = figure(&mut y.job, &beam, "weight");
    assert_eq!(weight.value, None);
    assert!(weight.note.contains("its type gives no weight per foot"), "{}", weight.note);
    // 3 1/2 in by 11 7/8 in: 3.5/12 = 0.291667 ft, 11.875/12 = 0.989583 ft; Assumed with the type.
    let (w, d) = (figure(&mut y.job, &beam, "width"), figure(&mut y.job, &beam, "depth"));
    assert!((w.value.unwrap() - 0.2917).abs() < 1.0e-4 && (d.value.unwrap() - 0.9896).abs() < 1.0e-4);
    assert_eq!((w.status, d.status), (Status::Assumed, Status::Assumed));
    // Its section is 3.5 x 11.875 = 41.5625 in2 = 0.288628 SF; its volume 16 x that / 27 = 0.171039 CY.
    let area = figure(&mut y.job, &beam, "section_area");
    assert!((area.value.unwrap() - 0.2886).abs() < 1.0e-4, "{:?}", area.value);
    let volume = figure(&mut y.job, &beam, "volume");
    assert!((volume.value.unwrap() - 0.1710).abs() < 1.0e-4, "{:?}", volume.value);
    // W8x18: 16 x 18 = 288 LB.
    let steel = ed::framing_type(&mut y.job, "W8x18", Kind::Beam, "5.25 in", Some("8.14 in"), Some("18 lb/ft"), ASSUMED).unwrap();
    y.job.set_on_line(&beam, "type", None, Some(&steel), Status::Known, "made-up survey").unwrap();
    y.job.derive();
    let weight = figure(&mut y.job, &beam, "weight");
    assert_eq!((weight.value, weight.unit.as_str(), weight.status), (Some(288.0), "LB", Status::Assumed));
    // A beam is changed to a type of a beam and to no other.
    let why = y.job.set_on_line(&beam, "type", None, Some(&y.post), Status::Known, "made-up survey").unwrap_err();
    assert!(why.contains("is not a type of beam"), "{why}");
}

#[test]
fn a_beam_that_does_not_run_along_is_refused_in_the_three_ways() {
    let mut y = yard();
    let before = save(&mut y.job);
    // The very same point twice: it stands, and what stands is a post.
    let why = ed::beams(&mut y.job, &y.level, &y.beam, &[[12.0, 0.0], [12.0, 0.0]], "9 ft", KNOWN).unwrap_err();
    assert!(why.contains("a beam runs along, and what stands is a post"), "{why}");
    // 0.01 ft long: too short.
    let why = ed::beams(&mut y.job, &y.level, &y.beam, &[[12.0, 0.0], [12.0, 0.01]], "9 ft", KNOWN).unwrap_err();
    assert!(why.contains("is 0.01 ft long; a run goes from one point to another"), "{why}");
    assert_eq!(save(&mut y.job), before);
    // A beam on a type of a post is refused; and a beam of a type nobody made.
    let why = ed::beams(&mut y.job, &y.level, &y.post, &[[0.0, 0.0], [10.0, 0.0]], "9 ft", KNOWN).unwrap_err();
    assert!(why.contains("is a beam, but its type"), "{why}");
    // Sloped past 75 degrees: a rise of 40 ft over 10 ft is 76 degrees. Said by the model's own check.
    let steep = Segment { type_name: Val::known(y.beam.clone(), "", "test"), line: Val::known([[0.0, 0.0], [10.0, 0.0]], "ft", "test"), offset: Val::known(Len::One(0.0), "ft", "test"), end_offset: Some(Val::known(Len::One(40.0), "ft", "test")), justify: Justify::Top };
    let why = building::check_segment("beam-0001", Kind::Beam, &steep).unwrap_err();
    assert!(why.contains("slopes 76.0 degrees, more than the 75 degrees a beam may"), "{why}");
    // A weight per foot is a beam's, and is typed with its unit.
    assert!(ed::framing_type(&mut y.job, "W", Kind::Beam, "5 in", Some("8 in"), Some("18"), ASSUMED).unwrap_err().contains("no unit"));
    assert!(ed::framing_type(&mut y.job, "P", Kind::Post, "5 in", Some("5 in"), Some("18 lb/ft"), ASSUMED).unwrap_err().contains("a beam's"));
    assert!(ed::framing_type(&mut y.job, "R", Kind::PadFooting, "36 in", None, None, ASSUMED).unwrap_err().contains("square or rectangular"));
}

#[test]
fn a_post_stands_to_what_it_is_told_and_to_its_levels_height_when_it_is_not_told() {
    let mut y = yard();
    let post = ed::post(&mut y.job, &y.level, &y.post, [12.0, 0.0], Some("8 ft"), None, KNOWN).unwrap();
    let h = figure(&mut y.job, &post, "height");
    assert_eq!((h.value, h.unit.as_str(), h.status), (Some(8.0), "LF", Status::Known));
    // 5.5 in = 0.458333 ft each way.
    assert!((figure(&mut y.job, &post, "width").value.unwrap() - 0.4583).abs() < 1.0e-4);
    // No height said: the level's, 9 ft, Assumed, and the value says so.
    let second = ed::post(&mut y.job, &y.level, &y.post, [12.0, 16.0], None, None, KNOWN).unwrap();
    let e = y.job.find(&second).unwrap();
    let c = y.job.world.get::<Stood>(e).unwrap();
    assert_eq!(c.height.status, Status::Assumed);
    assert!(c.height.source.starts_with("stands to its level's height"), "{}", c.height.source);
    assert_eq!(figure(&mut y.job, &second, "height").value, Some(9.0));
    // A post 0 ft tall: its top is above its base.
    let why = ed::post(&mut y.job, &y.level, &y.post, [1.0, 1.0], Some("0 ft"), None, KNOWN).unwrap_err();
    assert!(why.contains("a post's top is above its base"), "{why}");
}

#[test]
fn a_pad_footing_is_a_post_laid_flat_and_its_concrete_and_forms_are_worked_from_its_sides() {
    let mut y = yard();
    // 3 ft by 3 ft, 18 in thick, its top 1 ft below the floor: base -2.5 ft.
    let pad = ed::pad(&mut y.job, &y.level, &y.pad, [12.0, 0.0], "18 in", "-1 ft", KNOWN).unwrap();
    assert_eq!(figure(&mut y.job, &pad, "plan_area").value, Some(9.0));
    assert_eq!(figure(&mut y.job, &pad, "form_area").value, Some(18.0));
    let volume = figure(&mut y.job, &pad, "volume");
    // 3 x 3 x 1.5 = 13.5 CF = 0.5 CY.
    assert_eq!((volume.value, volume.unit.as_str(), volume.status), (Some(0.5), "CY", Status::Assumed));
    let e = y.job.find(&pad).unwrap();
    assert_eq!(y.job.world.get::<Stood>(e).unwrap().base_offset.feet(), Some(-2.5));
    // Drawn as a box standing on its base, 2.5 ft under the floor.
    let (drawn, _) = building::placements(&mut y.job);
    let shape = &drawn.iter().find(|p| p.path == pad).unwrap().shape;
    match shape {
        Shape::Box { at, size, .. } => assert_eq!((at[2], size[2]), (-2.5, 1.5)),
        other => panic!("{other:?}"),
    }
    // No thickness, no pad.
    let why = ed::pad(&mut y.job, &y.level, &y.pad, [0.0, 0.0], "0 ft", "-1 ft", KNOWN).unwrap_err();
    assert!(why.contains("a pad footing has a thickness, and nobody gave one"), "{why}");
}

#[test]
fn a_post_is_drawn_as_a_vertical_run_and_a_beam_as_a_run_held_to_its_top() {
    let mut y = yard();
    let beam = ed::beams(&mut y.job, &y.level, &y.beam, &[[12.0, 0.0], [12.0, 16.0]], "9 ft", KNOWN).unwrap().remove(0);
    let post = ed::post(&mut y.job, &y.level, &y.post, [12.0, 0.0], Some("8 ft"), None, KNOWN).unwrap();
    let (drawn, left) = building::placements(&mut y.job);
    assert!(left.is_empty(), "{left:?}");
    match &drawn.iter().find(|p| p.path == beam).unwrap().shape {
        // Its top at 9 ft, its section 11 7/8 in deep: its centre line is 5 15/16 in under.
        Shape::Run { a, b, .. } => assert!((a[2] - (9.0 - 11.875 / 24.0)).abs() < 1.0e-9 && a[2] == b[2]),
        other => panic!("{other:?}"),
    }
    match &drawn.iter().find(|p| p.path == post).unwrap().shape {
        Shape::Run { a, b, rect, .. } => assert_eq!((a[2], b[2], rect.is_some()), (0.0, 8.0, true)),
        other => panic!("{other:?}"),
    }
}

#[test]
fn framing_is_tested_against_what_runs_and_what_is_set_and_not_against_the_building() {
    let mut y = yard();
    let wt = ed::wall_type(&mut y.job, "wall 6 in", "6 in", KNOWN).unwrap();
    let walls = ed::walls(&mut y.job, &y.level, &wt, &[[12.0, 0.0], [12.0, 16.0]], false, ed::Top::Height("9 ft"), KNOWN).unwrap();
    let beam = ed::beams(&mut y.job, &y.level, &y.beam, &[[12.0, 0.0], [12.0, 16.0]], "9 ft", KNOWN).unwrap().remove(0);
    ed::post(&mut y.job, &y.level, &y.post, [12.0, 0.0], Some("8 ft"), None, KNOWN).unwrap();
    ed::pad(&mut y.job, &y.level, &y.pad, [12.0, 0.0], "18 in", "-1 ft", KNOWN).unwrap();
    // The beam lies in the wall's line and the post under its end: nothing clashes.
    let _ = &walls;
    assert!(clash::has_framing(&mut y.job));
    let found = clash::clashes(&mut y.job, None).unwrap();
    assert!(found.found.is_empty(), "{:?}", found.found);
    assert!(found.tested >= 4);
    // A round duct 12 in across, its middle 8.5 ft up, through the beam: a clash with it.
    let dt = ed::segment_type(&mut y.job, "12 in round", Kind::Duct, "12 in", None, KNOWN).unwrap();
    let duct = ed::runs(&mut y.job, &y.level, Kind::Duct, &dt, &[[6.0, 8.0], [18.0, 8.0]], "8.5 ft", None, KNOWN).unwrap().remove(0);
    let found = clash::clashes(&mut y.job, None).unwrap();
    let mut names: Vec<(String, String)> = found.found.iter().filter(|c| c.how == clash::ClashKind::Hard).map(|c| (c.a.clone(), c.b.clone())).collect();
    names.retain(|(a, b)| a == &beam || b == &beam);
    assert_eq!(names.len(), 1, "{:?}", found.found);
    assert!(names[0].0 == duct || names[0].1 == duct);
    // The wall is not one of them: the duct through it is a hole to cut, and nothing else
    // of the wall, the beam, the post and the pad is in anything.
    assert!(!found.found.iter().any(|c| !c.through && (walls.contains(&c.a) || walls.contains(&c.b))), "{:?}", found.found);
    assert_eq!(found.found.iter().filter(|c| c.a_kind.is_framing() || c.b_kind.is_framing()).count(), 1, "{:?}", found.found);
}

#[test]
fn a_job_with_none_of_it_saves_and_opens_as_it_was_and_a_job_with_it_does_too() {
    let mut old = house();
    let text = save(&mut old);
    // Nothing of the new kinds, and no key for them, is in a job without them.
    assert!(!text.contains("pl::column") && !text.contains("weight_per_foot") && !text.contains("\"Beam\"") && !text.contains("PadFooting"));
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut again), text);
    // With them it is saved and opened the same way.
    let mut y = yard();
    ed::beams(&mut y.job, &y.level, &y.beam, &[[12.0, 0.0], [12.0, 16.0]], "9 ft", KNOWN).unwrap();
    ed::post(&mut y.job, &y.level, &y.post, [12.0, 0.0], None, None, KNOWN).unwrap();
    ed::pad(&mut y.job, &y.level, &y.pad, [12.0, 0.0], "18 in", "-1 ft", KNOWN).unwrap();
    let steel = ed::framing_type(&mut y.job, "W8x18", Kind::Beam, "5.25 in", Some("8.14 in"), Some("18 lb/ft"), ASSUMED).unwrap();
    let _ = steel;
    let text = save(&mut y.job);
    assert!(text.contains("pl::column") && text.contains("weight_per_foot"));
    let (mut back, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut back), text);
}

#[test]
fn the_list_for_the_engineer_counts_the_sizes_that_are_not_known() {
    let mut y = yard();
    ed::beams(&mut y.job, &y.level, &y.beam, &[[12.0, 0.0], [12.0, 16.0]], "9 ft", KNOWN).unwrap();
    ed::post(&mut y.job, &y.level, &y.post, [12.0, 0.0], Some("8 ft"), None, KNOWN).unwrap();
    ed::pad(&mut y.job, &y.level, &y.pad, [12.0, 0.0], "18 in", "-1 ft", KNOWN).unwrap();
    let s = plumbline_world::structure::structure(&mut y.job);
    assert_eq!((s.members, s.to_size), (3, 3));
    assert_eq!(s.levels.iter().map(|l| l.members.len()).sum::<usize>(), 3);
    // The three types' sizes found as drawn: every one is Known.
    for t in [&y.beam, &y.post, &y.pad] {
        for what in ["width", "depth"] {
            ed::measure(&mut y.job, t, what, ed::Reading::Same, None, "engineer's letter, made up").unwrap();
        }
    }
    let s = plumbline_world::structure::structure(&mut y.job);
    assert_eq!((s.members, s.to_size), (3, 0));
}
