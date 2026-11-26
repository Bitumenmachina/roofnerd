//! Any screen, and work never lost (DECISIONS.md call 222, part A).
//!
//! The two windows are put on the screen they are given, whole; the main one is followed
//! as it is drawn in or out; what a person set is kept in a small text file of the
//! program's own on that computer (never in the job); the lettering can be made larger
//! in steps; and a job with changes not saved has a recovery copy beside it, written
//! exactly as Save would write it, so that a stopped program loses nothing.
//!
//! What is here that needs no screen is plain arithmetic and plain text, and is tested
//! without one: where two windows go on a screen of a given size (`place`, `fit`), which
//! panels fold at which width (`folded`), how the tools wrap (`bar_plan`), the settings
//! read and written (`Settings`), and the recovery copy (`Keeper`, `find_recovery`).
//! The systems that carry them to the windows are below.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::window::{Monitor, PrimaryMonitor, PrimaryWindow};

use crate::model::{Doc, Tone};
use crate::theme;

// ---------------------------------------------------------------- where windows go

/// A window's place and size, in dots as a person reads them on the screen (logical): the
/// left and top of the window, then its width and height.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    /// Every corner is on a screen of this size.
    #[cfg(test)]
    pub fn on(&self, screen: (u32, u32)) -> bool {
        self.x >= 0 && self.y >= 0 && self.x as i64 + self.w as i64 <= screen.0 as i64 && self.y as i64 + self.h as i64 <= screen.1 as i64
    }
    /// The two lie over one another.
    #[cfg(test)]
    pub fn over(&self, other: &Rect) -> bool {
        (self.x as i64) < other.x as i64 + other.w as i64 && (other.x as i64) < self.x as i64 + self.w as i64 && (self.y as i64) < other.y as i64 + other.h as i64 && (other.y as i64) < self.y as i64 + self.h as i64
    }
}

/// The model window as it opens where there is room, and the estimate window beside it.
pub const MODEL_FULL: (u32, u32) = (1600, 960);
pub const SHEET_FULL: (u32, u32) = (880, 960);
/// The smallest each window is let be drawn to: at it every control can still be reached
/// (the builder found them by trying; STATUS.md, "Any screen, and work never lost").
pub const MODEL_LEAST: (u32, u32) = (720, 520);
pub const SHEET_LEAST: (u32, u32) = (880, 420);
/// What the desktop keeps for itself above and below a window: its title bar and a bar
/// along the screen's foot. The program cannot ask the desktop, so it leaves this much.
pub const DESKTOP_H: u32 = 72;
/// The second window standing over the first begins this far down, clear of the first's top bar.
const OVER_AT: u32 = 48;

/// Where the two windows go on a screen of this size: the model window and the estimate
/// window. Where the model can keep its full size beside the estimate window they stand
/// side by side, as they always have. Where it cannot, the model takes the screen and the
/// estimate window stands over its right side at its own size, to be moved. Neither opens
/// with any part off the screen, and the screen is never asked for more than it has.
pub fn place(screen: (u32, u32)) -> (Rect, Rect) {
    let (sw, sh) = screen;
    let room_h = if sh > DESKTOP_H + 400 { sh - DESKTOP_H } else { sh };
    let sheet_size = (SHEET_FULL.0.min(sw), SHEET_FULL.1.min(room_h));
    if sw >= MODEL_FULL.0 + theme::GAP as u32 + sheet_size.0 {
        let model = Rect { x: 0, y: 0, w: MODEL_FULL.0, h: MODEL_FULL.1.min(room_h) };
        let sheet = Rect { x: (MODEL_FULL.0 + theme::GAP as u32) as i32, y: 0, w: sheet_size.0, h: sheet_size.1 };
        (model, sheet)
    } else {
        let model = Rect { x: 0, y: 0, w: sw, h: room_h };
        let down = OVER_AT.min(room_h.saturating_sub(sheet_size.1));
        let sheet = Rect { x: (sw - sheet_size.0) as i32, y: down as i32, w: sheet_size.0, h: sheet_size.1 };
        (model, sheet)
    }
}

/// A place and size that was kept, brought onto the screen it is on now: no smaller than
/// the least a window is let be, no larger than the screen, and moved in until every
/// corner is on it.
pub fn fit(saved: Rect, screen: (u32, u32), least: (u32, u32)) -> Rect {
    let w = saved.w.max(least.0).min(screen.0.max(1));
    let h = saved.h.max(least.1).min(screen.1.max(1));
    let x = saved.x.clamp(0, (screen.0.saturating_sub(w)) as i32);
    let y = saved.y.clamp(0, (screen.1.saturating_sub(h)) as i32);
    Rect { x, y, w, h }
}

// ---------------------------------------------------------------- panels that fold

/// A window narrower than this (in the units its panels are drawn in) puts the Details
/// panel away by itself; it comes back as the window widens.
pub const DETAILS_FOLDS_BELOW: f32 = 1500.0;
/// And narrower than this, the Job panel as well.
pub const JOB_FOLDS_BELOW: f32 = 1100.0;
/// Narrower than this the top bar drops the keys beside Save, Undo and Redo and cuts the
/// job's name short, so that the price and "Not saved" are never the part cut off.
pub const TIGHT_BELOW: f32 = 1300.0;
/// Narrower than this the price's own caption is left off too.
pub const BARE_BELOW: f32 = 900.0;

/// Which of the Job and Details panels are folded away at this width: `[job, details]`.
pub fn folded(width: f32) -> [bool; 2] {
    [width < JOB_FOLDS_BELOW, width < DETAILS_FOLDS_BELOW]
}

/// The tools' bar keeps the old squeeze of its buttons when it is this wide or wider;
/// narrower, the tools go onto as many lines as they need.
pub const BAR_SQUEEZE_AT: f32 = 960.0;
/// A tool's button is this wide, and the buttons stand this far apart.
const TOOL_W: f32 = 52.0;
const TOOL_GAP: f32 = 2.0;
/// Space between one group of tools and the next on a line.
pub const GROUP_GAP: f32 = 14.0;
/// The bar's side padding, and what each line of tools takes down: its heading and the buttons.
const BAR_PAD: f32 = 12.0;
pub const LINE_H: f32 = 58.0;
/// The line of fields at the head of the bar.
const STRIP_H: f32 = 46.0;
/// How many tools each group holds: the building's, the mechanical, electrical and plumbing, and edit.
pub const GROUPS: [usize; 3] = [8, 8, 2];

