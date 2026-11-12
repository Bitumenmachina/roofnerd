//! The main window's panels, set round the model the way a building game sets them
//! (DECISIONS.md, call 55): along the top the job, Save, Undo and Redo, the view and the
//! price; down the left what to do next, the levels, the drawings, what is on show and
//! the systems; along the bottom the build bar, with the fields of the tool in hand over
//! it; down the right what is selected, as fields and buttons; and what is written on
//! the model itself.
//!
//! Everything here is drawn from the snapshot. A panel shows what the world holds and how
//! sure it is; it works nothing out.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::model::{to_view, Doc, Entry, Snapshot, Tone};
use crate::scene::{facing, MainCamera, BUILD_H, LEFT_W, RIGHT_W, STATUS_H, TOP_H};
use crate::sheet::SheetScroll;
use crate::theme::{self, button, chip, heading, keycap, named, status_chip, swatch, text, Fonts, Named, Weight};
use plumbline_world::building::{Shape, SystemTakeoff};
use plumbline_world::value::Status;
use plumbline_world::{units, view, Discipline, Kind};

#[derive(Component)]
pub struct MainUi;
#[derive(Component)]
pub struct PointerText;
#[derive(Component)]
pub struct InspectorScroll;

/// Something written over the model, kept at a point of it as the view moves.
#[derive(Component)]
pub struct Label {
    pub at: Vec3,
    /// The way the surface it names faces; the label hides when that is away from the eye.
    pub facing: Option<Vec3>,
    /// Which label gives way when two would overlap: the higher number does.
    pub rank: u8,
}

/// The cameras that draw panels: one over the 3D view, one in the sheet window.
#[derive(Resource)]
pub struct Cameras {
    pub ui: Entity,
    pub sheet: Entity,
}

/// How far each scrolling panel has been scrolled, kept across redraws.
#[derive(Resource, Default)]
pub struct Scrolls {
    pub inspector: f32,
    pub sheet: f32,
    pub browser: f32,
}

/// The left panel: it scrolls when it holds more than fits.
#[derive(Component)]
pub struct BrowserScroll;

fn divider(p: &mut ChildSpawnerCommands) {
    p.spawn((Node { width: px(1.0), height: px(22.0), margin: UiRect::horizontal(px(2.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::hairline())));
}

/// The wordmark: a plumb line, and the name.
pub fn wordmark(p: &mut ChildSpawnerCommands, fonts: &Fonts) {
    p.spawn(Node { align_items: AlignItems::Center, column_gap: px(9.0), flex_shrink: 0.0, ..default() }).with_children(|m| {
        m.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }).with_children(|bob| {
            bob.spawn((Node { width: px(2.0), height: px(13.0), ..default() }, BackgroundColor(theme::accent())));
            bob.spawn((Node { width: px(8.0), height: px(8.0), border_radius: BorderRadius::all(px(4.0)), ..default() }, BackgroundColor(theme::accent())));
        });
        m.spawn((text(fonts, "plumbline", 15.0, Weight::Semibold, theme::ink()), TextLayout::no_wrap()));
    });
}

pub fn tally_chips(p: &mut ChildSpawnerCommands, fonts: &Fonts, tally: (usize, usize, usize)) {
    chip(p, fonts, &format!("{} known", view::figure(tally.0 as f64, 0)), theme::ink_soft(), theme::panel_alt());
    if tally.1 > 0 {
        chip(p, fonts, &format!("{} assumed", view::figure(tally.1 as f64, 0)), theme::amber(), theme::amber_wash());
    }
    if tally.2 > 0 {
        chip(p, fonts, &format!("{} unknown", view::figure(tally.2 as f64, 0)), theme::red(), theme::red_wash());
    }
}

/// A figure with its unit, or what stands in for one that is not there.
pub fn amount(value: Option<f64>, decimals: usize, unit: &str) -> String {
    match value {
        Some(x) => format!("{} {unit}", view::figure_shown(x, decimals)).trim().to_string(),
        None => "unknown".into(),
    }
}

/// A total as it is said: the figure, or that there is nothing to total, or what is missing.
pub fn total(sum: &view::Sum) -> String {
    match (sum.value, sum.members) {
        (_, 0) => "none".into(),
        // A count is a whole number; a length or an area is said to two places.
        (Some(x), _) => amount(Some(x), if sum.unit == "EA" { 0 } else { 2 }, sum.unit),
        (None, n) => format!("no total, {} of {n} missing", sum.missing.len()),
    }
}

pub fn pretty(label: &str) -> String {
    // What a thing's assembly is is kept in the job under its older name.
    let s = if label == "condition" { "assembly".to_string() } else { label.replace('_', " ") };
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => s,
    }
}

/// A row's value as the panels show it: a derived figure to two places, a pitch as 6:12.
pub fn shown(row: &view::Row) -> String {
    match row.number {
        Some(x) => amount(Some(x), if row.unit == "EA" { 0 } else { 2 }, &row.unit),
        None => theme::trade(&row.text),
    }
}

/// A message cut to what a one-line bar can hold. The whole of it is in the card over
/// the model whenever it is more than a plain "done".
pub fn one_line(message: &str, most: usize) -> String {
    let first = message.lines().next().unwrap_or("");
    if first.chars().count() <= most {
        return first.to_string();
    }
    let cut: String = first.chars().take(most.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

pub fn tone_ink(tone: Tone) -> (Color, Color) {
    match tone {
        Tone::Done => (theme::green(), theme::ink_soft()),
        Tone::Warned => (theme::amber(), theme::amber()),
        Tone::Refused => (theme::red(), theme::red()),
    }
}

/// The middle of a plan outline, for hanging a label on.
fn centroid(points: &[[f64; 2]]) -> [f64; 2] {
    let n = points.len();
    let (mut a, mut cx, mut cy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (p, q) = (points[i], points[(i + 1) % n]);
        let w = p[0] * q[1] - q[0] * p[1];
        a += w;
        cx += (p[0] + q[0]) * w;
        cy += (p[1] + q[1]) * w;
    }
    if a.abs() < 1e-9 {
        let k = n.max(1) as f64;
        return [points.iter().map(|p| p[0]).sum::<f64>() / k, points.iter().map(|p| p[1]).sum::<f64>() / k];
    }
    [cx / (3.0 * a), cy / (3.0 * a)]
}

/// A panel floating over the model: a sheet of frosted glass with round corners, a
/// light edge and a shadow, a gap clear of the window's edge and of the next panel.
/// The node says where it would sit hard against the edges; the gaps are put in here.
fn docked(mut node: Node, _edge: UiRect) -> impl Bundle {
    node.position_type = PositionType::Absolute;
    let float = |v: Val| match v {
        Val::Px(x) => px(if x == 0.0 { theme::GAP } else { x + 2.0 * theme::GAP }),
        other => other,
    };
    (node.top, node.left, node.right, node.bottom) = (float(node.top), float(node.left), float(node.right), float(node.bottom));
    node.border = UiRect::all(px(1.0));
    node.border_radius = BorderRadius::all(px(theme::PANEL_RADIUS));
    (node, BackgroundColor(theme::frost()), BorderColor::all(theme::frost_edge()), theme::panel_shadow())
}

/// What a click with the left button on this does.
fn on_click(e: &mut EntityCommands, act: impl Fn(&mut Doc) + Send + Sync + 'static) {
    e.observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
        if press.button == PointerButton::Primary {
            act(&mut doc);
        }
    });
}

/// Draw the panels again whenever the snapshot has changed.
#[allow(clippy::too_many_arguments)]
pub fn rebuild(mut commands: Commands, doc: Res<Doc>, snap: Res<Snapshot>, plans: Res<crate::plans::Plans>, fonts: Res<Fonts>, cameras: Res<Cameras>, scrolls: Res<Scrolls>, old: Query<Entity, With<MainUi>>, mut drawn: Local<u64>) {
    if *drawn == snap.revision {
        return;
    }
    *drawn = snap.revision;
    for e in &old {
        commands.entity(e).despawn();
    }
    let fonts = &*fonts;
    commands.spawn((Node { width: percent(100.0), height: percent(100.0), ..default() }, UiTargetCamera(cameras.ui), Pickable::IGNORE, MainUi)).with_children(|root| {
        overlay(root, fonts, &doc, &snap);
        left(root, fonts, &doc, &snap, &plans, scrolls.browser);
        right(root, fonts, &doc, &snap, &plans, scrolls.inspector);
        build_bar(root, fonts, &doc);
        top_bar(root, fonts, &doc, &snap);
        status_line(root, fonts, &doc);
    });
}

/// Along the top: the job, Save, Undo and Redo, the view, and at all times the price
/// and how much of the job is not yet sure.
fn top_bar(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    p.spawn(docked(Node { top: px(0.0), left: px(0.0), right: px(0.0), height: px(TOP_H), padding: UiRect::horizontal(px(12.0)), align_items: AlignItems::Center, column_gap: px(8.0), overflow: Overflow::clip(), ..default() }, UiRect::bottom(px(1.0)))).with_children(|bar| {
        wordmark(bar, fonts);
        divider(bar);
        // A long title is cut at a word's worth and says it is cut; it is whole on the right.
        bar.spawn(Node { flex_shrink: 0.0, ..default() }).with_child((theme::name(fonts, one_line(&snap.title, 26), 13.0, Weight::Semibold, theme::ink()), TextLayout::no_wrap()));
        divider(bar);
        on_click(&mut button(bar, fonts, "Save", "Ctrl+S", false), |doc| doc.press(KeyCode::F2));
        on_click(&mut button(bar, fonts, "Undo", "Ctrl+Z", false), |doc| doc.press(KeyCode::KeyZ));
        on_click(&mut button(bar, fonts, "Redo", "Ctrl+Y", false), |doc| doc.press(KeyCode::KeyY));
        divider(bar);
        for (label, key, plan, walk) in [("3D", "", false, false), ("Plan", "T", true, false), ("Walk", "V", false, true)] {
            let on = doc.plan == plan && doc.walk == walk;
            button(bar, fonts, label, key, on).observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>, mut orbit: ResMut<crate::scene::Orbit>, snap: Res<Snapshot>| {
                if press.button == PointerButton::Primary {
                    crate::set_view(&mut doc, &mut orbit, &snap, plan, walk);
                }
            });
        }
        on_click(&mut button(bar, fonts, "Frame", "F", false), |doc| doc.press(KeyCode::KeyF));
        bar.spawn(Node { flex_grow: 1.0, ..default() });
        // The price, in the colour of how sure it is.
        // A job with nothing priced has no price yet, which is not a price in doubt.
        let priced = snap.estimate.as_ref().is_some_and(|e| !e.lines.is_empty());
        let (price, status) = match (&snap.estimate, priced) {
            (Some(e), true) => crate::sheet::dollars(Some(&e.price)),
            _ => ("nothing priced yet".to_string(), Status::Known),
        };
        bar.spawn(Node { align_items: AlignItems::Baseline, column_gap: px(7.0), flex_shrink: 0.0, ..default() }).with_children(|b| {
            b.spawn((text(fonts, "PRICE", 10.0, Weight::Semibold, theme::ink_faint()), TextLayout::no_wrap()));
            b.spawn((text(fonts, price, if priced { 16.0 } else { 12.0 }, if priced { Weight::Semibold } else { Weight::Regular }, if priced { theme::status_ink(status) } else { theme::ink_faint() }), TextLayout::no_wrap()));
        });
        status_chip(bar, fonts, status);
        // What there is to verify in the field, and the dollars riding on it.
        let questions = snap.verify.questions.len();
        if questions > 0 {
            let said = match snap.verify.riding {
                Some(x) => format!("{questions} to verify, ${} riding", view::figure(x, 2)),
                None => format!("{questions} to verify"),
            };
            let mut go = button(bar, fonts, &said, "", doc.tab == crate::model::Tab::Verify);
            go.insert(named("To verify"));
            on_click(&mut go, |doc| {
                doc.tab = crate::model::Tab::Verify;
                doc.redraw();
            });
        }
        divider(bar);
        tally_chips(bar, fonts, snap.tally);
        divider(bar);
        bar.spawn((theme::name(fonts, snap.file.clone(), 12.0, Weight::Regular, theme::ink_soft()), TextLayout::no_wrap()));
        if doc.edited {
            chip(bar, fonts, "Not saved", theme::amber(), theme::amber_wash());
        } else {
            chip(bar, fonts, "Saved", theme::green(), theme::panel());
        }
    });
}

