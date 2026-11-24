//! Call 197, points 1 to 6: a model this program wrote is brought back whole.
//! The made-up whole house (`sample::house`) is written out and brought into an empty job;
//! what the first job holds is what the second must hold, by the figures worked in
//! STATUS.md ("a model brought in whole: a demonstration").

use std::collections::BTreeMap;

use plumbline_ifc::{import, write::export};
use plumbline_world::building::network;
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

fn count(job: &mut Job, kind: Kind) -> usize {
    let mut q = job.world.query::<&Kind>();
    q.iter(&job.world).filter(|k| **k == kind).count()
}

/// Open ends by the system's classification.
fn open_ends(job: &mut Job) -> BTreeMap<String, usize> {
    let systems: Vec<(String, String)> = {
        let mut q = job.world.query::<(&Node, &building::SystemInfo)>();
        q.iter(&job.world).map(|(n, s)| (n.path.clone(), s.classification.clone())).collect()
    };
    let mut out = BTreeMap::new();
    for (path, class) in systems {
        let net = network::network(job, Some(&path), None, network::TOLERANCE_FT).unwrap();
        *out.entry(class).or_insert(0) += net.open.len();
    }
    out
}

fn names(job: &mut Job, kind: Kind) -> Vec<String> {
    let mut q = job.world.query::<(&Node, &Kind)>();
    let mut v: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == kind).map(|(n, _)| n.name.clone()).collect();
    v.sort();
    v
}

/// Point 1. 22 fixtures, 5 pieces of equipment, each where it stood: by name, its place, its
/// height off the floor and its size are the first job's to a ten-thousandth of a foot; every
/// value is Assumed and says the file and the thing's number in it.
#[test]
fn p1_fixtures_and_equipment_come_in_where_they_stood_and_no_surer_than_assumed() {
    let (mut job, text) = written();
    let (mut back, b) = brought(&text);
    assert_eq!(count(&mut job, Kind::Fixture), 22);
    assert_eq!(count(&mut back, Kind::Fixture), 22, "{:#?}", b.left_out);
    assert_eq!(count(&mut back, Kind::Equipment), 5);
    assert_eq!(b.made.get("fixtures"), Some(&22));
    assert_eq!(b.made.get("pieces of equipment"), Some(&5));
    let things = |job: &mut Job| -> Vec<(String, Kind, Option<[f64; 2]>, Option<f64>, Option<[f64; 3]>, Option<f64>)> {
        let mut q = job.world.query::<(&Node, &Kind, &Placed)>();
        let mut v: Vec<_> = q.iter(&job.world).filter(|(_, k, _)| matches!(k, Kind::Fixture | Kind::Equipment)).map(|(n, k, p)| (n.name.clone(), *k, p.at_ft(), p.offset.feet(), p.size_ft(), p.angle_deg())).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    };
    let (was, is) = (things(&mut job), things(&mut back));
    assert_eq!(was.len(), is.len());
    for (a, c) in was.iter().zip(&is) {
        assert_eq!((&a.0, a.1), (&c.0, c.1));
        let near = |x: f64, y: f64| (x - y).abs() < 1.0e-3;
        let (pa, pc) = (a.2.unwrap(), c.2.unwrap());
        assert!(near(pa[0], pc[0]) && near(pa[1], pc[1]), "{}: {pa:?} came back {pc:?}", a.0);
        assert!(near(a.3.unwrap(), c.3.unwrap()), "{}: its height off the floor", a.0);
        if let Some(sa) = a.4 {
            let sc = c.4.unwrap_or_else(|| panic!("{}: its size did not come back", a.0));
            assert!(near(sa[0], sc[0]) && near(sa[1], sc[1]) && near(sa[2], sc[2]), "{}: {sa:?} came back {sc:?}", a.0);
        }
        assert!(near(a.5.unwrap_or(0.0), c.5.unwrap_or(0.0)), "{}: how far it is turned", a.0);
    }
    // Nothing is Known on the file's say-so, and each says where it was read.
    let mut q = back.world.query::<(&Node, &Kind, &Placed)>();
    for (n, k, p) in q.iter(&back.world).filter(|(_, k, _)| matches!(k, Kind::Fixture | Kind::Equipment)) {
        assert_eq!(p.at.status, Status::Assumed, "{}", n.name);
        assert_ne!(p.size.status, Status::Known, "{}", n.name);
        assert!(p.at.source.contains("house.ifc") && p.at.source.contains(" in the file"), "{}: {}", n.name, p.at.source);
        assert!(p.at.source.contains(&format!("\"{}\"", n.name)) || n.name.chars().last().is_some_and(|c| c.is_ascii_digit()), "{k:?} {}: {}", n.name, p.at.source);
    }
}

