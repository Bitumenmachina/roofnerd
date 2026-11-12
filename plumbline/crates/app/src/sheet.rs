//! The sheet: a second window on the same job, laid out as an estimator's sheet. A tab for
//! the roof, one for the building's walls and floors, and one for each discipline's
//! systems. Each is tables of what the world holds, every figure with its flag, and under
//! them the cost lines of whatever is selected, worked out step by step.
//!
//! Click a row and it is selected in the 3D view; pick something in the 3D view and its
//! row lights here. Nothing on the sheet is worked out by the sheet.

use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;

use crate::model::{Doc, Entry, Snapshot, Tab, Tone};
use crate::panels::{amount, one_line, pretty, shown, tally_chips, tone_ink, total, wordmark, Cameras, Scrolls};
use crate::theme::{self, rounded, status_chip, swatch, text, Fonts, Weight};
use plumbline_world::value::Status;
use plumbline_world::{units, view, Discipline, Kind};

/// How much of the sheet window its heading and its tabs take at the top, and its line
/// of status at the foot: what scrolls shows between them.
pub const HEAD_H: f32 = 100.0;
pub const FOOT_H: f32 = 34.0;

#[derive(Component)]
pub struct SheetWindow;
#[derive(Component)]
pub struct SheetUi;
#[derive(Component)]
pub struct SheetScroll;

fn stat_card(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str, value: String, unit: &str, status: Status, under: String) {
    p.spawn(theme::card(Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, flex_basis: px(0.0), padding: UiRect::axes(px(16.0), px(13.0)), row_gap: px(4.0), ..default() })).with_children(|c| {
        c.spawn(text(fonts, label.to_uppercase(), 10.0, Weight::Semibold, theme::ink_faint()));
        c.spawn(Node { align_items: AlignItems::Baseline, column_gap: px(5.0), ..default() }).with_children(|v| {
            v.spawn(text(fonts, value, 23.0, Weight::Semibold, theme::status_ink(status)));
            if !unit.is_empty() {
                v.spawn(text(fonts, unit, 12.0, Weight::Medium, theme::ink_soft()));
            }
        });
        c.spawn(text(fonts, under, 11.5, Weight::Regular, if status == Status::Unknown { theme::red() } else { theme::ink_faint() }));
    });
}

/// The first cell of a row that names a thing at length: what it is, on as many lines as
/// it needs, and a small word about it underneath. It takes the room that is left.
fn named(p: &mut ChildSpawnerCommands, fonts: &Fonts, what: String, ink: Color, about: String, about_ink: Color) {
    p.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(5.0)), row_gap: px(1.0), overflow: Overflow::clip(), ..default() }).with_children(|c| {
        c.spawn(text(fonts, what, 12.5, Weight::Regular, ink));
        if !about.is_empty() {
            c.spawn((text(fonts, about, 11.0, Weight::Regular, about_ink), TextLayout::no_wrap()));
        }
    });
}

/// One cell of a table row. A width of 0 takes the room that is left.
fn cell(p: &mut ChildSpawnerCommands, width: f32, right: bool, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    let mut node = Node { align_items: AlignItems::Center, column_gap: px(7.0), overflow: Overflow::clip(), justify_content: if right { JustifyContent::FlexEnd } else { JustifyContent::FlexStart }, ..default() };
    if width > 0.0 {
        node.width = px(width);
        node.flex_shrink = 0.0;
    } else {
        node.flex_grow = 1.0;
        node.flex_basis = px(0.0);
        node.min_width = px(0.0);
    }
    p.spawn(node).with_children(build);
}

/// A figure on the sheet that is changed where it stands: it is drawn as a field, a
/// click on it opens it, and while it is being typed it shows what is typed.
fn editable(c: &mut ChildSpawnerCommands, fonts: &Fonts, name: String, said: String, ink: Color, typed: Option<&str>, open: impl Fn(&mut Doc) + Send + Sync + 'static) {
    let mut e = c.spawn((
        // A field on the sheet looks like a field in the model window: round-cornered,
        // a little whiter than what it sits on, edged in the accent while it is typed in.
        Node { padding: UiRect::axes(px(7.0), px(2.0)), border: UiRect::all(px(1.0)), border_radius: BorderRadius::all(px(theme::SMALL_RADIUS)), align_items: AlignItems::Center, min_height: px(22.0), flex_shrink: 0.0, ..default() },
        BorderColor::all(if typed.is_some() { theme::accent() } else { theme::control_edge() }),
        BackgroundColor(if typed.is_some() { theme::panel() } else { Color::WHITE }),
        theme::Named { name, scrolls: true },
    ));
    e.with_children(|f| match typed {
        Some(typed) => {
            f.spawn((theme::typed(fonts, typed, 12.5, Weight::Medium, theme::ink()), TextLayout::no_wrap(), Pickable::IGNORE));
            f.spawn((Node { width: px(1.5), height: px(14.0), margin: UiRect::left(px(1.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::accent()), Pickable::IGNORE));
        }
        None => {
            f.spawn((text(fonts, said, 12.5, Weight::Regular, ink), TextLayout::no_wrap(), Pickable::IGNORE));
        }
    });
    e.observe(move |mut press: On<PointerClick>, mut doc: ResMut<Doc>| {
        if press.button == PointerButton::Primary {
            // The click is this figure's: the row under it does not also take it.
            press.propagate(false);
            open(&mut doc);
        }
    });
}

fn words(p: &mut ChildSpawnerCommands, fonts: &Fonts, s: impl Into<String>, size: f32, weight: Weight, colour: Color) {
    p.spawn((text(fonts, s, size, weight, colour), TextLayout::no_wrap()));
}

fn dot(p: &mut ChildSpawnerCommands, status: Status) {
    let colour = match status {
        Status::Known => Color::NONE,
        other => theme::status_ink(other),
    };
    p.spawn((rounded(Node { width: px(7.0), height: px(7.0), flex_shrink: 0.0, ..default() }, 4.0), BackgroundColor(colour)));
}

fn header_row(p: &mut ChildSpawnerCommands, fonts: &Fonts, columns: &[(&str, f32, bool)]) {
    p.spawn((Node { align_items: AlignItems::Center, column_gap: px(12.0), padding: UiRect::axes(px(16.0), px(8.0)), border: UiRect::bottom(px(1.0)), ..default() }, BackgroundColor(theme::panel_alt()), BorderColor::all(theme::hairline()))).with_children(|r| {
        for (label, width, right) in columns {
            cell(r, *width, *right, |c| words(c, fonts, label.to_uppercase(), 10.0, Weight::Semibold, theme::ink_faint()));
        }
    });
}

fn row(p: &mut ChildSpawnerCommands, selected: bool, select: Option<String>, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    let mut e = p.spawn((
        Node { align_items: AlignItems::Center, column_gap: px(12.0), padding: UiRect::axes(px(16.0), px(0.0)), min_height: px(31.0), border: UiRect::bottom(px(1.0)), ..default() },
        BackgroundColor(if selected { theme::accent_wash() } else { Color::NONE }),
        BorderColor::all(theme::rule()),
    ));
    e.with_children(build);
    if let Some(path) = select {
        // A script calls the row by what the thing is called.
        e.insert(theme::Named { name: theme::spoken(&path), scrolls: true });
        e.observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
            if press.button == PointerButton::Primary && doc.entry.is_none() {
                doc.select(&path);
            }
        });
    }
}

fn table(p: &mut ChildSpawnerCommands, fonts: &Fonts, title: &str, count: String, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    // A table is a sheet of its own, lifted a little off the page: the estimate is dense
    // text and is not glass, but it belongs to the same program (VISUAL.md F4).
    p.spawn((
        Node { flex_direction: FlexDirection::Column, overflow: Overflow::clip(), flex_shrink: 0.0, border: UiRect::all(px(1.0)), border_radius: BorderRadius::all(px(theme::BUTTON_RADIUS + 4.0)), ..default() },
        BackgroundColor(Color::WHITE),
        BorderColor::all(theme::control_edge()),
        theme::card_shadow(),
    ))
    .with_children(|c| {
        c.spawn(Node { align_items: AlignItems::Baseline, column_gap: px(8.0), padding: UiRect { left: px(16.0), right: px(16.0), top: px(13.0), bottom: px(11.0) }, ..default() }).with_children(|h| {
            h.spawn(text(fonts, title, 14.0, Weight::Semibold, theme::ink()));
            h.spawn(text(fonts, count, 12.0, Weight::Regular, theme::ink_faint()));
        });
        build(c);
    });
}

fn nothing(p: &mut ChildSpawnerCommands, fonts: &Fonts, what: &str) {
    p.spawn(theme::card(Node { padding: UiRect::axes(px(16.0), px(14.0)), flex_shrink: 0.0, ..default() })).with_child(text(fonts, what, 12.5, Weight::Regular, theme::ink_soft()));
}

