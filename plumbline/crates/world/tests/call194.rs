//! Call 194: the sample whole house prices its outside trim, and the figure it needs on the way:
//! `corner_board_length_above_<entry>`, the length of a wall's corner boards above a height the line
//! enters. Everything is made up. The hand figures are those of STATUS.md ("The sample whole house
//! prices its outside trim: a demonstration"), worked beside each test.

use plumbline_world::editing as ed;
use plumbline_world::file;
use plumbline_world::sample;
use plumbline_world::tables;
use plumbline_world::trim::{self, PieceKind as PK};
use plumbline_world::value::Status;
use plumbline_world::*;

fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up tracing" }
}
fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn rows(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .map(|line| {
            let (mut out, mut cell, mut quoted) = (Vec::new(), String::new(), false);
            for c in line.chars() {
                match (c, quoted) {
                    ('"', _) => quoted = !quoted,
                    (',', false) => out.push(std::mem::take(&mut cell)),
                    (c, _) => cell.push(c),
                }
            }
            out.push(cell);
            out
        })
        .collect()
}
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}
fn len3(p: &trim::Piece) -> f64 {
    ((p.to[0] - p.from[0]).powi(2) + (p.to[1] - p.from[1]).powi(2) + (p.to[2] - p.from[2]).powi(2)).sqrt()
}
fn figure(job: &mut Job, path: &str, name: &str) -> Derived {
    job.derive();
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).and_then(|m| m.0.get(name)).cloned().unwrap_or_else(|| panic!("no figure {name} on {path}"))
}
fn quantity(job: &mut Job, path: &str, line: &str) -> Derived {
    job.derive();
    let e = job.find(path).unwrap();
    job.world.get::<LineQuantities>(e).unwrap().0[line].quantity.clone()
}

const LINE: &str = "Corner boards";
const ABOVE: &str = "corner_board_length_above_wainscot";

/// A box 24 by 16 on the walls' lines, walls 6 in, `high` tall; each wall's Corner boards line priced by the figure, the
/// wainscot entered at `entry`.
fn box_house(high: &str, entry: &str, entered: ed::Entered<'static>, entry_status: Status) -> (Job, Vec<String>) {
    let mut job = Job::new("Box (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", high, entered).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", entered).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height(high), entered).unwrap();
    for w in &walls {
        job.add_line(w, LINE, "corner_board_length", Some(Class::Material), false).unwrap();
        job.set_formula(w, Some(LINE), ABOVE, &[("wainscot".to_string(), entry.to_string())], entry_status, "the stone line's name (made-up)").unwrap();
    }
    job.derive();
    (job, walls)
}

/// The pieces laid on a wall for its Corner boards line add to what the line prices, and start where the stone ends.
fn pieces_add_to_the_quantity(job: &mut Job, walls: &[String], from_z: f64, each: f64) {
    let ps = trim::pieces(job);
    for w in walls {
        let q = quantity(job, w, LINE).value.unwrap();
        let mine: Vec<&trim::Piece> = ps.iter().filter(|p| &p.of == w && p.kind == PK::CornerBoard).collect();
        let got: f64 = mine.iter().map(|p| len3(p)).sum();
        assert!(near(got, q), "{w}: the pieces lie {got} LF and the line prices {q}");
        for p in &mine {
            assert!(near(p.from[2], from_z) && near(len3(p), each), "{w}: {p:?}");
        }
    }
}

// By hand: 9 ft walls, 3 ft of stone: each of two corners 9 - 3 = 6 ft, 12 LF a wall (the plain figure is 18). 12 ft walls: 2 x 9 = 18.
// Pieces: two of 6 ft that start 3 ft above the foot; two of 9 ft that start 3 ft above the foot.
#[test]
fn the_boards_above_the_stone_are_two_corners_each_the_walls_height_less_the_stone() {
    for (high, per_wall, each) in [("9 ft", 12.0, 6.0), ("12 ft", 18.0, 9.0), ("10 ft", 14.0, 7.0)] {
        let (mut job, walls) = box_house(high, "3 ft", assumed(), Status::Assumed);
        for w in &walls {
            let d = figure(&mut job, w, ABOVE);
            assert_eq!((d.value, d.unit.as_str()), (Some(per_wall), "LF"), "{high} {w}: {d:?}");
            assert_eq!(quantity(&mut job, w, LINE).value, Some(per_wall), "{high} {w}");
        }
        pieces_add_to_the_quantity(&mut job, &walls, 3.0, each);
        assert_eq!(trim::pieces(&mut job).iter().filter(|p| p.kind == PK::CornerBoard).count(), 8, "{high}");
    }
}

