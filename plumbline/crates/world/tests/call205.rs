//! Call 205 (points 1 to 5): a shape changed by hand, the model's changes. The seven steps of the
//! demonstration in STATUS.md ("A shape changed by hand"), as tests on the model, then what each change
//! refuses. Made-up sizes: the starter chest's "Vanity, 36 in" is 36 in wide, 21 in deep, 34 1/2 in high,
//! drawn as base cabinet, set down twice; "Base cabinet, 36 in" is beside it, 36 x 24 x 34 1/2.

use plumbline_world::building::forms::Made;
use plumbline_world::building::shape::{self, csv_rows};
use plumbline_world::editing::{self as ed, BlockSet, History};
use plumbline_world::value::Status;
use plumbline_world::{file, tables, Job};

const HEAD: &str = "Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\n";

fn job() -> (Job, String, String) {
    let mut j = Job::new("Shapes (not a real job)", "test", "2026-11-24T00:00:00");
    tables::items_in(&mut j, &csv_rows(tables::STARTER_ITEMS), None).unwrap();
    let how = ed::Entered { status: Status::Assumed, source: "made-up plan" };
    let level = ed::storey(&mut j, "Ground", "0 ft", "9 ft", how).unwrap();
    let find = |j: &mut Job, name: &str| {
        let mut q = j.world.query::<(&plumbline_world::Node, &plumbline_world::Item)>();
        q.iter(&j.world).find(|(n, _)| n.name == name).map(|(n, _)| n.path.clone()).unwrap()
    };
    let vanity = find(&mut j, "Vanity, 36 in");
    let cabinet = find(&mut j, "Base cabinet, 36 in");
    for at in [[5.0, 5.0], [9.0, 5.0]] {
        ed::place_item(&mut j, &level, &vanity, at, "0 ft", ed::Entered { status: Status::Assumed, source: "set down by hand in the model" }).unwrap();
    }
    j.derive();
    (j, vanity, cabinet)
}

/// A shape's blocks at a size, in inches: [across from, across to, back from, back to, up from, up to].
fn blocks(j: &Job, name: &str, size_in: [f64; 3]) -> Vec<[f64; 6]> {
    let size = size_in.map(|x| x / 12.0);
    shape::find(&j.shapes, name).unwrap().worked(size).unwrap().iter().map(|b| [b.lo[0] * 12.0, b.hi[0] * 12.0, (size[1] / 2.0 - b.hi[1]) * 12.0, (size[1] / 2.0 - b.lo[1]) * 12.0, b.lo[2] * 12.0, b.hi[2] * 12.0]).collect()
}

fn near(a: [f64; 6], b: [f64; 6]) {
    for i in 0..6 {
        assert!((a[i] - b[i]).abs() < 6.0e-4, "{a:?} against {b:?}");
    }
}

const VANITY: [f64; 3] = [36.0, 21.0, 34.5];
const KEPT: &str = "vanity with a panel";

fn kept() -> (Job, String, String) {
    let (mut j, vanity, cabinet) = job();
    ed::keep_shape_as(&mut j, &vanity, KEPT).unwrap();
    (j, vanity, cabinet)
}

fn form_of(j: &mut Job, item: &str) -> Option<String> {
    let e = j.find(item).unwrap();
    j.world.get::<plumbline_world::Item>(e).unwrap().form.clone()
}

// ---- the demonstration's seven steps ----

#[test]
fn step_1_a_shape_kept_as_a_new_one_is_the_starters_word_for_word_and_the_item_is_drawn_as_it() {
    let (mut j, vanity, cabinet) = job();
    let said = ed::keep_shape_as(&mut j, &vanity, KEPT).unwrap();
    assert_eq!(j.shapes.len(), 1);
    assert_eq!(j.shapes[0].name, KEPT);
    assert_eq!(j.shapes[0].blocks, shape::starter_named("base cabinet").unwrap().blocks, "word for word");
    assert_eq!(j.shapes[0].blocks.len(), 3);
    assert_eq!(form_of(&mut j, &vanity).as_deref(), Some(KEPT));
    assert_eq!(form_of(&mut j, &cabinet).as_deref(), Some("base cabinet"));
    assert_eq!((said.items.len(), said.set_down.len(), said.blocks), (1, 2, 3));
    let text = said.said();
    assert!(text.contains("1 item is drawn as it") && text.contains("2 things set down from it follow"), "{text}");
    // The plinth, case and top at the vanity's sizes (STATUS.md step 1 of call 198): 3.937, 32.925, 34.5 in.
    let b = blocks(&j, KEPT, VANITY);
    near(b[0], [-18.0, 18.0, 0.0, 18.638, 0.0, 3.937]);
    near(b[1], [-18.0, 18.0, 0.0, 20.213, 3.937, 32.925]);
    near(b[2], [-18.0, 18.0, 0.0, 21.0, 32.925, 34.5]);
}

