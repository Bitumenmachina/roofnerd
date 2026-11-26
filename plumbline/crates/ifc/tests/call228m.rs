//! Call 228 (contract 228M): the clashes handed over as an issue list (BCF 2.1). The job is the made-up whole house, priced in all
//! sixteen divisions (`sample::house_priced`), whose clash list says: in the same space 11 (5 findings, 6 to verify in the field),
//! set against 10, through a wall or a floor 47 holes. Figures worked by hand in STATUS.md ("Clashes handed over as an issue list").

use std::io::Read;

use plumbline_ifc::bcf::{self, Sort};
use plumbline_ifc::write;
use plumbline_world::building::clash;
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-11-26T09:30:00";
const OTHER: &str = "2027-02-01T17:05:09";

fn files(bytes: &[u8]) -> Vec<(String, String)> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("a zip");
    let mut out = Vec::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        out.push((f.name().unwrap().to_string(), s));
    }
    out
}

/// What a STEP file makes of a name: anything outside the plain characters by its number.
fn step_text(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        match c {
            '\'' => out.push_str("''"),
            '\\' => out.push_str("\\\\"),
            c if (' '..='~').contains(&c) => out.push(c),
            c => {
                let mut u = [0u16; 2];
                out.push_str("\\X2\\");
                for x in c.encode_utf16(&mut u) {
                    out.push_str(&format!("{x:04X}"));
                }
                out.push_str("\\X0\\");
            }
        }
    }
    out.push('\'');
    out
}

#[test]
fn eleven_issues_in_the_clash_lists_order_five_findings_and_six_questions() {
    let mut job = sample::house_priced();
    let found = clash::clashes(&mut job, None).unwrap();
    let list = found.in_the_same_space();
    assert_eq!(list.len(), 11);
    let w = bcf::export(&mut job, false, WHEN).unwrap();
    assert_eq!(w.issues.len(), 11);
    assert_eq!((w.findings, w.questions, w.holes), (5, 6, 0));
    assert_eq!(w.set_against, 10);
    for (issue, c) in w.issues.iter().zip(&list) {
        assert_eq!(issue.sort, Sort::Clash);
        // The same certainty the clash list gives: never raised.
        assert_eq!(issue.finding, c.status == Status::Known, "{}", issue.title);
        assert_eq!(issue.things.iter().map(|t| t.path.as_str()).collect::<Vec<_>>(), vec![c.a.as_str(), c.b.as_str()]);
        assert_eq!(issue.at, c.at);
    }
    assert_eq!(w.issues[0].title, "first-floor trunk, west, piece 2 of 3 and soil stack: in each other by 3.50 in");
    // Each says whether it is a finding or a question for the field, in those words.
    for i in &w.issues {
        if i.finding {
            assert!(i.description.contains("A finding"), "{}", i.description);
            assert!(!i.description.contains("question for the field"), "{}", i.description);
        } else {
            assert!(i.description.contains("A question for the field"), "{}", i.description);
        }
    }
    // The file: the version, and a folder for each issue with its markup and its viewpoint.
    let f = files(&w.bytes);
    assert_eq!(f[0].0, "bcf.version");
    assert!(f[0].1.contains("VersionId=\"2.1\""));
    assert_eq!(f.len(), 1 + 2 * 11);
    for i in &w.issues {
        assert!(f.iter().any(|(n, _)| *n == format!("{}/markup.bcf", i.topic)));
        assert!(f.iter().any(|(n, _)| *n == format!("{}/viewpoint.bcfv", i.topic)));
    }
}

