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

/// The starter library the program ships with comes into an empty job whole (40
/// assemblies, none refused: call 132 added the footing's, the foundation wall's and the basement wall's,
/// call 135 the whole shingle roof's, call 139 the low-slope roof's, call 145 the slab by its thickness, call 159 taking a partition down, call 164 the beam, the post and the pad footing), goes back out byte for byte as the file in the repository,
/// has a line in every one of the sixteen divisions, and has no price in it.
#[test]
fn the_starter_library_comes_in_whole_goes_out_as_it_was_and_has_no_price_in_it() {
    let dir = folder("starter");
    assert!(run(&dir, &["new", "j.json", "Starter (not a real job)"]).0);
    let (ok, said) = run(&dir, &["import", "assemblies", "j.json", "starter"]);
    assert!(ok && said.contains("set: 41") && said.contains("not used: 0"), "{said}");
    assert!(run(&dir, &["export", "assemblies", "j.json", "out.csv"]).0);
    let out = std::fs::read_to_string(dir.join("out.csv")).unwrap();
    let shipped = include_str!("../../../library/sixteen-divisions.csv");
    assert_eq!(out, shipped);
    // The 27th cell of each line is its division (figures entered on a line follow it,
    // three cells each: call 132); the one before a "$?/unit" is no figure.
    let cells = |line: &str| -> Vec<String> {
        let (mut out, mut cell, mut quoted, mut chars) = (Vec::new(), String::new(), false, line.chars().peekable());
        while let Some(c) = chars.next() {
            match (c, quoted) {
                ('"', true) if chars.peek() == Some(&'"') => {
                    cell.push('"');
                    chars.next();
                }
                ('"', _) => quoted = !quoted,
                (',', false) => out.push(std::mem::take(&mut cell)),
                _ => cell.push(c),
            }
        }
        out.push(cell);
        out
    };
    let mut divisions = std::collections::BTreeSet::new();
    for line in shipped.lines().skip(1) {
        divisions.insert(cells(line)[26].parse::<u8>().unwrap());
        assert!(line.contains(",$?/") && line.contains(",Unknown,no price entered,"), "{line}");
        assert!(!line.chars().zip(line.chars().skip(1)).any(|(a, b)| a == '$' && b.is_ascii_digit()), "a price in the library: {line}");
    }
    assert_eq!(divisions.into_iter().collect::<Vec<_>>(), (1..=16).collect::<Vec<u8>>());
    // Brought in a second time, it is the same as the job already has.
    let (ok, said) = run(&dir, &["import", "assemblies", "j.json", "starter"]);
    assert!(ok && said.contains("set: 0") && said.contains("the same as the job already has: 200"), "{said}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A thing's assembly changed for another: what the two share stays as it is, price and
/// all; the rest of the old one's lines come off and are named; the new one's go on.
#[test]
fn an_assembly_is_changed_for_another_and_what_they_share_keeps_its_price() {
    let dir = folder("change");
    assert!(run(&dir, &["new", "j.json", "Change (not a real job)"]).0);
    assert!(run(&dir, &[&["level", "j.json", "Ground", "--elevation", "0 ft", "--height", "9 ft"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["type", "j.json", "wall", "w6", "--thickness", "6 in"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["wall", "j.json", "level-0002", "type-0003", "--points", "0,0 20,0", "--height", "9 ft"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &["import", "assemblies", "j.json", "starter"]).0);
    assert!(run(&dir, &["apply", "j.json", "wall-0004", "Exterior wall, 2x6, lap siding", "--source", "the plans"]).0);
    // 20 ft at 16 in is 16 studs, 18 with 10 % (call 137): at $4.00, $72.00. 20 ft by 9 ft
    // is 180 SF: siding at $9.50 is $1,710.00.
    assert!(run(&dir, &[&["set", "j.json", "wall-0004", "unit_cost", "$4.00", "--line", "Studs, 2x6 at 16 in"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["set", "j.json", "wall-0004", "unit_cost", "$9.50", "--line", "Lap siding"][..], &KNOWN[..]].concat()).0);
    let said = run(&dir, &["estimate", "j.json"]).1;
    assert!(said.contains("$72.00") && said.contains("$1,710.00"), "{said}");
    // Without saying so, a second assembly is refused and the file is as it was.
    let before = std::fs::read(dir.join("j.json")).unwrap();
    let (ok, said) = run(&dir, &["apply", "j.json", "wall-0004", "Exterior wall, 2x6, brick veneer", "--source", "the plans"]);
    assert!(!ok && said.contains("--change"), "{said}");
    assert_eq!(std::fs::read(dir.join("j.json")).unwrap(), before);
    let (ok, said) = run(&dir, &["apply", "j.json", "wall-0004", "Exterior wall, 2x6, brick veneer", "--source", "the plans", "--change"]);
    assert!(ok && said.contains("has gained 2 cost lines") && said.contains("Brick veneer") && said.contains("3 cost lines of the assembly it had came off") && said.contains("Lap siding") && said.contains("Hang the siding"), "{said}");
    // The framing labor is in both assemblies (call 92), so it stayed: it is not gained and did not come off.
    assert!(!said.contains("Frame and sheathe the wall"), "{said}");
    let said = run(&dir, &["estimate", "j.json"]).1;
    // The studs keep their $72.00; the siding and its $1,710.00 are gone; the brick has no price yet.
    assert!(said.contains("$72.00") && !said.contains("Lap siding") && !said.contains("$1,710.00") && said.contains("Brick veneer"), "{said}");
    assert!(run(&dir, &["check", "j.json"]).0);
    // Changing to what it already is is refused.
    let (ok, said) = run(&dir, &["apply", "j.json", "wall-0004", "Exterior wall, 2x6, brick veneer", "--source", "the plans", "--change"]);
    assert!(!ok && said.contains("is already"), "{said}");
    // An assembly that goes on something else is refused, in words.
    let (ok, said) = run(&dir, &["apply", "j.json", "wall-0004", "Asphalt shingle roof", "--source", "the plans", "--change"]);
    assert!(!ok && said.contains("goes on a roof plane, and wall-0004 is a wall"), "{said}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Call 92: a starter assembly's labor comes with no production rate and no wage, says
/// so, and has hours and dollars once both are typed. A 20 ft wall 9 ft tall has 180 SF
/// of face: at 30 SF/hr, 6 hr; at $60.00, $360.00.
#[test]
fn a_starter_assemblys_labor_has_no_hours_until_its_production_rate_is_typed() {
    let dir = folder("labor");
    assert!(run(&dir, &["new", "j.json", "Labor (not a real job)"]).0);
    assert!(run(&dir, &[&["level", "j.json", "Ground", "--elevation", "0 ft", "--height", "9 ft"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["type", "j.json", "wall", "w6", "--thickness", "6 in"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["wall", "j.json", "level-0002", "type-0003", "--points", "0,0 20,0", "--height", "9 ft"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &["import", "assemblies", "j.json", "starter"]).0);
    let (ok, said) = run(&dir, &["apply", "j.json", "wall-0004", "Interior partition, 2x4", "--source", "the plans"]);
    // Studs, plates, headers and sills in place of one line by the SF (call 137): seven.
    assert!(ok && said.contains("has gained 7 cost lines") && said.contains("Frame the partition"), "{said}");
    let said = run(&dir, &["estimate", "j.json"]).1;
    assert!(said.contains("\"Frame the partition\": the line has no production rate (no production rate entered) and no price (no price entered)"), "{said}");
    let assumed = ["--status", "Assumed", "--source", "made-up crew"];
    assert!(run(&dir, &[&["set", "j.json", "wall-0004", "unit_cost", "$60/hr", "--line", "Frame the partition"][..], &assumed[..]].concat()).0);
    let said = run(&dir, &["estimate", "j.json"]).1;
    assert!(said.contains("\"Frame the partition\": the line has no production rate (no production rate entered), so no hours to price"), "{said}");
    assert!(run(&dir, &[&["set", "j.json", "wall-0004", "order", "30 SF/hr", "--line", "Frame the partition"][..], &assumed[..]].concat()).0);
    let said = run(&dir, &["estimate", "j.json"]).1;
    assert!(said.contains("~6.0000 hr") && said.contains("~$360.00"), "{said}");
    assert!(!said.contains("\"Frame the partition\": the line has no"), "{said}");
    assert!(run(&dir, &["check", "j.json"]).0);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A coverage on a supplier's price list buys whole ones by the rule a typed one does
/// (call 84; M21-6: 100 SF of board at 32 SF to the sheet came in as 3.125 sheets,
/// $37.50, where the same words typed gave 4 sheets, $48.00).
#[test]
fn a_coverage_on_a_price_list_buys_whole_sheets_as_a_typed_one_does() {
    let dir = folder("coverage");
    let mut made = Vec::new();
    for way in ["list", "typed"] {
        let job = format!("{way}.json");
        assert!(run(&dir, &["new", &job, "Board (not a real job)"]).0);
        let level = run(&dir, &[&["level", &job, "ground", "--elevation", "0 ft", "--height", "10 ft"][..], &KNOWN[..]].concat()).1;
        let level = level.split_whitespace().next().unwrap().trim_end_matches(':').to_string();
        let kind = run(&dir, &[&["type", &job, "wall", "wall 6 in", "--thickness", "6 in"][..], &KNOWN[..]].concat()).1;
        let kind = kind.split_whitespace().next().unwrap().trim_end_matches(':').to_string();
        let wall = run(&dir, &[&["wall", &job, &level, &kind, "--points", "0,0 10,0", "--height", "10 ft"][..], &KNOWN[..]].concat()).1;
        let wall = wall.split(|c: char| c.is_whitespace() || c == ',' || c == ':').find(|w| w.starts_with("wall-")).unwrap().to_string();
        let material = run(&dir, &["material", &job, "Gyp board", "sheet"]);
        assert!(material.0, "{}", material.1);
        let line = run(&dir, &["line", &job, &wall, "Board", "face_area", "--cost-type", "material", "--of-material"]);
        assert!(line.0, "{}", line.1);
        let set = run(&dir, &[&["set", &job, &wall, "material", "Gyp board"][..], &KNOWN[..]].concat());
        assert!(set.0, "{}", set.1);
        made.push(job);
    }
    // By the list: no "Whole ones only" column, as a supplier's list has none.
    std::fs::write(dir.join("list.csv"), "Material,Unit,Price,Coverage\nGyp board,sheet,12.00,32 SF/sheet\n").unwrap();
    let brought = run(&dir, &[&["import", "prices", &made[0], "list.csv"][..], &KNOWN[..]].concat());
    assert!(brought.0 && brought.1.contains("ordered at 32 SF/sheet, whole ones only"), "{}", brought.1);
    // By the keyboard.
    let shown = run(&dir, &["show", &made[1]]).1;
    let material = shown.split_whitespace().find(|w| w.starts_with("material-")).unwrap().to_string();
    assert!(run(&dir, &[&["set", &made[1], &material, "price", "12"][..], &KNOWN[..]].concat()).0);
    assert!(run(&dir, &[&["set", &made[1], &material, "order", "32 SF/sheet"][..], &KNOWN[..]].concat()).0);
    // 100 SF at 32 SF to the sheet is 3.125 sheets: 4 bought, at $12.00, $48.00. Both ways.
    for job in &made {
        let estimate = run(&dir, &["estimate", job]).1;
        assert!(estimate.contains("$48.00") && !estimate.contains("$37.50"), "{job}: {estimate}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A quantity made of a value entered on the line is what was entered, as one typed
/// is (call 85; M21-4: 2.5 months of a dumpster at $650 came to $1,950.00 as "months"
/// and $1,625.00 as 2.5).
#[test]
fn a_value_entered_on_a_line_is_not_made_whole_as_a_count_of_pieces_is() {
    let dir = folder("months");
    assert!(run(&dir, &["new", "j.json", "General conditions (not a real job)"]).0);
    let added = run(&dir, &["line", "j.json", "job-0001", "Dumpster", "2.5", "--cost-type", "general", "--division", "1"]);
    assert!(added.0, "{}", added.1);
    assert!(run(&dir, &[&["set", "j.json", "job-0001", "unit_cost", "650", "--line", "Dumpster"][..], &KNOWN[..]].concat()).0);
    // 2.5 x $650.00 = $1,625.00, typed.
    let typed = run(&dir, &["estimate", "j.json"]).1;
    assert!(typed.contains("$1,625.00"), "{typed}");
    // The same 2.5 as a value on the line.
    let named = run(&dir, &[&["set", "j.json", "job-0001", "quantity", "months", "--line", "Dumpster", "--with", "months=2.5"][..], &KNOWN[..]].concat());
    assert!(named.0, "{}", named.1);
    let entered = run(&dir, &["estimate", "j.json"]).1;
    assert!(entered.contains("$1,625.00") && !entered.contains("$1,950.00") && !entered.contains("a count is whole"), "{entered}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A wall type and a footing type are what they were made as, from the first (call 87;
/// J21-2, J21-5, J21-14): a footing is not drawn on a wall's type nor a wall on a
/// footing's, a footing's type reads as a width before anything is drawn on it, and a
/// type in use is not removed, in words that are true.
#[test]
fn a_type_is_a_walls_or_a_footings_from_the_moment_it_is_made() {
    let dir = folder("typekinds");
    assert!(run(&dir, &["new", "j.json", "Types (not a real job)"]).0);
    let level = run(&dir, &[&["level", "j.json", "foundation", "--elevation", "-4 ft", "--height", "4 ft"][..], &KNOWN[..]].concat()).1;
    let level = level.split_whitespace().next().unwrap().trim_end_matches(':').to_string();
    let first = |said: String| said.split_whitespace().next().unwrap().trim_end_matches(':').to_string();
    let wall_type = first(run(&dir, &[&["type", "j.json", "wall", "fdn wall 8", "--thickness", "8 in"][..], &KNOWN[..]].concat()).1);
    let footing_type = first(run(&dir, &[&["type", "j.json", "footing", "ftg 24x12", "--width", "24 in"][..], &KNOWN[..]].concat()).1);
    // Before anything is drawn on it, the footing's type is listed as a width.
    let shown = run(&dir, &["show", "j.json"]).1;
    assert!(shown.contains("footing width    24 in") && shown.contains("wall thickness   8 in"), "{shown}");
    // A footing on the wall's type, and a wall on the footing's: each refused, saying what to use.
    let wrong = run(&dir, &[&["footing", "j.json", &level, &wall_type, "--points", "0,0 10,0", "--depth", "12 in"][..], &KNOWN[..]].concat());
    assert!(!wrong.0 && wrong.1.contains("was made as a wall's type") && wrong.1.contains(&footing_type), "{}", wrong.1);
    let wrong = run(&dir, &[&["wall", "j.json", &level, &footing_type, "--points", "0,0 10,0", "--height", "4 ft"][..], &KNOWN[..]].concat());
    assert!(!wrong.0 && wrong.1.contains("was made as a footing's type") && wrong.1.contains(&wall_type), "{}", wrong.1);
    // A type that is not there: refused before anything is made, offering the job's own.
    let missing = run(&dir, &[&["wall", "j.json", &level, "wall-type-0003", "--points", "0,0 10,0", "--height", "4 ft"][..], &KNOWN[..]].concat());
    assert!(!missing.0 && missing.1.contains("wall-type-0003 is not a wall's type") && missing.1.contains(&wall_type) && !missing.1.contains("wall-00"), "{}", missing.1);
    // Each on its own type is taken; then the footing's type cannot be removed from under it.
    let made = run(&dir, &[&["footing", "j.json", &level, &footing_type, "--points", "0,0 10,0", "--depth", "12 in"][..], &KNOWN[..]].concat());
    assert!(made.0, "{}", made.1);
    assert!(run(&dir, &[&["wall", "j.json", &level, &wall_type, "--points", "0,0 10,0", "--height", "4 ft"][..], &KNOWN[..]].concat()).0);
    let kept = run(&dir, &["remove", "j.json", &footing_type]);
    assert!(!kept.0 && kept.1.contains(&format!("{footing_type} is the type of footing-")) && !kept.1.contains("is not a footing type"), "{}", kept.1);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_estimate_is_read_by_item_and_comes_to_the_same_job_cost() {
    // The made-up whole house: 399 lines of the sheet are 52 items. Its four strip
    // footings, 2.9630 + 2.0741 + 2.9630 + 2.0741 = 10.0742 CY at a made-up $450.00,
    // are one row of 4 places at $4,533.40, under Division 3 at $32,485.40, and the
    // foot is the job cost `estimate` gives.
    let dir = std::env::temp_dir().join(format!("plumbline-by-item-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let job = dir.join("job.json");
    let bin = env!("CARGO_BIN_EXE_plumbline");
    assert!(Command::new(bin).arg("sixteen").arg(&job).output().unwrap().status.success());
    let out = Command::new(bin).arg("estimate").arg(&job).arg("by-item").output().unwrap();
    let said = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(out.status.success(), "{said}");
    assert!(said.contains("THE ESTIMATE, BY ITEM: 52 items from 399 lines"), "{said}");
    let squeezed = |line: &str| line.split_whitespace().collect::<Vec<_>>().join(" ");
    let rows: Vec<String> = said.lines().map(squeezed).collect();
    assert!(rows.iter().any(|r| r == "Division 3 Concrete $32,485.40"), "{said}");
    assert!(rows.iter().any(|r| r == "Concrete footing, in place Subcontract 4 10.0742 CY $450.00/CY $4,533.40"), "{said}");
    let every = String::from_utf8_lossy(&Command::new(bin).arg("estimate").arg(&job).output().unwrap().stdout).to_string();
    let job_cost = every.lines().map(squeezed).find(|l| l.starts_with("Job cost ")).unwrap();
    // Call 179, C2: $85.50 more than it was ($390,174.76): the siding below the 3 ft of the stone is taken off by what
    // lies below it (9 SF of windows with their sills at 2 1/2 ft, at $9.50), and not by the stone's length.
    assert!(job_cost.starts_with("Job cost $390,260.26"), "{job_cost}");
    assert!(rows.last().unwrap().ends_with("$390,260.26"), "{said}");
    // A word that is not a way to read it is refused, and says the two that are.
    let no = Command::new(bin).arg("estimate").arg(&job).arg("by-thing").output().unwrap();
    assert_eq!(no.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&no.stderr).contains("`plumbline estimate <file> by-item` is each item once"));
    std::fs::remove_dir_all(&dir).unwrap();
}
