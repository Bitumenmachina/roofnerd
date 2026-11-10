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
    job.set("parapet-0024", "height", Some("2.5 ft"), Status::Assumed, "typical; confirm on site").unwrap();
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
fn a_unit_the_program_does_not_know_is_refused_by_name_never_read_as_feet() {
    let text = save(&mut sample());
    let err = |t: String| load(&t).err().expect("should be refused");
    let height = |unit: &str| edit(&text, |v| attrs(v, "parapet-0024")["pl::height"] = serde_json::json!({"value": 30.0, "unit": unit, "status": "Known", "source": "x"}));
    // Metric is refused, and called metric.
    for unit in ["m", "mm", "cm", "metres"] {
        let e = err(height(unit));
        assert!(e.contains(&format!("{unit:?}")) && e.contains("metric"), "{e}");
    }
    // A unit nobody has heard of is refused by name.
    let e = err(height("cubits"));
    assert!(e.contains("\"cubits\"") && e.contains("not a unit this program knows"), "{e}");
    // Yards, and feet spelled some other way: the file's units are exactly ft, in, ft-in.
    assert!(err(height("yd")).contains("\"yd\""));
    assert!(err(height("FT")).contains("\"FT\""));
    // No unit at all is not feet.
    let e = err(edit(&text, |v| attrs(v, "parapet-0024")["pl::height"] = serde_json::json!({"value": 30.0, "status": "Known", "source": "x"})));
    assert!(e.contains("has no unit") && e.contains("never read as feet"), "{e}");
    // An outline in metres, and an eave line with no unit.
    assert!(err(edit(&text, |v| attrs(v, "plane-0017")["pl::outline"]["points"]["unit"] = "m".into())).contains("metric"));
    assert!(err(edit(&text, |v| {
        attrs(v, "plane-0017")["pl::outline"]["eave"].as_object_mut().unwrap().remove("unit");
    }))
    .contains("has no unit"));
    // Coordinates are plain numbers: feet-and-inches is for a single dimension.
    assert!(err(edit(&text, |v| attrs(v, "rake-0019")["pl::run"]["ends"]["unit"] = "ft-in".into())).contains("plan coordinates are plain numbers"));
    // The number has to suit the unit: a pair is ft-in, a single number is not.
    let pair = |unit: &str, value: Value| edit(&text, |v| attrs(v, "parapet-0024")["pl::height"] = serde_json::json!({"value": value, "unit": unit, "status": "Known", "source": "x"}));
    assert!(err(pair("ft", serde_json::json!([2, 6]))).contains("the unit for a pair is \"ft-in\""));
    assert!(err(pair("ft-in", serde_json::json!(30.0))).contains("write it as [feet, inches]"));
    assert!(err(pair("ft-in", serde_json::json!([2.5, 6]))).contains("whole number"));
    assert!(err(pair("ft-in", serde_json::json!([2, 14]))).contains("less than 12"));
    assert!(err(pair("ft-in", serde_json::json!([2, -6]))).contains("opposite signs"));
    assert!(err(pair("ft-in", serde_json::json!([2, 6, 1]))).contains("exactly two numbers"));
    assert!(err(pair("ft-in", serde_json::json!("2' 6\""))).contains("a length"));
    // A pitch has one unit. Degrees are refused; so is none.
    assert!(err(edit(&text, |v| attrs(v, "plane-0017")["pl::pitch"]["unit"] = "deg".into())).contains("has to be \"in/12\""));
    assert!(err(edit(&text, |v| {
        attrs(v, "plane-0017")["pl::pitch"].as_object_mut().unwrap().remove("unit");
    }))
    .contains("has to be \"in/12\""));
    // Derived lengths are worked out in feet; a job declared in metres is refused.
    assert!(err(edit(&text, |v| attrs(v, "job-0001")["pl::job"]["length_unit"] = "m".into())).contains("metric"));
    // A unit cost priced per LF against an SF quantity.
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["lines"][0]["unit_cost"]["unit"] = "$/LF".into())).contains("has to be \"$/SF\""));

    // The same at the keyboard. A bare number is not feet; metric is named; nothing is stored.
    let mut job = sample();
    let before = save(&mut job);
    let mut refuse = |typed: &str| job.set("parapet-0024", "height", Some(typed), Status::Known, "x").unwrap_err();
    let e = refuse("30");
    assert!(e.contains("has no unit") && e.contains("never read as feet"), "{e}");
    assert!(refuse("0.76 m").contains("metric"));
    assert!(refuse("762mm").contains("metric"));
    assert!(refuse("3 cubits").contains("\"cubits\""));
    assert!(refuse("1 yd").contains("\"yd\""));
    assert_eq!(save(&mut job), before);
}

