//! What a placed thing is drawn as: a vanity as a cabinet on its plinth under a
//! worktop, a water closet as a cistern and a bowl, a table as a top on four legs. Not
//! a box of its sizes.
//!
//! Ported from BIMCraft's library of families (`crates/core/src/family.rs`, `library`,
//! line 1975), at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), under
//! its MIT licence (NOTICE). A family there is a list of blocks, each a box (or a round) between
//! formulas in millimetres of the family's Width, Depth and Height. Here are the same
//! blocks in the same order, worked for the thing's own width, depth and height. Where
//! BIMCraft's family has sizes of its own (a WC is 400 by 650 by 800 there and takes no
//! parameters) its blocks are scaled to the thing's sizes, so the form always fills
//! the box the thing is, and never leaves it: the box is what is tested for clashes,
//! and what is drawn must be inside what is tested.
//!
//! Since call 233 there are twenty-seven (the fourteen after the thirteen are ours: appliances and small devices, no BIMCraft family;
//! BIMCraft's library was searched, see PORTMAP.md). Since call 198 the thirteen are rows of `library/shapes.csv` (`shape.rs`), one row a block, each
//! figure a formula in the thing's width, depth and height; `blocks` below works them and nothing else
//! does. The figures in the rows are the ones the code had (held to it, block for block, at three
//! sizes, before it was taken out). What is not ported: a family's parameters, types, formulas, connectors
//! and symbols (`family.rs` is 2,841 lines; this is the forms of ten of its families). A thing
//! here has no parameters: it has its sizes, each with where it came from.
//!
//! Three forms are ours and not BIMCraft's, whose library has no family for them (searched
//! `crates/core/src/family.rs` for sink, tank, heater, boiler, heat pump,
//! condenser and air handler: only a category for "sink", no blocks): a sink as a
//! bowl, two where it is wide enough; a water heater as an upright round tank; an
//! outdoor unit as a cabinet under its fan guard. They follow the same rule: inside
//! the thing's box, and reaching each of its sides.
//!
//! A form stands in the thing's own frame, in feet: x along its width, from the middle;
//! y from its front (less) to its back (more), from the middle; z up from its
//! underside. BIMCraft's y runs from 0 at the wall to minus the depth at the front, so
//! a thing's back is its +y side: the side that goes against a wall.

use serde::{Deserialize, Serialize};

use super::P3;

/// What a block is made of, for whoever draws it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Made {
    Timber,
    Worktop,
    Porcelain,
    Fabric,
    /// Bare metal: a stainless sink, a fan guard.
    Steel,
    /// Painted sheet metal: a water heater's jacket, an outdoor unit's cabinet.
    Enamel,
}

impl Made {
    pub const ALL: [Made; 6] = [Made::Timber, Made::Worktop, Made::Porcelain, Made::Fabric, Made::Steel, Made::Enamel];

    /// The word a shape's table says it by.
    pub fn word(self) -> &'static str {
        match self {
            Made::Timber => "timber",
            Made::Worktop => "worktop",
            Made::Porcelain => "porcelain",
            Made::Fabric => "fabric",
            Made::Steel => "steel",
            Made::Enamel => "enamel",
        }
    }

    /// The material of that word, however it is capitalised.
    pub fn named(word: &str) -> Option<Made> {
        Made::ALL.into_iter().find(|m| m.word().eq_ignore_ascii_case(word.trim()))
    }
}

/// One block of a form: a box from `lo` to `hi`, or, `round`, an upright round that
/// fills that box (a pedestal, the front of a bowl).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    pub lo: P3,
    pub hi: P3,
    pub round: bool,
    pub made: Made,
}

