//! Call 227 at the command line (contract 227M): the marks of a marked-up PDF listed and brought in as lines of the estimate,
//! kept with the drawing; a TIFF taken in as a plan. Every file is made up (demos/marks/make.py writes them to the PDF
//! standard; no file a marking program wrote was read). The figures are worked by hand in STATUS.md.

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
    let dir = std::env::temp_dir().join(format!("plumbline-227-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn demo(name: &str) -> String {
    format!("{}/../../demos/marks/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn plan(name: &str) -> String {
    format!("{}/../../demos/plans/{name}", env!("CARGO_MANIFEST_DIR"))
}

/// A new job and the made-up marked plan copied beside it (files are kept by their path from the job's folder).
fn start(name: &str) -> std::path::PathBuf {
    let dir = folder(name);
    ok(&dir, &["new", "job.json", "A made-up job"]);
    dir
}

fn line_of<'a>(said: &'a str, what: &str) -> &'a str {
    said.lines().find(|l| l.contains(what)).unwrap_or_else(|| panic!("no line with {what:?} in:\n{said}"))
}

#[test]
fn the_marks_of_a_page_are_listed_in_the_pages_order_with_what_each_says() {
    let dir = folder("list");
    let said = ok(&dir, &["marks", &demo("marked-plan.pdf")]);
    assert!(said.starts_with("marked-plan.pdf, page 1 of 1: 7 marks (2 lengths, 1 run of lengths, 2 areas, 1 count, 1 note)"), "{said}");
    // The page's own order.
    let order: Vec<usize> = ["South wall", "Outside walls", "Whole floor", "Bedroom", "Receptacle", "verify header", "North wall"].iter().map(|w| said.find(w).unwrap_or_else(|| panic!("{w}"))).collect();
    assert!(order.windows(2).all(|p| p[0] < p[1]), "{said}");
    for (what, want) in [("South wall  ", "30 LF"), ("Outside walls", "100 LF"), ("Whole floor", "600 SF"), ("Bedroom  ", "120 SF"), ("Receptacle", "4 EA")] {
        assert!(line_of(&said, what).contains(want), "{what}: {said}");
    }
    assert!(line_of(&said, "Receptacle").contains("this program's count of the symbols labelled so, not a figure the file states"), "{said}");
    assert!(line_of(&said, "verify header").contains("a note: it measures nothing"), "{said}");
    // North wall: both figures, flagged.
    let north = line_of(&said, "North wall  ");
    assert!(north.starts_with('!') && north.contains("31 LF") && north.contains("by its ends at 1/4\" = 1'-0\": 30 LF"), "{north}");
    assert!(said.contains("! North wall: it says 31 LF; by its ends at 1/4\" = 1'-0\" it works out to 30 LF: 1 LF apart. What the mark says is what comes in"), "{said}");
    // Only the flagged one is flagged.
    assert_eq!(said.lines().filter(|l| l.starts_with('!')).count(), 1, "{said}");
    // No word of the standard.
    for bad in ["annotation", "Annot", "dictionary", "Vertices", "Subtype", "Contents", "viewport", "Stamp"] {
        assert!(!said.contains(bad), "{bad}: {said}");
    }
}

#[test]
fn marks_on_a_page_with_none_say_so_and_a_picture_and_a_wrong_page_are_refused() {
    let dir = folder("none");
    assert_eq!(ok(&dir, &["marks", &plan("made-up-plan.pdf")]).trim(), "no marks on page 1 of made-up-plan.pdf");
    let why = refused(&dir, &["marks", &plan("made-up-plan.pdf"), "--page", "2"]);
    assert!(why.contains("the file has 1 page; page 2 was asked for"), "{why}");
    assert!(refused(&dir, &["marks", &plan("made-up-plan.png")]).contains("only a PDF has marks"));
    assert!(refused(&dir, &["marks"]).contains("marks takes one PDF"));
    assert!(refused(&dir, &["marks", &demo("marked-plan.pdf"), "--page", "x"]).contains("is not a page number"));
}