/// Draw the sheet again whenever the snapshot has changed.
#[allow(clippy::too_many_arguments)]
pub fn rebuild(mut commands: Commands, doc: Res<Doc>, snap: Res<Snapshot>, fonts: Res<Fonts>, cameras: Res<Cameras>, scrolls: Res<Scrolls>, old: Query<Entity, With<SheetUi>>, mut drawn: Local<u64>) {
    if *drawn == snap.revision {
        return;
    }
    *drawn = snap.revision;
    for e in &old {
        commands.entity(e).despawn();
    }
    let fonts = &*fonts;
    // `min_width` 0: the sheet is as wide as its window and no wider, however long a line
    // of text in it is. Without it the window's own layout lets the sheet grow to fit its
    // longest line, and the right-hand columns fall off the edge.
    commands.spawn((Node { width: percent(100.0), max_width: percent(100.0), min_width: px(0.0), height: percent(100.0), flex_direction: FlexDirection::Column, overflow: Overflow::clip(), ..default() }, BackgroundColor(theme::canvas()), UiTargetCamera(cameras.sheet), SheetUi)).with_children(|root| {
        // The sheet's own title bar.
        root.spawn((Node { height: px(52.0), flex_shrink: 0.0, padding: UiRect::horizontal(px(18.0)), align_items: AlignItems::Center, column_gap: px(12.0), border: UiRect::bottom(px(1.0)), overflow: Overflow::clip(), ..default() }, BackgroundColor(theme::panel()), BorderColor::all(theme::hairline()))).with_children(|bar| {
            wordmark(bar, fonts);
            bar.spawn((Node { width: px(1.0), height: px(22.0), margin: UiRect::horizontal(px(4.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(theme::hairline())));
            bar.spawn(text(fonts, "Sheet", 13.5, Weight::Semibold, theme::ink()));
            bar.spawn(Node { flex_grow: 1.0, ..default() });
            tally_chips(bar, fonts, snap.tally);
        });
        // The tabs.
        root.spawn((Node { flex_shrink: 0.0, padding: UiRect::axes(px(10.0), px(8.0)), column_gap: px(1.0), border: UiRect::bottom(px(1.0)), overflow: Overflow::clip(), ..default() }, BackgroundColor(theme::panel()), BorderColor::all(theme::hairline()))).with_children(|tabs| {
            // The estimate first: it is what the job is for. Then what is to verify and
            // what clashes; then the takeoff, by what was taken off.
            let mut all = vec![(Tab::Estimate, "Estimate".to_string(), snap.estimate.as_ref().map(|e| e.lines.len()).unwrap_or(0))];
            // The budget: the job priced from its allowances and from its lines, compared.
            all.push((Tab::Money, "Budget".to_string(), snap.pricing.as_ref().map(|p| p.elements.len()).unwrap_or(0)));
            all.push((Tab::Verify, "Verify in field".to_string(), snap.verify.questions.len()));
            all.push((Tab::Clashes, "Clashes".to_string(), snap.clashes.as_ref().and_then(|c| c.as_ref().ok()).map(|c| c.found.len()).unwrap_or(0)));
            all.push((Tab::Building, "Building".to_string(), snap.fabric.len()));
            all.push((Tab::Roof, "Roof".to_string(), snap.planes.len()));
            for d in [Discipline::Mechanical, Discipline::Electrical, Discipline::Plumbing] {
                all.push((Tab::Of(d), theme::discipline_name(d).to_string(), snap.counts.iter().find(|c| c.0 == d).map(|c| c.1).unwrap_or(0)));
            }
            for (tab, name, count) in all {
                let on = doc.tab == tab;
                // A job with nothing under a tab does not offer it.
                if count == 0 && !on {
                    continue;
                }
                tabs.spawn((rounded(Node { padding: UiRect::axes(px(8.0), px(6.0)), column_gap: px(5.0), align_items: AlignItems::Center, flex_shrink: 0.0, ..default() }, 8.0), BackgroundColor(if on { theme::accent_wash() } else { Color::NONE }), theme::named(name.clone())))
                    .with_children(|t| {
                        t.spawn((text(fonts, name, 12.0, Weight::Medium, if on { theme::accent() } else { theme::ink_soft() }), TextLayout::no_wrap()));
                        t.spawn((text(fonts, count.to_string(), 11.0, Weight::Regular, theme::ink_faint()), TextLayout::no_wrap()));
                    })
                    .observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
                        if press.button == PointerButton::Primary && doc.tab != tab {
                            doc.tab = tab;
                            doc.touch();
                        }
                    });
            }
        });
        root.spawn((Node { flex_grow: 1.0, flex_basis: px(0.0), min_height: px(0.0), flex_direction: FlexDirection::Column, padding: UiRect::all(px(18.0)), row_gap: px(14.0), overflow: Overflow::scroll_y(), ..default() }, ScrollPosition(Vec2::new(0.0, scrolls.sheet)), SheetScroll)).with_children(|body| {
            body.spawn(crate::panels::thumb(1));
            match doc.tab {
                Tab::Roof => {
                    if snap.totals.is_none() {
                        nothing(body, fonts, "This job has no roof.");
                    }
                    roof_summary(body, fonts, &snap);
                    planes(body, fonts, &doc, &snap);
                    lines(body, fonts, &doc, &snap);
                }
                Tab::Building => building(body, fonts, &doc, &snap),
                Tab::Of(discipline) => systems(body, fonts, &doc, &snap, discipline),
                Tab::Money => money(body, fonts, &doc, &snap),
                Tab::Estimate => estimate(body, fonts, &doc, &snap),
                Tab::Clashes => clashes(body, fonts, &doc, &snap),
                Tab::Verify => verify(body, fonts, &doc, &snap),
            }
            notes(body, fonts, &snap);
            cost_lines(body, fonts, &snap);
        });
        // The last thing that happened. It cannot push the sheet wider than its window.
        let first = one_line(&doc.message, 110);
        let (tone_dot, ink) = tone_ink(doc.tone);
        root.spawn((Node { height: px(32.0), flex_shrink: 0.0, padding: UiRect::horizontal(px(16.0)), align_items: AlignItems::Center, column_gap: px(9.0), border: UiRect::top(px(1.0)), overflow: Overflow::clip(), ..default() }, BackgroundColor(theme::panel()), BorderColor::all(theme::hairline()))).with_children(|bar| {
            bar.spawn((rounded(Node { width: px(7.0), height: px(7.0), flex_shrink: 0.0, ..default() }, 4.0), BackgroundColor(tone_dot)));
            bar.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), overflow: Overflow::clip(), ..default() }).with_children(|m| words(m, fonts, first, 12.0, Weight::Regular, ink));
        });
    });
}

fn roof_summary(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot) {
    let Some(totals) = &snap.totals else { return };
    let squares = view::in_squares(&totals.surface);
    let card = |r: &mut ChildSpawnerCommands, label: &str, sum: &view::Sum, under: &str| match (sum.value, sum.members) {
        (_, 0) => stat_card(r, fonts, label, "none".into(), "", Status::Unknown, "the roof has no planes".into()),
        (Some(x), _) => stat_card(r, fonts, label, view::figure_shown(x, 2), sum.unit, sum.status, under.into()),
        (None, n) if sum.missing.is_empty() => stat_card(r, fonts, label, "no figure".into(), "", Status::Unknown, format!("too small to say in {} ({n} planes)", sum.unit)),
        (None, n) => stat_card(r, fonts, label, "no total".into(), "", Status::Unknown, format!("{} of {n} have no figure", sum.missing.len())),
    };
    p.spawn(Node { column_gap: px(14.0), flex_shrink: 0.0, ..default() }).with_children(|r| {
        card(r, "Surface", &totals.surface, "on the slope");
        card(r, "Squares", &squares, "100 SF each");
        match totals.predominant_pitch {
            Some((pitch, status)) => stat_card(r, fonts, "Predominant pitch", theme::pitch_text(pitch), "", status, "most of the surface".into()),
            None => stat_card(r, fonts, "Predominant pitch", "not said".into(), "", Status::Unknown, totals.pitch_note.clone()),
        }
        stat_card(r, fonts, "Facets", totals.facets.to_string(), "", Status::Known, "roof planes".into());
    });
    if totals.lines.is_empty() {
        return;
    }
    p.spawn(theme::card(Node { flex_wrap: FlexWrap::Wrap, flex_shrink: 0.0, padding: UiRect::axes(px(16.0), px(12.0)), column_gap: px(26.0), row_gap: px(10.0), ..default() })).with_children(|c| {
        for (kind, sum) in &totals.lines {
            c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(3.0), ..default() }).with_children(|k| {
                k.spawn(Node { align_items: AlignItems::Center, column_gap: px(7.0), ..default() }).with_children(|t| {
                    swatch(t, theme::line_colour(*kind));
                    t.spawn(text(fonts, format!("{}  ·  {}", theme::kind_name(*kind), sum.members), 11.5, Weight::Medium, theme::ink_soft()));
                });
                k.spawn(text(fonts, total(sum), 14.5, Weight::Semibold, theme::status_ink(sum.status)));
            });
        }
    });
}

/// What in this job is not yet Known, and what each rests on.
fn notes(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot) {
    if snap.notes.is_empty() {
        return;
    }
    table(p, fonts, "To verify", format!("{}", snap.notes.len()), |t| {
        for (status, what, why) in &snap.notes {
            t.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(px(16.0), px(7.0)), row_gap: px(2.0), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::rule()))).with_children(|r| {
                r.spawn(Node { align_items: AlignItems::Center, column_gap: px(9.0), ..default() }).with_children(|top| {
                    dot(top, *status);
                    top.spawn(text(fonts, what.clone(), 12.5, Weight::Medium, theme::status_ink(*status)));
                });
                r.spawn(Node { padding: UiRect::left(px(16.0)), ..default() }).with_child(text(fonts, why.clone(), 11.5, Weight::Regular, theme::ink_faint()));
            });
        }
    });
}

