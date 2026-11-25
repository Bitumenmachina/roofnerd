//! Call 213 at the command line (contract 213M): what the second command-line judge found, each point driven through the program
//! from an empty job. Everything is made up; the figures are worked by hand in STATUS.md ("The command line judged a second time").
//! The starter lavatory is 20 in wide, 18 in deep, 34 in high; its cold water is 4 in right of the middle, 21 in up, at the back, so
//! at (10.3333, 20.75, 1.75) when it stands with its middle at (10, 20).

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
    let dir = std::env::temp_dir().join(format!("plumbline-213-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const K: [&str; 4] = ["--status", "Known", "--source", "made-up plan"];
const A: [&str; 4] = ["--status", "Assumed", "--source", "made-up quote"];

fn path_in(said: &str, word: &str) -> String {
    said.split_whitespace().map(|w| w.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))).find(|w| w.starts_with(word)).unwrap_or_else(|| panic!("no {word} in {said}")).to_string()
}

fn with(args: &[&str], more: &[&str]) -> Vec<String> {
    args.iter().chain(more.iter()).map(|s| s.to_string()).collect()
}

fn okv(dir: &std::path::Path, args: Vec<String>) -> String {
    ok(dir, &args.iter().map(String::as_str).collect::<Vec<_>>())
}

/// A job with the starter items and connections, a level Ground and the starter lavatory set down with its middle at (10, 20); the
/// level's path and the pipe type's path.
fn lavatory(dir: &std::path::Path) -> (String, String, String) {
    ok(dir, &["new", "j.json", "Lavatory (not a real job)"]);
    let level = path_in(&okv(dir, with(&["level", "j.json", "Ground", "--elevation", "0 ft", "--height", "9 ft"], &K)), "level-");
    ok(dir, &["import", "items", "j.json", "starter"]);
    ok(dir, &["import", "connections", "j.json", "starter"]);
    let placed = ok(dir, &[&["place", "j.json", &level, "Lavatory", "--at", "10,20", "--offset", "0 ft"][..], &K[..]].concat());
    let thing = path_in(&placed, "fixture-");
    let pipe = path_in(&okv(dir, with(&["type", "j.json", "pipe", "pipe half", "--round", "1/2 in"], &K)), "type-");
    (level, pipe, thing)
}

fn pipe(dir: &std::path::Path, level: &str, ty: &str, points: &str, height: &str) -> String {
    let said = ok(dir, &[&["run", "j.json", level, "pipe", ty, "--points", points, "--height", height][..], &A[..]].concat());
    said.lines().find_map(|l| l.strip_prefix("1 straight piece: ")).unwrap_or_else(|| panic!("{said}")).trim().to_string()
}

fn lines_with(said: &str, word: &str) -> String {
    said.lines().filter(|l| l.contains(word)).collect::<Vec<_>>().join("\n")
}

// ------------------------------------------------------------------ point 1 (F16)

/// An item renamed to the name of a material is refused, the job is what it was, and it still opens.
#[test]
fn point_1_an_item_cannot_take_the_name_of_a_material() {
    let dir = folder("p1");
    ok(&dir, &["new", "q.json", "Names (not a real job)"]);
    ok(&dir, &["import", "items", "q.json", "starter"]);
    std::fs::write(dir.join("m.csv"), "Material,Bought by,Price\nPlywood,SHEET,40\n").unwrap();
    ok(&dir, &["import", "prices", "q.json", "m.csv", "--add", "--status", "Assumed", "--source", "made-up list"]);
    let before = std::fs::read(dir.join("q.json")).unwrap();
    let said = refused(&dir, &["rename", "q.json", "Refrigerator", "Plywood"]);
    assert!(said.contains("there is already something called \"Plywood\"") && said.contains("two things in one place cannot go by one name"), "{said}");
    assert_eq!(std::fs::read(dir.join("q.json")).unwrap(), before, "the job is byte for byte what it was");
    assert!(ok(&dir, &["check", "q.json"]).contains("agree"), "the job still opens");
    // The same item renamed to a name nothing has goes on working.
    ok(&dir, &["rename", "q.json", "Refrigerator", "Fridge, 36 in"]);
    ok(&dir, &["check", "q.json"]);
}

