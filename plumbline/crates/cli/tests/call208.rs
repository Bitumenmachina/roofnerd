//! Call 208 point 1: a price has one road, whoever sets it. `set <material> price` and `quote` go by the road a price list
//! brought in takes, in the three cases that call 207 left on an older one. The job is made up; the figures are worked by hand:
//! a wall 20 ft by 8 ft is 160 SF.

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

const K: [&str; 4] = ["--status", "Known", "--source", "made-up plan"];
const A: [&str; 4] = ["--status", "Assumed", "--source", "made-up quote"];

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("plumbline-208-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A job with one wall 20 ft by 8 ft (160 SF) and the material "Siding (made up)" (material-0005) with a line made of it.
fn job(dir: &std::path::Path) {
    ok(dir, &["new", "j.json", "Prices (not a real job)"]);
    ok(dir, &[&["level", "j.json", "ground", "--elevation", "0 ft", "--height", "9 ft"][..], &K[..]].concat());
    ok(dir, &[&["type", "j.json", "wall", "w6", "--thickness", "6 in"][..], &K[..]].concat());
    ok(dir, &[&["wall", "j.json", "level-0002", "type-0003", "--points", "0,0 20,0", "--height", "8 ft"][..], &K[..]].concat());
    ok(dir, &["material", "j.json", "Siding (made up)", "SF"]);
    ok(dir, &[&["set", "j.json", "wall-0004", "material", "Siding (made up)"][..], &A[..]].concat());
    ok(dir, &["line", "j.json", "wall-0004", "Siding (made up)", "face_area", "--cost-type", "material", "--division", "7", "--of-material"]);
}

fn set(dir: &std::path::Path, args: &[&str]) -> (bool, String) {
    run(dir, &[&["set", "j.json"][..], args, &A[..]].concat())
}

/// (a) A supplier's first price on a material the job has no price for is the job's price too, as a list brings it: 160 SF at $5.00 = $800.00,
/// and the reply says the job takes it.
#[test]
fn a_suppliers_price_on_a_material_with_no_price_of_the_jobs_is_the_jobs_too() {
    let dir = folder("a");
    job(&dir);
    let (good, said) = set(&dir, &["material-0005", "quote", "$5.00/SF", "--supplier", "Supply One"]);
    assert!(good, "{said}");
    assert!(said.contains("the job had no price for it and takes Supply One's"), "{said}");
    let list = ok(&dir, &["price", "j.json", "list"]);
    assert!(list.contains("estimator   ~ $5.00/SF") || list.split_whitespace().collect::<Vec<_>>().join(" ").contains("estimator ~ $5.00/SF"), "{list}");
    assert!(ok(&dir, &["estimate", "j.json"]).contains("~$800.00"), "the line is priced: 160 SF x $5.00");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// (b) A supplier written another way is the job's own spelling, and the reply says so; no second supplier. 160 SF at $4.00 = $640.00 at Supply One.
#[test]
fn a_suppliers_name_written_another_way_is_taken_as_the_jobs_and_said() {
    let dir = folder("b");
    job(&dir);
    assert!(set(&dir, &["material-0005", "price", "$6.00/SF"]).0);
    assert!(set(&dir, &["material-0005", "quote", "$5.00/SF", "--supplier", "Supply One"]).0);
    let (good, said) = set(&dir, &["material-0005", "quote", "$4.00/SF", "--supplier", "supply  one"]);
    assert!(good, "{said}");
    assert!(said.contains("\"supply  one\"") && said.contains("\"Supply One\""), "the reply says which spelling was taken: {said}");
    let list = ok(&dir, &["price", "j.json", "list"]);
    let flat = list.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains("Supply One ~ $4.00/SF") && !flat.contains("supply  one"), "{list}");
    assert!(said.contains("$640.00"), "160 SF x $4.00 at Supply One: {said}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// (c) An unpriced line of the same name that is not made of the material is priced by it, and the reply names the line.
/// Two lines of 160 SF at $6.00 = $960.00 each, $1,920.00.
#[test]
fn an_unpriced_line_of_the_same_name_is_priced_and_named() {
    let dir = folder("c");
    job(&dir);
    ok(&dir, &[&["wall", "j.json", "level-0002", "type-0003", "--points", "0,10 20,10", "--height", "8 ft"][..], &K[..]].concat());
    ok(&dir, &["line", "j.json", "wall-0006", "Siding (made up)", "face_area", "--cost-type", "material", "--division", "7"]);
    let (good, said) = set(&dir, &["material-0005", "price", "$6.00/SF"]);
    assert!(good, "{said}");
    assert!(said.contains("\"Siding (made up)\" on wall 2: priced at $6.00/SF"), "{said}");
    assert!(ok(&dir, &["estimate", "j.json"]).contains("~$1,920.00"), "two lines of 160 SF at $6.00");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Point 3: `check` of a job that holds a shape under a starter shape's name says it does, and what to do. The job still opens.
#[test]
fn check_says_of_a_job_with_a_shape_under_a_starter_shapes_name_that_it_does() {
    let dir = folder("check");
    ok(&dir, &["new", "s.json", "Shapes (not a real job)"]);
    std::fs::write(dir.join("sh.csv"), "Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\nzzmine,timber,box,-width / 2,width / 2,0,depth,0,height,\n").unwrap();
    ok(&dir, &["import", "shapes", "s.json", "sh.csv"]);
    let plain = ok(&dir, &["check", "s.json"]);
    assert!(!plain.contains("starter shape's name"), "a shape of its own name is nothing to say: {plain}");
    // As a saved job of the hours between two commits of 2026-11-24 holds it: the name is a starter's.
    let text = std::fs::read_to_string(dir.join("s.json")).unwrap().replace("\"zzmine\"", "\"Bathtub\"");
    std::fs::write(dir.join("s.json"), text).unwrap();
    let said = ok(&dir, &["check", "s.json"]);
    assert!(said.contains("reads, its figures agree") && said.contains("! \"Bathtub\" is a starter shape's name") && said.contains("keep it under another name"), "{said}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A line with four formulas, each with a length typed in: stretch-out, spacing, turn-up and a plain number.
fn lengths(dir: &std::path::Path) {
    ok(dir, &["new", "j.json", "Lengths (not a real job)"]);
    ok(dir, &[&["level", "j.json", "ground", "--elevation", "0 ft", "--height", "9 ft"][..], &K[..]].concat());
    ok(dir, &[&["type", "j.json", "wall", "w6", "--thickness", "6 in"][..], &K[..]].concat());
    ok(dir, &[&["wall", "j.json", "level-0002", "type-0003", "--points", "0,0 20,0", "--height", "8 ft"][..], &K[..]].concat());
    for n in ["Cap", "Studs", "Turn", "Plain"] {
        ok(dir, &["line", "j.json", "wall-0004", n, "plan_length", "--cost-type", "material", "--division", "6"]);
    }
    for (line, formula, with) in [("Cap", "plan_length * stretch_out", "stretch_out=19 in"), ("Studs", "plan_length / spacing", "spacing=16 in"), ("Turn", "plan_length * turn_up", "turn_up=8 in"), ("Plain", "plan_length * lap", "lap=2")] {
        ok(dir, &[&["set", "j.json", "wall-0004", "quantity", formula, "--line", line, "--with", with][..], &K[..]].concat());
    }
}

/// Point 4: any length a line enters is refused below nothing: a stretch-out, a height, a spacing, a turn-up, whether set on its own or given
/// with a new formula, and the job is as it was. A plain number (a count of laps) is not a length, and is the estimator's to enter as it is.
#[test]
fn a_length_a_line_enters_below_nothing_is_refused_in_a_builders_words() {
    let dir = folder("lengths");
    lengths(&dir);
    let before = std::fs::read(dir.join("j.json")).unwrap();
    for (args, must) in [
        (vec!["stretch_out", "-19 in", "--line", "Cap"], "stretch out is below nothing (-19 in): a length a line enters is nothing or more"),
        (vec!["spacing", "-12 in", "--line", "Studs"], "spacing is below nothing (-12 in): a length a line enters is nothing or more"),
        (vec!["turn_up", "-8 in", "--line", "Turn"], "turn up is below nothing (-8 in): a length a line enters is nothing or more"),
        (vec!["quantity", "plan_length / spacing", "--line", "Studs", "--with", "spacing=-16 in"], "spacing is below nothing (-16 in)"),
        (vec!["quantity", "plan_length * turn_up", "--line", "Turn", "--with", "turn_up=-1 ft"], "turn up is below nothing (-1 ft)"),
        (vec!["wainscot", "-1 ft", "--line", "Cap"], "wainscot is below nothing (-1 ft): a height a line enters is nothing or more"),
    ] {
        let (good, said) = set(&dir, &[&["wall-0004"][..], &args[..]].concat());
        assert!(!good && said.contains(must), "{args:?}: {said}");
        assert_eq!(std::fs::read(dir.join("j.json")).unwrap(), before, "{args:?}: a refused change altered the job");
    }
    // Nothing is the least a length can be: taken, and a spacing of nothing is refused where it divides.
    let (good, said) = set(&dir, &["wall-0004", "turn_up", "0 in", "--line", "Turn"]);
    assert!(good, "{said}");
    // A count typed plain is not a length.
    assert!(set(&dir, &["wall-0004", "lap", "3", "--line", "Plain"]).0);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A price set for a name no line is made of, where a line OF THAT NAME is priced by it, says that and not that it prices nothing:
/// "Pitch pan" is a material, nothing is made of it, and two lines named "Pitch pan" (one on each wall) take its $12.00.
#[test]
fn a_price_for_a_name_no_line_is_made_of_but_lines_are_named_says_which_it_prices() {
    let dir = folder("named");
    job(&dir);
    ok(&dir, &[&["wall", "j.json", "level-0002", "type-0003", "--points", "0,10 20,10", "--height", "8 ft"][..], &K[..]].concat());
    ok(&dir, &["material", "j.json", "Pitch pan (made up)", "EA"]);
    for w in ["wall-0004", "wall-0006"] {
        ok(&dir, &["line", "j.json", w, "Pitch pan (made up)", "4", "--cost-type", "material", "--division", "7"]);
    }
    let (good, said) = set(&dir, &["Pitch pan (made up)", "price", "$12.00"]);
    assert!(good, "{said}");
    assert!(!said.contains("prices nothing on the job yet"), "{said}");
    assert!(said.contains("no line is made of Pitch pan (made up), and 2 lines of that name are priced by it: \"Pitch pan (made up)\" on wall 1, \"Pitch pan (made up)\" on wall 2"), "{said}");
    // And a price that prices nothing at all still says so.
    ok(&dir, &["material", "j.json", "Zinc (made up)", "EA"]);
    let (good, said) = set(&dir, &["Zinc (made up)", "price", "$3.00"]);
    assert!(good && said.contains("Zinc (made up) prices nothing on the job yet: no line is made of it"), "{said}");
    std::fs::remove_dir_all(&dir).unwrap();
}
