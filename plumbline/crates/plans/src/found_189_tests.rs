//! What call 189 decided about the scan's finder built on the published method (the points near the ceiling, where furniture does not reach, laid on the
//! plan as how dense they are; the outlines of the dense parts, made straight; opposite faces paired into walls; each wall's own face read for openings),
//! each as a test on a made-up scan or a small made-up raster with the figure worked by hand. Written before the finder was.

use crate::found::*;
use crate::found_189 as c189;
use crate::found_scenes::Scene;
use crate::scan::Scan;

const IN: f64 = 1.0 / 12.0;

fn said(f: &Found) -> String {
    format!("{:?} {:?}", f.walls, f.left_out)
}

/// A 12 by 14 ft room, 8 ft high, scanned as a phone scans: `dens` to the square foot, every wall `lean` off plumb and `extra` rougher than the
/// scan's own scatter; the south wall bowed `bow` deep over its 12 ft.
fn phone_room(dens: f64, bow: f64, lean: f64, extra: f64, turn: f64) -> Scan {
    let mut s = Scene::new(dens, 19);
    s.floor([0.0, 0.0, 12.0, 14.0]);
    s.ceiling([0.0, 0.0, 12.0, 14.0], 8.0, 8.0);
    s.face_rough([0.0, 0.0], [12.0, 0.0], None, bow, lean, extra, 0.0, dens);
    s.face_rough([12.0, 0.0], [12.0, 14.0], None, 0.0, lean, extra, 0.0, dens);
    s.face_rough([12.0, 14.0], [0.0, 14.0], None, 0.0, lean, extra, 0.0, dens);
    s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, lean, extra, 0.0, dens);
    s.scan(turn)
}

fn along(w: &Wall, east: bool) -> bool {
    if east {
        (w.to[1] - w.from[1]).abs() < 1.0
    } else {
        (w.to[0] - w.from[0]).abs() < 1.0
    }
}

// ---- the raster, its outlines, and making them straight: the pure rules ----

/// A mask `w` wide from rows of text (the first row is the north), '#' occupied.
fn mask(rows: &[&str]) -> c189::Mask {
    let h = rows.len();
    let w = rows[0].len();
    let mut m = c189::Mask::new(w, h);
    for (r, row) in rows.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            if ch == '#' {
                m.set(c, h - 1 - r);
            }
        }
    }
    m
}

#[test]
fn a_block_has_one_outline_and_a_ring_has_two_one_in_and_one_out() {
    let block = mask(&["......", ".####.", ".####.", ".####.", "......"]);
    let loops = block.borders();
    assert_eq!(loops.len(), 1, "{loops:?}");
    // 4 corners of a 4 by 3 block, the occupied side kept on the left (counterclockwise): the area is +12.
    let area: f64 = c189::area(&loops[0]);
    assert!((area - 12.0).abs() < 1e-9, "{area}");
    let ring = mask(&["#######", "#.....#", "#.....#", "#######"]);
    let loops = ring.borders();
    assert_eq!(loops.len(), 2, "{loops:?}");
    let mut areas: Vec<f64> = loops.iter().map(|l| c189::area(l)).collect();
    areas.sort_by(f64::total_cmp);
    // The hole is walked the other way round (clockwise) with the occupied side still on the left: 5 by 2 = -10; the outside 7 by 4 = +28.
    assert!((areas[0] + 10.0).abs() < 1e-9 && (areas[1] - 28.0).abs() < 1e-9, "{areas:?}");
}

#[test]
fn two_blocks_touching_at_a_corner_are_one_outline() {
    // Eight neighbours join (the published method's contours are of 8-connected parts): one outline round both.
    let m = mask(&["..##", "..##", "##..", "##.."]);
    assert_eq!(m.borders().len(), 1);
}

#[test]
fn an_outline_is_made_straight_to_within_a_tolerance_and_keeps_its_corners() {
    // A 20 by 10 rectangle whose long sides wobble by 1 unit: made straight to within 2 it is four corners; to within 0.2 it is not.
    let mut pts: Vec<[f64; 2]> = Vec::new();
    for x in 0..=20 {
        pts.push([f64::from(x), if x % 2 == 0 { 0.0 } else { 1.0 }]);
    }
    for y in 1..=10 {
        pts.push([20.0, f64::from(y)]);
    }
    for x in (0..20).rev() {
        pts.push([f64::from(x), if x % 2 == 0 { 10.0 } else { 9.0 }]);
    }
    for y in (1..10).rev() {
        pts.push([0.0, f64::from(y)]);
    }
    let rough = c189::straighten(&pts, 2.0, 0.0);
    assert_eq!(rough.len(), 4, "{rough:?}");
    let fine = c189::straighten(&pts, 0.2, 0.0);
    assert!(fine.len() > 8, "{}", fine.len());
}

