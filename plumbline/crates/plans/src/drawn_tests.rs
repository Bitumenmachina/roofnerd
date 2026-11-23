//! What call 181 decided about walls found on a drawn plan, each as a test on a made-up sheet with the
//! figure worked by hand. A sheet here is a list of lines in feet; `page` turns it to what a PDF's
//! reader hands over (points from the page's lower-left corner) at a scale. Nothing is a real plan.

use crate::drawn::*;

type P = [f64; 2];
const IN: f64 = 1.0 / 12.0;
/// Feet to the point at 1/4 in to the foot (18 points to the foot), 1/8, 1/2, 1/16.
const Q4: f64 = 1.0 / 18.0;
const Q8: f64 = 1.0 / 9.0;
const Q2: f64 = 1.0 / 36.0;
const Q16: f64 = 1.0 / 4.5;

#[derive(Clone, Default)]
struct Sheet {
    l: Vec<[P; 2]>,
    /// Lines drawn dashed (call 186): they go to the finder after the solid ones, marked.
    d: Vec<[P; 2]>,
    /// Places where the page cut a line off (the edge of what shows), in the sheet's own feet.
    cuts: Vec<P>,
}

impl Sheet {
    fn line(&mut self, a: P, b: P) -> &mut Self {
        self.l.push([a, b]);
        self
    }
    /// Four lines round a rectangle, the faces of a closed ring of walls `t` thick whose centre line is the rectangle.
    fn ring(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, t: f64) -> &mut Self {
        for h in [t / 2.0, -t / 2.0] {
            let (a, b, c, d) = (x0 - h, y0 - h, x1 + h, y1 + h);
            self.line([a, b], [c, b]).line([c, b], [c, d]).line([c, d], [a, d]).line([a, d], [a, b]);
        }
        self
    }
    /// Takes `lo`..`hi` out of every line lying on the line through `a` and `b`, measured from `a`.
    fn cut(&mut self, a: P, b: P, lo: f64, hi: f64) -> &mut Self {
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let u = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
        let on = |p: P| ((p[0] - a[0]) * -u[1] + (p[1] - a[1]) * u[0]).abs() < 1e-6;
        let t = |p: P| (p[0] - a[0]) * u[0] + (p[1] - a[1]) * u[1];
        let at = |s: f64| [a[0] + u[0] * s, a[1] + u[1] * s];
        let mut out = Vec::new();
        for l in std::mem::take(&mut self.l) {
            if !(on(l[0]) && on(l[1])) {
                out.push(l);
                continue;
            }
            let (s0, s1) = (t(l[0]).min(t(l[1])), t(l[0]).max(t(l[1])));
            if s1 <= lo || s0 >= hi {
                out.push(l);
                continue;
            }
            if s0 < lo {
                out.push([at(s0), at(lo)]);
            }
            if s1 > hi {
                out.push([at(hi), at(s1)]);
            }
        }
        self.l = out;
        self
    }
    /// An opening in the wall along a to b, whose centre line it is, from `lo` to `hi` along it: both faces broken,
    /// jambs across, and a door's leaf line down the middle or a window's glass (faces and a middle line).
    fn opening(&mut self, a: P, b: P, t: f64, lo: f64, hi: f64, window: bool) -> &mut Self {
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let u = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
        let n = [-u[1], u[0]];
        for h in [t / 2.0, -t / 2.0] {
            let (pa, pb) = ([a[0] + n[0] * h, a[1] + n[1] * h], [b[0] + n[0] * h, b[1] + n[1] * h]);
            self.cut(pa, pb, lo - 0.0, hi);
        }
        let p = |s: f64, o: f64| [a[0] + u[0] * s + n[0] * o, a[1] + u[1] * s + n[1] * o];
        for s in [lo, hi] {
            self.line(p(s, -t / 2.0), p(s, t / 2.0));
        }
        let offs: Vec<f64> = if window { vec![-t / 2.0, 0.0, t / 2.0] } else { vec![0.0] };
        for o in offs {
            self.line(p(lo, o), p(hi, o));
        }
        self
    }
    /// A door whose faces run on past its jambs into the opening by `over` (the jamb line stands inside the end of each piece).
    fn opening_over(&mut self, a: P, b: P, t: f64, lo: f64, hi: f64, over: f64) -> &mut Self {
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let u = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
        let n = [-u[1], u[0]];
        for h in [t / 2.0, -t / 2.0] {
            self.cut([a[0] + n[0] * h, a[1] + n[1] * h], [b[0] + n[0] * h, b[1] + n[1] * h], lo + over, hi - over);
        }
        let p = |s: f64, o: f64| [a[0] + u[0] * s + n[0] * o, a[1] + u[1] * s + n[1] * o];
        for s in [lo, hi] {
            self.line(p(s, -t / 2.0), p(s, t / 2.0));
        }
        self.line(p(lo, 0.0), p(hi, 0.0));
        self
    }
    /// A window as most sheets draw it: both faces broken, the jambs across, and two lines of glass between them, each
    /// 1/6 of the wall's thickness off the centre line (less than 3 in apart, so they are no wall).
    fn glass(&mut self, a: P, b: P, t: f64, lo: f64, hi: f64) -> &mut Self {
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let u = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
        let n = [-u[1], u[0]];
        for h in [t / 2.0, -t / 2.0] {
            self.cut([a[0] + n[0] * h, a[1] + n[1] * h], [b[0] + n[0] * h, b[1] + n[1] * h], lo, hi);
        }
        let p = |s: f64, o: f64| [a[0] + u[0] * s + n[0] * o, a[1] + u[1] * s + n[1] * o];
        for s in [lo, hi] {
            self.line(p(s, -t / 2.0), p(s, t / 2.0));
        }
        for o in [t / 6.0, -t / 6.0] {
            self.line(p(lo, o), p(hi, o));
        }
        self
    }
    /// A pair of lines for a partition from (x, y0) to (x, y1), `t` thick, ending on the faces of the walls.
    fn partition(&mut self, x: f64, y0: f64, y1: f64, t: f64) -> &mut Self {
        self.line([x - t / 2.0, y0], [x - t / 2.0, y1]).line([x + t / 2.0, y0], [x + t / 2.0, y1])
    }
    /// A dimension string `off` outside the line from a to b with a tick at each end and extension lines.
    fn dimension(&mut self, a: P, b: P, off: f64, tick: f64) -> &mut Self {
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let u = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
        let n = [-u[1], u[0]];
        let p = |e: P, o: f64| [e[0] + n[0] * o, e[1] + n[1] * o];
        self.line(p(a, off), p(b, off));
        for e in [a, b] {
            self.line(p(e, off * 0.2), p(e, off * 1.1));
            let c = p(e, off);
            let s = tick / 2.0 * std::f64::consts::FRAC_1_SQRT_2;
            let d = [u[0] * s + n[0] * s, u[1] * s + n[1] * s];
            self.line([c[0] - d[0], c[1] - d[1]], [c[0] + d[0], c[1] + d[1]]);
        }
        self
    }
    /// A dashed line: kept whole by the reader and marked.
    fn dash(&mut self, a: P, b: P) -> &mut Self {
        self.d.push([a, b]);
        self
    }
    fn turned(&self, degrees: f64) -> Sheet {
        let (s, c) = degrees.to_radians().sin_cos();
        let turn = |p: P| [p[0] * c - p[1] * s, p[0] * s + p[1] * c];
        Sheet { l: self.l.iter().map(|l| l.map(turn)).collect(), d: self.d.iter().map(|l| l.map(turn)).collect(), cuts: self.cuts.iter().map(|p| turn(*p)).collect() }
    }
    fn lowest(&self) -> P {
        self.l.iter().chain(&self.d).flatten().fold([f64::MAX, f64::MAX], |m, p| [m[0].min(p[0]), m[1].min(p[1])])
    }
    /// What a reader hands over: points from the page's corner, the plan's zero put 126 points in from it;
    /// the solid lines first, the dashed ones after them.
    fn page(&self, k: f64) -> Vec<[[f64; 2]; 2]> {
        let lo = self.lowest();
        self.l.iter().chain(&self.d).map(|l| l.map(|p| [(p[0] - lo[0]) / k + 126.0, (p[1] - lo[1]) / k + 126.0])).collect()
    }
    fn found(&self, k: f64) -> Drawn {
        let lo = self.lowest();
        let dashed: Vec<usize> = (self.l.len()..self.l.len() + self.d.len()).collect();
        let cuts: Vec<P> = self.cuts.iter().map(|p| [(p[0] - lo[0]) / k + 126.0, (p[1] - lo[1]) / k + 126.0]).collect();
        find_on(&self.page(k), Some(k), [3000.0, 2400.0], [0.0, 0.0], &Extra { dashed: &dashed, cuts: &cuts })
    }
}

/// The horizontal wall lowest on the sheet (the south wall) and the vertical one farthest east.
fn south(d: &Drawn) -> &Wall {
    d.walls.iter().filter(|w| (w.to[1] - w.from[1]).abs() < 1e-6).min_by(|a, b| a.from[1].total_cmp(&b.from[1])).expect("a south wall")
}
fn east(d: &Drawn) -> &Wall {
    d.walls.iter().filter(|w| (w.to[0] - w.from[0]).abs() < 1e-6).max_by(|a, b| a.from[0].total_cmp(&b.from[0])).expect("an east wall")
}

fn near(a: f64, b: f64, by: f64) -> bool {
    (a - b).abs() <= by
}

/// The lengths of the walls found that are not in doubt, sorted.
fn lengths(d: &Drawn) -> Vec<f64> {
    let mut v: Vec<f64> = d.walls.iter().filter(|w| w.doubt.is_none()).map(|w| w.length()).collect();
    v.sort_by(|a, b| a.total_cmp(b));
    v
}