// ------------------------------------------------------------------ point 2 (F1, F7)

/// How far a run's end stands off its connection is measured, as both now stand; a run already off that is carried farther is said;
/// a run the change brings onto its connection is said joined.
#[test]
fn point_2_how_far_a_run_stands_off_is_measured() {
    let dir = folder("p2");
    let (level, ty, _) = lavatory(&dir);
    // The cold pipe ends 2.4 in behind the cold connection (20.95 against 20.75).
    let cold = pipe(&dir, &level, &ty, "10.3333,26 10.3333,20.95", "21 in");
    // Deeper by 6 in: the back moves 3 in away, onto the pipe's end (21.0) and the pipe passes over it: joined.
    let said = ok(&dir, &["set", "j.json", "Lavatory", "depth", "24 in"]);
    assert!(said.contains("1 run is now joined to its connection") && said.contains(&cold) && said.contains("Cold water connection"), "{said}");
    assert!(!said.contains("stands off"), "{said}");
    // Back to 18 in: the pipe's end is 0.2 ft = 2.4 in from the connection, which the program writes 2 3/8 in; the connection moved 3 in.
    let said = ok(&dir, &["set", "j.json", "Lavatory", "depth", "18 in"]);
    assert!(said.contains("1 run now stands off its connection") && said.contains(&format!("is 2 3/8 in behind Lavatory's Cold water connection")), "{said}");
    assert!(!said.contains("is 3 in"), "the 3 in is how far the connection moved: {said}");
    // A second pipe ends exactly at the hot connection; the depth typed 12 in moves the back 3 in nearer.
    let hot = pipe(&dir, &level, &ty, "9.6667,26 9.6667,20.75", "21 in");
    let said = ok(&dir, &["set", "j.json", "Lavatory", "depth", "12 in"]);
    assert!(said.contains("1 run now stands off its connection") && said.contains(&format!("{hot}")) && said.contains("is 3 in behind Lavatory's Hot water connection"), "{said}");
    assert!(said.contains("1 run still stands off its connection") && said.contains(&format!("{cold}")) && said.contains("5 3/8 in behind Lavatory's Cold water connection") && said.contains("it was 2 3/8 in"), "{said}");
    // Back to 18 in: the hot pipe is joined again; the cold one stands off by 2 3/8 in again, from 5 3/8 in.
    let said = ok(&dir, &["set", "j.json", "Lavatory", "depth", "18 in"]);
    assert!(said.contains("1 run is now joined to its connection") && lines_with(&said, "joined").contains(&hot), "{said}");
    assert!(said.contains("1 run still stands off its connection") && said.contains("2 3/8 in behind Lavatory's Cold water connection") && said.contains("it was 5 3/8 in"), "{said}");
}

// ------------------------------------------------------------------ point 3 (F2)

/// A run that passes over a connection is joined and stays so; takeoff says it is a connection, naming the thing and the connection;
/// the clash says the run is joined to it and goes on so many inches inside.
#[test]
fn point_3_a_run_over_a_connection_is_said_as_that() {
    let dir = folder("p3");
    let (level, ty, _) = lavatory(&dir);
    // A pipe through the lavatory from (10, 25) to (10, 18) at 19 in up, over its waste connection at (10, 20.75).
    let through = pipe(&dir, &level, &ty, "10,25 10,18", "19 in");
    let take = ok(&dir, &["takeoff", "j.json"]);
    assert!(!take.contains("open: Lavatory: its Waste connection"), "the waste is not open: {take}");
    assert!(take.contains(&format!("{through}")) || take.contains("pipe 1"), "{take}");
    let over = lines_with(&take, "passes over");
    assert!(over.contains("Lavatory's Waste connection") && over.contains("pipe 1"), "takeoff names the thing and the connection: {take}");
    assert!(!take.contains("1 place where a run meets another along its length"), "{take}");
    let clashes = ok(&dir, &["clashes", "j.json"]);
    let words = lines_with(&clashes, "Waste connection");
    assert!(words.contains("joined to Lavatory's Waste connection") && words.contains("goes on 18 in inside it"), "{clashes}");
    // The same fact read from the other side: a pipe that ends at the old back, the depth typed deeper: joined by passing over, and 3 in inside.
    let dir = folder("p3b");
    let (level, ty, _) = lavatory(&dir);
    pipe(&dir, &level, &ty, "10.3333,25 10.3333,20.75", "21 in");
    ok(&dir, &["set", "j.json", "Lavatory", "depth", "24 in"]);
    let clashes = ok(&dir, &["clashes", "j.json"]);
    assert!(lines_with(&clashes, "Cold water connection").contains("joined to Lavatory's Cold water connection and goes on 3 in inside it"), "{clashes}");
    let take = ok(&dir, &["takeoff", "j.json"]);
    assert!(lines_with(&take, "passes over").contains("Lavatory's Cold water connection"), "{take}");
}

