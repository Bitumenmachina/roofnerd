//! The main window's panels, floating over the model: the title bar, the model browser on
//! the left (storeys, disciplines, systems), the inspector on the right, the status bar,
//! and what is written on the model itself.
//!
//! Everything here is drawn from the snapshot. A panel shows what the world holds and how
//! sure it is; it works nothing out.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::model::{to_view, Doc, Entry, Snapshot, Tone};
use crate::scene::{facing, MainCamera, BAR_H, BROWSER_W, GAP, INSPECTOR_W, STATUS_H};
use crate::sheet::SheetScroll;
use crate::theme::{self, chip, heading, keycap, rounded, status_chip, swatch, text, Fonts, Weight};
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
}

fn divider(p: &mut ChildSpawnerCommands) {
    p.spawn((Node { width: px(1.0), height: px(22.0), margin: UiRect::horizontal(px(4.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::hairline())));
}

/// The wordmark: a plumb line, and the name.
pub fn wordmark(p: &mut ChildSpawnerCommands, fonts: &Fonts) {
    p.spawn(Node { align_items: AlignItems::Center, column_gap: px(9.0), flex_shrink: 0.0, ..default() }).with_children(|m| {
        m.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }).with_children(|bob| {
            bob.spawn((Node { width: px(2.0), height: px(13.0), ..default() }, BackgroundColor(theme::accent())));
            bob.spawn((rounded(Node { width: px(8.0), height: px(8.0), ..default() }, 4.0), BackgroundColor(theme::accent())));
        });
        m.spawn(text(fonts, "plumbline", 16.5, Weight::Semibold, theme::ink()));
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
    let s = label.replace('_', " ");
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

fn floating(mut node: Node) -> impl Bundle {
    node.position_type = PositionType::Absolute;
    node.border = UiRect::all(px(1.0));
    node.border_radius = BorderRadius::all(px(12.0));
    (
        node,
        BackgroundColor(theme::panel().with_alpha(0.955)),
        BorderColor::all(theme::hairline().with_alpha(0.9)),
        BoxShadow(vec![ShadowStyle { color: Color::srgba(0.05, 0.08, 0.14, 0.22), x_offset: px(0.0), y_offset: px(6.0), spread_radius: px(0.0), blur_radius: px(22.0) }]),
    )
}

/// Draw the panels again whenever the snapshot has changed.
#[allow(clippy::too_many_arguments)]
pub fn rebuild(mut commands: Commands, doc: Res<Doc>, snap: Res<Snapshot>, fonts: Res<Fonts>, cameras: Res<Cameras>, scrolls: Res<Scrolls>, old: Query<Entity, With<MainUi>>, mut drawn: Local<u64>) {
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
        title_bar(root, fonts, &doc, &snap);
        browser(root, fonts, &doc, &snap);
        inspector(root, fonts, &doc, &snap, scrolls.inspector);
        status_bar(root, fonts, &doc);
    });
}

fn title_bar(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    p.spawn(floating(Node { top: px(GAP), left: px(GAP), right: px(GAP), height: px(BAR_H), padding: UiRect::horizontal(px(18.0)), align_items: AlignItems::Center, column_gap: px(12.0), overflow: Overflow::clip(), ..default() })).with_children(|bar| {
        wordmark(bar, fonts);
        divider(bar);
        bar.spawn((text(fonts, snap.title.clone(), 13.5, Weight::Semibold, theme::ink()), TextLayout::no_wrap()));
        bar.spawn(Node { flex_grow: 1.0, ..default() });
        tally_chips(bar, fonts, snap.tally);
        divider(bar);
        bar.spawn((text(fonts, snap.file.clone(), 12.5, Weight::Regular, theme::ink_soft()), TextLayout::no_wrap()));
        if doc.edited {
            chip(bar, fonts, "Not saved", theme::amber(), theme::amber_wash());
        } else {
            chip(bar, fonts, "Saved", theme::green(), theme::hex(0xE3F4EA));
        }
    });
}

fn status_bar(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc) {
    let first = one_line(&doc.message, 150);
    let (dot, ink) = tone_ink(doc.tone);
    p.spawn(floating(Node { bottom: px(GAP), left: px(GAP), right: px(GAP), height: px(STATUS_H), padding: UiRect::horizontal(px(16.0)), align_items: AlignItems::Center, column_gap: px(9.0), overflow: Overflow::clip(), ..default() })).with_children(|bar| {
        bar.spawn((rounded(Node { width: px(7.0), height: px(7.0), flex_shrink: 0.0, ..default() }, 4.0), BackgroundColor(dot)));
        bar.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), overflow: Overflow::clip(), ..default() }).with_child((text(fonts, first, 12.0, Weight::Regular, ink), TextLayout::no_wrap()));
        bar.spawn(Node { flex_shrink: 0.0, ..default() }).with_child((text(fonts, "", 12.0, Weight::Regular, theme::ink_soft()), TextLayout::no_wrap(), PointerText));
        bar.spawn((Node { width: px(1.0), height: px(16.0), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::hairline())));
        bar.spawn(Node { flex_shrink: 0.0, ..default() }).with_child(text(fonts, if doc.walk { "On foot" } else if doc.plan { "Plan" } else { "3D" }, 12.0, Weight::Medium, theme::ink_soft()));
    });
}

