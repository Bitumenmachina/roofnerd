//! How the application looks: one palette, one typeface, and the few building blocks every
//! panel is made of. Nothing here knows about roofs.
//!
//! The look is an instrument: paper, ink, flat panels set edge to edge and parted by
//! hairlines, no shadows and nothing rounded off, and colour kept for meaning. Amber is
//! always "assumed", red is always "unknown", blue is always "selected", and each kind of
//! roof line has one colour that the legend, the drawing and the sheet all share.

use bevy::prelude::*;
use bevy::text::{FontFeatureTag, FontFeatures};
use plumbline_world::value::Status;
use plumbline_world::Kind;

pub fn hex(rgb: u32) -> Color {
    Color::srgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub fn canvas() -> Color { hex(0xECEDEB) }
/// The ground's own colour. Lit, it comes out as the canvas, so the ground has no edge
/// and no horizon: only its grid and the building's shadow show it is there.
pub fn ground() -> Color { hex(0xF4F5F3) }
pub fn grid_minor() -> Color { hex(0xB9BDBF).with_alpha(0.6) }
pub fn grid_major() -> Color { hex(0x979DA3).with_alpha(0.85) }
pub fn panel() -> Color { hex(0xFFFFFF) }
pub fn panel_alt() -> Color { hex(0xF3F4F2) }
/// The ground a building stands on in 3D: a warm grey, darker than paper, so the
/// building and the glass both read against it. (On plan the ground is paper.)
pub fn yard() -> Color { hex(0xCFCBC2) }
/// The sky (QUESTIONS.md Q22): pale at the horizon, a soft blue overhead. One place to
/// change it, or to take it away: make both `canvas()`.
pub fn sky_horizon() -> Color { hex(0xE6ECF1) }
pub fn sky_overhead() -> Color { hex(0x86AEDB) }

// ---- Glass (call 62; DECISIONS.md call 63 has where the figures come from) ----
// A panel is a sheet of frosted glass over the model: a thick white tint (the text on
// it is dense, and both Apple and Microsoft say dense text wants thick glass), a light
// edge, round corners and two shadows, one near and one far. To retune the look of
// every panel, change these.

/// How far a panel floats from the window's edge and from the next panel.
pub const GAP: f32 = 10.0;
/// A floating panel's corners; a button's; a field's or a key's.
pub const PANEL_RADIUS: f32 = 16.0;
pub const BUTTON_RADIUS: f32 = 8.0;
pub const SMALL_RADIUS: f32 = 6.0;
/// The glass itself, and its light edge.
pub fn frost() -> Color { Color::srgba(0.985, 0.988, 0.992, 0.80) }
pub fn frost_edge() -> Color { Color::srgba(1.0, 1.0, 1.0, 0.72) }
/// What sits on the glass and can be pressed: a little whiter than the glass.
pub fn control() -> Color { Color::srgba(1.0, 1.0, 1.0, 0.62) }
pub fn control_edge() -> Color { Color::srgba(0.07, 0.09, 0.12, 0.10) }
/// A hairline on glass.
pub fn glass_rule() -> Color { Color::srgba(0.07, 0.09, 0.12, 0.09) }
/// What is selected or in hand, on glass: a wash of the accent, never accent-coloured
/// text alone.
pub fn accent_glass() -> Color { Color::srgba(0.114, 0.306, 0.847, 0.14) }
/// The two shadows under a floating panel: near and sharp, far and soft.
pub fn panel_shadow() -> BoxShadow {
    BoxShadow(vec![
        ShadowStyle { color: Color::srgba(0.05, 0.07, 0.10, 0.10), x_offset: px(0.0), y_offset: px(2.0), spread_radius: px(0.0), blur_radius: px(6.0) },
        ShadowStyle { color: Color::srgba(0.05, 0.07, 0.10, 0.16), x_offset: px(0.0), y_offset: px(10.0), spread_radius: px(-4.0), blur_radius: px(28.0) },
    ])
}
/// A field that can be typed in, at rest.
pub fn field_paper() -> Color { hex(0xFAFAF8) }
/// A hairline inside a panel, between rows.
pub fn rule() -> Color { hex(0xE7E9E6) }
pub fn hairline() -> Color { hex(0xC9CDD0) }
pub fn ink() -> Color { hex(0x16191D) }
pub fn ink_soft() -> Color { hex(0x4E5661) }
pub fn ink_faint() -> Color { hex(0x7E8791) }
pub fn accent() -> Color { hex(0x1D4ED8) }
pub fn accent_wash() -> Color { hex(0xE8F0FE) }
pub fn amber() -> Color { hex(0xA9690A) }
pub fn amber_wash() -> Color { hex(0xFFF3D1) }
pub fn red() -> Color { hex(0xB93227) }
pub fn red_wash() -> Color { hex(0xFDE7E4) }
pub fn green() -> Color { hex(0x1F7A4D) }

/// The building under the roof and the roof's own trim. They are there to give the roof
/// something to sit on; they are not elements of the job.
pub fn wall() -> Color { hex(0xF3F0EA) }
pub fn fascia() -> Color { hex(0x59636F) }
/// A roof plane nobody has given a condition yet, and one that has one.
pub fn roof_bare() -> Color { hex(0x97A3B1) }
pub fn roof_copper() -> Color { hex(0xB7733B) }
pub fn roof_other() -> Color { hex(0x6F7F94) }

/// The one colour each kind of roof line has, everywhere it is drawn or named.
pub fn line_colour(kind: Kind) -> Color {
    match kind {
        Kind::Eave => hex(0x1B8A5A),
        Kind::Rake => hex(0x8A5A2B),
        // Not red: red is Unknown, here as everywhere, and a ridge is not a warning.
        Kind::Ridge => hex(0xC026D3),
        Kind::Hip => hex(0xE07B00),
        Kind::Valley => hex(0x1F6FEB),
        Kind::PitchBreak => hex(0x7A7F1A),
        Kind::Headwall => hex(0x0F8B8D),
        Kind::Parapet => hex(0x475569),
        _ => hex(0x5A6676),
    }
}

pub fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Eave => "Eave",
        Kind::Rake => "Rake",
        Kind::Ridge => "Ridge",
        Kind::Hip => "Hip",
        Kind::Valley => "Valley",
        Kind::PitchBreak => "Pitch change",
        Kind::Headwall => "Headwall",
        Kind::RoofPlane => "Roof plane",
        Kind::LowSlopeArea => "Low-slope area",
        Kind::Parapet => "Parapet",
        Kind::Roof => "Roof",
        Kind::Job => "Job",
        Kind::Condition => "Condition",
        Kind::Storey => "Level",
        Kind::Wall => "Wall",
        Kind::Door => "Door",
        Kind::Window => "Window",
        Kind::Floor => "Floor",
        Kind::System => "System",
        Kind::Duct => "Duct",
        Kind::Pipe => "Pipe",
        Kind::Conduit => "Conduit",
        Kind::Cable => "Cable",
        Kind::Equipment => "Equipment",
        Kind::Fixture => "Fixture",
        Kind::Furnishing => "Furnishing",
        Kind::Type => "Type",
        Kind::Fitting => "Fitting",
        Kind::Room => "Room",
        Kind::Material => "Material",
        Kind::Drawing => "Drawing",
        Kind::Prior => "Prior estimate",
    }
}