fn assert_lengths(d: &Drawn, want: &[f64], by: f64) {
    let got = lengths(d);
    let mut want = want.to_vec();
    want.sort_by(|a, b| a.total_cmp(b));
    assert!(got.len() == want.len() && got.iter().zip(&want).all(|(g, w)| near(*g, *w, by)), "walls {got:?}, wanted {want:?}\n{d:#?}");
}

/// The house the tests share: 36 by 16 ft of 6 in walls, a partition 4 1/2 in thick at 18 ft, a door 3 ft
/// wide in the east wall (6 ft 6 in to 9 ft 6 in north) and a window 5 ft wide in the south wall (8 to 13 ft).
fn house() -> Sheet {
    let t = 0.5;
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 36.0, 16.0, t);
    s.cut([0.0, 0.25], [36.0, 0.25], 17.75, 18.25);
    s.cut([0.0, 15.75], [36.0, 15.75], 17.75, 18.25);
    s.partition(18.0, 0.25, 15.75, 0.375);
    s.opening([36.0, 0.0], [36.0, 16.0], t, 6.5, 9.5, false);
    s.opening([0.0, 0.0], [36.0, 0.0], t, 8.0, 13.0, true);
    s
}

// ---- the rules, one at a time ----

#[test]
fn a_sheet_with_no_scale_finds_nothing_and_says_so() {
    let d = find(&house().page(Q4), None, [3000.0, 2400.0], [0.0, 0.0]);
    assert!(d.no_scale && d.walls.is_empty());
}

#[test]
fn two_lines_a_thickness_apart_that_meet_others_are_walls_and_the_room_closes() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 20.0, 12.0, 0.5);
    // A ring is two boxes one thickness apart: walls, not a box on the sheet.
    let d = s.found(Q4);
    assert_lengths(&d, &[12.0, 12.0, 20.0, 20.0], 1.0 / 24.0);
    assert!(d.walls.iter().all(|w| near(w.thickness, 0.5, 1.0 / 96.0)), "{d:#?}");
    assert!(d.open_ends.is_empty(), "the four walls close: {:?}", d.open_ends);
    assert!(d.boxes.is_empty());
}

#[test]
fn a_line_across_the_two_lines_ends_a_wall_there() {
    // A south wall of 6 in with a line across it at 12 ft, where new work meets existing.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 12.0, 0.5);
    s.line([12.0, -0.25], [12.0, 0.25]);
    let d = s.found(Q4);
    assert_lengths(&d, &[12.0, 12.0, 18.0, 30.0, 12.0], 1.0 / 24.0);
    // Both halves meet the wall at their ends: no end is open.
    assert!(d.open_ends.is_empty(), "{:?}", d.open_ends);
}

#[test]
fn a_line_across_too_near_the_end_does_not_end_a_wall_and_is_counted() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 12.0, 0.5);
    s.line([1.0, -0.25], [1.0, 0.25]);
    let d = s.found(Q4);
    assert_lengths(&d, &[12.0, 12.0, 30.0, 30.0], 1.0 / 24.0);
    assert_eq!(d.counts.crosses_ignored, 1);
}

#[test]
fn three_or_more_evenly_spaced_lines_are_not_walls_and_are_counted_as_a_set() {
    // Twelve treads 10 in apart, 3 ft long: 13 lines.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    for i in 0..13 {
        s.line([10.0, 4.0 + i as f64 * 10.0 * IN], [13.0, 4.0 + i as f64 * 10.0 * IN]);
    }
    let d = s.found(Q4);
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 24.0);
    assert_eq!(d.sets.len(), 1, "{:?}", d.sets);
    assert_eq!(d.sets[0].lines, 13);
    assert!(near(d.sets[0].spacing, 10.0 * IN, 0.01));
    assert!(d.walls.iter().all(|w| w.doubt.is_none()));
}

#[test]
fn three_lines_six_inches_apart_are_a_set_not_a_wall() {
    // The contract's rule as written: three evenly spaced lines are not walls. (A wall with a lone line down
    // its middle reads so; it is said, with where.)
    let mut s = Sheet::default();
    for i in 0..3 {
        s.line([0.0, i as f64 * 0.5], [8.0, i as f64 * 0.5]);
    }
    let d = s.found(Q4);
    assert!(d.walls.is_empty(), "{d:#?}");
    assert_eq!(d.sets.len(), 1);
}

#[test]
fn a_pair_that_meets_nothing_is_in_doubt_and_said_where() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    // A beam, or a counter's two edges, 8 ft long and 4 in apart in the middle of the room.
    s.line([10.0, 8.0], [18.0, 8.0]).line([10.0, 8.0 + 4.0 * IN], [18.0, 8.0 + 4.0 * IN]);
    let d = s.found(Q4);
    let doubted: Vec<&Wall> = d.walls.iter().filter(|w| w.doubt.is_some()).collect();
    assert_eq!(doubted.len(), 1, "{d:#?}");
    let why = doubted[0].doubt.as_deref().unwrap();
    assert!(why.contains("meet no other wall at either end"), "{why}");
    // Where it is is on the wall, not in its reason: the middle of it, 14 ft along the room from its corner, plus the sheet's offset.
    assert!(near((doubted[0].from[0] + doubted[0].to[0]) / 2.0, 21.25, 0.05), "{doubted:?}");
    assert!(near(doubted[0].thickness, 4.0 * IN, 1.0 / 96.0) && near(doubted[0].length(), 8.0, 0.1));
    assert_eq!(d.walls.iter().filter(|w| w.doubt.is_none()).count(), 4);
}

#[test]
fn a_break_in_both_lines_is_an_opening_and_a_line_down_it_makes_it_a_door() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    s.opening([30.0, 0.0], [30.0, 16.0], 0.5, 5.0, 8.0, false);
    let d = s.found(Q4);
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 24.0);
    let east_wall = east(&d);
    assert_eq!(east_wall.openings.len(), 1, "{east_wall:?}");
    let o = &east_wall.openings[0];
    assert!(!o.window && near(o.from, 5.0, 1.0 / 12.0) && near(o.to, 8.0, 1.0 / 12.0), "{o:?}");
}

#[test]
fn thin_lines_across_the_break_inside_the_wall_make_it_a_window() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    // Glass drawn as the two faces across and a line between, and also as two lines between the broken faces.
    s.opening([0.0, 0.0], [30.0, 0.0], 0.5, 8.0, 13.0, true);
    let d = s.found(Q4);
    let wall = south(&d);
    assert_eq!(wall.openings.len(), 1, "{wall:?}");
    assert!(wall.openings[0].window && near(wall.openings[0].from, 8.0, 1.0 / 12.0) && near(wall.openings[0].to, 13.0, 1.0 / 12.0), "{wall:?}");
    let mut s2 = Sheet::default();
    s2.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    s2.opening([0.0, 0.0], [30.0, 0.0], 0.5, 8.0, 13.0, false);
    s2.line([8.0, 0.1], [13.0, 0.1]).line([8.0, -0.1], [13.0, -0.1]);
    let d2 = s2.found(Q4);
    let wall = south(&d2);
    assert!(wall.openings.len() == 1 && wall.openings[0].window, "{wall:?}");
}

#[test]
fn lettering_and_dimension_lines_are_not_walls() {
    // At 1/16 in to the foot the stems of an H stand 13 in apart at scale; and a dimension string with
    // its ticks lies 8 in off a wall at 1/2 in to the foot.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    for i in 0..30 {
        let x = 4.0 + i as f64 * 0.5;
        s.line([x, 20.0], [x, 20.0 + 1.5]).line([x + 1.1, 20.0], [x + 1.1, 21.5]).line([x, 20.75], [x + 1.1, 20.75]);
    }
    let d = s.found(Q16);
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 12.0);
    assert!(d.walls.iter().all(|w| w.doubt.is_none()));
    let mut t = Sheet::default();
    t.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    // A string 8 in below the south wall's outer face, its ticks 7 pt (1/2 in scale) long, with extension lines.
    t.dimension([0.0, -0.25], [30.0, -0.25], -8.0 * IN, 7.0 * Q2);
    let d = t.found(Q2);
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 12.0);
    assert!(d.walls.iter().all(|w| w.doubt.is_none()), "{d:#?}");
    assert!(d.counts.dimension >= 5, "{:?}", d.counts);
}

// ---- the cases a judge will try ----

/// What the shared house holds, found: five walls, the door and the window.
fn check_house(d: &Drawn, what: &str) {
    assert!(d.walls.iter().all(|w| w.doubt.is_none()), "{what}: {d:#?}");
    assert_lengths(d, &[15.625 + 0.375, 16.0, 16.0, 36.0, 36.0], 1.0 / 12.0);
    assert!(d.open_ends.is_empty(), "{what}: the walls close: {:?}", d.open_ends);
    let opens: Vec<(bool, f64, f64)> = d.walls.iter().flat_map(|w| w.openings.iter().map(|o| (o.window, o.from, o.to))).collect();
    assert_eq!(opens.len(), 2, "{what}: {opens:?}");
    assert!(opens.iter().any(|o| !o.0 && near(o.1, 6.5, 1.0 / 12.0) && near(o.2, 9.5, 1.0 / 12.0)), "{what}: a door 6 ft 6 in to 9 ft 6 in: {opens:?}");
    assert!(opens.iter().any(|o| o.0 && near(o.1, 8.0, 1.0 / 12.0) && near(o.2, 13.0, 1.0 / 12.0)), "{what}: a window 8 to 13 ft: {opens:?}");
    // The partition's thickness and the others'.
    let mut th: Vec<f64> = d.walls.iter().map(|w| w.thickness).collect();
    th.sort_by(|a, b| a.total_cmp(b));
    assert!(near(th[0], 0.375, 1.0 / 96.0) && near(th[4], 0.5, 1.0 / 96.0), "{what}: {th:?}");
}

