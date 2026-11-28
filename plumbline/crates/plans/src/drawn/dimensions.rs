use super::*;

/// One dimension string the page holds as words, with the dimension line it stands by.
#[derive(Clone, Debug, PartialEq)]
pub struct DimString {
    /// As written ("30'-0\"").
    pub text: String,
    /// What it says, in feet.
    pub feet: f64,
    /// Where the string is, in page units.
    pub at: P2,
    /// The two ends its dimension line measures between (its ticks), in page units, and how far apart they are.
    pub from: P2,
    pub to: P2,
    pub paper: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dimensions {
    pub strings: Vec<DimString>,
    /// The scale most of them give, in feet to a page unit, and which of them (by place in `strings`) give it.
    pub scale: Option<f64>,
    pub agree: Vec<usize>,
    /// Those that do not, each with what it measures at that scale, in feet.
    pub off: Vec<(usize, f64)>,
    /// Strings that read as a dimension and have no dimension line beside them.
    pub loose: Vec<(String, P2)>,
    /// Strings written as a dimension that could not be used (call 191 item 9): a length of nothing, inches alone, metric. Each with
    /// where it stands and why.
    pub unread: Vec<(String, P2, &'static str)>,
}

/// Why a string that looks like a dimension was not read as one: it is written as a measure and not in feet and inches. None for words,
/// sheet numbers and the like.
pub(super) fn dimension_like(text: &str) -> Option<&'static str> {
    let t = text.trim().to_lowercase().replace(['\u{201D}', '\u{201C}', '\u{2033}'], "\"").replace(['\u{2019}', '\u{2018}', '\u{2032}'], "'");
    if feet_written_raw(&t).is_some() {
        return Some("no length: a dimension of nothing is no dimension");
    }
    let number = |x: &str| -> bool {
        let x = x.trim().replace(',', ".");
        !x.is_empty() && x.chars().all(|c| c.is_ascii_digit() || c == '.') && x.chars().filter(|c| *c == '.').count() <= 1 && x.chars().any(|c| c.is_ascii_digit())
    };
    for (suffix, why) in [("mm", "metric"), ("cm", "metric"), ("m", "metric"), ("inches", "inches alone"), ("inch", "inches alone"), ("in", "inches alone"), ("\"", "inches alone")] {
        if let Some(n) = t.strip_suffix(suffix) {
            if number(n) {
                return Some(if why == "metric" { "metric: this reads dimensions in feet and inches only" } else { "inches alone: this reads dimensions written with feet" });
            }
        }
    }
    None
}

/// What a string of a plan says as a length in feet, where it is written as an architect's dimension: feet, a tick, and inches
/// that may carry a fraction ("30'-0\"", "5'-8 1/2\"", "16' 5\"", "10'"). Anything else is None: a room's name, a sheet's number,
/// inches alone (which has no feet to say it by).
pub fn feet_written(text: &str) -> Option<f64> {
    feet_written_raw(text).filter(|f| *f > 0.0)
}

/// `feet_written`, a length of nothing included.
pub(super) fn feet_written_raw(text: &str) -> Option<f64> {
    let norm: String = text
        .chars()
        .map(|c| match c {
            '\u{2019}' | '\u{2018}' | '\u{2032}' | '\u{00B4}' | '`' => '\'',
            '\u{201D}' | '\u{201C}' | '\u{2033}' | '\u{00A8}' => '"',
            '\u{2013}' | '\u{2014}' | '\u{2212}' | '\u{2010}' => '-',
            c => c,
        })
        .collect();
    let t = norm.trim();
    let (ft, rest) = t.split_once('\'')?;
    let ft = ft.trim();
    if ft.is_empty() || !ft.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let feet: f64 = ft.parse().ok()?;
    let rest = rest.trim().trim_start_matches('-').trim();
    let rest = rest.strip_suffix('"').unwrap_or(rest).trim();
    if rest.is_empty() {
        return Some(feet);
    }
    let mut inches = 0.0;
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    for (n, part) in parts.iter().enumerate() {
        if let Some((a, b)) = part.split_once('/') {
            let (a, b): (f64, f64) = (a.parse().ok()?, b.parse().ok()?);
            if b == 0.0 || n == 0 && parts.len() == 2 {
                return None;
            }
            inches += a / b;
        } else {
            if n == 1 || !part.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            inches += part.parse::<f64>().ok()?;
        }
    }
    if inches >= 12.0 {
        return None;
    }
    Some(feet + inches / 12.0)
}

/// The dimension strings of a page (its words, each where it stands) paired with the dimension lines they stand by, and whether the
/// scales they give agree (call 188 item 9). `lines` are the page's straight lines in its own units, `dashed` the places of the
/// dashed ones (not read). A string is paired with the dimension line parallel to it, within four letters' height of it and over its
/// span; the line is measured between its ticks or arrowheads, not between its ends (a line may run past its ticks). The scale most
/// of them give, within an inch or 1 per cent of what each says, is the scale taken; two that disagree with nothing to say which is
/// right give none, and a string that is off it is said with the length it measures at the scale the others give. What reads as a
/// dimension and has no dimension line beside it is `loose`, and never used.
pub fn dimension_strings(lines: &[[[f64; 2]; 2]], dashed: &[usize], texts: &[crate::Text]) -> Dimensions {
    let mut out = Dimensions::default();
    let dashed: std::collections::HashSet<usize> = dashed.iter().copied().collect();
    let segs: Vec<Seg> = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !dashed.contains(i))
        .filter_map(|(_, l)| {
            let len = norm(sub(l[1], l[0]));
            (len > 1e-9 && l.iter().flatten().all(|v| v.is_finite())).then_some(Seg { a: l[0], b: l[1], len })
        })
        .collect();
    let dims = dimension_spans(&segs);
    let mut taken = vec![false; dims.len()];
    for t in texts {
        let Some(feet) = feet_written(&t.text) else {
            if let Some(why) = dimension_like(&t.text) {
                out.unread.push((t.text.trim().to_string(), t.at, why));
            }
            continue;
        };
        let u = unit(t.dir);
        let n = perp(u);
        let len = t.text.chars().count() as f64 * 0.5 * t.tall;
        let centre = [t.at[0] + u[0] * len / 2.0 + n[0] * 0.35 * t.tall, t.at[1] + u[1] * len / 2.0 + n[1] * 0.35 * t.tall];
        let mut best: Option<(usize, f64)> = None;
        for (j, d) in dims.iter().enumerate() {
            if taken[j] {
                continue;
            }
            let (a, b) = (d.between[0], d.between[1]);
            let (du, span) = (unit(sub(b, a)), norm(sub(b, a)));
            if cross(u, du).abs() > 0.1 || span < 1e-9 {
                continue;
            }
            let along = dot(du, sub(centre, a));
            let across = dot(perp(du), sub(centre, a)).abs();
            if along < -t.tall || along > span + t.tall || across > 4.0 * t.tall + 4.0 {
                continue;
            }
            if best.map_or(true, |(_, bd)| across < bd) {
                best = Some((j, across));
            }
        }
        match best {
            Some((j, _)) => {
                taken[j] = true;
                let d = dims[j];
                out.strings.push(DimString { text: t.text.trim().to_string(), feet, at: centre, from: d.between[0], to: d.between[1], paper: norm(sub(d.between[1], d.between[0])) });
            }
            None => out.loose.push((t.text.trim().to_string(), centre)),
        }
    }
    let n = out.strings.len();
    let tol = |feet: f64| INCH.max(0.01 * feet);
    let scale_of = |d: &DimString| d.feet / d.paper;
    if n == 1 {
        out.scale = Some(scale_of(&out.strings[0]));
        out.agree = vec![0];
    } else if n >= 2 {
        let agreeing = |i: usize| -> Vec<usize> {
            let k = scale_of(&out.strings[i]);
            (0..n).filter(|&j| (out.strings[j].paper * k - out.strings[j].feet).abs() <= tol(out.strings[j].feet)).collect()
        };
        let best = (0..n).map(|i| (agreeing(i), i)).max_by(|a, b| a.0.len().cmp(&b.0.len()).then(out.strings[a.1].feet.total_cmp(&out.strings[b.1].feet)));
        if let Some((set, _)) = best.filter(|(set, _)| set.len() >= 2) {
            let k = set.iter().map(|&j| scale_of(&out.strings[j])).sum::<f64>() / set.len() as f64;
            out.scale = Some(k);
            out.off = (0..n).filter(|j| !set.contains(j)).map(|j| (j, out.strings[j].paper * k)).collect();
            out.agree = set;
        }
    }
    out
}
