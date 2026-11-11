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
    assert_eq!(paths.len(), 27);
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
    assert_eq!(found.problems.iter().filter(|p| p.contains("at different heights (20.00 ft and 17.50 ft)")).count(), 1);
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

// ---- Regression checks for what the fourth independent judgment found ----

/// A job of planes (outline, eave line; all 6:12 from 10 ft) and lines on them, built
/// unchecked and then loaded, so it has passed the entry checks.
fn roof_job(planes: &[(Vec<P2>, [P2; 2])], lines: &[(Kind, [P2; 2], &[usize])]) -> Result<Job, String> {
    let mut job = Job::new("Roof", "test", "test");
    let mut paths = Vec::new();
    for (i, (points, eave)) in planes.iter().enumerate() {
        let p = job.add("job-0001", Kind::RoofPlane, &format!("plane-{i}")).unwrap();
        job.put(&p, Outline { points: Val::known(points.clone(), "ft", "test"), eave: Val::known(*eave, "ft", "test"), eave_height: Val::known(Len::One(10.0), "ft", "test") }).unwrap();
        job.put(&p, Pitch(Val::known(6.0, "in/12", "test"))).unwrap();
        paths.push(p);
    }
    for (i, (kind, ends, on)) in lines.iter().enumerate() {
        let p = job.add("job-0001", *kind, &format!("line-{i}")).unwrap();
        job.put(&p, Run { ends: Val::known(*ends, "ft", "test"), planes: on.iter().map(|k| paths[*k].clone()).collect() }).unwrap();
    }
    job.derive();
    load(&save(&mut job)).map(|(job, _)| job)
}

fn length_of(job: &mut Job, name: &str) -> Derived {
    let path = job.children()["job-0001"][name].clone();
    measure(job, &path, "true_length")
}

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
    // A spacing typed with a minus: 20 x 2 / (-12 in / 12) = -40. Not zero fasteners.
    job.set_on_line("parapet-0024", "spacing", Some("Cleat fasteners"), Some("-12 in"), Status::Known, "typo").unwrap();
    job.derive();
    let f = figures(&mut job, "parapet-0024", "Cleat fasteners").quantity;
    assert_eq!((f.value, f.status), (None, Status::Unknown));
    assert!(f.note.contains("below zero"), "{}", f.note);
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

fn footprint(points: Vec<P2>, rises: &[Rise], eave_ft: f64) -> Footprint {
    Footprint {
        points: Val::known(points, "ft", "test plan"),
        eave_height: Val::known(Len::One(eave_ft), "ft", "test plan"),
        edges: rises.iter().map(|r| Val::known(*r, if matches!(r, Rise::Gable) { "" } else { "in/12" }, "test plan")).collect(),
    }
}

fn child(job: &mut Job, parent: &str, name: &str) -> String {
    job.children()[parent][name].clone()
}

#[test]
fn a_footprint_solves_into_the_roof_that_was_entered_by_hand() {
    // The sample's main roof was entered plane by plane and line by line. The same roof
    // from its footprint alone: 40 x 30, 6:12 all round, eaves at 10 ft.
    let mut job = Job::new("Solved", "test", "test");
    let (roof_path, report) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    assert_eq!((report.planes, report.lines), (4, 9));
    assert!(report.notes.is_empty(), "{:?}", report.notes);
    // South plane: 375 SF on plan, 419.2627 SF on the slope, as worked by hand for the sample.
    let south = child(&mut job, &roof_path, "plane 1");
    assert_eq!(measure(&mut job, &south, "plan_area").value, Some(375.0));
    assert_eq!(measure(&mut job, &south, "surface_area").value, Some(419.2627));
    let east = child(&mut job, &roof_path, "plane 2");
    assert_eq!(measure(&mut job, &east, "plan_area").value, Some(225.0));
    // The ridge is 10 LF and level; each hip is 22.5 LF on the slope.
    let ridge = child(&mut job, &roof_path, "ridge 1-3");
    assert_eq!(measure(&mut job, &ridge, "true_length").value, Some(10.0));
    for hip in ["hip 1-2", "hip 2-3", "hip 3-4", "hip 1-4"] {
        let path = child(&mut job, &roof_path, hip);
        assert_eq!(measure(&mut job, &path, "true_length").value, Some(22.5), "{hip}");
    }
    for (eave, lf) in [("eave 1", 40.0), ("eave 2", 30.0), ("eave 3", 40.0), ("eave 4", 30.0)] {
        let path = child(&mut job, &roof_path, eave);
        assert_eq!(measure(&mut job, &path, "true_length").value, Some(lf), "{eave}");
    }
    // The four planes cover the footprint: 1,200 SF on plan.
    let total: f64 = (1..=4).map(|i| { let p = child(&mut job, &roof_path, &format!("plane {i}")); measure(&mut job, &p, "plan_area").value.unwrap() }).sum();
    assert_eq!(total, 1200.0);
    // It is a job like any other: it saves, loads, and saves to the same bytes, and the
    // lines the planes imply are exactly the lines the solver wrote.
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut again), text);
    let (solid, _) = roof::solid_planes(&mut again);
    let found = roof::lines_of(&solid);
    let cmp = roof::compare(&mut again, &found);
    assert_eq!((cmp.agree.len(), cmp.entered_only.len(), cmp.implied_only.len()), (9, 0, 0));
    // Solving again changes nothing.
    solve::solve(&mut again, &roof_path).unwrap();
    assert_eq!(save(&mut again), text);
}

#[test]
fn changing_a_pitch_moves_the_ridge_and_keeps_every_element_it_can() {
    // 60 x 30, 6:12 all round, eaves at 10 ft: the ridge runs at y = 15 from x = 15 to 45.
    let mut job = Job::new("Solved", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [60.0, 0.0], [60.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    let south = child(&mut job, &roof_path, "plane 1");
    let ridge = child(&mut job, &roof_path, "ridge 1-3");
    assert_eq!(measure(&mut job, &ridge, "true_length").value, Some(30.0));
    // Put a condition on the south plane, as an estimator would.
    job.set(&south, "condition", Some("Standing seam copper"), Status::Known, "finish schedule").unwrap();
    let paths_before: Vec<String> = job.children()[&roof_path].values().cloned().collect();
    let before = save(&mut job);

    // Steepen the south edge to 12:12. Entered by hand, this broke a roof (the hips no
    // longer fitted). Solved, the ridge moves to where the two slopes now meet: south
    // rises 1 ft per ft and north 0.5 ft per ft, so y = (30 - y) / 2, y = 10.
    let e = job.find(&roof_path).unwrap();
    let mut fp = job.world.get::<Footprint>(e).unwrap().clone();
    fp.edges[0] = Val::known(Rise::Pitch(12.0), "in/12", "revised elevation");
    job.put(&roof_path, fp).unwrap();
    let report = solve::solve(&mut job, &roof_path).unwrap();
    assert!(report.removed.is_empty() && report.added.is_empty(), "{report:?}");
    // Same elements, same paths; the south plane still has its condition.
    let paths_after: Vec<String> = job.children()[&roof_path].values().cloned().collect();
    assert_eq!(paths_before, paths_after);
    let s = job.find(&south).unwrap();
    assert_eq!(job.world.get::<Condition>(s).unwrap().0.value.as_deref(), Some("Standing seam copper"));
    // The ridge is at y = 10 now, from x = 20 to x = 40: 20 LF, level, 10 ft above the eaves.
    let r = job.find(&ridge).unwrap();
    let ends = job.world.get::<Run>(r).unwrap().ends.value.unwrap();
    assert!(ends.iter().all(|p| (p[1] - 10.0).abs() < 1e-6), "{ends:?}");
    assert_eq!(measure(&mut job, &ridge, "true_length").value, Some(20.0));
    // South plane: trapezoid (60 + 20) / 2 x 10 = 400 SF on plan, x sqrt(2) = 565.6854 SF.
    // North plane: (60 + 20) / 2 x 20 = 800 SF on plan, x 1.118034 = 894.4272 SF.
    assert_eq!(measure(&mut job, &south, "plan_area").value, Some(400.0));
    assert_eq!(measure(&mut job, &south, "surface_area").value, Some(565.6854));
    let north = child(&mut job, &roof_path, "plane 3");
    assert_eq!(measure(&mut job, &north, "surface_area").value, Some(894.4272));
    // No line on it is flagged: the planes fit each other because they were solved together.
    for (_, path) in job.children()[&roof_path].clone() {
        if job.kind(&path).is_some_and(|k| k.is_run()) {
            assert!(measure(&mut job, &path, "true_length").value.is_some(), "{path}");
        }
    }
    // In the file, everything that changed is the roof and what was solved from it.
    let after = save(&mut job);
    assert!(touched(&before, &after).iter().all(|p| *p == roof_path || paths_after.contains(p)));

    // On a 40 x 30 footprint the same change closes the roof to a point at (20, 10): the
    // west hip (x / 2 = y) and the east hip ((40 - x) / 2 = y) both reach y = 10 at x = 20.
    // The ridge has no length left, so it is removed, and the solve says so.
    let mut job = Job::new("Pyramid", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    let ridge = child(&mut job, &roof_path, "ridge 1-3");
    let e = job.find(&roof_path).unwrap();
    let mut fp = job.world.get::<Footprint>(e).unwrap().clone();
    fp.edges[0] = Val::known(Rise::Pitch(12.0), "in/12", "revised elevation");
    job.put(&roof_path, fp).unwrap();
    let report = solve::solve(&mut job, &roof_path).unwrap();
    assert_eq!(report.removed, [(ridge, false)]);
    assert!(load(&save(&mut job)).is_ok());
}

#[test]
fn an_l_shaped_footprint_gets_its_valley_and_a_gable_gets_its_rakes() {
    // An L: 40 x 20 with a 20 x 20 wing to the north on its west half, 6:12 all round.
    // One inside corner at (20, 20), so one valley, running to where the two ridges meet.
    let mut job = Job::new("L", "test", "test");
    let l = vec![[0.0, 0.0], [40.0, 0.0], [40.0, 20.0], [20.0, 20.0], [20.0, 40.0], [0.0, 40.0]];
    let (roof_path, report) = solve::new_roof(&mut job, "main", footprint(l, &[Rise::Pitch(6.0); 6], 10.0)).unwrap();
    assert!(report.notes.is_empty(), "{:?}", report.notes);
    let kinds: Vec<Kind> = job.children()[&roof_path].values().filter_map(|p| job.kind(p)).collect();
    let count = |k: Kind| kinds.iter().filter(|x| **x == k).count();
    assert_eq!((count(Kind::RoofPlane), count(Kind::Eave), count(Kind::Valley), count(Kind::Rake)), (6, 6, 1, 0));
    // The valley runs from the inside corner (20, 20) to (10, 10): 14.1421 LF on plan,
    // climbing 10 ft of run at 6:12 = 5 ft: sqrt(200 + 25) = 15 LF.
    let valley = job.children()[&roof_path].iter().find(|(n, _)| n.starts_with("valley")).map(|(_, p)| p.clone()).unwrap();
    assert_eq!(measure(&mut job, &valley, "true_length").value, Some(15.0));
    // The planes cover the footprint: 40 x 20 + 20 x 20 = 1,200 SF on plan.
    let total: f64 = job.children()[&roof_path].clone().values().filter(|p| p.starts_with("plane")).map(|p| measure(&mut job, p, "plan_area").value.unwrap()).sum();
    assert!((total - 1200.0).abs() < 1e-3, "{total}");

    // A gable roof: the two short ends upright. Two planes, a ridge end to end, four rakes.
    let mut job = Job::new("Gable", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 20.0], [0.0, 20.0]], &[Rise::Pitch(6.0), Rise::Gable, Rise::Pitch(6.0), Rise::Gable], 10.0)).unwrap();
    let kinds: Vec<Kind> = job.children()[&roof_path].values().filter_map(|p| job.kind(p)).collect();
    let count = |k: Kind| kinds.iter().filter(|x| **x == k).count();
    assert_eq!((count(Kind::RoofPlane), count(Kind::Eave), count(Kind::Ridge), count(Kind::Rake), count(Kind::Hip)), (2, 2, 1, 4, 0));
    let ridge = child(&mut job, &roof_path, "ridge 1-3");
    assert_eq!(measure(&mut job, &ridge, "true_length").value, Some(40.0));
    // Each rake: 10 ft of run at 6:12 climbs 5 ft: sqrt(100 + 25) = 11.1803 LF.
    for (name, path) in job.children()[&roof_path].clone() {
        if name.starts_with("rake") {
            assert_eq!(measure(&mut job, &path, "true_length").value, Some(11.1803), "{name}");
        }
    }
    // Turning a hipped end into a gable removes the hips and says so; an edge with no
    // pitch entered cannot be solved, and says which.
    let mut job = Job::new("Change", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 20.0], [0.0, 20.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    let e = job.find(&roof_path).unwrap();
    let mut fp = job.world.get::<Footprint>(e).unwrap().clone();
    fp.edges[1] = Val::known(Rise::Gable, "", "elevation");
    job.put(&roof_path, fp.clone()).unwrap();
    let report = solve::solve(&mut job, &roof_path).unwrap();
    assert_eq!(report.removed.len(), 4, "{report:?}"); // the east plane, its eave, its two hips
    assert_eq!(report.added.len(), 2, "{report:?}"); // two rakes
    assert!(load(&save(&mut job)).is_ok());
    fp.edges[0] = Val::unknown("in/12", "not on the drawings");
    job.put(&roof_path, fp).unwrap();
    assert!(solve::solve(&mut job, &roof_path).unwrap_err().contains("edge 1 has no pitch entered"));
}

#[test]
fn the_skeleton_holds_in_feet_on_a_short_edge_and_a_long_way_from_the_origin() {
    // The port restated BIMCraft's fixed millimetre tolerance as a share of the footprint's
    // size. A 40 x 30 roof with a 3 in jog in one eave (an edge a quarter of a foot long),
    // a mile from the origin: it still solves, and its planes still cover its footprint.
    let o = 5280.0;
    let pts = vec![[o, o], [o + 20.0, o], [o + 20.0, o + 0.25], [o + 40.0, o + 0.25], [o + 40.0, o + 30.0], [o, o + 30.0]];
    let area = geom::plan_area(&pts);
    let mut job = Job::new("Far", "test", "test");
    let (roof_path, report) = solve::new_roof(&mut job, "main", footprint(pts, &[Rise::Pitch(6.0); 6], 10.0)).unwrap();
    assert!(report.notes.is_empty(), "{:?}", report.notes);
    let total: f64 = job.children()[&roof_path].clone().values().filter(|p| p.starts_with("plane")).map(|p| measure(&mut job, p, "plan_area").value.unwrap()).sum();
    assert!((total - area).abs() < 1e-3, "{total} vs {area}");
    for (_, path) in job.children()[&roof_path].clone() {
        if job.kind(&path).is_some_and(|k| k.is_run()) {
            assert!(measure(&mut job, &path, "true_length").value.is_some(), "{path}: {}", measure(&mut job, &path, "true_length").note);
        }
    }
}

// ---- Regression checks for what the sixth independent judgment found (the solved roof) ----

fn hip_roof() -> (Job, String) {
    let mut job = sample();
    let (roof_path, _) = solve::new_roof(&mut job, "house", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    (job, roof_path)
}

#[test]
fn a_footprint_changed_in_the_file_does_not_leave_the_old_roof_standing() {
    let (mut job, roof_path) = hip_roof();
    let south = child(&mut job, &roof_path, "plane 1");
    let text = save(&mut job);
    // Edge 1's pitch struck out by hand. The planes of the earlier solve were 419.2627 SF
    // and Known; they must not stay so.
    let struck = edit(&text, |v| attrs(v, &roof_path)["pl::footprint"]["edges"][0] = serde_json::json!({"value": null, "unit": "in/12", "status": "Unknown", "source": "not on the drawing"}));
    let (mut loaded, report) = load(&struck).unwrap();
    assert!(report.stale.contains(&south) && report.stale.contains(&roof_path), "{:?}", report.stale);
    let d = measure(&mut loaded, &south, "surface_area");
    assert_eq!((d.value, d.status), (None, Status::Unknown));
    let rows = show_rows(&mut loaded, &roof_path);
    assert!(rows.contains(&("solved roof".to_string(), "not solved".to_string(), Status::Unknown)), "{rows:?}");
    assert!(rows.contains(&("edge 1".to_string(), "unknown".to_string(), Status::Unknown)), "{rows:?}");
    // No line under it has a length, and no total is given.
    let totals = plumbline_world::view::roof_totals(&mut loaded, &roof_path);
    assert_eq!(totals.surface.value, None);
    assert!(totals.lines.iter().all(|(_, sum)| sum.value.is_none()));
    // Entering the pitch again solves it again: same planes, same paths.
    solve::edit(&mut loaded, &roof_path, solve::Change::Rise { edge: 1, rise: Val::known(Rise::Pitch(6.0), "in/12", "elevation A-301") }).unwrap();
    assert_eq!(measure(&mut loaded, &south, "surface_area").value, Some(419.2627));
    // A solved plane's pitch forged in the file is put back from the footprint, and said.
    let forged = edit(&text, |v| attrs(v, &south)["pl::pitch"]["value"] = 12.0.into());
    let (mut loaded, report) = load(&forged).unwrap();
    assert!(report.stale.contains(&south));
    assert_eq!(measure(&mut loaded, &south, "surface_area").value, Some(419.2627));
    // A footprint point moved in the file moves the roof with it: 42 x 30 now.
    let moved = edit(&text, |v| {
        let pts = &mut attrs(v, &roof_path)["pl::footprint"]["points"]["value"];
        pts[1][0] = 42.0.into();
        pts[2][0] = 42.0.into();
    });
    let (mut loaded, _) = load(&moved).unwrap();
    let total: f64 = (1..=4).map(|i| { let p = child(&mut loaded, &roof_path, &format!("plane {i}")); measure(&mut loaded, &p, "plan_area").value.unwrap() }).sum();
    assert_eq!(total, 1260.0);
}

#[test]
fn an_eave_cannot_be_moved_through_the_far_wall_and_a_footprint_has_to_be_clean() {
    let (mut job, roof_path) = hip_roof();
    let before = save(&mut job);
    for by_ft in [-31.0, -30.0, -60.0, -100.0] {
        let e = solve::edit(&mut job, &roof_path, solve::Change::MoveEdge { edge: 1, by_ft, status: Status::Known, source: "tape".into() }).unwrap_err();
        assert!(e.contains("inside out") || e.contains("same point") || e.contains("long"), "{by_ft}: {e}");
        assert_eq!(save(&mut job), before, "{by_ft}");
    }
    // A move that leaves a roof is taken: in by 10 ft leaves 40 x 20.
    solve::edit(&mut job, &roof_path, solve::Change::MoveEdge { edge: 1, by_ft: -10.0, status: Status::Known, source: "tape".into() }).unwrap();
    let total: f64 = (1..=4).map(|i| { let p = child(&mut job, &roof_path, &format!("plane {i}")); measure(&mut job, &p, "plan_area").value.unwrap() }).sum();
    assert_eq!(total, 800.0);

    // A corner 1/16 in off straight would get a hip of its own; an edge 1/16 in long is a
    // slip; a pitch of a thousandth is flat. Each is refused, with what to do.
    let refuse = |pts: Vec<P2>, rises: &[Rise]| solve::new_roof(&mut sample(), "x", footprint(pts, rises, 10.0)).unwrap_err();
    assert!(refuse(vec![[0.0, 0.0], [20.0, -0.005], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 5]).contains("off the straight line"));
    assert!(refuse(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [20.0, 30.0], [20.0, 30.0052083], [0.0, 30.0052083]], &[Rise::Pitch(6.0); 6]).contains("slip of the trace"));
    assert!(refuse(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(0.001); 4]).contains("is flat"));
    // A point exactly on the line is allowed, and a refused roof leaves no trace in the job.
    let mut job = sample();
    let clean = save(&mut job);
    assert!(solve::new_roof(&mut job, "x", footprint(vec![[0.0, 0.0], [20.0, -0.005], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 5], 10.0)).is_err());
    assert_eq!(save(&mut job), clean);
    assert!(solve::new_roof(&mut job, "x", footprint(vec![[0.0, 0.0], [20.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 5], 10.0)).is_ok());
}

#[test]
fn what_was_entered_on_a_slope_or_a_rake_stays_on_that_slope_or_rake() {
    // Two rakes. The west end is made a gable and its rake given a flashing and a
    // stretch-out; then the east end is made a gable too. The west rake keeps both.
    let (mut job, roof_path) = hip_roof();
    let gable = |edge| solve::Change::Rise { edge, rise: Val::known(Rise::Gable, "", "elevation") };
    solve::edit(&mut job, &roof_path, gable(4)).unwrap();
    let west_of = |job: &mut Job| -> Vec<String> {
        job.children()[&roof_path].clone().into_values().filter(|p| job.kind(p) == Some(Kind::Rake) && { let e = job.find(p).unwrap(); job.world.get::<Run>(e).unwrap().ends.value.unwrap().iter().all(|q| q[0].abs() < 1e-6) }).collect()
    };
    let west = west_of(&mut job);
    assert_eq!(west.len(), 2);
    let rake = west[0].clone();
    job.apply(&rake, "Copper flashing", "west rake detail").unwrap();
    job.set_on_line(&rake, "stretch_out", None, Some("7 in"), Status::Known, "detail 4/A5").unwrap();
    let report = solve::edit(&mut job, &roof_path, gable(2)).unwrap();
    assert!(report.removed.iter().all(|r| !r.1), "{report:?}");
    // Same path, still on the west side (x = 0), still with its line.
    assert!(west_of(&mut job).contains(&rake));
    let e = job.find(&rake).unwrap();
    assert_eq!(job.world.get::<Cost>(e).unwrap().lines[0].description, "Copper flashing");

    // A plane with a condition. A point is then added to the footprint by hand, on the
    // south edge, so every edge after it takes a new number. The condition stays on the
    // north slope; the solve says the plane is on a different edge number now.
    let (mut job, roof_path) = hip_roof();
    let north = child(&mut job, &roof_path, "plane 3");
    job.apply(&north, "Standing seam copper roof", "north slope, sheet A2").unwrap();
    let e = job.find(&roof_path).unwrap();
    let mut fp = job.world.get::<Footprint>(e).unwrap().clone();
    let pts = fp.points.value.as_mut().unwrap();
    pts.insert(1, [20.0, -5.0]);
    fp.edges.insert(1, Val::known(Rise::Pitch(6.0), "in/12", "test plan"));
    job.put(&roof_path, fp).unwrap();
    let report = solve::solve(&mut job, &roof_path).unwrap();
    assert!(report.renumbered.iter().any(|r| r.starts_with(&north) && r.contains("edge 4")), "{report:?}");
    let e = job.find(&north).unwrap();
    let eave = job.world.get::<Outline>(e).unwrap().eave.value.unwrap();
    assert!(eave.iter().all(|p| (p[1] - 30.0).abs() < 1e-6), "the north plane's eave is {eave:?}");
    assert_eq!(job.world.get::<Condition>(e).unwrap().0.value.as_deref(), Some("Standing seam copper roof"));
    assert_eq!(job.children()[&roof_path]["plane 4"], north);
    assert!(load(&save(&mut job)).is_ok());
}

#[test]
fn every_edge_of_a_solved_roof_has_a_line_or_a_note_that_stays_with_the_roof() {
    let all_lined = |job: &mut Job| {
        let (solid, _) = roof::solid_planes(job);
        let found = roof::lines_of(&solid);
        let cmp = roof::compare(job, &found);
        (cmp.implied_only.len(), cmp.entered_only.len())
    };
    // A 3:12 bay on a 12:12 house: where the bay's slope meets the main slopes both fall
    // the same way at different pitches. That is a change of pitch, and it gets a line.
    let mut job = Job::new("Bay", "test", "test");
    let mut rises = [Rise::Pitch(12.0); 5];
    rises[1] = Rise::Pitch(3.0);
    rises[2] = Rise::Pitch(3.0);
    let (roof_path, report) = solve::new_roof(&mut job, "house", footprint(vec![[0.0, 0.0], [40.0, 0.0], [50.0, 10.0], [40.0, 20.0], [0.0, 20.0]], &rises, 10.0)).unwrap();
    let breaks: Vec<String> = job.children()[&roof_path].clone().into_values().filter(|p| job.kind(p) == Some(Kind::PitchBreak)).collect();
    assert_eq!(breaks.len(), 2, "{report:?}");
    for b in &breaks {
        assert!(measure(&mut job, b, "true_length").value.is_some(), "{}", measure(&mut job, b, "true_length").note);
    }
    assert_eq!(all_lined(&mut job), (0, 0));
    assert!(load(&save(&mut job)).is_ok());

    // A gable beside an inside corner: a wall stands between the higher roof and the
    // lower one. Each plane has its own edge there, with a line, and the roof carries a
    // note saying so that `show` lists and the file keeps.
    let mut job = Job::new("Step", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "house", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [20.0, 40.0], [20.0, 20.0], [0.0, 20.0]], &[Rise::Pitch(6.0); 6], 10.0)).unwrap();
    solve::edit(&mut job, &roof_path, solve::Change::Rise { edge: 4, rise: Val::known(Rise::Gable, "", "elevation") }).unwrap();
    assert_eq!(all_lined(&mut job), (0, 0));
    let rows = show_rows(&mut job, &roof_path);
    assert!(rows.iter().any(|(label, _, status)| label == "solved roof" && *status == Status::Assumed), "{rows:?}");
    let text = save(&mut job);
    assert!(text.contains("\"pl::solve\"") && text.contains("A wall stands between them"));
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut again), text);
    // The planes still cover the footprint: 40 x 20 + 20 x 20 = 1,200 SF.
    let total: f64 = job.children()[&roof_path].clone().values().filter(|p| p.starts_with("plane")).map(|p| measure(&mut job, p, "plan_area").value.unwrap()).sum();
    assert!((total - 1200.0).abs() < 1e-3, "{total}");
}