#[test]
fn the_house_with_a_partition_a_door_and_a_window_is_found_at_four_scales_and_turned() {
    for (k, name) in [(Q4, "1/4"), (Q8, "1/8"), (Q2, "1/2")] {
        for turn in [0.0, 17.0, 33.0, 90.0, 123.0] {
            let d = house().turned(turn).found(k);
            check_house(&d, &format!("{name} in to the foot, turned {turn}"));
        }
    }
}

#[test]
fn walls_three_and_a_half_and_twelve_inches_thick_on_one_sheet() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 24.0, 16.0, 1.0);
    s.line([12.0, 0.5], [12.0, 15.5]).line([12.0 + 3.5 * IN, 0.5], [12.0 + 3.5 * IN, 15.5]);
    // The inner faces are broken by the partition.
    s.cut([0.0, 0.5], [24.0, 0.5], 12.0 - 0.01, 12.0 + 3.5 * IN + 0.01);
    s.cut([0.0, 15.5], [24.0, 15.5], 12.0 - 0.01, 12.0 + 3.5 * IN + 0.01);
    for turn in [0.0, 33.0] {
        let d = s.turned(turn).found(Q4);
        assert_eq!(d.walls.len(), 5, "{d:#?}");
        let mut th: Vec<f64> = d.walls.iter().map(|w| w.thickness).collect();
        th.sort_by(|a, b| a.total_cmp(b));
        assert!(near(th[0], 3.5 * IN, 1.0 / 96.0) && near(th[1], 1.0, 1.0 / 96.0) && near(th[4], 1.0, 1.0 / 96.0), "{th:?}");
        assert!(d.walls.iter().all(|w| w.doubt.is_none()));
    }
}

#[test]
fn a_wall_drawn_as_a_filled_shape_and_as_two_lines_is_the_same_wall() {
    // A filled bar's edges, and the same lines stroked over them: every line twice.
    let mut once = Sheet::default();
    once.ring(0.0, 0.0, 20.0, 12.0, 0.5);
    let mut twice = once.clone();
    twice.l.extend(once.l.clone());
    let (a, b) = (once.found(Q4), twice.found(Q4));
    assert_lengths(&b, &lengths(&a), 1.0e-6);
    assert_eq!(a.walls.len(), b.walls.len());
}

#[test]
fn hatching_inside_the_walls_does_not_change_them() {
    let mut s = house();
    // Diagonal poché across the south wall's thickness every 8 in, and a line down the middle of the east wall at 6 ft... no: ticks only.
    let mut x = 0.0;
    while x < 36.0 {
        s.line([x, -0.25], [x + 0.5, 0.25]);
        x += 8.0 * IN;
    }
    // And across the whole north wall, square to it, every 8 in: too near one another to end a wall.
    let mut x = 0.3;
    while x < 36.0 {
        s.line([x, 15.75], [x, 16.25]);
        x += 8.0 * IN;
    }
    let d = s.found(Q4);
    check_house(&d, "hatched");
    assert!(d.counts.crosses_ignored > 0);
}

#[test]
fn a_door_drawn_with_its_leaf_and_its_swing_is_a_door() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    s.opening([0.0, 0.0], [30.0, 0.0], 0.5, 10.0, 13.0, false);
    // The leaf, open, square to the wall from the hinge, and the swing as 16 short straight pieces.
    s.line([10.0, 0.25], [10.0, 3.25]);
    let n = 16;
    for i in 0..n {
        let a0 = std::f64::consts::FRAC_PI_2 * i as f64 / n as f64;
        let a1 = std::f64::consts::FRAC_PI_2 * (i + 1) as f64 / n as f64;
        s.line([10.0 + 3.0 * a0.sin(), 0.25 + 3.0 * a0.cos()], [10.0 + 3.0 * a1.sin(), 0.25 + 3.0 * a1.cos()]);
    }
    let d = s.found(Q4);
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 24.0);
    let wall = south(&d);
    assert!(wall.openings.len() == 1 && !wall.openings[0].window, "{wall:?}");
}

#[test]
fn a_partition_meeting_a_wall_is_a_t_and_the_wall_goes_on_through_it() {
    let d = house().found(Q4);
    // The south wall is one wall 36 ft long, the north wall one wall, the partition ends at their lines.
    let part = d.walls.iter().find(|w| near(w.thickness, 0.375, 1.0 / 96.0)).expect("a partition");
    assert!(near(part.length(), 16.0, 1.0 / 12.0), "to the walls' centre lines: {part:?}");
    assert!(d.walls.iter().filter(|w| near(w.length(), 36.0, 1.0 / 12.0)).count() == 2);
}

#[test]
fn a_dimension_string_close_to_a_wall_is_not_a_wall_with_it() {
    // 8 in off the south wall's outer face at 1/2 in to the foot, with its ticks: the wall is not read as 8 in
    // and 6 in lines that make two walls, and nothing is in doubt.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    s.dimension([0.0, -0.25], [30.0, -0.25], -8.0 * IN, 6.0 * Q2);
    let d = s.found(Q2);
    assert!(d.walls.iter().all(|w| w.doubt.is_none()), "{d:#?}");
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 12.0);
}

#[test]
fn a_line_beside_a_wall_with_nothing_to_say_what_it_is_makes_the_wall_in_doubt() {
    // The same string with no ticks at all: it cannot be told from a wall's line, and says so.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    let y = -0.25 - 8.0 * IN;
    s.line([0.0, y], [30.0, y]);
    let d = s.found(Q2);
    assert!(d.walls.iter().any(|w| w.doubt.as_deref().is_some_and(|d| d.contains("another line lies beside it"))), "{d:#?}");
}

#[test]
fn a_box_on_the_sheet_with_a_key_in_it_is_left_out_and_counted() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    // A key box 12 ft by 4 ft, off to one side, holding a swatch of a wall 8 in thick and 4 ft long.
    s.line([40.0, 0.0], [52.0, 0.0]).line([52.0, 0.0], [52.0, 4.0]).line([52.0, 4.0], [40.0, 4.0]).line([40.0, 4.0], [40.0, 0.0]);
    s.line([41.0, 1.0], [45.0, 1.0]).line([45.0, 1.0], [45.0, 1.0 + 8.0 * IN]).line([45.0, 1.0 + 8.0 * IN], [41.0, 1.0 + 8.0 * IN]).line([41.0, 1.0 + 8.0 * IN], [41.0, 1.0]);
    let d = s.found(Q4);
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 24.0);
    assert!(d.walls.iter().all(|w| w.doubt.is_none()));
    assert_eq!(d.boxes.len(), 1);
    assert_eq!(d.counts.boxed, 8);
}

#[test]
fn a_sheet_border_is_counted_and_a_room_line_is_a_single_line() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    // A room line across the middle, one line; and the sheet's border near the page edge.
    s.line([15.0, 0.25], [15.0, 15.75]);
    let mut page = s.page(Q4);
    let (w, h) = (3000.0, 2400.0);
    page.extend([[[18.0, 18.0], [w - 18.0, 18.0]], [[w - 18.0, 18.0], [w - 18.0, h - 18.0]], [[w - 18.0, h - 18.0], [18.0, h - 18.0]], [[18.0, h - 18.0], [18.0, 18.0]]]);
    let d = find(&page, Some(Q4), [w, h], [0.0, 0.0]);
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 24.0);
    assert_eq!(d.counts.border, 4);
    assert!(d.counts.single >= 1);
}

#[test]
fn a_wall_drawn_with_overlapping_rectangles_ends_where_it_meets_and_closes() {
    // Each wall a rectangle of its own, overlapping at the corners (a second way sheets are drawn).
    let mut s = Sheet::default();
    let t = 0.5;
    for (a, b) in [([0.0_f64, 0.0], [24.0, 0.0]), ([24.0, 0.0], [24.0, 14.0]), ([24.0, 14.0], [0.0, 14.0]), ([0.0, 14.0], [0.0, 0.0])] {
        let (u, n): (P, P) = {
            let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
            ([(b[0] - a[0]) / l, (b[1] - a[1]) / l], [-(b[1] - a[1]) / l, (b[0] - a[0]) / l])
        };
        let p = |s: f64, o: f64| [a[0] + u[0] * s + n[0] * o, a[1] + u[1] * s + n[1] * o];
        let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        s.line(p(-t / 2.0, t / 2.0), p(l + t / 2.0, t / 2.0)).line(p(l + t / 2.0, t / 2.0), p(l + t / 2.0, -t / 2.0)).line(p(l + t / 2.0, -t / 2.0), p(-t / 2.0, -t / 2.0)).line(p(-t / 2.0, -t / 2.0), p(-t / 2.0, t / 2.0));
    }
    let d = s.found(Q4);
    assert_lengths(&d, &[14.0, 14.0, 24.0, 24.0], 1.0 / 12.0);
    assert!(d.open_ends.is_empty(), "{:?}", d.open_ends);
}

#[test]
fn walls_that_do_not_close_are_said_with_where() {
    // Three walls of a room: the fourth is not drawn.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 20.0, 12.0, 0.5);
    s.cut([0.0, 0.25], [20.0, 0.25], 0.0, 20.0);
    s.cut([0.0, -0.25], [20.0, -0.25], -1.0, 21.0);
    let d = s.found(Q4);
    assert!(!d.open_ends.is_empty(), "{d:#?}");
}

#[test]
fn a_stair_and_a_deck_beside_the_house_are_sets_and_the_walls_are_unchanged() {
    let mut s = house();
    for i in 0..13 {
        s.line([20.0, 3.0 + i as f64 * 10.0 * IN], [23.0, 3.0 + i as f64 * 10.0 * IN]);
    }
    // A deck's boards outside, 5 1/2 in apart.
    for i in 0..8 {
        s.line([40.0, i as f64 * 5.5 * IN], [48.0, i as f64 * 5.5 * IN]);
    }
    let d = s.found(Q4);
    check_house(&d, "with a stair and a deck");
    assert_eq!(d.sets.len(), 2, "{:?}", d.sets);
}