fn planes(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    if snap.planes.is_empty() {
        return;
    }
    table(p, fonts, "Planes", format!("{}", snap.planes.len()), |t| {
        header_row(t, fonts, &[("", 7.0, false), ("Plane", 0.0, false), ("Pitch", 80.0, true), ("Surface", 120.0, true), ("Condition", 250.0, false)]);
        for plane in &snap.planes {
            let selected = doc.selected.as_deref() == Some(plane.path.as_str());
            row(t, selected, Some(plane.path.clone()), |r| {
                cell(r, 7.0, false, |c| dot(c, plane.worst()));
                cell(r, 0.0, false, |c| {
                    words(c, fonts, plane.name.clone(), 12.5, Weight::Medium, theme::ink());
                });
                cell(r, 80.0, true, |c| match plane.pitch {
                    Some(x) => words(c, fonts, theme::pitch_text(x), 12.5, Weight::Regular, theme::status_ink(plane.pitch_status)),
                    None => words(c, fonts, "unknown", 12.5, Weight::Regular, theme::red()),
                });
                cell(r, 120.0, true, |c| words(c, fonts, amount(plane.area, 2, "SF"), 12.5, Weight::Medium, theme::status_ink(plane.area_status)));
                cell(r, 250.0, false, |c| match &plane.condition {
                    Some(name) => {
                        let colour = if name.to_lowercase().contains("copper") { theme::roof_copper() } else { theme::roof_other() };
                        c.spawn((rounded(Node { width: px(10.0), height: px(10.0), flex_shrink: 0.0, ..default() }, 3.0), BackgroundColor(colour)));
                        words(c, fonts, name.clone(), 12.5, Weight::Regular, theme::ink());
                    }
                    None => words(c, fonts, "none yet", 12.5, Weight::Regular, theme::ink_faint()),
                });
            });
        }
        if let Some(totals) = &snap.totals {
            t.spawn((Node { align_items: AlignItems::Center, column_gap: px(12.0), padding: UiRect::axes(px(16.0), px(0.0)), min_height: px(34.0), ..default() }, BackgroundColor(theme::panel_alt()))).with_children(|r| {
                cell(r, 7.0, false, |c| dot(c, totals.surface.status));
                cell(r, 0.0, false, |c| words(c, fonts, "Total", 12.5, Weight::Semibold, theme::ink()));
                cell(r, 80.0, true, |_| {});
                cell(r, 120.0, true, |c| words(c, fonts, if totals.surface.value.is_some() { amount(totals.surface.value, 2, "SF") } else { "no total".into() }, 12.5, Weight::Semibold, theme::status_ink(totals.surface.status)));
                cell(r, 250.0, false, |c| {
                    if !totals.surface.missing.is_empty() {
                        words(c, fonts, format!("{} of {} have no area", totals.surface.missing.len(), totals.surface.members), 12.0, Weight::Regular, theme::red());
                    }
                });
            });
        }
    });
}

fn lines(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    if snap.lines.is_empty() {
        return;
    }
    table(p, fonts, "Lines", format!("{}", snap.lines.len()), |t| {
        header_row(t, fonts, &[("", 7.0, false), ("Line", 0.0, false), ("Kind", 130.0, false), ("On the tape", 120.0, true), ("Length", 110.0, true)]);
        for line in &snap.lines {
            let selected = doc.selected.as_deref() == Some(line.path.as_str());
            // A plane's own lines are marked while the plane is selected.
            let touched = doc.selected.as_ref().is_some_and(|s| line.planes.contains(s));
            row(t, selected, Some(line.path.clone()), |r| {
                cell(r, 7.0, false, |c| dot(c, line.status));
                cell(r, 0.0, false, |c| {
                    words(c, fonts, line.name.clone(), 12.5, if touched { Weight::Semibold } else { Weight::Medium }, if touched { theme::accent() } else { theme::ink() });
                });
                cell(r, 130.0, false, |c| {
                    swatch(c, theme::line_colour(line.kind));
                    words(c, fonts, theme::kind_name(line.kind), 12.5, Weight::Regular, theme::ink_soft());
                });
                cell(r, 120.0, true, |c| words(c, fonts, line.length.map(|x| units::feet_as_ft_in(x, 16)).unwrap_or_default(), 12.0, Weight::Regular, theme::ink_soft()));
                cell(r, 110.0, true, |c| words(c, fonts, amount(line.length, 2, "LF"), 12.5, Weight::Medium, theme::status_ink(line.status)));
            });
        }
    });
}

/// The walls and floors, storey by storey.
fn building(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    if snap.storeys.is_empty() {
        nothing(p, fonts, "This job has no levels: no walls or floors are entered.");
    }
    for storey in &snap.storeys {
        let figure = |el: &view::ElementRows, name: &str| el.rows.iter().find(|r| r.label == name).map(|r| (r.number, r.status)).unwrap_or((None, Status::Unknown));
        let here: Vec<&view::ElementRows> = snap.fabric.iter().filter(|(s, _)| *s == storey.path).map(|(_, el)| el).collect();
        let at = storey.elevation.map(|e| format!("floor at {}", units::feet_as_ft_in(e, 16))).unwrap_or("height unknown".into());
        table(p, fonts, &pretty(&storey.name), format!("{at}  ·  {} doors, {} windows", storey.doors, storey.windows), |t| {
            header_row(t, fonts, &[("", 7.0, false), ("Wall or floor, and its type", 0.0, false), ("Length", 100.0, true), ("Face, one side", 110.0, true), ("Openings", 100.0, true), ("Net, one side", 110.0, true)]);
            let rooms: Vec<&view::ElementRows> = here.iter().copied().filter(|el| el.kind == Kind::Room).collect();
            for el in here.iter().copied().filter(|el| el.kind != Kind::Room) {
                let selected = doc.selected.as_deref() == Some(el.path.as_str());
                let worst = el.rows.iter().filter(|r| r.derived).map(|r| r.status).max().unwrap_or(Status::Unknown);
                row(t, selected, Some(el.path.clone()), |r| {
                    cell(r, 7.0, false, |c| dot(c, worst));
                    cell(r, 0.0, false, |c| {
                        words(c, fonts, el.name.clone(), 12.5, Weight::Medium, theme::ink());
                        // What kind of wall or floor it is: its type, by name (J18-8: ten
                        // walls in a list could not be told a 6 in wall from a 4 in one).
                        if let Some(kind) = el.rows.iter().find(|r| r.label == "type").map(|r| r.text.split(" (").next().unwrap_or("").to_string()).filter(|k| !k.is_empty()) {
                            words(c, fonts, kind, 11.0, Weight::Regular, theme::ink_faint());
                        }
                    });
                    if el.kind == Kind::Wall {
                        for (name, unit, width) in [("plan_length", "LF", 100.0), ("face_area", "SF", 110.0), ("opening_area", "SF", 100.0), ("net_area", "SF", 110.0)] {
                            let (value, status) = figure(el, name);
                            cell(r, width, true, |c| words(c, fonts, amount(value, 2, unit), 12.5, if name == "net_area" { Weight::Medium } else { Weight::Regular }, theme::status_ink(status)));
                        }
                    } else {
                        let (value, status) = figure(el, "plan_area");
                        cell(r, 100.0, true, |_| {});
                        cell(r, 110.0, true, |_| {});
                        cell(r, 100.0, true, |_| {});
                        cell(r, 110.0, true, |c| words(c, fonts, amount(value, 2, "SF"), 12.5, Weight::Medium, theme::status_ink(status)));
                    }
                });
            }
            t.spawn((Node { align_items: AlignItems::Center, column_gap: px(12.0), padding: UiRect::axes(px(16.0), px(0.0)), min_height: px(34.0), ..default() }, BackgroundColor(theme::panel_alt()))).with_children(|r| {
                cell(r, 7.0, false, |c| dot(c, storey.wall_net.status.max(storey.walls.status)));
                cell(r, 0.0, false, |c| words(c, fonts, "Walls, total", 12.5, Weight::Semibold, theme::ink()));
                cell(r, 100.0, true, |c| words(c, fonts, total(&storey.walls), 12.5, Weight::Semibold, theme::status_ink(storey.walls.status)));
                cell(r, 110.0, true, |c| words(c, fonts, total(&storey.wall_face), 12.5, Weight::Semibold, theme::status_ink(storey.wall_face.status)));
                cell(r, 100.0, true, |_| {});
                cell(r, 110.0, true, |c| words(c, fonts, total(&storey.wall_net), 12.5, Weight::Semibold, theme::status_ink(storey.wall_net.status)));
            });
            // The same walls by their type, where the level has more than one: the
            // model's own sums, not the window's.
            if storey.walls_by_type.len() > 1 {
                for of in &storey.walls_by_type {
                    t.spawn((Node { align_items: AlignItems::Center, column_gap: px(12.0), padding: UiRect::axes(px(16.0), px(0.0)), min_height: px(28.0), ..default() }, BackgroundColor(theme::panel_alt()))).with_children(|r| {
                        cell(r, 7.0, false, |c| dot(c, of.net.status.max(of.length.status)));
                        cell(r, 0.0, false, |c| words(c, fonts, format!("of them, {}", of.name), 12.0, Weight::Medium, theme::ink_soft()));
                        cell(r, 100.0, true, |c| words(c, fonts, total(&of.length), 12.0, Weight::Medium, theme::status_ink(of.length.status)));
                        cell(r, 110.0, true, |c| words(c, fonts, total(&of.face), 12.0, Weight::Medium, theme::status_ink(of.face.status)));
                        cell(r, 100.0, true, |_| {});
                        cell(r, 110.0, true, |c| words(c, fonts, total(&of.net), 12.0, Weight::Medium, theme::status_ink(of.net.status)));
                    });
                }
            }
            // The storey's rooms: how far round each is, its area to the centre lines of
            // its walls, and the floor inside them.
            if !rooms.is_empty() {
                header_row(t, fonts, &[("", 7.0, false), ("Room", 0.0, false), ("Round", 100.0, true), ("To wall centers", 110.0, true), ("", 100.0, true), ("Inside the walls", 110.0, true)]);
                for el in rooms {
                    let selected = doc.selected.as_deref() == Some(el.path.as_str());
                    let worst = el.rows.iter().filter(|r| r.derived).map(|r| r.status).max().unwrap_or(Status::Unknown);
                    row(t, selected, Some(el.path.clone()), |r| {
                        cell(r, 7.0, false, |c| dot(c, worst));
                        cell(r, 0.0, false, |c| {
                            words(c, fonts, el.name.clone(), 12.5, Weight::Medium, theme::ink());
                            });
                        for (name, unit, width) in [("perimeter", "LF", 100.0), ("gross_area", "SF", 110.0), ("", "", 100.0), ("floor_area", "SF", 110.0)] {
                            if name.is_empty() {
                                cell(r, width, true, |_| {});
                                continue;
                            }
                            let (value, status) = figure(el, name);
                            cell(r, width, true, |c| words(c, fonts, amount(value, 2, unit), 12.5, if name == "floor_area" { Weight::Medium } else { Weight::Regular }, theme::status_ink(status)));
                        }
                    });
                }
                t.spawn((Node { align_items: AlignItems::Center, column_gap: px(12.0), padding: UiRect::axes(px(16.0), px(0.0)), min_height: px(34.0), ..default() }, BackgroundColor(theme::panel_alt()))).with_children(|r| {
                    cell(r, 7.0, false, |c| dot(c, storey.rooms.status));
                    cell(r, 0.0, false, |c| words(c, fonts, "Rooms, total", 12.5, Weight::Semibold, theme::ink()));
                    cell(r, 100.0, true, |_| {});
                    cell(r, 110.0, true, |_| {});
                    cell(r, 100.0, true, |_| {});
                    cell(r, 110.0, true, |c| words(c, fonts, total(&storey.rooms), 12.5, Weight::Semibold, theme::status_ink(storey.rooms.status)));
                });
            }
        });
    }
}