/// Along the bottom: what the last action did, where the pointer is, and the view.
fn status_line(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc) {
    let first = pretty(&one_line(&doc.message, 150));
    let (dot, ink) = tone_ink(doc.tone);
    p.spawn(docked(Node { bottom: px(0.0), left: px(0.0), right: px(0.0), height: px(STATUS_H), padding: UiRect::horizontal(px(12.0)), align_items: AlignItems::Center, column_gap: px(9.0), overflow: Overflow::clip(), ..default() }, UiRect::top(px(1.0)))).with_children(|bar| {
        bar.spawn((Node { width: px(7.0), height: px(7.0), flex_shrink: 0.0, ..default() }, BackgroundColor(dot)));
        bar.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), overflow: Overflow::clip(), ..default() }).with_child((text(fonts, first, 12.0, Weight::Regular, ink), TextLayout::no_wrap()));
        bar.spawn(Node { flex_shrink: 0.0, ..default() }).with_child((text(fonts, "", 12.0, Weight::Regular, theme::ink_soft()), TextLayout::no_wrap(), PointerText));
        divider(bar);
        bar.spawn(Node { flex_shrink: 0.0, ..default() }).with_child(text(fonts, if doc.walk { "On foot" } else if doc.plan { "Plan" } else { "3D" }, 12.0, Weight::Medium, theme::ink_soft()));
    });
}

/// A row of the left panel that does something when it is clicked.
fn row(p: &mut ChildSpawnerCommands, on: bool, name: String, act: impl Fn(&mut Doc) + Send + Sync + 'static, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    let mut e = p.spawn((Node { align_items: AlignItems::Center, column_gap: px(7.0), padding: UiRect::axes(px(6.0), px(3.0)), min_height: px(24.0), flex_shrink: 0.0, ..default() }, BackgroundColor(if on { theme::accent_wash() } else { Color::NONE }), Named { name, scrolls: true }));
    e.with_children(build);
    on_click(&mut e, act);
}

/// A switch: a pill with a knob at its right when it is on (in the accent) and at its
/// left when it is off (grey). It reads on or off at a glance, which a small square
/// did not. (About the proportions of the switch everyone knows, at this text's size.)
fn switch(p: &mut ChildSpawnerCommands, on: bool) {
    p.spawn((
        Node { width: px(30.0), height: px(17.0), padding: UiRect::all(px(2.0)), border_radius: BorderRadius::all(px(9.0)), justify_content: if on { JustifyContent::FlexEnd } else { JustifyContent::FlexStart }, align_items: AlignItems::Center, flex_shrink: 0.0, ..default() },
        BackgroundColor(if on { theme::accent() } else { Color::srgba(0.07, 0.09, 0.12, 0.18) }),
        Pickable::IGNORE,
    ))
    .with_child((Node { width: px(13.0), height: px(13.0), border_radius: BorderRadius::all(px(7.0)), ..default() }, BackgroundColor(Color::WHITE), BoxShadow(vec![ShadowStyle { color: Color::srgba(0.0, 0.0, 0.0, 0.25), x_offset: px(0.0), y_offset: px(1.0), spread_radius: px(0.0), blur_radius: px(2.0) }]), Pickable::IGNORE));
}

/// A round spot of a layer's colour: what the layer is drawn in.
fn spot(p: &mut ChildSpawnerCommands, colour: Color) {
    p.spawn((Node { width: px(9.0), height: px(9.0), border_radius: BorderRadius::all(px(5.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(colour), Pickable::IGNORE));
}

/// A box that is ticked or not.
fn tick(p: &mut ChildSpawnerCommands, on: bool, colour: Color) {
    p.spawn((Node { width: px(10.0), height: px(10.0), border: UiRect::all(px(1.5)), flex_shrink: 0.0, ..default() }, BackgroundColor(if on { colour } else { Color::NONE }), BorderColor::all(colour), Pickable::IGNORE));
}

/// One of several fields that go together, as the form being typed has it: the one
/// being typed in shows what is typed and the caret.
fn form_field<'a>(p: &'a mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, open: bool, at: usize, label: &str, like: &str, rest: &str, width: f32) -> EntityCommands<'a> {
    let (value, state) = match (&doc.entry, open) {
        (Some((_, typed)), true) if doc.form_at == at => (typed.clone(), 1),
        (_, true) => (doc.form.get(at).cloned().unwrap_or_default(), 0),
        _ => (rest.to_string(), 0),
    };
    let mut e = theme::field(p, fonts, label, &value, like, state, width);
    e.insert(Named { name: format!("{label} field"), scrolls: false });
    e
}

/// What a job needs next, read from the job itself. Each step is ticked when the job
/// shows it done; the first that is not says how it is done.
struct Step {
    done: bool,
    what: &'static str,
    how: String,
}

fn steps_of(doc: &Doc, snap: &Snapshot) -> Vec<Step> {
    let walls = snap.fabric.iter().any(|(_, el)| el.kind == Kind::Wall);
    let floors = snap.fabric.iter().any(|(_, el)| el.kind == Kind::Floor);
    let rooms = snap.fabric.iter().any(|(_, el)| el.kind == Kind::Room);
    let openings = snap.storeys.iter().any(|s| s.doors + s.windows > 0);
    let scaled = snap.drawings.iter().any(|l| l.scale_said().is_some());
    let services = snap.counts.iter().any(|(d, n)| *d != Discipline::Architecture && *n > 0);
    let priced = snap.pricing.as_ref().is_some_and(|p| !p.elements.is_empty());
    let unsure = snap.verify.questions.len();
    vec![
        Step { done: !snap.storeys.is_empty(), what: "Add a level", how: "Click Add level below. Type its name, the elevation of its floor, and floor-to-floor.".into() },
        Step {
            // Tracing is the usual way in, not the only one: walls drawn without a drawing count.
            done: scaled || (walls && snap.drawings.is_empty()),
            what: "Add a drawing and set its scale",
            how: if snap.drawings.is_empty() { "Click Add drawing and type the file's name (PDF, PNG or JPEG), or drop the file on this window.".into() } else { "Click Set scale, then the two ends of one dimension on the drawing, and type the dimension.".into() },
        },
        Step { done: walls, what: "Draw the walls", how: "Click Wall in the bar at the bottom, or press 1. Type its thickness and height, then click each corner.".into() },
        Step { done: openings, what: "Add doors and windows", how: "Click Door or Window, then click the place on a wall.".into() },
        Step { done: floors && rooms, what: "Add the floor and name the rooms", how: if floors { "Click Room, click inside a room, and type its name.".into() } else { "Click Floor, then anywhere on the level. Then Room, for each room.".into() } },
        Step { done: !snap.planes.is_empty(), what: "Add the roof", how: "Click Roof. Type its pitch and eave height, then click each corner of its outline and the first again.".into() },
        Step { done: services, what: "Run the ducts, pipes and wiring", how: "Click Duct, Pipe, Conduit or Cable, then click along its run. Equipment and Fixture stand where you click.".into() },
        Step { done: priced, what: "Price it", how: "Under Bring in, Starter assemblies gives the job a set to price with. Then click a wall, Assembly, and pick one; or Allowance for a price per SF. The sheet window shows the estimate.".into() },
        Step { done: priced && unsure == 0, what: "Verify in field", how: format!("{unsure} to verify. The sheet's Verify in field tab lists them by the dollars riding on each. Click one, click a value marked Measure, and type what the tape or laser reads.") },
        Step { done: !doc.edited && walls, what: "Save", how: "Click Save, or press Ctrl+S.".into() },
    ]
}

fn steps(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    let all = steps_of(doc, snap);
    let done = all.iter().filter(|s| s.done).count();
    let next = all.iter().position(|s| !s.done);
    let mut head = p.spawn((Node { align_items: AlignItems::Center, column_gap: px(6.0), margin: UiRect { top: px(4.0), bottom: px(4.0), ..default() }, padding: UiRect::bottom(px(3.0)), border: UiRect::bottom(px(1.0)), flex_shrink: 0.0, ..default() }, BorderColor::all(theme::hairline()), Named { name: "Next steps".into(), scrolls: true }));
    head.with_children(|h| {
        h.spawn((text(fonts, "NEXT STEPS", 10.0, Weight::Semibold, theme::ink_soft()), Pickable::IGNORE));
        h.spawn(Node { flex_grow: 1.0, ..default() });
        h.spawn((text(fonts, format!("{done} of {} done", all.len()), 10.5, Weight::Regular, theme::ink_faint()), Pickable::IGNORE));
        h.spawn((text(fonts, if doc.steps_open { "Hide" } else { "Show" }, 10.5, Weight::Medium, theme::accent()), Pickable::IGNORE));
    });
    on_click(&mut head, |doc| {
        doc.steps_open = !doc.steps_open;
        doc.touch();
    });
    if !doc.steps_open {
        // Folded away, the one thing to do next is still said.
        if let Some(i) = next {
            p.spawn(Node { padding: UiRect::axes(px(6.0), px(2.0)), flex_shrink: 0.0, ..default() }).with_child(text(fonts, format!("Next: {}", all[i].what), 11.5, Weight::Medium, theme::ink()));
        }
        return;
    }
    for (i, step) in all.iter().enumerate() {
        let now = next == Some(i);
        p.spawn((Node { column_gap: px(7.0), padding: UiRect::axes(px(6.0), px(2.0)), align_items: AlignItems::FlexStart, flex_shrink: 0.0, ..default() }, BackgroundColor(if now { theme::accent_wash() } else { Color::NONE }))).with_children(|r| {
            r.spawn(Node { margin: UiRect::top(px(3.0)), ..default() }).with_children(|m| tick(m, step.done, if step.done { theme::green() } else if now { theme::accent() } else { theme::ink_faint() }));
            r.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(2.0), flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), ..default() }).with_children(|c| {
                c.spawn(text(fonts, step.what, 11.5, if now { Weight::Semibold } else { Weight::Regular }, if step.done { theme::ink_faint() } else { theme::ink() }));
                if now {
                    c.spawn(text(fonts, step.how.clone(), 11.0, Weight::Regular, theme::ink_soft()));
                }
            });
        });
    }
}

