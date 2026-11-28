use plumbline_world::value::Status;
use plumbline_world::*;

#[allow(dead_code)]
mod common;
use common::{measure_of, path_named, opening_type, pieces, length_of_pieces, ft, named, piping, piece, thing, net, kinds};

// ---------------------------------------------------------------- the rest of the building

#[test]
fn the_sample_house_saves_and_loads_to_the_same_bytes_and_nothing_is_stale() {
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let (mut again, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty(), "stale: {:?}", report.stale);
    assert_eq!(file::save(&mut again), text);
}

#[test]
fn a_wall_is_its_length_by_its_height_less_its_openings() {
    let mut job = plumbline_world::sample::house();
    // The first floor's south wall: 40 ft long and 10 ft tall is 400 SF. A 3 ft by 6.67 ft
    // door is 20.01 SF and two 5 ft by 5 ft windows are 50: 70.01 SF of openings, 329.99 net.
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    assert_eq!(measure_of(&mut job, &south, "plan_length").value, Some(40.0));
    assert_eq!(measure_of(&mut job, &south, "face_area").value, Some(400.0));
    assert_eq!(measure_of(&mut job, &south, "opening_area").value, Some(70.01));
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!((net.value, net.status), (Some(329.99), Status::Known));
    // A wall with no openings has an opening area of zero, said as zero, and Known.
    let wet = job.children().get(&first).unwrap().get("wet wall").unwrap().clone();
    let none = measure_of(&mut job, &wet, "opening_area");
    assert_eq!((none.value, none.status), (Some(0.0), Status::Known));
    assert_eq!(measure_of(&mut job, &wet, "net_area").value, Some(90.0));
}

#[test]
fn an_opening_that_does_not_fit_its_wall_leaves_no_net_area() {
    use plumbline_world::Opening;
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let len = |x: f64| Val::known(Len::One(x), "ft", "test");
    // A window 4 ft wide centred 40 ft along a 40 ft wall runs past its end.
    let four_by_three = opening_type(&mut job, Kind::Window, len(4.0), len(3.0));
    let w = job.add(&south, Kind::Window, "too far").unwrap();
    job.put(&w, Opening { type_name: four_by_three, offset: len(40.0), sill: len(3.0) }).unwrap();
    job.derive();
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!((net.value, net.status), (None, Status::Unknown));
    assert!(net.note.contains("ends 42 ft along a wall 40 ft long"), "{}", net.note);  // call 226: a length in a sentence is said as a builder says it
    // One that lies over another: 2 ft wide centred at 7 ft, over the west window (5 to 10 ft).
    let two_by_three = opening_type(&mut job, Kind::Window, len(2.0), len(3.0));
    job.put(&w, Opening { type_name: two_by_three, offset: len(7.0), sill: len(3.0) }).unwrap();
    job.derive();
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!(net.value, None);
    assert!(net.note.contains("lie over each other"), "{}", net.note);
    // One whose width nobody has: no opening area, so no net area, and the gross stands.
    let no_width = opening_type(&mut job, Kind::Window, Val::unknown("ft", "not on the plan"), len(3.0));
    job.put(&w, Opening { type_name: no_width, offset: len(13.0), sill: len(3.0) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "opening_area").value, None);
    assert_eq!(measure_of(&mut job, &south, "net_area").value, None);
    assert_eq!(measure_of(&mut job, &south, "face_area").value, Some(400.0));
}

#[test]
fn a_run_is_straight_pieces_measured_in_space_with_an_elbow_at_each_corner() {
    use plumbline_world::{Fitting, FittingKind};
    let mut job = plumbline_world::sample::house();
    // The first-floor trunk, west, is drawn up 3.4 ft, south 8 ft, west 17 ft. It is
    // three straight pieces of those lengths, 28.4 LF in all, with two elbows.
    let trunk = pieces(&mut job, "first-floor trunk, west");
    let lengths: Vec<Option<f64>> = trunk.iter().map(|p| measure_of(&mut job, p, "true_length").value).collect();
    assert_eq!(lengths, [Some(3.4), Some(8.0), Some(17.0)]);
    assert_eq!(length_of_pieces(&mut job, "first-floor trunk, west"), (28.4, Status::Known));
    let elbows: Vec<String> = plumbline_world::view::elements(&mut job).into_iter().filter(|e| e.name.starts_with("first-floor trunk, west, elbow")).map(|e| e.path).collect();
    assert_eq!(elbows.len(), 2);
    for path in &elbows {
        let e = job.find(path).unwrap();
        assert_eq!(job.world.get::<Fitting>(e).unwrap().kind, FittingKind::Bend);
    }
    // It is 16 in by 8 in: 4 ft round, so the three pieces have 13.6 + 32 + 68 = 113.6 SF
    // of duct. The size is the type's, and Assumed.
    let skins: Vec<plumbline_world::Derived> = trunk.iter().map(|p| measure_of(&mut job, p, "surface_area")).collect();
    assert_eq!(skins.iter().map(|d| d.value).collect::<Vec<_>>(), [Some(13.6), Some(32.0), Some(68.0)]);
    assert!(skins.iter().all(|d| d.status == Status::Assumed));
    // A riser is a piece whose two ends are one point on the plan at two heights.
    let riser = measure_of(&mut job, &trunk[0], "true_length");
    assert!(riser.formula.contains("end_offset"), "{}", riser.formula);
    // A 6 in round branch 11 ft long: π × 0.5 × 11 = 17.2788 SF.
    let branch = path_named(&mut job, "branch to living west");
    assert_eq!(measure_of(&mut job, &branch, "surface_area").value, Some(17.2788));
    // A pipe with no size still has its length: 0.8 + 4.9 + 3.7 = 9.4 LF. No area and no
    // size is made up for it.
    assert_eq!(length_of_pieces(&mut job, "hot to the kitchen sink"), (9.4, Status::Known));
    // A piece that slopes: 3 ft along and 4 ft up is 5 ft of pipe.
    let (e, first) = (job.find(&trunk[1]).unwrap(), path_named(&mut job, "first floor"));
    let mut sloped = job.world.get::<plumbline_world::Segment>(e).unwrap().clone();
    sloped.line = Val::known([[0.0, 0.0], [3.0, 0.0]], "ft", "test");
    sloped.offset = ft(1.0);
    sloped.end_offset = Some(ft(5.0));
    let p = job.add(&first, Kind::Duct, "a sloped piece").unwrap();
    job.put(&p, sloped.clone()).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &p, "true_length").value, Some(5.0));
    // The same in inches and in feet and inches is the same 5 ft.
    sloped.line = Val::known([[0.0, 0.0], [36.0, 0.0]], "in", "test");
    sloped.end_offset = Some(Val::known(Len::FtIn([5.0, 0.0]), "ft-in", "test"));
    job.put(&p, sloped.clone()).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &p, "true_length").value, Some(5.0));
    // An end height nobody has leaves a sloped piece with no length; a level piece does
    // not need its height to have one.
    sloped.end_offset = Some(Val::unknown("ft", "not on the plan"));
    job.put(&p, sloped.clone()).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &p, "true_length").value, None);
    sloped.end_offset = None;
    sloped.offset = Val::unknown("ft", "not on the plan");
    job.put(&p, sloped).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &p, "true_length").value, Some(3.0));
}

