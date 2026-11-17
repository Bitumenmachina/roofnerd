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

#[test]
fn the_priced_sample_house_has_work_in_every_one_of_the_sixteen_and_they_add_to_the_job_cost() {
    let mut job = plumbline_world::sample::house_priced();
    job.derive();
    let est = view::estimate(&mut job);
    assert_eq!(est.divisions.len(), 16);
    for d in &est.divisions {
        assert!(d.total.members > 0, "division {} has no line in the priced sample", d.division.number());
        assert!(d.total.value.is_some(), "division {} has no total", d.division.number());
    }
    // Nothing in it is in no division, and the sixteen add to the job cost to the cent.
    assert_eq!(est.no_division.members, 0);
    let cents = |v: f64| (v * 100.0).round() as i64;
    let sum: i64 = est.divisions.iter().map(|d| cents(d.total.value.unwrap())).sum();
    assert_eq!(sum, cents(est.job_cost.value.unwrap()));
    // Every price in it says it is made up.
    for l in &est.lines {
        if let Some(c) = &l.unit_cost {
            assert!(c.source.contains("made-up"), "a price in the sample does not say it is made up: {}", c.source);
        }
    }
}

/// J17A-1, J17B-20. A job file changed by hand and refused is refused in a person's
/// words: nothing of the reader underneath (its variants, fields and number types) and
/// none of the file's own keys.
#[test]
fn a_job_file_changed_by_hand_is_refused_in_plain_words() {
    let (mut job, root, a, _) = made();
    priced(&mut job, &a, "Siding", "net_area", "$9.00/SF", Some(7));
    let _ = root;
    let text = save(&mut job);
    let changed = |edit: &dyn Fn(&mut serde_json::Value)| -> String {
        let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
        let wall = doc["data"].as_array_mut().unwrap().iter_mut().find(|e| e["path"] == a.as_str()).unwrap();
        edit(&mut wall["attributes"]["pl::cost"]["lines"][0]);
        load(&doc.to_string()).err().expect("the change is refused")
    };
    let cases: [(&dyn Fn(&mut serde_json::Value), &str); 6] = [
        (&|l| l["division"] = 7.5.into(), "its cost lines: it holds 7.5 where a whole number belongs"),
        (&|l| l["division"] = "7".into(), "it holds \"7\" where a whole number belongs"),
        (&|l| l["division"] = 300.into(), "it holds 300, which is not a figure that can stand there"),
        (&|l| l["class"] = "Roofer".into(), "it says \"Roofer\", which is not one of: Material, Labor, Fabrication, Supervision, Subcontract, General"),
        (&|l| l["unit_cost"]["status"] = "Maybe".into(), "it says \"Maybe\", which is not one of: Known, Assumed, Unknown"),
        (&|l| { l.as_object_mut().unwrap().remove("description"); }, "it has no \"description\", and needs one"),
    ];
    for (edit, want) in cases {
        let why = changed(edit);
        assert!(why.contains(want), "{why}");
        for word in ["u8", "f64", "variant", "field", "invalid", "struct", "pl::", "`"] {
            assert!(!why.contains(word), "{word:?} in {why:?}");
        }
    }
    // A file cut short, and one that is not a job at all.
    let cut = load(&text[..text.len() / 2]).err().unwrap();
    assert!(cut.contains("the file stops short") && cut.contains("(line ") && !cut.contains("EOF") && !cut.contains("column"), "{cut}");
    let other = load("hello").err().unwrap();
    assert_eq!(other, "not a job file: it is not written the way this program writes a job (line 1 of the file)");
}