/// A row of the browser that does something when it is clicked.
fn browser_row(p: &mut ChildSpawnerCommands, on: bool, act: impl Fn(&mut Doc) + Send + Sync + 'static, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn((rounded(Node { align_items: AlignItems::Center, column_gap: px(8.0), padding: UiRect::axes(px(8.0), px(5.0)), min_height: px(27.0), ..default() }, 7.0), BackgroundColor(if on { theme::accent_wash() } else { Color::NONE })))
        .with_children(build)
        .observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
            if press.button == PointerButton::Primary && doc.entry.is_none() {
                act(&mut doc);
            }
        });
}

/// What of the building is on show, and its systems. The browser is the legend too: a
/// system's swatch here is the colour its runs are drawn in.
fn browser(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    p.spawn(floating(Node { top: px(BAR_H + GAP * 2.0), left: px(GAP), width: px(BROWSER_W), max_height: percent(86.0), flex_direction: FlexDirection::Column, padding: UiRect::axes(px(10.0), px(10.0)), overflow: Overflow::clip(), ..default() })).with_children(|c| {
        // The view.
        c.spawn((rounded(Node { padding: UiRect::all(px(3.0)), column_gap: px(2.0), ..default() }, 9.0), BackgroundColor(theme::panel_alt()))).with_children(|t| {
            for (label, plan, walk) in [("3D", false, false), ("Plan", true, false), ("On foot", false, true)] {
                let on = doc.plan == plan && doc.walk == walk;
                t.spawn((rounded(Node { flex_grow: 1.0, justify_content: JustifyContent::Center, padding: UiRect::axes(px(6.0), px(5.0)), ..default() }, 7.0), BackgroundColor(if on { theme::panel() } else { Color::NONE })))
                    .with_child(text(fonts, label, 12.0, Weight::Medium, if on { theme::accent() } else { theme::ink_soft() }))
                    .observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>, mut orbit: ResMut<crate::scene::Orbit>, snap: Res<Snapshot>| {
                        if press.button == PointerButton::Primary {
                            crate::set_view(&mut doc, &mut orbit, &snap, plan, walk);
                        }
                    });
            }
        });

        if !snap.storeys.is_empty() {
            heading(c, fonts, "Storeys");
            browser_row(
                c,
                doc.show.storey.is_none(),
                |doc| {
                    doc.show.storey = None;
                    doc.redraw();
                },
                |r| {
                    r.spawn(text(fonts, "Whole building", 12.5, Weight::Medium, theme::ink()));
                },
            );
            for storey in &snap.storeys {
                let path = storey.path.clone();
                browser_row(
                    c,
                    doc.show.storey.as_deref() == Some(storey.path.as_str()),
                    move |doc| {
                        doc.show.storey = Some(path.clone());
                        doc.redraw();
                    },
                    |r| {
                        r.spawn(text(fonts, pretty(&storey.name), 12.5, Weight::Medium, theme::ink()));
                        r.spawn(Node { flex_grow: 1.0, ..default() });
                        r.spawn(text(fonts, storey.elevation.map(|e| units::feet_as_ft_in(e, 16)).unwrap_or("height unknown".into()), 11.5, Weight::Regular, if storey.elevation.is_some() { theme::ink_faint() } else { theme::red() }));
                    },
                );
            }
        }

        // Drawing, on the plan: each tool with what it draws with, as it was typed. A tool
        // that has not been told what to draw with asks before it draws.
        if doc.plan {
            use crate::tools::{self, Tool};
            heading(c, fonts, "Draw");
            let plain = |c: &mut ChildSpawnerCommands, said: &str, act: Box<dyn Fn(&mut Doc) + Send + Sync>| {
                browser_row(c, false, move |doc| act(doc), |r| {
                    r.spawn(text(fonts, said, 12.5, Weight::Medium, theme::ink()));
                });
            };
            if doc.show.storey.is_none() {
                let said = if snap.storeys.is_empty() { "This job has no storey to draw on yet." } else { "Choose the storey to draw on, above." };
                c.spawn(Node { padding: UiRect::axes(px(8.0), px(4.0)), ..default() }).with_child(text(fonts, said, 11.5, Weight::Regular, theme::ink_soft()));
            } else {
                for tool in Tool::DRAWING {
                    let said = doc.draw.settings.get(tool.name()).cloned().unwrap_or_else(|| "not set yet".to_string());
                    browser_row(c, doc.draw.tool == Some(tool), move |doc| tools::start(doc, tool), |r| {
                        r.spawn(Node { flex_direction: FlexDirection::Column, min_width: px(0.0), flex_grow: 1.0, flex_basis: px(0.0), overflow: Overflow::clip(), ..default() }).with_children(|col| {
                            col.spawn(text(fonts, tool.name(), 12.5, Weight::Medium, theme::ink()));
                            col.spawn((text(fonts, said.clone(), 11.0, Weight::Regular, theme::ink_faint()), TextLayout::no_wrap()));
                        });
                    });
                }
                if let Some(tool) = doc.draw.tool.filter(|t| *t != Tool::Move) {
                    let said = format!("Change what the {} draws with", tool.name().to_lowercase());
                    plain(c, &said, Box::new(move |doc| {
                        let was = doc.draw.settings.get(tool.name()).cloned().unwrap_or_default();
                        doc.entry_line = None;
                        doc.entry = Some((Entry::Settings(tool), was));
                        doc.touch();
                    }));
                }
                plain(c, "Move what is selected", Box::new(|doc| tools::start(doc, Tool::Move)));
                plain(c, "Remove what is selected", Box::new(tools::delete));
            }
            plain(c, "New storey", Box::new(|doc| {
                doc.entry_line = None;
                doc.entry = Some((Entry::Storey, String::new()));
                doc.touch();
            }));
            plain(c, "Undo", Box::new(|doc| tools::step(doc, false)));
            plain(c, "Redo", Box::new(|doc| tools::step(doc, true)));
        }

        heading(c, fonts, "Show");
        let layer = |c: &mut ChildSpawnerCommands, name: &str, key: &str, colour: Color, on: bool, count: Option<usize>, act: Box<dyn Fn(&mut Doc) + Send + Sync>| {
            browser_row(c, false, move |doc| act(doc), |r| {
                r.spawn((rounded(Node { width: px(10.0), height: px(10.0), border: UiRect::all(px(1.5)), flex_shrink: 0.0, ..default() }, 5.0), BackgroundColor(if on { colour } else { Color::NONE }), BorderColor::all(colour)));
                r.spawn(text(fonts, name, 12.5, Weight::Medium, if on { theme::ink() } else { theme::ink_faint() }));
                if let Some(count) = count {
                    r.spawn(text(fonts, count.to_string(), 11.5, Weight::Regular, theme::ink_faint()));
                }
                r.spawn(Node { flex_grow: 1.0, ..default() });
                keycap(r, fonts, key);
            });
        };
        for (i, d) in [Discipline::Architecture, Discipline::Mechanical, Discipline::Electrical, Discipline::Plumbing].into_iter().enumerate() {
            let count = snap.counts.iter().find(|c| c.0 == d).map(|c| c.1).unwrap_or(0);
            if i == 1 && !snap.planes.is_empty() {
                layer(c, "Roof", "2", theme::roof_bare(), doc.show.roof, Some(snap.planes.len()), Box::new(|doc| {
                    doc.show.roof = !doc.show.roof;
                    doc.redraw();
                }));
            }
            // A job with nothing of a discipline does not offer to show it.
            if count == 0 {
                continue;
            }
            let key = ["1", "3", "4", "5"][i];
            layer(c, theme::discipline_name(d), key, theme::discipline_colour(d), doc.show.discipline(d), Some(count), Box::new(move |doc| {
                doc.show.toggle(d);
                doc.redraw();
            }));
        }
        layer(c, "See through", "X", theme::accent(), doc.show.ghost, None, Box::new(|doc| {
            doc.show.ghost = !doc.show.ghost;
            doc.redraw();
        }));

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
                browser_row(c, !t.path.is_empty() && doc.selected.as_deref() == Some(t.path.as_str()), act, |r| {
                    swatch(r, if t.path.is_empty() { theme::status_ink(t.flag) } else { theme::system_colour(&t.classification) });
                    r.spawn((text(fonts, t.name.clone(), 12.5, Weight::Medium, if t.path.is_empty() { theme::status_ink(t.flag) } else { theme::ink() }), TextLayout::no_wrap()));
                    r.spawn(Node { flex_grow: 1.0, ..default() });
                    r.spawn((text(fonts, total(&t.runs), 11.5, Weight::Regular, if t.runs.members == 0 { theme::ink_faint() } else { theme::status_ink(t.runs.status) }), TextLayout::no_wrap()));
                });
            }
        }
    });
}

