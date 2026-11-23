//! What call 179 (A) decided about the scan's walls, each as a test on a made-up scan with the
//! figure worked by hand (`found_scenes` builds the scan). Each was written to fail first.

use crate::found::*;
use crate::found_scenes::Scene;
use crate::scan::Scan;

/// The house of the command line's `--scene house`: a room `w` by `d`, `h` high, with a partition
/// 4 1/2 in thick at 18 ft, `dens` points to the square foot.
fn house(w: f64, d: f64, h: f64, dens: f64, turn: f64) -> Scan {
    let mut s = Scene::new(dens, 11);
    s.room(w, d, h);
    s.thick([18.0, 0.0], [18.0, d], 0.375, None, &[], &[]);
    s.scan(turn)
}

fn said_any(f: &Found, word: &str) -> bool {
    f.left_out.iter().any(|l| l.contains(word)) || f.not_likely.as_deref().is_some_and(|l| l.contains(word))
}

// ---- A7: the wrong measure, or the wrong way up ----

#[test]
fn a_floor_to_ceiling_over_15_ft_is_not_likely_the_house_and_said() {
    // The 30 by 16 ft house read with the wrong way up reads 16 ft from floor to ceiling (the room's depth).
    let f = find(&house(30.0, 16.0, 15.5, 60.0, 0.0));
    let why = f.not_likely.expect("a room 15 ft 6 in high is not likely a house");
    assert!(why.contains("not likely the house") && why.contains("check the measure and which way is up"), "{why}");
    assert!(why.contains("15' 6\""), "it says what it read: {why}");
}

#[test]
fn a_floor_to_ceiling_under_6_ft_is_not_likely_the_house_and_said() {
    let f = find(&house(30.0, 16.0, 5.5, 60.0, 0.0));
    let why = f.not_likely.expect("a room 5 ft 6 in high is not likely a house");
    assert!(why.contains("5' 6\""), "{why}");
}

#[test]
fn a_scan_too_small_to_hold_a_ceiling_says_how_small_it_is() {
    // A house in meters read as feet is a third the size: 2 ft 5 in high, and no ceiling shows.
    let mut scan = house(30.0, 16.0, 8.0, 60.0, 0.0);
    for p in scan.points.iter_mut() {
        *p = [p[0] * 0.3048, p[1] * 0.3048, p[2] * 0.3048];
    }
    let f = find(&scan);
    let why = f.not_likely.expect("a scan 2 ft 5 in high is not likely a house");
    assert!(why.contains("check the measure") && why.contains("the whole scan is only 2' "), "{why}");
}

#[test]
fn a_house_over_150_ft_is_not_likely_and_an_ordinary_one_is_not_said_to_be_odd() {
    let f = find(&house(170.0, 16.0, 8.0, 30.0, 0.0));
    let why = f.not_likely.expect("a house 170 ft long is not likely");
    assert!(why.contains("not likely the house"), "{why}");
    let ok = find(&house(30.0, 16.0, 8.0, 60.0, 0.0));
    assert!(ok.not_likely.is_none(), "{:?}", ok.not_likely);
    assert!(!said_any(&ok, "not likely"));
}

// ---- A3: a cabinet bank to the ceiling is not laid as a wall on the scan's word ----

/// A 20 by 14 ft room, 8 ft high, a bank of cabinets 10 ft long and 2 ft deep against the north wall
/// (east 2 to 12 ft), its top `top` high; its back, and the wall behind it below its top, not seen.
fn bank_room(top: f64, turn: f64) -> Scan {
    let mut s = Scene::new(100.0, 21);
    s.room_hiding(20.0, 14.0, 8.0, &[(2, [8.0, 18.0, top])]);
    s.bx([2.0, 12.0, 12.0, 14.0], top, &["north"]);
    s.scan(turn)
}

fn index_at(f: &Found, north: f64) -> Option<usize> {
    f.walls.iter().position(|w| (w.from[1] - north).abs() < 1.0 / 12.0 && (w.to[1] - north).abs() < 1.0 / 12.0 && w.length() > 8.0 && w.length() < 11.0)
}