// ------------------------------------------------------------------ point 4 (F3)

#[test]
fn point_4_connect_outside_the_thing_names_the_connection_and_how_far() {
    let dir = folder("p4");
    let (level, ty, thing) = lavatory(&dir);
    let run = pipe(&dir, &level, &ty, "10.3333,25 10.3333,20.79", "21 in");
    let said = refused(&dir, &[&["connect", "j.json", &thing, &run][..], &A[..]].concat());
    assert!(said.contains("Cold water connection") && said.contains("1/2 in") && !said.contains("10.3333"), "{said}");
    // A run nowhere near it keeps the old words.
    let far = pipe(&dir, &level, &ty, "30,25 30,28", "21 in");
    let said = refused(&dir, &[&["connect", "j.json", &thing, &far][..], &A[..]].concat());
    assert!(said.contains("neither end of"), "{said}");
}

// ------------------------------------------------------------------ points 5, 6 (F4, F5)

#[test]
fn point_5_where_a_size_was_typed_is_said_by_who_typed_it() {
    let dir = folder("p5");
    lavatory(&dir);
    let said = ok(&dir, &["set", "j.json", "Lavatory", "depth", "24 in"]);
    assert!(said.contains("Assumed: typed at the command line; not yet checked against its maker's sheet"), "{said}");
    assert!(!said.contains("typed in the window"), "{said}");
    let items = ok(&dir, &["items", "j.json", "Lavatory"]);
    assert!(items.contains("depth: typed at the command line") && !items.contains("typed in the window"), "{items}");
}

#[test]
fn point_6_the_same_size_typed_again_at_the_command_line_is_not_a_check() {
    let dir = folder("p6");
    lavatory(&dir);
    ok(&dir, &["set", "j.json", "Lavatory", "depth", "24 in"]);
    let before = std::fs::read(dir.join("j.json")).unwrap();
    let said = ok(&dir, &["set", "j.json", "Lavatory", "depth", "24 in"]);
    assert!(said.contains("depth is 24 in already: nothing changed") && said.contains("--status Known --source"), "{said}");
    assert!(!said.contains("Known, checked on site"), "{said}");
    assert_eq!(std::fs::read(dir.join("j.json")).unwrap(), before, "nothing changed");
    assert!(!ok(&dir, &["items", "j.json", "Lavatory"]).contains("checked on site"));
    // With how sure and where from it is Known.
    let said = ok(&dir, &["set", "j.json", "Lavatory", "depth", "24 in", "--status", "Known", "--source", "J. Smith, tape"]);
    assert!(said.contains("depth 24 in is Known: J. Smith, tape"), "{said}");
    assert!(ok(&dir, &["items", "j.json", "Lavatory"]).contains("depth: J. Smith, tape"));
    let csv = dir.join("items.csv");
    ok(&dir, &["export", "items", "j.json", csv.to_str().unwrap()]);
    let text = std::fs::read_to_string(&csv).unwrap();
    assert!(text.contains("Known,J. Smith, tape") || text.contains("Known,\"J. Smith, tape\""), "{text}");
    // A Known size with another size typed says so: it is a measurement of that size.
    let said = ok(&dir, &["set", "j.json", "Lavatory", "width", "22 in", "--status", "Known", "--source", "J. Smith, tape"]);
    assert!(said.contains("width 22 in is Known: J. Smith, tape"), "{said}");
    // Unknown is no size.
    refused(&dir, &["set", "j.json", "Lavatory", "width", "22 in", "--status", "Unknown", "--source", "x"]);
}