/// A label's pill: white, hairline edge, and nothing in it takes the pointer, so a click
/// on a label still lands on the model under it.
fn pill(p: &mut ChildSpawnerCommands, label: Label, selected: bool, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn((
        rounded(Node { position_type: PositionType::Absolute, left: px(-2000.0), top: px(-2000.0), padding: UiRect::axes(px(8.0), px(3.0)), column_gap: px(6.0), align_items: AlignItems::Center, border: UiRect::all(px(1.0)), ..default() }, 999.0),
        UiTransform { translation: Val2::percent(-50.0, -50.0), ..default() },
        BackgroundColor(if selected { theme::accent() } else { theme::panel().with_alpha(0.94) }),
        BorderColor::all(if selected { theme::accent() } else { theme::hairline() }),
        BoxShadow(vec![ShadowStyle { color: Color::srgba(0.1, 0.14, 0.2, 0.18), x_offset: px(0.0), y_offset: px(1.0), spread_radius: px(0.0), blur_radius: px(4.0) }]),
        Pickable::IGNORE,
        label,
    ))
    .with_children(build);
}

fn quiet(p: &mut ChildSpawnerCommands, fonts: &Fonts, s: impl Into<String>, size: f32, weight: Weight, colour: Color) {
    p.spawn((text(fonts, s, size, weight, colour), TextLayout::no_wrap(), Pickable::IGNORE));
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
                rounded(Node { position_type: PositionType::Absolute, left: px(-2000.0), top: px(-2000.0), padding: UiRect::axes(px(5.0), px(1.0)), ..default() }, 4.0),
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
                // The flag stays on a selected facet too: a light dot on the blue.
                if plane.worst() != Status::Known {
                    let dot = if selected { theme::amber_wash() } else { theme::status_ink(plane.worst()) };
                    l.spawn((rounded(Node { width: px(6.0), height: px(6.0), ..default() }, 3.0), BackgroundColor(if selected && plane.worst() == Status::Unknown { theme::red_wash() } else { dot }), Pickable::IGNORE));
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
                        l.spawn((rounded(Node { width: px(6.0), height: px(6.0), ..default() }, 3.0), BackgroundColor(if status == Status::Unknown { theme::red_wash() } else { theme::amber_wash() }), Pickable::IGNORE));
                    }
                }
            });
        }

        // What is being typed, and anything the last action needs said in full.
        o.spawn((Node { position_type: PositionType::Absolute, top: px(BAR_H + GAP * 2.0), left: px(BROWSER_W + GAP * 2.0), right: px(INSPECTOR_W + GAP * 2.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(10.0), ..default() }, Pickable::IGNORE)).with_children(|mid| {
            if let Some((field, typed)) = &doc.entry {
                let name = snap.selected.as_ref().map(|s| s.name.clone()).unwrap_or_default();
                mid.spawn(theme::card(Node { width: px(360.0), flex_direction: FlexDirection::Column, padding: UiRect::all(px(14.0)), row_gap: px(9.0), ..default() })).with_children(|c| {
                    let title = match field {
                        Entry::Pitch => format!("Pitch of {name}"),
                        Entry::Move => format!("Move the eave of {name}"),
                        Entry::Allowance => format!("Allowance for {name}"),
                        Entry::UnitCost => format!("Unit cost of {:?} on {name}", doc.entry_line.clone().unwrap_or_default()),
                        Entry::Material => format!("What {name} is made of"),
                        Entry::Overhead => "Overhead on the job".to_string(),
                        Entry::Profit => "Profit on the job".to_string(),
                        Entry::Settings(tool) => format!("{}: {}", tool.name(), tool.asks().to_lowercase()),
                        Entry::Length => "Length of this piece".to_string(),
                        Entry::Storey => "New storey: name, floor height, height to the next".to_string(),
                    };
                    c.spawn(text(fonts, title, 13.0, Weight::Semibold, theme::ink()));
                    c.spawn((rounded(Node { padding: UiRect::axes(px(10.0), px(7.0)), border: UiRect::all(px(1.5)), min_height: px(36.0), align_items: AlignItems::Center, ..default() }, 7.0), BackgroundColor(theme::panel()), BorderColor::all(theme::accent()))).with_children(|f| {
                        f.spawn(text(fonts, typed.clone(), 15.0, Weight::Medium, theme::ink()));
                        f.spawn((Node { width: px(1.5), height: px(18.0), margin: UiRect::left(px(1.0)), ..default() }, BackgroundColor(theme::accent())));
                    });
                    c.spawn(Node { align_items: AlignItems::Center, column_gap: px(6.0), ..default() }).with_children(|h| {
                        let hint = match field {
                            Entry::Pitch => "Rise in 12, like 8:12",
                            Entry::Move => "Out is +, like 1' 6\" or -6 in",
                            Entry::Allowance => "Dollars per SF or LF, like 28 or $28.00",
                            Entry::UnitCost => "Dollars for what the line is priced in, like 31.50",
                            Entry::Material => "A material of the job, by name",
                            Entry::Overhead => "A percentage of the job cost, like 12",
                            Entry::Profit => "A percentage of cost and overhead, like 8",
                            Entry::Settings(tool) => tool.like(),
                            Entry::Length => "Toward the pointer, like 24 ft or 12' 6\"",
                            Entry::Storey => "like Ground, 0 ft, 9 ft",
                        };
                        h.spawn(text(fonts, hint, 11.5, Weight::Regular, theme::ink_faint()));
                        h.spawn(Node { flex_grow: 1.0, ..default() });
                        keycap(h, fonts, "Enter");
                        h.spawn(text(fonts, "set", 11.5, Weight::Regular, theme::ink_soft()));
                        keycap(h, fonts, "Esc");
                        h.spawn(text(fonts, "cancel", 11.5, Weight::Regular, theme::ink_soft()));
                    });
                });
            }
            if doc.entry.is_none() && doc.tone != Tone::Done {
                let (ink, wash) = if doc.tone == Tone::Refused { (theme::red(), theme::red_wash()) } else { (theme::amber(), theme::amber_wash()) };
                mid.spawn((rounded(Node { max_width: px(560.0), flex_direction: FlexDirection::Column, padding: UiRect::axes(px(14.0), px(10.0)), row_gap: px(3.0), border: UiRect::all(px(1.0)), ..default() }, 9.0), BackgroundColor(wash), BorderColor::all(ink.with_alpha(0.35)))).with_children(|c| {
                    for (i, line) in doc.message.lines().enumerate() {
                        c.spawn(text(fonts, if i == 0 { pretty(line) } else { line.to_string() }, 12.5, if i == 0 { Weight::Medium } else { Weight::Regular }, ink));
                    }
                });
            }
        });
    });
}

