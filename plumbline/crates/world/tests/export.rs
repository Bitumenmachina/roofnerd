//! The estimate written out: a workbook whose figures are formulas, and three lenses.
//! The job is the made-up priced job; its figures are worked by hand in `demos/d5.sh`
//! and, for what is written out, in STATUS.md ("Exports and lenses").

use plumbline_world::export::{estimate_out, lens_out, values, Lens, Out};
use plumbline_world::value::Status;
use plumbline_world::*;

fn job() -> Job {
    let mut job = sample::priced();
    let root = job.world.get::<Node>(job.root()).unwrap().path.clone();
    job.set(&root, "overhead", Some("10"), Status::Known, "made up").unwrap();
    job.set(&root, "profit", Some("8"), Status::Known, "made up").unwrap();
    job.derive();
    job
}

fn number(c: &Out) -> f64 {
    match c {
        Out::Number(x) | Out::Dollars(x) => *x,
        Out::Formula { value: Some(x), .. } => *x,
        other => panic!("not a figure: {other:?}"),
    }
}

#[test]
fn each_amount_is_a_formula_that_comes_to_the_jobs_figure_and_the_recap_is_formulas_on_them() {
    let mut job = job();
    let sheets = estimate_out(&mut job, true);
    let names: Vec<&str> = sheets.iter().map(|s| s.name.as_str()).collect();
    // (Call 57: the job cost is also written division by division.)
    assert_eq!(names, ["Estimate", "Recap", "Divisions", "Suppliers"]);
    let est = &sheets[0];
    // Fourteen lines under the headings. Thirteen come to something, each as a formula
    // on its own row's quantity and price; the north plane's has no price and no amount.
    let lines = &est.rows[est.head + 1..];
    assert_eq!(lines.len(), 14);
    let mut sum = 0.0;
    for (i, row) in lines.iter().enumerate() {
        let n = est.head + 2 + i;
        match &row[6] {
            Out::Formula { text, value: Some(v) } => {
                assert_eq!(*text, format!("=ROUND(D{n}*F{n},2)"));
                let worked = (number(&row[3]) * number(&row[5]) * 100.0).round() / 100.0;
                assert!((worked - v).abs() < 0.005, "row {n}: {} x {} is {worked}, the job has {v}", number(&row[3]), number(&row[5]));
                sum += v;
            }
            // A line with no price yet has its formula all the same, and it comes to
            // nothing until a price is typed on the sheet (the seventeenth judgment, J17A-6).
            Out::FormulaText { text, shown } => {
                // The one line of this job with no cost type and no price.
                assert_eq!(row[1], Out::Text("Copper standing seam, installed".into()));
                assert_eq!(*text, format!("=IF(OR(C{n}=\"\",D{n}=\"\",F{n}=\"\"),\"\",ROUND(D{n}*F{n},2))"));
                assert_eq!(shown, "");
            }
            other => panic!("row {n}: {other:?}"),
        }
    }
    assert!((sum - 25292.43).abs() < 0.005, "{sum}");
    // The recap: six classes, the job cost, overhead and profit on a percentage each, the price.
    let recap = &sheets[1];
    let figure = |label: &str| number(&recap.rows.iter().find(|r| r.first() == Some(&Out::Text(label.into()))).unwrap_or_else(|| panic!("no row {label}"))[1]);
    assert_eq!(figure("Material"), 8628.10);
    assert_eq!(figure("Labor"), 2147.97);
    assert_eq!(figure("Fabrication"), 402.50);
    assert_eq!(figure("Supervision"), 3800.00);
    assert_eq!(figure("Subcontract"), 10313.86);
    assert_eq!(figure("Job cost"), 25292.43);
    assert_eq!(figure("Overhead"), 2529.24);
    assert_eq!(figure("Profit"), 2225.73);
    assert_eq!(figure("Price"), 30047.40);
    // A cost type with nothing in it has no total, which is not $0.00: its formula comes
    // to "none" until a line of that type has an amount (J17A-6).
    let general = recap.rows.iter().find(|r| r.first() == Some(&Out::Text("General conditions".into()))).unwrap();
    match &general[1] {
        Out::FormulaText { text, shown } => {
            assert_eq!(shown, "none");
            assert!(text.starts_with("=IF(COUNTIFS(Estimate!$C$") && text.contains("\"General conditions\"") && text.contains("=0,\"none\",\"no figure\"),SUMIF("), "{text}");
        }
        other => panic!("{other:?}"),
    }
    // It has no note and no hours: there is nothing in it to be unsure of.
    assert_eq!((general[3].clone(), general[4].clone()), (Out::None, Out::Text(String::new())));
    // What the price leaves out is named under it, once.
    let said: Vec<String> = values(&sheets[1..2]).into_iter().flatten().collect();
    assert!(said.iter().any(|c| c == "Left out of the job cost, and of everything taken on it: 1 line"), "{said:#?}");
}

#[test]
fn the_lenses_are_the_estimates_quantities_with_none_of_our_prices() {
    let mut job = job();
    let supplier = &lens_out(&mut job, Lens::Supplier)[0];
    let rows: Vec<(String, f64, String, f64, String)> = supplier.rows[supplier.head + 1..].iter().take_while(|r| !r.is_empty()).map(|r| (r[0].said(), number(&r[1]), r[2].said(), number(&r[3]), r[4].said())).collect();
    // 461.1890 + 276.7134 lb; 419.2627 + 251.5576 SF; 123.6931 + 30 lb, Assumed.
    assert_eq!(
        rows,
        vec![
            ("Copper, 16 oz".to_string(), 737.9024, "lb".to_string(), 2.0, "Known".to_string()),
            ("Underlayment".to_string(), 670.8203, "SF".to_string(), 2.0, "Known".to_string()),
            ("Copper, 20 oz".to_string(), 153.6931, "lb".to_string(), 2.0, "Assumed".to_string()),
        ]
    );
    let sub = &lens_out(&mut job, Lens::Subcontractor)[0];
    let scope: Vec<Vec<String>> = sub.rows[sub.head + 1..].iter().take_while(|r| !r.is_empty()).map(|r| r.iter().map(Out::said).collect()).collect();
    assert_eq!(scope, vec![vec!["west", "Standing seam roofing, by subcontractor", "251.5576", "SF", "Known", ""]]);
    // Not a price, not an amount, not a dollar in either: neither has a column for one,
    // and no cell of either is a figure of money.
    for sheet in [supplier, sub] {
        assert!(!sheet.rows.iter().flatten().any(|c| matches!(c, Out::Dollars(_) | Out::Formula { .. })), "{}", sheet.name);
        assert!(!values(std::slice::from_ref(sheet)).iter().flatten().any(|c| c.contains('$') || c.contains("9.00") || c.contains("41.00") || c.contains("0.90")), "{}", sheet.name);
    }
    // The PM's: every line; the budget by class with no markup on it; what is to verify.
    let pm = lens_out(&mut job, Lens::Pm);
    assert_eq!(pm.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Lines", "Budget", "Verify in field"]);
    assert_eq!(pm[0].rows.len() - pm[0].head - 1, 14);
    let all: Vec<String> = values(&pm).into_iter().flatten().collect();
    assert!(all.iter().any(|c| c == "25292.43"));
    for markup in ["2529.24", "2225.73", "30047.40", "Overhead", "Profit", "Price"] {
        assert!(!all.iter().any(|c| c == markup), "the PM's lens holds {markup}");
    }
}