#[test]
fn a_parapet_height_in_inches_gives_the_same_face_area_as_in_feet() {
    // The sample parapet is 20 LF. 30 in of height is 2.5 ft: 50 SF either way.
    let mut inches = sample();
    inches.set("parapet-0024", "height", Some("30 in"), Status::Known, "detail 3/A-501").unwrap();
    inches.derive();
    let mut feet = sample();
    feet.set("parapet-0024", "height", Some("2.5 ft"), Status::Known, "detail 3/A-501").unwrap();
    feet.derive();
    let (a, b) = (measure(&mut inches, "parapet-0024", "face_area"), measure(&mut feet, "parapet-0024", "face_area"));
    assert_eq!((a.value, a.status), (Some(50.0), Status::Known));
    assert_eq!((b.value, b.status), (Some(50.0), Status::Known));
    // Each formula says what was done to the height it was given.
    assert_eq!(a.formula, "plan_length × height ÷ 12");
    assert_eq!(b.formula, "plan_length × height");
    // 30 in was not read as 30 ft: that would be 600 SF.
    assert_ne!(a.value, Some(600.0));
}

fn feet_of(typed: &str) -> f64 {
    let (len, unit) = units::parse_length(typed).unwrap_or_else(|e| panic!("{typed:?} refused: {e}"));
    len.feet(unit).unwrap()
}

