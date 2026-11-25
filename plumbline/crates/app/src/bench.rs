//! The bench (DECISIONS.md call 195, step A): one item looked at alone.
//!
//! The house, its sheets, its runs and its clashes are put away; the item stands at the
//! middle of a floor of its own, lined every foot, drawn by the same code that draws it
//! when it is set down (its form, its material); its width, depth and height are
//! dimensioned beside it, each in the words its panel uses and in the colour of how sure
//! it is. It is a way of looking: nothing here is saved and nothing here works a number
//! out. The model gives the size (`Item::size_ft`), the words (`units::show_length`) and
//! who follows (`editing::measure`); the window draws them.
//!
//! One piece of state says the window is on the bench: `Doc::bench`. It holds the item,
//! by path, and the view that was left, so that "Back to the house" and Esc fly back to it.
//! A button or a key asks (`Doc::bench_ask`); the one system that holds the camera
//! (`main::keys`) answers, so that the view always flies and never jumps.

use bevy::prelude::*;
use plumbline_world::building::forms::Block;
use plumbline_world::building::shape::Piece;
use plumbline_world::building::{forms, shape, Placement, Shape};
use plumbline_world::units;
use plumbline_world::value::Status;
use plumbline_world::{Item, Job};

use crate::model::{Doc, Snapshot, Tone};
use crate::scene::{Orbit, HOME_YAW};

/// How far the eye looks down on the bench when it opens.
pub const OPENING_PITCH: f32 = 0.42;
/// How far a dimension stands off the item, in feet.
const STAND_OFF: f32 = 0.7;

/// The view that was left when the bench was entered.
#[derive(Clone, Debug, PartialEq)]
pub struct Left {
    pub plan: bool,
    pub walk: bool,
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub home: (Vec3, f32),
}

/// The window is on the bench: the item (by path) and the view that was left.
#[derive(Clone, Debug, PartialEq)]
pub struct Bench {
    pub item: String,
    pub left: Left,
}

/// What a button or a key asked for; answered where the camera is held.
#[derive(Clone, Debug, PartialEq)]
pub enum Ask {
    /// Put this item on the bench (from nothing, or from another item).
    Enter(String),
    /// Go back to the house, to the view that was left.
    Leave,
}

/// One dimension: a line with ends, and the size in the words the item's panel uses.
#[derive(Clone, Debug)]
pub struct Dim {
    /// "width", "depth" or "height".
    pub what: &'static str,
    pub a: Vec3,
    pub b: Vec3,
    /// The way the end strokes run (a unit vector, across the line).
    pub tick: Vec3,
    pub words: String,
    pub status: Status,
}

/// Where everything on the bench stands, in the view's frame (x east, y up, z south), the
/// item's middle over the origin and its underside on the floor.
#[derive(Clone, Debug)]
pub struct Layout {
    /// The item's size in feet: width, depth, height.
    pub size: [f64; 3],
    /// Half the floor's width (east-west) and depth (north-south), in whole feet.
    pub half: [f32; 2],
    pub dims: Vec<Dim>,
    /// What the view looks at, and how far off it stands to frame the item with room round it.
    pub target: Vec3,
    pub distance: f32,
}

/// The item as a thing set down would be drawn: the same record the model gives a placed
/// thing, so that the one piece of code that draws those draws this. Its form is what the
/// item says, or what its name says.
pub fn placement(job: &mut Job, path: &str, origin: [f64; 3]) -> Result<Placement, String> {
    let e = job.find(path).ok_or_else(|| format!("{path} is not in the job"))?;
    let item = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{path} is not an item"))?;
    let name = job.world.get::<plumbline_world::Node>(e).map(|n| n.name.clone()).unwrap_or_else(|| path.to_string());
    let (size, status, _) = item.size_ft()?;
    let form = match item.form.as_deref() {
        Some(said) => forms::drawn_as(said).ok().flatten(),
        None => forms::form_of(&name),
    };
    Ok(Placement { path: path.to_string(), name, kind: item.category, storey: None, system: None, discipline: item.category.discipline(), shape: Shape::Box { at: origin, size, angle: 0.0 }, status, form, tag: None, stack: false })
}


/// What an item is drawn as, for the bench's "Shape": the shape's name, whether it is the job's own, its blocks as the
/// table writes them, and each block worked at the item's sizes by the model (`Shape::worked_block`; the window
/// works nothing out). None for an item drawn as a plain box, or that has no size to work them at.
#[derive(Clone, Debug)]
pub struct DrawnShape {
    pub name: String,
    pub own: bool,
    pub pieces: Vec<Piece>,
    pub worked: Vec<Option<Block>>,
    /// The item's width, depth and height, in feet.
    pub size: [f64; 3],
}