#[test]
fn a_condition_can_be_applied_as_assumed_and_a_hand_made_name_in_the_solvers_way_says_so() {
    let (mut job, roof_path) = hip_roof();
    let south = child(&mut job, &roof_path, "plane 1");
    job.apply_as(&south, "Standing seam copper roof", Status::Assumed, "set in the 3D view").unwrap();
    let e = job.find(&south).unwrap();
    assert_eq!(job.world.get::<Condition>(e).unwrap().0.status, Status::Assumed);
    assert!(job.apply_as(&south, "Standing seam copper roof", Status::Unknown, "x").is_err());
    // A plane entered by hand under the roof, with the name the solver would use, after
    // the solver's own plane lost its mark: the solve says what is in its way.
    let text = edit(&save(&mut job), |v| {
        attrs(v, &south).as_object_mut().unwrap().remove("pl::solved");
    });
    let e = load(&text).err().or_else(|| load(&text).ok().and_then(|(mut j, _)| solve::solve(&mut j, &roof_path).err()));
    // Loading solves again; the roof is left unsolved with the reason on it, or refused.
    match e {
        Some(why) => assert!(why.contains("did not make that element"), "{why}"),
        None => {
            let (mut loaded, _) = load(&text).unwrap();
            let rows = plumbline_world::view::elements(&mut loaded).into_iter().find(|e| e.path == roof_path).unwrap().rows;
            assert!(rows.iter().any(|r| r.label == "solved roof" && r.note.contains("did not make that element")), "{:?}", rows.iter().map(|r| (&r.label, &r.note)).collect::<Vec<_>>());
        }
    }
}

#[test]
fn random_house_footprints_solve_whole_with_every_edge_lined() {
    // Stepped footprints (a staircase up the right side and down the left, in whole
    // feet), with pitches drawn from the ones roofs are built at and some ends gabled.
    // For each one that solves: the planes cover the footprint, every line has a length,
    // the planes imply no line the solver did not write and the solver wrote none the
    // planes do not imply, and the job loads back as it was saved.
    let mut seed = 20261110u64;
    let mut rand = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    };
    let pitches = [3.0, 4.0, 6.0, 8.0, 9.0, 12.0];
    let (mut solved, mut refused) = (0, 0);
    for round in 0..300 {
        let steps = 1 + (rand() * 3.0) as usize;
        let (mut right, mut left): (Vec<P2>, Vec<P2>) = (Vec::new(), Vec::new());
        let mut y = 0.0;
        for _ in 0..steps {
            let (xr, xl, h) = (16.0 + (rand() * 6.0).floor() * 4.0, -8.0 - (rand() * 6.0).floor() * 4.0, 12.0 + (rand() * 4.0).floor() * 4.0);
            right.extend([[xr, y], [xr, y + h]]);
            left.splice(0..0, [[xl, y + h], [xl, y]]);
            y += h;
        }
        let mut ring: Vec<P2> = right;
        ring.extend(left);
        // Take out repeated points and points on the straight line between their neighbours.
        loop {
            let n = ring.len();
            let drop = (0..n).find(|&i| {
                let (a, b, c) = (ring[(i + n - 1) % n], ring[i], ring[(i + 1) % n]);
                geom::dist(a, b) < 1e-9 || geom::point_to_segment(b, a, c) < 1e-9
            });
            match drop {
                Some(i) => {
                    ring.remove(i);
                }
                None => break,
            }
        }
        if ring.len() < 4 {
            continue;
        }
        let mixed = round % 2 == 1;
        let rises: Vec<Rise> = (0..ring.len()).map(|_| if rand() < 0.15 { Rise::Gable } else if mixed { Rise::Pitch(pitches[(rand() * 6.0) as usize % 6]) } else { Rise::Pitch(6.0) }).collect();
        let area = geom::plan_area(&ring);
        let mut job = Job::new("Fuzz", "test", "test");
        let (roof_path, report) = match solve::new_roof(&mut job, "house", footprint(ring.clone(), &rises, 10.0)) {
            Ok(x) => x,
            // Gables that fence in part of the roof, or every edge a gable, cannot be a roof.
            // Nor can gables that pinch a slope into two pieces joined at a point; that one
            // is a limit of this program, said as such.
            Err(e) if e.contains("fence in") || e.contains("none of its edges slopes") || e.contains("comes back to touch itself") => {
                refused += 1;
                continue;
            }
            Err(e) => panic!("round {round}: {e}\n{ring:?}\n{rises:?}"),
        };
        solved += 1;
        let context = format!("round {round}\n{ring:?}\n{rises:?}\n{:?}", report.notes);
        let under: Vec<String> = job.children()[&roof_path].values().cloned().collect();
        let total: f64 = under.iter().filter(|p| p.starts_with("plane")).map(|p| measure(&mut job, p, "plan_area").value.unwrap_or(f64::NAN)).sum();
        assert!((total - area).abs() < 1e-3 * area.max(1.0), "planes cover {total} of {area}: {context}");
        for path in &under {
            if job.kind(path).is_some_and(|k| k.is_run()) {
                let d = measure(&mut job, path, "true_length");
                assert!(d.value.is_some(), "{path} has no length ({}): {context}", d.note);
            }
        }
        let (solid, left_out) = roof::solid_planes(&mut job);
        assert!(left_out.is_empty(), "{left_out:?}: {context}");
        let found = roof::lines_of(&solid);
        let cmp = roof::compare(&mut job, &found);
        assert!(cmp.implied_only.is_empty() && cmp.entered_only.is_empty(), "implied only {:?}; entered only {:?}: {context}", cmp.implied_only, cmp.entered_only);
        let text = save(&mut job);
        let (mut again, report) = load(&text).unwrap_or_else(|e| panic!("{e}: {context}"));
        assert!(report.stale.is_empty(), "{:?}: {context}", report.stale);
        assert_eq!(save(&mut again), text, "{context}");
    }
    assert!(solved > 200, "only {solved} solved, {refused} refused");
}

#[test]
fn a_roof_summary_counts_facets_and_names_the_pitch_most_of_it_is_at() {
    use plumbline_world::view::{figure, roof_totals};
    // 60 x 30, the south slope at 12:12 and the rest at 6:12. On plan: south 400, north
    // 800, and each end a triangle 30 wide and 20 deep, 300; 1,800 SF in all, the whole
    // footprint. On the slope: south 400 x 1.414214 = 565.6854; north 800 x 1.118034 =
    // 894.4272; each end 300 x 1.118034 = 335.4102. The three at 6:12 come to 1,565.2476
    // SF against 565.6854 at 12:12, so most of it is at 6:12. Total 2,130.933 SF.
    let mut job = Job::new("Summary", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [60.0, 0.0], [60.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(12.0), Rise::Pitch(6.0), Rise::Pitch(6.0), Rise::Pitch(6.0)], 10.0)).unwrap();
    let totals = roof_totals(&mut job, &roof_path);
    assert_eq!(totals.facets, 4);
    assert_eq!(totals.predominant_pitch, Some((6.0, Status::Known)));
    assert_eq!(totals.surface.value, Some(2130.933));
    // The same surface in squares: 100 SF each, same status, nothing added or left out.
    let squares = plumbline_world::view::in_squares(&totals.surface);
    assert_eq!((squares.value, squares.unit, squares.status, squares.members), (Some(21.3093), "SQ", Status::Known, 4));
    // A worked-out row carries its figure as a number for a view to set; an entered one does not.
    let rows = plumbline_world::view::elements(&mut job);
    let plane = rows.iter().find(|e| e.kind == Kind::RoofPlane).unwrap();
    let area = plane.rows.iter().find(|r| r.label == "surface_area").unwrap();
    assert_eq!((area.number, area.unit.as_str()), (Some(565.6854), "SF"));
    assert_eq!(plane.rows.iter().find(|r| r.label == "pitch").unwrap().number, None);
    // An assumed pitch on the planes that carry most of the roof flags the pitch named.
    solve::edit(&mut job, &roof_path, solve::Change::Rise { edge: 3, rise: Val::assumed(Rise::Pitch(6.0), "in/12", "guess") }).unwrap();
    assert_eq!(roof_totals(&mut job, &roof_path).predominant_pitch, Some((6.0, Status::Assumed)));
    // Figures as a sheet shows them.
    assert_eq!(figure(2130.933, 2), "2,130.93");
    assert_eq!(figure(1341.6408, 0), "1,342");
    assert_eq!(figure(40.0, 2), "40.00");
    assert_eq!(figure(1234567.5, 1), "1,234,567.5");
    assert_eq!(figure(-0.001, 2), "0.00");
    assert_eq!(figure(-12.5, 1), "-12.5");
    // A view may round, but not round a figure away: one that is not zero never prints as zero.
    use plumbline_world::view::figure_shown;
    assert_eq!(figure_shown(2130.933, 2), "2,130.93");
    assert_eq!(figure_shown(0.004, 2), "0.004");
    assert_eq!(figure_shown(0.4, 0), "0.4");
    assert_eq!(figure_shown(-0.0003, 2), "-0.0003");
    assert_eq!(figure_shown(0.0, 2), "0.00");
    // Nor round it up into something else: 0.0005 is not 0.001 (seventh judgment).
    assert_eq!(figure_shown(0.0005, 2), "0.0005");
    assert_eq!(figure(f64::NEG_INFINITY, 2), "not a number");
    // A pitch is said to the places the model keeps, not two.
    assert_eq!(plumbline_world::view::pitch_said(6.125), "6.125:12");
    assert_eq!(plumbline_world::view::pitch_said(0.004), "0.004:12");
    assert_eq!(plumbline_world::view::pitch_said(6.0), "6:12");
}

#[test]
fn a_roof_summary_does_not_say_more_than_it_knows() {
    use plumbline_world::view::{in_squares, roof_totals, Sum};
    // A roof with no planes has no surface total. It is not a Known zero.
    let mut job = Job::new("Empty", "test", "test");
    let bare = job.add("job-0001", Kind::Roof, "bare").unwrap();
    let totals = roof_totals(&mut job, &bare);
    assert_eq!((totals.surface.value, totals.surface.status, totals.surface.members), (None, Status::Unknown, 0));
    assert_eq!((totals.predominant_pitch, totals.pitch_note.as_str()), (None, "the roof has no planes"));
    // A path that names nothing is the same.
    assert_eq!(roof_totals(&mut job, "no-such-roof").surface.value, None);
    // One plane with no pitch might be the one that tips it: no predominant pitch is named.
    let mut job = sample::sample();
    job.set("plane-0003", "pitch", None, Status::Unknown, "test").unwrap();
    job.set("plane-0005", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.derive();
    let totals = roof_totals(&mut job, "roof-0002");
    assert_eq!(totals.predominant_pitch, None);
    assert!(totals.pitch_note.contains("plane-0003"), "{}", totals.pitch_note);
    // Two pitches covering the same surface: neither is the predominant one.
    let mut job = sample::sample();
    job.set("plane-0004", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.set("plane-0006", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.set("plane-0005", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.set("plane-0003", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.set("plane-0003", "pitch", Some("6"), Status::Known, "test").unwrap();
    job.set("plane-0005", "pitch", Some("6"), Status::Known, "test").unwrap();
    job.derive();
    // South and north (375 SF on plan each) at 6:12 against east and west (225 each) at 12:12.
    assert_eq!(roof_totals(&mut job, "roof-0002").predominant_pitch, Some((6.0, Status::Known)));
    let mut tie = Job::new("Tie", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut tie, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]], &[Rise::Pitch(6.0), Rise::Pitch(6.0), Rise::Pitch(6.0), Rise::Pitch(6.0)], 10.0)).unwrap();
    assert_eq!(roof_totals(&mut tie, &roof_path).predominant_pitch, Some((6.0, Status::Known)));
    // A parapet is one of the roof's lines, by its plan length.
    let mut job = sample::sample();
    let deck = roof_totals(&mut job, "roof-0022");
    let parapet = deck.lines.iter().find(|(k, _)| *k == Kind::Parapet).expect("the parapet is among the lines");
    assert_eq!((parapet.1.value, parapet.1.status), (Some(20.0), Status::Known));
    // A surface too small to show in squares is not zero squares.
    let tiny = Sum { value: Some(0.004), unit: "SF", status: Status::Known, members: 1, missing: Vec::new() };
    assert_eq!((in_squares(&tiny).value, in_squares(&tiny).status), (None, Status::Unknown));
}

// ---------------------------------------------------------------- the rest of the building

fn measure_of(job: &mut Job, path: &str, name: &str) -> plumbline_world::Derived {
    let e = job.find(path).unwrap();
    job.world.get::<plumbline_world::Measure>(e).unwrap().0.get(name).unwrap().clone()
}

fn path_named(job: &mut Job, name: &str) -> String {
    plumbline_world::view::elements(job).into_iter().find(|e| e.name == name).map(|e| e.path).unwrap_or_else(|| panic!("no element named {name}"))
}

/// A door or window type of a size, added to the job for a test.
fn opening_type(job: &mut Job, kind: Kind, width: Val<Len>, height: Val<Len>) -> Val<String> {
    let t = job.add("job-0001", Kind::Type, &format!("test type {}", job.order.len())).unwrap();
    job.put(&t, plumbline_world::OpeningType { category: kind, width, height }).unwrap();
    Val::known(t, "", "test")
}

#[test]
fn the_sample_house_saves_and_loads_to_the_same_bytes_and_nothing_is_stale() {
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let (mut again, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty(), "stale: {:?}", report.stale);
    assert_eq!(file::save(&mut again), text);
}

#[test]
fn a_wall_is_its_length_by_its_height_less_its_openings() {
    let mut job = plumbline_world::sample::house();
    // The first floor's south wall: 40 ft long and 10 ft tall is 400 SF. A 3 ft by 6.67 ft
    // door is 20.01 SF and two 5 ft by 5 ft windows are 50: 70.01 SF of openings, 329.99 net.
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    assert_eq!(measure_of(&mut job, &south, "plan_length").value, Some(40.0));
    assert_eq!(measure_of(&mut job, &south, "face_area").value, Some(400.0));
    assert_eq!(measure_of(&mut job, &south, "opening_area").value, Some(70.01));
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!((net.value, net.status), (Some(329.99), Status::Known));
    // A wall with no openings has an opening area of zero, said as zero, and Known.
    let wet = job.children().get(&first).unwrap().get("wet wall").unwrap().clone();
    let none = measure_of(&mut job, &wet, "opening_area");
    assert_eq!((none.value, none.status), (Some(0.0), Status::Known));
    assert_eq!(measure_of(&mut job, &wet, "net_area").value, Some(90.0));
}

#[test]
fn an_opening_that_does_not_fit_its_wall_leaves_no_net_area() {
    use plumbline_world::Opening;
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let len = |x: f64| Val::known(Len::One(x), "ft", "test");
    // A window 4 ft wide centred 40 ft along a 40 ft wall runs past its end.
    let four_by_three = opening_type(&mut job, Kind::Window, len(4.0), len(3.0));
    let w = job.add(&south, Kind::Window, "too far").unwrap();
    job.put(&w, Opening { type_name: four_by_three, offset: len(40.0), sill: len(3.0) }).unwrap();
    job.derive();
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!((net.value, net.status), (None, Status::Unknown));
    assert!(net.note.contains("ends 42.00 ft along a wall 40.00 ft long"), "{}", net.note);
    // One that lies over another: 2 ft wide centred at 7 ft, over the west window (5 to 10 ft).
    let two_by_three = opening_type(&mut job, Kind::Window, len(2.0), len(3.0));
    job.put(&w, Opening { type_name: two_by_three, offset: len(7.0), sill: len(3.0) }).unwrap();
    job.derive();
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!(net.value, None);
    assert!(net.note.contains("lie over each other"), "{}", net.note);
    // One whose width nobody has: no opening area, so no net area, and the gross stands.
    let no_width = opening_type(&mut job, Kind::Window, Val::unknown("ft", "not on the plan"), len(3.0));
    job.put(&w, Opening { type_name: no_width, offset: len(13.0), sill: len(3.0) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "opening_area").value, None);
    assert_eq!(measure_of(&mut job, &south, "net_area").value, None);
    assert_eq!(measure_of(&mut job, &south, "face_area").value, Some(400.0));
}

/// The straight pieces a run of the sample house was entered as, in order: the element
/// of that name, or the elements named "<name>, piece 1 of n" and on.
fn pieces(job: &mut Job, name: &str) -> Vec<String> {
    let lead = format!("{name}, piece ");
    plumbline_world::view::elements(job).into_iter().filter(|e| e.name == name || e.name.starts_with(&lead)).map(|e| e.path).collect()
}

fn length_of_pieces(job: &mut Job, name: &str) -> (f64, Status) {
    let paths = pieces(job, name);
    assert!(!paths.is_empty(), "no run named {name}");
    let figures: Vec<plumbline_world::Derived> = paths.iter().map(|p| measure_of(job, p, "true_length")).collect();
    (plumbline_world::value::round4(figures.iter().map(|d| d.value.expect("a length")).sum()), figures.iter().map(|d| d.status).max().unwrap())
}

#[test]
fn a_run_is_straight_pieces_measured_in_space_with_an_elbow_at_each_corner() {
    use plumbline_world::{Fitting, FittingKind};
    let mut job = plumbline_world::sample::house();
    // The first-floor trunk, west, is drawn up 3.4 ft, south 8 ft, west 17 ft. It is
    // three straight pieces of those lengths, 28.4 LF in all, with two elbows.
    let trunk = pieces(&mut job, "first-floor trunk, west");
    let lengths: Vec<Option<f64>> = trunk.iter().map(|p| measure_of(&mut job, p, "true_length").value).collect();
    assert_eq!(lengths, [Some(3.4), Some(8.0), Some(17.0)]);
    assert_eq!(length_of_pieces(&mut job, "first-floor trunk, west"), (28.4, Status::Known));
    let elbows: Vec<String> = plumbline_world::view::elements(&mut job).into_iter().filter(|e| e.name.starts_with("first-floor trunk, west, elbow")).map(|e| e.path).collect();
    assert_eq!(elbows.len(), 2);
    for path in &elbows {
        let e = job.find(path).unwrap();
        assert_eq!(job.world.get::<Fitting>(e).unwrap().kind, FittingKind::Bend);
    }
    // It is 16 in by 8 in: 4 ft round, so the three pieces have 13.6 + 32 + 68 = 113.6 SF
    // of duct. The size is the type's, and Assumed.
    let skins: Vec<plumbline_world::Derived> = trunk.iter().map(|p| measure_of(&mut job, p, "surface_area")).collect();
    assert_eq!(skins.iter().map(|d| d.value).collect::<Vec<_>>(), [Some(13.6), Some(32.0), Some(68.0)]);
    assert!(skins.iter().all(|d| d.status == Status::Assumed));
    // A riser is a piece whose two ends are one point on the plan at two heights.
    let riser = measure_of(&mut job, &trunk[0], "true_length");
    assert!(riser.formula.contains("end_offset"), "{}", riser.formula);
    // A 6 in round branch 11 ft long: π × 0.5 × 11 = 17.2788 SF.
    let branch = path_named(&mut job, "branch to living west");
    assert_eq!(measure_of(&mut job, &branch, "surface_area").value, Some(17.2788));
    // A pipe with no size still has its length: 0.8 + 4.9 + 3.7 = 9.4 LF. No area and no
    // size is made up for it.
    assert_eq!(length_of_pieces(&mut job, "hot to the kitchen sink"), (9.4, Status::Known));
    // A piece that slopes: 3 ft along and 4 ft up is 5 ft of pipe.
    let (e, first) = (job.find(&trunk[1]).unwrap(), path_named(&mut job, "first floor"));
    let mut sloped = job.world.get::<plumbline_world::Segment>(e).unwrap().clone();
    sloped.line = Val::known([[0.0, 0.0], [3.0, 0.0]], "ft", "test");
    sloped.offset = ft(1.0);
    sloped.end_offset = Some(ft(5.0));
    let p = job.add(&first, Kind::Duct, "a sloped piece").unwrap();
    job.put(&p, sloped.clone()).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &p, "true_length").value, Some(5.0));
    // The same in inches and in feet and inches is the same 5 ft.
    sloped.line = Val::known([[0.0, 0.0], [36.0, 0.0]], "in", "test");
    sloped.end_offset = Some(Val::known(Len::FtIn([5.0, 0.0]), "ft-in", "test"));
    job.put(&p, sloped.clone()).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &p, "true_length").value, Some(5.0));
    // An end height nobody has leaves a sloped piece with no length; a level piece does
    // not need its height to have one.
    sloped.end_offset = Some(Val::unknown("ft", "not on the plan"));
    job.put(&p, sloped.clone()).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &p, "true_length").value, None);
    sloped.end_offset = None;
    sloped.offset = Val::unknown("ft", "not on the plan");
    job.put(&p, sloped).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &p, "true_length").value, Some(3.0));
}

