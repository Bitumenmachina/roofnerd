//! Call 204, points 5 and 6: a beam and a column come in as a beam and a post where the body in the file is a straight
//! box, sized from it, Assumed, sourced to the file; any other is left out with the reason that is true. A fitting with
//! no size is written with its place and no body, and comes back with its size Unknown. Everything is made up. The
//! figures are those of STATUS.md ("A corner board as tall as its wall at that corner: a demonstration", the part on a model).

use plumbline_ifc::{import, write::export};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-01-31T09:30:00";
const MADE: Entered = Entered { status: Status::Known, source: "made-up survey" };
const HOW: Entered = Entered { status: Status::Assumed, source: "a made-up model" };

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1.0e-3
}

/// One level 9 ft high with a beam 12 ft long, 3 1/2 in by 9 1/4 in, its top 8 ft up, and a post 8 ft high, 3 1/2 in square.
fn framed() -> (Job, String, String) {
    let mut job = Job::new("Framing (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", MADE).unwrap();
    let beam_type = ed::framing_type(&mut job, "beam 3 1/2 by 9 1/4", Kind::Beam, "3 1/2 in", Some("9 1/4 in"), None, MADE).unwrap();
    let post_type = ed::framing_type(&mut job, "post 3 1/2 square", Kind::Post, "3 1/2 in", Some("3 1/2 in"), None, MADE).unwrap();
    let beam = ed::beams(&mut job, &level, &beam_type, &[[2.0, 3.0], [14.0, 3.0]], "8 ft", MADE).unwrap().remove(0);
    let post = ed::post(&mut job, &level, &post_type, [5.0, 7.0], Some("8 ft"), None, MADE).unwrap();
    job.derive();
    (job, beam, post)
}

fn segment(job: &mut Job, kind: Kind) -> Vec<(String, Segment)> {
    let mut q = job.world.query::<(&Node, &Kind, &Segment)>();
    q.iter(&job.world).filter(|(_, k, _)| **k == kind).map(|(n, _, s)| (n.path.clone(), s.clone())).collect()
}

fn brought(text: &str) -> (Job, plumbline_ifc::Brought) {
    let mut back = Job::new("brought in (not a real job)", "test", "test");
    let b = import(&mut back, text.as_bytes(), "framing.ifc", HOW).unwrap();
    back.derive();
    (back, b)
}

// By hand: the beam is 12 ft (2 to 14 along y = 3), 3.5 in by 9.25 in, its top 8 ft above the floor; the post stands at (5, 7),
// 8 ft tall on the floor, 3.5 in square. Written, then brought into an empty job: 1 beam and 1 post, those sizes, Assumed (the
// first job's were Known: a model is never Known on its say-so), each saying the file and its number in it.
#[test]
fn a_beam_and_a_column_in_a_file_come_in_as_a_beam_and_a_post_sized_from_their_bodies() {
    let (mut job, ..) = framed();
    let text = export(&mut job, "framing.ifc", WHEN).unwrap().text;
    assert!(text.contains("IFCBEAM(") && text.contains("IFCCOLUMN("));
    let (mut back, b) = brought(&text);
    assert_eq!((b.made.get("beams"), b.made.get("posts")), (Some(&1), Some(&1)), "{:#?}", b.left_out);
    assert!(b.left_out.iter().all(|(c, ..)| c != "beam" && c != "column"), "{:#?}", b.left_out);
    // The beam: its line, its top and its section.
    let beams = segment(&mut back, Kind::Beam);
    assert_eq!(beams.len(), 1);
    let (path, s) = &beams[0];
    let line = s.line_ft().unwrap();
    assert!(near(line[0][0], 2.0) && near(line[0][1], 3.0) && near(line[1][0], 14.0) && near(line[1][1], 3.0), "{line:?}");
    assert!(near(s.offset.feet().unwrap(), 8.0), "its top: {:?}", s.offset);
    assert_eq!(s.offset.status, Status::Assumed);
    assert!(s.line.source.contains("framing.ifc") && s.line.source.contains("beam") && s.line.source.contains("in the file"), "{}", s.line.source);
    let t = {
        let tp = s.type_name.value.clone().unwrap();
        let e = back.find(&tp).unwrap();
        back.world.get::<SegmentType>(e).unwrap().clone()
    };
    let (w, d) = t.rect_ft().unwrap();
    assert!(near(w, 3.5 / 12.0) && near(d, 9.25 / 12.0), "{w} by {d} ft");
    assert_eq!(t.status(), Status::Assumed);
    // Its length, worked out by the model: 12 LF.
    let m = {
        let e = back.find(path).unwrap();
        back.world.get::<Measure>(e).cloned().unwrap()
    };
    assert!(near(m.0["length"].value.unwrap(), 12.0), "{:?}", m.0["length"]);
    // The post: where it stands, how tall, its section.
    let posts: Vec<(String, Stood)> = {
        let mut q = back.world.query::<(&Node, &Kind, &Stood)>();
        q.iter(&back.world).filter(|(_, k, _)| **k == Kind::Post).map(|(n, _, c)| (n.path.clone(), c.clone())).collect()
    };
    assert_eq!(posts.len(), 1);
    let at = posts[0].1.at_ft().unwrap();
    assert!(near(at[0], 5.0) && near(at[1], 7.0) && near(posts[0].1.height.feet().unwrap(), 8.0) && near(posts[0].1.base_offset.feet().unwrap(), 0.0), "{:?}", posts[0].1);
    assert_eq!((posts[0].1.height.status, posts[0].1.at.status), (Status::Assumed, Status::Assumed));
    let pt = {
        let e = back.find(posts[0].1.type_name.value.as_ref().unwrap()).unwrap();
        back.world.get::<SegmentType>(e).unwrap().clone()
    };
    let (w, d) = pt.rect_ft().unwrap();
    assert!(near(w, 3.5 / 12.0) && near(d, 3.5 / 12.0));
    // The same file again counts neither twice.
    let again = import(&mut back, text.as_bytes(), "framing.ifc", HOW).unwrap_err();
    assert_eq!(segment(&mut back, Kind::Beam).len(), 1, "{again}");
    assert!(again.contains("this model is already in the job") && again.contains("counted twice"), "{again}");
}

