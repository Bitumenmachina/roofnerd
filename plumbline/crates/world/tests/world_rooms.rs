use plumbline_world::sample::sample;
use plumbline_world::value::Status;
use plumbline_world::*;

#[allow(dead_code)]
mod common;
use common::{measure_of, path_named, ft, named, piping, piece, thing, walled, wall, room, four_walls, figures_of, priced_of, extension_of, priced_sample, boxed};

// ---------------------------------------------------------------- the re-base on BIMCraft: rooms

#[test]
fn a_room_is_the_space_its_walls_enclose_measured_inside_their_faces_and_to_their_centres() {
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let room_named = |job: &mut Job, name: &str| job.children().get(&first).unwrap().get(name).unwrap().clone();
    // The sample's first floor is 40 ft by 28 ft to the centres of its outside walls,
    // which are 6 in thick. A hall partition, 4 1/2 in thick, runs east-west at 16 ft.
    //
    // The living room is all of it south of the hall: 40 by 16 to centre lines, 640 SF.
    // Inside the faces it is 40 − 0.25 − 0.25 = 39.5 ft by 16 − 0.25 − 0.1875 = 15.5625 ft:
    // 614.7188 SF, and 2 × (39.5 + 15.5625) = 110.125 LF round.
    let living = room_named(&mut job, "living room");
    assert_eq!(figures_of(&mut job, &living), [Some(614.7188), Some(640.0), Some(110.125)]);
    // The bath is between partitions at 14 and 24 ft, the hall, and the wet wall (6 in) at
    // 22 ft: 10 by 6, 60 SF to centre lines. Inside: 10 − 0.1875 − 0.1875 = 9.625 by
    // 6 − 0.1875 − 0.25 = 5.5625: 53.5391 SF, 30.375 LF round.
    let bath = room_named(&mut job, "bath");
    assert_eq!(figures_of(&mut job, &bath), [Some(53.5391), Some(60.0), Some(30.375)]);
    // The kitchen: 14 by 12 is 168; inside 13.5625 by 11.5625 is 156.8164; 50.25 round.
    let kitchen = room_named(&mut job, "kitchen");
    assert_eq!(figures_of(&mut job, &kitchen), [Some(156.8164), Some(168.0), Some(50.25)]);
    // The five rooms' areas to centre lines are the whole floor: 640 + 168 + 60 + 60 + 192.
    let gross: f64 = ["living room", "kitchen", "bath", "utility", "office"].iter().map(|n| room_named(&mut job, n)).collect::<Vec<_>>().iter().map(|r| measure_of(&mut job, r, "gross_area").value.unwrap()).sum();
    assert_eq!(gross, 1120.0);
    let totals = plumbline_world::building::storey_totals(&mut job);
    assert_eq!(totals[0].floors.value, Some(1120.0));
    // And inside their walls they come to 614.7188 + 156.8164 + 53.5391 + 52.9375 + 179.9414
    // = 1,057.9532 SF: the rest is under the walls. (A total is the sum of the figures
    // as each is shown, to four decimals, so it is what adding the lines up gives.)
    assert_eq!((totals[0].rooms.value, totals[0].rooms.members, totals[0].rooms.status), (Some(1057.9532), 5, Status::Known));
    // A room's figure names the walls it is measured to, and is Known when they are.
    let d = measure_of(&mut job, &bath, "floor_area");
    assert_eq!(d.status, Status::Known);
    assert_eq!(d.inputs.len(), 5, "the room and its four walls: {:?}", d.inputs);

    // Move the wet wall a foot north, to 23 ft, and the bath grows by 9.625 SF and the
    // utility room shrinks by as much, with nothing typed on either room.
    let wet = room_named(&mut job, "wet wall");
    let e = job.find(&wet).unwrap();
    job.world.get_mut::<plumbline_world::Wall>(e).unwrap().line = Val::known([[14.0, 23.0], [24.0, 23.0]], "ft", "test");
    job.derive();
    assert_eq!(measure_of(&mut job, &bath, "floor_area").value, Some(63.1641));
    let utility = room_named(&mut job, "utility");
    assert_eq!(measure_of(&mut job, &utility, "floor_area").value, Some(43.3125));
    // Say the wet wall does not bound rooms and the bath is both rooms' space: 10 by 12
    // to centre lines. The utility room, placed after it, is in a space that is taken.
    job.world.get_mut::<plumbline_world::Wall>(e).unwrap().room_bounding = false;
    job.derive();
    assert_eq!(measure_of(&mut job, &bath, "gross_area").value, Some(120.0));
    let lost = measure_of(&mut job, &utility, "floor_area");
    assert_eq!((lost.value, lost.status), (None, Status::Unknown));
    assert_eq!(lost.note, "utility has no area yet: its point is in the space bath has. Part the two with a wall or a room line, and each gets its own.");
}