#[test]
fn a_system_takeoff_groups_runs_by_size_and_says_what_it_cannot_size() {
    use plumbline_world::building::takeoff;
    let mut job = plumbline_world::sample::house();
    let systems = takeoff(&mut job);
    let hot = systems.iter().find(|s| s.name == "Hot water").unwrap();
    assert_eq!(hot.classification, "Domestic Hot Water");
    // Its runs fall on three lines: 1/2 in, 3/4 in, and the one with no size.
    let sizes: Vec<&str> = hot.lines.iter().map(|l| l.size.as_str()).collect();
    assert_eq!(sizes, ["1/2 in round", "3/4 in round", "size unknown"]);
    // The branch with no size is three straight pieces, 0.8 + 4.9 + 3.7 = 9.4 LF.
    let no_size = hot.lines.iter().find(|l| l.size == "size unknown").unwrap();
    assert_eq!((no_size.size_status, no_size.length.value, no_size.length.members), (Status::Unknown, Some(9.4), 3));
    // Hot to a lavatory is 1 + 0.9 = 1.9 LF and to a tub 1.5 + 1 = 2.5: two of each, 8.8 LF,
    // each of them two pieces.
    let half = hot.lines.iter().find(|l| l.size == "1/2 in round").unwrap();
    assert_eq!((half.length.value, half.length.members, half.length.status), (Some(8.8), 8, Status::Known));
    assert_eq!(hot.placed, vec![(Kind::Equipment, "Water heater, 50 gal".to_string(), 1, Status::Assumed)]);
    // Its elbows are counted by size: one at the corner of each of the four 1/2 in
    // branches, three on the 3/4 in main, and the two on the branch with no size are
    // counted and flagged, not sized by a guess.
    use plumbline_world::FittingKind::Bend;
    assert_eq!(
        hot.fittings,
        vec![(Some(Bend), "1/2 in elbow".to_string(), 4, Status::Known), (Some(Bend), "3/4 in elbow".to_string(), 3, Status::Known), (Some(Bend), "not said".to_string(), 2, Status::Unknown)]
    );
    // Every run and every placed thing in the job is on some line of some system.
    let rows = plumbline_world::view::elements(&mut job);
    let runs = rows.iter().filter(|e| e.kind.is_segment()).count();
    let placed = rows.iter().filter(|e| e.kind.is_placed()).count();
    assert_eq!(systems.iter().flat_map(|s| &s.lines).map(|l| l.length.members).sum::<usize>(), runs);
    assert_eq!(systems.iter().map(|s| s.placed.iter().map(|p| p.2).sum::<usize>() + s.fittings.iter().map(|f| f.2).sum::<usize>()).sum::<usize>(), placed);
}

#[test]
fn a_storey_totals_its_walls_and_floors_and_every_element_is_placed_or_accounted_for() {
    use plumbline_world::building::{placements, storey_totals};
    let mut job = plumbline_world::sample::house();
    let storeys = storey_totals(&mut job);
    assert_eq!(storeys.len(), 2);
    // Four outside walls (136 LF) and partitions of 40, 12, 12 and 10 LF: 210 LF a floor.
    assert_eq!(storeys[0].walls.value, Some(210.0));
    assert_eq!(storeys[0].floors.value, Some(1120.0));
    assert_eq!((storeys[0].doors, storeys[0].windows), (5, 8));
    assert_eq!((storeys[1].doors, storeys[1].windows), (4, 9));
    // Everything with a shape is drawn, but the two elbows on the branch that has no size:
    // nobody knows how big they are, and each is listed with that.
    let (drawn, not_drawn) = placements(&mut job);
    assert_eq!(not_drawn.len(), 2, "{not_drawn:?}");
    assert!(not_drawn.iter().all(|(_, why)| why == "its size is not known"), "{not_drawn:?}");
    let rows = plumbline_world::view::elements(&mut job);
    let shaped = rows.iter().filter(|e| matches!(e.kind, Kind::Wall | Kind::Floor) || e.kind.is_segment() || e.kind.is_placed()).count();
    assert_eq!(drawn.len() + not_drawn.len(), shaped);
    // Take a storey's elevation away and what stands on it is not drawn, and says why:
    // its walls and its floor, and every run and placed thing under it, whose heights
    // are above that storey.
    let first = path_named(&mut job, "first floor");
    let on_first = drawn.iter().filter(|p| p.storey.as_deref() == Some(first.as_str())).count();
    assert!(on_first > 9, "{on_first}");
    job.put(&first, plumbline_world::Level { elevation: Val::unknown("ft", "not surveyed"), height: Val::known(Len::One(10.0), "ft", "test"), computation_height: None }).unwrap();
    let (still, not_drawn) = placements(&mut job);
    // (The two elbows with no size are on the first floor too. A thing is listed once,
    // with the first thing that stops it being drawn.)
    assert_eq!(not_drawn.iter().filter(|(_, why)| why == "its level's elevation is not known").count(), on_first + 2);
    assert_eq!(not_drawn.len(), on_first + 2);
    assert!(still.iter().all(|p| p.storey.as_deref() != Some(first.as_str())));
    // The first floor's outside walls stood up to the second floor, whose elevation is
    // still known: nothing of theirs is drawn against a guessed base.
    assert_eq!(drawn.len() - still.len(), on_first);
}

