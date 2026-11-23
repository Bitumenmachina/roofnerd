//! Call 182 at the command line: the plan set, as a second judge read it. The job is made up;
//! the figures are worked by hand in `demos/planset.sh` (steps 11, 11a, 11b).

use std::process::Command;

fn run(dir: &std::path::Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_plumbline")).current_dir(dir).args(args).output().unwrap();
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn ok(dir: &std::path::Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(good, "{args:?}: {said}");
    said
}

const K: [&str; 4] = ["--status", "Known", "--source", "made-up survey"];

fn id(said: &str, word: &str) -> String {
    said.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')).find(|w| w.starts_with(word) && w.len() > word.len()).unwrap().to_string()
}

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A house 36 ft by 16 ft with a partition at 24 ft, a door and a window in the south wall and a door in the partition.
fn hand_house(d: &std::path::Path) -> String {
    let j = "job.json";
    ok(d, &["new", j, "Hand house (not a real job)"]);
    let s = id(&ok(d, &[&["level", j, "ground", "--elevation", "0 ft", "--height", "9 ft"], &K[..]].concat()), "level-");
    let t = id(&ok(d, &[&["type", j, "wall", "wall 6 in", "--thickness", "6 in"], &K[..]].concat()), "type-");
    let walls = ok(d, &[&["wall", j, &s, &t, "--points", "0,0 36,0 36,16 0,16", "--closed", "--height", "9 ft"], &K[..]].concat());
    let south = id(&walls, "wall-");
    let part = id(&ok(d, &[&["wall", j, &s, &t, "--points", "24,0 24,16", "--height", "9 ft"], &K[..]].concat()), "wall-");
    let door3 = id(&ok(d, &[&["type", j, "door", "door 3 ft", "--width", "3 ft", "--height", "7 ft"], &K[..]].concat()), "type-");
    let door28 = id(&ok(d, &[&["type", j, "door", "door 2 ft 8 in", "--width", "2 ft 8 in", "--height", "7 ft"], &K[..]].concat()), "type-");
    let win = id(&ok(d, &[&["type", j, "window", "window 5 ft", "--width", "5 ft", "--height", "4 ft"], &K[..]].concat()), "type-");
    ok(d, &[&["door", j, &south, &door3, "--offset", "6 ft", "--sill", "0 ft"], &K[..]].concat());
    ok(d, &[&["window", j, &south, &win, "--offset", "30 ft", "--sill", "3 ft"], &K[..]].concat());
    ok(d, &[&["door", j, &part, &door28, "--offset", "8 ft", "--sill", "0 ft"], &K[..]].concat());
    s
}

#[test]
fn the_hand_house_reads_three_strings_on_the_south_and_each_chain_adds_to_its_whole() {
    let d = folder("planset-hand");
    let s = hand_house(&d);
    let said = ok(&d, &["plan-set", "job.json", &s, "set.pdf", "--date", "2026-11-22", "--list"]);
    for want in [
        "south, openings: 6'-0\" 24'-0\" 6'-0\" (adds to 36'-0\")",
        "south, partitions: 24'-0\" 12'-0\" (adds to 36'-0\")",
        "south, overall: 36'-0\" (adds to 36'-0\")",
        "north, partitions: 24'-0\" 12'-0\" (adds to 36'-0\")",
        "west, overall: 16'-0\" (adds to 16'-0\")",
        "along it inside the plan, openings: 8'-0\" 8'-0\" (adds to 16'-0\")",
        "lettering faults: 0",
    ] {
        assert!(said.contains(want), "{want}\n{said}");
    }
    assert!(!said.contains("north, openings"), "{said}");
    // Five strings of 36 ft (648 points) on the written page: three south, two north.
    let page = ok(&d, &["page", "set.pdf", "--page", "1", "--lines", "--longer-than", "600"]);
    assert_eq!(page.matches(": 648.00 points (9.0000 in)").count(), 5, "{page}");
}

#[test]
fn an_ampersand_and_an_accent_are_lettered_and_what_is_not_is_counted() {
    let d = folder("planset-letters");
    let j = "job.json";
    ok(&d, &["new", j, "Henderson & Sons Résidence (not a real job)"]);
    let s = id(&ok(&d, &[&["level", j, "Première étage", "--elevation", "0 ft", "--height", "9 ft"], &K[..]].concat()), "level-");
    let t = id(&ok(&d, &[&["type", j, "wall", "wall 6 in", "--thickness", "6 in"], &K[..]].concat()), "type-");
    ok(&d, &[&["wall", j, &s, &t, "--points", "0,0 24,0 24,16 0,16", "--closed", "--height", "9 ft"], &K[..]].concat());
    ok(&d, &[&["room", j, &s, "Café & Bar", "--at", "12,8"], &K[..]].concat());
    let said = ok(&d, &["plan-set", j, &s, "a.pdf", "--date", "2026-11-22", "--list"]);
    assert!(said.contains("unlettered: 0 letters"), "{said}");
    // A letter no glyph is made for is said.
    ok(&d, &[&["room", j, &s, "Ωmega", "--at", "4,4"], &K[..]].concat());
    let said = ok(&d, &["plan-set", j, &s, "c.pdf", "--date", "2026-11-22"]);
    assert!(said.contains("could not be lettered as written") && said.contains("Ω as ?"), "{said}");
}

#[test]
fn a_date_is_a_date_or_is_refused_and_a_fault_is_said_without_the_list() {
    let d = folder("planset-date");
    let s = hand_house(&d);
    for bad in ["yesterday", "2026-13-45", "2026-02-30", "11/31/2026"] {
        let (good, why) = run(&d, &["plan-set", "job.json", &s, "x.pdf", "--date", bad]);
        assert!(!good && why.contains("is not a day of the calendar"), "{bad}: {why}");
        assert!(!d.join("x.pdf").exists());
    }
    let said = ok(&d, &["plan-set", "job.json", &s, "x.pdf", "--date", "11/22/2026", "--list"]);
    assert!(said.contains("11/22/2026"), "{said}");
    // Five stub walls 7 1/4 in apart: their figures cannot all be lettered clear, and plan-set says so unasked.
    let j = "crowded.json";
    ok(&d, &["new", j, "Crowded (not a real job)"]);
    let s = id(&ok(&d, &[&["level", j, "ground", "--elevation", "0 ft", "--height", "9 ft"], &K[..]].concat()), "level-");
    let t = id(&ok(&d, &[&["type", j, "wall", "wall 6 in", "--thickness", "6 in"], &K[..]].concat()), "type-");
    ok(&d, &[&["wall", j, &s, &t, "--points", "0,0 24,0 24,16 0,16", "--closed", "--height", "9 ft"], &K[..]].concat());
    for x in ["4", "4.6", "5.2", "5.8", "6.4"] {
        ok(&d, &[&["wall", j, &s, &t, "--points", &format!("{x},0 {x},2"), "--height", "9 ft"], &K[..]].concat());
    }
    let said = ok(&d, &["plan-set", j, &s, "c.pdf", "--date", "2026-11-22"]);
    assert!(said.contains("not right on the pages as written") && said.contains("page 1 of 1, FLOOR PLAN: the lettering"), "{said}");
}
