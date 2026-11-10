//! Checks on the world model. The figures asserted here are worked by hand in the comments.

use plumbline_world::file::{load, save};
use plumbline_world::sample::sample;
use plumbline_world::value::Status;
use plumbline_world::*;
use serde_json::Value;

fn entries(text: &str) -> Vec<(String, Value)> {
    let v: Value = serde_json::from_str(text).unwrap();
    v["data"].as_array().unwrap().iter().map(|e| (e["path"].as_str().unwrap().to_string(), e.clone())).collect()
}

fn touched(before: &str, after: &str) -> Vec<String> {
    let (a, b) = (entries(before), entries(after));
    assert_eq!(a.len(), b.len());
    a.iter().zip(&b).filter(|(x, y)| x.1 != y.1).map(|(x, _)| x.0.clone()).collect()
}

fn measure(job: &mut Job, path: &str, name: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).unwrap().0[name].clone()
}

#[test]
fn a_saved_job_loads_and_saves_back_to_the_same_bytes() {
    let text = save(&mut sample());
    let (mut job, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut job), text);
}

#[test]
fn the_file_has_one_entry_per_element() {
    let text = save(&mut sample());
    let paths: Vec<String> = entries(&text).into_iter().map(|e| e.0).collect();
    let mut unique = paths.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(paths.len(), unique.len());
    assert_eq!(paths.len(), 24);
}

#[test]
fn hand_worked_figures() {
    let mut job = sample();
    // South plane: trapezoid, parallel sides 40 and 10, 15 deep: (40 + 10) / 2 x 15 = 375 SF on plan.
    // 6:12 slope factor = sqrt(1 + 0.25) = 1.118034. Surface = 419.2627 SF.
    assert_eq!(measure(&mut job, "plane-0003", "plan_area").value, Some(375.0));
    assert_eq!(measure(&mut job, "plane-0003", "slope_factor").value, Some(1.118));
    assert_eq!(measure(&mut job, "plane-0003", "surface_area").value, Some(419.2627));
    // East plane: triangle, base 30, 15 deep: 225 SF on plan.
    assert_eq!(measure(&mut job, "plane-0004", "plan_area").value, Some(225.0));
    // The four planes cover the 40 x 30 footprint: 375 + 225 + 375 + 225 = 1200.
    // Hip: 15 over and 15 up on plan = 21.2132 LF; it climbs 15 ft of run at 6:12 = 7.5 ft;
    // sqrt(450 + 56.25) = 22.5 LF on the slope.
    assert_eq!(measure(&mut job, "hip-0012", "plan_length").value, Some(21.2132));
    assert_eq!(measure(&mut job, "hip-0012", "true_length").value, Some(22.5));
    // The ridge is level: 10 LF either way.
    assert_eq!(measure(&mut job, "ridge-0011", "true_length").value, Some(10.0));
    // Porch rake: 8 ft of run at 3:12 climbs 2 ft; sqrt(64 + 4) = 8.2462.
    assert_eq!(measure(&mut job, "rake-0019", "true_length").value, Some(8.2462));
}

#[test]
fn changing_one_pitch_touches_only_that_plane_and_what_is_derived_from_it() {
    let mut job = sample();
    let before = save(&mut job);
    job.set("plane-0017", "pitch", Some("4"), Status::Known, "field measured").unwrap();
    let changed = job.derive();
    let after = save(&mut job);
    let expect = ["plane-0017", "eave-0018", "rake-0019", "rake-0020", "headwall-0021"];
    assert_eq!(touched(&before, &after), expect);
    assert_eq!(changed, expect);
    // On the plane itself only the pitch and the measure moved.
    let (a, b) = (entries(&before), entries(&after));
    let (a, b) = (&a.iter().find(|e| e.0 == "plane-0017").unwrap().1["attributes"], &b.iter().find(|e| e.0 == "plane-0017").unwrap().1["attributes"]);
    for (key, value) in a.as_object().unwrap() {
        let same = value == &b[key];
        assert_eq!(same, !(key == "pl::pitch" || key == "pl::measure"), "{key}");
    }
}