#[test]
fn a_system_takeoff_groups_runs_by_size_and_says_what_it_cannot_size() {
    use plumbline_world::building::takeoff;
    let mut job = plumbline_world::sample::house();
    let systems = takeoff(&mut job);
    let hot = systems.iter().find(|s| s.name == "Hot water").unwrap();
    assert_eq!(hot.classification, "Domestic Hot Water");
    // Its runs fall on three lines: 1/2 in, 3/4 in, and the one with no size.
    let sizes: Vec<&str> = hot.lines.iter().map(|l| l.size.as_str()).collect();
    assert_eq!(sizes, ["1/2 in round", "3/4 in round", "size unknown"]);
    // The branch with no size is three straight pieces, 0.8 + 4.9 + 3.7 = 9.4 LF.
    let no_size = hot.lines.iter().find(|l| l.size == "size unknown").unwrap();
    assert_eq!((no_size.size_status, no_size.length.value, no_size.length.members), (Status::Unknown, Some(9.4), 3));
    // Hot to a lavatory is 1 + 0.9 = 1.9 LF and to a tub 1.5 + 1 = 2.5: two of each, 8.8 LF,
    // each of them two pieces.
    let half = hot.lines.iter().find(|l| l.size == "1/2 in round").unwrap();
    assert_eq!((half.length.value, half.length.members, half.length.status), (Some(8.8), 8, Status::Known));
    assert_eq!(hot.placed, vec![(Kind::Equipment, "Water heater, 50 gal".to_string(), 1, Status::Assumed)]);
    // Its elbows are counted by size: one at the corner of each of the four 1/2 in
    // branches, three on the 3/4 in main, and the two on the branch with no size are
    // counted and flagged, not sized by a guess.
    use plumbline_world::FittingKind::Bend;
    assert_eq!(
        hot.fittings,
        vec![(Some(Bend), "1/2 in elbow".to_string(), 4, Status::Known), (Some(Bend), "3/4 in elbow".to_string(), 3, Status::Known), (Some(Bend), "not said".to_string(), 2, Status::Unknown)]
    );
    // Every run and every placed thing in the job is on some line of some system.
    let rows = plumbline_world::view::elements(&mut job);
    let runs = rows.iter().filter(|e| e.kind.is_segment()).count();
    let placed = rows.iter().filter(|e| e.kind.is_placed()).count();
    assert_eq!(systems.iter().flat_map(|s| &s.lines).map(|l| l.length.members).sum::<usize>(), runs);
    assert_eq!(systems.iter().map(|s| s.placed.iter().map(|p| p.2).sum::<usize>() + s.fittings.iter().map(|f| f.2).sum::<usize>()).sum::<usize>(), placed);
}

#[test]
fn a_storey_totals_its_walls_and_floors_and_every_element_is_placed_or_accounted_for() {
    use plumbline_world::building::{placements, storey_totals};
    let mut job = plumbline_world::sample::house();
    let storeys = storey_totals(&mut job);
    assert_eq!(storeys.len(), 2);
    // Four outside walls (136 LF) and partitions of 40, 12, 12 and 10 LF: 210 LF a floor.
    assert_eq!(storeys[0].walls.value, Some(210.0));
    assert_eq!(storeys[0].floors.value, Some(1120.0));
    assert_eq!((storeys[0].doors, storeys[0].windows), (5, 8));
    assert_eq!((storeys[1].doors, storeys[1].windows), (4, 9));
    // Everything with a shape is drawn, but the two elbows on the branch that has no size:
    // nobody knows how big they are, and each is listed with that.
    let (drawn, not_drawn) = placements(&mut job);
    assert_eq!(not_drawn.len(), 2, "{not_drawn:?}");
    assert!(not_drawn.iter().all(|(_, why)| why == "its size is not known"), "{not_drawn:?}");
    let rows = plumbline_world::view::elements(&mut job);
    let shaped = rows.iter().filter(|e| matches!(e.kind, Kind::Wall | Kind::Floor) || e.kind.is_segment() || e.kind.is_placed()).count();
    assert_eq!(drawn.len() + not_drawn.len(), shaped);
    // Take a storey's elevation away and what stands on it is not drawn, and says why:
    // its walls and its floor, and every run and placed thing under it, whose heights
    // are above that storey.
    let first = path_named(&mut job, "first floor");
    let on_first = drawn.iter().filter(|p| p.storey.as_deref() == Some(first.as_str())).count();
    assert!(on_first > 9, "{on_first}");
    job.put(&first, plumbline_world::Level { elevation: Val::unknown("ft", "not surveyed"), height: Val::known(Len::One(10.0), "ft", "test"), computation_height: None }).unwrap();
    let (still, not_drawn) = placements(&mut job);
    // (The two elbows with no size are on the first floor too. A thing is listed once,
    // with the first thing that stops it being drawn.)
    assert_eq!(not_drawn.iter().filter(|(_, why)| why == "its storey's elevation is not known").count(), on_first + 2);
    assert_eq!(not_drawn.len(), on_first + 2);
    assert!(still.iter().all(|p| p.storey.as_deref() != Some(first.as_str())));
    // The first floor's outside walls stood up to the second floor, whose elevation is
    // still known: nothing of theirs is drawn against a guessed base.
    assert_eq!(drawn.len() - still.len(), on_first);
}

#[test]
fn the_building_checks_refuse_what_cannot_be() {
    use plumbline_world::building::*;
    let len = |x: f64| Val::known(Len::One(x), "in", "test");
    // A section is round, by a diameter, or rectangular, by a width and a depth.
    assert!(check_segment_type("t", Kind::Type, &SegmentType { category: Kind::Pipe, width: len(1.0), depth: Some(len(2.0)), round: true }).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType { category: Kind::Duct, width: len(12.0), depth: None, round: false }).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType::round(Kind::Pipe, len(0.0))).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType::round(Kind::Wall, len(1.0))).is_err());
    assert!(check_segment_type("t", Kind::Pipe, &SegmentType::round(Kind::Pipe, len(1.0))).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType::round(Kind::Pipe, len(300.0))).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType::round(Kind::Pipe, Val::unknown("in", "not on the plan"))).is_ok());
    // A run goes from one point to another; a length with no unit, or a metric one, is refused.
    let piece = |line: [[f64; 2]; 2], unit: &str, rise: Option<f64>| Segment { type_name: named("type-0002"), line: Val::known(line, unit, "test"), offset: ft(1.0), end_offset: rise.map(ft), justify: Justify::Centre };
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [1.0, 0.0]], "ft", None)).is_ok());
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [0.0, 0.0]], "ft", None)).unwrap_err().contains("a run goes from one point to another"));
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [0.0, 0.0]], "ft", Some(1.0))).is_err(), "a riser that rises nothing");
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [0.0, 0.0]], "ft", Some(9.0))).is_ok(), "a riser");
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [1.0, 0.0]], "", None)).is_err());
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [1.0, 0.0]], "m", None)).is_err());
    assert!(check_segment("r", Kind::Wall, &piece([[0.0, 0.0], [1.0, 0.0]], "ft", None)).is_err());
    // A placed thing is turned by an angle in degrees, and has a size in all three.
    let thing = |angle: Option<Val<f64>>, size: [f64; 3]| Placed { what: named("a thing"), at: Val::known([1.0, 1.0], "ft", "test"), offset: ft(0.0), angle, size: Val::known(size, "ft", "test") };
    assert!(check_placed("p", Kind::Fixture, &thing(None, [1.0, 1.0, 1.0])).is_ok());
    assert!(check_placed("p", Kind::Fitting, &thing(Some(Val::known(90.0, "deg", "test")), [1.0, 1.0, 1.0])).is_ok());
    assert!(check_placed("p", Kind::Fixture, &thing(Some(Val::known(90.0, "", "test")), [1.0, 1.0, 1.0])).is_err(), "an angle with no unit");
    assert!(check_placed("p", Kind::Fixture, &thing(Some(Val::known(720.0, "deg", "test")), [1.0, 1.0, 1.0])).is_err());
    assert!(check_placed("p", Kind::Fixture, &thing(None, [1.0, 0.0, 1.0])).is_err());
    assert!(check_placed("p", Kind::Wall, &thing(None, [1.0, 1.0, 1.0])).is_err());
    // A system's classification is one the program knows, and belongs to its discipline.
    assert!(check_system("s", Kind::System, &SystemInfo { discipline: Discipline::Plumbing, classification: "Supply Air".into() }).is_err());
    assert!(check_system("s", Kind::System, &SystemInfo { discipline: Discipline::Mechanical, classification: "Steam".into() }).is_err());
    // A saved job that names a system that is not there does not load.
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let system = path_named(&mut job, "Vent");
    let broken = text.replace(&format!("\"value\": \"{system}\""), "\"value\": \"system-9999\"");
    assert_ne!(broken, text);
    let err = file::load(&broken).err().expect("a dangling system is refused");
    assert!(err.contains("which is not a system in this job"), "{err}");
}

// ---------------------------------------------------------------- the eighth judgment

#[test]
fn a_storey_stands_under_the_job_and_counts_only_what_is_nearest_it() {
    use plumbline_world::building::storey_totals;
    // A file with the second floor moved under the first does not load.
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let (first, second) = (path_named(&mut job, "first floor"), path_named(&mut job, "second floor"));
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    for entry in doc["data"].as_array_mut().unwrap() {
        let path = entry["path"].as_str().unwrap().to_string();
        if path == "job-0001" {
            entry["children"].as_object_mut().unwrap().remove("second floor");
        } else if path == first {
            entry["children"].as_object_mut().unwrap().insert("second floor".into(), second.clone().into());
        }
    }
    let err = file::load(&doc.to_string()).err().expect("a storey under a storey is refused");
    assert!(err.contains("has to be directly under the job"), "{err}");
    // Built in code, where nothing stops it, each wall still counts once, on its own storey.
    let e = job.find(&second).unwrap();
    job.world.get_mut::<plumbline_world::Node>(e).unwrap().parent = Some(first.clone());
    let totals = storey_totals(&mut job);
    assert_eq!((totals[0].walls.members, totals[0].walls.value), (8, Some(210.0)));
    assert_eq!((totals[1].walls.members, totals[1].floors.value), (8, Some(1120.0)));
}

#[test]
fn a_takeoff_is_no_surer_than_what_is_on_the_system_and_one_size_is_one_line() {
    use plumbline_world::building::takeoff;
    use plumbline_world::{InSystem, Segment, SegmentType};
    let mut job = plumbline_world::sample::house();
    // The refrigerant line set is four straight pieces: 2 + 8.5 + 19.5 + 4.5 = 34.5 LF.
    let line_set = pieces(&mut job, "refrigerant line set");
    assert_eq!(line_set.len(), 4);
    let system = path_named(&mut job, "Refrigerant");
    // Said to be on the system, but only assumed so: its line is Assumed, and so is the total.
    for p in &line_set {
        job.put(p, InSystem(Val::assumed(system.clone(), "", "guessed from where it runs"))).unwrap();
    }
    let t = takeoff(&mut job);
    let refrigerant = t.iter().find(|s| s.name == "Refrigerant").unwrap();
    assert_eq!((refrigerant.lines[0].length.value, refrigerant.lines[0].length.status), (Some(34.5), Status::Assumed));
    assert_eq!(refrigerant.runs.status, Status::Assumed);
    // On a system nobody can name: not "on no system". It is in a group of its own,
    // flagged Unknown, with its length; and the system it left has no runs.
    for p in &line_set {
        job.put(p, InSystem(Val::unknown("", "not marked on the plan"))).unwrap();
    }
    let t = takeoff(&mut job);
    assert_eq!(t.iter().find(|s| s.name == "Refrigerant").unwrap().runs.members, 0);
    assert!(t.iter().all(|s| s.name != "On no system"));
    let lost = t.iter().find(|s| s.name == "System not known").expect("a group for what is on an unknown system");
    assert_eq!((lost.flag, lost.path.as_str(), lost.runs.value, lost.runs.members), (Status::Unknown, "", Some(34.5), 4));
    // With no system entered at all it is on no system, and that is Known.
    for p in &line_set {
        let e = job.find(p).unwrap();
        job.world.entity_mut(e).remove::<InSystem>();
    }
    let t = takeoff(&mut job);
    let none = t.iter().find(|s| s.name == "On no system").unwrap();
    assert_eq!((none.flag, none.runs.value), (Status::Known, Some(34.5)));

    // One size is one line, however each type's was typed, and lines are in order of size.
    let mut job = plumbline_world::sample::house();
    let a = path_named(&mut job, "branch to living west");
    let b = path_named(&mut job, "branch to kitchen");
    for (path, width) in [(&a, Val::known(Len::One(0.5), "ft", "test")), (&b, Val::known(Len::FtIn([0.0, 6.0]), "ft-in", "test"))] {
        let t = job.add("job-0001", Kind::Type, &format!("six inches, typed another way {}", job.order.len())).unwrap();
        job.put(&t, SegmentType::round(Kind::Duct, width)).unwrap();
        let e = job.find(path).unwrap();
        let mut piece = job.world.get::<Segment>(e).unwrap().clone();
        piece.type_name = named(&t);
        job.put(path, piece).unwrap();
    }
    job.derive();
    let t = takeoff(&mut job);
    let supply = t.iter().find(|s| s.name == "Supply air").unwrap();
    let sizes: Vec<&str> = supply.lines.iter().map(|l| l.size.as_str()).collect();
    assert_eq!(sizes, ["6 in round", "12 in × 8 in", "14 in × 8 in", "16 in × 8 in"]);
    // Nine branches: five on the first floor of one piece each and four on the second
    // of two, thirteen pieces, 88.4 LF.
    assert_eq!((supply.lines[0].length.members, supply.lines[0].length.value), (13, Some(88.4)));
    // The two typed in feet are Known where the rest are Assumed: the line is still Assumed.
    assert_eq!(supply.lines[0].size_status, Status::Assumed);
    // A run whose type says a size nobody has is a run of no known size, like one with no type.
    let unsized_type = job.add("job-0001", Kind::Type, "a duct nobody has sized").unwrap();
    job.put(&unsized_type, SegmentType::round(Kind::Duct, Val::unknown("in", "not on the schedule"))).unwrap();
    let e = job.find(&a).unwrap();
    let mut piece = job.world.get::<Segment>(e).unwrap().clone();
    piece.type_name = named(&unsized_type);
    job.put(&a, piece).unwrap();
    job.derive();
    let t = takeoff(&mut job);
    let supply = t.iter().find(|s| s.name == "Supply air").unwrap();
    let last = supply.lines.last().unwrap();
    assert_eq!((last.size.as_str(), last.size_status, last.length.value, last.length.members), ("size unknown", Status::Unknown, Some(11.0), 1));
    assert_eq!(measure_of(&mut job, &a, "surface_area").value, None);
}

#[test]
fn an_opening_that_cannot_be_cut_is_said_and_a_listing_keeps_a_door_under_its_wall() {
    use plumbline_world::building::placements;
    use plumbline_world::Opening;
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let door = job.children().get(&south).unwrap().get("front door").unwrap().clone();
    let len = |x: f64| Val::known(Len::One(x), "ft", "test");
    // A door moved past the end of its wall is not cut, and is listed as not cut, with why.
    let door_entity = job.find(&door).unwrap();
    let door_type = job.world.get::<Opening>(door_entity).unwrap().type_name.clone();
    job.put(&door, Opening { type_name: door_type, offset: len(40.5), sill: len(0.0) }).unwrap();
    job.derive();
    let (drawn, not_drawn) = placements(&mut job);
    let wall = drawn.iter().find(|p| p.path == south).unwrap();
    match &wall.shape {
        plumbline_world::building::Shape::Wall { holes, .. } => assert!(holes.iter().all(|h| h.0 != door)),
        other => panic!("{other:?}"),
    }
    assert!(not_drawn.contains(&(door.clone(), format!("it runs past the end of {south}, so it is not cut in it"))), "{not_drawn:?}");
    // One whose width nobody has is said too.
    let no_width = opening_type(&mut job, Kind::Door, Val::unknown("ft", "not on the plan"), len(6.67));
    job.put(&door, Opening { type_name: no_width, offset: len(20.0), sill: len(0.0) }).unwrap();
    let (_, not_drawn) = placements(&mut job);
    let said: Vec<&(String, String)> = not_drawn.iter().filter(|x| x.0 == door).collect();
    assert_eq!(said.len(), 1);
    assert!(said[0].1.contains("is not known"), "{}", said[0].1);
    // A flat reading lists each door and window straight after its own wall's rows, one
    // level in, whenever it was added: the front door is under the south wall.
    let rows = plumbline_world::view::elements(&mut job);
    let at = rows.iter().position(|e| e.path == south).unwrap();
    assert_eq!((rows[at + 1].path.as_str(), rows[at + 1].depth), (door.as_str(), rows[at].depth + 1));
    assert!(rows[at + 1..at + 4].iter().all(|e| e.kind.is_opening()));
    assert_eq!(rows.len(), job.order.len());
}

// ---------------------------------------------------------------- the re-base on BIMCraft: walls, openings, floors and their types

fn ft(x: f64) -> Val<Len> {
    Val::known(Len::One(x), "ft", "test")
}

fn named(path: &str) -> Val<String> {
    Val::known(path.to_string(), "", "test")
}

