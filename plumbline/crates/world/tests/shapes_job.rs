//! Call 198: a shape of the user's own, brought in as a table, named by an item, carried by the saved job, written
//! out and brought back. Figures worked by hand (the "wall-hung closet" is made up): 16 in wide, 26 in deep, 30 in
//! high; its tank 16 in across, from the back to 6.5 in out, up from 12 in to 30; its bowl, round, 14.4 in across,
//! from 7.8 in out to 26, up from 4 in to 12.

use plumbline_world::building::shape::{self, csv_rows};
use plumbline_world::file;
use plumbline_world::tables;
use plumbline_world::*;

const CLOSET: &str = "Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to
wall-hung closet,porcelain,box,-width / 2,width / 2,0,0.25 * depth,0.4 * height,height
wall-hung closet,porcelain,round,-0.45 * width,0.45 * width,0.3 * depth,depth,4 in,0.4 * height
";

const ITEMS: &str = "Item,Kind,Drawn as,Width,Depth,Height,How sure,Where from
Wall-hung closet,fixture,wall-hung closet,16 in,26 in,30 in,Assumed,made-up
Vanity,furnishing,,36 in,21 in,34.5 in,Assumed,made-up
";

fn job() -> Job {
    Job::new("Shapes (not a real job)", "test", "2026-11-24T00:00:00")
}

fn in_(job: &mut Job, text: &str) -> tables::Brought {
    tables::shapes_in(job, &csv_rows(text), None).unwrap()
}

#[test]
fn a_shape_of_the_users_own_is_worked_from_its_rows_at_the_items_sizes() {
    let mut j = job();
    let said = in_(&mut j, CLOSET);
    assert_eq!(said.done, vec!["wall-hung closet: added: 2 blocks".to_string()]);
    let size = [16.0 / 12.0, 26.0 / 12.0, 30.0 / 12.0];
    let blocks = shape::find(&j.shapes, "Wall-Hung Closet").unwrap().worked(size).unwrap();
    assert_eq!(blocks.len(), 2);
    let inch = |x: f64| x * 12.0;
    // The thing's own frame has y from the front; "back 0 to 6.5 in" is y from 13 - 6.5 to 13.
    let (b0, b1) = (&blocks[0], &blocks[1]);
    let near = |a: f64, b: f64| assert!((a - b).abs() < 1.0e-9, "{a} against {b}");
    near(inch(b0.lo[0]), -8.0);
    near(inch(b0.hi[0]), 8.0);
    near(inch(b0.lo[1]), 6.5);
    near(inch(b0.hi[1]), 13.0);
    near(inch(b0.lo[2]), 12.0);
    near(inch(b0.hi[2]), 30.0);
    assert!(!b0.round);
    near(inch(b1.lo[0]), -7.2);
    near(inch(b1.hi[0]), 7.2);
    near(inch(b1.lo[1]), -13.0);
    near(inch(b1.hi[1]), 13.0 - 7.8);
    near(inch(b1.lo[2]), 4.0);
    near(inch(b1.hi[2]), 12.0);
    assert!(b1.round);
}

#[test]
fn an_item_that_names_a_shape_that_is_not_there_is_refused_naming_it() {
    let mut j = job();
    let said = tables::items_in(&mut j, &csv_rows(ITEMS), None).unwrap();
    assert_eq!(said.not_used.len(), 1, "{said:?}");
    assert!(said.not_used[0].1.contains("\"wall-hung closet\" is not something a thing can be drawn as"), "{:?}", said.not_used);
    // The vanity, which names none, is in.
    assert!(said.done.iter().any(|d| d.starts_with("Vanity")), "{said:?}");
    // Once the shape is in, the item is.
    in_(&mut j, CLOSET);
    let said = tables::items_in(&mut j, &csv_rows(ITEMS), None).unwrap();
    assert!(said.not_used.is_empty(), "{said:?}");
    assert!(said.done.iter().any(|d| d.starts_with("Wall-hung closet: added")), "{said:?}");
}

#[test]
fn the_saved_job_carries_the_shape_and_reads_back_the_same() {
    let mut j = job();
    in_(&mut j, CLOSET);
    tables::items_in(&mut j, &csv_rows(ITEMS), None).unwrap();
    let text = file::save(&mut j);
    assert!(text.contains("\"shapes\""), "{text}");
    let (mut back, _) = file::load(&text).unwrap();
    assert_eq!(back.shapes, j.shapes);
    assert_eq!(file::save(&mut back), text, "saved, read and saved again: the same bytes");
    // The file with its shapes taken away names a shape nobody has: refused, naming it.
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut bare = value.clone();
    bare.as_object_mut().unwrap().remove("shapes");
    let why = file::load(&serde_json::to_string(&bare).unwrap()).err().unwrap();
    assert!(why.contains("wall-hung closet") && why.contains("drawn as"), "{why}");
}

