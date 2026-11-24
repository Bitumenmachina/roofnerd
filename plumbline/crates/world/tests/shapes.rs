//! Call 198: an item's shape as data. The starter shapes are the rows of `library/shapes.csv`, and
//! a shape is worked from its rows and nowhere else.
//!
//! The holding test: each shape's rows give the blocks the code they replace gave, block for block, at
//! three sizes, each figure within a hundred-millionth of a foot. The sizes: a small thing, a few inches
//! each way, where every "no more than a share of the thing" bites; the starter 36 in vanity (3 ft wide, 21 in
//! deep, 34 1/2 in high); and a large thing, where every fixed size bites.

use plumbline_world::building::forms::{Block, Form};
use plumbline_world::building::shape;

const SIZES: [[f64; 3]; 3] = [[0.4, 0.3, 0.5], [3.0, 1.75, 2.875], [6.5, 3.0, 2.6]];

/// All thirteen forms are rows (call 198, ruling 1).
const ROWS: [Form; 13] = Form::ALL;

fn near(a: &Block, b: &Block, what: &str) {
    for i in 0..3 {
        assert!((a.lo[i] - b.lo[i]).abs() < 1.0e-8, "{what}: lo[{i}] {} against {}", a.lo[i], b.lo[i]);
        assert!((a.hi[i] - b.hi[i]).abs() < 1.0e-8, "{what}: hi[{i}] {} against {}", a.hi[i], b.hi[i]);
    }
    assert_eq!((a.round, a.made), (b.round, b.made), "{what}");
}

#[test]
fn the_starter_table_has_the_thirteen_shapes_in_the_order_of_the_forms() {
    let names: Vec<&str> = shape::starter().iter().map(|s| s.name.as_str()).collect();
    let want: Vec<&str> = ROWS.iter().map(|f| f.name()).collect();
    assert_eq!(names, want);
}

/// What the code gave, written once from the code before it was taken out (`shapes_golden.txt`: the form, the
/// size's number, the block's number, its two corners, whether it is round, what it is made of; to twelve places).
/// The rows give it still, block for block, at the three sizes.
#[test]
fn each_shapes_rows_give_the_blocks_the_code_gave_at_three_sizes() {
    let golden = include_str!("shapes_golden.txt");
    let (mut seen, mut blocks) = (std::collections::BTreeSet::new(), 0);
    let mut lines = golden.lines().peekable();
    let mut sizes = 0;
    for form in ROWS {
        for (s, size) in SIZES.iter().enumerate() {
            sizes += 1;
            seen.insert(form.name());
            let rows = shape::starter_blocks(form.name(), *size).unwrap();
            let mut n = 0;
            while let Some(line) = lines.next_if(|l| l.starts_with(&format!("{}|{s}|", form.name()))) {
                let f: Vec<&str> = line.split('|').collect();
                let corner = |t: &str| -> [f64; 3] {
                    let v: Vec<f64> = t.split(',').map(|x| x.parse().unwrap()).collect();
                    [v[0], v[1], v[2]]
                };
                let want = Block { lo: corner(f[3]), hi: corner(f[4]), round: f[5] == "true", made: plumbline_world::building::forms::Made::named(f[6]).unwrap() };
                assert_eq!(f[2].parse::<usize>().unwrap(), n, "{line}");
                near(&rows[n], &want, &format!("{} at {size:?}, block {}", form.name(), n + 1));
                n += 1;
                blocks += 1;
            }
            assert_eq!(rows.len(), n, "{} at {size:?}: the number of blocks", form.name());
        }
    }
    assert!(lines.next().is_none(), "the golden file has a line no form takes");
    println!("held: {} shapes, {sizes} shape-and-size pairs, {blocks} blocks compared", seen.len());
    assert_eq!((seen.len(), sizes), (13, 39));
    println!("golden blocks: {blocks}");
}

/// The two conditions bite on both sides: the bed has one pillow below 1200 mm (3.937 ft) and two above; the sink has its
/// wall between two bowls from 30 in. The small size is below both, the typical (36 in) is below the bed's and above the
/// sink's, the large is above both.
#[test]
fn the_bed_and_the_sink_have_their_blocks_only_where_the_condition_holds() {
    let n = |f: Form, i: usize| shape::starter_blocks(f.name(), SIZES[i]).unwrap().len();
    assert_eq!((n(Form::Bed, 0), n(Form::Bed, 1), n(Form::Bed, 2)), (4, 4, 5), "pillows: one below 1200 mm, two above");
    assert_eq!((n(Form::Sink, 0), n(Form::Sink, 1), n(Form::Sink, 2)), (5, 6, 6), "the divider: not below 30 in");
}
