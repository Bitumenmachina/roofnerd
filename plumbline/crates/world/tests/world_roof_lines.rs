use plumbline_world::file::{load, save};
use plumbline_world::sample::sample;
use plumbline_world::value::Status;
use plumbline_world::*;
use serde_json::Value;

#[allow(dead_code)]
mod common;
use common::{entries, touched, measure, edit, entry, attrs, show_rows, feet_of, porch, trapezoid, figures, parapet_job, roof_job, length_of};

#[test]
fn the_eave_has_to_be_the_low_edge_of_its_plane() {
    let text = save(&mut sample());
    // A stepped outline with the eave entered on the stepped-back edge: part of the plane
    // would lie below its own eave. Refused.
    let stepped = serde_json::json!([[44.0, 0.0], [50.0, 0.0], [50.0, 2.0], [56.0, 2.0], [56.0, 8.0], [44.0, 8.0]]);
    let err = load(&edit(&text, |v| porch(v, stepped.clone(), serde_json::json!([[50.0, 2.0], [56.0, 2.0]])))).err().unwrap();
    assert!(err.contains("low side"), "{err}");
    // An L-shaped outline with the eave on the inside of the L: same.
    let ell = serde_json::json!([[44.0, 0.0], [56.0, 0.0], [56.0, 8.0], [50.0, 8.0], [50.0, 4.0], [44.0, 4.0]]);
    let err = load(&edit(&text, |v| porch(v, ell.clone(), serde_json::json!([[50.0, 4.0], [44.0, 4.0]])))).err().unwrap();
    assert!(err.contains("low side"), "{err}");
    // The stepped outline with its true low edge as the eave is fine, and a rake up its
    // full 8 ft of run at 12:12 is sqrt(64 + 64) = 11.3137 LF.
    let (mut job, _) = load(&edit(&text, |v| {
        porch(v, stepped.clone(), serde_json::json!([[44.0, 0.0], [50.0, 0.0]]));
        attrs(v, "plane-0017")["pl::pitch"] = serde_json::json!({"value": 12.0, "unit": "in/12", "status": "Known", "source": "x"});
        attrs(v, "eave-0018")["pl::run"]["ends"]["value"] = serde_json::json!([[44.0, 0.0], [50.0, 0.0]]);
        attrs(v, "rake-0020")["pl::run"]["ends"]["value"] = serde_json::json!([[56.0, 2.0], [56.0, 8.0]]);
    }))
    .unwrap();
    assert_eq!(measure(&mut job, "rake-0019", "true_length").value, Some(11.3137));
    // The short rake runs 6 ft: sqrt(36 + 36) = 8.4853.
    assert_eq!(measure(&mut job, "rake-0020", "true_length").value, Some(8.4853));
}

#[test]
fn a_crooked_sliver_of_an_eave_cannot_tilt_the_plane() {
    // A 0.028 ft eave entry lying at 45 degrees across the true eave edge. The slope is
    // measured from the outline edge it lies along, so the figures are the right ones.
    let text = save(&mut sample());
    let (mut job, _) = load(&edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["eave"]["value"] = serde_json::json!([[50.0, 0.0099], [50.0198, -0.0099]]))).unwrap();
    job.set("plane-0017", "pitch", Some("4"), Status::Known, "field measured").unwrap();
    job.derive();
    assert_eq!(measure(&mut job, "eave-0018", "true_length").value, Some(12.0));
    assert_eq!(measure(&mut job, "rake-0019", "true_length").value, Some(8.4327));
    assert_eq!(measure(&mut job, "rake-0020", "true_length").value, Some(8.4327));
    assert_eq!(measure(&mut job, "headwall-0021", "true_length").value, Some(12.0));
}

#[test]
fn the_same_key_twice_is_refused_anywhere() {
    let text = save(&mut sample());
    // Two values and two statuses on one pitch. Kept silently, the last would win.
    let doubled = text.replacen("\"value\": 3.0,", "\"value\": 3.0,\n          \"value\": 12.0,", 1);
    assert_ne!(doubled, text);
    assert!(load(&doubled).err().unwrap().contains("appears twice"));
    // Two kinds on one element.
    let doubled = text.replacen("\"pl::class\": \"Rake\",", "\"pl::class\": \"Rake\",\n        \"pl::class\": \"Eave\",", 1);
    assert_ne!(doubled, text);
    assert!(load(&doubled).err().unwrap().contains("appears twice"));
}

#[test]
fn a_line_has_to_run_along_its_plane_not_just_touch_it() {
    // An L-shaped porch with its true eave. A "rake" from one inside corner to another
    // has both ends on the outline but runs through the notch, outside the plane.
    let text = save(&mut sample());
    let ell = serde_json::json!([[44.0, 0.0], [56.0, 0.0], [56.0, 8.0], [50.0, 8.0], [50.0, 4.0], [44.0, 4.0]]);
    let (mut job, _) = load(&edit(&text, |v| {
        porch(v, ell.clone(), serde_json::json!([[44.0, 0.0], [56.0, 0.0]]));
        attrs(v, "rake-0019")["pl::run"]["ends"]["value"] = serde_json::json!([[44.0, 4.0], [50.0, 8.0]]);
    }))
    .unwrap();
    let d = measure(&mut job, "rake-0019", "true_length");
    assert_eq!(d.value, None);
    assert!(d.note.contains("does not run along the outline"), "{}", d.note);
}

#[test]
fn the_path_counter_cannot_hand_out_a_path_twice() {
    let text = save(&mut sample());
    // A counter set back below paths already in use is refused on load...
    let err = load(&edit(&text, |v| attrs(v, "job-0001")["pl::job"]["next_id"] = 3.into())).err().unwrap();
    assert!(err.contains("next_id"), "{err}");
    // ...and add skips a taken path even if the counter is wrong in memory.
    let mut job = sample();
    let root = job.root();
    job.world.get_mut::<JobInfo>(root).unwrap().next_id = 3;
    let path = job.add("roof-0002", Kind::RoofPlane, "new-plane").unwrap();
    assert_eq!(path, "plane-0028");
    assert!(load(&save(&mut job)).is_ok());
}

#[test]
fn a_parapet_names_no_plane() {
    let text = save(&mut sample());
    let err = load(&edit(&text, |v| attrs(v, "parapet-0024")["pl::run"]["planes"] = serde_json::json!(["lowslope-0023"]))).err().unwrap();
    assert!(err.contains("a parapet lies on no plane"), "{err}");
}

// ---- D2 groundwork: the lines a set of planes implies ----

#[test]
fn the_planes_of_the_sample_job_imply_the_lines_that_were_entered() {
    let mut job = sample();
    let (planes, left_out) = roof::solid_planes(&mut job);
    assert_eq!(planes.len(), 6);
    assert!(left_out.is_empty(), "{left_out:?}");
    let found = roof::lines_of(&planes);
    assert!(found.problems.is_empty(), "{:?}", found.problems);
    let count = |k: Kind| found.lines.iter().filter(|l| l.kind == k).count();
    // Hip roof: 4 eaves, 4 hips, 1 ridge. Porch shed: eave, 2 rakes, and its level top
    // edge, which the planes can only call a ridge. Deck: its eave, two sloped sides, top.
    assert_eq!((count(Kind::Eave), count(Kind::Hip), count(Kind::Ridge), count(Kind::Rake), count(Kind::Valley)), (6, 4, 3, 4, 0));
    // The southwest hip: from the corner to the ridge end, on the south and west planes.
    let hip = found.lines.iter().find(|l| l.kind == Kind::Hip && l.planes == ["plane-0003", "plane-0006"]).unwrap();
    assert!(hip.ends == [[0.0, 0.0], [15.0, 15.0]] || hip.ends == [[15.0, 15.0], [0.0, 0.0]], "{:?}", hip.ends);
    // Every roof line entered in the sample is one the planes imply, the headwall included.
    let cmp = roof::compare(&mut job, &found);
    assert_eq!(cmp.agree.len(), 13);
    assert!(cmp.entered_only.is_empty(), "{:?}", cmp.entered_only);
    // What nobody entered: the deck's four edges.
    assert_eq!(cmp.implied_only.len(), 4);
    assert!(cmp.implied_only.iter().all(|l| l.planes == ["lowslope-0023"]));
}