pub fn status_ink(status: Status) -> Color {
    match status {
        Status::Known => ink(),
        Status::Assumed => amber(),
        Status::Unknown => red(),
    }
}

/// The typeface: Inter, in the three weights the panels use, built into the program.
#[derive(Resource, Clone)]
pub struct Fonts {
    pub regular: Handle<Font>,
    pub medium: Handle<Font>,
    pub semibold: Handle<Font>,
}

#[derive(Clone, Copy)]
pub enum Weight {
    Regular,
    Medium,
    Semibold,
}

pub fn load_fonts(fonts: &mut Assets<Font>) -> Fonts {
    Fonts {
        regular: fonts.add(Font::from_bytes(include_bytes!("../assets/fonts/Inter-Regular.otf").to_vec())),
        medium: fonts.add(Font::from_bytes(include_bytes!("../assets/fonts/Inter-Medium.otf").to_vec())),
        semibold: fonts.add(Font::from_bytes(include_bytes!("../assets/fonts/Inter-SemiBold.otf").to_vec())),
    }
}

/// A run of text. Figures are set tabular, so columns of numbers line up.
/// What each element of the job is called, by its path in the job's file. Kept for
/// `spoken`: a path is how the file names a thing and is never shown.
static NAMES: std::sync::OnceLock<std::sync::RwLock<std::collections::HashMap<String, String>>> = std::sync::OnceLock::new();

