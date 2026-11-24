//! Where each piece of the outside trim lies (call 184, step 1, the model's half): fascia and soffit on an eave, rake fascia and
//! rake soffit on a rake, corner boards on a wall, casing round a window or a door. The window draws what this lists and works
//! nothing out.
//!
//! Ours. The rules, each said once:
//!
//! * **A piece exists only where the estimate prices it.** A thing has a piece of a kind only where a cost line on it prices it:
//!   a fascia line (its quantity names `true_length`, and its description says fascia) on an eave; a soffit line (names `overhang`,
//!   says soffit) on an eave; the same on a rake; a line naming `corner_board_length` on a wall; one naming `casing_length` on a
//!   window or a door. A labor line draws nothing (it is work, not a board). One piece is drawn for each of a thing's pieces, even
//!   where two lines price it (the second is said).
//! * **One arithmetic.** A piece's length is worked from the same positions the figure is (`trim::figures`, the cost line): the
//!   pieces of a kind on a thing add up to the priced quantity of its line, in LF, or in SF for a soffit (length by width, the
//!   width being the overhang). Where they would not (a person's formula is more than the figure alone) the pieces are not drawn
//!   and the list says so, never silently.
//! * **Heights are the model's.** A roof holds one eave height, in feet above the job's datum (the same datum as a wall's base and
//!   top), and a plane rises from its eave line at its pitch: so a roof edge's height is the height of the roof's SURFACE at the edge
//!   line, `eave_height + distance from the plane's eave line * pitch / 12`. The model holds no fascia depth, rafter or heel.
//! * **Sizes.** Where the line's own words name a size ("1x6", "5/4x4": the softwood standard's actual sizes) the piece is that
//!   size and says so. Where they name none the piece is drawn at a common size, a named constant here, Assumed, and said ("drawn at
//!   3 1/2 in: the line names no size"); that size is for drawing only and is in no quantity or price. A soffit's width is its
//!   overhang, which its line prices.
//! * **Where a piece lies.** `from` and `to` are the middle of the piece's back (the side against what it is fixed to) at its two
//!   ends; `face` is the way its visible face looks; `width` is across the face; `thickness` is out from what it is fixed to.
//!   A fascia's back is on the roof's edge line, the board hanging from the edge down; a rake fascia the same, plumb. A soffit is
//!   level across and its face is flush with the fascia's bottom edge; it spans from the roof's edge in to the wall's outside face.
//!   A corner board's back is on its wall's outside face, flush with the corner of the outside faces (the corner is where the two
//!   outside face lines meet), from the wall's foot to its top; a board on each face. Casing lies on the wall's outside face: the
//!   head over the opening, the sill under it, a side to each side; each is the priced length (the head is the width of the
//!   opening and the sides its height), none across a door's foot.
//! * **Status.** A piece is as sure as the worst of what its position rests on and the line's quantity.

use std::collections::HashMap;

use super::{direction_out, end_of, overhangs, path_number, walls_of, Ends, WallRec};
use crate::building::{self, Opening, Tree};
use crate::components::{Class, Cost, Kind as Thing, LineQuantities, Measure, Node, Run, P2};
use crate::job::Job;
use crate::roof::{self, SolidPlane};
use crate::value::{Derived, Status};

const IN: f64 = 1.0 / 12.0;
/// Drawing sizes, used only where a line names none (feet).
pub const FASCIA_WIDTH_FT: f64 = 5.5 * IN;
pub const FASCIA_THICKNESS_FT: f64 = 0.75 * IN;
pub const SOFFIT_THICKNESS_FT: f64 = 0.5 * IN;
pub const CORNER_BOARD_WIDTH_FT: f64 = 3.5 * IN;
pub const CORNER_BOARD_THICKNESS_FT: f64 = 0.75 * IN;
pub const CASING_WIDTH_FT: f64 = 3.5 * IN;
pub const CASING_THICKNESS_FT: f64 = 0.75 * IN;

/// What a piece is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Fascia,
    RakeFascia,
    Soffit,
    RakeSoffit,
    CornerBoard,
    Casing,
}