fn with_fourth() -> (Job, String) {
    let (mut j, vanity, _) = kept();
    let said = ed::shape_block_add(&mut j, KEPT, Some(Made::Timber)).unwrap();
    assert_eq!(said.blocks, 4);
    for (i, text) in ["-width / 2 + 2 in", "width / 2 - 2 in", "depth - 1 in", "depth", "4 in", "height - 6 in"].into_iter().enumerate() {
        ed::shape_block_set(&mut j, KEPT, 4, BlockSet::Figure(i, text.to_string())).unwrap();
    }
    (j, vanity)
}

#[test]
fn step_2_a_block_added_is_the_whole_box_until_its_figures_are_typed() {
    let (mut j, _, _) = kept();
    let said = ed::shape_block_add(&mut j, KEPT, None).unwrap();
    assert_eq!((said.blocks, said.block), (4, Some(4)));
    // Added: the whole size of the item, of the shape's first material (timber).
    near(blocks(&j, KEPT, VANITY)[3], [-18.0, 18.0, 0.0, 21.0, 0.0, 34.5]);
    assert_eq!(j.shapes[0].blocks[3].made, Made::Timber);
    assert_eq!((said.items.len(), said.set_down.len()), (1, 2));
    let (j, _) = with_fourth();
    assert_eq!(j.shapes[0].blocks.len(), 4);
    near(blocks(&j, KEPT, VANITY)[3], [-16.0, 16.0, 20.0, 21.0, 4.0, 28.5]);
}

#[test]
fn step_3_a_figure_that_leaves_the_box_is_refused_naming_the_block_the_size_and_how_far_and_the_shape_is_as_it_was() {
    let (mut j, _) = with_fourth();
    let before = file::save(&mut j);
    let why = ed::shape_block_set(&mut j, KEPT, 4, BlockSet::Figure(5, "height + 1 in".into())).unwrap_err();
    assert!(why.contains("block 4") && why.contains("on Vanity, 36 in (36 in wide, 21 in deep and 34 1/2 in high) it would stand 1 in above the top") && !why.contains("small") && !why.contains(" ft"), "{why}");
    assert!(why.contains("1 in above"), "it says how far and which way: {why}");
    assert_eq!(file::save(&mut j), before, "the shape and the job are as they were");
}

#[test]
fn step_4_a_starter_shape_is_not_changed() {
    let (mut j, _, _) = kept();
    let before = file::save(&mut j);
    for why in [
        ed::shape_block_add(&mut j, "base cabinet", None).unwrap_err(),
        ed::shape_block_set(&mut j, "Base Cabinet", 1, BlockSet::Round(true)).unwrap_err(),
        ed::shape_block_take(&mut j, "base cabinet", 1).unwrap_err(),
    ] {
        assert!(why.contains("starter shape") && why.contains("not changed") && why.contains("keep it as a new shape first"), "{why}");
    }
    assert_eq!(file::save(&mut j), before);
}

#[test]
fn step_5_a_table_with_a_shape_called_as_a_starter_is_refused_naming_it() {
    let (mut j, _, _) = job();
    let before = file::save(&mut j);
    let said = tables::shapes_in(&mut j, &csv_rows(&format!("{HEAD}base cabinet,timber,box,-width / 2,width / 2,0,depth,0,height,\nfine one,timber,box,-width / 2,width / 2,0,depth,0,height,\n")), None).unwrap();
    assert!(j.shapes.iter().all(|s| s.name != "base cabinet"), "{:?}", j.shapes);
    assert!(j.shapes.iter().any(|s| s.name == "fine one"), "the others still come in");
    let why = said.not_used.iter().map(|(_, w)| w.as_str()).collect::<Vec<_>>().join("\n");
    assert!(why.contains("base cabinet") && why.contains("starter shape") && why.contains("name"), "{why}");
    // Its own rows, word for word, are the same thing: carried by nobody, counted as the same (call 198).
    let mut k = Job::new("again", "test", "t");
    let said = tables::shapes_in(&mut k, &csv_rows(tables::STARTER_SHAPES), None).unwrap();
    // call 233: the starter chest grew
    assert!(k.shapes.is_empty() && said.same == 27 && said.not_used.is_empty(), "{said:?}");
    let _ = before;
}

#[test]
fn step_6_the_fourth_block_taken_away_leaves_the_three() {
    let (mut j, _) = with_fourth();
    let said = ed::shape_block_take(&mut j, KEPT, 4).unwrap();
    assert_eq!(said.blocks, 3);
    let b = blocks(&j, KEPT, VANITY);
    assert_eq!(b.len(), 3);
    near(b[0], [-18.0, 18.0, 0.0, 18.638, 0.0, 3.937]);
    near(b[2], [-18.0, 18.0, 0.0, 21.0, 32.925, 34.5]);
    assert!(said.said().contains("1 item is drawn as it"), "{}", said.said());
}