#[test]
fn a_room_that_is_not_closed_or_is_in_a_wall_has_no_area_and_says_why() {
    let (mut job, s, t) = walled();
    // Three walls of a 20 ft by 12 ft room: open to the west.
    wall(&mut job, &s, &t, "south", [0.0, 0.0], [20.0, 0.0], 9.0);
    wall(&mut job, &s, &t, "east", [20.0, 0.0], [20.0, 12.0], 9.0);
    wall(&mut job, &s, &t, "north", [20.0, 12.0], [0.0, 12.0], 9.0);
    let r = room(&mut job, &s, "den", [10.0, 6.0]);
    job.derive();
    for name in ["floor_area", "gross_area", "perimeter"] {
        let d = measure_of(&mut job, &r, name);
        assert_eq!((d.value, d.status, d.note.as_str()), (None, Status::Unknown, "no loop of walls closes round its point"), "{name}");
    }
    // Close it. 20 by 12 to centre lines is 240 SF; inside 19.5 by 11.5 is 224.25, 62 LF round.
    wall(&mut job, &s, &t, "west", [0.0, 12.0], [0.0, 0.0], 9.0);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(224.25), Some(240.0), Some(62.0)]);
    // A point in the body of a wall is not in a room at its faces.
    let in_wall = room(&mut job, &s, "in the wall", [10.0, 0.1]);
    job.derive();
    assert_eq!(measure_of(&mut job, &in_wall, "floor_area").note, "its point is inside a wall");
    // A point outside the walls is in no room.
    let outside = room(&mut job, &s, "outside", [30.0, 6.0]);
    job.derive();
    assert_eq!(measure_of(&mut job, &outside, "gross_area").note, "no loop of walls closes round its point");
    // A wall drawn by somebody's guess makes the room it bounds Assumed.
    let west = path_named(&mut job, "west");
    let e = job.find(&west).unwrap();
    job.world.get_mut::<plumbline_world::Wall>(e).unwrap().line = Val::assumed([[0.0, 12.0], [0.0, 0.0]], "ft", "scaled off a sketch");
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Assumed);
    // A wall of the storey that cannot be placed (nobody has its height) leaves every room
    // on the storey with no figure: whether it closes a room off is not known.
    job.world.get_mut::<plumbline_world::Wall>(e).unwrap().height = Some(Val::unknown("ft", "not on the section"));
    job.derive();
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!(d.value, None);
    assert_eq!(d.note, "west cannot be placed (its height is not known), so what bounds the rooms of this level is not known");
    // A room on another storey is still measured, but is Assumed, and says what it assumes.
    let upper = job.add("job-0001", Kind::Storey, "upper").unwrap();
    job.put(&upper, plumbline_world::Level { elevation: ft(10.0), height: ft(9.0), computation_height: None }).unwrap();
    four_walls(&mut job, &upper, &t, 20.0, 12.0);
    let up = room(&mut job, &upper, "loft", [10.0, 6.0]);
    job.derive();
    let d = measure_of(&mut job, &up, "floor_area");
    assert_eq!((d.value, d.status), (Some(224.25), Status::Assumed));
    assert_eq!(d.note, "assumes west does not bound this room: it cannot be placed (its height is not known)");
}

#[test]
fn rooms_close_at_corners_whatever_line_a_wall_was_drawn_on_and_an_island_is_taken_out() {
    use plumbline_world::{Location, Wall};
    // A 10 ft by 10 ft square drawn clockwise on the walls' outside faces. The exterior
    // is to the left of the way a wall is drawn, so drawn clockwise the walls stand inside
    // the square. They are 6 in thick: 9 ft by 9 ft inside, 81 SF, 36 LF round; to their
    // centre lines 9.5 by 9.5, 90.25 SF. The centre lines are run on to meet at the corners.
    let (mut job, s, t) = walled();
    for (name, a, b) in [("west", [0.0, 0.0], [0.0, 10.0]), ("north", [0.0, 10.0], [10.0, 10.0]), ("east", [10.0, 10.0], [10.0, 0.0]), ("south", [10.0, 0.0], [0.0, 0.0])] {
        let w = job.add(&s, Kind::Wall, name).unwrap();
        job.put(&w, Wall { location: Location::ExteriorFinish, ..Wall::tall(named(&t), Val::known([a, b], "ft", "test"), ft(9.0)) }).unwrap();
    }
    let r = room(&mut job, &s, "box", [5.0, 5.0]);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(81.0), Some(90.25), Some(36.0)]);

    // A 20 ft square of walls with a 4 ft square of walls standing free inside it, all to
    // centre lines. To centre lines the room is 400 − 16 = 384 SF. Inside the faces it is
    // 19.5² less the island's outside, 4.5²: 380.25 − 20.25 = 360 SF, and it is
    // 4 × 19.5 + 4 × 4.5 = 96 LF round, the island's faces included.
    let (mut job, s, t) = walled();
    four_walls(&mut job, &s, &t, 20.0, 20.0);
    for (name, a, b) in [("i south", [8.0, 8.0], [12.0, 8.0]), ("i east", [12.0, 8.0], [12.0, 12.0]), ("i north", [12.0, 12.0], [8.0, 12.0]), ("i west", [8.0, 12.0], [8.0, 8.0])] {
        wall(&mut job, &s, &t, name, a, b, 9.0);
    }
    let r = room(&mut job, &s, "hall", [3.0, 3.0]);
    let shaft = room(&mut job, &s, "shaft", [10.0, 10.0]);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(360.0), Some(384.0), Some(96.0)]);
    // The space inside the island is a room of its own: 4 by 4, and 3.5 by 3.5 inside.
    assert_eq!(figures_of(&mut job, &shaft), [Some(12.25), Some(16.0), Some(14.0)]);

    // A wall that stops short of where a room's walls are found does not bound it. A 3 ft
    // knee wall across a 20 ft by 12 ft room: at the usual 4 ft the room runs past it; told
    // to look at 2 ft, the storey finds two rooms.
    let (mut job, s, t) = walled();
    four_walls(&mut job, &s, &t, 20.0, 12.0);
    wall(&mut job, &s, &t, "knee wall", [10.0, 0.0], [10.0, 12.0], 3.0);
    let r = room(&mut job, &s, "west half", [5.0, 6.0]);
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "gross_area").value, Some(240.0));
    job.put(&s, plumbline_world::Level { elevation: ft(0.0), height: ft(10.0), computation_height: Some(ft(2.0)) }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "gross_area").value, Some(120.0));
    // Inside: 10 − 0.25 − 0.25 = 9.5 by 11.5 is 109.25.
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(109.25));
}

