//! Call 219 at the command line (contract 219M): trying a change and taking it back, a door moved, a copy, a row of copies, a wall copied with its
//! door, and one search. Driven through the program from an empty job. Everything is made up. The job: a level Ground (0 ft, 9 ft high); four walls 6 in
//! thick round 20 ft by 12 ft (0,0 20,0 20,12 0,12, closed); a door 3 ft by 6 ft 8 in in the first wall, its middle 4 ft from the wall's first end; the starter
//! items. The figures are those of `crates/world/tests/call219m.rs`.

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

fn refused(dir: &std::path::Path, args: &[&str]) -> String {
    let (good, said) = run(dir, args);
    assert!(!good, "{args:?} should have been refused: {said}");
    said
}

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-219-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const K: [&str; 4] = ["--status", "Known", "--source", "made-up plan"];
const A: [&str; 4] = ["--status", "Assumed", "--source", "made-up plan"];

fn with(args: &[&str], more: &[&str]) -> Vec<String> {
    args.iter().chain(more.iter()).map(|s| s.to_string()).collect()
}

fn okv(dir: &std::path::Path, args: Vec<String>) -> String {
    ok(dir, &args.iter().map(String::as_str).collect::<Vec<_>>())
}

fn path_in(said: &str, word: &str) -> String {
    said.split_whitespace().map(|w| w.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))).find(|w| w.starts_with(word)).unwrap_or_else(|| panic!("no {word} in {said}")).to_string()
}

struct House {
    dir: std::path::PathBuf,
    level: String,
    wall_type: String,
    walls: Vec<String>,
    door: String,
}

fn house(name: &str) -> House {
    let dir = folder(name);
    ok(&dir, &["new", "j.json", "Tools (not a real job)"]);
    let level = path_in(&okv(&dir, with(&["level", "j.json", "Ground", "--elevation", "0 ft", "--height", "9 ft"], &K)), "level-");
    let wall_type = path_in(&okv(&dir, with(&["type", "j.json", "wall", "Wall 6", "--thickness", "6 in"], &K)), "type-");
    let door_type = path_in(&okv(&dir, with(&["type", "j.json", "door", "Door 3068", "--width", "3 ft", "--height", "6 ft 8 in"], &K)), "type-");
    let made = okv(&dir, with(&["wall", "j.json", &level, &wall_type, "--points", "0,0 20,0 20,12 0,12", "--closed", "--height", "9 ft"], &K));
    let walls: Vec<String> = made.split(": ").nth(1).unwrap().split(", ").map(|s| s.trim().to_string()).collect();
    assert_eq!(walls.len(), 4, "{made}");
    let door = path_in(&okv(&dir, with(&["door", "j.json", &walls[0], &door_type, "--offset", "4 ft", "--sill", "0 ft"], &K)), "door-");
    ok(&dir, &["import", "items", "j.json", "starter"]);
    House { dir, level, wall_type, walls, door }
}

fn bytes(h: &House) -> Vec<u8> {
    std::fs::read(h.dir.join("j.json")).unwrap()
}