#[test]
fn faces_that_run_on_unseen_across_an_opening_still_give_a_door_and_a_window() {
    // How the program's own plan set draws them: the walls' faces run on across the opening (a white box
    // over them hides them on paper; the lines are in the file), jambs across, and a door has a leaf line and
    // a window its two faces drawn again from jamb to jamb and a line between.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    for (lo, hi) in [(5.0, 8.0)] {
        s.line([30.0 - 0.25, lo], [30.25, lo]).line([30.0 - 0.25, hi], [30.25, hi]).line([30.0, lo], [30.0, hi]);
    }
    for (lo, hi) in [(8.0, 13.0)] {
        s.line([lo, -0.25], [lo, 0.25]).line([hi, -0.25], [hi, 0.25]);
        for y in [-0.25, 0.0, 0.25] {
            s.line([lo, y], [hi, y]);
        }
    }
    for turn in [0.0, 33.0] {
        let d = s.turned(turn).found(Q4);
        assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 24.0);
        if turn == 0.0 {
            let (south_w, east_w) = (south(&d), east(&d));
            assert!(south_w.openings.len() == 1 && south_w.openings[0].window, "{south_w:?}");
            assert!(east_w.openings.len() == 1 && !east_w.openings[0].window && near(east_w.openings[0].from, 5.0, 1.0 / 12.0), "{east_w:?}");
        } else {
            let opens: Vec<bool> = d.walls.iter().flat_map(|w| w.openings.iter().map(|o| o.window)).collect();
            assert!(opens.len() == 2 && opens.contains(&true) && opens.contains(&false), "{opens:?}");
        }
    }
}

#[test]
fn nothing_found_is_known_and_a_sheet_of_nothing_finds_nothing() {
    let d = find(&[], Some(Q4), [800.0, 600.0], [0.0, 0.0]);
    assert!(d.walls.is_empty() && !d.no_scale);
    let d = find(&[[[0.0, 0.0], [0.0, 0.0]]], Some(Q4), [800.0, 600.0], [0.0, 0.0]);
    assert!(d.walls.is_empty());
}

#[test]
fn two_houses_in_one_line_are_not_one_wall_with_a_door_twenty_feet_wide() {
    // Two rings side by side, their south walls in one line with 19 ft 6 in between: a break wider than any opening.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 20.0, 12.0, 0.5);
    s.ring(40.0, 0.0, 64.0, 16.0, 0.5);
    let d = s.found(Q4);
    assert_lengths(&d, &[12.0, 16.0, 20.0, 20.0, 24.0, 24.0, 12.0, 16.0], 1.0 / 24.0);
    assert!(d.walls.iter().all(|w| w.openings.is_empty()), "{d:#?}");
    assert!(d.open_ends.is_empty());
}

#[test]
fn a_curve_drawn_in_short_straight_pieces_is_said_and_no_wall_is_made_of_it() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 14.0, 0.5);
    let (c, r) = ([22.0, 14.0], 8.0);
    let arc: Vec<P> = (0..=180).step_by(7).map(|a| [c[0] + r * (a as f64).to_radians().cos(), c[1] + r * (a as f64).to_radians().sin()]).collect();
    for w in arc.windows(2) {
        s.line(w[0], w[1]);
        let out = |p: P| [p[0] + (p[0] - c[0]) * 0.5 / r, p[1] + (p[1] - c[1]) * 0.5 / r];
        s.line(out(w[0]), out(w[1]));
    }
    let d = s.found(Q4);
    assert_lengths(&d, &[14.0, 14.0, 30.0, 30.0], 1.0 / 24.0);
    assert_eq!(d.curved.len(), 2, "the two lines of the bay: {:?}", d.curved);
    assert!(d.curved.iter().all(|c| c.0 >= 20 && c.1 > 20.0), "{:?}", d.curved);
}

#[test]
fn a_room_line_through_a_wall_does_not_end_it_and_does_not_stop_the_solid_line_beside_it_ending_it() {
    // Call 182's sheet draws a room line, one long line from wall centre to wall centre across the plan. Where
    // new work meets existing, the cap across the wall is drawn in two halves and the room line runs through
    // their joint, so the three lines are one line on one line.
    let ring = || {
        let mut s = Sheet::default();
        s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
        s
    };
    // The room line alone: no wall is ended by it, and it is no wall.
    let mut alone = ring();
    alone.line([12.0, 0.0], [12.0, 16.0]);
    let d = alone.found(Q4);
    assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 24.0);
    // The room line with the caps drawn in halves, south and north: both walls end at 12 ft.
    let mut s = ring();
    s.line([12.0, 0.0], [12.0, 16.0]);
    for y in [0.0, 16.0] {
        s.line([12.0, y - 0.25], [12.0, y]).line([12.0, y], [12.0, y + 0.25]);
    }
    for turn in [0.0, 33.0] {
        let d = s.turned(turn).found(Q4);
        assert_lengths(&d, &[12.0, 12.0, 16.0, 16.0, 18.0, 18.0], 1.0 / 24.0);
        assert!(d.walls.iter().all(|w| w.doubt.is_none()), "{d:#?}");
    }
}

// ---- call 186, item 1: a wall's lines are solid ----

#[test]
fn two_dashed_lines_are_never_a_wall_and_are_said_as_what_they_may_be() {
    // A beam shown dashed over the living room, wall to wall: 15 ft long, 8 in wide.
    for turn in [0.0, 33.0] {
        let mut s = house();
        s.dash([0.25, 8.0], [18.0 - 0.1875, 8.0]).dash([0.25, 8.0 - 8.0 * IN], [18.0 - 0.1875, 8.0 - 8.0 * IN]);
        let d = s.turned(turn).found(Q4);
        check_house(&d, &format!("a dashed beam, turned {turn}"));
        assert_eq!(d.overhead.len(), 1, "{d:#?}");
        let o = &d.overhead[0];
        assert!(o.lines == 2 && near(o.apart, 8.0 * IN, 1.0 / 48.0) && near(o.length, 17.8125 - 0.25, 1.0 / 12.0), "{o:?}");
        assert_eq!(d.counts.dashed, 2);
    }
}

#[test]
fn a_dashed_pair_between_two_walls_is_not_a_wall_in_doubt_either() {
    // What a renovation plan draws for a wall to be demolished: a pair of dashed lines, 4 1/2 in apart, wall to
    // wall. A wall's lines are solid: it is said as dashed, never laid, even with --take (it is not in the walls found).
    let mut s = house();
    s.dash([0.25, 4.0], [17.8125, 4.0]).dash([0.25, 4.0 - 4.5 * IN], [17.8125, 4.0 - 4.5 * IN]);
    let d = s.found(Q4);
    check_house(&d, "a dashed pair");
    assert_eq!(d.walls.len(), 5, "no sixth wall, in doubt or not: {d:#?}");
    assert_eq!(d.overhead.len(), 1);
}

#[test]
fn a_dashed_property_line_round_the_plan_is_no_box_and_no_wall() {
    // Four dashed lines in a ring 10 ft outside the house, and a dashed room line through it.
    let mut s = house();
    let (a, b) = (-10.0, 46.0);
    s.dash([a, a], [b, a]).dash([b, a], [b, 26.0]).dash([b, 26.0], [a, 26.0]).dash([a, 26.0], [a, a]);
    s.dash([9.0, 0.0], [9.0, 16.0]);
    let d = s.found(Q4);
    check_house(&d, "a dashed property line");
    assert_eq!(d.counts.dashed, 5);
    assert!(d.boxes.is_empty() && d.overhead.is_empty(), "{d:#?}");
}

#[test]
fn a_page_with_no_dashed_lines_is_found_as_before() {
    let d = house().found(Q4);
    check_house(&d, "no dashes");
    assert!(d.overhead.is_empty() && d.counts.dashed == 0);
}

// ---- call 186, item 4: a pier is part of its wall however short ----

#[test]
fn a_door_beside_a_window_with_a_pier_under_two_feet_between_them_loses_neither() {
    // The south wall of a 30 by 16 ft house: a door 10 to 13 ft and a window 5 ft wide a pier further on.
    for (k, name) in [(Q4, "1/4"), (Q8, "1/8"), (Q2, "1/2")] {
        for pier in [0.75, 1.0, 1.5] {
            for turn in [0.0, 33.0] {
                let mut s = Sheet::default();
                s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
                s.opening([0.0, 0.0], [30.0, 0.0], 0.5, 10.0, 13.0, false);
                s.glass([0.0, 0.0], [30.0, 0.0], 0.5, 13.0 + pier, 18.0 + pier);
                let d = s.turned(turn).found(k);
                let what = format!("{name} in to the foot, pier {pier} ft, turned {turn}");
                assert!(d.walls.iter().all(|w| w.doubt.is_none()), "{what}: {d:#?}");
                assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 12.0);
                let mut opens: Vec<(bool, f64, f64)> = d.walls.iter().flat_map(|w| w.openings.iter().map(|o| (o.window, o.from, o.to))).collect();
                opens.sort_by(|a, b| a.1.total_cmp(&b.1));
                assert_eq!(opens.len(), 2, "{what}: both found: {opens:?}");
                assert!(!opens[0].0 && near(opens[0].1, 10.0, 1.0 / 12.0) && near(opens[0].2, 13.0, 1.0 / 12.0), "{what}: the door: {opens:?}");
                assert!(opens[1].0 && near(opens[1].1, 13.0 + pier, 1.0 / 12.0) && near(opens[1].2, 18.0 + pier, 1.0 / 12.0), "{what}: the window: {opens:?}");
            }
        }
    }
}

