//! The main window around the 3D view: the title bar, the inspector, the status bar, and
//! what is written over the roof itself — each facet's number, area and pitch, the length
//! of each line, and the legend that says which colour is which kind of line.
//!
//! Everything here is drawn from the snapshot. A panel shows what the world holds and how
//! sure it is; it works nothing out.

use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::model::{to_view, Doc, Entry, Snapshot};
use crate::scene::{MainCamera, ViewRect, BAR_H, INSPECTOR_W, STATUS_H};
use crate::sheet::SheetScroll;
use crate::theme::{self, chip, heading, keycap, rounded, status_chip, swatch, text, Fonts, Weight};
use plumbline_world::value::Status;
use plumbline_world::{units, view};

#[derive(Component)]
pub struct MainUi;
#[derive(Component)]
pub struct PointerText;
#[derive(Component)]
pub struct InspectorScroll;

/// Something written over the roof, kept at a point of the model as the view moves.
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

pub fn ui_camera(commands: &mut Commands) -> Entity {
    // Its own layer, so the roof's lines are not drawn a second time over the panels.
    commands.spawn((Camera2d, Camera { order: 1, clear_color: ClearColorConfig::None, ..default() }, RenderLayers::layer(1), IsDefaultUiCamera)).id()
}

fn divider(p: &mut ChildSpawnerCommands) {
    p.spawn((Node { width: px(1.0), height: px(22.0), margin: UiRect::horizontal(px(4.0)), ..default() }, BackgroundColor(theme::hairline())));
}

/// The wordmark: a plumb line, and the name.
pub fn wordmark(p: &mut ChildSpawnerCommands, fonts: &Fonts) {
    p.spawn(Node { align_items: AlignItems::Center, column_gap: px(9.0), ..default() }).with_children(|m| {
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

fn pretty(label: &str) -> String {
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
        Some(x) => amount(Some(x), 2, &row.unit),
        None => theme::trade(&row.text),
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
        inspector(root, fonts, &doc, &snap, scrolls.inspector);
        status_bar(root, fonts, &doc);
    });
}

fn title_bar(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    p.spawn((
        Node { position_type: PositionType::Absolute, top: px(0.0), left: px(0.0), right: px(0.0), height: px(BAR_H), padding: UiRect::horizontal(px(18.0)), align_items: AlignItems::Center, column_gap: px(12.0), border: UiRect::bottom(px(1.0)), ..default() },
        BackgroundColor(theme::panel()),
        BorderColor::all(theme::hairline()),
    ))
    .with_children(|bar| {
        wordmark(bar, fonts);
        divider(bar);
        bar.spawn(text(fonts, snap.title.clone(), 13.5, Weight::Medium, theme::ink_soft()));
        bar.spawn(text(fonts, "›", 14.0, Weight::Regular, theme::ink_faint()));
        bar.spawn(text(fonts, snap.roof_name.clone(), 13.5, Weight::Semibold, theme::ink()));
        bar.spawn(Node { flex_grow: 1.0, ..default() });
        tally_chips(bar, fonts, snap.tally);
        divider(bar);
        bar.spawn(text(fonts, snap.file.clone(), 12.5, Weight::Regular, theme::ink_soft()));
        if doc.edited {
            chip(bar, fonts, "Not saved", theme::amber(), theme::amber_wash());
        } else {
            chip(bar, fonts, "Saved", theme::green(), theme::hex(0xE3F4EA));
        }
    });
}

fn status_bar(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc) {
    let first = doc.message.lines().next().unwrap_or("").to_string();
    let refused = first.starts_with("refused");
    p.spawn((
        Node { position_type: PositionType::Absolute, bottom: px(0.0), left: px(0.0), right: px(0.0), height: px(STATUS_H), padding: UiRect::horizontal(px(16.0)), align_items: AlignItems::Center, column_gap: px(9.0), border: UiRect::top(px(1.0)), overflow: Overflow::clip(), ..default() },
        BackgroundColor(theme::panel()),
        BorderColor::all(theme::hairline()),
    ))
    .with_children(|bar| {
        bar.spawn((rounded(Node { width: px(7.0), height: px(7.0), flex_shrink: 0.0, ..default() }, 4.0), BackgroundColor(if refused { theme::red() } else { theme::green() })));
        bar.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), overflow: Overflow::clip(), ..default() }).with_child((text(fonts, first, 12.0, Weight::Regular, if refused { theme::red() } else { theme::ink_soft() }), TextLayout::no_wrap()));
        bar.spawn(Node { flex_shrink: 0.0, ..default() }).with_child((text(fonts, "", 12.0, Weight::Regular, theme::ink_soft()), TextLayout::no_wrap(), PointerText));
        bar.spawn((Node { width: px(1.0), height: px(16.0), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::hairline())));
        bar.spawn(Node { flex_shrink: 0.0, ..default() }).with_child(text(fonts, if doc.plan { "Plan view" } else { "3D view" }, 12.0, Weight::Medium, theme::ink_soft()));
    });
}

