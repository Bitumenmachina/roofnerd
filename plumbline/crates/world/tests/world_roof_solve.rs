use plumbline_world::file::{load, save};
use plumbline_world::sample::sample;
use plumbline_world::value::Status;
use plumbline_world::*;

#[allow(dead_code)]
mod common;
use common::{touched, measure, edit, attrs, show_rows, footprint, child, hip_roof};

#[test]
fn a_footprint_solves_into_the_roof_that_was_entered_by_hand() {
    // The sample's main roof was entered plane by plane and line by line. The same roof
    // from its footprint alone: 40 x 30, 6:12 all round, eaves at 10 ft.
    let mut job = Job::new("Solved", "test", "test");
    let (roof_path, report) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    assert_eq!((report.planes, report.lines), (4, 9));
    assert!(report.notes.is_empty(), "{:?}", report.notes);
    // South plane: 375 SF on plan, 419.2627 SF on the slope, as worked by hand for the sample.
    let south = child(&mut job, &roof_path, "plane 1");
    assert_eq!(measure(&mut job, &south, "plan_area").value, Some(375.0));
    assert_eq!(measure(&mut job, &south, "surface_area").value, Some(419.2627));
    let east = child(&mut job, &roof_path, "plane 2");
    assert_eq!(measure(&mut job, &east, "plan_area").value, Some(225.0));
    // The ridge is 10 LF and level; each hip is 22.5 LF on the slope.
    let ridge = child(&mut job, &roof_path, "ridge 1-3");
    assert_eq!(measure(&mut job, &ridge, "true_length").value, Some(10.0));
    for hip in ["hip 1-2", "hip 2-3", "hip 3-4", "hip 1-4"] {
        let path = child(&mut job, &roof_path, hip);
        assert_eq!(measure(&mut job, &path, "true_length").value, Some(22.5), "{hip}");
    }
    for (eave, lf) in [("eave 1", 40.0), ("eave 2", 30.0), ("eave 3", 40.0), ("eave 4", 30.0)] {
        let path = child(&mut job, &roof_path, eave);
        assert_eq!(measure(&mut job, &path, "true_length").value, Some(lf), "{eave}");
    }
    // The four planes cover the footprint: 1,200 SF on plan.
    let total: f64 = (1..=4).map(|i| { let p = child(&mut job, &roof_path, &format!("plane {i}")); measure(&mut job, &p, "plan_area").value.unwrap() }).sum();
    assert_eq!(total, 1200.0);
    // It is a job like any other: it saves, loads, and saves to the same bytes, and the
    // lines the planes imply are exactly the lines the solver wrote.
    let text = save(&mut job);
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty());
    assert_eq!(save(&mut again), text);
    let (solid, _) = roof::solid_planes(&mut again);
    let found = roof::lines_of(&solid);
    let cmp = roof::compare(&mut again, &found);
    assert_eq!((cmp.agree.len(), cmp.entered_only.len(), cmp.implied_only.len()), (9, 0, 0));
    // Solving again changes nothing.
    solve::solve(&mut again, &roof_path).unwrap();
    assert_eq!(save(&mut again), text);
}