#[test]
fn an_allowance_on_a_thing_with_no_cost_line_is_carried_in_the_price_and_noted_as_one() {
    use plumbline_world::view;
    // Patrick, 2026-11-17 (call 114): "Allowances are allowed and noted as such". The
    // sample house's living room is 614.7188 SF inside its walls and has no cost line.
    // At a made-up $10.00/SF its allowance is $6,147.19 (6,147.188 to the cent).
    let mut job = plumbline_world::sample::house();
    let path = job.order.clone().into_iter().find_map(|e| job.world.get::<plumbline_world::Node>(e).filter(|n| n.name == "living room").map(|n| n.path.clone())).unwrap();
    assert_eq!(view::pricing(&mut job).allowances_alone(), None, "nothing is priced by an allowance alone to begin with");
    let before = view::estimate(&mut job);
    assert!(before.allowance_on.is_empty() && before.allowances.value.is_none());
    job.set(&path, "allowance", Some("10"), plumbline_world::Status::Assumed, "made-up budget").unwrap();
    job.derive();
    let pricing = view::pricing(&mut job);
    assert_eq!(pricing.allowances_alone(), Some((1, Some(6147.19))));
    let said = pricing.allowances_alone_said().unwrap();
    assert_eq!(said, "1 thing is priced by an allowance and no cost line ($6,147.19): it is carried in the job cost and the price as an allowance, and noted as such, until it is given cost lines");
    // It is in the job cost, on a row of its own, in no cost type and no division; the
    // sixteen and what they leave out still come to the job cost.
    let est = view::estimate(&mut job);
    let cents = |x: f64| (x * 100.0).round() / 100.0;
    assert_eq!((est.allowances.value, est.allowances.members, est.allowances.status), (Some(6147.19), 1, plumbline_world::Status::Assumed));
    // (The sample house has nothing priced to begin with: its job cost is the allowance.)
    assert_eq!(est.job_cost.value.map(cents), Some(cents(before.job_cost.value.unwrap_or(0.0) + 6147.19)));
    assert_eq!(est.classes.iter().filter_map(|c| c.total.value).sum::<f64>(), before.classes.iter().filter_map(|c| c.total.value).sum::<f64>());
    let sixteen: f64 = est.divisions.iter().filter_map(|d| d.total.value).sum();
    assert_eq!(cents(sixteen + est.no_division.value.unwrap_or(0.0)), cents(est.job_cost.value.unwrap()));
    assert_eq!(est.no_division.value.map(cents), Some(cents(before.no_division.value.unwrap_or(0.0) + 6147.19)));
    // The price is over it, where the job has its markup.
    if let (Some(was), Some(is)) = (before.price.value, est.price.value) {
        assert!(is > was + 6147.19 - 0.005, "{was} {is}");
    }
    // Given a cost line that comes to a figure, the room is priced by its line and the
    // allowance is no longer carried: 614.7188 SF at a made-up $2.00 is $1,229.44.
    job.add_line(&path, "Flooring", "floor_area", Some(plumbline_world::Class::Material), false).unwrap();
    job.set_on_line(&path, "unit_cost", Some("Flooring"), Some("$2.00/SF"), plumbline_world::Status::Assumed, "made up").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    assert!(est.allowance_on.is_empty() && est.allowances.value.is_none());
    assert_eq!(est.job_cost.value.map(cents), Some(cents(before.job_cost.value.unwrap_or(0.0) + 1229.44)));
}

#[test]
fn in_house_work_is_added_with_its_production_rate_or_not_at_all() {
    use plumbline_world::{view, Class, Status};
    // The sample house's south wall is 329.99 SF net. Siding labor at a made-up 40 SF
    // an hour is 8.2498 hr (329.99 / 40 = 8.24975); at a made-up $60.00 an hour that
    // is $494.99 (8.2498 x 60 = 494.988).
    let mut job = plumbline_world::sample::house();
    let wall = job.order.clone().into_iter().find_map(|e| job.world.get::<plumbline_world::Node>(e).filter(|n| n.name == "south wall").map(|n| n.path.clone())).unwrap();
    let lines_before = job.find(&wall).and_then(|e| job.world.get::<plumbline_world::Cost>(e)).map(|c| c.lines.len()).unwrap_or(0);
    // A rate that cannot be read leaves the wall as it was.
    let why = job.add_line_in_hours(&wall, "Siding labor", "net_area", Class::Labor, "fast", Status::Assumed, "made up").unwrap_err();
    assert!(why.contains("\"fast\" is not a production rate; write how fast the work goes"), "{why}");
    assert_eq!(job.find(&wall).and_then(|e| job.world.get::<plumbline_world::Cost>(e)).map(|c| c.lines.len()).unwrap_or(0), lines_before);
    // With one that can, the line is there, in hours, with no price yet.
    job.add_line_in_hours(&wall, "Siding labor", "net_area", Class::Labor, "40 SF/hr", Status::Assumed, "made up").unwrap();
    job.set_on_line(&wall, "unit_cost", Some("Siding labor"), Some("60"), Status::Assumed, "made up").unwrap();
    job.derive();
    let est = view::estimate(&mut job);
    let line = est.lines.iter().find(|l| l.path == wall && l.description == "Siding labor").unwrap();
    assert_eq!((line.class, line.priced.value, line.priced.unit.as_str(), line.amount.value), (Some(Class::Labor), Some(8.2498), "hr", Some(494.99)));
    // A rate said in squares is taken on a line that comes out in SF (J24A-7): a
    // square is 100 SF, so a made-up 0.5 SQ an hour is 50 SF an hour, 329.99 / 50 =
    // 6.5998 hr; and 2 hr a square is 0.02 hr/SF, 329.99 x 0.02 = 6.5998 hr again.
    for said in ["0.5 SQ/hr", "2 hr/SQ"] {
        job.set_on_line(&wall, "order", Some("Siding labor"), Some(said), Status::Assumed, "made up").unwrap();
        job.derive();
        let est = view::estimate(&mut job);
        let line = est.lines.iter().find(|l| l.path == wall && l.description == "Siding labor").unwrap();
        assert_eq!((line.priced.value, line.priced.unit.as_str()), (Some(6.5998), "hr"), "{said}");
    }
    assert_eq!(view::estimate(&mut job).lines.iter().find(|l| l.path == wall && l.description == "Siding labor").unwrap().rate_said().as_deref(), Some("0.02 hr/SF"));
    // A rate in a unit that is neither is still refused.
    assert!(job.set_on_line(&wall, "order", Some("Siding labor"), Some("3 LF/hr"), Status::Assumed, "made up").unwrap_err().contains("does not convert this line, which comes out in SF"));
}