/// A label's pill: white, hairline edge, and nothing in it takes the pointer, so a click
/// on a label still lands on the roof under it.
fn pill(p: &mut ChildSpawnerCommands, label: Label, selected: bool, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn((
        rounded(Node { position_type: PositionType::Absolute, left: px(-2000.0), top: px(-2000.0), padding: UiRect::axes(px(8.0), px(3.0)), column_gap: px(6.0), align_items: AlignItems::Center, border: UiRect::all(px(1.0)), ..default() }, 999.0),
        UiTransform { translation: Val2::percent(-50.0, -50.0), ..default() },
        BackgroundColor(if selected { theme::accent() } else { theme::panel().with_alpha(0.94) }),
        BorderColor::all(if selected { theme::accent() } else { theme::hairline() }),
        BoxShadow(vec![ShadowStyle { color: Color::srgba(0.1, 0.14, 0.2, 0.14), x_offset: px(0.0), y_offset: px(1.0), spread_radius: px(0.0), blur_radius: px(4.0) }]),
        Pickable::IGNORE,
        label,
    ))
    .with_children(build);
}

fn quiet(p: &mut ChildSpawnerCommands, fonts: &Fonts, s: impl Into<String>, size: f32, weight: Weight, colour: Color) {
    p.spawn((text(fonts, s, size, weight, colour), TextLayout::no_wrap(), Pickable::IGNORE));
}