#[test]
fn changing_a_pitch_moves_the_ridge_and_keeps_every_element_it_can() {
    // 60 x 30, 6:12 all round, eaves at 10 ft: the ridge runs at y = 15 from x = 15 to 45.
    let mut job = Job::new("Solved", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [60.0, 0.0], [60.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    let south = child(&mut job, &roof_path, "plane 1");
    let ridge = child(&mut job, &roof_path, "ridge 1-3");
    assert_eq!(measure(&mut job, &ridge, "true_length").value, Some(30.0));
    // Put a condition on the south plane, as an estimator would.
    job.set(&south, "condition", Some("Standing seam copper"), Status::Known, "finish schedule").unwrap();
    let paths_before: Vec<String> = job.children()[&roof_path].values().cloned().collect();
    let before = save(&mut job);

    // Steepen the south edge to 12:12. Entered by hand, this broke a roof (the hips no
    // longer fitted). Solved, the ridge moves to where the two slopes now meet: south
    // rises 1 ft per ft and north 0.5 ft per ft, so y = (30 - y) / 2, y = 10.
    let e = job.find(&roof_path).unwrap();
    let mut fp = job.world.get::<Footprint>(e).unwrap().clone();
    fp.edges[0] = Val::known(Rise::Pitch(12.0), "in/12", "revised elevation");
    job.put(&roof_path, fp).unwrap();
    let report = solve::solve(&mut job, &roof_path).unwrap();
    assert!(report.removed.is_empty() && report.added.is_empty(), "{report:?}");
    // Same elements, same paths; the south plane still has its condition.
    let paths_after: Vec<String> = job.children()[&roof_path].values().cloned().collect();
    assert_eq!(paths_before, paths_after);
    let s = job.find(&south).unwrap();
    assert_eq!(job.world.get::<Condition>(s).unwrap().0.value.as_deref(), Some("Standing seam copper"));
    // The ridge is at y = 10 now, from x = 20 to x = 40: 20 LF, level, 10 ft above the eaves.
    let r = job.find(&ridge).unwrap();
    let ends = job.world.get::<Run>(r).unwrap().ends.value.unwrap();
    assert!(ends.iter().all(|p| (p[1] - 10.0).abs() < 1e-6), "{ends:?}");
    assert_eq!(measure(&mut job, &ridge, "true_length").value, Some(20.0));
    // South plane: trapezoid (60 + 20) / 2 x 10 = 400 SF on plan, x sqrt(2) = 565.6854 SF.
    // North plane: (60 + 20) / 2 x 20 = 800 SF on plan, x 1.118034 = 894.4272 SF.
    assert_eq!(measure(&mut job, &south, "plan_area").value, Some(400.0));
    assert_eq!(measure(&mut job, &south, "surface_area").value, Some(565.6854));
    let north = child(&mut job, &roof_path, "plane 3");
    assert_eq!(measure(&mut job, &north, "surface_area").value, Some(894.4272));
    // No line on it is flagged: the planes fit each other because they were solved together.
    for (_, path) in job.children()[&roof_path].clone() {
        if job.kind(&path).is_some_and(|k| k.is_run()) {
            assert!(measure(&mut job, &path, "true_length").value.is_some(), "{path}");
        }
    }
    // In the file, everything that changed is the roof and what was solved from it.
    let after = save(&mut job);
    assert!(touched(&before, &after).iter().all(|p| *p == roof_path || paths_after.contains(p)));

    // On a 40 x 30 footprint the same change closes the roof to a point at (20, 10): the
    // west hip (x / 2 = y) and the east hip ((40 - x) / 2 = y) both reach y = 10 at x = 20.
    // The ridge has no length left, so it is removed, and the solve says so.
    let mut job = Job::new("Pyramid", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    let ridge = child(&mut job, &roof_path, "ridge 1-3");
    let e = job.find(&roof_path).unwrap();
    let mut fp = job.world.get::<Footprint>(e).unwrap().clone();
    fp.edges[0] = Val::known(Rise::Pitch(12.0), "in/12", "revised elevation");
    job.put(&roof_path, fp).unwrap();
    let report = solve::solve(&mut job, &roof_path).unwrap();
    assert_eq!(report.removed, [(ridge, false)]);
    assert!(load(&save(&mut job)).is_ok());
}

#[test]
fn an_l_shaped_footprint_gets_its_valley_and_a_gable_gets_its_rakes() {
    // An L: 40 x 20 with a 20 x 20 wing to the north on its west half, 6:12 all round.
    // One inside corner at (20, 20), so one valley, running to where the two ridges meet.
    let mut job = Job::new("L", "test", "test");
    let l = vec![[0.0, 0.0], [40.0, 0.0], [40.0, 20.0], [20.0, 20.0], [20.0, 40.0], [0.0, 40.0]];
    let (roof_path, report) = solve::new_roof(&mut job, "main", footprint(l, &[Rise::Pitch(6.0); 6], 10.0)).unwrap();
    assert!(report.notes.is_empty(), "{:?}", report.notes);
    let kinds: Vec<Kind> = job.children()[&roof_path].values().filter_map(|p| job.kind(p)).collect();
    let count = |k: Kind| kinds.iter().filter(|x| **x == k).count();
    assert_eq!((count(Kind::RoofPlane), count(Kind::Eave), count(Kind::Valley), count(Kind::Rake)), (6, 6, 1, 0));
    // The valley runs from the inside corner (20, 20) to (10, 10): 14.1421 LF on plan,
    // climbing 10 ft of run at 6:12 = 5 ft: sqrt(200 + 25) = 15 LF.
    let valley = job.children()[&roof_path].iter().find(|(n, _)| n.starts_with("valley")).map(|(_, p)| p.clone()).unwrap();
    assert_eq!(measure(&mut job, &valley, "true_length").value, Some(15.0));
    // The planes cover the footprint: 40 x 20 + 20 x 20 = 1,200 SF on plan.
    let total: f64 = job.children()[&roof_path].clone().values().filter(|p| p.starts_with("plane")).map(|p| measure(&mut job, p, "plan_area").value.unwrap()).sum();
    assert!((total - 1200.0).abs() < 1e-3, "{total}");

    // A gable roof: the two short ends upright. Two planes, a ridge end to end, four rakes.
    let mut job = Job::new("Gable", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 20.0], [0.0, 20.0]], &[Rise::Pitch(6.0), Rise::Gable, Rise::Pitch(6.0), Rise::Gable], 10.0)).unwrap();
    let kinds: Vec<Kind> = job.children()[&roof_path].values().filter_map(|p| job.kind(p)).collect();
    let count = |k: Kind| kinds.iter().filter(|x| **x == k).count();
    assert_eq!((count(Kind::RoofPlane), count(Kind::Eave), count(Kind::Ridge), count(Kind::Rake), count(Kind::Hip)), (2, 2, 1, 4, 0));
    let ridge = child(&mut job, &roof_path, "ridge 1-3");
    assert_eq!(measure(&mut job, &ridge, "true_length").value, Some(40.0));
    // Each rake: 10 ft of run at 6:12 climbs 5 ft: sqrt(100 + 25) = 11.1803 LF.
    for (name, path) in job.children()[&roof_path].clone() {
        if name.starts_with("rake") {
            assert_eq!(measure(&mut job, &path, "true_length").value, Some(11.1803), "{name}");
        }
    }
    // Turning a hipped end into a gable removes the hips and says so; an edge with no
    // pitch entered cannot be solved, and says which.
    let mut job = Job::new("Change", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 20.0], [0.0, 20.0]], &[Rise::Pitch(6.0); 4], 10.0)).unwrap();
    let e = job.find(&roof_path).unwrap();
    let mut fp = job.world.get::<Footprint>(e).unwrap().clone();
    fp.edges[1] = Val::known(Rise::Gable, "", "elevation");
    job.put(&roof_path, fp.clone()).unwrap();
    let report = solve::solve(&mut job, &roof_path).unwrap();
    assert_eq!(report.removed.len(), 4, "{report:?}"); // the east plane, its eave, its two hips
    assert_eq!(report.added.len(), 2, "{report:?}"); // two rakes
    assert!(load(&save(&mut job)).is_ok());
    fp.edges[0] = Val::unknown("in/12", "not on the drawings");
    job.put(&roof_path, fp).unwrap();
    assert!(solve::solve(&mut job, &roof_path).unwrap_err().contains("edge 1 has no pitch entered"));
}

