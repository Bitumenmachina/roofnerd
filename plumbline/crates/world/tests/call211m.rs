//! Call 211, the model's side (contract 211M): where an item connects. Written before the code. Every figure is made up and
//! worked by hand beside the test that holds it (the lead's figures are in STATUS.md, "Where an item connects: a
//! demonstration"; where the program's own way differs, the test holds the program's and says so).
//!
//! The item: "Test lavatory", 24 in wide, 18 in deep, 34 in high, a fixture. Three rows:
//!   Cold water   pipe, cold, in   across -4 in, back 0, up height - 12 in, faces back, 1/2 in
//!   Hot water    pipe, hot,  in   across  4 in, back 0, up height - 12 in, faces back, 1/2 in
//!   Waste        pipe, waste, out across  0,     back 0, up height - 16 in, faces back, 1 1/2 in
//! In the thing's own axes (feet): y = depth / 2 - back = 0.75, so cold (-1/3, 0.75, 22/12), hot (1/3, 0.75, 22/12), waste (0, 0.75, 1.5).

use plumbline_world::building::network::{network, Joint, Network, TOLERANCE_FT};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::tables;
use plumbline_world::units::{self, Len};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

const ASSUMED: Entered = Entered { status: Status::Assumed, source: "made-up plan" };
const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };
const TYPICAL: Entered = Entered { status: Status::Assumed, source: "a typical rough-in; the product is not chosen" };

fn len(typed: &str, status: Status) -> Val<Len> {
    let (l, unit) = units::parse_length(typed).expect("a length");
    Val { value: Some(l), unit: unit.to_string(), status, source: "a made-up size".to_string() }
}