#[test]
fn a_room_in_the_file_has_to_be_under_a_storey_and_have_a_point_that_makes_sense() {
    use plumbline_world::building::check_room;
    use plumbline_world::Room;
    assert!(check_room("r", Kind::Room, &Room { point: Val::known([1.0, 1.0], "ft", "test"), number: None }).is_ok());
    assert!(check_room("r", Kind::Room, &Room { point: Val::known([1.0, 1.0], "", "test"), number: None }).is_err());
    assert!(check_room("r", Kind::Room, &Room { point: Val::known([1.0, 1.0], "m", "test"), number: None }).is_err());
    assert!(check_room("r", Kind::Wall, &Room { point: Val::known([1.0, 1.0], "ft", "test"), number: None }).is_err());
    assert!(check_room("r", Kind::Room, &Room { point: Val::unknown("ft", "not placed yet"), number: None }).is_ok());
    // A room whose point nobody has is listed with no figure, not left out.
    let (mut job, s, t) = walled();
    four_walls(&mut job, &s, &t, 20.0, 12.0);
    let r = job.add(&s, Kind::Room, "somewhere").unwrap();
    job.put(&r, Room { point: Val::unknown("ft", "not placed yet"), number: None }).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").note, "its point is not known");
    // A room under the job, not under a storey, does not load.
    let mut job = plumbline_world::sample::house();
    let text = file::save(&mut job);
    let (first, living) = (path_named(&mut job, "first floor"), path_named(&mut job, "living room"));
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    for entry in doc["data"].as_array_mut().unwrap() {
        let path = entry["path"].as_str().unwrap().to_string();
        if path == first {
            entry["children"].as_object_mut().unwrap().remove("living room");
        } else if path == "job-0001" {
            entry["children"].as_object_mut().unwrap().insert("living room".into(), living.clone().into());
        }
    }
    let err = file::load(&doc.to_string()).err().expect("a room under the job");
    assert!(err.contains("a room is a space on a level"), "{err}");
}

// ---------------------------------------------------------------- D4: macro to micro

