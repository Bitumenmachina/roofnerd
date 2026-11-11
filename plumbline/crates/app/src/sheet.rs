//! The sheet: a second window on the same job, laid out as an estimator's sheet. What the
//! roof comes to at the top, then every plane and every line as a row, each with its flag,
//! then the cost lines of whatever is selected, worked out step by step.
//!
//! Click a row and it is selected in the 3D view; pick a plane in the 3D view and its row
//! lights here. Nothing on the sheet is worked out by the sheet.

use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;

use crate::model::{Doc, Snapshot};
use crate::panels::{amount, shown, tally_chips, wordmark, Cameras, Scrolls};
use crate::theme::{self, rounded, status_chip, swatch, text, Fonts, Weight};
use plumbline_world::value::Status;
use plumbline_world::{units, view};

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
        BorderColor::all(theme::hex(0xEEF1F5)),
    ));
    e.with_children(build);
    if let Some(path) = select {
        e.observe(move |click: On<PointerClick>, mut doc: ResMut<Doc>| {
            if click.button == PointerButton::Primary && doc.entry.is_none() {
                doc.selected = Some(path.clone());
                doc.message = format!("selected {path}");
                doc.touch();
            }
        });
    }
}

fn table(p: &mut ChildSpawnerCommands, fonts: &Fonts, title: &str, count: String, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(theme::card(Node { flex_direction: FlexDirection::Column, overflow: Overflow::clip(), flex_shrink: 0.0, ..default() })).with_children(|c| {
        c.spawn(Node { align_items: AlignItems::Baseline, column_gap: px(8.0), padding: UiRect { left: px(16.0), right: px(16.0), top: px(13.0), bottom: px(11.0) }, ..default() }).with_children(|h| {
            h.spawn(text(fonts, title, 14.0, Weight::Semibold, theme::ink()));
            h.spawn(text(fonts, count, 12.0, Weight::Regular, theme::ink_faint()));
        });
        build(c);
    });
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
    commands.spawn((Node { width: percent(100.0), height: percent(100.0), flex_direction: FlexDirection::Column, ..default() }, BackgroundColor(theme::canvas()), UiTargetCamera(cameras.sheet), SheetUi)).with_children(|root| {
        // The sheet's own title bar.
        root.spawn((Node { height: px(52.0), flex_shrink: 0.0, padding: UiRect::horizontal(px(18.0)), align_items: AlignItems::Center, column_gap: px(12.0), border: UiRect::bottom(px(1.0)), ..default() }, BackgroundColor(theme::panel()), BorderColor::all(theme::hairline()))).with_children(|bar| {
            wordmark(bar, fonts);
            bar.spawn((Node { width: px(1.0), height: px(22.0), margin: UiRect::horizontal(px(4.0)), ..default() }, BackgroundColor(theme::hairline())));
            bar.spawn(text(fonts, "Sheet", 13.5, Weight::Semibold, theme::ink()));
            bar.spawn(text(fonts, "›", 14.0, Weight::Regular, theme::ink_faint()));
            bar.spawn(text(fonts, snap.roof_name.clone(), 13.5, Weight::Medium, theme::ink_soft()));
            bar.spawn(Node { flex_grow: 1.0, ..default() });
            tally_chips(bar, fonts, snap.tally);
        });
        root.spawn((Node { flex_grow: 1.0, flex_basis: px(0.0), min_height: px(0.0), flex_direction: FlexDirection::Column, padding: UiRect::all(px(18.0)), row_gap: px(14.0), overflow: Overflow::scroll_y(), ..default() }, ScrollPosition(Vec2::new(0.0, scrolls.sheet)), SheetScroll)).with_children(|body| {
            summary(body, fonts, &snap);
            notes(body, fonts, &snap);
            planes(body, fonts, &doc, &snap);
            lines(body, fonts, &doc, &snap);
            cost_lines(body, fonts, &snap);
        });
        let first = doc.message.lines().next().unwrap_or("").to_string();
        let refused = first.starts_with("refused");
        root.spawn((Node { height: px(32.0), flex_shrink: 0.0, padding: UiRect::horizontal(px(16.0)), align_items: AlignItems::Center, column_gap: px(9.0), border: UiRect::top(px(1.0)), overflow: Overflow::clip(), ..default() }, BackgroundColor(theme::panel()), BorderColor::all(theme::hairline()))).with_children(|bar| {
            bar.spawn((rounded(Node { width: px(7.0), height: px(7.0), flex_shrink: 0.0, ..default() }, 4.0), BackgroundColor(if refused { theme::red() } else { theme::green() })));
            words(bar, fonts, first, 12.0, Weight::Regular, if refused { theme::red() } else { theme::ink_soft() });
        });
    });
}