#[test]
fn without_counts_five_lines_come_in_and_the_count_is_left_and_said_and_with_counts_it_is_a_sixth() {
    let dir = start("import");
    let said = ok(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf")]);
    assert!(said.contains("5 marks measured"), "{said}");
    assert!(said.contains("  5 added:"), "{said}");
    for l in ["South wall: 30 LF", "Outside walls: 100 LF", "Whole floor: 600 SF", "Bedroom: 120 SF", "North wall: 31 LF"] {
        assert!(said.contains(l), "{l}: {said}");
    }
    assert!(!said.contains("Receptacle: 4 EA"), "{said}");
    assert!(said.contains("counts left: 1 label, 4 symbols (Receptacle): the file states no count, this program counts the page's symbols; `--counts` brings them in as lines"), "{said}");
    assert!(said.contains("5 lines are not priced") && said.contains("with no division"), "{said}");
    assert!(said.contains("! North wall: it says 31 LF; by its ends at 1/4\" = 1'-0\" it works out to 30 LF"), "{said}");
    assert!(said.contains("1 note not brought in: it measures nothing"), "{said}");
    let sheet = ok(&dir, &["estimate", "job.json"]);
    assert!(sheet.contains("from marked-plan.pdf, page 1: 5 lines of its marks, each Assumed, not priced, no division set"), "{sheet}");
    assert!(!sheet.contains("Receptacle"), "{sheet}");
    assert!(line_of(&sheet, "North wall").contains("~31.00 LF"), "{sheet}");
    let by_item = ok(&dir, &["estimate", "job.json", "by-item"]);
    assert!(by_item.contains("No division") && by_item.contains("5 items from 5 lines"), "{by_item}");
    // Each line's source names its mark, the page and the file; none is Known.
    let saved = std::fs::read_to_string(dir.join("job.json")).unwrap();
    assert!(saved.contains("mk-north") && saved.contains("(length), page 1 of marked-plan.pdf -- its ends work out to 30 LF at the scale it carries; what it says is what was taken"), "{saved}");
    assert_eq!(saved.matches("\"source\": \"mark ").count(), 5);
    assert!(!saved.contains("\"status\": \"Known\""));
    assert!(ok(&dir, &["estimate", "job.json"]).contains("PRICE                    no figure"));
    // Again, without counts: the same five, nothing added, and the count is left again.
    let before = std::fs::read(dir.join("job.json")).unwrap();
    let again = ok(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf")]);
    assert!(again.contains("5 are the same as the job already has") && !again.contains("added"), "{again}");
    assert_eq!(std::fs::read(dir.join("job.json")).unwrap(), before);
    // With --counts: the count is a sixth line, and says what it is a count of.
    let counted = ok(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf"), "--counts"]);
    assert!(counted.contains("  1 added:") && counted.contains("Receptacle: 4 EA") && counted.contains("5 are the same"), "{counted}");
    assert!(!counted.contains("counts left"), "{counted}");
    let saved = std::fs::read_to_string(dir.join("job.json")).unwrap();
    assert!(saved.contains("4 symbols labelled Receptacle counted on page 1 of marked-plan.pdf"), "{saved}");
    assert!(line_of(&ok(&dir, &["estimate", "job.json"]), "Receptacle").contains("~4.00 EA") || line_of(&ok(&dir, &["estimate", "job.json"]), "Receptacle").contains("~4 EA"));
    let before = std::fs::read(dir.join("job.json")).unwrap();
    let again = ok(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf"), "--counts"]);
    assert!(again.contains("6 are the same as the job already has") && !again.contains("added"), "{again}");
    assert_eq!(std::fs::read(dir.join("job.json")).unwrap(), before);
    // Without --counts again the count line already there is not "gone".
    assert!(!ok(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf")]).contains("gone from the file"));
}

#[test]
fn a_figure_that_changed_moves_its_line_a_mark_taken_out_is_named_and_its_line_stays() {
    let dir = start("changes");
    ok(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf"), "--counts"]);
    // A copy of the file with one figure changed, under the file's own name (a mark is known by its name in the file and the
    // file's name).
    let copy = dir.join("copy");
    std::fs::create_dir_all(&copy).unwrap();
    let put = |variant: &str| {
        std::fs::copy(demo(variant), copy.join("marked-plan.pdf")).unwrap();
        copy.join("marked-plan.pdf").to_str().unwrap().to_string()
    };
    let moved = ok(&dir, &["import", "marks", "job.json", &put("floor-650.pdf"), "--counts"]);
    assert!(moved.contains("Whole floor: was 600 SF, now 650 SF; its line was moved") && moved.contains("5 are the same"), "{moved}");
    assert!(line_of(&ok(&dir, &["estimate", "job.json"]), "Whole floor").contains("~650.00 SF"));
    // The same file brought in with the figure back: moved back.
    let gone = ok(&dir, &["import", "marks", "job.json", &put("no-bedroom.pdf"), "--counts"]);
    assert!(gone.contains("Bedroom: gone from the file; its line is still in the job"), "{gone}");
    assert!(line_of(&ok(&dir, &["estimate", "job.json"]), "Bedroom").contains("~120.00 SF"), "the line is left");
}

#[test]
fn what_cannot_be_read_is_named_and_left_out_and_the_others_come_in() {
    let dir = start("unread");
    let said = ok(&dir, &["import", "marks", "job.json", &demo("unreadable.pdf")]);
    assert!(said.contains("1 mark measured") && said.contains("South wall: 30 LF"), "{said}");
    assert!(said.contains("not read, left out of the estimate: Round bed: its edge is a curve"), "{said}");
    assert!(said.contains("not read, left out of the estimate: Loose area: it has no scale"), "{said}");
    assert!(said.contains("not read, left out of the estimate: Metric run: it says \"9 m\", a figure in m, a unit this program does not hold"), "{said}");
    let sheet = ok(&dir, &["estimate", "job.json"]);
    for left in ["Round bed", "Loose area", "Metric run"] {
        assert!(!sheet.contains(left), "{left} is not a line: {sheet}");
    }
    // Listed, each is named with why.
    let list = ok(&dir, &["marks", &demo("unreadable.pdf")]);
    assert_eq!(list.lines().filter(|l| l.starts_with('?')).count(), 3, "{list}");
    // A scale set for part of the sheet is its own reason.
    let part = ok(&dir, &["marks", &demo("viewport.pdf")]);
    assert!(part.contains("not read: its scale is set for a part of the sheet, which is not read"), "{part}");
    // A mark that cannot be read now is not "gone" when its file is read again.
    let again = ok(&dir, &["import", "marks", "job.json", &demo("unreadable.pdf")]);
    assert!(!again.contains("gone from the file"), "{again}");
}

#[test]
fn a_page_with_no_marks_leaves_the_job_byte_for_byte_as_it_was() {
    let dir = start("nomarks");
    ok(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf")]);
    let before = std::fs::read(dir.join("job.json")).unwrap();
    let said = ok(&dir, &["import", "marks", "job.json", &plan("made-up-plan.pdf")]);
    assert!(said.starts_with("no marks on page 1 of made-up-plan.pdf: nothing was brought in"), "{said}");
    assert_eq!(std::fs::read(dir.join("job.json")).unwrap(), before);
    assert!(refused(&dir, &["import", "marks", "job.json", &plan("made-up-plan.png")]).contains("only a PDF has marks"));
    assert!(refused(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf"), "--page", "3"]).contains("page 3 was asked for"));
    assert!(refused(&dir, &["import", "marks", "job.json", &demo("marked-plan.pdf"), "--status", "Known", "--source", "x"]).contains("import marks does not take --status"));
}

#[test]
fn a_marked_pdf_added_as_a_drawing_keeps_its_marks_and_says_so_and_a_plain_one_is_as_it_was() {
    let dir = start("drawing");
    let level = ok(&dir, &["level", "job.json", "First Floor", "--elevation", "0 ft", "--height", "9 ft", "--status", "Known", "--source", "made up"]);
    let level = level.split(':').next().unwrap().to_string();
    std::fs::copy(demo("marked-plan.pdf"), dir.join("marked-plan.pdf")).unwrap();
    std::fs::copy(plan("made-up-plan.pdf"), dir.join("plain.pdf")).unwrap();
    let plain = ok(&dir, &["drawing", "job.json", &level, "Plain", "--file", "plain.pdf", "--status", "Assumed", "--source", "the drawing"]);
    assert!(!plain.contains("marks"), "a page with none says nothing of marks: {plain}");
    let marked = ok(&dir, &["drawing", "job.json", &level, "Marked", "--file", "marked-plan.pdf", "--status", "Assumed", "--source", "the drawing"]);
    assert!(marked.contains("marks: 7 marks on the page kept with the drawing (6 measured)"), "{marked}");
    let text = std::fs::read_to_string(dir.join("job.json")).unwrap();
    assert!(text.contains("\"marks\"") && text.contains("mk-south") && text.contains("Whole floor"), "kept in the job");
    // Each figure a mark says is Assumed and names the mark; none is Known.
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut seen = 0;
    for e in v["data"].as_array().unwrap() {
        if let Some(d) = e["attributes"].get("pl::drawing") {
            if let Some(marks) = d.get("marks") {
                for m in marks.as_array().unwrap() {
                    if let Some(says) = m.get("says") {
                        seen += 1;
                        assert_eq!(says["status"], "Assumed");
                        let src = says["source"].as_str().unwrap();
                        assert!(src.starts_with("mark \"") || src.contains(" symbols labelled "), "{says}");
                    }
                }
            } else {
                assert_eq!(d["file"]["value"], "plain.pdf", "only the marked drawing has marks");
            }
        }
    }
    assert_eq!(seen, 6);
}

/// The TIFFs: a sheet written by demos/marks/make.py from the made-up plan's PNG.
#[test]
fn a_tiff_is_added_as_a_drawing_the_size_of_the_png_and_a_wall_over_it_is_the_same_wall() {
    let dir = start("tiff");
    let level = ok(&dir, &["level", "job.json", "First Floor", "--elevation", "0 ft", "--height", "9 ft", "--status", "Known", "--source", "made up"]);
    let level = level.split(':').next().unwrap().to_string();
    ok(&dir, &["type", "job.json", "wall", "Wall 6", "--thickness", "6 in", "--status", "Known", "--source", "made up"]);
    std::fs::copy(demo("made-up-plan.tif"), dir.join("plan.tif")).unwrap();
    std::fs::copy(plan("made-up-plan.png"), dir.join("plan.png")).unwrap();
    let t = ok(&dir, &["drawing", "job.json", &level, "As a TIFF", "--file", "plan.tif", "--status", "Assumed", "--source", "the scan"]);
    let p = ok(&dir, &["drawing", "job.json", &level, "As a PNG", "--file", "plan.png", "--status", "Assumed", "--source", "the scan"]);
    assert!(t.contains("page 1 of 1 of plan.tif (TIFF picture), 1584 by 1224 dots"), "{t}");
    assert!(p.contains("page 1 of 1 of plan.png (PNG picture), 1584 by 1224 dots"), "{p}");
    let (td, pd) = (t.split(':').next().unwrap().to_string(), p.split(':').next().unwrap().to_string());
    // The same scale on each (the 540-dot south wall is 30 ft over a dot dimension of 1,080), and a wall of the same two points.
    for d in [&td, &pd] {
        ok(&dir, &["calibrate", "job.json", d, "--from", "252,252", "--to", "1332,252", "--distance", "30 ft", "--status", "Known", "--source", "the dimension"]);
    }
    let wt = "type-0003";
    let walls = |dir: &std::path::Path| line_of(&ok(dir, &["takeoff", "job.json"]), "walls 1").to_string();
    ok(&dir, &["wall", "job.json", &level, wt, "--points", "252,252 1332,252", "--height", "9 ft", "--on", &td]);
    let a = walls(&dir);
    ok(&dir, &["remove", "job.json", "wall-0006"]);
    ok(&dir, &["wall", "job.json", &level, wt, "--points", "252,252 1332,252", "--height", "9 ft", "--on", &pd]);
    let b = walls(&dir);
    assert!(a.contains("30.00 LF"), "{a}");
    assert_eq!(a, b, "the same wall over the TIFF and over the PNG");
}

#[test]
fn a_tiff_of_many_pages_gives_the_page_asked_for_and_a_packing_not_read_is_named() {
    let dir = start("pages");
    let level = ok(&dir, &["level", "job.json", "First Floor", "--elevation", "0 ft", "--height", "9 ft", "--status", "Known", "--source", "made up"]);
    let level = level.split(':').next().unwrap().to_string();
    std::fs::copy(demo("two-pages.tif"), dir.join("two.tif")).unwrap();
    std::fs::copy(demo("old-jpeg.tif"), dir.join("old.tif")).unwrap();
    let one = ok(&dir, &["drawing", "job.json", &level, "Sheet 1", "--file", "two.tif", "--status", "Assumed", "--source", "the scan"]);
    assert!(one.contains("page 1 of 2 of two.tif (TIFF picture), 1584 by 1224 dots"), "{one}");
    let two = ok(&dir, &["drawing", "job.json", &level, "Sheet 2", "--file", "two.tif", "--page", "2", "--status", "Assumed", "--source", "the scan"]);
    assert!(two.contains("page 2 of 2 of two.tif (TIFF picture), 800 by 600 dots"), "{two}");
    assert!(refused(&dir, &["drawing", "job.json", &level, "Sheet 3", "--file", "two.tif", "--page", "3", "--status", "Assumed", "--source", "x"]).contains("the file has 2 pages; page 3 was asked for"));
    let old = refused(&dir, &["drawing", "job.json", &level, "Old", "--file", "old.tif", "--status", "Assumed", "--source", "x"]);
    assert!(old.contains("packed as old-style JPEG, which is not read") && old.contains("save it again as an uncompressed, LZW, deflate, PackBits or CCITT group 4 TIFF"), "{old}");
}
