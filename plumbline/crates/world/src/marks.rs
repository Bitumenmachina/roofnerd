//! The measured marks of a marked-up PDF, as the job holds them (call 227).
//!
//! Two things are held. **The shapes**: when a PDF is added as a drawing its marks are kept with it (`MarkShape`), as the
//! page drew them, so that a window can show them over the sheet and catch their corners. And **the lines**: a mark
//! brought in (`bring_in`) is a line of the estimate, on the job itself, naming the mark, its page and its file in its
//! source. Such a line carries the figure the mark says, never a price, never a division, and is **Assumed**: a mark is
//! what someone drew and wrote, and is Known only when somebody measures it in the field.
//!
//! The marks are read by `plumbline-plans` (this crate reads no disk); the command line hands the figures here.
//!
//! The lines have no element of their own: an estimate's lines are on what is built, or on the job; a new kind of
//! element would have to be known to the window, and its heading is the command line's (the estimate prints the lines
//! of each file under the file's name, from their source).

use serde::{Deserialize, Serialize};

use crate::components::{Cost, Kind, Node, P2};
use crate::editing::all_or_nothing;
use crate::job::Job;
use crate::units::Len;
use crate::value::{Status, Val};

/// One mark as it was drawn on a page, kept with the drawing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkShape {
    /// `length`, `run of lengths`, `area`, `count` or `note`.
    pub kind: String,
    pub label: String,
    /// The mark's own name in the file, where it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Its points on the page, in the page's units from its lower-left corner; a count has none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<P2>,
    /// Where it is: its centre; a count: each symbol.
    pub places: Vec<P2>,
    /// The figure the mark says, in LF, SF or EA: Assumed, and naming the mark, the page and the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub says: Option<Val<f64>>,
    /// What its points work out to at the scale it carries, where they could be worked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worked: Option<f64>,
    /// Why it is not read, where it is not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unread: Option<String>,
}

pub const SHAPE_KINDS: [&str; 5] = ["length", "run of lengths", "area", "count", "note"];

pub fn check_shape(path: &str, m: &MarkShape) -> Result<(), String> {
    if !SHAPE_KINDS.contains(&m.kind.as_str()) {
        return Err(format!("{path}: a mark is a length, a run of lengths, an area, a count or a note, not {:?}", m.kind));
    }
    if m.label.trim().is_empty() || m.label.chars().any(char::is_control) {
        return Err(format!("{path}: a mark has a label with no control characters in it"));
    }
    let fine = |p: &P2| p.iter().all(|x| x.is_finite() && x.abs() < 1.0e7);
    if !m.points.iter().chain(&m.places).all(fine) {
        return Err(format!("{path}: mark {:?} has a point that is not a usable place on the page", m.label));
    }
    if let Some(s) = &m.says {
        s.check_basics(&format!("{path} mark {:?}", m.label))?;
        if !matches!(s.unit.as_str(), "LF" | "SF" | "EA") {
            return Err(format!("{path} mark {:?}: it says its figure in {}, which is not LF, SF or EA", m.label, s.unit));
        }
        if s.status == Status::Known {
            return Err(format!("{path} mark {:?}: a mark is never Known: it is Known when somebody has measured what it marks", m.label));
        }
    }
    Ok(())
}

/// Keep the marks of a page with its drawing, in the page's order. Nothing else of the drawing changes.
pub fn set_shapes(job: &mut Job, drawing: &str, shapes: Vec<MarkShape>) -> Result<(), String> {
    all_or_nothing(job, |job| {
        let e = job.find(drawing).ok_or_else(|| format!("no element {drawing}"))?;
        let mut d = job.world.get::<crate::plans::Drawing>(e).cloned().ok_or_else(|| format!("{drawing} is not a drawing"))?;
        d.marks = shapes;
        crate::plans::check(drawing, Kind::Drawing, &d)?;
        job.put(drawing, d)?;
        Ok(())
    })
}

// ---- The lines ----

/// A mark to bring in as a line.
#[derive(Clone, Debug, PartialEq)]
pub struct ToBring {
    /// What the mark is known by in its file (its own name, or what the reading made of its place and label).
    pub key: String,
    pub label: String,
    pub quantity: f64,
    /// LF, SF or EA.
    pub unit: &'static str,
    /// `length`, `run of lengths`, `area` or `count`, as the source says it.
    pub kind: String,
    /// What the mark's points work out to, where it is not what it says (said in the source of the line).
    pub points_say: Option<String>,
}

/// What bringing the marks in did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Brought {
    /// The line each new mark made: its description, quantity and unit.
    pub added: Vec<(String, f64, String)>,
    /// Marks the job already had, with the same figure.
    pub same: Vec<String>,
    /// Marks whose figure changed: the line's description, what it was, what it is, the unit.
    pub moved: Vec<(String, f64, f64, String)>,
    /// Marks whose figure changed in the file and whose line was left, each in a sentence and why.
    pub left_as_it_is: Vec<String>,
    /// Lines of marks the file no longer has: their descriptions. The lines are still there.
    pub gone: Vec<String>,
}

