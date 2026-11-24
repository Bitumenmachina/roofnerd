//! Call 192, the model's half: a wall under a rake, or under the high edge of a shed roof, rises to the roof.
//! Its `gable_area` (SF) and the line of points that is its top are worked out, never typed. Everything is made up.
//! The hand figures are those of STATUS.md ("A wall under a rake rises to the roof: a demonstration").
//!
//! The house: one level 9 ft high; walls on the lines of a box 24 by 16 ft, 6 in thick; a roof drawn 1 ft outside the
//! walls' lines all round (26 by 18 on plan), eave height 9 ft, 6 in 12. The walls run south, east, north, west, each
//! from the corner it starts at (the box is closed from (0,0) east, then north, then west, then south).

use plumbline_world::editing as ed;
use plumbline_world::file;
use plumbline_world::gable;
use plumbline_world::tables;
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

fn known() -> ed::Entered<'static> {
    ed::Entered { status: Status::Known, source: "made-up survey" }
}
fn assumed() -> ed::Entered<'static> {
    ed::Entered { status: Status::Assumed, source: "made-up tracing" }
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

/// A roof drawn from its footprint. `edges` says how each edge rises: P (a 6 in 12 pitch) or G (a gable).
fn roof(job: &mut Job, ring: &[[f64; 2]], edges: &str, status: Status) -> String {
    let rise = units::parse_pitch("6:12").unwrap();
    let (len, unit) = units::parse_length("9 ft").unwrap();
    let v = |x| Val { value: Some(x), unit: units::PITCH.into(), status, source: "made-up tracing".into() };
    let gable = Val { value: Some(Rise::Gable), unit: String::new(), status, source: "made-up tracing".into() };
    let footprint = Footprint {
        edges: edges.chars().map(|c| if c == 'G' { gable.clone() } else { v(Rise::Pitch(rise)) }).collect(),
        points: Val { value: Some(ring.to_vec()), unit: units::FT.into(), status, source: "made-up tracing".into() },
        eave_height: Val { value: Some(len), unit: unit.into(), status, source: "made-up tracing".into() },
    };
    solve::new_roof(job, "R", footprint).unwrap().0
}

const RING: [[f64; 2]; 4] = [[-1.0, -1.0], [25.0, -1.0], [25.0, 17.0], [-1.0, 17.0]];

struct House {
    job: Job,
    /// South, east, north, west.
    walls: Vec<String>,
}

/// The box with a roof of these edges (None: no roof drawn).
fn house(edges: Option<&str>, how: ed::Entered<'static>, roof_status: Status) -> House {
    let mut job = Job::new("Box (not a real job)", "test", "2026-11-24");
    let level = ed::storey(&mut job, "ground", "0 ft", "9 ft", how).unwrap();
    let w6 = ed::wall_type(&mut job, "w6", "6 in", how).unwrap();
    let walls = ed::walls(&mut job, &level, &w6, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, ed::Top::Height("9 ft"), how).unwrap();
    if let Some(e) = edges {
        roof(&mut job, &RING, e, roof_status);
    }
    job.derive();
    House { job, walls }
}

fn of(job: &mut Job, path: &str) -> gable::Gable {
    gable::figures(job).into_iter().find(|g| g.path == path).unwrap_or_else(|| panic!("no gable figure for {path}"))
}
fn quantity(job: &mut Job, path: &str, line: &str) -> Derived {
    let e = job.find(path).unwrap();
    job.world.get::<LineQuantities>(e).unwrap().0[line].quantity.clone()
}
fn measure(job: &mut Job, path: &str, name: &str) -> Option<Derived> {
    let e = job.find(path).unwrap();
    job.world.get::<Measure>(e).and_then(|m| m.0.get(name).cloned())
}
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3
}
fn same(top: &[(f64, f64)], want: &[(f64, f64)]) -> bool {
    top.len() == want.len() && top.iter().zip(want).all(|(a, b)| near(a.0, b.0) && near(a.1, b.1))
}