#[test]
fn a_wall_held_to_the_storey_above_is_as_tall_as_the_storeys_are_apart() {
    use plumbline_world::{Level, Wall};
    let mut job = plumbline_world::sample::house();
    let (first, second) = (path_named(&mut job, "first floor"), path_named(&mut job, "second floor"));
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    // The sample's first-floor outside walls run up to the second floor: 10 ft − 0 ft.
    let h = measure_of(&mut job, &south, "height");
    assert_eq!((h.value, h.status), (Some(10.0), Status::Known));
    assert!(h.inputs.contains(&format!("{second}#pl::level")), "{:?}", h.inputs);
    // Raise the second floor a foot and the wall is 11 ft tall and 440 SF, with nothing
    // typed on the wall. The door and two windows are still 70.01 SF: 369.99 net.
    job.put(&second, Level { elevation: ft(11.0), height: ft(9.0), computation_height: None }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "height").value, Some(11.0));
    assert_eq!(measure_of(&mut job, &south, "face_area").value, Some(440.0));
    assert_eq!(measure_of(&mut job, &south, "net_area").value, Some(369.99));
    // An elevation somebody assumed makes the height, the face and the net Assumed.
    job.put(&second, Level { elevation: Val::assumed(Len::One(11.0), "ft", "scaled off a section"), height: ft(9.0), computation_height: None }).unwrap();
    job.derive();
    for name in ["height", "face_area", "net_area"] {
        assert_eq!(measure_of(&mut job, &south, name).status, Status::Assumed, "{name}");
    }
    // One nobody has leaves no height and no area, and says which elevation.
    job.put(&second, Level { elevation: Val::unknown("ft", "not surveyed"), height: ft(9.0), computation_height: None }).unwrap();
    job.derive();
    let face = measure_of(&mut job, &south, "face_area");
    assert_eq!((face.value, face.status), (None, Status::Unknown));
    assert!(face.note.contains("the elevation of second floor, its top level, is not known"), "{}", face.note);
    assert_eq!(measure_of(&mut job, &south, "plan_length").value, Some(40.0));
    // Offsets: a top 1 ft 6 in under the storey above and a base 6 in above its own floor.
    // 11 − 1.5 − 0.5 = 9 ft.
    job.put(&second, Level { elevation: ft(11.0), height: ft(9.0), computation_height: None }).unwrap();
    let e = job.find(&south).unwrap();
    let mut wall = job.world.get::<Wall>(e).unwrap().clone();
    wall.top_offset = Some(Val::known(Len::FtIn([-1.0, -6.0]), "ft-in", "test"));
    wall.base_offset = Some(Val::known(Len::One(6.0), "in", "test"));
    job.put(&south, wall.clone()).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "height").value, Some(9.0));
    assert_eq!(measure_of(&mut job, &south, "face_area").value, Some(360.0));
    // A top at or under its base is not a wall: no height, and why.
    wall.top_offset = Some(ft(-10.5));
    job.put(&south, wall).unwrap();
    job.derive();
    let h = measure_of(&mut job, &south, "height");
    assert_eq!(h.value, None);
    assert!(h.note.contains("a wall's top is above its base"), "{}", h.note);
}

#[test]
fn a_size_is_entered_once_on_a_type_and_every_opening_of_it_follows() {
    use plumbline_world::OpeningType;
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let west_window = job.children().get(&south).unwrap().get("west window").unwrap().clone();
    // A window's width, height and area are its type's: 5 ft by 5 ft is 25 SF.
    assert_eq!(measure_of(&mut job, &west_window, "width").value, Some(5.0));
    assert_eq!(measure_of(&mut job, &west_window, "opening_area").value, Some(25.0));
    // Make the 5 ft by 5 ft type 6 ft wide. Both south windows are of it: each is 30 SF,
    // the wall's openings are 20.01 + 60 = 80.01 SF and its net is 319.99.
    let five = path_named(&mut job, "Window 5 ft × 5 ft");
    job.put(&five, OpeningType { category: Kind::Window, width: ft(6.0), height: ft(5.0) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &west_window, "opening_area").value, Some(30.0));
    assert_eq!(measure_of(&mut job, &south, "opening_area").value, Some(80.01));
    assert_eq!(measure_of(&mut job, &south, "net_area").value, Some(319.99));
    // A width in inches on the type is the same width: 72 in.
    job.put(&five, OpeningType { category: Kind::Window, width: Val::known(Len::One(72.0), "in", "test"), height: ft(5.0) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "net_area").value, Some(319.99));
    // An assumed size on the type makes every wall with one of its windows Assumed.
    job.put(&five, OpeningType { category: Kind::Window, width: Val::assumed(Len::One(6.0), "ft", "scaled"), height: ft(5.0) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "net_area").status, Status::Assumed);
    // An opening is placed by its centre. The sample's front door is 3 ft wide, centred
    // 20 ft along. Centred 1 ft along it would start half a foot before the wall does.
    let door = job.children().get(&south).unwrap().get("front door").unwrap().clone();
    let e = job.find(&door).unwrap();
    let mut o = job.world.get::<plumbline_world::Opening>(e).unwrap().clone();
    assert_eq!(o.offset.feet(), Some(20.0));
    o.offset = ft(1.0);
    job.put(&door, o).unwrap();
    job.derive();
    let net = measure_of(&mut job, &south, "net_area");
    assert!(net.note.contains("starts 0.50 ft before the start of its wall"), "{}", net.note);
    // An opening whose type nobody knows has no size, and its wall no net area.
    job.put(&door, plumbline_world::Opening { type_name: Val::unknown("", "not on the door schedule"), offset: ft(20.0), sill: ft(0.0) }).unwrap();
    job.derive();
    let area = measure_of(&mut job, &door, "opening_area");
    assert_eq!(area.value, None);
    assert!(area.note.contains("its type is not known"), "{}", area.note);
    assert_eq!(measure_of(&mut job, &south, "net_area").value, None);
}

#[test]
fn a_walls_line_can_be_any_line_of_the_wall_and_its_layers_add_up() {
    use plumbline_world::building::{check_wall, check_wall_type, placements, walls, Shape};
    use plumbline_world::{Function, Layer, Location, Wall, WallType};
    // A wall 0.4 ft thick: finish 0.05, structure 0.3, finish 0.05, from the exterior in.
    let layers = [(Function::Finish, 0.05), (Function::Structure, 0.3), (Function::Finish, 0.05)];
    // How far in from the exterior face each line lies.
    assert_eq!(walls::depth(0.4, &layers, Location::Centre), 0.2);
    assert_eq!(walls::depth(0.4, &layers, Location::ExteriorFinish), 0.0);
    assert_eq!(walls::depth(0.4, &layers, Location::InteriorFinish), 0.4);
    assert_eq!(walls::depth(0.4, &layers, Location::ExteriorCore), 0.05);
    assert!((walls::depth(0.4, &layers, Location::InteriorCore) - 0.35).abs() < 1e-12);
    assert!((walls::depth(0.4, &layers, Location::CoreCentre) - 0.2).abs() < 1e-12);
    // With no structure layer the core is the whole wall.
    assert_eq!(walls::depth(0.4, &[], Location::InteriorCore), 0.4);
    // The exterior is to the left of the line. Drawn on its exterior face, a wall's
    // faces are at 0 and −0.4, and its centre line is 0.2 to the right.
    assert_eq!(walls::wall_faces(0.4, &layers, Location::ExteriorFinish), (0.0, -0.4));
    assert_eq!(walls::centre(0.4, &layers, Location::ExteriorFinish), -0.2);

    // In a job: the sample's south wall is 6 in thick, drawn west to east along y = 0, so
    // its exterior is to the north of the line as drawn. Say the line is its exterior
    // face and the wall is drawn about a centre line 3 in to the south.
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let e = job.find(&south).unwrap();
    let mut wall = job.world.get::<Wall>(e).unwrap().clone();
    wall.location = Location::ExteriorFinish;
    job.put(&south, wall.clone()).unwrap();
    let (drawn, _) = placements(&mut job);
    match &drawn.iter().find(|p| p.path == south).unwrap().shape {
        Shape::Wall { a, b, thickness, .. } => assert_eq!((*a, *b, *thickness), ([0.0, -0.25], [40.0, -0.25], 0.5)),
        other => panic!("{other:?}"),
    }
    // Its length and its areas do not move: they are the line's.
    assert_eq!(measure_of(&mut job, &south, "plan_length").value, Some(40.0));

    // A type's layers add up to its thickness, or it is refused.
    let layer = |function, name: &str, inches: f64| Layer { function, material: Val::known(name.to_string(), "", "test"), thickness: Val::known(Len::One(inches), "in", "test") };
    let built_up = |thickness: f64| WallType { thickness: Val::known(Len::One(thickness), "in", "test"), layers: vec![layer(Function::Finish, "siding", 0.75), layer(Function::Membrane, "housewrap", 0.0), layer(Function::Structure, "2x6 studs", 5.5), layer(Function::Finish, "gypsum board", 0.5)] };
    assert!(check_wall_type("t", Kind::Type, &built_up(6.75)).is_ok());
    let err = check_wall_type("t", Kind::Type, &built_up(6.0)).unwrap_err();
    assert!(err.contains("its layers add up to 0.5625 ft, not its thickness, 0.5 ft"), "{err}");
    // Only a membrane may be as thin as nothing.
    let mut thin = built_up(6.75);
    thin.layers[0].thickness = Val::known(Len::One(0.0), "in", "test");
    assert!(check_wall_type("t", Kind::Type, &thin).is_err());
    assert!(check_wall_type("t", Kind::Wall, &built_up(6.75)).is_err());
    // A wall is a height tall or is held to a level: one, and not both or neither.
    let both = Wall { top_level: Some(named("storey-0003")), ..Wall::tall(named("type-0002"), Val::known([[0.0, 0.0], [1.0, 0.0]], "ft", "test"), ft(8.0)) };
    assert!(check_wall("w", Kind::Wall, &both).unwrap_err().contains("one or the other"));
    let neither = Wall { height: None, ..both.clone() };
    assert!(check_wall("w", Kind::Wall, &Wall { top_level: None, ..neither }).is_err());
    let offset_with_no_level = Wall { top_offset: Some(ft(1.0)), ..Wall::tall(named("type-0002"), Val::known([[0.0, 0.0], [1.0, 0.0]], "ft", "test"), ft(8.0)) };
    assert!(check_wall("w", Kind::Wall, &offset_with_no_level).unwrap_err().contains("top offset but no top level"));
}

#[test]
fn a_type_named_has_to_be_a_type_of_the_right_sort() {
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let outside = path_named(&mut job, "Outside wall, 6 in");
    let slab = path_named(&mut job, "Slab on grade, 4 in");
    let door = path_named(&mut job, "Door 3 ft × 6.67 ft");
    let window = path_named(&mut job, "Window 5 ft × 5 ft");
    let first = path_named(&mut job, "first floor");
    let swap = |from: &str, to: &str, n: usize| text.replacen(&format!("\"value\": \"{from}\""), &format!("\"value\": \"{to}\""), n);
    // A wall that names a floor type; a floor that names a wall type; a type that is not there.
    let err = file::load(&swap(&outside, &slab, 1)).err().expect("a wall of a floor type");
    assert!(err.contains("which is not a wall type in this job"), "{err}");
    let err = file::load(&swap(&slab, &outside, 1)).err().expect("a floor of a wall type");
    assert!(err.contains("which is not a floor type in this job"), "{err}");
    let err = file::load(&swap(&door, "type-9999", 1)).err().expect("a type that is not there");
    assert!(err.contains("which is not a door or window type in this job"), "{err}");
    // A door of a window's type.
    let err = file::load(&swap(&door, &window, 1)).err().expect("a door of a window type");
    assert!(err.contains("is a Door, but its type") && err.contains("is a Window's"), "{err}");
    // A top level that is not a storey.
    let second = path_named(&mut job, "second floor");
    let _ = second;
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let wall = doc["data"].as_array_mut().unwrap().iter_mut().find(|e| !e["attributes"]["pl::wall"]["top_level"].is_null()).expect("the saved job has a wall with a top level");
    wall["attributes"]["pl::wall"]["top_level"]["value"] = outside.clone().into();
    let broken = doc.to_string();
    let err = file::load(&broken).err().expect("a top level that is a type");
    assert!(err.contains("which is not a storey in this job"), "{err}");
    // A type under a storey instead of under the job.
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    for entry in doc["data"].as_array_mut().unwrap() {
        let path = entry["path"].as_str().unwrap().to_string();
        if path == "job-0001" {
            entry["children"].as_object_mut().unwrap().remove("Slab on grade, 4 in");
        } else if path == first {
            entry["children"].as_object_mut().unwrap().insert("Slab on grade, 4 in".into(), slab.clone().into());
        }
    }
    let err = file::load(&doc.to_string()).err().expect("a type under a storey");
    assert!(err.contains("a type belongs to the whole job"), "{err}");
    // Taken out of a job that is open, a type leaves its walls with no thickness to draw
    // and its openings with no size, each said: nothing falls back to a number.
    job.remove(&window).unwrap();
    job.derive();
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!(net.value, None);
    let (_, not_drawn) = plumbline_world::building::placements(&mut job);
    assert!(not_drawn.iter().any(|(_, why)| why.contains("is not known, so it is not cut in its wall")), "{not_drawn:?}");
}

// ---------------------------------------------------------------- the re-base on BIMCraft: connectors and the network

/// A job with one storey at the datum and a 2 in pipe type, a 1 in pipe type and a duct
/// type, for the network's tests: (job, storey, 2 in pipe, 1 in pipe, duct).
fn piping() -> (Job, String, String, String, String) {
    use plumbline_world::{Level, SegmentType};
    let mut job = Job::new("network test (not a real job)", "test", "test");
    let storey = job.add("job-0001", Kind::Storey, "ground").unwrap();
    job.put(&storey, Level { elevation: ft(0.0), height: ft(10.0), computation_height: None }).unwrap();
    let inch = |x: f64| Val::known(Len::One(x), "in", "test");
    let mut kind = |job: &mut Job, name: &str, t: SegmentType| {
        let p = job.add("job-0001", Kind::Type, name).unwrap();
        job.put(&p, t).unwrap();
        p
    };
    let two = kind(&mut job, "pipe 2 in", SegmentType::round(Kind::Pipe, inch(2.0)));
    let one = kind(&mut job, "pipe 1 in", SegmentType::round(Kind::Pipe, inch(1.0)));
    let duct = kind(&mut job, "duct 6 in", SegmentType::round(Kind::Duct, inch(6.0)));
    (job, storey, two, one, duct)
}

/// One straight piece from `a` to `b`, each [east, north, height above the storey].
fn piece(job: &mut Job, storey: &str, kind: Kind, name: &str, type_path: &str, a: [f64; 3], b: [f64; 3]) -> String {
    use plumbline_world::{Justify, Segment};
    let p = job.add(storey, kind, name).unwrap();
    job.put(&p, Segment { type_name: named(type_path), line: Val::known([[a[0], a[1]], [b[0], b[1]]], "ft", "test"), offset: ft(a[2]), end_offset: (a[2] != b[2]).then(|| ft(b[2])), justify: Justify::Centre }).unwrap();
    p
}

/// A placed thing with connectors, each (kind, domain, [x, y, z] in the thing's own axes).
fn thing(job: &mut Job, storey: &str, kind: Kind, name: &str, at: [f64; 3], angle: Option<f64>, connectors: &[(plumbline_world::ConnectorKind, Option<plumbline_world::Domain>, [f64; 3])]) -> String {
    use plumbline_world::{Connector, Connectors, Flow, Placed};
    let p = job.add(storey, kind, name).unwrap();
    job.put(&p, Placed { what: named(name), at: Val::known([at[0], at[1]], "ft", "test"), offset: ft(at[2]), angle: angle.map(|a| Val::known(a, "deg", "test")), size: Val::known([1.0, 1.0, 1.0], "ft", "test") }).unwrap();
    job.put(&p, Connectors(connectors.iter().map(|(kind, domain, at)| Connector { name: String::new(), kind: *kind, domain: *domain, flow: Flow::Both, at: Val::known(*at, "ft", "test"), size: None }).collect())).unwrap();
    p
}

fn net(job: &mut Job) -> plumbline_world::building::network::Network {
    use plumbline_world::building::network::{network, TOLERANCE_FT};
    network(job, None, None, TOLERANCE_FT).unwrap()
}

fn kinds(n: &plumbline_world::building::network::Network) -> Vec<&'static str> {
    n.warnings.iter().map(|w| w.kind).collect()
}