#[test]
fn step_7_saved_opened_and_saved_again_the_same_bytes_and_written_out_as_a_table_with_four_rows() {
    let (mut j, _) = with_fourth();
    let text = file::save(&mut j);
    let (mut back, _) = file::load(&text).unwrap();
    assert_eq!(file::save(&mut back), text);
    let rows = tables::shapes_out(&mut back);
    assert_eq!(rows.iter().filter(|r| r[0] == KEPT).count(), 4, "{rows:?}");
}

// ---- what each change refuses ----

#[test]
fn a_name_the_job_or_the_starters_have_is_refused_and_so_is_an_empty_one() {
    let (mut j, vanity, cabinet) = kept();
    let before = file::save(&mut j);
    // The job's own shape's name, however it is capitalised; a starter's; none; the plain box.
    let again = ed::keep_shape_as(&mut j, &cabinet, "Vanity With A Panel").unwrap_err();
    assert!(again.contains("vanity with a panel") && again.contains("already"), "{again}");
    for taken in ["Base Cabinet", "sink", "box"] {
        let why = ed::keep_shape_as(&mut j, &cabinet, taken).unwrap_err();
        assert!(why.contains(taken) && (why.contains("starter") || why.contains("box")), "{why}");
    }
    assert!(ed::keep_shape_as(&mut j, &cabinet, "  ").unwrap_err().contains("called something"));
    assert!(ed::keep_shape_as(&mut j, "no-such", "x").unwrap_err().contains("not in the job"));
    assert_eq!(file::save(&mut j), before);
    let _ = vanity;
}

#[test]
fn keeping_the_shape_of_an_item_drawn_as_a_box_or_a_form_with_no_rows_says_so() {
    let (mut j, _, _) = job();
    // call 235: the range was this test's item drawn as a box until call 233 gave it a shape of its own; a wall box still is one.
    let boxed = {
        let mut q = j.world.query::<(&plumbline_world::Node, &plumbline_world::Item)>();
        q.iter(&j.world).find(|(n, _)| n.name == "W1230").map(|(n, _)| n.path.clone()).unwrap()
    };
    let before = file::save(&mut j);
    let why = ed::keep_shape_as(&mut j, &boxed, "wall box shape").unwrap_err();
    assert!(why.contains("W1230") && why.contains("box"), "{why}");
    assert_eq!(file::save(&mut j), before);
    let why = ed::keep_shape_as(&mut j, "wall-0001", "x").unwrap_err();
    assert!(why.contains("not an item") || why.contains("no element") || why.contains("not in the job"), "{why}");
}

#[test]
fn a_block_number_that_is_not_there_is_refused_saying_how_many_there_are() {
    let (mut j, _, _) = kept();
    let before = file::save(&mut j);
    for n in [0, 4, 99] {
        let why = ed::shape_block_set(&mut j, KEPT, n, BlockSet::Round(true)).unwrap_err();
        assert!(why.contains(&format!("block {n}")) && why.contains("3 blocks"), "{why}");
        let why = ed::shape_block_take(&mut j, KEPT, n).unwrap_err();
        assert!(why.contains(&format!("block {n}")) && why.contains("3 blocks"), "{why}");
    }
    assert_eq!(file::save(&mut j), before);
    let why = ed::shape_block_add(&mut j, "no such shape", None).unwrap_err();
    assert!(why.contains("no shape called \"no such shape\"") && why.contains(KEPT), "{why}");
}

#[test]
fn the_last_block_cannot_be_taken_away() {
    let (mut j, _, _) = kept();
    ed::shape_block_take(&mut j, KEPT, 3).unwrap();
    ed::shape_block_take(&mut j, KEPT, 2).unwrap();
    let before = file::save(&mut j);
    let why = ed::shape_block_take(&mut j, KEPT, 1).unwrap_err();
    assert!(why.contains("no blocks") || why.contains("at least one"), "{why}");
    assert_eq!(file::save(&mut j), before);
}

#[test]
fn a_figure_that_is_not_a_formula_or_is_not_a_length_or_names_something_the_item_has_not_is_refused() {
    let (mut j, _, _) = kept();
    let before = file::save(&mut j);
    for (text, said) in [
        ("banana +", "block 1, Across to"),
        ("4", "plain number"),
        ("width * depth", "an area"),
        ("length", "\"length\""),
        ("length / 2", "width, depth and height"),
        ("", "Across to"),
    ] {
        let why = ed::shape_block_set(&mut j, KEPT, 1, BlockSet::Figure(1, text.into())).unwrap_err();
        assert!(why.contains(said), "{text:?}: {why}");
        assert_eq!(file::save(&mut j), before, "{text:?}");
    }
    // A figure number that is not one of the six.
    assert!(ed::shape_block_set(&mut j, KEPT, 1, BlockSet::Figure(6, "0".into())).unwrap_err().contains("six"));
}