#[test]
fn planes_that_disagree_along_a_shared_edge_are_reported_against_each_other() {
    // Steepen the south plane only. It shares the ridge with the north plane and a hip
    // with each of the east and west planes, and no longer agrees with any of them.
    let mut job = sample();
    job.set("plane-0003", "pitch", Some("8"), Status::Known, "revised").unwrap();
    job.derive();
    let (planes, _) = roof::solid_planes(&mut job);
    let found = roof::lines_of(&planes);
    assert_eq!(found.problems.len(), 3, "{:?}", found.problems);
    assert!(found.problems.iter().all(|p| p.contains("plane-0003")));
    // The two hips climb to a top the planes put at different heights: they do not fit.
    assert_eq!(found.problems.iter().filter(|p| p.contains("disagree about its height")).count(), 2);
    // The ridge is level on both planes, at 20 ft on one and 17.5 ft on the other. That is
    // a step or a misfit; the planes cannot say which, and each keeps its own top edge.
    assert_eq!(found.problems.iter().filter(|p| p.contains("at different heights (20 ft and 17 ft 6 in)")).count(), 1);  // call 226: a length in a sentence is said as a builder says it
    assert_eq!(found.lines.iter().filter(|l| l.kind == Kind::Ridge && l.planes.len() == 2).count(), 0);
    assert!(found.lines.iter().any(|l| l.kind == Kind::Ridge && l.planes == ["plane-0003"]));
    assert!(found.lines.iter().any(|l| l.kind == Kind::Ridge && l.planes == ["plane-0005"]));
    // The hips that do not touch the south plane are still found.
    assert_eq!(found.lines.iter().filter(|l| l.kind == Kind::Hip).count(), 2);
    // The entered ridge and hips are reported as lying on edges the planes dispute.
    let cmp = roof::compare(&mut job, &found);
    assert_eq!(cmp.entered_only.iter().filter(|e| e.contains("do not agree about")).count(), 3, "{:?}", cmp.entered_only);
}

#[test]
fn a_wing_meeting_a_main_roof_makes_two_valleys() {
    // A gable roof 40 x 20 with its ridge east-west, and a 10 ft wide gabled wing running
    // south from it, all 6:12 from 10 ft. The wing's ridge (5 ft of run above its eaves)
    // meets the main south plane 5 ft up it, at (20, 5); a valley runs to that point from
    // each inside corner, (15, 0) and (25, 0).
    let mut job = Job::new("Cross gable", "test", "test");
    let roof_path = job.add("job-0001", Kind::Roof, "roof").unwrap();
    let mut plane = |name: &str, pts: Vec<P2>, eave: [P2; 2]| {
        let p = job.add(&roof_path, Kind::RoofPlane, name).unwrap();
        job.put(&p, Outline { points: Val::known(pts, "ft", "test"), eave: Val::known(eave, "ft", "test"), eave_height: Val::known(Len::One(10.0), "ft", "test") }).unwrap();
        job.put(&p, Pitch(Val::known(6.0, "in/12", "test"))).unwrap();
        p
    };
    let south = plane("south", vec![[0.0, 0.0], [15.0, 0.0], [20.0, 5.0], [25.0, 0.0], [40.0, 0.0], [40.0, 10.0], [0.0, 10.0]], [[0.0, 0.0], [15.0, 0.0]]);
    let north = plane("north", vec![[40.0, 20.0], [0.0, 20.0], [0.0, 10.0], [40.0, 10.0]], [[40.0, 20.0], [0.0, 20.0]]);
    let west = plane("wing-west", vec![[15.0, -10.0], [15.0, 0.0], [20.0, 5.0], [20.0, -10.0]], [[15.0, -10.0], [15.0, 0.0]]);
    let east = plane("wing-east", vec![[25.0, 0.0], [25.0, -10.0], [20.0, -10.0], [20.0, 5.0]], [[25.0, 0.0], [25.0, -10.0]]);
    // The job as built passes the entry checks.
    let (mut job, _) = load(&save(&mut job)).unwrap();
    let (planes, left_out) = roof::solid_planes(&mut job);
    assert!(left_out.is_empty(), "{left_out:?}");
    let found = roof::lines_of(&planes);
    assert!(found.problems.is_empty(), "{:?}", found.problems);
    let has = |kind: Kind, a: P2, b: P2, on: &[&String]| {
        found.lines.iter().any(|l| l.kind == kind && (l.ends == [a, b] || l.ends == [b, a]) && l.planes.iter().collect::<Vec<_>>() == on)
    };
    assert!(has(Kind::Valley, [15.0, 0.0], [20.0, 5.0], &[&south, &west]), "{:#?}", found.lines);
    assert!(has(Kind::Valley, [25.0, 0.0], [20.0, 5.0], &[&south, &east]));
    assert!(has(Kind::Ridge, [0.0, 10.0], [40.0, 10.0], &[&south, &north]));
    assert!(has(Kind::Ridge, [20.0, -10.0], [20.0, 5.0], &[&west, &east]));
    // The main eave is in two pieces, one each side of the wing; both are eaves.
    assert!(has(Kind::Eave, [0.0, 0.0], [15.0, 0.0], &[&south]));
    assert!(has(Kind::Eave, [25.0, 0.0], [40.0, 0.0], &[&south]));
    // Gable ends are rakes: four on the main roof, two on the wing.
    assert_eq!(found.lines.iter().filter(|l| l.kind == Kind::Rake).count(), 6);
    assert_eq!(found.lines.iter().filter(|l| l.kind == Kind::Hip).count(), 0);
    // Two valleys, two ridges, three main eaves (the south one in two pieces), two wing
    // eaves, six rakes.
    assert_eq!(found.lines.len(), 2 + 2 + 3 + 2 + 6);
}

// ---- Regression checks for what the third independent judgment found ----

#[test]
fn a_line_that_shows_the_eave_is_on_the_wrong_edge_takes_the_length_off_every_line_on_that_plane() {
    let outline = vec![[0.0, 0.0], [20.0, 0.0], [25.0, 10.0], [0.0, 10.0]];
    let lines = [(Kind::Rake, "rake", [[20.0, 0.0], [25.0, 10.0]]), (Kind::Eave, "eave", [[0.0, 0.0], [20.0, 0.0]])];
    // Entered right: the eave is the low edge, y = 0. The rake runs 5 over and 10 up on
    // plan (11.1803 LF) and climbs 10 ft of run at 6:12 = 5 ft: sqrt(125 + 25) = 12.2474.
    let mut right = trapezoid([[0.0, 0.0], [20.0, 0.0]], outline.clone(), &lines);
    assert_eq!(measure(&mut right, "rake-0003", "true_length").value, Some(12.2474));
    assert_eq!(measure(&mut right, "eave-0004", "true_length").value, Some(20.0));
    // Entered wrong: the eave on the left edge, x = 0. The rake still climbs on that
    // reading (it would come out 11.4564), so its own kind does not give it away. The
    // eave line does: it climbs, which an eave cannot. That takes the rake's length too.
    let mut wrong = trapezoid([[0.0, 10.0], [0.0, 0.0]], outline, &lines);
    let eave = measure(&mut wrong, "eave-0004", "true_length");
    assert!(eave.value.is_none() && eave.note.contains("an eave is level"), "{}", eave.note);
    let rake = measure(&mut wrong, "rake-0003", "true_length");
    assert_eq!((rake.value, rake.status), (None, Status::Unknown));
    assert!(rake.note.contains("eave entered on the wrong edge") && rake.note.contains("eave-0004"), "{}", rake.note);
}