// ------------------------------------------------------------------ point 7 (F6)

#[test]
fn point_7_a_turn_is_said_one_way_and_a_turn_by_nothing_changes_nothing() {
    let dir = folder("p7");
    let (_, _, thing) = lavatory(&dir);
    let said = ok(&dir, &["turn", "j.json", &thing, "--by", "270", "--status", "Assumed", "--source", "x"]);
    assert!(said.contains("turned 270 degrees from east"), "{said}");
    assert!(ok(&dir, &["show", "j.json", &thing]).contains("270 deg"), "show says it the same way");
    let said = ok(&dir, &["turn", "j.json", &thing, "--by", "180", "--status", "Assumed", "--source", "x"]);
    assert!(said.contains("turned 90 degrees from east"), "{said}");
    let said = ok(&dir, &["turn", "j.json", &thing, "--by", "450", "--status", "Assumed", "--source", "x"]);
    assert!(said.contains("turned 180 degrees from east"), "{said}");
    let before = std::fs::read(dir.join("j.json")).unwrap();
    for by in ["0", "360"] {
        let said = refused(&dir, &["turn", "j.json", &thing, "--by", by, "--status", "Assumed", "--source", "x"]);
        assert!(said.contains("changes nothing") && !said.contains("not a figure"), "{said}");
    }
    assert_eq!(std::fs::read(dir.join("j.json")).unwrap(), before);
    let said = refused(&dir, &["turn", "j.json", &thing, "--by", "a lot", "--status", "Assumed", "--source", "x"]);
    assert!(said.contains("number of degrees"), "{said}");
    // Four quarters come back to 0 and say 0.
    let said = ok(&dir, &["turn", "j.json", &thing, "--by", "180", "--status", "Assumed", "--source", "x"]);
    assert!(said.contains("turned 0 degrees from east"), "{said}");
}

// ------------------------------------------------------------------ point 8 (F8, F20)

#[test]
fn point_8_a_length_inside_a_figure_is_written_as_a_length_is_written() {
    let dir = folder("p8");
    lavatory(&dir);
    for (typed, shown) in [("1/2 in", "up 1/2 in"), ("5 1/2 in", "up 5 1/2 in")] {
        ok(&dir, &["connection", "j.json", "Lavatory", "set", "Waste", "up", typed]);
        let items = ok(&dir, &["items", "j.json", "Lavatory"]);
        assert!(lines_with(&items, "Waste").contains(shown), "{typed}: {items}");
    }
    // A foot and inches, or marks: the same 3 ft 6 in = 42 in, on an item 34 in high it is 8 in above the item.
    for typed in ["3 ft 6 in", "3'6\"", "3' 6\"", "3 ft 6 in + 0 in"] {
        let said = refused(&dir, &["connection", "j.json", "Lavatory", "set", "Waste", "up", typed]);
        assert!(said.contains("8 in above the item"), "{typed}: {said}");
        assert!(!said.contains("'\\''"), "{typed}: {said}");
    }
    // A mark nothing can read says how to write a length, with no stray marks.
    let said = refused(&dir, &["connection", "j.json", "Lavatory", "set", "Waste", "up", "3#"]);
    assert!(said.contains("3 ft 6 in") && !said.contains("'\\''"), "{said}");
    // Shapes take it too.
    ok(&dir, &["shape", "j.json", "keep", "Lavatory", "My lavatory"]);
    let said = ok(&dir, &["shape", "j.json", "set", "My lavatory", "1", "back to", "3 1/2 in"]);
    assert!(said.contains("Back to, set to 3 1/2 in"), "{said}");
    let said = ok(&dir, &["shape", "j.json", "set", "My lavatory", "1", "back to", "1/2 in"]);
    assert!(said.contains("Back to, set to 1/2 in"), "{said}");
}

// ------------------------------------------------------------------ point 9 (F9)

