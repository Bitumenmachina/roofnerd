//! Call 200, points 1 to 3 (contract 200P): the file says what is connected, the way IFC says it. Each
//! connection of a thing set down is a port on it (`IfcDistributionPort`, nested by `IfcRelNests`), each
//! end of a run is a port, two that the model holds as joined are tied by `IfcRelConnectsPorts`; the
//! reader joins what the file's ports say is joined, by the change the Connect tool makes, and nothing
//! else. The figures are worked in STATUS.md ("The file says what is connected").

use std::collections::BTreeMap;

use plumbline_ifc::{import, write::export};
use plumbline_world::building::{self, Connectors};
use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::value::Status;
use plumbline_world::*;

const WHEN: &str = "2026-01-31T09:30:00";
const HOW: Entered = Entered { status: Status::Assumed, source: "its own model" };

fn written() -> (Job, String) {
    let mut job = sample::house_priced();
    let w = export(&mut job, "house.ifc", WHEN).unwrap();
    (job, w.text)
}

fn brought(text: &str) -> (Job, plumbline_ifc::Brought) {
    let mut back = Job::new("brought in (not a real job)", "test", "test");
    let b = import(&mut back, text.as_bytes(), "house.ifc", HOW).unwrap();
    back.derive();
    (back, b)
}

fn lines_of<'a>(text: &'a str, class: &str) -> Vec<&'a str> {
    let key = format!("={class}(");
    text.lines().filter(|l| l.contains(&key)).collect()
}

/// Open ends by the kind of system, as `takeoff` counts them (what a person reads): an end is said on its
/// run's system, a connection on the system that carries what it says.
fn open_ends(job: &mut Job) -> BTreeMap<String, usize> {
    job.derive();
    let mut out = BTreeMap::new();
    for t in building::takeoff(job).into_iter().filter(|t| !t.classification.is_empty()) {
        *out.entry(t.classification.clone()).or_insert(0) += t.joined.open.len();
    }
    out
}

/// Every connection of every fixture, piece of equipment and fitting: its thing's name, what it says it
/// is, and where (in the thing's own axes, in feet).
fn connections(job: &mut Job, kinds: &[Kind]) -> Vec<(String, String, String, Option<[f64; 3]>)> {
    let mut q = job.world.query::<(&Node, &Kind, &Connectors)>();
    let mut v = Vec::new();
    for (n, k, c) in q.iter(&job.world) {
        if !kinds.contains(k) {
            continue;
        }
        for x in &c.0 {
            v.push((n.name.clone(), format!("{:?}", x.kind), format!("{} {:?} {:?}", x.name, x.domain, x.flow), x.at_ft()));
        }
    }
    v.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.cmp(&b.2)));
    v
}

fn count(job: &mut Job, kind: Kind) -> usize {
    let mut q = job.world.query::<&Kind>();
    q.iter(&job.world).filter(|k| **k == kind).count()
}

/// A fixture, a pipe that ends at it and one that ends 0.5 ft short of it, in a made-up job.
fn small(second_run_off_by: f64, connected: bool) -> (Job, String) {
    let known = Entered { status: Status::Known, source: "made-up plan" };
    let mut job = Job::new("small (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", known).unwrap();
    let cold = ed::system(&mut job, "Cold water", "Domestic Cold Water").unwrap();
    let t = ed::segment_type(&mut job, "3/4 in", Kind::Pipe, "3/4 in", None, known).unwrap();
    let thing = ed::place(&mut job, &level, Kind::Fixture, "Water closet", [10.0, 10.0], "0 ft", Some([1.5, 2.0, 2.0]), Some(&cold), known).unwrap();
    // The box is 1.5 by 2 ft: its east face is at x = 10.75. One run ends in it, one ends short of it.
    let first = ed::runs(&mut job, &level, Kind::Pipe, &t, &[[10.5, 10.0], [10.5, 4.0]], "1 ft", Some(&cold), known).unwrap();
    ed::runs(&mut job, &level, Kind::Pipe, &t, &[[10.75 + second_run_off_by, 10.0], [10.75 + second_run_off_by, 4.0]], "1 ft", Some(&cold), known).unwrap();
    if connected {
        ed::connect(&mut job, &thing, &first[0], known).unwrap();
    }
    job.derive();
    let text = export(&mut job, "small.ifc", WHEN).unwrap().text;
    (job, text)
}