#[test]
fn a_plane_knows_the_other_planes_of_its_roof() {
    use plumbline_world::{Kind, Node};
    // The sample house's roof is solved from a footprint of four edges: four planes.
    // Each has the other three as the rest of its roof; a wall has none.
    let mut job = plumbline_world::sample::house();
    let mut q = job.world.query::<(&Node, &Kind)>();
    let planes: Vec<String> = q.iter(&job.world).filter(|(_, k)| **k == Kind::RoofPlane).map(|(n, _)| n.path.clone()).collect();
    assert_eq!(planes.len(), 4);
    for plane in &planes {
        let mut rest = job.sister_planes(plane);
        rest.sort();
        let mut want: Vec<String> = planes.iter().filter(|p| *p != plane).cloned().collect();
        want.sort();
        assert_eq!(rest, want, "{plane}");
    }
    let wall = q.iter(&job.world).find(|(_, k)| **k == Kind::Wall).map(|(n, _)| n.path.clone()).unwrap();
    assert!(job.sister_planes(&wall).is_empty());
}

#[test]
fn the_estimate_reads_item_by_item_and_comes_to_the_same_money() {
    use plumbline_world::view;
    // The sample whole house, priced. Its four strip footings are four lines of the
    // sheet, "Concrete footing, in place" at a made-up $450.00/CY: 2.9630 + 2.0741 +
    // 2.9630 + 2.0741 = 10.0742 CY, and 1,333.35 + 933.35 + 1,333.35 + 933.35 =
    // $4,533.40. Read by item they are one line of four places.
    let mut job = plumbline_world::sample::house_priced();
    let est = view::estimate(&mut job);
    let items = est.by_item();
    let footings = items.iter().find(|i| i.description == "Concrete footing, in place").unwrap();
    assert_eq!((footings.places, footings.quantity, footings.unit.as_str(), footings.unit_cost), (4, Some(10.0742), "CY", Some(450.0)));
    assert_eq!(footings.amount.value, Some(4533.40));
    assert_eq!(footings.division.map(|d| d.number()), Some(3));
    // Written to the places it needs: 10.07 x 450 is 4,531.50, not the amount, so the
    // four it is held to are shown (call 120). The slab, 1,120 SF at $6.50, needs two.
    assert_eq!(footings.quantity_said().as_deref(), Some("10.0742"));
    let slab = items.iter().find(|i| i.description == "Concrete slab on grade, 4 in").unwrap();
    assert_eq!((slab.quantity_said().as_deref(), slab.amount.value), (Some("1,120.00"), Some(7280.0)));
    // Every line that is on is in one item and no other, and the items come to what
    // the lines come to: nothing is worked out again and nothing is left behind.
    let on = est.lines.iter().filter(|l| !l.off).count();
    assert_eq!(items.iter().map(|i| i.places).sum::<usize>(), on);
    assert!(items.len() < on / 3, "{} items from {on} lines", items.len());
    let cents = |x: f64| (x * 100.0).round() / 100.0;
    let lines: f64 = est.lines.iter().filter(|l| !l.off && l.class.is_some()).filter_map(|l| l.amount.value).sum();
    let by_item: f64 = items.iter().filter(|i| i.class.is_some()).filter_map(|i| i.amount.value).sum();
    assert_eq!(cents(by_item), cents(lines));
    assert_eq!(Some(cents(by_item)), est.job_cost.value.map(cents));
    // The items are in the order of the divisions.
    let order: Vec<u8> = items.iter().map(|i| i.division.map(|d| d.number()).unwrap_or(u8::MAX)).collect();
    assert!(order.windows(2).all(|w| w[0] <= w[1]));
}
