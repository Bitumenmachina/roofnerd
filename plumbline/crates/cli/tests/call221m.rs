//! Call 221 at the command line (contract 221M): a CAD drawing (DXF) brought in as a sheet, its layers, written out, and the saved job.
//! Everything is made up; the figures are worked by hand in STATUS.md ("A CAD drawing in and out") and the three where the lead's
//! figures were not right are said in the contract's report: 10 wall lines (a door cuts both faces), 224.25 SF (inside the faces), and
//! 20 ft 6 in by 12 ft 6 in (the walls' lines are their centers).

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
    let dir = std::env::temp_dir().join(format!("plumbline-221-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const K: [&str; 4] = ["--status", "Known", "--source", "made up"];
const A: [&str; 4] = ["--status", "Assumed", "--source", "the drawing"];

fn with(args: &[&str], more: &[&str]) -> Vec<String> {
    args.iter().chain(more.iter()).map(|s| s.to_string()).collect()
}

fn okv(dir: &std::path::Path, args: Vec<String>) -> String {
    ok(dir, &args.iter().map(String::as_str).collect::<Vec<_>>())
}

/// A DXF of one rectangle 120 by 60 on WALLS and a circle on TREES, in the unit asked (`None`: no unit said).
fn drawing(unit: Option<i32>, frozen: bool) -> String {
    let mut s = String::new();
    let mut p = |c: i32, v: &str| s.push_str(&format!("{c}\n{v}\n"));
    p(0, "SECTION");
    p(2, "HEADER");
    if let Some(u) = unit {
        p(9, "$INSUNITS");
        p(70, &u.to_string());
    }
    p(0, "ENDSEC");
    p(0, "SECTION");
    p(2, "TABLES");
    p(0, "TABLE");
    p(2, "LAYER");
    for (n, f) in [("WALLS", 0), ("TREES", i32::from(frozen))] {
        p(0, "LAYER");
        p(2, n);
        p(70, &f.to_string());
    }
    p(0, "ENDTAB");
    p(0, "ENDSEC");
    p(0, "SECTION");
    p(2, "ENTITIES");
    for ([a, b], [c, d]) in [([0, 0], [120, 0]), ([120, 0], [120, 60]), ([120, 60], [0, 60]), ([0, 60], [0, 0])] {
        p(0, "LINE");
        p(8, "WALLS");
        p(10, &a.to_string());
        p(20, &b.to_string());
        p(11, &c.to_string());
        p(21, &d.to_string());
    }
    p(0, "CIRCLE");
    p(8, "TREES");
    p(10, "60");
    p(20, "30");
    p(40, "10");
    p(0, "ENDSEC");
    p(0, "EOF");
    s
}

fn start(name: &str) -> std::path::PathBuf {
    let dir = folder(name);
    ok(&dir, &["new", "j.json", "A made-up room"]);
    okv(&dir, with(&["level", "j.json", "Ground", "--elevation", "0 ft", "--height", "9 ft"], &K));
    dir
}

#[test]
fn a_drawing_in_feet_is_at_true_size_and_a_frozen_layer_comes_in_off() {
    let dir = start("feet");
    // 120 by 60 in feet is 120 ft by 60 ft; the layer the file froze is off as the file has it.
    std::fs::write(dir.join("room.dxf"), drawing(Some(2), true)).unwrap();
    let said = okv(&dir, with(&["drawing", "j.json", "level-0002", "Room", "--file", "room.dxf"], &A));
    assert!(said.contains("drawn in feet, 120 by 60 feet"), "{said}");
    assert!(said.contains("it covers 120' 0\" by 60' 0\""), "{said}");
    assert!(said.contains("its scale is Assumed, from the file's own unit (feet"), "{said}");
    let layers = ok(&dir, &["layers", "j.json", "drawing-0003"]);
    assert!(layers.contains("off TREES: 1 circle") && layers.contains("on  WALLS: 4 lines"), "{layers}");
    // 4 lines to snap to, the circle's pieces left off with its layer.
    assert!(layers.contains("4 straight pieces to snap to with these layers"), "{layers}");
    let job = std::fs::read_to_string(dir.join("j.json")).unwrap();
    assert!(job.contains("\"layers_off\"") && job.contains("TREES"), "the layer switched off is kept in the job");
    assert!(ok(&dir, &["check", "j.json"]).contains("saving it back writes the same file"));
    // Switched on, the circle's 36 pieces are among the lines.
    let on = ok(&dir, &["layer", "j.json", "drawing-0003", "trees", "on"]);
    assert!(on.contains("40 straight pieces to snap to now (4 before)"), "{on}");
    let job = std::fs::read_to_string(dir.join("j.json")).unwrap();
    assert!(!job.contains("layers_off"), "no layer off: the key is not written");
}

#[test]
fn a_millimeter_drawing_scales_from_its_unit_and_a_pdf_has_no_layers_key() {
    let dir = start("mm");
    // 120 mm is 4.7244 in, 4 3/4 in to the sixteenth, and 60 mm is 2.3622 in, 2 3/8 in: worked from 25.4 mm to the inch, not guessed.
    std::fs::write(dir.join("room.dxf"), drawing(Some(4), false)).unwrap();
    let said = okv(&dir, with(&["drawing", "j.json", "level-0002", "Room", "--file", "room.dxf"], &A));
    assert!(said.contains("drawn in millimeters"), "{said}");
    assert!(said.contains("it covers 0' 4 3/4\" by 0' 2 3/8\"") && said.contains("(millimeters, 304.8 to the foot)"), "{said}");
    // A PDF's drawing carries no such key: a job without one reads, and is written, as it was.
    okv(&dir, with(&["drawing", "j.json", "level-0002", "Plan", "--file", &format!("{}/demos/plans/made-up-plan.pdf", env!("CARGO_MANIFEST_DIR").trim_end_matches("/crates/cli"))], &A));
    let job = std::fs::read_to_string(dir.join("j.json")).unwrap();
    assert!(!job.contains("layers_off"));
    assert!(ok(&dir, &["check", "j.json"]).contains("saving it back writes the same file"));
    assert!(refused(&dir, &["layers", "j.json", "drawing-0004"]).contains("is not a CAD drawing (DXF)"));
}

#[test]
fn no_unit_means_no_scale_and_the_scale_is_asked_for_never_guessed() {
    let dir = start("none");
    std::fs::write(dir.join("room.dxf"), drawing(None, false)).unwrap();
    let said = okv(&dir, with(&["drawing", "j.json", "level-0002", "Room", "--file", "room.dxf"], &A));
    assert!(said.contains("no scale yet: the drawing does not say what it is drawn in"), "{said}");
    let (good, why) = run(&dir, &["wall", "j.json", "level-0002", "type-0003", "--points", "0,0 100,0", "--height", "9 ft", "--on", "drawing-0003"]);
    assert!(!good && why.contains("no scale"), "{why}");
}

#[test]
fn closed_formats_and_not_drawings_are_refused_and_the_job_is_not_changed() {
    let dir = start("closed");
    let before = std::fs::read_to_string(dir.join("j.json")).unwrap();
    std::fs::write(dir.join("a.dwg"), b"AC1032\0\0\0\0\0\0").unwrap();
    std::fs::write(dir.join("b.dat"), b"AC1015\0\0\0\0\0\0").unwrap();
    std::fs::write(dir.join("c.txt"), "0\nSECTION\nthis is cut").unwrap();
    for f in ["a.dwg", "b.dat"] {
        let why = refused(&dir, &with(&["drawing", "j.json", "level-0002", "X", "--file", f], &A).iter().map(String::as_str).collect::<Vec<_>>());
        assert!(why.contains("DWG") && why.contains("save it as a DXF"), "{why}");
    }
    let why = refused(&dir, &with(&["drawing", "j.json", "level-0002", "X", "--file", "c.txt"], &A).iter().map(String::as_str).collect::<Vec<_>>());
    assert!(why.contains("damaged or cut short") || why.contains("nothing"), "{why}");
    assert_eq!(std::fs::read_to_string(dir.join("j.json")).unwrap(), before);
}

#[test]
fn the_plan_of_a_level_goes_out_with_published_layer_names_and_says_what_is_not_written() {
    let dir = start("out");
    okv(&dir, with(&["type", "j.json", "wall", "Wall 6", "--thickness", "6 in"], &K));
    okv(&dir, with(&["type", "j.json", "door", "Door 3", "--width", "3 ft", "--height", "7 ft"], &K));
    okv(&dir, with(&["wall", "j.json", "level-0002", "type-0003", "--points", "0,0 20,0 20,12 0,12", "--closed", "--height", "9 ft"], &K));
    okv(&dir, with(&["door", "j.json", "wall-0005", "type-0004", "--offset", "8 ft", "--sill", "0 ft"], &K));
    okv(&dir, with(&["room", "j.json", "level-0002", "Kitchen", "--at", "10,6"], &K));
    okv(&dir, with(&["place", "j.json", "level-0002", "fixture", "Lavatory", "--at", "4,4", "--offset", "0 ft", "--size", "2,1.5,3"], &K));
    okv(&dir, with(&["place", "j.json", "level-0002", "furnishing", "Table", "--at", "14,8", "--offset", "0 ft", "--size", "4,3,2.5"], &K));
    okv(&dir, with(&["place", "j.json", "level-0002", "equipment", "Water heater", "--at", "16,3", "--offset", "0 ft", "--size", "2,2,5"], &K));
    let said = ok(&dir, &["export", "dxf", "j.json", "out.dxf", "--level", "Ground"]);
    for want in ["layer A-WALL: 10 lines drawn", "layer A-DOOR: 3 lines drawn", "layer A-AREA-IDEN: 1 room written", "layer P-FIXT:", "layer A-FURN:", "layer EQUIPMENT:", "(a plain name: no published name was found for it)", "not in this drawing: floors, roofs, footings, beams, posts"] {
        assert!(said.contains(want), "{want}: {said}");
    }
    // Written over nothing: a job is not overwritten, a name must end .dxf, and the level is asked for.
    std::fs::copy(dir.join("j.json"), dir.join("job.dxf")).unwrap();
    assert!(refused(&dir, &["export", "dxf", "j.json", "job.dxf", "--level", "Ground"]).contains("is a job"));
    assert!(refused(&dir, &["export", "dxf", "j.json", "j.json", "--level", "Ground"]).contains("a .dxf file"));
    assert!(refused(&dir, &["export", "dxf", "j.json", "out.txt", "--level", "Ground"]).contains("a .dxf file"));
    assert!(refused(&dir, &["export", "dxf", "j.json", "out2.dxf"]).contains("wants the level"));
    // Read back by the program's own reader: every layer, at true size.
    let back = ok(&dir, &["page", "out.dxf"]);
    assert!(back.contains("CAD drawing (DXF)"), "{back}");
    let made = okv(&dir, with(&["drawing", "j.json", "level-0002", "Back", "--file", "out.dxf"], &A));
    let sheet = made.split(':').next().unwrap().to_string();
    let layers = ok(&dir, &["layers", "j.json", &sheet]);
    for want in ["A-WALL: 10 lines", "A-DOOR: 3 lines", "P-FIXT: 4 lines, 1 line of words", "A-FURN:", "EQUIPMENT:", "A-AREA-IDEN: 2 lines of words"] {
        assert!(layers.contains(want), "{want}: {layers}");
    }
}