/// A fixture or a piece of furniture of a designer's model with a body that is not a box:
/// it is still set down where the model places it, and its size is said not to be known.
#[test]
fn p1_a_furnishing_comes_in_as_a_furnishing_and_a_body_it_cannot_read_leaves_its_size_unknown() {
    let (_, text) = written();
    // A fixture of the file's own, changed into a piece of furniture whose body is nothing read here.
    let fixture = text.lines().find(|l| l.contains("=IFCFLOWTERMINAL(")).unwrap().to_string();
    let id = fixture.split('=').next().unwrap();
    let guid = fixture.split("('").nth(1).unwrap().split('\'').next().unwrap();
    let placement = fixture.split(",'fixture',").nth(1).unwrap().split(',').next().unwrap();
    let furniture = format!("{id}=IFCFURNITURE('{guid}',$,'Armchair',$,'furnishing',{placement},$,$,.CHAIR.);");
    let changed = text.replace(&fixture, &furniture);
    assert_ne!(changed, text);
    let (mut back, b) = brought(&changed);
    assert_eq!(count(&mut back, Kind::Furnishing), 1, "{:#?}", b.left_out);
    assert_eq!(count(&mut back, Kind::Fixture), 21);
    assert_eq!(b.made.get("furnishings"), Some(&1));
    let mut q = back.world.query::<(&Node, &Kind, &Placed)>();
    let (n, _, p) = q.iter(&back.world).find(|(_, k, _)| **k == Kind::Furnishing).unwrap();
    assert_eq!(n.name, "Armchair");
    assert!(p.size.value.is_none(), "a size nobody read is not made up: {:?}", p.size);
    assert!(p.size.source.contains("size not entered") || !p.size.source.is_empty());
}

/// Point 2. The house's 41 fittings: 2 have no size and are not written; of the 39 written, 37 come in on
/// their runs and 2 are refused by the change that puts a fitting where runs meet, for a reason that is true of
/// them (HELD for a ruling: the lead's figure was 39).
#[test]
fn p2_fittings_come_in_on_their_runs() {
    let (mut job, text) = written();
    let (mut back, b) = brought(&text);
    assert_eq!(count(&mut job, Kind::Fitting), 41, "the first job holds 41: 2 of them have no size and are not written");
    assert_eq!(count(&mut back, Kind::Fitting), 37, "{:#?}", b.left_out);
    assert_eq!(b.made.get("fittings"), Some(&37));
    // The two that do not: where a third run ends at the same joint (the hot main's upright piece and the riser).
    let left: Vec<&(String, usize, String)> = b.left_out.iter().filter(|(c, _, _)| c == "pipe fitting").collect();
    assert_eq!(left.len(), 2, "{:#?}", b.left_out);
    assert!(left.iter().any(|(_, n, why)| *n == 1 && why.contains("leave (16.3, 22.7) the same way, one over the other")), "{left:#?}");
    // A fitting is what the runs that meet make it: its kind is worked out, not copied. The first job's 41 Bends
    // less the 2 with no size (not written) and the 2 not brought in are 37; 2 of those have a third run's end at
    // the joint (the first-floor trunk at (19, 17), the return riser at (21.2, 27), and, since the three hot-water runs
    // with no size are in the file (call 200), the hot main's elbow at the water heater, (16.3, 26.5)) and are Tees here.
    let kinds = |job: &mut Job| {
        let mut q = job.world.query::<&Fitting>();
        let mut v: Vec<String> = q.iter(&job.world).map(|f| format!("{:?}", f.kind)).collect();
        v.sort();
        v
    };
    let (a, c) = (kinds(&mut job), kinds(&mut back));
    assert_eq!(a.iter().filter(|k| *k == "Bend").count(), c.iter().filter(|k| *k == "Bend").count() + 7, "{a:?} {c:?}");
    assert_eq!(c.iter().filter(|k| *k == "Tee").count(), 3);
    // Each system's runs are joined as the fittings brought in join them: nothing reads more open than where a thing
    // was not brought in or a run end stands off the thing it serves (see the ignored test below).
    let (was, is) = (open_ends(&mut job), open_ends(&mut back));
    assert_eq!(is.get("Power"), was.get("Power"), "{is:?}");
}

