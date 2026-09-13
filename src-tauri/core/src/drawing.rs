// ── The drawings ───────────────────────────────────────────────────────────
// One drawing model, three drawings, and not one line of geometry on the far
// side of the wire.
//
// Patrick's ruling (D146) is that the 3D model is the function and the 2D
// outputs are produced FROM it. This is where that becomes true of drawings: the
// tapered layout, the fastening plan and the section through a drain are all
// built here, out of the same solved field the Model view stands up, and what
// goes out is polygons, lines, texts and dimensions in feet. The front end maps
// feet to the screen and paints; it decides nothing about the roof.
//
// That division is not tidiness. A drawing that works the roof out for itself is
// a second arithmetic, and `CLAUDE.md` names the one that disagrees silently:
// the drawing. The estimate, the Model, the Plan's overlays and the section now
// come off one solve, so they cannot disagree about what is on the roof.
//
// WHAT IS IN A DRAWING. Polygons with a role and, where it is a board, the letter
// printed on it. Lines with a role, dashed or not. Texts at a point. Dimensions
// between two points with the measure already written out, because how a roofer
// writes a length is this program's business and not the browser's. A key. A note
// saying what the drawing is drawn at. Roles are words, not colours — the
// stylesheet owns colour, and a payload carrying hex would be a second palette.
//
// COORDINATES. Feet, always, and plan y grows DOWN the sheet the way every trace
// in this program is stored. A plan drawing is in the roof's own plan
// coordinates, so the Plan can lay it straight over the sheet it was traced off.
// A section is in its own frame: x runs from the far end of the cut to the drain,
// and y is height off the top of deck with UP NEGATIVE — the same direction plan
// y goes, so one renderer draws both without a special case. The vertical
// exaggeration is stated and applied by the renderer, the way the Model's is: a
// quarter in twelve is invisible at true scale, and a board is half an inch.

use serde::Serialize;

use crate::geom::{bounds, inside_ring, Point, Shape};
use crate::model::{Board, Condition, Document, Line, SolvedTaper, TaperField, TaperRegion};
use crate::taper::{read_at, MIN_FLASHING_INCHES};

// ── the model ──────────────────────────────────────────────────────────────

/// A filled shape on a drawing.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawnShape {
    pub shape: Shape,
    /// What it is, in one word the stylesheet knows: `board`, `sump`, `nofall`,
    /// `cricket`, `deck`, `fill`, `panel`, `wall`, `coping`, `metal`, `drain`.
    pub role: String,
    /// The letter printed on the panel, where it carries one.
    pub letter: Option<String>,
    /// Which course of the build-up this is, for a section. `None` on a plan: a
    /// plan has no layers, and the order a section stacks in is read off this.
    pub layer: Option<String>,
}

/// A line, open or closed.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawnLine {
    pub points: Vec<Point>,
    /// `outline`, `valley`, `ridge`, `joint`, `membrane`, `coping`, `cleat`,
    /// `flashing`, `termbar`, `deck`, `leader`, `drain`.
    pub role: String,
    pub dashed: bool,
    pub closed: bool,
    /// Which course of the build-up it is, where it is one. The deck is a line
    /// and not a band — nothing in the job states how thick it is — and it is
    /// still the bottom of the stack, so it carries its place in the order.
    pub layer: Option<String>,
}

/// A word on the drawing, at a place on it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawnText {
    pub at: Point,
    pub text: String,
    /// `letter`, `label`, `thickness`, `key`.
    pub role: String,
    /// `start`, `middle` or `end`, as a drawing anchors its text.
    pub anchor: String,
    /// Where the leader comes from, when the word sits off the thing it names.
    pub leader: Option<Point>,
}

/// A measure between two places, already written the way a roofer writes it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub a: Point,
    pub b: Point,
    /// How far off the two points the line sits, square to them, in feet.
    pub offset: f64,
    pub text: String,
    /// True where the number came out of the solve rather than off the job.
    pub derived: bool,
}

/// The box a drawing needs, in its own feet.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Extent {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

/// One drawing: everything on it, in feet.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drawing {
    /// `layout`, `fastening`, or `section-<n>`.
    pub id: String,
    /// What it is called where an estimator reads it.
    pub title: String,
    /// What it is OF, in one line: which cut, through which drain, to which wall.
    pub subtitle: Option<String>,
    /// What it is drawn at, and how far the vertical is stretched, in words.
    pub note: String,
    /// Feet to the inch on paper. The renderer draws at this where the window has
    /// the room and says so when it has to come down.
    pub feet_per_inch: f64,
    /// How far the vertical is stretched. One on a plan.
    pub vertical_exaggeration: f64,
    /// Nothing to draw, and why — never an empty picture with no word on it.
    pub nothing_yet: Option<String>,
    pub shapes: Vec<DrawnShape>,
    pub lines: Vec<DrawnLine>,
    pub texts: Vec<DrawnText>,
    pub dimensions: Vec<Dimension>,
    /// The layer key, bottom to top, as a list the renderer stacks.
    pub key: Vec<String>,
    /// Where the key sits, and the thing it points at.
    pub key_at: Option<Point>,
    pub key_from: Option<Point>,
    pub extent: Option<Extent>,
    /// Every condition the drawing goes through. The Estimate names their lines.
    pub through: Vec<String>,
    /// Where the cut is on the PLAN, for a view that has to stand the drawing up
    /// again: the drain it runs out of, the way out of it, how far it runs and
    /// the datum it is measured from. A plan drawing has none of this, because a
    /// plan drawing is already where the roof is.
    pub cut_at: Option<Point>,
    pub cut_along: Option<Point>,
    pub cut_run_feet: f64,
    pub cut_elevation: f64,
}