/// A job with one level (floor at 0 ft) and the made-up item; its path.
fn job_with_item() -> (Job, String, String) {
    let mut job = Job::new("Connections (not a real job)", "test", "2026-11-25");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let not_chosen = || Val::<String>::unknown("", "not chosen");
    let item = Item { category: Kind::Fixture, maker: not_chosen(), model: not_chosen(), width: len("24 in", Status::Assumed), depth: len("18 in", Status::Assumed), height: len("34 in", Status::Assumed), weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
    let path = ed::item(&mut job, "Test lavatory", item).unwrap();
    (job, level, path)
}

fn rows3(job: &mut Job, item: &str) {
    for (name, domain, flow, across, up, size) in [("Cold water", Domain::Cold, Flow::In, "-4 in", "height - 12 in", "1/2 in"), ("Hot water", Domain::Hot, Flow::In, "4 in", "height - 12 in", "1/2 in"), ("Waste", Domain::Waste, Flow::Out, "0", "height - 16 in", "1 1/2 in")] {
        let row = ed::connection_row(name, ConnectorKind::Pipe, Some(domain), flow, across, "0", up, Facing::Back, Some(size), TYPICAL).unwrap();
        ed::connection_add(job, item, row).unwrap();
    }
}

fn with_rows() -> (Job, String, String) {
    let (mut job, level, item) = job_with_item();
    rows3(&mut job, &item);
    (job, level, item)
}

fn near(a: [f64; 3], b: [f64; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1.0e-4)
}

fn connectors(job: &mut Job, thing: &str) -> Vec<Connector> {
    let e = job.find(thing).unwrap();
    job.world.get::<Connectors>(e).map(|c| c.0.clone()).unwrap_or_default()
}

fn pipe_type(job: &mut Job) -> String {
    ed::segment_type(job, "pipe 1/2 in", Kind::Pipe, "1/2 in", None, ASSUMED).unwrap()
}

/// A pipe from (x, y_from) to (x, y_to) at `height` above the floor.
fn pipe_to(job: &mut Job, level: &str, ty: &str, x: f64, from: f64, to: f64, height: &str) -> String {
    ed::runs(job, level, Kind::Pipe, ty, &[[x, from], [x, to]], height, None, ASSUMED).unwrap().remove(0)
}

fn net(job: &mut Job) -> Network {
    network(job, None, None, TOLERANCE_FT).unwrap()
}

/// The connections of `thing` that meet no run, by name.
fn open_names(job: &mut Job, thing: &str) -> Vec<String> {
    let n = net(job);
    let mut out: Vec<String> = Vec::new();
    for &i in &n.open {
        for j in &n.nodes[i].joints {
            if let Joint::Connector { element, index, .. } = j {
                if element == thing {
                    out.push(connectors(job, thing)[*index].name.clone());
                }
            }
        }
    }
    out.sort();
    out
}

// ---------------------------------------------------------------- point 1: rows on an item

/// Step 1: the item shows its three connections at its sizes, in inches. Cold 4 in to one side of the middle, at the back (0 in
/// from it), 22 in up; hot 4 in the other side; waste on the middle, 18 in up. A figure written against the box (a formula of
/// width, depth and height, with units) is worked at the item's own size.
#[test]
fn point_1_an_item_says_where_it_connects_at_its_own_sizes() {
    let (mut job, _, item) = with_rows();
    let worked = ed::item_connections_worked(&mut job, &item).unwrap();
    assert_eq!(worked.len(), 3);
    let said: Vec<String> = worked.iter().map(|w| w.said.clone()).collect();
    assert!(said[0].contains("Cold water") && said[0].contains("across -4 in") && said[0].contains("back 0 in") && said[0].contains("up 22 in") && said[0].contains("faces back"), "{}", said[0]);
    assert!(said[1].contains("across 4 in") && said[1].contains("up 22 in"), "{}", said[1]);
    assert!(said[2].contains("Waste") && said[2].contains("across 0 in") && said[2].contains("up 18 in") && said[2].contains("1 1/2 in"), "{}", said[2]);
    // In the thing's own axes: y is depth / 2 - back (the thing's back is its +y, as a block's is).
    assert!(near(worked[0].at, [-1.0 / 3.0, 0.75, 22.0 / 12.0]), "{:?}", worked[0].at);
    assert!(near(worked[2].at, [0.0, 0.75, 1.5]), "{:?}", worked[2].at);
    // No word a builder would not use: no port, connector, coordinate or foot with decimals.
    for s in &said {
        for bad in ["port", "connector", "coordinate", " ft", "0.75", "1.8333"] {
            assert!(!s.contains(bad), "{s:?} says {bad:?}");
        }
    }
}

/// The rows are kept on the item, as a component of its own, and a job that never had rows saves as it always did.
#[test]
fn point_2_rows_are_kept_in_the_job_and_a_job_with_none_is_the_job_it_was() {
    // A job of one item and nothing else, as the program wrote it before this work (made with the program built from 5b8a1af:
    // `plumbline new` then `import items` of the one row below; the file is kept beside this test).
    let before = include_str!("call211m_item_before.json");
    let mut job = Job::new("Baseline (not a real job)", "plumbline", "2026-11-25");
    let csv = "Item,Trade,Kind,Width,Depth,Height,How sure,Where from\nTest lavatory,15 Mechanical,fixture,24 in,18 in,34 in,Assumed,a made-up size\n";
    let rows = plumbline_world::building::shape::csv_rows(csv);
    tables::items_in(&mut job, &rows, None).unwrap();
    assert_eq!(file::save(&mut job), before, "an item and nothing else saves as it did");
    // Opened and saved again, the same to the byte.
    let (mut back, _) = file::load(before).unwrap();
    assert_eq!(file::save(&mut back), before);
    // With rows: the key is there, the schema names it, and it comes back as it went.
    let item = job.find("type-0001").map(|_| "type-0001".to_string()).unwrap_or_else(|| {
        let mut q = job.world.query::<(&Node, &Item)>();
        q.iter(&job.world).map(|(n, _)| n.path.clone()).next().unwrap()
    });
    rows3(&mut job, &item);
    let text = file::save(&mut job);
    assert!(text.contains("\"pl::item_connections\""), "the rows are in the file");
    let (mut again, _) = file::load(&text).unwrap();
    assert_eq!(file::save(&mut again), text);
    // And the one that has none writes no key and no schema entry for it.
    assert!(!before.contains("item_connections"));
}

/// Step 10: the sample whole house does not move. Its saved job is byte for byte what the program wrote before (the file made by
/// the program built from 5b8a1af has 1,106,884 bytes; FNV-1a of it is 0x4b693d4593b5857f).
#[test]
fn point_9_the_sample_house_does_not_move() {
    let mut job = sample::house_priced();
    let text = file::save(&mut job);
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    assert_eq!((text.len(), h), (1_106_884, 0x4b69_3d45_93b5_857f), "plumbline sixteen is the job it was");
    assert!(!text.contains("item_connections"));
}

// ---------------------------------------------------------------- point 3: checked when written

/// Step 7, three refusals, each naming the item, the connection and the figure, in inches, with no feet with decimals, no
/// coordinates and no "small size".
#[test]
fn point_3_a_row_is_checked_when_it_is_written_and_a_refusal_can_be_read() {
    let (mut job, _, item) = with_rows();
    let before = file::save(&mut job);
    let bad = |job: &mut Job, row: Result<ItemConnection, String>| -> String {
        let row = match row {
            Ok(r) => r,
            Err(why) => return why,
        };
        let why = ed::connection_add(job, &item, row).expect_err("refused");
        assert_eq!(file::save(job), before, "a refused row changes nothing");
        why
    };
    // 2 in above the item.
    let above = bad(&mut job, ed::connection_row("Vent", ConnectorKind::Pipe, Some(Domain::Vent), Flow::Out, "0", "0", "height + 2 in", Facing::Up, Some("1 1/2 in"), TYPICAL));
    for must in ["Test lavatory", "Vent", "Up", "height + 2 in", "2 in", "above"] {
        assert!(above.contains(must), "{above:?} lacks {must:?}");
    }
    // A second row named Cold water (in any capitals).
    let twice = bad(&mut job, ed::connection_row("cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, "0", "0", "10 in", Facing::Back, None, TYPICAL));
    assert!(twice.contains("Test lavatory") && twice.contains("Cold water") && twice.contains("name"), "{twice:?}");
    // A duct that says it carries cold.
    let duct = bad(&mut job, ed::connection_row("Cold duct", ConnectorKind::Duct, Some(Domain::Cold), Flow::In, "0", "0", "10 in", Facing::Back, None, TYPICAL));
    assert!(duct.contains("Test lavatory") && duct.contains("Cold duct") && duct.contains("pipe"), "{duct:?}");
    for why in [&above, &twice, &duct] {
        for ugly in ["small", "large size", "typical size", "port", "connector", "coordinate", "0.", " ft"] {
            assert!(!why.contains(ugly), "{why:?} says {ugly:?}");
        }
    }
    // A figure that is not a length, a name the item has not got, a figure that is a plain 4, a connection nobody can place.
    for (across, back, up, why_has) in [("width * depth", "0", "10 in", "length"), ("0", "0", "colour * 2 in", "colour"), ("4", "0", "10 in", "unit"), ("0", "-2 in", "10 in", "behind")] {
        let why = bad(&mut job, ed::connection_row("Drain", ConnectorKind::Pipe, Some(Domain::Waste), Flow::Out, across, back, up, Facing::Down, None, TYPICAL));
        assert!(why.contains("Test lavatory") && why.contains("Drain") && why.contains(why_has), "{why:?} lacks {why_has:?}");
    }
    // Known on a row's say-so is refused: a place is Known only where somebody measures it on the thing.
    let known = bad(&mut job, ed::connection_row("Vent", ConnectorKind::Pipe, Some(Domain::Vent), Flow::Out, "0", "0", "10 in", Facing::Up, None, KNOWN));
    assert!(known.contains("Known") && known.contains("Vent"), "{known:?}");
    // More than 64 (a thing may hold 64).
    let (mut big, _, it) = job_with_item();
    for n in 0..64 {
        let row = ed::connection_row(&format!("C{n}"), ConnectorKind::Power, None, Flow::Both, "0", "0", "10 in", Facing::Back, None, TYPICAL).unwrap();
        ed::connection_add(&mut big, &it, row).unwrap();
    }
    let row = ed::connection_row("One more", ConnectorKind::Power, None, Flow::Both, "0", "0", "10 in", Facing::Back, None, TYPICAL).unwrap();
    let why = ed::connection_add(&mut big, &it, row).expect_err("the 65th");
    assert!(why.contains("64") && why.contains("Test lavatory"), "{why:?}");
}

// ---------------------------------------------------------------- point 4: set down

/// Step 2: set down with its middle at (10, 20), not turned, on the floor: 3 connections, all Assumed, each saying the item and the
/// connection it came from, at (9.6667, 20.75, 1.8333), (10.3333, 20.75, 1.8333), (10, 20.75, 1.5) in feet. `takeoff` names all
/// three open.
#[test]
fn point_4_a_thing_set_down_has_its_connections_from_that_moment() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    let c = connectors(&mut job, &thing);
    assert_eq!(c.len(), 3);
    for k in &c {
        assert_eq!(k.at.status, Status::Assumed, "never surer than Assumed, though the thing is set down Known");
        assert!(k.at.source.contains("Test lavatory") && k.at.source.contains(&k.name) && k.at.source.contains("a typical rough-in"), "{}", k.at.source);
        assert_eq!(k.facing, Some(Facing::Back));
    }
    let stood = ed::connections_standing(&mut job, &thing).unwrap();
    assert!(near(stood[0].at, [9.0 + 2.0 / 3.0, 20.75, 22.0 / 12.0]), "{:?}", stood[0].at);
    assert!(near(stood[1].at, [10.0 + 1.0 / 3.0, 20.75, 22.0 / 12.0]), "{:?}", stood[1].at);
    assert!(near(stood[2].at, [10.0, 20.75, 1.5]), "{:?}", stood[2].at);
    assert_eq!(open_names(&mut job, &thing), ["Cold water", "Hot water", "Waste"]);
    // And takeoff says so by name.
    let said: Vec<String> = building::takeoff(&mut job).iter().flat_map(|s| s.joined.open.iter().map(|o| o.said.clone())).collect();
    for name in ["Cold water", "Hot water", "Waste"] {
        assert!(said.iter().any(|s| s.contains(&format!("its {name} connection"))), "{said:?}");
    }
    // A thing whose item has no rows has no connections, as before.
    let (mut plain, level2, item2) = job_with_item();
    let t2 = ed::place_item(&mut plain, &level2, &item2, [3.0, 3.0], "0 ft", KNOWN).unwrap();
    assert!(connectors(&mut plain, &t2).is_empty());
    let e = plain.find(&t2).unwrap();
    assert!(plain.world.get::<Connectors>(e).is_none(), "no empty list is written");
}

/// A thing set down from an item whose sizes are not all known has no connections, and the reply says why.
#[test]
fn point_4_an_item_with_no_size_gives_no_connections_and_says_so() {
    let (mut job, level, item) = with_rows();
    // Depth not known.
    let e = job.find(&item).unwrap();
    let mut it = job.world.get::<Item>(e).cloned().unwrap();
    it.depth = Val::unknown("in", "not measured");
    job.put(&item, it).unwrap();
    let (thing, notes) = ed::place_item_said(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    assert!(connectors(&mut job, &thing).is_empty());
    assert!(notes.iter().any(|n| n.contains("connection") && n.contains("depth")), "{notes:?}");
}

// ---------------------------------------------------------------- one arithmetic

/// A block and a connection written with the same figures land on the same point, turned and not (the shape's own function turns
/// across, back and up into the thing's axes; the window turns a block by the thing's angle as the network turns a connection).
#[test]
fn one_arithmetic_a_block_and_a_connection_of_the_same_figures_are_at_one_point() {
    use plumbline_world::building::shape::{Piece, Shape};
    let (mut job, level, item) = job_with_item();
    // A shape of one block that is a point (its two figures on each axis the same): across a sixth of the width to the left, back 0, up 22/34 of the height.
    let piece = Piece { made: plumbline_world::building::forms::Made::Timber, round: false, across: ["-width / 6".into(), "-width / 6".into()], back: ["0".into(), "0".into()], up: ["height * 22 / 34".into(), "height * 22 / 34".into()], when: None };
    job.shapes.push(Shape { name: "a point".into(), blocks: vec![piece] });
    let row = ed::connection_row("Cold water", ConnectorKind::Pipe, Some(Domain::Cold), Flow::In, "-width / 6", "0", "height * 22 / 34", Facing::Back, None, TYPICAL).unwrap();
    ed::connection_add(&mut job, &item, row).unwrap();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    // Turned 0, 90, 37 and -90 degrees (each turn is by so much more, counter-clockwise from east).
    for by in [0.0, 90.0, -53.0, -127.0] {
        if by != 0.0 {
            ed::turn(&mut job, &thing, by, ASSUMED).unwrap();
        }
        let angle = {
            let e = job.find(&thing).unwrap();
            job.world.get::<Placed>(e).unwrap().angle_deg().unwrap()
        };
        // The block's point in the thing's frame, worked by the shape; turned as the window turns it.
        let size = [2.0, 1.5, 34.0 / 12.0];
        let b = job.shapes[0].worked(size).unwrap().remove(0);
        let (sin, cos) = angle.to_radians().sin_cos();
        let (x, y) = (b.lo[0], b.lo[1]);
        let block = [10.0 + x * cos - y * sin, 20.0 + x * sin + y * cos, b.lo[2]];
        let stood = ed::connections_standing(&mut job, &thing).unwrap();
        assert!(near(stood[0].at, block), "at {angle} degrees: the connection {:?} and the block {block:?}", stood[0].at);
    }
}

// ---------------------------------------------------------------- point 6: joined

/// Steps 3 and 4: a cold pipe that ends at the cold connection joins it (takeoff no longer names it; no clash). A hot pipe that
/// ends inside the lavatory 3 in in front of the hot connection does not join it: it is a clash, the line says the Hot water
/// connection is 3 in away, and `connect` is refused with the same 3 in, saying which way.
#[test]
fn point_6_joined_means_the_run_stands_at_the_point_and_a_clash_says_how_far() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    let ty = pipe_type(&mut job);
    // The pipe is on no system here (so nothing says what it carries): the nearest open connection of its sort is named.
    let cold = pipe_to(&mut job, &level, &ty, 10.0 - 1.0 / 3.0, 24.0, 20.75, "22 in");
    assert_eq!(open_names(&mut job, &thing), ["Hot water", "Waste"], "the cold connection is joined by standing at the pipe's end");
    let stood = ed::connections_standing(&mut job, &thing).unwrap();
    assert_eq!(stood[0].joined_to, [cold.clone()]);
    assert!(stood[1].joined_to.is_empty());
    let found = building::clash::clashes(&mut job, None).unwrap();
    assert!(found.in_the_same_space().is_empty(), "joined, so not a clash: {:?}", found.found.iter().map(|c| (&c.a, &c.b)).collect::<Vec<_>>());
    // Step 4: the hot pipe ends 3 in in front of the hot connection, inside the lavatory.
    let hot = pipe_to(&mut job, &level, &ty, 10.0 + 1.0 / 3.0, 17.0, 20.5, "22 in");
    assert_eq!(open_names(&mut job, &thing), ["Hot water", "Waste"], "3 in away joins nothing: nothing is joined because it is near");
    let found = building::clash::clashes(&mut job, None).unwrap();
    let hard = found.in_the_same_space();
    assert_eq!(hard.len(), 1, "{:?}", found.found.iter().map(|c| (&c.a, &c.b, c.set_against)).collect::<Vec<_>>());
    let words = building::clash::connection_words(&mut job, hard[0]).expect("words for a run that ends in a thing with an open connection of its sort");
    assert!(words.contains("Hot water connection is 3 in away"), "{words}");
    // connect is refused, naming the connection, the 3 in and the way.
    let why = ed::connect(&mut job, &thing, &hot, ASSUMED).expect_err("refused");
    assert!(why.contains("Hot water") && why.contains("3 in") && why.contains("in front"), "{why}");
    assert_eq!(open_names(&mut job, &thing), ["Hot water", "Waste"], "the hot connection is still open");
    // Where the thing has no connection of the run's sort, connect does what it does today: a duct to the lavatory.
    let dt = ed::segment_type(&mut job, "duct 6 in", Kind::Duct, "6 in", None, ASSUMED).unwrap();
    let duct = ed::runs(&mut job, &level, Kind::Duct, &dt, &[[10.0, 24.0], [10.0, 20.0]], "20 in", None, ASSUMED).unwrap().remove(0);
    let which = ed::connect(&mut job, &thing, &duct, ASSUMED).expect("a duct is joined as it always was");
    assert_eq!(which, 4);
}

/// `connect` on a run that already ends at an open connection of the item says it is joined by standing there.
#[test]
fn point_6_a_run_that_ends_at_a_connection_is_already_joined() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    let ty = pipe_type(&mut job);
    let waste = pipe_to(&mut job, &level, &ty, 10.0, 24.0, 20.75, "18 in");
    let why = ed::connect(&mut job, &thing, &waste, ASSUMED).expect_err("nothing to add");
    assert!(why.contains("Waste") && why.contains("joined"), "{why}");
}

// ---------------------------------------------------------------- point 5: follow

/// Step 5: the item's height typed as 36 in: the thing follows, its cold connection is 24 in up (2 ft), 2 in above where the cold
/// pipe ends; the reply says 1 thing set down follows and 1 run now stands off its connection; takeoff names the cold connection
/// open again. Typed back to 34 in: joined again.
#[test]
fn point_5_a_thing_follows_its_item_and_a_run_that_is_left_is_counted() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    let ty = pipe_type(&mut job);
    let cold = pipe_to(&mut job, &level, &ty, 10.0 - 1.0 / 3.0, 24.0, 20.75, "22 in");
    let hot = pipe_to(&mut job, &level, &ty, 10.0 + 1.0 / 3.0, 17.0, 20.5, "22 in");
    assert_eq!(open_names(&mut job, &thing), ["Hot water", "Waste"]);
    let done = ed::item_size(&mut job, &item, "height", "36 in", "").unwrap();
    let said = done.said();
    assert!(said.contains("1 thing set down from it follows"), "{said}");
    assert!(said.contains("1 run now stands off its connection"), "{said}");
    assert!(said.contains(&cold) || said.contains("Cold water"), "{said}");
    // The hot pipe was never joined: it stood 3 in off its connection, and the connection went up 2 in, so it still stands off, by another amount.
    assert!(said.contains("1 run still stands off its connection") && said.contains(&hot), "{said}");
    let stood = ed::connections_standing(&mut job, &thing).unwrap();
    assert!(near(stood[0].at, [9.0 + 2.0 / 3.0, 20.75, 2.0]), "{:?}", stood[0].at);
    assert!(near(stood[2].at, [10.0, 20.75, 20.0 / 12.0]), "waste: 36 - 16 = 20 in: {:?}", stood[2].at);
    assert_eq!(open_names(&mut job, &thing), ["Cold water", "Hot water", "Waste"]);
    // Typed back to 34 in: joined again.
    let back = ed::item_size(&mut job, &item, "height", "34 in", "").unwrap();
    // The cold pipe is joined again; the hot one, which stood off all along, still does and is said so.
    assert!(back.said().contains("is now joined to its connection") && back.said().contains(&cold), "{}", back.said());
    assert_eq!(open_names(&mut job, &thing), ["Hot water", "Waste"]);
    // The run was not moved.
    let e = job.find(&cold).unwrap();
    assert!((job.world.get::<Segment>(e).unwrap().offset.feet().unwrap() - 22.0 / 12.0).abs() < 1e-9);
}

