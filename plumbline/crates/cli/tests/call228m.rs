//! Call 228 at the command line (contract 228M): `plumbline export bcf`. The job is the made-up whole house (`plumbline sixteen`):
//! 11 clashes in the same space (5 findings, 6 to verify in the field), 10 set against, 47 holes to cut.

use std::io::Read;
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
    let dir = std::env::temp_dir().join(format!("plumbline-228-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn files(path: &std::path::Path) -> Vec<(String, String)> {
    let mut z = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).expect("a zip");
    (0..z.len())
        .map(|i| {
            let mut f = z.by_index(i).unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            (f.name().unwrap().to_string(), s)
        })
        .collect()
}

#[test]
fn the_clashes_are_written_as_an_issue_list_and_the_reply_says_what_went_and_what_did_not() {
    let dir = folder("list");
    ok(&dir, &["sixteen", "h.json"]);
    let said = ok(&dir, &["export", "bcf", "h.json", "h.bcfzip"]);
    assert!(said.contains("11 issues from the clashes of h.json"), "{said}");
    assert!(said.contains("in the same space: 11 (5 findings, 6 to verify in the field)"), "{said}");
    assert!(said.contains("not handed over: 47 holes to cut"), "{said}");
    assert!(said.contains("not handed over: 10 set against a wall"), "{said}");
    assert!(said.contains("not carried: a picture of the place"), "{said}");
    let f = files(&dir.join("h.bcfzip"));
    assert_eq!(f.len(), 1 + 2 * 11);
    assert_eq!(f.iter().filter(|(n, _)| n.ends_with("/markup.bcf")).count(), 11);
    // The first is the first-floor trunk and the soil stack, as the clash list writes it.
    let list = ok(&dir, &["clashes", "h.json"]);
    assert!(list.contains("first-floor trunk, west, piece 2 of 3 [duct-0100] and soil stack [pipe-0246]: in each other by 3.50 in"));
    assert!(f[1].1.contains("<Title>first-floor trunk, west, piece 2 of 3 and soil stack: in each other by 3.50 in</Title>"), "{}", f[1].1);
    // 5 findings and 6 questions.
    let markups: Vec<&String> = f.iter().filter(|(n, _)| n.ends_with("markup.bcf")).map(|(_, t)| t).collect();
    assert_eq!(markups.iter().filter(|t| t.contains("<Labels>Finding</Labels>")).count(), 5);
    assert_eq!(markups.iter().filter(|t| t.contains("<Labels>To verify in the field</Labels>")).count(), 6);
    // The ids are the BIM file's.
    ok(&dir, &["export", "ifc", "h.json", "h.ifc"]);
    let ifc = std::fs::read_to_string(dir.join("h.ifc")).unwrap();
    let mut ids = 0;
    for (_, t) in f.iter().filter(|(n, _)| n.ends_with(".bcfv")) {
        for part in t.split("IfcGuid=\"").skip(1) {
            let id = &part[..22];
            assert!(ifc.contains(&format!("('{id}',")), "{id} is not in the BIM file");
            ids += 1;
        }
    }
    assert_eq!(ids, 22);
}

#[test]
fn holes_only_when_asked_and_the_file_is_never_written_over_a_job_nor_under_another_name() {
    let dir = folder("holes");
    ok(&dir, &["sixteen", "h.json"]);
    let said = ok(&dir, &["export", "bcf", "h.json", "h.bcfzip", "--holes"]);
    assert!(said.contains("58 issues"), "{said}");
    assert!(said.contains("and 47 holes to cut"), "{said}");
    assert!(!said.contains("not handed over: 47 holes"), "{said}");
    let f = files(&dir.join("h.bcfzip"));
    assert_eq!(f.iter().filter(|(_, t)| t.contains("TopicType=\"Hole to cut\"")).count(), 47);
    let bad = refused(&dir, &["export", "bcf", "h.json", "h.json"]);
    assert!(bad.contains("is the job itself"), "{bad}");
    let bad = refused(&dir, &["export", "bcf", "h.json", "h.zip"]);
    assert!(bad.contains(".bcfzip"), "{bad}");
    let bad = refused(&dir, &["export", "bcf", "h.json", "h.bcfzip", "--add"]);
    assert!(bad.contains("does not take --add"), "{bad}");
}

#[test]
fn a_job_with_no_clash_is_written_with_no_issue_and_says_so() {
    let dir = folder("none");
    ok(&dir, &["sample", "s.json"]);
    let said = ok(&dir, &["export", "bcf", "s.json", "s.bcfzip"]);
    assert!(said.contains("there was no clash in the same space to hand over"), "{said}");
    let f = files(&dir.join("s.bcfzip"));
    assert_eq!(f.len(), 1);
}

#[test]
fn the_sample_house_does_not_move() {
    let dir = folder("same");
    ok(&dir, &["sixteen", "h.json"]);
    let before = std::fs::read(dir.join("h.json")).unwrap();
    let list = ok(&dir, &["clashes", "h.json"]);
    ok(&dir, &["export", "bcf", "h.json", "h.bcfzip", "--holes"]);
    assert_eq!(std::fs::read(dir.join("h.json")).unwrap(), before);
    assert_eq!(ok(&dir, &["clashes", "h.json"]), list);
}