fn summary(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot) {
    let Some(totals) = &snap.totals else { return };
    let squares = view::in_squares(&totals.surface);
    let gaps = |sum: &view::Sum, what: &str| if sum.missing.is_empty() { what.to_string() } else { format!("{} of {} have no figure", sum.missing.len(), sum.members) };
    p.spawn(Node { column_gap: px(14.0), flex_shrink: 0.0, ..default() }).with_children(|r| {
        stat_card(r, fonts, "Surface", totals.surface.value.map(|x| view::figure_shown(x, 2)).unwrap_or("no total".into()), if totals.surface.value.is_some() { "SF" } else { "" }, totals.surface.status, gaps(&totals.surface, "on the slope"));
        stat_card(r, fonts, "Squares", squares.value.map(|x| view::figure_shown(x, 2)).unwrap_or("no total".into()), if squares.value.is_some() { "SQ" } else { "" }, squares.status, gaps(&squares, "100 SF each"));
        match totals.predominant_pitch {
            Some((pitch, status)) => stat_card(r, fonts, "Predominant pitch", theme::pitch_text(pitch), "", status, "most of the surface".into()),
            None => stat_card(r, fonts, "Predominant pitch", "unknown".into(), "", Status::Unknown, "no plane has a pitch and an area".into()),
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
                k.spawn(text(fonts, if sum.value.is_some() { amount(sum.value, 2, "LF") } else { format!("no total, {} missing", sum.missing.len()) }, 14.5, Weight::Semibold, theme::status_ink(sum.status)));
            });
        }
    });
}

/// What on this roof is not yet Known, and what each rests on.
fn notes(p: &mut ChildSpawnerCommands, fonts: &Fonts, snap: &Snapshot) {
    if snap.notes.is_empty() {
        return;
    }
    table(p, fonts, "To settle", format!("{}", snap.notes.len()), |t| {
        for (status, what, why) in &snap.notes {
            row(t, false, None, |r| {
                cell(r, 7.0, false, |c| dot(c, *status));
                cell(r, 250.0, false, |c| words(c, fonts, what.clone(), 12.5, Weight::Medium, theme::status_ink(*status)));
                cell(r, 0.0, false, |c| words(c, fonts, why.clone(), 11.5, Weight::Regular, theme::ink_faint()));
            });
        }
    });
}

fn planes(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    table(p, fonts, "Planes", format!("{}", snap.planes.len()), |t| {
        header_row(t, fonts, &[("", 7.0, false), ("Plane", 0.0, false), ("Pitch", 70.0, true), ("Surface", 120.0, true), ("Condition", 250.0, false)]);
        for plane in &snap.planes {
            let selected = doc.selected.as_deref() == Some(plane.path.as_str());
            row(t, selected, Some(plane.path.clone()), |r| {
                cell(r, 7.0, false, |c| dot(c, plane.worst()));
                cell(r, 0.0, false, |c| {
                    words(c, fonts, plane.name.clone(), 12.5, Weight::Medium, theme::ink());
                    words(c, fonts, plane.path.clone(), 11.0, Weight::Regular, theme::ink_faint());
                });
                cell(r, 70.0, true, |c| match plane.pitch {
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
                cell(r, 70.0, true, |_| {});
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
                    words(c, fonts, line.path.clone(), 11.0, Weight::Regular, theme::ink_faint());
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