/// Point 1, written. A port for each connection of a thing set down or a fitting, two for each run (one at
/// each end); each nested in what it is on, so each is the related object of one `IfcRelNests`.
#[test]
fn w1_each_connection_and_each_end_of_a_run_is_a_port_nested_in_what_it_is_on() {
    let (mut job, text) = written();
    // (Two elbows of the house have no size, so no body: they are written with their place alone (call 204) and their two connections each are not.)
    let on_things = connections(&mut job, &[Kind::Fixture, Kind::Equipment, Kind::Fitting]).len() - 4;
    let runs: usize = [Kind::Duct, Kind::Pipe, Kind::Conduit, Kind::Cable].iter().map(|k| count(&mut job, *k)).sum();
    let ports = lines_of(&text, "IFCDISTRIBUTIONPORT");
    // Every run is written now (3 pipes with no size were left out before): 2 ports on each.
    assert_eq!(ports.len(), on_things + 2 * runs, "ports");
    let nests = lines_of(&text, "IFCRELNESTS");
    let nested: usize = nests.iter().map(|l| l.matches('#').count() - 2).sum();
    assert_eq!(nested, ports.len(), "each port is nested once");
    // A port says which way it goes, what sort it is and what system it is on, by the standard's words.
    assert!(ports.iter().any(|l| l.contains(".SOURCE.") && l.contains(".PIPE.") && l.contains(".DOMESTICHOTWATER.")), "{:?}", &ports[..3]);
    assert!(ports.iter().any(|l| l.contains(".SINK.") && l.contains(".DUCT.") && l.contains(".VENTILATION.")));
    assert!(ports.iter().any(|l| l.contains(".CABLE.") && l.contains(".ELECTRICAL.")));
}

/// Point 1, written: two that the model holds as joined are tied once, by `IfcRelConnectsPorts`; no port
/// is tied twice in one direction (the schema allows one tie each way) and a port is never tied to itself.
#[test]
fn w2_two_ports_the_model_holds_as_joined_are_tied_and_none_twice() {
    let (_, text) = written();
    let ties = lines_of(&text, "IFCRELCONNECTSPORTS");
    assert!(ties.len() >= 100, "{}", ties.len());
    let mut relating = std::collections::HashSet::new();
    let mut related = std::collections::HashSet::new();
    for t in &ties {
        let ids: Vec<&str> = t.split('#').skip(2).map(|x| x.split(|c: char| !c.is_ascii_digit()).next().unwrap()).collect();
        assert_ne!(ids[0], ids[1], "{t}");
        assert!(relating.insert(ids[0].to_string()), "tied twice as the relating port: {t}");
        assert!(related.insert(ids[1].to_string()), "tied twice as the related port: {t}");
    }
}

/// Point 3 of the call: a run with no size is written as the run it is (its line, no body), and comes back
/// with its size Unknown, counted; the hot-water system has the runs it had.
#[test]
fn w3_a_run_with_no_size_is_written_and_comes_back_unknown_and_counted() {
    let (mut job, text) = written();
    let mut unsized_first = Vec::new();
    {
        let mut q = job.world.query::<(&Node, &Kind, &Segment)>();
        for (n, k, s) in q.iter(&job.world) {
            if *k == Kind::Pipe && s.type_name.value.is_none() {
                unsized_first.push(n.name.clone());
            }
        }
    }
    assert_eq!(unsized_first.len(), 3, "{unsized_first:?}");
    for name in &unsized_first {
        let l = text.lines().find(|l| l.contains("=IFCPIPESEGMENT(") && l.contains(&format!("'{name}'"))).unwrap_or_else(|| panic!("{name} is not in the file"));
        // No body: only its line.
        assert!(l.contains(",$,.RIGIDSEGMENT.)"), "{l}");
    }
    let (mut back, b) = brought(&text);
    assert_eq!(count(&mut back, Kind::Pipe), count(&mut job, Kind::Pipe), "{:#?}", b.left_out);
    let mut unknown = 0;
    let mut q = back.world.query::<(&Node, &Kind, &Segment)>();
    for (n, k, s) in q.iter(&back.world) {
        if *k == Kind::Pipe && s.type_name.value.is_none() {
            unknown += 1;
            assert_eq!(s.type_name.status, Status::Unknown, "{}", n.name);
            assert!(unsized_first.contains(&n.name), "{}", n.name);
        }
    }
    assert_eq!(unknown, 3);
    assert!(b.notes.iter().any(|n| n.starts_with("3 runs have no size")), "{:#?}", b.notes);
}