#[test]
fn the_skeleton_holds_in_feet_on_a_short_edge_and_a_long_way_from_the_origin() {
    // The port restated BIMCraft's fixed millimetre tolerance as a share of the footprint's
    // size. A 40 x 30 roof with a 3 in jog in one eave (an edge a quarter of a foot long),
    // a mile from the origin: it still solves, and its planes still cover its footprint.
    let o = 5280.0;
    let pts = vec![[o, o], [o + 20.0, o], [o + 20.0, o + 0.25], [o + 40.0, o + 0.25], [o + 40.0, o + 30.0], [o, o + 30.0]];
    let area = geom::plan_area(&pts);
    let mut job = Job::new("Far", "test", "test");
    let (roof_path, report) = solve::new_roof(&mut job, "main", footprint(pts, &[Rise::Pitch(6.0); 6], 10.0)).unwrap();
    assert!(report.notes.is_empty(), "{:?}", report.notes);
    let total: f64 = job.children()[&roof_path].clone().values().filter(|p| p.starts_with("plane")).map(|p| measure(&mut job, p, "plan_area").value.unwrap()).sum();
    assert!((total - area).abs() < 1e-3, "{total} vs {area}");
    for (_, path) in job.children()[&roof_path].clone() {
        if job.kind(&path).is_some_and(|k| k.is_run()) {
            assert!(measure(&mut job, &path, "true_length").value.is_some(), "{path}: {}", measure(&mut job, &path, "true_length").note);
        }
    }
}

// ---- Regression checks for what the sixth independent judgment found (the solved roof) ----

/// A roof's own figures, from its planes and lines (call 134). A hip roof 40 by 30 at
/// 6:12: 1,200 SF on plan x sqrt(1.25) = 1,341.6408 SF, 13.4164 SQ; eaves 140 LF; the
/// ridge 40 - 30 = 10 LF; four hips, each 15 sqrt 2 on plan rising 7.5 ft, 22.5 ft: 90 LF;
/// no valleys, rakes or headwalls: 0 each, and said why. A plane with no surface leaves
/// the roof none, and the roof says which plane.
#[test]
fn a_roof_is_taken_off_whole_from_its_planes_and_lines() {
    let (mut job, roof_path) = hip_roof();
    job.derive();
    assert_eq!(measure(&mut job, &roof_path, "surface_area").value, Some(1341.6408));
    assert_eq!(measure(&mut job, &roof_path, "squares").value, Some(13.4164));
    assert_eq!(measure(&mut job, &roof_path, "eave_length").value, Some(140.0));
    assert_eq!(measure(&mut job, &roof_path, "ridge_length").value, Some(10.0));
    assert_eq!(measure(&mut job, &roof_path, "hip_length").value, Some(90.0));
    let valleys = measure(&mut job, &roof_path, "valley_length");
    assert_eq!((valleys.value, valleys.formula.as_str()), (Some(0.0), "the roof has no valley line"), "{valleys:?}");
    // The same as the panel's totals: one arithmetic.
    let totals = plumbline_world::view::roof_totals(&mut job, &roof_path);
    assert_eq!(totals.surface.value, Some(1341.6408));
    // A pitch struck out of the file: the roof has no surface and names a plane.
    let text = save(&mut job);
    let struck = edit(&text, |v| attrs(v, &roof_path)["pl::footprint"]["edges"][0] = serde_json::json!({"value": null, "unit": "in/12", "status": "Unknown", "source": "not on the drawing"}));
    let (mut loaded, _) = load(&struck).unwrap();
    let surface = measure(&mut loaded, &roof_path, "surface_area");
    assert_eq!((surface.value, surface.status), (None, Status::Unknown));
    assert!(surface.note.contains("has no surface"), "{surface:?}");
}

