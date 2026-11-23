//! Where lettering lies on a page, and what may not happen to it (call 179 D2): ours, not
//! BIMCraft's (its plan pass letters labels with no test for one lying over another).
//!
//! Every piece of lettering a page carries is recorded as a box (`Letter`) as the strokes are
//! drawn, so a check can ask the written page, and not the code that drew it, whether any
//! letter is under the least height drafting allows (3/32 in, the standard's smallest
//! capital: ordinary drafting practice and the US National CAD Standard's sheet conventions),
//! whether two pieces lie over one another, and whether a piece is out of the frame.

use super::font;

/// The least height of a capital on a printed sheet: 3/32 in, in points.
pub const MIN_CAP: f32 = 6.75;

/// One piece of lettering: a turned box about its centre, a point's units, y up.
#[derive(Clone, Debug, PartialEq)]
pub struct Letter {
    pub text: String,
    /// The capitals' height.
    pub height: f32,
    pub centre: (f32, f32),
    /// Half its width and half its height, stroke included.
    pub half: (f32, f32),
    /// Counter-clockwise from east, radians.
    pub angle: f32,
}

impl Letter {
    /// A piece of `text` with its capitals `height` tall, centred on `centre`, turned `angle`.
    pub fn centred(text: &str, centre: (f32, f32), height: f32, angle: f32) -> Letter {
        let stroke = (height / 10.0).max(0.5);
        Letter { text: text.to_string(), height, centre, half: (font::width(text, height) / 2.0 + stroke / 2.0, height / 2.0 + stroke / 2.0), angle }
    }

    /// The four corners.
    pub fn corners(&self) -> [(f32, f32); 4] {
        let (s, c) = self.angle.sin_cos();
        let (hw, hh) = self.half;
        [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)].map(|(x, y)| (self.centre.0 + x * c - y * s, self.centre.1 + x * s + y * c))
    }

    /// Whether this lies over `other` (by more than a hair: boxes that only touch do not).
    pub fn crosses(&self, other: &Letter) -> bool {
        let (a, b) = (self.corners(), other.corners());
        let axes = |l: &Letter| {
            let (s, c) = l.angle.sin_cos();
            [(c, s), (-s, c)]
        };
        for (ax, ay) in axes(self).into_iter().chain(axes(other)) {
            let project = |pts: &[(f32, f32); 4]| {
                let v: Vec<f32> = pts.iter().map(|(x, y)| x * ax + y * ay).collect();
                (v.iter().copied().fold(f32::MAX, f32::min), v.iter().copied().fold(f32::MIN, f32::max))
            };
            let ((lo_a, hi_a), (lo_b, hi_b)) = (project(&a), project(&b));
            if hi_a <= lo_b + 0.05 || hi_b <= lo_a + 0.05 {
                return false;
            }
        }
        true
    }

    /// Whether the segment `a` to `b` passes through this box (a drawn line across the lettering).
    pub fn touches_segment(&self, a: (f32, f32), b: (f32, f32)) -> bool {
        // Into the box's own frame, where it is level and centred on the origin.
        let (s, c) = (-self.angle).sin_cos();
        let local = |q: (f32, f32)| {
            let (x, y) = (q.0 - self.centre.0, q.1 - self.centre.1);
            (x * c - y * s, x * s + y * c)
        };
        let (p, q) = (local(a), local(b));
        // Liang and Barsky's clipping: the part of the segment inside the rectangle, if any.
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        for (d, lo_gap, hi_gap) in [(q.0 - p.0, p.0 + self.half.0, self.half.0 - p.0), (q.1 - p.1, p.1 + self.half.1, self.half.1 - p.1)] {
            if d.abs() < 1e-9 {
                if lo_gap < 0.0 || hi_gap < 0.0 {
                    return false;
                }
            } else {
                let (r0, r1) = (-lo_gap / d, hi_gap / d);
                let (near, far) = if d > 0.0 { (r0, r1) } else { (r1, r0) };
                t0 = t0.max(near);
                t1 = t1.min(far);
                if t0 > t1 {
                    return false;
                }
            }
        }
        true
    }

    /// Whether every corner is inside `rect` (x0, y0, x1, y1).
    pub fn within(&self, rect: [f32; 4]) -> bool {
        self.corners().iter().all(|&(x, y)| x >= rect[0] - 0.01 && x <= rect[2] + 0.01 && y >= rect[1] - 0.01 && y <= rect[3] + 0.01)
    }
}