#[test]
fn a_door_in_a_partition_two_feet_from_the_wall_it_meets_is_found() {
    // A partition 4 1/2 in thick from the south wall to the north, a door 3 ft wide whose near jamb is 9 in from the
    // south wall's face: the partition's first piece is 9 in long, and it is part of the partition.
    for (k, name) in [(Q4, "1/4"), (Q2, "1/2")] {
        for turn in [0.0, 33.0] {
            let mut s = Sheet::default();
            s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
            s.cut([0.0, 0.25], [30.0, 0.25], 14.8, 15.2);
            s.cut([0.0, 15.75], [30.0, 15.75], 14.8, 15.2);
            s.partition(15.0, 0.25, 15.75, 0.375);
            s.opening([15.0, 0.25], [15.0, 15.75], 0.375, 0.75, 3.75, false);
            let d = s.turned(turn).found(k);
            let what = format!("{name}, turned {turn}");
            assert!(d.walls.iter().all(|w| w.doubt.is_none()), "{what}: {d:#?}");
            assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0, 16.0], 1.0 / 12.0);
            let opens: Vec<(bool, f64)> = d.walls.iter().flat_map(|w| w.openings.iter().map(|o| (o.window, o.to - o.from))).collect();
            assert_eq!(opens.len(), 1, "{what}: {opens:?}");
            assert!(!opens[0].0 && near(opens[0].1, 3.0, 1.0 / 12.0), "{what}: {opens:?}");
        }
    }
}

#[test]
fn a_pier_is_no_license_for_lettering_to_make_a_wall() {
    // Rows of stems 8 in apart and as tall as a wall is thick, in two rows 8 in apart (lettering), still make no wall.
    let mut s = Sheet::default();
    for row in 0..2 {
        for i in 0..20 {
            let x = i as f64 * 10.0 * IN;
            s.line([x, row as f64 * 8.0 * IN], [x, row as f64 * 8.0 * IN + 6.0 * IN]);
        }
    }
    let d = s.found(Q4);
    assert!(d.walls.is_empty(), "{d:#?}");
}

// ---- call 186, item 2: a line is the face of one wall only ----

#[test]
fn a_line_that_is_a_walls_face_does_not_pair_again_with_a_line_beside_it() {
    // A counter's back edge 4 1/2 in off the north wall's inner face, 24 ft of its 30. The north wall is the pair
    // 6 in thick that three other walls match; the counter's line is beside it, said, and no wall.
    for turn in [0.0, 33.0] {
        let mut s = Sheet::default();
        s.ring(0.0, 0.0, 30.0, 20.0, 0.5);
        s.line([3.0, 19.75 - 4.5 * IN], [27.0, 19.75 - 4.5 * IN]);
        let d = s.turned(turn).found(Q4);
        assert_lengths(&d, &[20.0, 20.0, 30.0, 30.0], 1.0 / 24.0);
        let doubts: Vec<&str> = d.walls.iter().filter_map(|w| w.doubt.as_deref()).collect();
        assert_eq!(doubts.len(), 1, "turned {turn}: {d:#?}");
        assert!(doubts[0].contains("already the face of a wall"), "{doubts:?}");
    }
}

#[test]
fn a_wall_in_doubt_does_not_shorten_the_wall_it_lies_beside() {
    // The judge's counter on the north wall: the west wall was found 2 in short, its end carried to the doubtful pair's line.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 20.0, 0.5);
    s.line([0.25, 19.75 - 4.5 * IN], [29.75, 19.75 - 4.5 * IN]);
    let d = s.found(Q4);
    let west = d.walls.iter().filter(|w| (w.to[0] - w.from[0]).abs() < 1e-6 && w.doubt.is_none()).min_by(|a, b| a.from[0].total_cmp(&b.from[0])).expect("a west wall");
    assert!(near(west.length(), 20.0, 1.0 / 24.0), "{d:#?}");
}

// ---- call 186, item 5: what the page cut off ----

#[test]
fn walls_run_to_the_edge_of_what_the_page_shows_are_cut_off_there_and_do_not_close() {
    // A 30 by 16 ft house of which the page shows only x < 20 ft: the south and north walls stop at its edge, where the
    // lines were cut; the east wall is not shown.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    let mut kept = Sheet::default();
    for l in &s.l {
        // Cut each line at x = 20 (what a clip shows of it).
        let (a, b) = (l[0], l[1]);
        if a[0] > 20.0 && b[0] > 20.0 {
            continue;
        }
        if a[0] <= 20.0 && b[0] <= 20.0 {
            kept.line(a, b);
            continue;
        }
        let t = (20.0 - a[0]) / (b[0] - a[0]);
        let y = a[1] + (b[1] - a[1]) * t;
        kept.line(if a[0] <= 20.0 { a } else { b }, [20.0, y]);
        kept.cuts.push([20.0, y]);
    }
    let d = kept.found(Q4);
    assert_lengths(&d, &[16.0, 20.0, 20.0], 1.0 / 12.0);
    assert_eq!(d.cut_ends.len(), 2, "{d:#?}");
    let west = d.walls.iter().map(|w| w.from[0].min(w.to[0])).fold(f64::MAX, f64::min);
    assert!(d.cut_ends.iter().all(|(_, p)| near(p[0], west + 20.0, 1.0 / 12.0)), "{:?}", d.cut_ends);
    assert!(d.open_ends.is_empty(), "cut off is not open: {:?}", d.open_ends);
}

#[test]
fn a_wall_that_stops_where_nothing_cut_it_is_an_open_end_and_not_a_cut() {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 20.0, 12.0, 0.5);
    s.cut([0.0, 0.25], [20.0, 0.25], 0.0, 20.0);
    s.cut([0.0, -0.25], [20.0, -0.25], -1.0, 21.0);
    let d = s.found(Q4);
    assert!(!d.open_ends.is_empty() && d.cut_ends.is_empty(), "{d:#?}");
}

// ---- call 186, item 9: layers, single lines, a line across that overshoots ----

#[test]
fn a_wall_drawn_in_its_layers_is_one_wall_said_once() {
    // Each wall of a 30 by 20 ft house drawn with six lines: 8 in over all, lines 1/2 in and 1 in in from each face.
    for (outer, inner, name) in [(8.0 * IN, [7.0 * IN, 6.0 * IN], "8 in, six lines"), (12.0 * IN, [6.0 * IN, 3.0 * IN * 2.0], "12 in, four lines")] {
        for turn in [0.0, 33.0] {
            let mut s = Sheet::default();
            s.ring(0.0, 0.0, 30.0, 20.0, outer);
            for t in inner {
                if t != outer {
                    s.ring(0.0, 0.0, 30.0, 20.0, t);
                }
            }
            let d = s.turned(turn).found(Q4);
            assert!(d.walls.iter().all(|w| w.doubt.is_none()), "{name} turned {turn}: {d:#?}");
            assert_lengths(&d, &[20.0, 20.0, 30.0, 30.0], 1.0 / 24.0);
            assert!(d.walls.iter().all(|w| near(w.thickness, outer, 1.0 / 96.0)), "{name}: the faces are the outermost two lines: {d:#?}");
            assert!(d.walls.iter().all(|w| w.lines >= 4), "{name}: drawn with its lines said: {d:#?}");
            assert!(d.open_ends.is_empty(), "{name}: {:?}", d.open_ends);
        }
    }
}

#[test]
fn a_door_in_a_wall_drawn_in_its_layers_is_found() {
    let mut s = Sheet::default();
    for t in [8.0 * IN, 7.0 * IN, 6.0 * IN] {
        s.ring(0.0, 0.0, 30.0, 20.0, t);
    }
    for h in [4.0 * IN, 3.5 * IN, 3.0 * IN, -3.0 * IN, -3.5 * IN, -4.0 * IN] {
        s.cut([0.0, h], [30.0, h], 10.0, 13.0);
    }
    s.line([10.0, -4.0 * IN], [10.0, 4.0 * IN]).line([13.0, -4.0 * IN], [13.0, 4.0 * IN]).line([10.0, 0.0], [13.0, 0.0]);
    let d = s.found(Q4);
    assert_lengths(&d, &[20.0, 20.0, 30.0, 30.0], 1.0 / 24.0);
    let opens: Vec<(f64, f64)> = d.walls.iter().flat_map(|w| w.openings.iter().map(|o| (o.from, o.to))).collect();
    assert!(opens.len() == 1 && near(opens[0].1 - opens[0].0, 3.0, 1.0 / 12.0), "{opens:?}");
}

#[test]
fn a_wall_with_a_lone_line_down_its_middle_stays_a_set_as_the_contract_says_and_is_said() {
    // Call 181's rule, kept: three evenly spaced lines are a set. Said as a set, with where, in place of "no two lines
    // 3 in to 14 in apart" which is not true of them.
    let mut s = Sheet::default();
    for t in [8.0 * IN, 0.0] {
        s.ring(0.0, 0.0, 30.0, 20.0, t);
    }
    let d = s.found(Q4);
    assert!(d.walls.is_empty() && !d.sets.is_empty(), "{d:#?}");
}

#[test]
fn a_sheet_whose_walls_are_single_lines_is_told_so() {
    // An L-shaped house as a centre-line outline, one line a wall, and a partition as one line. (A plain rectangle of one
    // line is a closed box on the sheet and is said as one: its words name a house drawn so.)
    let mut s = Sheet::default();
    let l = [[0.0, 0.0], [30.0, 0.0], [30.0, 12.0], [18.0, 12.0], [18.0, 24.0], [0.0, 24.0]];
    for i in 0..6 {
        s.line(l[i], l[(i + 1) % 6]);
    }
    s.line([9.0, 0.0], [9.0, 24.0]);
    for turn in [0.0, 33.0] {
        let d = s.turned(turn).found(Q4);
        assert!(d.walls.is_empty(), "{d:#?}");
        assert_eq!(d.single_outlines.len(), 1, "turned {turn}: {d:#?}");
        assert!(d.single_outlines[0].0 >= 6, "{:?}", d.single_outlines);
    }
    // A house of two lines a wall with a room line is not one, nor is it for a symbol drawn with lines end to end.
    let mut s = house();
    s.line([9.0, 0.25], [9.0, 15.75]);
    for (a, b) in [([40.0, 0.0], [43.0, 4.0]), ([43.0, 4.0], [46.0, 0.0]), ([46.0, 0.0], [40.0, 0.0])] {
        s.line(a, b);
    }
    assert!(s.found(Q4).single_outlines.is_empty());
}

