//! The verify-in-field list: what is still a question, what to measure to settle it,
//! and the dollars riding on it. The job and every figure are made up, and worked by
//! hand in STATUS.md ("The verify-in-field list") before they are compared here.

use plumbline_world::editing::{self as ed, Entered, Reading, Top};
use plumbline_world::value::Status;
use plumbline_world::vif::vif;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };
const TRACED: Entered = Entered { status: Status::Assumed, source: "scaled off a made-up plan" };

/// The field job: two levels scaled off a plan; four Known walls from the ground's
/// floor to the upper's, the south one carrying gypsum board on its net face at
/// $2.40/SF; a traced duct; two Known pipes across it.
struct Made {
    job: Job,
    ground: String,
    upper: String,
    wall: String,
    duct: String,
    pipe_a: String,
    pipe_b: String,
}

fn made() -> Made {
    let mut job = Job::new("Verified in the field (not a real job)", "test", "test");
    let ground = ed::storey(&mut job, "ground", "0 ft", "9 ft", TRACED).unwrap();
    let upper = ed::storey(&mut job, "upper", "9 ft", "8 ft", TRACED).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let duct_type = ed::segment_type(&mut job, "12 x 8", Kind::Duct, "12 in", Some("8 in"), KNOWN).unwrap();
    let pipe_type = ed::segment_type(&mut job, "2 in", Kind::Pipe, "2 in", None, KNOWN).unwrap();
    let walls = ed::walls(&mut job, &ground, &wall_type, &[[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]], true, Top::Level(&upper), KNOWN).unwrap();
    let duct = ed::runs(&mut job, &ground, Kind::Duct, &duct_type, &[[2.0, 8.0], [22.0, 8.0]], "8 ft", None, TRACED).unwrap().remove(0);
    let pipe_a = ed::runs(&mut job, &ground, Kind::Pipe, &pipe_type, &[[12.0, 2.0], [12.0, 14.0]], "7 ft 9 in", None, KNOWN).unwrap().remove(0);
    let pipe_b = ed::runs(&mut job, &ground, Kind::Pipe, &pipe_type, &[[18.0, 2.0], [18.0, 14.0]], "8 ft", None, KNOWN).unwrap().remove(0);
    let wall = walls[0].clone();
    job.add_line(&wall, "Gypsum board, hung and finished", "net_area", Some(Class::Material), false).unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Gypsum board, hung and finished"), Some("$2.40/SF"), Status::Known, "made-up plan").unwrap();
    job.derive();
    Made { job, ground, upper, wall, duct, pipe_a, pipe_b }
}

/// A row as the test reads it: the thing, what it is in a clash with, its dollars, how
/// many lines, and what to measure or check.
fn rows(job: &mut Job) -> Vec<(String, Option<String>, Option<f64>, usize, Vec<String>)> {
    vif(job).questions.into_iter().map(|q| (q.path, q.clash.map(|c| c.other), q.dollars, q.lines, q.values.into_iter().map(|v| v.0).collect())).collect()
}