#[test]
fn each_way_of_writing_a_length_means_what_it_says() {
    use units::{parse_length, Len};
    // The four in the ruling. Three of them are the same length; the fourth is half an inch more.
    assert_eq!(parse_length("25' 6\"").unwrap(), (Len::FtIn([25.0, 6.0]), "ft-in"));
    assert_eq!(parse_length("25.5 ft").unwrap(), (Len::One(25.5), "ft"));
    assert_eq!(parse_length("306 in").unwrap(), (Len::One(306.0), "in"));
    assert_eq!(parse_length("25' 6 1/2\"").unwrap(), (Len::FtIn([25.0, 6.5]), "ft-in"));
    assert_eq!(feet_of("25' 6\""), 25.5);
    assert_eq!(feet_of("25.5 ft"), 25.5);
    assert_eq!(feet_of("306 in"), 25.5);
    assert_eq!(feet_of("25' 6 1/2\"") * 12.0, 306.5);
    // To the sixteenth: each sixteenth from 25' 6" to 25' 7" is its own length, exactly
    // 1/16 in more than the last, and the same number of feet as the same length in inches.
    for n in 0..=15u32 {
        let (mut num, mut den) = (n, 16);
        while num != 0 && num % 2 == 0 {
            num /= 2;
            den /= 2;
        }
        let typed = if num == 0 { "25' 6\"".to_string() } else { format!("25' 6 {num}/{den}\"") };
        let inches = 306.0 + f64::from(n) / 16.0;
        assert_eq!(feet_of(&typed), inches / 12.0, "{typed}");
        assert_eq!(feet_of(&typed), feet_of(&format!("{inches} in")), "{typed}");
        // And it is shown back exactly as it was written.
        let (len, unit) = parse_length(&typed).unwrap();
        assert_eq!(units::show_length(&len, unit), typed);
    }
    // Other ways the same things get written on drawings and tapes.
    for (typed, feet) in [
        ("25'-6\"", 25.5),
        ("25' - 6\"", 25.5),
        ("25'6\"", 25.5),
        ("25' 6", 25.5),
        ("25 ft 6 in", 25.5),
        ("25 feet 6 inches", 25.5),
        ("25ft 6in", 25.5),
        ("25'", 25.0),
        ("25.5'", 25.5),
        ("25.5 feet", 25.5),
        ("306\"", 25.5),
        ("306 inches", 25.5),
        ("306in", 25.5),
        ("0' 6\"", 0.5),
        ("6 1/2\"", 6.5 / 12.0),
        ("6-1/2\"", 6.5 / 12.0),
        ("1/2 in", 0.5 / 12.0),
        ("19 1/2 in", 19.5 / 12.0),
        ("25'-6 1/2\"", 306.5 / 12.0),
        ("25' 6-1/2\"", 306.5 / 12.0),
        ("25' 6.5\"", 306.5 / 12.0),
        ("25\u{2032} 6\u{00BD}\u{2033}", 306.5 / 12.0),
        ("25\u{2019}-6\u{201D}", 25.5),
        ("19\u{00BD}\u{2033}", 19.5 / 12.0),
        ("-2' 6\"", -2.5),
        ("-30 in", -2.5),
        (".5 ft", 0.5),
        ("3/64 in", 3.0 / 64.0 / 12.0),
    ] {
        assert_eq!(feet_of(typed), feet, "{typed}");
    }
    // What is refused, each with its reason. None of these is read as anything.
    for (typed, says) in [
        ("25.5", "has no unit"),
        ("25", "has no unit"),
        ("", "is not a number"),
        ("ft", "is not a number"),
        ("25.5' 6\"", "whole number"),
        ("25' 14\"", "less than 12"),
        ("25' 12\"", "less than 12"),
        ("25' 6' ", "gives feet twice"),
        ("25' 6\" 3", "something after the inches"),
        ("6 1/3 in", "halves, quarters"),
        ("6 3/2 in", "less than one"),
        ("6 0/2 in", "more than nothing"),
        ("6.5 1/2 in", "both a decimal point and a fraction"),
        ("1e3 ft", "\"e\""),
        ("1,200 ft", "not a measurement"),
        ("12 m", "metric"),
        ("12 metres", "metric"),
        ("12 mm", "metric"),
        ("12 yd", "\"yd\""),
        ("twelve ft", "is not a number"),
        ("12 ft ft", "is not a number"),
        ("--12 ft", "is not a number"),
        ("12 \u{00B0}", "not part of a measurement"),
    ] {
        let e = parse_length(typed).expect_err(typed);
        assert!(e.contains(says), "{typed:?}: {e}");
    }
}

#[test]
fn a_pitch_is_read_the_ways_a_drawing_writes_it() {
    use units::parse_pitch;
    for (typed, rise) in [
        ("6", 6.0),
        ("6:12", 6.0),
        ("6/12", 6.0),
        ("6 in/12", 6.0),
        ("6in/12", 6.0),
        ("6 in 12", 6.0),
        ("4.5:12", 4.5),
        ("1/4:12", 0.25),
        ("1/4", 0.25),
        ("1/4 in/ft", 0.25),
        ("1/4\" per ft", 0.25),
        ("12/12", 12.0),
        ("1/12", 1.0),
        ("6 1/2:12", 6.5),
        ("\u{00BC}:12", 0.25),
        ("-0", 0.0),
    ] {
        assert_eq!(parse_pitch(typed).unwrap_or_else(|e| panic!("{typed:?} refused: {e}")), rise, "{typed}");
    }
    for (typed, says) in [("6:10", "not a pitch"), ("26.57 deg", "degrees"), ("26.57\u{00B0}", "degrees"), ("50%", "percentage"), ("1e300", "not a pitch"), ("six", "is not a number"), ("6 12", "not a pitch"), ("", "is not a number")] {
        let e = parse_pitch(typed).expect_err(typed);
        assert!(e.contains(says), "{typed:?}: {e}");
    }
}