#[test]
fn a_derived_value_takes_the_worst_status_of_its_inputs() {
    let mut job = sample();
    // The porch pitch is Assumed, so everything that leans on it is Assumed.
    assert_eq!(measure(&mut job, "plane-0017", "plan_area").status, Status::Known);
    assert_eq!(measure(&mut job, "plane-0017", "surface_area").status, Status::Assumed);
    assert_eq!(measure(&mut job, "rake-0019", "true_length").status, Status::Assumed);
    // The eave height is not an input to an area, so it does not flag one.
    // The parapet height is Unknown: its length is known, its face area is not, and no number is given.
    assert_eq!(measure(&mut job, "parapet-0024", "plan_length").value, Some(20.0));
    let face = measure(&mut job, "parapet-0024", "face_area");
    assert_eq!((face.value, face.status), (None, Status::Unknown));
    // Enter the height and the face area appears: 20 LF x 2.5 ft = 50 SF, Assumed.
    job.set("parapet-0024", "height", Some("2.5"), Status::Assumed, "typical; confirm on site").unwrap();
    job.derive();
    let face = measure(&mut job, "parapet-0024", "face_area");
    assert_eq!((face.value, face.status), (Some(50.0), Status::Assumed));
    // Take it away again and the number goes with it.
    job.set("parapet-0024", "height", None, Status::Unknown, "not on plan").unwrap();
    job.derive();
    assert_eq!(measure(&mut job, "parapet-0024", "face_area").value, None);
}

#[test]
fn planes_that_disagree_about_a_hip_give_no_length() {
    let mut job = sample();
    let before = save(&mut job);
    // Steepen the south plane only. The plan linework was drawn for 6:12 all round, so the
    // hips and the ridge it shares no longer fit: south puts their tops at 20 ft, the others at 17.5.
    job.set("plane-0003", "pitch", Some("8"), Status::Known, "revised").unwrap();
    job.derive();
    let after = save(&mut job);
    assert_eq!(touched(&before, &after), ["plane-0003", "ridge-0011", "hip-0012", "hip-0013"]);
    for hip in ["ridge-0011", "hip-0012", "hip-0013"] {
        let d = measure(&mut job, hip, "true_length");
        assert_eq!((d.value, d.status), (None, Status::Unknown));
        assert!(d.note.contains("disagree"), "{}", d.note);
    }
    // The south eave is level on the south plane whatever its pitch: still 40 LF.
    assert_eq!(measure(&mut job, "eave-0007", "true_length").value, Some(40.0));
    // The hips on the far side never touched the south plane.
    assert_eq!(measure(&mut job, "hip-0014", "true_length").value, Some(22.5));
}

fn edit(text: &str, f: impl Fn(&mut Value)) -> String {
    let mut v: Value = serde_json::from_str(text).unwrap();
    f(&mut v);
    serde_json::to_string(&v).unwrap()
}

fn entry<'a>(v: &'a mut Value, path: &str) -> &'a mut Value {
    v["data"].as_array_mut().unwrap().iter_mut().find(|e| e["path"] == path).unwrap()
}

#[test]
fn loading_refuses_values_that_break_the_rules() {
    let text = save(&mut sample());
    let err = |t: String| load(&t).err().expect("should be refused");
    // No source.
    assert!(err(edit(&text, |v| entry(v, "plane-0003")["attributes"]["pl::pitch"]["source"] = "".into())).contains("no source"));
    // Unknown that carries a number.
    assert!(err(edit(&text, |v| entry(v, "parapet-0024")["attributes"]["pl::height"]["value"] = 3.0.into())).contains("Unknown but carries a value"));
    // Known with nothing in it.
    assert!(err(edit(&text, |v| entry(v, "plane-0003")["attributes"]["pl::pitch"]["value"] = Value::Null)).contains("carries no value"));
    // A negative pitch.
    assert!(err(edit(&text, |v| entry(v, "plane-0003")["attributes"]["pl::pitch"]["value"] = (-6.0).into())).contains("zero or more"));
    // The same element twice.
    assert!(err(edit(&text, |v| {
        let dup = entry(v, "plane-0003").clone();
        v["data"].as_array_mut().unwrap().push(dup);
    }))
    .contains("appears twice"));
    // A child that is not there.
    assert!(err(edit(&text, |v| entry(v, "roof-0002")["children"]["ghost"] = "plane-9999".into())).contains("not in the file"));
    // An element with no kind.
    assert!(err(edit(&text, |v| {
        entry(v, "plane-0003")["attributes"].as_object_mut().unwrap().remove("pl::class");
    }))
    .contains("pl::class"));
}