#[test]
fn pipes_end_to_end_are_one_network_with_two_open_ends() {
    let (mut job, s, two, ..) = piping();
    // 10 ft east, then 6 ft north, 1 ft above the floor.
    piece(&mut job, &s, Kind::Pipe, "a", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    piece(&mut job, &s, Kind::Pipe, "b", &two, [10.0, 0.0, 1.0], [10.0, 6.0, 1.0]);
    let n = net(&mut job);
    assert_eq!((n.nodes.len(), n.edges.len(), n.groups.len()), (3, 2, 1));
    assert_eq!(n.open.len(), 2, "the first pipe's start and the last's end");
    assert_eq!((n.edges[0].length, n.edges[1].length), (Some(10.0), Some(6.0)));
    assert!(n.warnings.is_empty() && n.left_out.is_empty());
    // Nothing is stored: the saved job is the same before and after asking.
    let before = file::save(&mut job);
    net(&mut job);
    assert_eq!(file::save(&mut job), before);
}

#[test]
fn a_connector_is_a_node_a_fitting_joins_through_it_and_a_fixture_does_not() {
    use plumbline_world::building::network::Joint;
    use plumbline_world::{ConnectorKind::Pipe, Domain};
    let (mut job, s, two, ..) = piping();
    // A water closet whose waste outlet is a foot to the north of its middle, at the
    // floor. A drain starts there. Its cold inlet meets nothing: it is open.
    let wc = thing(&mut job, &s, Kind::Fixture, "wc", [5.0, 5.0, 0.0], None, &[(Pipe, Some(Domain::Waste), [0.0, 1.0, 0.0]), (Pipe, Some(Domain::Cold), [-0.5, 0.0, 1.0])]);
    let drain = piece(&mut job, &s, Kind::Pipe, "drain", &two, [5.0, 6.0, 0.0], [12.0, 6.0, -0.2]);
    let n = net(&mut job);
    let at_outlet = n.nodes.iter().find(|x| x.at == [5.0, 6.0, 0.0]).expect("a node at the outlet");
    assert_eq!(at_outlet.joints, vec![Joint::Connector { element: wc.clone(), index: 0, domain: Some(Domain::Waste) }, Joint::RunEnd { run: drain.clone(), end: false }]);
    // Open: the cold inlet, and the far end of the drain.
    assert_eq!(n.open.len(), 2);
    assert!(n.open.iter().any(|&i| n.nodes[i].joints == vec![Joint::Connector { element: wc.clone(), index: 1, domain: Some(Domain::Cold) }]));
    // A pipe to the cold inlet is a second network: a fixture's waste and its water are
    // not joined through it.
    piece(&mut job, &s, Kind::Pipe, "cold", &two, [4.5, 5.0, 1.0], [4.5, 0.0, 1.0]);
    let n = net(&mut job);
    assert_eq!(n.groups.len(), 2);
    assert!(kinds(&n).is_empty(), "waste and cold are in networks of their own: {:?}", n.warnings);

    // A fitting does join what meets its two ends. Two pipes that stop half a foot short
    // of a corner are two networks; an elbow whose legs reach them makes them one, and
    // the way through it is an edge with no length.
    let (mut job, s, two, ..) = piping();
    piece(&mut job, &s, Kind::Pipe, "a", &two, [0.0, 0.0, 1.0], [9.5, 0.0, 1.0]);
    piece(&mut job, &s, Kind::Pipe, "b", &two, [10.0, 0.5, 1.0], [10.0, 6.0, 1.0]);
    assert_eq!(net(&mut job).groups.len(), 2);
    let elbow = thing(&mut job, &s, Kind::Fitting, "elbow", [10.0, 0.0, 1.0], None, &[(Pipe, None, [-0.5, 0.0, 0.0]), (Pipe, None, [0.0, 0.5, 0.0])]);
    let n = net(&mut job);
    assert_eq!((n.groups.len(), n.open.len()), (1, 2));
    assert!(n.edges.iter().any(|e| e.element == elbow && e.length.is_none()));
    // Turned a quarter turn counter-clockwise, a point (x, y) of the elbow goes to
    // (−y, x). The leg that was at (9.5, 0) is at (10, −0.5) and meets nothing; the leg
    // that was at (10, 0.5) is at (9.5, 0), on the end of pipe a. Pipe b is on its own.
    let e = job.find(&elbow).unwrap();
    job.world.get_mut::<plumbline_world::Placed>(e).unwrap().angle = Some(Val::known(90.0, "deg", "test"));
    let n = net(&mut job);
    let near = |x: &plumbline_world::building::network::Node, e: f64, nn: f64| (x.at[0] - e).abs() < 1e-9 && (x.at[1] - nn).abs() < 1e-9;
    let loose = n.nodes.iter().find(|x| near(x, 10.0, -0.5)).expect("a node at (10, −0.5)");
    assert_eq!(loose.joints, vec![Joint::Connector { element: elbow.clone(), index: 0, domain: None }]);
    let on_a = n.nodes.iter().find(|x| near(x, 9.5, 0.0)).expect("a node at (9.5, 0)");
    assert!(on_a.joints.contains(&Joint::Connector { element: elbow.clone(), index: 1, domain: None }) && on_a.joints.len() == 2);
    assert_eq!(n.groups.len(), 2);
}

#[test]
fn what_looks_wrong_in_a_network_is_a_warning_and_never_a_refusal() {
    use plumbline_world::building::network::{network, Joint};
    use plumbline_world::{ConnectorKind::Pipe, Domain, InSystem, SystemInfo};
    // Two systems in one network.
    let (mut job, s, two, one, _) = piping();
    let mut system = |job: &mut Job, name: &str, classification: &str| {
        let p = job.add("job-0001", Kind::System, name).unwrap();
        job.put(&p, SystemInfo { discipline: plumbline_world::Discipline::Plumbing, classification: classification.into() }).unwrap();
        p
    };
    let (cold, hot) = (system(&mut job, "Cold", "Domestic Cold Water"), system(&mut job, "Hot", "Domestic Hot Water"));
    let a = piece(&mut job, &s, Kind::Pipe, "a", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    let b = piece(&mut job, &s, Kind::Pipe, "b", &two, [10.0, 0.0, 1.0], [10.0, 6.0, 1.0]);
    job.put(&a, InSystem(named(&cold))).unwrap();
    job.put(&b, InSystem(named(&hot))).unwrap();
    let n = net(&mut job);
    assert_eq!(kinds(&n), ["systems_meet"]);
    assert_eq!(n.warnings[0].message, "runs of the systems Cold, Hot are joined in one network");
    // Asked for one system, the network is what that system's runs are joined to.
    assert_eq!(network(&mut job, Some(&cold), None, 0.01).unwrap().groups.len(), 1);
    let c = piece(&mut job, &s, Kind::Pipe, "c", &one, [0.0, 20.0, 1.0], [5.0, 20.0, 1.0]);
    assert_eq!(net(&mut job).groups.len(), 2);
    assert_eq!(network(&mut job, Some(&cold), None, 0.01).unwrap().groups.len(), 1, "c is on no system and joins nothing of Cold's");
    let _ = c;

    // A waste connector and a cold-water connector in one network.
    let (mut job, s, two, ..) = piping();
    thing(&mut job, &s, Kind::Fixture, "basin", [0.0, 0.0, 0.0], None, &[(Pipe, Some(Domain::Waste), [0.0, 0.0, 0.0])]);
    thing(&mut job, &s, Kind::Fixture, "heater", [10.0, 0.0, 0.0], None, &[(Pipe, Some(Domain::Cold), [0.0, 0.0, 0.0])]);
    piece(&mut job, &s, Kind::Pipe, "between", &two, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let n = net(&mut job);
    assert_eq!(kinds(&n), ["domains_meet"]);
    assert_eq!(n.warnings[0].message, "connectors of cold, waste are joined in one network");

    // A branch drawn to a main's centre line, 4 ft along it, with no fitting: the main is
    // two edges, 4 ft and 6 ft, of the one element, and the join is warned of.
    let (mut job, s, two, one, _) = piping();
    let main = piece(&mut job, &s, Kind::Pipe, "main", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    piece(&mut job, &s, Kind::Pipe, "branch", &one, [4.0, 0.0, 1.0], [4.0, 3.0, 1.0]);
    let n = net(&mut job);
    let on_main: Vec<Option<f64>> = n.edges.iter().filter(|e| e.element == main).map(|e| e.length).collect();
    assert_eq!(on_main, [Some(4.0), Some(6.0)]);
    assert_eq!(kinds(&n), ["joined_mid_run"]);
    assert!(n.nodes.iter().any(|x| x.joints.contains(&Joint::RunSide { run: main.clone() })));
    assert_eq!((n.groups.len(), n.open.len()), (1, 3));

    // A drain that narrows on its way down: 2 in coming down a stack to a 1 in pipe
    // leaving below it. Read where a waste connector says the network is a drain.
    let (mut job, s, two, one, _) = piping();
    thing(&mut job, &s, Kind::Fixture, "wc", [0.0, 0.0, 8.0], None, &[(Pipe, Some(Domain::Waste), [0.0, 0.0, 0.0])]);
    piece(&mut job, &s, Kind::Pipe, "stack", &two, [0.0, 0.0, 8.0], [0.0, 0.0, 0.0]);
    piece(&mut job, &s, Kind::Pipe, "drain", &one, [0.0, 0.0, 0.0], [10.0, 0.0, -0.5]);
    let n = net(&mut job);
    assert_eq!(kinds(&n), ["smaller_downstream"]);
    assert_eq!(n.warnings[0].message, "a drain narrows downstream: 1 in leaving where 2 in comes in");
    // The same two pipes the right way round say nothing; nor does one nobody has sized.
    let (mut job, s, two, one, _) = piping();
    thing(&mut job, &s, Kind::Fixture, "wc", [0.0, 0.0, 8.0], None, &[(Pipe, Some(Domain::Waste), [0.0, 0.0, 0.0])]);
    piece(&mut job, &s, Kind::Pipe, "stack", &one, [0.0, 0.0, 8.0], [0.0, 0.0, 0.0]);
    let drain = piece(&mut job, &s, Kind::Pipe, "drain", &two, [0.0, 0.0, 0.0], [10.0, 0.0, -0.5]);
    assert!(kinds(&net(&mut job)).is_empty());
    let e = job.find(&drain).unwrap();
    job.world.get_mut::<plumbline_world::Segment>(e).unwrap().type_name = Val::unknown("", "not on the plan");
    let n = net(&mut job);
    assert!(kinds(&n).is_empty() && n.left_out.is_empty() && n.groups.len() == 1, "a run with no size is still joined: {n:?}");
}

#[test]
fn how_close_is_one_node_and_what_cannot_be_placed_is_said() {
    use plumbline_world::building::network::network;
    let (mut job, s, two, _, duct) = piping();
    // Two pipes whose ends are 0.03 ft apart: two networks at the usual 0.01 ft, one at
    // 0.05 ft. Asked for a foot, it is held to 2 in.
    piece(&mut job, &s, Kind::Pipe, "a", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    piece(&mut job, &s, Kind::Pipe, "b", &two, [10.03, 0.0, 1.0], [20.0, 0.0, 1.0]);
    assert_eq!(network(&mut job, None, None, 0.01).unwrap().groups.len(), 2);
    assert_eq!(network(&mut job, None, None, 0.05).unwrap().groups.len(), 1);
    piece(&mut job, &s, Kind::Pipe, "c", &two, [20.5, 0.0, 1.0], [30.0, 0.0, 1.0]);
    assert_eq!(network(&mut job, None, None, 1.0).unwrap().groups.len(), 2, "half a foot is not one node at any tolerance");
    // A duct that ends on the very point a pipe ends at does not join it: a run joins its
    // own kind.
    piece(&mut job, &s, Kind::Duct, "d", &duct, [0.0, 0.0, 1.0], [0.0, 9.0, 1.0]);
    let n = network(&mut job, None, None, 0.05).unwrap();
    assert_eq!(n.groups.len(), 3);
    assert!(n.warnings.is_empty());
    // A second storey whose elevation nobody has: what is on it cannot be placed, so it
    // is in no network, and it is listed with why. It is not joined at a guessed height.
    let upper = job.add("job-0001", Kind::Storey, "upper").unwrap();
    job.put(&upper, plumbline_world::Level { elevation: Val::unknown("ft", "not surveyed"), height: ft(9.0), computation_height: None }).unwrap();
    let lost = piece(&mut job, &upper, Kind::Pipe, "upstairs", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    let n = network(&mut job, None, None, 0.05).unwrap();
    assert_eq!(n.left_out, vec![(lost.clone(), "its storey's elevation is not known".to_string())]);
    assert_eq!(n.groups.len(), 3);
    // Asked for one storey, only what is under it.
    assert_eq!(network(&mut job, None, Some(&upper), 0.05).unwrap().groups.len(), 0);
    assert_eq!(network(&mut job, None, Some(&s), 0.05).unwrap().left_out.len(), 0);
}

#[test]
fn the_sample_house_says_what_is_joined_what_is_open_and_where_a_tee_is_missing() {
    use plumbline_world::building::takeoff;
    let mut job = plumbline_world::sample::house();
    let systems = takeoff(&mut job);
    let of = |name: &str| &systems.iter().find(|s| s.name == name).unwrap().joined;
    // Supply air. The first floor's ducts and the attic's each start at the air handler,
    // and equipment does not join what meets it: two networks. The two ends of each
    // trunk are open (four). Five branches leave the first-floor trunks along their
    // length, the riser meets the attic trunk along its length, and four branches leave
    // the attic trunk: ten places with no fitting entered.
    let supply = of("Supply air");
    assert_eq!((supply.networks, supply.open.len(), supply.no_fitting.len()), (2, 4, 10));
    assert!(supply.open.iter().any(|o| o.said == "attic trunk: its first end" && o.at == [4.0, 14.0, 20.0]), "{:?}", supply.open);
    // Return air and refrigerant are each one network with nothing open: every end is on
    // a grille, a unit, or an elbow.
    for name in ["Return air", "Refrigerant"] {
        assert_eq!((of(name).networks, of(name).open.len(), of(name).no_fitting.len()), (1, 0, 0), "{name}");
    }
    // Cold water is one network; the only open end is the service where it comes in from
    // the street.
    let cold = of("Cold water");
    assert_eq!((cold.networks, cold.open.len()), (1, 1));
    assert_eq!(cold.open[0].said, "water service, piece 1 of 4: its first end");
    assert!(cold.warnings.is_empty(), "the drain's warnings are not the cold water's: {:?}", cold.warnings);
    // The drain and its vent are joined, as they are built, and that is said on both.
    for name in ["Sanitary", "Vent"] {
        assert_eq!(of(name).warnings, ["runs of the systems Sanitary, Vent are joined in one network"], "{name}");
    }
    assert_eq!(of("Sanitary").open.iter().map(|o| o.said.as_str()).collect::<Vec<_>>(), ["building drain: its second end"]);
    // Every fixture's connector is met by a run: no connector is among the open ends.
    assert!(systems.iter().flat_map(|s| &s.joined.open).all(|o| !o.said.contains("connector")));
    // Nothing was left out of the check.
    assert!(systems.iter().all(|s| s.joined.left_out.is_empty()));
    // The sample saves and loads with its connectors and fittings, to the same bytes.
    let text = file::save(&mut job);
    let (mut again, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(file::save(&mut again), text);
}

#[test]
fn a_connector_has_to_make_sense() {
    use plumbline_world::building::check_connectors;
    use plumbline_world::{Connector, ConnectorKind, Connectors, Domain, Flow};
    let at = |unit: &str| Val::known([0.0, 0.0, 0.0], unit, "test");
    let one = |kind, domain, unit: &str| Connectors(vec![Connector { name: "x".into(), kind, domain, flow: Flow::In, at: at(unit), size: None }]);
    assert!(check_connectors("f", Kind::Fixture, &one(ConnectorKind::Pipe, Some(Domain::Cold), "ft")).is_ok());
    assert!(check_connectors("f", Kind::Fitting, &one(ConnectorKind::Pipe, None, "in")).is_ok());
    // Only a pipe connector says what it carries; a wall has no connectors; a position
    // has a unit, and not a metric one.
    assert!(check_connectors("f", Kind::Fixture, &one(ConnectorKind::Duct, Some(Domain::Cold), "ft")).is_err());
    assert!(check_connectors("f", Kind::Wall, &one(ConnectorKind::Pipe, None, "ft")).is_err());
    assert!(check_connectors("f", Kind::Fixture, &one(ConnectorKind::Pipe, None, "")).is_err());
    assert!(check_connectors("f", Kind::Fixture, &one(ConnectorKind::Pipe, None, "mm")).is_err());
    let sized = Connectors(vec![Connector { name: String::new(), kind: ConnectorKind::Pipe, domain: None, flow: Flow::Both, at: at("ft"), size: Some(Val::known(Len::One(0.0), "in", "test")) }]);
    assert!(check_connectors("f", Kind::Fixture, &sized).is_err(), "a size of nothing");
}

// ---------------------------------------------------------------- the re-base on BIMCraft: rooms

/// A job with one storey at the datum and a 6 in wall type: (job, storey, wall type).
fn walled() -> (Job, String, String) {
    use plumbline_world::{Level, WallType};
    let mut job = Job::new("room test (not a real job)", "test", "test");
    let storey = job.add("job-0001", Kind::Storey, "ground").unwrap();
    job.put(&storey, Level { elevation: ft(0.0), height: ft(10.0), computation_height: None }).unwrap();
    let t = job.add("job-0001", Kind::Type, "wall 6 in").unwrap();
    job.put(&t, WallType { thickness: Val::known(Len::One(6.0), "in", "test"), layers: Vec::new() }).unwrap();
    (job, storey, t)
}

fn wall(job: &mut Job, storey: &str, type_path: &str, name: &str, a: [f64; 2], b: [f64; 2], tall: f64) -> String {
    let w = job.add(storey, Kind::Wall, name).unwrap();
    job.put(&w, plumbline_world::Wall::tall(named(type_path), Val::known([a, b], "ft", "test"), ft(tall))).unwrap();
    w
}

fn room(job: &mut Job, storey: &str, name: &str, at: [f64; 2]) -> String {
    let r = job.add(storey, Kind::Room, name).unwrap();
    job.put(&r, plumbline_world::Room { point: Val::known(at, "ft", "test"), number: None }).unwrap();
    r
}

/// Four walls round a rectangle from the origin, drawn counter-clockwise from the
/// south-west corner: [south, east, north, west].
fn four_walls(job: &mut Job, storey: &str, type_path: &str, east: f64, north: f64) -> [String; 4] {
    [
        wall(job, storey, type_path, "south", [0.0, 0.0], [east, 0.0], 9.0),
        wall(job, storey, type_path, "east", [east, 0.0], [east, north], 9.0),
        wall(job, storey, type_path, "north", [east, north], [0.0, north], 9.0),
        wall(job, storey, type_path, "west", [0.0, north], [0.0, 0.0], 9.0),
    ]
}

fn figures_of(job: &mut Job, room: &str) -> [Option<f64>; 3] {
    ["floor_area", "gross_area", "perimeter"].map(|name| measure_of(job, room, name).value)
}

#[test]
fn a_room_is_the_space_its_walls_enclose_measured_inside_their_faces_and_to_their_centres() {
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let room_named = |job: &mut Job, name: &str| job.children().get(&first).unwrap().get(name).unwrap().clone();
    // The sample's first floor is 40 ft by 28 ft to the centres of its outside walls,
    // which are 6 in thick. A hall partition, 4 1/2 in thick, runs east-west at 16 ft.
    //
    // The living room is all of it south of the hall: 40 by 16 to centre lines, 640 SF.
    // Inside the faces it is 40 − 0.25 − 0.25 = 39.5 ft by 16 − 0.25 − 0.1875 = 15.5625 ft:
    // 614.7188 SF, and 2 × (39.5 + 15.5625) = 110.125 LF round.
    let living = room_named(&mut job, "living room");
    assert_eq!(figures_of(&mut job, &living), [Some(614.7188), Some(640.0), Some(110.125)]);
    // The bath is between partitions at 14 and 24 ft, the hall, and the wet wall (6 in) at
    // 22 ft: 10 by 6, 60 SF to centre lines. Inside: 10 − 0.1875 − 0.1875 = 9.625 by
    // 6 − 0.1875 − 0.25 = 5.5625: 53.5391 SF, 30.375 LF round.
    let bath = room_named(&mut job, "bath");
    assert_eq!(figures_of(&mut job, &bath), [Some(53.5391), Some(60.0), Some(30.375)]);
    // The kitchen: 14 by 12 is 168; inside 13.5625 by 11.5625 is 156.8164; 50.25 round.
    let kitchen = room_named(&mut job, "kitchen");
    assert_eq!(figures_of(&mut job, &kitchen), [Some(156.8164), Some(168.0), Some(50.25)]);
    // The five rooms' areas to centre lines are the whole floor: 640 + 168 + 60 + 60 + 192.
    let gross: f64 = ["living room", "kitchen", "bath", "utility", "office"].iter().map(|n| room_named(&mut job, n)).collect::<Vec<_>>().iter().map(|r| measure_of(&mut job, r, "gross_area").value.unwrap()).sum();
    assert_eq!(gross, 1120.0);
    let totals = plumbline_world::building::storey_totals(&mut job);
    assert_eq!(totals[0].floors.value, Some(1120.0));
    // And inside their walls they come to 614.7188 + 156.8164 + 53.5391 + 52.9375 + 179.9414
    // = 1,057.9532 SF: the rest is under the walls. (A total is the sum of the figures
    // as each is shown, to four decimals, so it is what adding the lines up gives.)
    assert_eq!((totals[0].rooms.value, totals[0].rooms.members, totals[0].rooms.status), (Some(1057.9532), 5, Status::Known));
    // A room's figure names the walls it is measured to, and is Known when they are.
    let d = measure_of(&mut job, &bath, "floor_area");
    assert_eq!(d.status, Status::Known);
    assert_eq!(d.inputs.len(), 5, "the room and its four walls: {:?}", d.inputs);

    // Move the wet wall a foot north, to 23 ft, and the bath grows by 9.625 SF and the
    // utility room shrinks by as much, with nothing typed on either room.
    let wet = room_named(&mut job, "wet wall");
    let e = job.find(&wet).unwrap();
    job.world.get_mut::<plumbline_world::Wall>(e).unwrap().line = Val::known([[14.0, 23.0], [24.0, 23.0]], "ft", "test");
    job.derive();
    assert_eq!(measure_of(&mut job, &bath, "floor_area").value, Some(63.1641));
    let utility = room_named(&mut job, "utility");
    assert_eq!(measure_of(&mut job, &utility, "floor_area").value, Some(43.3125));
    // Say the wet wall does not bound rooms and the bath is both rooms' space: 10 by 12
    // to centre lines. The utility room, placed after it, is in a space that is taken.
    job.world.get_mut::<plumbline_world::Wall>(e).unwrap().room_bounding = false;
    job.derive();
    assert_eq!(measure_of(&mut job, &bath, "gross_area").value, Some(120.0));
    let lost = measure_of(&mut job, &utility, "floor_area");
    assert_eq!((lost.value, lost.status), (None, Status::Unknown));
    assert_eq!(lost.note, "bath was placed first and has the space this room's point is in");
}

#[test]
fn a_room_that_is_not_closed_or_is_in_a_wall_has_no_area_and_says_why() {
    let (mut job, s, t) = walled();
    // Three walls of a 20 ft by 12 ft room: open to the west.
    wall(&mut job, &s, &t, "south", [0.0, 0.0], [20.0, 0.0], 9.0);
    wall(&mut job, &s, &t, "east", [20.0, 0.0], [20.0, 12.0], 9.0);
    wall(&mut job, &s, &t, "north", [20.0, 12.0], [0.0, 12.0], 9.0);
    let r = room(&mut job, &s, "den", [10.0, 6.0]);
    job.derive();
    for name in ["floor_area", "gross_area", "perimeter"] {
        let d = measure_of(&mut job, &r, name);
        assert_eq!((d.value, d.status, d.note.as_str()), (None, Status::Unknown, "no loop of walls closes round its point"), "{name}");
    }
    // Close it. 20 by 12 to centre lines is 240 SF; inside 19.5 by 11.5 is 224.25, 62 LF round.
    wall(&mut job, &s, &t, "west", [0.0, 12.0], [0.0, 0.0], 9.0);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(224.25), Some(240.0), Some(62.0)]);
    // A point in the body of a wall is not in a room at its faces.
    let in_wall = room(&mut job, &s, "in the wall", [10.0, 0.1]);
    job.derive();
    assert_eq!(measure_of(&mut job, &in_wall, "floor_area").note, "its point is inside a wall");
    // A point outside the walls is in no room.
    let outside = room(&mut job, &s, "outside", [30.0, 6.0]);
    job.derive();
    assert_eq!(measure_of(&mut job, &outside, "gross_area").note, "no loop of walls closes round its point");
    // A wall drawn by somebody's guess makes the room it bounds Assumed.
    let west = path_named(&mut job, "west");
    let e = job.find(&west).unwrap();
    job.world.get_mut::<plumbline_world::Wall>(e).unwrap().line = Val::assumed([[0.0, 12.0], [0.0, 0.0]], "ft", "scaled off a sketch");
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Assumed);
    // A wall of the storey that cannot be placed (nobody has its height) leaves every room
    // on the storey with no figure: whether it closes a room off is not known.
    job.world.get_mut::<plumbline_world::Wall>(e).unwrap().height = Some(Val::unknown("ft", "not on the section"));
    job.derive();
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!(d.value, None);
    assert_eq!(d.note, "west cannot be placed (its height is not known), so what bounds the rooms of this storey is not known");
    // A room on another storey is still measured, but is Assumed, and says what it assumes.
    let upper = job.add("job-0001", Kind::Storey, "upper").unwrap();
    job.put(&upper, plumbline_world::Level { elevation: ft(10.0), height: ft(9.0), computation_height: None }).unwrap();
    four_walls(&mut job, &upper, &t, 20.0, 12.0);
    let up = room(&mut job, &upper, "loft", [10.0, 6.0]);
    job.derive();
    let d = measure_of(&mut job, &up, "floor_area");
    assert_eq!((d.value, d.status), (Some(224.25), Status::Assumed));
    assert_eq!(d.note, "assumes west does not bound this room: it cannot be placed (its height is not known)");
}

#[test]
fn rooms_close_at_corners_whatever_line_a_wall_was_drawn_on_and_an_island_is_taken_out() {
    use plumbline_world::{Location, Wall};
    // A 10 ft by 10 ft square drawn clockwise on the walls' outside faces. The exterior
    // is to the left of the way a wall is drawn, so drawn clockwise the walls stand inside
    // the square. They are 6 in thick: 9 ft by 9 ft inside, 81 SF, 36 LF round; to their
    // centre lines 9.5 by 9.5, 90.25 SF. The centre lines are run on to meet at the corners.
    let (mut job, s, t) = walled();
    for (name, a, b) in [("west", [0.0, 0.0], [0.0, 10.0]), ("north", [0.0, 10.0], [10.0, 10.0]), ("east", [10.0, 10.0], [10.0, 0.0]), ("south", [10.0, 0.0], [0.0, 0.0])] {
        let w = job.add(&s, Kind::Wall, name).unwrap();
        job.put(&w, Wall { location: Location::ExteriorFinish, ..Wall::tall(named(&t), Val::known([a, b], "ft", "test"), ft(9.0)) }).unwrap();
    }
    let r = room(&mut job, &s, "box", [5.0, 5.0]);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(81.0), Some(90.25), Some(36.0)]);

    // A 20 ft square of walls with a 4 ft square of walls standing free inside it, all to
    // centre lines. To centre lines the room is 400 − 16 = 384 SF. Inside the faces it is
    // 19.5² less the island's outside, 4.5²: 380.25 − 20.25 = 360 SF, and it is
    // 4 × 19.5 + 4 × 4.5 = 96 LF round, the island's faces included.
    let (mut job, s, t) = walled();
    four_walls(&mut job, &s, &t, 20.0, 20.0);
    for (name, a, b) in [("i south", [8.0, 8.0], [12.0, 8.0]), ("i east", [12.0, 8.0], [12.0, 12.0]), ("i north", [12.0, 12.0], [8.0, 12.0]), ("i west", [8.0, 12.0], [8.0, 8.0])] {
        wall(&mut job, &s, &t, name, a, b, 9.0);
    }
    let r = room(&mut job, &s, "hall", [3.0, 3.0]);
    let shaft = room(&mut job, &s, "shaft", [10.0, 10.0]);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(360.0), Some(384.0), Some(96.0)]);
    // The space inside the island is a room of its own: 4 by 4, and 3.5 by 3.5 inside.
    assert_eq!(figures_of(&mut job, &shaft), [Some(12.25), Some(16.0), Some(14.0)]);

    // A wall that stops short of where a room's walls are found does not bound it. A 3 ft
    // knee wall across a 20 ft by 12 ft room: at the usual 4 ft the room runs past it; told
    // to look at 2 ft, the storey finds two rooms.
    let (mut job, s, t) = walled();
    four_walls(&mut job, &s, &t, 20.0, 12.0);
    wall(&mut job, &s, &t, "knee wall", [10.0, 0.0], [10.0, 12.0], 3.0);
    let r = room(&mut job, &s, "west half", [5.0, 6.0]);
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "gross_area").value, Some(240.0));
    job.put(&s, plumbline_world::Level { elevation: ft(0.0), height: ft(10.0), computation_height: Some(ft(2.0)) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "gross_area").value, Some(120.0));
    // Inside: 10 − 0.25 − 0.25 = 9.5 by 11.5 is 109.25.
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(109.25));
}