#[test]
fn a_value_is_shown_and_saved_in_the_unit_it_was_entered_in() {
    let mut job = sample();
    job.set("parapet-0024", "height", Some("30 in"), Status::Known, "detail 3/A-501").unwrap();
    job.derive();
    let inches = save(&mut job);
    // Saved as typed: 30, in inches, with its status and source. Not 2.5 and not feet.
    let v: Value = serde_json::from_str(&inches).unwrap();
    let h = &v["data"].as_array().unwrap().iter().find(|e| e["path"] == "parapet-0024").unwrap()["attributes"]["pl::height"];
    assert_eq!(h, &serde_json::json!({"value": 30.0, "unit": "in", "status": "Known", "source": "detail 3/A-501"}));
    // Shown as typed.
    assert!(show_rows(&mut job, "parapet-0024").contains(&("height".to_string(), "30 in".to_string(), Status::Known)));
    // And it loads back as the same thing and saves to the same bytes.
    let (mut again, report) = load(&inches).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut again), inches);

    // Change only the unit it is written in: 30 in becomes 2.5 ft. The diff is that value
    // and the one figure derived from it (its formula no longer divides by 12).
    job.set("parapet-0024", "height", Some("2.5 ft"), Status::Known, "detail 3/A-501").unwrap();
    let changed = job.derive();
    let feet = save(&mut job);
    assert_eq!(touched(&inches, &feet), ["parapet-0024"]);
    assert_eq!(changed, ["parapet-0024"]);
    let (a, b) = (entries(&inches), entries(&feet));
    let (a, b) = (&a.iter().find(|e| e.0 == "parapet-0024").unwrap().1["attributes"], &b.iter().find(|e| e.0 == "parapet-0024").unwrap().1["attributes"]);
    for (key, value) in a.as_object().unwrap() {
        assert_eq!(value == &b[key], !(key == "pl::height" || key == "pl::measure"), "{key}");
    }
    assert_eq!(a["pl::measure"]["plan_length"], b["pl::measure"]["plan_length"]);
    assert_eq!(a["pl::measure"]["face_area"]["value"], b["pl::measure"]["face_area"]["value"]);
    assert_ne!(a["pl::measure"]["face_area"]["formula"], b["pl::measure"]["face_area"]["formula"]);

    // Feet and inches are saved as the pair and shown the way they are written.
    job.set("parapet-0024", "height", Some("25' 6 1/2\""), Status::Assumed, "scaled off the elevation").unwrap();
    job.derive();
    let text = save(&mut job);
    assert!(text.contains("\"value\": [25.0, 6.5],\n          \"unit\": \"ft-in\""), "{text}");
    assert!(show_rows(&mut job, "parapet-0024").contains(&("height".to_string(), "25' 6 1/2\"".to_string(), Status::Assumed)));
    // 20 LF x 306.5 in / 12 = 510.8333 SF.
    assert_eq!(measure(&mut job, "parapet-0024", "face_area").value, Some(510.8333));
    assert_eq!(measure(&mut job, "parapet-0024", "face_area").formula, "plan_length × height (feet + inches ÷ 12)");
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut again), text);

    // Marked unknown, it keeps the unit that was there and loses the number.
    job.set("parapet-0024", "height", None, Status::Unknown, "elevation superseded").unwrap();
    job.derive();
    assert_eq!(measure(&mut job, "parapet-0024", "face_area").value, None);
    assert!(save(&mut job).contains("\"value\": null,\n          \"unit\": \"ft-in\""));

    // An eave height takes the same units. The porch eave at 9 ft is 108 in is 9' 0".
    let base = save(&mut sample());
    for typed in ["108 in", "9' 0\"", "9 ft"] {
        let mut job = sample();
        job.set("plane-0017", "eave_height", Some(typed), Status::Known, "sample plan A-201").unwrap();
        job.derive();
        assert_eq!(measure(&mut job, "rake-0019", "true_length").value, Some(8.2462), "{typed}");
        // Only the plane and its four lines can differ from the sample, and only in how
        // the figure is written.
        for path in touched(&base, &save(&mut job)) {
            assert!(["plane-0017", "eave-0018", "rake-0019", "rake-0020", "headwall-0021"].contains(&path.as_str()), "{typed}: {path}");
        }
    }
}