/// Tell `spoken` what the job's elements are called now.
pub fn name_these(names: std::collections::HashMap<String, String>) {
    let held = NAMES.get_or_init(Default::default);
    // What a thing was called is kept after it is gone from the job: the message that
    // says it was removed still has to call it by its name. A path is never given out
    // twice, so an old name is never said of a new thing.
    if let Ok(mut held) = held.write() {
        held.extend(names);
    }
}

/// Text as it is written on screen. Every word either window draws passes through
/// here, so that none of the program's own shorthand reaches a person:
///
/// * a thing's path in the file (`wall-0008`) is said as the thing's name ("wall 8"),
///   and a name followed by its own path in brackets is said once;
/// * a figure's name in a formula (`plan_length`) is said as its words ("plan length");
///   a formula can be typed either way;
/// * the machine's number for a failure ("(os error 2)") is left out.
pub fn spoken(text: &str) -> String {
    let names = NAMES.get().and_then(|n| n.read().ok());
    let mut said: Vec<String> = Vec::new();
    let mut out = String::with_capacity(text.len());
    // Inside quotation marks is what a person typed or a file says: left as it is.
    let mut quoted = false;
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let opens = !quoted && word.starts_with(['"', '(']) && word.trim_start_matches('(').starts_with('"');
        let in_quotes = quoted || opens;
        if opens {
            quoted = true;
        }
        if quoted && word.len() > usize::from(opens) && word.trim_end_matches([',', '.', ';', ':', ')']).ends_with('"') {
            quoted = false;
        }
        let inner = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
        // "roof-0072's": the 's is not part of what the thing is called in the file.
        let (word, owns) = match word.find("'s").filter(|k| word[k + 2..].chars().all(|c| !c.is_ascii_alphanumeric())) {
            Some(k) => (&word[..k], &word[k..]),
            None => (word, ""),
        };
        let start = word.find(inner).unwrap_or(word.len());
        let end = word.rfind(inner).map(|k| k + 1).unwrap_or(start);
        let (before, core, after) = (&word[..start], &word[start..end.max(start)], &word[end.max(start)..]);
        // A path with a capital put on it because it began a sentence ("Plane-0073:").
        let capital = core.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && core[1..].chars().all(|c| !c.is_ascii_uppercase());
        let lowered = if capital { format!("{}{}", core[..1].to_ascii_lowercase(), &core[1..]) } else { core.to_string() };
        let core = if capital && lowered.rsplit_once('-').is_some_and(|(stem, number)| !stem.is_empty() && stem.chars().all(|c| c.is_ascii_lowercase()) && number.len() >= 4 && number.chars().all(|c| c.is_ascii_digit())) { lowered.as_str() } else { core };
        let is_path = core.rsplit_once('-').is_some_and(|(stem, number)| !stem.is_empty() && stem.chars().all(|c| c.is_ascii_lowercase()) && number.len() >= 4 && number.chars().all(|c| c.is_ascii_digit()));
        let is_name = core.contains('_') && !core.starts_with('_') && !core.ends_with('_') && core.chars().all(|c| c.is_ascii_lowercase() || c == '_');
        out.push_str(before);
        match (is_path, is_name) {
            (true, _) => match names.as_ref().and_then(|n| n.get(core)) {
                Some(name) => {
                    // At the head of a sentence the name takes the capital the path had.
                    let name = if capital { let mut c = name.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() } else { name.clone() };
                    out.push_str(&name);
                    said.push(name);
                }
                None => out.push_str(core),
            },
            (_, true) => out.push_str(&american(&core.replace('_', " "))),
            _ if in_quotes => out.push_str(core),
            _ => out.push_str(&american(core)),
        }
        out.push_str(after);
        out.push_str(owns);
    }
    // The model's "centre line" is one word on a drawing.
    for (as_written, word) in [("centre line", "centerline"), ("center line", "centerline"), ("Centre line", "Centerline"), ("Center line", "Centerline")] {
        if out.contains(as_written) {
            out = out.replace(as_written, word);
        }
    }
    for name in said {
        let twice = format!("{name} ({name})");
        if out.contains(&twice) {
            out = out.replace(&twice, &name);
        }
    }
    // A kind of thing as the program's own text names it (RoofPlane, LowSlopeArea) is
    // said in its words; and one written with a capital after "a" or "an" loses it.
    for (as_named, words) in [("RoofPlane", "roof plane"), ("LowSlopeArea", "low-slope area"), ("PitchBreak", "change of pitch")] {
        if out.contains(as_named) {
            out = out.replace(as_named, words);
        }
    }
    for kind in ["Job", "Roof", "Eave", "Rake", "Ridge", "Hip", "Valley", "Headwall", "Parapet", "Condition", "Storey", "Level", "Wall", "Door", "Window", "Floor", "System", "Duct", "Pipe", "Conduit", "Cable", "Equipment", "Fixture", "Fitting", "Type", "Room", "Material", "Drawing", "Prior"] {
        for article in ["a ", "an ", "A ", "An "] {
            let said_so = format!("{article}{kind}");
            if out.contains(&said_so) {
                let plain = format!("{article}{}", kind.to_lowercase());
                // Only where the kind's name ends there: "a Wall," and not "a Wallace".
                let mut next = String::with_capacity(out.len());
                let mut rest = out.as_str();
                while let Some(at) = rest.find(&said_so) {
                    let after = &rest[at + said_so.len()..];
                    let whole = !after.chars().next().is_some_and(|c| c.is_ascii_alphanumeric()) && (at == 0 || !rest[..at].chars().next_back().is_some_and(|c| c.is_ascii_alphanumeric()));
                    next.push_str(&rest[..at]);
                    next.push_str(if whole { &plain } else { &said_so });
                    rest = after;
                }
                next.push_str(rest);
                out = next;
            }
        }
    }
    // Money is written with its sign first and its unit hard after: $28.00/SF, not
    // "28.00 $/SF". The pound is LB.
    if out.contains(" $/") {
        let words: Vec<&str> = out.split(' ').collect();
        let mut joined: Vec<String> = Vec::with_capacity(words.len());
        let mut i = 0;
        while i < words.len() {
            let figure = words[i].trim_start_matches(['~', '?', '(']);
            if i + 1 < words.len() && words[i + 1].starts_with("$/") && !figure.is_empty() && figure.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
                let lead = &words[i][..words[i].len() - figure.len()];
                joined.push(format!("{lead}${figure}{}", &words[i + 1][1..]));
                i += 2;
            } else {
                joined.push(words[i].to_string());
                i += 1;
            }
        }
        out = joined.join(" ");
    }
    if out.contains("lb") {
        let bytes = out.as_bytes();
        let mut next = String::with_capacity(out.len());
        let mut i = 0;
        while i < out.len() {
            let here = out[i..].starts_with("lb") && !(i > 0 && bytes[i - 1].is_ascii_alphabetic()) && !bytes.get(i + 2).is_some_and(|b| b.is_ascii_alphabetic());
            if here {
                next.push_str("LB");
                i += 2;
            } else {
                let c = out[i..].chars().next().unwrap_or(' ');
                next.push(c);
                i += c.len_utf8();
            }
        }
        out = next;
    }
    // A backslash before a quote mark is how the program writes a quote inside a quote
    // for itself: 8' 10 1/2" is not written 8' 10 1/2\" on a drawing.
    if out.contains('\\') {
        out = out.replace("\\\"", "\"").replace("\\'", "'");
    }
    // "(os error 2)": the machine's number for why a file could not be read.
    while let Some(at) = out.find(" (os error ") {
        match out[at..].find(')') {
            Some(close) => out.replace_range(at..at + close + 1, ""),
            None => break,
        }
    }
    out
}

