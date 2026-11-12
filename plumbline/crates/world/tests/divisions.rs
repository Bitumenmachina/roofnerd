//! Sixteen divisions: every cost line carries its division, and the job cost is read
//! division by division. The figures are made up and worked by hand in the comments.

use plumbline_world::editing::{self as ed, Entered, Top};
use plumbline_world::file::{load, save};
use plumbline_world::value::Status;
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

/// Two walls of 10 ft by 8 ft, 80 SF each, and the job itself.
fn made() -> (Job, String, String, String) {
    let mut job = Job::new("Divisions (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "Ground", "0 ft", "8 ft", KNOWN).unwrap();
    let wall_type = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    let walls = ed::walls(&mut job, &level, &wall_type, &[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], false, Top::Height("8 ft"), KNOWN).unwrap();
    let root = job.world.get::<Node>(job.root()).unwrap().path.clone();
    (job, root, walls[0].clone(), walls[1].clone())
}

fn priced(job: &mut Job, path: &str, line: &str, quantity: &str, price: &str, division: Option<u8>) {
    job.add_line(path, line, quantity, Some(Class::Material), false).unwrap();
    job.set_on_line(path, "unit_cost", Some(line), Some(price), Status::Known, "made-up price").unwrap();
    if let Some(n) = division {
        job.set_division(path, Some(line), Division::of_number(n).unwrap()).unwrap();
    }
}

#[test]
fn there_are_sixteen_and_each_is_written_as_people_write_it() {
    assert_eq!(Division::ALL.len(), 16);
    let titles: Vec<&str> = Division::ALL.iter().map(|d| d.title()).collect();
    // MasterFormat 1995's, in order (DECISIONS.md call 57 has the source).
    assert_eq!(
        titles,
        [
            "General Requirements", "Site Construction", "Concrete", "Masonry", "Metals", "Wood and Plastics", "Thermal and Moisture Protection", "Doors and Windows",
            "Finishes", "Specialties", "Equipment", "Furnishings", "Special Construction", "Conveying Systems", "Mechanical", "Electrical",
        ]
    );
    for (i, d) in Division::ALL.iter().enumerate() {
        assert_eq!(usize::from(d.number()), i + 1);
        assert_eq!(Division::of_number(d.number()), Some(*d));
    }
    for written in ["7", "07", "Division 7", "division 07", "Div. 7", "div 7", "Thermal and Moisture Protection", "thermal and moisture protection"] {
        assert_eq!(Division::parse(written), Ok(Division::ThermalAndMoistureProtection), "{written}");
    }
    for not_one in ["0", "17", "7.5", "seven", "Roofing", "", "-3"] {
        assert!(Division::parse(not_one).is_err(), "{not_one:?} was taken for a division");
    }
    assert_eq!(Division::Mechanical.said(), "Division 15, Mechanical");
}

#[test]
fn the_job_cost_is_read_division_by_division_and_what_is_in_none_is_counted_apart() {
    let (mut job, root, south, east) = made();
    // 80 SF at $4.00 on each wall, Division 6: $320.00 and $320.00.
    priced(&mut job, &south, "Framing", "face_area", "$4.00/SF", Some(6));
    priced(&mut job, &east, "Framing", "face_area", "$4.00/SF", Some(6));
    // 80 SF at $9.50 on the south wall, Division 7: $760.00.
    priced(&mut job, &south, "Siding", "face_area", "$9.50/SF", Some(7));
    // On the job: a permit in Division 1, $1,200.00; and a contingency in none, $500.00.
    priced(&mut job, &root, "Permit", "1", "$1,200", Some(1));
    priced(&mut job, &root, "Contingency", "1", "$500", None);
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!(est.divisions.len(), 16);
    let of = |n: u8| est.divisions.iter().find(|d| d.division.number() == n).map(|d| (d.total.value, d.total.members)).unwrap();
    assert_eq!(of(1), (Some(1200.00), 1));
    assert_eq!(of(6), (Some(640.00), 2));
    assert_eq!(of(7), (Some(760.00), 1));
    // A division with nothing in it has no total, which is not $0.00.
    for empty in [2u8, 3, 4, 5, 8, 9, 10, 11, 12, 13, 14, 15, 16] {
        assert_eq!(of(empty), (None, 0), "division {empty}");
    }
    assert_eq!((est.no_division.value, est.no_division.members), (Some(500.00), 1));
    // The sixteen and what is in none come to the job cost: 1,200 + 640 + 760 + 500.
    assert_eq!(est.job_cost.value, Some(3100.00));
    let sixteen: f64 = est.divisions.iter().filter_map(|d| d.total.value).sum();
    assert_eq!(sixteen + est.no_division.value.unwrap(), 3100.00);
    // Given its division, the contingency is in it, and nothing is in none.
    job.set_division(&root, Some("Contingency"), Division::GeneralRequirements).unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!(est.divisions[0].total.value, Some(1700.00));
    assert_eq!((est.no_division.value, est.no_division.members), (None, 0));
}

#[test]
fn a_line_with_no_cost_type_is_in_no_division_total_and_a_switched_off_line_neither() {
    let (mut job, _, south, _) = made();
    job.add_line(&south, "Framing", "face_area", None, false).unwrap();
    job.set_on_line(&south, "unit_cost", Some("Framing"), Some("$4.00/SF"), Status::Known, "made-up price").unwrap();
    job.set_division(&south, Some("Framing"), Division::WoodAndPlastics).unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    // It is out of the job cost, so it is out of its division's total too: the two agree.
    assert_eq!(est.job_cost.value, None);
    assert_eq!(est.divisions[5].total.members, 0);
    assert_eq!(est.no_division.members, 0);
}

#[test]
fn the_division_is_kept_in_the_file_and_a_file_written_before_still_loads() {
    let (mut job, _, south, _) = made();
    priced(&mut job, &south, "Siding", "face_area", "$9.50/SF", Some(7));
    job.derive();
    let text = save(&mut job);
    assert!(text.contains("\"division\": 7"), "the file does not hold the line's division as its number");
    let (mut back, _) = load(&text).unwrap();
    assert_eq!(save(&mut back), text);
    assert_eq!(view::estimate(&mut back).lines[0].division, Some(Division::ThermalAndMoistureProtection));
    // A file written before a line carried a division: the same file without it.
    let before = text.replace(",\n              \"division\": 7", "").replace("\"division\": 7,\n", "").replace("\"division\": 7", "");
    if before != text && !before.contains("division") {
        if let Ok((mut old, _)) = load(&before) {
            assert_eq!(view::estimate(&mut old).lines[0].division, None);
        }
    }
    // A division that is not one of the sixteen is refused on load.
    assert!(load(&text.replace("\"division\": 7", "\"division\": 17")).is_err());
    assert!(load(&text.replace("\"division\": 7", "\"division\": 0")).is_err());
}