/// A figure of money as the sheet says it: the amount, or that there is none.
/// What is in the same space as what, then what passes through a wall or a floor, then
/// what was not tested. A row selects the second of its two elements, which is the run or
/// the thing rather than the wall, so the model shows where.
fn clashes(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    use plumbline_world::building::clash::{Clash, ClashKind, NEVER_TESTED};
    let found = match &snap.clashes {
        Some(Ok(found)) => found,
        Some(Err(why)) => return nothing(p, fonts, &format!("Clashes could not be worked out: {why}")),
        None => return nothing(p, fonts, "Clashes have not been worked out."),
    };
    let list = |p: &mut ChildSpawnerCommands, title: &str, rows: Vec<&Clash>, depth: bool| {
        // Between things that are Known a clash is a finding; where either rests on
        // something assumed it is a question for the field.
        let field = rows.iter().filter(|c| c.status != Status::Known).count();
        let found = rows.len() - field;
        let mut parts = vec![rows.len().to_string()];
        if found > 0 {
            parts.push(format!("{found} {}", if found == 1 { "finding" } else { "findings" }));
        }
        if field > 0 {
            parts.push(format!("{field} to verify in the field"));
        }
        let count = parts.join("  ·  ");
        table(p, fonts, title, count, |t| {
            let mut columns = vec![("", 7.0, false), ("What", 0.0, false)];
            if depth {
                columns.push(("In each other by", 120.0, true));
            }
            columns.push(("Where: east, north, up", 190.0, true));
            header_row(t, fonts, &columns);
            for c in rows {
                let selected = doc.selected.as_deref() == Some(c.b.as_str()) || doc.selected.as_deref() == Some(c.a.as_str());
                row(t, selected, Some(c.b.clone()), |r| {
                    cell(r, 7.0, false, |x| dot(x, c.status));
                    cell(r, 0.0, false, |x| named(x, fonts, format!("{} and {}", c.a_name, c.b_name), theme::ink(), format!("{} and {}", theme::kind_name(c.a_kind).to_lowercase(), theme::kind_name(c.b_kind).to_lowercase()), theme::ink_faint()));
                    if depth {
                        let by = c.by.map(|by| format!("{:.2} in", by * 12.0 + 0.0)).unwrap_or("not worked out".into());
                        cell(r, 120.0, true, |x| words(x, fonts, by, 12.5, Weight::Medium, theme::status_ink(c.status)));
                    }
                    let at = format!("{}, {}, {}", units::feet_as_ft_in(c.at[0], 16), units::feet_as_ft_in(c.at[1], 16), units::feet_as_ft_in(c.at[2], 16));
                    cell(r, 190.0, true, |x| words(x, fonts, at, 12.0, Weight::Regular, theme::ink_soft()));
                });
            }
        });
    };
    let hard: Vec<&Clash> = found.found.iter().filter(|c| c.how == ClashKind::Hard && !c.through).collect();
    let through: Vec<&Clash> = found.found.iter().filter(|c| c.how == ClashKind::Hard && c.through).collect();
    if hard.is_empty() && through.is_empty() {
        nothing(p, fonts, &format!("Nothing is in the same space as anything else: {} things tested.", found.tested));
    }
    if !hard.is_empty() {
        list(p, "In the same space", hard, true);
    }
    if !through.is_empty() {
        list(p, "Through a wall or a floor: holes to cut", through, false);
    }
    let count = match found.left_out.len() {
        0 => "everything with a shape was; what follows never is".to_string(),
        n => format!("{n} with no shape; and what follows never is"),
    };
    table(p, fonts, "Not tested", count, |t| {
        for (path, why) in &found.left_out {
            row(t, doc.selected.as_deref() == Some(path.as_str()), Some(path.clone()), |r| {
                cell(r, 7.0, false, |x| dot(x, Status::Unknown));
                cell(r, 0.0, false, |x| named(x, fonts, path.clone(), theme::ink(), why.clone(), theme::ink_soft()));
            });
        }
        for note in NEVER_TESTED {
            row(t, false, None, |r| {
                cell(r, 7.0, false, |_| {});
                cell(r, 0.0, false, |x| words(x, fonts, note, 12.0, Weight::Regular, theme::ink_soft()));
            });
        }
    });
}

/// The verify-in-field list: each question, what to measure or check to settle it, and
/// the dollars riding on it, largest first (`vif::vif`; nothing is added up here). A
/// click on a row selects the thing in the model window and takes the view to it.
fn verify(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    let list = &snap.verify;
    let money = |x: f64| format!("${}", view::figure(x, 2));
    let lines = |n: usize| if n == 1 { "1 line".to_string() } else { format!("{n} lines") };
    if list.questions.is_empty() {
        return nothing(p, fonts, "Nothing to verify: every value that can be measured is Known, and no clash is a question.");
    }
    let riding = match list.riding {
        Some(x) => format!("{} riding  ·  {} of the estimate, each counted once", money(x), lines(list.riding_lines)),
        None => "nothing priced rides on these yet".to_string(),
    };
    // What rides on a question and has no price yet is in no figure here, and is said.
    let riding = if list.riding_unpriced == 0 { riding } else { format!("{riding}  ·  {} more with no price yet", lines(list.riding_unpriced)) };
    table(p, fonts, "Verify in field", format!("{}  ·  {riding}", list.questions.len()), |t| {
        header_row(t, fonts, &[("", 7.0, false), ("Riding on it", 118.0, true), ("What", 0.0, false), ("Level", 96.0, false)]);
        for q in &list.questions {
            // A clash's row goes to the run or the thing in it, not to the wall.
            let go = q.clash.as_ref().map(|c| c.other.clone()).unwrap_or_else(|| q.path.clone());
            let selected = doc.selected.as_deref() == Some(go.as_str());
            let mut e = t.spawn((
                Node { align_items: AlignItems::Center, column_gap: px(12.0), padding: UiRect::axes(px(16.0), px(3.0)), min_height: px(38.0), border: UiRect::bottom(px(1.0)), ..default() },
                BackgroundColor(if selected { theme::accent_wash() } else { Color::NONE }),
                BorderColor::all(theme::rule()),
                theme::Named { name: format!("Verify {}", theme::spoken(&go)), scrolls: true },
            ));
            e.with_children(|r| {
                let worst = q.values.iter().map(|v| v.1).max().unwrap_or(Status::Assumed);
                cell(r, 7.0, false, |x| dot(x, worst));
                cell(r, 118.0, true, |x| match q.dollars {
                    Some(d) => named(x, fonts, money(d), theme::status_ink(q.dollars_status), lines(q.lines), theme::ink_faint()),
                    None => words(x, fonts, "no dollars", 12.0, Weight::Regular, theme::ink_faint()),
                });
                cell(r, 0.0, false, |x| match &q.clash {
                    Some(c) if c.through => {
                        named(x, fonts, format!("{} and {}", q.name, c.other_name), theme::ink(), "One passes through the other: a hole to cut. Where is still to verify.".to_string(), theme::ink_soft());
                    }
                    Some(c) => {
                        let by = c.by.map(|by| format!(", by {:.2} in", by * 12.0 + 0.0)).unwrap_or_default();
                        named(x, fonts, format!("{} and {}", q.name, c.other_name), theme::ink(), format!("In the same space{by}. Measure either one."), theme::ink_soft());
                    }
                    None => {
                        let what = |length: bool| q.values.iter().filter(|v| v.2 == length).map(|v| crate::panels::pretty(&v.0).to_lowercase()).collect::<Vec<_>>().join(", ");
                        let (measure, check) = (what(true), what(false));
                        let mut to_do = Vec::new();
                        if !measure.is_empty() {
                            to_do.push(format!("Measure {measure}"));
                        }
                        if !check.is_empty() {
                            to_do.push(format!("{} {check}", if to_do.is_empty() { "Check" } else { "check" }));
                        }
                        named(x, fonts, format!("{} ({})", q.name, theme::kind_name(q.kind).to_lowercase()), theme::ink(), format!("{}.", to_do.join("; ")), theme::ink_soft());
                    }
                });
                cell(r, 96.0, false, |x| words(x, fonts, q.storey.clone(), 12.0, Weight::Regular, theme::ink_soft()));
            });
            e.observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
                if press.button == PointerButton::Primary && doc.entry.is_none() {
                    doc.select(&go);
                    doc.look_at = Some(go.clone());
                }
            });
        }
    });
    if let Some(why) = &list.clashes_not_tested {
        nothing(p, fonts, &format!("The clashes are not on this list: they could not be worked out ({why})."));
    }
}