/// A size typed that would put a connection outside the item's box is refused, naming the item, the connection and how far, and
/// nothing changes (here the item's height typed as 14 in: the cold connection would be at 2 in, the waste below the foot).
#[test]
fn point_5_a_size_that_leaves_a_connection_outside_the_box_is_refused() {
    let (mut job, level, item) = with_rows();
    ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    let before = file::save(&mut job);
    let why = ed::item_size(&mut job, &item, "height", "14 in", "").expect_err("refused");
    for must in ["Test lavatory", "Waste", "Up", "below", "2 in"] {
        assert!(why.contains(must), "{why:?} lacks {must:?}");
    }
    assert_eq!(file::save(&mut job), before);
}

/// When the item's rows change, the connections that came from the item are worked again on each thing set down from it. A
/// connection a run put on the thing (a name the item has not got) is left alone, and so is one a person measured.
#[test]
fn point_5_when_a_row_changes_what_came_from_it_follows_and_what_did_not_is_left() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    // A duct ends inside the lavatory and is joined by hand: the item has no duct connection, so the thing gets one of its own.
    let dt = ed::segment_type(&mut job, "duct 6 in", Kind::Duct, "6 in", None, ASSUMED).unwrap();
    let extra = ed::runs(&mut job, &level, Kind::Duct, &dt, &[[10.0, 24.0], [10.0, 20.0]], "20 in", None, ASSUMED).unwrap().remove(0);
    ed::connect(&mut job, &thing, &extra, ASSUMED).unwrap();
    assert_eq!(connectors(&mut job, &thing).len(), 4);
    // Cold water row moved to 8 in up.
    let done = ed::connection_set(&mut job, &item, "Cold water", "up", "8 in").unwrap();
    assert_eq!(done.followed.len(), 1);
    assert!(done.said().contains("1 thing set down from it follows"), "{}", done.said());
    let c = connectors(&mut job, &thing);
    assert_eq!(c.len(), 4);
    assert_eq!(c[0].at.value.unwrap()[2], 8.0, "cold water is now 8 in up");
    assert_eq!(c[3].of_item, None, "the duct's own connection is left alone");
    assert_eq!(c[3].at.value.unwrap()[2], 20.0, "{:?}", c[3].at);
    // A row added is added to what is set down, at the end; a row taken away takes its connection off the thing.
    let row = ed::connection_row("Vent", ConnectorKind::Pipe, Some(Domain::Vent), Flow::Out, "0", "0", "height - 4 in", Facing::Up, None, TYPICAL).unwrap();
    ed::connection_add(&mut job, &item, row).unwrap();
    assert_eq!(connectors(&mut job, &thing).len(), 5);
    let took = ed::connection_take(&mut job, &item, "Hot water").unwrap();
    assert!(took.said().contains("1 thing set down from it follows"), "{}", took.said());
    let names: Vec<String> = connectors(&mut job, &thing).into_iter().map(|k| k.name).collect();
    assert_eq!(names, ["Cold water", "Waste", "", "Vent"], "the order of the others is kept, the new one is last");
    // One a person measured stays where it was measured, and is said.
    ed::measure(&mut job, &thing, "connection 1 (Cold water)", ed::Reading::Length("(0 in, 9 in, 10 in)"), None, "tape").ok();
    let (mut j2, l2, i2) = with_rows();
    let t2 = ed::place_item(&mut j2, &l2, &i2, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    let e = j2.find(&t2).unwrap();
    let mut c2 = j2.world.get::<Connectors>(e).cloned().unwrap();
    c2.0[0].at.status = Status::Known;
    c2.0[0].at.source = "measured with a tape".to_string();
    j2.put(&t2, c2).unwrap();
    let done = ed::item_size(&mut j2, &i2, "height", "36 in", "").unwrap();
    let k = connectors(&mut j2, &t2);
    assert_eq!(k[0].at.value.unwrap()[2], 22.0, "the measured place stays");
    assert_eq!(k[1].at.value.unwrap()[2], 24.0, "the others follow");
    assert!(done.said().contains("Cold water") && done.said().contains("measured"), "{}", done.said());
}