#[test]
fn there_when_is_set_changes_what_is_drawn_and_is_unset() {
    let (mut j, _, _) = kept();
    // A block added is there whatever the size; "there when width >= 40 in" takes it away at 36 in.
    ed::shape_block_add(&mut j, KEPT, None).unwrap();
    assert_eq!(blocks(&j, KEPT, VANITY).len(), 4);
    ed::shape_block_set(&mut j, KEPT, 4, BlockSet::When(Some("width >= 40 in".into()))).unwrap();
    assert_eq!(blocks(&j, KEPT, VANITY).len(), 3, "not there at 36 in");
    assert_eq!(blocks(&j, KEPT, [42.0, 21.0, 34.5]).len(), 4, "there at 42 in");
    assert_eq!(j.shapes[0].blocks[3].when.as_deref(), Some("width >= 40 in"));
    // A condition that is not a comparison is refused, the job as it was.
    let before = file::save(&mut j);
    let why = ed::shape_block_set(&mut j, KEPT, 4, BlockSet::When(Some("width".into()))).unwrap_err();
    assert!(why.contains("no comparison") && why.contains("block 4"), "{why}");
    assert_eq!(file::save(&mut j), before);
    // Unset: always there again, and a blank one is the same as none.
    ed::shape_block_set(&mut j, KEPT, 4, BlockSet::When(None)).unwrap();
    assert_eq!(j.shapes[0].blocks[3].when, None);
    assert_eq!(blocks(&j, KEPT, VANITY).len(), 4);
    ed::shape_block_set(&mut j, KEPT, 4, BlockSet::When(Some("width >= 40 in".into()))).unwrap();
    ed::shape_block_set(&mut j, KEPT, 4, BlockSet::When(Some("  ".into()))).unwrap();
    assert_eq!(j.shapes[0].blocks[3].when, None);
}

#[test]
fn the_form_and_the_material_are_set() {
    let (mut j, _, _) = kept();
    ed::shape_block_set(&mut j, KEPT, 3, BlockSet::Made(Made::Steel)).unwrap();
    ed::shape_block_set(&mut j, KEPT, 3, BlockSet::Round(true)).unwrap();
    let b = &j.shapes[0].blocks[2];
    assert!(b.round && b.made == Made::Steel);
    ed::shape_block_set(&mut j, KEPT, 3, BlockSet::Round(false)).unwrap();
    assert!(!j.shapes[0].blocks[2].round);
}

#[test]
fn a_shape_no_item_is_drawn_as_changes_and_says_nothing_follows() {
    let (mut j, _, _) = job();
    tables::shapes_in(&mut j, &csv_rows(&format!("{HEAD}lone,timber,box,-width / 2,width / 2,0,depth,0,height,\n")), None).unwrap();
    let said = ed::shape_block_add(&mut j, "lone", None).unwrap();
    assert_eq!((said.items.len(), said.set_down.len(), said.blocks), (0, 0, 2));
    let text = said.said();
    assert!(text.contains("No item is drawn as it"), "{text}");
}

#[test]
fn each_change_is_one_step_the_history_takes_back_and_does_again() {
    let (mut j, vanity, _) = job();
    let mut h = History::default();
    let steps = |h: &mut History, j: &mut Job, label: &str, f: &dyn Fn(&mut Job)| {
        let before = History::before(j);
        f(j);
        assert!(h.record(label, before, j), "{label}: a step");
    };
    let v = vanity.clone();
    steps(&mut h, &mut j, "keep", &move |j| {
        ed::keep_shape_as(j, &v, KEPT).unwrap();
    });
    steps(&mut h, &mut j, "add", &|j| {
        ed::shape_block_add(j, KEPT, None).unwrap();
    });
    assert_eq!(j.shapes[0].blocks.len(), 4);
    h.undo(&mut j).unwrap();
    assert_eq!(j.shapes[0].blocks.len(), 3);
    h.undo(&mut j).unwrap();
    assert!(j.shapes.is_empty());
    assert_eq!(form_of(&mut j, &vanity).as_deref(), Some("base cabinet"));
    h.redo(&mut j).unwrap();
    h.redo(&mut j).unwrap();
    assert_eq!(j.shapes[0].blocks.len(), 4);
    assert_eq!(form_of(&mut j, &vanity).as_deref(), Some(KEPT));
}