#[test]
fn a_line_across_a_notch_does_not_run_along_the_outline_however_the_gap_falls() {
    // The low edge has a 5 ft notch in it, from x = 9 to x = 14. An eave line drawn
    // straight across from 0 to 30 has 5 ft of itself where there is no roof. The five
    // points once tested (x = 0, 7.5, 15, 22.5, 30) all landed on edges.
    let notched = vec![[0.0, 0.0], [9.0, 0.0], [9.0, 5.0], [14.0, 5.0], [14.0, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]];
    let mut job = trapezoid([[14.0, 0.0], [30.0, 0.0]], notched.clone(), &[(Kind::Eave, "across", [[0.0, 0.0], [30.0, 0.0]]), (Kind::Eave, "left", [[0.0, 0.0], [9.0, 0.0]]), (Kind::Eave, "right", [[14.0, 0.0], [30.0, 0.0]])]);
    let d = measure(&mut job, "eave-0003", "true_length");
    assert_eq!(d.value, None);
    assert!(d.note.contains("does not run along the outline"), "{}", d.note);
    // The two real pieces of eave are 9 and 16 LF.
    assert_eq!(measure(&mut job, "eave-0004", "true_length").value, Some(9.0));
    assert_eq!(measure(&mut job, "eave-0005", "true_length").value, Some(16.0));
    // A gap of an inch is still a gap.
    let hairline = vec![[0.0, 0.0], [9.0, 0.0], [9.0, 5.0], [9.0833, 5.0], [9.0833, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]];
    let mut job = trapezoid([[9.0833, 0.0], [30.0, 0.0]], hairline, &[(Kind::Eave, "across", [[0.0, 0.0], [30.0, 0.0]])]);
    assert_eq!(measure(&mut job, "eave-0003", "true_length").value, None);
}

#[test]
fn an_edge_traced_with_a_point_part_way_along_it_is_still_one_edge() {
    // The low edge traced as two pieces, 0 to 10 and 10 to 20. The eave is the whole of
    // it, and a line along the whole of it runs along the outline.
    let outline = vec![[0.0, 0.0], [10.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]];
    let mut job = trapezoid([[0.0, 0.0], [20.0, 0.0]], outline, &[(Kind::Eave, "eave", [[0.0, 0.0], [20.0, 0.0]]), (Kind::Rake, "rake", [[20.0, 0.0], [20.0, 10.0]])]);
    assert_eq!(measure(&mut job, "eave-0003", "true_length").value, Some(20.0));
    // 10 ft of run at 6:12 climbs 5 ft: sqrt(100 + 25) = 11.1803.
    assert_eq!(measure(&mut job, "rake-0004", "true_length").value, Some(11.1803));
}

#[test]
fn a_bare_fraction_is_not_read_as_a_pitch_and_a_minus_on_the_inches_is_not_dropped() {
    use units::{parse_length, parse_pitch};
    // 3/4 is a rise of 3/4 in 12 to one estimator and 3 in 4 to another. Refused; with
    // its run it is read.
    for typed in ["3/4", "4/8", "8/16", "6/8", "1/2", "1/4", "-1/4"] {
        let e = parse_pitch(typed).expect_err(typed);
        assert!(e.contains("could be a rise of"), "{typed}: {e}");
    }
    assert_eq!(parse_pitch("3/4:12").unwrap(), 0.75);
    assert_eq!(parse_pitch("3/4 in/12").unwrap(), 0.75);
    assert_eq!(parse_pitch("6/12").unwrap(), 6.0);
    assert_eq!(parse_pitch("6 1/2").unwrap(), 6.5);
    // Nothing reaches the job.
    let mut job = sample();
    let before = save(&mut job);
    assert!(job.set("plane-0017", "pitch", Some("3/4"), Status::Known, "x").is_err());
    assert_eq!(save(&mut job), before);

    // 25'-6" and 25' - 6" use the dash as a separator. 0' -6" is a minus on the inches.
    for typed in ["0' -6\"", "25' -6\"", "1' -6 1/2\"", "25 ft -6 in"] {
        let e = parse_length(typed).expect_err(typed);
        assert!(e.contains("minus on the inches"), "{typed}: {e}");
    }
    assert_eq!(feet_of("25'-6\""), 25.5);
    assert_eq!(feet_of("25' - 6\""), 25.5);
    assert_eq!(feet_of("25'- 6\""), 25.5);
    assert!(job.set("parapet-0024", "height", Some("0' -6\""), Status::Known, "x").is_err());
    assert_eq!(save(&mut job), before);
    // A number written as something that a 64-bit number holds as nothing.
    let tiny = format!("0.{}1 ft", "0".repeat(400));
    assert!(parse_length(&tiny).unwrap_err().contains("too small a number"));
}

#[test]
fn nothing_is_silently_zero_or_silently_unread() {
    let text = save(&mut sample());
    let err = |t: String| load(&t).err().expect("should be refused");
    // A figure that is something but would be written 0.0000 is not shown as zero.
    let mut job = sample();
    job.set("headwall-0021", "stretch_out", Some("0.00001 in"), Status::Known, "x").unwrap();
    job.derive();
    let e = job.find("headwall-0021").unwrap();
    let d = job.world.get::<LineQuantities>(e).unwrap().0["Copper headwall flashing"].quantity.clone();
    assert_eq!((d.value, d.status), (None, Status::Unknown));
    assert!(d.note.contains("smaller than 0.0001"), "{}", d.note);
    // A value under a mis-cased key of this program's own is refused, not carried unread.
    let e = err(edit(&text, |v| {
        let pitch = attrs(v, "plane-0017").as_object_mut().unwrap().remove("pl::pitch").unwrap();
        attrs(v, "plane-0017")["pl::Pitch"] = pitch;
    }));
    assert!(e.contains("\"pl::Pitch\"") && e.contains("did you mean \"pl::pitch\""), "{e}");
    assert!(err(edit(&text, |v| attrs(v, "plane-0017")["pl::slope"] = 3.into())).contains("not one of this program's components"));
    // Another program's attributes are still kept.
    assert!(load(&edit(&text, |v| attrs(v, "plane-0017")["other::Pitch"] = 3.into())).is_ok());
    // Two cost lines that differ only in capitals or spacing are one line entered twice.
    let line = |d: &str| serde_json::json!({"description": d, "quantity": "true_length * stretch_out", "stretch_out": {"value": 12.0, "unit": "in", "status": "Known", "source": "x"}, "unit_cost": {"value": null, "unit": "$/SF", "status": "Unknown", "source": "x"}});
    for near in ["copper headwall flashing", "Copper  headwall flashing", "COPPER HEADWALL FLASHING"] {
        let e = err(edit(&text, |v| attrs(v, "headwall-0021")["pl::cost"]["lines"].as_array_mut().unwrap().push(line(near))));
        assert!(e.contains("two cost lines"), "{near}: {e}");
    }
    assert!(err(edit(&text, |v| attrs(v, "headwall-0021")["pl::cost"]["lines"].as_array_mut().unwrap().push(line("Copper headwall flashing ")))).contains("starts or ends with a space"));
    assert!(load(&edit(&text, |v| attrs(v, "headwall-0021")["pl::cost"]["lines"].as_array_mut().unwrap().push(line("Copper counterflashing")))).is_ok());
    // A minus zero typed into the file is shown as zero.
    let (mut job, _) = load(&edit(&text, |v| attrs(v, "lowslope-0023")["pl::pitch"]["value"] = (-0.0).into())).unwrap();
    assert!(show_rows(&mut job, "lowslope-0023").contains(&("pitch".to_string(), "0 in/12".to_string(), Status::Known)));
}

// ---- D3: derived conditions and stretch-out ----

