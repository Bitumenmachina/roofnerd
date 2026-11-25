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
//!
//! Also: `state<TAB>Ground<TAB>off`, one line for each switch of the Show list (and `state<TAB>Level<TAB>all`).
//!
//! Beside it, in `<file>.said`, is every message the window gave, one to a line in the
//! order given, in the words on the screen: a demonstration checks what a person was
//! told, and not only what the saved job holds.

use bevy::prelude::*;
use bevy::ui::{ComputedUiTargetCamera, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use plumbline_world::building::Shape;
use crate::model::{to_view, Doc, Snapshot};
use crate::panels::Cameras;
use crate::scene::{MainCamera, STATUS_H, TOP_H};
use crate::theme::Named;

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn write(
    doc: Res<Doc>,
    orbit: Res<crate::scene::Orbit>,
    snap: Res<Snapshot>,
    cameras: Option<Res<Cameras>>,
    named: Query<(&Named, &ComputedNode, &UiGlobalTransform, &ComputedUiTargetCamera, &InheritedVisibility, Option<&CalculatedClip>)>,
    windows: Query<(&Window, Has<PrimaryWindow>)>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<MainCamera>>,
    texts: Query<(&Text, &ComputedNode, &UiGlobalTransform, &ComputedUiTargetCamera, &InheritedVisibility, Option<&CalculatedClip>)>,
    mut last: Local<String>,
    mut file: Local<Option<Option<String>>>,
    mut said: Local<String>,
) {
    let file = file.get_or_insert_with(|| std::env::var("PLUMBLINE_MAP").ok());
    let (Some(file), Some(cameras)) = (file.as_ref(), cameras) else { return };
    // What the window said, each message once as it changes, as a person reads it: a
    // script that checks the words a person was given reads them from `<file>.said`.
    if *said != doc.message {
        *said = doc.message.clone();
        use std::io::Write;
        if let Ok(mut log) = std::fs::OpenOptions::new().create(true).append(true).open(format!("{file}.said")) {
            // Line by line, as the screen writes it: a name at the head of a second
            // line was left as the job keeps it ("fixture-0030 is one of it") where
            // the screen read "Lavatory is one of it" (J25A, minor).
            let lines: Vec<String> = doc.message.lines().map(crate::theme::spoken).collect();
            let _ = writeln!(log, "{}", crate::panels::pretty(&lines.join(" | ")));
        }
    }
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
    let mut away: Vec<(String, &str)> = Vec::new();
    for (name, node, place, target, visible, clip) in &named {
        if !visible.get() || node.size.x <= 0.0 {
            continue;
        }
        // What a panel has scrolled out of sight, or cut off at its edge, is not on
        // screen: a click there reaches nothing, and a script must not be told it would
        // (J19-37: buttons below the left panel's bottom edge were listed).
        if let Some(c) = clip.filter(|c| !c.contains_point(place.translation)) {
            // Where it has gone is said apart, so that a script can do what a person
            // does: turn the wheel over the panel, the way it lies, until it shows.
            if target.get() == Some(cameras.ui) && place.translation.x * node.inverse_scale_factor < crate::scene::LEFT_W + 2.0 * crate::theme::GAP {
                // Below the bottom of what cuts it off, it is further down; else further up.
                let below = match c {
                    CalculatedClip::Rects(rects) => rects.iter().any(|r| r.world_to_clip_local.transform_point2(place.translation).y > r.rect.max.y),
                    _ => true,
                };
                away.push((name.name.clone(), if below { "down" } else { "up" }));
            }
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
    // The words on both windows that are in sight, as they read: `text<TAB>model or sheet<TAB>the words<TAB>x<TAB>y`. A demonstration that counts what a
    // panel says (a word that stood twice, a line that is there) reads these.
    let mut words: Vec<(&str, String, f32, f32)> = Vec::new();
    for (words_of, node, place, target, visible, clip) in &texts {
        if !visible.get() || node.size.x <= 0.0 || words_of.0.trim().is_empty() {
            continue;
        }
        if clip.is_some_and(|c| !c.contains_point(place.translation)) {
            continue;
        }
        let on_model = target.get() == Some(cameras.ui);
        let at = place.translation * node.inverse_scale_factor;
        let corner = if on_model { model.0 } else { sheet.0 };
        words.push((if on_model { "model" } else { "sheet" }, words_of.0.replace(['\t', '\n'], " "), at.x + corner.x as f32, at.y + corner.y as f32));
    }
    words.sort_by(|a, b| a.0.cmp(b.0).then(a.3.total_cmp(&b.3)).then(a.2.total_cmp(&b.2)).then(a.1.cmp(&b.1)));
    let mut spoken = String::new();
    for (window, w, x, y) in &words {
        spoken.push_str(&format!("text\t{window}\t{w}\t{}\t{}\n", x.round(), y.round()));
    }
    // The box of a card laid over the model (the Cabinets card): `box<TAB>name<TAB>left<TAB>top<TAB>right<TAB>bottom`, so that a script
    // can ask whether it lies over what is drawn (call 197, point 11).
    let mut boxes = String::new();
    for (name, node, place, target, visible, _) in &named {
        if visible.get() && node.size.x > 0.0 && name.name.ends_with(" card") && target.get() == Some(cameras.ui) {
            let (c, half) = (place.translation * node.inverse_scale_factor, node.size * node.inverse_scale_factor / 2.0);
            boxes.push_str(&format!("box\t{}\t{}\t{}\t{}\t{}\n", name.name, (c.x - half.x + model.0.x as f32).round(), (c.y - half.y + model.0.y as f32).round(), (c.x + half.x + model.0.x as f32).round(), (c.y + half.y + model.0.y as f32).round()));
        }
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
    // What each switch of the Show list reads, so that a script can check what a button of
    // a trade set (call 177): `state<TAB>Ground<TAB>off`. And the level on show alone.
    {
        let show = &doc.show;
        let level = show.storey.as_ref().and_then(|s| snap.storeys.iter().find(|x| &x.path == s)).map(|s| s.name.clone()).unwrap_or_else(|| "all".to_string());
        for (name, on) in [
            ("Ground", show.ground),
            ("Finishes", show.finishes),
            ("Framing", show.framing),
            ("Trim", show.trim),
            ("Architecture", show.architecture),
            ("Roof", show.roof),
            ("Mechanical", show.mechanical),
            ("Electrical", show.electrical),
            ("Plumbing", show.plumbing),
            ("See through", show.ghost),
            ("Sizes", show.sizes),
            ("Known, Assumed, Unknown", show.certainty),
            ("Clashes", show.clashes),
        ] {
            out.push_str(&format!("state\t{name}\t{}\n", if on { "on" } else { "off" }));
        }
        out.push_str(&format!("state\tLevel\t{level}\n"));
    }
    out.push_str(&boxes);
    away.sort();
    away.dedup_by(|a, b| a.0 == b.0);
    for (name, way) in &away {
        out.push_str(&format!("away\t{name}\t{way}\n"));
    }
    // The plan, once the view is straight down: its zero, and pixels to the foot.
    if let (true, Ok((camera, eye, Projection::Orthographic(_)))) = (doc.plan, camera.single()) {
        let floor = doc.show.storey.as_ref().and_then(|s| snap.storeys.iter().find(|x| x.path == *s)).and_then(|s| s.elevation).unwrap_or(0.0);
        if let (Ok(zero), Ok(one)) = (camera.world_to_viewport(eye, to_view(doc.origin, [0.0, 0.0, floor])), camera.world_to_viewport(eye, to_view(doc.origin, [1.0, 0.0, floor]))) {
            out.push_str(&format!("plan\t{:.2}\t{:.2}\t{:.4}\n", zero.x + model.0.x as f32, zero.y + model.0.y as f32, one.x - zero.x));
        }
    }
    // What the Job panel says under a level's Floor where its figure is not the rooms' (call 193, F):
    // `note<TAB>Floor<TAB>out to the walls' center lines`.
    for storey in &snap.storeys {
        if let Some(said) = crate::panels::floor_note(&snap, storey) {
            out.push_str(&format!("note\tFloor\t{said}\n"));
        }
    }
    // The count the Trim switch reads (the model's): `count<TAB>Trim<TAB>27`.
    if !snap.trim.pieces.is_empty() {
        out.push_str(&format!("count\tTrim\t{}\n", snap.trim.pieces.len()));
    }
    // Where each piece of outside trim the model lays is on screen, at its two ends (call 184), in 3D with the Trim switch on:
    // `trim<TAB>kind<TAB>the thing it trims<TAB>x1<TAB>y1<TAB>x2<TAB>y2<TAB>its width in pixels`. A demonstration measures a picture against these.
    if let (true, Ok((camera, eye, _))) = (doc.show.trim && !doc.plan && !snap.trim.pieces.is_empty(), camera.single()) {
        for p in &snap.trim.pieces {
            let at = |q: [f64; 3]| camera.world_to_viewport(eye, to_view(doc.origin, q)).ok();
            let (a, b) = (at(p.from), at(p.to));
            // How wide it looks: the board's width carried across the way its face lies, at the middle.
            let mid = [(p.from[0] + p.to[0]) / 2.0, (p.from[1] + p.to[1]) / 2.0, (p.from[2] + p.to[2]) / 2.0];
            let run = [p.to[0] - p.from[0], p.to[1] - p.from[1], p.to[2] - p.from[2]];
            let across = [p.face[1] * run[2] - p.face[2] * run[1], p.face[2] * run[0] - p.face[0] * run[2], p.face[0] * run[1] - p.face[1] * run[0]];
            let long = (across[0] * across[0] + across[1] * across[1] + across[2] * across[2]).sqrt().max(1e-12);
            let side = [mid[0] + across[0] / long * p.width, mid[1] + across[1] / long * p.width, mid[2] + across[2] / long * p.width];
            let wide = match (at(mid), at(side)) {
                (Some(m), Some(n)) => m.distance(n),
                _ => 0.0,
            };
            if let (Some(a), Some(b)) = (a, b) {
                out.push_str(&format!("trim\t{}\t{}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\t{:.1}\n", p.kind.words(), p.of, a.x + model.0.x as f32, a.y + model.0.y as f32, b.x + model.0.x as f32, b.y + model.0.y as f32, wide));
            }
        }
    }
    // The bench (call 195): where the item's own width, depth and height run on screen, along its middle (so that they are
    // the same wherever its sides go when a size is typed), and where the house's south-west corner stood on the ground
    // when the house is in view. `bench<TAB>width<TAB>x1<TAB>y1<TAB>x2<TAB>y2` and `house<TAB>x<TAB>y`. A demonstration measures a
    // picture against these.
    if let Ok((camera, eye, _)) = camera.single() {
        let at = |q: Vec3| camera.world_to_viewport(eye, q).ok().map(|p| (p.x + model.0.x as f32, p.y + model.0.y as f32));
        match (&doc.bench, &snap.bench) {
            (Some(_), Some(l)) => {
                let (w, d, h) = (l.size[0] as f32, l.size[1] as f32, l.size[2] as f32);
                for (what, a, b) in [("width", Vec3::new(-w / 2.0, 0.0, 0.0), Vec3::new(w / 2.0, 0.0, 0.0)), ("depth", Vec3::new(0.0, 0.0, -d / 2.0), Vec3::new(0.0, 0.0, d / 2.0)), ("height", Vec3::ZERO, Vec3::new(0.0, h, 0.0))] {
                    if let (Some(a), Some(b)) = (at(a), at(b)) {
                        out.push_str(&format!("bench\t{what}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\n", a.0, a.1, b.0, b.1));
                    }
                }
                // The block of the shape that is picked (call 205): its front edge at its foot, and the item's own at the floor, on
                // screen, so that a demonstration measures one against the other in the picture.
                // `edge<TAB>block<TAB>x1<TAB>y1<TAB>x2<TAB>y2` and `edge<TAB>item<TAB>...`.
                // What the picked block's rows read, as the table writes them: `shape<TAB>block<TAB>N<TAB>made of<TAB>form<TAB>there when<TAB>the six figures`.
                if let Some((s, n)) = snap.bench_shape.as_ref().zip(doc.shape_block).filter(|(s, n)| (1..=s.pieces.len()).contains(n)) {
                    let piece = &s.pieces[n - 1];
                    out.push_str(&format!("shape\tblock\t{n}\t{}\t{}\t{}\t{}\n", piece.made.word(), if piece.round { "round" } else { "box" }, piece.when.clone().unwrap_or_else(|| "always".to_string()), piece.figures().join("\t")));
                }
                if let Some(Some(block)) = snap.bench_shape.as_ref().zip(doc.shape_block).and_then(|(s, n)| s.worked.get(n.wrapping_sub(1))) {
                    for (what, x, y, z) in [("block", (block.lo[0], block.hi[0]), block.lo[1], block.lo[2]), ("item", (-(w as f64) / 2.0, w as f64 / 2.0), -(d as f64) / 2.0, 0.0)] {
                        let end = |x: f64| at(Vec3::new(x as f32, z as f32, -y as f32));
                        if let (Some(a), Some(b)) = (end(x.0), end(x.1)) {
                            out.push_str(&format!("edge\t{what}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\n", a.0, a.1, b.0, b.1));
                        }
                    }
                }
            }
            (None, _) if !doc.plan => {
                if let Some(p) = snap.extent.and_then(|(lo, _, _)| at(to_view(doc.origin, [lo[0], lo[1], 0.0]))) {
                    out.push_str(&format!("house\t{:.1}\t{:.1}\n", p.0, p.1));
                }
                // The middle of what is drawn: what a swing begun off the model turns about (call 197, point 14).
                if let Some(p) = snap.extent.and_then(|(lo, hi, top)| at(to_view(doc.origin, [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, top / 2.0]))) {
                    out.push_str(&format!("middle\t{:.1}\t{:.1}\n", p.0, p.1));
                }
            }
            _ => {}
        }
    }
    // Each piece of a run that has no size, where it is drawn as its dashed centerline (call 209 point 6): `unsized<TAB>path<TAB>x1<TAB>y1<TAB>x2<TAB>y2`,
    // its two ends on screen. A demonstration reads the colour of the picture along it.
    if let (None, Ok((camera, eye, _))) = (&doc.bench, camera.single()) {
        for p in &snap.drawn {
            if let (Shape::Run { a, b, round: None, rect: None }, true) = (&p.shape, p.kind.is_segment()) {
                if let (Ok(q), Ok(r)) = (camera.world_to_viewport(eye, to_view(doc.origin, *a)), camera.world_to_viewport(eye, to_view(doc.origin, *b))) {
                    out.push_str(&format!("unsized\t{}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\n", p.path, q.x + model.0.x as f32, q.y + model.0.y as f32, r.x + model.0.x as f32, r.y + model.0.y as f32));
                }
            }
        }
    }
    // Where a place in the middle of each wall's gable is on screen, in 3D (call 192): halfway up between the wall's top and the roof's
    // surface over the middle of the wall (the model's top line), and the apex of that line:
    // `gable<TAB>the wall<TAB>x<TAB>y<TAB>apex x<TAB>apex y<TAB>feet along<TAB>feet up<TAB>wall x<TAB>wall y` (the last two: the wall's own face, halfway up). A demonstration reads the picture there.
    if let (false, Ok((camera, eye, _))) = (doc.plan, camera.single()) {
        for p in &snap.drawn {
            let (Shape::Wall { a, b, base, height, .. }, Some(g)) = (&p.shape, snap.gables.get(&p.path)) else { continue };
            if g.top.len() < 2 {
                continue;
            }
            let long = (b[0] - a[0]).hypot(b[1] - a[1]).max(1e-9);
            let (s0, s1) = (g.top[0].0, g.top[g.top.len() - 1].0);
            let mid = (s0 + s1) / 2.0;
            let roof = g.top.windows(2).find(|w| mid >= w[0].0 && mid <= w[1].0).map(|w| w[0].1 + (w[1].1 - w[0].1) * (mid - w[0].0) / (w[1].0 - w[0].0).max(1e-9)).unwrap_or(g.top[0].1);
            let up = base + height + (roof - (base + height)) / 2.0;
            let apex = g.top.iter().copied().fold(g.top[0], |m, q| if q.1 > m.1 { q } else { m });
            let place = |s: f64, z: f64| camera.world_to_viewport(eye, to_view(doc.origin, [a[0] + (b[0] - a[0]) * s / long, a[1] + (b[1] - a[1]) * s / long, z])).ok();
            if let (Some(m), Some(t), Some(w)) = (place(mid, up), place(apex.0, apex.1), place(mid, base + height / 2.0)) {
                out.push_str(&format!("gable\t{}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\t{:.2}\t{:.2}\t{:.0}\t{:.0}\n", p.path, m.x + model.0.x as f32, m.y + model.0.y as f32, t.x + model.0.x as f32, t.y + model.0.y as f32, mid, up, w.x + model.0.x as f32, w.y + model.0.y as f32));
            }
        }
    }
    // What the selected wall's panel says of its gable (call 192): `gable_panel<TAB>the wall<TAB>40.00 SF<TAB>Assumed`.
    if let Some(el) = &snap.selected {
        if let Some((value, unit, status)) = crate::panels::gable_stat(&snap, el) {
            out.push_str(&format!("gable_panel\t{}\t{} {}\t{:?}\n", el.path, value, unit, status));
        }
    }
    // The north arrow: how far its needle is turned (degrees clockwise from up) and how flat the disc lies (call 218, point 7).
    {
        let a = crate::nav::north_arrow(orbit.now.1, orbit.now.2);
        out.push_str(&format!("arrow\t{:.1}\t{:.2}\n", a.needle.to_degrees().rem_euclid(360.0), a.flat));
    }
    // The selected thing as it lies on the screen: its name, the middle of its box, and its width and height in dots
    // (call 218, point 10): a demonstration follows a receptacle or a door down a dive by it.
    if let (Some(path), Ok((camera, eye, _))) = (&doc.selected, camera.single()) {
        let shown: Vec<Vec2> = crate::scene::corners_of(&doc, &snap, path).iter().filter_map(|c| camera.world_to_viewport(eye, *c).ok()).collect();
        if let Some(first) = shown.first() {
            let (lo, hi) = shown.iter().fold((*first, *first), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            let name = snap.selected.as_ref().map(|e| e.name.clone()).unwrap_or_else(|| path.clone());
            out.push_str(&format!("selected\t{}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\n", name, (lo.x + hi.x) / 2.0 + model.0.x as f32, (lo.y + hi.y) / 2.0 + model.0.y as f32, hi.x - lo.x, hi.y - lo.y));
        }
    }
    out.push_str(&spoken);
    if out != *last {
        // Written whole, then put in place, so a script never reads half of it.
        let part = format!("{file}.part");
        if std::fs::write(&part, &out).is_ok() && std::fs::rename(&part, file).is_ok() {
            *last = out;
        }
    }
}