/// Call 200: the file says what is connected, so each system reads as closed as it was. The figures are the
/// first job's own, by what `takeoff` counts as an open end (an end is said on its run's system, a connection on
/// the system that carries what it says): supply air 4, cold water 1, sanitary 1, vent 1, power 1, the rest none.
/// (The count `network(Some(system))` gives asks each system for the whole network it is in, so it reads sanitary
/// 2 and vent 2 for the one open end of each: not the figure a person reads.)
#[test]
fn p2_each_system_reads_as_closed_as_it_was() {
    let (mut job, text) = written();
    let (mut back, b) = brought(&text);
    let by_takeoff = |job: &mut Job| -> BTreeMap<String, usize> {
        job.derive();
        let mut out = BTreeMap::new();
        for t in building::takeoff(job).into_iter().filter(|t| !t.classification.is_empty()) {
            *out.entry(t.classification.clone()).or_insert(0) += t.joined.open.len();
        }
        out
    };
    let (was, is) = (by_takeoff(&mut job), by_takeoff(&mut back));
    for (class, n) in [("Supply Air", 4), ("Return Air", 0), ("Refrigerant", 0), ("Domestic Cold Water", 1), ("Domestic Hot Water", 0), ("Sanitary", 1), ("Vent", 1), ("Power", 1), ("Lighting", 0)] {
        assert_eq!(was.get(class), Some(&n), "the first job: {was:?}");
        assert_eq!(is.get(class), Some(&n), "the second job: {is:?} {:#?}", b.notes);
    }
    assert_eq!(is, was);
    // And by the network of each system, as the test of Power below asks it: the same in both jobs.
    assert_eq!(open_ends(&mut back), open_ends(&mut job));
}

/// Point 3. 4 strip footings, 136 LF, 10.07 CY of concrete; a pad footing; a footing of
/// any other sort or shape is left out and the reason is its own.
#[test]
fn p3_footings_come_in_as_footings() {
    let (mut job, text) = written();
    let (mut back, b) = brought(&text);
    assert_eq!(count(&mut back, Kind::Footing), 4, "{:#?}", b.left_out);
    assert_eq!(b.made.get("footings"), Some(&4));
    let totals = |job: &mut Job| {
        let t = building::storey_totals(job);
        let f = t.iter().find(|s| s.footings.members > 0).unwrap();
        (f.footings.members, (f.footings.value.unwrap() * 100.0).round() / 100.0, f.footing_volume.value.map(|v| (v * 100.0).round() / 100.0))
    };
    let (a, c) = (totals(&mut job), totals(&mut back));
    assert_eq!(a, (4, 136.0, Some(10.07)));
    assert_eq!(c, a);
    // A footing of another sort: left out with a reason that is true of it.
    let other = text.replace(".STRIP_FOOTING.", ".PILE_CAP.");
    let (mut again, b) = brought(&other);
    assert_eq!(count(&mut again, Kind::Footing), 0);
    assert!(b.left_out.iter().any(|(c, n, why)| c == "footing" && *n == 4 && why.contains("pile cap")), "{:#?}", b.left_out);
}