/// The tools laid on lines for a bar this wide: each line holds pieces of groups, as
/// `(group, first, last + 1)`. A group that fits on the line it comes to stays whole; one
/// that does not goes on to the next, and one wider than a line is cut where the line ends.
pub fn bar_plan(width: f32) -> Vec<Vec<(usize, usize, usize)>> {
    let room = (width - BAR_PAD).max(TOOL_W);
    let wide = |n: usize| n as f32 * (TOOL_W + TOOL_GAP) - TOOL_GAP;
    let most = (((room + TOOL_GAP) / (TOOL_W + TOOL_GAP)).floor() as usize).max(1);
    let mut lines: Vec<Vec<(usize, usize, usize)>> = vec![Vec::new()];
    let mut used = 0.0_f32;
    for (g, count) in GROUPS.iter().copied().enumerate() {
        let mut from = 0;
        while from < count {
            let left = count - from;
            let gap = if used > 0.0 { GROUP_GAP } else { 0.0 };
            if used + gap + wide(left) <= room {
                lines.last_mut().expect("a line").push((g, from, count));
                used += gap + wide(left);
                from = count;
            } else if used > 0.0 {
                lines.push(Vec::new());
                used = 0.0;
            } else {
                let take = left.min(most);
                lines.last_mut().expect("a line").push((g, from, from + take));
                used = wide(take);
                from += take;
                if from < count {
                    lines.push(Vec::new());
                    used = 0.0;
                }
            }
        }
    }
    lines
}

/// How tall the tools' bar is for a bar this wide: as it always was, or taller by a line
/// for each line the tools go on.
pub fn bar_height(width: f32) -> f32 {
    if width >= BAR_SQUEEZE_AT {
        return crate::scene::BUILD_H;
    }
    STRIP_H + bar_plan(width).len() as f32 * LINE_H
}

// ---------------------------------------------------------------- lettering

/// The steps of lettering: what every word and control is scaled by. The first is as the
/// program has always been; each step after is a seventh to a sixth larger than the last.
pub const LETTERING: [f32; 4] = [1.0, 1.15, 1.3, 1.5];

/// The scale a step stands for (a step past the last is the last).
pub fn lettering_scale(step: u8) -> f32 {
    LETTERING[(step as usize).min(LETTERING.len() - 1)]
}

// ---------------------------------------------------------------- the keys

/// Ctrl is held, or Command (the key beside the space bar on a Mac keyboard) is: either does what Ctrl does.
pub fn command_held(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight, KeyCode::SuperLeft, KeyCode::SuperRight])
}

/// The word on the key that does what Ctrl does, by the operating system the program runs on (`std::env::consts::OS`): Cmd on a Mac, Ctrl on
/// any other (call 224, point 4).
pub fn command_word(os: &str) -> &'static str {
    if os == "macos" { "Cmd" } else { "Ctrl" }
}

/// A key as it is written on a button: the command key's word, a plus and the letter.
pub fn key_said(os: &str, letter: char) -> String {
    format!("{}+{}", command_word(os), letter.to_ascii_uppercase())
}

/// The key as it reads on this computer.
pub fn command_key(letter: char) -> String {
    key_said(std::env::consts::OS, letter)
}

/// Whether a window may be shown yet: the windows are not seen until they have been put where they go (call 224, point 3). The old ways
/// (no placing is done) show them at once.
pub fn may_be_seen(old_ways: bool, placed: bool) -> bool {
    old_ways || placed
}

// ---------------------------------------------------------------- settings

/// What a person set, kept on this computer: where each window stood and how big, which
/// panels were put away, the lettering. Plain text, a line to a setting. It is the
/// program's, not the job's: a job opened elsewhere is the same job.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Settings {
    pub model: Option<Rect>,
    pub sheet: Option<Rect>,
    pub away: [bool; 3],
    pub lettering: u8,
    /// What the second window shows and how each plan in it is looked at: the program's, never the job's (call 222, point 12).
    pub second: crate::second::Kept,
}

const PANEL_WORDS: [&str; 3] = ["Job", "Details", "Tools"];

impl Settings {
    /// The file as it is written.
    pub fn text(&self) -> String {
        let mut out = String::from("# BuildOS, as set on this computer. The program keeps this file; it may be changed by hand.\n");
        let put = |out: &mut String, name: &str, r: &Option<Rect>| {
            if let Some(r) = r {
                out.push_str(&format!("{name} = {} {} {} {}\n", r.x, r.y, r.w, r.h));
            }
        };
        put(&mut out, "model window", &self.model);
        put(&mut out, "estimate window", &self.sheet);
        let away: Vec<&str> = PANEL_WORDS.iter().copied().zip(self.away).filter(|(_, a)| *a).map(|(n, _)| n).collect();
        out.push_str(&format!("panels put away = {}\n", if away.is_empty() { "none".to_string() } else { away.join(" ") }));
        out.push_str(&format!("lettering = {}\n", self.lettering));
        out.push_str(&self.second.text());
        out
    }

    /// A file read. A line it does not understand is passed over; nothing in it can stop the program.
    pub fn parse(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            let line = line.trim();
            let Some((name, value)) = line.split_once('=') else { continue };
            let (name, value) = (name.trim(), value.trim());
            let rect = || {
                let n: Vec<i64> = value.split_whitespace().filter_map(|v| v.parse().ok()).collect();
                (n.len() == 4 && n[2] > 0 && n[3] > 0 && n.iter().all(|v| v.abs() < 100_000)).then(|| Rect { x: n[0] as i32, y: n[1] as i32, w: n[2] as u32, h: n[3] as u32 })
            };
            match name {
                "model window" => s.model = rect(),
                "estimate window" => s.sheet = rect(),
                "panels put away" => {
                    for (i, word) in PANEL_WORDS.iter().enumerate() {
                        s.away[i] = value.split_whitespace().any(|v| v == *word);
                    }
                }
                "lettering" => s.lettering = value.parse::<u8>().unwrap_or(0).min(LETTERING.len() as u8 - 1),
                _ => {
                    s.second.read(name, value);
                }
            }
        }
        s
    }
}

/// Where the settings file is, given the program's surroundings (`env` answers for a name):
/// the file a demonstration names; none at all when the demonstration's map is on and no
/// settings are named; else the operating system's own place for a program's settings.
pub fn settings_path_from(env: &dyn Fn(&str) -> Option<String>) -> Option<PathBuf> {
    if let Some(named) = env("PLUMBLINE_SETTINGS").filter(|n| !n.is_empty()) {
        return Some(PathBuf::from(named));
    }
    if env("PLUMBLINE_MAP").is_some() {
        return None;
    }
    let home = env("HOME").filter(|h| !h.is_empty()).map(PathBuf::from);
    let place = if cfg!(target_os = "windows") {
        env("APPDATA").filter(|h| !h.is_empty()).map(|a| PathBuf::from(a).join("BuildOS"))
    } else if cfg!(target_os = "macos") {
        home.map(|h| h.join("Library").join("Application Support").join("BuildOS"))
    } else {
        env("XDG_CONFIG_HOME").filter(|h| !h.is_empty()).map(|c| PathBuf::from(c).join("buildos")).or_else(|| home.map(|h| h.join(".config").join("buildos")))
    };
    place.map(|p| p.join("settings.txt"))
}

/// The program in its surroundings as they are.
pub fn settings_path() -> Option<PathBuf> {
    settings_path_from(&|name| std::env::var(name).ok())
}