impl Drawing {
    fn new(id: &str, title: &str, feet_per_inch: f64, exaggeration: f64) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            subtitle: None,
            note: String::new(),
            feet_per_inch,
            vertical_exaggeration: exaggeration,
            nothing_yet: None,
            shapes: Vec::new(),
            lines: Vec::new(),
            texts: Vec::new(),
            dimensions: Vec::new(),
            key: Vec::new(),
            key_at: None,
            key_from: None,
            extent: None,
            through: Vec::new(),
            cut_at: None,
            cut_along: None,
            cut_run_feet: 0.0,
            cut_elevation: 0.0,
        }
    }

    fn shape(&mut self, shape: Shape, role: &str, letter: Option<String>, layer: Option<&str>) {
        self.shapes.push(DrawnShape {
            shape,
            role: role.to_string(),
            letter,
            layer: layer.map(str::to_string),
        });
    }

    fn line(&mut self, points: Vec<Point>, role: &str, dashed: bool, closed: bool) {
        self.layered(points, role, dashed, closed, None);
    }

    fn layered(
        &mut self,
        points: Vec<Point>,
        role: &str,
        dashed: bool,
        closed: bool,
        layer: Option<&str>,
    ) {
        if points.len() < 2 {
            return;
        }
        self.lines.push(DrawnLine {
            points,
            role: role.to_string(),
            dashed,
            closed,
            layer: layer.map(str::to_string),
        });
    }

    fn text(&mut self, at: Point, text: String, role: &str, anchor: &str) {
        self.texts.push(DrawnText {
            at,
            text,
            role: role.to_string(),
            anchor: anchor.to_string(),
            leader: None,
        });
    }

    fn dimension(&mut self, a: Point, b: Point, offset: f64, text: String, derived: bool) {
        self.dimensions.push(Dimension { a, b, offset, text, derived });
    }

    /// The box everything on it needs. Worked out here rather than in the window,
    /// so that what a drawing is OF is the drawing's own answer.
    fn measure(&mut self) {
        let mut every: Vec<Point> = Vec::new();
        for s in &self.shapes {
            every.extend(s.shape.ring.iter().copied());
        }
        for l in &self.lines {
            every.extend(l.points.iter().copied());
        }
        for t in &self.texts {
            every.push(t.at);
        }
        for d in &self.dimensions {
            every.push(d.a);
            every.push(d.b);
        }
        self.extent = bounds(&every).map(|b| Extent {
            min_x: b.min_x,
            min_y: b.min_y,
            max_x: b.max_x,
            max_y: b.max_y,
        });
    }
}

/// The three drawings the 2D outputs are made of.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drawings {
    pub solve_id: u64,
    /// The tapered layout: the board grid with a letter on every board.
    pub layout: Drawing,
    /// The fastening plan. Empty until the wind zones are solved, and it says so.
    pub fastening: Drawing,
    /// One section per drain on the roof, cut along the boards.
    pub sections: Vec<Drawing>,
}

// ── how a roofer writes a length ───────────────────────────────────────────

/// Sixteenths, reduced, which is as fine as a tape reads and as fine as a
/// tapered schedule is ever specified.
fn sixteenths(v: f64) -> (i64, i64, i64) {
    let total = (v * 16.0).round() as i64;
    let whole = total.div_euclid(16);
    let mut n = total.rem_euclid(16);
    let mut d = 16;
    while n != 0 && n % 2 == 0 {
        n /= 2;
        d /= 2;
    }
    (whole, n, d)
}

/// Inches the way a roofer writes them: `1/2"`, `6"`, `4 1/2"`.
pub fn inches_written(v: f64) -> String {
    let (whole, n, d) = sixteenths(v);
    match (whole, n) {
        (0, 0) => "0\"".to_string(),
        (w, 0) => format!("{w}\""),
        (0, n) => format!("{n}/{d}\""),
        (w, n) => format!("{w} {n}/{d}\""),
    }
}

/// Feet and inches the way a roofer writes them: `22'-0"`, `4'-6"`, `3'-6 1/2"`.
pub fn feet_written(v: f64) -> String {
    let whole_feet = v.floor();
    let inches = (v - whole_feet) * 12.0;
    let (mut feet, mut left) = (whole_feet as i64, inches);
    // Twelve inches is a foot, and a drawing that says 21'-12" is a drawing
    // nobody trusts with the next number either.
    if (left - 12.0).abs() < 1.0 / 32.0 || left > 12.0 - 1.0 / 32.0 {
        feet += 1;
        left = 0.0;
    }
    let (whole, n, d) = sixteenths(left);
    match n {
        0 => format!("{feet}'-{whole}\""),
        _ => format!("{feet}'-{whole} {n}/{d}\""),
    }
}

/// Which way a direction points, in the eight words anybody can act on.
///
/// North is up the sheet, which is how a roof plan is read and the convention the
/// Model's own readout already keeps.
fn compass(u: Point) -> &'static str {
    let degrees = (u.x).atan2(-u.y).to_degrees();
    let points = ["north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west"];
    points[(((degrees + 360.0) % 360.0) / 45.0).round() as usize % 8]
}

