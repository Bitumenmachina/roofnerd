//! Call 227, the model's side (contract 227M): the measured marks of a marked-up PDF as lines of the estimate, and
//! kept with the drawing. Everything is made up; the figures are worked by hand beside each test.

use plumbline_world::editing::Entered;
use plumbline_world::marks::{self, MarkShape, ToBring};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

fn mark(key: &str, label: &str, q: f64, unit: &'static str, kind: &str) -> ToBring {
    ToBring { key: key.into(), label: label.into(), quantity: q, unit, kind: kind.into(), points_say: None }
}

fn three() -> Vec<ToBring> {
    vec![mark("mk-south", "South wall", 30.0, "LF", "length"), mark("mk-floor", "Whole floor", 600.0, "SF", "area"), mark("count Receptacle", "Receptacle", 4.0, "EA", "count")]
}

fn keys(m: &[ToBring]) -> Vec<String> {
    m.iter().map(|m| m.key.clone()).collect()
}

fn job() -> Job {
    Job::new("Marks (not a real job)", "test", "2026-11-26")
}

#[test]
fn each_mark_is_a_line_of_the_job_assumed_naming_its_mark_unpriced_and_with_no_division() {
    let mut j = job();
    let m = three();
    let got = marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap();
    assert_eq!(got.added, vec![("South wall".to_string(), 30.0, "LF".to_string()), ("Whole floor".to_string(), 600.0, "SF".to_string()), ("Receptacle".to_string(), 4.0, "EA".to_string())]);
    assert!(got.same.is_empty() && got.moved.is_empty() && got.gone.is_empty() && got.left_as_it_is.is_empty());
    let root = j.root();
    let cost = j.world.get::<Cost>(root).unwrap().clone();
    assert_eq!(cost.lines.len(), 3);
    for (line, want) in cost.lines.iter().zip(["mark \"mk-south\" (length), page 1 of marked-plan.pdf", "mark \"mk-floor\" (area), page 1 of marked-plan.pdf", "4 symbols labelled Receptacle counted on page 1 of marked-plan.pdf"]) {
        let figure = &line.inputs["measured"];
        assert_eq!(figure.status, Status::Assumed, "{}", line.description);
        assert_eq!(figure.source, want);
        // No price, no division, no cost type: not priced, said so, in no total.
        assert_eq!(line.unit_cost.value, None, "{}", line.description);
        assert_eq!((line.division, line.class), (None, None), "{}", line.description);
    }
    // The line's quantity is the figure the mark says, in its unit.
    let q = view::estimate(&mut j);
    let south = q.lines.iter().find(|l| l.description == "South wall").unwrap();
    assert_eq!((south.quantity.value, south.quantity.unit.as_str(), south.quantity.status), (Some(30.0), "LF", Status::Assumed));
    assert_eq!(south.amount.value, None);
    // Nothing is priced: the job's price is what it was, nothing.
    assert_eq!(q.job_cost.value, None);
}

#[test]
fn a_mark_is_known_again_by_its_name_in_the_file_and_is_not_added_twice() {
    let mut j = job();
    let m = three();
    marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap();
    let again = marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap();
    assert!(again.added.is_empty() && again.moved.is_empty() && again.gone.is_empty());
    assert_eq!(again.same, ["South wall", "Whole floor", "Receptacle"]);
    assert_eq!(j.world.get::<Cost>(j.root()).unwrap().lines.len(), 3);
    // Another page of the same file, or another file, is another mark: the same key is not the same mark.
    let p2 = marks::bring_in(&mut j, "marked-plan.pdf", 2, &m[..1], &keys(&m[..1])).unwrap();
    assert_eq!(p2.added.len(), 1);
    assert_eq!(p2.added[0].0, "South wall (marked-plan, page 2)", "the line of that name is taken: it is said which file and page");
    let other = marks::bring_in(&mut j, "other.pdf", 1, &m[..1], &keys(&m[..1])).unwrap();
    assert_eq!(other.added[0].0, "South wall (other, page 1)");
}

#[test]
fn a_mark_whose_figure_changed_moves_its_line_and_says_what_it_was() {
    let mut j = job();
    let m = three();
    marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap();
    let mut changed = three();
    changed[1].quantity = 650.0;
    let got = marks::bring_in(&mut j, "marked-plan.pdf", 1, &changed, &keys(&changed)).unwrap();
    assert_eq!(got.moved, vec![("Whole floor".to_string(), 600.0, 650.0, "SF".to_string())]);
    assert_eq!(got.same, ["South wall", "Receptacle"]);
    let q = view::estimate(&mut j);
    assert_eq!(q.lines.iter().find(|l| l.description == "Whole floor").unwrap().quantity.value, Some(650.0));
    assert_eq!(j.world.get::<Cost>(j.root()).unwrap().lines.len(), 3, "moved, not added");
}

