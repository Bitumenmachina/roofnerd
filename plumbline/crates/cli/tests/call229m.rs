//! Call 229 at the command line (contract 229M): `plumbline export gltf`. The job is the made-up whole house (`plumbline sixteen`): 236
//! things with a body or a line in its BIM file (20 walls, 4 footings, 9 doors, 17 windows, 2 floors, 10 rooms, 4 roof planes, 16 cables,
//! 2 conduits, 23 ducts, 63 pipes, 39 fittings, 22 fixtures, 5 pieces of equipment), and 2 fittings with no size that have none.

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
    let dir = std::env::temp_dir().join(format!("plumbline-229-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn u32_at(b: &[u8], at: usize) -> usize {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize
}

#[test]
fn the_house_is_written_as_one_glb_and_the_reply_says_what_it_made_and_what_it_left() {
    let dir = folder("write");
    ok(&dir, &["sixteen", "house.json"]);
    let said = ok(&dir, &["export", "gltf", "house.json", "house.glb"]);
    assert!(said.contains("as 236 parts in one glTF file, in metres"), "{said}");
    assert!(said.contains("written: 16 cables, 2 conduits, 9 doors, 23 ducts, 39 fittings, 22 fixtures, 2 floors, 4 footings, 5 pieces of equipment, 63 pipes, 4 roof planes, 1 roof, 10 rooms, 20 walls, 17 windows"), "{said}");
    assert!(said.contains("not carried:"));
    for needed in ["prices, and how sure each value is", "the patterns the window draws (lap siding, brick, shingles)", "what a thing set down is drawn as in the window", "hot to the kitchen sink, elbow 1: it has no shape", "a roof plane has no thickness"] {
        assert!(said.contains(needed), "{needed}: {said}");
    }
    // A builder's words: the BIM file, not a software term.
    for bad in ["IFC file", "identifier", "vertex", "triangle", "buffer", "accessor"] {
        assert!(!said.contains(bad), "{bad}: {said}");
    }
    let b = std::fs::read(dir.join("house.glb")).unwrap();
    assert_eq!(&b[0..4], b"glTF");
    assert_eq!(u32_at(&b, 8), b.len());
    assert_eq!(&b[16..20], b"JSON");
    let json: serde_json::Value = serde_json::from_slice(&b[20..20 + u32_at(&b, 12)]).unwrap();
    assert_eq!(json["nodes"].as_array().unwrap().iter().filter(|n| n.get("mesh").is_some()).count(), 236);
    // No path of this machine is in the file.
    let dir_text = dir.to_string_lossy().to_string();
    assert!(!String::from_utf8_lossy(&b).contains(&dir_text));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn it_changes_nothing_of_the_job_or_the_bim_file() {
    let dir = folder("same");
    ok(&dir, &["sixteen", "house.json"]);
    let before = std::fs::read(dir.join("house.json")).unwrap();
    ok(&dir, &["export", "ifc", "house.json", "house.ifc"]);
    let first = std::fs::read_to_string(dir.join("house.ifc")).unwrap();
    ok(&dir, &["export", "gltf", "house.json", "house.glb"]);
    ok(&dir, &["export", "gltf", "house.json", "again.glb"]);
    assert_eq!(std::fs::read(dir.join("house.glb")).unwrap(), std::fs::read(dir.join("again.glb")).unwrap(), "the same job, the same file");
    assert_eq!(std::fs::read(dir.join("house.json")).unwrap(), before, "the job is not touched");
    ok(&dir, &["export", "ifc", "house.json", "house.ifc"]);
    let second = std::fs::read_to_string(dir.join("house.ifc")).unwrap();
    let body = |t: &str| t.lines().filter(|l| !l.starts_with("FILE_NAME")).collect::<Vec<_>>().join("\n");
    assert_eq!(body(&first), body(&second), "the BIM file differs only in its time line");
    // And the sample whole house is the house it was: the made-up job's own sum begins 880d9fd3c364b5f7.
    let sum = Command::new("sha256sum").arg(dir.join("house.json")).output().unwrap();
    assert!(String::from_utf8_lossy(&sum.stdout).starts_with("880d9fd3c364b5f7"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_wrong_file_name_a_flag_and_a_job_with_no_level_are_refused_in_plain_words() {
    let dir = folder("refuse");
    ok(&dir, &["sixteen", "house.json"]);
    let said = refused(&dir, &["export", "gltf", "house.json", "house.gltf"]);
    assert!(said.contains("the house is written as a .glb file; name the file with that"), "{said}");
    assert!(!dir.join("house.gltf").exists());
    let said = refused(&dir, &["export", "gltf", "house.json", "house.glb", "--holes"]);
    assert!(!said.is_empty());
    assert!(!dir.join("house.glb").exists(), "nothing is written for a refused call");
    ok(&dir, &["new", "empty.json", "Empty (not a real job)"]);
    let said = refused(&dir, &["export", "gltf", "empty.json", "empty.glb"]);
    assert!(said.contains("no level"), "{said}");
    assert!(!dir.join("empty.glb").exists());
    // The list of what can be written out names it, and the usage says what it is.
    let said = refused(&dir, &["export", "dwg", "house.json", "x.dwg"]);
    assert!(said.contains("ifc, gltf or bcf"), "{said}");
    assert!(said.contains("plumbline export gltf <file> <out.glb>"), "{said}");
    let _ = std::fs::remove_dir_all(&dir);
}
