//! What call 188 (1 to 5) decided about the scan's walls on a phone's scan, each as a test on a made-up scan with the figure worked by hand.
//! A phone's scan has rough, bowed, leaning walls, uneven in how many points a wall shows, scanned in passes (`found_scenes::face_rough`),
//! and is large; the measures are those of three real scans, written beside the scene `phone` in `tools/made-up-scan.py`.

use crate::found::*;
use crate::found_scenes::Scene;
use crate::scan::Scan;

const IN: f64 = 1.0 / 12.0;

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

#[test]
fn a_bowed_wall_of_one_and_a_half_inches_over_12_ft_is_found_and_its_bow_said() {
    // The south wall bows 1 1/2 in over its 12 ft; every wall leans a degree (0.0175) and is 2 cm rougher than the scan's own 1.5 cm.
    let f = find(&phone_room(300.0, 1.5 * IN, 0.0175, 0.066, 0.0));
    assert_eq!(f.walls.len(), 4, "the four walls of the room are found: {:?}", f.left_out);
    assert!(f.walls.iter().all(|w| w.doubt.is_none()), "none in doubt: {:?}", f.walls.iter().map(|w| &w.doubt).collect::<Vec<_>>());
    let south = f.walls.iter().find(|w| w.from[1].abs() < 0.5 && w.to[1].abs() < 0.5).expect("a wall along the south");
    let (depth, over) = south.bow.expect("its bow is said");
    // 1 1/2 in over 12 ft: said within an inch of the depth and a foot of the length.
    assert!((depth - 1.5 * IN).abs() <= 1.0 * IN, "bowed {} in", depth * 12.0);
    assert!((over - 12.0).abs() <= 1.0, "over {over} ft");
    assert!(f.walls.iter().filter(|w| w.bow.is_some()).count() == 1, "only the south wall bows");
}

#[test]
fn a_wall_that_leans_a_degree_and_scatters_more_is_found_from_floor_to_ceiling() {
    let f = find(&phone_room(300.0, 0.0, 0.0175, 0.066, 0.0));
    assert_eq!(f.walls.len(), 4, "{:?}", f.left_out);
    assert!(f.walls.iter().all(|w| w.doubt.is_none() && w.bow.is_none()), "straight walls say no bow");
    assert!(f.walls.iter().all(|w| (w.length() - 12.0).abs() < 2.1 || (w.length() - 14.0).abs() < 2.1), "{:?}", f.walls.iter().map(Wall::length).collect::<Vec<_>>());
}

#[test]
fn a_clean_wall_says_no_bow_and_nothing_of_roughness() {
    let f = find(&phone_room(100.0, 0.0, 0.0, 0.0, 0.0));
    assert_eq!(f.walls.len(), 4);
    assert!(f.walls.iter().all(|w| w.bow.is_none() && w.joined == 0));
    assert!(f.scatter.is_none());
}