#[test]
fn the_building_checks_refuse_what_cannot_be() {
    use plumbline_world::building::*;
    let len = |x: f64| Val::known(Len::One(x), "in", "test");
    // A section is round, by a diameter, or rectangular, by a width and a depth.
    assert!(check_segment_type("t", Kind::Type, &SegmentType { category: Kind::Pipe, width: len(1.0), depth: Some(len(2.0)), round: true, weight_per_foot: None }).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType { category: Kind::Duct, width: len(12.0), depth: None, round: false, weight_per_foot: None }).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType::round(Kind::Pipe, len(0.0))).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType::round(Kind::Wall, len(1.0))).is_err());
    assert!(check_segment_type("t", Kind::Pipe, &SegmentType::round(Kind::Pipe, len(1.0))).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType::round(Kind::Pipe, len(300.0))).is_err());
    assert!(check_segment_type("t", Kind::Type, &SegmentType::round(Kind::Pipe, Val::unknown("in", "not on the plan"))).is_ok());
    // A run goes from one point to another; a length with no unit, or a metric one, is refused.
    let piece = |line: [[f64; 2]; 2], unit: &str, rise: Option<f64>| Segment { type_name: named("type-0002"), line: Val::known(line, unit, "test"), offset: ft(1.0), end_offset: rise.map(ft), justify: Justify::Centre };
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [1.0, 0.0]], "ft", None)).is_ok());
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [0.0, 0.0]], "ft", None)).unwrap_err().contains("a run goes from one point to another"));
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [0.0, 0.0]], "ft", Some(1.0))).is_err(), "a riser that rises nothing");
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [0.0, 0.0]], "ft", Some(9.0))).is_ok(), "a riser");
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [1.0, 0.0]], "", None)).is_err());
    assert!(check_segment("r", Kind::Pipe, &piece([[0.0, 0.0], [1.0, 0.0]], "m", None)).is_err());
    assert!(check_segment("r", Kind::Wall, &piece([[0.0, 0.0], [1.0, 0.0]], "ft", None)).is_err());
    // A placed thing is turned by an angle in degrees, and has a size in all three.
    let thing = |angle: Option<Val<f64>>, size: [f64; 3]| Placed { what: named("a thing"), at: Val::known([1.0, 1.0], "ft", "test"), offset: ft(0.0), angle, size: Val::known(size, "ft", "test"), item: None };
    assert!(check_placed("p", Kind::Fixture, &thing(None, [1.0, 1.0, 1.0])).is_ok());
    assert!(check_placed("p", Kind::Fitting, &thing(Some(Val::known(90.0, "deg", "test")), [1.0, 1.0, 1.0])).is_ok());
    assert!(check_placed("p", Kind::Fixture, &thing(Some(Val::known(90.0, "", "test")), [1.0, 1.0, 1.0])).is_err(), "an angle with no unit");
    assert!(check_placed("p", Kind::Fixture, &thing(Some(Val::known(720.0, "deg", "test")), [1.0, 1.0, 1.0])).is_err());
    assert!(check_placed("p", Kind::Fixture, &thing(None, [1.0, 0.0, 1.0])).is_err());
    assert!(check_placed("p", Kind::Wall, &thing(None, [1.0, 1.0, 1.0])).is_err());
    // A system's classification is one the program knows, and belongs to its discipline.
    assert!(check_system("s", Kind::System, &SystemInfo { discipline: Discipline::Plumbing, classification: "Supply Air".into() }).is_err());
    assert!(check_system("s", Kind::System, &SystemInfo { discipline: Discipline::Mechanical, classification: "Steam".into() }).is_err());
    // A saved job that names a system that is not there does not load.
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let system = path_named(&mut job, "Vent");
    let broken = text.replace(&format!("\"value\": \"{system}\""), "\"value\": \"system-9999\"");
    assert_ne!(broken, text);
    let err = file::load(&broken).err().expect("a dangling system is refused");
    assert!(err.contains("which is not a system in this job"), "{err}");
}

// ---------------------------------------------------------------- the eighth judgment

#[test]
fn a_storey_stands_under_the_job_and_counts_only_what_is_nearest_it() {
    use plumbline_world::building::storey_totals;
    // A file with the second floor moved under the first does not load.
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let (first, second) = (path_named(&mut job, "first floor"), path_named(&mut job, "second floor"));
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    for entry in doc["data"].as_array_mut().unwrap() {
        let path = entry["path"].as_str().unwrap().to_string();
        if path == "job-0001" {
            entry["children"].as_object_mut().unwrap().remove("second floor");
        } else if path == first {
            entry["children"].as_object_mut().unwrap().insert("second floor".into(), second.clone().into());
        }
    }
    let err = file::load(&doc.to_string()).err().expect("a level under a level is refused");
    assert!(err.contains("has to be directly under the job"), "{err}");
    // Built in code, where nothing stops it, each wall still counts once, on its own storey.
    let e = job.find(&second).unwrap();
    job.world.get_mut::<plumbline_world::Node>(e).unwrap().parent = Some(first.clone());
    let totals = storey_totals(&mut job);
    assert_eq!((totals[0].walls.members, totals[0].walls.value), (8, Some(210.0)));
    assert_eq!((totals[1].walls.members, totals[1].floors.value), (8, Some(1120.0)));
}

#[test]
fn a_takeoff_is_no_surer_than_what_is_on_the_system_and_one_size_is_one_line() {
    use plumbline_world::building::takeoff;
    use plumbline_world::{InSystem, Segment, SegmentType};
    let mut job = plumbline_world::sample::house();
    // The refrigerant line set is four straight pieces: 2 + 8.5 + 19.5 + 4.5 = 34.5 LF.
    let line_set = pieces(&mut job, "refrigerant line set");
    assert_eq!(line_set.len(), 4);
    let system = path_named(&mut job, "Refrigerant");
    // Said to be on the system, but only assumed so: its line is Assumed, and so is the total.
    for p in &line_set {
        job.put(p, InSystem(Val::assumed(system.clone(), "", "guessed from where it runs"))).unwrap();
    }
    let t = takeoff(&mut job);
    let refrigerant = t.iter().find(|s| s.name == "Refrigerant").unwrap();
    assert_eq!((refrigerant.lines[0].length.value, refrigerant.lines[0].length.status), (Some(34.5), Status::Assumed));
    assert_eq!(refrigerant.runs.status, Status::Assumed);
    // On a system nobody can name: not "on no system". It is in a group of its own,
    // flagged Unknown, with its length; and the system it left has no runs.
    for p in &line_set {
        job.put(p, InSystem(Val::unknown("", "not marked on the plan"))).unwrap();
    }
    let t = takeoff(&mut job);
    assert_eq!(t.iter().find(|s| s.name == "Refrigerant").unwrap().runs.members, 0);
    assert!(t.iter().all(|s| s.name != "On no system"));
    let lost = t.iter().find(|s| s.name == "System not known").expect("a group for what is on an unknown system");
    assert_eq!((lost.flag, lost.path.as_str(), lost.runs.value, lost.runs.members), (Status::Unknown, "", Some(34.5), 4));
    // With no system entered at all it is on no system, and that is Known.
    for p in &line_set {
        let e = job.find(p).unwrap();
        job.world.entity_mut(e).remove::<InSystem>();
    }
    let t = takeoff(&mut job);
    let none = t.iter().find(|s| s.name == "On no system").unwrap();
    assert_eq!((none.flag, none.runs.value), (Status::Known, Some(34.5)));

    // One size is one line, however each type's was typed, and lines are in order of size.
    let mut job = plumbline_world::sample::house();
    let a = path_named(&mut job, "branch to living west");
    let b = path_named(&mut job, "branch to kitchen");
    for (path, width) in [(&a, Val::known(Len::One(0.5), "ft", "test")), (&b, Val::known(Len::FtIn([0.0, 6.0]), "ft-in", "test"))] {
        let t = job.add("job-0001", Kind::Type, &format!("six inches, typed another way {}", job.order.len())).unwrap();
        job.put(&t, SegmentType::round(Kind::Duct, width)).unwrap();
        let e = job.find(path).unwrap();
        let mut piece = job.world.get::<Segment>(e).unwrap().clone();
        piece.type_name = named(&t);
        job.put(path, piece).unwrap();
    }
    job.derive();
    let t = takeoff(&mut job);
    let supply = t.iter().find(|s| s.name == "Supply air").unwrap();
    let sizes: Vec<&str> = supply.lines.iter().map(|l| l.size.as_str()).collect();
    assert_eq!(sizes, ["6 in round", "12 in × 8 in", "14 in × 8 in", "16 in × 8 in"]);
    // Nine branches: five on the first floor of one piece each and four on the second
    // of two, thirteen pieces, 88.4 LF.
    assert_eq!((supply.lines[0].length.members, supply.lines[0].length.value), (13, Some(88.4)));
    // The two typed in feet are Known where the rest are Assumed: the line is still Assumed.
    assert_eq!(supply.lines[0].size_status, Status::Assumed);
    // A run whose type says a size nobody has is a run of no known size, like one with no type.
    let unsized_type = job.add("job-0001", Kind::Type, "a duct nobody has sized").unwrap();
    job.put(&unsized_type, SegmentType::round(Kind::Duct, Val::unknown("in", "not on the schedule"))).unwrap();
    let e = job.find(&a).unwrap();
    let mut piece = job.world.get::<Segment>(e).unwrap().clone();
    piece.type_name = named(&unsized_type);
    job.put(&a, piece).unwrap();
    job.derive();
    let t = takeoff(&mut job);
    let supply = t.iter().find(|s| s.name == "Supply air").unwrap();
    let last = supply.lines.last().unwrap();
    assert_eq!((last.size.as_str(), last.size_status, last.length.value, last.length.members), ("size unknown", Status::Unknown, Some(11.0), 1));
    assert_eq!(measure_of(&mut job, &a, "surface_area").value, None);
}