/// The screen is written in US English, in the words a US drawing set uses: a floor of
/// the building is a level. The model and the job file keep their own spelling; this is
/// where a word of theirs is said. One word, or a few with spaces between.
fn american(words: &str) -> String {
    const SAID: [(&str, &str); 14] = [
        ("storey", "level"),
        ("storeys", "levels"),
        ("Storey", "Level"),
        ("Storeys", "Levels"),
        ("centre", "center"),
        ("centres", "centers"),
        ("Centre", "Center"),
        ("centred", "centered"),
        ("colour", "color"),
        ("colours", "colors"),
        ("metre", "meter"),
        ("metres", "meters"),
        ("millimetres", "millimeters"),
        ("grey", "gray"),
    ];
    if !words.contains(' ') {
        return SAID.iter().find(|(from, _)| *from == words).map(|(_, to)| to.to_string()).unwrap_or_else(|| words.to_string());
    }
    words.split(' ').map(american).collect::<Vec<_>>().join(" ")
}

/// A name a person gave, exactly as they gave it: a level, a room, a job.
pub fn name(fonts: &Fonts, s: impl Into<String>, size: f32, weight: Weight, colour: Color) -> impl Bundle {
    typed(fonts, s, size, weight, colour)
}

/// Text exactly as it was typed, for the field a person is typing in.
pub fn typed(fonts: &Fonts, s: impl Into<String>, size: f32, weight: Weight, colour: Color) -> impl Bundle {
    let font = match weight {
        Weight::Regular => fonts.regular.clone(),
        Weight::Medium => fonts.medium.clone(),
        Weight::Semibold => fonts.semibold.clone(),
    };
    (Text::new(s), TextFont { font: FontSource::Handle(font), font_size: FontSize::Px(size), ..default() }, TextColor(colour))
}