#[test]
fn a_job_is_priced_from_allowances_and_from_its_lines_and_the_two_are_set_against_each_other() {
    let mut job = priced_sample();
    // Design phase. The south plane is 419.2627 SF at $28.00: $11,739.3556, $11,739.36.
    let a = priced_of(&mut job, "plane-0003", "allowance");
    assert_eq!((a.value, a.unit.as_str(), a.status), (Some(11739.36), "$", Status::Known));
    // The headwall is 12 LF at $40.00: $480.00. It lies on the shed, whose pitch is
    // Assumed, so its length is no surer than that, and neither is a dollar priced on it.
    let h = priced_of(&mut job, "headwall-0021", "allowance");
    assert_eq!((h.value, h.status), (Some(480.0), Status::Assumed));
    // An assumed input shows in the money: the shed's pitch is Assumed, so its 98.9545 SF
    // is, and so is its $2,770.73 (98.9545 × 28 = 2,770.726).
    let shed = priced_of(&mut job, "plane-0017", "allowance");
    assert_eq!((shed.value, shed.status), (Some(2770.73), Status::Assumed));

    // Detail. The south plane's one line is its 419.2627 SF at $31.50: $13,206.77505,
    // $13,206.78. The headwall's flashing is 12 LF × 24 in ÷ 12 = 24 SF at $18.25: $438.00.
    assert_eq!(extension_of(&mut job, "plane-0003", "Copper standing seam, installed").value, Some(13206.78));
    assert_eq!(extension_of(&mut job, "headwall-0021", "Copper headwall flashing").value, Some(438.0));
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(13206.78));

    // Rolled back up: over the same figure the allowance was priced on. The plane's detail
    // is $13,206.78 ÷ 419.2627 SF = $31.50 a SF. The headwall's flashing is priced by the
    // SF and rolls up to the LF its allowance is in: $438.00 ÷ 12 LF = $36.50 a LF.
    let rate = priced_of(&mut job, "plane-0003", "detail_rate");
    assert_eq!((rate.value, rate.unit.as_str()), (Some(31.5), "$/SF"));
    let rate = priced_of(&mut job, "headwall-0021", "detail_rate");
    assert_eq!((rate.value, rate.unit.as_str()), (Some(36.5), "$/LF"));

    // Reconciled. Plane: 13,206.78 − 11,739.36 = 1,467.42 over, 12.5 %. Headwall:
    // 438.00 − 480.00 = 42.00 under, −8.75 %.
    assert_eq!(priced_of(&mut job, "plane-0003", "difference").value, Some(1467.42));
    assert_eq!(priced_of(&mut job, "plane-0003", "difference_percent").value, Some(12.5));
    assert_eq!(priced_of(&mut job, "headwall-0021", "difference").value, Some(-42.0));
    assert_eq!(priced_of(&mut job, "headwall-0021", "difference_percent").value, Some(-8.75));

    // The shed has an allowance and a line with no price. Its detail is not whole: it has
    // no detail figure, no rate and no difference, and each says which line and why. It is
    // not rolled up as $0.
    for name in ["detail", "detail_rate", "difference", "difference_percent"] {
        let d = priced_of(&mut job, "plane-0017", name);
        assert_eq!((d.value, d.status), (None, Status::Unknown), "{name}");
        assert!(d.note.contains("\"Copper standing seam, installed\": the line has no price (no price entered)"), "{name}: {}", d.note);
    }

    // The job. Allowances: 11,739.36 + 2,770.73 + 480.00 = 14,990.09 over three elements,
    // of which the shed's 2,770.73 and the headwall's 480.00, 3,250.73, rest on the
    // assumed pitch.
    let p = plumbline_world::view::pricing(&mut job);
    assert_eq!((p.allowance.value, p.allowance.members, p.allowance.status, p.allowance.assumed), (Some(14990.09), 3, Status::Assumed, 3250.73));
    assert!(p.allowance.left_out.is_empty());
    // Detail: 13,206.78 + 438.00 = 13,644.78 over the two elements whose detail is whole.
    // Four are left out, each named with why: the shed and the three planes nobody priced.
    // The headwall's 438.00 rests on the assumed pitch.
    assert_eq!((p.detail.value, p.detail.members, p.detail.status, p.detail.assumed), (Some(13644.78), 2, Status::Assumed, 438.0));
    assert_eq!(p.detail.left_out.len(), 4);
    assert!(p.detail.left_out.iter().all(|(_, why)| why.contains("the detail is not whole")), "{:?}", p.detail.left_out);
    // Like for like, over the two elements that have both: allowances 11,739.36 + 480.00
    // = 12,219.36 against detail 13,644.78.
    // Each side carries its own status (the headwall's share of each is Assumed).
    assert_eq!(p.both, Some(((12219.36, Status::Assumed), (13644.78, Status::Assumed), Status::Assumed, 2)));
    // What could be priced and is not, by kind: it is said, not left out of sight.
    assert!(p.not_priced.contains(&(Kind::Eave, 5)), "{:?}", p.not_priced);
    assert!(p.not_priced.contains(&(Kind::Hip, 4)), "{:?}", p.not_priced);

    // The geometry still drives both. Make the south plane 9:12: its plan area is 375 SF and
    // its slope factor 1.25, so 468.75 SF. Allowance 468.75 × 28 = 13,125.00; detail
    // 468.75 × 31.50 = 14,765.625, $14,765.63; and the rate it rolls up to is still $31.50.
    job.set("plane-0003", "pitch", Some("9:12"), Status::Known, "test").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "plane-0003", "allowance").value, Some(13125.0));
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").value, Some(14765.63));
    assert_eq!(priced_of(&mut job, "plane-0003", "detail_rate").value, Some(31.5));
    // And it saves and loads to the same bytes, with nothing stale.
    let text = file::save(&mut job);
    let (mut again, report) = file::load(&text).unwrap();
    assert!(report.stale.is_empty(), "{:?}", report.stale);
    assert_eq!(file::save(&mut again), text);
}