/// A connection on a thing comes back as the thing's connection at its place, with what it said it was.
#[test]
fn r1_a_connection_on_a_thing_comes_back_at_its_place_assumed_and_sourced() {
    let (mut job, text) = written();
    let (mut back, _) = brought(&text);
    let (a, c) = (connections(&mut job, &[Kind::Fixture, Kind::Equipment]), connections(&mut back, &[Kind::Fixture, Kind::Equipment]));
    assert_eq!(a.len(), c.len());
    for (x, y) in a.iter().zip(&c) {
        assert_eq!((&x.0, &x.1, &x.2), (&y.0, &y.1, &y.2));
        let (p, q) = (x.3.unwrap(), y.3.unwrap());
        assert!((0..3).all(|i| (p[i] - q[i]).abs() < 2.0e-3), "{}: {p:?} came back {q:?}", x.0);
    }
    let mut seen = 0;
    let mut qq = back.world.query::<(&Node, &Kind, &Connectors)>();
    for (n, k, cs) in qq.iter(&back.world).filter(|(_, k, _)| matches!(k, Kind::Fixture | Kind::Equipment)) {
        for x in &cs.0 {
            seen += 1;
            assert_eq!(x.at.status, Status::Assumed, "{} {k:?}", n.name);
            assert!(x.at.source.contains("house.ifc") && x.at.source.contains("connection point"), "{}: {}", n.name, x.at.source);
        }
    }
    assert_eq!(seen, 46, "the fixtures' and equipment's connections");
}

/// No end is joined that the file does not say is joined: a made-up file with a run ending 0.5 ft from a
/// fixture and no port on the fixture reads that end open, and says so. (The file has ports, the runs' own:
/// so it says what is connected, and the fixture it says nothing of is connected to nothing.)
#[test]
fn r2_a_run_ending_half_a_foot_from_a_fixture_with_no_port_reads_open_and_says_so() {
    let (_, text) = small(0.5, false);
    assert!(text.contains("=IFCDISTRIBUTIONPORT("));
    let (mut back, b) = brought(&text);
    let open = open_ends(&mut back);
    // Run 1: both ends open (the file ties nothing to the fixture); run 2: both ends open.
    assert_eq!(open.get("Domestic Cold Water"), Some(&4), "{open:?}");
    let said = b.notes.iter().find(|n| n.contains("0.50 ft")).unwrap_or_else(|| panic!("{:#?}", b.notes));
    assert!(said.contains("Water closet") && said.contains("no connection"), "{said}");
}

/// The same made-up file with its ports taken out is a file with no ports: it reads as it did (the end
/// inside the box is joined, with today's note; the end 0.5 ft off is open, and is said).
#[test]
fn r2b_a_file_with_no_ports_joins_what_stands_in_the_box_and_says_the_other_is_open() {
    let (_, text) = small(0.5, false);
    let bare: String = text.lines().filter(|l| !l.contains("=IFCDISTRIBUTIONPORT(") && !l.contains("=IFCRELNESTS(") && !l.contains("=IFCRELCONNECTSPORTS(")).map(|l| format!("{l}\n")).collect();
    let (mut back, b) = brought(&bare);
    let open = open_ends(&mut back);
    assert_eq!(open.get("Domestic Cold Water"), Some(&3), "{open:?}");
    assert!(b.notes.iter().any(|n| n.contains("joined to it, where it stood")), "{:#?}", b.notes);
    assert!(b.notes.iter().any(|n| n.contains("0.50 ft") && n.contains("Water closet")), "{:#?}", b.notes);
}

