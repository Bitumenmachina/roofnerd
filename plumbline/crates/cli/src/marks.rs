//! The marks on a marked-up PDF at the command line (call 227): `marks` lists them, `import marks` brings each measured
//! one into the estimate as a line, and `drawing` keeps their shapes with the sheet. The reading is `plumbline-plans`'; the
//! lines are the model's (`plumbline_world::marks`); this says them in a builder's words.

use std::fmt::Write as _;

use plumbline_plans::marks::{self as read_marks, Kind, Mark, Marks};
use plumbline_world::marks::{self, Brought, MarkShape, ToBring};
use plumbline_world::units;
use plumbline_world::value::{Status, Val};
use plumbline_world::view;

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// A figure as a builder says it: no trailing noughts, to two places.
fn fig(x: f64) -> String {
    view::num((x * 100.0).round() / 100.0)
}

/// The name of the file, as the estimate names it: the file's own name, not where it lies.
pub fn file_name(path: &str) -> String {
    std::path::Path::new(path).file_name().and_then(|f| f.to_str()).unwrap_or(path).to_string()
}

pub fn page_of(flag: Option<&str>) -> Result<usize, String> {
    match flag {
        None => Ok(1),
        Some(p) => p.parse::<usize>().map_err(|_| format!("--page {p:?} is not a page number")),
    }
}

/// Where a mark is on the sheet: across and up from its lower-left corner, in inches (a point is 1/72 in).
fn where_said(at: [f64; 2]) -> String {
    format!("{} across and {} up on the sheet", units::across_said(at[0] / 864.0), units::across_said(at[1] / 864.0))
}

/// What the points of a mark are, said for the kind of mark.
fn points_word(m: &Mark) -> &'static str {
    match m.kind {
        Kind::Length => "its ends",
        Kind::Run => "its points",
        _ => "its corners",
    }
}

/// What a mark says against what its points work out to, in one sentence, where they differ.
fn differ_said(m: &Mark) -> Option<String> {
    let (s, w) = (m.says.as_ref()?, m.worked?);
    if !m.differs() {
        return None;
    }
    let at = m.scale.as_deref().map(|r| format!(" at {r}")).unwrap_or_else(|| " at the scale it carries".into());
    Some(format!("it says {} {}; by {}{at} it works out to {} {}: {} {} apart. What the mark says is what comes in", fig(s.value), s.unit, points_word(m), fig(w), s.unit, fig((s.value - w).abs()), s.unit))
}

fn plural_kind(k: Kind, n: usize) -> String {
    match (k, n) {
        (Kind::Length, 1) => "1 length".into(),
        (Kind::Length, n) => format!("{n} lengths"),
        (Kind::Run, 1) => "1 run of lengths".into(),
        (Kind::Run, n) => format!("{n} runs of lengths"),
        (Kind::Area, 1) => "1 area".into(),
        (Kind::Area, n) => format!("{n} areas"),
        (Kind::Count, 1) => "1 count".into(),
        (Kind::Count, n) => format!("{n} counts"),
        (Kind::Note, 1) => "1 note".into(),
        (Kind::Note, n) => format!("{n} notes"),
    }
}

/// `plumbline marks <pdf> [--page n]`: the marks of a page, in the page's own order.
pub fn list(args: &[String]) -> Result<(), String> {
    let mut plain: Vec<&str> = Vec::new();
    let mut page: Option<&str> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--page" => {
                page = Some(args.get(i + 1).ok_or("--page needs a page number after it")?.as_str());
                i += 1;
            }
            f if f.starts_with("--") => return Err(format!("marks does not take {f}: plumbline marks <a marked-up PDF> [--page <n>]")),
            a => plain.push(a),
        }
        i += 1;
    }
    let [pdf] = plain.as_slice() else {
        return Err("marks takes one PDF and nothing else: plumbline marks <a marked-up PDF> [--page <n>]".into());
    };
    let page = page_of(page)?;
    let bytes = std::fs::read(pdf).map_err(|e| format!("cannot read {pdf}: {e}"))?;
    let got = read_marks::read(&bytes, page).map_err(|why| format!("{pdf}: {why}"))?;
    let _ = std::io::Write::write_all(&mut std::io::stdout(), listing(&file_name(pdf), &got).as_bytes());
    Ok(())
}