// A beam that slopes, one round in section, a post turned on the plan, a beam whose section is turned about its length: each left
// out with the reason that is true of it, and none made.
#[test]
fn what_is_not_a_straight_box_is_left_out_with_the_reason_that_is_true() {
    let (mut job, beam, post) = framed();
    // A beam that slopes: its far end 1 ft higher.
    {
        let e = job.find(&beam).unwrap();
        let mut s = job.world.get_mut::<Segment>(e).unwrap();
        s.end_offset = Some(Val { value: Some(Len::One(9.0)), unit: "ft".into(), status: Status::Known, source: "made-up survey".into() });
    }
    let sloped = export(&mut job, "framing.ifc", WHEN).unwrap().text;
    let (back, b) = brought(&sloped);
    assert_eq!(b.made.get("beams"), None);
    assert!(b.left_out.iter().any(|(c, n, why)| c == "beam" && *n == 1 && why.contains("slopes")), "{:#?}", b.left_out);
    assert_eq!(b.made.get("posts"), Some(&1));
    let _ = back;
    // A round beam.
    let mut round = Job::new("Round (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut round, "ground", "0 ft", "9 ft", MADE).unwrap();
    let t = ed::framing_type(&mut round, "pole 8 in", Kind::Beam, "8 in", None, None, MADE).unwrap();
    ed::beams(&mut round, &level, &t, &[[0.0, 0.0], [10.0, 0.0]], "8 ft", MADE).unwrap();
    let text = export(&mut round, "round.ifc", WHEN).unwrap().text;
    // Nothing else in the file is a thing, so it is refused whole, and says why.
    let mut back = Job::new("brought in (not a real job)", "test", "test");
    let why = import(&mut back, text.as_bytes(), "round.ifc", HOW).unwrap_err();
    assert!(why.contains("1 beam (its section is round, not a rectangle"), "{why}");
    assert!(segment(&mut back, Kind::Beam).is_empty());
    let _ = post;
}

// A model that says nothing's sure of itself: said Known, everything comes in Assumed and the notes say so (as for any model).
#[test]
fn a_beam_said_known_comes_in_assumed() {
    let (mut job, ..) = framed();
    let text = export(&mut job, "framing.ifc", WHEN).unwrap().text;
    let mut back = Job::new("brought in (not a real job)", "test", "test");
    import(&mut back, text.as_bytes(), "framing.ifc", Entered { status: Status::Known, source: "a made-up model" }).unwrap();
    for (_, s) in segment(&mut back, Kind::Beam) {
        assert_eq!((s.line.status, s.offset.status), (Status::Assumed, Status::Assumed));
    }
}

// The sample house: its two elbows with no size (between the kitchen's three hot-water pieces with no size) are written with their
// place and no body, say so, and come back with their size Unknown on the fittings that the runs there make. Nothing is said to
// have no shape that has one now.
#[test]
fn a_fitting_with_no_size_is_written_with_its_place_and_comes_back_with_its_size_unknown() {
    let mut job = sample::house();
    let written = export(&mut job, "house.ifc", WHEN).unwrap();
    assert_eq!(written.made.get("fittings"), Some(&41), "{:#?}", written.not_written);
    assert!(written.not_written.iter().all(|n| !n.2.contains("size is not known")), "{:#?}", written.not_written);
    let (mut back, b) = brought(&written.text);
    let unsized_at = |job: &mut Job| -> Vec<(f64, f64, f64)> {
        let mut q = job.world.query::<(&Kind, &Placed)>();
        let mut v: Vec<(f64, f64, f64)> = q.iter(&job.world).filter(|(k, p)| **k == Kind::Fitting && p.size_ft().is_none()).map(|(_, p)| (p.at_ft().unwrap()[0], p.at_ft().unwrap()[1], p.offset.feet().unwrap())).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v
    };
    // The first job's two elbows with no size, and where each stood: each is there again, its size Unknown (a third, the hot main's
    // tee at the water heater, had no size from the runs there before this and still has none).
    let (was, is) = (unsized_at(&mut job), unsized_at(&mut back));
    assert_eq!(was.len(), 2, "{was:?}");
    for w in &was {
        assert!(is.iter().any(|p| near(p.0, w.0) && near(p.1, w.1) && near(p.2, w.2)), "the elbow at {w:?} did not come back: {is:?} {:#?}", b.left_out);
    }
    assert_eq!(is.len(), 3, "{is:?}");
    // 41 written; the two the tool refuses where a third end meets are left out and said, as before: 39 come back.
    assert_eq!(b.made.get("fittings"), Some(&39), "{:#?}", b.left_out);
}