#[test]
fn a_line_across_a_wall_that_overshoots_its_faces_by_less_than_the_thickness_is_still_a_line_across() {
    // Where new work meets existing: the cap drawn 2 in past each face of a 6 in wall.
    for turn in [0.0, 33.0] {
        let mut s = Sheet::default();
        s.ring(0.0, 0.0, 30.0, 12.0, 0.5);
        s.line([12.0, -0.25 - 2.0 * IN], [12.0, 0.25 + 2.0 * IN]);
        let d = s.turned(turn).found(Q4);
        assert_lengths(&d, &[12.0, 12.0, 18.0, 30.0, 12.0], 1.0 / 24.0);
    }
    // 7 in past each face of a 6 in wall is not an overshoot of the wall's: it is a line through it.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 12.0, 0.5);
    s.line([12.0, -0.25 - 7.0 * IN], [12.0, 0.25 + 7.0 * IN]);
    assert_lengths(&s.found(Q4), &[12.0, 12.0, 30.0, 30.0], 1.0 / 24.0);
}

#[test]
fn faces_that_run_on_past_a_jamb_into_the_opening_do_not_end_the_wall_at_the_jamb() {
    // The judge's sheet: every face runs 1 in past the jamb, into the opening. A door 3 ft wide in the south wall, a
    // second 4 ft wide in the east wall: both found, four walls whole.
    for (k, name) in [(Q4, "1/4"), (Q2, "1/2")] {
        for turn in [0.0, 33.0] {
            let mut s = Sheet::default();
            s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
            s.opening_over([0.0, 0.0], [30.0, 0.0], 0.5, 10.0, 13.0, IN);
            s.opening_over([30.0, 0.0], [30.0, 16.0], 0.5, 5.0, 9.0, IN);
            let d = s.turned(turn).found(k);
            let what = format!("{name}, turned {turn}");
            assert!(d.walls.iter().all(|w| w.doubt.is_none()), "{what}: {d:#?}");
            assert_lengths(&d, &[16.0, 16.0, 30.0, 30.0], 1.0 / 12.0);
            let mut opens: Vec<f64> = d.walls.iter().flat_map(|w| w.openings.iter().map(|o| o.to - o.from)).collect();
            opens.sort_by(|a, b| a.total_cmp(b));
            assert!(opens.len() == 2 && near(opens[0], 3.0, 1.0 / 12.0) && near(opens[1], 4.0, 1.0 / 12.0), "{what}: {opens:?}");
        }
    }
}

// ---- call 186, item 3 (offered, not part of the main patch: it needs the lead's word) ----

#[test]
fn a_group_of_walls_far_smaller_than_the_plan_and_apart_from_it_is_in_doubt_as_a_detail_or_a_section() {
    // The house, and beside it a wall section drawn at 3/8 in (1.5 times the plan's scale): a wall 9 in thick and 12 ft high
    // with a slab 6 in thick meeting it.
    for turn in [0.0, 33.0] {
        let mut s = house();
        s.line([50.0, 3.75], [50.0, 16.0]).line([50.0 + 0.75, 3.75], [50.0 + 0.75, 16.0]);
        s.line([44.0, 4.0], [50.375, 4.0]).line([44.0, 3.5], [50.375, 3.5]);
        let d = s.turned(turn).found(Q4);
        let apart: Vec<&Wall> = d.walls.iter().filter(|w| w.doubt.as_deref().is_some_and(|d| d.contains("apart from the plan"))).collect();
        assert!(apart.len() >= 2, "turned {turn}: {d:#?}");
        // The house is found as it was.
        assert_eq!(d.walls.iter().filter(|w| w.doubt.is_none()).count(), 5, "turned {turn}: {d:#?}");
    }
    // Two houses of like size on one sheet are both plans (the shared test of call 181 stays).
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 20.0, 12.0, 0.5);
    s.ring(40.0, 0.0, 64.0, 16.0, 0.5);
    assert!(s.found(Q4).walls.iter().all(|w| w.doubt.is_none()));
}

// ---- call 186, as built: item 2 narrowed ----

fn thick_doubts(d: &Drawn) -> usize {
    d.walls.iter().filter(|w| w.doubt.as_deref().is_some_and(|x| x.contains("thicker than every other wall"))).count()
}

#[test]
fn a_pair_thicker_than_every_other_wall_and_alone_at_that_thickness_is_in_doubt() {
    // A 12 in closet shelf, 17 ft long, between the west wall and a 4 1/2 in partition in a ring of 6 in walls.
    for turn in [0.0, 33.0] {
        let mut s = Sheet::default();
        s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
        s.cut([0.0, 0.25], [30.0, 0.25], 17.8, 18.2);
        s.cut([0.0, 15.75], [30.0, 15.75], 17.8, 18.2);
        s.partition(18.0, 0.25, 15.75, 0.375);
        s.line([0.25, 8.0], [17.8125, 8.0]).line([0.25, 9.0], [17.8125, 9.0]);
        let d = s.turned(turn).found(Q4);
        assert_eq!(thick_doubts(&d), 1, "turned {turn}: {d:#?}");
        assert_eq!(d.walls.iter().filter(|w| w.doubt.is_none()).count(), 5, "turned {turn}");
    }
}

#[test]
fn a_thin_partition_a_pair_of_thick_walls_and_a_lone_wall_are_not_in_doubt_for_their_thickness() {
    // One 4 1/2 in partition in a ring of 6 in walls: found (the shared house).
    assert_eq!(thick_doubts(&house().found(Q4)), 0);
    // One 3 1/2 in partition among 12 in walls: found.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 24.0, 16.0, 1.0);
    s.line([12.0, 0.5], [12.0, 15.5]).line([12.0 + 3.5 * IN, 0.5], [12.0 + 3.5 * IN, 15.5]);
    s.cut([0.0, 0.5], [24.0, 0.5], 11.99, 12.0 + 3.5 * IN + 0.01);
    s.cut([0.0, 15.5], [24.0, 15.5], 11.99, 12.0 + 3.5 * IN + 0.01);
    let d = s.found(Q4);
    assert_eq!(thick_doubts(&d), 0, "{d:#?}");
    assert!(d.walls.iter().all(|w| w.doubt.is_none()));
    // Two 12 in walls among 6 in walls: both found, neither is alone at its thickness.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 40.0, 16.0, 0.5);
    for x in [10.0, 30.0] {
        s.partition(x, 0.25, 15.75, 1.0);
        s.cut([0.0, 0.25], [40.0, 0.25], x - 0.51, x + 0.51);
        s.cut([0.0, 15.75], [40.0, 15.75], x - 0.51, x + 0.51);
    }
    let d = s.found(Q4);
    assert_eq!(thick_doubts(&d), 0, "{d:#?}");
    // One 8 in wall (a garage wall) alone among 6 1/2 in walls: thicker, but not by a wall's least (3 in): found.
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 40.0, 16.0, 6.5 / 12.0);
    s.partition(20.0, 6.5 / 24.0, 16.0 - 6.5 / 24.0, 8.0 / 12.0);
    s.cut([0.0, 6.5 / 24.0], [40.0, 6.5 / 24.0], 19.6, 20.4);
    s.cut([0.0, 16.0 - 6.5 / 24.0], [40.0, 16.0 - 6.5 / 24.0], 19.6, 20.4);
    assert_eq!(thick_doubts(&s.found(Q4)), 0);
    // A sheet with one wall only: its thickness is not in doubt (it meets nothing, and says that).
    let mut s = Sheet::default();
    s.line([0.0, 0.0], [20.0, 0.0]).line([0.0, 1.0], [20.0, 1.0]);
    assert_eq!(thick_doubts(&s.found(Q4)), 0);
}

// ---- call 188, items 6 to 9: what real sheets showed wrong in the finder ----
//
// Items 7 and 8 are drawn with the causes found on the real sheets (a face drawn twice; lines of a thin leaf or a shower's
// glass inside a partition whose faces run on); each figure is worked by hand beside its test.

impl Sheet {
    /// A wall along a to b (its centre line) drawn the way a real sheet drew one that was found two to four times: its two
    /// outer faces `span` apart, and a second line 1 in inside one face and 0.6 in inside the other. The lines run the
    /// whole length asked, corner to corner as `ring`'s do.
    fn doubled(&mut self, a: P, b: P, span: f64) -> &mut Self {
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let u = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
        let n = [-u[1], u[0]];
        for off in [-span / 2.0, -span / 2.0 + IN, span / 2.0 - 0.6 * IN, span / 2.0] {
            self.line([a[0] + n[0] * off, a[1] + n[1] * off], [b[0] + n[0] * off, b[1] + n[1] * off]);
        }
        self
    }
    /// A hexagon `r` from its centre to a corner: a tag (a window's or a door's mark on a plan).
    fn tag(&mut self, c: P, r: f64) -> &mut Self {
        let pts: Vec<P> = (0..6).map(|i| [c[0] + r * (i as f64 * 60.0_f64).to_radians().cos(), c[1] + r * (i as f64 * 60.0_f64).to_radians().sin()]).collect();
        for i in 0..6 {
            self.line(pts[i], pts[(i + 1) % 6]);
        }
        self
    }
}

