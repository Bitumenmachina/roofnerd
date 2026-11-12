//! The command line as a person meets it.

use std::process::Command;

#[cfg(unix)]
#[test]
fn an_argument_that_is_not_text_is_refused_not_a_panic() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let dir = std::env::temp_dir().join(format!("plumbline-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let job = dir.join("job.json");
    assert!(Command::new(env!("CARGO_BIN_EXE_plumbline")).arg("sample").arg(&job).output().unwrap().status.success());
    let before = std::fs::read(&job).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_plumbline")).arg("set").arg(&job).args(["plane-0017", "pitch"]).arg(OsStr::from_bytes(b"4\xff")).args(["--status", "Known", "--source", "x"]).output().unwrap();
    let said = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{said}");
    assert!(said.contains("not readable text") && !said.contains("panicked"), "{said}");
    assert_eq!(std::fs::read(&job).unwrap(), before);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_reader_that_stops_listening_is_not_a_panic() {
    use std::process::Stdio;
    let dir = std::env::temp_dir().join(format!("plumbline-pipe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let job = dir.join("job.json");
    let bin = env!("CARGO_BIN_EXE_plumbline");
    assert!(Command::new(bin).arg("sample").arg(&job).output().unwrap().status.success());
    // Each command, with nobody reading what it prints (as under `| head -1`).
    let job_text = job.to_str().unwrap();
    for args in [
        vec!["show", job_text],
        vec!["lines", job_text],
        vec!["check", job_text],
        vec!["apply", job_text, "parapet-0024", "Parapet with copper coping", "--source", "x"],
        vec!["set", job_text, "parapet-0024", "stretch_out", "24 in", "--line", "Copper coping cap", "--status", "Known", "--source", "x"],
        vec!["set", job_text, "parapet-0024", "quantity", "plan_length", "--line", "Continuous cleat", "--status", "Known", "--source", "x"],
    ] {
        let mut child = Command::new(bin).args(&args).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        drop(child.stdout.take());
        let out = child.wait_with_output().unwrap();
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success() && !said.contains("panicked"), "{args:?}: {said}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A system's total length is no surer than the runs in it, and the takeoff marks it so
/// (ninth judgment): one piece only assumed to be on a system makes its heading read `~`.
#[test]
fn a_takeoffs_heading_carries_the_flag_of_its_total() {
    let dir = std::env::temp_dir().join(format!("plumbline-takeoff-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let job = dir.join("house.json");
    let bin = env!("CARGO_BIN_EXE_plumbline");
    assert!(Command::new(bin).arg("house").arg(&job).output().unwrap().status.success());
    let takeoff = |job: &std::path::Path| String::from_utf8(Command::new(bin).arg("takeoff").arg(job).output().unwrap().stdout).unwrap();
    // As written, every run on the vent is known to be on it: 12.10 LF, no mark, one piece.
    let before = takeoff(&job);
    assert!(before.contains("Vent  (Vent; 1 straight piece, 12.10 LF)"), "{before}");
    assert!(before.contains(" 1 piece "), "one piece is not \"1 pieces\": {before}");
    // Mark that run as only assumed to be on the vent system.
    let mut doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&job).unwrap()).unwrap();
    let vent = doc["data"].as_array().unwrap().iter().find(|e| e["attributes"]["pl::system"]["classification"] == "Vent").unwrap()["path"].as_str().unwrap().to_string();
    for entry in doc["data"].as_array_mut().unwrap() {
        if entry["attributes"]["pl::class"] == "Pipe" && entry["attributes"]["pl::in_system"]["value"] == vent.as_str() {
            entry["attributes"]["pl::in_system"]["status"] = "Assumed".into();
        }
    }
    std::fs::write(&job, serde_json::to_string_pretty(&doc).unwrap()).unwrap();
    let after = takeoff(&job);
    assert!(after.contains("Vent  (Vent; 1 straight piece, ~12.10 LF)"), "{after}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Run the program in a folder of its own; what it printed and whether it went through.
fn run(dir: &std::path::Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_plumbline")).current_dir(dir).args(args).output().unwrap();
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const KNOWN: [&str; 4] = ["--status", "Known", "--source", "made-up plan"];

/// The seventeenth judgment, J17A-16: a made-up job is not written over a file, and a
/// file that is not there is said to be not there in plain words.
#[test]
fn a_made_up_job_is_not_written_over_a_file_and_a_missing_file_is_said_plainly() {
    let dir = folder("over");
    assert!(run(&dir, &["new", "mine.json", "My job (not a real job)"]).0);
    let before = std::fs::read(dir.join("mine.json")).unwrap();
    for verb in ["sample", "house", "sixteen", "priced"] {
        let (ok, said) = run(&dir, &[verb, "mine.json"]);
        assert!(!ok && said.contains("is already there; a made-up job is not written over a file"), "{verb}: {said}");
        assert_eq!(std::fs::read(dir.join("mine.json")).unwrap(), before, "{verb}");
    }
    let (ok, said) = run(&dir, &["estimate", "nothing.json"]);
    assert!(!ok && said.contains("cannot read nothing.json") && !said.contains("os error"), "{said}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// J17B-21: a thing named by its name where its path is wanted is told its path.
/// J17A-13: an order factor that would take the figure off a priced line is refused.
/// J17A-8, 15: the refusals for a wall's height and a stretch-out say what to do.
#[test]
fn a_refusal_says_what_to_give_instead() {
    let dir = folder("refusals");
    assert!(run(&dir, &["new", "j.json", "Refusals (not a real job)"]).0);
    assert!(run(&dir, &[&["level", "j.json", "Ground", "--elevation", "0 ft", "--height", "8 ft"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["type", "j.json", "wall", "w6", "--thickness", "6 in"][..], &KNOWN[..]].concat()).0);
    let (ok, said) = run(&dir, &[&["wall", "j.json", "ground", "type-0003", "--points", "0,0 20,0", "--height", "8 ft"][..], &KNOWN[..]].concat());
    assert!(!ok && said.contains("\"ground\" is the name of level-0002: give level-0002"), "{said}");
    assert!(run(&dir, &[&["wall", "j.json", "level-0002", "type-0003", "--points", "0,0 20,0", "--height", "8 ft"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &["material", "j.json", "Siding", "SF"]).0);
    assert!(run(&dir, &[&["set", "j.json", "material-0005", "price", "$6.25/SF"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["set", "j.json", "wall-0004", "material", "Siding"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &["line", "j.json", "wall-0004", "Siding", "face_area", "--cost-type", "material", "--division", "7", "--of-material"]).0);
    // 20 ft by 8 ft at $6.25: $1,000.00.
    assert!(run(&dir, &["estimate", "j.json"]).1.contains("$1,000.00"));
    let before = std::fs::read(dir.join("j.json")).unwrap();
    let (ok, said) = run(&dir, &[&["set", "j.json", "material-0005", "order", "32 SF/sheet"][..], &KNOWN[..]].concat());
    assert!(!ok && said.contains("1 line that comes to a figure now would come to none: \"Siding\" on wall 1") && said.contains("Nothing was changed"), "{said}");
    assert_eq!(std::fs::read(dir.join("j.json")).unwrap(), before);
    // An order factor that fits is still taken: the same siding bought by the sheet.
    assert!(run(&dir, &["material", "j.json", "Sheathing", "sheet"]).0);
    assert!(run(&dir, &[&["set", "j.json", "material-0006", "order", "32 SF/sheet"][..], &KNOWN[..]].concat()).0);
    let (ok, said) = run(&dir, &[&["set", "j.json", "wall-0004", "height", "8 ft 6 in"][..], &KNOWN[..]].concat());
    assert!(!ok && said.contains("the height of a wall is not set this way") && said.contains("plumbline measure"), "{said}");
    let (ok, said) = run(&dir, &["line", "j.json", "wall-0004", "Fascia wrap", "plan_length * stretch_out", "--cost-type", "material"]);
    assert!(!ok && said.contains("Give the two together") && said.contains("--with"), "{said}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// J17A-19: the estimate says what is built and has nothing priced on it.
#[test]
fn the_estimate_says_what_has_nothing_priced_on_it() {
    let dir = folder("unpriced");
    assert!(run(&dir, &["new", "j.json", "Two walls (not a real job)"]).0);
    assert!(run(&dir, &[&["level", "j.json", "Ground", "--elevation", "0 ft", "--height", "8 ft"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["type", "j.json", "wall", "w6", "--thickness", "6 in"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["wall", "j.json", "level-0002", "type-0003", "--points", "0,0 20,0 20,10", "--height", "8 ft"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &["line", "j.json", "wall-0004", "Framing", "face_area", "--cost-type", "material", "--division", "6"]).0);
    assert!(run(&dir, &[&["set", "j.json", "wall-0004", "unit_cost", "$4.20", "--line", "Framing"][..], &KNOWN[..]].concat()).0);
    let said = run(&dir, &["estimate", "j.json"]).1;
    // 160 SF at $4.20 is $672.00; the second wall has no line at all.
    assert!(said.contains("$672.00") && said.contains("! nothing is priced on 1 thing that could be (1 wall): it is in no line and no total above"), "{said}");
    // The priced sample house has something priced on everything that could be.
    assert!(run(&dir, &["sixteen", "s.json"]).0);
    assert!(!run(&dir, &["estimate", "s.json"]).1.contains("nothing is priced on"));
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The starter library the program ships with comes into an empty job whole (30
/// assemblies, none refused), goes back out byte for byte as the file in the repository,
/// has a line in every one of the sixteen divisions, and has no price in it.
#[test]
fn the_starter_library_comes_in_whole_goes_out_as_it_was_and_has_no_price_in_it() {
    let dir = folder("starter");
    assert!(run(&dir, &["new", "j.json", "Starter (not a real job)"]).0);
    let (ok, said) = run(&dir, &["import", "assemblies", "j.json", "starter"]);
    assert!(ok && said.contains("set: 30") && said.contains("not used: 0"), "{said}");
    assert!(run(&dir, &["export", "assemblies", "j.json", "out.csv"]).0);
    let out = std::fs::read_to_string(dir.join("out.csv")).unwrap();
    let shipped = include_str!("../../../library/sixteen-divisions.csv");
    assert_eq!(out, shipped);
    // The last cell of each line is its division; the one before a "$?/unit" is no figure.
    let mut divisions = std::collections::BTreeSet::new();
    for line in shipped.lines().skip(1) {
        divisions.insert(line.rsplit(',').next().unwrap().parse::<u8>().unwrap());
        assert!(line.contains(",$?/") && line.contains(",Unknown,no price entered,"), "{line}");
        assert!(!line.chars().zip(line.chars().skip(1)).any(|(a, b)| a == '$' && b.is_ascii_digit()), "a price in the library: {line}");
    }
    assert_eq!(divisions.into_iter().collect::<Vec<_>>(), (1..=16).collect::<Vec<u8>>());
    // Brought in a second time, it is the same as the job already has.
    let (ok, said) = run(&dir, &["import", "assemblies", "j.json", "starter"]);
    assert!(ok && said.contains("set: 0") && said.contains("the same as the job already has: 93"), "{said}");
    std::fs::remove_dir_all(&dir).unwrap();
}