#[test]
fn a_footprint_changed_in_the_file_does_not_leave_the_old_roof_standing() {
    let (mut job, roof_path) = hip_roof();
    let south = child(&mut job, &roof_path, "plane 1");
    let text = save(&mut job);
    // Edge 1's pitch struck out by hand. The planes of the earlier solve were 419.2627 SF
    // and Known; they must not stay so.
    let struck = edit(&text, |v| attrs(v, &roof_path)["pl::footprint"]["edges"][0] = serde_json::json!({"value": null, "unit": "in/12", "status": "Unknown", "source": "not on the drawing"}));
    let (mut loaded, report) = load(&struck).unwrap();
    assert!(report.stale.contains(&south) && report.stale.contains(&roof_path), "{:?}", report.stale);
    let d = measure(&mut loaded, &south, "surface_area");
    assert_eq!((d.value, d.status), (None, Status::Unknown));
    let rows = show_rows(&mut loaded, &roof_path);
    assert!(rows.contains(&("solved roof".to_string(), "not solved".to_string(), Status::Unknown)), "{rows:?}");
    assert!(rows.contains(&("edge 1".to_string(), "unknown".to_string(), Status::Unknown)), "{rows:?}");
    // No line under it has a length, and no total is given.
    let totals = plumbline_world::view::roof_totals(&mut loaded, &roof_path);
    assert_eq!(totals.surface.value, None);
    assert!(totals.lines.iter().all(|(_, sum)| sum.value.is_none()));
    // Entering the pitch again solves it again: same planes, same paths.
    solve::edit(&mut loaded, &roof_path, solve::Change::Rise { edge: 1, rise: Val::known(Rise::Pitch(6.0), "in/12", "elevation A-301") }).unwrap();
    assert_eq!(measure(&mut loaded, &south, "surface_area").value, Some(419.2627));
    // A solved plane's pitch forged in the file is put back from the footprint, and said.
    let forged = edit(&text, |v| attrs(v, &south)["pl::pitch"]["value"] = 12.0.into());
    let (mut loaded, report) = load(&forged).unwrap();
    assert!(report.stale.contains(&south));
    assert_eq!(measure(&mut loaded, &south, "surface_area").value, Some(419.2627));
    // A footprint point moved in the file moves the roof with it: 42 x 30 now.
    let moved = edit(&text, |v| {
        let pts = &mut attrs(v, &roof_path)["pl::footprint"]["points"]["value"];
        pts[1][0] = 42.0.into();
        pts[2][0] = 42.0.into();
    });
    let (mut loaded, _) = load(&moved).unwrap();
    let total: f64 = (1..=4).map(|i| { let p = child(&mut loaded, &roof_path, &format!("plane {i}")); measure(&mut loaded, &p, "plan_area").value.unwrap() }).sum();
    assert_eq!(total, 1260.0);
}

#[test]
fn an_eave_cannot_be_moved_through_the_far_wall_and_a_footprint_has_to_be_clean() {
    let (mut job, roof_path) = hip_roof();
    let before = save(&mut job);
    for by_ft in [-31.0, -30.0, -60.0, -100.0] {
        let e = solve::edit(&mut job, &roof_path, solve::Change::MoveEdge { edge: 1, by_ft, status: Status::Known, source: "tape".into() }).unwrap_err();
        assert!(e.contains("inside out") || e.contains("same point") || e.contains("long"), "{by_ft}: {e}");
        assert_eq!(save(&mut job), before, "{by_ft}");
    }
    // A move that leaves a roof is taken: in by 10 ft leaves 40 x 20.
    solve::edit(&mut job, &roof_path, solve::Change::MoveEdge { edge: 1, by_ft: -10.0, status: Status::Known, source: "tape".into() }).unwrap();
    let total: f64 = (1..=4).map(|i| { let p = child(&mut job, &roof_path, &format!("plane {i}")); measure(&mut job, &p, "plan_area").value.unwrap() }).sum();
    assert_eq!(total, 800.0);

    // A corner 1/16 in off straight would get a hip of its own; an edge 1/16 in long is a
    // slip; a pitch of a thousandth is flat. Each is refused, with what to do.
    let refuse = |pts: Vec<P2>, rises: &[Rise]| solve::new_roof(&mut sample(), "x", footprint(pts, rises, 10.0)).unwrap_err();
    assert!(refuse(vec![[0.0, 0.0], [20.0, -0.005], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 5]).contains("off the straight line"));
    assert!(refuse(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [20.0, 30.0], [20.0, 30.0052083], [0.0, 30.0052083]], &[Rise::Pitch(6.0); 6]).contains("slip of the trace"));
    assert!(refuse(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(0.001); 4]).contains("is flat"));
    // A point exactly on the line is allowed, and a refused roof leaves no trace in the job.
    let mut job = sample();
    let clean = save(&mut job);
    assert!(solve::new_roof(&mut job, "x", footprint(vec![[0.0, 0.0], [20.0, -0.005], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 5], 10.0)).is_err());
    assert_eq!(save(&mut job), clean);
    assert!(solve::new_roof(&mut job, "x", footprint(vec![[0.0, 0.0], [20.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(6.0); 5], 10.0)).is_ok());
}