#[test]
fn a_bank_of_cabinets_to_the_ceiling_with_free_ends_and_a_wall_behind_is_said_and_not_laid_alone() {
    for turn in [0.0, 33.0] {
        let f = find(&bank_room(7.8, turn));
        let bank = if turn == 0.0 { index_at(&f, 12.0) } else { f.walls.iter().position(|w| w.doubt.is_some()) };
        let bank = bank.unwrap_or_else(|| panic!("turned {turn}: the bank's face is found as a face that reaches the ceiling: {:?}", f.walls));
        let why = f.walls[bank].doubt.as_deref().unwrap_or_else(|| panic!("turned {turn}: no doubt said of the bank: {:?}", f.walls[bank]));
        assert!(why.contains("a wall or something tall standing at the wall"), "{why}");
        assert_eq!(f.walls.iter().filter(|w| w.doubt.is_some()).count(), 1, "turned {turn}: only the bank is in doubt: {:?}", f.walls);
        // Never laid by saying "lay" alone, laid when the person says it is a wall.
        let (laid, not_laid) = lay(&f, Some(0.5));
        assert!(not_laid.contains(&bank) && laid.iter().all(|l| l.of != bank), "turned {turn}: the bank was laid on the scan's word: {not_laid:?}");
        let (laid, not_laid) = lay_with(&f, Some(0.5), &[bank]);
        assert!(!not_laid.contains(&bank) && laid.iter().any(|l| l.of == bank), "turned {turn}: said to be a wall, it is laid");
    }
}

#[test]
fn a_bank_the_ceiling_does_not_reach_is_a_thing_as_before() {
    let f = find(&bank_room(6.5, 0.0));
    assert!(f.walls.iter().all(|w| w.doubt.is_none()), "{:?}", f.walls);
    assert!(!f.things.is_empty(), "a bank 6 ft 6 in high stands short: {:?}", f.things);
}

#[test]
fn a_two_faced_wall_with_free_ends_is_a_wall_and_not_in_doubt() {
    // A wall of two faces 4 1/2 in apart that stops in the room: both faces seen, so a wall (call 173),
    // though each is a face with a face behind it within 3 ft.
    let mut s = Scene::new(100.0, 22);
    s.room(20.0, 14.0, 8.0);
    s.thick([10.0, 0.0], [10.0, 8.0], 0.375, None, &[], &[]);
    let f = find(&s.scan(0.0));
    let part = f.walls.iter().find(|w| w.thickness.is_some()).expect("a wall of two faces");
    assert!(part.doubt.is_none(), "{part:?}");
    assert!(f.walls.iter().all(|w| w.doubt.is_none()), "{:?}", f.walls);
}

// ---- A2: a wall half hidden is found, and whether the walls close is said ----

/// A 30 by 16 ft room, 8 ft high, cabinets 2 ft deep and 7 ft high before the north wall from its east
/// end for `pct` in 100 of it; the wall behind them is not seen below 7 ft.
fn hidden_room(pct: f64, turn: f64) -> Scan {
    let hide = 30.0 * pct / 100.0;
    let mut s = Scene::new(100.0, 31);
    s.room_hiding(30.0, 16.0, 8.0, &[(2, [0.0, hide, 7.0])]);
    s.bx([30.0 - hide, 14.0, 30.0, 16.0], 7.0, &["north"]);
    s.scan(turn)
}