#[test]
fn two_passes_of_one_wall_three_quarters_of_an_inch_apart_are_one_wall() {
    let mut s = Scene::new(150.0, 23);
    s.floor([0.0, 0.0, 12.0, 14.0]);
    s.ceiling([0.0, 0.0, 12.0, 14.0], 8.0, 8.0);
    s.face_rough([0.0, 0.0], [12.0, 0.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    s.face_rough([12.0, 0.0], [12.0, 14.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    s.face_rough([12.0, 14.0], [0.0, 14.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    // the west wall twice, 3/4 in apart, each pass at half the density
    s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, 0.0, 0.0, 0.0, 75.0);
    s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, 0.0, 0.0, 0.75 * IN, 75.0);
    let f = find(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "the west wall once: {:?}", f.walls.iter().map(|w| (w.from, w.to)).collect::<Vec<_>>());
}

#[test]
fn two_faces_a_wall_scanned_in_passes_ten_inches_apart_are_joined_and_it_says_how_many() {
    let mut s = Scene::new(150.0, 29);
    s.floor([0.0, 0.0, 12.0, 14.0]);
    s.ceiling([0.0, 0.0, 12.0, 14.0], 8.0, 8.0);
    s.face_rough([0.0, 0.0], [12.0, 0.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    s.face_rough([12.0, 0.0], [12.0, 14.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    s.face_rough([12.0, 14.0], [0.0, 14.0], None, 0.0, 0.0, 0.0, 0.0, 150.0);
    // the west wall in two passes 10 in apart, each at half the density
    s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, 0.0, 0.0, 0.0, 75.0);
    s.face_rough([0.0, 14.0], [0.0, 0.0], None, 0.0, 0.0, 0.0, 10.0 * IN, 75.0);
    let f = find(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "the west wall is found once: {:?}", f.walls.iter().map(|w| (w.from, w.to)).collect::<Vec<_>>());
    let west = f.walls.iter().find(|w| w.from[0].abs() < 1.5 && w.to[0].abs() < 1.5).expect("a west wall");
    assert_eq!(west.joined, 1, "one face was joined into it, and that is said");
}

#[test]
fn a_scan_turned_on_its_side_is_said_first_and_nothing_is_laid_without_the_persons_word() {
    // The 30 by 16 by 8 ft house with its depth taken as up: the floor and the ceiling are across the scan's other direction.
    let mut s = Scene::new(60.0, 31);
    s.room(30.0, 16.0, 8.0);
    s.thick([18.0, 0.0], [18.0, 16.0], 0.375, None, &[], &[]);
    let upright = s.scan(0.0);
    let mut sideways = Scan { points: upright.points.iter().map(|p| [p[0], p[2], p[1]]).collect(), lost: Vec::new() };
    sideways.level_by_floor();
    let f = find(&sideways);
    let why = f.not_likely.expect("a house on its side is said");
    assert!(why.contains("which way is up") && why.contains("floor and its ceiling"), "{why}");
    assert!(why.contains("lie 8' 0\" across") || why.contains("lie 7' 11\" across") || why.contains("lie 7' 10\" across") || why.contains("lie 8' 1\" across"), "it says how far apart the two flat level things are (8 ft, to the bin): {why}");
    let ok = find(&upright);
    assert!(ok.not_likely.is_none(), "{:?}", ok.not_likely);
}

#[test]
fn a_large_scan_is_thinned_first_and_that_is_said() {
    // 400,000 points are over what is read whole.
    let mut s = Scene::new(300.0, 37);
    s.room(30.0, 16.0, 8.0);
    let scan = s.scan(0.0);
    assert!(scan.points.len() > crate::found_188::MOST_POINTS, "{}", scan.points.len());
    let f = find(&scan);
    let (before, kept) = f.thinned.expect("a scan of that many points is thinned, and said");
    assert_eq!(before, scan.points.len());
    assert!(kept <= crate::found_188::MOST_POINTS && kept * 3 > crate::found_188::MOST_POINTS / 2, "{kept}");
    assert_eq!(f.walls.len(), 4, "its four walls are found all the same");
    let small = find(&{
        let mut s = Scene::new(60.0, 41);
        s.room(30.0, 16.0, 8.0);
        s.scan(0.0)
    });
    assert!(small.thinned.is_none());
}

#[test]
fn a_face_6_ft_or_longer_that_reaches_the_ceiling_and_not_the_floor_is_offered_by_its_number_as_likely_a_wall() {
    // A room 14 by 12 ft, 8 ft high, a face 8 ft long hanging from 3 ft up to the ceiling, from the east wall.
    let mut s = Scene::new(100.0, 43);
    s.room(14.0, 12.0, 8.0);
    s.hung([6.0, 5.0], [14.0, 5.0], 3.0);
    let f = find(&s.scan(0.0));
    assert_eq!(f.walls.len(), 5, "{:?}", f.left_out);
    let offered: Vec<&Wall> = f.walls.iter().filter(|w| w.doubt.as_deref().is_some_and(|d| d.starts_with("likely a wall"))).collect();
    assert_eq!(offered.len(), 1, "one face is offered: {:?}", f.walls.iter().map(|w| &w.doubt).collect::<Vec<_>>());
    assert!((offered[0].length() - 8.0).abs() < 1.0, "{}", offered[0].length());
    // it is laid only on the person's word
    let (laid, _) = lay(&f, None);
    assert!(laid.len() <= 4, "not laid by itself: {}", laid.len());
}

#[test]
fn a_hanging_face_under_6_ft_is_said_and_not_offered() {
    let mut s = Scene::new(100.0, 47);
    s.room(14.0, 12.0, 8.0);
    s.hung([10.0, 5.0], [14.0, 5.0], 3.0);
    let f = find(&s.scan(0.0));
    assert!(f.walls.iter().all(|w| w.doubt.as_deref().is_none_or(|d| !d.starts_with("likely a wall"))), "a face of 4 ft is not offered");
}

#[test]
fn a_bank_of_cabinets_standing_short_of_the_ceiling_is_a_cabinet_and_is_not_offered() {
    // The wardrobe of 8 ft long and 7 ft 2 in high in a room 8 ft high: 10 in short, no wall.
    let mut s = Scene::new(100.0, 53);
    s.room(14.0, 12.0, 8.0);
    s.bx([3.0, 5.0, 11.0, 7.0], 7.1667, &[]);
    let f = find(&s.scan(0.0));
    assert_eq!(f.walls.len(), 4, "{:?}", f.walls.iter().map(|w| (&w.doubt, w.length())).collect::<Vec<_>>());
    assert!(f.walls.iter().all(|w| w.doubt.is_none()));
}
