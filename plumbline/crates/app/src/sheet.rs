//! The sheet: a second window on the same job, laid out as an estimator's sheet. A tab for
//! the roof, one for the building's walls and floors, and one for each discipline's
//! systems. Each is tables of what the world holds, every figure with its flag, and under
//! them the cost lines of whatever is selected, worked out step by step.
//!
//! Click a row and it is selected in the 3D view; pick something in the 3D view and its
//! row lights here. Nothing on the sheet is worked out by the sheet.

use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;

use crate::model::{Doc, Snapshot, Tab};
use crate::panels::{amount, one_line, pretty, shown, tally_chips, tone_ink, total, wordmark, Cameras, Scrolls};
use crate::theme::{self, rounded, status_chip, swatch, text, Fonts, Weight};
use plumbline_world::value::Status;
use plumbline_world::{units, view, Discipline, Kind};

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
        e.observe(move |press: On<PointerClick>, mut doc: ResMut<Doc>| {
            if press.button == PointerButton::Primary && doc.entry.is_none() {
                doc.select(&path);
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
        root.spawn((Node { flex_shrink: 0.0, padding: UiRect::axes(px(18.0), px(8.0)), column_gap: px(4.0), border: UiRect::bottom(px(1.0)), ..default() }, BackgroundColor(theme::panel()), BorderColor::all(theme::hairline()))).with_children(|tabs| {
            let mut all = vec![(Tab::Roof, "Roof".to_string(), snap.planes.len())];
            all.push((Tab::Building, "Building".to_string(), snap.fabric.len()));
            for d in [Discipline::Mechanical, Discipline::Electrical, Discipline::Plumbing] {
                all.push((Tab::Of(d), theme::discipline_name(d).to_string(), snap.counts.iter().find(|c| c.0 == d).map(|c| c.1).unwrap_or(0)));
            }
            // Money is offered once anything has an allowance or a cost line.
            all.push((Tab::Money, "Money".to_string(), snap.pricing.as_ref().map(|p| p.elements.len()).unwrap_or(0)));
            // The estimate, once anything has a cost line.
            all.push((Tab::Estimate, "Estimate".to_string(), snap.estimate.as_ref().map(|e| e.lines.len()).unwrap_or(0)));
            for (tab, name, count) in all {
                let on = doc.tab == tab;
                // A job with nothing under a tab does not offer it.
                if count == 0 && !on {
                    continue;
                }
                tabs.spawn((rounded(Node { padding: UiRect::axes(px(12.0), px(6.0)), column_gap: px(6.0), align_items: AlignItems::Center, ..default() }, 8.0), BackgroundColor(if on { theme::accent_wash() } else { Color::NONE })))
                    .with_children(|t| {
                        t.spawn(text(fonts, name, 12.5, Weight::Medium, if on { theme::accent() } else { theme::ink_soft() }));
                        t.spawn(text(fonts, count.to_string(), 11.5, Weight::Regular, theme::ink_faint()));
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
    table(p, fonts, "To settle", format!("{}", snap.notes.len()), |t| {
        for (status, what, why) in &snap.notes {
            t.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(px(16.0), px(7.0)), row_gap: px(2.0), border: UiRect::bottom(px(1.0)), ..default() }, BorderColor::all(theme::hex(0xEEF1F5)))).with_children(|r| {
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
                    words(c, fonts, plane.path.clone(), 11.0, Weight::Regular, theme::ink_faint());
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

/// The walls and floors, storey by storey.
fn building(p: &mut ChildSpawnerCommands, fonts: &Fonts, doc: &Doc, snap: &Snapshot) {
    if snap.storeys.is_empty() {
        nothing(p, fonts, "This job has no storeys: no walls or floors are entered.");
    }
    for storey in &snap.storeys {
        let figure = |el: &view::ElementRows, name: &str| el.rows.iter().find(|r| r.label == name).map(|r| (r.number, r.status)).unwrap_or((None, Status::Unknown));
        let here: Vec<&view::ElementRows> = snap.fabric.iter().filter(|(s, _)| *s == storey.path).map(|(_, el)| el).collect();
        let at = storey.elevation.map(|e| format!("floor at {}", units::feet_as_ft_in(e, 16))).unwrap_or("height unknown".into());
        table(p, fonts, &pretty(&storey.name), format!("{at}  ·  {} doors, {} windows", storey.doors, storey.windows), |t| {
            header_row(t, fonts, &[("", 7.0, false), ("Wall or floor", 0.0, false), ("Length", 100.0, true), ("Face", 110.0, true), ("Openings", 100.0, true), ("Net", 110.0, true)]);
            let rooms: Vec<&view::ElementRows> = here.iter().copied().filter(|el| el.kind == Kind::Room).collect();
            for el in here.iter().copied().filter(|el| el.kind != Kind::Room) {
                let selected = doc.selected.as_deref() == Some(el.path.as_str());
                let worst = el.rows.iter().filter(|r| r.derived).map(|r| r.status).max().unwrap_or(Status::Unknown);
                row(t, selected, Some(el.path.clone()), |r| {
                    cell(r, 7.0, false, |c| dot(c, worst));
                    cell(r, 0.0, false, |c| {
                        words(c, fonts, el.name.clone(), 12.5, Weight::Medium, theme::ink());
                        words(c, fonts, el.path.clone(), 11.0, Weight::Regular, theme::ink_faint());
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
            // The storey's rooms: how far round each is, its area to the centre lines of
            // its walls, and the floor inside them.
            if !rooms.is_empty() {
                header_row(t, fonts, &[("", 7.0, false), ("Room", 0.0, false), ("Round", 100.0, true), ("To wall centres", 110.0, true), ("", 100.0, true), ("Inside the walls", 110.0, true)]);
                for el in rooms {
                    let selected = doc.selected.as_deref() == Some(el.path.as_str());
                    let worst = el.rows.iter().filter(|r| r.derived).map(|r| r.status).max().unwrap_or(Status::Unknown);
                    row(t, selected, Some(el.path.clone()), |r| {
                        cell(r, 7.0, false, |c| dot(c, worst));
                        cell(r, 0.0, false, |c| {
                            words(c, fonts, el.name.clone(), 12.5, Weight::Medium, theme::ink());
                            words(c, fonts, el.path.clone(), 11.0, Weight::Regular, theme::ink_faint());
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
fn dollars(d: Option<&plumbline_world::Derived>) -> (String, Status) {
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
    table(p, fonts, "The sheet", format!("{} lines", est.lines.len()), |t| {
        header_row(t, fonts, &[("", 7.0, false), ("Line", 0.0, false), ("Priced", 96.0, true), ("At", 92.0, true), ("Material", 96.0, true), ("Labor", 96.0, true), ("General", 96.0, true)]);
        let mut last = String::new();
        for l in &est.lines {
            let selected = doc.selected.as_deref() == Some(l.path.as_str());
            if l.path != last {
                last = l.path.clone();
                row(t, selected, Some(l.path.clone()), |r| {
                    cell(r, 7.0, false, |_| {});
                    cell(r, 0.0, false, |c| {
                        words(c, fonts, l.element.clone(), 12.5, Weight::Semibold, theme::ink());
                        words(c, fonts, l.path.clone(), 11.0, Weight::Regular, theme::ink_faint());
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
                    (None, None) => "no class".to_string(),
                    _ => String::new(),
                };
                named(r, fonts, l.description.clone(), if l.off { theme::ink_faint() } else { theme::ink() }, tag, if l.class.is_none() { theme::status_ink(Status::Unknown) } else { theme::ink_faint() });
                cell(r, 96.0, true, |c| words(c, fonts, amount(l.priced.value, 2, &l.priced.unit), 12.5, Weight::Regular, theme::status_ink(l.priced.status)));
                cell(r, 92.0, true, |c| match l.unit_cost.as_ref().and_then(|v| v.value.map(|x| (x, v))) {
                    Some((x, v)) => words(c, fonts, format!("${}/{}", view::rate(x), v.unit.trim_start_matches("$/")), 12.5, Weight::Regular, theme::status_ink(v.status)),
                    None => words(c, fonts, "no price", 12.5, Weight::Regular, theme::status_ink(Status::Unknown)),
                });
                let (said, ink) = match (l.off, l.amount.value) {
                    (true, _) => (format!("off ({})", money(&l.amount).0), theme::ink_faint()),
                    (false, Some(_)) => (money(&l.amount).0, theme::status_ink(l.amount.status)),
                    (false, None) => ("no figure".to_string(), theme::status_ink(Status::Unknown)),
                };
                for col in [Column::Material, Column::Labor, Column::General] {
                    // A line with no class stands in no column; its amount is said in the last.
                    let here = l.class.map(Class::column) == Some(col) || (l.class.is_none() && col == Column::General);
                    cell(r, 96.0, true, |c| words(c, fonts, if here { said.clone() } else { String::new() }, 12.5, Weight::Regular, ink));
                }
            });
            if l.amount.value.is_none() || !l.amount.note.is_empty() {
                t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(0.0), bottom: px(7.0) }, ..default() }).with_child(text(fonts, l.amount.note.clone(), 11.5, Weight::Regular, theme::status_ink(if l.amount.value.is_none() { Status::Unknown } else { Status::Assumed })));
            }
        }
        row(t, false, None, |r| {
            cell(r, 7.0, false, |_| {});
            cell(r, 0.0, false, |c| words(c, fonts, "Column totals", 12.5, Weight::Semibold, theme::ink()));
            cell(r, 96.0, true, |_| {});
            cell(r, 92.0, true, |_| {});
            for col in [Column::Material, Column::Labor, Column::General] {
                let total = est.columns.iter().find(|x| x.0 == col).map(|x| &x.1);
                cell(r, 96.0, true, |c| match total.and_then(|t| t.value.map(|x| (x, t.status))) {
                    Some((x, status)) => words(c, fonts, format!("${}", view::figure(x, 2)), 12.5, Weight::Semibold, theme::status_ink(status)),
                    None => words(c, fonts, "none", 12.5, Weight::Semibold, theme::ink_faint()),
                });
            }
        });
    });
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
            None => line(t, "Job cost".to_string(), "no classed line comes to a figure".to_string(), String::new(), "none".to_string(), Status::Unknown, true),
        }
        let rate = |v: Option<&plumbline_world::Val<f64>>| v.and_then(|v| v.value.map(|x| format!("at {} %", view::figure(x, 2)))).unwrap_or_default();
        for (name, d, at, strong) in [("Overhead", &est.overhead, rate(est.markup.as_ref().map(|m| &m.overhead)), false), ("Profit", &est.profit, rate(est.markup.as_ref().map(|m| &m.profit)), false), ("Price", &est.price, String::new(), true)] {
            let (said, status) = money(d);
            line(t, name.to_string(), at, String::new(), said, status, strong);
            if d.value.is_none() {
                t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(0.0), bottom: px(6.0) }, ..default() }).with_child(text(fonts, d.note.clone(), 11.5, Weight::Regular, theme::status_ink(Status::Unknown)));
            }
        }
        for (who, why) in &est.job_cost.left_out {
            t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(2.0), bottom: px(4.0) }, ..default() }).with_child(text(fonts, format!("left out of the job cost: {who}: {why}"), 11.5, Weight::Regular, theme::status_ink(Status::Unknown)));
        }
        for l in est.lines.iter().filter(|l| l.off) {
            t.spawn(Node { padding: UiRect { left: px(35.0), right: px(16.0), top: px(2.0), bottom: px(4.0) }, ..default() }).with_child(text(fonts, format!("switched off, out of every total: {}: {:?} ({})", l.path, l.description, money(&l.amount).0), 11.5, Weight::Regular, theme::ink_soft()));
        }
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
    table(p, fonts, "Allowance against detail", format!("{} elements", pricing.elements.len()), |t| {
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
                    words(c, fonts, match total.value { Some(_) => format!("over {} element{}", total.members, if total.members == 1 { "" } else { "s" }), None => none.to_string() }, 11.5, Weight::Regular, theme::ink_faint());
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
        line(t, "Allowances", &pricing.allowance, "no element has an allowance that comes to a figure");
        line(t, "Detail", &pricing.detail, "no element has a detail that is whole");
        row(t, false, None, |r| {
            cell(r, 7.0, false, |c| dot(c, pricing.both.map(|b| b.2).unwrap_or(Status::Unknown)));
            cell(r, 0.0, false, |c| match pricing.both {
                None => words(c, fonts, "Like for like: no element has both an allowance and a whole detail", 12.5, Weight::Regular, theme::ink_soft()),
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
