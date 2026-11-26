//! Call 226 at the command line (contract 226M): lengths said as a builder says them, the database's reply, a cabinet rule taken away, a height
//! the rule does not stock. Driven through the program from an empty job. Everything is made up. The job of `call219m.rs`: a level Ground, four
//! walls 6 in thick round 20 ft by 12 ft, a door 3 ft by 6 ft 8 in in the first wall, the starter items.

use plumbline_sheets as sheets;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_plumbline")).current_dir(dir).args(args).output().unwrap();
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn ok(dir: &Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(good, "{args:?}: {said}");
    said
}

fn refused(dir: &Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(!good, "{args:?} should have been refused: {said}");
    said
}

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-226-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const K: [&str; 4] = ["--status", "Known", "--source", "made-up plan"];
const A: [&str; 4] = ["--status", "Assumed", "--source", "made-up plan"];

fn with(args: &[&str], more: &[&str]) -> Vec<String> {
    args.iter().chain(more.iter()).map(|s| s.to_string()).collect()
}

fn okv(dir: &Path, args: Vec<String>) -> String {
    ok(dir, &args.iter().map(String::as_str).collect::<Vec<_>>())
}

fn path_in(said: &str, word: &str) -> String {
    said.split_whitespace().map(|w| w.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))).find(|w| w.starts_with(word)).unwrap_or_else(|| panic!("no {word} in {said}")).to_string()
}

struct House {
    dir: PathBuf,
    level: String,
    wall_type: String,
    walls: Vec<String>,
    room: String,
}

fn house(name: &str) -> House {
    let dir = folder(name);
    ok(&dir, &["new", "j.json", "Loose ends (not a real job)"]);
    let level = path_in(&okv(&dir, with(&["level", "j.json", "Ground", "--elevation", "0 ft", "--height", "9 ft"], &K)), "level-");
    let wall_type = path_in(&okv(&dir, with(&["type", "j.json", "wall", "Wall 6", "--thickness", "6 in"], &K)), "type-");
    let door_type = path_in(&okv(&dir, with(&["type", "j.json", "door", "Door 3068", "--width", "3 ft", "--height", "6 ft 8 in"], &K)), "type-");
    let made = okv(&dir, with(&["wall", "j.json", &level, &wall_type, "--points", "0,0 20,0 20,12 0,12", "--closed", "--height", "9 ft"], &K));
    let walls: Vec<String> = made.split(": ").nth(1).unwrap().split(", ").map(|s| s.trim().to_string()).collect();
    okv(&dir, with(&["door", "j.json", &walls[0], &door_type, "--offset", "4 ft", "--sill", "0 ft"], &K));
    ok(&dir, &["import", "items", "j.json", "starter"]);
    let room = path_in(&okv(&dir, with(&["room", "j.json", &level, "Kitchen", "--at", "10,6"], &K)), "room-");
    House { dir, level, wall_type, walls, room }
}

fn bytes(h: &House) -> Vec<u8> {
    std::fs::read(h.dir.join("j.json")).unwrap()
}

/// Step 1: lengths in sentences; the quantities of the estimate and the lists stay as they were.
#[test]
fn step_1_a_length_in_a_sentence_is_said_as_a_builder_says_it() {
    let h = house("one");
    let takeoff = ok(&h.dir, &["takeoff", "j.json"]);
    let before = bytes(&h);
    let said = refused(&h.dir, &with(&["try", "j.json", "wall", &h.level, &h.wall_type, "--points", "0,0 10,0", "--height", "9 ft"], &K).iter().map(String::as_str).collect::<Vec<_>>());
    assert!(said.contains("lies along wall 1 for 10 ft") && !said.contains("10.00"), "{said}");
    let duct = path_in(&okv(&h.dir, with(&["type", "j.json", "duct", "Duct 14 x 8", "--width", "14 in", "--depth", "8 in"], &K)), "type-");
    let said = okv(&h.dir, with(&["try", "j.json", "run", &h.level, "duct", &duct, "--points", "-2,6 22,6", "--height", "8 ft"], &A));
    assert_eq!(said.matches("14 in by 8 in").count(), 2, "{said}");
    assert!(!said.contains("14.00"), "{said}");
    // A quantity on a line is not a sentence: the takeoff writes what it wrote.
    assert!(takeoff.contains("wall face, net     556.00 SF"), "{takeoff}");
    assert_eq!(ok(&h.dir, &["takeoff", "j.json"]), takeoff);
    assert_eq!(bytes(&h).len(), std::fs::read(h.dir.join("j.json")).unwrap().len());
    let _ = before;
    // The list of clashes still writes a hole's size to two places (a list is not a sentence).
    okv(&h.dir, with(&["run", "j.json", &h.level, "duct", &duct, "--points", "-2,6 22,6", "--height", "8 ft"], &A));
    let list = ok(&h.dir, &["clashes", "j.json"]);
    assert!(list.contains("14.00 in by 8.00 in"), "{list}");
}