#[test]
fn what_was_entered_on_a_slope_or_a_rake_stays_on_that_slope_or_rake() {
    // Two rakes. The west end is made a gable and its rake given a flashing and a
    // stretch-out; then the east end is made a gable too. The west rake keeps both.
    let (mut job, roof_path) = hip_roof();
    let gable = |edge| solve::Change::Rise { edge, rise: Val::known(Rise::Gable, "", "elevation") };
    solve::edit(&mut job, &roof_path, gable(4)).unwrap();
    let west_of = |job: &mut Job| -> Vec<String> {
        job.children()[&roof_path].clone().into_values().filter(|p| job.kind(p) == Some(Kind::Rake) && { let e = job.find(p).unwrap(); job.world.get::<Run>(e).unwrap().ends.value.unwrap().iter().all(|q| q[0].abs() < 1e-6) }).collect()
    };
    let west = west_of(&mut job);
    assert_eq!(west.len(), 2);
    let rake = west[0].clone();
    job.apply(&rake, "Copper flashing", "west rake detail").unwrap();
    job.set_on_line(&rake, "stretch_out", None, Some("7 in"), Status::Known, "detail 4/A5").unwrap();
    let report = solve::edit(&mut job, &roof_path, gable(2)).unwrap();
    assert!(report.removed.iter().all(|r| !r.1), "{report:?}");
    // Same path, still on the west side (x = 0), still with its line.
    assert!(west_of(&mut job).contains(&rake));
    let e = job.find(&rake).unwrap();
    assert_eq!(job.world.get::<Cost>(e).unwrap().lines[0].description, "Copper flashing");

    // A plane with a condition. A point is then added to the footprint by hand, on the
    // south edge, so every edge after it takes a new number. The condition stays on the
    // north slope; the solve says the plane is on a different edge number now.
    let (mut job, roof_path) = hip_roof();
    let north = child(&mut job, &roof_path, "plane 3");
    job.apply(&north, "Standing seam copper roof", "north slope, sheet A2").unwrap();
    let e = job.find(&roof_path).unwrap();
    let mut fp = job.world.get::<Footprint>(e).unwrap().clone();
    let pts = fp.points.value.as_mut().unwrap();
    pts.insert(1, [20.0, -5.0]);
    fp.edges.insert(1, Val::known(Rise::Pitch(6.0), "in/12", "test plan"));
    job.put(&roof_path, fp).unwrap();
    let report = solve::solve(&mut job, &roof_path).unwrap();
    assert!(report.renumbered.iter().any(|r| r.starts_with(&north) && r.contains("edge 4")), "{report:?}");
    let e = job.find(&north).unwrap();
    let eave = job.world.get::<Outline>(e).unwrap().eave.value.unwrap();
    assert!(eave.iter().all(|p| (p[1] - 30.0).abs() < 1e-6), "the north plane's eave is {eave:?}");
    assert_eq!(job.world.get::<Condition>(e).unwrap().0.value.as_deref(), Some("Standing seam copper roof"));
    assert_eq!(job.children()[&roof_path]["plane 4"], north);
    assert!(load(&save(&mut job)).is_ok());
}