/// Whether the program is run by a demonstration that names no settings: it opens the two
/// windows as it always did, reads and writes no settings, and keeps no recovery copy.
pub fn old_ways() -> bool {
    old_ways_from(&|name| std::env::var(name).ok())
}

pub fn old_ways_from(env: &dyn Fn(&str) -> Option<String>) -> bool {
    env("PLUMBLINE_MAP").is_some() && env("PLUMBLINE_SETTINGS").filter(|n| !n.is_empty()).is_none()
}

// ---------------------------------------------------------------- the recovery copy

/// The recovery copy of a job: beside it, its own file's name with ".recovery" after it,
/// so that it is never taken for a job and never lies over one.
pub fn recovery_path(job: &Path) -> PathBuf {
    let mut name = job.file_name().map(|n| n.to_os_string()).unwrap_or_else(|| "job.json".into());
    name.push(".recovery");
    job.with_file_name(name)
}

/// The recovery copy written: whole, to a file of its own and then put in place, so that
/// a program stopped in the middle of writing leaves the last copy, not half of a new one.
pub fn write_recovery(job: &Path, text: &str) -> std::io::Result<()> {
    let to = recovery_path(job);
    let mut part = to.clone().into_os_string();
    part.push(".part");
    let part = PathBuf::from(part);
    std::fs::write(&part, text)?;
    std::fs::rename(&part, &to)
}

/// The recovery copy taken away (there is none to take away: not an error).
pub fn remove_recovery(job: &Path) {
    let _ = std::fs::remove_file(recovery_path(job));
}

/// Work found beside a job that was never saved.
#[derive(Clone, PartialEq, Debug)]
pub struct Recovered {
    /// The job as Save would have written it.
    pub text: String,
    /// How long ago it was written, in seconds.
    pub age: u64,
}

/// The recovery copy beside this job, if there is one and it is newer than the job and
/// differs from it. A job not yet saved has no file, so a copy beside it is newer by
/// being the only one. A copy older than the job, or the same as it, is not offered; it
/// is not removed either (Save and a clean close remove it).
pub fn find_recovery(job: &Path) -> Option<Recovered> {
    let there = recovery_path(job);
    let text = std::fs::read_to_string(&there).ok()?;
    let wrote = std::fs::metadata(&there).and_then(|m| m.modified()).ok()?;
    if let Ok(saved) = std::fs::metadata(job) {
        let saved_at = saved.modified().ok()?;
        if wrote <= saved_at || std::fs::read_to_string(job).is_ok_and(|now| now == text) {
            return None;
        }
    }
    Some(Recovered { text, age: wrote.elapsed().map(|d| d.as_secs()).unwrap_or(0) })
}

/// How long ago, as a builder says it.
pub fn age_said(secs: u64) -> String {
    let plural = |n: u64, unit: &str| if n == 1 { format!("1 {unit} ago") } else { format!("{n} {unit}s ago") };
    match secs {
        0..=59 => "less than a minute ago".to_string(),
        60..=3599 => plural(secs / 60, "minute"),
        3600..=86_399 => plural(secs / 3600, "hour"),
        _ => plural(secs / 86_400, "day"),
    }
}

/// When a recovery copy is written. While a job has changes not saved, once a minute
/// after the first of them and once a minute after each copy, and at once after every
/// tenth change; and not at all when nothing has changed since the last copy.
#[derive(Default, Debug)]
pub struct Keeper {
    /// Changes counted when the job was last clean or a copy was last written.
    mark: u64,
    /// When the minute began: the job's becoming unsaved, or the last copy.
    clock: Option<f64>,
}

/// A minute, and how many changes are the most that wait for one.
pub const EVERY: f64 = 60.0;
pub const EVERY_CHANGES: u64 = 10;

impl Keeper {
    /// Asked once a frame with the time, the changes counted so far and whether any are
    /// unsaved. True: write a copy now.
    pub fn due(&mut self, now: f64, edits: u64, edited: bool) -> bool {
        // The count goes back to the start when another job is opened in the window: it is counted from there.
        if !edited || edits < self.mark {
            self.mark = edits;
            self.clock = None;
            if !edited {
                return false;
            }
        }
        let began = *self.clock.get_or_insert(now);
        let changed = edits.saturating_sub(self.mark);
        let write = changed >= EVERY_CHANGES || (changed > 0 && now - began >= EVERY);
        if write {
            self.mark = edits;
            self.clock = Some(now);
        }
        write
    }
}

// ---------------------------------------------------------------- what the windows hold

/// How the main window is at the moment, as the panels are laid out for it. Part of the
/// open job's window state (`Doc`), so that a click that changes it is one more change
/// of what is drawn, like any other.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fit {
    /// The window's width in the units the panels are drawn in (its dots over the lettering's scale).
    pub width: f32,
    /// The Job and Details panels folded away by the width (not by a switch).
    pub folded: [bool; 2],
    /// A folded panel the person asked for with its switch: it stands over the model until
    /// the window is wide enough to hold it, or the switch is put off.
    pub opened: [bool; 2],
    /// The step of lettering the person chose.
    pub lettering: u8,
}

impl Fit {
    pub fn scale(&self) -> f32 {
        lettering_scale(self.lettering)
    }
}

/// How the main window is laid out at the moment: which panels are in it, how far in from
/// each side the cards over the model begin, whether the top bar is tight, and how many
/// lines the tools are on (0: as the bar has always been, its buttons squeezed to fit).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub shown: [bool; 3],
    pub left_in: f32,
    pub right_in: f32,
    /// How wide the tools' bar is.
    pub bar: f32,
    pub tight: bool,
    pub lines: usize,
}

/// What a `Doc` holds of the work found beside it: set when the job is opened, answered
/// by one of two controls.
pub type Offer = Option<Recovered>;

// ---------------------------------------------------------------- the systems

/// What the program keeps between runs and how it keeps it.
#[derive(Resource)]
pub struct Kept {
    /// Where the settings file is; none, and nothing is read or written.
    pub path: Option<PathBuf>,
    /// What is on disk now, or was last written.
    pub written: Settings,
    /// Seconds since the last write: a write follows a change by at least this much.
    since: f64,
    /// The old ways: windows as they always opened, no recovery copy.
    pub old_ways: bool,
    /// The windows have been put where they go.
    placed: bool,
    /// And have been shown, a frame after (call 224, point 3).
    revealed: bool,
    /// Frames spent waiting for the screen to be known.
    waited: u32,
    keeper: Keeper,
    /// The settings read at start, to be carried to the windows once the screen is known.
    pub read: Settings,
    /// The screen as the map writes it, in dots as a person reads them.
    pub screen: Option<(u32, u32)>,
}

impl Kept {
    pub fn start() -> Kept {
        let old_ways = old_ways();
        let path = settings_path();
        let read = path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).map(|t| Settings::parse(&t)).unwrap_or_default();
        Kept { path, written: read.clone(), since: 0.0, old_ways, placed: false, revealed: false, waited: 0, keeper: Keeper::default(), read, screen: None }
    }
}