// By hand: the roof is 9 ft at its eave (1 ft out), 0.5 ft higher for each foot in: 9.5 ft over the south and north walls'
// lines and 13.5 ft at the ridge, 8 ft in. West wall, 16 ft: 16 x 0.5 + 16 x 4 / 2 = 8 + 32 = 40.00 SF; its top (0, 9.5), (8, 13.5),
// (16, 9.5). The east wall the same. Both ends 80.00 SF. Assumed, as the tracing is.
#[test]
fn the_two_ends_of_a_gable_roof_rise_to_the_ridge_forty_square_feet_each() {
    let mut h = house(Some("PGPG"), assumed(), Status::Assumed);
    let [south, east, north, west] = [&h.walls[0], &h.walls[1], &h.walls[2], &h.walls[3]].map(|s| s.clone());
    for p in [&east, &west] {
        let g = of(&mut h.job, p);
        assert_eq!((g.area.value, g.area.unit.as_str(), g.area.status), (Some(40.0), "SF", Status::Assumed), "{p}: {:?}", g.area);
        assert!(same(&g.top, &[(0.0, 9.5), (8.0, 13.5), (16.0, 9.5)]), "{p}: {:?}", g.top);
        assert!(g.area.inputs.iter().any(|i| i.starts_with("wall-")), "it says what it rests on: {:?}", g.area.inputs);
        assert!(g.area.inputs.iter().any(|i| i.starts_with("plane-")), "the roof's plane: {:?}", g.area.inputs);
        assert!(g.area.inputs.iter().any(|i| i.starts_with("rake-")), "the rake over it: {:?}", g.area.inputs);
        assert!(!g.area.formula.is_empty());
    }
    for p in [&south, &north] {
        let g = of(&mut h.job, p);
        assert_eq!(g.area.value, Some(0.0), "{p}: {:?}", g.area);
        assert!(g.top.is_empty());
        assert!(g.why.contains("an eave is over it"), "{p}: {}", g.why);
    }
}

// Drawn Known, both figures are Known; none stays Known and is zero, so a hip roof leaves every estimate as it was.
#[test]
fn known_where_what_it_rests_on_is_known() {
    let mut h = house(Some("PGPG"), known(), Status::Known);
    let g = of(&mut h.job, &h.walls[3].clone());
    assert_eq!((g.area.value, g.area.status), (Some(40.0), Status::Known), "{:?}", g.area);
    let n = of(&mut h.job, &h.walls[2].clone());
    assert_eq!((n.area.value, n.area.status), (Some(0.0), Status::Known), "{:?}", n.area);
}

// A hip roof has eaves all round: none on any wall (Assumed, as the roof is), and each says an eave is over it.
#[test]
fn a_hip_roof_gives_no_wall_a_gable() {
    let mut h = house(Some("PPPP"), assumed(), Status::Assumed);
    for p in h.walls.clone() {
        let g = of(&mut h.job, &p);
        assert_eq!((g.area.value, g.area.status), (Some(0.0), Status::Assumed), "{p}: {:?}", g.area);
        assert!(g.why.contains("an eave is over it"), "{p}: {}", g.why);
    }
}

// No roof drawn: none, and it says there is no roof over it.
#[test]
fn a_wall_with_no_roof_over_it_has_no_gable_and_says_so() {
    let mut h = house(None, assumed(), Status::Assumed);
    for p in h.walls.clone() {
        let g = of(&mut h.job, &p);
        assert_eq!(g.area.value, Some(0.0), "{p}");
        assert!(g.top.is_empty());
        assert!(g.why.contains("no roof"), "{p}: {}", g.why);
    }
}

// A shed: one plane, its eave on the south (y = -1), rising north at 6 in 12; the other three edges stand upright.
// Over the walls' lines: 9.5 ft at y = 0, 17.5 ft at y = 16. East wall (runs south to north), 16 x (0.5 + 8.5) / 2 = 72.00 SF,
// top (0, 9.5), (16, 17.5). West wall (runs north to south): the same area, top (0, 17.5), (16, 9.5).
// North wall, under the high edge: 24 x 8.5 = 204.00 SF, top level at 17.5 ft: (0, 17.5), (24, 17.5). South: eave, none.
#[test]
fn a_shed_roofs_sides_and_high_edge_rise_to_it() {
    let mut h = house(Some("PGGG"), assumed(), Status::Assumed);
    let [south, east, north, west] = [&h.walls[0], &h.walls[1], &h.walls[2], &h.walls[3]].map(|s| s.clone());
    let (e, w, n, s) = (of(&mut h.job, &east), of(&mut h.job, &west), of(&mut h.job, &north), of(&mut h.job, &south));
    assert_eq!((e.area.value, e.area.status), (Some(72.0), Status::Assumed), "{:?}", e.area);
    assert!(same(&e.top, &[(0.0, 9.5), (16.0, 17.5)]), "{:?}", e.top);
    assert_eq!(w.area.value, Some(72.0), "{:?}", w.area);
    assert!(same(&w.top, &[(0.0, 17.5), (16.0, 9.5)]), "{:?}", w.top);
    assert_eq!(n.area.value, Some(204.0), "{:?}", n.area);
    assert!(same(&n.top, &[(0.0, 17.5), (24.0, 17.5)]), "{:?}", n.top);
    assert_eq!(s.area.value, Some(0.0), "{:?}", s.area);
    assert!(s.why.contains("an eave is over it"), "{}", s.why);
}

