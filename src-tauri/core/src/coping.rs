// ── The coping ───────────────────────────────────────────────────────────
// What caps a wall or a curb: how wide, what gauge, how many sections, how many
// corners, and the cleat that holds it down.
//
// NRCA *Architectural Metal Flashing* 2022, §2.1, pp.55–60, is the source for
// everything typed here that is not already the job's own number.
//
//   CAP WIDTH        typed `W`, in feet; untyped, `WALL + 2 x CAP_OVERHANG_IN`
//                     (p.58 shows a cap overhanging both faces, no minimum
//                     stated — a GAP, recorded rather than invented as a rule).
//   STRETCH-OUT       stays typed against the detail (`STRETCHOUT`); the flat
//                     width of coil a profile eats is the detail's own number,
//                     never the material's (`CLAUDE.md`, D134).
//   GAUGE             off the cap's own face height (the leg the girth leaves
//                     over the cap, each side) — Fig. 2-3, p.58: 24 ga
//                     galvanized to 8 in, 22 ga 8–10 in. Past 10 in the figure
//                     has no third row ("designers may consider... breaks to
//                     stiffen... or a two-piece coping cap"), so this reports
//                     rather than invents a heavier gauge.
//   SECTIONS          10 ft typical (p.58: "generally... 10-foot... 8 to 20
//                     feet"), typed `COPING_SECTION_FT`, warned outside 8–20.
//   CORNERS           off the run's own vertices, classed outside or inside by
//                     the turn's sign against the ring's own winding. Arcs
//                     contribute none (`measures.ts:24-26`).
//   PIECES            `ceil(LF / section) + corners`.
//   CLEAT             both faces: `LF x 2`.
//   FASTEN OC         cleat and term-bar spacing, typed `FASTEN_OC`, inches.
//                     GAP: Versico's own term bar comes punched every 6 in
//                     (Manual PDF p.42), which says how the bar is made and not
//                     how many of those holes get a fastener; nothing else in
//                     the box states a spacing.

use crate::geom::{offset_ring, run_length, Point, Shape};
use crate::model::Condition;

/// A coping cap, as far as the bill and the section need it.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Coping {
    pub cap_width_feet: Option<f64>,
    pub cap_width_stated: bool,
    pub girth_inches: Option<f64>,
    /// The face height a gauge is read off: what the girth leaves over the cap,
    /// each side. `None` where there is no girth to read one off.
    pub face_height_inches: Option<f64>,
    /// 24 or 22. `None` past 10 in, which is reported rather than invented —
    /// Fig. 2-3 has no row for it.
    pub gauge_ga: Option<u32>,
    /// The cap's own two edges, centred on the wall — each the outer trace
    /// offset by half the wall thickness, then out or in again by half the
    /// cap's own width. `None` wherever `cap_width_feet` or `wall_inches` is,
    /// because a cap centred on a wall with no stated thickness has nothing to
    /// centre on.
    pub cap_outer: Option<Shape>,
    pub cap_inner: Option<Shape>,
    pub section_feet: f64,
    pub section_stated: bool,
    pub section_out_of_range: bool,
    pub corners_outside: usize,
    pub corners_inside: usize,
    pub run_feet: f64,
    pub pieces_ea: usize,
    pub cleat_feet: f64,
    pub fasten_oc_inches: f64,
    pub fasten_oc_stated: bool,
}

/// How far a coping cap overhangs the wall it caps, each side, in inches, where
/// nobody states the cap's own width. NRCA gives no minimum; recorded as a GAP
/// rather than presented as a rule.
pub const CAP_OVERHANG_DEFAULT_IN: f64 = 3.0;

/// Coping section length, feet, where nobody states one. NRCA *Architectural
/// Metal Flashing* 2022 p.58.
pub const COPING_SECTION_DEFAULT_FT: f64 = 10.0;
pub const COPING_SECTION_MIN_FT: f64 = 8.0;
pub const COPING_SECTION_MAX_FT: f64 = 20.0;

/// Cleat and termination-bar spacing, inches, where nobody states one. GAP —
/// see the module comment.
pub const FASTEN_OC_DEFAULT_IN: f64 = 12.0;

/// The gauge a cap is formed from, off its own face height. `None` past 10 in:
/// Fig. 2-3 (p.58) has no row for it.
pub fn gauge_of(face_height_inches: f64) -> Option<u32> {
    if face_height_inches <= 8.0 {
        Some(24)
    } else if face_height_inches <= 10.0 {
        Some(22)
    } else {
        None
    }
}