/// Which of BIMCraft's families a thing is drawn as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    BaseCabinet,
    Wardrobe,
    Wc,
    Washbasin,
    Bathtub,
    Table,
    Chair,
    Bed,
    Sofa,
    Desk,
    /// Ours: a sink's bowl, or two.
    Sink,
    /// Ours: an upright round tank (a water heater).
    Tank,
    /// Ours: a heat pump's or a condenser's outdoor unit.
    OutdoorUnit,
    /// Ours (call 233): range.
    Range,
    /// Ours (call 233): refrigerator.
    Refrigerator,
    /// Ours (call 233): dishwasher.
    Dishwasher,
    /// Ours (call 233): laundry machine.
    LaundryMachine,
    /// Ours (call 233): air handler.
    AirHandler,
    /// Ours (call 233): panelboard.
    Panelboard,
    /// Ours (call 233): receptacle.
    Receptacle,
    /// Ours (call 233): switch.
    Switch,
    /// Ours (call 233): ceiling light.
    CeilingLight,
    /// Ours (call 233): recessed light.
    RecessedLight,
    /// Ours (call 233): smoke alarm.
    SmokeAlarm,
    /// Ours (call 233): thermostat.
    Thermostat,
    /// Ours (call 233): register.
    Register,
    /// Ours (call 233): shower base.
    ShowerBase,
}

impl Form {
    pub const ALL: [Form; 27] = [Form::BaseCabinet, Form::Wardrobe, Form::Wc, Form::Washbasin, Form::Bathtub, Form::Table, Form::Chair, Form::Bed, Form::Sofa, Form::Desk, Form::Sink, Form::Tank, Form::OutdoorUnit, Form::Range, Form::Refrigerator, Form::Dishwasher, Form::LaundryMachine, Form::AirHandler, Form::Panelboard, Form::Receptacle, Form::Switch, Form::CeilingLight, Form::RecessedLight, Form::SmokeAlarm, Form::Thermostat, Form::Register, Form::ShowerBase];

    /// What it is called where a person picks what an item is drawn as, and in an item
    /// library's "Drawn as" column.
    pub fn name(self) -> &'static str {
        match self {
            Form::BaseCabinet => "base cabinet",
            Form::Wardrobe => "wardrobe",
            Form::Wc => "water closet",
            Form::Washbasin => "lavatory",
            Form::Bathtub => "bathtub",
            Form::Table => "table",
            Form::Chair => "chair",
            Form::Bed => "bed",
            Form::Sofa => "sofa",
            Form::Desk => "desk",
            Form::Sink => "sink",
            Form::Tank => "tank",
            Form::OutdoorUnit => "outdoor unit",
            Form::Range => "range",
            Form::Refrigerator => "refrigerator",
            Form::Dishwasher => "dishwasher",
            Form::LaundryMachine => "laundry machine",
            Form::AirHandler => "air handler",
            Form::Panelboard => "panelboard",
            Form::Receptacle => "receptacle",
            Form::Switch => "switch",
            Form::CeilingLight => "ceiling light",
            Form::RecessedLight => "recessed light",
            Form::SmokeAlarm => "smoke alarm",
            Form::Thermostat => "thermostat",
            Form::Register => "register",
            Form::ShowerBase => "shower base",
        }
    }

    /// The form of that name, however it is capitalised. None for any other word.
    pub fn named(name: &str) -> Option<Form> {
        let name = name.trim();
        Form::ALL.into_iter().find(|f| f.name().eq_ignore_ascii_case(name))
    }
}

/// What an item says it is drawn as, read: a form by its name; "box", the box of its
/// sizes and nothing else. Anything else is an error that names what there is.
pub const BOX: &str = "box";
pub fn drawn_as(said: &str) -> Result<Option<Form>, String> {
    let said = said.trim();
    if said.eq_ignore_ascii_case(BOX) {
        return Ok(None);
    }
    Form::named(said).map(Some).ok_or_else(|| format!("{said:?} is not something a thing can be drawn as. There are: {BOX}, {}", Form::ALL.map(Form::name).join(", ")))
}