// An entry of nothing is the whole board: 2 x 9 = 18, from the foot. An entry as tall as the wall, or taller, leaves nothing:
// 0 LF and no piece (never less than nothing).
#[test]
fn an_entry_of_nothing_is_the_whole_board_and_one_as_tall_as_the_wall_or_taller_is_nothing() {
    let (mut job, walls) = box_house("9 ft", "0 ft", assumed(), Status::Assumed);
    for w in &walls {
        assert_eq!(figure(&mut job, w, ABOVE).value, Some(18.0));
        assert_eq!(figure(&mut job, w, "corner_board_length").value, Some(18.0));
    }
    pieces_add_to_the_quantity(&mut job, &walls, 0.0, 9.0);
    for entry in ["9 ft", "10 ft", "40 ft"] {
        let (mut job, walls) = box_house("9 ft", entry, assumed(), Status::Assumed);
        for w in &walls {
            assert_eq!(figure(&mut job, w, ABOVE).value, Some(0.0), "{entry}");
        }
        assert!(trim::pieces(&mut job).iter().all(|p| p.kind != PK::CornerBoard), "{entry}: a board of no length is not drawn");
    }
}

// A wall with one outside corner (the L's two walls at its inside corner): 9 - 3 = 6 LF, one piece. A wall with none (a partition
// that meets another wall's middle, a tee): 0 LF and no piece.
#[test]
fn a_wall_with_one_outside_corner_has_one_board_and_a_wall_with_none_has_none() {
    let mut job = Job::new("L (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", assumed()).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", assumed()).unwrap();
    let ring = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [12.0, 16.0], [12.0, 26.0], [0.0, 26.0]];
    let mut walls = ed::walls(&mut job, &level, &w6, &ring, true, ed::Top::Height("9 ft"), assumed()).unwrap();
    let tee = ed::walls(&mut job, &level, &w6, &[[6.0, 0.0], [6.0, 26.0]], false, ed::Top::Height("9 ft"), assumed()).unwrap();
    walls.extend(tee);
    for w in &walls {
        job.add_line(w, LINE, "corner_board_length", Some(Class::Material), false).unwrap();
        job.set_formula(w, Some(LINE), ABOVE, &[("wainscot".to_string(), "3 ft".to_string())], Status::Assumed, "made-up").unwrap();
    }
    job.derive();
    let corners: Vec<f64> = walls.iter().map(|w| figure(&mut job, w, "outside_corners").value.unwrap()).collect();
    assert_eq!(corners, vec![2.0, 2.0, 1.0, 1.0, 2.0, 2.0, 0.0], "the plan is the L of call 184 and a tee across it");
    let above: Vec<f64> = walls.iter().map(|w| figure(&mut job, w, ABOVE).value.unwrap()).collect();
    // 2 x 6, 2 x 6, 1 x 6, 1 x 6, 2 x 6, 2 x 6, 0.
    assert_eq!(above, vec![12.0, 12.0, 6.0, 6.0, 12.0, 12.0, 0.0]);
    let ps = trim::pieces(&mut job);
    assert_eq!(ps.iter().filter(|p| p.kind == PK::CornerBoard).count(), 10);
    for (w, want) in walls.iter().zip(&above) {
        let got: f64 = ps.iter().filter(|p| &p.of == w).map(|p| len3(p)).sum();
        assert!(near(got, *want), "{w}: {got} {want}");
    }
    assert!(ps.iter().filter(|p| p.kind == PK::CornerBoard).all(|p| near(p.from[2], 3.0) && near(p.to[2], 9.0)));
}