pub fn stat(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str, value: String, unit: &str, status: Status) {
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(3.0), flex_grow: 1.0, flex_basis: px(0.0), ..default() }).with_children(|s| {
        s.spawn(text(fonts, label.to_uppercase(), 10.0, Weight::Semibold, theme::ink_faint()));
        s.spawn(Node { align_items: AlignItems::Baseline, column_gap: px(4.0), ..default() }).with_children(|v| {
            v.spawn(text(fonts, value, 19.0, Weight::Semibold, theme::status_ink(status)));
            if !unit.is_empty() {
                v.spawn(text(fonts, unit, 11.5, Weight::Medium, theme::ink_soft()));
            }
        });
    });
}

/// A value in the inspector: its name, a flag when it is not Known, the value, and under
/// it where it came from, or for a worked-out figure, how. A long value takes its own line.
fn property(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str, value: String, status: Status, basis: &str, note: &str) {
    let long = value.chars().count() > 22;
    p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(7.0)), row_gap: px(3.0), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::hex(0xEEF1F5)))).with_children(|r| {
        r.spawn(Node { align_items: AlignItems::Center, column_gap: px(8.0), ..default() }).with_children(|top| {
            top.spawn((text(fonts, label, 12.5, Weight::Regular, theme::ink_soft()), TextLayout::no_wrap()));
            status_chip(top, fonts, status);
            top.spawn(Node { flex_grow: 1.0, ..default() });
            if !long {
                top.spawn((text(fonts, value.clone(), 12.5, Weight::Medium, theme::status_ink(status)), TextLayout::no_wrap()));
            }
        });
        if long {
            r.spawn(text(fonts, value, 12.5, Weight::Medium, theme::status_ink(status)));
        }
        if !basis.is_empty() {
            r.spawn(text(fonts, basis, 11.0, Weight::Regular, theme::ink_faint()));
        }
        if !note.is_empty() {
            r.spawn(text(fonts, note, 11.0, Weight::Regular, theme::status_ink(status)));
        }
    });
}

