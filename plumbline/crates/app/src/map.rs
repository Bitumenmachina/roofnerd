//! Where each button is on screen, written out for a script that drives the window.
//!
//! The demonstrations move the real pointer and press the real button. Where to click
//! used to be copied into each script as a pair of numbers, and every change to the
//! layout broke every script. With `PLUMBLINE_MAP=<file>` the program writes, whenever
//! it changes, one line for everything on screen that can be clicked:
//!
//!   model<TAB>Wall<TAB>312<TAB>905
//!   sheet<TAB>Estimate<TAB>2044<TAB>73
//!   plan<TAB>640.0<TAB>455.0<TAB>11.25
//!
//! The numbers are places on the whole screen, in pixels. The `plan` line is there on
//! the plan view once it has come to rest: where the plan's zero is on screen, and how
//! many pixels a foot is. Two things with one name are told apart as "Height" and
//! "Height #2", from the top of the screen down. Without the variable nothing is written.

use bevy::prelude::*;
use bevy::ui::{ComputedUiTargetCamera, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::model::{to_view, Doc, Snapshot};
use crate::panels::Cameras;
use crate::scene::{MainCamera, STATUS_H, TOP_H};
use crate::theme::Named;

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn write(
    doc: Res<Doc>,
    snap: Res<Snapshot>,
    cameras: Option<Res<Cameras>>,
    named: Query<(&Named, &ComputedNode, &UiGlobalTransform, &ComputedUiTargetCamera, &InheritedVisibility)>,
    windows: Query<(&Window, Has<PrimaryWindow>)>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<MainCamera>>,
    mut last: Local<String>,
    mut file: Local<Option<Option<String>>>,
) {
    let file = file.get_or_insert_with(|| std::env::var("PLUMBLINE_MAP").ok());
    let (Some(file), Some(cameras)) = (file.as_ref(), cameras) else { return };
    let mut model = (IVec2::ZERO, Vec2::ZERO);
    let mut sheet = (IVec2::ZERO, Vec2::ZERO);
    for (window, primary) in &windows {
        let at = match window.position {
            WindowPosition::At(at) => at,
            _ => IVec2::ZERO,
        };
        if primary {
            model = (at, Vec2::new(window.width(), window.height()));
        } else {
            sheet = (at, Vec2::new(window.width(), window.height()));
        }
    }
    let mut found: Vec<(&str, String, f32, f32)> = Vec::new();
    for (name, node, place, target, visible) in &named {
        if !visible.get() || node.size.x <= 0.0 {
            continue;
        }
        let at = place.translation * node.inverse_scale_factor;
        let on_model = target.get() == Some(cameras.ui);
        // What has scrolled off its panel is not on screen.
        if name.scrolls && on_model && (at.y < TOP_H + 2.0 * crate::theme::GAP || at.y > model.1.y - STATUS_H - 2.0 * crate::theme::GAP) {
            continue;
        }
        // The sheet's tables scroll under its heading and its tabs, over its foot.
        if name.scrolls && !on_model && (at.y < crate::sheet::HEAD_H || at.y > sheet.1.y - crate::sheet::FOOT_H) {
            continue;
        }
        let (window, corner) = if on_model { ("model", model.0) } else { ("sheet", sheet.0) };
        found.push((window, name.name.clone(), at.x + corner.x as f32, at.y + corner.y as f32));
    }
    found.sort_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(&b.1)).then(a.3.total_cmp(&b.3)).then(a.2.total_cmp(&b.2)));
    let mut out = String::new();
    let mut before: Option<(&str, String)> = None;
    let mut count = 1;
    for (window, name, x, y) in &found {
        if before.as_ref().is_some_and(|b| b.0 == *window && b.1 == *name) {
            count += 1;
        } else {
            count = 1;
        }
        before = Some((window, name.clone()));
        let called = if count == 1 { name.clone() } else { format!("{name} #{count}") };
        out.push_str(&format!("{window}\t{called}\t{}\t{}\n", x.round(), y.round()));
    }
    // The plan, once the view is straight down: its zero, and pixels to the foot.
    if let (true, Ok((camera, eye, Projection::Orthographic(_)))) = (doc.plan, camera.single()) {
        let floor = doc.show.storey.as_ref().and_then(|s| snap.storeys.iter().find(|x| x.path == *s)).and_then(|s| s.elevation).unwrap_or(0.0);
        if let (Ok(zero), Ok(one)) = (camera.world_to_viewport(eye, to_view(doc.origin, [0.0, 0.0, floor])), camera.world_to_viewport(eye, to_view(doc.origin, [1.0, 0.0, floor]))) {
            out.push_str(&format!("plan\t{:.2}\t{:.2}\t{:.4}\n", zero.x + model.0.x as f32, zero.y + model.0.y as f32, one.x - zero.x));
        }
    }
    if out != *last {
        // Written whole, then put in place, so a script never reads half of it.
        let part = format!("{file}.part");
        if std::fs::write(&part, &out).is_ok() && std::fs::rename(&part, file).is_ok() {
            *last = out;
        }
    }
}