fn normal(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// What the source of a mark's line says: its key, page and file.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkSource {
    pub key: String,
    pub kind: String,
    pub page: usize,
    pub file: String,
}

/// The source a mark's line carries. A key never holds a double quote (`clean_key`).
pub fn source_text(key: &str, kind: &str, page: usize, file: &str, points_say: Option<&str>) -> String {
    // A count says it is a count of symbols (call 231): the file states no count, this program counted.
    if kind == "count" {
        let label = clean_key(key);
        let label = label.strip_prefix("count ").unwrap_or(&label).to_string();
        let n = points_say.and_then(|p| p.parse::<usize>().ok()).unwrap_or(0);
        return count_source_text(&label, n, page, file);
    }
    let mut s = format!("mark \"{}\" ({kind}), page {page} of {file}", clean_key(key));
    if let Some(p) = points_say {
        s.push_str(" -- ");
        s.push_str(p);
    }
    s
}

/// The source of a line that comes from a count of symbols: "4 symbols labelled Receptacle counted on page 1 of FILE".
pub fn count_source_text(label: &str, n: usize, page: usize, file: &str) -> String {
    format!("{n} symbol{} labelled {} counted on page {page} of {file}", if n == 1 { "" } else { "s" }, clean_key(label))
}

pub fn clean_key(key: &str) -> String {
    key.chars().map(|c| if c == '"' { '\'' } else if c.is_control() { ' ' } else { c }).collect::<String>().trim().to_string()
}

/// What a source says, if it is one a mark's line made.
pub fn parse_source(source: &str) -> Option<MarkSource> {
    if let Some(rest) = source.strip_prefix(|c: char| c.is_ascii_digit()).map(|_| source.trim_start_matches(|c: char| c.is_ascii_digit())) {
        let rest = rest.strip_prefix(" symbols labelled ").or_else(|| rest.strip_prefix(" symbol labelled "))?;
        let at = rest.rfind(" counted on page ")?;
        let (label, tail) = (&rest[..at], &rest[at + " counted on page ".len()..]);
        let (page, file) = tail.split_once(" of ")?;
        return Some(MarkSource { key: clean_key(&format!("count {label}")), kind: "count".into(), page: page.parse().ok()?, file: file.to_string() });
    }
    let rest = source.strip_prefix("mark \"")?;
    let (key, rest) = rest.split_once("\" (")?;
    let (kind, rest) = rest.split_once("), page ")?;
    let (page, rest) = rest.split_once(" of ")?;
    let file = rest.split(" -- ").next()?;
    Some(MarkSource { key: key.to_string(), kind: kind.to_string(), page: page.parse().ok()?, file: file.to_string() })
}

/// Every line of the job that a mark made: the line's description, its figure and the source it names.
pub fn lines_from_marks(job: &mut Job) -> Vec<(String, Val<Len>, MarkSource)> {
    let root = job.root();
    let Some(cost) = job.world.get::<Cost>(root) else { return Vec::new() };
    cost.lines.iter().filter_map(|l| l.inputs.get("measured").and_then(|m| parse_source(&m.source).map(|p| (l.description.clone(), m.clone(), p)))).collect()
}

/// The source a mark to bring in gives its line: a count says it is a count of symbols.
fn source_of(m: &ToBring, page: usize, file: &str) -> String {
    if m.kind == "count" {
        count_source_text(&m.label, m.quantity.round() as usize, page, file)
    } else {
        source_text(&m.key, &m.kind, page, file, m.points_say.as_deref())
    }
}

fn tidy(x: f64) -> f64 {
    (x * 1.0e6).round() / 1.0e6
}