#[test]
fn every_edge_of_a_solved_roof_has_a_line_or_a_note_that_stays_with_the_roof() {
    let all_lined = |job: &mut Job| {
        let (solid, _) = roof::solid_planes(job);
        let found = roof::lines_of(&solid);
        let cmp = roof::compare(job, &found);
        (cmp.implied_only.len(), cmp.entered_only.len())
    };
    // A 3:12 bay on a 12:12 house: where the bay's slope meets the main slopes both fall
    // the same way at different pitches. That is a change of pitch, and it gets a line.
    let mut job = Job::new("Bay", "test", "test");
    let mut rises = [Rise::Pitch(12.0); 5];
    rises[1] = Rise::Pitch(3.0);
    rises[2] = Rise::Pitch(3.0);
    let (roof_path, report) = solve::new_roof(&mut job, "house", footprint(vec![[0.0, 0.0], [40.0, 0.0], [50.0, 10.0], [40.0, 20.0], [0.0, 20.0]], &rises, 10.0)).unwrap();
    let breaks: Vec<String> = job.children()[&roof_path].clone().into_values().filter(|p| job.kind(p) == Some(Kind::PitchBreak)).collect();
    assert_eq!(breaks.len(), 2, "{report:?}");
    for b in &breaks {
        assert!(measure(&mut job, b, "true_length").value.is_some(), "{}", measure(&mut job, b, "true_length").note);
    }
    assert_eq!(all_lined(&mut job), (0, 0));
    assert!(load(&save(&mut job)).is_ok());

    // A gable beside an inside corner: a wall stands between the higher roof and the
    // lower one. Each plane has its own edge there, with a line, and the roof carries a
    // note saying so that `show` lists and the file keeps.
    let mut job = Job::new("Step", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "house", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [20.0, 40.0], [20.0, 20.0], [0.0, 20.0]], &[Rise::Pitch(6.0); 6], 10.0)).unwrap();
    solve::edit(&mut job, &roof_path, solve::Change::Rise { edge: 4, rise: Val::known(Rise::Gable, "", "elevation") }).unwrap();
    assert_eq!(all_lined(&mut job), (0, 0));
    let rows = show_rows(&mut job, &roof_path);
    assert!(rows.iter().any(|(label, _, status)| label == "solved roof" && *status == Status::Assumed), "{rows:?}");
    let text = save(&mut job);
    assert!(text.contains("\"pl::solve\"") && text.contains("A wall stands between them"));
    let (mut again, report) = load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(save(&mut again), text);
    // The planes still cover the footprint: 40 x 20 + 20 x 20 = 1,200 SF.
    let total: f64 = job.children()[&roof_path].clone().values().filter(|p| p.starts_with("plane")).map(|p| measure(&mut job, p, "plan_area").value.unwrap()).sum();
    assert!((total - 1200.0).abs() < 1e-3, "{total}");
}

#[test]
fn a_condition_can_be_applied_as_assumed_and_a_hand_made_name_in_the_solvers_way_says_so() {
    let (mut job, roof_path) = hip_roof();
    let south = child(&mut job, &roof_path, "plane 1");
    job.apply_as(&south, "Standing seam copper roof", Status::Assumed, "set in the 3D view").unwrap();
    let e = job.find(&south).unwrap();
    assert_eq!(job.world.get::<Condition>(e).unwrap().0.status, Status::Assumed);
    assert!(job.apply_as(&south, "Standing seam copper roof", Status::Unknown, "x").is_err());
    // A plane entered by hand under the roof, with the name the solver would use, after
    // the solver's own plane lost its mark: the solve says what is in its way.
    let text = edit(&save(&mut job), |v| {
        attrs(v, &south).as_object_mut().unwrap().remove("pl::solved");
    });
    let e = load(&text).err().or_else(|| load(&text).ok().and_then(|(mut j, _)| solve::solve(&mut j, &roof_path).err()));
    // Loading solves again; the roof is left unsolved with the reason on it, or refused.
    match e {
        Some(why) => assert!(why.contains("did not make that element"), "{why}"),
        None => {
            let (mut loaded, _) = load(&text).unwrap();
            let rows = plumbline_world::view::elements(&mut loaded).into_iter().find(|e| e.path == roof_path).unwrap().rows;
            assert!(rows.iter().any(|r| r.label == "solved roof" && r.note.contains("did not make that element")), "{:?}", rows.iter().map(|r| (&r.label, &r.note)).collect::<Vec<_>>());
        }
    }
}