#[test]
fn a_wall_hidden_behind_cabinets_is_found_from_what_shows_above_and_between_them() {
    for (pct, turn) in [(30.0, 0.0), (60.0, 0.0), (90.0, 0.0), (60.0, 33.0)] {
        let f = find(&hidden_room(pct, turn));
        assert_eq!(f.walls.len(), 4, "{pct} per cent hidden, turned {turn}: {:?}", f.walls);
        let north = f.walls.iter().find(|w| w.length() > 29.0 && (turn != 0.0 || (w.from[1] - 16.0).abs() < 0.2)).unwrap_or_else(|| panic!("{pct}%/{turn}: no wall 30 ft long: {:?}", f.walls));
        assert!((north.length() - 30.0).abs() < 0.5, "{pct}%: {}", north.length());
        // What was not seen is said, with how much: the cabinets' length, to a few inches.
        let said: f64 = north.hidden.iter().map(|h| h.to - h.from).sum();
        assert!((said - 30.0 * pct / 100.0).abs() < 0.6, "{pct}%/{turn} hidden said {said}: {:?}", north.hidden);
        assert!(f.open_ends.is_empty(), "{pct}%/{turn}: the four walls close: {:?}", f.open_ends);
        assert!(f.left_out.iter().all(|l| !l.contains("not taken as a wall")), "{:?}", f.left_out);
    }
}