#[test]
fn a_measure_edited_in_the_file_is_caught_and_not_used() {
    let text = save(&mut sample());
    let forged = edit(&text, |v| entry(v, "plane-0003")["attributes"]["pl::measure"]["surface_area"]["value"] = 999.0.into());
    let (mut job, report) = load(&forged).unwrap();
    assert_eq!(report.stale, ["plane-0003"]);
    assert_eq!(measure(&mut job, "plane-0003", "surface_area").value, Some(419.2627));
}

#[test]
fn attributes_this_program_does_not_know_survive_a_round_trip() {
    let text = save(&mut sample());
    let with_extra = edit(&text, |v| entry(v, "plane-0003")["attributes"]["other::note"] = serde_json::json!({"keep": [1, 2, 3]}));
    let (mut job, _) = load(&with_extra).unwrap();
    let again = save(&mut job);
    let v: Value = serde_json::from_str(&again).unwrap();
    let kept = &v["data"].as_array().unwrap().iter().find(|e| e["path"] == "plane-0003").unwrap()["attributes"]["other::note"];
    assert_eq!(kept, &serde_json::json!({"keep": [1, 2, 3]}));
}

#[test]
fn a_sixteenth_of_an_inch_survives_a_mile_from_the_origin() {
    // Plan coordinates are 64-bit. A line 5,280 ft from the origin, made 1/16 in longer,
    // still measures 1/16 in longer after a save and a load.
    let sixteenth = 1.0 / 192.0;
    let mut job = Job::new("Far corner", "test", "test");
    let run = job.add("job-0001", Kind::Parapet, "far").unwrap();
    job.put(&run, Run { ends: Val::known([[5280.0, 5280.0], [5290.0 + sixteenth, 5280.0]], "ft", "test"), planes: vec![] }).unwrap();
    job.derive();
    let (mut loaded, _) = load(&save(&mut job)).unwrap();
    let e = loaded.find(&run).unwrap();
    let [a, b] = loaded.world.get::<Run>(e).unwrap().ends.value.unwrap();
    assert!(((b[0] - a[0]) - (10.0 + sixteenth)).abs() < 1e-9);
    assert_eq!(measure(&mut loaded, &run, "plan_length").value, Some(10.0052));
}

// ---- Regression checks for what the first independent judgment found ----

fn attrs<'a>(v: &'a mut Value, path: &str) -> &'a mut Value {
    &mut entry(v, path)["attributes"]
}

fn show_rows(job: &mut Job, path: &str) -> Vec<(String, String, Status)> {
    plumbline_world::view::elements(job).into_iter().find(|e| e.path == path).unwrap().rows.into_iter().map(|r| (r.label, r.text, r.status)).collect()
}

#[test]
fn reordering_an_outline_does_not_move_the_eave() {
    // The same porch rectangle traced from a different corner, and traced clockwise.
    // The eave is its own line, so the eave is still the eave and the rakes still climb.
    let text = save(&mut sample());
    for pts in [serde_json::json!([[56.0, 0.0], [56.0, 8.0], [44.0, 8.0], [44.0, 0.0]]), serde_json::json!([[44.0, 0.0], [44.0, 8.0], [56.0, 8.0], [56.0, 0.0]])] {
        let (mut job, _) = load(&edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["points"]["value"] = pts.clone())).unwrap();
        assert_eq!(measure(&mut job, "eave-0018", "true_length").value, Some(12.0));
        assert_eq!(measure(&mut job, "rake-0019", "true_length").value, Some(8.2462));
        assert_eq!(measure(&mut job, "plane-0017", "surface_area").value, Some(98.9545));
    }
}