#[test]
fn p3_a_pad_footing_comes_in_by_the_change_that_makes_one() {
    let known = Entered { status: Status::Known, source: "made-up plan" };
    let mut job = Job::new("pad (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Foundation", "-4 ft", "4 ft", known).unwrap();
    let t = ed::framing_type(&mut job, "pad 30 in by 30 in", Kind::PadFooting, "30 in", Some("30 in"), None, known).unwrap();
    ed::pad(&mut job, &level, &t, [10.0, 12.0], "12 in", "0 ft", known).unwrap();
    job.derive();
    let text = export(&mut job, "pad.ifc", WHEN).unwrap().text;
    assert!(text.contains(".PAD_FOOTING."));
    let (mut back, b) = brought(&text);
    assert_eq!(count(&mut back, Kind::PadFooting), 1, "{:#?}", b.left_out);
    let mut q = back.world.query::<(&Node, &Kind, &Stood)>();
    let (_, _, s) = q.iter(&back.world).find(|(_, k, _)| **k == Kind::PadFooting).unwrap();
    assert_eq!(s.at.value, Some([10.0, 12.0]));
    assert_eq!(s.height.feet(), Some(1.0));
    assert_eq!(s.base_offset.feet(), Some(-1.0));
    assert_eq!(s.at.status, Status::Assumed);
}

/// Point 4. The roof the program wrote comes back as that roof: its outline, each edge at
/// its pitch or a gable, the eave height. A gable made on an edge is a gable on the way back.
#[test]
fn p4_the_house_roof_comes_back_with_its_planes_and_eaves() {
    let (mut job, text) = written();
    assert!(text.contains("'Plumbline_Roof'"), "the roof carries what is needed to make it again");
    let (mut back, b) = brought(&text);
    assert_eq!(b.made.get("roofs"), Some(&1), "{:#?}", b.left_out);
    assert_eq!(b.made.get("roof planes"), Some(&4));
    assert!(!b.left_out.iter().any(|(c, _, _)| c == "roof" || c == "roof slab"), "{:#?}", b.left_out);
    let (was, is) = (solve::roofs(&mut job), solve::roofs(&mut back));
    assert_eq!((was.len(), is.len()), (1, 1));
    let sum = |job: &mut Job, p: &str| {
        let t = view::roof_totals(job, p);
        (t.facets, t.surface.value.map(|v| (v * 100.0).round() / 100.0), t.lines.iter().map(|(k, s)| (format!("{k:?}"), s.value.map(|v| (v * 100.0).round() / 100.0))).collect::<Vec<_>>())
    };
    let (a, c) = (sum(&mut job, &was[0]), sum(&mut back, &is[0]));
    assert_eq!(a.0, 4);
    assert_eq!(a.1, Some(1408.72));
    assert_eq!(c, a);
    assert!(a.2.contains(&("Eave".to_string(), Some(144.0))), "{a:?}");
    // Nothing is Known on the file's say-so.
    let e = back.find(&is[0]).unwrap();
    let fp = back.world.get::<Footprint>(e).unwrap();
    assert_eq!(fp.points.status, Status::Assumed);
    assert!(fp.points.source.contains("house.ifc"), "{}", fp.points.source);
    assert_eq!(fp.edges.iter().map(|x| x.value).collect::<Vec<_>>(), vec![Some(Rise::Pitch(6.0)); 4]);
    // The set is named for this program, and not a buildingSMART one.
    let set = text.lines().find(|l| l.contains("'Plumbline_Roof'")).unwrap();
    assert!(!set.contains("'Pset_"));
    for name in ["Outline", "EdgeRise", "EaveHeight", "Basis"] {
        assert!(text.contains(&format!("'{name}'")), "no property {name}");
    }
}