pub fn text(fonts: &Fonts, s: impl Into<String>, size: f32, weight: Weight, colour: Color) -> impl Bundle {
    let font = match weight {
        Weight::Regular => fonts.regular.clone(),
        Weight::Medium => fonts.medium.clone(),
        Weight::Semibold => fonts.semibold.clone(),
    };
    let features: FontFeatures = FontFeatures::builder().enable(FontFeatureTag::TABULAR_FIGURES).build();
    (Text::new(spoken(&s.into())), TextFont { font: FontSource::Handle(font), font_size: FontSize::Px(size), font_features: features, ..default() }, TextColor(colour))
}

/// A card on the glass: a whiter sheet with round corners and a faint edge.
pub fn card(mut node: Node) -> impl Bundle {
    node.border = UiRect::all(px(1.0));
    node.border_radius = BorderRadius::all(px(BUTTON_RADIUS + 2.0));
    (node, BackgroundColor(control()), BorderColor::all(control_edge()))
}

/// A node with round corners: a button, a key, a field.
pub fn rounded(mut node: Node, radius: f32) -> Node {
    node.border_radius = BorderRadius::all(px(radius));
    node
}

/// A small tag: a square mark in the colour of what it says, and the word.
pub fn chip(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str, fg: Color, _bg: Color) {
    p.spawn(Node { align_items: AlignItems::Center, column_gap: px(5.0), flex_shrink: 0.0, ..default() }).with_children(|c| {
        c.spawn((Node { width: px(6.0), height: px(6.0), ..default() }, BackgroundColor(fg)));
        c.spawn((text(fonts, label, 11.0, Weight::Medium, fg), TextLayout::no_wrap()));
    });
}

pub fn status_chip(p: &mut ChildSpawnerCommands, fonts: &Fonts, status: Status) {
    match status {
        Status::Known => {}
        Status::Assumed => chip(p, fonts, "Assumed", amber(), amber_wash()),
        Status::Unknown => chip(p, fonts, "Unknown", red(), red_wash()),
    }
}

/// A keyboard key, drawn as a key.
pub fn keycap(p: &mut ChildSpawnerCommands, fonts: &Fonts, key: &str) {
    p.spawn((rounded(Node { padding: UiRect::axes(px(4.0), px(0.0)), border: UiRect::all(px(1.0)), min_width: px(16.0), justify_content: JustifyContent::Center, flex_shrink: 0.0, ..default() }, 4.0), BackgroundColor(Color::srgba(0.07, 0.09, 0.12, 0.05)), BorderColor::all(control_edge()), Pickable::IGNORE))
        .with_child((text(fonts, key, 10.0, Weight::Medium, ink_soft()), Pickable::IGNORE));
}

/// A section heading inside a panel: small capitals over a hairline.
pub fn heading(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str) {
    p.spawn((Node { margin: UiRect { top: px(12.0), bottom: px(4.0), ..default() }, padding: UiRect::bottom(px(3.0)), border: UiRect::bottom(px(1.0)), flex_shrink: 0.0, ..default() }, BorderColor::all(glass_rule()))).with_child(text(fonts, label.to_uppercase(), 10.0, Weight::Semibold, ink_soft()));
}