/// Solve the coping on one wall or curb run, or none where the wall has
/// neither a typed cap width nor a thickness to derive one from.
///
/// `closed` and `sign` are the same facts `walls.rs` already worked out for the
/// run's own solid and flashing walk — whether the trace closes on itself, and
/// which of a corner's two turns is the one that wraps AWAY from the roof — so
/// a corner is classed the same way here as the offset was signed.
pub fn solve_one(
    c: &Condition,
    outer_ring: &[Point],
    closed: bool,
    sign: f64,
    wall_inches: Option<f64>,
) -> Option<Coping> {
    let stated_width = c.property("W");
    let cap_width_feet = stated_width.or_else(|| {
        wall_inches.map(|w| (w + 2.0 * CAP_OVERHANG_DEFAULT_IN) / 12.0)
    });
    let girth_inches = c.property("STRETCHOUT");
    let face_height_inches = match (girth_inches, cap_width_feet) {
        (Some(girth), Some(width)) => Some(((girth - width * 12.0) / 2.0).max(0.0)),
        _ => None,
    };
    let gauge_ga = face_height_inches.and_then(gauge_of);

    // The cap centred on the wall: its own centreline is the outer trace
    // offset inboard by half the wall's thickness, and the cap's two edges
    // are that centreline offset out and in again by half its own width —
    // the same offset the wall's own inboard face uses, at different
    // distances, so a cap on a reflex corner is exactly as sound as the wall
    // under it is.
    let (cap_outer, cap_inner) = match (wall_inches, cap_width_feet) {
        (Some(wall), Some(width)) => {
            let centre = wall / 2.0 / 12.0;
            let half = width / 2.0;
            (
                offset_ring(outer_ring, closed, sign, centre - half),
                offset_ring(outer_ring, closed, sign, centre + half),
            )
        }
        _ => (None, None),
    };

    let section_feet = c.property("COPING_SECTION_FT").unwrap_or(COPING_SECTION_DEFAULT_FT);
    let section_stated = c.property("COPING_SECTION_FT").is_some();
    let section_out_of_range =
        section_feet < COPING_SECTION_MIN_FT || section_feet > COPING_SECTION_MAX_FT;

    let (corners_outside, corners_inside) = corners_of(outer_ring, closed, sign);
    let corners = corners_outside + corners_inside;
    let run_feet = run_length(outer_ring);
    let pieces_ea = if section_feet > 0.0 {
        (run_feet / section_feet).ceil() as usize + corners
    } else {
        corners
    };

    let fasten_oc_inches = c.property("FASTEN_OC").unwrap_or(FASTEN_OC_DEFAULT_IN);
    let fasten_oc_stated = c.property("FASTEN_OC").is_some();

    Some(Coping {
        cap_width_feet,
        cap_width_stated: stated_width.is_some(),
        girth_inches,
        face_height_inches,
        gauge_ga,
        cap_outer,
        cap_inner,
        section_feet,
        section_stated,
        section_out_of_range,
        corners_outside,
        corners_inside,
        run_feet,
        pieces_ea,
        cleat_feet: run_feet * 2.0,
        fasten_oc_inches,
        fasten_oc_stated,
    })
}