#[test]
fn money_is_never_silently_zero_and_a_flag_reaches_every_dollar_it_touches() {
    let mut job = sample();
    // No price anywhere: no element has a detail, no total has a figure, and none is $0.
    let p = plumbline_world::view::pricing(&mut job);
    assert_eq!((p.allowance.value, p.allowance.members), (None, 0));
    assert_eq!((p.detail.value, p.detail.members), (None, 0));
    assert_eq!(p.both, None);
    let e = extension_of(&mut job, "plane-0003", "Copper standing seam, installed");
    assert_eq!((e.value, e.status), (None, Status::Unknown));
    assert_eq!(e.note, "the line has no price (no price entered)");
    // A price of nothing is a price: $0.00 is said when somebody entered it.
    job.set("plane-0003", "unit_cost", Some("0"), Status::Known, "furnished by the owner").unwrap();
    job.derive();
    let e = extension_of(&mut job, "plane-0003", "Copper standing seam, installed");
    assert_eq!((e.value, e.status), (Some(0.0), Status::Known));
    // A price too small to make a cent of the whole line is refused, not written as $0.00:
    // 24 SF at $0.0001 is $0.0024.
    job.set("headwall-0021", "unit_cost", Some("0.0001"), Status::Known, "test").unwrap();
    job.derive();
    let e = extension_of(&mut job, "headwall-0021", "Copper headwall flashing");
    assert_eq!(e.value, None);
    assert!(e.note.contains("less than half a cent"), "{}", e.note);
    // On the south plane, whose geometry is all Known: an assumed price makes an assumed
    // extension, detail, rate and difference; an assumed allowance makes an assumed
    // difference though the detail is Known.
    job.set("plane-0003", "unit_cost", Some("31.50"), Status::Assumed, "last year's price").unwrap();
    job.set("plane-0003", "allowance", Some("28"), Status::Known, "made-up budget rate").unwrap();
    job.derive();
    for name in ["detail", "detail_rate", "difference", "difference_percent"] {
        assert_eq!(priced_of(&mut job, "plane-0003", name).status, Status::Assumed, "{name}");
    }
    assert_eq!(priced_of(&mut job, "plane-0003", "allowance").status, Status::Known);
    job.set("plane-0003", "unit_cost", Some("31.50"), Status::Known, "quote").unwrap();
    job.set("plane-0003", "allowance", Some("28"), Status::Assumed, "a guess").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "plane-0003", "detail").status, Status::Known);
    assert_eq!(priced_of(&mut job, "plane-0003", "difference").status, Status::Assumed);
    job.set("headwall-0021", "unit_cost", Some("18.25"), Status::Known, "quote").unwrap();
    // An allowance nobody has: no amount, and it says so. A stretch-out nobody has: the
    // line has no quantity, so no extension, and the element's detail is not whole.
    job.set("headwall-0021", "allowance", None, Status::Unknown, "not budgeted yet").unwrap();
    job.set("headwall-0021", "stretch_out", None, Status::Unknown, "detail not drawn").unwrap();
    job.derive();
    let a = priced_of(&mut job, "headwall-0021", "allowance");
    assert_eq!(a.value, None);
    assert!(a.note.contains("the allowance has no rate (not budgeted yet)"), "{}", a.note);
    let d = priced_of(&mut job, "headwall-0021", "detail");
    assert!(d.note.contains("the detail is not whole") && d.note.contains("the line has no quantity to price"), "{}", d.note);
    // An allowance of $0 with a detail: the difference is the detail, and no percentage
    // of nothing is made up.
    job.set("headwall-0021", "allowance", Some("0"), Status::Known, "included elsewhere").unwrap();
    job.set("headwall-0021", "stretch_out", Some("24 in"), Status::Known, "detail").unwrap();
    job.derive();
    assert_eq!(priced_of(&mut job, "headwall-0021", "difference").value, Some(438.0));
    let pct = priced_of(&mut job, "headwall-0021", "difference_percent");
    assert_eq!(pct.value, None);
    assert!(pct.note.contains("not a share of nothing"), "{}", pct.note);
}

#[test]
fn a_price_is_entered_for_the_unit_the_thing_is_priced_in_and_nothing_else() {
    use plumbline_world::units::parse_price;
    // The ways a price is typed.
    assert_eq!(parse_price("4.25", "$/SF"), Ok(4.25));
    assert_eq!(parse_price("$4.25", "$/SF"), Ok(4.25));
    assert_eq!(parse_price("$4.25/SF", "$/SF"), Ok(4.25));
    assert_eq!(parse_price("4.25 $/SF", "$/SF"), Ok(4.25));
    assert_eq!(parse_price("$1,250.00/sheet", "$/sheet"), Ok(1250.0));
    assert!(parse_price("$4.25/LF", "$/SF").unwrap_err().contains("a price does not carry from one unit to another"));
    assert!(parse_price("-4", "$/SF").is_err());
    assert!(parse_price("four", "$/SF").is_err());
    assert!(parse_price("", "$/SF").is_err());
    let mut job = sample();
    // A plane is priced by the SF and a headwall by the LF: the other is refused.
    assert!(job.set("plane-0003", "allowance", Some("$28/LF"), Status::Known, "test").is_err());
    assert!(job.set("headwall-0021", "allowance", Some("$40/SF"), Status::Known, "test").is_err());
    // A roof, a job and a condition have nothing a rate could be on.
    let err = job.set("roof-0002", "allowance", Some("5"), Status::Known, "test").unwrap_err();
    assert!(err.contains("cannot carry an allowance"), "{err}");
    // A value needs a source, as everywhere.
    assert!(job.set("plane-0003", "allowance", Some("28"), Status::Known, "").is_err());
    // A line's price is for what the line is priced in. The headwall's line is in SF.
    assert!(job.set("headwall-0021", "unit_cost", Some("$18.25/LF"), Status::Known, "test").is_err());
    job.set("headwall-0021", "unit_cost", Some("$18.25/SF"), Status::Known, "test").unwrap();
    // An allowance on a kind it cannot be on, or in the wrong unit, does not load.
    job.set("plane-0003", "allowance", Some("28"), Status::Known, "test").unwrap();
    job.derive();
    let text = file::save(&mut job);
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let plane = doc["data"].as_array_mut().unwrap().iter_mut().find(|e| e["path"] == "plane-0003").unwrap();
    assert_eq!(plane["attributes"]["pl::allowance"]["rate"]["unit"], "$/SF");
    plane["attributes"]["pl::allowance"]["rate"]["unit"] = "$/LF".into();
    let err = file::load(&doc.to_string()).err().expect("an allowance by the LF on a plane");
    assert!(err.contains("a roof plane is priced by its surface area, in SF"), "{err}");
    // A price somebody typed into the saved figures is not trusted: it is worked out again.
    let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    let plane = doc["data"].as_array_mut().unwrap().iter_mut().find(|e| e["path"] == "plane-0003").unwrap();
    plane["attributes"]["pl::priced"]["allowance"]["value"] = 1.0.into();
    let (mut job, report) = file::load(&doc.to_string()).unwrap();
    assert_eq!(report.stale, ["plane-0003"]);
    assert_eq!(priced_of(&mut job, "plane-0003", "allowance").value, Some(11739.36));
}