/// Down the left: what the job needs next, its levels, the drawings under the level on
/// show, what of the building is on show, and its systems. The lists are the legend
/// too: a system's swatch here is the colour its runs are drawn in.
fn left(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot, plans: &crate::plans::Plans, scrolled: f32) {
    use crate::tools::{self, Tool};
    // It scrolls when it holds more than fits: a row is never squeezed.
    p.spawn((
        docked(Node { top: px(TOP_H), left: px(0.0), max_height: Val::Vh(83.0), width: px(LEFT_W), flex_direction: FlexDirection::Column, padding: UiRect { left: px(8.0), right: px(8.0), top: px(4.0), bottom: px(10.0) }, overflow: Overflow::scroll_y(), ..default() }, UiRect::right(px(1.0))),
        ScrollPosition(Vec2::new(0.0, scrolled)),
        BrowserScroll,
    ))
    .with_children(|c| {
        steps(c, fonts, doc, snap);

        heading(c, fonts, "Levels");
        if snap.storeys.len() > 1 {
            row(
                c,
                doc.show.storey.is_none(),
                "All levels".into(),
                |doc| {
                    doc.show.storey = None;
                    doc.redraw();
                },
                |r| {
                    r.spawn((text(fonts, "All levels", 12.0, Weight::Medium, theme::ink()), Pickable::IGNORE));
                },
            );
        }
        // From the top down, as a building section reads.
        let mut levels: Vec<&plumbline_world::building::StoreyTotals> = snap.storeys.iter().collect();
        levels.sort_by(|a, b| b.elevation.unwrap_or(f64::MIN).total_cmp(&a.elevation.unwrap_or(f64::MIN)));
        for storey in levels {
            let path = storey.path.clone();
            row(
                c,
                doc.show.storey.as_deref() == Some(storey.path.as_str()),
                format!("Level {}", storey.name),
                move |doc| {
                    // The level is put on show, and selected: its elevation and its
                    // floor-to-floor are in the panel on the right, to be measured.
                    doc.show.storey = Some(path.clone());
                    doc.select(&path);
                    doc.redraw();
                },
                |r| {
                    r.spawn((theme::name(fonts, storey.name.clone(), 12.0, Weight::Medium, theme::ink()), Pickable::IGNORE));
                    r.spawn(Node { flex_grow: 1.0, ..default() });
                    r.spawn((text(fonts, storey.elevation.map(|e| units::feet_as_ft_in(e, 16)).unwrap_or("no elevation".into()), 11.5, Weight::Regular, if storey.elevation.is_some() { theme::ink_soft() } else { theme::red() }), Pickable::IGNORE));
                },
            );
        }
        let adding = matches!(doc.entry, Some((Entry::Storey, _)));
        if adding {
            c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(4.0), padding: UiRect::axes(px(6.0), px(4.0)), flex_shrink: 0.0, ..default() }).with_children(|f| {
                for (i, (label, like)) in [("Name", "Ground"), ("Elevation", "0 ft"), ("Floor-to-floor", "9 ft")].into_iter().enumerate() {
                    on_click(&mut form_field(f, fonts, doc, true, i, label, like, "", LEFT_W - 30.0), move |doc| {
                        if let Some((Entry::Storey, typed)) = doc.entry.clone() {
                            let mut values = doc.form.clone();
                            let at = doc.form_at;
                            values[at] = typed;
                            doc.open_form(Entry::Storey, values, i);
                        }
                    });
                }
                f.spawn(text(fonts, "Tab to the next. Enter adds it. Esc cancels.", 10.5, Weight::Regular, theme::ink_faint()));
            });
        } else {
            c.spawn(Node { padding: UiRect::axes(px(6.0), px(3.0)), flex_shrink: 0.0, ..default() }).with_children(|b| {
                on_click(&mut button(b, fonts, "Add level", "", false), |doc| {
                    doc.open_form(Entry::Storey, vec![String::new(); 3], 0);
                    doc.say(Tone::Done, "Add level: type its name, the elevation of its floor, and floor-to-floor.");
                });
            });
        }

        // The drawings laid under the level on show: what is traced over. Each says its
        // scale, or that it has none yet.
        heading(c, fonts, "Drawings");
        let on_show: Vec<&crate::plans::Laid> = snap.drawings.iter().filter(|l| doc.show.storey.as_deref().is_none_or(|s| s == l.storey)).collect();
        for l in &on_show {
            let path = l.path.clone();
            let (said, ink) = if plans.is_reading(&l.path) {
                ("reading its page".to_string(), theme::ink_soft())
            } else if plans.faults.contains_key(&l.path) {
                ("its page cannot be shown".to_string(), theme::red())
            } else if plans.read.get(&l.path).is_some_and(|r| r.changed) {
                ("its file has changed since it was added".to_string(), theme::red())
            } else {
                // The scale in the colour of how sure it is, as every figure is.
                match l.scale_said() {
                    Some(said) => (said, theme::status_ink(l.drawing.status())),
                    None => ("no scale yet".to_string(), theme::red()),
                }
            };
            row(c, doc.selected.as_deref() == Some(l.path.as_str()), format!("Drawing {}", l.name), move |doc| doc.select(&path), |r| {
                r.spawn((Node { flex_direction: FlexDirection::Column, min_width: px(0.0), flex_grow: 1.0, flex_basis: px(0.0), overflow: Overflow::clip(), ..default() }, Pickable::IGNORE)).with_children(|col| {
                    col.spawn((theme::name(fonts, l.name.clone(), 12.0, Weight::Medium, theme::ink()), TextLayout::no_wrap(), Pickable::IGNORE));
                    col.spawn((text(fonts, said.clone(), 11.0, Weight::Regular, ink), TextLayout::no_wrap(), Pickable::IGNORE));
                });
            });
        }
        if matches!(doc.entry, Some((Entry::Drawing, _))) {
            c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(4.0), padding: UiRect::axes(px(6.0), px(4.0)), flex_shrink: 0.0, ..default() }).with_children(|f| {
                let typed = doc.entry.as_ref().map(|e| e.1.clone()).unwrap_or_default();
                theme::field(f, fonts, "File, page", &typed, "", 1, LEFT_W - 30.0);
                f.spawn(text(fonts, "A PDF, PNG or JPEG, like plans/a-1.pdf, 2. Enter adds it. Or drop the file on this window.", 10.5, Weight::Regular, theme::ink_faint()));
            });
        } else {
            c.spawn(Node { padding: UiRect::axes(px(6.0), px(3.0)), column_gap: px(4.0), row_gap: px(4.0), flex_wrap: FlexWrap::Wrap, flex_shrink: 0.0, ..default() }).with_children(|b| {
                on_click(&mut button(b, fonts, "Add drawing", "", false), |doc| {
                    doc.open(Entry::Drawing, "");
                    doc.say(Tone::Done, "Add drawing: type the file's name, or drop the file on this window.");
                });
                if !on_show.is_empty() {
                    on_click(&mut button(b, fonts, "Set scale", "", doc.draw.tool == Some(Tool::Scale)), |doc| tools::start(doc, Tool::Scale));
                    on_click(&mut button(b, fonts, "Line up", "", doc.draw.tool == Some(Tool::LineUp)), |doc| tools::start(doc, Tool::LineUp));
                }
            });
        }

        // What people already have, brought in: each button asks for its file.
        heading(c, fonts, "Bring in");
        match doc.entry {
            Some((Entry::BringIn(what), ref typed)) => {
                c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(4.0), padding: UiRect::axes(px(6.0), px(4.0)), flex_shrink: 0.0, ..default() }).with_children(|f| {
                    theme::field(f, fonts, &format!("{}: its file", what.name()), typed, "", 1, LEFT_W - 30.0);
                    let hint = match what {
                        crate::model::Bring::Prices => "A CSV or XLSX, like lists/ridge.xlsx. A supplier's name can follow a comma. Enter brings it in. Or drop the file on this window.",
                        crate::model::Bring::Model => "An IFC file, like models/mechanical.ifc. Enter brings it in. Or drop the file on this window.",
                        _ => "A CSV or XLSX. Enter brings it in. Or drop the file on this window.",
                    };
                    f.spawn(text(fonts, hint, 10.5, Weight::Regular, theme::ink_faint()));
                });
            }
            _ => {
                c.spawn(Node { padding: UiRect::axes(px(6.0), px(3.0)), column_gap: px(4.0), row_gap: px(4.0), flex_wrap: FlexWrap::Wrap, flex_shrink: 0.0, ..default() }).with_children(|b| {
                    // The set of assemblies the program ships with: no file to find.
                    on_click(&mut button(b, fonts, "Starter assemblies", "", false), crate::bring::starter);
                    for what in crate::model::Bring::ALL {
                        on_click(&mut button(b, fonts, what.name(), "", false), move |doc| {
                            // A table dropped on the window and waiting to be told what it is.
                            match doc.dropped.take() {
                                Some(file) if what != crate::model::Bring::Model => crate::bring::bring(doc, what, &file),
                                _ => {
                                    doc.open(Entry::BringIn(what), "");
                                    doc.say(Tone::Done, format!("{}: type the file's name, or drop the file on this window.", what.name()));
                                }
                            }
                        });
                    }
                });
            }
        }

        heading(c, fonts, "Show");
        let layer = |c: &mut ChildSpawnerCommands, name: &str, key: &str, colour: Color, on: bool, count: Option<usize>, act: Box<dyn Fn(&mut Doc) + Send + Sync>| {
            row(c, false, format!("Show {name}"), move |doc| act(doc), |r| {
                spot(r, colour);
                r.spawn((text(fonts, name, 12.0, Weight::Medium, if on { theme::ink() } else { theme::ink_faint() }), Pickable::IGNORE));
                if let Some(count) = count {
                    r.spawn((text(fonts, count.to_string(), 11.0, Weight::Regular, theme::ink_faint()), Pickable::IGNORE));
                }
                r.spawn(Node { flex_grow: 1.0, ..default() });
                if !key.is_empty() {
                    keycap(r, fonts, key);
                }
                switch(r, on);
            });
        };
        for (i, d) in [Discipline::Architecture, Discipline::Mechanical, Discipline::Electrical, Discipline::Plumbing].into_iter().enumerate() {
            let count = snap.counts.iter().find(|c| c.0 == d).map(|c| c.1).unwrap_or(0);
            if i == 1 && !snap.planes.is_empty() {
                layer(c, "Roof", "", theme::roof_bare(), doc.show.roof, Some(snap.planes.len()), Box::new(|doc| {
                    doc.show.roof = !doc.show.roof;
                    doc.redraw();
                }));
            }
            // A job with nothing of a trade does not offer to show it.
            if count == 0 {
                continue;
            }
            layer(c, theme::discipline_name(d), "", theme::discipline_colour(d), doc.show.discipline(d), Some(count), Box::new(move |doc| {
                doc.show.toggle(d);
                doc.redraw();
            }));
        }
        layer(c, "See through", "X", theme::accent(), doc.show.ghost, None, Box::new(|doc| {
            doc.show.ghost = !doc.show.ghost;
            doc.redraw();
        }));
        layer(c, "Known, Assumed, Unknown", "", theme::amber(), doc.show.certainty, None, Box::new(|doc| {
            doc.show.certainty = !doc.show.certainty;
            doc.redraw();
        }));
        let found = snap.clashes.as_ref().and_then(|c| c.as_ref().ok()).map(|c| c.found.iter().filter(|x| !x.through).count()).unwrap_or(0);
        if found > 0 {
            layer(c, "Clashes", "", theme::clash(), doc.show.clashes || doc.tab == crate::model::Tab::Clashes, Some(found), Box::new(|doc| {
                doc.show.clashes = !doc.show.clashes;
                doc.redraw();
            }));
        }

        if !snap.systems.is_empty() {
            heading(c, fonts, "Systems");
            for system in &snap.systems {
                let t = &system.takeoff;
                let path = t.path.clone();
                // What is on no system, or on one nobody has named, is listed too. It is
                // not a system and cannot be selected as one.
                let act = move |doc: &mut Doc| {
                    if !path.is_empty() {
                        doc.select(&path);
                    }
                };
                row(c, !t.path.is_empty() && doc.selected.as_deref() == Some(t.path.as_str()), format!("System {}", t.name), act, |r| {
                    swatch(r, if t.path.is_empty() { theme::status_ink(t.flag) } else { theme::system_colour(&t.classification) });
                    r.spawn((text(fonts, t.name.clone(), 12.0, Weight::Medium, if t.path.is_empty() { theme::status_ink(t.flag) } else { theme::ink() }), TextLayout::no_wrap(), Pickable::IGNORE));
                    r.spawn(Node { flex_grow: 1.0, ..default() });
                    r.spawn((text(fonts, total(&t.runs), 11.0, Weight::Regular, if t.runs.members == 0 { theme::ink_faint() } else { theme::status_ink(t.runs.status) }), TextLayout::no_wrap(), Pickable::IGNORE));
                });
            }
        }
    });
}