/// A file with no ports reads as it does today: the ends in a thing's box are joined, with today's note.
#[test]
fn r3_a_file_with_no_ports_reads_as_it_did() {
    let (_, text) = written();
    let bare: String = text.lines().filter(|l| !l.contains("=IFCDISTRIBUTIONPORT(") && !l.contains("=IFCRELNESTS(") && !l.contains("=IFCRELCONNECTSPORTS(")).map(|l| format!("{l}\n")).collect();
    let (mut back, b) = brought(&bare);
    let open = open_ends(&mut back);
    assert_eq!(open.get("Supply Air"), Some(&13), "{open:?}");
    assert!(b.notes.iter().any(|n| n.contains("joined to it, where it stood")), "{:#?}", b.notes);
}

/// A tie in the file between a thing's port and a run's end, where the run's end is not at the port, joins
/// nothing, and the reply says so.
#[test]
fn r4_a_tie_to_a_run_that_does_not_reach_the_port_joins_nothing() {
    let (_, text) = small(0.5, true);
    let id = |l: &str| l.split('=').next().unwrap().trim_start_matches('#').parse::<u64>().unwrap();
    let tie = lines_of(&text, "IFCRELCONNECTSPORTS");
    assert_eq!(tie.len(), 1, "the fixture's one connection is tied to the run that ends at it: {tie:?}");
    // As the file stands: the connection and its run's end are one place, and the run's end is joined.
    let (mut back, b) = brought(&text);
    assert_eq!(open_ends(&mut back).get("Domestic Cold Water"), Some(&3), "{:#?}", b.notes);
    assert!(b.notes.iter().any(|n| n.contains("1 connection on fixtures and equipment was brought in")), "{:#?}", b.notes);
    // The same tie pointed at the far run's end (0.75 ft off): not joined, and said.
    let far = lines_of(&text, "IFCDISTRIBUTIONPORT").into_iter().filter(|l| l.contains("'first end'")).map(id).max().unwrap();
    let parts: Vec<&str> = tie[0].split(',').collect();
    let moved = tie[0].replace(&format!("{},$);", parts[5]), &format!("#{far},$);"));
    let changed = text.replace(tie[0], &moved);
    assert_ne!(changed, text);
    let (mut again, b) = brought(&changed);
    assert!(b.notes.iter().any(|n| n.contains("tied in the file to the end of a run that is not at it, and not joined")), "{:#?}", b.notes);
    // (The connection is still where the file puts it, which is where the first run ends: that end is at it, and
    // the model has the two at one place. Nothing was joined for the tie.)
    assert_eq!(open_ends(&mut again).get("Domestic Cold Water"), Some(&3), "{:#?}", b.notes);
}

/// A fitting whose kind the runs make is not what the file calls it: it is brought in as what the runs make,
/// and the reply names the place and both words (three of them in the house: a "bend" where three ends meet; the
/// third, at the water heater, was two runs short before the three runs with no size came in).
#[test]
fn f1_a_fitting_the_runs_make_another_kind_names_the_place_and_both_words() {
    let (_, text) = written();
    let (_, b) = brought(&text);
    let said: Vec<&String> = b.notes.iter().filter(|n| n.contains("the file calls it a bend") && n.contains("the runs there make a tee")).collect();
    assert_eq!(said.len(), 3, "{:#?}", b.notes);
    assert!(said.iter().any(|n| n.contains("(16.3, 26.5")), "{said:#?}");
    assert!(said.iter().any(|n| n.contains("(19, 17")), "{said:#?}");
    assert!(said.iter().any(|n| n.contains("(21.2, 27")), "{said:#?}");
}

/// Everything brought in stays Assumed and sourced to the file; the same file twice sets nothing down twice.
#[test]
fn r5_the_same_model_twice_makes_no_connection_twice() {
    let (_, text) = written();
    let mut job = Job::new("twice (not a real job)", "test", "test");
    import(&mut job, text.as_bytes(), "house.ifc", HOW).unwrap();
    let once = connections(&mut job, &[Kind::Fixture, Kind::Equipment]).len();
    let _ = import(&mut job, text.as_bytes(), "house.ifc", HOW);
    job.derive();
    assert_eq!(connections(&mut job, &[Kind::Fixture, Kind::Equipment]).len(), once);
}