#[test]
fn a_figure_somebody_made_known_in_the_field_is_not_overwritten_and_is_said() {
    let mut j = job();
    let m = three();
    marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap();
    // The field measurement: the line's figure is made Known.
    j.set_on_line(&j.world.get::<Node>(j.root()).unwrap().path.clone(), "measured", Some("Whole floor"), Some("598 SF"), Status::Known, "measured in the field").unwrap();
    let mut changed = three();
    changed[1].quantity = 650.0;
    let got = marks::bring_in(&mut j, "marked-plan.pdf", 1, &changed, &keys(&changed)).unwrap();
    assert!(got.moved.is_empty(), "{:?} {:?}", got.moved, j.world.get::<Cost>(j.root()).unwrap().lines);
    assert_eq!(got.left_as_it_is.len(), 1);
    assert_eq!(got.left_as_it_is, ["Whole floor: the job has a line of that name that was measured in the field (Known), 598 SF; the mark says 650 SF; nothing was added or changed"]);
    // Nothing was added beside it to be counted twice.
    assert_eq!(j.world.get::<Cost>(j.root()).unwrap().lines.len(), 3);
    let q = view::estimate(&mut j);
    let l = q.lines.iter().find(|l| l.description == "Whole floor").unwrap();
    assert_eq!((l.quantity.value, l.quantity.status), (Some(598.0), Status::Known));
}

#[test]
fn a_mark_the_file_no_longer_has_is_named_and_its_line_is_left_and_one_that_cannot_be_read_now_has_not_gone() {
    let mut j = job();
    let m = three();
    marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap();
    // The file now lacks "mk-floor" and has "mk-south" unreadable: the second is in the file but not brought.
    let now = vec![m[2].clone()];
    let in_file = vec!["mk-south".to_string(), "count Receptacle".to_string()];
    let got = marks::bring_in(&mut j, "marked-plan.pdf", 1, &now, &in_file).unwrap();
    assert_eq!(got.gone, ["Whole floor"]);
    assert_eq!(got.same, ["Receptacle"]);
    assert_eq!(j.world.get::<Cost>(j.root()).unwrap().lines.len(), 3, "nothing is taken away because a file lacks it");
    // Another file's marks are not gone because this file lacks them.
    let other = marks::bring_in(&mut j, "other.pdf", 1, &[], &[]).unwrap();
    assert!(other.gone.is_empty());
}

#[test]
fn a_line_with_the_marks_label_already_on_the_job_is_not_overwritten_and_is_told_apart() {
    let mut j = job();
    let root = j.world.get::<Node>(j.root()).unwrap().path.clone();
    // A priced-by-hand line of the same name that is no measured figure (a lump sum typed with a formula) is not the mark's.
    j.add_line(&root, "Receptacle", "2 * 3", None, false).unwrap();
    let m = vec![mark("count Receptacle", "Receptacle", 4.0, "EA", "count"), mark("a", "Wall", 5.0, "LF", "length"), mark("b", "Wall", 6.0, "LF", "length")];
    let got = marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap();
    let names: Vec<&str> = got.added.iter().map(|a| a.0.as_str()).collect();
    assert_eq!(names, ["Receptacle (marked-plan, page 1)", "Wall", "Wall (marked-plan, page 1)"]);
    let q = view::estimate(&mut j);
    assert_eq!(q.lines.iter().find(|l| l.description == "Receptacle").unwrap().quantity.value, Some(6.0));
}

#[test]
fn a_line_of_the_marks_label_measured_in_the_field_is_that_line_and_nothing_is_added_beside_it() {
    let mut j = job();
    let root = j.world.get::<Node>(j.root()).unwrap().path.clone();
    j.add_line_measured(&root, "Receptacle", "9 EA", None, false, Status::Known, "counted in the field").unwrap();
    let m = vec![mark("count Receptacle", "Receptacle", 4.0, "EA", "count")];
    let got = marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap();
    assert!(got.added.is_empty());
    assert_eq!(got.left_as_it_is.len(), 1);
    assert_eq!(j.world.get::<Cost>(j.root()).unwrap().lines.len(), 1);
    // The same figure is the same.
    let m = vec![mark("count Receptacle", "Receptacle", 9.0, "EA", "count")];
    assert_eq!(marks::bring_in(&mut j, "marked-plan.pdf", 1, &m, &keys(&m)).unwrap().same, ["Receptacle"]);
}

#[test]
fn a_mark_that_says_nothing_to_count_is_refused_and_nothing_is_changed() {
    let mut j = job();
    let before = file::save(&mut j);
    for q in [0.0, -3.0, f64::NAN] {
        let err = marks::bring_in(&mut j, "marked-plan.pdf", 1, &[mark("k", "Nought", q, "LF", "length")], &["k".into()]).unwrap_err();
        assert!(err.contains("Nought") && err.contains("is no quantity"), "{err}");
    }
    assert_eq!(file::save(&mut j), before);
}