#[test]
fn as_traced_two_levels_carry_the_wall_and_the_duct_and_its_clashes_carry_nothing() {
    let mut m = made();
    let list = vif(&mut m.job);
    // 24 ft by 9 ft is 216 SF, at $2.40: $518.40, one line, counted once in the total.
    assert_eq!((list.riding, list.riding_status, list.riding_lines), (Some(518.40), Status::Assumed, 1));
    let found = rows(&mut m.job);
    let levels = vec!["elevation".to_string(), "level height".to_string()];
    assert_eq!(found.len(), 5, "{found:#?}");
    assert_eq!(found[0], (m.ground.clone(), None, Some(518.40), 1, levels.clone()));
    assert_eq!(found[1], (m.upper.clone(), None, Some(518.40), 1, levels));
    // The two clashes, with no dollars, before the duct itself.
    assert_eq!((found[2].0.as_str(), found[2].1.as_deref(), found[2].2, found[2].3), (m.duct.as_str(), Some(m.pipe_a.as_str()), None, 0));
    assert_eq!((found[3].0.as_str(), found[3].1.as_deref(), found[3].2, found[3].3), (m.duct.as_str(), Some(m.pipe_b.as_str()), None, 0));
    assert_eq!((found[4].0.as_str(), found[4].1.as_deref(), found[4].2), (m.duct.as_str(), None, None));
    for value in ["type", "line on the plan", "height above its level"] {
        assert!(found[4].4.iter().any(|v| v == value), "the duct's {value} is to be verified: {:?}", found[4].4);
    }
    // What is Known asks nothing: no wall, no pipe, no type is on the list.
    for known in [&m.wall, &m.pipe_a, &m.pipe_b] {
        assert!(!found.iter().any(|r| r.0 == **known && r.1.is_none()), "{known} is Known and is on the list");
    }
}

#[test]
fn a_measurement_takes_its_question_off_and_a_line_that_is_known_has_nothing_riding() {
    let mut m = made();
    ed::measure(&mut m.job, &m.ground, "elevation", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.ground, "level height", Reading::Length("8'10\"1/2"), None, "JM with the laser").unwrap();
    // The wall is 24 by 8.875 now, 213 SF, $511.20, and Known: the upper level still has
    // its own height to verify, and no dollars ride on that.
    let found = rows(&mut m.job);
    assert!(!found.iter().any(|r| r.0 == m.ground), "the ground level is measured and is still on the list");
    let upper = found.iter().find(|r| r.0 == m.upper).expect("the upper level's height is still as scaled");
    assert_eq!((upper.2, upper.3, upper.4.clone()), (None, 0, vec!["level height".to_string()]));
    assert_eq!(vif(&mut m.job).riding, None);

    ed::measure(&mut m.job, &m.duct, "line on the plan", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.duct, "type", Reading::Same, None, "JM on site").unwrap();
    ed::measure(&mut m.job, &m.duct, "height above its level", Reading::Length("7'10\"3/4"), Some(Justify::Bottom), "JM with the laser").unwrap();
    // Pipe A is clear; pipe B in the duct is a finding now, and a finding is not a question.
    let found = rows(&mut m.job);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].0, m.upper);

    ed::measure(&mut m.job, &m.upper, "level height", Reading::Same, None, "JM on site").unwrap();
    let list = vif(&mut m.job);
    assert!(list.questions.is_empty(), "{:#?}", list.questions);
    assert_eq!(list.riding, None);
}

#[test]
fn a_line_on_two_things_in_question_is_counted_once_in_the_total() {
    let mut m = made();
    // The duct is priced too: 20 LF at $12.00, $240.00. It rides on the duct, and on
    // each of the duct's two clashes; the total counts it once: $518.40 + $240.00.
    m.job.add_line(&m.duct, "Duct, 12 x 8, hung", "true_length", Some(Class::Material), false).unwrap();
    m.job.set_on_line(&m.duct, "unit_cost", Some("Duct, 12 x 8, hung"), Some("$12.00/LF"), Status::Known, "made-up plan").unwrap();
    m.job.derive();
    let list = vif(&mut m.job);
    assert_eq!((list.riding, list.riding_lines), (Some(758.40), 2));
    let duct_rows: Vec<&plumbline_world::vif::Question> = list.questions.iter().filter(|q| q.path == m.duct).collect();
    assert_eq!(duct_rows.len(), 3);
    assert!(duct_rows.iter().all(|q| q.dollars == Some(240.00) && q.lines == 1), "{duct_rows:#?}");
    // Largest first: the two levels at $518.40, then the three at $240.00.
    let order: Vec<Option<f64>> = list.questions.iter().map(|q| q.dollars).collect();
    assert_eq!(order, vec![Some(518.40), Some(518.40), Some(240.00), Some(240.00), Some(240.00)]);
}
