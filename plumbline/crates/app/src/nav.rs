//! How the view moves (call 218): the wheel from the whole lot to a single hinge, the tilt as it
//! dives, timed flights that frame what they fly to, the north arrow's figures, and the rule by
//! size on screen that says when a thing's name is written over it. Everything here is
//! arithmetic with no window in it, so that each figure is tested without a screen; `scene.rs`
//! and `panels.rs` only apply it.

use bevy::prelude::*;

/// A notch of the wheel closes this much of what is left (call 63).
pub const NOTCH: f32 = 0.9;
/// The nearest the eye comes to what it looks at, in feet: 8 in, a hinge or a receptacle fills the window.
pub const MIN_DISTANCE: f32 = 8.0 / 12.0;
/// Out the wheel goes this many times the distance the job is framed at (1.6 sizes, so the house is about a sixth of the width): the house is then a tenth to a fifth of the window's width.
pub const OUT: f32 = 3.0;
/// Looking nearly straight down, in 3D (a plan is another view).
pub const STRAIGHT_DOWN: f32 = 1.52;

/// How far out the wheel will go: 3 times the distance the job is framed at (that is 1.6 times the building's size, so the
/// building is 0.17 x (its longer side / its size) of the window's width: a tenth to a fifth, on its ground), never under 40 ft,
/// never over 6000 ft, and never less than where the eye already is.
pub fn furthest(home: f32, was: f32) -> f32 {
    (home * OUT).clamp(40.0, 6000.0).max(was)
}

/// The nearest and the furthest of what is drawn, in feet, for an eye this far from what it
/// looks at: a hinge at 6 in is whole, and so is the lot at 400 ft.
pub fn clip_planes(distance: f32) -> (f32, f32) {
    ((distance * 0.05).clamp(0.01, 0.5), (distance * 100.0).max(40_000.0))
}

/// The steepest the view may look down from, in radians above level, at this distance: looking
/// straight down from high above, across from close in (Google Earth leans a view as it dives).
pub fn tilt_limit(distance: f32) -> f32 {
    let degrees = if distance <= 1.0 { 10.0 } else { 10.0 + 80.0 * distance.ln() / 200.0_f32.ln() };
    degrees.clamp(10.0, 90.0).to_radians()
}

/// The tilt after a notch to `distance`. Going in, the view only leans over (never back on
/// itself); going out it rights itself, as far as the tilt the person chose (`wish`).
pub fn tilt_after(pitch: f32, wish: f32, distance: f32, going_in: bool) -> f32 {
    let limit = tilt_limit(distance);
    if going_in { pitch.min(limit) } else { pitch.max(wish.min(limit)) }
}

/// Where the view stands and looks: what a flight carries from one to the other.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