impl Kind {
    pub const ALL: [Kind; 6] = [Kind::Fascia, Kind::RakeFascia, Kind::Soffit, Kind::RakeSoffit, Kind::CornerBoard, Kind::Casing];
    /// What a builder calls it.
    pub fn words(self) -> &'static str {
        match self {
            Kind::Fascia => "fascia",
            Kind::RakeFascia => "rake fascia",
            Kind::Soffit => "soffit",
            Kind::RakeSoffit => "rake soffit",
            Kind::CornerBoard => "corner boards",
            Kind::Casing => "casing",
        }
    }
    /// What it is priced in.
    pub fn unit(self) -> &'static str {
        match self {
            Kind::Soffit | Kind::RakeSoffit => "SF",
            _ => "LF",
        }
    }
    fn index(self) -> usize {
        Kind::ALL.iter().position(|k| *k == self).unwrap_or(0)
    }
}

/// One piece of trim and where it lies. Feet: east, north, up.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub kind: Kind,
    /// The path of the roof edge, wall or opening it trims.
    pub of: String,
    /// The cost line that prices it, as the estimate names it.
    pub line: String,
    /// The middle of the piece's back at one end.
    pub from: [f64; 3],
    /// The same at the other end.
    pub to: [f64; 3],
    /// A unit direction: the way the piece's face looks (out from the wall; down for a soffit).
    pub face: [f64; 3],
    /// Across the piece on its face (a soffit's is the overhang).
    pub width: f64,
    /// Out from what it is fixed to.
    pub thickness: f64,
    /// Where width and thickness came from, in words.
    pub sized_from: String,
    pub status: Status,
}

/// Every piece, what is left out and why, and what each kind's lines price in all.
#[derive(Clone, Debug, Default)]
pub struct Laid {
    pub pieces: Vec<Piece>,
    /// A line that prices a piece that is not drawn, and why (one sentence each).
    pub left_out: Vec<String>,
    /// For each kind, the sum of the quantities of the lines that price it (only the lines that have a figure), or None where no line
    /// prices that kind.
    pub priced: Vec<(Kind, Option<f64>)>,
    /// For each kind, how many of the lines that price it have no figure (call 191 item 11): its total is then no figure, not nothing.
    pub no_figure: Vec<(Kind, usize)>,
}

impl Laid {
    pub fn priced_of(&self, kind: Kind) -> Option<f64> {
        self.priced.iter().find(|(k, _)| *k == kind).and_then(|(_, q)| *q)
    }
    /// How many of the lines that price this kind have no figure.
    pub fn no_figure_of(&self, kind: Kind) -> usize {
        self.no_figure.iter().find(|(k, _)| *k == kind).map_or(0, |(_, n)| *n)
    }
}

/// Every piece of outside trim the job's lines price, where each lies.
pub fn pieces(job: &mut Job) -> Vec<Piece> {
    lay(job).pieces
}

// ------------------------------------------------------------------------------------------------ sizes

struct Size {
    width: f64,
    thickness: f64,
    said: String,
}

/// "5 1/2 in", "3/4 in", "9 in": to the sixteenth.
pub fn inches_said(ft: f64) -> String {
    let sixteenths = (ft * 12.0 * 16.0).round() as i64;
    let (whole, rem) = (sixteenths / 16, sixteenths % 16);
    let g = {
        let (mut a, mut b) = (rem.max(1), 16);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    };
    match (whole, rem) {
        (w, 0) => format!("{w} in"),
        (0, r) => format!("{}/{} in", r / g, 16 / g),
        (w, r) => format!("{w} {}/{} in", r / g, 16 / g),
    }
}

/// A nominal softwood size in the words of a line ("1x6", "5/4x4"): the token as written, and its actual thickness and width in
/// inches (the American softwood standard's finished sizes).
fn named_size(desc: &str) -> Option<(String, f64, f64)> {
    let s: Vec<char> = desc.to_lowercase().chars().collect();
    for (i, &c) in s.iter().enumerate() {
        if c != 'x' && c != '\u{d7}' {
            continue;
        }
        let mut a = i;
        while a > 0 && (s[a - 1].is_ascii_digit() || s[a - 1] == '/') {
            a -= 1;
        }
        let mut b = i + 1;
        while b < s.len() && s[b].is_ascii_digit() {
            b += 1;
        }
        if a == i || b == i + 1 || (a > 0 && s[a - 1].is_alphabetic()) {
            continue;
        }
        let (left, right): (String, String) = (s[a..i].iter().collect(), s[i + 1..b].iter().collect());
        let thickness = match left.as_str() {
            "1" => 0.75,
            "5/4" => 1.0,
            "2" => 1.5,
            _ => continue,
        };
        let width = match right.as_str() {
            "3" => 2.5,
            "4" => 3.5,
            "5" => 4.5,
            "6" => 5.5,
            "8" => 7.25,
            "10" => 9.25,
            "12" => 11.25,
            _ => continue,
        };
        return Some((format!("{left}x{right}"), thickness, width));
    }
    None
}