// The figure is given where a line names it, as the trim's are: a job no line asks it of carries none (nothing is stale in
// an old file), and a line that names it buys it. net_area is what it was either way: 16 x 9 = 144.00 SF.
#[test]
fn the_figure_is_given_where_a_line_names_it_and_net_area_is_as_it_was() {
    let mut h = house(Some("PGPG"), assumed(), Status::Assumed);
    let west = h.walls[3].clone();
    assert!(measure(&mut h.job, &west, "gable_area").is_none(), "no line names it");
    let before = measure(&mut h.job, &west, "net_area").unwrap();
    assert_eq!(before.value, Some(144.0));
    h.job.add_line(&west, "Lap siding", "net_area + gable_area", Some(Class::Material), false).unwrap();
    h.job.add_line(&west, "Gypsum board", "net_area", Some(Class::Material), false).unwrap();
    h.job.derive();
    let after = measure(&mut h.job, &west, "net_area").unwrap();
    assert_eq!((after.value, after.status), (before.value, before.status), "net_area is as it was");
    let g = measure(&mut h.job, &west, "gable_area").expect("a line names it");
    assert_eq!(g.value, Some(40.0));
    let siding = quantity(&mut h.job, &west, "Lap siding");
    assert_eq!((siding.value, siding.status), (Some(184.0), Status::Assumed), "{siding:?}");
    assert_eq!(quantity(&mut h.job, &west, "Gypsum board").value, Some(144.0), "inside stays on net_area");
    // At $4.00 a square foot: 184 x 4 = $736.00, where it was 144 x 4 = $576.00.
    h.job.set_on_line(&west, "unit_cost", Some("Lap siding"), Some("$4.00/SF"), Status::Known, "made-up price").unwrap();
    h.job.derive();
    let e = h.job.find(&west).unwrap();
    let ext = h.job.world.get::<LineQuantities>(e).unwrap().0["Lap siding"].extension.clone().unwrap();
    assert_eq!(ext.value, Some(736.0), "{ext:?}");
}

// On a house with no gable, a line that names gable_area comes to what net_area alone did, and is as sure.
#[test]
fn a_line_naming_it_on_a_wall_with_none_comes_to_what_it_was_and_as_sure() {
    let mut h = house(Some("PPPP"), known(), Status::Known);
    let south = h.walls[0].clone();
    h.job.add_line(&south, "Lap siding", "net_area + gable_area", Some(Class::Material), false).unwrap();
    h.job.derive();
    let q = quantity(&mut h.job, &south, "Lap siding");
    assert_eq!((q.value, q.status), (Some(216.0), Status::Known), "{q:?}");
    // And it says nothing the line did not say before: a house with no gable is noted as it was.
    assert!(q.note.is_empty(), "{q:?}");
}

// The frame of a gable is not counted, and the wall's framing figures say so where the wall has a gable: not on a wall that has none.
#[test]
fn the_framing_figures_say_a_gables_frame_is_not_counted() {
    let mut h = house(Some("PGPG"), assumed(), Status::Assumed);
    let (south, west) = (h.walls[0].clone(), h.walls[3].clone());
    for w in [&south, &west] {
        h.job.add_line(w, "Studs, 2x6 at 16 in", "studs", Some(Class::Material), false).unwrap();
    }
    h.job.derive();
    let on_west = measure(&mut h.job, &west, "studs").unwrap();
    assert!(on_west.value.is_some());
    assert!(on_west.note.contains("gable") && on_west.note.contains("not counted"), "{}", on_west.note);
    let on_south = measure(&mut h.job, &south, "studs").unwrap();
    assert!(!on_south.note.contains("gable"), "{}", on_south.note);
    // It does not give the wall a gable_area row no line asked for.
    assert!(measure(&mut h.job, &west, "gable_area").is_none());
}