#[test]
fn an_opening_that_cannot_be_cut_is_said_and_a_listing_keeps_a_door_under_its_wall() {
    use plumbline_world::building::placements;
    use plumbline_world::Opening;
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let door = job.children().get(&south).unwrap().get("front door").unwrap().clone();
    let len = |x: f64| Val::known(Len::One(x), "ft", "test");
    // A door moved past the end of its wall is not cut, and is listed as not cut, with why.
    let door_entity = job.find(&door).unwrap();
    let door_type = job.world.get::<Opening>(door_entity).unwrap().type_name.clone();
    job.put(&door, Opening { type_name: door_type, offset: len(40.5), sill: len(0.0) }).unwrap();
    job.derive();
    let (drawn, not_drawn) = placements(&mut job);
    let wall = drawn.iter().find(|p| p.path == south).unwrap();
    match &wall.shape {
        plumbline_world::building::Shape::Wall { holes, .. } => assert!(holes.iter().all(|h| h.0 != door)),
        other => panic!("{other:?}"),
    }
    assert!(not_drawn.contains(&(door.clone(), format!("it runs past the end of {south}, so it is not cut in it"))), "{not_drawn:?}");
    // One whose width nobody has is said too.
    let no_width = opening_type(&mut job, Kind::Door, Val::unknown("ft", "not on the plan"), len(6.67));
    job.put(&door, Opening { type_name: no_width, offset: len(20.0), sill: len(0.0) }).unwrap();
    let (_, not_drawn) = placements(&mut job);
    let said: Vec<&(String, String)> = not_drawn.iter().filter(|x| x.0 == door).collect();
    assert_eq!(said.len(), 1);
    assert!(said[0].1.contains("is not known"), "{}", said[0].1);
    // A flat reading lists each door and window straight after its own wall's rows, one
    // level in, whenever it was added: the front door is under the south wall.
    let rows = plumbline_world::view::elements(&mut job);
    let at = rows.iter().position(|e| e.path == south).unwrap();
    assert_eq!((rows[at + 1].path.as_str(), rows[at + 1].depth), (door.as_str(), rows[at].depth + 1));
    assert!(rows[at + 1..at + 4].iter().all(|e| e.kind.is_opening()));
    assert_eq!(rows.len(), job.order.len());
}

// ---------------------------------------------------------------- the re-base on BIMCraft: walls, openings, floors and their types

#[test]
fn a_wall_held_to_the_storey_above_is_as_tall_as_the_storeys_are_apart() {
    use plumbline_world::{Level, Wall};
    let mut job = plumbline_world::sample::house();
    let (first, second) = (path_named(&mut job, "first floor"), path_named(&mut job, "second floor"));
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    // The sample's first-floor outside walls run up to the second floor: 10 ft − 0 ft.
    let h = measure_of(&mut job, &south, "height");
    assert_eq!((h.value, h.status), (Some(10.0), Status::Known));
    assert!(h.inputs.contains(&format!("{second}#pl::level")), "{:?}", h.inputs);
    // Raise the second floor a foot and the wall is 11 ft tall and 440 SF, with nothing
    // typed on the wall. The door and two windows are still 70.01 SF: 369.99 net.
    job.put(&second, Level { elevation: ft(11.0), height: ft(9.0), computation_height: None }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "height").value, Some(11.0));
    assert_eq!(measure_of(&mut job, &south, "face_area").value, Some(440.0));
    assert_eq!(measure_of(&mut job, &south, "net_area").value, Some(369.99));
    // An elevation somebody assumed makes the height, the face and the net Assumed.
    job.put(&second, Level { elevation: Val::assumed(Len::One(11.0), "ft", "scaled off a section"), height: ft(9.0), computation_height: None }).unwrap();
    job.derive();
    for name in ["height", "face_area", "net_area"] {
        assert_eq!(measure_of(&mut job, &south, name).status, Status::Assumed, "{name}");
    }
    // One nobody has leaves no height and no area, and says which elevation.
    job.put(&second, Level { elevation: Val::unknown("ft", "not surveyed"), height: ft(9.0), computation_height: None }).unwrap();
    job.derive();
    let face = measure_of(&mut job, &south, "face_area");
    assert_eq!((face.value, face.status), (None, Status::Unknown));
    assert!(face.note.contains("the elevation of second floor, its top level, is not known"), "{}", face.note);
    assert_eq!(measure_of(&mut job, &south, "plan_length").value, Some(40.0));
    // Offsets: a top 1 ft 6 in under the storey above and a base 6 in above its own floor.
    // 11 − 1.5 − 0.5 = 9 ft.
    job.put(&second, Level { elevation: ft(11.0), height: ft(9.0), computation_height: None }).unwrap();
    let e = job.find(&south).unwrap();
    let mut wall = job.world.get::<Wall>(e).unwrap().clone();
    wall.top_offset = Some(Val::known(Len::FtIn([-1.0, -6.0]), "ft-in", "test"));
    wall.base_offset = Some(Val::known(Len::One(6.0), "in", "test"));
    job.put(&south, wall.clone()).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "height").value, Some(9.0));
    assert_eq!(measure_of(&mut job, &south, "face_area").value, Some(360.0));
    // A top at or under its base is not a wall: no height, and why.
    wall.top_offset = Some(ft(-10.5));
    job.put(&south, wall).unwrap();
    job.derive();
    let h = measure_of(&mut job, &south, "height");
    assert_eq!(h.value, None);
    assert!(h.note.contains("a wall's top is above its base"), "{}", h.note);
}

