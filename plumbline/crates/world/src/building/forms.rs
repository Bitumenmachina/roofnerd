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
//! What is not ported: a family's parameters, types, formulas, connectors and symbols
//! (`family.rs` is 2,841 lines; this is the forms of ten of its families). A thing
//! here has no parameters: it has its sizes, each with where it came from.
//!
//! A form stands in the thing's own frame, in feet: x along its width, from the middle;
//! y from its front (less) to its back (more), from the middle; z up from its
//! underside. BIMCraft's y runs from 0 at the wall to minus the depth at the front, so
//! a thing's back is its +y side: the side that goes against a wall.

use super::P3;

/// What a block is made of, for whoever draws it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Made {
    Timber,
    Worktop,
    Porcelain,
    Fabric,
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
}

/// The form a thing's name says it has: the earliest word in the name that names one
/// ("Harbor Vanity 36" is a base cabinet, "Water closet" a WC). None when the name
/// names none; the thing is then drawn as the box it is.
pub fn form_of(name: &str) -> Option<Form> {
    const WORDS: [(&str, Form); 27] = [
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
/// thing's box, and together they reach every side of it.
pub fn blocks(form: Form, size: P3) -> Vec<Block> {
    const MM: f64 = 304.8;
    // The thing's sizes in millimetres, as BIMCraft's formulas have them.
    let (w, d, h) = (size[0] * MM, size[1] * MM, size[2] * MM);
    let mut out: Vec<Block> = Vec::new();
    // A block as BIMCraft writes it: x from the middle, y from 0 at the back to -depth
    // at the front, z up, in millimetres. `c` is a fixed size from BIMCraft (a 100 mm
    // plinth) kept no bigger than a share of the thing, so a small thing is not all
    // plinth.
    let c = |fixed: f64, of: f64, share: f64| fixed.min(of * share);
    let mut put = |x: [f64; 2], y: [f64; 2], z: [f64; 2], round: bool, made: Made| {
        let (y0, y1) = (y[0].min(y[1]) + d / 2.0, y[0].max(y[1]) + d / 2.0);
        out.push(Block { lo: [x[0].min(x[1]) / MM, y0 / MM, z[0].min(z[1]) / MM], hi: [x[0].max(x[1]) / MM, y1 / MM, z[0].max(z[1]) / MM], round, made });
    };
    let half = w / 2.0;
    match form {
        // `family.rs:2113`: its plinth, set back for toes; its carcass; its worktop over the front.
        Form::BaseCabinet => {
            let (plinth, top, toe, lip) = (c(100.0, h, 0.14), c(40.0, h, 0.06), c(60.0, d, 0.14), c(20.0, d, 0.05));
            put([-half, half], [-d + toe, 0.0], [0.0, plinth], false, Made::Timber);
            put([-half, half], [-d + lip, 0.0], [plinth, h - top], false, Made::Timber);
            put([-half, half], [-d, 0.0], [h - top, h], false, Made::Worktop);
        }
        // `family.rs:2099`: a wardrobe on its plinth.
        Form::Wardrobe => {
            let (plinth, inset) = (c(80.0, h, 0.06), c(50.0, w.min(d), 0.1));
            put([-half, half], [-d, 0.0], [plinth, h], false, Made::Timber);
            put([-half + inset, half - inset], [-d + inset, 0.0], [0.0, plinth], false, Made::Timber);
        }
        // `family.rs:2127`: its cistern against the wall, its bowl. 400 by 650 by 800
        // there; scaled to the thing.
        Form::Wc => {
            let (kx, ky, kz) = (w / 400.0, d / 650.0, h / 800.0);
            put([-200.0 * kx, 200.0 * kx], [-180.0 * ky, 0.0], [380.0 * kz, 800.0 * kz], false, Made::Porcelain);
            put([-160.0 * kx, 160.0 * kx], [-380.0 * ky, -150.0 * ky], [0.0, 400.0 * kz], false, Made::Porcelain);
            put([-180.0 * kx, 180.0 * kx], [-650.0 * ky, -290.0 * ky], [0.0, 400.0 * kz], true, Made::Porcelain);
        }
        // `family.rs:2147`: a washbasin on its pedestal.
        Form::Washbasin => {
            let (bowl, r) = (c(160.0, h, 0.25), c(90.0, w.min(d), 0.22));
            put([-half, half], [-d, 0.0], [h - bowl, h], false, Made::Porcelain);
            let back = c(150.0, d, 0.4);
            put([-r, r], [-back - r, -back + r], [0.0, h - bowl], true, Made::Porcelain);
        }
        // `family.rs:2169`: its bottom, and its four sides round the tub. (Its length
        // there is the thing's width here: the long way, along the wall.)
        Form::Bathtub => {
            let (bottom, side) = (c(120.0, h, 0.25), c(60.0, w.min(d), 0.12));
            put([-half, half], [-d, 0.0], [0.0, bottom], false, Made::Porcelain);
            put([-half, half], [-side, 0.0], [bottom, h], false, Made::Porcelain);
            put([-half, half], [-d, -d + side], [bottom, h], false, Made::Porcelain);
            put([-half, -half + side], [-d + side, -side], [bottom, h], false, Made::Porcelain);
            put([half - side, half], [-d + side, -side], [bottom, h], false, Made::Porcelain);
        }
        // `family.rs:2012`: a top on four legs, each 50 in from the top's edges.
        Form::Table => {
            let (top, leg, inset) = (c(40.0, h, 0.08), c(60.0, w.min(d), 0.12), c(50.0, w.min(d), 0.08));
            put([-half, half], [-d, 0.0], [h - top, h], false, Made::Timber);
            for (east, north) in [(true, true), (true, false), (false, true), (false, false)] {
                let x = if east { [half - inset - leg, half - inset] } else { [-half + inset, -half + inset + leg] };
                let y = if north { [-inset - leg, -inset] } else { [-d + inset, -d + inset + leg] };
                put(x, y, [0.0, h - top], false, Made::Timber);
            }
        }
        // `family.rs:2028`: its seat, its back over the seat's rear edge, and its legs.
        // The seat is 450 of 850 high there.
        Form::Chair => {
            let (seat, thick, leg) = (h * 450.0 / 850.0, c(40.0, h, 0.06), c(40.0, w.min(d), 0.12));
            put([-half, half], [-d, 0.0], [seat - thick, seat], false, Made::Timber);
            put([-half, half], [-thick, 0.0], [seat, h], false, Made::Timber);
            for (east, north) in [(true, true), (true, false), (false, true), (false, false)] {
                let x = if east { [half - leg, half] } else { [-half, -half + leg] };
                let y = if north { [-leg, 0.0] } else { [-d, -d + leg] };
                put(x, y, [0.0, seat - thick], false, Made::Timber);
            }
        }
        // `family.rs:2046`: its base, its mattress, its headboard, and a pillow, two on
        // a double. 450 to the top of the mattress and 900 to the top of the headboard
        // there; the thing's height is its headboard's.
        Form::Bed => {
            let k = h / 900.0;
            let (base, mattress, head, edge) = (300.0 * k, 450.0 * k, c(60.0, d, 0.05), c(20.0, w, 0.03));
            put([-half, half], [-d, 0.0], [0.0, base], false, Made::Timber);
            put([-half + edge, half - edge], [-d + edge, -head], [base, mattress], false, Made::Fabric);
            put([-half, half], [-head, 0.0], [0.0, h], false, Made::Timber);
            let (near, far, tall, gap) = (c(100.0, d, 0.06), c(500.0, d, 0.26), 120.0 * k, c(80.0, w, 0.08));
            if w > 1200.0 {
                put([-half + gap, -gap / 2.0], [-far, -near], [mattress, mattress + tall], false, Made::Fabric);
                put([gap / 2.0, half - gap], [-far, -near], [mattress, mattress + tall], false, Made::Fabric);
            } else {
                put([-half + gap, half - gap], [-far, -near], [mattress, mattress + tall], false, Made::Fabric);
            }
        }
        // `family.rs:2064`: its seat between its arms, its back. Seat 420, arms 600 and
        // back 800 high there; arms 180 wide and the back 200 deep.
        Form::Sofa => {
            let k = h / 800.0;
            let (arm, back) = (c(180.0, w, 0.12), c(200.0, d, 0.25));
            put([-half + arm, half - arm], [-d, -back], [0.0, 420.0 * k], false, Made::Fabric);
            put([-half, half], [-back, 0.0], [0.0, h], false, Made::Fabric);
            put([-half, -half + arm], [-d, -back], [0.0, 600.0 * k], false, Made::Fabric);
            put([half - arm, half], [-d, -back], [0.0, 600.0 * k], false, Made::Fabric);
        }
        // `family.rs:2078`: its top on two side panels, a modesty panel between them.
        Form::Desk => {
            let (top, panel, inset) = (c(30.0, h, 0.06), c(25.0, w, 0.04), c(20.0, d, 0.04));
            put([-half, half], [-d, 0.0], [h - top, h], false, Made::Timber);
            put([-half, -half + panel], [-d + inset, -inset], [0.0, h - top], false, Made::Timber);
            put([half - panel, half], [-d + inset, -inset], [0.0, h - top], false, Made::Timber);
            put([-half + panel, half - panel], [-c(60.0, d, 0.12), -c(40.0, d, 0.08)], [h * 0.4, h - top], false, Made::Timber);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Form; 10] = [Form::BaseCabinet, Form::Wardrobe, Form::Wc, Form::Washbasin, Form::Bathtub, Form::Table, Form::Chair, Form::Bed, Form::Sofa, Form::Desk];

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
        // A thing whose name names none of them is the box it is.
        assert_eq!(form_of("AH-24 Air Handler"), None);
        assert_eq!(form_of("Ceiling register, 4 x 10"), None);
        assert_eq!(form_of("Stable door"), None);
    }
}