fn overlay(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    p.spawn((Node { position_type: PositionType::Absolute, top: px(BAR_H), left: px(0.0), right: px(INSPECTOR_W), bottom: px(STATUS_H), overflow: Overflow::clip(), ..default() }, Pickable::IGNORE)).with_children(|o| {
        // Each line's length, for the selected plane's own lines, or for all of them on plan.
        for line in &snap.lines {
            let (Some([a, b]), Some(length)) = (line.ends, line.length) else { continue };
            let chosen = doc.selected.as_ref().is_some_and(|s| *s == line.path || line.planes.contains(s));
            if !(doc.plan || chosen) {
                continue;
            }
            let at = (to_view(doc.origin, a) + to_view(doc.origin, b)) / 2.0 + Vec3::Y * 0.1;
            let colour = theme::line_colour(line.kind);
            o.spawn((
                rounded(Node { position_type: PositionType::Absolute, left: px(-2000.0), top: px(-2000.0), padding: UiRect::axes(px(5.0), px(1.0)), ..default() }, 4.0),
                UiTransform { translation: Val2::percent(-50.0, -50.0), ..default() },
                BackgroundColor(theme::panel().with_alpha(0.9)),
                Pickable::IGNORE,
                Label { at, facing: None, rank: 2 },
            ))
            .with_children(|l| quiet(l, fonts, units::feet_as_ft_in(length, 1), 11.0, Weight::Medium, if line.status == Status::Known { colour } else { theme::status_ink(line.status) }));
        }
        // Each facet: its number, its area, its pitch.
        for plane in &snap.planes {
            let Some(solid) = &plane.solid else { continue };
            let c = centroid(&solid.points);
            let at = to_view(doc.origin, [c[0], c[1], solid.height_at(c)]);
            // Up the slope and out of it: the eave's direction crossed with the fall line.
            let positions: Vec<Vec3> = solid.points.iter().map(|q| to_view(doc.origin, [q[0], q[1], solid.height_at(*q)])).collect();
            let mut normal = Vec3::ZERO;
            for i in 0..positions.len() {
                let (u, v) = (positions[i], positions[(i + 1) % positions.len()]);
                normal += Vec3::new((u.y - v.y) * (u.z + v.z), (u.z - v.z) * (u.x + v.x), (u.x - v.x) * (u.y + v.y));
            }
            let mut normal = normal.normalize_or_zero();
            if normal.y < 0.0 {
                normal = -normal;
            }
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
                if plane.worst() != Status::Known && !selected {
                    l.spawn((rounded(Node { width: px(6.0), height: px(6.0), ..default() }, 3.0), BackgroundColor(theme::status_ink(plane.worst())), Pickable::IGNORE));
                }
            });
        }

        // 3D or plan.
        o.spawn(theme::card(Node { position_type: PositionType::Absolute, top: px(14.0), left: px(14.0), padding: UiRect::all(px(3.0)), column_gap: px(2.0), ..default() })).with_children(|t| {
            for (label, plan) in [("3D", false), ("Plan", true)] {
                let on = doc.plan == plan;
                t.spawn((rounded(Node { padding: UiRect::axes(px(12.0), px(5.0)), ..default() }, 7.0), BackgroundColor(if on { theme::accent_wash() } else { Color::NONE })))
                    .with_child(text(fonts, label, 12.0, Weight::Medium, if on { theme::accent() } else { theme::ink_soft() }))
                    .observe(move |click: On<PointerClick>, mut doc: ResMut<Doc>| {
                        if click.button == PointerButton::Primary && doc.plan != plan {
                            doc.plan = plan;
                            doc.touch();
                        }
                    });
            }
        });

        legend(o, fonts, snap);

        // What is being typed, and anything the last action needs said in full.
        o.spawn((Node { position_type: PositionType::Absolute, top: px(14.0), left: px(128.0), right: px(128.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(10.0), ..default() }, Pickable::IGNORE)).with_children(|mid| {
            if let Some((field, typed)) = &doc.entry {
                let name = snap.selected.as_ref().map(|s| s.name.clone()).unwrap_or_default();
                mid.spawn(theme::card(Node { width: px(360.0), flex_direction: FlexDirection::Column, padding: UiRect::all(px(14.0)), row_gap: px(9.0), ..default() })).with_children(|c| {
                    c.spawn(text(fonts, if *field == Entry::Pitch { format!("Pitch of {name}") } else { format!("Move the eave of {name}") }, 13.0, Weight::Semibold, theme::ink()));
                    c.spawn((rounded(Node { padding: UiRect::axes(px(10.0), px(7.0)), border: UiRect::all(px(1.5)), min_height: px(36.0), align_items: AlignItems::Center, ..default() }, 7.0), BackgroundColor(theme::panel()), BorderColor::all(theme::accent()))).with_children(|f| {
                        f.spawn(text(fonts, typed.clone(), 15.0, Weight::Medium, theme::ink()));
                        f.spawn((Node { width: px(1.5), height: px(18.0), margin: UiRect::left(px(1.0)), ..default() }, BackgroundColor(theme::accent())));
                    });
                    c.spawn(Node { align_items: AlignItems::Center, column_gap: px(6.0), ..default() }).with_children(|h| {
                        h.spawn(text(fonts, if *field == Entry::Pitch { "Rise in 12, like 8:12" } else { "Out is +, like 1' 6\" or -6 in" }, 11.5, Weight::Regular, theme::ink_faint()));
                        h.spawn(Node { flex_grow: 1.0, ..default() });
                        keycap(h, fonts, "Enter");
                        h.spawn(text(fonts, "set", 11.5, Weight::Regular, theme::ink_soft()));
                        keycap(h, fonts, "Esc");
                        h.spawn(text(fonts, "cancel", 11.5, Weight::Regular, theme::ink_soft()));
                    });
                });
            }
            let refused = doc.message.starts_with("refused");
            if doc.entry.is_none() && (refused || doc.message.lines().count() > 1) {
                let (ink, wash) = if refused { (theme::red(), theme::red_wash()) } else { (theme::amber(), theme::amber_wash()) };
                mid.spawn((rounded(Node { max_width: px(560.0), flex_direction: FlexDirection::Column, padding: UiRect::axes(px(14.0), px(10.0)), row_gap: px(3.0), border: UiRect::all(px(1.0)), ..default() }, 9.0), BackgroundColor(wash), BorderColor::all(ink.with_alpha(0.35)))).with_children(|c| {
                    for (i, line) in doc.message.lines().enumerate() {
                        let line = if i == 0 { line.strip_prefix("refused: ").map(|l| format!("Refused: {l}")).unwrap_or(line.to_string()) } else { line.to_string() };
                        c.spawn(text(fonts, line, 12.5, if i == 0 { Weight::Medium } else { Weight::Regular }, ink));
                    }
                });
            }
        });
    });
}