#[test]
fn walls_hidden_behind_cabinets_on_three_sides_are_found_and_the_hidden_stretches_said() {
    // A 20 by 14 ft room, cabinets 2 ft deep and 6 ft 6 in high along the south, north and west walls.
    let mut s = Scene::new(100.0, 32);
    s.room_hiding(20.0, 14.0, 8.0, &[(0, [1.0, 19.0, 6.5]), (2, [1.0, 19.0, 6.5]), (3, [2.0, 12.0, 6.5])]);
    s.bx([1.0, 0.0, 19.0, 2.0], 6.5, &["south"]);
    s.bx([1.0, 12.0, 19.0, 14.0], 6.5, &["north"]);
    s.bx([0.0, 2.0, 2.0, 12.0], 6.5, &["west"]);
    let f = find(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{:?}", f.walls);
    assert!(f.walls.iter().filter(|w| !w.hidden.is_empty()).count() >= 3, "three walls are hidden in part: {:?}", f.walls);
    assert!(f.open_ends.is_empty(), "{:?}", f.open_ends);
}

#[test]
fn the_walls_found_say_where_they_do_not_close() {
    // A room with no north wall at all: the west and east walls' north ends meet nothing.
    let mut s = Scene::new(100.0, 33);
    s.floor([0.0, 0.0, 30.0, 16.0]);
    s.ceiling([0.0, 0.0, 30.0, 16.0], 8.0, 8.0);
    s.face([0.0, 0.0], [30.0, 0.0], None, &[], &[]);
    s.face([30.0, 0.0], [30.0, 16.0], None, &[], &[]);
    s.face([0.0, 16.0], [0.0, 0.0], None, &[], &[]);
    let f = find(&s.scan(0.0));
    assert_eq!(f.walls.len(), 3, "{:?}", f.walls);
    assert_eq!(f.open_ends.len(), 2, "{:?}", f.open_ends);
    assert!(f.open_ends.iter().all(|(_, p)| (p[1] - 16.0).abs() < 0.3 && (p[0].abs() < 0.3 || (p[0] - 30.0).abs() < 0.3)), "{:?}", f.open_ends);
}

#[test]
fn the_wall_behind_a_bank_of_cabinets_is_one_wall_with_the_gap_said_not_seen() {
    for turn in [0.0, 33.0] {
        let f = find(&bank_room(7.8, turn));
        let long: Vec<&Wall> = f.walls.iter().filter(|w| w.doubt.is_none() && w.length() > 19.0 && w.length() < 21.0).collect();
        // The room is 20 ft each way on the south and north: both are one wall; the north has the bank before it.
        assert_eq!(long.len(), 2, "turned {turn}: south and north, each one wall: {:?}", f.walls);
        let behind = long.iter().find(|w| !w.hidden.is_empty()).unwrap_or_else(|| panic!("turned {turn}: the gap behind the bank is said: {:?}", f.walls));
        let said: f64 = behind.hidden.iter().map(|h| h.to - h.from).sum();
        assert!((said - 10.0).abs() < 1.5, "turned {turn}: 10 ft behind the bank, said {said}: {:?}", behind.hidden);
        assert!(f.open_ends.is_empty(), "turned {turn}: the walls close: {:?}", f.open_ends);
    }
}

// ---- A1: a scan too thin is said, and a thin one is read by its own thickness ----

#[test]
fn a_thin_scan_is_read_by_its_own_thickness_and_finds_the_walls() {
    for (dens, turn) in [(10.0, 0.0), (20.0, 0.0), (20.0, 33.0), (40.0, 60.0)] {
        let f = find(&house(30.0, 16.0, 8.0, dens, turn));
        assert!(f.too_thin.is_none(), "{dens} to the square foot, turned {turn}: {:?} {:?}", f.too_thin, f.density);
        assert_eq!(f.walls.len(), 5, "{dens} to the square foot, turned {turn}: {:?}", f.walls);
        let part = f.walls.iter().find(|w| w.thickness.is_some()).expect("the partition of two faces");
        assert!((part.thickness.unwrap() - 0.375).abs() < 1.0 / 24.0, "{dens}/{turn}: {:?}", part.thickness);
        let d = f.density.expect("it says how thick the scan is");
        assert!((d - dens).abs() < 0.2 * dens, "said {d} to the square foot of a scan made at {dens}");
        assert!(f.things.is_empty() && f.open_ends.is_empty(), "{dens}/{turn}: {:?} {:?}", f.things, f.open_ends);
        assert!(f.thin_note.is_some(), "a scan thinner than the measures were made for says what that does");
    }
}

#[test]
fn a_scan_too_thin_to_tell_a_wall_says_so_and_finds_nothing() {
    let f = find(&house(30.0, 16.0, 8.0, 4.0, 0.0));
    let why = f.too_thin.clone().expect("4 points to the square foot is too thin");
    assert!(why.contains("too thin to tell a wall from furniture") && why.contains("4 points to the square foot"), "{why}");
    assert!(f.walls.is_empty() && f.things.is_empty(), "nothing is found, and nothing is called furniture: {:?} {:?}", f.walls, f.things);
    assert!(said_any(&f, "too thin"));
}

#[test]
fn a_dense_scan_says_how_thick_it_is_and_nothing_about_thin() {
    let f = find(&house(30.0, 16.0, 8.0, 100.0, 0.0));
    assert!(f.thin_note.is_none() && f.too_thin.is_none());
    assert!(f.density.unwrap() > 80.0, "{:?}", f.density);
}

// ---- A4: a wall is judged against the ceiling over it ----

/// Two rooms 15 by 16 ft, the west 8 ft high and the east 10 ft; a partition 4 1/2 in thick between them, 8 ft high.
fn two_heights(turn: f64) -> Scan {
    let mut s = Scene::new(100.0, 41);
    s.floor([0.0, 0.0, 30.0, 16.0]);
    s.ceiling([0.0, 0.0, 15.0, 16.0], 8.0, 8.0);
    s.ceiling([15.0, 0.0, 30.0, 16.0], 10.0, 10.0);
    for (a, b) in [([0.0, 0.0], [30.0, 0.0]), ([30.0, 0.0], [30.0, 16.0]), ([30.0, 16.0], [0.0, 16.0]), ([0.0, 16.0], [0.0, 0.0])] {
        s.face(a, b, None, &[], &[]);
    }
    s.thick([15.0, 0.0], [15.0, 16.0], 0.375, Some(8.0), &[], &[]);
    s.scan(turn)
}

#[test]
fn two_ceiling_heights_are_said_and_each_wall_is_judged_against_the_ceiling_over_it() {
    for turn in [0.0, 33.0] {
        let f = find(&two_heights(turn));
        let note = f.ceiling_note.clone().unwrap_or_else(|| panic!("turned {turn}: the ceiling is not one height, and that is said"));
        assert!(note.contains("the ceiling is not one height: 8' 0\" and 10' 0\""), "{note}");
        assert_eq!(f.walls.len(), 5, "turned {turn}: the low room's walls are not furniture: {:?} {:?}", f.walls, f.things);
        assert!(f.things.is_empty(), "turned {turn}: {:?}", f.things);
        // Each wall's own ceiling: the west wall 8 ft, the east 10, the partition the lower of its two sides, 8.
        let mut heights: Vec<f64> = f.walls.iter().map(|w| w.height.expect("a wall's height")).collect();
        heights.sort_by(f64::total_cmp);
        // (the south and north walls run under both and are laid at the height over their middle: either, by a tie)
        assert!(heights.iter().filter(|h| (**h - 8.0).abs() < 0.3).count() >= 2 && heights.iter().filter(|h| (**h - 10.0).abs() < 0.3).count() >= 1, "turned {turn}: {heights:?}");
        assert!(note.contains("the step in its top is not drawn"), "what could not be done is said: {note}");
        assert!(f.open_ends.is_empty(), "turned {turn}: {:?}", f.open_ends);
    }
}

#[test]
fn a_sloped_ceiling_is_a_ceiling_the_walls_are_found_and_the_slope_is_said() {
    // A room 30 by 16 ft whose ceiling rises from 8 ft at the south wall to 12 ft at the north.
    let mut s = Scene::new(100.0, 42);
    s.floor([0.0, 0.0, 30.0, 16.0]);
    s.ceiling([0.0, 0.0, 30.0, 16.0], 8.0, 12.0);
    for (a, b) in [([0.0, 0.0], [30.0, 0.0]), ([30.0, 0.0], [30.0, 16.0]), ([30.0, 16.0], [0.0, 16.0]), ([0.0, 16.0], [0.0, 0.0])] {
        s.face(a, b, None, &[], &[]);
    }
    let f = find(&s.scan(0.0));
    assert!(!f.no_ceiling, "a ceiling shows: it is not level");
    let note = f.ceiling_note.clone().expect("the slope is said");
    assert!(note.contains("the ceiling slopes, from 8' ") && (note.contains(" to 12' ") || note.contains(" to 11' 1")), "{note}");
    assert!(note.contains("its sloped top is not drawn"), "what could not be done there is said: {note}");
    assert_eq!(f.walls.len(), 4, "{:?} {:?}", f.walls, f.things);
    // The south wall under 8 ft, the north under 12; the east and west under what they span, laid at their middle: 10.
    let at = |north: f64| f.walls.iter().find(|w| (w.from[1] - north).abs() < 0.3 && (w.to[1] - north).abs() < 0.3).and_then(|w| w.height).unwrap_or(0.0);
    assert!((at(0.0) - 8.0).abs() < 0.6 && (at(16.0) - 12.0).abs() < 0.6, "{} {}", at(0.0), at(16.0));
    assert!(f.open_ends.is_empty(), "{:?}", f.open_ends);
}

#[test]
fn a_house_of_one_ceiling_height_says_nothing_of_heights() {
    let f = find(&house(30.0, 16.0, 8.0, 80.0, 0.0));
    assert!(f.ceiling_note.is_none(), "{:?}", f.ceiling_note);
    assert!(f.walls.iter().all(|w| (w.height.unwrap() - 8.0).abs() < 0.1), "{:?}", f.walls.iter().map(|w| w.height).collect::<Vec<_>>());
}

// ---- A5: never leaning; a thin wall beside a thick one; a short return said ----

/// A 30 by 16 ft room, 8 ft high, a 12 in wall at 10 ft east and a 3 1/2 in wall at 20 ft east, both across, both faces.
fn thick_and_thin(turn: f64) -> Scan {
    let mut s = Scene::new(100.0, 51);
    s.room(30.0, 16.0, 8.0);
    s.thick([10.0, 0.0], [10.0, 16.0], 1.0, None, &[], &[]);
    s.thick([20.0, 0.0], [20.0, 16.0], 3.5 / 12.0, None, &[], &[]);
    s.scan(turn)
}

/// Every wall found lies along one of the house's two directions to within half a degree: none leans.
fn none_lean(f: &Found, turn: f64) {
    for w in &f.walls {
        let (dx, dy) = (w.to[0] - w.from[0], w.to[1] - w.from[1]);
        let angle = dy.atan2(dx).to_degrees() - turn;
        let off = (angle.rem_euclid(90.0)).min(90.0 - angle.rem_euclid(90.0));
        assert!(off < 0.5, "turned {turn}: a wall leans {off} degrees: {w:?}");
    }
}

#[test]
fn a_3_in_wall_beside_a_12_in_wall_is_found_whole_and_nothing_leans() {
    for turn in [0.0, 33.0] {
        let f = find(&thick_and_thin(turn));
        assert_eq!(f.walls.len(), 6, "turned {turn}: {:?}", f.walls);
        let mut thick: Vec<f64> = f.walls.iter().filter_map(|w| w.thickness).collect();
        thick.sort_by(f64::total_cmp);
        assert!(thick.len() == 2 && (thick[0] - 3.5 / 12.0).abs() < 0.5 / 12.0 && (thick[1] - 1.0).abs() < 0.5 / 12.0, "turned {turn}: {thick:?}");
        none_lean(&f, turn);
        assert!(f.walls.iter().all(|w| w.length() > 15.0), "no stub of a wall: {:?}", f.walls);
    }
}

#[test]
fn a_closet_of_3_in_walls_is_found_without_leaning_walls_and_the_short_return_is_said() {
    // A 20 by 12 ft room, a closet of two walls 3 1/2 in thick 2 ft apart (6 and 8 ft north, 0 to 6 ft east) and a 2 ft cross wall.
    let mut s = Scene::new(100.0, 52);
    s.room(20.0, 12.0, 8.0);
    let t = 3.5 / 12.0;
    s.thick([0.0, 6.0], [6.0, 6.0], t, None, &[], &[]);
    s.thick([0.0, 8.0], [6.0, 8.0], t, None, &[], &[]);
    s.thick([6.0, 6.0], [6.0, 8.0], t, None, &[], &[]);
    let f = find(&s.scan(0.0));
    none_lean(&f, 0.0);
    let thin: Vec<&Wall> = f.walls.iter().filter(|w| w.thickness.is_some_and(|t| (t - 3.5 / 12.0).abs() < 0.5 / 12.0)).collect();
    assert_eq!(thin.len(), 2, "the two 6 ft closet walls, each two faces: {:?}", f.walls);
    assert!(f.walls.len() >= 6 && f.things.is_empty(), "{:?}", f.walls);
    assert!(f.left_out.iter().any(|l| l.contains("under 2 ft long")), "the 2 ft cross wall is not found, and that is said: {:?}", f.left_out);
}

// ---- A6: two faces up to 24 in apart with no floor seen between them are one wall ----

fn thick_partition(inches: f64, turn: f64) -> Scan {
    let mut s = Scene::new(100.0, 61);
    s.room(30.0, 16.0, 8.0);
    s.thick([15.0, 0.0], [15.0, 15.5], inches / 12.0, None, &[], &[]);
    s.scan(turn)
}

#[test]
fn an_18_in_wall_and_a_22_in_wall_are_one_wall_each() {
    for (inches, turn) in [(18.0, 0.0), (22.0, 0.0), (18.0, 33.0)] {
        let f = find(&thick_partition(inches, turn));
        assert_eq!(f.walls.len(), 5, "{inches} in, turned {turn}: not two walls: {:?}", f.walls);
        let thick = f.walls.iter().find_map(|w| w.thickness).unwrap_or_else(|| panic!("{inches} in: a wall of two faces: {:?}", f.walls));
        assert!((thick * 12.0 - inches).abs() < 0.75, "{inches} in, turned {turn}: {} in", thick * 12.0);
        assert!(f.open_ends.is_empty(), "{:?}", f.open_ends);
    }
}

#[test]
fn two_walls_20_in_apart_with_the_floor_seen_between_them_are_two_walls() {
    // Two partitions 4 1/2 in thick, their faces 20 in apart, the floor of the room between them.
    let mut s = Scene::new(100.0, 62);
    s.room(30.0, 16.0, 8.0);
    s.thick([10.0, 0.0], [10.0, 16.0], 0.375, None, &[], &[]);
    s.thick([10.0 + 0.1875 * 2.0 + 20.0 / 12.0, 0.0], [10.0 + 0.1875 * 2.0 + 20.0 / 12.0, 16.0], 0.375, None, &[], &[]);
    let f = find(&s.scan(0.0));
    assert_eq!(f.walls.iter().filter(|w| w.thickness.is_some()).count(), 2, "two walls of two faces, not one of 3 ft: {:?}", f.walls);
    assert!(f.walls.iter().filter_map(|w| w.thickness).all(|t| t < 0.5), "{:?}", f.walls);
}

// ---- A9: the small ones ----

#[test]
fn doors_an_inch_apart_in_the_noise_are_one_type() {
    let wall = |openings: Vec<Opening>| Wall { from: [0.0, 0.0], to: [30.0, 0.0], thickness: Some(0.5), seen_from: None, openings, ..Wall::default() };
    let door = |from: f64, to: f64, head: f64| Opening { window: false, from, to, sill: 0.0, head };
    // Three doors read 3 ft 0 in by 6 ft 7 in, 3 ft 0 in by 6 ft 8 in and 3 ft 1/2 in by 6 ft 7 1/2 in; a fourth, 2 ft 6 in, is its own.
    let found = Found { walls: vec![wall(vec![door(2.0, 5.0, 6.6), door(10.0, 13.0, 6.667), door(15.0, 18.04, 6.625), door(22.0, 24.5, 6.667)])], ..Found::default() };
    let (laid, _) = lay(&found, None);
    let sizes: Vec<(i64, i64)> = laid[0].openings.iter().map(|o| (((o.to - o.from) * 12.0).round() as i64, (o.head * 12.0).round() as i64)).collect();
    assert_eq!(sizes[0], sizes[1], "{sizes:?}");
    assert_eq!(sizes[0], sizes[2], "{sizes:?}");
    assert_ne!(sizes[0], sizes[3], "{sizes:?}");
    // Each is still where it was found: about its own middle.
    assert!((laid[0].openings[1].from + laid[0].openings[1].to) / 2.0 - 11.5 < 1e-9);
}

#[test]
fn a_wall_of_two_faces_ends_at_the_near_face_of_the_wall_it_meets() {
    // A room whose four walls are each 8 in thick, both faces scanned: the inner faces end at the inner faces of the walls
    // they meet (0 and 30 ft east), the outer faces at the outer corners (8 in beyond): the wall ends at the inner face,
    // not 4 in past it.
    let mut s = Scene::new(100.0, 71);
    // (call 188, 2: the outer faces are scanned from outside, so there is floor outside them: a face is seen from the side it has floor on, and the two
    // faces of a wall are seen from opposite sides. Without it the outer faces are seen from the room, through the wall, as the inner ones are: one wall
    // scanned twice.)
    s.floor([-8.0, -8.0, 38.0, 24.0]);
    s.ceiling([-8.0, -8.0, 38.0, 24.0], 8.0, 8.0);
    let t = 8.0 / 12.0;
    // Each wall's inner face runs between the inner faces of the walls it meets, its outer face between their outer ones.
    for (a, b) in [([0.0, 0.0], [30.0, 0.0]), ([30.0, 0.0], [30.0, 16.0]), ([30.0, 16.0], [0.0, 16.0]), ([0.0, 16.0], [0.0, 0.0])] {
        s.face(a, b, None, &[], &[]);
    }
    for (a, b) in [([-t, -t], [30.0 + t, -t]), ([30.0 + t, -t], [30.0 + t, 16.0 + t]), ([30.0 + t, 16.0 + t], [-t, 16.0 + t]), ([-t, 16.0 + t], [-t, -t])] {
        s.face(a, b, None, &[], &[]);
    }
    let f = find(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{:?}", f.walls);
    let south = f.walls.iter().find(|w| (w.from[1] + t / 2.0).abs() < 0.2 && (w.to[1] + t / 2.0).abs() < 0.2).expect("the south wall");
    assert!(south.from[0] > -0.15 && south.to[0] < 30.15, "ends at the faces it meets, not past them: {south:?}");
    assert!((south.thickness.unwrap() - t).abs() < 0.05, "{south:?}");
}