#[test]
fn one_parapet_and_one_condition_make_four_lines_and_nothing_is_assumed_for_them() {
    let mut job = parapet_job();
    // The parapet carries the condition's name and the four lines, each with its formula.
    let e = job.find("parapet-0024").unwrap();
    assert_eq!(job.world.get::<Condition>(e).unwrap().0.value.as_deref(), Some("Parapet with copper coping"));
    let lines: Vec<(String, String)> = job.world.get::<Cost>(e).unwrap().lines.iter().map(|l| (l.description.clone(), l.quantity.clone())).collect();
    assert_eq!(
        lines,
        [
            ("Copper wall flashing".to_string(), "plan_length * height".to_string()),
            ("Copper coping cap".to_string(), "plan_length * stretch_out".to_string()),
            ("Continuous cleat".to_string(), "plan_length * sides".to_string()),
            ("Cleat fasteners".to_string(), "plan_length * sides / spacing".to_string()),
        ]
    );
    // The height is not on the plan and no stretch-out or spacing has been entered: three
    // lines have no number. Nothing was filled in for them, and nothing on them is Assumed.
    for line in ["Copper wall flashing", "Copper coping cap", "Cleat fasteners"] {
        let f = figures(&mut job, "parapet-0024", line);
        assert_eq!((f.quantity.value, f.quantity.status), (None, Status::Unknown), "{line}");
    }
    let coping = figures(&mut job, "parapet-0024", "Copper coping cap");
    assert_eq!(coping.with_waste.as_ref().map(|d| d.value), Some(None));
    assert_eq!(coping.order.as_ref().map(|d| d.value), Some(None));
    // The cleat needs only the parapet's length and the condition's rule, a cleat each
    // side: 20 LF x 2 = 40 LF.
    let cleat = figures(&mut job, "parapet-0024", "Continuous cleat");
    assert_eq!((cleat.quantity.value, cleat.quantity.unit.as_str(), cleat.quantity.status), (Some(40.0), "LF", Status::Known));
    assert_eq!(cleat.quantity.formula, "plan_length × sides");
    // It saves, loads and saves to the same bytes, and the recipe is still in the job.
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut again), text);
    // Only the parapet changed when the condition was applied.
    assert_eq!(touched(&save(&mut sample()), &text), ["parapet-0024"]);

    // A condition goes on the kind of element it is for, once.
    let mut job = sample();
    assert!(job.apply("plane-0003", "Parapet with copper coping", "x").unwrap_err().contains("applies to a parapet"));
    assert!(job.apply("parapet-0024", "No such condition", "x").unwrap_err().contains("no assembly"));
    job.apply("parapet-0024", "condition-0025", "x").unwrap();
    assert!(job.apply("parapet-0024", "condition-0025", "x").unwrap_err().contains("has every line of it"));
}

#[test]
fn the_sheet_metal_chain_is_worked_step_by_step_and_an_unknown_stops_it() {
    let mut job = parapet_job();
    let k = |job: &mut Job, field: &str, line: Option<&str>, typed: &str| {
        job.set_on_line("parapet-0024", field, line, Some(typed), Status::Known, "detail 3/A-501").unwrap();
        job.derive();
    };
    // Wall flashing: a height on its LF. 20 LF x 30 in / 12 = 50 SF.
    k(&mut job, "height", None, "30 in");
    let f = figures(&mut job, "parapet-0024", "Copper wall flashing");
    assert_eq!((f.quantity.value, f.quantity.unit.as_str(), f.quantity.status), (Some(50.0), "SF", Status::Known));
    assert_eq!(f.quantity.formula, "plan_length × height ÷ 12");
    assert_eq!(f.quantity.inputs, ["parapet-0024#pl::measure", "parapet-0024#pl::height"]);
    // Coping: 20 LF x 19 1/2 in / 12 = 32.5 SF. With no waste and no sheet size yet, the
    // steps after it have no number.
    k(&mut job, "stretch_out", Some("Copper coping cap"), "19 1/2\"");
    let f = figures(&mut job, "parapet-0024", "Copper coping cap");
    assert_eq!((f.quantity.value, f.quantity.formula.as_str()), (Some(32.5), "plan_length × stretch_out ÷ 12"));
    assert_eq!(f.with_waste.as_ref().unwrap().value, None);
    assert_eq!(f.order.as_ref().unwrap().value, None);
    // Waste of 10 percent, assumed: 32.5 x 1.10 = 35.75 SF, Assumed.
    job.set_on_line("parapet-0024", "waste", Some("Copper coping cap"), Some("10%"), Status::Assumed, "shop rule of thumb").unwrap();
    job.derive();
    let f = figures(&mut job, "parapet-0024", "Copper coping cap");
    let w = f.with_waste.as_ref().unwrap();
    assert_eq!((w.value, w.status, w.formula.as_str()), (Some(35.75), Status::Assumed, "quantity × (1 + waste ÷ 100)"));
    assert_eq!(f.order.as_ref().unwrap().value, None);
    // Sheets of 30 SF (3 ft x 10 ft): 35.75 / 30 = 1.19, so 2 sheets.
    k(&mut job, "order", Some("Copper coping cap"), "30 SF/sheet");
    let o = figures(&mut job, "parapet-0024", "Copper coping cap").order.unwrap();
    assert_eq!((o.value, o.unit.as_str(), o.status), (Some(2.0), "sheet", Status::Assumed));
    assert_eq!(o.formula, "with_waste ÷ factor, rounded up to a whole sheet");
    let e = job.find("parapet-0024").unwrap();
    assert_eq!(job.world.get::<Cost>(e).unwrap().lines[1].unit_cost.unit, "$/sheet");
    // Or pounds, at 1.25 lb to the SF (20 oz copper): 35.75 x 1.25 = 44.6875 lb, not rounded.
    k(&mut job, "order", Some("Copper coping cap"), "1.25 lb/SF");
    let o = figures(&mut job, "parapet-0024", "Copper coping cap").order.unwrap();
    assert_eq!((o.value, o.unit.as_str(), o.formula.as_str()), (Some(44.6875), "lb", "with_waste × factor"));
    assert_eq!(job.world.get::<Cost>(e).unwrap().lines[1].unit_cost.unit, "$/lb");
    // Fasteners by spacing, a cleat each side: 20 LF x 2 / (12 in / 12) = 40. At 18 in it
    // is 26.67, and a fastener is a whole thing: 27.
    k(&mut job, "spacing", Some("Cleat fasteners"), "12 in");
    assert_eq!(figures(&mut job, "parapet-0024", "Cleat fasteners").quantity.value, Some(40.0));
    k(&mut job, "spacing", Some("Cleat fasteners"), "18 in");
    let f = figures(&mut job, "parapet-0024", "Cleat fasteners").quantity;
    assert_eq!((f.value, f.unit.as_str()), (Some(27.0), "EA"));
    assert_eq!(f.formula, "plan_length × sides ÷ (spacing ÷ 12), rounded up to a whole number");
    // A sheet that the coping fills exactly is not rounded past: 20 LF x 36 in = 60 SF,
    // no waste, 30 SF sheets: 2, not 3.
    k(&mut job, "stretch_out", Some("Copper coping cap"), "36 in");
    k(&mut job, "waste", Some("Copper coping cap"), "0");
    k(&mut job, "order", Some("Copper coping cap"), "30 SF/sheet");
    assert_eq!(figures(&mut job, "parapet-0024", "Copper coping cap").order.unwrap().value, Some(2.0));

    // Take the stretch-out away: every step after it loses its number. None becomes zero.
    job.set_on_line("parapet-0024", "stretch_out", Some("Copper coping cap"), None, Status::Unknown, "detail withdrawn").unwrap();
    job.derive();
    let f = figures(&mut job, "parapet-0024", "Copper coping cap");
    for d in [&f.quantity, f.with_waste.as_ref().unwrap(), f.order.as_ref().unwrap()] {
        assert_eq!((d.value, d.status), (None, Status::Unknown));
    }
    // The file still round-trips.
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut again), text);
}

#[test]
fn an_edit_on_a_line_changes_that_line_and_nothing_else() {
    let mut job = parapet_job();
    for (field, line, typed) in [("height", None, "30 in"), ("stretch_out", Some("Copper coping cap"), "19 1/2 in"), ("spacing", Some("Cleat fasteners"), "12 in")] {
        job.set_on_line("parapet-0024", field, line, Some(typed), Status::Known, "detail").unwrap();
    }
    job.derive();
    let before = save(&mut job);
    let quantities = |text: &str| entries(text).into_iter().find(|e| e.0 == "parapet-0024").unwrap().1["attributes"]["pl::line_quantities"].clone();
    // The coping's stretch-out, on its line.
    job.set_on_line("parapet-0024", "stretch_out", Some("Copper coping cap"), Some("24 in"), Status::Known, "detail rev B").unwrap();
    assert_eq!(job.derive(), ["parapet-0024"]);
    let after = save(&mut job);
    assert_eq!(touched(&before, &after), ["parapet-0024"]);
    let (a, b) = (quantities(&before), quantities(&after));
    for line in ["Copper wall flashing", "Continuous cleat", "Cleat fasteners"] {
        assert_eq!(a[line], b[line], "{line}");
    }
    assert_eq!((a["Copper coping cap"]["quantity"]["value"].as_f64(), b["Copper coping cap"]["quantity"]["value"].as_f64()), (Some(32.5), Some(40.0)));

    // The cleat's formula, on its line: one side instead of two. The value the formula no
    // longer names comes off the line and the caller is told.
    let dropped = job.set_formula("parapet-0024", Some("Continuous cleat"), "plan_length", &[], Status::Known, "detail rev B").unwrap();
    assert_eq!(dropped, ["sides"]);
    job.derive();
    let last = save(&mut job);
    assert_eq!(touched(&after, &last), ["parapet-0024"]);
    let c = quantities(&last);
    for line in ["Copper wall flashing", "Copper coping cap", "Cleat fasteners"] {
        assert_eq!(b[line], c[line], "{line}");
    }
    assert_eq!(c["Continuous cleat"]["quantity"]["value"].as_f64(), Some(20.0));

    // A formula can bring in a value of its own: the flashing turned out 8 in onto the
    // roof. 20 LF x (30 in + 8 in) / 12 = 63.3333 SF.
    job.set_formula("parapet-0024", Some("Copper wall flashing"), "plan_length * (height + turn_out)", &[("turn_out".to_string(), "8 in".to_string())], Status::Known, "detail rev B").unwrap();
    job.derive();
    let f = figures(&mut job, "parapet-0024", "Copper wall flashing").quantity;
    assert_eq!(f.value, Some(63.3333));
    assert_eq!(f.formula, "plan_length × (height ÷ 12 + turn_out ÷ 12)");
    assert_eq!(f.inputs, ["parapet-0024#pl::measure", "parapet-0024#pl::height", "parapet-0024#pl::cost"]);
    assert!(load(&save(&mut job)).is_ok());
}

