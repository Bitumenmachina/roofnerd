//! Call 167 at the command line: what a person reads of what the judge of calls 159 to 166
//! found. The job is made up; the figures are worked by hand in STATUS.md.

use std::process::Command;

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

fn with(args: &[&str], more: &[&str]) -> Vec<String> {
    args.iter().chain(more.iter()).map(|s| s.to_string()).collect()
}

fn ok(dir: &std::path::Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(good, "{args:?}: {said}");
    said
}

fn path_in(said: &str, word: &str) -> String {
    said.split_whitespace().map(|w| w.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))).find(|w| w.starts_with(word)).unwrap_or_else(|| panic!("no {word} in {said}")).to_string()
}

/// A 24 by 16 house of four 6 in walls, 9 ft tall, nothing priced, and a partition at east 12.
fn house(dir: &std::path::Path) -> (String, Vec<String>) {
    ok(dir, &["new", "job.json", "A house (not a real job)"]);
    let level = path_in(&ok(dir, &with(&["level", "job.json", "ground", "--elevation", "0 ft", "--height", "9 ft"], &KNOWN).iter().map(String::as_str).collect::<Vec<_>>()), "level-");
    let wall_type = path_in(&ok(dir, &with(&["type", "job.json", "wall", "wall 6 in", "--thickness", "6 in"], &KNOWN).iter().map(String::as_str).collect::<Vec<_>>()), "type");
    let walls = ok(dir, &with(&["wall", "job.json", &level, &wall_type, "--points", "0,0 24,0 24,16 0,16", "--closed", "--height", "9 ft"], &KNOWN).iter().map(String::as_str).collect::<Vec<_>>());
    let walls: Vec<String> = walls.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')).filter(|w| w.starts_with("wall-")).map(str::to_string).collect();
    let part = ok(dir, &with(&["wall", "job.json", &level, &wall_type, "--points", "12,0 12,16", "--height", "9 ft"], &KNOWN).iter().map(String::as_str).collect::<Vec<_>>());
    let part = path_in(&part, "wall-");
    (part, walls)
}

#[test]
fn a_bearing_wall_the_work_opens_says_so_under_the_estimate_with_nothing_priced() {
    let dir = folder("call167-caution");
    let (part, walls) = house(&dir);
    let mut all: Vec<&str> = walls.iter().map(String::as_str).collect();
    all.push(&part);
    ok(&dir, &with(&["mark", "job.json"], &all).iter().map(String::as_str).chain(["existing", "--status", "Known", "--source", "made-up plan"]).collect::<Vec<_>>());
    ok(&dir, &["mark", "job.json", &part, "demolish", "--status", "Known", "--source", "made-up plan"]);
    ok(&dir, &["set", "job.json", &part, "bearing", "load bearing", "--status", "Assumed", "--source", "joists lap over it"]);
    // Nothing is priced on it, or on anything: the estimate has no line, and says what comes down.
    let said = ok(&dir, &["estimate", "job.json"]);
    assert!(said.contains("Nothing in this job has a cost line."), "{said}");
    assert!(said.contains("is bearing and comes down") && said.contains("None of that is in this price"), "{said}");
    let said = ok(&dir, &["estimate", "job.json", "by-item"]);
    assert!(said.contains("is bearing and comes down"), "{said}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn what_estimators_write_is_taken_and_what_is_not_a_measure_is_refused_with_the_units() {
    let dir = folder("call167-measure");
    let (_, walls) = house(&dir);
    let south = &walls[0];
    let said = ok(&dir, &["line", "job.json", south, "Patch", "1,200 sq ft", "--status", "Assumed", "--source", "walked it"]);
    assert!(said.contains("\"Patch\" = 1200 SF (Assumed, walked it)"), "{said}");
    let said = ok(&dir, &["line", "job.json", south, "Edge", "24 ft", "--status", "Assumed", "--source", "walked it"]);
    assert!(said.contains("\"Edge\" = 24 LF (Assumed, walked it)"), "{said}");
    let before = std::fs::read(dir.join("job.json")).unwrap();
    for (typed, wanted) in [("-3 SF", "is below zero"), ("60 boards", "one of LF, SF, SQ, CY, EA")] {
        let (good, said) = run(&dir, &["line", "job.json", south, "Bad", typed, "--status", "Assumed", "--source", "walked it"]);
        assert!(!good && said.contains(wanted), "{typed}: {said}");
        let (good, said) = run(&dir, &["set", "job.json", south, "measured", typed, "--line", "Patch", "--status", "Assumed", "--source", "walked it"]);
        assert!(!good && said.contains(wanted), "{typed}: {said}");
    }
    assert_eq!(std::fs::read(dir.join("job.json")).unwrap(), before, "a refusal leaves the job as it was");
    // And the field list asks both, to measure, by line and what was typed.
    let said = ok(&dir, &["vif", "job.json"]);
    assert!(said.contains("measure: ~\"Patch\" (1200 SF), ~\"Edge\" (24 LF)"), "{said}");
    let _ = std::fs::remove_dir_all(&dir);
}