/// A tool's mark, drawn the way a plan draws the thing: a wall is a heavy line, a
/// window a line with its glass, a duct two lines. Twenty-four by eighteen.
fn mark(p: &mut ChildSpawnerCommands, what: &str, ink: Color) {
    p.spawn((Node { width: px(24.0), height: px(18.0), flex_shrink: 0.0, ..default() }, Pickable::IGNORE)).with_children(|m| {
        let mut bar = |x: f32, y: f32, w: f32, h: f32| {
            m.spawn((Node { position_type: PositionType::Absolute, left: px(x), top: px(y), width: px(w), height: px(h), ..default() }, BackgroundColor(ink), Pickable::IGNORE));
        };
        match what {
            "Wall" => bar(2.0, 6.0, 20.0, 6.0),
            "Door" => {
                bar(2.0, 12.0, 6.0, 3.0);
                bar(16.0, 12.0, 6.0, 3.0);
                bar(8.0, 3.0, 1.5, 10.0);
            }
            "Conduit" => {
                bar(2.0, 6.5, 20.0, 1.5);
                bar(2.0, 10.0, 20.0, 1.5);
            }
            "Cable" => {
                bar(2.0, 8.0, 5.0, 2.0);
                bar(9.5, 8.0, 5.0, 2.0);
                bar(17.0, 8.0, 5.0, 2.0);
            }
            "Fitting" => {
                bar(5.0, 3.0, 3.0, 11.0);
                bar(5.0, 11.0, 14.0, 3.0);
            }
            "Move" => {
                bar(11.2, 2.0, 1.5, 14.0);
                bar(4.0, 8.2, 16.0, 1.5);
            }
            "Line up" => {
                bar(4.0, 3.0, 1.5, 12.0);
                bar(4.0, 13.5, 16.0, 1.5);
            }
            _ => {}
        }
        let mut frame = |x: f32, y: f32, w: f32, h: f32, round: f32, fill: Color| {
            m.spawn((Node { position_type: PositionType::Absolute, left: px(x), top: px(y), width: px(w), height: px(h), border: UiRect::all(px(1.5)), border_radius: BorderRadius::all(px(round)), ..default() }, BackgroundColor(fill), BorderColor::all(ink), Pickable::IGNORE));
        };
        match what {
            "Window" => frame(2.0, 6.0, 20.0, 6.0, 0.0, Color::NONE),
            "Floor" => frame(3.0, 3.0, 18.0, 12.0, 0.0, ink.with_alpha(0.22)),
            "Room" => frame(3.0, 3.0, 18.0, 12.0, 0.0, Color::NONE),
            "Duct" => frame(2.0, 5.0, 20.0, 8.0, 0.0, Color::NONE),
            "Pipe" => frame(16.0, 5.0, 8.0, 8.0, 4.0, Color::NONE),
            "Equipment" => frame(5.0, 2.0, 14.0, 14.0, 0.0, Color::NONE),
            "Fixture" => frame(6.0, 3.0, 12.0, 12.0, 6.0, Color::NONE),
            "Connect" => frame(15.0, 4.0, 7.0, 10.0, 0.0, Color::NONE),
            "Set scale" => frame(2.0, 6.0, 20.0, 6.0, 0.0, Color::NONE),
            _ => {}
        }
        let mut more = |x: f32, y: f32, w: f32, h: f32, turn: f32| {
            m.spawn((Node { position_type: PositionType::Absolute, left: px(x), top: px(y), width: px(w), height: px(h), ..default() }, UiTransform::from_rotation(Rot2::degrees(turn)), BackgroundColor(ink), Pickable::IGNORE));
        };
        match what {
            "Window" => more(2.0, 8.4, 20.0, 1.2, 0.0),
            "Room" => more(10.5, 7.5, 3.0, 3.0, 0.0),
            "Pipe" => more(2.0, 8.0, 14.0, 2.0, 0.0),
            "Equipment" => more(8.0, 5.5, 8.0, 1.5, 0.0),
            "Connect" => {
                more(3.0, 7.0, 4.0, 4.0, 0.0);
                more(7.0, 8.4, 8.0, 1.2, 0.0);
            }
            "Roof" => {
                more(1.5, 7.0, 12.5, 1.6, -32.0);
                more(10.0, 7.0, 12.5, 1.6, 32.0);
                more(2.0, 13.5, 20.0, 1.5, 0.0);
            }
            "Delete" => {
                more(4.0, 8.2, 16.0, 1.6, 45.0);
                more(4.0, 8.2, 16.0, 1.6, -45.0);
            }
            _ => {}
        }
    });
}

/// One button of the build bar: its mark, its word, and its number key.
fn tool_button<'a>(p: &'a mut ChildSpawnerCommands, fonts: &Fonts, label: &str, key: &str, on: bool) -> EntityCommands<'a> {
    let ink = if on { theme::accent() } else { theme::ink() };
    let mut e = p.spawn((
        Node { width: px(52.0), height: px(44.0), border_radius: BorderRadius::all(px(theme::BUTTON_RADIUS)), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, row_gap: px(2.0), border: UiRect::all(px(1.0)), flex_shrink: 0.0, ..default() },
        BackgroundColor(if on { theme::accent_wash() } else { theme::panel() }),
        BorderColor::all(if on { theme::accent() } else { theme::rule() }),
        named(label),
    ));
    e.with_children(|b| {
        mark(b, label, ink);
        b.spawn((text(fonts, label, 10.5, Weight::Medium, ink), TextLayout::no_wrap(), Pickable::IGNORE));
        if !key.is_empty() {
            b.spawn((Node { position_type: PositionType::Absolute, top: px(1.0), right: px(3.0), ..default() }, Pickable::IGNORE)).with_child((text(fonts, key, 9.5, Weight::Medium, theme::ink_faint()), Pickable::IGNORE));
        }
    });
    e
}

/// Along the bottom of the model: the tool in hand with its fields and what to do with
/// it, over every tool there is, grouped by what it builds.
fn build_bar(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc) {
    use crate::tools::{self, Tool};
    p.spawn(docked(Node { left: px(LEFT_W), right: px(RIGHT_W), bottom: px(STATUS_H), height: px(BUILD_H), flex_direction: FlexDirection::Column, ..default() }, UiRect::top(px(1.0)))).with_children(|bar| {
        // The tool in hand: its fields, each typed into where it stands.
        let typing = match doc.entry {
            Some((Entry::Settings(tool), _)) => Some(tool),
            _ => None,
        };
        bar.spawn((Node { height: px(46.0), padding: UiRect::horizontal(px(12.0)), align_items: AlignItems::Center, column_gap: px(10.0), border: UiRect::bottom(px(1.0)), overflow: Overflow::clip(), flex_shrink: 0.0, ..default() }, BorderColor::all(theme::rule()))).with_children(|strip| {
            let Some(tool) = typing.or(doc.draw.tool) else {
                strip.spawn(text(fonts, "Pick a tool below to draw, or click anything on the model to select it.", 11.5, Weight::Regular, theme::ink_faint()));
                return;
            };
            strip.spawn((text(fonts, tool.name(), 13.0, Weight::Semibold, theme::accent()), TextLayout::no_wrap()));
            let set = doc.draw.settings.get(tool.name());
            for (i, (label, like)) in tool.fields().iter().enumerate() {
                let rest = set.and_then(|s| s.get(i)).cloned().unwrap_or_default();
                let width = if *label == "System" || label.starts_with("What") { 170.0 } else { 96.0 };
                on_click(&mut form_field(strip, fonts, doc, typing.is_some(), i, label, like, &rest, width), move |doc| {
                    // What is typed so far stays; the caret moves to the field clicked.
                    let values = match doc.entry.clone() {
                        Some((Entry::Settings(t), typed)) if t == tool => {
                            let mut values = doc.form.clone();
                            let at = doc.form_at;
                            values[at] = typed;
                            values
                        }
                        _ => doc.draw.settings.get(tool.name()).cloned().unwrap_or_else(|| vec![String::new(); tool.takes()]),
                    };
                    doc.open_form(Entry::Settings(tool), values, i);
                });
            }
            // What is typed at the pointer while drawing: a length, a dimension, a name.
            if let Some((field @ (Entry::Length | Entry::Distance | Entry::Room), typed)) = &doc.entry {
                let (label, width) = match field {
                    Entry::Length => (if tool == Tool::Move { "How far" } else { "Length" }, 110.0),
                    Entry::Distance => ("Dimension", 110.0),
                    _ => ("Room name", 170.0),
                };
                theme::field(strip, fonts, label, typed, "", 1, width);
            }
            let hint = match &doc.entry {
                Some((Entry::Settings(_), _)) => "Tab to the next. Enter takes the tool in hand. Esc cancels.".to_string(),
                Some((Entry::Length, _)) => "Toward the pointer, like 24 ft or 12' 6\". Enter sets it.".to_string(),
                Some((Entry::Distance, _)) => "As the drawing says it, like 41' 3\" or 30 ft. Enter sets it.".to_string(),
                Some((Entry::Room, _)) => "Like Kitchen. Enter adds the room.".to_string(),
                _ => tool.how().to_string(),
            };
            strip.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), ..default() }).with_child(text(fonts, hint, 11.5, Weight::Regular, theme::ink_soft()));
        });
        // Every tool, in three groups.
        bar.spawn(Node { flex_grow: 1.0, padding: UiRect::horizontal(px(10.0)), align_items: AlignItems::Center, column_gap: px(12.0), overflow: Overflow::clip(), ..default() }).with_children(|all| {
            let in_hand = typing.or(doc.draw.tool);
            let group = |all: &mut ChildSpawnerCommands, name: &str, tools: &[Tool], delete: bool| {
                all.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(1.0), flex_shrink: 0.0, ..default() }).with_children(|g| {
                    g.spawn((text(fonts, name.to_uppercase(), 9.0, Weight::Semibold, theme::ink_faint()), TextLayout::no_wrap()));
                    g.spawn(Node { column_gap: px(3.0), ..default() }).with_children(|r| {
                        for tool in tools.iter().copied() {
                            on_click(&mut tool_button(r, fonts, tool.name(), tool.key(), in_hand == Some(tool)), move |doc| tools::start(doc, tool));
                        }
                        if delete {
                            on_click(&mut tool_button(r, fonts, "Delete", "Del", false), |doc| doc.press(KeyCode::Delete));
                        }
                    });
                });
            };
            group(all, "Building", &Tool::BUILDING, false);
            divider(all);
            group(all, "Mechanical, electrical, plumbing", &Tool::MEP, false);
            divider(all);
            group(all, "Edit", &[Tool::Move], true);
        });
    });
}