// ---------------------------------------------------------------- the ninth judgment

#[test]
fn a_room_is_no_surer_than_a_height_that_decides_whether_a_wall_bounds_it() {
    use plumbline_world::Wall;
    // The box alone: 19.5 by 9.5 inside is 185.25 SF, 200 to centre lines, 58 LF round.
    let (mut job, s, t, r) = boxed();
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(185.25), Some(200.0), Some(58.0)]);
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Known);
    // A partition across the middle. With it the room is the west half: 9.5 by 9.5, 90.25.
    let p = wall(&mut job, &s, &t, "partition", [10.0, 0.0], [10.0, 10.0], 10.0);
    job.derive();
    assert_eq!(figures_of(&mut job, &r), [Some(90.25), Some(100.0), Some(38.0)]);
    let set = |job: &mut Job, change: &dyn Fn(&mut Wall)| {
        let e = job.find(&p).unwrap();
        change(&mut job.world.get_mut::<Wall>(e).unwrap());
        job.derive();
    };
    // Say its base is assumed to be 5 ft up. That is above the 4 ft the rooms find their
    // walls at, so it does not bound the room, which is 185.25 SF: but only if the
    // assumption holds. The figure is Assumed, and says which wall it turns on.
    set(&mut job, &|w| w.base_offset = Some(Val::assumed(Len::One(5.0), "ft", "guessed")));
    for name in ["floor_area", "gross_area", "perimeter"] {
        let d = measure_of(&mut job, &r, name);
        assert_eq!(d.status, Status::Assumed, "{name}");
        // (Named by its path as well as its name since the tenth judgment: two walls can share a name.)
        assert_eq!(d.note, format!("its outline counts on partition ({p}), whose height is assumed"), "{name}");
        assert!(d.inputs.contains(&format!("{p}#pl::wall")), "{name}: {:?}", d.inputs);
    }
    assert_eq!(figures_of(&mut job, &r), [Some(185.25), Some(200.0), Some(58.0)]);
    // Assumed to be 1 ft up, it does bound the room: 90.25, and Assumed for the same reason.
    set(&mut job, &|w| w.base_offset = Some(Val::assumed(Len::One(1.0), "ft", "guessed")));
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status), (Some(90.25), Status::Assumed));
    // A height that is assumed does the same: 3 ft tall, it stops short of 4 ft.
    set(&mut job, &|w| {
        w.base_offset = None;
        w.height = Some(Val::assumed(Len::One(3.0), "ft", "guessed"));
    });
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status), (Some(185.25), Status::Assumed));
    // Known to be 3 ft tall, the same wall leaves the room 185.25 and Known.
    set(&mut job, &|w| w.height = Some(ft(3.0)));
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status, d.note.as_str()), (Some(185.25), Status::Known, ""));
    // A wall with an assumed height that could not bound this room whatever its height
    // (it stands outside the box) does not make the room Assumed.
    let far = wall(&mut job, &s, &t, "garden wall", [30.0, 0.0], [30.0, 10.0], 10.0);
    let e = job.find(&far).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().height = Some(Val::assumed(Len::One(6.0), "ft", "guessed"));
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").status, Status::Known);
    // A partition entered on a storey below, whose elevation is assumed: whether it rises
    // into this storey rests on that.
    let (mut job, _s, t, r) = boxed();
    let below = job.add("job-0001", Kind::Storey, "basement").unwrap();
    job.put(&below, plumbline_world::Level { elevation: Val::assumed(Len::One(-10.0), "ft", "scaled off a section"), height: ft(10.0), computation_height: None }).unwrap();
    wall(&mut job, &below, &t, "basement partition", [10.0, 0.0], [10.0, 10.0], 10.0);
    job.derive();
    let d = measure_of(&mut job, &r, "floor_area");
    assert_eq!((d.value, d.status), (Some(185.25), Status::Assumed));
    assert!(d.note.contains("its outline counts on basement partition (wall-") && d.note.contains("), whose height is assumed"), "{}", d.note);
}

#[test]
fn a_wall_with_nothing_entered_and_a_point_in_a_wall_leave_no_figure_standing() {
    // A wall element with no line entered is a wall nobody can place. The rooms of its
    // storey have no figure, and say which wall.
    let (mut job, s, _t, r) = boxed();
    let bare = job.add(&s, Kind::Wall, "wall with nothing entered").unwrap();
    job.derive();
    for name in ["floor_area", "gross_area", "perimeter"] {
        let d = measure_of(&mut job, &r, name);
        assert_eq!((d.value, d.status), (None, Status::Unknown), "{name}");
        assert_eq!(d.note, "wall with nothing entered cannot be placed (it has no wall line entered), so what bounds the rooms of this level is not known", "{name}");
    }
    // And such a file loads the same way: the wall is not dropped.
    let text = file::save(&mut job);
    let (mut again, _) = file::load(&text).unwrap();
    assert_eq!(measure_of(&mut again, &r, "floor_area").value, None);
    job.remove(&bare).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(185.25));
    // A point in the body of a wall is in no room, at the faces or at the centre lines.
    let in_wall = room(&mut job, &s, "in the wall", [0.1, 5.0]);
    job.derive();
    for name in ["floor_area", "gross_area", "perimeter"] {
        let d = measure_of(&mut job, &in_wall, name);
        assert_eq!((d.value, d.note.as_str()), (None, "its point is inside a wall"), "{name}");
    }
    // And it does not take the space from the room that is there.
    assert_eq!(measure_of(&mut job, &r, "gross_area").value, Some(200.0));
}

