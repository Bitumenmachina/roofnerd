//! A trade's view of the building (call 177): one button each at the head of the Show
//! list. A trade is a setting of the switches that are already there and not a second
//! state: pressing one moves the switches below it, where they can be read and changed,
//! and "All" puts them as they were before the first trade was pressed.

use plumbline_world::Kind;

use crate::model::{Doc, Show, Snapshot, Tone};

/// The buttons, in the order they are laid out.
pub const TRADES: [&str; 8] = ["All", "Foundation", "Framing", "Outside", "Plumbing", "HVAC", "Electrical", "Inside"];

/// The level a person looks into for the inside of the house: the lowest one above the
/// ground that has a wall on it, else the lowest that has anything.
fn level_inside(snap: &Snapshot) -> Option<String> {
    let mut levels: Vec<&plumbline_world::building::StoreyTotals> = snap.storeys.iter().filter(|s| s.elevation.is_some()).collect();
    levels.sort_by(|a, b| a.elevation.unwrap_or(0.0).total_cmp(&b.elevation.unwrap_or(0.0)));
    let walled = |s: &&plumbline_world::building::StoreyTotals| snap.drawn.iter().any(|p| p.kind == Kind::Wall && p.storey.as_ref() == Some(&s.path));
    levels.iter().find(|s| s.elevation.unwrap_or(0.0) >= -0.5 && walled(s)).or_else(|| levels.first()).map(|s| s.path.clone())
}

/// Set the switches for a trade, and say what was set.
pub fn press(doc: &mut Doc, snap: &Snapshot, trade: &'static str) {
    if trade == "All" {
        // As they were before the first trade was pressed; where none was, as the job opens.
        let back = doc.before_trade.take().unwrap_or(Show { sizes: false, ..opening(&doc.show) });
        let off_levels = back.off_levels.clone();
        let said = if doc.trade.is_none() { "All: every switch as the job opens." } else { "All: every switch as it was before a trade was pressed." };
        doc.show = Show { off_levels, ..back };
        doc.trade = None;
        doc.redraw();
        doc.say(Tone::Done, said);
        return;
    }
    if doc.before_trade.is_none() {
        doc.before_trade = Some(doc.show.clone());
    }
    let s = &mut doc.show;
    // What every trade begins from: the whole house, nothing alone, nothing taken off.
    s.storey = None;
    s.architecture = true;
    s.roof = true;
    s.ground = true;
    s.finishes = true;
    s.framing = false;
    s.ghost = false;
    s.sizes = false;
    s.mechanical = true;
    s.electrical = true;
    s.plumbing = true;
    let said: String;
    match trade {
        "Foundation" => {
            s.ground = false;
            s.ghost = true;
            s.sizes = true;
            s.mechanical = false;
            s.electrical = false;
            s.plumbing = false;
            said = "Foundation: the ground taken away, the footings and foundation walls solid and the house above them shown as glass, sizes on.".into();
        }
        "Framing" => {
            s.framing = true;
            s.finishes = false;
            s.ghost = true;
            s.sizes = true;
            s.mechanical = false;
            s.electrical = false;
            s.plumbing = false;
            said = "Framing: the walls as their frames, with the joists, rafters, beams and posts, the rest of the house shown as glass, sizes on.".into();
        }
        "Outside" => {
            said = "Outside: the house as it will be built.".into();
        }
        "Plumbing" | "HVAC" | "Electrical" => {
            s.ghost = true;
            s.sizes = true;
            s.mechanical = trade == "HVAC";
            s.electrical = trade == "Electrical";
            s.plumbing = trade == "Plumbing";
            said = format!("{trade}: the rest of the house shown as glass, sizes on.");
        }
        _ => {
            s.roof = false;
            s.storey = level_inside(snap);
            said = "Inside: the roof off and the level looked into, with its finishes.".into();
        }
    }
    doc.trade = Some((trade, doc.show.clone()));
    doc.redraw();
    doc.say(Tone::Done, said);
}

/// The switches as a job opens.
fn opening(now: &Show) -> Show {
    Show { architecture: true, roof: true, mechanical: true, electrical: true, plumbing: true, ghost: false, storey: None, certainty: false, clashes: now.clashes, ground: true, framing: false, finishes: true, demolition: now.demolition, existing: now.existing, off_levels: now.off_levels.clone(), off_systems: now.off_systems.clone(), away: now.away, sizes: false }
}