pub fn dollars(d: Option<&plumbline_world::Derived>) -> (String, Status) {
    match d {
        None => ("none entered".into(), Status::Known),
        Some(d) => match d.value {
            Some(x) if d.unit == "%" => (format!("{} %", view::figure(x, 2)), d.status),
            Some(x) if x < 0.0 => (format!("-${}", view::figure(-x, 2)), d.status),
            Some(x) => (format!("${}", view::figure(x, 2)), d.status),
            None => ("no figure".into(), Status::Unknown),
        },
    }
}

/// The estimate sheet: every cost line under its element with its amount in one of three
/// columns, the recap, and the suppliers side by side. Every figure is the world's
/// (`view::estimate`); nothing is added up here.
fn estimate(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    use plumbline_world::{Class, Column};
    let Some(est) = &snap.estimate else { return };
    if est.lines.is_empty() {
        nothing(p, fonts, "Nothing in this job has a cost line.");
        return;
    }
    let money = |d: &plumbline_world::Derived| dollars(Some(d));
    // The recap first: what the job comes to and how the price is built is what the
    // sheet is opened for. Then the same cost by division, then every line.
    table(p, fonts, "The recap", String::new(), |t| {
        let line = |t: &mut ChildSpawnerCommands, name: String, about: String, note: String, said: String, status: Status, strong: bool| {
            row(t, false, None, |r| {
                cell(r, 7.0, false, |c| dot(c, status));
                cell(r, 0.0, false, |c| {
                    words(c, fonts, name, 12.5, if strong { Weight::Semibold } else { Weight::Medium }, theme::ink());
                    words(c, fonts, about, 11.5, Weight::Regular, theme::ink_faint());
                });
                cell(r, 250.0, true, |c| words(c, fonts, note, 11.5, Weight::Regular, theme::status_ink(Status::Assumed)));
                cell(r, 130.0, true, |c| words(c, fonts, said, if strong { 13.5 } else { 12.5 }, Weight::Semibold, theme::status_ink(status)));
            });
        };
        let lines_of = |n: usize| format!("{n} line{}", if n == 1 { "" } else { "s" });
        let rests = |assumed: f64| if assumed != 0.0 { format!("${} of it rests on something assumed", view::figure(assumed, 2)) } else { String::new() };
        for c in &est.classes {
            match c.total.value {
                Some(x) => {
                    // Hours are held and added to four places and shown to two, as a time sheet has
                    // them (J19-4: "480.0000 hr").
                    let hours = c.hours.map(|(h, _)| format!("  {} hr", view::figure(h, 2))).unwrap_or_default();
                    line(t, c.class.title().to_string(), format!("{}{hours}", lines_of(c.total.members)), rests(c.total.assumed), format!("${}", view::figure(x, 2)), c.total.status, false);
                }
                None => row(t, false, None, |r| {
                    cell(r, 7.0, false, |_| {});
                    cell(r, 0.0, false, |c2| words(c2, fonts, c.class.title(), 12.5, Weight::Medium, theme::ink_soft()));
                    cell(r, 130.0, true, |c2| words(c2, fonts, "none", 12.5, Weight::Regular, theme::ink_faint()));
                }),
            }
        }
        match est.job_cost.value {
            Some(x) => line(t, "Job cost".to_string(), lines_of(est.job_cost.members), rests(est.job_cost.assumed), format!("${}", view::figure(x, 2)), est.job_cost.status, true),
            None => line(t, "Job cost".to_string(), "no line with a cost type comes to a figure".to_string(), String::new(), "none".to_string(), Status::Unknown, true),
        }
        let rate = |v: Option<&plumbline_world::Val<f64>>| v.and_then(|v| v.value.map(|x| format!("at {} %", view::rate(x)))).unwrap_or_default();
        for (name, d, at, strong) in [("Overhead", &est.overhead, rate(est.markup.as_ref().map(|m| &m.overhead)), false), ("Profit", &est.profit, rate(est.markup.as_ref().map(|m| &m.profit)), false), ("Price", &est.price, String::new(), true)] {
            let (said, status) = money(d);
            let field = match name {
                "Overhead" => Some((Entry::Overhead, KeyCode::KeyO)),
                "Profit" => Some((Entry::Profit, KeyCode::KeyR)),
                _ => None,
            };
            match field {
                None => line(t, name.to_string(), at, String::new(), said, status, strong),
                // Overhead and profit are each a percentage, changed here.
                Some((field, key)) => row(t, false, None, |r| {
                    cell(r, 7.0, false, |c| dot(c, status));
                    cell(r, 0.0, false, |c| {
                        words(c, fonts, name, 12.5, Weight::Medium, theme::ink());
                        let typed = match &doc.entry {
                            Some((f, typed)) if *f == field => Some(typed.as_str()),
                            _ => None,
                        };
                        editable(c, fonts, format!("{name} percent"), if at.is_empty() { "not set".to_string() } else { at.clone() }, if at.is_empty() { theme::status_ink(Status::Unknown) } else { theme::ink_soft() }, typed, move |doc| doc.press(key));
                    });
                    cell(r, 130.0, true, |c| words(c, fonts, said.clone(), 12.5, Weight::Semibold, theme::status_ink(status)));
                }),
            }
            // Why there is no figure; or, on a price over a job cost that leaves lines
            // out, that it is the price of what is in it and not of the job.
            if d.value.is_none() || !d.note.is_empty() {
                t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(0.0), bottom: px(6.0) }, ..default() }).with_child(text(fonts, d.note.clone(), 11.5, Weight::Regular, theme::status_ink(if d.value.is_none() { Status::Unknown } else { Status::Assumed })));
            }
        }
        for (who, why) in &est.job_cost.left_out {
            t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(2.0), bottom: px(4.0) }, ..default() }).with_child(text(fonts, format!("left out of the job cost: {who}: {why}"), 11.5, Weight::Regular, theme::status_ink(Status::Unknown)));
        }
        for l in est.lines.iter().filter(|l| l.off) {
            t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(2.0), bottom: px(4.0) }, ..default() }).with_child(text(fonts, format!("switched off, out of every total: {}: {:?} ({})", l.path, l.description, money(&l.amount).0), 11.5, Weight::Regular, theme::ink_soft()));
        }
    });
    // The job cost, division by division: all sixteen, and what is in none.
    table(p, fonts, "By division", format!("{} of 16 with work in them", est.divisions.iter().filter(|d| d.total.members > 0 || !d.total.left_out.is_empty()).count()), |t| {
        for d in &est.divisions {
            row(t, false, None, |r| {
                // "none" is no work in it; "no figure" is work in it and none of it priced
                // yet, with how many lines wait for a price (the same two words as the
                // command line, the workbook and the PM's lens).
                let (waits, has_work) = (d.total.left_out.len(), d.total.members > 0 || !d.total.left_out.is_empty());
                cell(r, 7.0, false, |c| dot(c, if d.total.members > 0 { d.total.status } else if waits > 0 { Status::Unknown } else { Status::Known }));
                cell(r, 24.0, true, |c| words(c, fonts, d.division.number().to_string(), 12.5, Weight::Medium, theme::ink_soft()));
                cell(r, 0.0, false, |c| words(c, fonts, d.division.title(), 12.5, if has_work { Weight::Medium } else { Weight::Regular }, if has_work { theme::ink() } else { theme::ink_faint() }));
                let lines = |n: usize| if n == 1 { "1 line".to_string() } else { format!("{n} lines") };
                let counted = match (d.total.members, waits) {
                    (0, 0) => String::new(),
                    (n, 0) => lines(n),
                    (0, w) => format!("{} to price", lines(w)),
                    (n, w) => format!("{}, {w} to price", lines(n)),
                };
                cell(r, 118.0, true, |c| words(c, fonts, counted, 11.5, Weight::Regular, theme::ink_faint()));
                cell(r, 130.0, true, |c| match d.total.value {
                    Some(x) => words(c, fonts, format!("${}", view::figure(x, 2)), 12.5, Weight::Semibold, theme::status_ink(d.total.status)),
                    None if waits > 0 => words(c, fonts, "no figure", 12.5, Weight::Medium, theme::status_ink(Status::Unknown)),
                    None => words(c, fonts, "none", 12.5, Weight::Regular, theme::ink_faint()),
                });
            });
        }
        if est.no_division.members > 0 {
            row(t, false, None, |r| {
                cell(r, 7.0, false, |c| dot(c, Status::Unknown));
                cell(r, 24.0, true, |_| {});
                cell(r, 0.0, false, |c| words(c, fonts, "In no division: in the job cost, and in none of the sixteen", 12.5, Weight::Medium, theme::status_ink(Status::Unknown)));
                cell(r, 80.0, true, |c| words(c, fonts, if est.no_division.members == 1 { "1 line".to_string() } else { format!("{} lines", est.no_division.members) }, 11.5, Weight::Regular, theme::ink_faint()));
                cell(r, 130.0, true, |c| words(c, fonts, est.no_division.value.map(|x| format!("${}", view::figure(x, 2))).unwrap_or_else(|| "no figure".to_string()), 12.5, Weight::Semibold, theme::status_ink(Status::Unknown)));
            });
        }
    });
    table(p, fonts, "The sheet", format!("{} lines", est.lines.len()), |t| {
        header_row(t, fonts, &[("", 7.0, false), ("Line", 0.0, false), ("Priced", 96.0, true), ("At", 108.0, true), ("Material", 96.0, true), ("Labor", 96.0, true), ("General", 96.0, true)]);
        // The lines, division by division; then those in no division.
        let mut ordered: Vec<(String, String, Status, Vec<&view::SheetLine>)> = Vec::new();
        for d in &est.divisions {
            let its: Vec<&view::SheetLine> = est.lines.iter().filter(|l| l.division == Some(d.division)).collect();
            if !its.is_empty() {
                let (said, status) = match d.total.value {
                    Some(x) => (format!("${}", view::figure(x, 2)), d.total.status),
                    None => ("none".to_string(), Status::Known),
                };
                ordered.push((format!("Division {}  {}", d.division.number(), d.division.title()), said, status, its));
            }
        }
        let undivided: Vec<&view::SheetLine> = est.lines.iter().filter(|l| l.division.is_none()).collect();
        if !undivided.is_empty() {
            ordered.push(("No division".to_string(), est.no_division.value.map(|x| format!("${}", view::figure(x, 2))).unwrap_or_default(), Status::Unknown, undivided));
        }
        for (heading, said, status, its) in &ordered {
        t.spawn((Node { align_items: AlignItems::Center, column_gap: px(12.0), padding: UiRect::axes(px(16.0), px(5.0)), border: UiRect::bottom(px(1.0)), ..default() }, BackgroundColor(theme::panel_alt()), BorderColor::all(theme::hairline()))).with_children(|r| {
            cell(r, 0.0, false, |c| words(c, fonts, heading.to_uppercase(), 11.0, Weight::Semibold, if heading == "No division" { theme::status_ink(Status::Unknown) } else { theme::ink() }));
            cell(r, 130.0, true, |c| words(c, fonts, said.clone(), 12.5, Weight::Semibold, theme::status_ink(*status)));
        });
        let mut last = String::new();
        for l in its.iter().copied() {
            let selected = doc.selected.as_deref() == Some(l.path.as_str());
            if l.path != last {
                last = l.path.clone();
                row(t, selected, Some(l.path.clone()), |r| {
                    cell(r, 7.0, false, |_| {});
                    cell(r, 0.0, false, |c| {
                        words(c, fonts, l.element.clone(), 12.5, Weight::Semibold, theme::ink());
                    });
                });
            }
            let status = if l.amount.value.is_some() { l.amount.status } else { Status::Unknown };
            row(t, selected, Some(l.path.clone()), |r| {
                cell(r, 7.0, false, |c| dot(c, if l.off { Status::Known } else { status }));
                let tag = match (l.class, &l.material) {
                    (_, Some(m)) if !m.is_empty() => format!("made of {m}"),
                    (_, Some(_)) => "material not said".to_string(),
                    (Some(c), None) if !matches!(c, Class::Material | Class::Labor) => c.title().to_string(),
                    (None, None) => "no cost type".to_string(),
                    _ => String::new(),
                };
                // What is being typed on this line, if anything is.
                let typing = |field: Entry| match &doc.entry {
                    Some((f, typed)) if *f == field && selected && doc.entry_line.as_deref() == Some(l.description.as_str()) => Some(typed.as_str()),
                    _ => None,
                };
                let (path, line) = (l.path.clone(), l.description.clone());
                r.spawn(Node { flex_grow: 1.0, flex_basis: px(0.0), min_width: px(0.0), flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(4.0)), row_gap: px(2.0), overflow: Overflow::clip(), ..default() }).with_children(|c| {
                    c.spawn(text(fonts, l.description.clone(), 12.5, Weight::Regular, if l.off { theme::ink_faint() } else { theme::ink() }));
                    c.spawn(Node { align_items: AlignItems::Center, column_gap: px(8.0), flex_wrap: FlexWrap::Wrap, row_gap: px(2.0), ..default() }).with_children(|under| {
                        if !tag.is_empty() {
                            under.spawn((text(fonts, tag.clone(), 11.0, Weight::Regular, if l.class.is_none() { theme::status_ink(Status::Unknown) } else { theme::ink_faint() }), TextLayout::no_wrap()));
                        }
                        // Its division, said and changed here. A line with none says so.
                        {
                            let (path, line) = (path.clone(), line.clone());
                            let said = l.division.map(|d| format!("Div {}", d.number())).unwrap_or_else(|| "no division".to_string());
                            editable(under, fonts, format!("Division of {} on {}", l.description, l.element), said, if l.division.is_some() { theme::ink_soft() } else { theme::status_ink(Status::Unknown) }, typing(Entry::Division), move |doc| {
                                doc.select(&path);
                                doc.open(Entry::Division, "");
                                doc.entry_line = Some(line.clone());
                            });
                        }
                        // Its stretch-out, changed here: it belongs to the line.
                        if let Some(so) = &l.stretch_out {
                            under.spawn((text(fonts, "stretch-out", 11.0, Weight::Regular, theme::ink_faint()), TextLayout::no_wrap()));
                            let said = so.value.as_ref().map(|v| units::show_length(v, &so.unit)).unwrap_or_else(|| "not set".to_string());
                            let (path, line) = (path.clone(), line.clone());
                            editable(under, fonts, format!("Stretch-out of {} on {}", l.description, l.element), said, theme::status_ink(so.status), typing(Entry::StretchOut), move |doc| {
                                doc.select(&path);
                                doc.open(Entry::StretchOut, "");
                                doc.entry_line = Some(line.clone());
                            });
                        }
                        // In-house labor is switched off or on, line by line.
                        if l.class == Some(Class::Labor) {
                            let (path, line, off) = (path.clone(), line.clone(), l.off);
                            editable(under, fonts, format!("Labor of {} on {}", l.description, l.element), if off { "Off: switch on".to_string() } else { "On: switch off".to_string() }, if off { theme::ink_faint() } else { theme::ink_soft() }, None, move |doc| {
                                match doc.job.set_off(&path, Some(&line), !off) {
                                    Ok(()) => {
                                        doc.job.derive();
                                        doc.edit("Labor on or off");
                                        doc.say(Tone::Done, format!("{path}: {line:?} switched {}.", if off { "on" } else { "off" }));
                                    }
                                    Err(why) => crate::tools::cant(doc, "switch labor", why),
                                }
                                doc.redraw();
                            });
                        }
                    });
                });
                // Hours are held and added to four places and shown to two, as a time
                // sheet has them.
                let places = 2;
                cell(r, 96.0, true, |c| words(c, fonts, amount(l.priced.value, places, &l.priced.unit), 12.5, Weight::Regular, theme::status_ink(l.priced.status)));
                cell(r, 108.0, true, |c| {
                    let (said, ink) = match l.unit_cost.as_ref().and_then(|v| v.value.map(|x| (x, v))) {
                        Some((x, v)) => (format!("${}/{}", view::rate(x), v.unit.trim_start_matches("$/")), theme::status_ink(v.status)),
                        None => ("no price".to_string(), theme::status_ink(Status::Unknown)),
                    };
                    // A line with a price of its own is priced here. One made of a
                    // material takes its material's price, which is the price list's.
                    if l.material.is_none() {
                        let (path, line) = (l.path.clone(), l.description.clone());
                        editable(c, fonts, format!("Unit cost of {} on {}", l.description, l.element), said, ink, typing(Entry::UnitCost), move |doc| {
                            doc.select(&path);
                            doc.open(Entry::UnitCost, "");
                            doc.entry_line = Some(line.clone());
                        });
                    } else {
                        words(c, fonts, said, 12.5, Weight::Regular, ink);
                    }
                });
                let (said, ink) = match (l.off, l.amount.value) {
                    (true, _) => (format!("off ({})", money(&l.amount).0), theme::ink_faint()),
                    (false, Some(_)) => (money(&l.amount).0, theme::status_ink(l.amount.status)),
                    (false, None) => ("no figure".to_string(), theme::status_ink(Status::Unknown)),
                };
                for col in [Column::Material, Column::Labor, Column::General] {
                    // A line with no class stands in no column: its amount is said under it.
                    let here = l.class.map(Class::column) == Some(col);
                    cell(r, 96.0, true, |c| words(c, fonts, if here { said.clone() } else { String::new() }, 12.5, Weight::Regular, ink));
                }
            });
            let mut under: Vec<(String, Status)> = Vec::new();
            if l.amount.value.is_none() || !l.amount.note.is_empty() {
                under.push((l.amount.note.clone(), if l.amount.value.is_none() { Status::Unknown } else { Status::Assumed }));
            }
            if l.class.is_none() && !l.off {
                under.push((format!("no cost type: it is in no column and no row of the recap{}", l.amount.value.map(|_| format!(" (it comes to {})", money(&l.amount).0)).unwrap_or_default()), Status::Unknown));
            }
            if !l.quantity.note.is_empty() && l.quantity.value.is_some() {
                under.push((format!("its quantity: {}", l.quantity.note), Status::Assumed));
            }
            for (said, status) in under {
                t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(0.0), bottom: px(7.0) }, ..default() }).with_child(text(fonts, said, 11.5, Weight::Regular, theme::status_ink(status)));
            }
        }
        }
        row(t, false, None, |r| {
            cell(r, 7.0, false, |_| {});
            cell(r, 0.0, false, |c| words(c, fonts, "Column totals", 12.5, Weight::Semibold, theme::ink()));
            cell(r, 96.0, true, |_| {});
            cell(r, 108.0, true, |_| {});
            for col in [Column::Material, Column::Labor, Column::General] {
                let total = est.columns.iter().find(|x| x.0 == col).map(|x| &x.1);
                cell(r, 96.0, true, |c| match total.and_then(|t| t.value.map(|x| (x, t.status))) {
                    Some((x, status)) => words(c, fonts, format!("${}", view::figure(x, 2)), 12.5, Weight::Semibold, theme::status_ink(status)),
                    None => words(c, fonts, "none", 12.5, Weight::Semibold, theme::ink_faint()),
                });
            }
        });
    });
    let made: Vec<&view::SheetLine> = est.lines.iter().filter(|l| l.material.is_some() && !l.off).collect();
    if !made.is_empty() {
        table(p, fonts, "Suppliers, side by side", format!("{} lines made of a material", made.len()), |t| {
            let width = if est.suppliers.len() > 3 { 96.0 } else { 118.0 };
            let mut columns: Vec<(&str, f32, bool)> = vec![("", 7.0, false), ("Line", 0.0, false), ("Generic", width, true)];
            columns.extend(est.suppliers.iter().map(|s| (s.as_str(), width, true)));
            header_row(t, fonts, &columns);
            for l in &made {
                row(t, doc.selected.as_deref() == Some(l.path.as_str()), Some(l.path.clone()), |r| {
                    cell(r, 7.0, false, |c| dot(c, if l.amount.value.is_some() { l.amount.status } else { Status::Unknown }));
                    named(r, fonts, format!("{}: {}", l.element, l.description), theme::ink(), l.material.clone().unwrap_or_default(), theme::ink_faint());
                    let (said, status) = money(&l.amount);
                    cell(r, width, true, |c| words(c, fonts, said, 12.5, Weight::Medium, theme::status_ink(status)));
                    for s in &est.suppliers {
                        cell(r, width, true, |c| match l.quotes.get(s) {
                            Some(d) => {
                                let (said, status) = money(d);
                                words(c, fonts, said, 12.5, Weight::Regular, theme::status_ink(status))
                            }
                            None => words(c, fonts, "no quote", 12.5, Weight::Regular, theme::status_ink(Status::Unknown)),
                        });
                    }
                });
            }
            let totals: Vec<&view::MoneyTotal> = std::iter::once(&est.at_generic).chain(est.at_supplier.iter().map(|x| &x.1)).collect();
            row(t, false, None, |r| {
                cell(r, 7.0, false, |_| {});
                cell(r, 0.0, false, |c| words(c, fonts, "Total", 12.5, Weight::Semibold, theme::ink()));
                for total in &totals {
                    cell(r, width, true, |c| match total.value {
                        Some(x) => words(c, fonts, format!("${}", view::figure(x, 2)), 12.5, Weight::Semibold, theme::status_ink(total.status)),
                        None => words(c, fonts, "none", 12.5, Weight::Semibold, theme::ink_faint()),
                    });
                }
            });
            // A total over fewer lines is not the same job: each says how many it is over.
            row(t, false, None, |r| {
                cell(r, 7.0, false, |_| {});
                cell(r, 0.0, false, |c| words(c, fonts, "Lines in the total", 11.5, Weight::Regular, theme::ink_soft()));
                for total in &totals {
                    let short = total.members < made.len();
                    cell(r, width, true, |c| words(c, fonts, format!("{} of {}", total.members, made.len()), 11.5, Weight::Regular, if short { theme::status_ink(Status::Unknown) } else { theme::ink_soft() }));
                }
            });
            for (s, total) in std::iter::once(("generic", &est.at_generic)).chain(est.at_supplier.iter().map(|(s, t)| (s.as_str(), t))) {
                for (who, why) in &total.left_out {
                    t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(2.0), bottom: px(4.0) }, ..default() }).with_child(text(fonts, format!("{s} leaves out {who}: {why}"), 11.5, Weight::Regular, theme::status_ink(Status::Unknown)));
                }
            }
        });
    }
}