/// Quintic ease in and out: slow, fast, slow (Cesium's easing, call 63).
pub fn quintic(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

/// How long a flight takes by how far it goes: 0.4 s for a short one to 1.2 s for the longest (call 63).
pub fn flight_secs(from: &Pose, to: &Pose) -> f32 {
    let far = from.target.distance(to.target) / from.distance.max(to.distance).max(1.0);
    let travel = far + 0.5 * (to.distance / from.distance.max(1e-3)).ln().abs() + 0.15 * turn_between(from.yaw, to.yaw).abs();
    0.4 + 0.8 * (1.0 - (-0.8 * travel).exp())
}

/// How far to turn from one heading to another, the short way round, in radians.
pub fn turn_between(from: f32, to: f32) -> f32 {
    let d = (to - from) % std::f32::consts::TAU;
    if d > std::f32::consts::PI {
        d - std::f32::consts::TAU
    } else if d < -std::f32::consts::PI {
        d + std::f32::consts::TAU
    } else {
        d
    }
}

/// A flight in the air: the pose it began at, the pose it is going to, and how long it has gone.
#[derive(Clone, Copy, Debug)]
pub struct Flight {
    pub from: Pose,
    pub to: Pose,
    pub gone: f32,
    pub secs: f32,
}

impl Flight {
    pub fn new(from: Pose, to: Pose) -> Flight {
        Flight { from, to, gone: 0.0, secs: flight_secs(&from, &to) }
    }
    /// Time goes on; where the view is now.
    pub fn step(&mut self, dt: f32) -> Pose {
        self.gone = (self.gone + dt).min(self.secs);
        let k = quintic(self.gone / self.secs);
        Pose {
            target: self.from.target.lerp(self.to.target, k),
            yaw: self.from.yaw + turn_between(self.from.yaw, self.to.yaw) * k,
            pitch: self.from.pitch + (self.to.pitch - self.from.pitch) * k,
            distance: self.from.distance * (self.to.distance / self.from.distance.max(1e-3)).powf(k),
        }
    }
    pub fn landed(&self) -> bool {
        self.gone >= self.secs
    }
}

/// How far the eye stands from a thing of this radius (feet) for it to fill `fill` of the
/// window's height (a sphere round it spans that much).
pub fn framing_distance(radius: f32, fill: f32) -> f32 {
    (radius / (fill * (crate::scene::LENS / 2.0).tan())).max(MIN_DISTANCE)
}

/// Whether a thing's name and size are written over it: by how big the thing is on the screen,
/// and for nothing else. `feet` is the thing's size, `away` how far the eye is from it, `height`
/// the window's height in dots. Too small to be what is looked at: no. Bigger than the window
/// shows: no (the house's own name is not lettered over a hinge).
pub fn label_shown(feet: f32, away: f32, height: f32) -> bool {
    let dots = dots_across(feet, away, height);
    dots >= 48.0 && dots <= 1.5 * height
}

/// Where a name is written over a thing that is big on the screen but whose anchor lies outside the clear part of the window (a
/// receptacle that fills the window has its name above it, out of sight): kept inside `lo`..`hi`, so the name is still read. A thing
/// whose anchor is further out than the thing is big (`dots`) is simply off the screen, and its name stays where it is.
pub fn pull_in(at: Vec2, dots: f32, lo: Vec2, hi: Vec2) -> Vec2 {
    let out = Vec2::new((lo.x - at.x).max(at.x - hi.x).max(0.0), (lo.y - at.y).max(at.y - hi.y).max(0.0));
    if out.max_element() > dots {
        at
    } else {
        at.clamp(lo, hi)
    }
}

/// Whether the name and size of a placed thing are written over it with no layer switched on (call 218, round 2): the thing that is
/// selected, or the one that fills the middle of the view. `dots` is its size on the screen, `off_middle` how far its middle is from the
/// middle of the clear part of the window, `height` the window's height, both in dots. Everything else waits for the Sizes layer.
pub fn name_without_layer(selected: bool, dots: f32, off_middle: f32, height: f32) -> bool {
    dots >= 48.0 && dots <= 1.5 * height && (selected || (dots >= 0.25 * height && off_middle <= 0.2 * height))
}

/// How big a thing of `feet` looks, in dots.
pub fn dots_across(feet: f32, away: f32, height: f32) -> f32 {
    feet * height / (2.0 * away.max(0.01) * (crate::scene::LENS / 2.0).tan())
}

/// The north arrow: how far its needle is turned from straight up (radians, clockwise on the
/// screen) and how flat the disc lies (1 looking straight down, three quarters as high across the horizon: it stays a disc a person can read).
#[derive(Clone, Copy, Debug)]
pub struct Arrow {
    pub needle: f32,
    pub flat: f32,
}

pub fn north_arrow(yaw: f32, pitch: f32) -> Arrow {
    Arrow { needle: yaw, flat: 0.75 + 0.25 * pitch.sin().clamp(0.0, 1.0) }
}

/// How much of a step the eye may take before a face is in the way: `hit` is how far along the
/// step a face lies. Stops 0.1 ft short of it.
pub fn stop_short(step: f32, hit: Option<f32>) -> f32 {
    match hit {
        None => 1.0,
        Some(h) => ((h - 0.1).clamp(0.0, step)) / step.max(1e-6),
    }
}

/// How many notches it takes to go from one distance to another.
#[cfg(test)]
pub fn notches_between(from: f32, to: f32) -> u32 {
    ((to / from).ln() / NOTCH.ln()).ceil().max(0.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deg(r: f32) -> f32 {
        r.to_degrees()
    }

    #[test]
    fn the_wheel_runs_from_the_whole_lot_to_a_hinge() {
        // The nearest is eight inches, the furthest is never under the lot, and a big job goes further, to 6000 ft.
        assert_eq!(MIN_DISTANCE, 8.0 / 12.0);
        assert_eq!(furthest(10.0, 10.0), 40.0);
        assert_eq!(furthest(5000.0, 5000.0), 6000.0);
        // Where the eye already is, it is not pulled in by a limit.
        assert_eq!(furthest(60.0, 900.0), 900.0);
        // A tenth of what is left each notch: from the lot to eight inches is about sixty notches.
        let n = notches_between(400.0, 8.0 / 12.0);
        assert!((59..=62).contains(&n), "{n} notches");
        // From the farthest of the made-up house (92.8 ft framed, 2.5 times that out): about 55.
        assert!((56..=60).contains(&notches_between(278.0, 8.0 / 12.0)));
        // 0.9 to the 60th of 400 ft is 0.72 ft, the lead's own working.
        assert!((400.0 * NOTCH.powi(60) - 0.7).abs() < 0.1);
    }

    #[test]
    fn at_the_farthest_the_house_is_a_tenth_to_a_fifth_of_the_windows_width_whatever_its_shape() {
        // The view is framed at 1.6 times the building's size (plan diagonal and height): `scene.rs`, where `home` is set.
        for (w, d, h) in [(24.0_f32, 16.0, 17.0), (42.0, 30.0, 25.0), (120.0, 40.0, 30.0), (60.0, 60.0, 12.0)] {
            let size = (w * w + d * d + h * h).sqrt();
            let home = size * 1.6;
            let far = furthest(home, home);
            let across = 2.0 * far * (crate::scene::LENS / 2.0).tan() * (1600.0 / 960.0);
            let fraction = w.max(d) / across;
            assert!((0.1..=0.2).contains(&fraction), "{w} x {d} x {h}: {fraction} of the width at {far} ft");
        }
        // And never pulled in by a limit below where it is.
        assert_eq!(furthest(60.0, 900.0), 900.0);
    }

    #[test]
    fn the_name_and_size_of_what_is_selected_or_fills_the_middle_need_no_layer() {
        let window = 960.0;
        // Selected and big enough: named. Selected and a speck: not.
        assert!(name_without_layer(true, 120.0, 400.0, window));
        assert!(!name_without_layer(true, 10.0, 0.0, window));
        // Not selected, filling the middle: named.
        assert!(name_without_layer(false, 300.0, 40.0, window));
        // Not selected, in a corner or small: left to the Sizes layer.
        assert!(!name_without_layer(false, 300.0, 600.0, window));
        assert!(!name_without_layer(false, 60.0, 0.0, window));
    }

    #[test]
    fn what_is_drawn_near_and_far_follows_the_distance() {
        // At the lot: a near plane far enough out for the depth buffer; the far one past the whole lot.
        let (near, far) = clip_planes(400.0);
        assert!(near >= 0.4 && near <= 20.0, "near {near}");
        assert!(far >= 40_000.0, "far {far}");
        // At a hinge: nothing is cut away by coming near. The eye is 8 in off what it looks at.
        let (near, far) = clip_planes(MIN_DISTANCE);
        assert!(near <= 0.05, "near {near} would cut a hinge 8 in from the eye");
        assert!(far >= 40_000.0 && far / near < 2.0e6, "depth range {near} to {far}");
        // The nearer the eye, the nearer the near plane: never further out than the eye is from its target.
        let mut last = f32::MAX;
        for d in [400.0, 100.0, 20.0, 5.0, 1.0, MIN_DISTANCE] {
            let (n, _) = clip_planes(d);
            assert!(n <= last && n < d * 0.1, "near {n} at {d}");
            last = n;
        }
    }

    #[test]
    fn the_view_leans_from_looking_down_toward_looking_across_as_it_dives() {
        // High above it is free; close in it is at most a few degrees from level plus a little.
        assert!(deg(tilt_limit(400.0)) >= 89.0);
        let near = deg(tilt_limit(MIN_DISTANCE));
        assert!((5.0..=25.0).contains(&near), "{near} degrees at the nearest");
        // It never steepens as the eye comes in.
        let mut last = f32::MAX;
        for k in 0..70 {
            let l = tilt_limit(400.0 * NOTCH.powi(k));
            assert!(l <= last + 1e-6);
            last = l;
        }
    }

    #[test]
    fn a_dive_started_at_sixty_degrees_ends_between_five_and_twenty_five_and_never_goes_back() {
        let start = 60.0_f32.to_radians();
        let (mut pitch, mut d) = (start, 400.0_f32);
        let mut last = pitch;
        while d > MIN_DISTANCE {
            d = (d * NOTCH).max(MIN_DISTANCE);
            pitch = tilt_after(pitch, start, d, true);
            assert!(pitch <= last + 1e-6, "the tilt went back on itself at {d} ft: {} then {}", deg(last), deg(pitch));
            last = pitch;
        }
        assert!((5.0..=25.0).contains(&deg(pitch)), "ended at {} degrees", deg(pitch));
        // Coming back out it rights itself, to what was chosen and no further.
        for _ in 0..80 {
            d = (d / NOTCH).min(400.0);
            pitch = tilt_after(pitch, start, d, false);
            assert!(pitch >= last - 1e-6 && pitch <= start + 1e-6);
            last = pitch;
        }
        assert!((deg(pitch) - 60.0).abs() < 0.01, "righted to {} degrees", deg(pitch));
    }

    #[test]
    fn a_view_already_looking_across_is_not_leaned_further_than_it_is() {
        // The opening view is 15 degrees up: three notches in leave it as it is.
        let home = 0.27_f32;
        let mut p = home;
        let mut d = 60.0_f32;
        for _ in 0..6 {
            d *= NOTCH;
            p = tilt_after(p, home, d, true);
        }
        assert_eq!(p, home);
    }

    #[test]
    fn a_quintic_ease_is_slow_fast_slow() {
        assert_eq!(quintic(0.0), 0.0);
        assert_eq!(quintic(1.0), 1.0);
        assert!((quintic(0.5) - 0.5).abs() < 1e-6);
        let mut last_step = 0.0;
        let mut x = 0.0;
        // Each step is bigger than the one before until the middle, and smaller after.
        for i in 0..20 {
            let step = quintic((i + 1) as f32 / 20.0) - quintic(i as f32 / 20.0);
            if i < 10 {
                assert!(step >= last_step - 1e-6, "step {i} {step} after {last_step}");
            } else {
                assert!(step <= last_step + 1e-6, "step {i} {step} after {last_step}");
            }
            last_step = step;
            x += step;
        }
        assert!((x - 1.0).abs() < 1e-5);
    }

    fn pose(x: f32, d: f32) -> Pose {
        Pose { target: Vec3::new(x, 0.0, 0.0), yaw: 0.0, pitch: 0.3, distance: d }
    }

    #[test]
    fn a_flight_takes_between_four_tenths_and_one_and_a_fifth_seconds_by_how_far_it_goes() {
        let short = flight_secs(&pose(0.0, 20.0), &pose(0.5, 19.0));
        let long = flight_secs(&pose(0.0, 400.0), &pose(900.0, 3.0));
        assert!((0.4..0.45).contains(&short), "short {short}");
        assert!((1.1..=1.2).contains(&long), "long {long}");
        let mid = flight_secs(&pose(0.0, 60.0), &pose(30.0, 20.0));
        assert!(short < mid && mid < long);
    }

    #[test]
    fn a_flight_arrives_exactly_where_it_was_going_and_its_speed_rises_then_falls() {
        let (a, b) = (pose(0.0, 60.0), Pose { target: Vec3::new(30.0, 3.0, -8.0), yaw: 1.2, pitch: 0.2, distance: 20.0 });
        let mut f = Flight::new(a, b);
        assert!(!f.landed());
        let mut last = a;
        let mut steps = Vec::new();
        for _ in 0..100 {
            let p = f.step(0.02);
            steps.push(p.target.distance(last.target));
            last = p;
            if f.landed() {
                break;
            }
        }
        assert!(f.landed());
        assert!(last.target.distance(b.target) < 1e-4 && (last.distance - 20.0).abs() < 1e-3 && (last.yaw - 1.2).abs() < 1e-5);
        let top = steps.iter().cloned().fold(0.0, f32::max);
        let at = steps.iter().position(|s| *s == top).unwrap();
        for w in steps[..at].windows(2) {
            assert!(w[1] >= w[0] - 1e-5, "speeding up: {} then {}", w[0], w[1]);
        }
        for w in steps[at..].windows(2) {
            assert!(w[1] <= w[0] + 1e-5, "slowing down: {} then {}", w[0], w[1]);
        }
        // And no step goes the wrong way: the distance along the flight only grows.
        let mut d = Flight::new(a, b);
        let mut was = 0.0;
        for _ in 0..80 {
            let p = d.step(0.02);
            let along = p.target.distance(a.target);
            assert!(along >= was - 1e-5);
            was = along;
        }
    }

    #[test]
    fn a_flight_turns_the_short_way_round() {
        let a = Pose { target: Vec3::ZERO, yaw: 3.0, pitch: 0.3, distance: 30.0 };
        let b = Pose { yaw: -3.0, ..a };
        let mut f = Flight::new(a, b);
        let mid = f.step(f.secs / 2.0);
        // Half way from 3.0 to -3.0 by the short way is about pi, not 0.
        assert!(mid.yaw.abs() > 3.0, "went the long way round: {}", mid.yaw);
    }

    #[test]
    fn a_thing_framed_fills_about_half_the_windows_height() {
        // A door 3 ft by 6 2/3 ft: a sphere round it is 3.7 ft in radius.
        let r = (3.0_f32 * 3.0 + 6.667 * 6.667).sqrt() / 2.0;
        let d = framing_distance(r, 0.5);
        let height_seen = 2.0 * d * (crate::scene::LENS / 2.0).tan();
        let fill = 6.667 / height_seen;
        assert!((1.0 / 3.0..=2.0 / 3.0).contains(&fill), "the door fills {fill} of the window");
        // A receptacle: its box is 3 by 5 in.
        let r = (0.25_f32 * 0.25 + 0.4167 * 0.4167).sqrt() / 2.0;
        assert!(framing_distance(r, 0.5) >= MIN_DISTANCE);
    }

    #[test]
    fn a_things_name_is_written_over_it_by_its_size_on_the_screen_and_by_nothing_else() {
        let window = 980.0;
        // A receptacle, 5 in: far out it is a speck, no name; close in it is a thing, named.
        assert!(!label_shown(0.42, 400.0, window));
        assert!(!label_shown(0.42, 60.0, window));
        assert!(label_shown(0.42, 10.0, window));
        assert!(label_shown(0.42, 1.0, window));
        // A roof plane of 25 ft: named when the house is seen, not when a hinge fills the window.
        assert!(label_shown(25.0, 90.0, window));
        assert!(!label_shown(25.0, 0.7, window));
        // Dots across: 5 in at 10 ft is a little more than 40 dots in a window 980 high.
        let d = dots_across(0.42, 10.0, window);
        assert!((50.0..60.0).contains(&d), "{d}");
        // Smaller than 48 dots is no label, and the edge is the same each way it is come to.
        assert!(!label_shown(0.42, 15.0, window) || dots_across(0.42, 15.0, window) >= 48.0);
    }

    #[test]
    fn a_name_over_a_thing_that_fills_the_window_is_kept_where_it_can_be_read() {
        let (lo, hi) = (Vec2::new(300.0, 120.0), Vec2::new(1200.0, 760.0));
        // Inside: where it is.
        assert_eq!(pull_in(Vec2::new(700.0, 300.0), 800.0, lo, hi), Vec2::new(700.0, 300.0));
        // Above the window by less than the thing is big: brought in to the top edge of the clear part, across where it was.
        assert_eq!(pull_in(Vec2::new(700.0, -400.0), 800.0, lo, hi), Vec2::new(700.0, 120.0));
        // To the left by less than the thing is big: brought in.
        assert_eq!(pull_in(Vec2::new(40.0, 300.0), 800.0, lo, hi), Vec2::new(300.0, 300.0));
        // Further out than the thing is big: the thing is off the screen, and so is its name.
        assert_eq!(pull_in(Vec2::new(700.0, -2000.0), 800.0, lo, hi), Vec2::new(700.0, -2000.0));
    }

    #[test]
    fn the_north_arrow_turns_with_the_view_and_leans_with_its_tilt() {
        // North up: the needle straight up, the disc flat when looking down.
        let a = north_arrow(0.0, STRAIGHT_DOWN);
        assert!(a.needle.abs() < 1e-6 && a.flat > 0.99);
        // A quarter turn of the view: the needle is a quarter turn off.
        let q = north_arrow(std::f32::consts::FRAC_PI_2, 0.5);
        assert!((q.needle.abs() - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        // Across the horizon the disc is edge on, but never gone.
        let level = north_arrow(0.0, 0.06);
        assert!(level.flat >= 0.7 && level.flat < 0.85, "{}", level.flat);
        // The lean follows the tilt, as sine.
        assert!(north_arrow(0.0, 1.0).flat > north_arrow(0.0, 0.5).flat);
        // Turning one way and the other is symmetric.
        assert!((north_arrow(0.7, 0.5).needle + north_arrow(-0.7, 0.5).needle).abs() < 1e-6);
    }

    #[test]
    fn the_eye_stops_at_a_face_and_does_not_pass_through() {
        // Nothing in the way: the whole step.
        assert_eq!(stop_short(2.0, None), 1.0);
        // A face 5 ft off with a step of 2: the whole step again.
        assert_eq!(stop_short(2.0, Some(5.0)), 1.0);
        // A face 1 ft along a step of 2: to 0.1 ft short of it, 0.45 of the step.
        assert!((stop_short(2.0, Some(1.0)) - 0.45).abs() < 1e-6);
        // A face closer than that: none of it.
        assert_eq!(stop_short(2.0, Some(0.05)), 0.0);
    }
}