#[test]
fn an_eave_that_is_not_level_on_its_plane_is_flagged() {
    // Enter the porch plane's east side as its eave. The element called the eave now
    // climbs, which an eave cannot do, and the rakes no longer fit either.
    let text = save(&mut sample());
    let (mut job, _) = load(&edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["eave"]["value"] = serde_json::json!([[56.0, 0.0], [56.0, 8.0]]))).unwrap();
    let d = measure(&mut job, "eave-0018", "true_length");
    assert_eq!((d.value, d.status), (None, Status::Unknown));
    assert!(d.note.contains("an eave is level"), "{}", d.note);
    // The rakes come out level and the headwall climbs, so they are flagged too: nothing
    // on a plane whose eave is on the wrong edge is left looking Known.
    for (line, says) in [("rake-0019", "a rake climbs"), ("rake-0020", "a rake climbs"), ("headwall-0021", "a headwall is level")] {
        let d = measure(&mut job, line, "true_length");
        assert_eq!(d.value, None, "{line}");
        assert!(d.note.contains(says), "{line}: {}", d.note);
    }
    // An eave line that is not along any edge of the outline is refused outright.
    let err = load(&edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["eave"]["value"] = serde_json::json!([[44.0, 0.0], [56.0, 8.0]]))).err().unwrap();
    assert!(err.contains("does not lie along any one edge"), "{err}");
}

#[test]
fn a_value_in_the_wrong_unit_is_refused_not_read_as_feet() {
    let text = save(&mut sample());
    let err = |t: String| load(&t).err().expect("should be refused");
    // A parapet height typed in inches.
    assert!(err(edit(&text, |v| attrs(v, "parapet-0024")["pl::height"] = serde_json::json!({"value": 30.0, "unit": "in", "status": "Known", "source": "x"}))).contains("has to be \"ft\""));
    // An outline in metres.
    assert!(err(edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["points"]["unit"] = "m".into())).contains("has to be \"ft\""));
    // A pitch in degrees.
    assert!(err(edit(&text, |v| attrs(v, "plane-0017")["pl::pitch"]["unit"] = "deg".into())).contains("has to be \"in/12\""));
    // No unit at all.
    assert!(err(edit(&text, |v| {
        attrs(v, "plane-0017")["pl::pitch"].as_object_mut().unwrap().remove("unit");
    }))
    .contains("has to be \"in/12\""));
    // A job declared in metres.
    assert!(err(edit(&text, |v| attrs(v, "job-0001")["pl::job"]["length_unit"] = "m".into())).contains("works in \"ft\" only"));
    // A unit cost priced per LF against an SF quantity.
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["lines"][0]["unit_cost"]["unit"] = "$/LF".into())).contains("has to be \"$/SF\""));
}

#[test]
fn a_loop_of_children_is_refused() {
    let text = save(&mut sample());
    // The main roof hung under one of its own planes instead of under the job.
    let err = load(&edit(&text, |v| {
        entry(v, "job-0001")["children"].as_object_mut().unwrap().remove("main-roof");
        entry(v, "plane-0003")["children"] = serde_json::json!({"loop": "roof-0002"});
    }))
    .err()
    .unwrap();
    assert!(err.contains("loop of children"), "{err}");
    // An element that is its own child.
    let err = load(&edit(&text, |v| {
        entry(v, "roof-0016")["children"].as_object_mut().unwrap().remove("shed");
        entry(v, "plane-0017")["children"] = serde_json::json!({"me": "plane-0017"});
    }))
    .err()
    .unwrap();
    assert!(err.contains("loop of children"), "{err}");
    // A second job element, and a first element that is not the job.
    assert!(load(&edit(&text, |v| attrs(v, "roof-0002")["pl::class"] = "Job".into())).err().unwrap().contains("second job element"));
    assert!(load(&edit(&text, |v| attrs(v, "roof-0002")["pl::job"] = serde_json::json!({"title": "x", "length_unit": "ft", "next_id": 9}))).err().unwrap().contains("only the job element"));
    assert!(load(&edit(&text, |v| attrs(v, "job-0001")["pl::class"] = "Roof".into())).err().unwrap().contains("has to be the job"));
}