#[test]
fn point_9_a_size_typed_on_a_connection_says_it_was_typed() {
    let dir = folder("p9");
    lavatory(&dir);
    let said = ok(&dir, &["connection", "j.json", "Lavatory", "set", "Hot water", "size", "1/2 in"]);
    assert!(said.contains("size 1/2 in (Assumed: typed at the command line; not checked against the maker's sheet)"), "{said}");
    let csv = dir.join("c.csv");
    ok(&dir, &["export", "connections", "j.json", csv.to_str().unwrap()]);
    let text = std::fs::read_to_string(&csv).unwrap();
    let row = text.lines().find(|l| l.starts_with("Lavatory,Hot water")).unwrap();
    assert!(row.ends_with("1/2 in,Assumed,typed at the command line; not checked against the maker's sheet"), "{row}");
    // With how sure and where from: as said, and the place's words do not stand in for it.
    let said = ok(&dir, &["connection", "j.json", "Lavatory", "set", "Cold water", "size", "3/4 in", "--status", "Assumed", "--source", "J. Smith, rough-in sheet"]);
    assert!(said.contains("size 3/4 in (Assumed: J. Smith, rough-in sheet)"), "{said}");
    // A size nobody measured is not Known on its own say-so.
    refused(&dir, &["connection", "j.json", "Lavatory", "set", "Cold water", "size", "3/4 in", "--status", "Known", "--source", "me"]);
}

// ------------------------------------------------------------------ point 10 (F10, F11, F12, F14, F17, F19)

/// A workbook is written as a workbook and said to be one; rows the job already has are counted as that.
#[test]
fn point_10_a_workbook_is_not_called_csv_and_rows_the_job_has_are_counted() {
    let dir = folder("p10a");
    lavatory(&dir);
    let book = dir.join("c.xlsx");
    ok(&dir, &["export", "connections", "j.json", book.to_str().unwrap()]);
    assert_eq!(&std::fs::read(&book).unwrap()[..2], b"PK", "a workbook, not CSV text under another name");
    ok(&dir, &["new", "g.json", "Another job (not a real job)"]);
    ok(&dir, &["import", "items", "g.json", "starter"]);
    let said = ok(&dir, &["import", "connections", "g.json", book.to_str().unwrap()]);
    assert!(said.contains("c.xlsx (XLSX") && !said.contains("(CSV)"), "{said}");
    assert!(said.contains("5 rows under its headings"), "{said}");
    // The starter again, into a job that has them: five rows, every one the same as the job has, none "0 rows".
    let said = ok(&dir, &["import", "connections", "g.json", "starter"]);
    assert!(said.contains("5 rows under its headings") && said.contains("the same as the job already has: 5") && !said.contains("0 rows"), "{said}");
    // The other tables the same.
    for what in ["items", "shapes", "prices", "assemblies"] {
        let out = dir.join(format!("{what}.xlsx"));
        ok(&dir, &["export", what, "j.json", out.to_str().unwrap()]);
        assert_eq!(&std::fs::read(&out).unwrap()[..2], b"PK", "{what}");
    }
    // Out and back stays the same to the byte, as a CSV table.
    let (a, b) = (dir.join("a.csv"), dir.join("b.csv"));
    ok(&dir, &["export", "items", "j.json", a.to_str().unwrap()]);
    ok(&dir, &["new", "h.json", "Third job (not a real job)"]);
    ok(&dir, &["import", "items", "h.json", dir.join("items.xlsx").to_str().unwrap()]);
    ok(&dir, &["export", "items", "h.json", b.to_str().unwrap()]);
    assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
}