fn shortcut(p: &mut ChildSpawnerCommands, fonts: &Fonts, keys: &[&str], what: &str) {
    p.spawn(Node { align_items: AlignItems::Center, column_gap: px(5.0), padding: UiRect::vertical(px(3.0)), ..default() }).with_children(|r| {
        r.spawn(Node { width: px(112.0), column_gap: px(4.0), flex_shrink: 0.0, ..default() }).with_children(|k| {
            for key in keys {
                keycap(k, fonts, key);
            }
        });
        r.spawn(text(fonts, what, 12.0, Weight::Regular, theme::ink_soft()));
    });
}

fn inspector(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot, scrolled: f32) {
    p.spawn((
        floating(Node { top: px(BAR_H + GAP * 2.0), right: px(GAP), bottom: px(STATUS_H + GAP * 2.0), width: px(INSPECTOR_W), flex_direction: FlexDirection::Column, padding: UiRect::axes(px(18.0), px(16.0)), overflow: Overflow::scroll_y(), ..default() }),
        ScrollPosition(Vec2::new(0.0, scrolled)),
        InspectorScroll,
    ))
    .with_children(|c| {
        let editable = snap.rises_from.is_some();
        match &snap.selected {
            Some(el) if el.kind == Kind::System => system(c, fonts, snap, el),
            Some(el) => selection(c, fonts, snap, el),
            None => summary(c, fonts, snap),
        }
        heading(c, fonts, if editable { "Change it" } else { "Keys" });
        if editable {
            shortcut(c, fonts, &["=", "-"], "Pitch up or down 1 in 12");
            shortcut(c, fonts, &["]", "["], "Eave out or in 6 in");
            shortcut(c, fonts, &["P"], "Type a pitch");
            shortcut(c, fonts, &["M"], "Type a distance to move the eave");
            shortcut(c, fonts, &["G"], "Make its edge a gable");
        }
        if let Some(el) = &snap.selected {
            shortcut(c, fonts, &["C"], "Give it a condition");
            // Only what can carry an allowance is offered one, in the unit it is priced by.
            if let Some((_, unit)) = el.kind.priced_by() {
                shortcut(c, fonts, &["B"], &format!("Type an allowance, $ per {unit}"));
            }
            shortcut(c, fonts, &["U"], "Type the unit cost of its next unpriced line");
            // Only what is built is made of a material, and carries labor of its own.
            if !matches!(el.kind, Kind::Job | Kind::Condition | Kind::Material | Kind::Type | Kind::System | Kind::Storey) {
                shortcut(c, fonts, &["N"], "Name the material it is made of");
                shortcut(c, fonts, &["L"], "Switch its labor line off or on");
            }
            shortcut(c, fonts, &["Esc"], "Let go of it");
        }
        shortcut(c, fonts, &["O", "R"], "Type the job's overhead, or its profit");
        shortcut(c, fonts, &["W", "A", "S", "D"], "Move over the ground");
        shortcut(c, fonts, &["Q", "E"], "Down and up");
        shortcut(c, fonts, &["T"], if doc.plan { "Back to 3D" } else { "Plan: straight down" });
        shortcut(c, fonts, &["V"], if doc.walk { "Back to 3D" } else { "Go in on foot" });
        shortcut(c, fonts, &["F"], "Frame the building");
        shortcut(c, fonts, &["Tab"], "Next roof plane");
        shortcut(c, fonts, &["F2"], "Save the job");
        c.spawn(Node { margin: UiRect::top(px(8.0)), ..default() }).with_child(text(fonts, "Right-drag to turn, middle-drag to pan, wheel to zoom. Click anything to select it.", 11.5, Weight::Regular, theme::ink_faint()));
    });
}