/// The form a thing's name says it has: the earliest word in the name that names one
/// ("Harbor Vanity 36" is a base cabinet, "Water closet" a WC). None when the name
/// names none; the thing is then drawn as the box it is.
pub fn form_of(name: &str) -> Option<Form> {
    const WORDS: [(&str, Form); 34] = [
        ("sink", Form::Sink),
        ("water heater", Form::Tank),
        ("tank", Form::Tank),
        ("heat pump", Form::OutdoorUnit),
        ("condenser", Form::OutdoorUnit),
        ("condensing unit", Form::OutdoorUnit),
        ("outdoor unit", Form::OutdoorUnit),
        ("vanity", Form::BaseCabinet),
        ("base cabinet", Form::BaseCabinet),
        ("cabinet", Form::BaseCabinet),
        ("casework", Form::BaseCabinet),
        ("counter", Form::BaseCabinet),
        ("wardrobe", Form::Wardrobe),
        ("bookcase", Form::Wardrobe),
        ("locker", Form::Wardrobe),
        ("shelving", Form::Wardrobe),
        ("closet", Form::Wardrobe),
        ("water closet", Form::Wc),
        ("toilet", Form::Wc),
        ("wc", Form::Wc),
        ("lavatory", Form::Washbasin),
        ("washbasin", Form::Washbasin),
        ("basin", Form::Washbasin),
        ("bathtub", Form::Bathtub),
        ("tub", Form::Bathtub),
        ("bath", Form::Bathtub),
        ("table", Form::Table),
        ("chair", Form::Chair),
        ("bed", Form::Bed),
        ("sofa", Form::Sofa),
        ("couch", Form::Sofa),
        ("desk", Form::Desk),
        ("credenza", Form::BaseCabinet),
        ("dresser", Form::Wardrobe),
    ];
    let lower = name.to_ascii_lowercase();
    let words: Vec<&str> = lower.split(|c: char| !c.is_ascii_alphabetic()).filter(|w| !w.is_empty()).collect();
    // The earliest place in the name wins, and at one place the longer phrase: a
    // "water closet" is a WC, not a closet.
    let mut best: Option<(usize, usize, Form)> = None;
    for (phrase, form) in WORDS {
        let parts: Vec<&str> = phrase.split(' ').collect();
        if let Some(at) = words.windows(parts.len()).position(|w| w == parts.as_slice()) {
            // "water closet" begins a word before "closet" and so is found first.
            if best.is_none_or(|(was, long, _)| at < was || (at == was && parts.len() > long)) {
                best = Some((at, parts.len(), form));
            }
        }
    }
    best.map(|(_, _, form)| form)
}