#[test]
fn random_house_footprints_solve_whole_with_every_edge_lined() {
    // Stepped footprints (a staircase up the right side and down the left, in whole
    // feet), with pitches drawn from the ones roofs are built at and some ends gabled.
    // For each one that solves: the planes cover the footprint, every line has a length,
    // the planes imply no line the solver did not write and the solver wrote none the
    // planes do not imply, and the job loads back as it was saved.
    let mut seed = 20261110u64;
    let mut rand = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    };
    let pitches = [3.0, 4.0, 6.0, 8.0, 9.0, 12.0];
    let (mut solved, mut refused) = (0, 0);
    for round in 0..300 {
        let steps = 1 + (rand() * 3.0) as usize;
        let (mut right, mut left): (Vec<P2>, Vec<P2>) = (Vec::new(), Vec::new());
        let mut y = 0.0;
        for _ in 0..steps {
            let (xr, xl, h) = (16.0 + (rand() * 6.0).floor() * 4.0, -8.0 - (rand() * 6.0).floor() * 4.0, 12.0 + (rand() * 4.0).floor() * 4.0);
            right.extend([[xr, y], [xr, y + h]]);
            left.splice(0..0, [[xl, y + h], [xl, y]]);
            y += h;
        }
        let mut ring: Vec<P2> = right;
        ring.extend(left);
        // Take out repeated points and points on the straight line between their neighbours.
        loop {
            let n = ring.len();
            let drop = (0..n).find(|&i| {
                let (a, b, c) = (ring[(i + n - 1) % n], ring[i], ring[(i + 1) % n]);
                geom::dist(a, b) < 1e-9 || geom::point_to_segment(b, a, c) < 1e-9
            });
            match drop {
                Some(i) => {
                    ring.remove(i);
                }
                None => break,
            }
        }
        if ring.len() < 4 {
            continue;
        }
        let mixed = round % 2 == 1;
        let rises: Vec<Rise> = (0..ring.len()).map(|_| if rand() < 0.15 { Rise::Gable } else if mixed { Rise::Pitch(pitches[(rand() * 6.0) as usize % 6]) } else { Rise::Pitch(6.0) }).collect();
        let area = geom::plan_area(&ring);
        let mut job = Job::new("Fuzz", "test", "test");
        let (roof_path, report) = match solve::new_roof(&mut job, "house", footprint(ring.clone(), &rises, 10.0)) {
            Ok(x) => x,
            // Gables that fence in part of the roof, or every edge a gable, cannot be a roof.
            // Nor can gables that pinch a slope into two pieces joined at a point; that one
            // is a limit of this program, said as such.
            Err(e) if e.contains("fence in") || e.contains("none of its edges slopes") || e.contains("comes back to touch itself") => {
                refused += 1;
                continue;
            }
            Err(e) => panic!("round {round}: {e}\n{ring:?}\n{rises:?}"),
        };
        solved += 1;
        let context = format!("round {round}\n{ring:?}\n{rises:?}\n{:?}", report.notes);
        let under: Vec<String> = job.children()[&roof_path].values().cloned().collect();
        let total: f64 = under.iter().filter(|p| p.starts_with("plane")).map(|p| measure(&mut job, p, "plan_area").value.unwrap_or(f64::NAN)).sum();
        assert!((total - area).abs() < 1e-3 * area.max(1.0), "planes cover {total} of {area}: {context}");
        for path in &under {
            if job.kind(path).is_some_and(|k| k.is_run()) {
                let d = measure(&mut job, path, "true_length");
                assert!(d.value.is_some(), "{path} has no length ({}): {context}", d.note);
            }
        }
        let (solid, left_out) = roof::solid_planes(&mut job);
        assert!(left_out.is_empty(), "{left_out:?}: {context}");
        let found = roof::lines_of(&solid);
        let cmp = roof::compare(&mut job, &found);
        assert!(cmp.implied_only.is_empty() && cmp.entered_only.is_empty(), "implied only {:?}; entered only {:?}: {context}", cmp.implied_only, cmp.entered_only);
        let text = save(&mut job);
        let (mut again, report) = load(&text).unwrap_or_else(|e| panic!("{e}: {context}"));
        assert!(report.stale.is_empty(), "{:?}: {context}", report.stale);
        assert_eq!(save(&mut again), text, "{context}");
    }
    assert!(solved > 200, "only {solved} solved, {refused} refused");
}

#[test]
fn a_roof_summary_counts_facets_and_names_the_pitch_most_of_it_is_at() {
    use plumbline_world::view::{figure, roof_totals};
    // 60 x 30, the south slope at 12:12 and the rest at 6:12. On plan: south 400, north
    // 800, and each end a triangle 30 wide and 20 deep, 300; 1,800 SF in all, the whole
    // footprint. On the slope: south 400 x 1.414214 = 565.6854; north 800 x 1.118034 =
    // 894.4272; each end 300 x 1.118034 = 335.4102. The three at 6:12 come to 1,565.2476
    // SF against 565.6854 at 12:12, so most of it is at 6:12. Total 2,130.933 SF.
    let mut job = Job::new("Summary", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut job, "main", footprint(vec![[0.0, 0.0], [60.0, 0.0], [60.0, 30.0], [0.0, 30.0]], &[Rise::Pitch(12.0), Rise::Pitch(6.0), Rise::Pitch(6.0), Rise::Pitch(6.0)], 10.0)).unwrap();
    let totals = roof_totals(&mut job, &roof_path);
    assert_eq!(totals.facets, 4);
    assert_eq!(totals.predominant_pitch, Some((6.0, Status::Known)));
    assert_eq!(totals.surface.value, Some(2130.933));
    // The same surface in squares: 100 SF each, same status, nothing added or left out.
    let squares = plumbline_world::view::in_squares(&totals.surface);
    assert_eq!((squares.value, squares.unit, squares.status, squares.members), (Some(21.3093), "SQ", Status::Known, 4));
    // A worked-out row carries its figure as a number for a view to set; an entered one does not.
    let rows = plumbline_world::view::elements(&mut job);
    let plane = rows.iter().find(|e| e.kind == Kind::RoofPlane).unwrap();
    let area = plane.rows.iter().find(|r| r.label == "surface_area").unwrap();
    assert_eq!((area.number, area.unit.as_str()), (Some(565.6854), "SF"));
    assert_eq!(plane.rows.iter().find(|r| r.label == "pitch").unwrap().number, None);
    // An assumed pitch on the planes that carry most of the roof flags the pitch named.
    solve::edit(&mut job, &roof_path, solve::Change::Rise { edge: 3, rise: Val::assumed(Rise::Pitch(6.0), "in/12", "guess") }).unwrap();
    assert_eq!(roof_totals(&mut job, &roof_path).predominant_pitch, Some((6.0, Status::Assumed)));
    // Figures as a sheet shows them.
    assert_eq!(figure(2130.933, 2), "2,130.93");
    assert_eq!(figure(1341.6408, 0), "1,342");
    assert_eq!(figure(40.0, 2), "40.00");
    assert_eq!(figure(1234567.5, 1), "1,234,567.5");
    assert_eq!(figure(-0.001, 2), "0.00");
    assert_eq!(figure(-12.5, 1), "-12.5");
    // A view may round, but not round a figure away: one that is not zero never prints as zero.
    use plumbline_world::view::figure_shown;
    assert_eq!(figure_shown(2130.933, 2), "2,130.93");
    assert_eq!(figure_shown(0.004, 2), "0.004");
    assert_eq!(figure_shown(0.4, 0), "0.4");
    assert_eq!(figure_shown(-0.0003, 2), "-0.0003");
    assert_eq!(figure_shown(0.0, 2), "0.00");
    // Nor round it up into something else: 0.0005 is not 0.001 (seventh judgment).
    assert_eq!(figure_shown(0.0005, 2), "0.0005");
    assert_eq!(figure(f64::NEG_INFINITY, 2), "not a number");
    // A pitch is said to the places the model keeps, not two.
    assert_eq!(plumbline_world::view::pitch_said(6.125), "6.125:12");
    assert_eq!(plumbline_world::view::pitch_said(0.004), "0.004:12");
    assert_eq!(plumbline_world::view::pitch_said(6.0), "6:12");
}