/// The job priced both ways. Every figure is the world's (`view::pricing`); nothing is
/// added up here.
fn money(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    let Some(pricing) = &snap.pricing else { return };
    if pricing.elements.is_empty() {
        nothing(p, fonts, "Nothing in this job has an allowance or a cost line. Select something in the model and press B to give it an allowance.");
        return;
    }
    table(p, fonts, "Allowance against detail", format!("{} items", pricing.elements.len()), |t| {
        header_row(t, fonts, &[("", 7.0, false), ("Element", 0.0, false), ("Priced by", 110.0, true), ("Allowance", 110.0, true), ("Detail", 110.0, true), ("Comes to", 90.0, true), ("Difference", 170.0, true)]);
        for e in &pricing.elements {
            let selected = doc.selected.as_deref() == Some(e.path.as_str());
            let worst = [&e.allowance, &e.detail].iter().filter_map(|d| d.as_ref()).map(|d| d.status).max().unwrap_or(Status::Unknown);
            row(t, selected, Some(e.path.clone()), |r| {
                cell(r, 7.0, false, |c| dot(c, worst));
                cell(r, 0.0, false, |c| {
                    words(c, fonts, e.name.clone(), 12.5, Weight::Medium, theme::ink());
                    words(c, fonts, e.path.clone(), 11.0, Weight::Regular, theme::ink_faint());
                });
                cell(r, 110.0, true, |c| match &e.figure {
                    Some((_, unit, d)) => words(c, fonts, amount(d.value, 2, unit), 12.5, Weight::Regular, theme::status_ink(d.status)),
                    None => words(c, fonts, "nothing", 12.5, Weight::Regular, theme::ink_faint()),
                });
                for (d, width, weight) in [(&e.allowance, 110.0, Weight::Regular), (&e.detail, 110.0, Weight::Medium)] {
                    let (said, status) = dollars(d.as_ref());
                    cell(r, width, true, |c| words(c, fonts, said, 12.5, weight, if d.is_none() { theme::ink_faint() } else { theme::status_ink(status) }));
                }
                cell(r, 90.0, true, |c| match e.detail_rate.as_ref().and_then(|d| d.value.map(|x| (x, d))) {
                    Some((x, d)) => words(c, fonts, format!("${} / {}", view::rate(x), d.unit.trim_start_matches("$/")), 12.5, Weight::Regular, theme::status_ink(d.status)),
                    None => words(c, fonts, "", 12.5, Weight::Regular, theme::ink_faint()),
                });
                cell(r, 170.0, true, |c| match (e.difference.as_ref().and_then(|d| d.value.map(|x| (x, d.status))), e.difference_percent.as_ref().and_then(|d| d.value)) {
                    (Some((x, status)), pct) => {
                        let word = if x > 0.0 { "over" } else if x < 0.0 { "under" } else { "even" };
                        words(c, fonts, format!("${} {word}{}", view::figure(x.abs(), 2), pct.map(|v| format!("  {} %", view::figure(v, 2))).unwrap_or_default()), 12.5, Weight::Regular, theme::status_ink(status))
                    }
                    _ => words(c, fonts, "", 12.5, Weight::Regular, theme::ink_faint()),
                });
            });
            // Why a figure is missing is said under the element, not left for a tooltip.
            for d in [&e.allowance, &e.detail].iter().filter_map(|d| d.as_ref()).filter(|d| d.value.is_none()) {
                t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(0.0), bottom: px(7.0) }, ..default() }).with_child(text(fonts, d.note.clone(), 11.5, Weight::Regular, theme::status_ink(Status::Unknown)));
            }
        }
    });
    table(p, fonts, "The job", String::new(), |t| {
        let line = |t: &mut ChildSpawnerCommands, name: &str, total: &view::MoneyTotal, none: &str| {
            row(t, false, None, |r| {
                cell(r, 7.0, false, |c| dot(c, if total.value.is_some() { total.status } else { Status::Unknown }));
                cell(r, 0.0, false, |c| {
                    words(c, fonts, name, 12.5, Weight::Semibold, theme::ink());
                    words(c, fonts, match total.value { Some(_) => format!("over {} item{}", total.members, if total.members == 1 { "" } else { "s" }), None => none.to_string() }, 11.5, Weight::Regular, theme::ink_faint());
                });
                cell(r, 260.0, true, |c| {
                    if total.assumed != 0.0 {
                        words(c, fonts, format!("${} of it rests on something assumed", view::figure(total.assumed, 2)), 11.5, Weight::Regular, theme::status_ink(Status::Assumed));
                    }
                });
                cell(r, 130.0, true, |c| match total.value {
                    Some(x) => words(c, fonts, format!("${}", view::figure(x, 2)), 13.5, Weight::Semibold, theme::status_ink(total.status)),
                    None => words(c, fonts, "none", 13.5, Weight::Semibold, theme::ink_faint()),
                });
            });
            for (path, why) in &total.left_out {
                t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(0.0), bottom: px(6.0) }, ..default() }).with_child(text(fonts, format!("left out: {path}: {why}"), 11.5, Weight::Regular, theme::status_ink(Status::Unknown)));
            }
        };
        line(t, "Allowances", &pricing.allowance, "nothing has an allowance that comes to a figure");
        line(t, "Detail", &pricing.detail, "nothing has a detail that is whole");
        row(t, false, None, |r| {
            cell(r, 7.0, false, |c| dot(c, pricing.both.map(|b| b.2).unwrap_or(Status::Unknown)));
            cell(r, 0.0, false, |c| match pricing.both {
                None => words(c, fonts, "Like for like: nothing has both an allowance and a whole detail", 12.5, Weight::Regular, theme::ink_soft()),
                Some(((a, _), (d, _), status, n)) => {
                    let diff = plumbline_world::lines::cents(d - a);
                    let said = if diff == 0.0 {
                        "the detail is the same".to_string()
                    } else {
                        let pct = if a != 0.0 { format!(" ({} %)", view::figure(diff / a * 100.0, 2)) } else { String::new() };
                        format!("the detail is ${} {}{pct}", view::figure(diff.abs(), 2), if diff > 0.0 { "over" } else { "under" })
                    };
                    words(c, fonts, format!("Like for like, over the {n} with both: allowances ${}, detail ${}: {said}", view::figure(a, 2), view::figure(d, 2)), 12.5, Weight::Medium, theme::status_ink(status))
                }
            });
        });
        if !pricing.not_priced.is_empty() {
            let n: usize = pricing.not_priced.iter().map(|x| x.1).sum();
            let kinds: Vec<String> = pricing.not_priced.iter().map(|(k, n)| format!("{n} {}", theme::kind_name(*k).to_lowercase())).collect();
            t.spawn(Node { padding: UiRect::axes(px(16.0), px(9.0)), ..default() }).with_child(text(fonts, format!("Not priced at all: {n} that could be ({})", kinds.join(", ")), 11.5, Weight::Regular, theme::ink_faint()));
        }
    });
}