/// A label's tag: flat, hairline edge, and nothing in it takes the pointer, so a click
/// on a label still lands on the model under it.
fn pill(p: &mut ChildSpawnerCommands, label: Label, selected: bool, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn((
        Node { position_type: PositionType::Absolute, left: px(-2000.0), top: px(-2000.0), padding: UiRect::axes(px(6.0), px(2.0)), column_gap: px(6.0), align_items: AlignItems::Center, border: UiRect::all(px(1.0)), ..default() },
        UiTransform { translation: Val2::percent(-50.0, -50.0), ..default() },
        BackgroundColor(if selected { theme::accent() } else { theme::panel().with_alpha(0.94) }),
        BorderColor::all(if selected { theme::accent() } else { theme::hairline() }),
        Pickable::IGNORE,
        label,
    ))
    .with_children(build);
}

fn quiet(p: &mut ChildSpawnerCommands, fonts: &Fonts, s: impl Into<String>, size: f32, weight: Weight, colour: Color) {
    p.spawn((text(fonts, s, size, weight, colour), TextLayout::no_wrap(), Pickable::IGNORE));
}

/// What a value being typed is, and what could be typed: for the fields that open in
/// the panel on the right.
fn entry_words(doc: &Doc, snap: &Snapshot, field: Entry) -> (String, &'static str) {
    let name = snap.selected.as_ref().map(|s| s.name.clone()).unwrap_or_default();
    let title = match field {
        Entry::Pitch => format!("Pitch of {name}"),
        Entry::Move => format!("Move the eave of {name}"),
        Entry::Allowance => format!("Allowance for {name}"),
        Entry::UnitCost => format!("Unit cost of {:?}", doc.entry_line.clone().unwrap_or_default()),
        Entry::StretchOut => format!("Stretch-out of {:?}", doc.entry_line.clone().unwrap_or_default()),
        Entry::Division => format!("Division of {:?}", doc.entry_line.clone().unwrap_or_default()),
        Entry::Material => format!("Material of {name}"),
        Entry::Overhead => "Overhead on the job".to_string(),
        Entry::Profit => "Profit on the job".to_string(),
        Entry::Measure => match &doc.measuring {
            // A height is to a line: a run's to its centre unless it is held to its
            // bottom or its top, a placed thing's to its underside.
            Some((_, what, true)) if what.starts_with("height above its level") => {
                let to = match snap.selected.as_ref() {
                    Some(el) if el.kind.is_segment() => el.rows.iter().find(|r| r.label == "its heights are those of its").map(|r| r.text.clone()).unwrap_or_else(|| "centre".to_string()),
                    _ => "underside".to_string(),
                };
                format!("Measured on site, to its {to}")
            }
            Some((_, _, true)) => "Measured on site".to_string(),
            Some((_, _, false)) => "Checked on site".to_string(),
            None => "Measured on site".to_string(),
        },
        _ => String::new(),
    };
    let hint = match field {
        Entry::Pitch => "Rise in 12, like 8:12",
        Entry::Move => "Out is +, like 1' 6\" or -6 in",
        Entry::Allowance => "Dollars per SF or LF, like 28 or $28.00",
        Entry::UnitCost => "Dollars for what the line is priced in, like 31.50",
        Entry::StretchOut => "The girth of the piece, like 24 in or 2 ft",
        Entry::Division => "Its number, 1 to 16, like 7 for Thermal and Moisture Protection",
        Entry::Material => "A material of the job, by name",
        Entry::Overhead => "A percentage of the job cost, like 12",
        Entry::Profit => "A percentage of cost and overhead, like 8",
        Entry::Measure => match &doc.measuring {
            Some((_, what, true)) if what.starts_with("height above its level") && snap.selected.as_ref().is_some_and(|s| s.kind.is_segment()) => "Like bottom 7' 10 3/4\", or same",
            Some((_, _, true)) => "As the tape or laser reads, or same",
            _ => "Type same if it is as drawn",
        },
        _ => "",
    };
    (title, hint)
}

/// The value being typed in the panel on the right: what it is, the field, and what
/// could be typed.
fn entry_box(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    let Some((field, typed)) = &doc.entry else { return };
    let (title, hint) = entry_words(doc, snap, *field);
    p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::all(px(8.0)), margin: UiRect::vertical(px(6.0)), row_gap: px(5.0), border: UiRect::all(px(1.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::accent_wash()), BorderColor::all(theme::accent()))).with_children(|c| {
        theme::field(c, fonts, &title, typed, "", 1, RIGHT_W - 48.0);
        c.spawn(Node { align_items: AlignItems::Center, column_gap: px(6.0), ..default() }).with_children(|h| {
            h.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), ..default() }).with_child(text(fonts, hint, 11.0, Weight::Regular, theme::ink_soft()));
            keycap(h, fonts, "Enter");
            h.spawn(text(fonts, "sets", 11.0, Weight::Regular, theme::ink_soft()));
            keycap(h, fonts, "Esc");
            h.spawn(text(fonts, "cancels", 11.0, Weight::Regular, theme::ink_soft()));
        });
    });
}

fn overlay(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    p.spawn((Node { position_type: PositionType::Absolute, top: px(0.0), left: px(0.0), right: px(0.0), bottom: px(0.0), overflow: Overflow::clip(), ..default() }, Pickable::IGNORE)).with_children(|o| {
        let roof_on_show = doc.show.roof && !doc.show.ghost && doc.show.storey.is_none() && !doc.walk;
        // Each roof line's length, for the selected plane's own lines, or for all on plan.
        for line in snap.lines.iter().filter(|_| roof_on_show) {
            let (Some([a, b]), Some(length)) = (line.ends, line.length) else { continue };
            let chosen = doc.selected.as_ref().is_some_and(|s| *s == line.path || line.planes.contains(s));
            if !(doc.plan || chosen) {
                continue;
            }
            let at = (to_view(doc.origin, a) + to_view(doc.origin, b)) / 2.0 + Vec3::Y * 0.1;
            o.spawn((
                Node { position_type: PositionType::Absolute, left: px(-2000.0), top: px(-2000.0), padding: UiRect::axes(px(4.0), px(1.0)), ..default() },
                UiTransform { translation: Val2::percent(-50.0, -50.0), ..default() },
                BackgroundColor(theme::panel().with_alpha(0.9)),
                Pickable::IGNORE,
                Label { at, facing: None, rank: 2 },
            ))
            .with_children(|l| quiet(l, fonts, units::feet_as_ft_in(length, 16), 11.0, Weight::Medium, if line.status == Status::Known { theme::line_colour(line.kind) } else { theme::status_ink(line.status) }));
        }
        // Each facet: its number, its area, its pitch, and its flag.
        for plane in snap.planes.iter().filter(|_| roof_on_show) {
            let Some(solid) = &plane.solid else { continue };
            let c = centroid(&solid.points);
            let at = to_view(doc.origin, [c[0], c[1], solid.height_at(c)]);
            let positions: Vec<Vec3> = solid.points.iter().map(|q| to_view(doc.origin, [q[0], q[1], solid.height_at(*q)])).collect();
            let normal = facing(&positions);
            let selected = doc.selected.as_deref() == Some(plane.path.as_str());
            let (main, soft) = if selected { (Color::WHITE, Color::WHITE.with_alpha(0.82)) } else { (theme::ink(), theme::ink_soft()) };
            pill(o, Label { at: at + normal * 0.15, facing: Some(normal), rank: if selected { 0 } else { 1 } }, selected, |l| {
                quiet(l, fonts, plane.short(), 12.0, Weight::Semibold, main);
                match plane.area {
                    Some(area) => quiet(l, fonts, amount(Some(area), 0, "SF"), 12.0, Weight::Medium, if selected || plane.area_status == Status::Known { main } else { theme::status_ink(plane.area_status) }),
                    None => quiet(l, fonts, "no area", 12.0, Weight::Medium, if selected { main } else { theme::red() }),
                }
                if let Some(pitch) = plane.pitch {
                    quiet(l, fonts, theme::pitch_text(pitch), 11.5, Weight::Regular, soft);
                }
                // The flag stays on a selected facet too: a light mark on the blue.
                if plane.worst() != Status::Known {
                    let dot = if selected { theme::amber_wash() } else { theme::status_ink(plane.worst()) };
                    l.spawn((Node { width: px(6.0), height: px(6.0), ..default() }, BackgroundColor(if selected && plane.worst() == Status::Unknown { theme::red_wash() } else { dot }), Pickable::IGNORE));
                }
            });
        }
        // The selected element of the rest of the building: its name on it.
        if let Some(placed) = doc.selected.as_ref().and_then(|s| snap.drawn.iter().find(|p| p.path == *s)) {
            let at = match &placed.shape {
                Shape::Wall { a, b, base, height, .. } => to_view(doc.origin, [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, base + height]),
                Shape::Floor { boundary, top, .. } => {
                    let c = centroid(boundary);
                    to_view(doc.origin, [c[0], c[1], *top])
                }
                Shape::Run { a, b, .. } => (to_view(doc.origin, *a) + to_view(doc.origin, *b)) / 2.0,
                Shape::Box { at, size, .. } => to_view(doc.origin, [at[0], at[1], at[2] + size[2]]),
            };
            let figure = snap.selected.as_ref().and_then(|el| el.rows.iter().find(|r| r.label == "true_length" || r.label == "net_area" || r.label == "plan_area")).map(|r| (shown(r), r.status));
            pill(o, Label { at: at + Vec3::Y * 0.6, facing: None, rank: 0 }, true, |l| {
                quiet(l, fonts, placed.name.clone(), 12.0, Weight::Semibold, Color::WHITE);
                if let Some((figure, status)) = figure {
                    quiet(l, fonts, figure, 12.0, Weight::Medium, Color::WHITE.with_alpha(0.9));
                    if status != Status::Known {
                        l.spawn((Node { width: px(6.0), height: px(6.0), ..default() }, BackgroundColor(if status == Status::Unknown { theme::red_wash() } else { theme::amber_wash() }), Pickable::IGNORE));
                    }
                }
            });
        }

        // Anything the last action needs said in full: over the model, where it is seen.
        if doc.tone != Tone::Done {
            o.spawn((Node { position_type: PositionType::Absolute, top: px(TOP_H + 10.0), left: px(LEFT_W + 10.0), right: px(RIGHT_W + 10.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }, Pickable::IGNORE)).with_children(|mid| {
                let (ink, wash) = if doc.tone == Tone::Refused { (theme::red(), theme::red_wash()) } else { (theme::amber(), theme::amber_wash()) };
                mid.spawn((Node { max_width: px(620.0), flex_direction: FlexDirection::Column, padding: UiRect::axes(px(12.0), px(8.0)), row_gap: px(3.0), border: UiRect { left: px(4.0), right: px(1.0), top: px(1.0), bottom: px(1.0) }, ..default() }, BackgroundColor(wash), BorderColor::all(ink), Pickable::IGNORE)).with_children(|c| {
                    for (i, line) in doc.message.lines().enumerate() {
                        c.spawn((text(fonts, if i == 0 { pretty(line) } else { line.to_string() }, 12.5, if i == 0 { Weight::Medium } else { Weight::Regular }, ink), Pickable::IGNORE));
                    }
                });
            });
        }
    });
}

pub fn stat(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str, value: String, unit: &str, status: Status) {
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(2.0), flex_grow: 1.0, flex_basis: px(0.0), ..default() }).with_children(|s| {
        s.spawn(text(fonts, label.to_uppercase(), 9.5, Weight::Semibold, theme::ink_faint()));
        s.spawn(Node { align_items: AlignItems::Baseline, column_gap: px(4.0), ..default() }).with_children(|v| {
            v.spawn(text(fonts, value, 17.0, Weight::Semibold, theme::status_ink(status)));
            if !unit.is_empty() {
                v.spawn(text(fonts, unit, 11.0, Weight::Medium, theme::ink_soft()));
            }
        });
    });
}