/// A 30 by 20 ft house of 6 in walls with a 4 1/2 in partition at 12 ft, and 50 ft east of it a roof plan: an eave 32 by 22 ft
/// (centre lines) with a second line 6 in inside it all round.
fn plan_and_roof() -> Sheet {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 20.0, 0.5);
    s.cut([0.0, 0.25], [30.0, 0.25], 12.0 - 0.19, 12.0 + 0.19);
    s.cut([0.0, 19.75], [30.0, 19.75], 12.0 - 0.19, 12.0 + 0.19);
    s.partition(12.0, 0.25, 19.75, 0.375);
    s.ring(80.0, 0.0, 112.0, 22.0, 0.5);
    s
}

fn feet_of_walls(d: &Drawn) -> f64 {
    d.walls.iter().map(|w| w.length()).sum()
}

#[test]
fn a_sheet_with_a_plan_and_a_roof_plan_of_like_size_says_there_are_two_and_lists_them_largest_first() {
    // By hand: the house is 30 + 30 + 20 + 20 + the partition 20 = 120 LF in 5 walls; the eave is 32 + 22 + 32 + 22 = 108 LF in
    // 4 pieces. 108 is over half of 120: of like size, both drawings. Numbered largest first: 1 the house, 2 the roof.
    for turn in [0.0, 33.0] {
        let d = plan_and_roof().turned(turn).found(Q4);
        assert_eq!(d.drawings.len(), 2, "turned {turn}: {d:#?}");
        let (a, b) = (&d.drawings[0], &d.drawings[1]);
        assert!(near(a.length, 120.0, 1.0 / 12.0) && a.walls.len() == 5, "turned {turn}: {a:?}");
        assert!(near(b.length, 108.0, 1.0 / 12.0) && b.walls.len() == 4, "turned {turn}: {b:?}");
        // Nothing is dropped: every wall is found and listed, and none is in doubt for being of the other drawing.
        assert_eq!(d.walls.len(), 9);
        assert!(d.chosen.is_none());
    }
    // One plan alone, or a section far under half its size beside it (call 186, kept), is not two drawings.
    assert!(house().found(Q4).drawings.is_empty());
    let mut s = house();
    s.line([50.0, 3.75], [50.0, 16.0]).line([50.0 + 0.75, 3.75], [50.0 + 0.75, 16.0]);
    s.line([44.0, 4.0], [50.375, 4.0]).line([44.0, 3.5], [50.375, 3.5]);
    let d = s.found(Q4);
    assert!(d.drawings.is_empty(), "{d:#?}");
    assert!(d.walls.iter().filter(|w| w.doubt.as_deref().is_some_and(|x| x.contains("apart from the plan"))).count() >= 2);
}

#[test]
fn a_plan_is_taken_by_its_number_or_by_a_box_round_it_and_a_number_that_is_not_there_is_refused() {
    let s = plan_and_roof();
    let lo = s.lowest();
    let page = s.page(Q4);
    let k = Q4;
    let origin = [0.0, 0.0];
    let extra = Extra::default();
    let want = [(1usize, 5usize, 120.0), (2, 4, 108.0)];
    for (n, walls, lf) in want {
        let d = find_in(&page, Some(k), [3000.0, 2400.0], origin, &extra, &Pick { plan: Some(n), ..Pick::default() }).unwrap();
        assert_eq!(d.walls.len(), walls, "plan {n}: {d:#?}");
        assert!(near(feet_of_walls(&d), lf, 1.0 / 6.0), "plan {n}: {}", feet_of_walls(&d));
        assert_eq!(d.chosen, Some(n));
        assert_eq!(d.drawings.len(), 2, "the sheet's drawings are still said");
    }
    // A box round the house: corners in feet on the plan. The page's corner is the plan's (0, 0) here, and the sheet's own
    // lowest line stands 126 points in from it, so the house's south-west corner is at (126 pt * k) = 7 ft.
    let pad = 126.0 * k - (-0.25 - 0.0) - 0.0;
    let _ = (lo, pad);
    let house_box = [[126.0 * k - 1.0, 126.0 * k - 1.0], [126.0 * k + 31.0, 126.0 * k + 21.0]];
    let d = find_in(&page, Some(k), [3000.0, 2400.0], origin, &extra, &Pick { within: Some(house_box), ..Pick::default() }).unwrap();
    assert_eq!(d.walls.len(), 5, "{d:#?}");
    assert!(near(feet_of_walls(&d), 120.0, 1.0 / 6.0));
    assert!(d.drawings.is_empty(), "a box with one drawing in it holds one: {:?}", d.drawings);
    // The corners given either way round are the same box.
    let flipped = [house_box[1], house_box[0]];
    let d2 = find_in(&page, Some(k), [3000.0, 2400.0], origin, &extra, &Pick { within: Some(flipped), ..Pick::default() }).unwrap();
    assert_eq!(d2.walls.len(), 5);
    // 3 is refused where there are 2, 0 too.
    for n in [3usize, 0] {
        let e = find_in(&page, Some(k), [3000.0, 2400.0], origin, &extra, &Pick { plan: Some(n), ..Pick::default() }).unwrap_err();
        assert!(e.contains("2 drawings"), "{e}");
    }
}

#[test]
fn each_wall_drawn_with_its_faces_twice_is_found_once_and_says_how_many_pairs_were_joined() {
    // By hand: a 30 by 20 ft house whose four walls are each drawn with four lines (the two outer faces 7 in, 6 1/2 in, 7 1/2 in and
    // 8 in apart, a second line 1 in inside one face and 0.6 in inside the other), as a real sheet draws them: four pairs of lines
    // over each wall, and no two walls of one span. Four walls, 30 + 20 + 30 + 20 = 100 LF, none in doubt, each says it joined 4 pairs.
    for turn in [0.0, 33.0] {
        let mut s = Sheet::default();
        s.doubled([-0.2, 0.0], [30.2, 0.0], 7.0 * IN);
        s.doubled([30.0, -0.2], [30.0, 20.2], 6.5 * IN);
        s.doubled([30.2, 20.0], [-0.2, 20.0], 7.5 * IN);
        s.doubled([0.0, 20.2], [0.0, -0.2], 8.0 * IN);
        let d = s.turned(turn).found(Q4);
        assert_eq!(d.walls.len(), 4, "turned {turn}: {d:#?}");
        assert!(d.walls.iter().all(|w| w.doubt.is_none()), "turned {turn}: {d:#?}");
        assert_lengths(&d, &[20.0, 20.0, 30.0, 30.0], 1.0 / 12.0);
        assert!(d.walls.iter().all(|w| w.joined == 4), "turned {turn}: {:?}", d.walls.iter().map(|w| w.joined).collect::<Vec<_>>());
        let mut spans: Vec<f64> = d.walls.iter().map(|w| w.thickness * 12.0).collect();
        spans.sort_by(|a, b| a.total_cmp(b));
        for (got, want) in spans.iter().zip([6.5, 7.0, 7.5, 8.0]) {
            assert!(near(*got, want, 1.0 / 8.0), "{spans:?}");
        }
    }
    // A wall found once says it was one pair; the shared house has no wall joined.
    assert!(house().found(Q4).walls.iter().all(|w| w.joined == 1));
}

#[test]
fn a_fixture_drawn_with_lines_beside_one_another_is_one_thing_in_doubt_and_says_how_many_pairs_made_it() {
    // By hand: four lines 4 1/2 ft long, 3 3/4 in, 6 in and 4 1/4 in apart (not evenly spaced, so no set), nothing else on the
    // sheet: the pairs 3 3/4, 9 3/4, 6, 10, 4 1/4 and 14 in (all between 3 in and 14 in) are six pairs of one stack. One thing, in doubt,
    // from the outermost line to the outermost (14 in), joined from 6 pairs.
    let mut s = Sheet::default();
    for y in [0.0, 3.75 * IN, 9.75 * IN, 14.0 * IN] {
        s.line([0.0, y], [4.5, y]);
    }
    let d = s.found(Q4);
    assert_eq!(d.walls.len(), 1, "{d:#?}");
    assert_eq!(d.walls[0].joined, 6, "{d:#?}");
    assert!(d.walls[0].doubt.as_deref().is_some_and(|x| x.contains("another line lies beside it")), "{d:#?}");
}

/// The house of the lead's figures: 30 by 20 ft, 6 in walls, a window 5 ft wide in the south wall (8 to 13 ft), a door 3 ft wide
/// in the east wall (6 ft 6 in to 9 ft 6 in north), and across it at 12 ft a partition 4 1/2 in thick.
fn house_with_a_window_and_a_door() -> Sheet {
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 20.0, 0.5);
    s.cut([0.0, 0.25], [30.0, 0.25], 12.0 - 0.19, 12.0 + 0.19);
    s.cut([0.0, 19.75], [30.0, 19.75], 12.0 - 0.19, 12.0 + 0.19);
    s.partition(12.0, 0.25, 19.75, 0.375);
    s.opening([0.0, 0.0], [30.0, 0.0], 0.5, 8.0, 13.0, true);
    s.opening([30.0, 0.0], [30.0, 20.0], 0.5, 6.5, 9.5, false);
    s
}

fn openings(d: &Drawn) -> (usize, usize) {
    let all: Vec<&Opening> = d.walls.iter().filter(|w| w.doubt.is_none()).flat_map(|w| w.openings.iter()).collect();
    (all.iter().filter(|o| o.window).count(), all.iter().filter(|o| !o.window).count())
}

#[test]
fn glass_lines_inside_a_partition_whose_faces_run_on_are_not_a_window() {
    // A shower's glass 3 ft long in the partition: its faces run on past it, a line across at each end (a cap) and two lines
    // of glass inside its thickness, as the real sheet draws a leaf in a partition. A person reads no window there: a window is in
    // a wall with a face to the outside, or one cut clean through. By hand: the house has 1 window and 1 door, as without it.
    for turn in [0.0, 33.0] {
        let mut s = house_with_a_window_and_a_door();
        for y in [6.0, 9.0] {
            s.line([12.0 - 0.1875, y], [12.0 + 0.1875, y]);
        }
        for x in [12.0 - 0.0625, 12.0 + 0.0625] {
            s.line([x, 6.0], [x, 9.0]);
        }
        let d = s.turned(turn).found(Q4);
        assert_eq!(openings(&d), (1, 1), "turned {turn}: {d:#?}");
        assert!(d.counts.glass_inside >= 1 || d.notes.iter().any(|n| n.contains("glass")), "what was left out is said: {d:#?}");
    }
    // Without the glass, the same: the lead's house has one window and one door.
    assert_eq!(openings(&house_with_a_window_and_a_door().found(Q4)), (1, 1));
}