/// One discipline's systems: each one's runs by kind and size, and what is set on it.
fn systems(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot, discipline: Discipline) {
    // The two groups that are not systems (on none, or on one that is not known) belong
    // to no discipline, so every discipline's tab lists them: they are somebody's.
    let mine: Vec<_> = snap.systems.iter().map(|s| &s.takeoff).filter(|t| t.discipline == discipline || t.path.is_empty()).collect();
    if mine.is_empty() {
        nothing(p, fonts, &format!("This job has no {} systems.", theme::discipline_name(discipline).to_lowercase()));
    }
    for t in mine {
        let selected = doc.selected.as_deref() == Some(t.path.as_str());
        let path = t.path.clone();
        p.spawn(theme::card(Node { flex_direction: FlexDirection::Column, overflow: Overflow::clip(), flex_shrink: 0.0, ..default() })).with_children(|c| {
            c.spawn((Node { align_items: AlignItems::Center, column_gap: px(9.0), padding: UiRect { left: px(16.0), right: px(16.0), top: px(12.0), bottom: px(11.0) }, ..default() }, BackgroundColor(if selected { theme::accent_wash() } else { Color::NONE })))
                .with_children(|h| {
                    swatch(h, if t.path.is_empty() { theme::status_ink(t.flag) } else { theme::system_colour(&t.classification) });
                    h.spawn(text(fonts, t.name.clone(), 14.0, Weight::Semibold, if t.path.is_empty() { theme::status_ink(t.flag) } else { theme::ink() }));
                    h.spawn(text(fonts, t.classification.clone(), 12.0, Weight::Regular, theme::ink_faint()));
                    status_chip(h, fonts, t.flag);
                    h.spawn(Node { flex_grow: 1.0, ..default() });
                    h.spawn(text(fonts, format!("{} straight pieces", t.runs.members), 12.0, Weight::Regular, theme::ink_faint()));
                    h.spawn(text(fonts, total(&t.runs), 13.0, Weight::Semibold, if t.runs.members == 0 { theme::ink_faint() } else { theme::status_ink(t.runs.status) }));
                })
                .observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
                    if press.button == PointerButton::Primary && doc.entry.is_none() && !path.is_empty() {
                        doc.select(&path);
                    }
                });
            if !t.lines.is_empty() {
                header_row(c, fonts, &[("", 7.0, false), ("Run", 0.0, false), ("Size", 170.0, false), ("Pieces", 50.0, true), ("", 80.0, true), ("Length", 120.0, true)]);
                for line in &t.lines {
                    row(c, false, None, |r| {
                        cell(r, 7.0, false, |d| dot(d, line.size_status.max(line.length.status)));
                        cell(r, 0.0, false, |d| words(d, fonts, theme::kind_name(line.kind), 12.5, Weight::Medium, theme::ink()));
                        cell(r, 170.0, false, |d| {
                            words(d, fonts, line.size.clone(), 12.5, Weight::Regular, theme::status_ink(line.size_status));
                            status_chip(d, fonts, line.size_status);
                        });
                        cell(r, 50.0, true, |d| words(d, fonts, line.length.members.to_string(), 12.5, Weight::Regular, theme::ink_soft()));
                        cell(r, 80.0, true, |_| {});
                        cell(r, 120.0, true, |d| words(d, fonts, total(&line.length), 12.5, Weight::Medium, theme::status_ink(line.length.status)));
                    });
                }
            }
            for (kind, what, count, status) in &t.fittings {
                row(c, false, None, |r| {
                    cell(r, 7.0, false, |d| dot(d, *status));
                    cell(r, 0.0, false, |d| {
                        words(d, fonts, what.clone(), 12.5, Weight::Medium, theme::status_ink(*status));
                        status_chip(d, fonts, *status);
                    });
                    cell(r, 170.0, false, |d| words(d, fonts, kind.map(|k| k.title()).unwrap_or("Fitting"), 12.5, Weight::Regular, theme::ink_soft()));
                    cell(r, 50.0, true, |_| {});
                    cell(r, 80.0, true, |_| {});
                    cell(r, 120.0, true, |d| words(d, fonts, format!("{count} EA"), 12.5, Weight::Medium, theme::ink()));
                });
            }
            for (kind, what, count, status) in &t.placed {
                row(c, false, None, |r| {
                    cell(r, 7.0, false, |d| dot(d, *status));
                    cell(r, 0.0, false, |d| {
                        words(d, fonts, what.clone(), 12.5, Weight::Medium, theme::status_ink(*status));
                        status_chip(d, fonts, *status);
                    });
                    cell(r, 170.0, false, |d| words(d, fonts, theme::kind_name(*kind), 12.5, Weight::Regular, theme::ink_soft()));
                    cell(r, 50.0, true, |_| {});
                    cell(r, 80.0, true, |_| {});
                    cell(r, 120.0, true, |d| words(d, fonts, format!("{count} EA"), 12.5, Weight::Medium, theme::ink()));
                });
            }
        });
    }
}