/// Which colour is which kind of line, and what each kind comes to.
fn legend(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot) {
    let Some(totals) = &snap.totals else { return };
    if totals.lines.is_empty() {
        return;
    }
    p.spawn(theme::card(Node { position_type: PositionType::Absolute, bottom: px(14.0), left: px(14.0), flex_direction: FlexDirection::Column, padding: UiRect::axes(px(14.0), px(11.0)), row_gap: px(6.0), ..default() })).with_children(|c| {
        c.spawn(text(fonts, "LINES", 10.5, Weight::Semibold, theme::ink_faint()));
        for (kind, sum) in &totals.lines {
            c.spawn(Node { align_items: AlignItems::Center, column_gap: px(9.0), ..default() }).with_children(|r| {
                swatch(r, theme::line_colour(*kind));
                r.spawn(Node { width: px(86.0), ..default() }).with_child(text(fonts, theme::kind_name(*kind), 12.0, Weight::Medium, theme::ink()));
                r.spawn(Node { width: px(18.0), justify_content: JustifyContent::FlexEnd, ..default() }).with_child(text(fonts, sum.members.to_string(), 12.0, Weight::Regular, theme::ink_faint()));
                r.spawn(Node { width: px(84.0), justify_content: JustifyContent::FlexEnd, ..default() }).with_child(text(fonts, if sum.value.is_some() { amount(sum.value, 2, "LF") } else { "no total".into() }, 12.0, Weight::Medium, theme::status_ink(sum.status)));
            });
        }
        if snap.lines.iter().any(|l| l.length.is_none()) {
            c.spawn(Node { align_items: AlignItems::Center, column_gap: px(9.0), ..default() }).with_children(|r| {
                swatch(r, theme::red());
                r.spawn(text(fonts, "No length worked out", 12.0, Weight::Medium, theme::red()));
            });
        }
    });
}

fn stat(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str, value: String, unit: &str, status: Status) {
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
        r.spawn(Node { width: px(74.0), column_gap: px(4.0), ..default() }).with_children(|k| {
            for key in keys {
                keycap(k, fonts, key);
            }
        });
        r.spawn(text(fonts, what, 12.0, Weight::Regular, theme::ink_soft()));
    });
}

fn inspector(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot, scrolled: f32) {
    p.spawn((
        Node { position_type: PositionType::Absolute, top: px(BAR_H), right: px(0.0), bottom: px(STATUS_H), width: px(INSPECTOR_W), flex_direction: FlexDirection::Column, padding: UiRect::axes(px(18.0), px(16.0)), border: UiRect::left(px(1.0)), overflow: Overflow::scroll_y(), ..default() },
        BackgroundColor(theme::panel()),
        BorderColor::all(theme::hairline()),
        ScrollPosition(Vec2::new(0.0, scrolled)),
        InspectorScroll,
    ))
    .with_children(|c| {
        match &snap.selected {
            Some(el) => selection(c, fonts, snap, el),
            None => roof_summary(c, fonts, snap),
        }
        heading(c, fonts, if snap.selected.is_some() { "Change it" } else { "In this view" });
        if snap.selected.is_some() {
            shortcut(c, fonts, &["=", "-"], "Pitch up or down 1 in 12");
            shortcut(c, fonts, &["]", "["], "Eave out or in 6 in");
            shortcut(c, fonts, &["P"], "Type a pitch");
            shortcut(c, fonts, &["M"], "Type a distance to move the eave");
            shortcut(c, fonts, &["G"], "Make its edge a gable");
            shortcut(c, fonts, &["C"], "Give it a condition");
            shortcut(c, fonts, &["Esc"], "Let go of it");
        } else {
            shortcut(c, fonts, &["Tab"], "Select the next plane, or click one");
        }
        shortcut(c, fonts, &["T"], if doc.plan { "Back to the 3D view" } else { "Look straight down: the plan" });
        shortcut(c, fonts, &["F"], "Frame the roof");
        shortcut(c, fonts, &["F2"], "Save the job");
        c.spawn(Node { margin: UiRect::top(px(8.0)), ..default() }).with_child(text(fonts, "Right-drag to orbit, middle-drag to pan, wheel to zoom.", 11.5, Weight::Regular, theme::ink_faint()));
    });
}