// ---------------------------------------------------------------- step 6: turned

/// Step 6: turned a quarter turn (90 degrees, counter-clockwise from east, as `turn` takes it), its connections turn with it:
/// cold (9.25, 19.6667, 1.8333), hot (9.25, 20.3333, 1.8333), waste (9.25, 20, 1.5). The lead's cold figure is right.
#[test]
fn step_6_its_connections_turn_with_it() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    ed::turn(&mut job, &thing, 90.0, ASSUMED).unwrap();
    let stood = ed::connections_standing(&mut job, &thing).unwrap();
    assert!(near(stood[0].at, [9.25, 19.0 + 2.0 / 3.0, 22.0 / 12.0]), "{:?}", stood[0].at);
    assert!(near(stood[1].at, [9.25, 20.0 + 1.0 / 3.0, 22.0 / 12.0]), "{:?}", stood[1].at);
    assert!(near(stood[2].at, [9.25, 20.0, 1.5]), "{:?}", stood[2].at);
    // The other way round.
    ed::turn(&mut job, &thing, -180.0, ASSUMED).unwrap();
    let stood = ed::connections_standing(&mut job, &thing).unwrap();
    assert!(near(stood[0].at, [10.75, 20.0 + 1.0 / 3.0, 22.0 / 12.0]), "{:?}", stood[0].at);
}