#[test]
fn a_stretch_out_in_inches_works_against_a_length_in_feet() {
    // The sample porch headwall is 12 LF and carries a copper flashing line whose
    // stretch-out the detail gives as 24 in. Nobody converts anything:
    // 12 LF x 24 in / 12 = 24 SF.
    let mut job = sample();
    let area = |job: &mut Job| {
        let e = job.find("headwall-0021").unwrap();
        job.world.get::<LineQuantities>(e).unwrap().0["Copper headwall flashing"].clone()
    };
    let d = area(&mut job);
    assert_eq!((d.value, d.unit.as_str()), (Some(24.0), "SF"));
    assert_eq!(d.formula, "true_length × stretch_out ÷ 12");
    assert_eq!(d.inputs, ["headwall-0021#pl::measure", "headwall-0021#pl::cost"]);
    // The headwall's length leans on the porch's assumed pitch, so the area is Assumed too.
    assert_eq!(d.status, Status::Assumed);
    // It is on the sheet with its stretch-out in inches and its area in SF.
    let rows = show_rows(&mut job, "headwall-0021");
    assert!(rows.contains(&("stretch-out: Copper headwall flashing".to_string(), "24 in".to_string(), Status::Known)), "{rows:?}");
    assert!(rows.contains(&("sheet area: Copper headwall flashing".to_string(), "24 SF".to_string(), Status::Assumed)), "{rows:?}");
    // It is in the file, and the file loads back to the same bytes.
    let text = save(&mut job);
    assert!(text.contains("\"pl::line_quantities\""));
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut again), text);

    // The brief's four stretch-outs on the same 12 LF: eave 12 in, step flashing 8 in,
    // valley 36 in, headwall 24 in give 12, 8, 36 and 24 SF.
    for (typed, sf) in [("12 in", 12.0), ("8 in", 8.0), ("36 in", 36.0), ("24 in", 24.0), ("2 ft", 24.0), ("2' 0\"", 24.0), ("19 1/2\"", 19.5), ("1' 7 1/2\"", 19.5)] {
        job.set("headwall-0021", "stretch_out", Some(typed), Status::Known, "detail").unwrap();
        let changed = job.derive();
        assert_eq!(area(&mut job).value, Some(sf), "{typed}");
        assert!(changed.is_empty() || changed == ["headwall-0021"], "{typed}: {changed:?}");
    }
    // Entered in feet, the formula does not divide.
    job.set("headwall-0021", "stretch_out", Some("2 ft"), Status::Known, "detail").unwrap();
    job.derive();
    assert_eq!(area(&mut job).formula, "true_length × stretch_out");
    // Nobody has the stretch-out: there is no area, not a zero one.
    job.set("headwall-0021", "stretch_out", None, Status::Unknown, "detail not issued").unwrap();
    job.derive();
    assert_eq!((area(&mut job).value, area(&mut job).status), (None, Status::Unknown));
    // A stretch-out of nothing, or with no unit, or in metric, is refused.
    assert!(job.set("headwall-0021", "stretch_out", Some("0 in"), Status::Known, "x").unwrap_err().contains("more than zero"));
    assert!(job.set("headwall-0021", "stretch_out", Some("24"), Status::Known, "x").unwrap_err().contains("has no unit"));
    assert!(job.set("headwall-0021", "stretch_out", Some("610 mm"), Status::Known, "x").unwrap_err().contains("metric"));
    // An element with no cost line says so.
    assert!(job.set("eave-0018", "stretch_out", Some("12 in"), Status::Known, "x").unwrap_err().contains("no cost line"));

    // In the file: a stretch-out on an area line is refused, a forged area is caught, and
    // two lines with one description are refused.
    let err = |t: String| load(&t).err().expect("should be refused");
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::cost"]["lines"][0]["stretch_out"] = serde_json::json!({"value": 24.0, "unit": "in", "status": "Known", "source": "x"}))).contains("turns a length (LF) into an area"));
    assert!(err(edit(&text, |v| attrs(v, "headwall-0021")["pl::cost"]["lines"][0]["unit_cost"]["unit"] = "$/LF".into())).contains("has to be \"$/SF\""));
    assert!(err(edit(&text, |v| attrs(v, "headwall-0021")["pl::cost"]["lines"][0]["stretch_out"]["unit"] = "mm".into())).contains("metric"));
    let (mut forged, report) = load(&edit(&text, |v| attrs(v, "headwall-0021")["pl::line_quantities"]["Copper headwall flashing"]["value"] = 999.0.into())).unwrap();
    assert_eq!(report.stale, ["headwall-0021"]);
    assert_eq!(area(&mut forged).value, Some(24.0));
    assert!(err(edit(&text, |v| {
        let line = attrs(v, "headwall-0021")["pl::cost"]["lines"][0].clone();
        attrs(v, "headwall-0021")["pl::cost"]["lines"].as_array_mut().unwrap().push(line);
    }))
    .contains("two cost lines"));
}

