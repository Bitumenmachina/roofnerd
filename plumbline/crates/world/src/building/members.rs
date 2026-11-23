//! Joists and rafters counted across a floor and a roof plane (call 165).
//!
//! Ours. BIMCraft lays out nothing across a floor or a roof (`research/structure-port-survey.md`:
//! only `timber.rs`, one wall, ported here as `framing::frame`), so the layout is the framer's
//! own rule, JLC's "joists = floor width / spacing + 1", and its stations are laid the way
//! `timber.rs` lays a wall's studs: on multiples of the spacing from one end.
//!
//! `members_across` takes an outline on plan, the way the members run and a spacing, and
//! gives the length on plan of every member. One function for a floor's joists and a roof
//! plane's rafters; a rafter's length on the slope is its length here times the plane's
//! slope factor, which is the caller's. Nothing is sized: the lumber is the priced line's
//! words, typed, and is not looked at here. Hangers, blocking, bridging, doubled joists under
//! a partition and the headers round a stair are not in the count.
//!
//! The rule:
//! * A member is 1 1/2 in thick. Its middle is 3/4 in past its layout mark.
//! * An **edge member** lies 3/4 in inside every edge of the outline that runs with the
//!   members (a rim or end joist; the rafter at a rake), as long as that edge.
//! * **Marks** are at each multiple of the spacing from the outline's first side, square to
//!   the members (its west or south end first); the member at a mark is the line there cut to
//!   the outline: one piece for each length of it that lies inside.
//! * A piece that lies within a member's thickness of an edge member beside it, over the same
//!   length, is that edge member and is not counted twice.
//! * A piece under 6 in long is not a piece: it is left out, and the figure says how many.

use crate::components::P2;

/// How thick a member is: 1 1/2 in.
pub const MEMBER_THICKNESS_FT: f64 = 1.5 / 12.0;
/// How long a piece has to be to be one: 6 in.
pub const SHORTEST_PIECE_FT: f64 = 0.5;

/// What was laid across an outline.
#[derive(Clone, Debug, PartialEq)]
pub struct Across {
    /// The length on plan of each member, edge members first, then by their marks.
    pub pieces: Vec<f64>,
    /// How many were under 6 in and left out.
    pub left_out: usize,
}

impl Across {
    pub fn count(&self) -> usize {
        self.pieces.len()
    }
    pub fn total(&self) -> f64 {
        self.pieces.iter().sum()
    }
    pub fn longest(&self) -> f64 {
        self.pieces.iter().copied().fold(0.0, f64::max)
    }
}

const EPS: f64 = 1.0e-9;

/// The unit vector square to `u` that points east, or north where `u` points that way.
fn across(u: P2) -> P2 {
    let v = [-u[1], u[0]];
    if v[0] < -EPS || (v[0].abs() <= EPS && v[1] < 0.0) {
        [-v[0], -v[1]]
    } else {
        v
    }
}

fn unit(run: P2) -> Result<P2, String> {
    let long = run[0].hypot(run[1]);
    if !(long.is_finite() && long > EPS) {
        return Err("the way the members run is not a direction".into());
    }
    Ok([run[0] / long, run[1] / long])
}

/// Whether a point is inside an outline given as (along, across) points: crossings of a
/// line along `a` at its `across`, counted.
fn inside(ring: &[P2], a: f64, b: f64) -> bool {
    let mut inside = false;
    for i in 0..ring.len() {
        let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
        if (p[1] > b) != (q[1] > b) && a < p[0] + (b - p[1]) / (q[1] - p[1]) * (q[0] - p[0]) {
            inside = !inside;
        }
    }
    inside
}

/// The length of what is square to the members: the outline's edges measured across them
/// (a floor's rim: the band the joist ends meet).
pub fn rim_length(outline: &[P2], run: P2) -> Result<f64, String> {
    let u = unit(run)?;
    let v = across(u);
    Ok((0..outline.len()).map(|i| {
        let (p, q) = (outline[i], outline[(i + 1) % outline.len()]);
        ((q[0] - p[0]) * v[0] + (q[1] - p[1]) * v[1]).abs()
    }).sum())
}

/// Members across `outline` (feet on the plan, either way round) running along `run`, on
/// marks `spacing` apart from the outline's first side.
pub fn members_across(outline: &[P2], run: P2, spacing: f64) -> Result<Across, String> {
    let (laid, left_out) = members_laid(outline, run, spacing)?;
    Ok(Across { pieces: laid.iter().map(|m| m.0).collect(), left_out })
}