// ---------------------------------------------------------------- point 8: the tables

/// Step 8: the rows written out as a table and brought into a new job with the same item: the two tables alike, byte for byte.
#[test]
fn step_8_the_table_comes_back_as_it_went() {
    let (mut job, _, _) = with_rows();
    let out = tables::connections_out(&mut job);
    assert_eq!(out[0], tables::CONNECTION_HEADINGS.iter().map(|h| h.to_string()).collect::<Vec<_>>());
    assert_eq!(out.len(), 4);
    assert_eq!(out[1][..4], ["Test lavatory", "Cold water", "pipe", "cold"]);
    let (mut other, _, _) = job_with_item();
    let brought = tables::connections_in(&mut other, &out, None).unwrap();
    assert_eq!(brought.done.len(), 1, "{:?}", brought.done);
    assert!(brought.done[0].contains("Test lavatory") && brought.done[0].contains("3 connections"), "{:?}", brought.done);
    let again = tables::connections_out(&mut other);
    assert_eq!(out, again);
    // The same table again changes nothing.
    let same = tables::connections_in(&mut other, &out, None).unwrap();
    assert_eq!((same.done.len(), same.same), (0, 1));
    // An item the job has not got is not used, and says so.
    let (mut empty, _) = (Job::new("Empty (not a real job)", "test", "test"), ());
    let none = tables::connections_in(&mut empty, &out, None).unwrap();
    assert_eq!(none.not_used.len(), 3);
    assert!(none.not_used[0].1.contains("Test lavatory") && none.not_used[0].1.contains("no item"), "{:?}", none.not_used);
}