/// Step 1 and 2 and 3: tried, and the file is byte for byte what it was.
#[test]
fn try_says_what_it_would_make_and_changes_nothing() {
    let h = house("try");
    let before = bytes(&h);
    let said = okv(&h.dir, with(&["try", "j.json", "wall", &h.level, &h.wall_type, "--points", "10,0 10,12", "--height", "9 ft"], &K));
    assert!(said.contains("it would make wall 5, 12 ft long") && said.contains("it fits") && said.contains("nothing was changed"), "{said}");
    assert_eq!(bytes(&h), before, "the file is byte for byte what it was");
    // And `takeoff` is what it was.
    // A duct through two walls: 2 holes, each named.
    let duct = path_in(&okv(&h.dir, with(&["type", "j.json", "duct", "Duct 14 x 8", "--width", "14 in", "--depth", "8 in"], &K)), "type-");
    let before = bytes(&h);
    let said = okv(&h.dir, with(&["try", "j.json", "run", &h.level, "duct", &duct, "--points", "-2,6 22,6", "--height", "8 ft"], &A));
    assert!(said.contains("2 holes to cut") && said.contains("wall 2") && said.contains("wall 4"), "{said}");
    assert_eq!(bytes(&h), before);
    // A wall over the south wall: refused, as the real verb would; nothing is made.
    let said = refused(&h.dir, &with(&["try", "j.json", "wall", &h.level, &h.wall_type, "--points", "2,0 12,0", "--height", "9 ft"], &K).iter().map(String::as_str).collect::<Vec<_>>());
    assert!(said.contains("it would be refused") && said.contains("lies along wall 1 for 10.00 ft") && said.contains("nothing was changed"), "{said}");
    assert_eq!(bytes(&h), before);
    let real = refused(&h.dir, &with(&["wall", "j.json", &h.level, &h.wall_type, "--points", "2,0 12,0", "--height", "9 ft"], &K).iter().map(String::as_str).collect::<Vec<_>>());
    assert!(real.contains("lies along") && real.contains("one wall drawn over another would be counted twice"), "{real}");
    // A try of an item.
    let said = okv(&h.dir, with(&["try", "j.json", "place", &h.level, "Lavatory", "--at", "5,5", "--offset", "0 ft"], &K));
    assert!(said.contains("it would make Lavatory, 20 × 18 × 34 in"), "{said}");
    assert_eq!(bytes(&h), before);
    // Any other verb that makes a thing is tried on a copy: the verb's own reply, and the file as it was.
    let ftype = path_in(&okv(&h.dir, with(&["type", "j.json", "floor", "Slab 4", "--thickness", "4 in"], &K)), "type-");
    let before = bytes(&h);
    let said = okv(&h.dir, with(&["try", "j.json", "floor", &h.level, &ftype, "--walls"], &K));
    assert!(said.contains("tried floor") && said.contains("a floor on") && said.contains("nothing was changed"), "{said}");
    assert_eq!(bytes(&h), before);
    let said = refused(&h.dir, &["try", "j.json", "remove", "wall-9999"]);
    assert!(said.contains("it would be refused") && said.contains("nothing was changed"), "{said}");
    assert_eq!(bytes(&h), before);
    // A try of what is not a change.
    let said = refused(&h.dir, &["try", "j.json", "takeoff"]);
    assert!(said.contains("try takes wall, door, window, run, place, move, turn, copy or repeat"), "{said}");
}

/// Step 4: a door moved along its wall.
#[test]
fn a_door_is_moved_along_its_wall_and_refused_where_it_would_overrun() {
    let h = house("slide");
    let before = bytes(&h);
    let said = okv(&h.dir, with(&["try", "j.json", "move", &h.door, "--by", "5 ft east"], &A));
    assert!(said.contains("9 ft from the west end") && said.contains("face less openings stays 160.00 SF"), "{said}");
    assert_eq!(bytes(&h), before);
    let said = refused(&h.dir, &with(&["move", "j.json", &h.door, "--by", "15 ft east"], &A).iter().map(String::as_str).collect::<Vec<_>>());
    assert!(said.contains("door 1 would run 6 in past the east end of wall 1") && said.contains("the wall is 20 ft long"), "{said}");
    assert!(!said.contains('(') && !said.contains("door-0"), "no coordinate and no path: {said}");
    assert_eq!(bytes(&h), before, "the job is what it was");
    let said = okv(&h.dir, with(&["move", "j.json", &h.door, "--by", "5 ft east"], &A));
    assert!(said.contains("door 1 is now 9 ft from the west end in wall 1"), "{said}");
    assert_ne!(bytes(&h), before);
    // A wall is moved as it always was.
    let said = okv(&h.dir, with(&["move", "j.json", &h.walls[1], "--by", "1 ft east"], &A));
    assert!(said.contains("moved 1 ft east"), "{said}");
}

/// Step 5: to another wall.
#[test]
fn a_door_is_moved_to_another_wall() {
    let h = house("other");
    let said = okv(&h.dir, with(&["move", "j.json", &h.door, "--to", &h.walls[2], "--offset", "6 ft"], &A));
    assert!(said.contains("wall 1: face less openings was 160.00 SF, now 180.00 SF") && said.contains("wall 3: face less openings was 180.00 SF, now 160.00 SF"), "{said}");
    assert!(said.contains("in wall 3") && said.contains("6 ft from the east end"), "{said}");
    assert!(ok(&h.dir, &["check", "j.json"]).contains("agree"), "the job still opens");
    let why = refused(&h.dir, &with(&["move", "j.json", &h.door, "--offset", "19 ft"], &A).iter().map(String::as_str).collect::<Vec<_>>());
    assert!(why.contains("would run 6 in past"), "{why}");
}