#[test]
fn every_id_in_the_issue_list_is_the_id_the_ifc_file_gives_a_thing_of_the_same_name() {
    let mut job = sample::house_priced();
    let ifc = write::export(&mut job, "house.ifc", WHEN).unwrap().text;
    let title = write::job_title(&mut job);
    let w = bcf::export(&mut job, true, WHEN).unwrap();
    let (placed, _) = plumbline_world::building::placements(&mut job);
    let mut said_otherwise = 0;
    let mut checked = 0;
    for i in &w.issues {
        for t in &i.things {
            assert_eq!(t.id, write::guid(&title, &t.path), "made by the one function");
            let line = ifc.lines().find(|l| l.contains(&format!("('{}',", t.id))).unwrap_or_else(|| panic!("{} [{}] is not in the BIM file as {}", t.name, t.path, t.id));
            // The BIM file names a thing by its own name; the clash list says a placed thing with its size and its type, so the name
            // that is held to the file's is the job's own for that path.
            let own = &placed.iter().find(|p| p.path == t.path).unwrap().name;
            assert!(line.contains(&step_text(own)), "{} is the id of another thing: {line}", t.id);
            if *own != t.name {
                said_otherwise += 1;
            }
            checked += 1;
        }
    }
    assert!(checked >= 22 + 47, "{checked}");
    eprintln!("{checked} ids checked; {said_otherwise} of them are said in the clash list with a size or a type the file's name does not have");
    // And the file says them: each markup's viewpoint selects them by those ids.
    for (name, text) in files(&w.bytes) {
        if name.ends_with(".bcfv") {
            assert_eq!(text.matches("IfcGuid=\"").count(), w.issues.iter().find(|i| name.starts_with(&i.topic)).unwrap().things.len(), "{name}");
        }
    }
    for (name, text) in files(&w.bytes) {
        if name.ends_with("markup.bcf") {
            assert!(text.contains(&format!("IfcProject=\"{}\"", write::guid(&title, "project"))), "{name}");
        }
    }
}

#[test]
fn the_view_looks_at_the_place_from_outside_both_things_with_both_in_it() {
    let mut job = sample::house_priced();
    let (placed, _) = plumbline_world::building::placements(&mut job);
    let w = bcf::export(&mut job, true, WHEN).unwrap();
    assert!(w.issues.len() > 11);
    for i in &w.issues {
        // Metres, the model's axes (east, north, up): the target is the clash's place.
        let target = [i.at[0] * 0.3048, i.at[1] * 0.3048, i.at[2] * 0.3048];
        let len = |v: [f64; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len(i.direction) - 1.0).abs() < 1e-9 && (len(i.up) - 1.0).abs() < 1e-9, "{}", i.title);
        assert!((i.direction[0] * i.up[0] + i.direction[1] * i.up[1] + i.direction[2] * i.up[2]).abs() < 1e-9);
        // The eye looks at the place: the place is on the line of sight, within a foot (0.3048 m).
        let to = [target[0] - i.eye[0], target[1] - i.eye[1], target[2] - i.eye[2]];
        let along = to[0] * i.direction[0] + to[1] * i.direction[1] + to[2] * i.direction[2];
        assert!(along > 0.0);
        let off = [to[0] - along * i.direction[0], to[1] - along * i.direction[1], to[2] - along * i.direction[2]];
        assert!(len(off) < 0.3048, "{}: the place is {} m off the line of sight", i.title, len(off));
        for t in &i.things {
            let p = placed.iter().find(|p| p.path == t.path).unwrap();
            let (lo, hi) = clash::extent(p).unwrap();
            let feet = [i.eye[0] / 0.3048, i.eye[1] / 0.3048, i.eye[2] / 0.3048];
            let inside = (0..3).all(|k| feet[k] >= lo[k] && feet[k] <= hi[k]);
            assert!(!inside, "{}: the eye is inside {}", i.title, t.name);
            // The part of the thing nearest the place is inside the view (half of 60 degrees each side).
            let near: Vec<f64> = (0..3).map(|k| i.at[k].clamp(lo[k], hi[k]) * 0.3048 - i.eye[k]).collect();
            let a = near[0] * i.direction[0] + near[1] * i.direction[1] + near[2] * i.direction[2];
            let n = (near[0] * near[0] + near[1] * near[1] + near[2] * near[2]).sqrt();
            assert!((a / n).clamp(-1.0, 1.0).acos().to_degrees() < 30.0, "{}: {} is out of the view", i.title, t.name);
        }
    }
}

#[test]
fn holes_go_out_only_when_asked_each_of_its_own_kind() {
    let mut job = sample::house_priced();
    let plain = bcf::export(&mut job, false, WHEN).unwrap();
    assert!(plain.issues.iter().all(|i| i.sort == Sort::Clash));
    let w = bcf::export(&mut job, true, WHEN).unwrap();
    assert_eq!(w.issues.len(), 11 + 47);
    assert_eq!(w.issues.iter().filter(|i| i.sort == Sort::Hole).count(), 47);
    assert_eq!(w.holes, 47);
    // What is set against never goes out: no issue is one of the ten.
    let found = clash::clashes(&mut job, None).unwrap();
    for c in found.set_against() {
        assert!(!w.issues.iter().any(|i| i.things.iter().map(|t| t.path.as_str()).collect::<Vec<_>>() == vec![c.a.as_str(), c.b.as_str()]));
    }
    let hole = w.issues.iter().find(|i| i.sort == Sort::Hole).unwrap();
    assert!(hole.description.contains("A hole to cut"), "{}", hole.description);
    let f = files(&w.bytes);
    assert!(f.iter().any(|(_, t)| t.contains("TopicType=\"Hole to cut\"")));
    assert!(f.iter().any(|(_, t)| t.contains("TopicType=\"Clash\"")));
}