/// An item table brought in again, with no rows in it, changes nothing about the rows an item has.
#[test]
fn step_8_a_chest_that_lacks_rows_changes_nothing() {
    let (mut job, _, _) = with_rows();
    let before = file::save(&mut job);
    let csv = "Item,Trade,Kind,Width,Depth,Height,How sure,Where from\nTest lavatory,15 Mechanical,fixture,24 in,18 in,34 in,Assumed,a made-up size\n";
    tables::items_in(&mut job, &plumbline_world::building::shape::csv_rows(csv), None).unwrap();
    assert_eq!(tables::connections_out(&mut job).len(), 4);
    assert!(file::save(&mut job).contains("Cold water"));
    let _ = before;
}

/// A row that cannot be read leaves its item's rows as they were, and says which row and why; the other items' rows are still used.
#[test]
fn step_8_a_row_that_cannot_be_used_says_why() {
    let (mut job, _, _) = job_with_item();
    let head = "Item,Connection,Takes,Carries,Goes,Across,Back,Up,Faces,Place: how sure,Place: where from,Size,Size: how sure,Size: where from\n";
    let rows = |body: &str| plumbline_world::building::shape::csv_rows(&format!("{head}{body}"));
    // A row that cannot be read: the item's other rows are left out with it, and say so.
    let unread = "Test lavatory,Cold water,pipe,cold,in,-4 in,0,height - 12 in,back,Assumed,a typical rough-in,1/2 in,Assumed,a typical rough-in\nTest lavatory,Drain,pipe,waste,out,0,0,10 in,sideways,Assumed,a typical rough-in,,,\n";
    let got = tables::connections_in(&mut job, &rows(unread), None).unwrap();
    assert!(got.done.is_empty(), "{:?}", got.done);
    assert_eq!(got.not_used.len(), 2, "{:?}", got.not_used);
    assert!(got.not_used.iter().any(|(n, why)| *n == 3 && why.contains("sideways")), "{:?}", got.not_used);
    assert!(got.not_used.iter().any(|(n, why)| *n == 2 && why.contains("row 3")), "{:?}", got.not_used);
    assert_eq!(tables::connections_out(&mut job).len(), 1, "nothing was kept");
    // A place outside the box, said in inches and with the item and the connection.
    let outside = "Test lavatory,Vent,pipe,vent,out,0,0,height + 2 in,up,Assumed,a typical rough-in,,,\n";
    let got = tables::connections_in(&mut job, &rows(outside), None).unwrap();
    assert!(got.done.is_empty());
    assert!(got.not_used.len() == 1 && got.not_used[0].1.contains("Test lavatory") && got.not_used[0].1.contains("Vent") && got.not_used[0].1.contains("2 in above"), "{:?}", got.not_used);
    // A known place is refused: a connection is never Known on a table's say-so.
    let known = "Test lavatory,Vent,pipe,vent,out,0,0,10 in,up,Known,measured by a person who is not here,,,\n";
    let got = tables::connections_in(&mut job, &rows(known), None).unwrap();
    assert!(got.done.is_empty() && got.not_used[0].1.contains("Known"), "{:?}", got.not_used);
}