/// One value: its name, its flag, what it is, and under it where it came from, or for a
/// worked-out figure, how. A long value takes its own line. `act`, when there is one,
/// is what a click on the row does ("Measure"), said at the row's foot.
fn property<'a>(p: &'a mut ChildSpawnerCommands, fonts: &Fonts, label: &str, value: String, status: Status, basis: &str, note: &str, act: &str) -> EntityCommands<'a> {
    let long = value.chars().count() > 24;
    let mut row = p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(4.0)), row_gap: px(2.0), border: UiRect::bottom(px(1.0)), flex_shrink: 0.0, ..default() }, BorderColor::all(theme::rule())));
    if !act.is_empty() {
        row.insert(Named { name: theme::spoken(label), scrolls: true });
    }
    row.with_children(|r| {
        r.spawn((Node { align_items: AlignItems::Center, column_gap: px(8.0), ..default() }, Pickable::IGNORE)).with_children(|top| {
            top.spawn((text(fonts, label, 12.0, Weight::Regular, theme::ink_soft()), TextLayout::no_wrap(), Pickable::IGNORE));
            status_chip(top, fonts, status);
            top.spawn(Node { flex_grow: 1.0, ..default() });
            if !long {
                top.spawn((text(fonts, value.clone(), 12.0, Weight::Medium, theme::status_ink(status)), TextLayout::no_wrap(), Pickable::IGNORE));
            }
        });
        if long {
            r.spawn((text(fonts, value, 12.0, Weight::Medium, theme::status_ink(status)), Pickable::IGNORE));
        }
        if !basis.is_empty() || !act.is_empty() {
            r.spawn((Node { align_items: AlignItems::FlexStart, column_gap: px(8.0), ..default() }, Pickable::IGNORE)).with_children(|foot| {
                foot.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), ..default() }).with_child((text(fonts, basis, 10.5, Weight::Regular, theme::ink_faint()), Pickable::IGNORE));
                if !act.is_empty() {
                    foot.spawn((Node { flex_shrink: 0.0, padding: UiRect::axes(px(5.0), px(0.0)), border: UiRect::all(px(1.0)), ..default() }, BorderColor::all(theme::accent()), Pickable::IGNORE)).with_child((text(fonts, act, 10.5, Weight::Semibold, theme::accent()), TextLayout::no_wrap(), Pickable::IGNORE));
                }
            });
        }
        if !note.is_empty() {
            r.spawn((text(fonts, note, 10.5, Weight::Regular, theme::status_ink(status)), Pickable::IGNORE));
        }
    });
    row
}

/// What can be done to what is selected, as buttons: each says the key that does the same.
/// Move and Delete are on the build bar, once, for everything.
fn actions(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot, el: &view::ElementRows) {
    let line = |p: &mut ChildSpawnerCommands, build: &dyn Fn(&mut ChildSpawnerCommands)| {
        p.spawn(Node { margin: UiRect::top(px(8.0)), column_gap: px(4.0), row_gap: px(4.0), flex_wrap: FlexWrap::Wrap, flex_shrink: 0.0, ..default() }).with_children(|r| build(r));
    };
    let press = |r: &mut ChildSpawnerCommands, label: &str, key: &str, code: KeyCode| {
        on_click(&mut button(r, fonts, label, key, false), move |doc| doc.press(code));
    };
    // A roof plane worked out from its roof's outline: its pitch, its eave, its gable.
    if snap.rises_from.is_some() {
        line(p, &|r| {
            press(r, "Pitch", "P", KeyCode::KeyP);
            press(r, "Steeper", "=", KeyCode::Equal);
            press(r, "Flatter", "-", KeyCode::Minus);
            press(r, "Move eave", "M", KeyCode::KeyM);
            press(r, "Eave out 6 in", "]", KeyCode::BracketRight);
            press(r, "Eave in 6 in", "[", KeyCode::BracketLeft);
            press(r, "Gable", "G", KeyCode::KeyG);
        });
    }
    // A drawing is a sheet under the tracing: it has no condition, no price and no
    // material, and is offered none.
    // Nor does a level, a system, a type, a material or a condition: lines are priced
    // on what is built, and on the job itself (its general conditions).
    let made = !matches!(el.kind, Kind::Drawing | Kind::Prior | Kind::Job | Kind::Condition | Kind::Material | Kind::Type | Kind::System | Kind::Storey);
    let built = made || el.kind == Kind::Job;
    line(p, &|r| {
        if built {
            press(r, "Assembly", "C", KeyCode::KeyC);
        }
        // Only what can carry an allowance is offered one, in the unit it is priced by.
        if let Some((_, unit)) = el.kind.priced_by() {
            press(r, &format!("Allowance, $/{unit}"), "B", KeyCode::KeyB);
        }
        if built {
            press(r, "Unit cost", "U", KeyCode::KeyU);
        }
        // Only what is built is made of a material, and carries labor of its own.
        if made {
            press(r, "Material", "N", KeyCode::KeyN);
            press(r, "Labor on or off", "L", KeyCode::KeyL);
        }
    });
    if matches!(doc.entry, Some((Entry::Pitch | Entry::Move | Entry::Allowance | Entry::UnitCost | Entry::StretchOut | Entry::Division | Entry::Material, _))) {
        entry_box(p, fonts, doc, snap);
    }
    // The assemblies that go on this kind of thing, to pick one from: each by its name
    // and how many lines it brings. The one the thing has is marked.
    if doc.choosing && built {
        let has = el.rows.iter().find(|r| r.label == "condition").map(|r| r.text.clone());
        let fit: Vec<&(String, Vec<Kind>, usize)> = snap.assemblies.iter().filter(|a| a.1.contains(&el.kind)).collect();
        heading(p, fonts, &format!("Assemblies for a {}", theme::kind_name(el.kind).to_lowercase()));
        p.spawn(Node { padding: UiRect::bottom(px(4.0)), ..default() }).with_child(text(
            fonts,
            if has.is_some() { "Click one to change to it. Lines the two share keep their prices; the rest of the old one's come off." } else { "Click one. Its cost lines go on, each to be priced on the sheet." },
            11.0,
            Weight::Regular,
            theme::ink_faint(),
        ));
        for (name, _, lines) in fit {
            let mine = has.as_deref() == Some(name.as_str());
            let pick = name.clone();
            row(p, mine, format!("Assembly {name}"), move |doc| {
                doc.give = Some(pick.clone());
                doc.touch();
            }, |r| {
                r.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), ..default() }).with_child(text(fonts, name.clone(), 12.0, if mine { Weight::Semibold } else { Weight::Regular }, theme::ink()));
                r.spawn(Node { flex_shrink: 0.0, ..default() }).with_child((text(fonts, if mine { "on it now".to_string() } else if *lines == 1 { "1 line".to_string() } else { format!("{lines} lines") }, 11.0, Weight::Regular, theme::ink_faint()), TextLayout::no_wrap()));
            });
        }
    }
}

/// Down the right: what is selected, as fields and buttons; with nothing selected, the job.
fn right(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot, plans: &crate::plans::Plans, scrolled: f32) {
    p.spawn((
        docked(Node { top: px(TOP_H), right: px(0.0), bottom: px(STATUS_H), width: px(RIGHT_W), flex_direction: FlexDirection::Column, padding: UiRect::axes(px(14.0), px(10.0)), overflow: Overflow::scroll_y(), ..default() }, UiRect::left(px(1.0))),
        ScrollPosition(Vec2::new(0.0, scrolled)),
        InspectorScroll,
    ))
    .with_children(|c| {
        // The job's overhead or profit, typed while something is selected: its field is
        // here, since the job's own values are not on show.
        if snap.selected.is_some() && matches!(doc.entry, Some((Entry::Overhead | Entry::Profit, _))) {
            entry_box(c, fonts, doc, snap);
        }
        match &snap.selected {
            Some(el) if el.kind == Kind::System => system(c, fonts, snap, el),
            Some(el) if el.kind == Kind::Drawing => {
                selection(c, fonts, doc, snap, el);
                drawing(c, fonts, snap, plans, el);
            }
            Some(el) => selection(c, fonts, doc, snap, el),
            None => summary(c, fonts, doc, snap),
        }
        c.spawn(Node { margin: UiRect::top(px(12.0)), flex_shrink: 0.0, ..default() }).with_child(text(fonts, "Right-drag turns the model. Middle-drag slides it. The wheel zooms. W A S D move over the ground.", 10.5, Weight::Regular, theme::ink_faint()));
    });
}

/// A selected drawing: its scale, how big its sheet is on the ground, what there is on
/// it to snap to, and everything its file could not give.
fn drawing(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot, plans: &crate::plans::Plans, el: &view::ElementRows) {
    let Some(l) = snap.drawings.iter().find(|l| l.path == el.path) else { return };
    let line = |p: &mut ChildSpawnerCommands, said: String, ink: Color| {
        p.spawn(Node { margin: UiRect::top(px(4.0)), ..default() }).with_child(text(fonts, said, 12.0, Weight::Regular, ink));
    };
    heading(p, fonts, "Scale");
    match l.scale_said() {
        Some(said) => line(p, said, theme::status_ink(l.drawing.status())),
        None => line(p, "No scale yet. Click Set scale, then the two ends of one dimension on it. Nothing is traced over this drawing until then.".to_string(), theme::red()),
    }
    if let Some(s) = &l.drawing.scale {
        line(p, format!("from a dimension of {} ({})", s.distance.value.as_ref().map(|d| units::show_length(d, &s.distance.unit)).unwrap_or_default(), s.distance.source), theme::ink_soft());
    }
    heading(p, fonts, "File");
    line(p, format!("{}, page {}", l.drawing.file.value.clone().unwrap_or_default(), l.drawing.page), theme::ink_soft());
    if plans.is_reading(&l.path) {
        line(p, "Its page is being read.".to_string(), theme::ink_soft());
    }
    if let Some((_, why)) = plans.faults.get(&l.path) {
        line(p, format!("Its page cannot be shown: {why}"), theme::red());
    }
    if let Some(read) = plans.read.get(&l.path) {
        if read.changed {
            line(p, "The file has changed since this drawing was added. What was traced over it may no longer lie on it.".to_string(), theme::red());
        }
        line(p, match read.lines.len() { 0 => "Nothing on it to snap to.".to_string(), 1 => "1 straight line to snap to.".to_string(), n => format!("{n} straight lines to snap to.") }, theme::ink_soft());
        if !read.lost.is_empty() {
            heading(p, fonts, "Not read from it");
            for lost in &read.lost {
                line(p, lost.clone(), theme::ink_soft());
            }
        }
    }
}

fn title_block(p: &mut ChildSpawnerCommands, fonts: &Fonts, kind: &str, name: &str, under: String) {
    p.spawn(text(fonts, kind.to_uppercase(), 10.0, Weight::Semibold, theme::ink_faint()));
    p.spawn(Node { margin: UiRect::top(px(2.0)), ..default() }).with_child(theme::name(fonts, name, 17.0, Weight::Semibold, theme::ink()));
    if !under.is_empty() {
        p.spawn(Node { margin: UiRect::top(px(2.0)), ..default() }).with_child(text(fonts, under, 11.5, Weight::Regular, theme::ink_faint()));
    }
}