pub fn listing(file: &str, got: &Marks) -> String {
    let mut out = String::new();
    if got.marks.is_empty() {
        let _ = writeln!(out, "no marks on page {} of {file}", got.page);
        for l in &got.left {
            let _ = writeln!(out, "  not listed: {l}");
        }
        return out;
    }
    let read: Vec<&Mark> = got.marks.iter().filter(|m| m.unread.is_none()).collect();
    let mut parts: Vec<String> = Vec::new();
    for k in [Kind::Length, Kind::Run, Kind::Area, Kind::Count, Kind::Note] {
        let n = read.iter().filter(|m| m.kind == k).count();
        if n > 0 {
            parts.push(plural_kind(k, n));
        }
    }
    let unread = got.marks.len() - read.len();
    if unread > 0 {
        parts.push(format!("{} not read", unread));
    }
    let flagged = read.iter().filter(|m| m.differs()).count();
    let _ = writeln!(out, "{file}, page {} of {}: {} ({})", got.page, got.pages, count(got.marks.len(), "mark", "marks"), parts.join(", "));
    let _ = writeln!(out, "  each figure is what the mark itself says; ! marks where its points, at the scale it carries, work out to another figure; ? marks one that cannot be read");
    for m in &got.marks {
        let kind = m.kind.said();
        let place = where_said(m.at());
        if let Some(why) = &m.unread {
            let said = m.says.as_ref().map(|s| format!(" (it says {})", s.text)).unwrap_or_default();
            let _ = writeln!(out, "? {kind:<14} {:<22} not read: {why}{said}   at {place}", m.label);
            continue;
        }
        match (m.kind, &m.says) {
            (Kind::Note, _) => {
                let _ = writeln!(out, "  {kind:<14} {:<22} {}   at {place}", m.label, if m.text.is_empty() || m.text == m.label { "(a note: it measures nothing)".to_string() } else { format!("{:?} (a note: it measures nothing)", m.text) });
            }
            (Kind::Count, Some(s)) => {
                let _ = writeln!(out, "  {kind:<14} {:<22} {:>8} {}   ({}: this program's count of the symbols labelled so, not a figure the file states; the first at {place})", m.label, fig(s.value), s.unit, count(m.places.len(), "symbol", "symbols"));
            }
            (_, Some(s)) => {
                let by = match (m.worked, &m.not_worked) {
                    (Some(w), _) => format!("   by {} at {}: {} {}", points_word(m), m.scale.as_deref().unwrap_or("the scale it carries"), fig(w), s.unit),
                    (None, Some(why)) => format!("   its points could not be worked: {why}"),
                    (None, None) => String::new(),
                };
                let flag = if m.differs() { "!" } else { " " };
                let _ = writeln!(out, "{flag} {kind:<14} {:<22} {:>8} {}{by}   at {place}", m.label, fig(s.value), s.unit);
                if let Some(d) = differ_said(m) {
                    let _ = writeln!(out, "    ! {}: {d}", m.label);
                }
            }
            (_, None) => {}
        }
    }
    for l in &got.left {
        let _ = writeln!(out, "  not listed: {l}");
    }
    if flagged > 0 {
        let _ = writeln!(out, "  {} to check against the sheet: the mark's own figure is the one taken, and the other is said beside it", count(flagged, "mark", "marks"));
    }
    out
}

/// The marks of a page as the shapes the drawing keeps (call 227 point 7): every mark, the ones not read included, with how
/// each was read. What a mark says is Assumed and names the mark, the page and the file; it is never Known.
pub fn shapes(file: &str, page: usize, got: &Marks) -> Vec<MarkShape> {
    let keys = read_marks::keys(&got.marks);
    got.marks
        .iter()
        .zip(keys)
        .map(|(m, key)| MarkShape {
            kind: m.kind.said().to_string(),
            label: m.label.clone(),
            name: m.name.clone(),
            points: m.points.iter().map(|p| [round4(p[0]), round4(p[1])]).collect(),
            places: m.places.iter().map(|p| [round4(p[0]), round4(p[1])]).collect(),
            says: m.says.as_ref().filter(|_| m.unread.is_none()).map(|s| Val { value: Some(s.value), unit: s.unit.to_string(), status: Status::Assumed, source: if m.kind == Kind::Count { marks::count_source_text(&m.label, m.places.len(), page, file) } else { marks::source_text(&key, m.kind.said(), page, file, None) } }),
            worked: m.worked.map(|w| (w * 1.0e6).round() / 1.0e6),
            unread: m.unread.clone(),
        })
        .collect()
}

fn round4(x: f64) -> f64 {
    (x * 1.0e4).round() / 1.0e4
}

/// What `drawing` says of the marks it kept, to follow its own line: none said where the page has none.
pub fn kept_said(got: &Marks) -> String {
    if got.marks.is_empty() && got.left.is_empty() {
        return String::new();
    }
    let measured = got.marks.iter().filter(|m| !matches!(m.kind, Kind::Note) && m.unread.is_none()).count();
    let unread = got.marks.iter().filter(|m| m.unread.is_some()).count();
    let mut out = String::new();
    if !got.marks.is_empty() {
        let _ = write!(out, "\n  marks: {} on the page kept with the drawing ({} measured", count(got.marks.len(), "mark", "marks"), measured);
        if unread > 0 {
            let _ = write!(out, ", {unread} not read");
        }
        let _ = write!(out, "); `plumbline marks` lists them and `plumbline import marks` brings the measured ones into the estimate");
    }
    for l in &got.left {
        let _ = write!(out, "\n  not kept: {l}");
    }
    out
}