/// The starter rows: only for the starter items the program has a source for, each Assumed, "a typical rough-in; the product is
/// not chosen"; brought into a job whose starter items are there they add rows and say so; the other starter items have none.
#[test]
fn point_8_the_starter_rows_are_assumed_and_only_where_a_source_gives_them() {
    let mut job = Job::new("Starter (not a real job)", "test", "test");
    tables::items_in(&mut job, &plumbline_world::building::shape::csv_rows(tables::STARTER_ITEMS), None).unwrap();
    let got = tables::connections_in(&mut job, &plumbline_world::building::shape::csv_rows(tables::STARTER_CONNECTIONS), None).unwrap();
    assert!(!got.done.is_empty(), "{:?}", got);
    assert!(got.not_used.is_empty(), "{:?}", got.not_used);
    let out = tables::connections_out(&mut job);
    assert!(out.len() > 1);
    let h = |name: &str| out[0].iter().position(|c| c == name).unwrap();
    for row in &out[1..] {
        assert_eq!(row[h("Place: how sure")], "Assumed", "{row:?}");
        assert!(row[h("Place: where from")].contains("a typical rough-in; the product is not chosen"), "{row:?}");
        if !row[h("Size")].is_empty() {
            assert_eq!(row[h("Size: how sure")], "Assumed", "{row:?}");
        }
    }
    // The four fixtures the contract names are among them.
    for item in ["Water closet", "Lavatory"] {
        assert!(out[1..].iter().any(|r| r[0] == item), "{item} has rows");
    }
    // Every row, worked at its item's own size, is on or within its box (the starter is held to its own rule).
    assert!(ed::connections_outside_their_boxes(&mut job).is_empty());
}

// ---------------------------------------------------------------- the by-hand verbs

/// A row changed by hand: each says what it did, in the item's size and inches; a change that fails is refused and the job is as
/// it was.
#[test]
fn the_verbs_say_what_they_did_in_inches_and_a_refusal_changes_nothing() {
    let (mut job, _, item) = with_rows();
    let done = ed::connection_set(&mut job, &item, "waste", "up", "height - 20 in").unwrap();
    let said = done.said();
    assert!(said.contains("Waste") && said.contains("up 14 in"), "{said}");
    assert!(said.contains("Nothing is set down from it"), "{said}");
    let before = file::save(&mut job);
    let why = ed::connection_set(&mut job, &item, "Waste", "up", "height + 1 in").expect_err("outside");
    assert!(why.contains("Waste") && why.contains("above"), "{why}");
    assert_eq!(file::save(&mut job), before);
    let none = ed::connection_set(&mut job, &item, "Overflow", "up", "4 in").expect_err("there is none");
    assert!(none.contains("Overflow") && none.contains("Cold water"), "{none}");
    let took = ed::connection_take(&mut job, &item, "Hot water").unwrap();
    assert!(took.said().contains("2 connections"), "{}", took.said());
    // The last one taken away leaves the item with none, and no key.
    ed::connection_take(&mut job, &item, "Cold water").unwrap();
    ed::connection_take(&mut job, &item, "Waste").unwrap();
    assert!(!file::save(&mut job).contains("item_connections"));
}

/// `check` can name an item whose rows stand outside its box (a job whose item was made smaller outside this program): it still
/// opens, and `connections_outside_their_boxes` names the item and the connection.
#[test]
fn a_job_that_holds_a_row_outside_its_box_still_opens_and_is_named() {
    let (mut job, _, _) = with_rows();
    let text = file::save(&mut job);
    // The lavatory 10 in high in the file: its rows reach 12 in and 16 in below the top.
    let tall = "\"value\": [\n          {\n            \"Inches\"";
    let _ = tall;
    let (mut back, _) = file::load(&text).expect("opens");
    let e = back.find("type-0001").or_else(|| {
        let mut q = back.world.query::<(&Node, &Item)>();
        q.iter(&back.world).map(|(n, _)| n.path.clone()).next().and_then(|p| back.find(&p))
    });
    let e = e.expect("the item");
    let mut it = back.world.get::<Item>(e).cloned().unwrap();
    it.height = len("10 in", Status::Assumed);
    back.world.entity_mut(e).insert(it);
    let outside = ed::connections_outside_their_boxes(&mut back);
    assert_eq!(outside.len(), 1, "{outside:?}");
    assert!(outside[0].1.contains("Test lavatory") && outside[0].1.contains("Waste") || outside[0].1.contains("Cold water"), "{outside:?}");
    // A saved job always opens and saves again as it was.
    let again = file::save(&mut back);
    assert!(file::load(&again).is_ok());
}

// ---------------------------------------------------------------- round 2