#[test]
fn a_room_in_the_file_has_to_be_under_a_storey_and_have_a_point_that_makes_sense() {
    use plumbline_world::building::check_room;
    use plumbline_world::Room;
    assert!(check_room("r", Kind::Room, &Room { point: Val::known([1.0, 1.0], "ft", "test"), number: None }).is_ok());
    assert!(check_room("r", Kind::Room, &Room { point: Val::known([1.0, 1.0], "", "test"), number: None }).is_err());
    assert!(check_room("r", Kind::Room, &Room { point: Val::known([1.0, 1.0], "m", "test"), number: None }).is_err());
    assert!(check_room("r", Kind::Wall, &Room { point: Val::known([1.0, 1.0], "ft", "test"), number: None }).is_err());
    assert!(check_room("r", Kind::Room, &Room { point: Val::unknown("ft", "not placed yet"), number: None }).is_ok());
    // A room whose point nobody has is listed with no figure, not left out.
    let (mut job, s, t) = walled();
    four_walls(&mut job, &s, &t, 20.0, 12.0);
    let r = job.add(&s, Kind::Room, "somewhere").unwrap();
    job.put(&r, Room { point: Val::unknown("ft", "not placed yet"), number: None }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").note, "its point is not known");
    // A room under the job, not under a storey, does not load.
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let (first, living) = (path_named(&mut job, "first floor"), path_named(&mut job, "living room"));
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    for entry in doc["data"].as_array_mut().unwrap() {
        let path = entry["path"].as_str().unwrap().to_string();
        if path == first {
            entry["children"].as_object_mut().unwrap().remove("living room");
        } else if path == "job-0001" {
            entry["children"].as_object_mut().unwrap().insert("living room".into(), living.clone().into());
        }
    }
    let err = file::load(&doc.to_string()).err().expect("a room under the job");
    assert!(err.contains("a room is a space on a storey"), "{err}");
}

// ---------------------------------------------------------------- D4: macro to micro

fn priced_of(job: &mut Job, path: &str, name: &str) -> plumbline_world::Derived {
    let e = job.find(path).unwrap();
    job.world.get::<plumbline_world::Priced>(e).unwrap_or_else(|| panic!("{path} has no price")).0.get(name).unwrap_or_else(|| panic!("{path} has no {name}")).clone()
}

fn extension_of(job: &mut Job, path: &str, line: &str) -> plumbline_world::Derived {
    let e = job.find(path).unwrap();
    job.world.get::<plumbline_world::LineQuantities>(e).unwrap().0.get(line).unwrap().extension.clone().expect("an extension")
}

/// The sample job priced both ways with made-up prices. Nothing here is from a real job.
/// South plane: 419.2627 SF. Shed: 98.9545 SF, Assumed (its pitch is). Headwall: 12 LF,
/// with a flashing line of 24 SF.
fn priced_sample() -> Job {
    let mut job = sample();
    let known = Status::Known;
    job.set("plane-0003", "allowance", Some("$28.00/SF"), known, "made-up budget rate").unwrap();
    job.set("plane-0017", "allowance", Some("28"), known, "made-up budget rate").unwrap();
    job.set("headwall-0021", "allowance", Some("40.00 $/LF"), known, "made-up budget rate").unwrap();
    job.set("plane-0003", "unit_cost", Some("31.50"), known, "made-up quote").unwrap();
    job.set("headwall-0021", "unit_cost", Some("$18.25/SF"), known, "made-up quote").unwrap();
    job.derive();
    job
}

#[test]
fn a_job_is_priced_from_allowances_and_from_its_lines_and_the_two_are_set_against_each_other() {
    let mut job = priced_sample();
    // Design phase. The south plane is 419.2627 SF at $28.00: $11,739.3556, $11,739.36.
    let a = priced_of(&mut job, "plane-0003", "allowance");
    assert_eq!((a.value, a.unit.as_str(), a.status), (Some(11739.36), "$", Status::Known));
    // The headwall is 12 LF at $40.00: $480.00. It lies on the shed, whose pitch is
    // Assumed, so its length is no surer than that, and neither is a dollar priced on it.
    let h = priced_of(&mut job, "headwall-0021", "allowance");
    assert_eq!((h.value, h.status), (Some(480.0), Status::Assumed));
    // An assumed input shows in the money: the shed's pitch is Assumed, so its 98.9545 SF
    // is, and so is its $2,770.73 (98.9545 × 28 = 2,770.726).
    let shed = priced_of(&mut job, "plane-0017", "allowance");
    assert_eq!((shed.value, shed.status), (Some(2770.73), Status::Assumed));

    // Detail. The south plane's one line is its 419.2627 SF at $31.50: $13,206.77505,
    // $13,206.78. The headwall's flashing is 12 LF × 24 in ÷ 12 = 24 SF at $18.25: $438.00.
    assert_eq!(extension_of(&mut job, "plane-0003", "Copper standing seam, installed").value, Some(13206.78));
    assert_eq!(extension_of(&mut job, "headwall-0021", "Copper headwall flashing").value, Some(438.0));
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(13206.78));

    // Rolled back up: over the same figure the allowance was priced on. The plane's detail
    // is $13,206.78 ÷ 419.2627 SF = $31.50 a SF. The headwall's flashing is priced by the
    // SF and rolls up to the LF its allowance is in: $438.00 ÷ 12 LF = $36.50 a LF.
    let rate = priced_of(&mut job, "plane-0003", "detail_rate");
    assert_eq!((rate.value, rate.unit.as_str()), (Some(31.5), "$/SF"));
    let rate = priced_of(&mut job, "headwall-0021", "detail_rate");
    assert_eq!((rate.value, rate.unit.as_str()), (Some(36.5), "$/LF"));

    // Reconciled. Plane: 13,206.78 − 11,739.36 = 1,467.42 over, 12.5 %. Headwall:
    // 438.00 − 480.00 = 42.00 under, −8.75 %.
    assert_eq!(priced_of(&mut job, "plane-0003", "difference").value, Some(1467.42));
    assert_eq!(priced_of(&mut job, "plane-0003", "difference_percent").value, Some(12.5));
    assert_eq!(priced_of(&mut job, "headwall-0021", "difference").value, Some(-42.0));
    assert_eq!(priced_of(&mut job, "headwall-0021", "difference_percent").value, Some(-8.75));

    // The shed has an allowance and a line with no price. Its detail is not whole: it has
    // no detail figure, no rate and no difference, and each says which line and why. It is
    // not rolled up as $0.
    for name in ["detail", "detail_rate", "difference", "difference_percent"] {
        let d = priced_of(&mut job, "plane-0017", name);
        assert_eq!((d.value, d.status), (None, Status::Unknown), "{name}");
        assert!(d.note.contains("\"Copper standing seam, installed\": the line has no price (no price entered)"), "{name}: {}", d.note);
    }

    // The job. Allowances: 11,739.36 + 2,770.73 + 480.00 = 14,990.09 over three elements,
    // of which the shed's 2,770.73 and the headwall's 480.00, 3,250.73, rest on the
    // assumed pitch.
    let p = plumbline_world::view::pricing(&mut job);
    assert_eq!((p.allowance.value, p.allowance.members, p.allowance.status, p.allowance.assumed), (Some(14990.09), 3, Status::Assumed, 3250.73));
    assert!(p.allowance.left_out.is_empty());
    // Detail: 13,206.78 + 438.00 = 13,644.78 over the two elements whose detail is whole.
    // Four are left out, each named with why: the shed and the three planes nobody priced.
    // The headwall's 438.00 rests on the assumed pitch.
    assert_eq!((p.detail.value, p.detail.members, p.detail.status, p.detail.assumed), (Some(13644.78), 2, Status::Assumed, 438.0));
    assert_eq!(p.detail.left_out.len(), 4);
    assert!(p.detail.left_out.iter().all(|(_, why)| why.contains("the detail is not whole")), "{:?}", p.detail.left_out);
    // Like for like, over the two elements that have both: allowances 11,739.36 + 480.00
    // = 12,219.36 against detail 13,644.78.
    // Each side carries its own status (the headwall's share of each is Assumed).
    assert_eq!(p.both, Some(((12219.36, Status::Assumed), (13644.78, Status::Assumed), Status::Assumed, 2)));
    // What could be priced and is not, by kind: it is said, not left out of sight.
    assert!(p.not_priced.contains(&(Kind::Eave, 5)), "{:?}", p.not_priced);
    assert!(p.not_priced.contains(&(Kind::Hip, 4)), "{:?}", p.not_priced);

    // The geometry still drives both. Make the south plane 9:12: its plan area is 375 SF and
    // its slope factor 1.25, so 468.75 SF. Allowance 468.75 × 28 = 13,125.00; detail
    // 468.75 × 31.50 = 14,765.625, $14,765.63; and the rate it rolls up to is still $31.50.
    job.set("plane-0003", "pitch", Some("9:12"), Status::Known, "test").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "plane-0003", "allowance").value, Some(13125.0));
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(14765.63));
    assert_eq!(priced_of(&mut job, "plane-0003", "detail_rate").value, Some(31.5));
    // And it saves and loads to the same bytes, with nothing stale.
    let text = file::save(&mut job);
    let (mut again, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(file::save(&mut again), text);
}

#[test]
fn money_is_never_silently_zero_and_a_flag_reaches_every_dollar_it_touches() {
    let mut job = sample();
    // No price anywhere: no element has a detail, no total has a figure, and none is $0.
    let p = plumbline_world::view::pricing(&mut job);
    assert_eq!((p.allowance.value, p.allowance.members), (None, 0));
    assert_eq!((p.detail.value, p.detail.members), (None, 0));
    assert_eq!(p.both, None);
    let e = extension_of(&mut job, "plane-0003", "Copper standing seam, installed");
    assert_eq!((e.value, e.status), (None, Status::Unknown));
    assert_eq!(e.note, "the line has no price (no price entered)");
    // A price of nothing is a price: $0.00 is said when somebody entered it.
    job.set("plane-0003", "unit_cost", Some("0"), Status::Known, "furnished by the owner").unwrap();
    job.derive();
    let e = extension_of(&mut job, "plane-0003", "Copper standing seam, installed");
    assert_eq!((e.value, e.status), (Some(0.0), Status::Known));
    // A price too small to make a cent of the whole line is refused, not written as $0.00:
    // 24 SF at $0.0001 is $0.0024.
    job.set("headwall-0021", "unit_cost", Some("0.0001"), Status::Known, "test").unwrap();
    job.derive();
    let e = extension_of(&mut job, "headwall-0021", "Copper headwall flashing");
    assert_eq!(e.value, None);
    assert!(e.note.contains("less than half a cent"), "{}", e.note);
    // On the south plane, whose geometry is all Known: an assumed price makes an assumed
    // extension, detail, rate and difference; an assumed allowance makes an assumed
    // difference though the detail is Known.
    job.set("plane-0003", "unit_cost", Some("31.50"), Status::Assumed, "last year's price").unwrap();
    job.set("plane-0003", "allowance", Some("28"), Status::Known, "made-up budget rate").unwrap();
    job.derive();
    for name in ["detail", "detail_rate", "difference", "difference_percent"] {
        assert_eq!(priced_of(&mut job, "plane-0003", name).status, Status::Assumed, "{name}");
    }
    assert_eq!(priced_of(&mut job, "plane-0003", "allowance").status, Status::Known);
    job.set("plane-0003", "unit_cost", Some("31.50"), Status::Known, "quote").unwrap();
    job.set("plane-0003", "allowance", Some("28"), Status::Assumed, "a guess").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").status, Status::Known);
    assert_eq!(priced_of(&mut job, "plane-0003", "difference").status, Status::Assumed);
    job.set("headwall-0021", "unit_cost", Some("18.25"), Status::Known, "quote").unwrap();
    // An allowance nobody has: no amount, and it says so. A stretch-out nobody has: the
    // line has no quantity, so no extension, and the element's detail is not whole.
    job.set("headwall-0021", "allowance", None, Status::Unknown, "not budgeted yet").unwrap();
    job.set("headwall-0021", "stretch_out", None, Status::Unknown, "detail not drawn").unwrap();
    job.derive();
    let a = priced_of(&mut job, "headwall-0021", "allowance");
    assert_eq!(a.value, None);
    assert!(a.note.contains("the allowance has no rate (not budgeted yet)"), "{}", a.note);
    let d = priced_of(&mut job, "headwall-0021", "detail");
    assert!(d.note.contains("the detail is not whole") && d.note.contains("the line has no quantity to price"), "{}", d.note);
    // An allowance of $0 with a detail: the difference is the detail, and no percentage
    // of nothing is made up.
    job.set("headwall-0021", "allowance", Some("0"), Status::Known, "included elsewhere").unwrap();
    job.set("headwall-0021", "stretch_out", Some("24 in"), Status::Known, "detail").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "headwall-0021", "difference").value, Some(438.0));
    let pct = priced_of(&mut job, "headwall-0021", "difference_percent");
    assert_eq!(pct.value, None);
    assert!(pct.note.contains("not a share of nothing"), "{}", pct.note);
}

#[test]
fn a_price_is_entered_for_the_unit_the_thing_is_priced_in_and_nothing_else() {
    use plumbline_world::units::parse_price;
    // The ways a price is typed.
    assert_eq!(parse_price("4.25", "$/SF"), Ok(4.25));
    assert_eq!(parse_price("$4.25", "$/SF"), Ok(4.25));
    assert_eq!(parse_price("$4.25/SF", "$/SF"), Ok(4.25));
    assert_eq!(parse_price("4.25 $/SF", "$/SF"), Ok(4.25));
    assert_eq!(parse_price("$1,250.00/sheet", "$/sheet"), Ok(1250.0));
    assert!(parse_price("$4.25/LF", "$/SF").unwrap_err().contains("a price does not carry from one unit to another"));
    assert!(parse_price("-4", "$/SF").is_err());
    assert!(parse_price("four", "$/SF").is_err());
    assert!(parse_price("", "$/SF").is_err());
    let mut job = sample();
    // A plane is priced by the SF and a headwall by the LF: the other is refused.
    assert!(job.set("plane-0003", "allowance", Some("$28/LF"), Status::Known, "test").is_err());
    assert!(job.set("headwall-0021", "allowance", Some("$40/SF"), Status::Known, "test").is_err());
    // A roof, a job and a condition have nothing a rate could be on.
    let err = job.set("roof-0002", "allowance", Some("5"), Status::Known, "test").unwrap_err();
    assert!(err.contains("cannot carry an allowance"), "{err}");
    // A value needs a source, as everywhere.
    assert!(job.set("plane-0003", "allowance", Some("28"), Status::Known, "").is_err());
    // A line's price is for what the line is priced in. The headwall's line is in SF.
    assert!(job.set("headwall-0021", "unit_cost", Some("$18.25/LF"), Status::Known, "test").is_err());
    job.set("headwall-0021", "unit_cost", Some("$18.25/SF"), Status::Known, "test").unwrap();
    // An allowance on a kind it cannot be on, or in the wrong unit, does not load.
    job.set("plane-0003", "allowance", Some("28"), Status::Known, "test").unwrap();
    job.derive();
    let text = file::save(&mut job);
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let plane = doc["data"].as_array_mut().unwrap().iter_mut().find(|e| e["path"] == "plane-0003").unwrap();
    assert_eq!(plane["attributes"]["pl::allowance"]["rate"]["unit"], "$/SF");
    plane["attributes"]["pl::allowance"]["rate"]["unit"] = "$/LF".into();
    let err = file::load(&doc.to_string()).err().expect("an allowance by the LF on a plane");
    assert!(err.contains("priced by its surface_area, in SF"), "{err}");
    // A price somebody typed into the saved figures is not trusted: it is worked out again.
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let plane = doc["data"].as_array_mut().unwrap().iter_mut().find(|e| e["path"] == "plane-0003").unwrap();
    plane["attributes"]["pl::priced"]["allowance"]["value"] = 1.0.into();
    let (mut job, report) = file::load(&doc.to_string()).unwrap();
    assert_eq!(report.stale, ["plane-0003"]);
    assert_eq!(priced_of(&mut job, "plane-0003", "allowance").value, Some(11739.36));
}

// ---------------------------------------------------------------- the ninth judgment

/// A 20 ft by 10 ft box of 6 in walls 10 ft tall, and a room in its west half.
fn boxed() -> (Job, String, String, String) {
    let (mut job, s, t) = walled();
    for (name, a, b) in [("south", [0.0, 0.0], [20.0, 0.0]), ("east", [20.0, 0.0], [20.0, 10.0]), ("north", [20.0, 10.0], [0.0, 10.0]), ("west", [0.0, 10.0], [0.0, 0.0])] {
        wall(&mut job, &s, &t, name, a, b, 10.0);
    }
    let r = room(&mut job, &s, "room", [5.0, 5.0]);
    (job, s, t, r)
}

#[test]
fn a_room_is_no_surer_than_a_height_that_decides_whether_a_wall_bounds_it() {
    use plumbline_world::Wall;
    // The box alone: 19.5 by 9.5 inside is 185.25 SF, 200 to centre lines, 58 LF round.
    let (mut job, s, t, r) = boxed();
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(185.25), Some(200.0), Some(58.0)]);
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Known);
    // A partition across the middle. With it the room is the west half: 9.5 by 9.5, 90.25.
    let p = wall(&mut job, &s, &t, "partition", [10.0, 0.0], [10.0, 10.0], 10.0);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(90.25), Some(100.0), Some(38.0)]);
    let set = |job: &mut Job, change: &dyn Fn(&mut Wall)| {
        let e = job.find(&p).unwrap();
        change(&mut job.world.get_mut::<Wall>(e).unwrap());
        job.derive();
    };
    // Say its base is assumed to be 5 ft up. That is above the 4 ft the rooms find their
    // walls at, so it does not bound the room, which is 185.25 SF: but only if the
    // assumption holds. The figure is Assumed, and says which wall it turns on.
    set(&mut job, &|w| w.base_offset = Some(Val::assumed(Len::One(5.0), "ft", "guessed")));
    for name in ["floor_area", "gross_area", "perimeter"] {
        let d = measure_of(&mut job, &r, name);
        assert_eq!(d.status, Status::Assumed, "{name}");
        // (Named by its path as well as its name since the tenth judgment: two walls can share a name.)
        assert_eq!(d.note, format!("whether partition ({p}) bounds this room rests on its heights, which are assumed"), "{name}");
        assert!(d.inputs.contains(&format!("{p}#pl::wall")), "{name}: {:?}", d.inputs);
    }
    assert_eq!(figures_of(&mut job, &r), [Some(185.25), Some(200.0), Some(58.0)]);
    // Assumed to be 1 ft up, it does bound the room: 90.25, and Assumed for the same reason.
    set(&mut job, &|w| w.base_offset = Some(Val::assumed(Len::One(1.0), "ft", "guessed")));
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status), (Some(90.25), Status::Assumed));
    // A height that is assumed does the same: 3 ft tall, it stops short of 4 ft.
    set(&mut job, &|w| {
        w.base_offset = None;
        w.height = Some(Val::assumed(Len::One(3.0), "ft", "guessed"));
    });
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status), (Some(185.25), Status::Assumed));
    // Known to be 3 ft tall, the same wall leaves the room 185.25 and Known.
    set(&mut job, &|w| w.height = Some(ft(3.0)));
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status, d.note.as_str()), (Some(185.25), Status::Known, ""));
    // A wall with an assumed height that could not bound this room whatever its height
    // (it stands outside the box) does not make the room Assumed.
    let far = wall(&mut job, &s, &t, "garden wall", [30.0, 0.0], [30.0, 10.0], 10.0);
    let e = job.find(&far).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().height = Some(Val::assumed(Len::One(6.0), "ft", "guessed"));
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Known);
    // A partition entered on a storey below, whose elevation is assumed: whether it rises
    // into this storey rests on that.
    let (mut job, _s, t, r) = boxed();
    let below = job.add("job-0001", Kind::Storey, "basement").unwrap();
    job.put(&below, plumbline_world::Level { elevation: Val::assumed(Len::One(-10.0), "ft", "scaled off a section"), height: ft(10.0), computation_height: None }).unwrap();
    wall(&mut job, &below, &t, "basement partition", [10.0, 0.0], [10.0, 10.0], 10.0);
    job.derive();
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status), (Some(185.25), Status::Assumed));
    assert!(d.note.contains("whether basement partition (wall-") && d.note.contains(") bounds this room"), "{}", d.note);
}