#[test]
fn the_briefs_four_stretch_outs_each_worked_by_hand() {
    let mut job = sample();
    // Eave 12 in, on the south eave, 40 LF: 40 SF.
    // Step flashing 8 in, up the porch's west rake, 8.2462 LF: 8.2462 x 8 / 12 = 5.4975 SF.
    // Valley 36 in. The sample roof has no valley; the figure is on a hip, 22.5 LF, which
    // is the same arithmetic: 22.5 x 36 / 12 = 67.5 SF.
    // Headwall 24 in, on the porch headwall, 12 LF: 24 SF (the line the sample carries).
    for (path, typed, sf) in [("eave-0007", "12 in", 40.0), ("rake-0019", "8 in", 5.4975), ("hip-0012", "36 in", 67.5)] {
        job.apply(path, "Copper flashing", "sample finish schedule").unwrap();
        job.set_on_line(path, "stretch_out", None, Some(typed), Status::Known, "the brief").unwrap();
        job.derive();
        let f = figures(&mut job, path, "Copper flashing").quantity;
        assert_eq!((f.value, f.unit.as_str()), (Some(sf), "SF"), "{path}");
        assert_eq!(f.formula, "true_length × stretch_out ÷ 12");
    }
    assert_eq!(figures(&mut job, "headwall-0021", "Copper headwall flashing").quantity.value, Some(24.0));
    // The step flashing rests on the porch's assumed pitch, so it is Assumed; the eave is not.
    assert_eq!(figures(&mut job, "rake-0019", "Copper flashing").quantity.status, Status::Assumed);
    assert_eq!(figures(&mut job, "eave-0007", "Copper flashing").quantity.status, Status::Known);
}

#[test]
fn a_formula_that_does_not_make_sense_is_refused_with_the_reason() {
    use formula::parse;
    for (text, says) in [("", "is blank"), ("plan_length *", "stops where"), ("(plan_length", "never closed"), ("plan_length)", "closing bracket"), ("plan_length 2", "side by side"), ("2 ^ 3", "not something a formula can hold"), ("* 2", "nothing before it"), ("1.2.3", "not a number"), ("=A1*B1", "not something a formula can hold")] {
        let e = parse(text).expect_err(text);
        assert!(e.contains(says), "{text:?}: {e}");
    }
    // On a line: names the element does not have, lengths added to areas, more than a
    // volume, one-over-a-length, and division by a zero.
    let mut job = parapet_job();
    let mut refuse = |formula: &str, with: &[(&str, &str)]| {
        let with: Vec<(String, String)> = with.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
        job.set_formula("parapet-0024", Some("Copper wall flashing"), formula, &with, Status::Known, "x").unwrap_err()
    };
    assert!(refuse("plan_length * hieght", &[]).contains("is not a figure of this thing"));
    assert!(refuse("true_length * height", &[]).contains("is not a figure of this thing"));
    assert!(refuse("plan_length + face_area", &[]).contains("adds what cannot be added"));
    // A volume is a quantity now, in CY (call 130); more than a volume is not.
    assert!(refuse("plan_length * height * height * height", &[]).contains("more than a volume"));
    assert!(refuse("height / face_area", &[]).contains("one over a length"));
    assert!(refuse("slope_factor", &[]).contains("is not a figure of this thing"));
    assert!(refuse("plan_length * height", &[("extra", "2")]).contains("does not use"));
    assert!(refuse("plan_length * height * Sides", &[("Sides", "2")]).contains("small letters"));
    assert!(refuse("plan_length * waste", &[("waste", "2")]).contains("already means something"));
    // Nothing above changed the line.
    assert_eq!(figures(&mut job, "parapet-0024", "Copper wall flashing").quantity.formula, "plan_length × height");
    // A spacing of zero does not divide: it is refused as it is entered, and says why.
    let e = job.set_on_line("parapet-0024", "spacing", Some("Cleat fasteners"), Some("0 in"), Status::Known, "x").unwrap_err();
    assert!(e.contains("divides by") && e.contains("zero"), "{e}");
    // A plain number where a length is wanted is refused, and the other way about.
    assert!(job.set_on_line("parapet-0024", "spacing", Some("Cleat fasteners"), Some("12"), Status::Known, "x").unwrap_err().contains("has no unit"));
    assert!(job.set_on_line("parapet-0024", "sides", Some("Cleat fasteners"), Some("2 ft"), Status::Known, "x").unwrap_err().contains("plain number"));
    // A plane's squares stand alone as a quantity, and cannot be mixed into a formula.
    let text = save(&mut sample());
    assert!(load(&edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["lines"][0] = serde_json::json!({"description": "x", "quantity": "squares", "unit_cost": {"value": null, "unit": "$/SQ", "status": "Unknown", "source": "x"}}))).is_ok());
    let e = load(&edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["lines"][0]["quantity"] = "squares * 2".into())).err().unwrap();
    assert!(e.contains("which is in SQ"), "{e}");
}

#[test]
fn a_condition_in_the_file_has_to_be_a_recipe_that_makes_sense() {
    let text = save(&mut sample());
    let err = |t: String| load(&t).err().expect("should be refused");
    assert!(err(edit(&text, |v| {
        attrs(v, "condition-0025").as_object_mut().unwrap().remove("pl::recipe");
    }))
    .contains("an assembly is its lines"));
    assert!(err(edit(&text, |v| attrs(v, "condition-0025")["pl::recipe"]["applies_to"] = serde_json::json!([]))).contains("does not say what it goes on"));
    // (Call 57: an assembly goes on anything built and on the job; not on a level.)
    assert!(err(edit(&text, |v| attrs(v, "condition-0025")["pl::recipe"]["applies_to"] = serde_json::json!(["Storey"]))).contains("not on a level"));
    // A recipe line has to make sense on every kind it applies to: a hip has no height.
    assert!(err(edit(&text, |v| attrs(v, "condition-0025")["pl::recipe"]["applies_to"] = serde_json::json!(["Parapet", "Hip"]))).contains("is not a figure of this thing"));
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::recipe"] = attrs(&mut serde_json::from_str(&text).unwrap(), "condition-0025")["pl::recipe"].clone())).contains("only an assembly can"));
    // The order conversion has to be between the line's unit and the order unit, and waste
    // is a percentage.
    let coping = |v: &mut Value, key: &str, val: Value| attrs(v, "condition-0025")["pl::recipe"]["cost"]["lines"][1][key] = val;
    assert!(err(edit(&text, |v| coping(v, "order", serde_json::json!({"unit": "sheet", "factor": {"value": 30.0, "unit": "LF/sheet", "status": "Known", "source": "x"}, "round_up": true})))).contains("order factor in"));
    assert!(err(edit(&text, |v| coping(v, "order", serde_json::json!({"unit": "sheet", "factor": {"value": 0.0, "unit": "SF/sheet", "status": "Known", "source": "x"}, "round_up": true})))).contains("more than zero"));
    assert!(err(edit(&text, |v| coping(v, "waste", serde_json::json!({"value": -5.0, "unit": "%", "status": "Known", "source": "x"})))).contains("percentage from 0"));
    assert!(err(edit(&text, |v| coping(v, "waste", serde_json::json!({"value": 5.0, "unit": "", "status": "Known", "source": "x"})))).contains("has to be \"%\""));
    assert!(err(edit(&text, |v| coping(v, "unit_cost", serde_json::json!({"value": null, "unit": "$/SF", "status": "Unknown", "source": "x"})))).contains("has to be \"$/sheet\""));
}