// Its status is the weakest of the corner boards' length and the entered height: Known walls and a Known 3 ft are Known;
// either one Assumed makes it Assumed.
#[test]
fn it_is_as_sure_as_the_weakest_of_the_boards_and_the_entered_height() {
    for (wall, entry, want) in [(known(), Status::Known, Status::Known), (known(), Status::Assumed, Status::Assumed), (assumed(), Status::Known, Status::Assumed), (assumed(), Status::Assumed, Status::Assumed)] {
        let (mut job, walls) = box_house("9 ft", "3 ft", wall, entry);
        let d = figure(&mut job, &walls[0], ABOVE);
        assert_eq!(d.status, want, "walls {:?} entry {entry:?}", wall.status);
        assert!(!d.formula.is_empty() && d.inputs.iter().any(|i| i.starts_with("wall-")), "it says what it is made of: {d:?}");
        // The pieces and the line are as sure.
        assert_eq!(quantity(&mut job, &walls[0], LINE).status, want);
    }
}

// The refusals follow net_area_below_<entry>: a formula that names the figure with no entry on the line is refused when it is set;
// an entry that is no length leaves the figure refused with a reason, not nothing and not the whole board.
#[test]
fn a_line_without_its_entry_is_refused_and_an_entry_that_is_no_length_is_not_worked() {
    let (mut job, walls) = box_house("9 ft", "3 ft", assumed(), Status::Assumed);
    job.add_line(&walls[0], "Bare", "corner_board_length", Some(Class::Material), false).unwrap();
    let err = job.set_formula(&walls[0], Some("Bare"), ABOVE, &[], Status::Assumed, "made-up").unwrap_err();
    assert!(err.contains("which takes a height the line enters as") && err.contains("the line has none"), "{err}");
    let below = job.set_formula(&walls[0], Some("Bare"), "net_area_below_wainscot", &[], Status::Assumed, "made-up").unwrap_err();
    assert!(below.contains("which takes a height the line enters as"), "the figure beside it says the same: {below}");
    // An entry in other units than a length.
    match job.set_formula(&walls[1], Some(LINE), ABOVE, &[("wainscot".to_string(), "3".to_string())], Status::Assumed, "made-up") {
        Err(e) => assert!(!e.is_empty()),
        Ok(_) => {
            let d = figure(&mut job, &walls[1], ABOVE);
            assert!(d.value.is_none() && !d.note.is_empty(), "no length is no figure, said: {d:?}");
        }
    }
    // Two lines of one wall giving two heights: refused, said.
    job.add_line(&walls[2], "Second", "corner_board_length", Some(Class::Material), false).unwrap();
    job.set_formula(&walls[2], Some("Second"), ABOVE, &[("wainscot".to_string(), "4 ft".to_string())], Status::Assumed, "made-up").unwrap();
    let d = figure(&mut job, &walls[2], ABOVE);
    assert!(d.value.is_none() && d.note.contains("two heights"), "{d:?}");
}

// Where the program asks whether a wall's corner boards are priced, a line priced by the figure is one: its pieces are laid
// (the window draws trim where `trim::lay` lays it), a plain line beside it is said and not doubled, and a line priced by the
// figure is not "another figure" to say_other.
#[test]
fn a_line_priced_by_the_figure_is_a_corner_board_line_to_the_pieces_and_what_is_left_out() {
    let (mut job, walls) = box_house("9 ft", "3 ft", assumed(), Status::Assumed);
    let laid = trim::lay(&mut job);
    assert_eq!(laid.pieces.iter().filter(|p| p.kind == PK::CornerBoard).count(), 8);
    assert!(laid.left_out.iter().all(|s| !s.contains("not corner board length")), "{:?}", laid.left_out);
    assert_eq!(laid.priced_of(trim::PieceKind::CornerBoard), Some(48.0), "4 walls x 12 LF");
    assert!(laid.pieces.iter().all(|p| p.line == LINE));
    let _ = walls;
}

