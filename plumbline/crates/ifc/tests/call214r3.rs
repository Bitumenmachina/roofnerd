//! Call 214, round 3: which way a connection faces is carried by the file this program writes, and comes back both ways: by `import ifc`
//! onto a thing and by `import maker-model` onto an item's row. A connection with no facing is written with no axis. Made up.

use plumbline_ifc::{import, maker_model, write::export};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::units::{self, Len};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

const HOW: Entered = Entered { status: Status::Assumed, source: "a made-up plan" };

fn len(typed: &str) -> Val<Len> {
    let (l, unit) = units::parse_length(typed).expect("a length");
    Val { value: Some(l), unit: unit.to_string(), status: Status::Assumed, source: "a made-up size".to_string() }
}

/// A job: one level, an item 24 by 18 by 34 in with a waste connection facing down and a cold water connection that says nothing of its facing,
/// one of it set down.
fn house() -> Job {
    let mut job = Job::new("Round 3 (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", HOW).unwrap();
    let no = || Val::<String>::unknown("", "not chosen");
    let item = Item { category: Kind::Fixture, maker: no(), model: no(), width: len("24 in"), depth: len("18 in"), height: len("34 in"), weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
    let path = ed::item(&mut job, "Test lavatory", item).unwrap();
    let waste = ed::connection_row("Waste", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out, "0", "0", "2 in", Facing::Down, None, HOW).unwrap();
    ed::connection_add(&mut job, &path, waste).unwrap();
    let cold = ed::connection_row_facing("Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, "-4 in", "0", "22 in", None, None, HOW).unwrap();
    ed::connection_add(&mut job, &path, cold).unwrap();
    let left = ed::connection_row("Hot water", ConnectorKind::Pipe, Some(Domain::Hot), Flow::In, "4 in", "0", "22 in", Facing::Left, None, HOW).unwrap();
    ed::connection_add(&mut job, &path, left).unwrap();
    ed::place_item(&mut job, &level, &path, [10.0, 12.0], "0 ft", HOW).unwrap();
    job
}

fn facings(job: &mut Job) -> Vec<(String, Option<Facing>)> {
    let mut q = job.world.query::<&Connectors>();
    let mut out: Vec<(String, Option<Facing>)> = q.iter(&job.world).flat_map(|c| c.0.iter().map(|k| (k.name.clone(), k.facing)).collect::<Vec<_>>()).collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn no_axis(text: &str) -> usize {
    text.lines().filter(|l| l.contains("=IFCAXIS2PLACEMENT3D(#") && l.ends_with(",$,$);")).count()
}

#[test]
fn the_file_carries_which_way_a_connection_faces_and_says_nothing_where_there_is_none() {
    let mut job = house();
    let written = export(&mut job, "round3.ifc", "2026-11-25T00:00:00").unwrap();
    // The line that said which way it faces is not written is gone.
    assert!(!written.not_written.iter().any(|(_, _, why)| why.contains("which way it faces")), "{:?}", written.not_written);
    // The same job with the cold water facing back has one placement with no axis fewer.
    let mut other = house();
    ed::connection_set(&mut other, "Test lavatory", "Cold water", "faces", "back").unwrap();
    let with = export(&mut other, "round3.ifc", "2026-11-25T00:00:00").unwrap();
    assert_eq!(no_axis(&written.text), no_axis(&with.text) + 1);
}

/// Out and back by `import ifc`: the thing's connections have the facings they were written with.
#[test]
fn the_facing_comes_back_onto_a_thing() {
    let mut job = house();
    let written = export(&mut job, "round3.ifc", "2026-11-25T00:00:00").unwrap();
    let mut back = Job::new("Round 3 back", "test", "2026-11-25");
    import(&mut back, written.text.as_bytes(), "round3.ifc", HOW).unwrap();
    assert_eq!(facings(&mut back), vec![("Cold water".to_string(), None), ("Hot water".to_string(), Some(Facing::Left)), ("Waste".to_string(), Some(Facing::Down))]);
}

/// Out and back by `import maker-model`: the item's rows have the facings they were written with, and the one with none has none.
#[test]
fn the_facing_comes_back_onto_an_items_row() {
    let mut job = house();
    let written = export(&mut job, "round3.ifc", "2026-11-25T00:00:00").unwrap();
    let mut chest = Job::new("A chest", "test", "2026-11-25");
    let m = maker_model(&mut chest, written.text.as_bytes(), "round3.ifc", Some(Kind::Fixture)).unwrap();
    assert_eq!(m.made.len(), 1, "{m:?}");
    let rows = ed::item_rows(&mut chest, &m.made[0].path).unwrap();
    let by = |n: &str| rows.iter().find(|r| r.name == n).unwrap_or_else(|| panic!("no {n}: {rows:?} {:?}", m.made[0].connections_not_written)).faces;
    assert_eq!((by("Waste"), by("Hot water"), by("Cold water")), (Some(Facing::Down), Some(Facing::Left), None));
    assert!(m.made[0].notes.iter().any(|n| n.contains("1 connection point says nothing of which way it faces")), "{:?}", m.made[0].notes);
}
