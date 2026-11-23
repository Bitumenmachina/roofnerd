//! Call 168 at the command line: a plan set on paper. The job is made up; the figures are
//! worked by hand in STATUS.md ("A plan set on paper").

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

fn house(dir: &std::path::Path, north: &[&str]) -> (String, String) {
    let j = "job.json";
    ok(dir, &["new", j, "Plan set (not a real job)"]);
    let s = id(&ok(dir, &[&["level", j, "ground", "--elevation", "0 ft", "--height", "9 ft"], &K[..]].concat()), "level-");
    let t = id(&ok(dir, &[&["type", j, "wall", "wall 6 in", "--thickness", "6 in"], &K[..]].concat()), "type-");
    let wall = |pts: &str, how: &[&str]| id(&ok(dir, &[&["wall", j, &s, &t, "--points", pts, "--height", "9 ft"], how].concat()), "wall-");
    let south = wall("0,0 24,0", &K);
    let east = wall("24,0 24,16", &K);
    let n = wall("24,16 0,16", if north.is_empty() { &K } else { north });
    let west = wall("0,16 0,0", &K);
    ok(dir, &[&["mark", j, &east, "demolish"], &K[..]].concat());
    ok(dir, &[&["mark", j, &south, &n, &west, "existing"], &K[..]].concat());
    ok(dir, &[&["wall", j, &s, &t, "--points", "24,0 36,0 36,16 24,16", "--height", "9 ft"], &K[..]].concat());
    ok(dir, &[&["room-line", j, &s, "--points", "24,0 24,16"], &K[..]].concat());
    ok(dir, &[&["room", j, &s, "Living", "--at", "6,8"], &K[..]].concat());
    ok(dir, &[&["room", j, &s, "Kitchen", "--at", "30,8"], &K[..]].concat());
    (s, south)
}

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_three_pages_come_out_to_the_figures_worked_by_hand() {
    let d = folder("planset-figures");
    let (s, _) = house(&d, &[]);
    let said = ok(&d, &["plan-set", "job.json", &s, "set.pdf", "--date", "2026-11-22", "--list"]);
    for want in ["3 pages, ARCH A paper (12 by 9 in)", "page 1 of 3: EXISTING PLAN", "walls: 4 (4 heavy, 0 light, 0 dashed)", "LIVING 364 SF", "page 2 of 3: DEMOLITION PLAN", "walls: 4 (0 heavy, 3 light, 1 dashed)", "1 WALL COMES DOWN, 16 LF", "page 3 of 3: PROPOSED PLAN", "walls: 6 (3 heavy, 3 light, 0 dashed)", "LIVING 368 SF", "KITCHEN 182 SF", "NO LENGTH CARRIES VIF"] {
        assert!(said.contains(want), "{want}\n{said}");
    }
    // Kitchen is lettered only on the proposed plan.
    assert_eq!(said.matches("KITCHEN 182 SF").count(), 1);
    // The longest line on the sheet is now the overall dimension, 36 ft at 1/4 in: 648
    // points (call 182, item 1: each side carries its strings and an overall; before, the
    // longest was the south wall's own 24 ft string, 432 points).
    let page = ok(&d, &["page", "set.pdf", "--page", "3", "--lines", "--longer-than", "600"]);
    assert!(page.contains("864 by 648 points (12 by 9 in)"), "{page}");
    assert!(page.contains(": 648.00 points (9.0000 in)"), "{page}");
    let again = ok(&d, &["plan-set", "job.json", &s, "again.pdf", "--date", "2026-11-22"]);
    assert!(again.starts_with("wrote again.pdf"));
    assert_eq!(std::fs::read(d.join("set.pdf")).unwrap(), std::fs::read(d.join("again.pdf")).unwrap());
}

#[test]
fn a_length_that_is_not_known_carries_vif_and_a_scale_that_does_not_fit_is_refused() {
    let d = folder("planset-vif");
    let (s, _) = house(&d, &["--status", "Assumed", "--source", "made-up sketch"]);
    let said = ok(&d, &["plan-set", "job.json", &s, "set.pdf", "--date", "2026-11-22", "--list"]);
    // On each of the three pages the 24 ft wall is lettered on its own string and again on
    // the string of partitions outside it (call 182, item 1): 6 where it was 3. The note
    // still counts the model's one length to verify, once a page.
    assert_eq!(said.matches("24'-0\" VIF").count(), 6, "{said}");
    assert_eq!(said.matches("1 LENGTH IS TO VERIFY IN FIELD (VIF).").count(), 3);
    let before = std::fs::read(d.join("job.json")).unwrap();
    let (good, why) = run(&d, &["plan-set", "job.json", &s, "small.pdf", "--scale", "1/2 in", "--paper", "ARCH A"]);
    assert!(!good && why.contains("at 1/4\" = 1'-0\" it does"), "{why}");
    assert!(!d.join("small.pdf").exists());
    let big = ok(&d, &["plan-set", "job.json", &s, "big.pdf", "--scale", "1/2 in", "--list"]);
    assert!(big.contains("ARCH C paper (24 by 18 in)"), "{big}");
    assert_eq!(before, std::fs::read(d.join("job.json")).unwrap());
}

#[test]
fn a_door_is_its_gap_with_no_swing_and_a_level_with_no_mark_has_one_page() {
    let d = folder("planset-door");
    let (s, south) = house(&d, &[]);
    let t = id(&ok(&d, &[&["type", "job.json", "door", "door 3 ft", "--width", "3 ft", "--height", "7 ft"], &K[..]].concat()), "type-");
    ok(&d, &[&["door", "job.json", &south, &t, "--offset", "12 ft", "--sill", "0 ft"], &K[..]].concat());
    let said = ok(&d, &["plan-set", "job.json", &s, "set.pdf", "--date", "2026-11-22", "--list"]);
    // An unmarked door is new work: it is on the proposed plan alone, not on the existing
    // or the demolition plan (call 179, D1; it was drawn on all three).
    assert_eq!(said.matches(": door, gap 3'-0\"").count(), 1, "{said}");
    assert!(said.contains(", no swing") && said.contains("DOOR SWINGS ARE NOT SET."));
    let p = folder("planset-plain");
    ok(&p, &["new", "j.json", "Plain (not a real job)"]);
    let s = id(&ok(&p, &[&["level", "j.json", "ground", "--elevation", "0 ft", "--height", "9 ft"], &K[..]].concat()), "level-");
    let t = id(&ok(&p, &[&["type", "j.json", "wall", "w", "--thickness", "6 in"], &K[..]].concat()), "type-");
    ok(&p, &[&["wall", "j.json", &s, &t, "--points", "0,0 24,0 24,16 0,16", "--closed", "--height", "9 ft"], &K[..]].concat());
    let said = ok(&p, &["plan-set", "j.json", &s, "p.pdf", "--list"]);
    assert!(said.contains("1 page,") && said.contains("FLOOR PLAN") && !said.contains("EXISTING PLAN"), "{said}");
}