/// `text` cut at a word, with an ellipsis, to be at most `width` wide at `height`; the whole
/// text when it fits. A first word that alone is too wide is cut where it fits.
pub fn cut_to(text: &str, height: f32, width: f32) -> String {
    if font::width(text, height) <= width {
        return text.to_string();
    }
    let mut kept = String::new();
    for word in text.split_whitespace() {
        let tried = if kept.is_empty() { word.to_string() } else { format!("{kept} {word}") };
        if font::width(&format!("{tried}..."), height) > width {
            break;
        }
        kept = tried;
    }
    if kept.is_empty() {
        let mut chars: Vec<char> = text.chars().collect();
        while !chars.is_empty() && font::width(&format!("{}...", chars.iter().collect::<String>()), height) > width {
            chars.pop();
        }
        return format!("{}...", chars.iter().collect::<String>());
    }
    format!("{kept}...")
}

/// `text` on at most `lines` lines each at most `width` wide at `height`, broken at words;
/// what does not fit in that many lines is cut at a word with an ellipsis.
pub fn lines_of(text: &str, height: f32, width: f32, lines: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut words = text.split_whitespace().peekable();
    while let Some(word) = words.next() {
        match out.last_mut() {
            Some(l) if font::width(&format!("{l} {word}"), height) <= width => {
                l.push(' ');
                l.push_str(word);
            }
            _ => {
                if out.len() == lines {
                    // Out of lines: the last one is cut where it is.
                    let rest: Vec<&str> = std::iter::once(word).chain(words.by_ref()).collect();
                    let last = out.pop().unwrap_or_default();
                    out.push(cut_to(&format!("{last} {}", rest.join(" ")), height, width));
                    return out;
                }
                out.push(cut_to(word, height, width));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lettering_that_overlaps_is_found_and_lettering_beside_it_is_not() {
        let a = Letter::centred("KITCHEN", (100.0, 100.0), 9.0, 0.0);
        let over = Letter::centred("182 SF", (100.0, 104.0), 7.2, 0.0);
        let beside = Letter::centred("182 SF", (100.0, 80.0), 7.2, 0.0);
        let turned = Letter::centred("12'-0\"", (100.0, 100.0), 7.2, std::f32::consts::FRAC_PI_2);
        assert!(a.crosses(&over) && over.crosses(&a));
        assert!(!a.crosses(&beside));
        assert!(a.crosses(&turned));
        assert!(!a.crosses(&Letter::centred("X", (300.0, 100.0), 9.0, 0.3)));
    }

    #[test]
    fn a_drawn_line_across_lettering_is_found() {
        let l = Letter::centred("SB36", (100.0, 100.0), 7.2, std::f32::consts::FRAC_PI_2);
        assert!(l.touches_segment((90.0, 100.0), (110.0, 100.0)));
        assert!(!l.touches_segment((90.0, 130.0), (110.0, 130.0)));
        assert!(!l.touches_segment((110.0, 80.0), (110.0, 120.0)));
    }

    #[test]
    fn a_title_that_does_not_fit_is_cut_at_a_word_with_an_ellipsis() {
        assert_eq!(cut_to("SHORT", 8.0, 200.0), "SHORT");
        let cut = cut_to("A VERY LONG NAME OF A JOB THAT GOES ON", 8.0, 100.0);
        assert!(cut.ends_with("..."), "{cut}");
        assert!(font::width(&cut, 8.0) <= 100.0);
        assert!(!cut.contains("GOES"));
        let lines = lines_of("A VERY LONG NAME OF A JOB THAT GOES ON AND ON AND ON", 8.0, 100.0, 2);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].ends_with("..."));
        assert!(lines.iter().all(|l| font::width(l, 8.0) <= 100.0));
    }
}
