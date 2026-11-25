//! Call 198: the formula language gains `min(a, b)`, `max(a, b)` and a fixed size written with its unit
//! (`4 in`, `100 mm`, `100 SF`), for cost lines and shapes alike. Figures worked by hand; a formula written before
//! reads as it did.

use plumbline_world::formula::{self, Expr, Operand};
use plumbline_world::value::Status;

fn work(text: &str, names: &[(&str, Option<f64>, i32)]) -> Result<formula::Worked, String> {
    let expr = formula::parse(text).map_err(|why| format!("{text:?} {why}"))?;
    formula::work(&expr, &|name| match names.iter().find(|(n, _, _)| *n == name) {
        Some((n, value, dim)) => Ok(Operand { value: *value, dim: *dim, status: Status::Known, shown: n.to_string() }),
        None => Err(format!("no name {name}")),
    })
}

#[test]
fn the_lesser_and_the_greater_of_two_figures_with_a_unit() {
    // A wall of 120 SF net: min(net_area, 100 SF) is 100; of 80 SF, 80 (call 198, point 6).
    let w = work("min(net_area, 100 SF)", &[("net_area", Some(120.0), 2)]).unwrap();
    assert_eq!((w.value, w.dim), (Some(100.0), 2));
    assert_eq!(w.text, "min(net_area, 100 SF)");
    let w = work("min(net_area, 100 SF)", &[("net_area", Some(80.0), 2)]).unwrap();
    assert_eq!(w.value, Some(80.0));
    let w = work("max(net_area, 100 SF)", &[("net_area", Some(80.0), 2)]).unwrap();
    assert_eq!(w.value, Some(100.0));
    // More than two, and nested.
    let w = work("max(1, min(7, 3), 2)", &[]).unwrap();
    assert_eq!(w.value, Some(3.0));
}

#[test]
fn a_fixed_size_is_written_with_its_unit_and_worked_in_feet() {
    let near = |text: &str, want: f64, dim: i32, shown: &str| {
        let w = work(text, &[]).unwrap();
        assert!((w.value.unwrap() - want).abs() < 1.0e-12, "{text}: {:?}", w.value);
        assert_eq!(w.dim, dim, "{text}");
        assert_eq!(w.text, shown, "{text}");
    };
    near("4 in", 4.0 / 12.0, 1, "4 \u{00F7} 12");
    near("100 mm", 100.0 / 304.8, 1, "100 \u{00F7} 304.8");
    near("3 ft", 3.0, 1, "3 ft");
    near("20 LF", 20.0, 1, "20 LF");
    near("100 SF", 100.0, 2, "100 SF");
    // A share of a size, and the lesser of a fixed size and a share: "a 100 mm plinth, no more than 14 % of the height".
    let w = work("min(100 mm, 0.14 * height)", &[("height", Some(2.875), 1)]).unwrap();
    assert!((w.value.unwrap() - 100.0 / 304.8).abs() < 1.0e-12);
    let w = work("min(100 mm, 0.14 * height)", &[("height", Some(0.5), 1)]).unwrap();
    assert!((w.value.unwrap() - 0.07).abs() < 1.0e-12);
}

#[test]
fn an_unknown_figure_leaves_no_result_and_a_length_is_not_compared_with_an_area() {
    let w = work("min(net_area, 100 SF)", &[("net_area", None, 2)]).unwrap();
    assert_eq!(w.value, None, "never zero");
    assert_eq!(w.status, Status::Unknown);
    let why = work("min(net_length, 100 SF)", &[("net_length", Some(5.0), 1)]).unwrap_err();
    assert!(why.contains("cannot be compared"), "{why}");
    assert!(formula::parse("min(5)").unwrap_err().contains("two figures or more"));
    assert!(formula::parse("min(1, 2").unwrap_err().contains("never closed"));
    assert!(formula::parse("1, 2").unwrap_err().contains("comma"));
}

#[test]
fn a_formula_written_before_reads_as_it_did() {
    // `min` and `max` alone are names, as they were; no bracket, no lesser.
    assert_eq!(formula::parse("min * 2").unwrap(), Expr::Bin(Box::new(Expr::Name("min".into())), '*', Box::new(Expr::Num(2.0))));
    assert_eq!(formula::parse("max").unwrap(), Expr::Name("max".into()));
    // A number and a name side by side was an error and is still, unless the name is a unit.
    assert!(formula::parse("4 plan_length").unwrap_err().contains("side by side"));
    assert_eq!(formula::parse("plan_length * sides / spacing").unwrap().clone(), formula::parse("plan length * sides / spacing").unwrap());
    assert_eq!(formula::names(&formula::parse("min(net_area, wall_area) + stretch_out").unwrap()), vec!["net_area", "wall_area", "stretch_out"]);
}

#[test]
fn a_comparison_is_no_part_of_the_formula_language() {
    // A cost line's quantity is a quantity: the comparison of a shape's "There when" is read in the shape's row, and here it is refused as it was.
    for text in ["net_area > 100 SF", "net_area >= 100 SF", "net_area < 100 SF", "net_area = 100 SF"] {
        let why = formula::parse(text).unwrap_err();
        assert!(why.contains("is not something a formula can hold"), "{text}: {why}");
    }
}