/// `plumbline import marks <job> <pdf> [--page n]`: the said text, and whether the job changed.
pub fn import(job: &mut plumbline_world::Job, pdf: &str, page: usize, counts: bool) -> Result<(String, bool), String> {
    let bytes = std::fs::read(pdf).map_err(|e| format!("cannot read {pdf}: {e}"))?;
    let got = read_marks::read(&bytes, page).map_err(|why| format!("{pdf}: {why}"))?;
    let file = file_name(pdf);
    let mut out = String::new();
    if got.marks.is_empty() {
        let _ = writeln!(out, "no marks on page {page} of {file}: nothing was brought in");
        for l in &got.left {
            let _ = writeln!(out, "  not brought in: {l}");
        }
        return Ok((out, false));
    }
    let keys = read_marks::keys(&got.marks);
    let mut bring: Vec<ToBring> = Vec::new();
    let mut notes = 0usize;
    let mut left_out: Vec<String> = Vec::new();
    let mut flagged: Vec<String> = Vec::new();
    let mut counts_left: Vec<(String, usize)> = Vec::new();
    for (m, key) in got.marks.iter().zip(&keys) {
        if let Some(why) = &m.unread {
            left_out.push(format!("{}: {why}", m.label));
            continue;
        }
        let Some(says) = &m.says else {
            notes += 1;
            continue;
        };
        // The standard has no count: it is this program's count of the page's symbols, and is brought in only when asked for.
        if m.kind == Kind::Count && !counts {
            counts_left.push((m.label.clone(), says.value as usize));
            continue;
        }
        let points_say = differ_said(m).map(|_| format!("its {} work out to {} {} at the scale it carries; what it says is what was taken", match m.kind { Kind::Length => "ends", Kind::Run => "points", _ => "corners" }, fig(m.worked.unwrap_or(0.0)), says.unit));
        if let Some(d) = differ_said(m) {
            flagged.push(format!("{}: {d}", m.label));
        }
        bring.push(ToBring { key: key.clone(), label: m.label.clone(), quantity: says.value, unit: says.unit, kind: m.kind.said().to_string(), points_say });
    }
    let done: Brought = marks::bring_in(job, &file, page, &bring, &keys)?;
    let changed = !done.added.is_empty() || !done.moved.is_empty();
    let _ = writeln!(out, "{file}, page {page}: {} measured: each is a line of the estimate under the file's name, Assumed, naming its mark, the page and the file, not priced, its division not set", count(bring.len(), "mark", "marks"));
    if !done.added.is_empty() {
        let _ = writeln!(out, "  {} added:", done.added.len());
        for (d, q, u) in &done.added {
            let _ = writeln!(out, "    {d}: {} {u}", fig(*q));
        }
        let _ = writeln!(out, "  {} not priced (a price goes on each line) and with no division (set with `plumbline division`); the job's price is what it was", count(done.added.len(), "line is", "lines are"));
    }
    if !done.same.is_empty() {
        let _ = writeln!(out, "  {} the same as the job already has", count(done.same.len(), "is", "are"));
    }
    for (d, was, now, u) in &done.moved {
        let _ = writeln!(out, "  {d}: was {} {u}, now {} {u}; its line was moved", fig(*was), fig(*now));
    }
    for l in &done.left_as_it_is {
        let _ = writeln!(out, "  left as it is: {l}");
    }
    for g in &done.gone {
        let _ = writeln!(out, "  {g}: gone from the file; its line is still in the job, and nothing is taken away because a file lacks it");
    }
    for f in &flagged {
        let _ = writeln!(out, "  ! {f}");
    }
    for l in &left_out {
        let _ = writeln!(out, "  not read, left out of the estimate: {l}");
    }
    if !counts_left.is_empty() {
        let symbols: usize = counts_left.iter().map(|c| c.1).sum();
        let _ = writeln!(out, "  counts left: {}, {} ({}): the file states no count, this program counts the page's symbols; `--counts` brings them in as lines", count(counts_left.len(), "label", "labels"), count(symbols, "symbol", "symbols"), counts_left.iter().map(|c| c.0.as_str()).collect::<Vec<_>>().join(", "));
    }
    if notes > 0 {
        let _ = writeln!(out, "  {} not brought in: it measures nothing", count(notes, "note", "notes"));
    }
    for l in &got.left {
        let _ = writeln!(out, "  not brought in: {l}");
    }
    Ok((out, changed))
}