/// A line of a list: something on the left, a figure on the right.
fn listed(p: &mut ChildSpawnerCommands, fonts: &Fonts, colour: Option<Color>, name: &str, count: String, figure: String, status: Status) {
    p.spawn((Node { align_items: AlignItems::Center, column_gap: px(8.0), padding: UiRect::vertical(px(4.0)), border: UiRect::bottom(px(1.0)), flex_shrink: 0.0, ..default() }, BorderColor::all(theme::rule()))).with_children(|r| {
        if let Some(colour) = colour {
            swatch(r, colour);
        }
        // A long name wraps; the figure beside it keeps its place.
        r.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), column_gap: px(7.0), flex_wrap: FlexWrap::Wrap, align_items: AlignItems::Baseline, ..default() }).with_children(|n| {
            n.spawn(text(fonts, name, 12.5, Weight::Medium, theme::ink()));
            if !count.is_empty() {
                n.spawn((text(fonts, count, 12.0, Weight::Regular, theme::ink_faint()), TextLayout::no_wrap()));
            }
        });
        status_chip(r, fonts, status);
        r.spawn((text(fonts, figure, 12.5, Weight::Medium, theme::status_ink(status)), TextLayout::no_wrap()));
    });
}

/// Nothing selected: what the job comes to.
fn summary(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    title_block(p, fonts, "Job", &snap.title, String::new());
    if snap.storeys.is_empty() && snap.totals.is_none() {
        p.spawn(Node { margin: UiRect::top(px(10.0)), ..default() }).with_child(text(fonts, "Nothing is drawn yet. Next steps, on the left, says where to start.", 12.0, Weight::Regular, theme::ink_soft()));
    }
    // The job's markup: each is typed into here.
    if let Some(est) = &snap.estimate {
        heading(p, fonts, "Markup");
        for (label, key, code, field, d) in [("Overhead", "O", KeyCode::KeyO, Entry::Overhead, est.markup.as_ref().map(|m| &m.overhead)), ("Profit", "R", KeyCode::KeyR, Entry::Profit, est.markup.as_ref().map(|m| &m.profit))] {
            let (said, status) = match d.and_then(|v| v.value.map(|x| (x, v.status))) {
                Some((x, status)) => (format!("{} %", view::figure(x, 2)), status),
                None => ("not set".to_string(), Status::Unknown),
            };
            on_click(&mut property(p, fonts, label, said, status, d.map(|v| v.source.as_str()).unwrap_or(""), "", "Set"), move |doc| doc.press(code));
            let _ = key;
            if matches!(doc.entry, Some((f, _)) if f == field) {
                entry_box(p, fonts, doc, snap);
            }
        }
    }
    for storey in &snap.storeys {
        heading(p, fonts, &format!("Level: {}", storey.name));
        listed(p, fonts, None, "Walls", storey.walls.members.to_string(), total(&storey.walls), if storey.walls.members == 0 { Status::Known } else { storey.walls.status });
        listed(p, fonts, None, "Wall face, less openings", String::new(), total(&storey.wall_net), if storey.wall_net.members == 0 { Status::Known } else { storey.wall_net.status });
        listed(p, fonts, None, "Floor", storey.floors.members.to_string(), total(&storey.floors), if storey.floors.members == 0 { Status::Known } else { storey.floors.status });
        if storey.rooms.members > 0 {
            listed(p, fonts, None, "Rooms, inside their walls", storey.rooms.members.to_string(), total(&storey.rooms), storey.rooms.status);
        }
        listed(p, fonts, None, "Doors and windows", String::new(), format!("{} and {}", storey.doors, storey.windows), Status::Known);
    }
    if let Some(totals) = &snap.totals {
        heading(p, fonts, &format!("Roof: {}", snap.roof_name));
        let squares = view::in_squares(&totals.surface);
        let big = |sum: &view::Sum| match (sum.value, sum.members) {
            (_, 0) => ("none".to_string(), ""),
            (Some(x), _) => (view::figure_shown(x, 2), sum.unit),
            (None, _) => ("no total".to_string(), ""),
        };
        p.spawn(Node { margin: UiRect::top(px(6.0)), column_gap: px(12.0), ..default() }).with_children(|r| {
            let (s, u) = big(&totals.surface);
            stat(r, fonts, "Surface", s, u, totals.surface.status);
            let (s, u) = big(&squares);
            stat(r, fonts, "Squares", s, u, squares.status);
        });
        p.spawn(Node { margin: UiRect::top(px(12.0)), column_gap: px(12.0), ..default() }).with_children(|r| {
            match totals.predominant_pitch {
                Some((pitch, status)) => stat(r, fonts, "Predominant pitch", theme::pitch_text(pitch), "", status),
                None => stat(r, fonts, "Predominant pitch", "not said".into(), "", Status::Unknown),
            }
            stat(r, fonts, "Facets", totals.facets.to_string(), "", Status::Known);
        });
        if totals.predominant_pitch.is_none() {
            p.spawn(Node { margin: UiRect::top(px(6.0)), ..default() }).with_child(text(fonts, format!("No predominant pitch: {}.", totals.pitch_note), 11.5, Weight::Regular, theme::red()));
        }
        if !totals.surface.missing.is_empty() {
            p.spawn(Node { margin: UiRect::top(px(6.0)), ..default() }).with_child(text(fonts, format!("No surface total: {} of {} planes have no area.", totals.surface.missing.len(), totals.surface.members), 11.5, Weight::Regular, theme::red()));
        }
        for (kind, sum) in &totals.lines {
            listed(p, fonts, Some(theme::line_colour(*kind)), theme::kind_name(*kind), sum.members.to_string(), total(sum), sum.status);
        }
        if snap.lines.iter().any(|l| l.length.is_none()) {
            listed(p, fonts, Some(theme::red()), "No length worked out", String::new(), "drawn broken".into(), Status::Unknown);
        }
    }
    if !snap.notes.is_empty() {
        heading(p, fonts, "To verify");
        for (status, what, why) in &snap.notes {
            p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(5.0)), row_gap: px(2.0), border: UiRect::bottom(px(1.0)), flex_shrink: 0.0, ..default() }, BorderColor::all(theme::rule()))).with_children(|r| {
                r.spawn(Node { ..default() }).with_children(|c| status_chip(c, fonts, *status));
                r.spawn(text(fonts, what.clone(), 12.0, Weight::Medium, theme::status_ink(*status)));
                r.spawn(text(fonts, why.clone(), 10.5, Weight::Regular, theme::ink_faint()));
            });
        }
    }
}

/// A selected system: what it is and what it comes to.
fn system(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot, el: &view::ElementRows) {
    let Some(t) = snap.system(&el.path) else { return };
    title_block(p, fonts, &format!("{} system", theme::discipline_name(t.discipline)), &t.name, t.classification.clone());
    p.spawn(Node { margin: UiRect::top(px(16.0)), column_gap: px(12.0), ..default() }).with_children(|r| {
        match (t.runs.value, t.runs.members) {
            (_, 0) => stat(r, fonts, "Runs", "none".into(), "", Status::Known),
            (Some(x), _) => stat(r, fonts, "Runs", view::figure_shown(x, 2), "LF", t.runs.status),
            (None, _) => stat(r, fonts, "Runs", "no total".into(), "", Status::Unknown),
        }
        stat(r, fonts, "Pieces", t.runs.members.to_string(), "", Status::Known);
    });
    takeoff(p, fonts, t);
}

/// A system's takeoff: its runs by kind and size, its fittings, then what is set on it.
pub fn takeoff(p: &mut ChildSpawnerCommands, fonts: &Fonts, t: &SystemTakeoff) {
    if !t.lines.is_empty() {
        heading(p, fonts, "Runs by size");
        for line in &t.lines {
            p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(6.0)), row_gap: px(2.0), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::rule()))).with_children(|r| {
                r.spawn(Node { align_items: AlignItems::Center, column_gap: px(8.0), ..default() }).with_children(|top| {
                    swatch(top, theme::system_colour(&t.classification));
                    top.spawn((text(fonts, format!("{}, {}", theme::kind_name(line.kind), line.size), 12.5, Weight::Medium, theme::status_ink(line.size_status)), TextLayout::no_wrap()));
                    top.spawn(Node { flex_grow: 1.0, ..default() });
                    top.spawn((text(fonts, total(&line.length), 12.5, Weight::Medium, theme::status_ink(line.length.status)), TextLayout::no_wrap()));
                });
                r.spawn(Node { align_items: AlignItems::Center, column_gap: px(8.0), ..default() }).with_children(|under| {
                    under.spawn(text(fonts, format!("{} straight pieces", line.length.members), 11.0, Weight::Regular, theme::ink_faint()));
                    under.spawn(Node { flex_grow: 1.0, ..default() });
                    status_chip(under, fonts, line.size_status.max(line.length.status));
                });
            });
        }
    }
    // How it hangs together: what is joined, what is open, where a tee is not entered.
    // Each is a fact about the job as entered, so none is flagged as a doubtful value;
    // only what could not be checked is flagged, as not known.
    if t.runs.members > 0 {
        let j = &t.joined;
        heading(p, fonts, "Connections");
        let plural = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        listed(p, fonts, None, &plural(j.networks, "joined length", "joined lengths"), "each is runs joined end to end, through fittings".into(), String::new(), plumbline_world::Status::Known);
        if !j.no_fitting.is_empty() {
            listed(p, fonts, None, &plural(j.no_fitting.len(), "join with no fitting", "joins with no fitting"), "a run meets another along its length".into(), String::new(), plumbline_world::Status::Known);
        }
        for o in &j.open {
            listed(p, fonts, None, &o.said, "open end".into(), format!("{}, {}, {}", view::num(o.at[0]), view::num(o.at[1]), view::num(o.at[2])), plumbline_world::Status::Known);
        }
        for warning in &j.warnings {
            listed(p, fonts, None, warning, "to look at".into(), String::new(), plumbline_world::Status::Known);
        }
        for (path, why) in &j.left_out {
            listed(p, fonts, None, &format!("{path}: {why}"), "not checked".into(), String::new(), plumbline_world::Status::Unknown);
        }
    }
    if !t.fittings.is_empty() {
        heading(p, fonts, "Fittings");
        for (kind, what, count, status) in &t.fittings {
            listed(p, fonts, None, what, kind.map(|k| k.title()).unwrap_or("fitting").to_lowercase(), format!("{count} EA"), *status);
        }
    }
    if !t.placed.is_empty() {
        heading(p, fonts, "Equipment and fixtures");
        for (kind, what, count, status) in &t.placed {
            listed(p, fonts, None, what, theme::kind_name(*kind).to_lowercase(), format!("{count} EA"), *status);
        }
    }
}