#[test]
fn the_source_of_a_mark_line_is_written_and_read_back_whatever_the_key_holds() {
    for key in ["mk-south", "North wall #2", "count Receptacle", "a \"quoted\" name", "with (parens), page 3 of x"] {
        let s = marks::source_text(key, "length", 4, "My plan (rev 2).pdf", Some("its points work out to 30 LF"));
        let p = marks::parse_source(&s).unwrap_or_else(|| panic!("{s}"));
        assert_eq!((p.key.as_str(), p.kind.as_str(), p.page, p.file.as_str()), (marks::clean_key(key).as_str(), "length", 4, "My plan (rev 2).pdf"), "{s}");
    }
    assert_eq!(marks::parse_source("measured on the plans"), None);
    // A count says how many symbols of which label it counted (call 231), and is read back as the count of that label.
    for label in ["Receptacle", "Switch, 3 way", "a \"quoted\" label", "x counted on page 9 of y"] {
        let s = marks::count_source_text(label, 4, 2, "plan (rev 2).pdf");
        assert!(s.starts_with("4 symbols labelled ") && s.ends_with(" counted on page 2 of plan (rev 2).pdf"), "{s}");
        let p = marks::parse_source(&s).unwrap_or_else(|| panic!("{s}"));
        assert_eq!((p.key.as_str(), p.kind.as_str(), p.page, p.file.as_str()), (marks::clean_key(&format!("count {label}")).as_str(), "count", 2, "plan (rev 2).pdf"), "{s}");
    }
    assert_eq!(marks::count_source_text("Switch", 1, 1, "a.pdf"), "1 symbol labelled Switch counted on page 1 of a.pdf");
}

#[test]
fn the_marks_of_a_page_are_kept_with_the_drawing_and_a_drawing_with_none_is_written_as_it_was() {
    // A job written by the tree this call began from (the program built at cf48938: a level, a page of the made-up plan laid
    // under it as a drawing, and its scale set from the dimension line) is read and written again to the byte.
    let text = include_str!("data/call227m_before.json");
    let (mut j, _) = file::load(text).unwrap();
    assert_eq!(file::save(&mut j), text);
    // Its drawing, given marks, writes them; given none again, writes as it did.
    let d = {
        let mut q = j.world.query::<(&Node, &plans::Drawing)>();
        q.iter(&j.world).map(|(n, _)| n.path.clone()).next().unwrap()
    };
    let shape = MarkShape {
        kind: "length".into(),
        label: "South wall".into(),
        name: Some("mk-south".into()),
        points: vec![[126.0, 126.0], [666.0, 126.0]],
        places: vec![[396.0, 126.0]],
        says: Some(Val { value: Some(30.0), unit: "LF".into(), status: Status::Assumed, source: "mark \"mk-south\" (length), page 1 of marked-plan.pdf".into() }),
        worked: Some(30.0),
        unread: None,
    };
    marks::set_shapes(&mut j, &d, vec![shape.clone()]).unwrap();
    let with = file::save(&mut j);
    assert!(with.contains("South wall") && with.contains("mk-south"));
    let (mut back, _) = file::load(&with).unwrap();
    let e = back.find(&d).unwrap();
    let kept = back.world.get::<plans::Drawing>(e).unwrap().marks.clone();
    assert_eq!(kept, vec![shape]);
    marks::set_shapes(&mut j, &d, Vec::new()).unwrap();
    assert_eq!(file::save(&mut j), text);
}

#[test]
fn a_shape_that_is_known_or_in_no_unit_or_off_the_page_is_refused() {
    let text = include_str!("data/call227m_before.json");
    let (mut j, _) = file::load(text).unwrap();
    let d = {
        let mut q = j.world.query::<(&Node, &plans::Drawing)>();
        q.iter(&j.world).map(|(n, _)| n.path.clone()).next().unwrap()
    };
    let base = MarkShape { kind: "length".into(), label: "A".into(), name: None, points: vec![[1.0, 1.0], [2.0, 2.0]], places: vec![[1.0, 1.0]], says: Some(Val { value: Some(3.0), unit: "LF".into(), status: Status::Assumed, source: "mark".into() }), worked: None, unread: None };
    let mut known = base.clone();
    known.says.as_mut().unwrap().status = Status::Known;
    assert!(marks::set_shapes(&mut j, &d, vec![known]).unwrap_err().contains("a mark is never Known"));
    let mut metres = base.clone();
    metres.says.as_mut().unwrap().unit = "m".into();
    assert!(marks::set_shapes(&mut j, &d, vec![metres]).unwrap_err().contains("which is not LF, SF or EA"));
    let mut kind = base.clone();
    kind.kind = "wall".into();
    assert!(marks::set_shapes(&mut j, &d, vec![kind]).unwrap_err().contains("not \"wall\""));
    let mut far = base.clone();
    far.points[0][0] = f64::NAN;
    assert!(marks::set_shapes(&mut j, &d, vec![far]).unwrap_err().contains("not a usable place"));
    assert_eq!(file::save(&mut j), text, "refused: nothing changed");
    let _ = Entered { status: Status::Assumed, source: "" };
}