fn title_block(p: &mut ChildSpawnerCommands, fonts: &Fonts, kind: &str, name: &str, under: String) {
    p.spawn(text(fonts, kind.to_uppercase(), 10.0, Weight::Semibold, theme::ink_faint()));
    p.spawn(Node { margin: UiRect::top(px(3.0)), ..default() }).with_child(text(fonts, name, 19.0, Weight::Semibold, theme::ink()));
    p.spawn(Node { margin: UiRect::top(px(2.0)), ..default() }).with_child(text(fonts, under, 11.5, Weight::Regular, theme::ink_faint()));
}

fn roof_summary(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot) {
    title_block(p, fonts, "Roof", &snap.roof_name, snap.roof_path.clone());
    let Some(totals) = &snap.totals else { return };
    let squares = view::in_squares(&totals.surface);
    p.spawn(Node { margin: UiRect::top(px(16.0)), column_gap: px(12.0), ..default() }).with_children(|r| {
        stat(r, fonts, "Surface", totals.surface.value.map(|x| view::figure_shown(x, 2)).unwrap_or("no total".into()), if totals.surface.value.is_some() { "SF" } else { "" }, totals.surface.status);
        stat(r, fonts, "Squares", squares.value.map(|x| view::figure_shown(x, 2)).unwrap_or("no total".into()), if squares.value.is_some() { "SQ" } else { "" }, squares.status);
    });
    p.spawn(Node { margin: UiRect::top(px(14.0)), column_gap: px(12.0), ..default() }).with_children(|r| {
        match totals.predominant_pitch {
            Some((pitch, status)) => stat(r, fonts, "Predominant pitch", theme::pitch_text(pitch), "", status),
            None => stat(r, fonts, "Predominant pitch", "unknown".into(), "", Status::Unknown),
        }
        stat(r, fonts, "Facets", totals.facets.to_string(), "", Status::Known);
    });
    if !totals.surface.missing.is_empty() {
        p.spawn(Node { margin: UiRect::top(px(8.0)), ..default() }).with_child(text(fonts, format!("No surface total: {} of {} planes have no area.", totals.surface.missing.len(), totals.surface.members), 11.5, Weight::Regular, theme::red()));
    }
    heading(p, fonts, "Lines by kind");
    for (kind, sum) in &totals.lines {
        p.spawn((Node { align_items: AlignItems::Center, column_gap: px(9.0), padding: UiRect::vertical(px(6.0)), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::hex(0xEEF1F5)))).with_children(|r| {
            swatch(r, theme::line_colour(*kind));
            r.spawn(text(fonts, theme::kind_name(*kind), 12.5, Weight::Medium, theme::ink()));
            r.spawn(text(fonts, format!("{}", sum.members), 12.0, Weight::Regular, theme::ink_faint()));
            r.spawn(Node { flex_grow: 1.0, ..default() });
            status_chip(r, fonts, sum.status);
            r.spawn(text(fonts, if sum.value.is_some() { amount(sum.value, 2, "LF") } else { format!("no total, {} of {} missing", sum.missing.len(), sum.members) }, 12.5, Weight::Medium, theme::status_ink(sum.status)));
        });
    }
    if !snap.notes.is_empty() {
        heading(p, fonts, "To settle");
        for (status, what, why) in &snap.notes {
            property(p, fonts, what, String::new(), *status, why, "");
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
    p.spawn(Node { margin: UiRect::top(px(16.0)), column_gap: px(12.0), ..default() }).with_children(|r| {
        if el.kind.is_area() {
            match find("surface_area") {
                Some(row) => stat(r, fonts, "Surface", row.number.map(|x| view::figure_shown(x, 2)).unwrap_or("unknown".into()), if row.number.is_some() { "SF" } else { "" }, row.status),
                None => stat(r, fonts, "Surface", "unknown".into(), "", Status::Unknown),
            }
            match find("pitch") {
                Some(row) => stat(r, fonts, "Pitch", theme::trade(&row.text), "", row.status),
                None => stat(r, fonts, "Pitch", "none".into(), "", Status::Known),
            }
        } else if let Some(row) = find("true_length") {
            stat(r, fonts, "Length", row.number.map(|x| view::figure_shown(x, 2)).unwrap_or("unknown".into()), if row.number.is_some() { "LF" } else { "" }, row.status);
            stat(r, fonts, "On the tape", row.number.map(|x| units::feet_as_ft_in(x, 16)).unwrap_or("unknown".into()), "", row.status);
        }
    });
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
    heading(p, fonts, "Entered");
    for row in own.iter().filter(|r| !r.derived) {
        property(p, fonts, &pretty(&row.label), shown(row), row.status, &row.basis, "");
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
            p.spawn((rounded(Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(px(11.0), px(9.0)), margin: UiRect::bottom(px(8.0)), row_gap: px(4.0), border: UiRect::all(px(1.0)), ..default() }, 8.0), BackgroundColor(theme::panel_alt()), BorderColor::all(theme::hex(0xE6EAF0)))).with_children(|c| {
                c.spawn(Node { margin: UiRect::bottom(px(2.0)), ..default() }).with_child(text(fonts, line, 12.5, Weight::Semibold, theme::ink()));
                for row in costed.iter().filter(|r| r.label.split_once(": ").is_some_and(|x| x.1 == line)) {
                    let what = row.label.split_once(": ").map(|x| x.0).unwrap_or("");
                    c.spawn(Node { align_items: AlignItems::Center, column_gap: px(8.0), ..default() }).with_children(|r| {
                        r.spawn(text(fonts, pretty(what), 12.0, if row.derived { Weight::Medium } else { Weight::Regular }, if row.derived { theme::ink() } else { theme::ink_soft() }));
                        r.spawn(Node { flex_grow: 1.0, ..default() });
                        r.spawn(text(fonts, shown(row), 12.0, Weight::Medium, theme::status_ink(row.status)));
                    });
                }
            });
        }
    }
}