// ---- Regression checks for what the fourth independent judgment found ----

#[test]
fn evidence_of_a_wrong_eave_is_not_lost_behind_another_fault_or_another_plane() {
    // Two planes either side of a ridge at y = 15. Entered right, each has its eave on its
    // long outer edge and the rake from (0, 0) to (5, 15) is sqrt(250 + 7.5 x 7.5) = 17.5 LF.
    let (south, north) = (vec![[0.0, 0.0], [30.0, 0.0], [30.0, 15.0], [5.0, 15.0]], vec![[0.0, 30.0], [30.0, 30.0], [30.0, 15.0], [5.0, 15.0]]);
    let (south_eave, north_eave) = ([[0.0, 0.0], [30.0, 0.0]], [[0.0, 30.0], [30.0, 30.0]]);
    let wrong_south = [[30.0, 0.0], [30.0, 15.0]];
    let wrong_north = [[30.0, 30.0], [30.0, 15.0]];
    let lines: [(Kind, [P2; 2], &[usize]); 3] = [(Kind::Rake, [[0.0, 0.0], [5.0, 15.0]], &[0]), (Kind::Rake, [[0.0, 30.0], [5.0, 15.0]], &[1]), (Kind::Ridge, [[5.0, 15.0], [30.0, 15.0]], &[0, 1])];
    let mut right = roof_job(&[(south.clone(), south_eave), (north.clone(), north_eave)], &lines).unwrap();
    assert_eq!(length_of(&mut right, "line-0").value, Some(17.5));
    assert_eq!(length_of(&mut right, "line-2").value, Some(25.0));

    // The south eave on the wrong edge. The ridge climbs on the south plane, and the two
    // planes also disagree about its height. The disagreement is what the ridge reports,
    // but the climb is still evidence about the south plane: its rake has no length.
    let mut job = roof_job(&[(south.clone(), wrong_south), (north.clone(), north_eave)], &lines).unwrap();
    assert!(length_of(&mut job, "line-2").note.contains("disagree"));
    let rake = length_of(&mut job, "line-0");
    assert_eq!(rake.value, None);
    assert!(rake.note.contains("eave entered on the wrong edge"), "{}", rake.note);
    // The north plane is entered right and its rake keeps its length.
    assert_eq!(length_of(&mut job, "line-1").value, Some(17.5));

    // Both eaves wrong the same way: the ridge climbs on both planes and they agree about
    // its height. Both planes are shown wrong, whichever the ridge names first.
    for order in [[0usize, 1], [1, 0]] {
        let lines: [(Kind, [P2; 2], &[usize]); 3] = [lines[0], lines[1], (Kind::Ridge, [[5.0, 15.0], [30.0, 15.0]], &order)];
        let mut job = roof_job(&[(south.clone(), wrong_south), (north.clone(), wrong_north)], &lines).unwrap();
        for rake in ["line-0", "line-1"] {
            let d = length_of(&mut job, rake);
            assert_eq!(d.value, None, "{order:?} {rake}");
            assert!(d.note.contains("eave entered on the wrong edge"), "{}", d.note);
        }
    }
}

#[test]
fn gaps_under_a_line_add_up_and_bends_in_an_eave_do_not() {
    // A 10 ft rake edge with a hundred notches each 0.009 ft wide: 0.9 ft of it has no roof
    // under it. Each gap is inside the tolerance; together they are not.
    let mut outline: Vec<P2> = vec![[0.0, 0.0], [10.0, 0.0]];
    for k in 0..100 {
        let y = 0.05 + 0.1 * k as f64;
        outline.extend([[10.0, y], [9.5, y], [9.5, y + 0.009], [10.0, y + 0.009]]);
    }
    outline.extend([[10.0, 10.0], [0.0, 10.0]]);
    let mut job = roof_job(&[(outline, [[0.0, 0.0], [10.0, 0.0]])], &[(Kind::Rake, [[10.0, 0.0], [10.0, 10.0]], &[0])]).unwrap();
    let d = length_of(&mut job, "line-0");
    assert_eq!(d.value, None);
    assert!(d.note.contains("does not run along the outline"), "{}", d.note);

    // A low edge traced as 400 points on a sag 1 ft deep. Each point is within the
    // tolerance of the line between its neighbours; the edge as a whole is not straight,
    // and an eave drawn straight across it is refused.
    let mut sag: Vec<P2> = (0..=400).map(|k| {
        let x = 40.0 * k as f64 / 400.0;
        [x, 1.0 - (x - 20.0).powi(2) / 400.0]
    }).collect();
    sag[0] = [0.0, 0.0];
    sag[400] = [40.0, 0.0];
    sag.extend([[40.0, 20.0], [0.0, 20.0]]);
    let err = roof_job(&[(sag, [[0.0, 0.0], [40.0, 0.0]])], &[]).err().unwrap();
    assert!(err.contains("does not lie along any one edge"), "{err}");
    // Three pieces of one straight edge are still one edge, and an eave on the middle
    // piece, or across all three, is taken.
    let pieces = vec![[0.0, 0.0], [10.0, 0.0], [20.0, 0.0], [30.0, 0.0], [30.0, 10.0], [0.0, 10.0]];
    for eave in [[[10.0, 0.0], [20.0, 0.0]], [[0.0, 0.0], [30.0, 0.0]], [[5.0, 0.0], [25.0, 0.0]]] {
        let mut job = roof_job(&[(pieces.clone(), eave)], &[(Kind::Rake, [[30.0, 0.0], [30.0, 10.0]], &[0])]).unwrap();
        assert_eq!(length_of(&mut job, "line-0").value, Some(11.1803), "{eave:?}");
    }
}

#[test]
fn a_mistyped_key_of_ours_and_a_description_with_an_unseen_character_are_refused() {
    let text = save(&mut sample());
    let err = |t: String| load(&t).err().expect("should be refused");
    for key in [" pl::pitch", "pl: :pitch", "pI::pitch", "pl;;pitch", "pl\u{FF1A}\u{FF1A}pitch", "PL::PITCH", "pl::pich"] {
        let e = err(edit(&text, |v| {
            let pitch = attrs(v, "plane-0017").as_object_mut().unwrap().remove("pl::pitch").unwrap();
            attrs(v, "plane-0017")[key] = pitch;
        }));
        assert!(e.contains("did you mean \"pl::pitch\""), "{key:?}: {e}");
    }
    // Something that is plainly another program's is still kept.
    for key in ["other::pitch", "ifc::Pitch", "note", "pitch"] {
        assert!(load(&edit(&text, |v| attrs(v, "plane-0017")[key] = 3.into())).is_ok(), "{key}");
    }
    let line = |d: &str| serde_json::json!({"description": d, "quantity": "true_length * stretch_out", "stretch_out": {"value": 12.0, "unit": "in", "status": "Known", "source": "x"}, "unit_cost": {"value": null, "unit": "$/SF", "status": "Unknown", "source": "x"}});
    for unseen in ["Copper headwall flashing\u{200B}", "Copper\u{00AD} headwall flashing", "Copper headwall\tflashing"] {
        let e = err(edit(&text, |v| attrs(v, "headwall-0021")["pl::cost"]["lines"].as_array_mut().unwrap().push(line(unseen))));
        assert!(e.contains("nobody can see"), "{unseen:?}: {e}");
    }
}