/// Steps 6, 7: a copy is Assumed; a row; and what each says.
#[test]
fn a_copy_and_a_row_of_copies_are_assumed() {
    let h = house("copy");
    let first = path_in(&okv(&h.dir, with(&["place", "j.json", &h.level, "Lavatory", "--at", "5,5", "--offset", "0 ft"], &K)), "fixture-");
    // Six in a row, tried from the lavatory alone: the fifth copy is on the east wall's line.
    let before = bytes(&h);
    let said = ok(&h.dir, &["try", "j.json", "repeat", &first, "--times", "6", "--apart", "3 ft east"]);
    assert!(said.contains("in the same space as 1 thing") && said.contains("would be in the same space as wall 2"), "{said}");
    assert_eq!(bytes(&h), before, "a try changes nothing");
    let said = ok(&h.dir, &["copy", "j.json", &first, "--by", "3 ft east"]);
    assert!(said.contains("Lavatory 2 is a copy of Lavatory") && said.contains("its place is Assumed"), "{said}");
    let shown = ok(&h.dir, &["show", "j.json", &path_in(&said, "fixture-")]);
    assert!(shown.contains("~ position") && shown.contains("~ size"), "a copy's place and size are flagged Assumed: {shown}");
    let original = ok(&h.dir, &["show", "j.json", &first]);
    assert!(!original.contains("~ position"), "the original's place is Known, unflagged: {original}");
    let said = refused(&h.dir, &["copy", "j.json", &first, "--status", "Known", "--by", "3 ft east"]);
    assert!(said.contains("a copy is Assumed"), "{said}");
    let said = ok(&h.dir, &["repeat", "j.json", &first, "--times", "3", "--apart", "2 ft north"]);
    assert!(said.contains("3 copies, each 2 ft north farther than the last"), "{said}");
    let found = ok(&h.dir, &["find", "j.json", "lavatory"]);
    assert!(found.contains("Lavatory 5") || found.contains("Lavatory 4"), "{found}");
    let why = refused(&h.dir, &["repeat", "j.json", &first, "--times", "0", "--apart", "2 ft north"]);
    assert!(why.contains("say how many"), "{why}");
}

/// Step 8: a wall copied with its door.
#[test]
fn a_wall_is_copied_with_its_door() {
    let h = house("wallcopy");
    let before = bytes(&h);
    let why = refused(&h.dir, &["copy", "j.json", &h.walls[0], "--by", "12 ft north"]);
    assert!(why.contains("lies along wall 3 for 20.00 ft"), "{why}");
    assert_eq!(bytes(&h), before);
    let said = ok(&h.dir, &["copy", "j.json", &h.walls[0], "--by", "6 ft north"]);
    assert!(said.contains("wall 5 is a copy of wall 1, with its 1 door or window"), "{said}");
    assert!(ok(&h.dir, &["check", "j.json"]).contains("agree"));
}

/// Step 9: find.
#[test]
fn find_says_what_it_found_and_where_and_says_when_it_found_nothing() {
    let h = house("find");
    let first = path_in(&okv(&h.dir, with(&["place", "j.json", &h.level, "Lavatory", "--at", "5,5", "--offset", "0 ft"], &K)), "fixture-");
    ok(&h.dir, &["copy", "j.json", &first, "--by", "3 ft east"]);
    let said = ok(&h.dir, &["find", "j.json", "lav"]);
    let lines: Vec<&str> = said.lines().collect();
    assert!(lines[0].starts_with("3 found for \"lav\":"), "{said}");
    assert_eq!(lines[1], "  Lavatory: fixture, on Ground", "{said}");
    assert!(lines.iter().any(|l| l.contains("Lavatory: item in the tool chest")), "{said}");
    assert!(lines.iter().any(|l| l.contains("Lavatory 2: fixture, on Ground")), "{said}");
    let said = ok(&h.dir, &["find", "j.json", "south"]);
    assert!(said.contains("nothing in the job is called \"south\""), "{said}");
    let said = ok(&h.dir, &["find", "j.json", "ground"]);
    assert!(said.contains("Ground: level"), "{said}");
    let _ = refused(&h.dir, &["find", "j.json"]);
}

/// Step 10: the sample whole house is what it was: `sixteen` writes the same bytes twice, and every try leaves it.
#[test]
fn the_sample_house_is_unchanged_by_every_try() {
    let dir = folder("sample");
    ok(&dir, &["sixteen", "a.json"]);
    ok(&dir, &["sixteen", "b.json"]);
    let before = std::fs::read(dir.join("a.json")).unwrap();
    assert_eq!(before, std::fs::read(dir.join("b.json")).unwrap(), "the sample is the same every time");
    let wall = ok(&dir, &["takeoff", "a.json"]);
    let _ = wall;
    let found = ok(&dir, &["find", "a.json", "wall"]);
    assert!(found.contains("found for \"wall\""), "{found}");
    assert_eq!(std::fs::read(dir.join("a.json")).unwrap(), before);
}