// The saved job and the table of assemblies carry the line as it is: the entry is on the line, the quantity text is the figure's
// name; written and read back it is the same.
#[test]
fn the_job_file_and_the_assembly_table_round_trip_the_line() {
    let (mut job, walls) = box_house("9 ft", "3 ft", assumed(), Status::Assumed);
    let text = file::save(&mut job);
    let (mut back, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(quantity(&mut back, &walls[0], LINE).value, Some(12.0));
    assert_eq!(file::save(&mut back), text);
    // The table of assemblies: a library written from a wall that carries the line, read into an empty job, gives the line back.
    let mut library = Job::new("Library (not a real job)", "test", "2026-11-24");
    tables::assemblies_in(&mut library, &rows(tables::STARTER_LIBRARY), None).unwrap();
    let out = tables::assemblies_out(&mut library);
    let wanted = out.iter().find(|r| r[0] == "Exterior wall, 2x6, lap siding over a stone wainscot" && r[7] == "Corner boards").expect("the line is written");
    assert_eq!(wanted[9], ABOVE);
    assert!(wanted.iter().any(|c| c == "wainscot = 3 ft"), "{wanted:?}");
    let mut again = Job::new("Library (not a real job)", "test", "2026-11-24");
    tables::assemblies_in(&mut again, &out, None).unwrap();
    assert_eq!(tables::assemblies_out(&mut again), out, "written and read back, the table is the same");
}

// The starter wainscot wall applied to a 10 ft wall with two outside corners, its wainscot 3 ft: Corner boards 14 LF, where it was 20.
// The plain wall keeps the plain figure: 20.
#[test]
fn the_starter_wainscot_wall_buys_its_corner_boards_from_the_top_of_the_stone() {
    for (assembly, want) in [("Exterior wall, 2x6, lap siding over a stone wainscot", 14.0), ("Exterior wall, 2x6, lap siding", 20.0)] {
        let mut job = Job::new("Box (not a real job)", "test", "2026-11-24");
        let level = ed::storey(&mut job, "ground", "0 ft", "10 ft", assumed()).unwrap();
        let w6 = ed::wall_type(&mut job, "w6", "6 in", assumed()).unwrap();
        let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("10 ft"), assumed()).unwrap();
        tables::assemblies_in(&mut job, &rows(tables::STARTER_LIBRARY), None).unwrap();
        job.apply(&walls[0], assembly, "the starter library").unwrap();
        let q = quantity(&mut job, &walls[0], "Corner boards");
        assert_eq!((q.value, q.status), (Some(want), Status::Assumed), "{assembly}: {q:?}");
        let ps = trim::pieces(&mut job);
        let got: f64 = ps.iter().filter(|p| p.of == walls[0] && p.kind == PK::CornerBoard).map(len3).sum();
        assert!(near(got, want), "{assembly}: the pieces lie {got}");
    }
}

// The sample whole house, by the hand table of STATUS.md: 33 lines added, $3,721.40.
#[test]
fn the_sample_house_prices_its_outside_trim_to_the_hand_table() {
    let mut job = sample::house_priced();
    let est = view::estimate(&mut job);
    assert_eq!(est.job_cost.value.map(|x| (x * 100.0).round() / 100.0), Some(393_981.66), "job cost was 390,260.26: 3,721.40 more");
    assert_eq!(est.job_cost.members, 432, "was 399 lines: 33 more");
    let sum = |word: &str| -> (usize, f64, f64) {
        let ls: Vec<_> = est.lines.iter().filter(|l| !l.off && l.description.starts_with(word)).collect();
        (ls.len(), ls.iter().map(|l| l.quantity.value.unwrap()).sum(), ls.iter().map(|l| l.amount.value.unwrap()).sum())
    };
    let (n, q, a) = sum("Corner boards, 5/4 x 6 cedar");
    assert_eq!((n, q, (a * 100.0).round() / 100.0), (8, 128.0, 800.0), "72 LF above and 56 LF on the stone, at $6.25");
    let (n, q, a) = sum("Exterior casing, 5/4 x 4 cedar");
    assert_eq!((n, q, (a * 100.0).round() / 100.0), (17, 289.0, 1560.6), "135 + 154 LF at $5.40");
    let (n, q, a) = sum("Fascia, 1x8 cedar");
    assert_eq!((n, q, (a * 100.0).round() / 100.0), (4, 144.0, 691.2));
    let (n, q, a) = sum("Soffit, cedar plywood, vented");
    assert_eq!((n, q, (a * 100.0).round() / 100.0), (4, 108.0, 669.6));
}