#[test]
fn nothing_is_dropped_quietly() {
    let text = save(&mut sample());
    // Kept and written back: extra keys at the top, in the header, on an entry, and IFC5's inherits.
    let with_extras = edit(&text, |v| {
        v["notes"] = "top-level note".into();
        v["header"]["project_number"] = "24-017".into();
        v["schemas"]["other::thing"] = serde_json::json!({"value": {"dataType": "String"}});
        entry(v, "plane-0003")["inherits"] = serde_json::json!({"type": "plane-0004"});
        entry(v, "plane-0003")["tag"] = serde_json::json!([1, 2]);
    });
    let (mut job, _) = load(&with_extras).unwrap();
    job.set("plane-0004", "condition", Some("Slate"), Status::Known, "revised").unwrap();
    let again: Value = serde_json::from_str(&save(&mut job)).unwrap();
    assert_eq!(again["notes"], "top-level note");
    assert_eq!(again["header"]["project_number"], "24-017");
    assert_eq!(again["schemas"]["other::thing"]["value"]["dataType"], "String");
    let e = again["data"].as_array().unwrap().iter().find(|e| e["path"] == "plane-0003").unwrap();
    assert_eq!(e["inherits"]["type"], "plane-0004");
    assert_eq!(e["tag"], serde_json::json!([1, 2]));
    // Refused by name: an unknown field inside one of this program's own components.
    // A price with no status cannot ride in on a cost line.
    let err = |t: String| load(&t).err().expect("should be refused");
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["lines"][0]["extended_total"] = 41926.27.into())).contains("extended_total"));
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::pitch"]["checked_by"] = "x".into())).contains("checked_by"));
    assert!(err(edit(&text, |v| attrs(v, "job-0001")["pl::job"]["client"] = "x".into())).contains("client"));
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["code"]["value"]["division_note"] = "x".into())).contains("division_note"));
}

#[test]
fn a_coordinate_comes_back_as_the_same_number() {
    // A figure with more digits than fit neatly: it has to survive a load and a save exactly.
    let text = save(&mut sample());
    let raw = text.replace("[44.0, 0.0], [56.0, 0.0], [56.0, 8.0], [44.0, 8.0]", "[44.0, 0.0], [56.0, 0.0], [56.0, 8.000000000000002], [44.0, 8.0]");
    assert_ne!(raw, text);
    let (mut job, _) = load(&raw).unwrap();
    assert!(save(&mut job).contains("8.000000000000002"));
}

#[test]
fn a_missing_component_shows_as_unknown_instead_of_vanishing() {
    let text = save(&mut sample());
    // Parapet with its height deleted from the file: the face area is still listed, Unknown.
    let (mut job, _) = load(&edit(&text, |v| {
        attrs(v, "parapet-0024").as_object_mut().unwrap().remove("pl::height");
    }))
    .unwrap();
    let d = measure(&mut job, "parapet-0024", "face_area");
    assert_eq!((d.value, d.status, d.note.as_str()), (None, Status::Unknown, "no height entered"));
    // Plane with its pitch deleted: plan area still given, the rest listed as Unknown, and
    // its lines say what the plane lacks instead of calling it absent.
    let (mut job, _) = load(&edit(&text, |v| {
        attrs(v, "plane-0017").as_object_mut().unwrap().remove("pl::pitch");
    }))
    .unwrap();
    assert_eq!(measure(&mut job, "plane-0017", "plan_area").value, Some(96.0));
    assert_eq!(measure(&mut job, "plane-0017", "surface_area").note, "no pitch entered");
    assert!(measure(&mut job, "rake-0019", "true_length").note.contains("plane plane-0017 has no pitch entered"));
    assert!(show_rows(&mut job, "plane-0017").iter().any(|(l, t, s)| l == "surface_area" && t == "unknown" && *s == Status::Unknown));
}

#[test]
fn numbers_too_large_to_mean_anything_are_refused() {
    let mut job = sample();
    assert!(job.set("plane-0017", "pitch", Some("1e300"), Status::Known, "typo").unwrap_err().contains("is a wall"));
    assert!(job.set("plane-0017", "pitch", Some("241"), Status::Known, "typo").is_err());
    assert!(job.set("plane-0017", "pitch", Some("-0"), Status::Known, "flat").is_ok());
    let text = save(&mut sample());
    let err = load(&edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["points"]["value"] = serde_json::json!([[0.0, 0.0], [1e200, 0.0], [1e200, 1e200]]))).err().unwrap();
    assert!(err.contains("not a usable number"), "{err}");
}

