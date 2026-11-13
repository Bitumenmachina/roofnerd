//! A wall's framing, laid out: what a framer builds from a wall's length, its height,
//! its openings, and the lumber and spacing its framing line names.
//!
//! Ported from BIMCraft's timber layout (`crates/core/src/timber.rs`, `layout`, line
//! 70), at commit f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), under its MIT
//! licence (NOTICE). The rule there is the one here: a bottom plate and two top plates
//! the length of the wall; a stud at each end; beside an opening a jack under each end
//! of its header and a king outside each jack; a header from jack to jack over the
//! opening and a sill under it; and studs on stations at multiples of the spacing from
//! the wall's first end, whole where the wall is clear, and as cripples on the same
//! stations above the header and below the sill. It is platform framing as the
//! International Residential Code draws it (R602.3, R602.7).
//!
//! What is not taken, and what is changed. BIMCraft lays out one window in one
//! isolated wall and refuses anything else; here a wall has as many openings as it
//! has, and a door (an opening to the floor) has no sill and nothing under it. There
//! a station stud that a king or a jack would cut into is clipped to what is left of
//! it; here it is left out, because a sliver is not a piece of lumber. There the
//! header's height is asked for; here it is taken from the span by a rule of thumb.
//! Its keys, its bays (the clear spaces between members) and its layers are not taken.
//!
//! Two things here are rules of thumb and say so where they are used: a header's depth
//! is taken from its span (the code's tables size it from the load it carries, which a
//! wall alone does not say), and the bottom plate is laid whole under a door (it is cut
//! out after the wall stands).
//!
//! Everything is in feet, in the wall's own plane: `along` from the wall's first end,
//! `up` from its base.

/// What a piece of the frame is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    BottomPlate,
    TopPlate,
    /// A stud of the layout, full height between the plates.
    Stud,
    /// A full-height stud beside an opening.
    King,
    /// The stud under each end of a header.
    Jack,
    Header,
    /// The rough sill under a window.
    Sill,
    /// A short stud above a header or below a sill, on the layout's centres.
    Cripple,
}

impl Piece {
    pub fn said(self) -> &'static str {
        match self {
            Piece::BottomPlate => "bottom plate",
            Piece::TopPlate => "top plate",
            Piece::Stud => "stud",
            Piece::King => "king stud",
            Piece::Jack => "jack stud",
            Piece::Header => "header",
            Piece::Sill => "rough sill",
            Piece::Cripple => "cripple",
        }
    }
}

/// One piece of lumber in the wall: from where to where along it, and from what height
/// to what height. It is as thick through the wall as the wall's lumber is deep.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub piece: Piece,
    pub along: [f64; 2],
    pub up: [f64; 2],
}

impl Member {
    /// How long the piece of lumber is: a plate, a header or a sill along the wall, a
    /// stud up it.
    pub fn length(&self) -> f64 {
        match self.piece {
            Piece::BottomPlate | Piece::TopPlate | Piece::Header | Piece::Sill => self.along[1] - self.along[0],
            _ => self.up[1] - self.up[0],
        }
    }
}

/// Dimension lumber as it is dressed (American Softwood Lumber Standard, PS 20): a
/// nominal 2 is 1 1/2 in; a nominal 4, 6, 8, 10 and 12 are 3 1/2, 5 1/2, 7 1/4, 9 1/4
/// and 11 1/4 in. In feet. None for a size that is not one of these.
pub fn dressed(nominal: u32) -> Option<f64> {
    let inches = match nominal {
        2 => 1.5,
        4 => 3.5,
        6 => 5.5,
        8 => 7.25,
        10 => 9.25,
        12 => 11.25,
        _ => return None,
    };
    Some(inches / 12.0)
}