/// How many corners on a run turn the same way `sign` calls inboard (outside,
/// a convex corner a cap wraps around) against how many turn the other way
/// (inside, a reflex corner the cap is mitred into) — the run's own distinct
/// vertices, the way `measures.ts` counts them: a duplicated closing point is
/// not a second corner, and an arc (never seen on a wall trace today) would
/// contribute none.
///
/// For a simple ring traversed one consistent way, "the interior is on the
/// left of the boundary" and "the boundary is traversed the positive way" are
/// the same fact — so a corner turning the same way `sign` does is convex, and
/// one turning against it is reflex, whichever segment `sign` itself happened
/// to be read off. An open run has two ends and neither is a corner at all: a
/// cap does not mitre where the metal simply stops.
fn corners_of(ring: &[Point], closed: bool, sign: f64) -> (usize, usize) {
    let pts = {
        let mut p = ring.to_vec();
        if p.len() > 1 && p[0] == p[p.len() - 1] {
            p.pop();
        }
        p
    };
    let n = pts.len();
    if n < 3 {
        return (0, 0);
    }
    let at: Vec<usize> = if closed { (0..n).collect() } else { (1..n.saturating_sub(1)).collect() };
    let mut outside = 0;
    let mut inside = 0;
    for i in at {
        let a = pts[(i + n - 1) % n];
        let b = pts[i];
        let c = pts[(i + 1) % n];
        let turn = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
        if turn.abs() < 1e-9 {
            continue; // three points in a straight line: not a corner at all.
        }
        if turn.signum() == sign.signum() {
            outside += 1;
        } else {
            inside += 1;
        }
    }
    (outside, inside)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The demo's parapet: 8 distinct vertices, a rectangle with a rectangular
    /// bump on its north side. The two places the boundary steps INTO the bump
    /// — (50, 50) and (116, 50) — are reflex; the other six, including the two
    /// at the bump's own far corners, are convex. By hand, not typed against
    /// the solve: a ring that turns the same way six times and against itself
    /// twice is what "a rectangle with a bump" IS.
    #[test]
    fn the_demo_s_parapet_has_six_outside_corners_and_two_inside() {
        let ring = vec![
            Point::new(14.0, 50.0), Point::new(50.0, 50.0), Point::new(50.0, 42.0),
            Point::new(116.0, 42.0), Point::new(116.0, 50.0), Point::new(122.0, 50.0),
            Point::new(122.0, 134.0), Point::new(14.0, 134.0), Point::new(14.0, 50.0),
        ];
        // This ring traces its own roof, so the interior is on the left of its
        // first segment (14,50)->(50,50) — the same fact `inboard_sign` would
        // read off the job.
        let (outside, inside) = corners_of(&ring, true, 1.0);
        assert_eq!((outside, inside), (6, 2));
    }

    /// The same ring, wound the other way: the sign flips and the classing
    /// does not — a corner is convex or reflex because of its own shape, not
    /// because of which direction somebody happened to trace it in.
    #[test]
    fn winding_the_ring_backward_does_not_change_which_corners_are_which() {
        let mut ring = vec![
            Point::new(14.0, 50.0), Point::new(50.0, 50.0), Point::new(50.0, 42.0),
            Point::new(116.0, 42.0), Point::new(116.0, 50.0), Point::new(122.0, 50.0),
            Point::new(122.0, 134.0), Point::new(14.0, 134.0), Point::new(14.0, 50.0),
        ];
        ring.reverse();
        let (outside, inside) = corners_of(&ring, true, -1.0);
        assert_eq!((outside, inside), (6, 2));
    }

    /// An open run has two ends and neither is a corner: a wall drawn along
    /// one side of a building, not traced as a closed loop, mitres at its one
    /// interior bend and stops plainly at either end.
    #[test]
    fn an_open_run_s_two_ends_are_not_corners() {
        let ring = vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0), Point::new(10.0, 10.0)];
        let (outside, inside) = corners_of(&ring, false, 1.0);
        assert_eq!((outside, inside), (1, 0), "one interior bend, and neither end counted");
    }

    /// Fig. 2-3, p.58: the step is exactly at 8 and at 10 inches of face.
    #[test]
    fn gauge_steps_at_eight_and_ten_inches() {
        assert_eq!(gauge_of(7.99), Some(24));
        assert_eq!(gauge_of(8.0), Some(24));
        assert_eq!(gauge_of(8.01), Some(22));
        assert_eq!(gauge_of(10.0), Some(22));
        assert_eq!(gauge_of((10.0 + 0.01)), None, "past 10 in the figure has no row, and none is invented");
    }

    /// The demo's own cap: `W` untyped would derive to 8 + 2x3 = 14 in on the
    /// 8 in parapet, which is exactly the 14 in cap the job types — the
    /// derivation and the typed number agree because that is where the number
    /// came from.
    #[test]
    fn an_untyped_cap_width_derives_from_the_wall_and_the_overhang() {
        let c: Condition = serde_json::from_value(serde_json::json!({
            "id": "c-wall", "kind": "line",
            "properties": { "H": 3.5, "WALL": 8.0, "STRETCHOUT": 19.5 },
        }))
        .expect("a condition reads");
        let ring = vec![
            Point::new(0.0, 0.0), Point::new(20.0, 0.0),
            Point::new(20.0, 20.0), Point::new(0.0, 20.0), Point::new(0.0, 0.0),
        ];
        let coping =
            solve_one(&c, &ring, true, 1.0, c.property("WALL")).expect("a wall this tall has a cap");
        assert!(!coping.cap_width_stated);
        assert_eq!(coping.cap_width_feet, Some(14.0 / 12.0));
        // Girth 19.5 in less a 14 in cap, split two legs: 2.75 in each — under 8,
        // so 24 gauge.
        assert_eq!(coping.face_height_inches, Some(2.75));
        assert_eq!(coping.gauge_ga, Some(24));

        // Centred on the wall: the outer edge overhangs the traced line by
        // exactly `CAP_OVERHANG_IN` (3 in = 0.25 ft, since W derived FROM that
        // overhang in the first place), and the inner edge sits at half the
        // wall's own 8 in plus that same 0.25 ft inboard of it — 11/12 ft.
        let outer = coping.cap_outer.expect("a wall with a wall thickness gets a cap outer edge");
        let inner = coping.cap_inner.expect("and an inner edge");
        let box_of = |shape: &crate::geom::Shape| crate::geom::bounds(&shape.ring).unwrap();
        let (ob, ib) = (box_of(&outer), box_of(&inner));
        assert!((ob.min_x - (-0.25)).abs() < 1e-9, "the cap's outer edge came to {}", ob.min_x);
        assert!((ib.min_x - 11.0 / 12.0).abs() < 1e-9, "the cap's inner edge came to {}", ib.min_x);
    }
}