#[test]
fn outlines_and_lines_that_make_no_sense_are_refused_or_flagged() {
    let text = save(&mut sample());
    let err = |t: String| load(&t).err().expect("should be refused");
    // A bowtie.
    assert!(err(edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["points"]["value"] = serde_json::json!([[44.0, 0.0], [56.0, 0.0], [44.0, 8.0], [56.0, 8.0]]))).contains("cross or touch"));
    // Three points in a row.
    assert!(err(edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["points"]["value"] = serde_json::json!([[44.0, 0.0], [50.0, 0.0], [56.0, 0.0]]))).contains("outline"));
    // A hip carrying an outline.
    assert!(err(edit(&text, |v| attrs(v, "hip-0012")["pl::outline"] = attrs(&mut serde_json::from_str(&text).unwrap(), "plane-0003")["pl::outline"].clone())).contains("cannot carry an outline"));
    // A ridge naming a plane it does not lie on: flagged, no length.
    let (mut job, _) = load(&edit(&text, |v| attrs(v, "ridge-0011")["pl::run"]["planes"] = serde_json::json!(["plane-0017"]))).unwrap();
    assert!(measure(&mut job, "ridge-0011", "true_length").note.contains("does not run along the outline"));
    // A hip naming one plane, and a line naming another line as its plane.
    let (mut job, _) = load(&edit(&text, |v| attrs(v, "hip-0012")["pl::run"]["planes"] = serde_json::json!(["plane-0003"]))).unwrap();
    assert!(measure(&mut job, "hip-0012", "true_length").note.contains("lies on 2 plane(s)"));
    let (mut job, _) = load(&edit(&text, |v| attrs(v, "rake-0019")["pl::run"]["planes"] = serde_json::json!(["hip-0012"]))).unwrap();
    assert!(measure(&mut job, "rake-0019", "true_length").note.contains("is a Hip"));
}

#[test]
fn cost_lines_have_to_make_sense() {
    let text = save(&mut sample());
    let err = |t: String| load(&t).err().expect("should be refused");
    let line = |v: &mut Value, key: &str, val: Value| attrs(v, "plane-0003")["pl::cost"]["lines"][0][key] = val;
    assert!(err(edit(&text, |v| line(v, "quantity", "volume".into()))).contains("does not have"));
    assert!(err(edit(&text, |v| line(v, "quantity", "slope_factor".into()))).contains("a factor"));
    assert!(err(edit(&text, |v| line(v, "description", "".into()))).contains("no description"));
    assert!(err(edit(&text, |v| line(v, "unit_cost", serde_json::json!({"value": -1e9, "unit": "$/SF", "status": "Known", "source": "x"})))).contains("zero or more"));
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["code"]["value"]["code"] = "".into())).contains("scheme and a code"));
}

#[test]
fn two_children_of_one_parent_cannot_share_a_name() {
    let mut job = sample();
    assert!(job.add("roof-0002", Kind::Hip, "ridge").unwrap_err().contains("already has a child"));
    assert!(job.add("nowhere", Kind::Hip, "x").is_err());
    assert!(job.add("roof-0002", Kind::Job, "x").is_err());
    // And what add does build still loads.
    job.add("roof-0002", Kind::Valley, "valley-1").unwrap();
    let text = save(&mut job);
    assert!(load(&text).is_ok());
}

// ---- Regression checks for what the second independent judgment found ----

fn porch(v: &mut Value, points: Value, eave: Value) {
    attrs(v, "plane-0017")["pl::outline"]["points"]["value"] = points;
    attrs(v, "plane-0017")["pl::outline"]["eave"]["value"] = eave;
}

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
    assert_eq!(path, "plane-0025");
    assert!(load(&save(&mut job)).is_ok());
}

#[test]
fn a_parapet_names_no_plane() {
    let text = save(&mut sample());
    let err = load(&edit(&text, |v| attrs(v, "parapet-0024")["pl::run"]["planes"] = serde_json::json!(["lowslope-0023"]))).err().unwrap();
    assert!(err.contains("a parapet lies on no plane"), "{err}");
}