/// One way of knowing joined: the network's node. `takeoff`'s open list and `connections_standing` give the same answer for every
/// connection when a run's end stands at the point, a run passes over it, an end is 3 in off, and nothing is near.
#[test]
fn round_2_joined_is_one_answer_for_the_open_list_and_the_function() {
    // (what is drawn: from y to y at x of the hot connection, 10.3333 ft), and which of the three are joined by it (cold, hot, waste)
    let cases: [(&str, Option<(f64, f64)>, [bool; 3]); 4] = [
        ("an end at the point", Some((24.0, 20.75)), [false, true, false]),
        ("a run passing over it", Some((24.0, 20.5)), [false, true, false]),
        ("an end 3 in off", Some((17.0, 20.5)), [false, false, false]),
        ("nothing", None, [false, false, false]),
    ];
    for (what, pipe, want) in cases {
        let (mut job, level, item) = with_rows();
        let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
        let ty = pipe_type(&mut job);
        if let Some((from, to)) = pipe {
            pipe_to(&mut job, &level, &ty, 10.0 + 1.0 / 3.0, from, to, "22 in");
        }
        let open = open_names(&mut job, &thing);
        let stood = ed::connections_standing(&mut job, &thing).unwrap();
        let mine: Vec<String> = { let mut v: Vec<String> = stood.iter().filter(|s| s.joined_to.is_empty()).map(|s| s.name.clone()).collect(); v.sort(); v };
        assert_eq!(open, mine, "{what}: takeoff's open list and connections_standing differ");
        let joined: Vec<bool> = stood.iter().map(|s| !s.joined_to.is_empty()).collect();
        assert_eq!(joined, want, "{what}");
        // And the takeoff says the same, by name.
        let said: Vec<String> = building::takeoff(&mut job).iter().flat_map(|s| s.joined.open.iter().map(|o| o.said.clone())).collect();
        for s in &stood {
            assert_eq!(said.iter().any(|t| t.contains(&format!("its {} connection", s.name))), s.joined_to.is_empty(), "{what}: {}", s.name);
        }
    }
}

/// A run passing over a connection is joined for `connect`'s refusal too: nothing is open there to be refused for.
#[test]
fn round_2_a_run_passing_over_a_connection_is_not_refused_as_off_it() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    let ty = pipe_type(&mut job);
    let over = pipe_to(&mut job, &level, &ty, 10.0 + 1.0 / 3.0, 24.0, 20.5, "22 in");
    let why = ed::connect(&mut job, &thing, &over, ASSUMED);
    // Cold and waste are open and the pipe ends nearer cold than waste: refused naming an open one, never the one it passes over.
    let why = why.expect_err("it ends at none of the open connections");
    assert!(!why.contains("Hot water"), "{why}");
}

/// An item renamed: the connections on what is set down from it stay the item's, and a size typed after the rename still moves them.
#[test]
fn round_2_an_item_renamed_keeps_its_connections_and_they_follow_after() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    ed::rename_item(&mut job, &item, "Lavatory, 20 in").unwrap();
    let c = connectors(&mut job, &thing);
    assert_eq!(c.len(), 3);
    assert_eq!(c.iter().map(|k| k.of_item.clone().unwrap()).collect::<Vec<_>>(), ["Cold water", "Hot water", "Waste"]);
    for k in &c {
        assert!(k.at.source.contains("Lavatory, 20 in") && !k.at.source.contains("Test lavatory"), "the source names the item as it is called now: {}", k.at.source);
    }
    // The rows are the item's still, and a change of size after the rename moves the thing's connections.
    let done = ed::item_size(&mut job, &item, "height", "36 in", "").unwrap();
    assert!(done.said().contains("1 thing set down from it follows"), "{}", done.said());
    let stood = ed::connections_standing(&mut job, &thing).unwrap();
    assert!(near(stood[0].at, [9.0 + 2.0 / 3.0, 20.75, 2.0]), "{:?}", stood[0].at);
    let c = connectors(&mut job, &thing);
    assert!(c[0].at.source.contains("Lavatory, 20 in"));
    // And by the new name a row is changed and the thing follows.
    ed::connection_set(&mut job, "Lavatory, 20 in", "Waste", "up", "6 in").unwrap();
    assert_eq!(connectors(&mut job, &thing)[2].at.value.unwrap()[2], 6.0);
}

/// An item deleted: today the program refuses while things are set down from it ("take them out of the job first"), so no
/// connection is ever left that is nobody's; that is held, and so is what follows once they are out.
#[test]
fn round_2_an_item_with_things_set_down_is_not_deleted_and_nothing_is_left_dangling() {
    let (mut job, level, item) = with_rows();
    let thing = ed::place_item(&mut job, &level, &item, [10.0, 20.0], "0 ft", KNOWN).unwrap();
    let before = file::save(&mut job);
    let why = ed::remove(&mut job, &item).expect_err("refused while a thing is set down from it");
    assert!(why.contains("set down from") && why.contains(&item), "{why}");
    assert_eq!(file::save(&mut job), before, "the job is as it was");
    // Take the thing out, then the item: nothing carries a connection of an item that is not there.
    ed::remove(&mut job, &thing).unwrap();
    ed::remove(&mut job, &item).unwrap();
    let dangling = {
        let mut q = job.world.query::<&Connectors>();
        q.iter(&job.world).filter(|c| c.0.iter().any(|k| k.of_item.is_some())).count()
    };
    assert_eq!(dangling, 0);
    assert!(ed::connections_outside_their_boxes(&mut job).is_empty());
    assert!(!file::save(&mut job).contains("item_connections"));
    assert!(file::load(&file::save(&mut job)).is_ok());
}