#[test]
fn a_partition_stops_at_the_face_of_a_wall_that_does_not_bound_rooms() {
    use plumbline_world::Wall;
    // The box, a wall N across the middle that does not bound rooms, and a partition P
    // from N's line east to the east wall, at y = 5. With N not bounding, the room is the
    // whole box less P's body. P stops at N's face (x = 10.25), as BIMCraft joins it, and
    // at the east wall's face (x = 19.75): 9.5 ft of a 6 in wall, 4.75 SF.
    // Floor: 185.25 − 4.75 = 180.5 SF. Round: 58, plus P's two faces (19) and its west end
    // (0.5), less the 0.5 of the east wall's face P covers: 77 LF. To centre lines: 200.
    let (mut job, s, t, _r) = boxed();
    let n = wall(&mut job, &s, &t, "N", [10.0, 0.0], [10.0, 10.0], 10.0);
    let e = job.find(&n).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().room_bounding = false;
    wall(&mut job, &s, &t, "P", [10.0, 5.0], [20.0, 5.0], 10.0);
    let east = room(&mut job, &s, "east side", [15.0, 7.5]);
    job.derive();
    // (The room placed first, at (5, 5), has the space: the east room is in it.)
    assert_eq!(measure_of(&mut job, &east, "floor_area").note, "east side has no area yet: its point is in the space room has. Part the two with a wall or a room line, and each gets its own.");
    let first = path_named(&mut job, "room");
    assert_eq!(figures_of(&mut job, &first), [Some(180.5), Some(200.0), Some(77.0)]);
    // The same with N only 3 ft tall and bounding rooms where it stands: it stops short of
    // the 4 ft the rooms are found at, so it bounds nothing, and P still stops at its face.
    job.world.get_mut::<Wall>(e).unwrap().room_bounding = true;
    job.world.get_mut::<Wall>(e).unwrap().height = Some(ft(3.0));
    job.derive();
    assert_eq!(figures_of(&mut job, &first), [Some(180.5), Some(200.0), Some(77.0)]);
}

#[test]
fn walls_join_within_a_hundredth_of_a_foot_and_no_further() {
    use plumbline_world::building::check_wall;
    use plumbline_world::Wall;
    // The west wall stops 0.005 ft short of the corner: within the 0.01 ft two points are
    // told apart by, so it is the corner, and the room closes at the centre lines: 200 SF.
    let (mut job, s, t) = walled();
    for (name, a, b) in [("south", [0.0, 0.0], [20.0, 0.0]), ("east", [20.0, 0.0], [20.0, 10.0]), ("north", [20.0, 10.0], [0.0, 10.0])] {
        wall(&mut job, &s, &t, name, a, b, 10.0);
    }
    let west = wall(&mut job, &s, &t, "west", [0.0, 10.0], [0.0, 0.005], 10.0);
    let r = room(&mut job, &s, "room", [5.0, 5.0]);
    job.derive();
    assert_eq!(measure_of(&mut job, &r, "gross_area").value, Some(200.0));
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(185.25));
    // 0.02 ft short, it is not the corner. The centre lines do not meet, so there is no
    // area to centre lines, and it says so. The walls' bodies still overlap at the corner
    // (each is 3 in either side of its line), so the floor inside their faces is closed:
    // 185.25 SF.
    let e = job.find(&west).unwrap();
    job.world.get_mut::<Wall>(e).unwrap().line = Val::known([[0.0, 10.0], [0.0, 0.02]], "ft", "test");
    job.derive();
    let gross = measure_of(&mut job, &r, "gross_area");
    assert_eq!((gross.value, gross.note.as_str()), (None, "no loop of walls closes round its point"));
    assert_eq!(measure_of(&mut job, &r, "floor_area").value, Some(185.25));
    // A wall no longer than that tolerance is not a wall.
    let short = Wall::tall(named("type-0003"), Val::known([[3.0, 3.0], [3.005, 3.0]], "ft", "test"), ft(8.0));
    assert!(check_wall("w", Kind::Wall, &short).unwrap_err().contains("a wall is longer than the 0.01 ft"));
    let long_enough = Wall::tall(named("type-0003"), Val::known([[3.0, 3.0], [3.02, 3.0]], "ft", "test"), ft(8.0));
    assert!(check_wall("w", Kind::Wall, &long_enough).is_ok());
    // In inches too: 0.06 in is 0.005 ft.
    let short = Wall::tall(named("type-0003"), Val::known([[36.0, 36.0], [36.06, 36.0]], "in", "test"), ft(8.0));
    assert!(check_wall("w", Kind::Wall, &short).is_err());
}