fn title_block(p: &mut ChildSpawnerCommands, fonts: &Fonts, kind: &str, name: &str, under: String) {
    p.spawn(text(fonts, kind.to_uppercase(), 10.0, Weight::Semibold, theme::ink_faint()));
    p.spawn(Node { margin: UiRect::top(px(3.0)), ..default() }).with_child(text(fonts, name, 19.0, Weight::Semibold, theme::ink()));
    if !under.is_empty() {
        p.spawn(Node { margin: UiRect::top(px(2.0)), ..default() }).with_child(text(fonts, under, 11.5, Weight::Regular, theme::ink_faint()));
    }
}

/// A line of a list: something on the left, a figure on the right.
fn listed(p: &mut ChildSpawnerCommands, fonts: &Fonts, colour: Option<Color>, name: &str, count: String, figure: String, status: Status) {
    p.spawn((Node { align_items: AlignItems::Center, column_gap: px(9.0), padding: UiRect::vertical(px(6.0)), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::hex(0xEEF1F5)))).with_children(|r| {
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
fn summary(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot) {
    title_block(p, fonts, "Job", &snap.title, snap.file.clone());
    if snap.storeys.is_empty() && snap.totals.is_none() {
        p.spawn(Node { margin: UiRect::top(px(12.0)), ..default() }).with_child(text(fonts, "This job has no roof and no building in it yet.", 12.5, Weight::Regular, theme::red()));
    }
    for storey in &snap.storeys {
        heading(p, fonts, &storey.name);
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
        heading(p, fonts, "To settle");
        for (status, what, why) in &snap.notes {
            p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(7.0)), row_gap: px(3.0), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::hex(0xEEF1F5)))).with_children(|r| {
                r.spawn(Node { ..default() }).with_children(|c| status_chip(c, fonts, *status));
                r.spawn(text(fonts, what.clone(), 12.5, Weight::Medium, theme::status_ink(*status)));
                r.spawn(text(fonts, why.clone(), 11.0, Weight::Regular, theme::ink_faint()));
            });
        }
    }
}