#[test]
fn a_roof_summary_does_not_say_more_than_it_knows() {
    use plumbline_world::view::{in_squares, roof_totals, Sum};
    // A roof with no planes has no surface total. It is not a Known zero.
    let mut job = Job::new("Empty", "test", "test");
    let bare = job.add("job-0001", Kind::Roof, "bare").unwrap();
    let totals = roof_totals(&mut job, &bare);
    assert_eq!((totals.surface.value, totals.surface.status, totals.surface.members), (None, Status::Unknown, 0));
    assert_eq!((totals.predominant_pitch, totals.pitch_note.as_str()), (None, "the roof has no planes"));
    // A path that names nothing is the same.
    assert_eq!(roof_totals(&mut job, "no-such-roof").surface.value, None);
    // One plane with no pitch might be the one that tips it: no predominant pitch is named.
    let mut job = sample::sample();
    job.set("plane-0003", "pitch", None, Status::Unknown, "test").unwrap();
    job.set("plane-0005", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.derive();
    let totals = roof_totals(&mut job, "roof-0002");
    assert_eq!(totals.predominant_pitch, None);
    assert!(totals.pitch_note.contains("plane-0003"), "{}", totals.pitch_note);
    // Two pitches covering the same surface: neither is the predominant one.
    let mut job = sample::sample();
    job.set("plane-0004", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.set("plane-0006", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.set("plane-0005", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.set("plane-0003", "pitch", Some("12"), Status::Known, "test").unwrap();
    job.set("plane-0003", "pitch", Some("6"), Status::Known, "test").unwrap();
    job.set("plane-0005", "pitch", Some("6"), Status::Known, "test").unwrap();
    job.derive();
    // South and north (375 SF on plan each) at 6:12 against east and west (225 each) at 12:12.
    assert_eq!(roof_totals(&mut job, "roof-0002").predominant_pitch, Some((6.0, Status::Known)));
    let mut tie = Job::new("Tie", "test", "test");
    let (roof_path, _) = solve::new_roof(&mut tie, "main", footprint(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]], &[Rise::Pitch(6.0), Rise::Pitch(6.0), Rise::Pitch(6.0), Rise::Pitch(6.0)], 10.0)).unwrap();
    assert_eq!(roof_totals(&mut tie, &roof_path).predominant_pitch, Some((6.0, Status::Known)));
    // A parapet is one of the roof's lines, by its plan length.
    let mut job = sample::sample();
    let deck = roof_totals(&mut job, "roof-0022");
    let parapet = deck.lines.iter().find(|(k, _)| *k == Kind::Parapet).expect("the parapet is among the lines");
    assert_eq!((parapet.1.value, parapet.1.status), (Some(20.0), Status::Known));
    // A surface too small to show in squares is not zero squares.
    let tiny = Sum { value: Some(0.004), unit: "SF", status: Status::Known, members: 1, missing: Vec::new() };
    assert_eq!((in_squares(&tiny).value, in_squares(&tiny).status), (None, Status::Unknown));
}