#[test]
fn an_openings_area_is_its_size_and_where_it_is_matters_to_the_net() {
    use plumbline_world::Opening;
    let mut job = plumbline_world::sample::house();
    let first = path_named(&mut job, "first floor");
    let south = job.children().get(&first).unwrap().get("south wall").unwrap().clone();
    let window = job.children().get(&south).unwrap().get("west window").unwrap().clone();
    let e = job.find(&window).unwrap();
    let mut o = job.world.get::<Opening>(e).unwrap().clone();
    // A window whose sill nobody has. It is still 5 by 5: the wall's openings still come
    // to 70.01 SF, Known. Whether it fits cannot be checked, so the wall has no net area,
    // and the reason is the position, not the area.
    o.sill = Val::unknown("ft", "not on the elevation");
    job.put(&window, o.clone()).unwrap();
    job.derive();
    let open = measure_of(&mut job, &south, "opening_area");
    assert_eq!((open.value, open.status), (Some(70.01), Status::Known));
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!(net.value, None);
    assert_eq!(net.note, format!("where {window} is in the wall is not known, so whether it fits cannot be checked"));
    // A sill somebody assumed: the openings' area is still Known; the net is Assumed.
    o.sill = Val::assumed(Len::One(2.5), "ft", "typical");
    job.put(&window, o).unwrap();
    job.derive();
    assert_eq!(measure_of(&mut job, &south, "opening_area").status, Status::Known);
    let net = measure_of(&mut job, &south, "net_area");
    assert_eq!((net.value, net.status), (Some(329.99), Status::Assumed));
}

#[test]
fn an_open_connector_is_listed_where_its_run_would_be() {
    use plumbline_world::building::takeoff;
    use plumbline_world::{ConnectorKind::Pipe, Domain, InSystem, SystemInfo};
    let (mut job, s, two, ..) = piping();
    let system = |job: &mut Job, name: &str, classification: &str| {
        let p = job.add("job-0001", Kind::System, name).unwrap();
        job.put(&p, SystemInfo { discipline: plumbline_world::Discipline::Plumbing, classification: classification.into() }).unwrap();
        p
    };
    let (cold, _drain) = (system(&mut job, "Cold water", "Domestic Cold Water"), system(&mut job, "Drain", "Sanitary"));
    // A water closet, on the cold water system as fixtures are. Its cold inlet is met by
    // a pipe. Nothing is on its waste outlet.
    let wc = thing(&mut job, &s, Kind::Fixture, "wc", [5.0, 5.0, 0.0], None, &[(Pipe, Some(Domain::Cold), [-0.5, 0.0, 1.0]), (Pipe, Some(Domain::Waste), [0.0, 1.0, 0.0])]);
    job.put(&wc, InSystem(named(&cold))).unwrap();
    let supply = piece(&mut job, &s, Kind::Pipe, "supply", &two, [4.5, 5.0, 1.0], [4.5, 0.0, 1.0]);
    job.put(&supply, InSystem(named(&cold))).unwrap();
    job.derive();
    let t = takeoff(&mut job);
    let of = |name: &str| t.iter().find(|x| x.name == name).unwrap().joined.open.iter().map(|o| o.said.clone()).collect::<Vec<_>>();
    // The drain is told its fixture is not connected; the cold water is told only of the
    // far end of its own pipe.
    assert_eq!(of("Drain"), ["wc: its connection 2"]);
    assert_eq!(of("Cold water"), ["supply: its second end"]);
}

// ---- D5: the estimate sheet, the recap, materials and suppliers ----
//
// The job is `sample::priced()`: the sample roof with a made-up estimate on it. Every
// price, rate and supplier in it is made up. The figures it rests on, from the sample:
// south plane 419.2627 SF, east 251.5576 SF, west 251.5576 SF, shed 98.9545 SF (Assumed:
// its pitch is), headwall 12 LF (Assumed: it lies on the shed).
//
// Worked by hand, each step from the figure the step before shows (four places):
//   south  panels     419.2627 x 1.10 = 461.1890 SF x 1 lb/SF = 461.1890 lb x 9.00  = 4,150.70
//          underlay   419.2627 x 0.90                                               =   377.34
//          fabricate  419.2627 / 100 = 4.1926 hr x 60                               =   251.56
//          install    419.2627 / 25  = 16.7705 hr x 72                              = 1,207.48
//   east   panels     251.5576 x 1.10 = 276.7134 lb x 9.00                          = 2,490.42
//          underlay   251.5576 x 0.90                                               =   226.40
//          fabricate  251.5576 / 100 = 2.5156 hr x 60                               =   150.94
//          install    251.5576 / 25  = 10.0623 hr x 72                              =   724.49
//   west   by a sub   251.5576 x 41.00                                              = 10,313.86
//   shed   panels     98.9545 x 1.25 lb/SF = 123.6931 lb x 9.00                     = 1,113.24 (Assumed)
//   headwall flashing 12 x 24 in = 24 SF x 1.25 = 30 lb x 9.00                      =   270.00 (Assumed)
//          install    12 x 0.25 hr/LF = 3 hr x 72                                   =   216.00 (Assumed)
//   job    foreman    5 x 8 hr = 40 hr x 95                                         = 3,800.00
//   north  one line, no price, no class: in no row.
//
//   Material    4,150.70 + 377.34 + 2,490.42 + 226.40 + 1,113.24 + 270.00 =  8,628.10
//   Labor       1,207.48 + 724.49 + 216.00                                =  2,147.97   29.8328 hr
//   Fabrication 251.56 + 150.94                                           =    402.50    6.7082 hr
//   Supervision                                                           =  3,800.00   40 hr
//   Subcontract                                                           = 10,313.86
//   Job cost                                                              = 25,292.43