#[test]
fn a_size_is_entered_once_on_a_type_and_every_opening_of_it_follows() {
    use plumbline_world::OpeningType;
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let west_window = job.children().get(&south).unwrap().get("west window").unwrap().clone();
    // A window's width, height and area are its type's: 5 ft by 5 ft is 25 SF.
    assert_eq!(measure_of(&mut job, &west_window, "width").value, Some(5.0));
    assert_eq!(measure_of(&mut job, &west_window, "opening_area").value, Some(25.0));
    // Make the 5 ft by 5 ft type 6 ft wide. Both south windows are of it: each is 30 SF,
    // the wall's openings are 20.01 + 60 = 80.01 SF and its net is 319.99.
    let five = path_named(&mut job, "Window 5 ft × 5 ft");
    job.put(&five, OpeningType { category: Kind::Window, width: ft(6.0), height: ft(5.0) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &west_window, "opening_area").value, Some(30.0));
    assert_eq!(measure_of(&mut job, &south, "opening_area").value, Some(80.01));
    assert_eq!(measure_of(&mut job, &south, "net_area").value, Some(319.99));
    // A width in inches on the type is the same width: 72 in.
    job.put(&five, OpeningType { category: Kind::Window, width: Val::known(Len::One(72.0), "in", "test"), height: ft(5.0) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "net_area").value, Some(319.99));
    // An assumed size on the type makes every wall with one of its windows Assumed.
    job.put(&five, OpeningType { category: Kind::Window, width: Val::assumed(Len::One(6.0), "ft", "scaled"), height: ft(5.0) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "net_area").status, Status::Assumed);
    // An opening is placed by its centre. The sample's front door is 3 ft wide, centred
    // 20 ft along. Centred 1 ft along it would start half a foot before the wall does.
    let door = job.children().get(&south).unwrap().get("front door").unwrap().clone();
    let e = job.find(&door).unwrap();
    let mut o = job.world.get::<plumbline_world::Opening>(e).unwrap().clone();
    assert_eq!(o.offset.feet(), Some(20.0));
    o.offset = ft(1.0);
    job.put(&door, o).unwrap();
    job.derive();
    let net = measure_of(&mut job, &south, "net_area");
    assert!(net.note.contains("starts 6 in before the start of its wall"), "{}", net.note);  // call 226: a length in a sentence is said as a builder says it
    // An opening whose type nobody knows has no size, and its wall no net area.
    job.put(&door, plumbline_world::Opening { type_name: Val::unknown("", "not on the door schedule"), offset: ft(20.0), sill: ft(0.0) }).unwrap();
    job.derive();
    let area = measure_of(&mut job, &door, "opening_area");
    assert_eq!(area.value, None);
    assert!(area.note.contains("its type is not known"), "{}", area.note);
    assert_eq!(measure_of(&mut job, &south, "net_area").value, None);
}

#[test]
fn a_walls_line_can_be_any_line_of_the_wall_and_its_layers_add_up() {
    use plumbline_world::building::{check_wall, check_wall_type, placements, walls, Shape};
    use plumbline_world::{Function, Layer, Location, Wall, WallType};
    // A wall 0.4 ft thick: finish 0.05, structure 0.3, finish 0.05, from the exterior in.
    let layers = [(Function::Finish, 0.05), (Function::Structure, 0.3), (Function::Finish, 0.05)];
    // How far in from the exterior face each line lies.
    assert_eq!(walls::depth(0.4, &layers, Location::Centre), 0.2);
    assert_eq!(walls::depth(0.4, &layers, Location::ExteriorFinish), 0.0);
    assert_eq!(walls::depth(0.4, &layers, Location::InteriorFinish), 0.4);
    assert_eq!(walls::depth(0.4, &layers, Location::ExteriorCore), 0.05);
    assert!((walls::depth(0.4, &layers, Location::InteriorCore) - 0.35).abs() < 1e-12);
    assert!((walls::depth(0.4, &layers, Location::CoreCentre) - 0.2).abs() < 1e-12);
    // With no structure layer the core is the whole wall.
    assert_eq!(walls::depth(0.4, &[], Location::InteriorCore), 0.4);
    // The exterior is to the left of the line. Drawn on its exterior face, a wall's
    // faces are at 0 and −0.4, and its centre line is 0.2 to the right.
    assert_eq!(walls::wall_faces(0.4, &layers, Location::ExteriorFinish), (0.0, -0.4));
    assert_eq!(walls::centre(0.4, &layers, Location::ExteriorFinish), -0.2);

    // In a job: the sample's south wall is 6 in thick, drawn west to east along y = 0, so
    // its exterior is to the north of the line as drawn. Say the line is its exterior
    // face and the wall is drawn about a centre line 3 in to the south.
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let e = job.find(&south).unwrap();
    let mut wall = job.world.get::<Wall>(e).unwrap().clone();
    wall.location = Location::ExteriorFinish;
    job.put(&south, wall.clone()).unwrap();
    let (drawn, _) = placements(&mut job);
    match &drawn.iter().find(|p| p.path == south).unwrap().shape {
        Shape::Wall { a, b, thickness, .. } => assert_eq!((*a, *b, *thickness), ([0.0, -0.25], [40.0, -0.25], 0.5)),
        other => panic!("{other:?}"),
    }
    // Its length and its areas do not move: they are the line's.
    assert_eq!(measure_of(&mut job, &south, "plan_length").value, Some(40.0));

    // A type's layers add up to its thickness, or it is refused.
    let layer = |function, name: &str, inches: f64| Layer { function, material: Val::known(name.to_string(), "", "test"), thickness: Val::known(Len::One(inches), "in", "test") };
    let built_up = |thickness: f64| WallType { thickness: Val::known(Len::One(thickness), "in", "test"), layers: vec![layer(Function::Finish, "siding", 0.75), layer(Function::Membrane, "housewrap", 0.0), layer(Function::Structure, "2x6 studs", 5.5), layer(Function::Finish, "gypsum board", 0.5)], footing: false };
    assert!(check_wall_type("t", Kind::Type, &built_up(6.75)).is_ok());
    let err = check_wall_type("t", Kind::Type, &built_up(6.0)).unwrap_err();
    assert!(err.contains("its layers add up to 0.5625 ft, not its thickness, 0.5 ft"), "{err}");
    // Only a membrane may be as thin as nothing.
    let mut thin = built_up(6.75);
    thin.layers[0].thickness = Val::known(Len::One(0.0), "in", "test");
    assert!(check_wall_type("t", Kind::Type, &thin).is_err());
    assert!(check_wall_type("t", Kind::Wall, &built_up(6.75)).is_err());
    // A wall is a height tall or is held to a level: one, and not both or neither.
    let both = Wall { top_level: Some(named("storey-0003")), ..Wall::tall(named("type-0002"), Val::known([[0.0, 0.0], [1.0, 0.0]], "ft", "test"), ft(8.0)) };
    assert!(check_wall("w", Kind::Wall, &both).unwrap_err().contains("one or the other"));
    let neither = Wall { height: None, ..both.clone() };
    assert!(check_wall("w", Kind::Wall, &Wall { top_level: None, ..neither }).is_err());
    let offset_with_no_level = Wall { top_offset: Some(ft(1.0)), ..Wall::tall(named("type-0002"), Val::known([[0.0, 0.0], [1.0, 0.0]], "ft", "test"), ft(8.0)) };
    assert!(check_wall("w", Kind::Wall, &offset_with_no_level).unwrap_err().contains("top offset but no top level"));
}