#[test]
fn more_ways_a_drawing_writes_a_pitch_and_an_inch() {
    use units::{parse_length, parse_pitch, show_length};
    for (typed, rise) in [("1/4\"/ft", 0.25), ("1/4\":12", 0.25), ("1/4 in./ft", 0.25), ("6 in./12", 6.0), ("6 / 12", 6.0), ("6 :12", 6.0)] {
        assert_eq!(parse_pitch(typed).unwrap_or_else(|e| panic!("{typed:?} refused: {e}")), rise, "{typed}");
    }
    // Two apostrophes are an inch mark.
    assert_eq!(feet_of("6''"), 0.5);
    assert_eq!(feet_of("25'6''"), 25.5);
    // After feet the whole inches are written even when there are none.
    let (len, unit) = parse_length("25' 0 1/2\"").unwrap();
    assert_eq!(show_length(&len, unit), "25' 0 1/2\"");
    assert_eq!(show_length(&units::Len::One(-0.0), "ft"), "0 ft");
}

#[test]
fn what_lines_says_does_not_depend_on_which_plane_comes_first() {
    // Two planes whose shared edge is traced a hair (0.009 ft) out of line. The valley
    // between them is found, once, and no stretch of it is also called a rake.
    let a = (vec![[20.0, 40.0], [20.0, 20.0], [30.0, 10.0], [30.0, 30.0]], [[20.0, 40.0], [20.0, 20.0]]);
    let b = (vec![[20.0, 20.0], [10.0, 20.0], [25.0064, 15.0064]], [[20.0, 20.0], [10.0, 20.0]]);
    for planes in [[a.clone(), b.clone()], [b.clone(), a.clone()]] {
        let mut job = roof_job(&planes, &[(Kind::Valley, [[20.0, 20.0], [25.0, 15.0]], &[0, 1])]).unwrap();
        let (solid, _) = roof::solid_planes(&mut job);
        let found = roof::lines_of(&solid);
        assert_eq!(found.lines.iter().filter(|l| l.kind == Kind::Valley).count(), 1, "{:#?}", found.lines);
        assert!(!found.lines.iter().any(|l| l.kind != Kind::Valley && l.ends.contains(&[20.0, 20.0]) && l.ends.contains(&[25.0064, 15.0064])));
        let cmp = roof::compare(&mut job, &found);
        assert_eq!(cmp.agree.len(), 1, "{:?}", cmp.entered_only);
    }
}

#[test]
fn lines_says_what_is_so_about_overlaps_steps_and_lines_entered_twice() {
    let shed = |y0: f64, y1: f64| (vec![[0.0, y0], [12.0, y0], [12.0, y1], [0.0, y1]], [[0.0, y0], [12.0, y0]]);
    // The same plane entered twice: an overlap, and no valley invented along its eave.
    let mut job = roof_job(&[shed(0.0, 8.0), shed(0.0, 8.0)], &[]).unwrap();
    let (solid, _) = roof::solid_planes(&mut job);
    let found = roof::lines_of(&solid);
    assert!(found.lines.iter().all(|l| l.kind != Kind::Valley && l.planes.len() == 1), "{:#?}", found.lines);
    assert!(found.problems.iter().any(|p| p.contains("one overlaps the other")), "{:?}", found.problems);

    // A stepped roof: a lower shed whose top edge (y = 8, at 14 ft) is the plan line of an
    // upper shed's eave (at 10 ft here; both from 10 ft, so the lower top is 4 ft above).
    // One line on plan, two in the air. Each plane keeps its own edge and the entered
    // headwall and eave both agree.
    let mut job = roof_job(&[shed(0.0, 8.0), shed(8.0, 20.0)], &[(Kind::Headwall, [[0.0, 8.0], [12.0, 8.0]], &[0]), (Kind::Eave, [[0.0, 8.0], [12.0, 8.0]], &[1])]).unwrap();
    let (solid, _) = roof::solid_planes(&mut job);
    let found = roof::lines_of(&solid);
    assert_eq!(found.problems.len(), 1, "{:?}", found.problems);
    assert!(found.problems[0].contains("a step with a wall between them"));
    assert_eq!(roof::compare(&mut job, &found).agree.len(), 2);

    // One plane traced in two pieces is one plane.
    let halves = [(vec![[0.0, 0.0], [6.0, 0.0], [6.0, 8.0], [0.0, 8.0]], [[0.0, 0.0], [6.0, 0.0]]), (vec![[6.0, 0.0], [12.0, 0.0], [12.0, 8.0], [6.0, 8.0]], [[6.0, 0.0], [12.0, 0.0]])];
    let mut job = roof_job(&halves, &[]).unwrap();
    let (solid, _) = roof::solid_planes(&mut job);
    let found = roof::lines_of(&solid);
    assert!(found.problems.iter().any(|p| p.contains("one plane traced in two pieces")), "{:?}", found.problems);
    assert!(!found.problems.iter().any(|p| p.contains("change of pitch")));

    // On one shed: a wrong valley entered on the left edge before the right rake, the
    // eave entered twice, and half an eave. The right rake still agrees; each of the
    // others is told what it is.
    let lines: [(Kind, [P2; 2], &[usize]); 5] = [(Kind::Valley, [[0.0, 0.0], [0.0, 8.0]], &[0]), (Kind::Rake, [[0.0, 8.0], [0.0, 0.0]], &[0]), (Kind::Eave, [[0.0, 0.0], [12.0, 0.0]], &[0]), (Kind::Eave, [[12.0, 0.0], [0.0, 0.0]], &[0]), (Kind::Eave, [[0.0, 0.0], [6.0, 0.0]], &[0])];
    let mut job = roof_job(&[shed(0.0, 8.0)], &lines).unwrap();
    let (solid, _) = roof::solid_planes(&mut job);
    let found = roof::lines_of(&solid);
    let cmp = roof::compare(&mut job, &found);
    assert_eq!(cmp.agree.iter().map(|a| a.1).collect::<Vec<_>>(), [Kind::Rake, Kind::Eave]);
    assert!(cmp.entered_only.iter().any(|e| e.contains("entered as a Valley") && e.contains("a Rake")), "{:?}", cmp.entered_only);
    assert!(cmp.entered_only.iter().any(|e| e.contains("entered twice")));
    assert!(cmp.entered_only.iter().any(|e| e.contains("covers only part of the Eave")));
    assert!(!cmp.entered_only.iter().any(|e| e.contains("no edge of any plane runs there")));
}

// ---- Regression checks for what the fifth independent judgment found ----

#[test]
fn a_count_below_zero_is_refused_not_rounded_up_to_nothing() {
    let mut job = parapet_job();
    // A spacing typed with a minus: 20 x 2 / (-12 in / 12) = -40, not zero fasteners. Since call 208 (point 4) a length below nothing is
    // refused as it is entered, so the typo never reaches the quantity; the quantity's own refusal below is held by the formula that follows.
    let err = job.set_on_line("parapet-0024", "spacing", Some("Cleat fasteners"), Some("-12 in"), Status::Known, "typo").unwrap_err();
    assert!(err.contains("spacing is below nothing (-12 in): a length a line enters is nothing or more"), "{err}");
    // The same without a typo: a formula that takes off more than there is.
    // (20 - 2 x 17) x 2 / (16 in / 12) = -21.
    job.set_formula("parapet-0024", Some("Cleat fasteners"), "(plan_length - 2 * end_gap) * sides / spacing", &[("end_gap".to_string(), "17 ft".to_string()), ("spacing".to_string(), "16 in".to_string())], Status::Known, "x").unwrap();
    job.derive();
    let f = figures(&mut job, "parapet-0024", "Cleat fasteners").quantity;
    assert_eq!(f.value, None);
    assert!(f.note.contains("below zero"), "{}", f.note);
    // With a 3 in gap each end: (20 - 0.5) x 2 / 1.3333 = 29.25, so 30.
    job.set_on_line("parapet-0024", "end_gap", Some("Cleat fasteners"), Some("3 in"), Status::Known, "x").unwrap();
    job.derive();
    assert_eq!(figures(&mut job, "parapet-0024", "Cleat fasteners").quantity.value, Some(30.0));
    // A count too small to tell from zero is neither zero nor one.
    job.set_formula("parapet-0024", Some("Cleat fasteners"), "plan_length / plan_length * 0.0000000001", &[], Status::Known, "x").unwrap();
    job.derive();
    let f = figures(&mut job, "parapet-0024", "Cleat fasteners").quantity;
    assert_eq!(f.value, None);
    assert!(f.note.contains("too small to tell from zero"), "{}", f.note);
    // An exact count is not pushed up by a crumb, and a real zero is zero.
    job.set_formula("parapet-0024", Some("Cleat fasteners"), "plan_length * 0.1 * 3 / 0.3 / plan_length", &[], Status::Known, "x").unwrap();
    job.derive();
    assert_eq!(figures(&mut job, "parapet-0024", "Cleat fasteners").quantity.value, Some(1.0));
    job.set_formula("parapet-0024", Some("Cleat fasteners"), "(plan_length - plan_length) / plan_length", &[], Status::Known, "x").unwrap();
    job.derive();
    assert_eq!(figures(&mut job, "parapet-0024", "Cleat fasteners").quantity.value, Some(0.0));
    // A count with waste on it is still a count: 40 fasteners + 5 percent = 42.
    job.set_formula("parapet-0024", Some("Cleat fasteners"), "plan_length * sides / spacing", &[("sides".to_string(), "2".to_string()), ("spacing".to_string(), "12 in".to_string())], Status::Known, "x").unwrap();
    job.set_on_line("parapet-0024", "waste", Some("Cleat fasteners"), Some("5"), Status::Known, "x").unwrap();
    job.derive();
    let f = figures(&mut job, "parapet-0024", "Cleat fasteners");
    assert_eq!((f.quantity.value, f.with_waste.unwrap().value), (Some(40.0), Some(42.0)));
}