fn selection(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot, el: &view::ElementRows) {
    let under = match &snap.rises_from {
        Some((_, edge)) => format!("rises from edge {edge} of {}", snap.roof_name),
        None => String::new(),
    };
    title_block(p, fonts, theme::kind_name(el.kind), &el.name, under);
    let find = |label: &str| el.rows.iter().find(|r| r.label == label);
    // The two figures that matter most for this kind of element, large.
    let lead: &[(&str, &str)] = match el.kind {
        k if k.is_area() => &[("surface_area", "Surface")],
        Kind::Parapet => &[("plan_length", "Length"), ("face_area", "Face")],
        k if k.is_run() => &[("true_length", "Length")],
        Kind::Wall => &[("net_area", "Net face"), ("plan_length", "Length")],
        Kind::Door | Kind::Window => &[("opening_area", "Opening")],
        Kind::Floor => &[("plan_area", "Area")],
        Kind::Room => &[("floor_area", "Floor area"), ("perimeter", "Perimeter")],
        Kind::Duct => &[("true_length", "Length"), ("surface_area", "Duct surface")],
        k if k.is_segment() => &[("true_length", "Length")],
        _ => &[],
    };
    if !lead.is_empty() {
        p.spawn(Node { margin: UiRect::top(px(10.0)), column_gap: px(12.0), flex_shrink: 0.0, ..default() }).with_children(|r| {
            for (measure, label) in lead {
                match find(measure) {
                    Some(row) => stat(r, fonts, label, row.number.map(|x| view::figure_shown(x, if row.unit == "EA" { 0 } else { 2 })).unwrap_or("unknown".into()), if row.number.is_some() { &row.unit } else { "" }, row.status),
                    None => stat(r, fonts, label, "unknown".into(), "", Status::Unknown),
                }
            }
            if el.kind.is_area() {
                match find("pitch") {
                    Some(row) => stat(r, fonts, "Pitch", theme::trade(&row.text), "", row.status),
                    None => stat(r, fonts, "Pitch", "none".into(), "", Status::Known),
                }
            } else if lead.len() == 1 && el.kind.is_run() {
                let row = find("true_length");
                stat(r, fonts, "On the tape", row.and_then(|r| r.number).map(|x| units::feet_as_ft_in(x, 16)).unwrap_or("unknown".into()), "", row.map(|r| r.status).unwrap_or(Status::Unknown));
            }
        });
    }
    // What it is dressed in, and what priced on it says so (call 58).
    let finish = plumbline_world::finish::said_of(&snap.dress, &el.path, el.kind);
    if !finish.is_empty() {
        heading(p, fonts, "Finish");
        for (what, why) in finish {
            p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(4.0)), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::rule()))).with_children(|r| {
                r.spawn(text(fonts, what, 12.5, Weight::Medium, theme::ink()));
                r.spawn(text(fonts, why, 11.0, Weight::Regular, theme::ink_faint()));
            });
        }
    }
    actions(p, fonts, doc, snap, el);
    // The plane's corners, from the model, to the sixteenth of an inch.
    if !snap.corners.is_empty() {
        heading(p, fonts, "Corners");
        let column = |r: &mut ChildSpawnerCommands, s: String, width: f32, weight: Weight, colour: Color| {
            r.spawn(Node { width: px(width), justify_content: if width > 30.0 { JustifyContent::FlexEnd } else { JustifyContent::FlexStart }, ..default() }).with_child((text(fonts, s, 12.0, weight, colour), TextLayout::no_wrap()));
        };
        p.spawn(Node { padding: UiRect::bottom(px(3.0)), ..default() }).with_children(|r| {
            column(r, String::new(), 22.0, Weight::Regular, theme::ink_faint());
            for name in ["EAST", "NORTH", "HEIGHT"] {
                column(r, name.into(), 96.0, Weight::Semibold, theme::ink_faint());
            }
        });
        for (i, c) in snap.corners.iter().enumerate() {
            p.spawn((Node { padding: UiRect::vertical(px(4.0)), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::rule()))).with_children(|r| {
                column(r, format!("{}", i + 1), 22.0, Weight::Regular, theme::ink_faint());
                for v in c {
                    column(r, units::feet_as_ft_in(*v, 16), 96.0, Weight::Medium, theme::ink());
                }
            });
        }
    }
    // A cost line's own values are named "what: line"; everything else is the element's.
    let (own, costed): (Vec<&view::Row>, Vec<&view::Row>) = el.rows.iter().partition(|r| !r.label.contains(": "));
    if own.iter().any(|r| !r.derived) {
        heading(p, fonts, "As drawn");
        for row in own.iter().filter(|r| !r.derived) {
            // A value a measurement can be put on says so, and a click on it asks for
            // what was read: a length is measured, anything else is checked.
            match snap.takes.iter().find(|t| t.0 == row.label) {
                None => {
                    property(p, fonts, &pretty(&row.label), shown(row), row.status, &row.basis, "", "");
                }
                Some((what, length)) => {
                    let (path, what, length) = (el.path.clone(), what.clone(), *length);
                    let open = matches!(doc.entry, Some((Entry::Measure, _))) && doc.measuring.as_ref().is_some_and(|m| m.0 == path && m.1 == what);
                    on_click(&mut property(p, fonts, &pretty(&row.label), shown(row), row.status, &row.basis, "", if length { "Measure" } else { "Check" }), move |doc| {
                        doc.open(Entry::Measure, "");
                        doc.measuring = Some((path.clone(), what.clone(), length));
                    });
                    // What is read is typed right under the value it is read for.
                    if open {
                        entry_box(p, fonts, doc, snap);
                    }
                }
            }
        }
    }
    if own.iter().any(|r| r.derived) {
        heading(p, fonts, "Takeoff");
        for row in own.iter().filter(|r| r.derived) {
            property(p, fonts, &pretty(&row.label), shown(row), row.status, &row.basis, &row.note, "");
        }
    }
    if !costed.is_empty() {
        heading(p, fonts, "Cost lines");
        let mut lines: Vec<&str> = Vec::new();
        for row in &costed {
            let line = row.label.split_once(": ").map(|x| x.1).unwrap_or("");
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        for line in lines {
            p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(px(8.0), px(6.0)), margin: UiRect::bottom(px(6.0)), row_gap: px(4.0), border: UiRect::all(px(1.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::panel_alt()), BorderColor::all(theme::rule()))).with_children(|c| {
                c.spawn(Node { margin: UiRect::bottom(px(2.0)), ..default() }).with_child(text(fonts, line, 12.5, Weight::Semibold, theme::ink()));
                for row in costed.iter().filter(|r| r.label.split_once(": ").is_some_and(|x| x.1 == line)) {
                    let what = row.label.split_once(": ").map(|x| x.0).unwrap_or("");
                    // Each step keeps its flag, and under it what it rests on.
                    c.spawn(Node { align_items: AlignItems::Center, column_gap: px(8.0), ..default() }).with_children(|r| {
                        r.spawn(text(fonts, pretty(what), 12.0, if row.derived { Weight::Medium } else { Weight::Regular }, if row.derived { theme::ink() } else { theme::ink_soft() }));
                        status_chip(r, fonts, row.status);
                        r.spawn(Node { flex_grow: 1.0, ..default() });
                        r.spawn(text(fonts, shown(row), 12.0, Weight::Medium, theme::status_ink(row.status)));
                    });
                    let basis = if row.note.is_empty() { row.basis.clone() } else { format!("{}  ({})", row.basis, row.note) };
                    if !basis.is_empty() {
                        c.spawn(text(fonts, basis, 10.5, Weight::Regular, theme::ink_faint()));
                    }
                }
            });
        }
    }
}

/// Keep each label at its point of the model as the view moves. Where two would lie over
/// each other, the one nearer the eye stays and the other waits until there is room.
pub fn place_labels(doc: Res<Doc>, camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>, mut labels: Query<(Entity, &Label, &mut Node, &mut Visibility, &ComputedNode)>) {
    let Ok((camera, eye)) = camera.single() else { return };
    let mut wanted: Vec<(Entity, u8, f32, Vec2, Vec2)> = Vec::new();
    for (entity, label, _, _, computed) in &labels {
        let towards = if doc.plan { Vec3::Y } else { eye.translation() - label.at };
        if label.facing.is_some_and(|n| n.dot(towards) <= 0.0) {
            continue;
        }
        let Ok(at) = camera.world_to_viewport(eye, label.at) else { continue };
        let half = computed.size * computed.inverse_scale_factor / 2.0 + Vec2::splat(2.0);
        wanted.push((entity, label.rank, eye.translation().distance(label.at), at, half));
    }
    wanted.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.total_cmp(&b.2)));
    let mut kept: Vec<(Entity, Vec2, Vec2)> = Vec::new();
    for (entity, _, _, at, half) in wanted {
        let clear = kept.iter().all(|(_, other, size)| (at.x - other.x).abs() > half.x + size.x || (at.y - other.y).abs() > half.y + size.y);
        if clear {
            kept.push((entity, at, half));
        }
    }
    for (entity, _, mut node, mut visibility, _) in &mut labels {
        match kept.iter().find(|k| k.0 == entity) {
            Some((_, at, _)) => {
                let (left, top) = (px(at.x.round()), px(at.y.round()));
                if node.left != left || node.top != top {
                    node.left = left;
                    node.top = top;
                }
                if *visibility != Visibility::Inherited {
                    *visibility = Visibility::Inherited;
                }
            }
            None => {
                if *visibility != Visibility::Hidden {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

/// Where the pointer is on the model, to the sixteenth, in the status bar.
pub fn write_pointer(doc: Res<Doc>, mut out: Query<&mut Text, With<PointerText>>) {
    let Ok(mut text) = out.single_mut() else { return };
    // With a tool in hand, what the next pick would snap to is said beside where it is.
    let snapped = doc.draw.found.as_ref().map(|f| f.kind.name()).filter(|n| !n.is_empty()).map(|n| format!("     {n}")).unwrap_or_default();
    let s = match doc.pointer {
        Some(p) => format!("E {}     N {}     H {}{snapped}", units::feet_as_ft_in(p[0], 16), units::feet_as_ft_in(p[1], 16), units::feet_as_ft_in(p[2], 16)),
        None => String::new(),
    };
    // Through the one place every word on screen is said, as all other text is.
    let s = theme::spoken(&s);
    if text.0 != s {
        text.0 = s;
    }
}

/// The wheel scrolls the panel the pointer is over.
#[allow(clippy::type_complexity)]
pub fn scroll_panels(
    scroll: Res<AccumulatedMouseScroll>,
    windows: Query<(&Window, Has<PrimaryWindow>)>,
    mut scrolls: ResMut<Scrolls>,
    mut inspector: Query<(&mut ScrollPosition, &ComputedNode), (With<InspectorScroll>, Without<SheetScroll>, Without<BrowserScroll>)>,
    mut sheet: Query<(&mut ScrollPosition, &ComputedNode), (With<SheetScroll>, Without<InspectorScroll>, Without<BrowserScroll>)>,
    mut browser: Query<(&mut ScrollPosition, &ComputedNode), (With<BrowserScroll>, Without<InspectorScroll>, Without<SheetScroll>)>,
) {
    if scroll.delta.y == 0.0 {
        return;
    }
    let by = if scroll.unit == MouseScrollUnit::Pixel { scroll.delta.y } else { scroll.delta.y * 36.0 };
    for (window, primary) in &windows {
        let Some(at) = window.cursor_position() else { continue };
        let limit = |node: &ComputedNode| ((node.content_size.y - node.size.y) * node.inverse_scale_factor).max(0.0);
        if primary {
            if at.x > window.width() - RIGHT_W && at.y > TOP_H {
                if let Ok((mut position, node)) = inspector.single_mut() {
                    scrolls.inspector = (scrolls.inspector - by).clamp(0.0, limit(node));
                    position.0.y = scrolls.inspector;
                }
            } else if at.x < LEFT_W && at.y > TOP_H {
                if let Ok((mut position, node)) = browser.single_mut() {
                    scrolls.browser = (scrolls.browser - by).clamp(0.0, limit(node));
                    position.0.y = scrolls.browser;
                }
            }
        } else if let Ok((mut position, node)) = sheet.single_mut() {
            scrolls.sheet = (scrolls.sheet - by).clamp(0.0, limit(node));
            position.0.y = scrolls.sheet;
        }
    }
}