#[test]
fn a_type_named_has_to_be_a_type_of_the_right_sort() {
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let outside = path_named(&mut job, "Outside wall, 6 in");
    let slab = path_named(&mut job, "Slab on grade, 4 in");
    let door = path_named(&mut job, "Door 3 ft × 6.67 ft");
    let window = path_named(&mut job, "Window 5 ft × 5 ft");
    let first = path_named(&mut job, "first floor");
    let swap = |from: &str, to: &str, n: usize| text.replacen(&format!("\"value\": \"{from}\""), &format!("\"value\": \"{to}\""), n);
    // A wall that names a floor type; a floor that names a wall type; a type that is not there.
    let err = file::load(&swap(&outside, &slab, 1)).err().expect("a wall of a floor type");
    assert!(err.contains("which is not a wall type in this job"), "{err}");
    let err = file::load(&swap(&slab, &outside, 1)).err().expect("a floor of a wall type");
    assert!(err.contains("which is not a floor type in this job"), "{err}");
    let err = file::load(&swap(&door, "type-9999", 1)).err().expect("a type that is not there");
    assert!(err.contains("which is not a door or window type in this job"), "{err}");
    // A door of a window's type.
    let err = file::load(&swap(&door, &window, 1)).err().expect("a door of a window type");
    assert!(err.contains("is a door, but its type") && err.contains("is a window's"), "{err}");
    // A top level that is not a storey.
    let second = path_named(&mut job, "second floor");
    let _ = second;
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let wall = doc["data"].as_array_mut().unwrap().iter_mut().find(|e| !e["attributes"]["pl::wall"]["top_level"].is_null()).expect("the saved job has a wall with a top level");
    wall["attributes"]["pl::wall"]["top_level"]["value"] = outside.clone().into();
    let broken = doc.to_string();
    let err = file::load(&broken).err().expect("a top level that is a type");
    assert!(err.contains("which is not a level in this job"), "{err}");
    // A type under a storey instead of under the job.
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    for entry in doc["data"].as_array_mut().unwrap() {
        let path = entry["path"].as_str().unwrap().to_string();
        if path == "job-0001" {
            entry["children"].as_object_mut().unwrap().remove("Slab on grade, 4 in");
        } else if path == first {
            entry["children"].as_object_mut().unwrap().insert("Slab on grade, 4 in".into(), slab.clone().into());
        }
    }
    let err = file::load(&doc.to_string()).err().expect("a type under a level");
    assert!(err.contains("a type belongs to the whole job"), "{err}");
    // Taken out of a job that is open, a type leaves its walls with no thickness to draw
    // and its openings with no size, each said: nothing falls back to a number.
    job.remove(&window).unwrap();
    job.derive();
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!(net.value, None);
    let (_, not_drawn) = plumbline_world::building::placements(&mut job);
    assert!(not_drawn.iter().any(|(_, why)| why.contains("is not known, so it is not cut in its wall")), "{not_drawn:?}");
}

// ---------------------------------------------------------------- the re-base on BIMCraft: connectors and the network