/// The blocks of a form for a thing `size` feet wide, deep and high. Each is inside the
/// thing's box, and together they reach every side of it. Worked from the form's rows in
/// `library/shapes.csv` (`shape.rs`); this is what the window calls with the form and the
/// size it has.
pub fn blocks(form: Form, size: P3) -> Vec<Block> {
    super::shape::starter_blocks(form.name(), size).unwrap_or_else(|| panic!("the starter shapes have no rows for {}", form.name()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // call 233: the starter chest grew
    const ALL: [Form; 27] = Form::ALL;

    /// What is drawn is inside what is tested for clashes, and fills it: every block of
    /// every form is inside the thing's box, and together they reach each of its six
    /// sides. At the sizes BIMCraft gives each family, at a made-up vanity's, and at a
    /// thing a few inches each way.
    #[test]
    fn every_form_is_inside_the_things_box_and_fills_it() {
        for form in ALL {
            for size in [[3.0, 1.75, 2.875], [5.0, 2.5, 3.0], [1.5, 2.25, 2.5], [0.4, 0.3, 0.5], [6.5, 3.0, 2.6]] {
                let made = blocks(form, size);
                assert!(!made.is_empty(), "{form:?}");
                let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
                for b in &made {
                    for i in 0..3 {
                        assert!(b.lo[i] < b.hi[i] + 1.0e-12, "{form:?} at {size:?}: a block inside out: {b:?}");
                        (lo[i], hi[i]) = (lo[i].min(b.lo[i]), hi[i].max(b.hi[i]));
                    }
                }
                let want = [[-size[0] / 2.0, size[0] / 2.0], [-size[1] / 2.0, size[1] / 2.0], [0.0, size[2]]];
                for i in 0..3 {
                    assert!(lo[i] >= want[i][0] - 1.0e-9 && hi[i] <= want[i][1] + 1.0e-9, "{form:?} at {size:?}: outside the box on axis {i}: {lo:?} to {hi:?}");
                    assert!((lo[i] - want[i][0]).abs() < 1.0e-9 && (hi[i] - want[i][1]).abs() < 1.0e-9, "{form:?} at {size:?}: does not reach the box's sides on axis {i}: {lo:?} to {hi:?}");
                }
            }
        }
    }

    /// The base cabinet at BIMCraft's own size, 1800 by 600 by 900 mm, block for block
    /// against `family.rs:2113`: a plinth 100 high set 60 back from the front; a
    /// carcass from 100 to 860, 20 back; a worktop 40 thick over the whole.
    #[test]
    fn the_base_cabinet_is_bimcrafts_block_for_block() {
        let mm = |b: &Block| ([b.lo, b.hi].map(|p| p.map(|v| (v * 304.8 * 1000.0).round() / 1000.0)), b.made);
        let made = blocks(Form::BaseCabinet, [1800.0 / 304.8, 600.0 / 304.8, 900.0 / 304.8]);
        // y here runs from the front (-300) to the back (+300): BIMCraft's -600 to 0.
        assert_eq!(mm(&made[0]), ([[-900.0, -240.0, 0.0], [900.0, 300.0, 100.0]], Made::Timber));
        assert_eq!(mm(&made[1]), ([[-900.0, -280.0, 100.0], [900.0, 300.0, 860.0]], Made::Timber));
        assert_eq!(mm(&made[2]), ([[-900.0, -300.0, 860.0], [900.0, 300.0, 900.0]], Made::Worktop));
        assert_eq!(made.len(), 3);
    }

    #[test]
    fn a_name_says_what_form_a_thing_has_or_none() {
        assert_eq!(form_of("Harbor Vanity 36"), Some(Form::BaseCabinet));
        assert_eq!(form_of("Water closet"), Some(Form::Wc));
        assert_eq!(form_of("Water closet 2"), Some(Form::Wc));
        assert_eq!(form_of("Bookcase BK-30"), Some(Form::Wardrobe));
        assert_eq!(form_of("Lavatory"), Some(Form::Washbasin));
        assert_eq!(form_of("Bathtub, 5 ft"), Some(Form::Bathtub));
        // The three that are ours.
        assert_eq!(form_of("Kitchen sink, double bowl"), Some(Form::Sink));
        assert_eq!(form_of("Water heater, 50 gal"), Some(Form::Tank));
        assert_eq!(form_of("Heat pump, outdoor unit"), Some(Form::OutdoorUnit));
        // A double bowl has a wall between its bowls; a bar sink has one bowl.
        assert_eq!(blocks(Form::Sink, [33.0 / 12.0, 21.625 / 12.0, 8.375 / 12.0]).len(), 6);
        assert_eq!(blocks(Form::Sink, [15.0 / 12.0, 15.0 / 12.0, 6.0 / 12.0]).len(), 5);
        // Each form has a name of its own, read back however it is capitalised; an
        // item is drawn as one of them, or as its box.
        for form in Form::ALL {
            assert_eq!(Form::named(&form.name().to_uppercase()), Some(form));
            assert_eq!(drawn_as(form.name()), Ok(Some(form)));
        }
        assert_eq!(drawn_as("Box"), Ok(None));
        assert!(drawn_as("gazebo").unwrap_err().contains("\"gazebo\" is not something a thing can be drawn as. There are: box, base cabinet"));
        // A thing whose name names none of them is the box it is.
        assert_eq!(form_of("AH-24 Air Handler"), None);
        assert_eq!(form_of("Ceiling register, 4 x 10"), None);
        assert_eq!(form_of("Stable door"), None);
    }
}