/// The members `members_across` counts, each placed: its length on plan and its two ends on
/// the plan, in the order it counts them (edge members first, then by their marks), and how
/// many were under 6 in and left out. A window draws these; it works out none of them.
pub fn members_placed(outline: &[P2], run: P2, spacing: f64) -> Result<(Vec<[P2; 2]>, usize), String> {
    let (laid, left_out) = members_laid(outline, run, spacing)?;
    Ok((laid.iter().map(|m| [m.1, m.2]).collect(), left_out))
}

type Laid = (f64, P2, P2);

fn members_laid(outline: &[P2], run: P2, spacing: f64) -> Result<(Vec<Laid>, usize), String> {
    if outline.len() < 3 {
        return Err("an outline has at least 3 corners".into());
    }
    if !(spacing.is_finite() && spacing > EPS) {
        return Err("the spacing is not a length of more than nothing".into());
    }
    let u = unit(run)?;
    let v = across(u);
    // The outline as (along the members, across them).
    let ring: Vec<P2> = outline.iter().map(|p| [p[0] * u[0] + p[1] * u[1], p[0] * v[0] + p[1] * v[1]]).collect();
    let (low, high) = ring.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| (lo.min(p[1]), hi.max(p[1])));
    let half = MEMBER_THICKNESS_FT / 2.0;
    let mut pieces: Vec<Laid> = Vec::new();
    let mut left_out = 0;
    // A piece lies along the members at `across`, from `lo` to `hi` along them.
    let back = |lo: f64, hi: f64, at: f64| -> (P2, P2) { ([u[0] * lo + v[0] * at, u[1] * lo + v[1] * at], [u[0] * hi + v[0] * at, u[1] * hi + v[1] * at]) };
    let mut keep = |lo: f64, hi: f64, at: f64, pieces: &mut Vec<Laid>| {
        if hi - lo < SHORTEST_PIECE_FT - EPS {
            left_out += 1;
        } else {
            let (from, to) = back(lo, hi, at);
            pieces.push((hi - lo, from, to));
        }
    };
    // Edge members: each edge that runs with the members, 3/4 in inside, as long as the edge.
    let mut edges: Vec<(f64, f64, f64)> = Vec::new();
    for i in 0..ring.len() {
        let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
        if (p[1] - q[1]).abs() > 1.0e-7 || (p[0] - q[0]).abs() <= EPS {
            continue;
        }
        let (lo, hi) = (p[0].min(q[0]), p[0].max(q[0]));
        let middle = (lo + hi) / 2.0;
        // Inside is the side the outline lies on.
        let b = if inside(&ring, middle, p[1] + 1.0e-4) { p[1] + half } else { p[1] - half };
        edges.push((b, lo, hi));
    }
    for (at, lo, hi) in &edges {
        keep(*lo, *hi, *at, &mut pieces);
    }
    // Marks, from the first side: the line there cut to the outline.
    let mut k = 0usize;
    loop {
        let b = low + k as f64 * spacing + half;
        k += 1;
        if b >= high - EPS {
            break;
        }
        let mut cuts: Vec<f64> = Vec::new();
        for i in 0..ring.len() {
            let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
            if (p[1] > b) != (q[1] > b) {
                cuts.push(p[0] + (b - p[1]) / (q[1] - p[1]) * (q[0] - p[0]));
            }
        }
        cuts.sort_by(f64::total_cmp);
        for pair in cuts.chunks_exact(2) {
            let length = pair[1] - pair[0];
            if length <= EPS {
                continue;
            }
            // An edge member is that piece, and is counted once.
            if edges.iter().any(|(eb, lo, hi)| (b - eb).abs() <= MEMBER_THICKNESS_FT + EPS && ((hi - lo) - length).abs() < 1.0e-6 && (lo - pair[0]).abs() < 1.0e-6) {
                continue;
            }
            keep(pair[0], pair[1], b, &mut pieces);
        }
        if k > 100_000 {
            return Err("that many members are not a floor or a roof: check the spacing".into());
        }
    }
    Ok((pieces, left_out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1.0e-9
    }

    /// 24 ft by 16 ft at 16 in: joists across the short way. Edge joists 0.0625 ft inside the
    /// west and east edges; marks every 1.3333 ft from the west, 1 to 17 (the mark at 0 is the
    /// west edge joist; the one at 24.0625 is past the east edge): 19 joists, JLC's
    /// 24 / 1.3333 + 1 = 19, each 16 ft: 304 LF; the rim 2 x 24 = 48 LF.
    #[test]
    fn a_floor_24_by_16_at_16_in_has_19_joists_of_16_ft_and_48_ft_of_rim_the_short_way() {
        let floor = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
        let a = members_across(&floor, [0.0, 1.0], 16.0 / 12.0).unwrap();
        assert_eq!(a.count(), 19);
        assert!(near(a.total(), 304.0) && near(a.longest(), 16.0) && a.left_out == 0);
        assert!(near(rim_length(&floor, [0.0, 1.0]).unwrap(), 48.0));
    }

    /// The long way: marks along the 16 ft, 11 of them (1.3958 to 14.7292), and two edge
    /// joists: 13 joists of 24 ft, 312 LF; the rim 2 x 16 = 32 LF.
    #[test]
    fn the_same_floor_the_long_way_has_13_joists_of_24_ft_and_32_ft_of_rim() {
        let floor = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
        let a = members_across(&floor, [1.0, 0.0], 16.0 / 12.0).unwrap();
        assert_eq!(a.count(), 13);
        assert!(near(a.total(), 312.0) && near(a.longest(), 24.0));
        assert!(near(rim_length(&floor, [1.0, 0.0]).unwrap(), 32.0));
    }

    /// The L: 0,0; 24,0; 24,10; 12,10; 12,16; 0,16, joists running north. Edge joists inside
    /// the west edge (16 ft), the east edge (10 ft) and the edge at east 12 (6 ft, at 11.9375).
    /// Marks 1 to 8 are 16 ft (8 joists), marks 9 to 17 are 10 ft (9 joists): 20 joists,
    /// (1 + 8) x 16 + 6 + (9 + 1) x 10 = 144 + 6 + 100 = 250 LF; rim 24 + 12 + 12 = 48 LF.
    #[test]
    fn an_l_shaped_floor_has_20_joists_250_ft_and_48_ft_of_rim() {
        let l = [[0.0, 0.0], [24.0, 0.0], [24.0, 10.0], [12.0, 10.0], [12.0, 16.0], [0.0, 16.0]];
        let a = members_across(&l, [0.0, 1.0], 16.0 / 12.0).unwrap();
        assert_eq!(a.count(), 20);
        assert!(near(a.total(), 250.0) && near(a.longest(), 16.0), "{a:?}");
        assert!(near(rim_length(&l, [0.0, 1.0]).unwrap(), 48.0));
    }

    /// A gable plane 24 ft along the eave and 8 ft up the slope: an edge rafter at each rake and
    /// marks 1 to 11 at 24 in: 13 rafters of 8 ft on plan (24 / 2 + 1).
    #[test]
    fn a_gable_plane_has_13_rafters_of_8_ft_on_plan() {
        let plane = [[0.0, 0.0], [24.0, 0.0], [24.0, 8.0], [0.0, 8.0]];
        let a = members_across(&plane, [0.0, 1.0], 2.0).unwrap();
        assert_eq!(a.count(), 13);
        assert!(near(a.total(), 104.0) && a.left_out == 0);
    }

    /// The long hip plane (eave 24, ridge 8): no edge runs with the rafters; the rafter at x is
    /// min(x, 8, 24 - x) on plan. Marks 0.0625, 2.0625, ... 22.0625: the first is under 6 in and
    /// left out and said; 11 rafters, 63.9375 ft on plan. The short plane (a triangle on 16 ft):
    /// 2.0625 to 7.9375 and back to 1.9375: 7 rafters, 31.9375 ft, and the one at 0.0625 left out.
    #[test]
    fn the_planes_of_a_hip_roof_have_no_edge_rafters_and_leave_out_the_sliver_at_the_corner() {
        let long = [[0.0, 0.0], [24.0, 0.0], [16.0, 8.0], [8.0, 8.0]];
        let a = members_across(&long, [0.0, 1.0], 2.0).unwrap();
        assert_eq!((a.count(), a.left_out), (11, 1));
        assert!(near(a.total(), 63.9375), "{a:?}");
        let short = [[0.0, 0.0], [16.0, 0.0], [8.0, 8.0]];
        let b = members_across(&short, [0.0, 1.0], 2.0).unwrap();
        assert_eq!((b.count(), b.left_out), (7, 1));
        assert!(near(b.total(), 31.9375), "{b:?}");
    }

    #[test]
    fn what_is_not_an_outline_or_a_spacing_is_refused() {
        let floor = [[0.0, 0.0], [24.0, 0.0], [24.0, 16.0], [0.0, 16.0]];
        assert!(members_across(&floor[..2], [0.0, 1.0], 1.0).is_err());
        assert!(members_across(&floor, [0.0, 1.0], 0.0).is_err());
        assert!(members_across(&floor, [0.0, 0.0], 1.0).is_err());
    }
}