/// `try --timed N`: the change tried N times, each timed, and the least said; nothing changed.
#[test]
fn step_2_a_try_is_timed_and_changes_nothing() {
    let h = house("two");
    let before = bytes(&h);
    let said = okv(&h.dir, with(&["try", "j.json", "wall", &h.level, &h.wall_type, "--points", "10,0 10,12", "--height", "9 ft", "--timed", "3"], &K));
    assert!(said.contains("try 1: ") && said.contains("try 3: ") && said.contains("the least of 3: ") && said.contains("it would make wall 5, 12 ft long"), "{said}");
    assert!(said.contains("writing the whole job out: ") && said.contains("reading it back: "), "{said}");
    assert_eq!(bytes(&h), before);
    let why = refused(&h.dir, &with(&["try", "j.json", "wall", &h.level, &h.wall_type, "--points", "10,0 10,12", "--height", "9 ft", "--timed", "none"], &K).iter().map(String::as_str).collect::<Vec<_>>());
    assert!(why.contains("--timed is a whole number of tries"), "{why}");
}

// ---------------------------------------------------------------- step 3: the database's reply

fn tabs_of(path: &Path) -> Vec<(String, Vec<Vec<String>>)> {
    sheets::read_tabs(&std::fs::read(path).unwrap()).unwrap().into_iter().map(|s| (s.name, s.rows)).collect()
}

/// A workbook with a tab "Notes" added by hand: the line about how cells are shown is said once, and the tab is named as it was typed.
#[test]
fn step_3_the_line_about_how_cells_are_shown_is_said_once_and_a_tab_is_named_as_typed() {
    let dir = folder("three");
    ok(&dir, &["new", "a.json", "Database (not a real job)"]);
    for t in ["items", "connections", "assemblies"] {
        ok(&dir, &["import", t, "a.json", "starter"]);
    }
    ok(&dir, &["export", "database", "a.json", "my.xlsx"]);
    let mut tabs = tabs_of(&dir.join("my.xlsx"));
    tabs.push(("Notes \"draft\"".into(), vec![vec!["Remember".into()], vec!["call the supplier".into()]]));
    let tabs: Vec<sheets::Tab> = tabs
        .into_iter()
        .map(|(name, rows)| sheets::Tab { name, rows: rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { sheets::Cell::Empty } else { sheets::Cell::Text(c.clone()) }).collect()).collect(), head: 0, widths: vec![] })
        .collect();
    std::fs::write(dir.join("edited.xlsx"), sheets::xlsx(&tabs).unwrap()).unwrap();
    ok(&dir, &["new", "b.json", "Second (not a real job)"]);
    let said = ok(&dir, &["import", "database", "b.json", "edited.xlsx"]);
    assert_eq!(said.matches("how its cells are shown").count(), 1, "{said}");
    assert!(said.contains("not read: the tab \"Notes \"draft\"\": this program has no table of that name"), "{said}");
    assert!(!said.contains("\\\""), "no program's writing of a string: {said}");
}

// ---------------------------------------------------------------- step 4: a rule taken away

fn rule_file(dir: &Path, name: &str, body: &str) {
    std::fs::write(dir.join(name), format!("Code,What it is,Number reads,Width,Depth,Height,Stands,Drawn as,Stocked widths,How sure,Where from\n{body}")).unwrap();
}

fn boxes(h: &House, codes: &str) -> (bool, String) {
    boxes_from(h, codes, "1 ft")
}

fn boxes_from(h: &House, codes: &str, from: &str) -> (bool, String) {
    let args = with(&["cabinets", "j.json", &h.level, &h.walls[1], "--in", &h.room, "--from", from, "--boxes", codes], &A);
    run(&h.dir, &args.iter().map(String::as_str).collect::<Vec<_>>())
}