// ── the drawings ───────────────────────────────────────────────────────────

/// Every drawing this job comes to.
pub fn drawings(document: &Document, taper: &SolvedTaper) -> Drawings {
    Drawings {
        solve_id: taper.solve_id,
        layout: layout_of(taper),
        fastening: fastening_of(taper),
        sections: sections_of(document, taper),
    }
}

// ── the tapered layout ─────────────────────────────────────────────────────

/// The board grid, with a letter on every board.
///
/// This is the drawing a crew orders off and cuts to, and it is the layout NRCA
/// draws (pp.165–166): the ground split between the drains, four planes to each,
/// the valleys at 45 degrees where two of them tie, the boards on a 4 ft module
/// square to the wall they are laid to, and the sump cut down at each drain.
/// Every board on it is a board in C2's bill — the same `Board`, the same shape
/// after the cut, the same letter — so a board on this drawing is a board that
/// got bought.
fn layout_of(taper: &SolvedTaper) -> Drawing {
    let mut d = Drawing::new("layout", "Tapered layout", 8.0, 1.0);
    d.note = "The tapered layout · boards 4'-0\" square, laid to the longest wall".to_string();

    if taper.fields.is_empty() {
        d.nothing_yet = Some(match taper.refused.first() {
            Some(refused) => format!("there is no layout to draw: {}", refused.why),
            None => "nothing tapered is traced yet, so there is no layout to draw".to_string(),
        });
        return d;
    }

    let mut lettered = 0usize;
    let mut unlettered = 0usize;

    for field in &taper.fields {
        d.through.push(field.condition_id.clone());
        d.line(field.ring.clone(), "outline", false, true);

        for (index, region) in field.regions.iter().enumerate() {
            for board in &region.boards {
                for piece in &board.shapes {
                    d.shape(piece.clone(), "board", board.letter.clone(), None);
                }
                match &board.letter {
                    Some(letter) => {
                        lettered += 1;
                        d.text(board.centre, letter.clone(), "letter", "middle");
                    }
                    None => unlettered += 1,
                }
            }
            // The ground the cap holds flat, which is where the water stops.
            for facet in region.facets.iter().filter(|f| f.flat) {
                d.shape(facet.shape.clone(), "nofall", None, None);
            }
            for valley in &region.valleys {
                d.line(vec![valley.a, valley.b], "valley", true, false);
            }
            // A ridge belongs to BOTH the regions it divides, so it is drawn
            // once — by the earlier of the two — rather than twice in the same
            // place. Two dashed lines at one x with different dash phases read
            // as a doubled line, which is a mark that is not on the roof.
            for ridge in region.ridges.iter().filter(|r| r.meets > index) {
                d.line(vec![ridge.a, ridge.b], "ridge", true, false);
            }
            // What the roof comes to at the drain and at the far corner of this
            // ground: the two numbers a layout is read for.
            d.text(
                region.drain,
                inches_written(region.thin_inches),
                "thickness",
                "middle",
            );
        }

        if let Some(thick) = field.thick_inches {
            if let Some(at) = thickest_place(field) {
                d.text(at, inches_written(thick), "thickness", "middle");
            }
        }

        for sump in &field.sumps {
            for piece in &sump.shapes {
                d.shape(piece.clone(), "sump", None, None);
            }
        }
    }

    for cricket in &taper.crickets {
        d.through.push(cricket.condition_id.clone());
        let plan = plan_of(&cricket.positions);
        if plan.len() >= 3 {
            d.line(plan, "cricket", false, true);
        }
    }
    for sump in &taper.sumps {
        d.through.push(sump.condition_id.clone());
    }

    if let Some(bill) = taper.fields.iter().find_map(|f| f.bill.as_ref()) {
        d.key.push(format!(
            "{} {} · {} in 12",
            bill.manufacturer,
            bill.product,
            inches_written(bill.slope_inches_per_foot).trim_end_matches('"'),
        ));
        for (letter, row) in &bill.by_letter {
            d.key.push(format!("{letter} · {} boards", row.boards));
        }
        d.key.push(format!(
            "flat board {} · {} of them",
            inches_written(bill.fill_step_inches),
            bill.fill_boards
        ));
    }
    if unlettered > 0 {
        d.key.push(format!("{unlettered} boards the table prints no letter for"));
    }
    d.key.push(format!("{lettered} lettered boards on the roof"));

    d.measure();
    d
}

/// The far corner of a field: where the layout is thickest, for the label.
fn thickest_place(field: &TaperField) -> Option<Point> {
    let mut best: Option<(f64, Point)> = None;
    for region in &field.regions {
        for board in &region.boards {
            let deep = board.thickness_inches;
            if best.map(|(had, _)| deep > had).unwrap_or(true) {
                best = Some((deep, board.centre));
            }
        }
    }
    best.map(|(_, p)| p)
}