#[test]
fn a_gentle_bow_is_one_stretch_where_a_corner_is_two() {
    // 12 ft long, 6 in deep in the middle (1 in 24): with the bow allowed as 1 in 20 of the length it is one line; a corner of 6 ft out is not.
    let bow: Vec<[f64; 2]> = (0..=24).map(|i| { let u = f64::from(i) / 12.0 - 1.0; [f64::from(i) * 0.5, 0.5 * (1.0 - u * u)] }).collect();
    assert_eq!(c189::straighten_open(&bow, 0.05, 1.0 / 20.0).len(), 2);
    let corner: Vec<[f64; 2]> = (0..=12).map(|i| [f64::from(i), 0.0]).chain((1..=12).map(|i| [12.0, f64::from(i)])).collect();
    assert_eq!(c189::straighten_open(&corner, 0.05, 1.0 / 20.0).len(), 3);
}

// ---- the finder, on made-up scans ----

#[test]
fn a_wall_behind_furniture_is_found_from_its_top() {
    // The south wall of a 12 by 14 ft room is not seen below 6 ft 6 in from 2 ft to 10 ft along it: a cabinet 6 ft high stands before it.
    // The points that are near the ceiling are the wall's; the foot is not seen, but something stands before it: the wall is a wall, whole.
    let mut s = Scene::new(150.0, 31);
    s.room_hiding(12.0, 14.0, 8.0, &[(0, [2.0, 10.0, 6.5])]);
    s.bx([2.0, 0.3, 10.0, 2.3], 6.0, &["south"]);
    let f = find_by_ceiling(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{}", said(&f));
    assert!(f.walls.iter().all(|w| w.doubt.is_none()), "none in doubt: {}", said(&f));
    let south = f.walls.iter().find(|w| w.from[1].abs() < 1.0 && w.to[1].abs() < 1.0).expect("the south wall");
    assert!((south.length() - 12.0).abs() < 1.0, "{}", south.length());
    assert!(f.open_ends.is_empty(), "the walls close: {:?}", f.open_ends);
}

#[test]
fn two_faces_with_a_thickness_between_are_one_wall_with_that_thickness() {
    // A wall 6 in thick on the centre line x = 10, 8 ft long from the south wall, both its faces seen, no floor between them.
    let mut s = Scene::new(150.0, 32);
    s.room(20.0, 14.0, 8.0);
    s.thick([10.0, 0.0], [10.0, 8.0], 0.5, None, &[], &[]);
    let f = find_by_ceiling(&s.scan(0.0));
    let two: Vec<&Wall> = f.walls.iter().filter(|w| w.thickness.is_some()).collect();
    assert_eq!(two.len(), 1, "one wall of two faces: {}", said(&f));
    assert!((two[0].thickness.unwrap() - 0.5).abs() < 1.5 * IN, "{} in", two[0].thickness.unwrap() * 12.0);
    assert!(two[0].seen_from.is_none(), "both faces seen: no one side");
    assert!((two[0].length() - 8.0).abs() < 1.2, "{}", two[0].length());
    assert_eq!(f.walls.len(), 5, "{}", said(&f));
}

#[test]
fn a_lone_face_is_a_wall_seen_from_one_side_with_no_thickness_made_up() {
    let mut s = Scene::new(150.0, 33);
    s.room(12.0, 14.0, 8.0);
    let f = find_by_ceiling(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{}", said(&f));
    for w in &f.walls {
        assert!(w.thickness.is_none(), "no thickness for a wall seen from one side: {w:?}");
        let from = w.seen_from.expect("the side it was seen from");
        let mid = [(w.from[0] + w.to[0]) / 2.0, (w.from[1] + w.to[1]) / 2.0];
        let inside = [mid[0] + from[0], mid[1] + from[1]];
        assert!(inside[0] > 0.0 && inside[0] < 12.0 && inside[1] > 0.0 && inside[1] < 14.0, "seen from the room's side: {w:?}");
    }
}

#[test]
fn a_rough_leaning_wall_is_made_straight_to_within_the_scans_own_roughness() {
    // Every wall leans a degree and is 2 cm rougher than the scan's own 1.5 cm: still four straight walls of the right length, none in doubt.
    let f = find_by_ceiling(&phone_room(300.0, 0.0, 0.0175, 0.066, 0.0));
    assert_eq!(f.walls.len(), 4, "{}", said(&f));
    assert!(f.walls.iter().all(|w| w.doubt.is_none()), "{}", said(&f));
    assert!(f.walls.iter().all(|w| (w.length() - 12.0).abs() < 1.5 || (w.length() - 14.0).abs() < 1.5), "{:?}", f.walls.iter().map(Wall::length).collect::<Vec<_>>());
    assert!(f.open_ends.is_empty(), "{:?}", f.open_ends);
}

#[test]
fn a_bowed_wall_is_one_wall_and_its_bow_is_said() {
    let f = find_by_ceiling(&phone_room(300.0, 1.5 * IN, 0.0175, 0.066, 0.0));
    assert_eq!(f.walls.len(), 4, "{}", said(&f));
    let south = f.walls.iter().find(|w| along(w, true) && w.from[1] < 5.0).expect("the south wall");
    let (depth, over) = south.bow.expect("its bow is said");
    assert!((depth - 1.5 * IN).abs() <= 1.0 * IN, "bowed {} in", depth * 12.0);
    assert!((over - 12.0).abs() <= 1.5, "over {over} ft");
    assert_eq!(f.walls.iter().filter(|w| w.bow.is_some()).count(), 1, "only the south wall bows");
}

#[test]
fn passes_of_one_wall_are_one_wall() {
    // The west wall scanned twice, 3/4 in apart: one wall, not two faces and not a wall 3/4 in thick.
    let mut s = Scene::new(150.0, 23);
    s.floor([0.0, 0.0, 12.0, 14.0]);
    s.ceiling([0.0, 0.0, 12.0, 14.0], 8.0, 8.0);
    s.face_rough([0.0, 0.0], [12.0, 0.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    s.face_rough([12.0, 0.0], [12.0, 14.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    s.face_rough([12.0, 14.0], [0.0, 14.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, 0.0, 0.0, 0.75 * IN, 150.0);
    let f = find_by_ceiling(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{}", said(&f));
    assert!(f.walls.iter().all(|w| w.thickness.is_none()), "no thickness is made up from two passes: {}", said(&f));
}

#[test]
fn a_doorway_and_a_window_are_read_in_the_face_of_their_wall() {
    // A 16 by 12 ft room, 8 ft high: a doorway 3 ft wide, 6 ft 8 in high, 4 ft to 7 ft along the south wall; a window 4 ft wide from 3 ft to 6 ft up, 10 ft to 14 ft.
    let mut s = Scene::new(250.0, 41);
    s.floor([0.0, 0.0, 16.0, 12.0]);
    s.ceiling([0.0, 0.0, 16.0, 12.0], 8.0, 8.0);
    s.face([0.0, 0.0], [16.0, 0.0], None, &[[4.0, 7.0, 0.0, 6.67], [10.0, 14.0, 3.0, 6.0]], &[]);
    s.face([16.0, 0.0], [16.0, 12.0], None, &[], &[]);
    s.face([16.0, 12.0], [0.0, 12.0], None, &[], &[]);
    s.face([0.0, 12.0], [0.0, 0.0], None, &[], &[]);
    let f = find_by_ceiling(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{}", said(&f));
    let south = f.walls.iter().find(|w| w.from[1].abs() < 1.0 && w.to[1].abs() < 1.0).expect("the south wall");
    assert_eq!(south.openings.len(), 2, "{:?}", south.openings);
    let door = south.openings.iter().find(|o| !o.window).expect("a doorway");
    assert!((door.to - door.from - 3.0).abs() < 0.4, "{} ft wide", door.to - door.from);
    assert!((door.head - 6.67).abs() < 0.4, "{} high", door.head);
    let win = south.openings.iter().find(|o| o.window).expect("a window");
    assert!((win.to - win.from - 4.0).abs() < 0.4 && (win.sill - 3.0).abs() < 0.4 && (win.head - 6.0).abs() < 0.4, "{win:?}");
}

#[test]
fn hidden_is_not_open() {
    // The north wall of a 12 by 14 ft room is not seen below 7 ft from 3 ft to 9 ft along it, and a cabinet 6 ft high stands before it:
    // not an opening, a stretch not seen.
    let mut s = Scene::new(200.0, 42);
    s.room_hiding(12.0, 14.0, 8.0, &[(2, [3.0, 9.0, 7.0])]);
    s.bx([3.0, 12.0, 9.0, 13.7], 6.0, &["north"]);
    let f = find_by_ceiling(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{}", said(&f));
    let north = f.walls.iter().find(|w| w.from[1] > 10.0 && w.to[1] > 10.0).expect("the north wall");
    assert!(north.openings.is_empty(), "not an opening: {:?}", north.openings);
    assert_eq!(north.hidden.len(), 1, "one stretch not seen: {north:?}");
    assert!((north.hidden[0].to - north.hidden[0].from - 6.0).abs() < 1.0, "{:?}", north.hidden);
}

#[test]
fn a_beam_hung_from_the_ceiling_is_in_doubt_and_not_a_wall_laid() {
    // A 10 ft beam hanging 1 ft below an 8 ft ceiling in the middle of the room: it shows in the band near the ceiling, its foot is not there, nothing stands
    // before it: in doubt ("reaches the ceiling and not the floor"), and the room's four walls are not.
    let mut s = Scene::new(150.0, 43);
    s.room(20.0, 14.0, 8.0);
    s.hung([5.0, 7.0], [15.0, 7.0], 7.0);
    let f = find_by_ceiling(&s.scan(0.0));
    let sure: Vec<&Wall> = f.walls.iter().filter(|w| w.doubt.is_none()).collect();
    assert_eq!(sure.len(), 4, "{}", said(&f));
    let beam = f.walls.iter().find(|w| w.doubt.is_some() && w.from[1] > 5.0 && w.from[1] < 9.0);
    assert!(beam.is_some(), "the beam is offered, in doubt: {}", said(&f));
    assert!(!f.open_ends.iter().any(|(i, _)| f.walls[*i].doubt.is_some()), "a wall in doubt is not counted in whether the walls close");
}

#[test]
fn a_bank_of_cabinets_to_the_ceiling_before_a_wall_is_in_doubt() {
    // The front of a bank 8 ft long, 7 ft 8 in high, 2 ft before the north wall of a 20 by 14 ft room, its ends free (its back is not seen): its face
    // reaches the band near the ceiling, its foot is seen: a wall or something tall standing at a wall: in doubt, not laid alone (call 179, A3).
    let mut s = Scene::new(150.0, 44);
    s.room(20.0, 14.0, 8.0);
    s.face([6.0, 11.8], [14.0, 11.8], Some(7.65), &[], &[]);
    let f = find_by_ceiling(&s.scan(0.0));
    let sure = f.walls.iter().filter(|w| w.doubt.is_none()).count();
    assert!(sure >= 4, "the room's walls are found: {}", said(&f));
    assert!(f.walls.iter().any(|w| w.doubt.is_some()), "the bank is in doubt: {}", said(&f));
}

#[test]
fn nothing_is_found_where_no_ceiling_shows_and_that_is_said() {
    let mut s = Scene::new(150.0, 45);
    s.floor([0.0, 0.0, 12.0, 14.0]);
    s.face([0.0, 0.0], [12.0, 0.0], Some(8.0), &[], &[]);
    s.face([12.0, 0.0], [12.0, 14.0], Some(8.0), &[], &[]);
    let f = find_by_ceiling(&s.scan(0.0));
    assert!(f.walls.is_empty(), "{}", said(&f));
    assert!(f.no_ceiling || f.not_likely.is_some() || !f.left_out.is_empty(), "something is said");
}

#[test]
fn a_thin_scan_is_read_by_a_deeper_band_and_says_so() {
    // WRITTEN AFTER the finder's band was made to deepen (found on the made-up houses at 30 to the square foot, where the top tenth of a wall shows under 3 points to a pixel).
    // A 20 by 14 ft room at 30 to the square foot (the floor reads 29): the pixel is 0.1 ft times the root of 80 over 29, 0.166 ft; 8 points to a pixel want a band of
    // 8 / (29 x 0.166) = 1.66 ft of the face, 21 in 100 of the 8 ft (the top tenth, 0.55 ft after the ceiling's 3 in, shows under 3 points to a pixel). Said; the four walls are found.
    let mut s = Scene::new(30.0, 52);
    s.room(20.0, 14.0, 8.0);
    let f = find_by_ceiling(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{}", said(&f));
    assert!(f.thin_note.as_deref().is_some_and(|n| n.contains("top 21 in 100 of the height")), "{:?}", f.thin_note);
    // A dense scan is not said to be read by any other band.
    let g = find_by_ceiling(&phone_room(300.0, 0.0, 0.0, 0.0, 0.0));
    assert!(g.thin_note.is_none(), "{:?}", g.thin_note);
}