/// The shape an item is drawn as: what it says, else what its name says; the job's own shapes first (`shape::find`).
pub fn drawn_shape(job: &mut Job, path: &str) -> Option<DrawnShape> {
    let e = job.find(path)?;
    let item = job.world.get::<Item>(e).cloned()?;
    let name = job.world.get::<plumbline_world::Node>(e).map(|n| n.name.clone())?;
    let said = match item.form.as_deref().map(str::trim) {
        Some(said) if said.eq_ignore_ascii_case(forms::BOX) => return None,
        Some(said) => said.to_string(),
        None => forms::form_of(&name)?.name().to_string(),
    };
    let found = shape::find(&job.shapes, &said)?;
    let own = job.shapes.iter().any(|s| s.name.eq_ignore_ascii_case(&found.name));
    let (size, _, _) = item.size_ft().ok()?;
    let worked = (0..found.blocks.len()).map(|n| found.worked_block(n, size).ok().flatten()).collect();
    Some(DrawnShape { name: found.name.clone(), own, pieces: found.blocks.clone(), worked, size })
}

/// The blocks of every thing set down that is drawn as a shape of the job's own, by its path, and of the item on the
/// bench the same way (by the item's path): the model's `Shape::worked` at each one's size. A starter shape is a `Form`
/// and is drawn from that.
pub fn own_blocks(job: &mut Job, drawn: &[Placement], bench: Option<&str>) -> std::collections::HashMap<String, Vec<Block>> {
    let mut out = std::collections::HashMap::new();
    if job.shapes.is_empty() {
        return out;
    }
    let mut take = |job: &mut Job, key: &str, item: &str, size: [f64; 3]| {
        let Some(said) = job.find(item).and_then(|e| job.world.get::<Item>(e)).and_then(|it| it.form.clone()) else { return };
        if let Some(found) = job.shapes.iter().find(|s| s.name.eq_ignore_ascii_case(said.trim())) {
            if let Ok(blocks) = found.worked(size) {
                out.insert(key.to_string(), blocks);
            }
        }
    };
    for p in drawn.iter().filter(|p| p.form.is_none() && !p.kind.is_framing()) {
        let Shape::Box { size, .. } = &p.shape else { continue };
        let item = job.find(&p.path).and_then(|e| job.world.get::<plumbline_world::Placed>(e)).and_then(|c| c.item.clone());
        if let Some(item) = item {
            take(job, &p.path, &item, *size);
        }
    }
    if let Some(item) = bench {
        if let Some(size) = job.find(item).and_then(|e| job.world.get::<Item>(e)).and_then(|it| it.size_ft().ok()).map(|s| s.0) {
            take(job, item, item, size);
        }
    }
    out
}

/// The twelve edges of a block, in the view's frame, for the item standing with its middle over the origin and its underside
/// on the floor: a block's own frame is x along its width from the middle, y from its middle to the back, z up from its foot.
pub fn block_edges(b: &Block) -> Vec<(Vec3, Vec3)> {
    let corner = |i: usize| {
        let (x, y, z) = (if i & 1 == 0 { b.lo[0] } else { b.hi[0 ] }, if i & 2 == 0 { b.lo[1] } else { b.hi[1] }, if i & 4 == 0 { b.lo[2] } else { b.hi[2] });
        Vec3::new(x as f32, z as f32, -y as f32)
    };
    let mut out = Vec::new();
    for i in 0..8usize {
        for bit in [1usize, 2, 4] {
            if i & bit == 0 {
                out.push((corner(i), corner(i | bit)));
            }
        }
    }
    out
}

/// The floor, the dimensions and the framing for the item.
pub fn layout(job: &mut Job, path: &str) -> Result<Layout, String> {
    let e = job.find(path).ok_or_else(|| format!("{path} is not in the job"))?;
    let item = job.world.get::<Item>(e).cloned().ok_or_else(|| format!("{path} is not an item"))?;
    let (size, _, _) = item.size_ft()?;
    let (w, d, h) = (size[0] as f32, size[1] as f32, size[2] as f32);
    let g = STAND_OFF;
    let words = |v: &plumbline_world::Val<units::Len>| v.value.as_ref().map(|len| units::show_length(len, &v.unit)).unwrap_or_default();
    // The item stands with its front to the south (its width east and west), its middle over the origin and its
    // underside on the floor. The width runs along the floor in front of it, the depth along the floor at its
    // west side, and the height up a pole at the front corner on the other side from the depth: from the view
    // the bench opens at, none of the three is over the item or over another.
    let dims = vec![
        Dim { what: "width", a: Vec3::new(-w / 2.0, 0.01, d / 2.0 + g), b: Vec3::new(w / 2.0, 0.01, d / 2.0 + g), tick: Vec3::Z, words: words(&item.width), status: item.width.status },
        Dim { what: "depth", a: Vec3::new(-w / 2.0 - g, 0.01, -d / 2.0), b: Vec3::new(-w / 2.0 - g, 0.01, d / 2.0), tick: Vec3::X, words: words(&item.depth), status: item.depth.status },
        Dim { what: "height", a: Vec3::new(w / 2.0 + g, 0.0, d / 2.0 + g), b: Vec3::new(w / 2.0 + g, h, d / 2.0 + g), tick: Vec3::X, words: words(&item.height), status: item.height.status },
    ];
    // A floor a little larger than the item, to whole feet, with its dimensions on it.
    let margin = (0.3 * w.max(d)).max(g + 0.35);
    let half = [(w / 2.0 + margin).ceil(), (d / 2.0 + margin).ceil()];
    // The eye stands off far enough to take in the item and its dimensions with room round them.
    let across = 0.5 * ((w + 2.0 * g).powi(2) + (d + g).powi(2) + h * h).sqrt();
    let distance = (across / (crate::scene::LENS / 2.0).tan() * 1.8).max(6.0);
    // The eye looks a little above the middle, so that the item stands low in the window, clear of the message that is
    // said over the top of the model when the bench opens.
    Ok(Layout { size, half, dims, target: Vec3::new(0.0, h / 2.0 + 0.2 * across, g / 2.0), distance })
}