/// Bring the marks of one page of one file into the job as lines on the job itself: unpriced, with no division, Assumed,
/// naming the mark, the page and the file. A mark the job already has (known by its key, its page and its file) is not
/// added again: with the same figure it is the same; with another it is moved and the figure it was is said; a line whose
/// figure somebody made Known is left as it is and said. A mark the file no longer has is named, and its line stays.
/// `in_the_file` is the key of every mark on the page, read or not: a mark that cannot be read now has not gone.
/// All of it or none.
pub fn bring_in(job: &mut Job, file: &str, page: usize, marks: &[ToBring], in_the_file: &[String]) -> Result<Brought, String> {
    all_or_nothing(job, |job| {
        let root = job.world.get::<Node>(job.root()).map(|n| n.path.clone()).ok_or("the job has no root")?;
        let mut out = Brought::default();
        let stem = std::path::Path::new(file).file_stem().and_then(|s| s.to_str()).unwrap_or(file).to_string();
        for m in marks {
            if !(m.quantity.is_finite() && m.quantity > 0.0) {
                return Err(format!("{}: a mark that says {} {} is no quantity", m.label, m.quantity, m.unit));
            }
            let q = tidy(m.quantity);
            let have = lines_from_marks(job).into_iter().find(|(_, _, s)| s.key == clean_key(&m.key) && s.page == page && s.file == file);
            match have {
                Some((description, figure, _)) => {
                    let was = figure.value.as_ref().and_then(|l| if let Len::One(x) = l { Some(*x) } else { None });
                    if figure.unit != m.unit {
                        out.left_as_it_is.push(format!("{description}: the mark now says its figure in {}, and the line is in {}; the line was left as it is", m.unit, figure.unit));
                    } else if was.is_some_and(|w| (w - q).abs() < 1.0e-9) {
                        out.same.push(description);
                    } else if figure.status == Status::Known {
                        out.left_as_it_is.push(format!("{description}: the line's {} {} was measured in the field (Known), and the file now says {} {}; the line was left as it is", crate::view::num(was.unwrap_or(0.0)), figure.unit, crate::view::num(q), m.unit));
                    } else {
                        let source = source_of(m, page, file);
                        set_figure(job, &root, &description, q, &source)?;
                        out.moved.push((description, was.unwrap_or(0.0), q, m.unit.to_string()));
                    }
                }
                None => {
                    // A line a person measured in the field has a source of their own, and the mark's name is gone from it:
                    // a line of the mark's own description, with a measured figure that is Known and is not a mark's, is that
                    // line. It is left as it is and said; nothing is added beside it to be counted twice.
                    let stem_form = format!("{} ({stem}, page {page})", m.label.trim());
                    let measured = lines_by_description(job, &[m.label.trim(), &stem_form]);
                    if let Some((description, figure)) = measured.into_iter().find(|(_, f)| f.status == Status::Known && parse_source(&f.source).is_none()) {
                        let was = figure.value.as_ref().and_then(|l| if let Len::One(x) = l { Some(*x) } else { None }).unwrap_or(0.0);
                        if figure.unit == m.unit && (was - q).abs() < 1.0e-9 {
                            out.same.push(description);
                        } else {
                            out.left_as_it_is.push(format!("{description}: the job has a line of that name that was measured in the field (Known), {} {}; the mark says {} {}; nothing was added or changed", crate::view::num(was), figure.unit, crate::view::num(q), m.unit));
                        }
                        continue;
                    }
                    // The description is the label; where the job already has a line by that name it is the label with the
                    // file and the page, and then a number.
                    let taken = |job: &mut Job, d: &str| job.world.get::<Cost>(job.root()).is_some_and(|c| c.lines.iter().any(|l| normal(&l.description) == normal(d)));
                    let mut description = m.label.trim().to_string();
                    if taken(job, &description) {
                        description = format!("{} ({stem}, page {page})", m.label.trim());
                    }
                    let mut n = 2;
                    let base = description.clone();
                    while taken(job, &description) {
                        description = format!("{base} #{n}");
                        n += 1;
                    }
                    let source = source_of(m, page, file);
                    job.add_line_measured(&root, &description, &format!("{q} {}", m.unit), None, false, Status::Assumed, &source)?;
                    out.added.push((description, q, m.unit.to_string()));
                }
            }
        }
        // What the job has of this file and page and the file no longer has.
        let present: Vec<String> = in_the_file.iter().map(|k| clean_key(k)).collect();
        for (description, _, s) in lines_from_marks(job) {
            if s.file == file && s.page == page && !present.contains(&s.key) {
                out.gone.push(description);
            }
        }
        Ok(out)
    })
}

/// The measured figure of each line of the job's own whose description is one of these.
fn lines_by_description(job: &mut Job, names: &[&str]) -> Vec<(String, Val<Len>)> {
    let root = job.root();
    let Some(cost) = job.world.get::<Cost>(root) else { return Vec::new() };
    cost.lines.iter().filter(|l| names.iter().any(|n| normal(n) == normal(&l.description))).filter_map(|l| l.inputs.get("measured").map(|m| (l.description.clone(), m.clone()))).collect()
}

fn set_figure(job: &mut Job, root: &str, description: &str, q: f64, source: &str) -> Result<(), String> {
    let e = job.find(root).ok_or("the job has no root")?;
    let kind = *job.world.get::<Kind>(e).ok_or("the job has no kind")?;
    let mut cost = job.world.get::<Cost>(e).cloned().ok_or("the job has no cost lines")?;
    let line = cost.lines.iter_mut().find(|l| l.description == description).ok_or_else(|| format!("no cost line {description:?}"))?;
    let m = line.inputs.get_mut("measured").ok_or_else(|| format!("{description:?} is not a mark's line"))?;
    m.value = Some(Len::One(q));
    m.status = Status::Assumed;
    m.source = source.to_string();
    crate::check::cost(root, kind, &cost)?;
    job.world.entity_mut(e).insert(cost);
    Ok(())
}