#[test]
fn a_wall_with_nothing_entered_and_a_point_in_a_wall_leave_no_figure_standing() {
    // A wall element with no line entered is a wall nobody can place. The rooms of its
    // storey have no figure, and say which wall.
    let (mut job, s, _t, r) = boxed();
    let bare = job.add(&s, Kind::Wall, "wall with nothing entered").unwrap();
    job.derive();
    for name in ["floor_area", "gross_area", "perimeter"] {
        let d = measure_of(&mut job, &r, name);
        assert_eq!((d.value, d.status), (None, Status::Unknown), "{name}");
        assert_eq!(d.note, "wall with nothing entered cannot be placed (it has no wall line entered), so what bounds the rooms of this storey is not known", "{name}");
    }
    // And such a file loads the same way: the wall is not dropped.
    let text = file::save(&mut job);
    let (mut again, _) = file::load(&text).unwrap();
    assert_eq!(measure_of(&mut again, &r, "floor_area").value, None);
    job.remove(&bare).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(185.25));
    // A point in the body of a wall is in no room, at the faces or at the centre lines.
    let in_wall = room(&mut job, &s, "in the wall", [0.1, 5.0]);
    job.derive();
    for name in ["floor_area", "gross_area", "perimeter"] {
        let d = measure_of(&mut job, &in_wall, name);
        assert_eq!((d.value, d.note.as_str()), (None, "its point is inside a wall"), "{name}");
    }
    // And it does not take the space from the room that is there.
    assert_eq!(measure_of(&mut job, &r, "gross_area").value, Some(200.0));
}

#[test]
fn a_partition_stops_at_the_face_of_a_wall_that_does_not_bound_rooms() {
    use plumbline_world::Wall;
    // The box, a wall N across the middle that does not bound rooms, and a partition P
    // from N's line east to the east wall, at y = 5. With N not bounding, the room is the
    // whole box less P's body. P stops at N's face (x = 10.25), as BIMCraft joins it, and
    // at the east wall's face (x = 19.75): 9.5 ft of a 6 in wall, 4.75 SF.
    // Floor: 185.25 − 4.75 = 180.5 SF. Round: 58, plus P's two faces (19) and its west end
    // (0.5), less the 0.5 of the east wall's face P covers: 77 LF. To centre lines: 200.
    let (mut job, s, t, _r) = boxed();
    let n = wall(&mut job, &s, &t, "N", [10.0, 0.0], [10.0, 10.0], 10.0);
    let e = job.find(&n).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().room_bounding = false;
    wall(&mut job, &s, &t, "P", [10.0, 5.0], [20.0, 5.0], 10.0);
    let east = room(&mut job, &s, "east side", [15.0, 7.5]);
    job.derive();
    // (The room placed first, at (5, 5), has the space: the east room is in it.)
    assert_eq!(measure_of(&mut job, &east, "floor_area").note, "room was placed first and has the space this room's point is in");
    let first = path_named(&mut job, "room");
    assert_eq!(figures_of(&mut job, &first), [Some(180.5), Some(200.0), Some(77.0)]);
    // The same with N only 3 ft tall and bounding rooms where it stands: it stops short of
    // the 4 ft the rooms are found at, so it bounds nothing, and P still stops at its face.
    job.world.get_mut::<Wall>(e).unwrap().room_bounding = true;
    job.world.get_mut::<Wall>(e).unwrap().height = Some(ft(3.0));
    job.derive();
    assert_eq!(figures_of(&mut job, &first), [Some(180.5), Some(200.0), Some(77.0)]);
}

#[test]
fn walls_join_within_a_hundredth_of_a_foot_and_no_further() {
    use plumbline_world::building::check_wall;
    use plumbline_world::Wall;
    // The west wall stops 0.005 ft short of the corner: within the 0.01 ft two points are
    // told apart by, so it is the corner, and the room closes at the centre lines: 200 SF.
    let (mut job, s, t) = walled();
    for (name, a, b) in [("south", [0.0, 0.0], [20.0, 0.0]), ("east", [20.0, 0.0], [20.0, 10.0]), ("north", [20.0, 10.0], [0.0, 10.0])] {
        wall(&mut job, &s, &t, name, a, b, 10.0);
    }
    let west = wall(&mut job, &s, &t, "west", [0.0, 10.0], [0.0, 0.005], 10.0);
    let r = room(&mut job, &s, "room", [5.0, 5.0]);
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "gross_area").value, Some(200.0));
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(185.25));
    // 0.02 ft short, it is not the corner. The centre lines do not meet, so there is no
    // area to centre lines, and it says so. The walls' bodies still overlap at the corner
    // (each is 3 in either side of its line), so the floor inside their faces is closed:
    // 185.25 SF.
    let e = job.find(&west).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().line = Val::known([[0.0, 10.0], [0.0, 0.02]], "ft", "test");
    job.derive();
    let gross = measure_of(&mut job, &r, "gross_area");
    assert_eq!((gross.value, gross.note.as_str()), (None, "no loop of walls closes round its point"));
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(185.25));
    // A wall no longer than that tolerance is not a wall.
    let short = Wall::tall(named("type-0003"), Val::known([[3.0, 3.0], [3.005, 3.0]], "ft", "test"), ft(8.0));
    assert!(check_wall("w", Kind::Wall, &short).unwrap_err().contains("a wall is longer than the 0.01 ft"));
    let long_enough = Wall::tall(named("type-0003"), Val::known([[3.0, 3.0], [3.02, 3.0]], "ft", "test"), ft(8.0));
    assert!(check_wall("w", Kind::Wall, &long_enough).is_ok());
    // In inches too: 0.06 in is 0.005 ft.
    let short = Wall::tall(named("type-0003"), Val::known([[36.0, 36.0], [36.06, 36.0]], "in", "test"), ft(8.0));
    assert!(check_wall("w", Kind::Wall, &short).is_err());
}

#[test]
fn an_openings_area_is_its_size_and_where_it_is_matters_to_the_net() {
    use plumbline_world::Opening;
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let window = job.children().get(&south).unwrap().get("west window").unwrap().clone();
    let e = job.find(&window).unwrap();
    let mut o = job.world.get::<Opening>(e).unwrap().clone();
    // A window whose sill nobody has. It is still 5 by 5: the wall's openings still come
    // to 70.01 SF, Known. Whether it fits cannot be checked, so the wall has no net area,
    // and the reason is the position, not the area.
    o.sill = Val::unknown("ft", "not on the elevation");
    job.put(&window, o.clone()).unwrap();
    job.derive();
    let open = measure_of(&mut job, &south, "opening_area");
    assert_eq!((open.value, open.status), (Some(70.01), Status::Known));
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!(net.value, None);
    assert_eq!(net.note, format!("where {window} is in the wall is not known, so whether it fits cannot be checked"));
    // A sill somebody assumed: the openings' area is still Known; the net is Assumed.
    o.sill = Val::assumed(Len::One(2.5), "ft", "typical");
    job.put(&window, o).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "opening_area").status, Status::Known);
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!((net.value, net.status), (Some(329.99), Status::Assumed));
}

#[test]
fn an_open_connector_is_listed_where_its_run_would_be() {
    use plumbline_world::building::takeoff;
    use plumbline_world::{ConnectorKind::Pipe, Domain, InSystem, SystemInfo};
    let (mut job, s, two, ..) = piping();
    let mut system = |job: &mut Job, name: &str, classification: &str| {
        let p = job.add("job-0001", Kind::System, name).unwrap();
        job.put(&p, SystemInfo { discipline: plumbline_world::Discipline::Plumbing, classification: classification.into() }).unwrap();
        p
    };
    let (cold, _drain) = (system(&mut job, "Cold water", "Domestic Cold Water"), system(&mut job, "Drain", "Sanitary"));
    // A water closet, on the cold water system as fixtures are. Its cold inlet is met by
    // a pipe. Nothing is on its waste outlet.
    let wc = thing(&mut job, &s, Kind::Fixture, "wc", [5.0, 5.0, 0.0], None, &[(Pipe, Some(Domain::Cold), [-0.5, 0.0, 1.0]), (Pipe, Some(Domain::Waste), [0.0, 1.0, 0.0])]);
    job.put(&wc, InSystem(named(&cold))).unwrap();
    let supply = piece(&mut job, &s, Kind::Pipe, "supply", &two, [4.5, 5.0, 1.0], [4.5, 0.0, 1.0]);
    job.put(&supply, InSystem(named(&cold))).unwrap();
    job.derive();
    let t = takeoff(&mut job);
    let of = |name: &str| t.iter().find(|x| x.name == name).unwrap().joined.open.iter().map(|o| o.said.clone()).collect::<Vec<_>>();
    // The drain is told its fixture is not connected; the cold water is told only of the
    // far end of its own pipe.
    assert_eq!(of("Drain"), ["wc: its connector 2 connector"]);
    assert_eq!(of("Cold water"), ["supply: its second end"]);
}

// ---- D5: the estimate sheet, the recap, materials and suppliers ----
//
// The job is `sample::priced()`: the sample roof with a made-up estimate on it. Every
// price, rate and supplier in it is made up. The figures it rests on, from the sample:
// south plane 419.2627 SF, east 251.5576 SF, west 251.5576 SF, shed 98.9545 SF (Assumed:
// its pitch is), headwall 12 LF (Assumed: it lies on the shed).
//
// Worked by hand, each step from the figure the step before shows (four places):
//   south  panels     419.2627 x 1.10 = 461.1890 SF x 1 lb/SF = 461.1890 lb x 9.00  = 4,150.70
//          underlay   419.2627 x 0.90                                               =   377.34
//          fabricate  419.2627 / 100 = 4.1926 hr x 60                               =   251.56
//          install    419.2627 / 25  = 16.7705 hr x 72                              = 1,207.48
//   east   panels     251.5576 x 1.10 = 276.7134 lb x 9.00                          = 2,490.42
//          underlay   251.5576 x 0.90                                               =   226.40
//          fabricate  251.5576 / 100 = 2.5156 hr x 60                               =   150.94
//          install    251.5576 / 25  = 10.0623 hr x 72                              =   724.49
//   west   by a sub   251.5576 x 41.00                                              = 10,313.86
//   shed   panels     98.9545 x 1.25 lb/SF = 123.6931 lb x 9.00                     = 1,113.24 (Assumed)
//   headwall flashing 12 x 24 in = 24 SF x 1.25 = 30 lb x 9.00                      =   270.00 (Assumed)
//          install    12 x 0.25 hr/LF = 3 hr x 72                                   =   216.00 (Assumed)
//   job    foreman    5 x 8 hr = 40 hr x 95                                         = 3,800.00
//   north  one line, no price, no class: in no row.
//
//   Material    4,150.70 + 377.34 + 2,490.42 + 226.40 + 1,113.24 + 270.00 =  8,628.10
//   Labor       1,207.48 + 724.49 + 216.00                                =  2,147.97   29.8328 hr
//   Fabrication 251.56 + 150.94                                           =    402.50    6.7082 hr
//   Supervision                                                           =  3,800.00   40 hr
//   Subcontract                                                           = 10,313.86
//   Job cost                                                              = 25,292.43

fn class_total(est: &view::Estimate, class: Class) -> &view::ClassTotal {
    est.classes.iter().find(|c| c.class == class).unwrap()
}

#[test]
fn the_estimate_puts_every_line_in_one_class_and_adds_up_what_is_shown() {
    let mut job = sample::priced();
    let est = view::estimate(&mut job);
    let total = |c: Class| class_total(&est, c).total.clone();
    assert_eq!((total(Class::Material).value, total(Class::Material).status, total(Class::Material).members), (Some(8628.10), Status::Assumed, 6));
    assert_eq!(total(Class::Material).assumed, 1383.24);
    assert_eq!((total(Class::Labor).value, total(Class::Labor).status), (Some(2147.97), Status::Assumed));
    assert_eq!(class_total(&est, Class::Labor).hours, Some((29.8328, Status::Assumed)));
    assert_eq!((total(Class::Fabrication).value, total(Class::Fabrication).status), (Some(402.50), Status::Known));
    assert_eq!(class_total(&est, Class::Fabrication).hours, Some((6.7082, Status::Known)));
    assert_eq!(total(Class::Supervision).value, Some(3800.00));
    assert_eq!(class_total(&est, Class::Supervision).hours, Some((40.0, Status::Known)));
    assert_eq!(total(Class::Subcontract).value, Some(10313.86));
    assert_eq!(class_total(&est, Class::Subcontract).hours, None);
    // Nothing is in general conditions, and that is not $0.00.
    assert_eq!(total(Class::General).value, None);
    // The three columns: material; labor with fabrication and supervision; the rest.
    let column = |c: Column| est.columns.iter().find(|x| x.0 == c).unwrap().1.value;
    assert_eq!(column(Column::Material), Some(8628.10));
    assert_eq!(column(Column::Labor), Some(6350.47)); // 2,147.97 + 402.50 + 3,800.00
    assert_eq!(column(Column::General), Some(10313.86));
    // The job cost is the classes added up, and it names the line that is in none.
    assert_eq!((est.job_cost.value, est.job_cost.status, est.job_cost.members), (Some(25292.43), Status::Assumed, 13));
    assert_eq!(est.job_cost.left_out.len(), 1);
    assert!(est.job_cost.left_out[0].0.starts_with("plane-0005") && est.job_cost.left_out[0].1.contains("no class"), "{:?}", est.job_cost.left_out);
    assert_eq!(est.not_classed, [("plane-0005".to_string(), "Copper standing seam, installed".to_string())]);
    // Every line that is on, classed and has a figure is in exactly one class.
    let in_classes: usize = est.classes.iter().map(|c| c.total.members).sum();
    assert_eq!(in_classes, est.lines.iter().filter(|l| !l.off && l.class.is_some() && l.amount.value.is_some()).count());
    // Overhead and profit are not entered: there is no price, and the recap says why.
    assert_eq!(est.price.value, None);
    assert!(est.price.note.contains("overhead has not been entered"), "{}", est.price.note);
    // It saves, and loads to the same bytes.
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut again), text);
}

#[test]
fn overhead_and_profit_are_entered_and_neither_has_a_default() {
    let mut job = sample::priced();
    let known = Status::Known;
    // Overhead alone: 25,292.43 x 10 % = 2,529.243 -> 2,529.24. Still no price: no profit.
    job.set("job-0001", "overhead", Some("10"), known, "made-up markup").unwrap();
    let est = view::estimate(&mut job);
    assert_eq!(est.overhead.value, Some(2529.24));
    assert_eq!(est.profit.value, None);
    assert!(est.price.note.contains("profit is not known (not entered)"), "{}", est.price.note);
    // Profit is on the job cost and the overhead together:
    // (25,292.43 + 2,529.24) x 8 % = 27,821.67 x 0.08 = 2,225.7336 -> 2,225.73.
    job.set("job-0001", "profit", Some("8 %"), known, "made-up markup").unwrap();
    let est = view::estimate(&mut job);
    assert_eq!(est.profit.value, Some(2225.73));
    // The price is the three figures as shown: 25,292.43 + 2,529.24 + 2,225.73 = 30,047.40.
    // It rests on the shed's assumed pitch, so it is Assumed.
    assert_eq!((est.price.value, est.price.status), (Some(30047.40), Status::Assumed));
    // Changed: 12 % is 3,035.0916 -> 3,035.09; profit 28,327.52 x 0.08 = 2,266.2016 -> 2,266.20.
    job.set("job-0001", "overhead", Some("12"), known, "made-up markup").unwrap();
    let est = view::estimate(&mut job);
    assert_eq!((est.overhead.value, est.profit.value, est.price.value), (Some(3035.09), Some(2266.20), Some(30593.72)));
    // Nought entered is nought, and a price: it is an answer. Unknown is not.
    job.set("job-0001", "overhead", Some("0"), known, "made-up markup").unwrap();
    let est = view::estimate(&mut job);
    // Profit 25,292.43 x 0.08 = 2,023.3944 -> 2,023.39; price 27,315.82.
    assert_eq!((est.overhead.value, est.profit.value, est.price.value), (Some(0.0), Some(2023.39), Some(27315.82)));
    job.set("job-0001", "overhead", None, Status::Unknown, "not settled with the owner").unwrap();
    let est = view::estimate(&mut job);
    assert_eq!((est.overhead.value, est.profit.value, est.price.value), (None, None, None));
    assert!(est.profit.note.contains("the overhead has no figure"), "{}", est.profit.note);
    // An assumed percentage makes an assumed price even on a job that is all Known.
    // Refused: not a number; below nought; on something that is not the job.
    assert!(job.set("job-0001", "overhead", Some("ten"), known, "x").is_err());
    assert!(job.set("job-0001", "profit", Some("-5"), known, "x").unwrap_err().contains("percentage from 0"));
    assert!(job.set("plane-0003", "profit", Some("5"), known, "x").unwrap_err().contains("the job's"));
}

#[test]
fn a_labor_line_is_switched_off_and_leaves_every_total_by_its_amount() {
    let mut job = sample::priced();
    // The south plane's detail: 4,150.70 + 377.34 + 251.56 + 1,207.48 = 5,987.08.
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(5987.08));
    job.set_off("plane-0003", Some("Install panels"), true).unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    // Labor: 2,147.97 - 1,207.48 = 940.49. Hours: 29.8328 - 16.7705 = 13.0623.
    assert_eq!(class_total(&est, Class::Labor).total.value, Some(940.49));
    assert_eq!(class_total(&est, Class::Labor).hours, Some((13.0623, Status::Assumed)));
    assert_eq!(est.job_cost.value, Some(24084.95)); // 25,292.43 - 1,207.48
    assert_eq!(est.job_cost.left_out.len(), 1, "a line switched off is not a line left out");
    // The element's detail leaves it too: 5,987.08 - 1,207.48 = 4,779.60.
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(4779.60));
    // It is still on the sheet, marked off, with what it would come to.
    let line = est.lines.iter().find(|l| l.path == "plane-0003" && l.description == "Install panels").unwrap();
    assert!(line.off);
    assert_eq!(line.amount.value, Some(1207.48));
    // The file says it is off, and loads so.
    let text = save(&mut job);
    assert!(text.contains("\"off\": true"));
    let (mut again, _) = load(&text).unwrap();
    assert_eq!(view::estimate(&mut again).job_cost.value, Some(24084.95));
    // On again: back to the cent.
    job.set_off("plane-0003", Some("Install panels"), false).unwrap();
    job.derive();
    assert_eq!(view::estimate(&mut job).job_cost.value, Some(25292.43));
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(5987.08));
    assert!(!save(&mut job).contains("\"off\""));
    // Only a Labor line has the switch.
    for line in ["Standing seam panels", "Underlayment", "Fabricate panels"] {
        assert!(job.set_off("plane-0003", Some(line), true).unwrap_err().contains("only a Labor line"), "{line}");
    }
    assert!(job.set_off("plane-0006", None, true).unwrap_err().contains("only a Labor line"));
}

#[test]
fn in_house_work_is_a_production_rate_and_a_subcontract_is_a_price() {
    let mut job = sample::priced();
    // 419.2627 SF at 25 SF to the hour is 16.7705 hours, not rounded up to 17.
    let f = figures(&mut job, "plane-0003", "Install panels");
    assert_eq!((f.order.as_ref().unwrap().value, f.order.as_ref().unwrap().unit.as_str()), (Some(16.7705), "hr"));
    assert_eq!(f.extension.unwrap().value, Some(1207.48));
    // Stated the other way up: 12 LF at a quarter of an hour each is 3 hours.
    assert_eq!(figures(&mut job, "headwall-0021", "Install headwall flashing").order.unwrap().value, Some(3.0));
    // A rate typed on the command line is not rounded up to whole hours either:
    // 419.2627 / 40 = 10.4816 hr.
    job.set_on_line("plane-0003", "order", Some("Install panels"), Some("40 SF/hr"), Status::Known, "made-up rate").unwrap();
    job.derive();
    assert_eq!(figures(&mut job, "plane-0003", "Install panels").order.unwrap().value, Some(10.4816));
    // A subcontract line is refused a production rate, and in-house work is refused
    // without one.
    assert!(job.set_class("plane-0003", Some("Install panels"), Class::Subcontract).unwrap_err().contains("Subcontract and carries a production rate"));
    assert!(job.set_class("plane-0003", Some("Install panels"), Class::Material).unwrap_err().contains("is in hours"));
    assert!(job.set_class("plane-0006", None, Class::Labor).unwrap_err().contains("is not in hours"));
    assert!(job.set_class("plane-0003", Some("Underlayment"), Class::Fabrication).unwrap_err().contains("is not in hours"));
    // Fabrication, labor and supervision are each a class of in-house work, and a line
    // in hours can be moved between them.
    job.set_class("plane-0003", Some("Install panels"), Class::Supervision).unwrap();
    job.set_class("plane-0003", Some("Install panels"), Class::Labor).unwrap();
    // A class is typed in a few plain ways and nothing else.
    assert_eq!(Class::parse("Sub"), Ok(Class::Subcontract));
    assert_eq!(Class::parse(" general "), Ok(Class::General));
    assert!(Class::parse("overhead").is_err());
}