/// The three sizes in the words the panel uses, each with how sure it is: "36 in wide
/// (Assumed), 21 in deep (Assumed) and 34 1/2 in high (Assumed)".
pub fn said(job: &mut Job, path: &str) -> String {
    let Some(item) = job.find(path).and_then(|e| job.world.get::<Item>(e)).cloned() else { return String::new() };
    let certainty = |s: Status| match s {
        Status::Known => "Known",
        Status::Assumed => "Assumed",
        Status::Unknown => "Unknown",
    };
    let part = |v: &plumbline_world::Val<units::Len>, adjective: &str, noun: &str| match &v.value {
        Some(len) => format!("{} {adjective} ({})", units::show_length(len, &v.unit), certainty(v.status)),
        None => format!("no {noun} known"),
    };
    format!("{}, {} and {}", part(&item.width, "wide", "width"), part(&item.depth, "deep", "depth"), part(&item.height, "high", "height"))
}

/// Put the item on the bench. From the house the view that is left is kept; from another
/// item it is the one kept already. Whatever was in hand is let go of.
pub fn enter(doc: &mut Doc, orbit: &mut Orbit, snap: &Snapshot, item: &str) {
    let is_item = doc.job.find(item).is_some_and(|e| doc.job.world.get::<Item>(e).is_some());
    if !is_item {
        crate::tools::cant(doc, "put it on the bench", format!("{item} is not an item"));
        doc.touch();
        return;
    }
    // What was in hand is let go of, as a click elsewhere lets go of it; a value half typed is said to be let go.
    doc.letting_go(None);
    if doc.entry.take().is_some() {
        doc.form.clear();
        doc.measuring = None;
    }
    doc.draw.tool = None;
    doc.draw.picked.clear();
    doc.placing = None;
    doc.choosing = false;
    doc.marking = None;
    doc.together.clear();
    doc.selected = Some(item.to_string());
    if doc.bench.as_ref().is_none_or(|b| b.item != item) {
        doc.shape_block = None;
        doc.shaping = None;
    }
    // The view that is left: kept once, from the house.
    let left = match &doc.bench {
        Some(b) => b.left.clone(),
        None => {
            let left = Left { plan: doc.plan, walk: doc.walk, target: orbit.target, yaw: orbit.yaw, pitch: orbit.pitch, distance: orbit.distance, home: orbit.home };
            // The bench is looked at in 3D, from outside: a plan or a walk is left by the road every change of view takes.
            crate::set_view(doc, orbit, snap, false, false);
            left
        }
    };
    doc.bench = Some(Bench { item: item.to_string(), left });
    let name = doc.job.find(item).and_then(|e| doc.job.world.get::<plumbline_world::Node>(e)).map(|n| n.name.clone()).unwrap_or_else(|| item.to_string());
    match layout(&mut doc.job, item) {
        Ok(l) => {
            orbit.target = l.target;
            orbit.distance = l.distance;
            orbit.home = (l.target, l.distance);
            let sizes = said(&mut doc.job, item);
            doc.say(Tone::Done, format!("On the bench: {name}, {sizes}. Nothing of the house is drawn here. Type a size on its panel and everything set down from it follows. Back to the house, or Esc, goes back to where you were."));
        }
        Err(why) => {
            let (target, distance) = (Vec3::new(0.0, 1.5, 0.0), 10.0);
            orbit.target = target;
            orbit.distance = distance;
            orbit.home = (target, distance);
            doc.say(Tone::Warned, format!("On the bench: {name}, which cannot be drawn yet: {why}. Type what is missing on its panel. Back to the house, or Esc, goes back to where you were."));
        }
    }
    orbit.yaw = HOME_YAW;
    orbit.pitch = OPENING_PITCH;
    doc.redraw();
}

/// Back to the house, to the view that was left.
pub fn leave(doc: &mut Doc, orbit: &mut Orbit, snap: &Snapshot) {
    let Some(bench) = doc.bench.take() else { return };
    doc.shape_block = None;
    doc.shaping = None;
    let left = bench.left;
    if left.plan || left.walk {
        crate::set_view(doc, orbit, snap, left.plan, left.walk);
    }
    orbit.target = left.target;
    orbit.yaw = left.yaw;
    orbit.pitch = left.pitch;
    orbit.distance = left.distance;
    orbit.home = left.home;
    doc.say(Tone::Done, "Back to the house.");
    doc.redraw();
}

