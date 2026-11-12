//! What each thing is finished with, read from what is priced on it (DECISIONS.md call
//! 58). Every job and every line here is made up.

use plumbline_world::editing::{self as ed, Entered, Top};
use plumbline_world::file::{load, save};
use plumbline_world::finish::{self, Cladding, Leaf, Outside, Roofing, Sash};
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

/// Four walls round 24 ft by 16 ft drawn counter-clockwise from the south-west corner
/// (south, east, north, west), and a partition across the middle.
fn made() -> (Job, Vec<String>, String) {
    let mut job = Job::new("Finishes (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let walls = ed::walls(&mut job, &level, &wall_type, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, Top::Height("9 ft"), KNOWN).unwrap();
    let partition = ed::walls(&mut job, &level, &wall_type, &[[12.0, 0.0], [12.0, 16.0]], false, Top::Height("9 ft"), KNOWN).unwrap();
    (job, walls, partition[0].clone())
}

fn line(job: &mut Job, path: &str, text: &str) {
    job.add_line(path, text, "1", Some(Class::Material), false).unwrap();
}

#[test]
fn every_word_of_the_trade_reads_as_its_finish() {
    for (text, want) in [
        ("Brick veneer, red", Cladding::Brick),
        ("Stone veneer", Cladding::Stone),
        ("Limestone ashlar", Cladding::Stone),
        ("Stucco, three coat", Cladding::Stucco),
        ("Board and batten siding", Cladding::BoardAndBatten),
        ("Cedar shingle siding", Cladding::Shingles),
        ("Fiber cement lap siding, white", Cladding::LapSiding),
        ("Clapboards, primed", Cladding::LapSiding),
    ] {
        assert_eq!(finish::cladding_in(text), Some(want), "{text}");
    }
    for (text, want) in [
        ("Standing seam roofing, charcoal", Roofing::StandingSeam),
        ("Copper panels", Roofing::StandingSeam),
        ("Slate roofing", Roofing::Slate),
        ("Clay barrel tile", Roofing::Tile),
        ("Cedar shakes", Roofing::Shakes),
        ("Architectural shingles", Roofing::AsphaltShingles),
        ("EPDM membrane", Roofing::Membrane),
    ] {
        assert_eq!(finish::roofing_in(text), Some(want), "{text}");
    }
    assert_eq!(finish::sash_in("Window unit, clad wood, double-hung"), Some(Sash::DoubleHung));
    assert_eq!(finish::sash_in("Casement, aluminum clad"), Some(Sash::Casement));
    assert_eq!(finish::sash_in("Picture window"), Some(Sash::Fixed));
    assert_eq!(finish::leaf_in("Entry door, six panel, prehung"), Some(Leaf::Panel));
    assert_eq!(finish::leaf_in("French door"), Some(Leaf::Glazed));
    assert_eq!(finish::leaf_in("Overhead door, 16 ft"), Some(Leaf::Overhead));
    assert_eq!(finish::divided_in("double-hung, 6 over 6"), Some(Some((3, 2))));
    assert_eq!(finish::divided_in("divided lites"), Some(None));
    assert_eq!(finish::divided_in("double-hung"), None);
    // What names no finish reads as none: a wall is not dressed in its framing or its paint.
    for text in ["Wall framing, 2x6 at 16 in", "Batt insulation", "Paint, two coats", "Gypsum board, hung and finished", "Sheathing and weather barrier"] {
        assert_eq!(finish::cladding_in(text), None, "{text}");
    }
    assert_eq!(finish::roofing_in("Roof sheathing"), None);
}

#[test]
fn a_color_is_a_whole_word() {
    assert_eq!(finish::tone_in("Fiber cement lap siding, white").map(|t| t.word), Some("white"));
    assert_eq!(finish::tone_in("Brick veneer, red").map(|t| t.word), Some("red"));
    assert_eq!(finish::tone_in("Cedar lap siding").map(|t| t.word), Some("cedar"));
    // "tan" is in "standing" and "red" is in "prepared"; neither is a color there.
    assert_eq!(finish::tone_in("Standing seam roofing"), None);
    assert_eq!(finish::tone_in("Prepared surface"), None);
    assert_eq!(finish::tone_in("Standing seam roofing, light gray").map(|t| t.word), Some("light gray"));
}

#[test]
fn the_outside_face_is_the_one_outside_the_outline_and_a_partition_has_none() {
    let (mut job, walls, partition) = made();
    let d = finish::dress(&mut job);
    // Drawn counter-clockwise, the inside is to the left of every wall: outside is its right.
    let faces: Vec<&str> = walls.iter().map(|w| finish::compass(d.walls[w].out.unwrap())).collect();
    assert_eq!(faces, ["south", "east", "north", "west"]);
    assert!(walls.iter().all(|w| d.walls[w].outside == Outside::Right));
    assert_eq!(d.walls[&partition].outside, Outside::Neither);
    assert_eq!(d.walls[&partition].out, None);
    // Each outside wall runs half a wall past each end, 3 in, to close its corners; the
    // partition ends on the middle of two walls and runs past neither.
    for w in &walls {
        assert!(d.walls[w].past.iter().all(|p| (p - 0.25).abs() < 1e-9), "{:?}", d.walls[w].past);
    }
    assert_eq!(d.walls[&partition].past, [0.0, 0.0]);
    // Nothing is priced, so everything is bare.
    assert!(d.walls.values().all(|w| w.cladding.is_none() && w.wainscot.is_none()));
}

#[test]
fn walls_that_enclose_nothing_have_no_known_outside() {
    let mut job = Job::new("One wall (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "9 ft", KNOWN).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let walls = ed::walls(&mut job, &level, &wall_type, &[[0.0, 0.0], [10.0, 0.0]], false, Top::Height("9 ft"), KNOWN).unwrap();
    line(&mut job, &walls[0], "Brick veneer");
    let d = finish::dress(&mut job);
    assert_eq!(d.walls[&walls[0]].outside, Outside::NotKnown);
    assert_eq!(d.walls[&walls[0]].cladding.as_ref().map(|c| c.what), Some(Cladding::Brick));
    let said = finish::report(&mut job).join("\n");
    assert!(said.contains("drawn on both faces"), "{said}");
}

#[test]
fn a_wall_is_dressed_in_what_its_live_lines_name_and_says_which_line() {
    let (mut job, walls, partition) = made();
    line(&mut job, &walls[0], "Wall framing, 2x6 at 16 in");
    line(&mut job, &walls[0], "Brick veneer, red");
    line(&mut job, &walls[2], "Stone veneer wainscot, 3 ft 6 in");
    line(&mut job, &walls[2], "Cedar shingle siding");
    line(&mut job, &walls[3], "Stone wainscot");
    line(&mut job, &partition, "Brick veneer");
    let d = finish::dress(&mut job);
    let south = d.walls[&walls[0]].cladding.as_ref().unwrap();
    assert_eq!((south.what, south.tone.map(|t| t.word), south.by.as_str()), (Cladding::Brick, Some("red"), "the line \"Brick veneer, red\""));
    assert!(d.walls[&walls[1]].cladding.is_none());
    let north = &d.walls[&walls[2]];
    assert_eq!(north.cladding.as_ref().map(|c| c.what), Some(Cladding::Shingles));
    let (low, to, written) = north.wainscot.as_ref().unwrap();
    assert_eq!((low.what, *to, *written), (Cladding::Stone, 3.5, true));
    // A second cladding priced on one wall: the first is drawn and the second is said.
    line(&mut job, &walls[0], "Stucco, three coat");
    let twice = finish::dress(&mut job);
    let south = twice.walls[&walls[0]].cladding.as_ref().unwrap();
    assert_eq!(south.what, Cladding::Brick);
    assert_eq!(south.also, [("stucco".to_string(), "the line \"Stucco, three coat\"".to_string())]);
    assert!(finish::report(&mut job).join("\n").contains("also priced on it and not drawn: stucco"));
    // A wainscot whose height no line gives is drawn 3 ft and says the height is not written.
    let (_, to, written) = d.walls[&walls[3]].wainscot.as_ref().unwrap();
    assert_eq!((*to, *written), (3.0, false));
    let said = finish::report(&mut job).join("\n");
    assert!(said.contains("a height no line gives"), "{said}");
    // The partition has brick priced on it and no outside face: priced, not drawn, and said.
    assert!(said.contains("so no cladding is drawn on it; brick veneer is priced on it"), "{said}");
    assert!(said.contains("bare, on its east face: nothing priced on it names a cladding"), "{said}");

    // A line switched off is out of the estimate, so it is off the house.
    // (Only a labor line can be switched off, so the wall here is named by its labor line.)
    job.add_line(&walls[1], "Install board and batten siding", "1", None, false).unwrap();
    job.set_on_line(&walls[1], "order", Some("Install board and batten siding"), Some("0.5 hr/EA"), Status::Known, "made-up rate").unwrap();
    job.set_on_line(&walls[1], "unit_cost", Some("Install board and batten siding"), Some("$70/hr"), Status::Known, "made-up rate").unwrap();
    job.set_class(&walls[1], Some("Install board and batten siding"), Class::Labor).unwrap();
    assert_eq!(finish::dress(&mut job).walls[&walls[1]].cladding.as_ref().map(|c| c.what), Some(Cladding::BoardAndBatten));
    job.set_off(&walls[1], Some("Install board and batten siding"), true).unwrap();
    assert!(finish::dress(&mut job).walls[&walls[1]].cladding.is_none());
}

#[test]
fn the_reading_is_the_same_after_a_save_and_a_load() {
    let mut job = sample::house_priced();
    job.derive();
    let before = finish::report(&mut job);
    let text = save(&mut job);
    let (mut back, _) = load(&text).unwrap();
    assert_eq!(finish::report(&mut back), before);
    // The priced sample house: cedar lap siding on all eight outside walls, a stone
    // wainscot on the four of the first floor, standing seam on four planes, a panel
    // door at the front, and every window double-hung with divided lites.
    let d = finish::dress(&mut back);
    assert_eq!(d.walls.values().filter(|w| w.cladding.as_ref().is_some_and(|c| c.what == Cladding::LapSiding)).count(), 8);
    assert_eq!(d.walls.values().filter(|w| w.wainscot.is_some()).count(), 4);
    assert_eq!(d.walls.values().filter(|w| w.outside == Outside::Neither).count(), 8);
    assert_eq!(d.roofs.values().filter(|r| r.what == Roofing::StandingSeam).count(), 4);
    assert_eq!(d.gutters.len(), 4);
    assert!(d.openings.values().filter(|o| o.sash.is_some()).all(|o| o.sash.as_ref().unwrap().what == Sash::DoubleHung && o.divided.is_some()));
    assert_eq!(d.openings.values().filter(|o| o.sash.is_some()).count(), 17);
    // Unpriced, the same house is bare all over.
    let mut bare = sample::house();
    let d = finish::dress(&mut bare);
    assert!(d.walls.values().all(|w| w.cladding.is_none()) && d.roofs.is_empty() && d.gutters.is_empty());
}