#[test]
fn written_twice_the_two_differ_in_the_time_and_nothing_else() {
    let mut job = sample::house_priced();
    let a = files(&bcf::export(&mut job, true, WHEN).unwrap().bytes);
    let b = files(&bcf::export(&mut job, true, OTHER).unwrap().bytes);
    assert_eq!(a.len(), b.len());
    for ((na, ta), (nb, tb)) in a.iter().zip(&b) {
        assert_eq!(na, nb);
        assert_eq!(ta.replace(WHEN, "T").replace("2026-11-26T09:30:00Z", "T"), tb.replace(OTHER, "T").replace("2027-02-01T17:05:09Z", "T"), "{na}");
    }
    assert!(a.iter().any(|(_, t)| t.contains(WHEN)));
    // The author is the program, not a person.
    assert!(a.iter().any(|(_, t)| t.contains("<CreationAuthor>BuildOS</CreationAuthor>")));
    // Written again at the same time it is the same bytes.
    assert_eq!(bcf::export(&mut job, true, WHEN).unwrap().bytes, bcf::export(&mut sample::house_priced(), true, WHEN).unwrap().bytes);
}

#[test]
fn no_machine_or_person_or_path_and_no_decimal_feet_in_what_is_read() {
    let mut job = sample::house_priced();
    let w = bcf::export(&mut job, true, WHEN).unwrap();
    for (name, text) in files(&w.bytes) {
        for bad in ["/home", "/tmp", "C:\\", "roofnerd", "root"] {
            assert!(!text.contains(bad), "{name} has {bad:?}");
        }
    }
    for i in &w.issues {
        // "10 ft", never "10.00 ft"; and no "ft" after a decimal.
        let b = i.description.as_bytes();
        for k in 0..b.len().saturating_sub(5) {
            if b[k].is_ascii_digit() && b[k + 1] == b'.' && b[k + 2].is_ascii_digit() {
                let rest = &i.description[k + 1..];
                let digits = rest[1..].chars().take_while(|c| c.is_ascii_digit()).count();
                assert!(!rest[1 + digits..].starts_with(" ft"), "decimal feet in {}", i.description);
            }
        }
        assert!(i.description.contains(" ft") || i.description.contains(" in"), "{}", i.description);
    }
    let first = &w.issues[0].description;
    assert!(first.contains("18 ft 6 in east"), "{first}");
    assert!(first.contains("22 ft north"), "{first}");
}

#[test]
fn what_rides_on_a_question_is_what_the_verify_in_field_list_says() {
    let mut job = sample::house_priced();
    let list = plumbline_world::vif::vif(&mut job);
    let w = bcf::export(&mut job, false, WHEN).unwrap();
    let mut with = 0;
    for i in &w.issues {
        let q = list.questions.iter().find(|q| q.clash.as_ref().is_some_and(|b| !b.through && q.path == i.things[0].path && b.other == i.things[1].path && b.at == i.at));
        match (i.finding, q) {
            (true, q) => assert!(q.is_none(), "a finding is on the verify-in-field list"),
            (false, None) => panic!("a question the list does not have: {}", i.title),
            (false, Some(q)) => match q.dollars {
                Some(x) => {
                    with += 1;
                    assert!(i.description.contains(&format!("${} rides on it", view::figure(x, 2))), "{}", i.description);
                }
                None => assert!(!i.description.contains(" rides on it") || i.description.contains("Nothing priced rides on it"), "{}", i.description),
            },
        }
    }
    assert!(with > 0, "none of the six has dollars: the figure is not tested");
}

#[test]
fn a_job_with_no_clash_gives_a_file_with_no_issue() {
    let mut job = sample::sample();
    let found = clash::clashes(&mut job, None);
    let found = found.unwrap();
    assert_eq!(found.in_the_same_space().len(), 0, "the sample is meant to have none");
    let w = bcf::export(&mut job, false, WHEN).unwrap();
    assert_eq!(w.issues.len(), 0);
    let f = files(&w.bytes);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].0, "bcf.version");
}
