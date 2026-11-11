//! How the application looks: one palette, one typeface, and the few building blocks every
//! panel is made of. Nothing here knows about roofs.
//!
//! The look is a drafting table: a pale canvas, ink-dark text, white panels with a
//! hairline border, and colour kept for meaning. Amber is always "assumed", red is always
//! "unknown", blue is always "selected", and each kind of roof line has one colour that
//! the legend, the drawing and the sheet all share.

use bevy::prelude::*;
use bevy::text::{FontFeatureTag, FontFeatures};
use plumbline_world::value::Status;
use plumbline_world::Kind;

pub fn hex(rgb: u32) -> Color {
    Color::srgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub fn canvas() -> Color { hex(0xE9EDF2) }
/// The ground's own colour. Lit by the sun it comes out as the canvas, so the ground has
/// no edge and no horizon: only its grid and the building's shadow show it is there.
pub fn ground() -> Color { hex(0xF2F7FC) }
pub fn grid_minor() -> Color { hex(0xB4BFCB).with_alpha(0.6) }
pub fn grid_major() -> Color { hex(0x93A0AF).with_alpha(0.85) }
pub fn panel() -> Color { hex(0xFFFFFF) }
pub fn panel_alt() -> Color { hex(0xF6F8FA) }
pub fn hairline() -> Color { hex(0xD9DFE7) }
pub fn ink() -> Color { hex(0x1B2430) }
pub fn ink_soft() -> Color { hex(0x5A6676) }
pub fn ink_faint() -> Color { hex(0x8B96A5) }
pub fn accent() -> Color { hex(0x2563EB) }
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
        Kind::Storey => "Storey",
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
        Kind::Type => "Type",
        Kind::Fitting => "Fitting",
        Kind::Room => "Room",
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
pub fn text(fonts: &Fonts, s: impl Into<String>, size: f32, weight: Weight, colour: Color) -> impl Bundle {
    let font = match weight {
        Weight::Regular => fonts.regular.clone(),
        Weight::Medium => fonts.medium.clone(),
        Weight::Semibold => fonts.semibold.clone(),
    };
    let features: FontFeatures = FontFeatures::builder().enable(FontFeatureTag::TABULAR_FIGURES).build();
    (Text::new(s), TextFont { font: FontSource::Handle(font), font_size: FontSize::Px(size), font_features: features, ..default() }, TextColor(colour))
}

/// A white card with a hairline border and a soft shadow. The node says how it is laid
/// out; the card gives it its edge.
pub fn card(mut node: Node) -> impl Bundle {
    node.border = UiRect::all(px(1.0));
    node.border_radius = BorderRadius::all(px(10.0));
    (
        node,
        BackgroundColor(panel()),
        BorderColor::all(hairline()),
        BoxShadow(vec![ShadowStyle { color: Color::srgba(0.1, 0.14, 0.2, 0.10), x_offset: px(0.0), y_offset: px(2.0), spread_radius: px(0.0), blur_radius: px(10.0) }]),
    )
}

/// A node with rounded corners.
pub fn rounded(mut node: Node, radius: f32) -> Node {
    node.border_radius = BorderRadius::all(px(radius));
    node
}

/// A small rounded tag: "Assumed", "Unknown", a count.
pub fn chip(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str, fg: Color, bg: Color) {
    p.spawn((rounded(Node { padding: UiRect::axes(px(8.0), px(3.0)), align_items: AlignItems::Center, column_gap: px(6.0), ..default() }, 999.0), BackgroundColor(bg))).with_children(|c| {
        c.spawn((rounded(Node { width: px(6.0), height: px(6.0), ..default() }, 3.0), BackgroundColor(fg)));
        c.spawn(text(fonts, label, 11.5, Weight::Medium, fg));
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
    p.spawn((rounded(Node { padding: UiRect::axes(px(6.0), px(2.0)), border: UiRect::all(px(1.0)), min_width: px(22.0), justify_content: JustifyContent::Center, ..default() }, 5.0), BackgroundColor(panel_alt()), BorderColor::all(hairline())))
        .with_child(text(fonts, key, 11.0, Weight::Medium, ink_soft()));
}

/// A section heading inside a panel: small, quiet, set apart by space.
pub fn heading(p: &mut ChildSpawnerCommands, fonts: &Fonts, label: &str) {
    p.spawn(Node { margin: UiRect { top: px(16.0), bottom: px(6.0), ..default() }, ..default() }).with_child(text(fonts, label.to_uppercase(), 10.5, Weight::Semibold, ink_faint()));
}

/// A swatch of a line's colour.
pub fn swatch(p: &mut ChildSpawnerCommands, colour: Color) {
    p.spawn((rounded(Node { width: px(14.0), height: px(4.0), ..default() }, 2.0), BackgroundColor(colour)));
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
pub fn glass() -> Color { Color::srgba(0.62, 0.80, 0.90, 0.38) }
pub fn door_leaf() -> Color { hex(0x7A5A3C) }
pub fn equipment() -> Color { hex(0xC5CBD3) }
pub fn porcelain() -> Color { hex(0xF8F8F6) }
pub fn grass() -> Color { hex(0xA9B79A) }