// The job's format does not change: nothing of the gable is saved, and a job read again works it out again.
#[test]
fn nothing_of_it_is_saved() {
    let mut h = house(Some("PGPG"), assumed(), Status::Assumed);
    let west = h.walls[3].clone();
    h.job.add_line(&west, "Lap siding", "net_area + gable_area", Some(Class::Material), false).unwrap();
    h.job.derive();
    let text = file::save(&mut h.job);
    assert!(!text.contains("gable_points") && !text.contains("\"gable_top\""), "no point of it is saved");
    let (mut back, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    back.derive();
    let q = quantity(&mut back, &west, "Lap siding");
    assert_eq!(q.value, Some(184.0));
}

// The starter assemblies' outside lines buy the gable: the weather barrier, the sheathing, the siding and hanging it on
// net_area + gable_area (the sheathing on face_area + gable_area: it is bought over the openings); what is inside the house
// (batts, gypsum board, paint) stays on net_area. West wall, no openings: weather barrier 144 + 40 = 184; batts 144.
#[test]
fn the_starter_outside_walls_price_the_gable_and_the_inside_does_not() {
    let mut h = house(Some("PGPG"), assumed(), Status::Assumed);
    tables::assemblies_in(&mut h.job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    let west = h.walls[3].clone();
    h.job.apply(&west, "Exterior wall, 2x6, lap siding", "the starter library").unwrap();
    h.job.derive();
    for (line, want) in [("Weather barrier", 184.0), ("Wall sheathing", 184.0), ("Lap siding", 184.0), ("Hang the siding", 184.0), ("Batt insulation", 144.0), ("Gypsum board, hung and finished", 144.0), ("Paint, two coats", 144.0)] {
        let q = quantity(&mut h.job, &west, line);
        assert_eq!(q.value.map(|v| (v * 100.0).round() / 100.0), Some(want), "{line}: {q:?}");
    }
    // The frame of the gable is not counted: the labor of framing stays on the face.
    assert_eq!(quantity(&mut h.job, &west, "Frame and sheathe the wall").value, Some(144.0));
}

// With a stone wainscot 3 ft up: the siding is net_area - net_area_below_wainscot + gable_area = (144 - 48) + 40 = 136.00 SF.
#[test]
fn the_wainscot_keeps_its_form_and_the_siding_adds_the_gable() {
    let mut h = house(Some("PGPG"), assumed(), Status::Assumed);
    tables::assemblies_in(&mut h.job, &rows(tables::STARTER_LIBRARY), None).unwrap();
    let west = h.walls[3].clone();
    h.job.apply(&west, "Exterior wall, 2x6, lap siding over a stone wainscot", "the starter library").unwrap();
    h.job.derive();
    assert_eq!(quantity(&mut h.job, &west, "Lap siding").value, Some(136.0));
    assert_eq!(quantity(&mut h.job, &west, "Hang the siding").value, Some(136.0));
    assert_eq!(quantity(&mut h.job, &west, "Weather barrier").value, Some(184.0));
}

// A roof the tracing has not given an eave height: the gable is not known, and says why; a line that names it has no figure.
#[test]
fn a_roof_that_cannot_be_placed_leaves_the_gable_unknown_and_says_why() {
    let mut h = house(Some("PGPG"), assumed(), Status::Assumed);
    let west = h.walls[3].clone();
    let plane = {
        let mut q = h.job.world.query::<(&Node, &Kind)>();
        q.iter(&h.job.world).find(|(_, k)| **k == Kind::RoofPlane).map(|(n, _)| n.path.clone()).unwrap()
    };
    // Take the pitch of a plane away: its height over the wall's line is not known.
    let e = h.job.find(&plane).unwrap();
    h.job.world.get_mut::<Pitch>(e).unwrap().0.value = None;
    h.job.world.get_mut::<Pitch>(e).unwrap().0.status = Status::Unknown;
    let g = of(&mut h.job, &west);
    assert_eq!(g.area.value, None, "{:?}", g.area);
    assert_eq!(g.area.status, Status::Unknown);
    assert!(!g.area.note.is_empty());
}

// None rests on where the roof's edge lies and what kind it is: a Known wall under the eave of an Assumed roof has an Assumed none,
// names the eave among what it rests on, and a siding quantity that adds it reads Assumed. A roof that is not there rests on nothing.
#[test]
fn none_under_an_eave_is_as_sure_as_the_roof_it_rests_on() {
    let mut h = house(Some("PPPP"), known(), Status::Assumed);
    let south = h.walls[0].clone();
    h.job.add_line(&south, "Lap siding", "net_area + gable_area", Some(Class::Material), false).unwrap();
    h.job.derive();
    let g = measure(&mut h.job, &south, "gable_area").unwrap();
    assert_eq!((g.value, g.status), (Some(0.0), Status::Assumed), "{g:?}");
    assert!(g.inputs.iter().any(|i| i.starts_with("eave-")), "{:?}", g.inputs);
    let q = quantity(&mut h.job, &south, "Lap siding");
    assert_eq!((q.value, q.status), (Some(216.0), Status::Assumed), "{q:?}");
    // Known roof, Known wall: Known. No roof: Known 0, resting on nothing.
    let mut k = house(Some("PPPP"), known(), Status::Known);
    assert_eq!(of(&mut k.job, &k.walls[0].clone()).area.status, Status::Known);
    let mut n = house(None, known(), Status::Known);
    assert_eq!(of(&mut n.job, &n.walls[0].clone()).area.status, Status::Known);
}
