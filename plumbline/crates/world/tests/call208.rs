//! Call 208 points 2 to 4, in the model. Made-up jobs; the figures are worked by hand beside each test.

use plumbline_world::editing::shapes as sh;
use plumbline_world::tables;
use plumbline_world::*;

fn row(cells: &[&str]) -> Vec<String> {
    cells.iter().map(|c| c.to_string()).collect()
}

/// Point 3: a job that holds a shape under a starter shape's name (as a saved job of the hours between two commits of 2026-11-24 could) still opens,
/// and `check` says it does and what to do.
#[test]
fn a_shape_held_under_a_starter_shapes_name_is_found_and_said() {
    let mut job = Job::new("Shapes (not a real job)", "test", "2026-11-25");
    tables::shapes_in(&mut job, &[row(&["Shape", "Made of", "Form", "Across from", "Across to", "Back from", "Back to", "Up from", "Up to", "There when"]), row(&["zz mine", "timber", "box", "-width / 2", "width / 2", "0", "depth", "0", "height", ""])], None).unwrap();
    assert!(sh::shapes_under_starter_names(&mut job).is_empty(), "a shape with a name of its own is nothing to say");
    // As an old file holds it: the shape's name changed with nothing checked.
    job.shapes[0].name = "Bathtub".into();
    let said = sh::shapes_under_starter_names(&mut job);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].1.contains("\"Bathtub\" is a starter shape's name") && said[0].1.contains("keep it under another name"), "{said:?}");
}