/// What a script that drives the window calls something that can be clicked: the word
/// on it. With `PLUMBLINE_MAP` set, the program writes where each of these is on screen.
/// `scrolls` is set on what lies in a panel that scrolls: off the panel's edge it is not
/// on screen, and is not listed.
#[derive(Component, Clone)]
pub struct Named {
    pub name: String,
    pub scrolls: bool,
}

pub fn named(name: impl Into<String>) -> Named {
    Named { name: name.into(), scrolls: false }
}

/// A button: a word, the key that does the same if there is one, and whether it is the
/// thing in hand. It does nothing by itself; whoever spawns it says what a click does.
pub fn button<'a>(p: &'a mut ChildSpawnerCommands, fonts: &Fonts, label: &str, key: &str, on: bool) -> EntityCommands<'a> {
    let mut e = p.spawn((
        rounded(Node { align_items: AlignItems::Center, column_gap: px(6.0), padding: UiRect::axes(px(10.0), px(4.0)), border: UiRect::all(px(1.0)), min_height: px(26.0), flex_shrink: 0.0, ..default() }, BUTTON_RADIUS),
        BackgroundColor(if on { accent_glass() } else { control() }),
        BorderColor::all(if on { accent() } else { control_edge() }),
        named(label),
    ));
    e.with_children(|b| {
        b.spawn((text(fonts, label, 12.0, Weight::Medium, if on { accent() } else { ink() }), TextLayout::no_wrap(), Pickable::IGNORE));
        if !key.is_empty() {
            keycap(b, fonts, key);
        }
    });
    e
}