/// What a framing line names: the lumber's depth through the wall and the studs'
/// spacing, both in feet. "Wall framing, 2×6 at 16 in" is 5 1/2 in at 16 in; "2x4 @ 24
/// in. o.c." is 3 1/2 in at 24 in. None for words that name no framing: nothing is
/// framed on a guess.
pub fn named(words: &str) -> Option<(f64, f64)> {
    // As estimators write it: "2x6 at 16 in", "2 x 6 @ 24", "2X4 studs 16 OC",
    // "2x6 16 in o.c.".
    let lower = words.to_lowercase().replace('×', "x").replace(" x ", "x");
    if !lower.contains("framing") && !lower.contains("stud") {
        return None;
    }
    // "2x6": a 2 by something that dressed lumber comes in.
    let at = lower.find("2x")?;
    let digits: String = lower[at + 2..].chars().take_while(char::is_ascii_digit).collect();
    let deep = dressed(digits.parse().ok()?).filter(|_| digits != "2")?;
    // The spacing: a number said to be one, by "at" or "@" before it or by inches or
    // "on centre" after it. A number that is neither (a length the lumber comes in, a
    // count) is not taken for a spacing.
    let rest: Vec<char> = lower[at + 2 + digits.len()..].chars().filter(|c| !c.is_whitespace()).collect();
    let mut i = 0;
    while i < rest.len() {
        if !rest[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < rest.len() && (rest[i].is_ascii_digit() || rest[i] == '.') {
            i += 1;
        }
        let before: String = rest[start.saturating_sub(2)..start].iter().collect();
        let after: String = rest[i..rest.len().min(i + 4)].iter().collect();
        let said = before.ends_with("at") || before.ends_with('@') || ["in", "\"", "oc", "o.c", "onc"].iter().any(|m| after.starts_with(m));
        let number: String = rest[start..i].iter().collect();
        if let (true, Ok(spacing)) = (said, number.trim_end_matches('.').parse::<f64>()) {
            if (8.0..=48.0).contains(&spacing) {
                return Some((deep, spacing / 12.0));
            }
        }
    }
    None
}

/// A header's depth for its span, in feet: a rule of thumb for a one-storey bearing
/// wall (a 2x6 to 4 ft, a 2x8 to 6 ft, a 2x10 to 8 ft, a 2x12 beyond). The code sizes a
/// header from what it carries; a drawing of the frame needs a depth to draw.
pub fn header_depth(span: f64) -> f64 {
    let nominal = if span <= 4.0 {
        6
    } else if span <= 6.0 {
        8
    } else if span <= 8.0 {
        10
    } else {
        12
    };
    dressed(nominal).unwrap_or(0.0)
}

/// The frame of a wall `length` long and `height` high with `holes` in it (each from,
/// to along the wall and sill, head above its base), its studs `spacing` on centre from
/// its first end. Every piece is 1 1/2 in thick in the wall's plane.
pub fn frame(length: f64, height: f64, holes: &[[f64; 4]], spacing: f64) -> Vec<Member> {
    let t = 1.5 / 12.0;
    let mut out = Vec::new();
    if !(length > 2.0 * t && height > 4.0 * t && spacing > t) {
        return out;
    }
    // One bottom plate, two top plates, the whole length of the wall.
    out.push(Member { piece: Piece::BottomPlate, along: [0.0, length], up: [0.0, t] });
    for k in [2.0, 1.0] {
        out.push(Member { piece: Piece::TopPlate, along: [0.0, length], up: [height - k * t, height - (k - 1.0) * t] });
    }
    let (low, high) = (t, height - 2.0 * t);
    // Each opening, kept to the wall, and what is built round it: a jack under each
    // end of the header and a king outside each jack.
    let holes: Vec<[f64; 4]> = holes.iter().map(|h| [h[0].max(2.0 * t), h[1].min(length - 2.0 * t), h[2].max(0.0), h[3].min(high)]).filter(|h| h[1] - h[0] > t && h[3] - h[2] > t).collect();
    // The layout: a stud at the first end, one on each centre, one at the last end.
    let mut edges: Vec<f64> = vec![0.0];
    let mut centre = spacing;
    while centre + t / 2.0 < length - t {
        edges.push(centre - t / 2.0);
        centre += spacing;
    }
    edges.push(length - t);
    for left in edges {
        // A layout stud that falls where an opening and its jacks and kings are is not
        // set whole: above the header and below the sill it is a cripple.
        match holes.iter().find(|h| left < h[1] + 2.0 * t && left + t > h[0] - 2.0 * t) {
            None => out.push(Member { piece: Piece::Stud, along: [left, left + t], up: [low, high] }),
            Some(h) if left >= h[0] && left + t <= h[1] => {
                let top = h[3] + header_depth(h[1] - h[0]);
                if high - top > t {
                    out.push(Member { piece: Piece::Cripple, along: [left, left + t], up: [top, high] });
                }
                if h[2] - t - low > t {
                    out.push(Member { piece: Piece::Cripple, along: [left, left + t], up: [low, h[2] - t] });
                }
            }
            // Where the jacks and kings stand, the layout stud is theirs.
            Some(_) => {}
        }
    }
    for h in &holes {
        let deep = header_depth(h[1] - h[0]).min((high - h[3]).max(0.0));
        for (jack, king) in [(h[0] - t, h[0] - 2.0 * t), (h[1], h[1] + t)] {
            out.push(Member { piece: Piece::King, along: [king, king + t], up: [low, high] });
            out.push(Member { piece: Piece::Jack, along: [jack, jack + t], up: [low, h[3]] });
        }
        if deep > 0.0 {
            out.push(Member { piece: Piece::Header, along: [h[0] - t, h[1] + t], up: [h[3], h[3] + deep] });
        }
        // A window has a rough sill; a door's opening goes to the floor.
        if h[2] - t > low {
            out.push(Member { piece: Piece::Sill, along: [h[0], h[1]], up: [h[2] - t, h[2]] });
        }
    }
    out
}

/// How many pieces of each sort, and their lumber added up, in feet: what a count of
/// the frame as drawn comes to.
pub fn tally(frame: &[Member]) -> Vec<(Piece, usize, f64)> {
    let mut out: Vec<(Piece, usize, f64)> = Vec::new();
    for m in frame {
        match out.iter_mut().find(|(p, _, _)| *p == m.piece) {
            Some(row) => {
                row.1 += 1;
                row.2 += m.length();
            }
            None => out.push((m.piece, 1, m.length())),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inches(x: f64) -> f64 {
        (x * 12.0 * 1000.0).round() / 1000.0
    }
    fn count(frame: &[Member], piece: Piece) -> usize {
        frame.iter().filter(|m| m.piece == piece).count()
    }

    #[test]
    fn a_plain_wall_has_a_stud_at_each_end_and_on_each_centre() {
        // 12 ft at 16 in: centres at 16, 32, ... 128 in (eight of them) and a stud at
        // each end: ten. The rule of thumb, length over spacing plus one: 9 + 1.
        let f = frame(12.0, 9.0, &[], 16.0 / 12.0);
        assert_eq!(count(&f, Piece::Stud), 10);
        let lefts: Vec<f64> = f.iter().filter(|m| m.piece == Piece::Stud).map(|m| inches(m.along[0])).collect();
        assert_eq!(lefts, vec![0.0, 15.25, 31.25, 47.25, 63.25, 79.25, 95.25, 111.25, 127.25, 142.5]);
        // Each stands on the bottom plate and under the two top plates: 108 - 4.5.
        assert!(f.iter().filter(|m| m.piece == Piece::Stud).all(|m| inches(m.length()) == 103.5));
        // One bottom plate and two top plates, each the length of the wall: 36 LF.
        assert_eq!((count(&f, Piece::BottomPlate), count(&f, Piece::TopPlate)), (1, 2));
        let plate: f64 = f.iter().filter(|m| matches!(m.piece, Piece::BottomPlate | Piece::TopPlate)).map(Member::length).sum();
        assert!((plate - 36.0).abs() < 1.0e-9);
    }

    #[test]
    fn a_window_takes_studs_out_and_puts_its_own_round_it() {
        // The worked wall of STATUS.md: 12 ft by 9 ft, studs at 16 in, a window 3 ft
        // wide from 54 in to 90 in along, its sill at 36 in and its head at 84 in.
        let f = frame(12.0, 9.0, &[[54.0 / 12.0, 90.0 / 12.0, 3.0, 7.0]], 16.0 / 12.0);
        // The layout studs centred on 64 and 80 in fall in the opening: 10 - 2 = 8.
        assert_eq!(count(&f, Piece::Stud), 8);
        assert_eq!((count(&f, Piece::King), count(&f, Piece::Jack)), (2, 2));
        // Kings at 51 to 52.5 and 91.5 to 93 in; jacks at 52.5 to 54 and 90 to 91.5.
        let at = |p: Piece| -> Vec<f64> { f.iter().filter(|m| m.piece == p).map(|m| inches(m.along[0])).collect() };
        assert_eq!(at(Piece::King), vec![51.0, 91.5]);
        assert_eq!(at(Piece::Jack), vec![52.5, 90.0]);
        // A jack runs from the plate to the head: 84 - 1.5.
        assert!(f.iter().filter(|m| m.piece == Piece::Jack).all(|m| inches(m.length()) == 82.5));
        // The header: 3 ft is a 2x6 by the rule of thumb, 5 1/2 in deep, on the jacks:
        // 52.5 to 91.5 in along (39 in), 84 to 89.5 in up.
        let header = f.iter().find(|m| m.piece == Piece::Header).expect("a header");
        assert_eq!((inches(header.along[0]), inches(header.along[1]), inches(header.up[0]), inches(header.up[1])), (52.5, 91.5, 84.0, 89.5));
        // The rough sill: the width of the opening, under it.
        let sill = f.iter().find(|m| m.piece == Piece::Sill).expect("a sill");
        assert_eq!((inches(sill.along[0]), inches(sill.along[1]), inches(sill.up[0]), inches(sill.up[1])), (54.0, 90.0, 34.5, 36.0));
        // Cripples on the two centres taken out: two below the sill (1.5 to 34.5) and
        // two above the header (89.5 to 105).
        assert_eq!(count(&f, Piece::Cripple), 4);
        let lengths: Vec<f64> = f.iter().filter(|m| m.piece == Piece::Cripple).map(|m| inches(m.length())).collect();
        assert_eq!(lengths.iter().filter(|l| **l == 33.0).count(), 2);
        assert_eq!(lengths.iter().filter(|l| **l == 15.5).count(), 2);
        // Nothing stands in the opening.
        assert!(f.iter().all(|m| !(m.along[0] < 90.0 / 12.0 && m.along[1] > 54.0 / 12.0 && m.up[0] < 7.0 && m.up[1] > 3.0)));
    }

    #[test]
    fn a_door_has_no_sill_and_nothing_under_it() {
        let f = frame(12.0, 9.0, &[[4.0, 7.0, 0.0, 6.0 + 8.0 / 12.0]], 16.0 / 12.0);
        assert_eq!(count(&f, Piece::Sill), 0);
        // Cripples above the header only.
        assert!(f.iter().filter(|m| m.piece == Piece::Cripple).all(|m| m.up[0] > 6.5));
    }

    #[test]
    fn a_framing_line_names_its_lumber_and_its_spacing() {
        let close = |a: Option<(f64, f64)>, b: (f64, f64)| a.is_some_and(|a| (a.0 - b.0).abs() < 1.0e-9 && (a.1 - b.1).abs() < 1.0e-9);
        assert!(close(named("Wall framing, 2×6 at 16 in"), (5.5 / 12.0, 16.0 / 12.0)));
        assert!(close(named("Partition framing, 2x4 at 16 in"), (3.5 / 12.0, 16.0 / 12.0)));
        assert!(close(named("2x4 studs @ 24 in. o.c."), (3.5 / 12.0, 2.0)));
        // The ways the twentieth judgment wrote it and it was not read.
        assert!(close(named("Wall framing 2 x 6 @ 24"), (5.5 / 12.0, 2.0)));
        assert!(close(named("2X4 studs 16 OC"), (3.5 / 12.0, 16.0 / 12.0)));
        assert!(close(named("wall framing 2x6 16 in o.c."), (5.5 / 12.0, 16.0 / 12.0)));
        // Words that name no framing frame nothing.
        for said in ["Cedar lap siding, installed", "Floor framing and subfloor", "Wall framing", "Framing, 2x5 at 16 in", "Roof sheathing 2x6 at 16", "Wall framing, 2x6, 20 ft lengths", "Studs 2x4, 92 5/8"] {
            assert_eq!(named(said), None, "{said}");
        }
    }

    #[test]
    fn the_frame_as_drawn_is_counted() {
        let f = frame(12.0, 9.0, &[[54.0 / 12.0, 90.0 / 12.0, 3.0, 7.0]], 16.0 / 12.0);
        let t = tally(&f);
        let of = |p: Piece| t.iter().find(|r| r.0 == p).map(|r| (r.1, (r.2 * 12.0 * 100.0).round() / 100.0));
        assert_eq!(of(Piece::Stud), Some((8, 8.0 * 103.5)));
        assert_eq!(of(Piece::TopPlate), Some((2, 288.0)));
        assert_eq!(of(Piece::Header), Some((1, 39.0)));
    }
}
