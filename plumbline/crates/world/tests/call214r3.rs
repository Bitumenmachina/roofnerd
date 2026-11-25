//! Call 214, round 3: an item's connection may say nothing of which way it faces. Written before the code. Made up.

use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::tables;
use plumbline_world::units::{self, Len};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

const TYPICAL: Entered = Entered { status: Status::Assumed, source: "a typical rough-in; the product is not chosen" };
const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

fn len(typed: &str) -> Val<Len> {
    let (l, unit) = units::parse_length(typed).expect("a length");
    Val { value: Some(l), unit: unit.to_string(), status: Status::Assumed, source: "a made-up size".to_string() }
}

fn job_with_item() -> (Job, String, String) {
    let mut job = Job::new("Connections (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let no = || Val::<String>::unknown("", "not chosen");
    let item = Item { category: Kind::Fixture, maker: no(), model: no(), width: len("24 in"), depth: len("18 in"), height: len("34 in"), weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
    let path = ed::item(&mut job, "Test lavatory", item).unwrap();
    (job, level, path)
}

/// A job saved by the tree this work began from (an item library, the starter connections and a maker's item) reads and is written back
/// byte for byte: the facing key is there where it was and nothing is added.
#[test]
fn a_job_saved_before_reads_and_writes_back_the_same() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/job-with-connections-before-round-3.json")).unwrap();
    let (mut job, _) = file::load(&text).unwrap();
    assert_eq!(file::save(&mut job), text);
}

/// The table written before comes back byte for byte (rows as they stood).
#[test]
fn a_table_written_before_comes_back_the_same() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/job-with-connections-before-round-3.json")).unwrap();
    let (mut job, _) = file::load(&text).unwrap();
    let rows = tables::connections_out(&mut job);
    assert!(rows.iter().skip(1).all(|r| !r[8].is_empty()), "{rows:?}");
    let again = tables::connections_out(&mut job);
    assert_eq!(rows, again);
}

#[test]
fn a_connection_with_no_facing_is_kept_without_the_key_and_set_down_without_one() {
    let (mut job, level, item) = job_with_item();
    let row = ed::connection_row_facing("Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, "-4 in", "0", "height - 12 in", None, None, TYPICAL).unwrap();
    ed::connection_add(&mut job, &item, row).unwrap();
    let saved = file::save(&mut job);
    assert!(!saved.contains("\"faces\""), "the key is absent where there is none");
    // Said as it is: no way it faces.
    let worked = ed::item_connections_worked(&mut job, &item).unwrap();
    assert!(worked[0].said.contains("which way it faces is not said"), "{}", worked[0].said);
    // Read back, the same bytes.
    let (mut back, _) = file::load(&saved).unwrap();
    assert_eq!(file::save(&mut back), saved);
    // A thing set down from it has a connection with no facing.
    let thing = ed::place_item(&mut job, &level, &item, [5.0, 5.0], "0 ft", TYPICAL).unwrap();
    let e = job.find(&thing).unwrap();
    let c = job.world.get::<Connectors>(e).unwrap();
    assert_eq!(c.0.len(), 1);
    assert_eq!(c.0[0].facing, None);
}

/// `connection set faces none` clears it; a way sets it again; a word that is no way is refused.
#[test]
fn a_facing_is_set_cleared_and_refused() {
    let (mut job, level, item) = job_with_item();
    let row = ed::connection_row("Waste", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out, "0", "0", "height - 16 in", Facing::Down, None, TYPICAL).unwrap();
    ed::connection_add(&mut job, &item, row).unwrap();
    let thing = ed::place_item(&mut job, &level, &item, [5.0, 5.0], "0 ft", TYPICAL).unwrap();
    ed::connection_set(&mut job, &item, "Waste", "faces", "none").unwrap();
    assert_eq!(ed::item_rows(&mut job, &item).unwrap()[0].faces, None);
    let e = job.find(&thing).unwrap();
    assert_eq!(job.world.get::<Connectors>(e).unwrap().0[0].facing, None, "the thing follows");
    ed::connection_set(&mut job, &item, "Waste", "faces", "left").unwrap();
    assert_eq!(ed::item_rows(&mut job, &item).unwrap()[0].faces, Some(Facing::Left));
    assert!(ed::connection_set(&mut job, &item, "Waste", "faces", "sideways").is_err());
}

/// The table: a row with no facing writes an empty Faces cell and brings it back empty, byte for byte.
#[test]
fn the_table_says_an_empty_faces_cell_and_brings_it_back() {
    let (mut job, _, item) = job_with_item();
    let row = ed::connection_row_facing("Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, "-4 in", "0", "10 in", None, None, TYPICAL).unwrap();
    ed::connection_add(&mut job, &item, row).unwrap();
    let row = ed::connection_row("Waste", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out, "0", "0", "2 in", Facing::Down, None, TYPICAL).unwrap();
    ed::connection_add(&mut job, &item, row).unwrap();
    let rows = tables::connections_out(&mut job);
    assert_eq!(rows[1][8], "");
    assert_eq!(rows[2][8], "down");
    let before = file::save(&mut job);
    let brought = tables::connections_in(&mut job, &rows, None).unwrap();
    assert!(brought.not_used.is_empty(), "{:?}", brought.not_used);
    assert_eq!(file::save(&mut job), before);
    assert_eq!(tables::connections_out(&mut job), rows);
}