fn size_for(kind: Kind, desc: Option<&str>, overhang: Option<f64>) -> Size {
    let (dw, dt) = match kind {
        Kind::Fascia | Kind::RakeFascia => (FASCIA_WIDTH_FT, FASCIA_THICKNESS_FT),
        Kind::Soffit | Kind::RakeSoffit => (overhang.unwrap_or(0.0), SOFFIT_THICKNESS_FT),
        Kind::CornerBoard => (CORNER_BOARD_WIDTH_FT, CORNER_BOARD_THICKNESS_FT),
        Kind::Casing => (CASING_WIDTH_FT, CASING_THICKNESS_FT),
    };
    let soffit = matches!(kind, Kind::Soffit | Kind::RakeSoffit);
    match desc.and_then(named_size) {
        Some((token, t_in, w_in)) => {
            let (t, w) = (t_in * IN, w_in * IN);
            let said = if soffit {
                format!("its width is the overhang, {}, which its line prices; its thickness is {}, from the line's own words \"{token}\" (the softwood standard's actual size)", inches_said(dw), inches_said(t))
            } else {
                format!("from the line's own words \"{token}\": {} by {} (the softwood standard's actual size)", inches_said(t), inches_said(w))
            };
            Size { width: if soffit { dw } else { w }, thickness: t, said }
        }
        None => {
            let said = if soffit {
                format!("its width is the overhang, {}, which its line prices; its thickness is drawn at {}: the line names no size, and a drawing size is in no quantity or price", inches_said(dw), inches_said(dt))
            } else {
                format!("drawn at {} by {}: the line names no size, and a drawing size is in no quantity or price", inches_said(dw), inches_said(dt))
            };
            Size { width: dw, thickness: dt, said }
        }
    }
}

// ------------------------------------------------------------------------------------------------ the lines

/// The cost lines on a thing that price a piece: material or other work that is bought, never labor, switched on, whose quantity
/// names the figure and (where a word is given) whose description says it.
fn priced_lines(job: &mut Job, path: &str, figure: &str, word: Option<&str>) -> Vec<(String, Derived)> {
    let Some(e) = job.find(path) else { return Vec::new() };
    let Some(cost) = job.world.get::<Cost>(e) else { return Vec::new() };
    let quantities = job.world.get::<LineQuantities>(e);
    cost.lines
        .iter()
        .filter(|l| !l.off && l.class != Some(Class::Labor))
        .filter(|l| word.is_none_or(|w| l.description.to_lowercase().contains(w)))
        .filter(|l| crate::formula::parse(&l.quantity).is_ok_and(|x| crate::formula::names(&x).iter().any(|n| n == figure)))
        .map(|l| {
            let q = quantities
                .and_then(|q| q.0.get(&l.description))
                .map(|f| f.quantity.clone())
                .unwrap_or_else(|| Derived::refused("", "", &[], "its quantity is not worked out yet"));
            (l.description.clone(), q)
        })
        .collect()
}

/// The cost lines on a thing whose description says this kind of trim and whose quantity does NOT name its figure (a plan length, a
/// typed number): they price the trim and no piece is drawn from them (call 191 item 10). Never labor, never switched off.
fn other_lines(job: &mut Job, path: &str, figure: &str, word: &str) -> Vec<(String, String, Derived)> {
    let Some(e) = job.find(path) else { return Vec::new() };
    let Some(cost) = job.world.get::<Cost>(e) else { return Vec::new() };
    let quantities = job.world.get::<LineQuantities>(e);
    cost.lines
        .iter()
        .filter(|l| !l.off && l.class != Some(Class::Labor))
        .filter(|l| l.description.to_lowercase().contains(word))
        .filter(|l| !crate::formula::parse(&l.quantity).is_ok_and(|x| crate::formula::names(&x).iter().any(|n| n == figure)))
        .map(|l| {
            let q = quantities.and_then(|q| q.0.get(&l.description)).map(|f| f.quantity.clone()).unwrap_or_else(|| Derived::refused("", "", &[], "its quantity is not worked out yet"));
            (l.description.clone(), l.quantity.clone(), q)
        })
        .collect()
}