/// Each bad row of a connections table gives its own reason, once, with no name said twice.
#[test]
fn point_10_each_bad_row_gives_its_own_reason_once() {
    let dir = folder("p10b");
    ok(&dir, &["new", "g.json", "Rows (not a real job)"]);
    ok(&dir, &["import", "items", "g.json", "starter"]);
    let head = "Item,Connection,Takes,Carries,Goes,Across,Back,Up,Faces,Place: how sure,Place: where from";
    let text = format!(
        "{head}\nWater closet,Waste,pipe,waste,out,0,12 in,0,down,Known,a made-up place\nWater closet,Cold water,pipe,cold,in,-6 in,0,6 in,back,Assumed,a made-up place\nWater closet,Vent,pipe,vent,out,0,0,2 in,up,Known,a made-up place\nLavatory,Waste,pipe,waste,out,0,0,19 in,back,Known,a made-up place\n"
    );
    std::fs::write(dir.join("bad.csv"), text).unwrap();
    let said = ok(&dir, &["import", "connections", "g.json", "bad.csv"]);
    let rows: Vec<&str> = said.lines().filter(|l| l.trim_start().starts_with("row ")).collect();
    assert_eq!(rows.len(), 4, "{said}");
    assert!(rows[0].contains("row 2") && rows[0].contains("Waste connection's place is said to be Known"), "{said}");
    assert!(rows[2].contains("row 4") && rows[2].contains("Vent connection's place is said to be Known"), "{said}");
    assert!(rows[1].contains("row 3") && rows[1].contains("its connections are left out, as 2 of its rows cannot be used (rows 2 and 4)"), "{said}");
    assert!(rows[3].contains("row 5") && rows[3].contains("Lavatory: the Waste connection's place is said to be Known"), "{said}");
    assert!(!said.contains("\"Water closet\": ") && !said.contains("\"Lavatory\": ") && !said.contains("row 2 of them"), "no name twice, no count read as a row number: {said}");
}

#[test]
fn point_10_a_refused_size_names_the_size_typed_and_says_to_move_the_connection_first() {
    let dir = folder("p10c");
    lavatory(&dir);
    let said = refused(&dir, &["set", "j.json", "Lavatory", "height", "20 in"]);
    assert!(said.contains("height 20 in") && said.contains("Hot water connection") && said.contains("1 in above the item") && said.contains("move the connection first"), "{said}");
    assert!(!said.contains("34 in high"), "{said}");
}

#[test]
fn point_10_a_price_that_is_not_a_figure_says_how_to_write_one() {
    let dir = folder("p10d");
    ok(&dir, &["new", "q.json", "Prices (not a real job)"]);
    ok(&dir, &["import", "items", "q.json", "starter"]);
    for typed in ["1850 each", "$1.85k", "abc"] {
        let said = refused(&dir, &["set", "q.json", "Water heater, 50 gal", "price", typed, "--status", "Assumed", "--source", "x"]);
        assert!(said.contains("is not a figure") && said.contains("$180/EA"), "{typed}: {said}");
    }
    let said = refused(&dir, &["set", "q.json", "Water heater, 50 gal", "price", "$1e3", "--status", "Assumed", "--source", "x"]);
    assert!(said.contains("$180/EA"), "{said}");
    // A price to a fraction of a cent is taken, as before.
    let said = ok(&dir, &["set", "q.json", "Water heater, 50 gal", "price", "$1,850.456", "--status", "Assumed", "--source", "x"]);
    assert!(said.contains("$1,850.456/EA"), "{said}");
}

#[test]
fn point_10_taking_the_last_block_names_the_way_to_draw_the_item_as_something_else() {
    let dir = folder("p10e");
    lavatory(&dir);
    ok(&dir, &["shape", "j.json", "keep", "Lavatory", "My lavatory"]);
    ok(&dir, &["shape", "j.json", "take", "My lavatory", "1"]);
    let said = refused(&dir, &["shape", "j.json", "take", "My lavatory", "1"]);
    assert!(said.contains("a shape keeps at least one block") && said.contains("Drawn as") && !said.contains("take the item's shape off"), "{said}");
}

// ------------------------------------------------------------------ point 13 (F18)