/// Step 4: a rule X brought in (reads the width; 24 in deep, 30 in high; a base). X30 asked for: made. The rule taken away: X33 is refused, X30
/// is still in the chest and still says the rule for X made it. B changed to 21 in deep and taken away: B45 is made 24 in deep. A rule the job
/// does not carry, taken away: it is said to be the program's own and nothing changes.
#[test]
fn step_4_a_rule_is_taken_away_in_the_three_cases() {
    let h = house("four");
    // Table brought in with the eleven headings of a table written before the heights' column: it reads as before.
    rule_file(&h.dir, "x.csv", "X,base,the width,from the number,24 in,30 in,on the floor,base cabinet,,Assumed,a made-up rule\n");
    ok(&h.dir, &["import", "cabinet-rules", "j.json", "x.csv"]);
    let (good, said) = boxes(&h, "X30");
    assert!(good && said.contains("X30 was made by the rule for X: 30 in wide, 24 in deep and 30 in high"), "{said}");
    let said = ok(&h.dir, &["remove-cabinet-rule", "j.json", "X"]);
    assert!(said.contains("the rule for X was taken away") && said.contains("no rule reads X now") && said.contains("X30") && said.contains("stay in the tool chest"), "{said}");
    let (good, said) = boxes(&h, "X33");
    assert!(!good && said.contains("no cabinet rule reads X"), "{said}");
    ok(&h.dir, &["export", "items", "j.json", "items.csv"]);
    let items = std::fs::read_to_string(h.dir.join("items.csv")).unwrap();
    assert!(items.lines().any(|l| l.starts_with("X30,") && l.contains("the cabinet rule for X")), "X30 is still in the chest, saying the rule for X made it");
    assert!(!String::from_utf8(bytes(&h)).unwrap().contains("cabinetRules"), "the job carries no rule of its own any more");
    // Case 2: B changed to 21 in deep, then taken away.
    ok(&h.dir, &["export", "cabinet-rules", "j.json", "rules.csv"]);
    let rules = std::fs::read_to_string(h.dir.join("rules.csv")).unwrap();
    let changed = rules.replacen("B,base,the width,from the number,24 in,", "B,base,the width,from the number,21 in,", 1);
    assert_ne!(rules, changed);
    std::fs::write(h.dir.join("rules.csv"), changed).unwrap();
    ok(&h.dir, &["import", "cabinet-rules", "j.json", "rules.csv"]);
    let said = ok(&h.dir, &["remove-cabinet-rule", "j.json", "B"]);
    assert!(said.contains("B goes back to the program's own") && said.contains("the job's rule for B was taken away"), "{said}");
    let (good, said) = boxes_from(&h, "B45", "6 ft");
    assert!(good && said.contains("B45 was made by the rule for B: 45 in wide, 24 in deep"), "{said}");
    // Case 3: nothing carried: said to be the program's own, and the file is what it was. A code nothing reads is refused.
    let before = bytes(&h);
    let said = ok(&h.dir, &["remove-cabinet-rule", "j.json", "SB"]);
    assert!(said.contains("SB is the program's own rule and the job carries nothing of it: nothing was changed"), "{said}");
    assert_eq!(bytes(&h), before);
    let why = refused(&h.dir, &["remove-cabinet-rule", "j.json", "Q"]);
    assert!(why.contains("no cabinet rule reads Q"), "{why}");
    assert_eq!(bytes(&h), before);
}

/// Step 5: W3342 made 33 by 12 by 42 and flagged twice; W3030 is the chest's own and says nothing.
#[test]
fn step_5_a_height_the_rule_does_not_stock_is_flagged_twice() {
    let h = house("five");
    let (good, said) = boxes(&h, "W3342");
    assert!(good, "{said}");
    assert!(said.contains("W3342 was made by the rule for W: 33 in wide, 12 in deep and 42 in high"), "{said}");
    assert!(said.contains("33 in is not a stocked width of W") && said.contains("42 in is not a stocked height of W"), "{said}");
    let (good, said) = boxes_from(&h, "W3030", "6 ft");
    assert!(good && !said.contains("not a stocked"), "{said}");
    // The heights are in the table, last, with a heading.
    ok(&h.dir, &["export", "cabinet-rules", "j.json", "rules.csv"]);
    let rules = std::fs::read_to_string(h.dir.join("rules.csv")).unwrap();
    assert!(rules.lines().next().unwrap().ends_with(",Stocked heights"), "{rules}");
    assert!(rules.contains("\"12, 18, 30 in\"") && rules.contains(",84 in"), "{rules}");
}
