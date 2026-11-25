//! Call 209, the model's side of points 2, 3, 4, 5 and 8 (contract 209M). Every figure is made up and worked by hand.

use plumbline_world::editing::{self as ed, Entered};
use plumbline_world::units::{parse_length, Len};
use plumbline_world::value::{Status, Val};
use plumbline_world::*;

const KNOWN: Entered = Entered { status: Status::Known, source: "made-up plan" };

fn item(job: &mut Job, name: &str) -> String {
    let inches = |x: f64| Val { value: Some(Len::One(x)), unit: "in".to_string(), status: Status::Assumed, source: "a made-up catalog".to_string() };
    let unknown = || Val::<String>::unknown("", "not chosen");
    let i = Item { category: Kind::Furnishing, maker: unknown(), model: unknown(), width: inches(36.0), depth: inches(21.0), height: inches(34.5), weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
    ed::item(job, name, i).unwrap()
}

fn names(job: &mut Job) -> Vec<String> {
    let mut q = job.world.query::<(&Node, &Placed)>();
    let mut v: Vec<String> = q.iter(&job.world).map(|(n, _)| n.name.clone()).collect();
    v.sort();
    v
}

fn set_down(job: &mut Job, level: &str, item: &str, n: usize) -> Vec<String> {
    (0..n).map(|i| ed::place_item(job, level, item, [4.0 + 5.0 * i as f64, 4.0], "0 ft", KNOWN).unwrap()).collect()
}

// ---------------------------------------------------------------- point 2

/// One, two and five set down: every one is called by the new name, each keeping its number; what comes back is how many.
#[test]
fn point_2_every_thing_set_down_is_renamed_and_keeps_its_number() {
    for n in [1usize, 2, 5] {
        let mut job = Job::new("Renamed (not a real job)", "test", "test");
        let level = ed::storey(&mut job, "ground", "0 ft", "8 ft", KNOWN).unwrap();
        let vanity = item(&mut job, "Vanity, 36 in");
        set_down(&mut job, &level, &vanity, n);
        let count = ed::rename_item(&mut job, &vanity, "Vanity, 42 in").unwrap();
        assert_eq!(count, n, "the count renamed");
        let mut want: Vec<String> = (1..=n).map(|i| if i == 1 { "Vanity, 42 in".to_string() } else { format!("Vanity, 42 in {i}") }).collect();
        want.sort();
        assert_eq!(names(&mut job), want, "{n} set down");
    }
}

/// A name a person gave a thing since is left, and counted as left: it is theirs. One that is the item's name with a number is the
/// program's, and is renamed. And the same on the way back: renamed again, the numbers stay.
#[test]
fn point_2_a_name_a_person_gave_is_left_and_counted() {
    let mut job = Job::new("Renamed (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "ground", "0 ft", "8 ft", KNOWN).unwrap();
    let vanity = item(&mut job, "Vanity, 36 in");
    let made = set_down(&mut job, &level, &vanity, 3);
    // The second is called, by hand, something of its own.
    let e = job.find(&made[1]).unwrap();
    job.world.get_mut::<Node>(e).unwrap().name = "Guest bath vanity".to_string();
    let done = ed::rename_item_counted(&mut job, &vanity, "Vanity, 42 in").unwrap();
    assert_eq!((done.set_down, done.renamed), (3, 2));
    assert_eq!(done.kept, ["Guest bath vanity"]);
    assert_eq!(names(&mut job), ["Guest bath vanity", "Vanity, 42 in", "Vanity, 42 in 3"]);
    // Said by the model, with the count of the ones left.
    let said = done.said();
    assert!(said.contains("2 of the 3 set down are called that too"), "{said}");
    assert!(said.contains("1 has a name a person gave it and keeps it: Guest bath vanity"), "{said}");
    // The item's own sentence does not name a command, and says nothing of a thing left when none is.
    let mut job2 = Job::new("Renamed (not a real job)", "test", "test");
    let l2 = ed::storey(&mut job2, "ground", "0 ft", "8 ft", KNOWN).unwrap();
    let v2 = item(&mut job2, "Vanity, 36 in");
    set_down(&mut job2, &l2, &v2, 2);
    let all = ed::rename_item_counted(&mut job2, &v2, "Vanity, 42 in").unwrap();
    assert_eq!(all.said(), "The 2 set down are called that too.");
    assert!(all.kept.is_empty());
    let none = {
        let mut j = Job::new("Renamed (not a real job)", "test", "test");
        let i = item(&mut j, "Vanity, 36 in");
        ed::rename_item_counted(&mut j, &i, "Vanity, 42 in").unwrap()
    };
    assert_eq!((none.set_down, none.renamed, none.said().as_str()), (0, 0, ""));
}

// ---------------------------------------------------------------- point 5

fn ft(typed: &str) -> f64 {
    let (len, unit) = parse_length(typed).unwrap_or_else(|e| panic!("{typed:?} refused: {e}"));
    len.feet(unit).unwrap()
}

/// "12-6" is twelve feet six, as the trades write it; what was read before is read as it was.
#[test]
fn point_5_twelve_hyphen_six_is_twelve_feet_six() {
    assert_eq!(parse_length("12-6").unwrap(), (Len::FtIn([12.0, 6.0]), "ft-in"));
    assert_eq!(parse_length("12-6 1/2").unwrap(), (Len::FtIn([12.0, 6.5]), "ft-in"));
    assert_eq!(parse_length("12-6-1/2").unwrap(), (Len::FtIn([12.0, 6.5]), "ft-in"));
    assert_eq!(parse_length("12-6\"").unwrap(), (Len::FtIn([12.0, 6.0]), "ft-in"));
    assert_eq!(parse_length("6-0").unwrap(), (Len::FtIn([6.0, 0.0]), "ft-in"));
    assert_eq!(parse_length("0-6").unwrap(), (Len::FtIn([0.0, 6.0]), "ft-in"));
    assert_eq!(parse_length("-12-6").unwrap(), (Len::FtIn([-12.0, -6.0]), "ft-in"));
    assert_eq!(ft("12-6"), 12.5);
    assert_eq!(ft("6-0"), 6.0);
    // The same length written the other ways it always could be.
    assert_eq!(parse_length("12'-6\"").unwrap(), (Len::FtIn([12.0, 6.0]), "ft-in"));
    assert_eq!(parse_length("12 ft-6").unwrap(), (Len::FtIn([12.0, 6.0]), "ft-in"));
}

/// What is still refused, and each says what is wrong in words that are true: no commas are mentioned where there are none.
#[test]
fn point_5_what_is_still_refused_says_why() {
    for (typed, says) in [
        ("-6", "has no unit"),                      // a bare minus six is below nothing, and has no unit
        ("6-1/2", "has no unit"),                   // six and a half of what
        ("12-12", "less than 12"),                  // twelve inches is a foot more
        ("12-13", "less than 12"),
        ("12.5-6", "feet as a whole number"),
        ("12-6 ft", "feet after the inches"),            // a range, not a length
        ("12-6 in x", "something after the inches"),
        ("12 6", "12-6"),                           // two numbers with a space: ask for the hyphen or the marks
        ("12 - 6", "12-6"),
        ("12-", "12-6"),
        ("12--6", "12-6"),
    ] {
        let e = parse_length(typed).expect_err(typed);
        assert!(e.contains(says), "{typed:?}: {e}");
        assert!(!e.contains("no commas"), "{typed:?} is refused for commas, and has none: {e}");
    }
    // A comma is still a comma.
    assert!(parse_length("1,200 ft").unwrap_err().contains("has a comma"));
    // The refusal of a thing that is not a length at all is not made to talk of commas either.
    let e = parse_length("abc").unwrap_err();
    assert!(!e.contains("commas"), "{e}");
}

/// A typed 12-6 goes through every field that takes a length: the wall's height, kept in feet and inches, and Known when typed Known.
#[test]
fn point_5_a_field_takes_12_6() {
    let mut job = Job::new("A level (not a real job)", "test", "test");
    let level = ed::storey(&mut job, "ground", "0 ft", "12-6", KNOWN).unwrap();
    let e = job.find(&level).unwrap();
    let l = job.world.get::<Level>(e).unwrap();
    assert_eq!(l.height.value, Some(Len::FtIn([12.0, 6.0])));
}

// ---------------------------------------------------------------- point 8

/// The top bar counts the building. An empty job with a tool chest brought in has nothing of the building to count: the chest's
/// values are the chest's, and are counted apart. The two together are what the one count always gave.
#[test]
fn point_8_the_building_and_the_tool_chest_are_counted_apart() {
    let mut job = Job::new("A chest (not a real job)", "test", "test");
    item(&mut job, "Vanity, 36 in");
    item(&mut job, "Chair");
    let elements = view::elements(&mut job);
    let both = view::tally(&elements);
    let apart = view::tallies(&mut job, &elements);
    assert_eq!(apart.building, (0, 0, 0), "an empty job has nothing of the building to count");
    // Each item shows (`plumbline show`): what it is, its trade and its shape (3 with no mark: Known), its width, depth and height
    // (3 Assumed, from a catalog), its maker and model (2 Unknown). Two items: 6, 6 and 4.
    assert_eq!(apart.chest, (6, 6, 4), "{apart:?}");
    assert_eq!(apart.total(), both);
    // A level and a wall type go to the building; the chest does not change.
    let level = ed::storey(&mut job, "ground", "0 ft", "8 ft", KNOWN).unwrap();
    let wt = ed::wall_type(&mut job, "wall 6 in", "6 in", KNOWN).unwrap();
    ed::walls(&mut job, &level, &wt, &[[0.0, 0.0], [10.0, 0.0]], false, ed::Top::Height("8 ft"), KNOWN).unwrap();
    let elements = view::elements(&mut job);
    let after = view::tallies(&mut job, &elements);
    assert_eq!(after.chest, apart.chest);
    assert!(after.building.0 + after.building.1 + after.building.2 > 0);
    assert_eq!(after.total(), view::tally(&elements));
}