#[test]
fn a_line_refused_for_one_plane_is_still_evidence_about_another() {
    // The roof of the fourth judgment's case: the south plane's eave on the wrong edge,
    // the ridge naming both planes. Here the ridge is refused for the north plane before
    // its slope on the south plane is looked at: the north plane's ridge edge is traced
    // 0.05 ft off. The south plane's rake still loses its wrong length, whichever plane
    // the ridge names first.
    let south = (vec![[0.0, 0.0], [30.0, 0.0], [30.0, 15.0], [5.0, 15.0]], [[30.0, 0.0], [30.0, 15.0]]);
    let north = (vec![[0.0, 30.0], [30.0, 30.0], [30.0, 15.05], [5.0, 15.05]], [[0.0, 30.0], [30.0, 30.0]]);
    for order in [[0usize, 1], [1, 0]] {
        let lines: [(Kind, [P2; 2], &[usize]); 2] = [(Kind::Rake, [[0.0, 0.0], [5.0, 15.0]], &[0]), (Kind::Ridge, [[5.0, 15.0], [30.0, 15.0]], &order)];
        let mut job = roof_job(&[south.clone(), north.clone()], &lines).unwrap();
        assert!(length_of(&mut job, "line-1").value.is_none());
        let rake = length_of(&mut job, "line-0");
        assert_eq!(rake.value, None, "{order:?}");
        assert!(rake.note.contains("eave entered on the wrong edge"), "{}", rake.note);
    }
    // And when the other plane named does not exist at all.
    let mut job = roof_job(&[south.clone()], &[(Kind::Rake, [[0.0, 0.0], [5.0, 15.0]], &[0]), (Kind::Ridge, [[5.0, 15.0], [30.0, 15.0]], &[0])]).unwrap();
    let text = edit(&save(&mut job), |v| attrs(v, "ridge-0004")["pl::run"]["planes"] = serde_json::json!(["plane-9999", "plane-0002"]));
    let (mut job, _) = load(&text).unwrap();
    assert!(measure(&mut job, "ridge-0004", "true_length").note.contains("not in the job"));
    assert_eq!(measure(&mut job, "rake-0003", "true_length").value, None);
}

#[test]
fn an_element_is_one_condition_and_a_recipe_carries_no_stretch_out() {
    // The south plane is already "Standing seam copper, 16 oz". A second condition is not
    // written over it.
    let text = save(&mut sample());
    let with_recipe = edit(&text, |v| {
        let mut recipe = attrs(v, "condition-0026").clone();
        recipe["pl::recipe"]["applies_to"] = serde_json::json!(["RoofPlane"]);
        recipe["pl::recipe"]["cost"]["lines"] = serde_json::json!([{"description": "Underlayment", "quantity": "surface_area", "unit_cost": {"value": null, "unit": "$/SF", "status": "Unknown", "source": "x"}}]);
        recipe["pl::recipe"]["cost"]["code"] = attrs(v, "plane-0003")["pl::cost"]["code"].clone();
        v["data"].as_array_mut().unwrap().push(serde_json::json!({"path": "condition-0028", "attributes": recipe}));
        entry(v, "job-0001")["children"]["Underlayment only"] = "condition-0028".into();
        attrs(v, "job-0001")["pl::job"]["next_id"] = 29.into();
    });
    let (mut job, _) = load(&with_recipe).unwrap();
    let before = save(&mut job);
    let e = job.apply("plane-0003", "Underlayment only", "x").unwrap_err();
    assert!(e.contains("is already \"Standing seam copper, 16 oz\""), "{e}");
    assert_eq!(save(&mut job), before);
    // A recipe with a stretch-out filled in is refused: it belongs to the detail.
    let e = load(&edit(&text, |v| attrs(v, "condition-0026")["pl::recipe"]["cost"]["lines"][0]["stretch_out"] = serde_json::json!({"value": 12.0, "unit": "in", "status": "Known", "source": "x"}))).err().unwrap();
    assert!(e.contains("belongs to the detail"), "{e}");
    // What else a recipe states is shown where the recipe is read.
    let mut job = sample();
    let notes = plumbline_world::view::elements(&mut job).into_iter().find(|e| e.path == "condition-0025").unwrap().notes;
    assert!(notes.iter().any(|n| n.contains("Continuous cleat") && n.contains("sides: 2") && n.contains("price: blank")), "{notes:?}");
    assert!(notes.iter().any(|n| n.contains("Copper coping cap") && n.contains("stretch-out: blank") && n.contains("waste: blank")), "{notes:?}");
}

#[test]
fn a_formula_too_long_to_read_is_refused_and_a_factor_is_not_a_count() {
    // 2,500 terms once ran the program out of stack. It is refused at 200 tokens.
    let long = vec!["plan_length"; 2500].join(" + ");
    assert!(formula::parse(&long).unwrap_err().contains("holds at most 200"));
    let text = save(&mut sample());
    let e = load(&edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["lines"][0]["quantity"] = long.clone().into())).err().unwrap();
    assert!(e.contains("holds at most 200"), "{}", &e[e.len().saturating_sub(120)..]);
    let just_fits = vec!["plan_area"; 100].join(" + ");
    assert!(formula::parse(&just_fits).is_ok());
    // The minus a worked formula is written with can be typed back.
    assert!(formula::parse("plan_length \u{2212} 2").is_ok());
    // A factor times one is still a factor.
    let e = load(&edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["lines"][0]["quantity"] = "slope_factor * 1".into())).err().unwrap();
    assert!(e.contains("a factor, not a quantity"), "{e}");
    // Another program's key that only resembles ours is kept; a slip of ours is not.
    for key in ["ex::cost", "my::job", "ui::run", "op::pitch", "x::job", "ifc::class"] {
        assert!(load(&edit(&text, |v| attrs(v, "plane-0017")[key] = 3.into())).is_ok(), "{key}");
    }
    let e = load(&edit(&text, |v| attrs(v, "plane-0017")["pI::cost"] = 3.into())).err().unwrap();
    assert!(e.contains("spelled like one") && !e.contains("own prefix"), "{e}");
    // An order conversion says whether whole ones are bought, and its unit is one word.
    let mut job = parapet_job();
    job.set_on_line("parapet-0024", "stretch_out", Some("Copper coping cap"), Some("19 1/2 in"), Status::Known, "x").unwrap();
    job.set_on_line("parapet-0024", "waste", Some("Copper coping cap"), Some("0"), Status::Known, "x").unwrap();
    job.set_on_line("parapet-0024", "order", Some("Copper coping cap"), Some("0.5 SF/lb exact"), Status::Known, "x").unwrap();
    job.derive();
    assert_eq!(figures(&mut job, "parapet-0024", "Copper coping cap").order.unwrap().value, Some(65.0));
    assert!(job.set_on_line("parapet-0024", "order", Some("Copper coping cap"), Some("30 SF/she et"), Status::Known, "x").is_err());
}

// ---- D2: a roof solved from its footprint ----