/// What a button or a key asked for, answered.
pub fn answer(doc: &mut Doc, orbit: &mut Orbit, snap: &Snapshot) {
    match doc.bench_ask.take() {
        Some(Ask::Enter(item)) => enter(doc, orbit, snap, &item),
        Some(Ask::Leave) => leave(doc, orbit, snap),
        None => {}
    }
    // An item taken out of the job while it is on the bench leaves nothing to look at: the bench is left, and what was
    // said of the item going is what stays said.
    if doc.bench.as_ref().is_some_and(|b| doc.job.find(&b.item).is_none()) {
        let (tone, said) = (doc.tone, doc.message.clone());
        leave(doc, orbit, snap);
        doc.say(tone, said);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plumbline_world::editing as ed;
    use crate::model::Entry;
    use plumbline_world::value::Val;

    /// A job with a level, a vanity 36 x 21 x 34 1/2 in as an item, two of it set down, and
    /// a water heater as an item never set down. Made-up sizes.
    fn doc_with_items() -> (Doc, String, String) {
        let mut doc = crate::build_doc("/nonexistent/bench-test.json".to_string(), None).expect("a new job");
        let how = ed::Entered { status: Status::Assumed, source: "made-up plan" };
        let level = ed::storey(&mut doc.job, "Ground", "0 ft", "9 ft", how).expect("a level");
        let size = |typed: &str| {
            let (len, unit) = units::parse_length(typed).expect("a length");
            Val { value: Some(len), unit: unit.to_string(), status: Status::Assumed, source: "a typical size; the product is not chosen".to_string() }
        };
        let not_chosen = || Val::<String>::unknown("", "not chosen");
        let item = |kind, w: &str, d: &str, h: &str| Item { category: kind, maker: not_chosen(), model: not_chosen(), width: size(w), depth: size(d), height: size(h), weight: None, sheet: String::new(), division: None, form: None, priced_as: None };
        let vanity = ed::item(&mut doc.job, "Vanity, 36 in", item(plumbline_world::Kind::Furnishing, "36 in", "21 in", "34 1/2 in")).expect("the vanity");
        let heater = ed::item(&mut doc.job, "Water heater, 50 gal", item(plumbline_world::Kind::Equipment, "22 in", "22 in", "60 in")).expect("the heater");
        for at in [[5.0, 5.0], [9.0, 5.0]] {
            ed::place_item(&mut doc.job, &level, &vanity, at, "0 ft", ed::Entered { status: Status::Assumed, source: "set down by hand in the model" }).expect("set down");
        }
        doc.job.derive();
        (doc, vanity, heater)
    }

    /// Where a point of the bench is on a window `aspect` wide to high, from the eye the
    /// bench opens at: the camera's own arithmetic, as `place_camera` has it.
    fn on_screen(p: Vec3, at: Vec3, yaw: f32, pitch: f32, distance: f32, aspect: f32) -> Vec2 {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let eye = at + Vec3::new(sy * cp, sp, cy * cp) * distance;
        let lens = Projection::from(PerspectiveProjection { fov: crate::scene::LENS, aspect_ratio: aspect, near: 0.5, far: 40_000.0, ..default() });
        let clip = lens.get_clip_from_view() * Transform::from_translation(eye).looking_at(at, Vec3::Y).to_matrix().inverse() * p.extend(1.0);
        Vec2::new(clip.x / clip.w, clip.y / clip.w)
    }

    fn inside(hull: &[Vec2], p: Vec2) -> bool {
        let mut sign = 0.0_f32;
        for i in 0..hull.len() {
            let (a, b) = (hull[i], hull[(i + 1) % hull.len()]);
            let cross = (b - a).perp_dot(p - a);
            if cross.abs() < 1.0e-9 {
                continue;
            }
            if sign == 0.0 {
                sign = cross.signum();
            } else if cross.signum() != sign {
                return false;
            }
        }
        true
    }

    fn hull_of(points: &[Vec2]) -> Vec<Vec2> {
        let mut pts = points.to_vec();
        pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        let mut out: Vec<Vec2> = Vec::new();
        for pass in 0..2 {
            let start = out.len();
            for p in pts.iter() {
                while out.len() >= start + 2 && (out[out.len() - 1] - out[out.len() - 2]).perp_dot(*p - out[out.len() - 2]) <= 0.0 {
                    out.pop();
                }
                out.push(*p);
            }
            out.pop();
            pts.reverse();
            let _ = pass;
        }
        out
    }

    #[test]
    fn the_item_is_drawn_as_a_thing_set_down_is() {
        let (mut doc, vanity, heater) = doc_with_items();
        let o = doc.origin;
        let v = placement(&mut doc.job, &vanity, o).expect("the vanity is drawn");
        assert_eq!(v.kind, plumbline_world::Kind::Furnishing);
        assert_eq!(v.form, Some(forms::Form::BaseCabinet), "a vanity is a base cabinet, by its name");
        assert_eq!(v.status, Status::Assumed);
        match v.shape {
            Shape::Box { size, angle, .. } => {
                assert!((size[0] - 3.0).abs() < 1e-9 && (size[1] - 1.75).abs() < 1e-9 && (size[2] - 2.875).abs() < 1e-9, "{size:?}");
                assert_eq!(angle, 0.0);
            }
            other => panic!("not a box: {other:?}"),
        }
        // What a thing set down from it is drawn as is what the bench draws.
        let set_down: Vec<_> = plumbline_world::building::placements(&mut doc.job).0.into_iter().filter(|p| p.name.starts_with("Vanity")).collect();
        assert_eq!(set_down.len(), 2);
        assert_eq!(set_down[0].form, v.form);
        assert_eq!(set_down[0].kind, v.kind);
        assert_eq!(set_down[0].discipline, v.discipline);
        // An item never set down is drawn the same way: a tank.
        let h = placement(&mut doc.job, &heater, o).expect("the heater is drawn");
        assert_eq!(h.form, Some(forms::Form::Tank));
    }

    #[test]
    fn an_item_whose_size_is_not_known_has_nothing_to_draw_and_says_which() {
        let (mut doc, vanity, _) = doc_with_items();
        let e = doc.job.find(&vanity).expect("it is there");
        let mut item = doc.job.world.get::<Item>(e).cloned().expect("an item");
        item.depth = Val::unknown("in", "not on the sheet");
        doc.job.put(&vanity, item).expect("put");
        let o = doc.origin;
        let why = placement(&mut doc.job, &vanity, o).expect_err("no size, no shape");
        assert!(why.contains("depth"), "{why}");
        assert!(layout(&mut doc.job, &vanity).is_err());
    }

    #[test]
    fn three_dimensions_in_the_panels_words_and_none_over_the_item_or_each_other() {
        let (mut doc, vanity, _) = doc_with_items();
        let l = layout(&mut doc.job, &vanity).expect("a layout");
        let words: Vec<(&str, &str)> = l.dims.iter().map(|d| (d.what, d.words.as_str())).collect();
        assert_eq!(words, vec![("width", "36 in"), ("depth", "21 in"), ("height", "34 1/2 in")]);
        assert!(l.dims.iter().all(|d| d.status == Status::Assumed));
        // The floor is a little larger than the item, in whole feet each way.
        assert!(l.half[0] * 2.0 > 3.0 + 1.0 && l.half[1] * 2.0 > 1.75 + 1.0, "{:?}", l.half);
        assert!(l.half[0].fract() == 0.0 && l.half[1].fract() == 0.0);
        assert!(l.half[0] * 2.0 < 3.0 * 4.0, "a little larger, not a field: {:?}", l.half);
        // From the view the bench opens at, on a window 1.6 wide to high, no dimension's line
        // or end lies over the item or over another dimension.
        let (yaw, pitch) = (HOME_YAW, OPENING_PITCH);
        let aspect = 1.6;
        let at = |p: Vec3| on_screen(p, l.target, yaw, pitch, l.distance, aspect);
        let (w, d, h) = (l.size[0] as f32, l.size[1] as f32, l.size[2] as f32);
        let mut corners = Vec::new();
        for x in [-w / 2.0, w / 2.0] {
            for z in [-d / 2.0, d / 2.0] {
                for y in [0.0, h] {
                    corners.push(at(Vec3::new(x, y, z)));
                }
            }
        }
        let hull = hull_of(&corners);
        let samples = |dim: &Dim| -> Vec<Vec2> {
            let along = (0..=20).map(|i| dim.a.lerp(dim.b, i as f32 / 20.0));
            let ends = [dim.a, dim.b].into_iter().flat_map(|e| [e - dim.tick * 0.12, e + dim.tick * 0.12]);
            along.chain(ends).map(at).collect()
        };
        for dim in &l.dims {
            assert!(samples(dim).iter().all(|p| !inside(&hull, *p)), "the {} dimension lies over the item", dim.what);
        }
        for (i, a) in l.dims.iter().enumerate() {
            for b in l.dims.iter().skip(i + 1) {
                let (sa, sb) = (samples(a), samples(b));
                let gap = sa.iter().flat_map(|p| sb.iter().map(move |q| p.distance(*q))).fold(f32::MAX, f32::min);
                assert!(gap > 0.01, "the {} and the {} dimensions lie over one another ({gap})", a.what, b.what);
            }
        }
        // And the whole of it is in the window with room round it.
        let all: Vec<Vec2> = l.dims.iter().flat_map(|d| samples(d)).chain(corners.iter().copied()).collect();
        // Clear of the panels (down the left to -0.67 and down the right from 0.56 of a window 1600 wide), of the message card
        // over the top of the model (from 0.6 up) and of the status line (below -0.9).
        assert!(all.iter().all(|p| p.x > -0.55 && p.x < 0.5 && p.y > -0.85 && p.y < 0.55), "framed with room round it: {:?}", all.iter().fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |m, p| (m.0.min(p.x), m.1.max(p.x), m.2.min(p.y), m.3.max(p.y))));
        assert!(all.iter().any(|p| p.x.abs() > 0.2 || p.y.abs() > 0.2), "and not a speck in the middle");
    }

    #[test]
    fn a_tall_item_is_framed_with_room_round_it_too() {
        let (mut doc, _, heater) = doc_with_items();
        let l = layout(&mut doc.job, &heater).expect("a layout");
        assert_eq!(l.dims.iter().map(|d| d.words.as_str()).collect::<Vec<_>>(), vec!["22 in", "22 in", "60 in"]);
        let (w, d, h) = (l.size[0] as f32, l.size[1] as f32, l.size[2] as f32);
        let at = |p: Vec3| on_screen(p, l.target, HOME_YAW, OPENING_PITCH, l.distance, 1.6);
        let mut all = Vec::new();
        for x in [-w / 2.0, w / 2.0] {
            for z in [-d / 2.0, d / 2.0] {
                for y in [0.0, h] {
                    all.push(at(Vec3::new(x, y, z)));
                }
            }
        }
        for dim in &l.dims {
            all.push(at(dim.a));
            all.push(at(dim.b));
        }
        assert!(all.iter().all(|p| p.x > -0.55 && p.x < 0.5 && p.y > -0.85 && p.y < 0.55), "{all:?}");
    }

    #[test]
    fn the_bench_is_entered_and_left_and_the_view_that_was_left_comes_back() {
        let (mut doc, vanity, _) = doc_with_items();
        let snap = Snapshot::default();
        let mut orbit = Orbit::new();
        orbit.target = Vec3::new(3.0, 4.0, 5.0);
        orbit.distance = 77.0;
        orbit.yaw = 0.9;
        orbit.pitch = 0.5;
        orbit.home = (Vec3::new(1.0, 2.0, 3.0), 99.0);
        let before = orbit.target;
        // Something in hand is let go of; a tool is put down.
        doc.draw.tool = Some(crate::tools::Tool::Wall);
        doc.placing = Some(vanity.clone());
        let saved = plumbline_world::file::save(&mut doc.job);
        enter(&mut doc, &mut orbit, &snap, &vanity);
        assert_eq!(doc.bench.as_ref().map(|b| b.item.as_str()), Some(vanity.as_str()));
        assert!(doc.draw.tool.is_none() && doc.placing.is_none());
        assert_eq!(doc.selected.as_deref(), Some(vanity.as_str()), "its panel stays");
        assert_ne!(orbit.target, before, "the view goes to the bench");
        assert!(orbit.distance < 30.0, "{}", orbit.distance);
        assert!(doc.message.contains("36 in wide") && doc.message.contains("Back to the house"), "{}", doc.message);
        // Entering again, on another item, keeps the first view left, not the bench's.
        let heater = doc.job.world.query::<(&plumbline_world::Node, &Item)>().iter(&doc.job.world).find(|(n, _)| n.name.starts_with("Water")).map(|(n, _)| n.path.clone()).expect("the heater");
        enter(&mut doc, &mut orbit, &snap, &heater);
        assert_eq!(doc.bench.as_ref().map(|b| b.item.as_str()), Some(heater.as_str()));
        leave(&mut doc, &mut orbit, &snap);
        assert!(doc.bench.is_none());
        assert_eq!((orbit.target, orbit.yaw, orbit.pitch, orbit.distance, orbit.home), (before, 0.9, 0.5, 77.0, (Vec3::new(1.0, 2.0, 3.0), 99.0)));
        // The bench is a way of looking: the job is what it was, to the byte.
        assert_eq!(plumbline_world::file::save(&mut doc.job), saved);
    }

    #[test]
    fn what_follows_an_item_whose_three_sizes_were_all_checked_is_assumed_when_one_is_typed_over() {
        let (mut doc, vanity, _) = doc_with_items();
        // All three sizes checked on site: the two set down say Known.
        for what in ["width", "depth", "height"] {
            ed::measure(&mut doc.job, &vanity, what, ed::Reading::Same, None, "checked on site").expect("checked");
        }
        let status = |doc: &mut Doc| doc.job.world.query::<&plumbline_world::Placed>().iter(&doc.job.world).filter(|p| p.item.is_some()).map(|p| p.size.status).collect::<Vec<_>>();
        assert_eq!(status(&mut doc), vec![Status::Known, Status::Known]);
        doc.measuring = Some((vanity.clone(), "width".to_string(), true));
        crate::tools::measured(&mut doc, "40 in");
        assert_eq!(status(&mut doc), vec![Status::Assumed, Status::Assumed], "the ones set down are as sure as the item now is");
        let e = doc.job.find(&vanity).expect("there");
        assert_eq!(doc.job.world.get::<Item>(e).expect("an item").width.status, Status::Assumed);
    }

    #[test]
    fn an_item_taken_out_while_it_is_on_the_bench_ends_the_bench_and_what_was_said_stays_said() {
        let (mut doc, _, heater) = doc_with_items();
        let (snap, mut orbit) = (Snapshot::default(), Orbit::new());
        enter(&mut doc, &mut orbit, &snap, &heater);
        plumbline_world::editing::remove(&mut doc.job, &heater).expect("nothing is set down from it");
        doc.say(Tone::Warned, "Took the item out of the job. Undo brings it back.");
        answer(&mut doc, &mut orbit, &snap);
        assert!(doc.bench.is_none(), "an item that is gone is not on the bench");
        assert_eq!(doc.message, "Took the item out of the job. Undo brings it back.");
        assert!(doc.tone == Tone::Warned);
    }

    #[test]
    fn nothing_is_drawn_or_started_on_the_bench() {
        let (mut doc, vanity, _) = doc_with_items();
        let snap = Snapshot::default();
        let mut orbit = Orbit::new();
        enter(&mut doc, &mut orbit, &snap, &vanity);
        crate::tools::start(&mut doc, crate::tools::Tool::Wall);
        assert!(doc.draw.tool.is_none(), "no tool on the bench");
        assert!(doc.message.contains("bench") || doc.message.contains("Bench"), "{}", doc.message);
        // Nothing but the item can be picked.
        let other = doc.job.world.query::<(&plumbline_world::Node, &plumbline_world::Kind)>().iter(&doc.job.world).find(|(_, k)| **k == plumbline_world::Kind::Storey).map(|(n, _)| n.path.clone()).expect("a level");
        doc.select(&other);
        assert_eq!(doc.selected.as_deref(), Some(vanity.as_str()));
        // A key that would take a plan or a walk is refused.
        crate::set_view(&mut doc, &mut orbit, &snap, true, false);
        assert!(!doc.plan);
    }

    #[test]
    fn a_size_typed_on_an_item_is_assumed_and_says_how_many_follow_and_the_same_again_is_known() {
        let (mut doc, vanity, _) = doc_with_items();
        let size = |doc: &mut Doc, what: &str| {
            let e = doc.job.find(&vanity).expect("there");
            let item = doc.job.world.get::<Item>(e).cloned().expect("an item");
            let v = match what {
                "width" => item.width,
                "depth" => item.depth,
                _ => item.height,
            };
            (v.value.map(|l| units::show_length(&l, &v.unit)), v.status, v.source)
        };
        doc.selected = Some(vanity.clone());
        doc.measuring = Some((vanity.clone(), "width".to_string(), true));
        crate::tools::measured(&mut doc, "48 in");
        let (said, status, source) = size(&mut doc, "width");
        assert_eq!(said.as_deref(), Some("48 in"));
        assert_eq!(status, Status::Assumed, "{source}");
        assert_eq!(source, "typed in the window; not yet checked against its maker's sheet");
        assert!(doc.message.contains("48 in") && doc.message.contains("2 things set down from it follow"), "{}", doc.message);
        assert!(doc.message.contains("Assumed") && doc.message.contains("type 48 in again"), "{}", doc.message);
        assert!(doc.message.contains("still reads 36 in") && doc.message.contains("Rename item"), "{}", doc.message);
        // The ones set down follow, and are Assumed.
        let followers: Vec<_> = plumbline_world::building::placements(&mut doc.job).0.into_iter().filter(|p| p.name.starts_with("Vanity")).collect();
        assert!(followers.iter().all(|p| matches!(&p.shape, Shape::Box { size, .. } if (size[0] - 4.0).abs() < 1e-9) && p.status == Status::Assumed));
        // The same size again is the check: Known, checked on site.
        doc.measuring = Some((vanity.clone(), "width".to_string(), true));
        crate::tools::measured(&mut doc, "48 in");
        let (said, status, source) = size(&mut doc, "width");
        assert_eq!(said.as_deref(), Some("48 in"));
        assert_eq!(status, Status::Known, "{source}");
        assert!(source.contains("checked on site"), "{source}");
        assert!(doc.message.contains("checked on site") && doc.message.contains("Known"), "{}", doc.message);
        // A depth typed after is Assumed again, and the width stays what it is.
        doc.measuring = Some((vanity.clone(), "depth".to_string(), true));
        crate::tools::measured(&mut doc, "22 in");
        assert_eq!(size(&mut doc, "depth").1, Status::Assumed);
        assert_eq!(size(&mut doc, "width").1, Status::Known);
    }

    // ---- A shape on the bench (call 205, point 6) ----

    /// The bench's item kept as a new shape and a fourth block added, through the window's own functions.
    fn kept_and_added(doc: &mut Doc, vanity: &str) -> String {
        doc.selected = Some(vanity.to_string());
        crate::tools::keep_shape(doc, "vanity with a panel");
        crate::tools::shape_add(doc, "vanity with a panel");
        "vanity with a panel".to_string()
    }

    #[test]
    fn an_item_drawn_as_a_starter_shape_lists_its_blocks_and_is_not_the_jobs_own() {
        let (mut doc, vanity, heater) = doc_with_items();
        let shape = drawn_shape(&mut doc.job, &vanity).expect("a vanity is drawn as a base cabinet");
        assert_eq!((shape.name.as_str(), shape.own, shape.pieces.len()), ("base cabinet", false, 3));
        assert_eq!(shape.pieces.iter().map(|p| p.made.word()).collect::<Vec<_>>(), vec!["timber", "timber", "worktop"]);
        assert!(shape.worked.iter().all(Option::is_some));
        assert_eq!(shape.size, [3.0, 1.75, 2.875]);
        // A tank is a starter shape too; an item drawn as a box has none.
        assert_eq!(drawn_shape(&mut doc.job, &heater).map(|s| s.name), Some("tank".to_string()));
        ed::item_drawn_as(&mut doc.job, &vanity, Some("box")).expect("a box");
        assert!(drawn_shape(&mut doc.job, &vanity).is_none());
    }

    #[test]
    fn a_thing_set_down_is_drawn_from_the_rows_of_the_jobs_own_shape_and_the_bench_too() {
        let (mut doc, vanity, _) = doc_with_items();
        let (snap, mut orbit) = (Snapshot::default(), Orbit::new());
        enter(&mut doc, &mut orbit, &snap, &vanity);
        let name = kept_and_added(&mut doc, &vanity);
        assert_eq!(doc.shape_block, Some(4), "the block added is picked: {}", doc.message);
        let shape = drawn_shape(&mut doc.job, &vanity).expect("drawn as the copy");
        assert_eq!((shape.name.as_str(), shape.own, shape.pieces.len()), (name.as_str(), true, 4));
        // The model's blocks for each thing set down and for the item on the bench: four, the fourth the item's whole box.
        let drawn = plumbline_world::building::placements(&mut doc.job).0;
        let blocks = own_blocks(&mut doc.job, &drawn, Some(&vanity));
        let things: Vec<_> = drawn.iter().filter(|p| p.name.starts_with("Vanity")).collect();
        assert_eq!(things.len(), 2);
        for t in &things {
            assert_eq!(t.form, None, "a job's own shape is no form");
            let own = blocks.get(&t.path).expect("its blocks");
            assert_eq!(own.len(), 4);
            assert!((own[3].hi[0] - 1.5).abs() < 1e-9 && (own[3].hi[2] - 2.875).abs() < 1e-9, "{:?}", own[3]);
        }
        assert_eq!(blocks.get(&vanity).map(Vec::len), Some(4), "the bench's own item");
        // The same item drawn as a starter has no entry: it is drawn from its form.
        ed::item_drawn_as(&mut doc.job, &vanity, Some("base cabinet")).expect("back to the starter");
        let drawn = plumbline_world::building::placements(&mut doc.job).0;
        assert!(own_blocks(&mut doc.job, &drawn, Some(&vanity)).is_empty());
    }

    #[test]
    fn a_figure_typed_is_the_models_change_and_a_refusal_leaves_the_shape_and_says_why() {
        let (mut doc, vanity, _) = doc_with_items();
        let (snap, mut orbit) = (Snapshot::default(), Orbit::new());
        enter(&mut doc, &mut orbit, &snap, &vanity);
        // A starter shape is refused in the model's words, and nothing is changed.
        let saved = plumbline_world::file::save(&mut doc.job);
        doc.shaping = Some(("base cabinet".to_string(), 3, 5));
        crate::tools::shape_typed(&mut doc, "height + 1 in");
        assert!(doc.message.contains("starter shape") && doc.message.contains("keep it as a new shape first"), "{}", doc.message);
        assert_eq!(plumbline_world::file::save(&mut doc.job), saved);
        // The copy: the fourth block's figures typed, each its own change.
        let name = kept_and_added(&mut doc, &vanity);
        for (what, text) in [(0, "-width / 2 + 2 in"), (1, "width / 2 - 2 in"), (2, "depth - 1 in"), (3, "depth"), (4, "4 in"), (5, "height - 6 in")] {
            doc.shaping = Some((name.clone(), 4, what));
            crate::tools::shape_typed(&mut doc, text);
            assert!(doc.message.contains("1 item is drawn as it") && doc.message.contains("2 things set down from it follow"), "{}", doc.message);
        }
        let shape = drawn_shape(&mut doc.job, &vanity).expect("the copy");
        assert_eq!(shape.pieces[3].figures(), ["-width / 2 + 2 in", "width / 2 - 2 in", "depth - 1 in", "depth", "4 in", "height - 6 in"]);
        // Above the box: refused, as the model says it, and the row is as it was; the field is open on what was typed.
        let saved = plumbline_world::file::save(&mut doc.job);
        doc.shaping = Some((name.clone(), 4, 5));
        crate::tools::shape_typed(&mut doc, "height + 1 in");
        // (The words are the model's: "above the top" since call 212; they were "above the box".)
        assert!(doc.tone == Tone::Refused && doc.message.contains("1 in above the top"), "{}", doc.message);
        assert_eq!(plumbline_world::file::save(&mut doc.job), saved);
        assert!(matches!(doc.entry, Some((Entry::ShapeBlock, _))));
        // A form, a material and a condition are typed the same way; a word that is none of them is refused.
        for (what, text, said) in [(6, "round", "an upright round"), (6, "box", "a box"), (7, "steel", "made of steel"), (8, "width >= 30 in", "there when width >= 30 in"), (8, "none", "every size")] {
            doc.shaping = Some((name.clone(), 4, what));
            crate::tools::shape_typed(&mut doc, text);
            assert!(doc.message.contains(said), "{}: {}", text, doc.message);
        }
        doc.shaping = Some((name.clone(), 4, 7));
        crate::tools::shape_typed(&mut doc, "brass");
        assert!(doc.tone == Tone::Refused && doc.message.contains("timber"), "{}", doc.message);
        // Take block away takes the picked one, and the next is picked.
        crate::tools::shape_take(&mut doc, &name, 4);
        assert_eq!(drawn_shape(&mut doc.job, &vanity).map(|s| s.pieces.len()), Some(3));
        assert_eq!(doc.shape_block, Some(3));
        // A name taken is refused in the model's words.
        doc.selected = Some(vanity.clone());
        ed::item_drawn_as(&mut doc.job, &vanity, Some("base cabinet")).expect("the starter again");
        crate::tools::keep_shape(&mut doc, "Vanity with a panel");
        assert!(doc.message.contains("already"), "{}", doc.message);
    }
}
