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
    assert_eq!(paths.len(), 26);
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
        assert_eq!(same, !(key == "pl::pitch" || key == "pl::measure" || key == "pl::line_quantities"), "{key}");
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
    for (typed, says) in [("1/4", "could be"), ("6:10", "not a pitch"), ("26.57 deg", "degrees"), ("26.57\u{00B0}", "degrees"), ("50%", "percentage"), ("1e300", "not a pitch"), ("six", "is not a number"), ("6 12", "not a pitch"), ("", "is not a number")] {
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
        job.world.get::<LineQuantities>(e).unwrap().0["Copper headwall flashing"].quantity.clone()
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
    assert!(rows.contains(&("quantity: Copper headwall flashing".to_string(), "24 SF".to_string(), Status::Assumed)), "{rows:?}");
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
    let (mut forged, report) = load(&edit(&text, |v| attrs(v, "headwall-0021")["pl::line_quantities"]["Copper headwall flashing"]["quantity"]["value"] = 999.0.into())).unwrap();
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
    assert_eq!(path, "plane-0027");
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

// ---- Regression checks for what the third independent judgment found ----

/// A one-plane job: a trapezoid whose low edge is y = 0 and whose right side is a rake.
fn trapezoid(eave: [P2; 2], points: Vec<P2>, lines: &[(Kind, &str, [P2; 2])]) -> Job {
    let mut job = Job::new("Trapezoid", "test", "test");
    let plane = job.add("job-0001", Kind::RoofPlane, "plane").unwrap();
    job.put(&plane, Outline { points: Val::known(points, "ft", "test"), eave: Val::known(eave, "ft", "test"), eave_height: Val::known(Len::One(0.0), "ft", "test") }).unwrap();
    job.put(&plane, Pitch(Val::known(6.0, "in/12", "test"))).unwrap();
    for (kind, name, ends) in lines {
        let p = job.add("job-0001", *kind, name).unwrap();
        job.put(&p, Run { ends: Val::known(*ends, "ft", "test"), planes: vec![plane.clone()] }).unwrap();
    }
    job.derive();
    let (job, _) = load(&save(&mut job)).unwrap();
    job
}

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
    assert!(err(edit(&text, |v| attrs(v, "plane-0017")["pl::slope"] = 3.into())).contains("not one of its components"));
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

fn figures(job: &mut Job, path: &str, line: &str) -> LineFigures {
    let e = job.find(path).unwrap();
    job.world.get::<LineQuantities>(e).unwrap().0[line].clone()
}

/// The sample parapet (20 LF) with the parapet condition applied and nothing entered yet.
fn parapet_job() -> Job {
    let mut job = sample();
    let added = job.apply("parapet-0024", "Parapet with copper coping", "sample finish schedule").unwrap();
    assert_eq!(added, ["Copper wall flashing", "Copper coping cap", "Continuous cleat", "Cleat fasteners"]);
    job.derive();
    job
}

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
    assert!(job.apply("plane-0003", "Parapet with copper coping", "x").unwrap_err().contains("applies to a Parapet"));
    assert!(job.apply("parapet-0024", "No such condition", "x").unwrap_err().contains("no condition"));
    job.apply("parapet-0024", "condition-0025", "x").unwrap();
    assert!(job.apply("parapet-0024", "condition-0025", "x").unwrap_err().contains("two cost lines"));
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
    // On a line: names the element does not have, lengths added to areas, volumes,
    // one-over-a-length, and division by a zero.
    let mut job = parapet_job();
    let mut refuse = |formula: &str, with: &[(&str, &str)]| {
        let with: Vec<(String, String)> = with.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
        job.set_formula("parapet-0024", Some("Copper wall flashing"), formula, &with, Status::Known, "x").unwrap_err()
    };
    assert!(refuse("plan_length * hieght", &[]).contains("does not have"));
    assert!(refuse("true_length * height", &[]).contains("does not have"));
    assert!(refuse("plan_length + face_area", &[]).contains("adds what cannot be added"));
    assert!(refuse("plan_length * height * height", &[]).contains("volume"));
    assert!(refuse("height / face_area", &[]).contains("one over a length"));
    assert!(refuse("slope_factor", &[]).contains("does not have"));
    assert!(refuse("plan_length * height", &[("extra", "2")]).contains("does not use"));
    assert!(refuse("plan_length * height * Sides", &[("Sides", "2")]).contains("lower-case"));
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
    .contains("a condition is its recipe"));
    assert!(err(edit(&text, |v| attrs(v, "condition-0025")["pl::recipe"]["applies_to"] = serde_json::json!([]))).contains("does not say what kind"));
    assert!(err(edit(&text, |v| attrs(v, "condition-0025")["pl::recipe"]["applies_to"] = serde_json::json!(["Job"]))).contains("not to a Job"));
    // A recipe line has to make sense on every kind it applies to: a hip has no height.
    assert!(err(edit(&text, |v| attrs(v, "condition-0025")["pl::recipe"]["applies_to"] = serde_json::json!(["Parapet", "Hip"]))).contains("does not have"));
    assert!(err(edit(&text, |v| attrs(v, "plane-0003")["pl::recipe"] = attrs(&mut serde_json::from_str(&text).unwrap(), "condition-0025")["pl::recipe"].clone())).contains("only a Condition element can"));
    // The order conversion has to be between the line's unit and the order unit, and waste
    // is a percentage.
    let coping = |v: &mut Value, key: &str, val: Value| attrs(v, "condition-0025")["pl::recipe"]["cost"]["lines"][1][key] = val;
    assert!(err(edit(&text, |v| coping(v, "order", serde_json::json!({"unit": "sheet", "factor": {"value": 30.0, "unit": "LF/sheet", "status": "Known", "source": "x"}, "round_up": true})))).contains("order factor in"));
    assert!(err(edit(&text, |v| coping(v, "order", serde_json::json!({"unit": "sheet", "factor": {"value": 0.0, "unit": "SF/sheet", "status": "Known", "source": "x"}, "round_up": true})))).contains("more than zero"));
    assert!(err(edit(&text, |v| coping(v, "waste", serde_json::json!({"value": -5.0, "unit": "%", "status": "Known", "source": "x"})))).contains("percentage from 0"));
    assert!(err(edit(&text, |v| coping(v, "waste", serde_json::json!({"value": 5.0, "unit": "", "status": "Known", "source": "x"})))).contains("has to be \"%\""));
    assert!(err(edit(&text, |v| coping(v, "unit_cost", serde_json::json!({"value": null, "unit": "$/SF", "status": "Unknown", "source": "x"})))).contains("has to be \"$/sheet\""));
}