#[test]
fn point_13_a_shape_no_item_is_drawn_as_is_written_out_and_counted() {
    let dir = folder("p13");
    lavatory(&dir);
    for name in ["Lavatory a", "Lavatory b", "Lavatory c"] {
        ok(&dir, &["shape", "j.json", "keep", "Lavatory", name]);
    }
    let out = dir.join("x.csv");
    let said = ok(&dir, &["export", "shapes", "j.json", out.to_str().unwrap()]);
    let text = std::fs::read_to_string(&out).unwrap();
    for name in ["Lavatory a", "Lavatory b", "Lavatory c"] {
        assert!(text.lines().any(|l| l.starts_with(&format!("{name},"))), "{name} is written: {text}");
    }
    assert!(said.contains("no item is drawn as 2 of them"), "{said}");
    // Brought into a new job: all three are there.
    ok(&dir, &["new", "n.json", "New (not a real job)"]);
    ok(&dir, &["import", "shapes", "n.json", out.to_str().unwrap()]);
    let list = ok(&dir, &["shape", "n.json"]);
    for name in ["Lavatory a", "Lavatory b", "Lavatory c"] {
        assert!(list.contains(name), "{list}");
    }
    // Out and back is the same table to the byte.
    let again = dir.join("y.csv");
    ok(&dir, &["export", "shapes", "n.json", again.to_str().unwrap()]);
    let (a, b) = (std::fs::read_to_string(&out).unwrap(), std::fs::read_to_string(&again).unwrap());
    for name in ["Lavatory a", "Lavatory b", "Lavatory c"] {
        let rows = |t: &str| t.lines().filter(|l| l.starts_with(&format!("{name},"))).map(str::to_string).collect::<Vec<_>>();
        assert_eq!(rows(&a), rows(&b), "{name}");
    }
}

// ------------------------------------------------------------------ points 14, 15 (F21, F22)

#[test]
fn point_14_a_refused_figure_is_tried_at_the_items_own_size_first() {
    let dir = folder("p14");
    lavatory(&dir);
    ok(&dir, &["new", "v.json", "Vanity (not a real job)"]);
    ok(&dir, &["import", "items", "v.json", "starter"]);
    ok(&dir, &["shape", "v.json", "keep", "Vanity, 36 in", "Vanity panel"]);
    // The vanity is 36 in wide, 21 in deep, 34 1/2 in high.
    let said = refused(&dir, &["shape", "v.json", "set", "Vanity panel", "1", "up to", "height + 1 ft"]);
    assert!(said.contains("Vanity, 36 in") && said.contains("it would stand 12 in above the top") && !said.contains("4 13/16"), "{said}");
    let said = refused(&dir, &["shape", "v.json", "set", "Vanity panel", "1", "up to", "36 in"]);
    assert!(said.contains("it would stand 1 1/2 in above the top") && said.contains("Vanity, 36 in") && !said.contains("4 13/16"), "{said}");
    // 30 in holds on the vanity, and fails on the smaller item a shape is also tried on: it says so, and why.
    let said = refused(&dir, &["shape", "v.json", "set", "Vanity panel", "1", "up to", "30 in"]);
    assert!(said.contains("holds on Vanity, 36 in") && said.contains("4 13/16 in wide") && said.contains("a shape has to hold on a smaller item too"), "{said}");
}

#[test]
fn point_15_a_block_with_no_thickness_or_written_backwards_is_refused() {
    let dir = folder("p15");
    ok(&dir, &["new", "v.json", "Vanity (not a real job)"]);
    ok(&dir, &["import", "items", "v.json", "starter"]);
    ok(&dir, &["shape", "v.json", "keep", "Vanity, 36 in", "Vanity panel"]);
    let before = std::fs::read(dir.join("v.json")).unwrap();
    // Block 3 is the top: up from height - (40 mm or less) to height. Up from typed height is flat.
    let said = refused(&dir, &["shape", "v.json", "set", "Vanity panel", "3", "up from", "height"]);
    assert!(said.contains("block 3") && said.contains("Up") && said.contains("no thickness") && said.contains("34 1/2 in"), "{said}");
    assert_eq!(std::fs::read(dir.join("v.json")).unwrap(), before);
    // Across to 0 is fine (from -18 in to 0); then Across from width / 2 is past it: backwards, naming both ends.
    ok(&dir, &["shape", "v.json", "set", "Vanity panel", "3", "across to", "0"]);
    let said = refused(&dir, &["shape", "v.json", "set", "Vanity panel", "3", "across from", "width / 2"]);
    assert!(said.contains("block 3") && said.contains("Across") && said.contains("backwards") && said.contains("18 in") && said.contains("0 in"), "{said}");
    // Flat across too.
    let said = refused(&dir, &["shape", "v.json", "set", "Vanity panel", "3", "across from", "0"]);
    assert!(said.contains("no thickness"), "{said}");
}