#[test]
fn a_window_in_an_outside_wall_drawn_with_its_faces_running_on_is_still_a_window() {
    // The faces run on across the window (a white box hides them on paper), jambs across, glass between: in the south wall, which
    // has a face to the outside. A window all the same (the call 181 test of this stays).
    let mut s = Sheet::default();
    s.ring(0.0, 0.0, 30.0, 16.0, 0.5);
    s.line([8.0, -0.25], [8.0, 0.25]).line([13.0, -0.25], [13.0, 0.25]);
    for y in [-0.25, 0.0, 0.25] {
        s.line([8.0, y], [13.0, y]);
    }
    s.partition(18.0, 0.25, 15.75, 0.375);
    s.cut([0.0, 0.25], [30.0, 0.25], 17.8, 18.2);
    s.cut([0.0, 15.75], [30.0, 15.75], 17.8, 18.2);
    let d = s.found(Q4);
    assert_eq!(openings(&d), (1, 0), "{d:#?}");
}

#[test]
fn a_tag_beside_a_door_or_over_a_wall_is_not_an_opening() {
    // A hexagon tag 0.35 ft to a corner, beside the door in the east wall; and another sitting on a wall with its centre on the wall's
    // line, small enough to lie within its thickness. By hand: still 1 window and 1 door.
    for turn in [0.0, 33.0] {
        let mut s = house_with_a_window_and_a_door();
        s.tag([30.9, 8.0], 0.35);
        s.tag([30.0, 15.0], 0.2);
        s.tag([15.0, -0.0], 0.2);
        let d = s.turned(turn).found(Q4);
        assert_eq!(openings(&d), (1, 1), "turned {turn}: {d:#?}");
        assert!(d.walls.iter().all(|w| w.doubt.is_none()), "turned {turn}: {d:#?}");
        assert_lengths(&d, &[20.0, 20.0, 30.0, 30.0, 20.0].iter().copied().filter(|_| true).collect::<Vec<_>>(), 1.0 / 12.0);
        assert!(d.counts.tags >= 3, "the tags are counted and said: {d:#?}");
    }
}

// ---- call 188, item 9: the sheet's own dimension strings ----

fn text(s: &str, at: P, tall: f64, dir: P) -> crate::Text {
    crate::Text { text: s.to_string(), at, tall, dir }
}

/// A dimension line from a to b on the page (points) with a slash tick across each end and an extension line at each.
fn dim_line(lines: &mut Vec<[[f64; 2]; 2]>, a: P, b: P) {
    lines.push([a, b]);
    let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
    let u = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
    let n = [-u[1], u[0]];
    for e in [a, b] {
        let t0 = [e[0] + (u[0] + n[0]) * 4.0, e[1] + (u[1] + n[1]) * 4.0];
        let t1 = [e[0] - (u[0] + n[0]) * 4.0, e[1] - (u[1] + n[1]) * 4.0];
        lines.push([t0, t1]);
        lines.push([[e[0] + n[0] * 8.0, e[1] + n[1] * 8.0], [e[0] - n[0] * 40.0, e[1] - n[1] * 40.0]]);
    }
}

#[test]
fn the_dimension_strings_of_a_page_are_read_each_with_its_line_and_the_scale_they_give_and_the_ones_that_disagree() {
    // By hand: "30'-0\"" by a line 540 points long: 30/540 = 1/18 ft to the point (1/4 in to the foot); "20'-0\"" by a line 360 points
    // long: 20/360 = 1/18. They agree. "12'-0\"" by a line 180 points long: 12/180 = 1/15; at 1/18 that line is 10 ft: it does not agree,
    // and it measures 10'-0" at the scale the other two give. A level one and a rotated one (the vertical dimension reads upward).
    let mut lines: Vec<[[f64; 2]; 2]> = Vec::new();
    dim_line(&mut lines, [126.0, 600.0], [666.0, 600.0]);
    dim_line(&mut lines, [100.0, 130.0], [100.0, 490.0]);
    dim_line(&mut lines, [300.0, 700.0], [480.0, 700.0]);
    let texts = vec![
        text("30'-0\"", [370.0, 604.0], 9.0, [1.0, 0.0]),
        text("20'-0\"", [96.0, 290.0], 9.0, [0.0, 1.0]),
        text("12'-0\"", [370.0, 704.0], 9.0, [1.0, 0.0]),
        text("A101", [500.0, 500.0], 9.0, [1.0, 0.0]),
        text("6\"", [500.0, 520.0], 9.0, [1.0, 0.0]),
        text("8'-0\"", [800.0, 300.0], 9.0, [1.0, 0.0]),
    ];
    let d = dimension_strings(&lines, &[], &texts);
    assert_eq!(d.strings.len(), 3, "{d:#?}");
    let (a, b, c) = (&d.strings[0], &d.strings[1], &d.strings[2]);
    assert!(near(a.feet, 30.0, 1e-9) && near(b.feet, 20.0, 1e-9) && near(c.feet, 12.0, 1e-9), "{d:#?}");
    assert!(near(a.paper, 540.0, 1.0) && near(b.paper, 360.0, 1.0) && near(c.paper, 180.0, 1.0), "{d:#?}");
    assert!(near(d.scale.unwrap(), 1.0 / 18.0, 1e-6), "{d:#?}");
    assert_eq!(d.agree, vec![0, 1], "{d:#?}");
    assert_eq!(d.off.len(), 1, "{d:#?}");
    assert!(d.off[0].0 == 2 && near(d.off[0].1, 10.0, 0.05), "{d:#?}");
    // A string that reads as a dimension with no dimension line beside it is said, and is not used.
    assert_eq!(d.loose.len(), 1, "{d:#?}");
    assert_eq!(d.loose[0].0, "8'-0\"");
    // Inches and feet written the ways sheets write them.
    for (said, feet) in [("5'-8 1/2\"", 5.0 + 8.5 / 12.0), ("2'-11\"", 2.0 + 11.0 / 12.0), ("30' 0\"", 30.0), ("16'-5\"", 16.0 + 5.0 / 12.0), ("10'-0 1/4\"", 10.0 + 0.25 / 12.0)] {
        let mut l: Vec<[[f64; 2]; 2]> = Vec::new();
        dim_line(&mut l, [100.0, 100.0], [100.0 + feet * 18.0, 100.0]);
        let d = dimension_strings(&l, &[], &[text(said, [100.0 + feet * 9.0 - 10.0, 104.0], 9.0, [1.0, 0.0])]);
        assert!(d.strings.len() == 1 && near(d.strings[0].feet, feet, 1e-9), "{said}: {d:#?}");
    }
    // Two strings that disagree with nothing to say which is right: no scale is offered.
    let mut l: Vec<[[f64; 2]; 2]> = Vec::new();
    dim_line(&mut l, [100.0, 100.0], [640.0, 100.0]);
    dim_line(&mut l, [100.0, 300.0], [640.0, 300.0]);
    let d = dimension_strings(&l, &[], &[text("30'-0\"", [300.0, 104.0], 9.0, [1.0, 0.0]), text("25'-0\"", [300.0, 304.0], 9.0, [1.0, 0.0])]);
    assert_eq!(d.strings.len(), 2);
    assert!(d.scale.is_none() && d.agree.is_empty(), "{d:#?}");
}

// ---- call 188, sent back: a drawing is told from every wall found, in doubt or not ----

/// A house whose walls are all in doubt, as the real floor plan's were (each of four pairs of lines meets nothing, the corners open by 3 ft),
/// and 50 ft east of it a clean roof plan of like size.
fn doubtful_house_and_clean_roof() -> Sheet {
    let mut s = Sheet::default();
    // South and north 24 ft, west and east 14 ft, each 6 in thick, none meeting another.
    s.line([3.0, -0.25], [27.0, -0.25]).line([3.0, 0.25], [27.0, 0.25]);
    s.line([3.0, 19.75], [27.0, 19.75]).line([3.0, 20.25], [27.0, 20.25]);
    s.line([-0.25, 3.0], [-0.25, 17.0]).line([0.25, 3.0], [0.25, 17.0]);
    s.line([29.75, 3.0], [29.75, 17.0]).line([30.25, 3.0], [30.25, 17.0]);
    s.ring(80.0, 0.0, 112.0, 22.0, 0.5);
    s
}

#[test]
fn a_house_all_in_doubt_beside_a_clean_roof_plan_of_like_size_is_two_drawings_each_with_its_doubt_said() {
    // By hand: the house's pieces are 24 + 24 + 14 + 14 = 76 LF, every one in doubt (each meets nothing); the roof plan is 108 LF, clean.
    // 76 is over half of 108: of like size, counted over every wall found. Largest first: 1 the roof (108, none in doubt), 2 the house (76, all in doubt).
    for turn in [0.0, 33.0] {
        let d = doubtful_house_and_clean_roof().turned(turn).found(Q4);
        assert_eq!(d.drawings.len(), 2, "turned {turn}: {d:#?}");
        let (a, b) = (&d.drawings[0], &d.drawings[1]);
        assert!(near(a.length, 108.0, 1.0 / 6.0) && a.in_doubt < 0.01 && a.walls.len() == 4, "{a:?}");
        assert!(near(b.length, 76.0, 1.0 / 6.0) && near(b.in_doubt, 76.0, 1.0 / 6.0) && b.walls.len() == 4, "{b:?}");
    }
}