/// The windows as they are made: where there are kept settings, at them; else as the
/// program has always opened them (they are brought onto the screen once it is known).
pub fn opening(kept: &Kept) -> ((u32, u32), (u32, u32)) {
    let model = kept.read.model.map(|r| (r.w, r.h)).unwrap_or(MODEL_FULL);
    let sheet = kept.read.sheet.map(|r| (r.w, r.h)).unwrap_or(SHEET_FULL);
    (model, sheet)
}

/// Once the screen is known, put each window where it goes: where the person left it, brought
/// onto this screen, or where `place` puts it. Runs once; the old ways do not run it.
#[allow(clippy::too_many_arguments)]
pub fn put_windows(mut kept: ResMut<Kept>, mut doc: ResMut<Doc>, monitors: Query<(&Monitor, Has<PrimaryMonitor>)>, mut windows: Query<(&mut Window, Has<PrimaryWindow>)>, mut scale: ResMut<UiScale>) {
    if kept.old_ways {
        return;
    }
    // A window is shown a frame after it has been put where it goes, and not before: it was seen first at the size it opened at and moved after
    // (call 224, point 3).
    if kept.placed {
        if !kept.revealed {
            kept.revealed = true;
            for (mut window, _) in &mut windows {
                window.visible = true;
            }
        }
        return;
    }
    // The letters are as they were left, from the first frame; and what the second window showed.
    if kept.waited == 0 {
        doc.fit.lettering = kept.read.lettering;
        doc.show.away = kept.read.away;
        scale.0 = lettering_scale(doc.fit.lettering);
        let read = kept.read.second.clone();
        doc.second.load(&read);
    }
    kept.waited += 1;
    let monitor = monitors.iter().find(|(_, primary)| *primary).or_else(|| monitors.iter().next()).map(|(m, _)| m);
    let Some(monitor) = monitor else {
        // No screen is reported (some desktops do not): the windows stay as they opened.
        if kept.waited > 240 {
            kept.placed = true;
        }
        return;
    };
    let k = monitor.scale_factor.max(0.1) as f32;
    let screen = ((monitor.physical_width as f32 / k) as u32, (monitor.physical_height as f32 / k) as u32);
    kept.screen = Some(screen);
    let (model, sheet) = place(screen);
    for (mut window, primary) in &mut windows {
        let (saved, least, fresh) = if primary { (kept.read.model, MODEL_LEAST, model) } else { (kept.read.sheet, SHEET_LEAST, sheet) };
        let to = saved.map(|s| fit(s, screen, least)).unwrap_or(fresh);
        window.resolution.set(to.w as f32, to.h as f32);
        window.position.set(IVec2::new((to.x as f32 * k) as i32, (to.y as f32 * k) as i32));
    }
    kept.placed = true;
}

/// Where each window really stands, asked of the desktop each frame: a window moved by a hand (or by the desktop) is not always
/// reported, and what is kept and what the map says must be where it is. The window's own record is put right without being
/// taken as a request to move it.
pub fn track(_main: bevy::ecs::system::NonSendMarker, kept: Res<Kept>, mut windows: Query<(Entity, &mut Window)>) {
    if kept.old_ways {
        return;
    }
    for (entity, mut window) in &mut windows {
        // A window manager's frame (its edges) is outside the window's own size: a window made as wide as the screen stands a frame's width past it.
        // Where the frame takes a window past the screen's right edge by no more than the frame's own width, the window is made that much narrower.
        if let Some(screen) = kept.screen.filter(|_| kept.revealed) {
            let over = bevy::winit::WINIT_WINDOWS.with_borrow(|w| {
                w.get_window(entity).and_then(|x| {
                    let (pos, outer, inner) = (x.outer_position().ok()?, x.outer_size(), x.inner_size());
                    let frame = outer.width.saturating_sub(inner.width) as i32;
                    let right = pos.x + outer.width as i32;
                    let edge = (screen.0 as f32 * window.scale_factor()) as i32;
                    (frame > 0 && right > edge && right - edge <= frame).then_some(((right - edge) as f32 / window.scale_factor()).ceil())
                })
            });
            if let Some(over) = over {
                if window.width() - over >= SHEET_LEAST.0 as f32 {
                    let (w, h) = (window.width() - over, window.height());
                    window.resolution.set(w, h);
                } else if let WindowPosition::At(at) = window.position {
                    // Not narrower than a window may be: moved in by what it stands past the screen.
                    window.position = WindowPosition::At(IVec2::new(at.x - (over * window.scale_factor()) as i32, at.y));
                    continue;
                }
            }
        }
        let at = bevy::winit::WINIT_WINDOWS.with_borrow(|w| w.get_window(entity).and_then(|x| x.outer_position().ok()).map(|p| IVec2::new(p.x, p.y)));
        if let Some(at) = at {
            if window.position != WindowPosition::At(at) {
                window.bypass_change_detection().position = WindowPosition::At(at);
            }
        }
    }
}

/// The least a window is let be drawn to: one drawn smaller is brought back up to it.
pub fn keep_least(mut windows: Query<(&mut Window, Has<PrimaryWindow>)>) {
    for (mut window, primary) in &mut windows {
        let least = if primary { MODEL_LEAST } else { SHEET_LEAST };
        let (w, h) = (window.width(), window.height());
        if (w as u32) < least.0 || (h as u32) < least.1 {
            window.resolution.set(w.max(least.0 as f32), h.max(least.1 as f32));
        }
    }
}

/// The lettering's scale carried to every word and control, and the main window's width, in
/// the units its panels are drawn in, carried to what folds. What changes from one to the
/// next is drawn again.
pub fn follow(mut doc: ResMut<Doc>, windows: Query<&Window, With<PrimaryWindow>>, mut scale: ResMut<UiScale>) {
    let want = doc.fit.scale();
    if scale.0 != want {
        scale.0 = want;
    }
    let Ok(window) = windows.single() else { return };
    let width = window.width() / want;
    let was = crate::panels::layout(&doc);
    let (fit, kept_open) = {
        let fit = &doc.fit;
        let now = folded(width);
        // A panel that is no longer folded needs no asking to stand: its asking is done.
        let opened = [fit.opened[0] && now[0], fit.opened[1] && now[1]];
        (Fit { width, folded: now, opened, lettering: fit.lettering }, opened != fit.opened || now != fit.folded || (width - fit.width).abs() > 0.5)
    };
    if kept_open {
        doc.bypass_change_detection().fit = fit;
        // Drawn again only when what is drawn is not as it was: a width that changes no
        // panel and no line of the tools is not a reason to draw it all.
        if crate::panels::layout(&doc) != was {
            doc.touch();
        }
    }
}