/// A selected system: what it is and what it comes to.
fn system(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot, el: &view::ElementRows) {
    let Some(t) = snap.system(&el.path) else { return };
    title_block(p, fonts, &format!("{} system", theme::discipline_name(t.discipline)), &t.name, format!("{}  ·  {}", el.path, t.classification));
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
            p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(6.0)), row_gap: px(2.0), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::hex(0xEEF1F5)))).with_children(|r| {
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
        heading(p, fonts, "Joined");
        let plural = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        listed(p, fonts, None, &plural(j.networks, "network", "networks"), "runs joined end to end, through fittings".into(), String::new(), plumbline_world::Status::Known);
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

fn selection(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot, el: &view::ElementRows) {
    let under = match &snap.rises_from {
        Some((_, edge)) => format!("{}  ·  rises from edge {edge} of {}", el.path, snap.roof_name),
        None => el.path.clone(),
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
        Kind::Room => &[("floor_area", "Floor inside the walls"), ("perimeter", "Round")],
        Kind::Duct => &[("true_length", "Length"), ("surface_area", "Duct surface")],
        k if k.is_segment() => &[("true_length", "Length")],
        _ => &[],
    };
    if !lead.is_empty() {
        p.spawn(Node { margin: UiRect::top(px(16.0)), column_gap: px(12.0), ..default() }).with_children(|r| {
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
            p.spawn((Node { padding: UiRect::vertical(px(4.0)), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::hex(0xEEF1F5)))).with_children(|r| {
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
        heading(p, fonts, "Entered");
        for row in own.iter().filter(|r| !r.derived) {
            property(p, fonts, &pretty(&row.label), shown(row), row.status, &row.basis, "");
        }
    }
    if own.iter().any(|r| r.derived) {
        heading(p, fonts, "Worked out");
        for row in own.iter().filter(|r| r.derived) {
            property(p, fonts, &pretty(&row.label), shown(row), row.status, &row.basis, &row.note);
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
            p.spawn((rounded(Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(px(11.0), px(9.0)), margin: UiRect::bottom(px(8.0)), row_gap: px(5.0), border: UiRect::all(px(1.0)), ..default() }, 8.0), BackgroundColor(theme::panel_alt()), BorderColor::all(theme::hex(0xE6EAF0)))).with_children(|c| {
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
        None => "Point at the model to read a position".to_string(),
    };
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
    mut inspector: Query<(&mut ScrollPosition, &ComputedNode), (With<InspectorScroll>, Without<SheetScroll>)>,
    mut sheet: Query<(&mut ScrollPosition, &ComputedNode), (With<SheetScroll>, Without<InspectorScroll>)>,
) {
    if scroll.delta.y == 0.0 {
        return;
    }
    let by = if scroll.unit == MouseScrollUnit::Pixel { scroll.delta.y } else { scroll.delta.y * 36.0 };
    for (window, primary) in &windows {
        let Some(at) = window.cursor_position() else { continue };
        let limit = |node: &ComputedNode| ((node.content_size.y - node.size.y) * node.inverse_scale_factor).max(0.0);
        if primary {
            if at.x > window.width() - INSPECTOR_W - GAP && at.y > BAR_H + GAP {
                if let Ok((mut position, node)) = inspector.single_mut() {
                    scrolls.inspector = (scrolls.inspector - by).clamp(0.0, limit(node));
                    position.0.y = scrolls.inspector;
                }
            }
        } else if let Ok((mut position, node)) = sheet.single_mut() {
            scrolls.sheet = (scrolls.sheet - by).clamp(0.0, limit(node));
            position.0.y = scrolls.sheet;
        }
    }
}