#[test]
fn pipes_end_to_end_are_one_network_with_two_open_ends() {
    let (mut job, s, two, ..) = piping();
    // 10 ft east, then 6 ft north, 1 ft above the floor.
    piece(&mut job, &s, Kind::Pipe, "a", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    piece(&mut job, &s, Kind::Pipe, "b", &two, [10.0, 0.0, 1.0], [10.0, 6.0, 1.0]);
    let n = net(&mut job);
    assert_eq!((n.nodes.len(), n.edges.len(), n.groups.len()), (3, 2, 1));
    assert_eq!(n.open.len(), 2, "the first pipe's start and the last's end");
    assert_eq!((n.edges[0].length, n.edges[1].length), (Some(10.0), Some(6.0)));
    assert!(n.warnings.is_empty() && n.left_out.is_empty());
    // Nothing is stored: the saved job is the same before and after asking.
    let before = file::save(&mut job);
    net(&mut job);
    assert_eq!(file::save(&mut job), before);
}

#[test]
fn a_connector_is_a_node_a_fitting_joins_through_it_and_a_fixture_does_not() {
    use plumbline_world::building::network::Joint;
    use plumbline_world::{ConnectorKind::Pipe, Domain};
    let (mut job, s, two, ..) = piping();
    // A water closet whose waste outlet is a foot to the north of its middle, at the
    // floor. A drain starts there. Its cold inlet meets nothing: it is open.
    let wc = thing(&mut job, &s, Kind::Fixture, "wc", [5.0, 5.0, 0.0], None, &[(Pipe, Some(Domain::Waste), [0.0, 1.0, 0.0]), (Pipe, Some(Domain::Cold), [-0.5, 0.0, 1.0])]);
    let drain = piece(&mut job, &s, Kind::Pipe, "drain", &two, [5.0, 6.0, 0.0], [12.0, 6.0, -0.2]);
    let n = net(&mut job);
    let at_outlet = n.nodes.iter().find(|x| x.at == [5.0, 6.0, 0.0]).expect("a node at the outlet");
    assert_eq!(at_outlet.joints, vec![Joint::Connector { element: wc.clone(), index: 0, domain: Some(Domain::Waste) }, Joint::RunEnd { run: drain.clone(), end: false }]);
    // Open: the cold inlet, and the far end of the drain.
    assert_eq!(n.open.len(), 2);
    assert!(n.open.iter().any(|&i| n.nodes[i].joints == vec![Joint::Connector { element: wc.clone(), index: 1, domain: Some(Domain::Cold) }]));
    // A pipe to the cold inlet is a second network: a fixture's waste and its water are
    // not joined through it.
    piece(&mut job, &s, Kind::Pipe, "cold", &two, [4.5, 5.0, 1.0], [4.5, 0.0, 1.0]);
    let n = net(&mut job);
    assert_eq!(n.groups.len(), 2);
    assert!(kinds(&n).is_empty(), "waste and cold are in networks of their own: {:?}", n.warnings);

    // A fitting does join what meets its two ends. Two pipes that stop half a foot short
    // of a corner are two networks; an elbow whose legs reach them makes them one, and
    // the way through it is an edge with no length.
    let (mut job, s, two, ..) = piping();
    piece(&mut job, &s, Kind::Pipe, "a", &two, [0.0, 0.0, 1.0], [9.5, 0.0, 1.0]);
    piece(&mut job, &s, Kind::Pipe, "b", &two, [10.0, 0.5, 1.0], [10.0, 6.0, 1.0]);
    assert_eq!(net(&mut job).groups.len(), 2);
    let elbow = thing(&mut job, &s, Kind::Fitting, "elbow", [10.0, 0.0, 1.0], None, &[(Pipe, None, [-0.5, 0.0, 0.0]), (Pipe, None, [0.0, 0.5, 0.0])]);
    let n = net(&mut job);
    assert_eq!((n.groups.len(), n.open.len()), (1, 2));
    assert!(n.edges.iter().any(|e| e.element == elbow && e.length.is_none()));
    // Turned a quarter turn counter-clockwise, a point (x, y) of the elbow goes to
    // (−y, x). The leg that was at (9.5, 0) is at (10, −0.5) and meets nothing; the leg
    // that was at (10, 0.5) is at (9.5, 0), on the end of pipe a. Pipe b is on its own.
    let e = job.find(&elbow).unwrap();
    job.world.get_mut::<plumbline_world::Placed>(e).unwrap().angle = Some(Val::known(90.0, "deg", "test"));
    let n = net(&mut job);
    let near = |x: &plumbline_world::building::network::Node, e: f64, nn: f64| (x.at[0] - e).abs() < 1e-9 && (x.at[1] - nn).abs() < 1e-9;
    let loose = n.nodes.iter().find(|x| near(x, 10.0, -0.5)).expect("a node at (10, −0.5)");
    assert_eq!(loose.joints, vec![Joint::Connector { element: elbow.clone(), index: 0, domain: None }]);
    let on_a = n.nodes.iter().find(|x| near(x, 9.5, 0.0)).expect("a node at (9.5, 0)");
    assert!(on_a.joints.contains(&Joint::Connector { element: elbow.clone(), index: 1, domain: None }) && on_a.joints.len() == 2);
    assert_eq!(n.groups.len(), 2);
}

#[test]
fn what_looks_wrong_in_a_network_is_a_warning_and_never_a_refusal() {
    use plumbline_world::building::network::{network, Joint};
    use plumbline_world::{ConnectorKind::Pipe, Domain, InSystem, SystemInfo};
    // Two systems in one network.
    let (mut job, s, two, one, _) = piping();
    let system = |job: &mut Job, name: &str, classification: &str| {
        let p = job.add("job-0001", Kind::System, name).unwrap();
        job.put(&p, SystemInfo { discipline: plumbline_world::Discipline::Plumbing, classification: classification.into() }).unwrap();
        p
    };
    let (cold, hot) = (system(&mut job, "Cold", "Domestic Cold Water"), system(&mut job, "Hot", "Domestic Hot Water"));
    let a = piece(&mut job, &s, Kind::Pipe, "a", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    let b = piece(&mut job, &s, Kind::Pipe, "b", &two, [10.0, 0.0, 1.0], [10.0, 6.0, 1.0]);
    job.put(&a, InSystem(named(&cold))).unwrap();
    job.put(&b, InSystem(named(&hot))).unwrap();
    let n = net(&mut job);
    assert_eq!(kinds(&n), ["systems_meet"]);
    assert_eq!(n.warnings[0].message, "runs of the systems Cold, Hot are joined in one network");
    // Asked for one system, the network is what that system's runs are joined to.
    assert_eq!(network(&mut job, Some(&cold), None, 0.01).unwrap().groups.len(), 1);
    let c = piece(&mut job, &s, Kind::Pipe, "c", &one, [0.0, 20.0, 1.0], [5.0, 20.0, 1.0]);
    assert_eq!(net(&mut job).groups.len(), 2);
    assert_eq!(network(&mut job, Some(&cold), None, 0.01).unwrap().groups.len(), 1, "c is on no system and joins nothing of Cold's");
    let _ = c;

    // A waste connector and a cold-water connector in one network.
    let (mut job, s, two, ..) = piping();
    thing(&mut job, &s, Kind::Fixture, "basin", [0.0, 0.0, 0.0], None, &[(Pipe, Some(Domain::Waste), [0.0, 0.0, 0.0])]);
    thing(&mut job, &s, Kind::Fixture, "heater", [10.0, 0.0, 0.0], None, &[(Pipe, Some(Domain::Cold), [0.0, 0.0, 0.0])]);
    piece(&mut job, &s, Kind::Pipe, "between", &two, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let n = net(&mut job);
    assert_eq!(kinds(&n), ["domains_meet"]);
    assert_eq!(n.warnings[0].message, "connections of cold, waste are joined in one network");

    // A branch drawn to a main's centre line, 4 ft along it, with no fitting: the main is
    // two edges, 4 ft and 6 ft, of the one element, and the join is warned of.
    let (mut job, s, two, one, _) = piping();
    let main = piece(&mut job, &s, Kind::Pipe, "main", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    piece(&mut job, &s, Kind::Pipe, "branch", &one, [4.0, 0.0, 1.0], [4.0, 3.0, 1.0]);
    let n = net(&mut job);
    let on_main: Vec<Option<f64>> = n.edges.iter().filter(|e| e.element == main).map(|e| e.length).collect();
    assert_eq!(on_main, [Some(4.0), Some(6.0)]);
    assert_eq!(kinds(&n), ["joined_mid_run"]);
    assert!(n.nodes.iter().any(|x| x.joints.contains(&Joint::RunSide { run: main.clone() })));
    assert_eq!((n.groups.len(), n.open.len()), (1, 3));

    // A drain that narrows on its way down: 2 in coming down a stack to a 1 in pipe
    // leaving below it. Read where a waste connector says the network is a drain.
    let (mut job, s, two, one, _) = piping();
    thing(&mut job, &s, Kind::Fixture, "wc", [0.0, 0.0, 8.0], None, &[(Pipe, Some(Domain::Waste), [0.0, 0.0, 0.0])]);
    piece(&mut job, &s, Kind::Pipe, "stack", &two, [0.0, 0.0, 8.0], [0.0, 0.0, 0.0]);
    piece(&mut job, &s, Kind::Pipe, "drain", &one, [0.0, 0.0, 0.0], [10.0, 0.0, -0.5]);
    let n = net(&mut job);
    assert_eq!(kinds(&n), ["smaller_downstream"]);
    assert_eq!(n.warnings[0].message, "a drain narrows downstream: 1 in leaving where 2 in comes in");
    // The same two pipes the right way round say nothing; nor does one nobody has sized.
    let (mut job, s, two, one, _) = piping();
    thing(&mut job, &s, Kind::Fixture, "wc", [0.0, 0.0, 8.0], None, &[(Pipe, Some(Domain::Waste), [0.0, 0.0, 0.0])]);
    piece(&mut job, &s, Kind::Pipe, "stack", &one, [0.0, 0.0, 8.0], [0.0, 0.0, 0.0]);
    let drain = piece(&mut job, &s, Kind::Pipe, "drain", &two, [0.0, 0.0, 0.0], [10.0, 0.0, -0.5]);
    assert!(kinds(&net(&mut job)).is_empty());
    let e = job.find(&drain).unwrap();
    job.world.get_mut::<plumbline_world::Segment>(e).unwrap().type_name = Val::unknown("", "not on the plan");
    let n = net(&mut job);
    assert!(kinds(&n).is_empty() && n.left_out.is_empty() && n.groups.len() == 1, "a run with no size is still joined: {n:?}");
}

#[test]
fn how_close_is_one_node_and_what_cannot_be_placed_is_said() {
    use plumbline_world::building::network::network;
    let (mut job, s, two, _, duct) = piping();
    // Two pipes whose ends are 0.03 ft apart: two networks at the usual 0.01 ft, one at
    // 0.05 ft. Asked for a foot, it is held to 2 in.
    piece(&mut job, &s, Kind::Pipe, "a", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    piece(&mut job, &s, Kind::Pipe, "b", &two, [10.03, 0.0, 1.0], [20.0, 0.0, 1.0]);
    assert_eq!(network(&mut job, None, None, 0.01).unwrap().groups.len(), 2);
    assert_eq!(network(&mut job, None, None, 0.05).unwrap().groups.len(), 1);
    piece(&mut job, &s, Kind::Pipe, "c", &two, [20.5, 0.0, 1.0], [30.0, 0.0, 1.0]);
    assert_eq!(network(&mut job, None, None, 1.0).unwrap().groups.len(), 2, "half a foot is not one node at any tolerance");
    // A duct that ends on the very point a pipe ends at does not join it: a run joins its
    // own kind.
    piece(&mut job, &s, Kind::Duct, "d", &duct, [0.0, 0.0, 1.0], [0.0, 9.0, 1.0]);
    let n = network(&mut job, None, None, 0.05).unwrap();
    assert_eq!(n.groups.len(), 3);
    assert!(n.warnings.is_empty());
    // A second storey whose elevation nobody has: what is on it cannot be placed, so it
    // is in no network, and it is listed with why. It is not joined at a guessed height.
    let upper = job.add("job-0001", Kind::Storey, "upper").unwrap();
    job.put(&upper, plumbline_world::Level { elevation: Val::unknown("ft", "not surveyed"), height: ft(9.0), computation_height: None }).unwrap();
    let lost = piece(&mut job, &upper, Kind::Pipe, "upstairs", &two, [0.0, 0.0, 1.0], [10.0, 0.0, 1.0]);
    let n = network(&mut job, None, None, 0.05).unwrap();
    assert_eq!(n.left_out, vec![(lost.clone(), "its level's elevation is not known".to_string())]);
    assert_eq!(n.groups.len(), 3);
    // Asked for one storey, only what is under it.
    assert_eq!(network(&mut job, None, Some(&upper), 0.05).unwrap().groups.len(), 0);
    assert_eq!(network(&mut job, None, Some(&s), 0.05).unwrap().left_out.len(), 0);
}

#[test]
fn the_sample_house_says_what_is_joined_what_is_open_and_where_a_tee_is_missing() {
    use plumbline_world::building::takeoff;
    let mut job = plumbline_world::sample::house();
    let systems = takeoff(&mut job);
    let of = |name: &str| &systems.iter().find(|s| s.name == name).unwrap().joined;
    // Supply air. The first floor's ducts and the attic's each start at the air handler,
    // and equipment does not join what meets it: two networks. The two ends of each
    // trunk are open (four). Five branches leave the first-floor trunks along their
    // length, the riser meets the attic trunk along its length, and four branches leave
    // the attic trunk: ten places with no fitting entered.
    let supply = of("Supply air");
    assert_eq!((supply.networks, supply.open.len(), supply.no_fitting.len()), (2, 4, 10));
    assert!(supply.open.iter().any(|o| o.said == "attic trunk: its first end" && o.at == [4.0, 14.0, 20.0]), "{:?}", supply.open);
    // Return air and refrigerant are each one network with nothing open: every end is on
    // a grille, a unit, or an elbow.
    for name in ["Return air", "Refrigerant"] {
        assert_eq!((of(name).networks, of(name).open.len(), of(name).no_fitting.len()), (1, 0, 0), "{name}");
    }
    // Cold water is one network; the only open end is the service where it comes in from
    // the street.
    let cold = of("Cold water");
    assert_eq!((cold.networks, cold.open.len()), (1, 1));
    assert_eq!(cold.open[0].said, "water service, piece 1 of 4: its first end");
    assert!(cold.warnings.is_empty(), "the drain's warnings are not the cold water's: {:?}", cold.warnings);
    // The drain and its vent are joined, as they are built, and that is said on both.
    for name in ["Sanitary", "Vent"] {
        assert_eq!(of(name).warnings, ["runs of the systems Sanitary, Vent are joined in one network"], "{name}");
    }
    assert_eq!(of("Sanitary").open.iter().map(|o| o.said.as_str()).collect::<Vec<_>>(), ["building drain: its second end"]);
    // Every fixture's connector is met by a run: no connector is among the open ends.
    assert!(systems.iter().flat_map(|s| &s.joined.open).all(|o| !o.said.contains("connection")));
    // Nothing was left out of the check.
    assert!(systems.iter().all(|s| s.joined.left_out.is_empty()));
    // The sample saves and loads with its connectors and fittings, to the same bytes.
    let text = file::save(&mut job);
    let (mut again, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(file::save(&mut again), text);
}

#[test]
fn a_connector_has_to_make_sense() {
    use plumbline_world::building::check_connectors;
    use plumbline_world::{Connector, ConnectorKind, Connectors, Domain, Flow};
    let at = |unit: &str| Val::known([0.0, 0.0, 0.0], unit, "test");
    let one = |kind, domain, unit: &str| Connectors(vec![Connector { name: "x".into(), kind, domain, flow: Flow::In, at: at(unit), size: None, facing: None, of_item: None }]);
    assert!(check_connectors("f", Kind::Fixture, &one(ConnectorKind::Pipe, Some(Domain::Cold), "ft")).is_ok());
    assert!(check_connectors("f", Kind::Fitting, &one(ConnectorKind::Pipe, None, "in")).is_ok());
    // Only a pipe connector says what it carries; a wall has no connectors; a position
    // has a unit, and not a metric one.
    assert!(check_connectors("f", Kind::Fixture, &one(ConnectorKind::Duct, Some(Domain::Cold), "ft")).is_err());
    assert!(check_connectors("f", Kind::Wall, &one(ConnectorKind::Pipe, None, "ft")).is_err());
    assert!(check_connectors("f", Kind::Fixture, &one(ConnectorKind::Pipe, None, "")).is_err());
    assert!(check_connectors("f", Kind::Fixture, &one(ConnectorKind::Pipe, None, "mm")).is_err());
    let sized = Connectors(vec![Connector { name: String::new(), kind: ConnectorKind::Pipe, domain: None, flow: Flow::Both, at: at("ft"), size: Some(Val::known(Len::One(0.0), "in", "test")), facing: None, of_item: None }]);
    assert!(check_connectors("f", Kind::Fixture, &sized).is_err(), "a size of nothing");
}