/// What is kept: each window's place and size, the panels put away and the lettering,
/// written when any of them has changed (and no oftener than once in a second).
pub fn keep(time: Res<Time<Real>>, mut kept: ResMut<Kept>, doc: Res<Doc>, windows: Query<(&Window, Has<PrimaryWindow>)>) {
    if kept.old_ways || !kept.placed {
        return;
    }
    let Some(path) = kept.path.clone() else { return };
    let now = Settings { model: None, sheet: None, away: doc.show.away, lettering: doc.fit.lettering, second: doc.second.kept() };
    let mut now = now;
    for (window, primary) in &windows {
        let k = window.scale_factor().max(0.1);
        let at = match window.position {
            WindowPosition::At(at) => at,
            _ => IVec2::ZERO,
        };
        let r = Rect { x: (at.x as f32 / k) as i32, y: (at.y as f32 / k) as i32, w: window.width().round() as u32, h: window.height().round() as u32 };
        if primary {
            now.model = Some(r);
        } else {
            now.sheet = Some(r);
        }
    }
    kept.since += time.delta_secs_f64();
    if now != kept.written && kept.since >= 1.0 {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if std::fs::write(&path, now.text()).is_ok() {
            kept.written = now;
            kept.since = 0.0;
        }
    }
}

/// The job as Save would write it, put beside the job as its recovery copy.
pub fn write_copy(doc: &mut Doc) -> std::io::Result<()> {
    let text = plumbline_world::file::save(&mut doc.job);
    write_recovery(&doc.path, &text)
}

/// A recovery copy written while the job has changes not saved, as Save would write it. While work found beside the job is
/// waiting to be taken or left it is not written over.
pub fn recover(time: Res<Time<Real>>, mut kept: ResMut<Kept>, mut doc: ResMut<Doc>) {
    if kept.old_ways {
        return;
    }
    let now = time.elapsed_secs_f64();
    let (edits, edited) = (doc.draw.edits, doc.edited);
    if kept.keeper.due(now, edits, edited && doc.recovered.is_none()) {
        if write_copy(doc.bypass_change_detection()).is_err() && !doc.message.starts_with("A recovery copy") {
            // Work that cannot be kept is the one thing to be told, once.
            doc.say(Tone::Warned, "A recovery copy could not be kept beside this job. Save as you go.");
            doc.touch();
        }
    }
}

/// The lettering one step larger or smaller; at the end of its steps it says so.
pub fn lettering_step(doc: &mut Doc, by: i8) {
    let now = doc.fit.lettering as i16;
    let next = (now + by as i16).clamp(0, LETTERING.len() as i16 - 1);
    if next == now {
        doc.say(Tone::Done, if by > 0 { "The lettering is as large as it goes." } else { "The lettering is as small as it goes." });
    } else {
        doc.fit.lettering = next as u8;
        doc.say(Tone::Done, format!("Lettering set one step {}.", if by > 0 { "larger" } else { "smaller" }));
    }
    doc.touch();
}

/// "Take the recovered work": the job becomes the work found beside it, not yet saved: Save keeps it.
pub fn take(doc: &mut Doc) {
    let Some(found) = doc.recovered.take() else { return };
    let path = doc.path.to_string_lossy().to_string();
    match crate::build_doc_with(path, None, Some(found.text.clone())) {
        Ok(mut next) => {
            next.revision = doc.revision + 1;
            next.geometry = doc.geometry + 1;
            next.shots = doc.shots;
            next.show.away = doc.show.away;
            next.fit = doc.fit.clone();
            next.second = std::mem::take(&mut doc.second);
            *doc = next;
        }
        Err(why) => {
            doc.recovered = Some(found);
            crate::tools::cant(doc, "take the recovered work", why);
        }
    }
    doc.touch();
}

