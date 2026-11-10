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
    let run = job.add("job-0001", Kind::Parapet, "far");
    job.put(&run, Run { ends: Val::known([[5280.0, 5280.0], [5290.0 + sixteenth, 5280.0]], "ft", "test"), planes: vec![] });
    job.derive();
    let (mut loaded, _) = load(&save(&mut job)).unwrap();
    let e = loaded.find(&run).unwrap();
    let [a, b] = loaded.world.get::<Run>(e).unwrap().ends.value.unwrap();
    assert!(((b[0] - a[0]) - (10.0 + sixteenth)).abs() < 1e-9);
    assert_eq!(measure(&mut loaded, &run, "plan_length").value, Some(10.0052));
}