/// A cricket's footprint, out of the triangles the solve placed.
fn plan_of(positions: &[f64]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    for t in positions.chunks(3) {
        if t.len() < 3 {
            continue;
        }
        let p = Point::new(t[0], t[2]);
        if !out.iter().any(|q| (q.x - p.x).abs() < 1e-9 && (q.y - p.y).abs() < 1e-9) {
            out.push(p);
        }
    }
    // The corners in the order they go round, which triangles do not arrive in.
    if out.len() >= 3 {
        let middle = Point::new(
            out.iter().map(|p| p.x).sum::<f64>() / out.len() as f64,
            out.iter().map(|p| p.y).sum::<f64>() / out.len() as f64,
        );
        out.sort_by(|a, b| {
            let one = (a.y - middle.y).atan2(a.x - middle.x);
            let two = (b.y - middle.y).atan2(b.x - middle.x);
            one.partial_cmp(&two).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    out
}

// ── the fastening plan ─────────────────────────────────────────────────────

/// Empty, and saying so.
///
/// The zones, the pressures and the counts are C4's. An overlay that drew a
/// pattern from nothing would be the worst thing in this program: a fastening
/// plan is what a crew works to, and one that looks solved and is not would be
/// followed.
fn fastening_of(taper: &SolvedTaper) -> Drawing {
    let mut d = Drawing::new("fastening", "Fastening plan", 8.0, 1.0);
    d.note = "The fastening plan".to_string();
    d.nothing_yet = Some(
        "Nothing is drawn here yet. The corner and edge zones come off the wind \
         pressure for this building and its exposure, and nothing has worked those \
         out, so there is no pattern to lay down."
            .to_string(),
    );
    let _ = taper;
    d
}

// ── the section ────────────────────────────────────────────────────────────

/// Where a cut runs, and what it ends against.
struct Cut<'a> {
    field: &'a TaperField,
    region: &'a TaperRegion,
    drain: Point,
    /// The way out of the drain the cut runs, as a unit vector on the plan.
    away: Point,
    run_feet: f64,
    /// True where the far end is the roof's own edge rather than a ridge.
    at_edge: bool,
}

impl Cut<'_> {
    /// A place on the cut, at x feet from the far end, in plan.
    fn plan_at(&self, x: f64) -> Point {
        let t = self.run_feet - x;
        Point::new(self.drain.x + self.away.x * t, self.drain.y + self.away.y * t)
    }

    /// How deep the roof is at x, in inches.
    fn depth_at(&self, x: f64) -> f64 {
        read_at(
            self.plan_at(x),
            &self.region.planes,
            self.field.start_inches,
            self.region.cap_inches,
        )
        .map(|r| r.thickness_inches)
        .unwrap_or(self.field.start_inches)
    }
}

/// One section for every drain on the roof.
fn sections_of(document: &Document, taper: &SolvedTaper) -> Vec<Drawing> {
    let mut out = Vec::new();
    let mut at = 0usize;
    for field in &taper.fields {
        for region in &field.regions {
            let Some(cut) = cut_for(field, region) else { continue };
            out.push(section_of(document, taper, &cut, at));
            at += 1;
        }
    }
    out
}

/// Which way to cut through one drain, and how far the cut runs.
///
/// ALONG THE BOARDS. That is the direction the layout is set out in, the
/// direction a tapered schedule states its run in, and the direction the panels
/// step — so it is the direction a section has anything to show. Cutting square
/// to it would draw one thickness the whole way across.
///
/// TO THE WALL, WHERE THERE IS ONE. A section detail exists to prove the assembly
/// where the roof stops: the flashing, the coping, the cleat and the bar are all
/// at the wall, and a cut that stops at a ridge shows none of them. So where one
/// of the two ways out reaches the roof's own edge, that one wins, and where both
/// do the longer wins. Where neither does, the longer wins and the drawing says
/// the far end is the ridge it shares with the next drain.
fn cut_for<'a>(field: &'a TaperField, region: &'a TaperRegion) -> Option<Cut<'a>> {
    let drain = region.drain;
    let reach = span_of(&field.ring) * 2.0 + 1.0;
    let field_ground = [Shape::of(field.ring.clone())];

    let mut best: Option<Cut<'a>> = None;
    for away in [
        Point::new(-field.axis_along.x, -field.axis_along.y),
        field.axis_along,
    ] {
        let run = reach_along(&region.shapes, drain, away, reach);
        if run <= 1e-6 {
            continue;
        }
        let to_the_edge = reach_along(&field_ground, drain, away, reach);
        let at_edge = run >= to_the_edge - 1e-6;
        let offer = Cut { field, region, drain, away, run_feet: run, at_edge };
        best = Some(match best {
            None => offer,
            // A cut that reaches the roof's edge beats one that stops at a ridge,
            // whatever the two lengths are; between two of a kind, the longer.
            Some(had) => {
                if (offer.at_edge && !had.at_edge)
                    || (offer.at_edge == had.at_edge && offer.run_feet > had.run_feet)
                {
                    offer
                } else {
                    had
                }
            }
        });
    }
    best
}

/// How far a ray from a point gets before it leaves the ground it starts on.
fn reach_along(ground: &[Shape], from: Point, u: Point, reach: f64) -> f64 {
    let to = Point::new(from.x + u.x * reach, from.y + u.y * reach);
    let mut out: f64 = 0.0;
    for shape in ground {
        for (a, b) in inside_ring(from, to, &shape.ring) {
            let at_a = (a.x - from.x) * u.x + (a.y - from.y) * u.y;
            let at_b = (b.x - from.x) * u.x + (b.y - from.y) * u.y;
            if at_a <= 1e-6 {
                out = out.max(at_b);
            }
        }
    }
    out
}

/// How far apart the two furthest corners of a ring are.
fn span_of(ring: &[Point]) -> f64 {
    match bounds(ring) {
        None => 0.0,
        Some(b) => (b.max_x - b.min_x).hypot(b.max_y - b.min_y),
    }
}

/// How far the vertical is stretched on a section.
///
/// The handoff's Frame 3, and the reason is the roof: a board is half an inch on
/// a run of twenty-two feet. At true scale the whole build-up is a line and the
/// drawing proves nothing. Four times, stated on the drawing in as many words,
/// so nobody scales a thickness off it.
const SECTION_EXAGGERATION: f64 = 4.0;

/// Feet to the inch on a section. 1" = 4'-0" is the handoff's.
const SECTION_FEET_PER_INCH: f64 = 4.0;

/// How far below the deck the drain's leader is drawn, in feet.
const LEADER_FEET: f64 = 1.5;

/// How deep the deck is DRAWN where nothing states how deep it is, in inches.
///
/// A symbol, the same kind of thing as the drain's leader: a section that stops
/// at the datum with nothing under it reads as a roof floating in air, and Frame
/// 3 draws the deck it lands on. So a band goes in, no dimension is put on it,
/// and the label says nothing in the job states it. Where the field types `DECK`
/// the band is that and the label says so.
const DECK_SYMBOL_INCHES: f64 = 1.5;

fn section_of(document: &Document, taper: &SolvedTaper, cut: &Cut, index: usize) -> Drawing {
    let letter = (b'A' + (index % 26) as u8) as char;
    let named = format!("Cut {letter}\u{2013}{letter}");
    let mut d = Drawing::new(
        &format!("section-{index}"),
        &format!("Section {letter}\u{2013}{letter}"),
        SECTION_FEET_PER_INCH,
        SECTION_EXAGGERATION,
    );
    d.note = format!(
        "Section {letter}\u{2013}{letter} · horizontal 1\" = {} · vertical exaggerated \u{d7}{} so the board thickness reads",
        feet_written(SECTION_FEET_PER_INCH),
        SECTION_EXAGGERATION as i64,
    );

    let field = cut.field;
    let run = cut.run_feet;
    let start = field.start_inches;
    d.through.push(field.condition_id.clone());
    d.cut_at = Some(cut.drain);
    d.cut_along = Some(cut.away);
    d.cut_run_feet = run;
    d.cut_elevation = field.elevation;

    // ── what the cut ends against ───────────────────────────────────────────
    let far = cut.plan_at(0.0);
    let wall = if cut.at_edge { wall_at(document, far) } else { None };
    let drain_named = drain_at(document, cut.drain);
    let drain_number = which_drain(taper, cut.drain);
    if let Some(id) = &drain_named {
        d.through.push(id.clone());
    }

    d.subtitle = Some(match (&wall, cut.at_edge) {
        (Some(w), _) => format!(
            "{named} · Drain {drain_number} {} to {}",
            compass(cut.away),
            w.name,
        ),
        (None, true) => format!(
            "{named} · Drain {drain_number} {} to the roof's edge",
            compass(cut.away)
        ),
        (None, false) => format!(
            "{named} · Drain {drain_number} {} to the ridge it shares with the next drain",
            compass(cut.away)
        ),
    });
    if let Some(w) = &wall {
        d.through.push(w.id.clone());
    }

    // ── the deck it all lands on ────────────────────────────────────────────
    //
    // Existing construction: it is under the datum, it is not in the bill, and
    // it is the bottom of the stack a section is read up from. Drawn as a band
    // because a section that stops at the datum with nothing under it reads as a
    // roof in mid air — and at a SYMBOL depth with no dimension on it where
    // nobody states one, which is the same rule the drain's leader follows.
    let field_condition = document.conditions.iter().find(|c| c.id == field.condition_id);
    let deck_inches = field_condition.and_then(|c| c.property("DECK"));
    let deck_feet = deck_inches.unwrap_or(DECK_SYMBOL_INCHES) / 12.0;
    d.shape(
        Shape::of(vec![
            Point::new(0.0, 0.0),
            Point::new(run, 0.0),
            Point::new(run, deck_feet),
            Point::new(0.0, deck_feet),
        ]),
        "deck",
        None,
        Some("deck"),
    );
    d.line(vec![Point::new(0.0, 0.0), Point::new(run, 0.0)], "deck", false, false);
    // Under the deck and hard left, where there is nothing else: the words on a
    // section have to stand clear of each other or the drawing is a paragraph
    // with lines through it.
    d.text(
        Point::new(0.2, deck_feet + 0.45),
        match deck_inches {
            Some(deep) => format!(
                "existing deck {} · the datum · existing construction, not in the bill",
                inches_written(deep)
            ),
            None => "existing deck · the datum · nothing states its thickness, so it is drawn as a symbol"
                .to_string(),
        },
        "label",
        "start",
    );

    // ── the boards, cell by cell, the way they get laid ─────────────────────
    let sump = field.sumps.iter().find(|s| s.at == cut.drain);
    let half_sump = sump.map(|s| s.side_feet / 2.0).unwrap_or(0.0);
    let panel_feet = field.bill.as_ref().map(|b| b.panel_feet).unwrap_or(4.0);
    let step = field.bill.as_ref().map(|b| b.fill_step_inches).unwrap_or(0.0);

    for board in boards_along(cut, panel_feet) {
        let t = (board.centre.x - cut.drain.x) * cut.away.x
            + (board.centre.y - cut.drain.y) * cut.away.y;
        let near = (run - t - panel_feet / 2.0).max(0.0);
        let one = (run - t + panel_feet / 2.0).min(run);
        if one - near <= 1e-6 {
            continue;
        }
        // The flat board under the tapered panel, course by course, because the
        // joint between two courses is a line a crew stagger and a line a section
        // is read for.
        for course in 0..board.fill_boards {
            let under = -(course as f64 * step) / 12.0;
            let over = -((course + 1) as f64 * step) / 12.0;
            d.shape(
                Shape::of(vec![
                    Point::new(near, under),
                    Point::new(one, under),
                    Point::new(one, over),
                    Point::new(near, over),
                ]),
                "fill",
                None,
                Some("fill"),
            );
        }
        // And the lettered panel on top of it, its top the solved surface itself.
        let fill = -(board.fill_inches) / 12.0;
        d.shape(
            Shape::of(vec![
                Point::new(near, fill),
                Point::new(one, fill),
                Point::new(one, -cut.depth_at(one) / 12.0),
                Point::new(near, -cut.depth_at(near) / 12.0),
            ]),
            "panel",
            board.letter.clone(),
            Some("panel"),
        );
        if let Some(l) = &board.letter {
            let middle = (near + one) / 2.0;
            d.text(
                Point::new(middle, (fill - cut.depth_at(middle) / 12.0) / 2.0),
                l.clone(),
                "letter",
                "middle",
            );
        }
    }

    // ── the sump the drain sits in ──────────────────────────────────────────
    if let Some(s) = sump {
        let rim = (run - half_sump).max(0.0);
        d.shape(
            Shape::of(vec![
                Point::new(rim, 0.0),
                Point::new(run, 0.0),
                Point::new(run, -start / 12.0),
                Point::new(rim, -cut.depth_at(rim) / 12.0),
            ]),
            "sump",
            None,
            Some("sump"),
        );
        d.dimension(
            Point::new(run - s.side_feet, -start / 12.0),
            Point::new(run, -start / 12.0),
            LEADER_FEET + 1.0,
            format!(
                "sump {} \u{d7} {} deep",
                feet_written(s.side_feet),
                inches_written(s.depth_inches)
            ),
            true,
        );
    }

    // ── the surface, and everything the condition says goes over it ─────────
    let surface = surface_line(cut, panel_feet);
    let mut over = 0.0;
    let layers: Vec<&Line> = field_condition.map(Condition::layers).unwrap_or_default();
    for layer in &layers {
        let thick = layer.thickness.unwrap_or(0.0);
        let under: Vec<Point> =
            surface.iter().map(|p| Point::new(p.x, p.y - over / 12.0)).collect();
        let above: Vec<Point> = surface
            .iter()
            .rev()
            .map(|p| Point::new(p.x, p.y - (over + thick) / 12.0))
            .collect();
        let mut ring = under;
        ring.extend(above);
        d.shape(Shape::of(ring), "cover", None, Some(&layer.description));
        over += thick;
    }
    // The finished surface, as a line, because that is what water runs on.
    d.line(
        surface.iter().map(|p| Point::new(p.x, p.y - over / 12.0)).collect(),
        "membrane",
        false,
        false,
    );

    // ── the wall it stops against ───────────────────────────────────────────
    let roof_at_wall = -cut.depth_at(0.0) / 12.0 - over / 12.0;
    if let Some(w) = &wall {
        let face = w.thick_feet.unwrap_or(0.0);
        let top = -w.height_feet;
        d.shape(
            Shape::of(vec![
                Point::new(0.0, 0.0),
                Point::new(face, 0.0),
                Point::new(face, top),
                Point::new(0.0, top),
            ]),
            "wall",
            None,
            None,
        );
        // Above the cap, where there is nothing else. Beside the wall it lands on
        // top of its own height dimension, which is what Frame 3 does and what a
        // drawing should not.
        d.text(Point::new(-0.25, top - 1.15), w.name.clone(), "label", "start");
        // Outboard of the wall, which is where a parapet height is dimensioned
        // and the only side of it with room.
        d.dimension(
            Point::new(0.0, top),
            Point::new(0.0, 0.0),
            3.0,
            format!("{} parapet", inches_written(w.height_feet * 12.0)),
            false,
        );

        // The cap on it, as formed metal: the top, and a leg down each face as
        // long as the girth has left over after the cap's own width. That is what
        // a stretch-out IS — the flat width of coil the profile eats — so the legs
        // are read off the job rather than drawn at a length somebody liked.
        if let Some(cap) = w.cap_feet {
            let over_hang = (cap - face) / 2.0;
            let leg = w
                .girth_inches
                .map(|g| ((g - cap * 12.0) / 2.0 / 12.0).max(0.0))
                .unwrap_or(0.0);
            let outer = -over_hang;
            let inner = face + over_hang;
            d.line(
                vec![
                    Point::new(outer, top + leg),
                    Point::new(outer, top),
                    Point::new(inner, top),
                    Point::new(inner, top + leg),
                ],
                "coping",
                false,
                false,
            );
            d.text(
                Point::new(inner + 0.4, top - 0.45),
                match w.girth_inches {
                    Some(g) => format!("coping cap · {} girth", inches_written(g)),
                    None => "coping cap · no girth stated".to_string(),
                },
                "label",
                "start",
            );
            // The continuous cleat under the outer leg, which is what holds it
            // down (NRCA's parapet figures draw one at every coping).
            if leg > 0.0 {
                d.line(
                    vec![
                        Point::new(outer + 0.05, top + leg * 0.9),
                        Point::new(outer + 0.05, top + leg * 0.3),
                        Point::new(outer + 0.05 + leg * 0.6, top + leg * 0.3),
                    ],
                    "cleat",
                    false,
                    false,
                );
            }
        }

        // The membrane, turned up the wall. How far up is typed as `FLASHUP`
        // until the wall solver works it out; where nobody typed one, it is drawn
        // at the least NRCA will accept and the drawing says which.
        let (up_inches, stated) = match w.flash_up_inches {
            Some(up) => (up, true),
            None => (MIN_FLASHING_INCHES, false),
        };
        let top_of_flashing = roof_at_wall - up_inches / 12.0;
        d.line(
            vec![
                Point::new(face + 0.02, roof_at_wall),
                Point::new(face + 0.02, top_of_flashing),
            ],
            "flashing",
            false,
            false,
        );
        d.line(
            vec![
                Point::new(face + 0.02, top_of_flashing),
                Point::new(face + 0.6, top_of_flashing),
            ],
            "termbar",
            false,
            false,
        );
        // Two short words rather than one long one: a label that reaches half way
        // across a drawing lands on whatever is there.
        d.text(
            Point::new(face + 0.8, top_of_flashing + 0.55),
            if stated {
                format!("wall flashing {} up", inches_written(up_inches))
            } else {
                format!(
                    "wall flashing · nobody states how far it runs up, so it is drawn at NRCA's least, {}",
                    inches_written(up_inches)
                )
            },
            "label",
            "start",
        );
        d.text(
            Point::new(face + 0.8, top_of_flashing - 0.12),
            "termination bar and sealant".to_string(),
            "label",
            "start",
        );
    } else {
        d.text(
            Point::new(0.0, roof_at_wall - 1.0),
            match cut.at_edge {
                true => "the roof's own edge · no wall on the job flashes against it here".to_string(),
                false => "the ridge this ground shares with the next drain".to_string(),
            },
            "label",
            "start",
        );
    }

    // ── the drain at the low end ────────────────────────────────────────────
    d.line(
        vec![Point::new(run, 0.0), Point::new(run, LEADER_FEET)],
        "drain",
        false,
        false,
    );
    d.line(
        vec![
            Point::new(run - 0.5, -start / 12.0),
            Point::new(run + 0.5, -start / 12.0),
        ],
        "drain",
        false,
        false,
    );
    d.text(
        Point::new(run - 0.4, LEADER_FEET + 0.5),
        match &drain_named {
            Some(_) => format!("Drain {drain_number} · drawn as a symbol; nothing states the bowl"),
            None => "the low point · nothing counted a drain here".to_string(),
        },
        "label",
        "end",
    );

    // ── the measures ────────────────────────────────────────────────────────
    d.dimension(
        Point::new(0.0, 0.0),
        Point::new(run, 0.0),
        LEADER_FEET + 2.5,
        format!("{} high point to drain", feet_written(run)),
        true,
    );
    // The two ends of the fall, each dimensioned on the side with room: the
    // drain's outboard of the drain, the thick end inboard of the wall.
    d.dimension(
        Point::new(run, 0.0),
        Point::new(run, -start / 12.0),
        1.2,
        format!("{} at the drain", inches_written(start)),
        false,
    );
    // Measured AT the far end, where the layout's run ends and the traced line
    // is, and drawn inboard of it, which is the side with room on it. The offset
    // moves the line and not the two points it is between.
    d.dimension(
        Point::new(0.0, 0.0),
        Point::new(0.0, -cut.depth_at(0.0) / 12.0),
        1.8,
        format!(
            "{} at the {}",
            inches_written(cut.depth_at(0.0)),
            if cut.at_edge { "wall" } else { "ridge" }
        ),
        true,
    );

    // ── the key, bottom to top, in the words the job uses for each course ───
    d.key.push(match deck_inches {
        Some(deep) => format!("existing deck · {}", inches_written(deep)),
        None => "existing deck · no thickness stated".to_string(),
    });
    if let Some(bill) = &field.bill {
        d.key.push(format!(
            "flat board {} · {} courses at the deepest",
            inches_written(bill.fill_step_inches),
            bill.courses_max.saturating_sub(1)
        ));
        d.key.push(format!(
            "{} · {} in 12 · {}",
            bill.product,
            inches_written(bill.slope_inches_per_foot).trim_end_matches('"'),
            bill.by_letter.keys().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    for layer in &layers {
        d.key.push(format!(
            "{} · {}",
            layer.description,
            inches_written(layer.thickness.unwrap_or(0.0))
        ));
    }
    if !layers.is_empty() {
        d.key.push("the layers over the boards are what each line says it is".to_string());
    }
    let at_key = run * 0.72;
    d.key_from = Some(Point::new(at_key, -cut.depth_at(at_key) / 12.0 - over / 12.0));
    // In the open ground over the roof, on the far half of the cut: beside the
    // wall it lands on top of the words on the flashing, which is the collision
    // a five-line key is most likely to make.
    let key_top = match &wall {
        Some(w) => -w.height_feet + 0.35,
        None => -cut.depth_at(0.0) / 12.0 - 1.9,
    };
    d.key_at = Some(Point::new(run * 0.62, key_top));

    d.measure();
    d
}

/// The boards this cut passes through, in order from the far end to the drain.
fn boards_along<'a>(cut: &'a Cut, panel_feet: f64) -> Vec<&'a Board> {
    let mut out: Vec<(f64, &Board)> = Vec::new();
    for board in &cut.region.boards {
        let across = (board.centre.x - cut.drain.x) * -cut.away.y
            + (board.centre.y - cut.drain.y) * cut.away.x;
        if across.abs() > panel_feet / 4.0 {
            continue;
        }
        let along = (board.centre.x - cut.drain.x) * cut.away.x
            + (board.centre.y - cut.drain.y) * cut.away.y;
        if along < -panel_feet / 4.0 || along > cut.run_feet + panel_feet {
            continue;
        }
        out.push((along, board));
    }
    out.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    out.into_iter().map(|(_, b)| b).collect()
}

/// The top of the tapered package along the cut, from the far end to the drain.
///
/// Read at every board edge and at the ends, which is enough to be exact: the
/// surface is planar over each cell and folds only where the governing plane
/// changes hands or the cap takes over, and both of those land on a cell edge on
/// a grid anchored at the drain.
fn surface_line(cut: &Cut, panel_feet: f64) -> Vec<Point> {
    let mut at: Vec<f64> = vec![0.0, cut.run_feet];
    let step = if panel_feet > 0.0 { panel_feet } else { 4.0 };
    let mut x = cut.run_feet - step / 2.0;
    while x > 0.0 {
        at.push(x);
        x -= step;
    }
    at.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    at.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    at.into_iter().map(|x| Point::new(x, -cut.depth_at(x) / 12.0)).collect()
}

/// A wall the roof flashes against, as far as a section needs it.
struct Wall {
    id: String,
    name: String,
    height_feet: f64,
    thick_feet: Option<f64>,
    cap_feet: Option<f64>,
    girth_inches: Option<f64>,
    flash_up_inches: Option<f64>,
}

/// The wall standing at a place on the plan, or none.
///
/// A traced run passes through the point, within a foot of it — which is the
/// tolerance a traced line off a sheet deserves and is well inside the distance
/// between two walls of a building.
fn wall_at(document: &Document, at: Point) -> Option<Wall> {
    const NEAR_FEET: f64 = 1.0;
    let scales = document.scales();
    for c in &document.conditions {
        if c.hidden || c.kind != "line" || c.has_role("ridge") {
            continue;
        }
        let height = c.property("H").unwrap_or(0.0);
        if height <= 0.0 {
            continue;
        }
        for trace in &c.traces {
            let ring = document.feet(trace, &scales);
            for pair in ring.windows(2) {
                if away_from_segment(at, pair[0], pair[1]) <= NEAR_FEET {
                    return Some(Wall {
                        id: c.id.clone(),
                        name: if c.name.is_empty() { "the wall".to_string() } else { c.name.clone() },
                        height_feet: height,
                        thick_feet: c.property("WALL").map(|w| w / 12.0),
                        cap_feet: c.property("W"),
                        girth_inches: c.property("STRETCHOUT"),
                        flash_up_inches: c.property("FLASHUP"),
                    });
                }
            }
        }
    }
    None
}

/// How far a point is from a segment, in feet.
fn away_from_segment(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = dx * dx + dy * dy;
    if len < 1e-12 {
        return p.distance(a);
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len).clamp(0.0, 1.0);
    p.distance(Point::new(a.x + dx * t, a.y + dy * t))
}

/// Which condition counted the drain at a place, or none.
fn drain_at(document: &Document, at: Point) -> Option<String> {
    let scales = document.scales();
    for c in &document.conditions {
        if c.hidden || !c.has_role("drain") {
            continue;
        }
        for trace in &c.traces {
            for p in document.feet(trace, &scales) {
                if p.distance(at) < 1e-6 {
                    return Some(c.id.clone());
                }
            }
        }
    }
    None
}

/// Which drain this is, by its place in the order the job counted them.
fn which_drain(taper: &SolvedTaper, at: Point) -> usize {
    taper.drains.iter().position(|d| d.distance(at) < 1e-6).map(|i| i + 1).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_length_is_written_the_way_a_roofer_writes_it() {
        assert_eq!(inches_written(0.5), "1/2\"");
        assert_eq!(inches_written(6.0), "6\"");
        assert_eq!(inches_written(4.5), "4 1/2\"");
        assert_eq!(inches_written(0.25), "1/4\"");
        assert_eq!(feet_written(22.0), "22'-0\"");
        assert_eq!(feet_written(4.5), "4'-6\"");
        assert_eq!(feet_written(3.5), "3'-6\"");
        // Eleven and fifteen sixteenths of an inch is a foot on a drawing, and a
        // drawing that says 21'-12" is one nobody trusts with the next number.
        assert_eq!(feet_written(21.9999), "22'-0\"");
    }

    #[test]
    fn north_is_up_the_sheet() {
        assert_eq!(compass(Point::new(1.0, 0.0)), "east");
        assert_eq!(compass(Point::new(-1.0, 0.0)), "west");
        assert_eq!(compass(Point::new(0.0, 1.0)), "south");
        assert_eq!(compass(Point::new(0.0, -1.0)), "north");
    }
}