/// Keep each label at its point of the model as the view moves. Where two would lie over
/// each other, the one nearer the eye stays and the other waits until there is room.
pub fn place_labels(doc: Res<Doc>, camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>, rect: Res<ViewRect>, mut labels: Query<(Entity, &Label, &mut Node, &mut Visibility, &ComputedNode)>) {
    let Ok((camera, eye)) = camera.single() else { return };
    // Where each label would go, how big it is, and how far off.
    let mut wanted: Vec<(Entity, u8, f32, Vec2, Vec2)> = Vec::new();
    for (entity, label, _, _, computed) in &labels {
        let towards = if doc.plan { Vec3::Y } else { eye.translation() - label.at };
        if label.facing.is_some_and(|n| n.dot(towards) <= 0.0) {
            continue;
        }
        let Ok(at) = camera.world_to_viewport(eye, label.at) else { continue };
        let half = computed.size * computed.inverse_scale_factor / 2.0 + Vec2::splat(2.0);
        wanted.push((entity, label.rank, eye.translation().distance(label.at), at - rect.0.min, half));
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

/// Where the pointer is on the roof, to the sixteenth, in the status bar.
pub fn write_pointer(doc: Res<Doc>, mut out: Query<&mut Text, With<PointerText>>) {
    let Ok(mut text) = out.single_mut() else { return };
    let s = match doc.pointer {
        Some(p) => format!("E {}     N {}     H {}", units::feet_as_ft_in(p[0], 16), units::feet_as_ft_in(p[1], 16), units::feet_as_ft_in(p[2], 16)),
        None => "Point at the roof to read a position".to_string(),
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
            if at.x > window.width() - INSPECTOR_W && at.y > BAR_H {
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