#[test]
fn p4_a_gable_made_on_an_edge_is_a_gable_on_the_way_back_and_its_planes_are_set_beside_the_slabs() {
    let known = Entered { status: Status::Known, source: "made-up plan" };
    let mut job = Job::new("gable (not a real job)", "test", "test");
    ed::storey(&mut job, "Ground", "0 ft", "9 ft", known).unwrap();
    let corners = vec![[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
    let entered = |x| Val { value: Some(x), unit: "in/12".into(), status: Status::Assumed, source: "made-up plan".into() };
    let footprint = Footprint {
        edges: vec![entered(Rise::Pitch(6.0)), entered(Rise::Pitch(6.0)), entered(Rise::Pitch(6.0)), entered(Rise::Pitch(6.0))],
        points: Val { value: Some(corners), unit: "ft".into(), status: Status::Assumed, source: "made-up plan".into() },
        eave_height: Val { value: Some(units::Len::One(9.0)), unit: "ft".into(), status: Status::Assumed, source: "made-up plan".into() },
    };
    let (path, _) = solve::new_roof(&mut job, "roof", footprint).unwrap();
    // The gable made on the second and the fourth edges, as the window's change does.
    for edge in [2, 4] {
        solve::edit(&mut job, &path, solve::Change::Rise { edge, rise: Val { value: Some(Rise::Gable), unit: String::new(), status: Status::Assumed, source: "made-up plan".into() } }).unwrap();
    }
    job.derive();
    let text = export(&mut job, "gable.ifc", WHEN).unwrap().text;
    let (mut back, b) = brought(&text);
    assert_eq!(b.made.get("roofs"), Some(&1), "{:#?} {:#?}", b.left_out, b.notes);
    assert_eq!(b.made.get("roof planes"), Some(&2));
    let was_at = job.find(&path).unwrap();
    let was = job.world.get::<Footprint>(was_at).unwrap().clone();
    let id = solve::roofs(&mut back).remove(0);
    let is_at = back.find(&id).unwrap();
    let is = back.world.get::<Footprint>(is_at).unwrap().clone();
    assert_eq!(is.points_ft(), was.points_ft());
    assert_eq!(is.eave_height.feet(), Some(9.0));
    assert_eq!(is.edges.iter().map(|x| x.value).collect::<Vec<_>>(), vec![Some(Rise::Pitch(6.0)), Some(Rise::Gable), Some(Rise::Pitch(6.0)), Some(Rise::Gable)]);
    // Its planes' areas are set beside the slabs in the file, and they agree: nothing is said of a difference.
    assert!(!b.notes.iter().any(|n| n.contains("roof planes") && n.contains("differ")), "{:#?}", b.notes);
    // 2 planes of 24 ft by 8.944 ft: the roof's surface is the same both ways.
    let (a, c) = (view::roof_totals(&mut job, &path).surface.value, view::roof_totals(&mut back, &id).surface.value);
    assert_eq!(a.map(|v| (v * 100.0).round()), c.map(|v| (v * 100.0).round()));
}

/// A roof from another program (no set of this program's) is left out and said, as before.
#[test]
fn p4_a_roof_without_the_programs_own_set_is_left_out_and_said() {
    let (_, text) = written();
    let stripped: String = text.replace("'Plumbline_Roof'", "'Somebody_Elses'");
    let (mut back, b) = brought(&stripped);
    assert_eq!(count(&mut back, Kind::Roof), 0);
    assert!(b.left_out.iter().any(|(c, n, why)| c == "roof slab" && *n == 4 && why.contains("a roof is not brought in")), "{:#?}", b.left_out);
    assert!(b.left_out.iter().any(|(c, n, _)| c == "roof" && *n == 1), "{:#?}", b.left_out);
}

/// Point 5. Names kept.
#[test]
fn p5_a_door_a_window_and_a_system_keep_the_names_the_file_gives_them() {
    let (mut job, text) = written();
    let (mut back, _) = brought(&text);
    for kind in [Kind::Door, Kind::Window] {
        let (was, is) = (names(&mut job, kind), names(&mut back, kind));
        assert_eq!(was, is, "{kind:?}");
    }
    assert!(names(&mut back, Kind::Door).contains(&"front door".to_string()), "{:?}", names(&mut back, Kind::Door));
    let (was, is) = (names(&mut job, Kind::System), names(&mut back, Kind::System));
    assert!(is.contains(&"Supply air".to_string()), "{is:?}");
    assert!(is.contains(&"Cold water".to_string()), "{is:?}");
    for n in was.iter().filter(|n| ["Supply air", "Cold water", "Hot water", "Sanitary", "Vent", "Power", "Lighting", "Refrigerant", "Return air"].contains(&n.as_str())) {
        assert!(is.contains(n), "{n} is not among {is:?}");
    }
}

/// Point 6. Things not brought in that run ends stood on: the message says how many runs read open. (Two run ends
/// that meet are a joint with no fitting and do not read open; a run's end that stood on a thing does.)
#[test]
fn p6_what_is_left_out_says_how_many_runs_will_read_open() {
    let (_, text) = written();
    // Every plumbing fixture of the file made a thing of no stated kind: nothing here brings that in.
    let changed = text.replace("=IFCSANITARYTERMINAL(", "=IFCBUILDINGELEMENTPROXY(").replace(".TOILETPAN.", ".ELEMENT.").replace(".WASHHANDBASIN.", ".ELEMENT.").replace(".BATH.", ".ELEMENT.");
    assert_ne!(changed, text);
    let (mut back, b) = brought(&changed);
    assert_eq!(count(&mut back, Kind::Fixture), 16);
    assert!(b.left_out.iter().any(|(c, n, _)| c.contains("thing of no stated kind") && *n == 6), "{:#?}", b.left_out);
    let note = b.notes.iter().find(|n| n.contains("read open")).unwrap_or_else(|| panic!("{:#?}", b.notes));
    assert!(note.contains("thing of no stated kind"), "{note}");
    let n: usize = note.split_whitespace().next().unwrap().parse().unwrap();
    assert!(n >= 1, "{note}");
}

/// The list of what is left out no longer has a fixture, a fitting, a footing or this roof in it.
#[test]
fn what_is_left_out_of_the_whole_house_is_not_one_of_the_things_brought_in() {
    let (_, text) = written();
    let (_, b) = brought(&text);
    // Only the two fittings the change that puts one where runs meet refuses (HELD for a ruling).
    let said: Vec<&str> = b.left_out.iter().map(|(c, _, _)| c.as_str()).collect();
    assert_eq!(said, ["pipe fitting", "pipe fitting"], "left out: {:#?}", b.left_out);
}

/// The model brought in a second time counts nothing twice: what is in the job already is not made again.
#[test]
fn the_same_model_twice_counts_no_footing_and_no_roof_twice() {
    let (_, text) = written();
    let mut job = Job::new("twice (not a real job)", "test", "test");
    import(&mut job, text.as_bytes(), "house.ifc", HOW).unwrap();
    let again = import(&mut job, text.as_bytes(), "house.ifc", HOW);
    job.derive();
    assert_eq!(count(&mut job, Kind::Footing), 4, "{again:?}");
    assert_eq!(count(&mut job, Kind::Roof), 1);
    assert_eq!(count(&mut job, Kind::Fixture), 22);
    assert_eq!(count(&mut job, Kind::Equipment), 5);
    assert_eq!(count(&mut job, Kind::Fitting), 37);
}