#[test]
fn a_job_that_names_no_shape_of_its_own_saves_as_it_always_did() {
    let mut j = job();
    tables::items_in(&mut j, &csv_rows(&ITEMS.replace("wall-hung closet", "box")), None).unwrap();
    let text = file::save(&mut j);
    assert!(!text.contains("\"shapes\":"), "no shapes key in a job that carries none");
    let (mut back, _) = file::load(&text).unwrap();
    assert!(back.shapes.is_empty());
    assert_eq!(file::save(&mut back), text);
    // The starter shapes are the program's: bringing them in again carries nothing.
    let said = tables::shapes_in(&mut j, &csv_rows(plumbline_world::tables::STARTER_SHAPES), None).unwrap();
    assert!(j.shapes.is_empty(), "{said:?}");
    // call 233: the starter chest grew
    assert_eq!(said.same, 27);
}

#[test]
fn the_two_tables_come_back_the_same_into_a_second_job() {
    let mut a = job();
    in_(&mut a, CLOSET);
    tables::items_in(&mut a, &csv_rows(ITEMS), None).unwrap();
    let (items, shapes) = (tables::items_out(&mut a), tables::shapes_out(&mut a));
    assert_eq!(shapes.len(), 3, "the headings and the closet's two blocks: {shapes:?}");
    let mut b = job();
    tables::shapes_in(&mut b, &shapes, None).unwrap();
    tables::items_in(&mut b, &items, None).unwrap();
    assert_eq!(tables::shapes_out(&mut b), shapes);
    assert_eq!(tables::items_out(&mut b), items);
    // An item that names a starter shape writes its rows too, and they come back as the starter's, carrying nothing.
    let mut c = job();
    tables::items_in(&mut c, &csv_rows("Item,Kind,Drawn as,Width,Depth,Height,How sure,Where from\nVanity,furnishing,base cabinet,36 in,21 in,34.5 in,Assumed,made-up\n"), None).unwrap();
    let vanity = tables::shapes_out(&mut c);
    assert_eq!(vanity.len(), 4, "{vanity:?}");
    let mut d = job();
    let said = tables::shapes_in(&mut d, &vanity, None).unwrap();
    assert!(d.shapes.is_empty() && said.same == 1, "{said:?}");
}

#[test]
fn a_shape_that_cannot_be_worked_is_not_used_and_says_which_block_and_which_figure() {
    let mut j = job();
    let bad = "Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to
ok shape,timber,box,-width / 2,width / 2,0,depth,0,height
area shape,timber,box,-width / 2,width / 2,0,depth,0,height
area shape,timber,box,-width / 2,width / 2,0,width * depth,0,height
stuff shape,timber,box,-width / 2,width / 2,0,length,0,height
paint shape,paint,box,-width / 2,width / 2,0,depth,0,height
sum shape,timber,box,-width / 2,width / 2,0,depth + 3,0,height
";
    let said = in_(&mut j, bad);
    assert_eq!(j.shapes.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), vec!["ok shape"]);
    let all: String = said.not_used.iter().map(|(_, w)| w.clone()).collect::<Vec<_>>().join("\n");
    assert!(all.contains("block 2, Back to") && all.contains("comes out as an area"), "{all}");
    assert!(all.contains("\"length\"") && all.contains("width, depth and height"), "{all}");
    assert!(all.contains("\"paint\", which is none of timber"), "{all}");
    assert!(all.contains("adds what cannot be added"), "{all}");
}

#[test]
fn the_rows_say_a_block_in_either_order_and_a_name_is_not_a_figure() {
    let mut j = job();
    in_(&mut j, "Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to\nflip,steel,box,width / 2,-width / 2,depth,0,height,0\n");
    let b = shape::find(&j.shapes, "flip").unwrap().worked([2.0, 1.0, 3.0]).unwrap();
    assert_eq!((b[0].lo, b[0].hi), ([-1.0, -0.5, 0.0], [1.0, 0.5, 3.0]));
}

const HEAD: &str = "Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\n";