/// A field: its name over it, and in it its value, or what is being typed with the caret
/// after it, or faintly what could be typed. `state` is 0 at rest, 1 being typed in,
/// 2 wanted and still empty.
pub fn field<'a>(p: &'a mut ChildSpawnerCommands, fonts: &Fonts, label: &str, value: &str, like: &str, state: u8, width: f32) -> EntityCommands<'a> {
    let mut e = p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: px(2.0), flex_shrink: 0.0, ..default() }, named(label)));
    e.with_children(|f| {
        f.spawn((text(fonts, label.to_uppercase(), 9.5, Weight::Semibold, ink_faint()), TextLayout::no_wrap(), Pickable::IGNORE));
        let edge = match state {
            1 => accent(),
            2 => red(),
            _ => control_edge(),
        };
        f.spawn((rounded(Node { width: px(width), height: px(24.0), padding: UiRect::horizontal(px(6.0)), border: UiRect::all(px(if state == 1 { 1.5 } else { 1.0 })), align_items: AlignItems::Center, overflow: Overflow::clip(), ..default() }, SMALL_RADIUS), BackgroundColor(if state == 1 { panel() } else { control() }), BorderColor::all(edge), Pickable::IGNORE)).with_children(|b| {
            if value.is_empty() && state != 1 {
                b.spawn((text(fonts, like, 12.0, Weight::Regular, ink_faint()), TextLayout::no_wrap(), Pickable::IGNORE));
            } else {
                b.spawn((typed(fonts, value, 12.5, Weight::Medium, ink()), TextLayout::no_wrap(), Pickable::IGNORE));
            }
            if state == 1 {
                b.spawn((Node { width: px(1.5), height: px(15.0), margin: UiRect::left(px(1.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(accent()), Pickable::IGNORE));
            }
        });
    });
    e
}

/// A swatch of a line's colour.
pub fn swatch(p: &mut ChildSpawnerCommands, colour: Color) {
    p.spawn((Node { width: px(14.0), height: px(4.0), flex_shrink: 0.0, ..default() }, BackgroundColor(colour)));
}

/// A pitch the way the trade says it: 6:12. Said by the model, to the places it keeps.
pub fn pitch_text(rise: f64) -> String {
    plumbline_world::view::pitch_said(rise)
}

/// An entered value's text as a panel shows it: a pitch said the trade's way, and a set
/// of points said with the unit they are in.
pub fn trade(text: &str) -> String {
    if let Some(rise) = text.strip_suffix(" in/12") {
        return format!("{rise}:12");
    }
    if let Some(count) = text.strip_suffix(" points ft") {
        return format!("{count} points, in feet");
    }
    text.to_string()
}

/// The colour a system is drawn in, by its classification. These follow what mechanical
/// drawings already use: supply air blue, return air pink, hot water red, cold water cyan,
/// waste brown, vent green, power yellow.
pub fn system_colour(classification: &str) -> Color {
    match classification {
        "Supply Air" => hex(0x3B82F6),
        "Return Air" => hex(0xEC4899),
        "Exhaust Air" => hex(0xA3A635),
        "Refrigerant" => hex(0x8B5CF6),
        "Domestic Cold Water" => hex(0x06B6D4),
        "Domestic Hot Water" => hex(0xEF4444),
        "Sanitary" => hex(0x8B5E34),
        "Vent" => hex(0x65A30D),
        "Power" => hex(0xEAB308),
        "Lighting" => hex(0xF59E0B),
        _ => hex(0x64748B),
    }
}

pub fn discipline_name(d: plumbline_world::Discipline) -> &'static str {
    use plumbline_world::Discipline::*;
    match d {
        Architecture => "Architecture",
        Mechanical => "Mechanical",
        Electrical => "Electrical",
        Plumbing => "Plumbing",
    }
}

pub fn discipline_colour(d: plumbline_world::Discipline) -> Color {
    use plumbline_world::Discipline::*;
    match d {
        Architecture => hex(0x94A3B8),
        Mechanical => hex(0x3B82F6),
        Electrical => hex(0xEAB308),
        Plumbing => hex(0x06B6D4),
    }
}

/// The building's own materials.
pub fn wall_outside() -> Color { hex(0xEDE8DF) }
pub fn wall_inside() -> Color { hex(0xF4F2EE) }
pub fn floor_colour() -> Color { hex(0xC9B79C) }
/// A floor on the plan: paper.
pub fn plan_paper() -> Color { hex(0xFBFBF8) }
pub fn glass() -> Color { Color::srgba(0.40, 0.52, 0.62, 0.55) }
pub fn door_leaf() -> Color { hex(0x7A5A3C) }
/// Casing, sills, corner boards and a window nobody has named a color for: trim white.
pub fn trim() -> Color { hex(0xF0EEE8) }
/// A knob.
pub fn hardware() -> Color { hex(0x3A3632) }
/// What a surface with a pattern on it is multiplied by: a little under white, so that
/// one picked or pointed at can lighten. The patterns are made that much lighter.
pub fn dressed() -> Color { Color::srgb(0.90, 0.90, 0.90) }
pub fn equipment() -> Color { hex(0xC5CBD3) }
pub fn porcelain() -> Color { hex(0xF8F8F6) }

/// A door as a plan draws it: its span through the wall.
pub fn plan_door() -> Color {
    Color::srgb(0.42, 0.24, 0.1)
}

/// A window as a plan draws it: its span through the wall, with the glass down the middle.
pub fn plan_window() -> Color {
    Color::srgb(0.09, 0.15, 0.3)
}

/// The building shown by how sure it is: what is Known in a pale neutral, what is
/// Assumed in amber, what is Unknown in red.
pub fn certainty(status: Status) -> Color {
    match status {
        Status::Known => hex(0xDFE2E3),
        Status::Assumed => hex(0xE3B04B),
        Status::Unknown => hex(0xD6584C),
    }
}

/// Where two things are in the same space.
pub fn clash() -> Color {
    Color::srgb(0.86, 0.12, 0.16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_written_on_screen_is_in_words_not_in_the_files_shorthand() {
        name_these([("wall-0008", "wall 8"), ("type-0004", "wall 6 in"), ("drawing-0003", "A-1 floor plan")].into_iter().map(|(p, n)| (p.to_string(), n.to_string())).collect());
        // A path is said as the thing's name, wherever it stands in the sentence.
        assert_eq!(spoken("removed wall-0008"), "removed wall 8");
        assert_eq!(spoken("wall-0008: 24' 0\" long"), "wall 8: 24' 0\" long");
        assert_eq!(spoken("(wall-0008)"), "(wall 8)");
        // A name with its own path after it in brackets is said once.
        assert_eq!(spoken("Type of wall 8: wall 6 in (type-0004)"), "Type of wall 8: wall 6 in");
        // A figure's name is said as its words; so is every one in a formula.
        assert_eq!(spoken("plan_length × height"), "plan length × height");
        assert_eq!(spoken("face_area − opening_area"), "face area − opening area");
        // The machine's number for a failure is left out.
        assert_eq!(spoken("cannot read a-1.pdf: No such file or directory (os error 2)"), "cannot read a-1.pdf: No such file or directory");
        // What is not the program's shorthand is left as it is: a file's name, a size, a
        // thing nobody has named.
        assert_eq!(spoken("made_up_plan.pdf, page 2"), "made_up_plan.pdf, page 2");
        assert_eq!(spoken("3/4 in x 1-1/2 in"), "3/4 in x 1-1/2 in");
        assert_eq!(spoken("pipe-0099 is not in the job"), "pipe-0099 is not in the job");
    }

    /// The fifteenth judgment, part B: the holes the judge found, each held here.
    #[test]
    fn j15b_the_holes_the_judge_found_in_what_is_written_on_screen() {
        name_these([("duct-0015", "trunk"), ("roof-0072", "roof"), ("plane-0073", "plane 1"), ("eave-0077", "eave 1"), ("job-0001", "the job")].into_iter().map(|(p, n)| (p.to_string(), n.to_string())).collect());
        // A thing taken out of the job is still called by its name in the message that
        // says so: the names known later no longer hold it.
        name_these([("wall-0008", "wall 8")].into_iter().map(|(p, n)| (p.to_string(), n.to_string())).collect());
        assert_eq!(spoken("removed duct-0015"), "removed trunk");
        assert_eq!(spoken("gone: plane-0073, eave-0077"), "gone: plane 1, eave 1");
        // A path that owns something.
        assert_eq!(spoken("plane 1 is solved from roof-0072's footprint"), "plane 1 is solved from roof's footprint");
        // A path given a capital because it began a sentence takes its name, capital kept.
        assert_eq!(spoken("Plane-0073: material set to copper"), "Plane 1: material set to copper");
        // The job itself.
        assert_eq!(spoken("job-0001: overhead set to 12"), "the job: overhead set to 12");
        // The program's own quoting.
        assert_eq!(spoken("Unit cost of \"Gutter 6\\\" half round\" on west"), "Unit cost of \"Gutter 6\" half round\" on west");
        // A kind of thing as the program names it.
        assert_eq!(spoken("a RoofPlane is not moved this way; a wall is"), "a roof plane is not moved this way; a wall is");
        assert_eq!(spoken("storey-0002 is a Wall, not a storey"), "storey-0002 is a wall, not a level");
        assert_eq!(spoken("that is a Storey"), "that is a level");
        assert_eq!(spoken("a Wallace Road job"), "a Wallace Road job");
        // Money with its sign first; the pound in capitals; a word with "lb" in it left.
        assert_eq!(spoken("allowance: 28.00 $/SF"), "allowance: $28.00/SF");
        assert_eq!(spoken("~28.00 $/LF over 2"), "~$28.00/LF over 2");
        assert_eq!(spoken("861.60 lb at $9.00/lb"), "861.60 LB at $9.00/LB");
        assert_eq!(spoken("a bulb and an elbow"), "a bulb and an elbow");
    }

    /// Call 55: the screen is in US English and says "level"; what a person typed, and
    /// what the file calls a thing, is left alone.
    #[test]
    fn the_screen_says_level_and_center_and_leaves_what_was_typed() {
        name_these([("storey-0005", "Ground")].into_iter().map(|(p, n)| (p.to_string(), n.to_string())).collect());
        assert_eq!(spoken("choose the storey to draw on"), "choose the level to draw on");
        assert_eq!(spoken("Storey height of storey-0005: 9 ft"), "Level height of Ground: 9 ft");
        assert_eq!(spoken("height above its storey, to its centre"), "height above its level, to its center");
        assert_eq!(spoken("its centre line is 6 in off"), "its centerline is 6 in off");
        assert_eq!(spoken("two storeys' walls"), "two levels' walls");
        // A name in quotation marks is the person's own.
        assert_eq!(spoken("there is already something called \"Storey 2\" under the job"), "there is already something called \"Storey 2\" under the job");
        assert_eq!(spoken("(\"Centre Street\") on this storey"), "(\"Centre Street\") on this level");
        // An inch mark is not a quotation mark.
        assert_eq!(spoken("8' 10 1/2\" above its storey"), "8' 10 1/2\" above its level");
        // A path is still a path, and a name with the word in it is not a word.
        assert_eq!(spoken("storey-0099 is not in the job"), "storey-0099 is not in the job");
        assert_eq!(spoken("multistorey"), "multistorey");
    }
}