#[test]
fn an_outline_entered_in_inches_measures_the_same_as_in_feet() {
    // The porch plane and its rake with every coordinate in inches, against lines and an
    // eave height left in feet. Same areas, same lengths; the formulas say what was divided.
    let text = save(&mut sample());
    let (mut job, report) = load(&edit(&text, |v| {
        let o = &mut attrs(v, "plane-0017")["pl::outline"];
        o["points"] = serde_json::json!({"value": [[528.0, 0.0], [672.0, 0.0], [672.0, 96.0], [528.0, 96.0]], "unit": "in", "status": "Known", "source": "x"});
        o["eave"] = serde_json::json!({"value": [[528.0, 0.0], [672.0, 0.0]], "unit": "in", "status": "Known", "source": "x"});
        attrs(v, "rake-0019")["pl::run"]["ends"] = serde_json::json!({"value": [[528.0, 0.0], [528.0, 96.0]], "unit": "in", "status": "Known", "source": "x"});
    }))
    .unwrap();
    assert_eq!(report.stale, ["eave-0018", "headwall-0021", "plane-0017", "rake-0019", "rake-0020"]);
    assert_eq!(measure(&mut job, "plane-0017", "plan_area").value, Some(96.0));
    assert_eq!(measure(&mut job, "plane-0017", "surface_area").value, Some(98.9545));
    assert_eq!(measure(&mut job, "plane-0017", "plan_area").formula, "area of the outline on the plan ÷ 144 (entered in inches)");
    assert_eq!(measure(&mut job, "rake-0019", "plan_length").value, Some(8.0));
    assert_eq!(measure(&mut job, "rake-0019", "plan_length").formula, "length of the line on the plan ÷ 12 (entered in inches)");
    for (line, lf) in [("eave-0018", 12.0), ("rake-0019", 8.2462), ("rake-0020", 8.2462), ("headwall-0021", 12.0)] {
        let d = measure(&mut job, line, "true_length");
        assert_eq!(d.value, Some(lf), "{line}");
        assert!(d.formula.ends_with("converted to feet first (inches ÷ 12)"), "{line}: {}", d.formula);
    }
    // The limits are in feet whatever the unit: a coordinate of 13,000,000 in is past the
    // 1,000,000 ft limit.
    let err = load(&edit(&text, |v| attrs(v, "rake-0019")["pl::run"]["ends"] = serde_json::json!({"value": [[13.0e6, 0.0], [528.0, 96.0]], "unit": "in", "status": "Known", "source": "x"}))).err().unwrap();
    assert!(err.contains("not a usable number"), "{err}");
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
    // At the keyboard 1e300 is not something an estimator writes; in a file it is a number
    // and is refused as a wall.
    assert!(job.set("plane-0017", "pitch", Some("1e300"), Status::Known, "typo").unwrap_err().contains("not a pitch"));
    assert!(job.set("plane-0017", "pitch", Some("300"), Status::Known, "typo").unwrap_err().contains("is a wall"));
    let wall = load(&edit(&save(&mut sample()), |v| attrs(v, "plane-0017")["pl::pitch"]["value"] = 1e300.into())).err().unwrap();
    assert!(wall.contains("is a wall"), "{wall}");
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
    assert!(found.problems.iter().all(|p| p.contains("plane-0003") && p.contains("disagree about its height")));
    // The lines that do not touch the south plane's high edges are still found.
    assert_eq!(found.lines.iter().filter(|l| l.kind == Kind::Hip).count(), 2);
    assert_eq!(found.lines.iter().filter(|l| l.kind == Kind::Ridge && l.planes.len() == 2).count(), 0);
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