/// "Leave it": the job stays as it was last saved. The recovered work stays beside it until the job is saved.
pub fn leave(doc: &mut Doc) {
    if doc.recovered.take().is_some() {
        doc.say(Tone::Done, "Left it. The job is as it was last saved; the recovered work stays beside it until the job is saved.");
        doc.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plumbline_world::file;
    use plumbline_world::value::Status;

    #[test]
    fn the_key_that_does_what_ctrl_does_reads_cmd_on_a_mac_and_ctrl_elsewhere() {
        assert_eq!(command_word("macos"), "Cmd");
        for os in ["linux", "windows", "freebsd", ""] {
            assert_eq!(command_word(os), "Ctrl", "{os}");
        }
        assert_eq!(key_said("macos", 's'), "Cmd+S");
        assert_eq!(key_said("linux", 'z'), "Ctrl+Z");
        assert_eq!(key_said("windows", 'Y'), "Ctrl+Y");
        // And on the machine the tests run on it is whatever that machine's word is.
        assert_eq!(command_key('s'), key_said(std::env::consts::OS, 's'));
    }

    #[test]
    fn a_window_is_not_seen_until_it_has_been_put_where_it_goes_unless_the_old_ways_are_on() {
        assert!(!may_be_seen(false, false), "unplaced: hidden");
        assert!(may_be_seen(false, true), "placed: shown");
        assert!(may_be_seen(true, false), "the old ways never place, so show at once");
        assert!(may_be_seen(true, true));
    }

    #[test]
    fn what_is_kept_of_the_second_window_is_read_back_the_same() {
        use crate::second::{Look, Mode};
        let mut s = Settings::default();
        s.second.mode = Mode::Two;
        s.second.looks = [Some(Look::Level("level/first-floor".into())), Some(Look::Sheet("drawing/made-up".into()))];
        s.second.away = [vec!["Plumbing".into()], vec!["Roof".into(), "Mechanical".into()]];
        s.second.views = [Some([12.5, 7.25, 31.4]), None];
        let back = Settings::parse(&s.text());
        assert_eq!(back, s);
        assert!(s.text().contains("second window = two plans"));
        assert!(s.text().contains("plan 1 view = 12.5 7.25 31.4"));
        // A file with nothing of it opens on the estimate, as it always did; a line not understood is passed over.
        let none = Settings::parse("lettering = 1\nsecond window = something else\nplan 1 shows = nonsense\n");
        assert_eq!(none.second.mode, Mode::Estimate);
        assert_eq!(none.second.looks, [None, None]);
    }

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("buildos-screen-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a folder");
        dir
    }

    // ------------------------------------------------ where the windows go

    #[test]
    fn on_a_large_screen_the_two_windows_stand_side_by_side_as_they_always_have() {
        let (model, sheet) = place((2560, 1440));
        assert_eq!(model, Rect { x: 0, y: 0, w: 1600, h: 960 });
        assert_eq!(sheet, Rect { x: 1610, y: 0, w: 880, h: 960 });
        assert!(!model.over(&sheet));
        assert!(model.on((2560, 1440)) && sheet.on((2560, 1440)));
    }

    #[test]
    fn on_a_smaller_screen_the_model_takes_it_and_the_estimate_stands_over_its_right_side() {
        for screen in [(1920, 1080), (1366, 768), (1280, 720), (1024, 600)] {
            let (model, sheet) = place(screen);
            assert!(model.on(screen), "model on {screen:?}: {model:?}");
            assert!(sheet.on(screen), "estimate on {screen:?}: {sheet:?}");
            assert_eq!(model.w, screen.0, "the model takes the screen's width on {screen:?}");
            assert_eq!(sheet.x as u32 + sheet.w, screen.0, "the estimate is at the right side on {screen:?}");
        }
    }

    #[test]
    fn no_screen_from_the_smallest_to_the_largest_has_a_window_off_it() {
        for w in (640..=3840).step_by(64) {
            for h in (480..=2160).step_by(60) {
                let (model, sheet) = place((w, h));
                assert!(model.on((w, h)) && sheet.on((w, h)), "{w}x{h}: {model:?} {sheet:?}");
            }
        }
    }

    #[test]
    fn a_place_that_was_kept_is_brought_onto_the_screen_it_is_on_now() {
        let screen = (1366, 768);
        // Off the right and the foot of a smaller screen than the one it was kept on.
        let was = Rect { x: 3000, y: 1500, w: 1600, h: 960 };
        let now = fit(was, screen, MODEL_LEAST);
        assert!(now.on(screen), "{now:?}");
        assert_eq!((now.w, now.h), (1366, 768));
        // Too small to use: brought up to the least.
        let tiny = fit(Rect { x: 10, y: 10, w: 100, h: 100 }, screen, MODEL_LEAST);
        assert_eq!((tiny.w, tiny.h), MODEL_LEAST);
        // A place that is fine stays where it is.
        let fine = Rect { x: 40, y: 30, w: 900, h: 600 };
        assert_eq!(fit(fine, screen, MODEL_LEAST), fine);
        // Off the left and the top: moved in.
        assert_eq!(fit(Rect { x: -50, y: -9, w: 900, h: 600 }, screen, MODEL_LEAST), Rect { x: 0, y: 0, w: 900, h: 600 });
    }

    // ------------------------------------------------ panels that fold

    #[test]
    fn the_details_panel_folds_first_and_the_job_panel_after_it_and_both_come_back_as_the_window_widens() {
        assert_eq!(folded(1600.0), [false, false]);
        assert_eq!(folded(DETAILS_FOLDS_BELOW), [false, false]);
        assert_eq!(folded(DETAILS_FOLDS_BELOW - 1.0), [false, true]);
        assert_eq!(folded(JOB_FOLDS_BELOW), [false, true]);
        assert_eq!(folded(JOB_FOLDS_BELOW - 1.0), [true, true]);
        assert_eq!(folded(640.0), [true, true]);
        // Back as it widens, in the other order.
        let widths = [640.0, 1000.0, 1200.0, 1499.0, 1500.0, 1700.0];
        let jobs: Vec<bool> = widths.iter().map(|w| folded(*w)[0]).collect();
        let details: Vec<bool> = widths.iter().map(|w| folded(*w)[1]).collect();
        assert_eq!(jobs, [true, true, false, false, false, false]);
        assert_eq!(details, [true, true, true, true, false, false]);
    }

    #[test]
    fn the_tools_are_every_one_on_the_bar_at_every_width_and_none_twice() {
        for width in (420..=1100).step_by(20) {
            let plan = bar_plan(width as f32);
            let mut seen = [0usize; 3];
            for line in &plan {
                for (g, from, to) in line {
                    assert!(from < to && *to <= GROUPS[*g], "{width}: {g} {from}..{to}");
                    seen[*g] += to - from;
                }
            }
            assert_eq!(seen, GROUPS, "every tool once at {width}");
            // And no line is wider than the bar holds.
            for line in &plan {
                let buttons: usize = line.iter().map(|(_, from, to)| to - from).sum();
                let wide = buttons as f32 * (TOOL_W + TOOL_GAP) - TOOL_GAP + GROUP_GAP * (line.len() as f32 - 1.0);
                assert!(wide <= (width as f32 - BAR_PAD).max(TOOL_W) + 0.01, "{width}: a line {wide} wide");
            }
        }
    }

    #[test]
    fn the_groups_are_the_tools_there_are() {
        assert_eq!(GROUPS[0], crate::tools::Tool::BUILDING.len());
        assert_eq!(GROUPS[1], crate::tools::Tool::MEP.len());
        assert_eq!(GROUPS[2], 2, "Move and Delete");
    }

    #[test]
    fn the_bar_is_taller_by_a_line_for_each_line_the_tools_go_on_and_as_it_was_where_it_is_wide() {
        assert_eq!(bar_height(964.0), crate::scene::BUILD_H);
        assert_eq!(bar_height(BAR_SQUEEZE_AT), crate::scene::BUILD_H);
        let mut last = 0.0;
        for width in [900.0_f32, 700.0, 500.0, 420.0] {
            let h = bar_height(width);
            assert!(h >= last, "{width}: {h} after {last}");
            last = h;
        }
        assert!(bar_height(960.0 - 1.0) >= crate::scene::BUILD_H);
        // Two lines at a thousand less forty: the building tools and the mechanical beside one another do not fit, and the third group rides on.
        assert_eq!(bar_plan(900.0).len(), 2);
        assert_eq!(bar_height(900.0), STRIP_H + 2.0 * LINE_H);
    }

    // ------------------------------------------------ the lettering

    #[test]
    fn each_step_of_lettering_is_between_a_tenth_and_a_quarter_larger_than_the_last() {
        assert_eq!(lettering_scale(0), 1.0);
        for step in 1..LETTERING.len() {
            let by = LETTERING[step] / LETTERING[step - 1];
            assert!(by > 1.1 && by < 1.25, "step {step} is {by}");
        }
        // A step past the last is the last.
        assert_eq!(lettering_scale(200), *LETTERING.last().unwrap());
        // The word on a button, in 12-dot letters, is 13.8 dots at the first step: a tenth to a quarter taller.
        let taller = 12.0 * lettering_scale(1) / 12.0;
        assert!((1.1..=1.25).contains(&taller), "{taller}");
    }

    // ------------------------------------------------ settings

    #[test]
    fn settings_are_written_as_plain_lines_and_read_back_as_they_were() {
        let s = Settings { model: Some(Rect { x: 0, y: 0, w: 1500, h: 900 }), sheet: Some(Rect { x: 1510, y: 20, w: 700, h: 800 }), away: [true, false, true], lettering: 2, second: Default::default() };
        let text = s.text();
        assert!(text.contains("model window = 0 0 1500 900\n"), "{text}");
        assert!(text.contains("estimate window = 1510 20 700 800\n"), "{text}");
        assert!(text.contains("panels put away = Job Tools\n"), "{text}");
        assert!(text.contains("lettering = 2\n"), "{text}");
        assert_eq!(Settings::parse(&text), s);
        // Nothing put away is said so.
        let none = Settings { away: [false; 3], ..s.clone() };
        assert!(none.text().contains("panels put away = none\n"));
        assert_eq!(Settings::parse(&none.text()), none);
    }

    #[test]
    fn a_settings_file_that_is_wrong_in_places_is_read_as_far_as_it_goes() {
        let read = Settings::parse("# a note\nmodel window = 1 2 three 4\nestimate window = 5 6 700 800\nlettering = 99\nsomething else = 1\nno equals here\npanels put away = Details Elsewhere\n");
        assert_eq!(read.model, None);
        assert_eq!(read.sheet, Some(Rect { x: 5, y: 6, w: 700, h: 800 }));
        assert_eq!(read.lettering, LETTERING.len() as u8 - 1);
        assert_eq!(read.away, [false, true, false]);
        assert_eq!(Settings::parse(""), Settings::default());
        assert_eq!(Settings::parse("model window = 1 2 0 4"), Settings::default(), "a window of no size is not read");
    }

    #[test]
    fn the_settings_file_is_the_one_a_demonstration_names_none_when_its_map_is_on_and_else_the_systems_own_place() {
        let with = |pairs: &'static [(&'static str, &'static str)]| move |name: &str| pairs.iter().find(|(k, _)| *k == name).map(|(_, v)| v.to_string());
        // Named: that one, map or not.
        assert_eq!(settings_path_from(&with(&[("PLUMBLINE_SETTINGS", "/tmp/s.txt"), ("PLUMBLINE_MAP", "/tmp/m")])), Some(PathBuf::from("/tmp/s.txt")));
        assert!(!old_ways_from(&with(&[("PLUMBLINE_SETTINGS", "/tmp/s.txt"), ("PLUMBLINE_MAP", "/tmp/m")])));
        // The map on and none named: no file at all, and the old ways.
        assert_eq!(settings_path_from(&with(&[("PLUMBLINE_MAP", "/tmp/m"), ("HOME", "/home/x")])), None);
        assert!(old_ways_from(&with(&[("PLUMBLINE_MAP", "/tmp/m")])));
        // Neither: the old ways are not in force.
        assert!(!old_ways_from(&with(&[])));
        #[cfg(target_os = "linux")]
        {
            assert_eq!(settings_path_from(&with(&[("HOME", "/home/x")])), Some(PathBuf::from("/home/x/.config/buildos/settings.txt")));
            assert_eq!(settings_path_from(&with(&[("HOME", "/home/x"), ("XDG_CONFIG_HOME", "/cfg")])), Some(PathBuf::from("/cfg/buildos/settings.txt")));
            assert_eq!(settings_path_from(&with(&[])), None, "no home, no place to keep it: nothing is kept");
        }
    }

    // ------------------------------------------------ the recovery copy

    #[test]
    fn the_recovery_copy_is_beside_the_job_under_a_name_that_is_not_a_job() {
        let there = recovery_path(Path::new("/work/Smith house.json"));
        assert_eq!(there, PathBuf::from("/work/Smith house.json.recovery"));
        assert!(there.extension().is_some_and(|e| e != "json"));
    }

    #[test]
    fn the_recovery_copy_is_what_save_writes_byte_for_byte_and_save_takes_it_away() {
        let dir = folder("bytes");
        let mut doc = crate::build_doc(dir.join("job.json").to_string_lossy().to_string(), None).expect("a new job");
        let how = plumbline_world::editing::Entered { status: Status::Assumed, source: "made-up plan" };
        let level = plumbline_world::editing::storey(&mut doc.job, "Ground", "0 ft", "9 ft", how).expect("a level");
        let t = plumbline_world::editing::wall_type_for(&mut doc.job, "6 in", how).expect("a wall type");
        plumbline_world::editing::walls(&mut doc.job, &level, &t, &[[0.0, 0.0], [20.0, 0.0]], false, plumbline_world::editing::Top::Height("9 ft"), how).expect("a wall");
        doc.job.derive();
        doc.edit("a wall");
        assert!(write_copy(&mut doc).is_ok());
        let copy = std::fs::read(recovery_path(&doc.path)).expect("the copy");
        crate::save(&mut doc);
        let saved = std::fs::read(&doc.path).expect("the job");
        assert_eq!(copy, saved, "the copy is the job as Save writes it, byte for byte");
        assert!(!recovery_path(&doc.path).exists(), "Save takes the copy away");
        assert!(!doc.edited);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn work_beside_a_job_is_offered_only_when_it_is_newer_and_differs() {
        let dir = folder("newer");
        let job = dir.join("job.json");
        // A job never saved: a copy beside it is the only work there is.
        write_recovery(&job, "{\"work\": 1}").expect("a copy");
        assert_eq!(find_recovery(&job).map(|r| r.text), Some("{\"work\": 1}".to_string()));
        // The job saved later than the copy: the copy is older and is not offered.
        std::fs::write(&job, "{\"saved\": 2}").expect("a job");
        let long_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        std::fs::File::options().write(true).open(recovery_path(&job)).expect("open").set_modified(long_ago).expect("time");
        assert_eq!(find_recovery(&job), None);
        // The copy written later than the job: offered, with its age.
        write_recovery(&job, "{\"work\": 3}").expect("a copy");
        std::fs::File::options().write(true).open(&job).expect("open").set_modified(long_ago).expect("time");
        let found = find_recovery(&job).expect("newer");
        assert_eq!(found.text, "{\"work\": 3}");
        assert!(found.age < 120, "{}", found.age);
        // The same as the job: nothing to offer.
        write_recovery(&job, "{\"saved\": 2}").expect("a copy");
        assert_eq!(find_recovery(&job), None);
        // No copy: nothing.
        remove_recovery(&job);
        assert_eq!(find_recovery(&job), None);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_copy_is_put_in_place_whole_and_leaves_no_half_written_file_beside_it() {
        let dir = folder("whole");
        let job = dir.join("job.json");
        write_recovery(&job, "first").expect("a copy");
        write_recovery(&job, "second").expect("a copy");
        assert_eq!(std::fs::read_to_string(recovery_path(&job)).unwrap(), "second");
        let names: Vec<String> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect();
        assert_eq!(names, vec!["job.json.recovery".to_string()]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn taking_the_recovered_work_gives_the_job_not_yet_saved_and_leaving_it_gives_the_job_as_saved() {
        let dir = folder("take");
        let path = dir.join("job.json");
        // The job as saved, and work after it that was not.
        let mut saved = crate::build_doc(path.to_string_lossy().to_string(), None).expect("a job");
        crate::save(&mut saved);
        let was = std::fs::read(&path).unwrap();
        let how = plumbline_world::editing::Entered { status: Status::Assumed, source: "made-up plan" };
        let level = plumbline_world::editing::storey(&mut saved.job, "Ground", "0 ft", "9 ft", how).expect("a level");
        let _ = level;
        saved.job.derive();
        saved.edit("a level");
        let after = file::save(&mut saved.job);
        std::fs::File::options().write(true).open(&path).unwrap().set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(600)).unwrap();
        write_recovery(&path, &after).expect("a copy");
        // Opened again: the work is offered, and nothing is taken.
        let mut next = crate::build_doc(path.to_string_lossy().to_string(), None).expect("opened");
        let offer = next.recovered.clone().expect("offered");
        assert_eq!(offer.text, after);
        assert_eq!(file::save(&mut next.job), String::from_utf8(was.clone()).unwrap(), "nothing is taken without the click");
        assert!(!next.edited);
        // Not saved over while it waits.
        crate::save(&mut next);
        assert!(next.message.starts_with("Not saved: there is recovered work"), "{}", next.message);
        assert_eq!(std::fs::read(&path).unwrap(), was);
        // Taken: the job is that work, not yet saved; Save keeps it.
        let mut taken = crate::build_doc(path.to_string_lossy().to_string(), None).unwrap();
        take(&mut taken);
        assert!(taken.recovered.is_none() && taken.edited);
        assert_eq!(file::save(&mut taken.job), after);
        crate::save(&mut taken);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
        assert!(!recovery_path(&path).exists());
        // Left: the job stays as it was saved and the work stays beside it.
        std::fs::write(&path, &was).unwrap();
        std::fs::File::options().write(true).open(&path).unwrap().set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(600)).unwrap();
        write_recovery(&path, &after).expect("a copy");
        let mut left = crate::build_doc(path.to_string_lossy().to_string(), None).unwrap();
        assert!(left.recovered.is_some());
        leave(&mut left);
        assert!(left.recovered.is_none() && !left.edited);
        assert_eq!(std::fs::read(&path).unwrap(), was);
        assert!(recovery_path(&path).exists(), "the work stays beside the job until it is saved");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_copy_is_written_a_minute_after_the_first_change_and_after_every_tenth() {
        let mut k = Keeper::default();
        // Nothing changed: nothing written, however long.
        assert!(!k.due(0.0, 0, false) && !k.due(500.0, 0, false));
        // Changed at t = 100: not at once.
        assert!(!k.due(100.0, 1, true));
        assert!(!k.due(159.0, 1, true));
        // A minute after the first change.
        assert!(k.due(160.0, 1, true));
        // Nothing more has changed: no rewrite.
        assert!(!k.due(230.0, 1, true) && !k.due(900.0, 1, true));
        // One more change long after the last copy: written at once, a minute having passed.
        assert!(k.due(901.0, 2, true));
        // Another straight after: a minute after that copy.
        assert!(!k.due(902.0, 3, true));
        assert!(!k.due(960.0, 3, true));
        assert!(k.due(961.0, 3, true));
        // Ten changes at once: written at once, not a minute on.
        assert!(!k.due(962.0, 12, true));
        assert!(k.due(962.5, 13, true));
        assert!(!k.due(963.0, 14, true));
        assert!(k.due(1023.0, 14, true));
        // Saved: the count starts again from clean.
        assert!(!k.due(1030.0, 14, false));
        assert!(!k.due(1031.0, 15, true));
        assert!(k.due(1091.0, 15, true));
        // Another job opened in the window: the count is that job's own.
        assert!(!k.due(1200.0, 0, false));
        assert!(!k.due(1201.0, 1, true));
    }

    #[test]
    fn how_long_ago_is_said_as_a_builder_says_it() {
        assert_eq!(age_said(5), "less than a minute ago");
        assert_eq!(age_said(60), "1 minute ago");
        assert_eq!(age_said(12 * 60 + 30), "12 minutes ago");
        assert_eq!(age_said(3600), "1 hour ago");
        assert_eq!(age_said(5 * 3600), "5 hours ago");
        assert_eq!(age_said(86_400 * 2), "2 days ago");
    }

    // ------------------------------------------------ the keys

    #[test]
    fn command_does_what_ctrl_does() {
        let mut keys = ButtonInput::<KeyCode>::default();
        assert!(!command_held(&keys));
        keys.press(KeyCode::ControlLeft);
        assert!(command_held(&keys));
        keys.release(KeyCode::ControlLeft);
        keys.press(KeyCode::SuperLeft);
        assert!(command_held(&keys));
        keys.release(KeyCode::SuperLeft);
        keys.press(KeyCode::SuperRight);
        assert!(command_held(&keys));
        keys.release(KeyCode::SuperRight);
        keys.press(KeyCode::ShiftLeft);
        assert!(!command_held(&keys));
    }

    // ------------------------------------------------ what the panels do with the width

    fn doc_at(width: f32) -> Doc {
        let mut doc = crate::build_doc("/nonexistent/screen-test.json".to_string(), None).expect("a new job");
        doc.fit = Fit { width, folded: folded(width), opened: [false; 2], lettering: 0 };
        doc
    }

    #[test]
    fn at_the_old_size_the_layout_is_the_old_layout() {
        for width in [0.0_f32, 1600.0, 2000.0] {
            let mut doc = doc_at(width);
            if width == 0.0 {
                doc.fit = Fit::default();
            }
            let l = crate::panels::layout(&doc);
            assert_eq!((l.shown, l.left_in, l.right_in, l.tight, l.lines), ([true; 3], crate::scene::LEFT_W, crate::scene::RIGHT_W, false, 0), "{width}");
        }
        // A panel put away with its switch keeps its room, as it always did.
        let mut doc = doc_at(1600.0);
        doc.show.away[1] = true;
        let l = crate::panels::layout(&doc);
        assert_eq!((l.shown[1], l.right_in), (false, crate::scene::RIGHT_W));
    }

    #[test]
    fn a_narrow_window_gives_the_models_room_back_and_a_panel_asked_for_stands_over_it() {
        let doc = doc_at(1300.0);
        let l = crate::panels::layout(&doc);
        assert_eq!((l.shown[0], l.shown[1], l.shown[2]), (true, false, true));
        assert_eq!((l.left_in, l.right_in), (crate::scene::LEFT_W, 0.0));
        let mut doc = doc_at(900.0);
        let l = crate::panels::layout(&doc);
        assert_eq!((l.shown[0], l.shown[1]), (false, false));
        assert_eq!((l.left_in, l.right_in), (0.0, 0.0));
        assert!(l.tight);
        doc.fit.opened[1] = true;
        let l = crate::panels::layout(&doc);
        assert_eq!((l.shown[1], l.right_in), (true, crate::scene::RIGHT_W));
        // Something being typed in a folded panel brings it, so that nothing is typed out of sight.
        let mut doc = doc_at(900.0);
        doc.entry = Some((crate::model::Entry::Pitch, String::new()));
        assert!(crate::panels::layout(&doc).shown[1]);
    }

    #[test]
    fn both_panels_open_in_a_narrow_window_put_the_tools_on_more_lines() {
        let mut doc = doc_at(1200.0);
        doc.fit.opened = [true, true];
        let l = crate::panels::layout(&doc);
        assert!(l.lines >= 2, "{l:?}");
        assert!(l.bar < BAR_SQUEEZE_AT);
        assert!(crate::panels::bar_height(&doc) > crate::scene::BUILD_H);
    }

    #[test]
    fn larger_lettering_folds_the_panels_at_a_wider_window() {
        // The window is as wide as it is; its panels are laid out in the units the lettering scales.
        let window = 1600.0_f32;
        assert_eq!(folded(window / lettering_scale(0)), [false, false]);
        assert_eq!(folded(window / lettering_scale(1)), [false, true]);
    }
}