#[test]
fn a_bare_number_is_no_length_but_zero_is_the_one_that_may_stand_alone() {
    let mut j = job();
    let said = in_(&mut j, &format!("{HEAD}ok,timber,box,-width / 2,width / 2,0,depth,0,height,\nbare,timber,box,-width / 2,width / 2,0,depth,0,height,\nbare,timber,box,-width / 2,width / 2,0,depth,4,height,\n"));
    assert_eq!(j.shapes.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), vec!["ok"]);
    // The row with the 4 is named (the third of the table), and says why.
    let row3: Vec<&String> = said.not_used.iter().filter(|(n, _)| *n == 4).map(|(_, w)| w).collect();
    assert!(row3.len() == 1 && row3[0].contains("Up from") && row3[0].contains("comes out as a plain number") && row3[0].contains("only 0 stands without one"), "{:?}", said.not_used);
    assert!(said.not_used.iter().any(|(n, w)| *n == 3 && w.contains("left out") && w.contains("row 4")), "{:?}", said.not_used);
}

#[test]
fn a_block_that_reaches_outside_the_box_is_refused_with_its_row_and_the_size() {
    let mut j = job();
    let said = in_(&mut j, &format!("{HEAD}wide,timber,box,-width / 2,width / 2,0,depth,0,height,\nwide,timber,box,-width / 2,width / 2 + 1 in,0,depth,0,height,\n"));
    assert!(j.shapes.is_empty());
    let (n, why) = said.not_used.iter().find(|(n, _)| *n == 3).unwrap();
    assert!(why.contains("block 2") && why.contains("on an item 4 13/16 in wide, 3 5/8 in deep and 6 in high it would stand 1 in past the right side") && why.contains("a shape is held inside its item at every size") && !why.contains("small") && !why.contains(" ft"), "row {n}: {why}");
    // A block whose condition does not hold at a size is not tested there: this one is outside only when it is there.
    let said = in_(&mut j, &format!("{HEAD}tall,timber,box,-width / 2,width / 2,0,depth,0,height,\ntall,timber,box,-width / 2,width,0,depth,0,height,width > 100 in\n"));
    assert!(j.shapes.iter().any(|s| s.name == "tall"), "{said:?}");
}

#[test]
fn a_block_may_be_there_only_when_a_comparison_holds_and_the_comparison_is_read_strictly() {
    let mut j = job();
    in_(&mut j, &format!("{HEAD}gate,steel,box,-width / 2,width / 2,0,depth,0,height,\ngate,steel,box,-width / 4,width / 4,0,depth,0,height,width >= 30 in\ngate,steel,box,-width / 8,width / 8,0,depth,0,height,width < 30 in\n"));
    let g = shape::find(&j.shapes, "gate").unwrap();
    let n = |w: f64| g.worked([w, 1.0, 1.0]).unwrap().len();
    assert_eq!((n(2.0), n(2.5), n(3.0)), (2, 2, 2), "30 in is there, below it the other");
    assert_eq!(g.worked([2.4, 1.0, 1.0]).unwrap()[1].hi[0], 2.4 / 8.0);
    assert_eq!(g.worked([2.5, 1.0, 1.0]).unwrap()[1].hi[0], 2.5 / 4.0);
    // Refused, naming the row: no comparison, two, an area against a length, a bare number against a length.
    for (bad, said) in [("width", "no comparison"), ("width > 1 in > 2 in", "more than one comparison"), ("width > depth * depth", "an area"), ("width > 4", "a plain number"), ("length > 2 in", "width, depth and height")] {
        let mut k = job();
        let r = in_(&mut k, &format!("{HEAD}x,steel,box,-width / 2,width / 2,0,depth,0,height,{}\n", format_args!("\"{bad}\"")));
        assert!(k.shapes.is_empty() && r.not_used.len() == 1 && r.not_used[0].0 == 2 && r.not_used[0].1.contains(said), "{bad}: {r:?}");
    }
    // Zero stands against a length.
    let mut k = job();
    in_(&mut k, &format!("{HEAD}x,steel,box,-width / 2,width / 2,0,depth,0,height,width > 0\n"));
    assert_eq!(k.shapes.len(), 1);
    // The condition is carried by the saved job and written out in its own column.
    let rows = tables::shapes_out(&mut { let mut a = job(); in_(&mut a, &format!("{HEAD}gate,steel,box,-width / 2,width / 2,0,depth,0,height,width >= 30 in\n")); tables::items_in(&mut a, &csv_rows("Item,Kind,Drawn as,Width,Depth,Height,How sure,Where from\nG,fixture,gate,36 in,21 in,34 in,Assumed,made-up\n"), None).unwrap(); a });
    assert_eq!(rows[0][9], "There when");
    assert_eq!(rows[1][9], "width >= 30 in");
}