fn unit_vec(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-12);
    [d[0] / n, d[1] / n, d[2] / n]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dist3(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

// ------------------------------------------------------------------------------------------------ the whole

/// A line's quantity set against the kind's total, or counted as having no figure.
fn price_in(totals: &mut [Option<f64>; 6], no_fig: &mut [usize; 6], kind: Kind, q: &Derived) {
    match q.worked() {
        Some(v) => *totals[kind.index()].get_or_insert(0.0) += v,
        None => no_fig[kind.index()] += 1,
    }
}

/// The lines that price a kind of trim on a figure that is not the piece's length: said, and their quantity set against the kind's total
/// where it is a length (or an area, for a soffit) of the kind's own unit (call 191 item 10).
#[allow(clippy::too_many_arguments)]
fn say_other(out: &mut Laid, totals: &mut [Option<f64>; 6], no_fig: &mut [usize; 6], called: &str, kind: Kind, figure: &str, lines: &[(String, String, Derived)]) {
    for (desc, quantity, q) in lines {
        match q.worked() {
            Some(v) if q.unit == kind.unit() => {
                *totals[kind.index()].get_or_insert(0.0) += v;
                out.left_out.push(format!("{called}: \"{desc}\" prices {} {} by \"{quantity}\", which is not {figure}: no piece of its {} is drawn from it, and {} {} of what it prices is not drawn", crate::view::figure2(v), kind.unit(), kind.words(), crate::view::figure2(v), kind.unit()));
            }
            Some(v) => out.left_out.push(format!("{called}: \"{desc}\" prices {} {} by \"{quantity}\", which is not {figure} and not a figure in {}: no piece of its {} is drawn from it and it is set against no total", crate::view::figure2(v), if q.unit.is_empty() { "of something" } else { q.unit.as_str() }, kind.unit(), kind.words())),
            None => {
                no_fig[kind.index()] += 1;
                out.left_out.push(format!("{called}: \"{desc}\" has no figure (\"{quantity}\", which is not {figure}), so its {} is not drawn", kind.words()));
            }
        }
    }
}

pub fn lay(job: &mut Job) -> Laid {
    let mut out = Laid::default();
    let mut totals: [Option<f64>; 6] = [None; 6];
    let mut no_fig = [0usize; 6];
    let mut things: Vec<(String, String, Thing)> = {
        let mut q = job.world.query::<(&Node, &Thing)>();
        q.iter(&job.world).filter(|(_, k)| matches!(**k, Thing::Eave | Thing::Rake | Thing::Wall | Thing::Window | Thing::Door)).map(|(n, k)| (n.path.clone(), n.name.clone(), *k)).collect()
    };
    things.sort_by_key(|(p, ..)| path_number(p));
    let (planes, _) = roof::solid_planes(job);
    let over = overhangs(&mut job.world, None);
    let tree = Tree::of(&mut job.world);
    let (walls, not_placed) = walls_of(&mut job.world, &tree);
    let storeys: std::collections::BTreeSet<String> = walls.iter().map(|w| w.storey.clone()).collect();
    let levels: HashMap<String, Vec<&WallRec>> = storeys.iter().map(|s| (s.clone(), walls.iter().filter(|w| &w.storey == s).collect())).collect();
    let ends: HashMap<String, Ends> = levels.iter().map(|(s, l)| (s.clone(), Ends::of(l))).collect();
    let mut enclosed: HashMap<String, Result<building::Enclosed, String>> = HashMap::new();

    for (path, name, thing) in things {
        let called = format!("{name} [{path}]");
        match thing {
            Thing::Eave | Thing::Rake => {
                let (fascia, soffit) = if thing == Thing::Eave { (Kind::Fascia, Kind::Soffit) } else { (Kind::RakeFascia, Kind::RakeSoffit) };
                let fascia_line = priced_lines(job, &path, "true_length", Some("fascia"));
                let soffit_line = priced_lines(job, &path, "overhang", Some("soffit"));
                let fascia_other = other_lines(job, &path, "true_length", "fascia");
                let soffit_other = other_lines(job, &path, "overhang", "soffit");
                if fascia_line.is_empty() && soffit_line.is_empty() && fascia_other.is_empty() && soffit_other.is_empty() {
                    continue;
                }
                say_other(&mut out, &mut totals, &mut no_fig, &called, fascia, "its true length", &fascia_other);
                say_other(&mut out, &mut totals, &mut no_fig, &called, soffit, "its overhang", &soffit_other);
                let edge = edge_of(job, &path, &planes);
                // How deep the fascia hangs: the soffit's face is flush with its lower edge.
                let fascia_width = size_for(fascia, fascia_line.first().map(|l| l.0.as_str()), None).width;
                for (kind, lines) in [(fascia, &fascia_line), (soffit, &soffit_line)] {
                    let Some((desc, q)) = lines.first() else { continue };
                    if lines.len() > 1 {
                        out.left_out.push(format!("{called}: {} lines price its {} ({}); it is drawn once, from \"{desc}\"", lines.len(), kind.words(), lines.iter().map(|l| format!("\"{}\"", l.0)).collect::<Vec<_>>().join(", ")));
                    }
                    price_in(&mut totals, &mut no_fig, kind, q);
                    for (_, extra) in lines.iter().skip(1) {
                        price_in(&mut totals, &mut no_fig, kind, extra);
                    }
                    let Some(want) = q.worked() else {
                        let why = if !q.note.is_empty() { q.note.as_str() } else { over.get(&path).filter(|_| kind == soffit).map(|o| o.note.as_str()).filter(|n| !n.is_empty()).unwrap_or("a value it is worked out from is not known") };
                        out.left_out.push(format!("{called}: \"{desc}\" has no figure, so its {} is not drawn ({why})", kind.words()));
                        continue;
                    };
                    let edge = match &edge {
                        Ok(e) => e,
                        Err(why) => {
                            out.left_out.push(format!("{called}: \"{desc}\": its {} is not drawn ({why})", kind.words()));
                            continue;
                        }
                    };
                    let piece = if kind == fascia {
                        let size = size_for(kind, Some(desc), None);
                        Piece {
                            kind,
                            of: path.clone(),
                            line: desc.clone(),
                            from: [edge.a[0], edge.a[1], edge.a[2] - size.width / 2.0],
                            to: [edge.b[0], edge.b[1], edge.b[2] - size.width / 2.0],
                            face: [edge.out[0], edge.out[1], 0.0],
                            width: size.width,
                            thickness: size.thickness,
                            sized_from: size.said,
                            status: q.status.max(edge.status),
                        }
                    } else {
                        let Some(o) = over.get(&path) else { continue };
                        let Some(overhang) = o.worked() else {
                            out.left_out.push(format!("{called}: \"{desc}\": its {} is not drawn ({})", kind.words(), if o.note.is_empty() { "its overhang is not known" } else { o.note.as_str() }));
                            continue;
                        };
                        let size = size_for(kind, Some(desc), Some(overhang));
                        // The back's height: the face flush with the fascia's bottom edge, the board's thickness up from it.
                        let up = |z: f64| z - fascia_width + size.thickness;
                        let (inward, half) = ([-edge.out[0], -edge.out[1]], overhang / 2.0);
                        let from = [edge.a[0] + inward[0] * half, edge.a[1] + inward[1] * half, up(edge.a[2])];
                        let to = [edge.b[0] + inward[0] * half, edge.b[1] + inward[1] * half, up(edge.b[2])];
                        let mut face = cross([inward[0], inward[1], 0.0], unit_vec(from, to));
                        if face[2] > 0.0 {
                            face = [-face[0], -face[1], -face[2]];
                        }
                        let n = (face[0] * face[0] + face[1] * face[1] + face[2] * face[2]).sqrt().max(1e-12);
                        Piece {
                            kind,
                            of: path.clone(),
                            line: desc.clone(),
                            from,
                            to,
                            face: [face[0] / n, face[1] / n, face[2] / n],
                            width: size.width,
                            thickness: size.thickness,
                            sized_from: size.said,
                            status: q.status.max(edge.status).max(o.status),
                        }
                    };
                    let got = dist3(piece.from, piece.to) * if matches!(kind, Kind::Soffit | Kind::RakeSoffit) { piece.width } else { 1.0 };
                    if (got - want).abs() > 1e-3 {
                        out.left_out.push(format!("{called}: \"{desc}\" prices {} {} and its {} lies {} {}: its quantity is not the length alone, so it is not drawn", crate::view::num(want), kind.unit(), kind.words(), crate::view::num(got), kind.unit()));
                        continue;
                    }
                    out.pieces.push(piece);
                }
            }
            Thing::Wall => {
                let lines = priced_lines(job, &path, "corner_board_length", None);
                let other = other_lines(job, &path, "corner_board_length", "corner board");
                say_other(&mut out, &mut totals, &mut no_fig, &called, Kind::CornerBoard, "its corner board length", &other);
                let Some((desc, q)) = lines.first() else { continue };
                if lines.len() > 1 {
                    out.left_out.push(format!("{called}: {} lines price its corner boards; they are drawn once, from \"{desc}\"", lines.len()));
                }
                for (_, l) in &lines {
                    price_in(&mut totals, &mut no_fig, Kind::CornerBoard, l);
                }
                let Some(want) = q.worked() else {
                    out.left_out.push(format!("{called}: \"{desc}\" has no figure, so its corner boards are not drawn ({})", if q.note.is_empty() { "a value it is worked out from is not known" } else { q.note.as_str() }));
                    continue;
                };
                let Some(i) = walls.iter().position(|w| w.path == path) else {
                    out.left_out.push(format!("{called}: \"{desc}\": its corner boards are not drawn ({})", not_placed.get(&path).cloned().unwrap_or_else(|| "the wall cannot be placed".into())));
                    continue;
                };
                let w = &walls[i];
                let height = match &w.height {
                    Ok(h) => *h,
                    Err(why) => {
                        out.left_out.push(format!("{called}: \"{desc}\": its corner boards are not drawn ({why})"));
                        continue;
                    }
                };
                let level = &levels[&w.storey];
                let at = level.iter().position(|x| x.path == w.path).unwrap_or(0);
                let size = size_for(Kind::CornerBoard, Some(desc), None);
                let mut mine = Vec::new();
                for e in 0..2 {
                    match end_of(&ends[&w.storey], level, at, e) {
                        Ok((true, _)) => {
                            let (j, ej) = ends[&w.storey].meets[at][e][0];
                            let other = level[j];
                            let u = direction_out(w, e);
                            let v = direction_out(other, ej);
                            let outward = |own: P2, against: P2| {
                                let left = [-own[1], own[0]];
                                if against[0] * left[0] + against[1] * left[1] > 0.0 {
                                    [-left[0], -left[1]]
                                } else {
                                    left
                                }
                            };
                            let (nw, nj) = (outward(u, v), outward(v, u));
                            let (pw, pj) = (outside_point(w, nw), outside_point(other, nj));
                            let denom = u[0] * v[1] - u[1] * v[0];
                            if denom.abs() < 1e-9 {
                                continue;
                            }
                            let s = ((pj[0] - pw[0]) * v[1] - (pj[1] - pw[1]) * v[0]) / denom;
                            let corner = [pw[0] + u[0] * s, pw[1] + u[1] * s];
                            let back = [corner[0] + u[0] * size.width / 2.0, corner[1] + u[1] * size.width / 2.0];
                            mine.push(Piece {
                                kind: Kind::CornerBoard,
                                of: path.clone(),
                                line: desc.clone(),
                                from: [back[0], back[1], w.base],
                                to: [back[0], back[1], w.base + height],
                                face: [nw[0], nw[1], 0.0],
                                width: size.width,
                                thickness: size.thickness,
                                sized_from: size.said.clone(),
                                status: q.status.max(w.status).max(w.height_status).max(other.status),
                            });
                        }
                        Ok(_) => {}
                        Err(why) => out.left_out.push(format!("{called}: \"{desc}\": a corner board is not drawn ({why})")),
                    }
                }
                let got: f64 = mine.iter().map(|p| dist3(p.from, p.to)).sum();
                if (got - want).abs() > 1e-3 {
                    out.left_out.push(format!("{called}: \"{desc}\" prices {} LF and its corner boards lie {} LF: its quantity is not the length alone, so they are not drawn", crate::view::num(want), crate::view::num(got)));
                    continue;
                }
                out.pieces.extend(mine);
            }
            Thing::Window | Thing::Door => {
                let lines = priced_lines(job, &path, "casing_length", None);
                let other = other_lines(job, &path, "casing_length", "casing");
                say_other(&mut out, &mut totals, &mut no_fig, &called, Kind::Casing, "its casing length", &other);
                let Some((desc, q)) = lines.first() else { continue };
                if lines.len() > 1 {
                    out.left_out.push(format!("{called}: {} lines price its casing; it is drawn once, from \"{desc}\"", lines.len()));
                }
                for (_, l) in &lines {
                    price_in(&mut totals, &mut no_fig, Kind::Casing, l);
                }
                let Some(want) = q.worked() else {
                    out.left_out.push(format!("{called}: \"{desc}\" has no figure, so its casing is not drawn ({})", if q.note.is_empty() { "a value it is worked out from is not known" } else { q.note.as_str() }));
                    continue;
                };
                let Some(e) = job.find(&path) else { continue };
                let (Some(opening), Some(measure)) = (job.world.get::<Opening>(e).cloned(), job.world.get::<Measure>(e).cloned()) else {
                    out.left_out.push(format!("{called}: \"{desc}\": its casing is not drawn (it has no opening or no size)"));
                    continue;
                };
                let parent = job.world.get::<Node>(e).and_then(|n| n.parent.clone()).unwrap_or_default();
                let Some(w) = walls.iter().find(|w| w.path == parent) else {
                    out.left_out.push(format!("{called}: \"{desc}\": its casing is not drawn (the wall it is in cannot be placed)"));
                    continue;
                };
                let (Some(wd), Some(ht)) = (measure.0.get("width"), measure.0.get("height")) else {
                    out.left_out.push(format!("{called}: \"{desc}\": its casing is not drawn (it has no size)"));
                    continue;
                };
                let (Some(width), Some(height), Some(at), Some(sill)) = (wd.worked(), ht.worked(), opening.offset.feet(), opening.sill.feet()) else {
                    out.left_out.push(format!("{called}: \"{desc}\": its casing is not drawn (its size or its place on the wall is not known)"));
                    continue;
                };
                let Ok(base) = w.height.as_ref().map(|_| w.base) else {
                    out.left_out.push(format!("{called}: \"{desc}\": its casing is not drawn (its wall has no height)"));
                    continue;
                };
                let encl = enclosed.entry(w.storey.clone()).or_insert_with(|| building::enclosed(job, &w.storey));
                let encl = match encl {
                    Ok(x) => x,
                    Err(why) => {
                        out.left_out.push(format!("{called}: \"{desc}\": its casing is not drawn ({why})"));
                        continue;
                    }
                };
                // Which face of the wall is outdoors: a place half a foot out from each face, at the wall's middle.
                let (d, mid) = wall_dir(w);
                let left = [-d[1], d[0]];
                let probe = |lat: f64| [mid[0] + left[0] * lat, mid[1] + left[1] * lat];
                let (left_out, right_out) = (!encl.indoors(probe(w.ext + 0.5)), !encl.indoors(probe(w.int - 0.5)));
                let (n_out, lat) = match (left_out, right_out) {
                    (true, false) => (left, w.ext),
                    (false, true) => ([-left[0], -left[1]], w.int),
                    (false, false) => {
                        out.left_out.push(format!("{called}: \"{desc}\": its exterior casing is not drawn (both faces of its wall are indoors)"));
                        continue;
                    }
                    (true, true) => {
                        out.left_out.push(format!("{called}: \"{desc}\": its exterior casing is not drawn (both faces of its wall are outdoors, so which is its outside is not told)"));
                        continue;
                    }
                };
                let size = size_for(Kind::Casing, Some(desc), None);
                let cw = size.width;
                let on = |s: f64, z: f64| {
                    let p = [w.line[0][0] + d[0] * s + left[0] * lat, w.line[0][1] + d[1] * s + left[1] * lat];
                    [p[0], p[1], z]
                };
                let (foot, head) = (base + sill, base + sill + height);
                let status = q.status.max(wd.status).max(ht.status).max(opening.offset.status).max(opening.sill.status).max(w.status).max(w.height_status);
                let mk = |from: [f64; 3], to: [f64; 3]| Piece {
                    kind: Kind::Casing,
                    of: path.clone(),
                    line: desc.clone(),
                    from,
                    to,
                    face: [n_out[0], n_out[1], 0.0],
                    width: size.width,
                    thickness: size.thickness,
                    sized_from: size.said.clone(),
                    status,
                };
                let mut mine = vec![mk(on(at - width / 2.0, head + cw / 2.0), on(at + width / 2.0, head + cw / 2.0))];
                if thing == Thing::Window {
                    mine.push(mk(on(at - width / 2.0, foot - cw / 2.0), on(at + width / 2.0, foot - cw / 2.0)));
                }
                mine.push(mk(on(at - width / 2.0 - cw / 2.0, foot), on(at - width / 2.0 - cw / 2.0, head)));
                mine.push(mk(on(at + width / 2.0 + cw / 2.0, foot), on(at + width / 2.0 + cw / 2.0, head)));
                let got: f64 = mine.iter().map(|p| dist3(p.from, p.to)).sum();
                if (got - want).abs() > 1e-3 {
                    out.left_out.push(format!("{called}: \"{desc}\" prices {} LF and its casing lies {} LF: its quantity is not the length alone, so it is not drawn", crate::view::num(want), crate::view::num(got)));
                    continue;
                }
                out.pieces.extend(mine);
            }
            _ => {}
        }
    }
    out.pieces.sort_by_key(|p| (p.kind.index(), path_number(&p.of)));
    out.priced = Kind::ALL.iter().map(|k| (*k, totals[k.index()])).collect();
    out.no_figure = Kind::ALL.iter().map(|k| (*k, no_fig[k.index()])).collect();
    out
}

// ------------------------------------------------------------------------------------------------ edges and walls

/// A roof edge placed: its ends with their heights, the way it looks out, and how sure.
struct Edge {
    a: [f64; 3],
    b: [f64; 3],
    /// The horizontal unit direction out of the roof.
    out: P2,
    status: Status,
}

fn edge_of(job: &mut Job, path: &str, planes: &[SolidPlane]) -> Result<Edge, String> {
    let e = job.find(path).ok_or("it is not in the job")?;
    let run = job.world.get::<Run>(e).cloned().ok_or("it has no line")?;
    let [p, q] = run.ends_ft().ok_or("its ends are not known")?;
    let plane = run.planes.first().and_then(|n| planes.iter().find(|s| &s.path == n)).ok_or("its roof plane has no outline, eave height or pitch to place it by")?;
    let (zp, zq) = (plane.height_at(p), plane.height_at(q));
    let thing = job.world.get::<Thing>(e).copied();
    // Eaves: west to east, then south to north. Rakes: from the eave up to the ridge.
    let flip = if thing == Some(Thing::Rake) && (zp - zq).abs() > 1e-9 { zp > zq } else { p[0] > q[0] + 1e-9 || ((p[0] - q[0]).abs() <= 1e-9 && p[1] > q[1]) };
    let (a, b) = if flip { ([q[0], q[1], zq], [p[0], p[1], zp]) } else { ([p[0], p[1], zp], [q[0], q[1], zq]) };
    let len = (b[0] - a[0]).hypot(b[1] - a[1]);
    if len < 1e-9 {
        return Err("it has no length on the plan".into());
    }
    let left = [-(b[1] - a[1]) / len, (b[0] - a[0]) / len];
    let probe = [(a[0] + b[0]) / 2.0 + left[0] * 0.05, (a[1] + b[1]) / 2.0 + left[1] * 0.05];
    let out = if crate::vif::inside(&plane.points, probe) { [-left[0], -left[1]] } else { left };
    Ok(Edge { a, b, out, status: run.ends.status.max(plane.status) })
}

fn wall_dir(w: &WallRec) -> (P2, P2) {
    let (dx, dy) = (w.line[1][0] - w.line[0][0], w.line[1][1] - w.line[0][1]);
    let long = dx.hypot(dy).max(1e-12);
    ([dx / long, dy / long], [(w.line[0][0] + w.line[1][0]) / 2.0, (w.line[0][1] + w.line[1][1]) / 2.0])
}

/// A point on the wall's face on the side `out` looks to, at the wall's start.
fn outside_point(w: &WallRec, out: P2) -> P2 {
    let (d, _) = wall_dir(w);
    let left = [-d[1], d[0]];
    let lat = if out[0] * left[0] + out[1] * left[1] > 0.0 { w.ext } else { w.int };
    [w.line[0][0] + left[0] * lat, w.line[0][1] + left[1] * lat]
}