#[test]
fn changing_what_an_element_is_made_of_reprices_the_lines_made_of_it_and_no_others() {
    let mut job = sample::priced();
    let before = view::estimate(&mut job);
    // South plane, 16 oz to 20 oz copper: 461.1890 SF x 1.25 lb/SF = 576.48625 -> 576.4863 lb
    // (it was 461.1890 lb), x 9.00 = 5,188.3767 -> 5,188.38 (it was 4,150.70).
    job.set("plane-0003", "material", Some("Copper, 20 oz"), Status::Known, "made-up revision").unwrap();
    job.derive();
    let f = figures(&mut job, "plane-0003", "Standing seam panels");
    assert_eq!(f.order.as_ref().unwrap().value, Some(576.4863));
    assert_eq!(f.extension.as_ref().unwrap().value, Some(5188.38));
    // Its quotes are the new material's: Harbor Metals 576.4863 x 8.90 = 5,130.72807 -> 5,130.73;
    // Tinners Yard has not quoted 20 oz.
    assert_eq!(f.quotes["Harbor Metals"].value, Some(5130.73));
    assert!(!f.quotes.contains_key("Tinners Yard"));
    // The lines not made of it have not moved.
    for (line, amount) in [("Underlayment", 377.34), ("Fabricate panels", 251.56), ("Install panels", 1207.48)] {
        assert_eq!(extension_of(&mut job, "plane-0003", line).value, Some(amount), "{line}");
    }
    // The recap moved by the difference and nothing else: 5,188.38 - 4,150.70 = 1,037.68.
    let after = view::estimate(&mut job);
    assert_eq!(class_total(&after, Class::Material).total.value, Some(9665.78)); // 8,628.10 + 1,037.68
    assert_eq!(after.job_cost.value, Some(26330.11)); // 25,292.43 + 1,037.68
    for class in [Class::Labor, Class::Fabrication, Class::Supervision, Class::Subcontract] {
        assert_eq!(class_total(&after, class).total, class_total(&before, class).total, "{class:?}");
    }
    // An assumed material makes assumed lines.
    job.set("plane-0003", "material", Some("Copper, 20 oz"), Status::Assumed, "not confirmed with the architect").unwrap();
    job.derive();
    assert_eq!(extension_of(&mut job, "plane-0003", "Standing seam panels").status, Status::Assumed);
    assert_eq!(extension_of(&mut job, "plane-0003", "Underlayment").status, Status::Known);
    // A material the job does not have: the line has no figure and says why; it is not
    // priced at the old material, and not at nothing.
    job.set("plane-0003", "material", Some("Zinc, 0.7 mm"), Status::Known, "made-up revision").unwrap();
    job.derive();
    let d = extension_of(&mut job, "plane-0003", "Standing seam panels");
    assert_eq!(d.value, None);
    assert!(d.note.contains("the job has no material of that name"), "{}", d.note);
    let est = view::estimate(&mut job);
    assert!(est.job_cost.left_out.iter().any(|(who, why)| who.starts_with("plane-0003") && why.contains("no material of that name")), "{:?}", est.job_cost.left_out);
    // Not known what it is made of: the same, with that reason.
    job.set("plane-0003", "material", None, Status::Unknown, "finish schedule not issued").unwrap();
    job.derive();
    assert!(extension_of(&mut job, "plane-0003", "Standing seam panels").note.contains("is not known (finish schedule not issued)"));
    // Changing a material's price reprices every element made of it. East plane, 16 oz at
    // 9.50: 276.7134 x 9.50 = 2,628.7773 -> 2,628.78.
    let copper = job.index().into_iter().map(|(p, _)| p).find(|p| p.starts_with("material-") && job.find(p).and_then(|e| job.world.get::<Node>(e).map(|n| n.name == "Copper, 16 oz")).unwrap_or(false)).unwrap();
    job.set(&copper, "price", Some("$9.50/lb"), Status::Known, "made-up price").unwrap();
    job.derive();
    assert_eq!(extension_of(&mut job, "plane-0004", "Standing seam panels").value, Some(2628.78));
    // A material's price is for the unit it is bought by.
    assert!(job.set(&copper, "price", Some("$9.50/SF"), Status::Known, "x").unwrap_err().contains("does not carry from one unit to another"));
    // A line made of the element's material carries no price or order of its own.
    assert!(job.set_on_line("plane-0004", "unit_cost", Some("Standing seam panels"), Some("9"), Status::Known, "x").unwrap_err().contains("its price is the material's"));
    assert!(job.set_on_line("plane-0004", "order", Some("Standing seam panels"), Some("1 lb/SF"), Status::Known, "x").unwrap_err().contains("is the material's"));
    // A thing that is not made of a material cannot be given one.
    assert!(job.set("job-0001", "material", Some("Copper, 16 oz"), Status::Known, "x").unwrap_err().contains("is not made of a material"));
}

#[test]
fn suppliers_stand_side_by_side_and_a_total_names_what_it_leaves_out() {
    let mut job = sample::priced();
    let est = view::estimate(&mut job);
    assert_eq!(est.suppliers, ["Harbor Metals", "Ridge Supply", "Tinners Yard"]);
    // Generic, over the four lines made of a material: 4,150.70 + 2,490.42 + 1,113.24 + 270.00.
    assert_eq!((est.at_generic.value, est.at_generic.members), (Some(8024.36), 4));
    let at = |s: &str| est.at_supplier.iter().find(|x| x.0 == s).unwrap().1.clone();
    // Harbor Metals: 461.1890 x 8.80 = 4,058.46; 276.7134 x 8.80 = 2,435.08;
    // 123.6931 x 8.90 = 1,100.87; 30 x 8.90 = 267.00. Total 7,861.41.
    assert_eq!((at("Harbor Metals").value, at("Harbor Metals").members), (Some(7861.41), 4));
    // Ridge Supply: 4,266.00 + 2,559.60 + 1,150.35 + 279.00 = 8,254.95.
    assert_eq!(at("Ridge Supply").value, Some(8254.95));
    // Tinners Yard quoted 16 oz only: 461.1890 x 9.10 = 4,196.82; 276.7134 x 9.10 = 2,518.09.
    // Its total is over two lines, and it names the two it leaves out.
    let tinners = at("Tinners Yard");
    assert_eq!((tinners.value, tinners.members, tinners.left_out.len()), (Some(6714.91), 2, 2));
    assert!(tinners.left_out.iter().all(|(_, why)| why.contains("has not quoted \"Copper, 20 oz\"")), "{:?}", tinners.left_out);
    // A quote is entered for the unit the material is bought by, and no other.
    let m20 = job.index().into_iter().map(|(p, _)| p).find(|p| job.find(p).and_then(|e| job.world.get::<Node>(e).map(|n| n.name == "Copper, 20 oz")).unwrap_or(false)).unwrap();
    assert!(job.set_quote(&m20, "Tinners Yard", Some("$9.40/SF"), Status::Known, "x").unwrap_err().contains("does not carry from one unit to another"));
    assert!(job.set_quote("plane-0003", "Tinners Yard", Some("9.40"), Status::Known, "x").unwrap_err().contains("not a material"));
    // Entered: 123.6931 x 9.40 = 1,162.71514 -> 1,162.72; 30 x 9.40 = 282.00.
    // Tinners Yard is now over all four: 6,714.91 + 1,162.72 + 282.00 = 8,159.63.
    job.set_quote(&m20, "Tinners Yard", Some("$9.40/lb"), Status::Assumed, "made-up quote, by telephone").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    let tinners = est.at_supplier.iter().find(|x| x.0 == "Tinners Yard").unwrap().1.clone();
    assert_eq!((tinners.value, tinners.members, tinners.left_out.len()), (Some(8159.63), 4, 0));
    // A quote asked for and not come is a quote with no figure, named: not a price of nothing.
    job.set_quote(&m20, "Tinners Yard", None, Status::Unknown, "asked for, not received").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    let tinners = est.at_supplier.iter().find(|x| x.0 == "Tinners Yard").unwrap().1.clone();
    assert_eq!((tinners.value, tinners.members), (Some(6714.91), 2));
    assert!(tinners.left_out.iter().all(|(_, why)| why.contains("has no price (asked for, not received)")), "{:?}", tinners.left_out);
}

#[test]
fn a_line_typed_by_hand_onto_the_job_lands_in_general_conditions() {
    let mut job = sample::priced();
    job.add_line("job-0001", "Dumpster", "2", Some(Class::General), false).unwrap();
    job.derive();
    // Typed and not yet priced: it has no figure and is named. General conditions are not $0.00.
    let est = view::estimate(&mut job);
    assert_eq!(class_total(&est, Class::General).total.value, None);
    assert!(est.job_cost.left_out.iter().any(|(who, why)| who.contains("Dumpster") && why.contains("has no price")), "{:?}", est.job_cost.left_out);
    // 2 x 650.00 = 1,300.00; a permit, 1 x 1,850.00.
    job.set_on_line("job-0001", "unit_cost", Some("Dumpster"), Some("650"), Status::Known, "made-up price").unwrap();
    job.add_line("job-0001", "Permit", "1", Some(Class::General), false).unwrap();
    job.set_on_line("job-0001", "unit_cost", Some("Permit"), Some("$1,850"), Status::Known, "made-up price").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!((class_total(&est, Class::General).total.value, class_total(&est, Class::General).total.members), (Some(3150.00), 2));
    assert_eq!(est.job_cost.value, Some(28442.43)); // 25,292.43 + 3,150.00
    // The general column is the subcontract and the general conditions: 10,313.86 + 3,150.00.
    assert_eq!(est.columns.iter().find(|x| x.0 == Column::General).unwrap().1.value, Some(13463.86));
    // A line with no class can be typed; it is named and is in no row until it has one.
    job.add_line("job-0001", "Mobilization", "1", None, false).unwrap();
    job.set_on_line("job-0001", "unit_cost", Some("Mobilization"), Some("500"), Status::Known, "made-up price").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!(est.job_cost.value, Some(28442.43));
    assert!(est.not_classed.contains(&("job-0001".to_string(), "Mobilization".to_string())));
    job.set_class("job-0001", Some("Mobilization"), Class::General).unwrap();
    job.derive();
    assert_eq!(view::estimate(&mut job).job_cost.value, Some(28942.43));
    // Two lines on one element are not described the same.
    assert!(job.add_line("job-0001", "dumpster", "1", Some(Class::General), false).is_err());
    // A line made of the element's material is Material, and on an element that does not
    // say what it is made of it has no figure and says so.
    assert!(job.add_line("plane-0006", "Trim", "surface_area", Some(Class::General), true).unwrap_err().contains("its class is Material"));
    job.add_line("plane-0006", "Trim", "surface_area", None, true).unwrap();
    job.derive();
    assert!(extension_of(&mut job, "plane-0006", "Trim").note.contains("does not say what it is made of"));
}

// ---- The tenth judgment: each finding, as a test that fails without its fix ----

#[test]
fn j10_1_an_assumed_or_unknown_condition_shows_in_what_its_lines_come_to() {
    let mut job = sample();
    job.set("plane-0003", "unit_cost", Some("31.50"), Status::Known, "made-up quote").unwrap();
    job.set("plane-0003", "allowance", Some("28"), Status::Known, "made-up budget rate").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").status, Status::Known);
    // Only assumed to be that condition: everything its lines come to is assumed.
    job.set("plane-0003", "condition", Some("Standing seam copper, 16 oz"), Status::Assumed, "not on the finish schedule yet").unwrap();
    job.derive();
    for name in ["detail", "detail_rate", "difference", "difference_percent"] {
        let d = priced_of(&mut job, "plane-0003", name);
        assert_eq!((d.value.is_some(), d.status), (true, Status::Assumed), "{name}");
    }
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(13206.78));
    // The allowance is a rate on its area and does not rest on the condition.
    assert_eq!(priced_of(&mut job, "plane-0003", "allowance").status, Status::Known);
    let p = view::pricing(&mut job);
    assert_eq!((p.detail.status, p.detail.assumed), (Status::Assumed, 13206.78));
    // Not known: its lines come to no figure, with the reason; nothing is priced as if it were.
    job.set("plane-0003", "condition", None, Status::Unknown, "nobody knows").unwrap();
    job.derive();
    let d = priced_of(&mut job, "plane-0003", "detail");
    assert_eq!(d.value, None);
    assert!(d.note.contains("the element's condition is not known (nobody knows)"), "{}", d.note);
}

#[test]
fn j10_3_a_line_whose_quantity_is_nothing_says_so() {
    let mut job = sample();
    job.set("plane-0003", "unit_cost", Some("31.50"), Status::Known, "made-up quote").unwrap();
    job.set_formula("plane-0003", None, "surface_area * 0", &[], Status::Known, "made-up").unwrap();
    job.derive();
    let d = extension_of(&mut job, "plane-0003", "Copper standing seam, installed");
    assert_eq!(d.value, Some(0.0));
    assert_eq!(d.note, "the line's quantity is 0, so it comes to nothing");
    let detail = priced_of(&mut job, "plane-0003", "detail");
    assert_eq!(detail.value, Some(0.0));
    assert!(detail.note.contains("1 of its 1 lines comes to nothing because the quantity is 0"), "{}", detail.note);
    // A price of $0 that somebody entered is an answer, and carries no note.
    let mut job = sample();
    job.set("plane-0003", "unit_cost", Some("0"), Status::Known, "included in the panel price").unwrap();
    job.derive();
    let d = extension_of(&mut job, "plane-0003", "Copper standing seam, installed");
    assert_eq!((d.value, d.note.as_str()), (Some(0.0), ""));
}

#[test]
fn j10_5_a_comma_in_a_price_is_a_thousands_mark_or_it_is_refused() {
    use plumbline_world::units::parse_price;
    assert_eq!(parse_price("1,250", "$/SF"), Ok(1250.0));
    assert_eq!(parse_price("$12,500.75/SF", "$/SF"), Ok(12500.75));
    assert_eq!(parse_price("1,234,567", "$/SF"), Ok(1234567.0));
    for wrong in ["1,25", ",5", "12,34.5", "1,2345", "1234,567", "1,,250", "1,250,", "1.250,5", "1,25.0"] {
        let why = parse_price(wrong, "$/SF").unwrap_err();
        assert!(why.contains("comma") || why.contains("not a price"), "{wrong}: {why}");
    }
}

#[test]
fn j10_6_a_rate_under_a_cent_is_not_written_as_nothing() {
    assert_eq!(view::rate(28.0), "28.00");
    assert_eq!(view::rate(31.5), "31.50");
    assert_eq!(view::rate(0.004), "0.004");
    assert_eq!(view::rate(1.015), "1.015");
    assert_eq!(view::rate(1250.0), "1,250.00");
    assert_eq!(view::rate(0.00305), "0.00305");
    let mut job = sample();
    job.set("plane-0004", "allowance", Some("0.004"), Status::Known, "made-up").unwrap();
    job.derive();
    let rows = view::elements(&mut job);
    let row = rows.iter().find(|e| e.path == "plane-0004").unwrap().rows.iter().find(|r| r.label == "allowance").unwrap().clone();
    assert!(row.text.starts_with("0.004"), "{}", row.text);
}

#[test]
fn j10_7_money_is_rounded_to_the_cent_as_it_would_be_on_paper() {
    use plumbline_world::lines::cents;
    // 1.015 is held as 1.01499999999999990230...; on paper it is half a cent and goes up.
    assert_eq!(cents(1.015), 1.02);
    assert_eq!(cents(4.0 * 0.25375), 1.02);
    assert_eq!(cents(2.675), 2.68);
    assert_eq!(cents(1.0149), 1.01);
    assert_eq!(cents(1.014999), 1.01);
    assert_eq!(cents(-1.015), -1.02);
    assert_eq!(cents(0.0), 0.0);
    let mut job = sample();
    job.set_formula("plane-0004", None, "n", &[("n".to_string(), "1".to_string())], Status::Known, "made-up").unwrap();
    job.set("plane-0004", "unit_cost", Some("1.015"), Status::Known, "made-up").unwrap();
    job.derive();
    assert_eq!(extension_of(&mut job, "plane-0004", "Copper standing seam, installed").value, Some(1.02));
}

#[test]
fn j10_8_what_is_bought_by_the_each_is_counted_when_it_is_not_priced() {
    let mut job = sample::house();
    let p = view::pricing(&mut job);
    let count = |k: Kind| p.not_priced.iter().find(|x| x.0 == k).map(|x| x.1).unwrap_or(0);
    // The sample house has nine doors and seventeen windows, and none has a price.
    assert_eq!((count(Kind::Door), count(Kind::Window)), (9, 17));
    assert!(count(Kind::Fitting) > 0 && count(Kind::Fixture) > 0 && count(Kind::Equipment) > 0);
    // What is not a thing to price is not counted: a storey, a system, a type.
    assert_eq!((count(Kind::Storey), count(Kind::System), count(Kind::Type)), (0, 0, 0));
}

#[test]
fn j10_11_money_past_what_holds_to_the_cent_is_refused() {
    let d = plumbline_world::lines::dollars(Some(999_999_999_989_999_861_760.0), &[Status::Known], "", &[]);
    assert_eq!(d.value, None);
    assert!(d.note.contains("more than can be held to the cent"));
    assert_eq!(plumbline_world::lines::dollars(Some(9.9e12), &[Status::Known], "", &[]).value, Some(9.9e12));
}

#[test]
fn j10_2_a_room_is_no_surer_than_a_wall_its_walls_are_cut_to() {
    use plumbline_world::{Wall, WallType};
    // The box of J9-4: N across the middle, not bounding rooms, and a partition P from N's
    // line east to the east wall. P is cut to N's face, so the room rests on N's thickness
    // and line though N bounds nothing.
    let (mut job, s, t, r) = boxed();
    let thick = job.add("job-0001", Kind::Type, "N's type").unwrap();
    job.put(&thick, WallType { thickness: Val::known(Len::One(6.0), "in", "test"), layers: Vec::new() }).unwrap();
    let n = wall(&mut job, &s, &thick, "N", [10.0, 0.0], [10.0, 10.0], 10.0);
    let e = job.find(&n).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().room_bounding = false;
    wall(&mut job, &s, &t, "P", [10.0, 5.0], [20.0, 5.0], 10.0);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(180.5), Some(200.0), Some(77.0)]);
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Known);
    // N's thickness assumed, 12 in: P stops at x = 10.5, so it is 9.25 ft of a 6 in wall,
    // 4.625 SF. Floor 185.25 − 4.625 = 180.625; round 58 + 18.5 + 0.5 − 0.5 = 76.5.
    let te = job.find(&thick).unwrap();
    job.world.get_mut::<WallType>(te).unwrap().thickness = Val::assumed(Len::One(12.0), "in", "not on the wall schedule");
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(180.625), Some(200.0), Some(76.5)]);
    for name in ["floor_area", "perimeter"] {
        let d = measure_of(&mut job, &r, name);
        assert_eq!(d.status, Status::Assumed, "{name}");
        assert!(d.note.contains(&format!("is cut to N ({n}), whose line or thickness is assumed")), "{name}: {}", d.note);
        assert!(d.inputs.contains(&format!("{n}#pl::wall")), "{name}: {:?}", d.inputs);
    }
    // N's line assumed instead, at x = 11: P is cut at x = 11.25, 8.5 ft of it, 4.25 SF.
    job.world.get_mut::<WallType>(te).unwrap().thickness = Val::known(Len::One(6.0), "in", "test");
    job.world.get_mut::<Wall>(e).unwrap().line = Val::assumed([[11.0, 0.0], [11.0, 10.0]], "ft", "scaled off a sketch");
    let p = path_named(&mut job, "P");
    let pe = job.find(&p).unwrap();
    job.world.get_mut::<Wall>(pe).unwrap().line = Val::known([[11.0, 5.0], [20.0, 5.0]], "ft", "test");
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(181.0)); // 185.25 − 4.25
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Assumed);
}

#[test]
fn j10_10_a_wall_that_starts_above_a_room_does_not_flag_it_whatever_its_top() {
    use plumbline_world::{Level, Wall};
    // The box on the ground, and a storey above at 10 ft with a wall across the middle of
    // the plan whose height is assumed. Its base is Known at 10 ft: however tall it turns
    // out, it does not come down to the 4 ft the ground-floor room is found at.
    let (mut job, _s, t, r) = boxed();
    let upper = job.add("job-0001", Kind::Storey, "upper").unwrap();
    job.put(&upper, Level { elevation: ft(10.0), height: ft(9.0), computation_height: None }).unwrap();
    let above = wall(&mut job, &upper, &t, "above", [10.0, 0.0], [10.0, 10.0], 8.0);
    let e = job.find(&above).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().height = Some(Val::assumed(Len::One(8.0), "ft", "not dimensioned"));
    job.derive();
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status, d.note.as_str()), (Some(185.25), Status::Known, ""));
    // If where it starts is itself unsure, it might come down, and the room says so,
    // naming the wall by its path as well as its name.
    job.world.get_mut::<Wall>(e).unwrap().base_offset = Some(Val::assumed(Len::One(0.0), "ft", "not dimensioned"));
    job.derive();
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!(d.status, Status::Assumed);
    assert!(d.note.contains(&format!("whether above ({above}) bounds this room rests on its heights")), "{}", d.note);
}
