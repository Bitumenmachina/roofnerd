//! Call 233 at the command line (contract 233M): the sample whole house does not move, and the new shapes and items show in the words a
//! person reads. Everything is made up.

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

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-233-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sum(text: &[u8]) -> String {
    use std::io::Write;
    let mut child = Command::new("sha256sum").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(text).unwrap();
    String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout)[..16].to_string()
}

/// The sample whole house, its estimate, its clash list and its verify-in-field list: held to the sums of copies made before call 233.
#[test]
fn the_sample_whole_house_and_what_is_said_of_it_are_what_they_were() {
    let dir = folder("sample");
    ok(&dir, &["sixteen", "h.json"]);
    assert_eq!(sum(&std::fs::read(dir.join("h.json")).unwrap()), "880d9fd3c364b5f7");
    for (verb, want) in [("estimate", "b07a6b07390ff523"), ("clashes", "0250881a220082d8"), ("vif", "14bd89dcaf5908e3")] {
        let said = ok(&dir, &[verb, "h.json"]);
        assert_eq!(sum(said.as_bytes()), want, "{verb} of the sample house moved");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The chest at the command line: 57 items; a receptacle says its shape and its blocks; the range is drawn as a range.
#[test]
fn the_chest_lists_the_new_items_and_the_seven_say_what_they_are_drawn_as() {
    let dir = folder("chest");
    ok(&dir, &["new", "j.json", "Devices (not a real job)"]);
    let said = ok(&dir, &["import", "items", "j.json", "starter"]);
    assert!(said.contains("  set: 57"), "{said}");
    let list = ok(&dir, &["items", "j.json"]);
    assert!(list.contains("the things that can be placed, by trade (57)"), "{list}");
    for (item, drawn) in [("Range, 30 in", "a range"), ("Refrigerator", "a refrigerator"), ("Dishwasher", "a dishwasher"), ("Washer", "a laundry machine"), ("Dryer", "a laundry machine"), ("Air handler", "an air handler"), ("Panelboard, 200 A", "a panelboard"), ("Duplex receptacle", "a receptacle"), ("Switch", "a switch")] {
        let line = list.lines().find(|l| l.contains(&format!("{item} ["))).unwrap_or_else(|| panic!("{item}: {list}"));
        assert!(line.contains(&format!("drawn as {drawn}")), "{line}");
    }
    let one = ok(&dir, &["items", "j.json", "Duplex receptacle"]);
    assert!(one.contains("sizes: 2.75 in wide, 0.5 in deep, 4.5 in high; ~"), "{one}");
    assert!(one.contains("drawn as a receptacle") && one.contains("its shape is "), "{one}");
    let _ = std::fs::remove_dir_all(&dir);
}