/// The selected element's cost lines: each one's inputs, then quantity, waste and order.
fn cost_lines(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot) {
    let Some(el) = &snap.selected else { return };
    let costed: Vec<&view::Row> = el.rows.iter().filter(|r| r.label.contains(": ")).collect();
    if costed.is_empty() {
        return;
    }
    let mut names: Vec<&str> = Vec::new();
    for r in &costed {
        let name = r.label.split_once(": ").map(|x| x.1).unwrap_or("");
        if !names.contains(&name) {
            names.push(name);
        }
    }
    table(p, fonts, "Cost lines", format!("on {}", el.name), |t| {
        for name in names {
            t.spawn((Node { padding: UiRect::axes(px(16.0), px(8.0)), border: UiRect::vertical(px(1.0)), ..default() }, BackgroundColor(theme::panel_alt()), BorderColor::all(theme::hairline()))).with_child(text(fonts, name, 12.5, Weight::Semibold, theme::ink()));
            for r in costed.iter().filter(|r| r.label.split_once(": ").is_some_and(|x| x.1 == name)) {
                let what = r.label.split_once(": ").map(|x| x.0).unwrap_or("");
                row(t, false, None, |c| {
                    cell(c, 7.0, false, |d| dot(d, r.status));
                    cell(c, 110.0, false, |d| words(d, fonts, what, 12.5, if r.derived { Weight::Medium } else { Weight::Regular }, if r.derived { theme::ink() } else { theme::ink_soft() }));
                    cell(c, 120.0, true, |d| words(d, fonts, shown(r), 12.5, Weight::Medium, theme::status_ink(r.status)));
                    cell(c, 0.0, false, |d| {
                        status_chip(d, fonts, r.status);
                        words(d, fonts, if r.note.is_empty() { r.basis.clone() } else { format!("{}  ({})", r.basis, r.note) }, 11.5, Weight::Regular, theme::ink_faint());
                    });
                });
            }
        }
    });
}
