//! A scan of a building as a point cloud (a PLY file), read and cut into a sheet to
//! trace over (DECISIONS.md call 128).
//!
//! PLY is a short public format: a header in words, then the points as text or as
//! bytes. It is read here and not by a library. Of each point its x, y and z are kept
//! and nothing else. The file does not say what a unit is or which way is up: the
//! caller says, and says so to the person.

/// What one unit of the file is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Units {
    Metres,
    Feet,
}

/// Which of the file's axes points up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Up {
    Z,
    Y,
}

/// The points of a scan, in feet: east, north, up.
#[derive(Clone, Debug)]
pub struct Scan {
    pub points: Vec<[f64; 3]>,
    /// What reading it did not keep, each in a sentence.
    pub lost: Vec<String>,
}

const FEET_PER_METRE: f64 = 1.0 / 0.3048;
/// The most points read: past this a file is refused, not half read.
pub const MOST_POINTS: usize = 60_000_000;

/// How many bytes a property's type takes, and whether it is a real number.
fn width(kind: &str) -> Option<(usize, bool)> {
    Some(match kind {
        "char" | "uchar" | "int8" | "uint8" => (1, false),
        "short" | "ushort" | "int16" | "uint16" => (2, false),
        "int" | "uint" | "int32" | "uint32" => (4, false),
        "float" | "float32" => (4, true),
        "double" | "float64" => (8, true),
        _ => return None,
    })
}

/// Read a PLY file's points.
pub fn read_ply(bytes: &[u8], units: Units, up: Up) -> Result<Scan, String> {
    if !bytes.starts_with(b"ply") {
        return Err("it is not a PLY file (a PLY file begins with the word ply)".into());
    }
    let end = bytes.windows(11).position(|w| w == b"end_header\n").or_else(|| bytes.windows(12).position(|w| w == b"end_header\r\n").map(|i| i + 1)).ok_or("its heading has no end (no line saying end_header)")?;
    let head = String::from_utf8_lossy(&bytes[..end]).to_string();
    let body = &bytes[end + 11..];
    let (mut format, mut count, mut in_vertex, mut before_vertex) = (None, None, false, false);
    // How many faces a mesh file has (a scan saved as a mesh): its points are read, its
    // faces are not, and that is said.
    let mut faces = 0usize;
    // Each property of a point: its name, how many bytes, whether a real number.
    let mut props: Vec<(String, usize, bool, String)> = Vec::new();
    for line in head.lines().map(str::trim) {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["format", f, ..] => format = Some(f.to_string()),
            ["element", "vertex", n] => {
                count = Some(n.parse::<usize>().map_err(|_| format!("it says it has {n:?} points, which is not a count"))?);
                in_vertex = true;
            }
            ["element", "face", n] => {
                faces = n.parse::<usize>().unwrap_or(0);
                before_vertex |= count.is_none();
                in_vertex = false;
            }
            ["element", ..] => {
                before_vertex |= count.is_none();
                in_vertex = false;
            }
            ["property", "list", ..] if in_vertex => return Err("its points carry a list, which this does not read".into()),
            ["property", kind, name] if in_vertex => {
                let (bytes, real) = width(kind).ok_or_else(|| format!("its points carry a {kind:?}, which is not a kind of number a PLY file has"))?;
                props.push((name.to_string(), bytes, real, kind.to_string()));
            }
            _ => {}
        }
    }
    let format = format.ok_or("its heading does not say whether it is text or bytes")?;
    let count = count.ok_or("it has no points (no line saying element vertex)")?;
    if before_vertex {
        return Err("something comes before its points, which this does not read: save it again with the points first".into());
    }
    if count > MOST_POINTS {
        return Err(format!("it has {count} points, more than the {MOST_POINTS} this reads: thin it in the program it was cleaned in"));
    }
    let at = |name: &str| props.iter().position(|p| p.0 == name);
    let (Some(ix), Some(iy), Some(iz)) = (at("x"), at("y"), at("z")) else {
        return Err("its points have no x, y and z".into());
    };
    let mut raw: Vec<[f64; 3]> = Vec::with_capacity(count);
    match format.as_str() {
        "ascii" => {
            let text = String::from_utf8_lossy(body);
            let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
            for (n, line) in lines.iter().enumerate().take(count) {
                let cells: Vec<&str> = line.split_whitespace().collect();
                let get = |i: usize| cells.get(i).and_then(|c| c.parse::<f64>().ok());
                match (get(ix), get(iy), get(iz)) {
                    (Some(x), Some(y), Some(z)) => raw.push([x, y, z]),
                    // The file's last line, broken off: the file is cut short, said below.
                    _ if n + 1 == lines.len() && !text.ends_with('\n') => break,
                    _ => return Err(format!("point {} is not three numbers: {:?}", raw.len() + 1, line.chars().take(40).collect::<String>())),
                }
            }
        }
        "binary_little_endian" | "binary_big_endian" => {
            let little = format == "binary_little_endian";
            let stride: usize = props.iter().map(|p| p.1).sum();
            let offset = |i: usize| props[..i].iter().map(|p| p.1).sum::<usize>();
            let number = |row: &[u8], i: usize| -> f64 {
                let (o, (_, n, real, kind)) = (offset(i), &props[i]);
                let mut b = [0u8; 8];
                b[..*n].copy_from_slice(&row[o..o + n]);
                if !little {
                    b[..*n].reverse();
                }
                match (*n, *real, kind.starts_with('u')) {
                    (4, true, _) => f64::from(f32::from_le_bytes([b[0], b[1], b[2], b[3]])),
                    (8, true, _) => f64::from_le_bytes(b),
                    (1, _, true) => f64::from(b[0]),
                    (1, _, false) => f64::from(b[0] as i8),
                    (2, _, true) => f64::from(u16::from_le_bytes([b[0], b[1]])),
                    (2, _, false) => f64::from(i16::from_le_bytes([b[0], b[1]])),
                    (_, _, true) => f64::from(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
                    _ => f64::from(i32::from_le_bytes([b[0], b[1], b[2], b[3]])),
                }
            };
            for row in body.chunks_exact(stride).take(count) {
                raw.push([number(row, ix), number(row, iy), number(row, iz)]);
            }
        }
        other => return Err(format!("it is written as {other:?}, which is not text or bytes as a PLY file has them")),
    }
    if raw.len() < count {
        return Err(format!("it is cut short: it says {count} points and holds {}", raw.len()));
    }
    if raw.iter().any(|p| p.iter().any(|c| !c.is_finite())) {
        return Err("a point in it is not a number".into());
    }
    let k = if units == Units::Metres { FEET_PER_METRE } else { 1.0 };
    // East, north, up. With y up, north is the file's minus z (a right-handed turn).
    let points = raw.into_iter().map(|[x, y, z]| if up == Up::Z { [x * k, y * k, z * k] } else { [x * k, -z * k, y * k] }).collect();
    let others: Vec<&str> = props.iter().map(|p| p.0.as_str()).filter(|n| !matches!(*n, "x" | "y" | "z")).collect();
    let mut lost = Vec::new();
    if !others.is_empty() {
        // A file with a great many properties (a splat file has sixty) is not listed in full.
        let named = if others.len() > 8 { format!("{} and {} more", others[..8].join(", "), others.len() - 8) } else { others.join(", ") };
        lost.push(format!("each point's {named} (only where a point is is kept)"));
    }
    if faces > 0 {
        lost.push(format!("its {faces} faces (the surface drawn over the points: only where the points are is kept)"));
    }
    Ok(Scan { points, lost })
}

impl Scan {
    /// The least and the most of the points, east, north and up, in feet.
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        let first = *self.points.first()?;
        Some(self.points.iter().fold((first, first), |(lo, hi), p| ([lo[0].min(p[0]), lo[1].min(p[1]), lo[2].min(p[2])], [hi[0].max(p[0]), hi[1].max(p[1]), hi[2].max(p[2])])))
    }

    /// How many points lie at each inch of height, from the lowest point up.
    fn by_height(&self) -> Option<(f64, Vec<usize>)> {
        let (lo, hi) = self.bounds()?;
        let inch = 1.0 / 12.0;
        let bins = (((hi[2] - lo[2]) / inch).ceil() as usize + 1).min(100_000);
        let mut count = vec![0usize; bins];
        for p in &self.points {
            count[(((p[2] - lo[2]) / inch).round() as usize).min(bins - 1)] += 1;
        }
        Some((lo[2], count))
    }

    /// The floor and the ceiling: the two heights, at least 5 ft apart, where most
    /// points lie (a floor and a ceiling are each a great many points at one height;
    /// a wall is few at each). Each is the fullest inch of its group of full inches,
    /// not the group's edge: a scan that leans spreads a floor over several inches,
    /// and the edges read the room 3 in too tall. Either is None where the scan does
    /// not show it.
    pub fn floor_and_ceiling(&self) -> (Option<f64>, Option<f64>) {
        let Some((low, count)) = self.by_height() else { return (None, None) };
        let most = count.iter().copied().max().unwrap_or(0);
        // A level surface: an inch holding more than a third of what the fullest holds.
        let full: Vec<usize> = (0..count.len()).filter(|i| count[*i] * 3 > most && count[*i] > 0).collect();
        let height = |i: usize| low + i as f64 / 12.0;
        // The fullest inch among those within 6 in of an end of the list.
        let fullest = |from: usize, near: &dyn Fn(usize) -> bool| full.iter().copied().filter(|i| near(*i)).max_by_key(|i| count[*i]).unwrap_or(from);
        let (Some(first), Some(last)) = (full.first().copied(), full.last().copied()) else { return (None, None) };
        let floor = fullest(first, &|i| i <= first + 6);
        let ceiling = fullest(last, &|i| i + 6 >= last);
        (Some(height(floor)), Some(height(ceiling)).filter(|c| c - height(floor) >= 5.0))
    }

    /// Level the scan by its own floor: fit a flat plane to the points within 6 in of
    /// the floor and take its slope out of every point. A scan made by hand is seldom
    /// level, and a cut through a leaning scan is not a plan. Returns how far it leaned,
    /// in feet for each foot; None where no floor shows.
    pub fn level_by_floor(&mut self) -> Option<f64> {
        // Three times over: the walls' feet, caught with the floor, pull a first fit
        // toward half the lean, and each pass takes out most of what the last left.
        let mut leaned: Option<[f64; 2]> = None;
        for _ in 0..3 {
            let [a, b] = self.level_once()?;
            leaned = Some(leaned.map_or([a, b], |[x, y]| [x + a, y + b]));
        }
        leaned.map(|[a, b]| a.hypot(b))
    }

    fn level_once(&mut self) -> Option<[f64; 2]> {
        let floor = self.floor_and_ceiling().0?;
        let near: Vec<[f64; 3]> = self.points.iter().copied().filter(|p| (p[2] - floor).abs() <= 0.5).collect();
        if near.len() < 100 {
            return None;
        }
        let n = near.len() as f64;
        let (mx, my, mz) = (near.iter().map(|p| p[0]).sum::<f64>() / n, near.iter().map(|p| p[1]).sum::<f64>() / n, near.iter().map(|p| p[2]).sum::<f64>() / n);
        let (mut sxx, mut sxy, mut syy, mut sxz, mut syz) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for p in &near {
            let (x, y, z) = (p[0] - mx, p[1] - my, p[2] - mz);
            sxx += x * x;
            sxy += x * y;
            syy += y * y;
            sxz += x * z;
            syz += y * z;
        }
        let det = sxx * syy - sxy * sxy;
        if det.abs() < 1.0e-9 {
            return None;
        }
        let (a, b) = ((sxz * syy - syz * sxy) / det, (syz * sxx - sxz * sxy) / det);
        for p in self.points.iter_mut() {
            p[2] -= a * (p[0] - mx) + b * (p[1] - my);
        }
        Some([a, b])
    }

    /// The points within `band` feet of a height, seen from overhead with north up, as
    /// a picture: `dots` to the foot, with `margin` feet of paper all round.
    pub fn cut(&self, height: f64, band: f64, dots: f64, margin: f64) -> Result<Cut, String> {
        let at: Vec<[f64; 3]> = self.points.iter().copied().filter(|p| (p[2] - height).abs() <= band).collect();
        if at.is_empty() {
            return Err("no point of it is at that height: nothing to draw".into());
        }
        // A stray is a point with no company: a wall puts many points in each square
        // foot of plan it crosses, a stray through a window or off a mirror one. Only
        // points in a square foot that holds 3 or more are the building; the sheet is
        // fitted to those, and the rest are left off and counted. (Fitted to every
        // point, a house 30 ft by 20 ft with strays round it read as 69 ft by 59 ft.)
        let cell = |p: &[f64; 3]| ((p[0]).floor() as i64, (p[1]).floor() as i64);
        let mut held: std::collections::HashMap<(i64, i64), usize> = std::collections::HashMap::new();
        for p in &at {
            *held.entry(cell(p)).or_default() += 1;
        }
        let kept: Vec<[f64; 3]> = at.iter().copied().filter(|p| held[&cell(p)] >= 3).collect();
        // A thin scan (few points to the foot) has no square foot with 3: keep them all.
        let (kept, strays) = if kept.len() * 2 >= at.len() { let n = at.len() - kept.len(); (kept, n) } else { (at, 0) };
        let first = kept[0];
        let (lo, hi) = kept.iter().fold(([first[0], first[1]], [first[0], first[1]]), |(lo, hi), p| ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])]));
        let (wide, high) = ((hi[0] - lo[0] + 2.0 * margin) * dots, (hi[1] - lo[1] + 2.0 * margin) * dots);
        let (w, h) = (wide.round().max(1.0) as u32, high.round().max(1.0) as u32);
        if w > 12_000 || h > 12_000 {
            return Err(format!("it is {} ft by {} ft on plan, too large to draw as one sheet: cut it to the building in the program it was cleaned in", (hi[0] - lo[0]).round(), (hi[1] - lo[1]).round()));
        }
        let mut picture = image::GrayImage::from_pixel(w, h, image::Luma([255u8]));
        for p in &kept {
            // To the nearest dot: a point on a dot's edge (a wall at a whole foot) is
            // the same dot whether its file kept it as text or as bytes.
            let (x, y) = (((p[0] - lo[0] + margin) * dots).round() as i64, ((p[1] - lo[1] + margin) * dots).round() as i64);
            // A point is a dot four across (two inches at 24 to the foot), so that a wall
            // of points reads as a line on the screen at the size a house is seen at:
            // at two across, the sheet was sampled between its dots and the walls read
            // 222 of 255, too faint to trace over (call 153). The dot is a mid gray, 100
            // of 255: a wall drawn over it in ink still shows against it. The dot's
            // centre is where it was.
            for dx in -1..=2 {
                for dy in -1..=2 {
                    let (px, py) = (x + dx, i64::from(h) - 1 - (y + dy));
                    if px >= 0 && py >= 0 && px < i64::from(w) && py < i64::from(h) {
                        picture.put_pixel(px as u32, py as u32, image::Luma([100u8]));
                    }
                }
            }
        }
        let mut png = Vec::new();
        picture.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).map_err(|e| format!("the picture of it could not be made: {e}"))?;
        // How far the building runs: between the hundredth and the ninety-ninth of every
        // hundred of its points each way, so that a last stray beside a wall, or the
        // few points furthest out of a wall's fuzz, do not stretch it.
        let span = |axis: usize| {
            let mut all: Vec<f64> = kept.iter().map(|p| p[axis]).collect();
            all.sort_by(f64::total_cmp);
            // (A handful of points has no hundredths to speak of: end to end.)
            if all.len() < 200 { all[all.len() - 1] - all[0] } else { all[(all.len() - 1) * 99 / 100] - all[(all.len() - 1) / 100] }
        };
        Ok(Cut { png, size: [w, h], dots, corner: [lo[0] - margin, lo[1] - margin], points: kept.len(), strays, extent: [span(0), span(1)] })
    }
}

/// A scan cut at a height, as a sheet.
#[derive(Clone, Debug)]
pub struct Cut {
    pub png: Vec<u8>,
    /// Its width and height in dots.
    pub size: [u32; 2],
    /// Dots to the foot.
    pub dots: f64,
    /// Where its lower-left corner is in the scan's own feet, east and north.
    pub corner: [f64; 2],
    /// How many points are drawn on it.
    pub points: usize,
    /// How many points at that height were left off as strays (alone in their square foot).
    pub strays: usize,
    /// How far the building runs on the sheet, east to west and north to south, in feet.
    pub extent: [f64; 2],
}

/// What the person said of a scan file; the rest is worked out and said.
pub struct Said {
    pub units: Units,
    /// None: up is the way the house is shortest, as far as that shows.
    pub up: Option<Up>,
    /// How high above the floor the sheet is cut, in feet.
    pub cut_at: f64,
}

/// The sheet made from a scan, handed to whoever lays it under a level.
pub struct Sheet<'a> {
    /// The scan's own name, without its extension.
    pub called: &'a str,
    /// What was taken for granted: "taken as meters with z up".
    pub taken: &'a str,
    pub cut_at: f64,
    pub png: &'a [u8],
    pub checksum: &'a str,
    /// The picture's size in dots, as the reader of pictures gives it.
    pub size: [f64; 2],
    pub cut: &'a Cut,
}

/// What bringing a scan in came to, for the caller to say in its own words.
pub struct Brought {
    pub called: String,
    pub taken: String,
    /// Why up is the way it was taken.
    pub why_up: &'static str,
    pub points: usize,
    /// How far it leaned, in feet for each foot, where it was levelled by its own floor.
    pub leaned: Option<f64>,
    pub floor: Option<f64>,
    pub ceiling: Option<f64>,
    pub base: f64,
    pub cut: Cut,
    /// What laying the sheet returned (the drawing's path).
    pub laid: String,
    pub lost: Vec<String>,
}

/// Bring a scan in: read it, say which way is up (as it was said, or the way the house
/// is shortest), level it by its floor, cut it above its floor into a sheet, and hand the
/// sheet to `lay`, which puts it under a level. Both the command line and the window
/// call this and nothing else (call 153); each says the result in its own words.
/// `label` is how errors name the file.
pub fn bring_in(bytes: &[u8], label: &str, stem: &str, said: &Said, lay: &mut dyn FnMut(&Sheet) -> Result<String, String>) -> Result<Brought, String> {
    let (up, why_up) = match said.up {
        Some(up) => (up, "as you said"),
        None => {
            // The file's own axes, as read with z up: its y and its z, how far each runs.
            let flat = read_ply(bytes, said.units, Up::Z).map_err(|why| format!("{label}: {why}"))?;
            let (lo, hi) = flat.bounds().ok_or_else(|| format!("{label}: it has no points"))?;
            let (along_y, along_z) = (hi[1] - lo[1], hi[2] - lo[2]);
            if along_y < 0.8 * along_z {
                (Up::Y, "the way the house is shortest")
            } else if along_z < 0.8 * along_y {
                (Up::Z, "the way the house is shortest")
            } else {
                (Up::Z, "its height does not show which way is up")
            }
        }
    };
    let mut scan = read_ply(bytes, said.units, up).map_err(|why| format!("{label}: {why}"))?;
    // A scan made by hand is seldom level: it is levelled by its own floor, and how far
    // it leaned is said.
    let leaned = scan.level_by_floor();
    let taken = format!("taken as {} with {} up", if said.units == Units::Metres { "meters" } else { "feet" }, if up == Up::Z { "z" } else { "y" });
    let (lo, _) = scan.bounds().ok_or_else(|| format!("{label}: it has no points"))?;
    let (floor, ceiling) = scan.floor_and_ceiling();
    // Cut above the floor; where no floor shows, above the lowest point, and said.
    let base = floor.unwrap_or(lo[2]);
    let cut = scan.cut(base + said.cut_at, 2.0 / 12.0, 24.0, 2.0).map_err(|why| format!("{label}: {why}"))?;
    let read = crate::read(&cut.png, 1).map_err(|why| format!("{label}: the sheet made from it: {why}"))?;
    let checksum = crate::checksum(&cut.png);
    let laid = lay(&Sheet { called: stem, taken: &taken, cut_at: said.cut_at, png: &cut.png, checksum: &checksum, size: read.size, cut: &cut })?;
    Ok(Brought { called: stem.to_string(), taken, why_up, points: scan.points.len(), leaned, floor, ceiling, base, cut, laid, lost: scan.lost })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(points: &[[f64; 3]]) -> Vec<u8> {
        let mut s = format!("ply\nformat ascii 1.0\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nend_header\n", points.len());
        for p in points {
            s.push_str(&format!("{} {} {}\n", p[0], p[1], p[2]));
        }
        s.into_bytes()
    }

    #[test]
    fn metres_with_z_up_are_feet_east_north_and_up() {
        let scan = read_ply(&text(&[[0.3048, 0.6096, 0.9144]]), Units::Metres, Up::Z).unwrap();
        let p = scan.points[0];
        assert!((p[0] - 1.0).abs() < 1e-9 && (p[1] - 2.0).abs() < 1e-9 && (p[2] - 3.0).abs() < 1e-9, "{p:?}");
        // The same place written in feet with y up: east 1, up 3, south 2.
        let p = read_ply(&text(&[[1.0, 3.0, -2.0]]), Units::Feet, Up::Y).unwrap().points[0];
        assert_eq!(p, [1.0, 2.0, 3.0]);
    }

    /// A scan as a phone app writes it: bytes, little-endian, each point's place as three
    /// real numbers and its colour as three whole ones (with a fourth, opacity), the
    /// triangles of a mesh after the points. Made up; written to the shape the format
    /// allows, since the app's own page does not say which it uses.
    fn phone_bytes(points: &[[f32; 3]], faces: usize, normals: bool) -> Vec<u8> {
        let mut out = format!("ply\nformat binary_little_endian 1.0\ncomment made up\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\n", points.len()).into_bytes();
        if normals {
            out.extend_from_slice(b"property float nx\nproperty float ny\nproperty float nz\n");
        }
        out.extend_from_slice(b"property uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\n");
        if faces > 0 {
            out.extend_from_slice(format!("element face {faces}\nproperty list uchar int vertex_indices\n").as_bytes());
        }
        out.extend_from_slice(b"end_header\n");
        for (i, p) in points.iter().enumerate() {
            for c in p {
                out.extend_from_slice(&c.to_le_bytes());
            }
            if normals {
                for c in [0.0f32, 0.0, 1.0] {
                    out.extend_from_slice(&c.to_le_bytes());
                }
            }
            out.extend_from_slice(&[(i % 256) as u8, 120, 200, 255]);
        }
        for f in 0..faces {
            out.push(3);
            for k in 0..3i32 {
                out.extend_from_slice(&(((f as i32 + k) % points.len() as i32).to_le_bytes()));
            }
        }
        out
    }

    #[test]
    fn a_phone_scan_with_colour_and_faces_is_read_by_its_points() {
        let points = [[0.0f32, 0.0, 0.0], [3.048, 0.0, 0.0], [3.048, 2.0, 2.4384], [0.0, 2.0, 2.4384]];
        for (faces, normals) in [(0, false), (2, false), (2, true)] {
            let scan = read_ply(&phone_bytes(&points, faces, normals), Units::Metres, Up::Z).unwrap();
            assert_eq!(scan.points.len(), 4, "faces {faces} normals {normals}");
            let (p, q) = (scan.points[1], scan.points[2]);
            assert!((p[0] - 10.0).abs() < 1e-4 && (q[2] - 8.0).abs() < 1e-4, "{p:?} {q:?}");
            // The loss is said: the colour (and normals), and the faces where there are any.
            assert!(scan.lost.iter().any(|l| l.contains("red, green, blue, alpha")), "{:?}", scan.lost);
            assert_eq!(scan.lost.iter().any(|l| l.contains("2 faces")), faces > 0, "{:?}", scan.lost);
        }
        // Metres with y up, as an app that keeps the phone's own axes would write it:
        // the same corner is east 10 ft, north 0 and 8 ft up.
        let up = read_ply(&phone_bytes(&[[3.048, 2.4384, 0.0]], 0, false), Units::Metres, Up::Y).unwrap().points[0];
        assert!((up[0] - 10.0).abs() < 1e-4 && up[1].abs() < 1e-9 && (up[2] - 8.0).abs() < 1e-4, "{up:?}");
        // Cut short anywhere in the points it is refused, in words, not read part way.
        let mut cut = phone_bytes(&points, 0, false);
        cut.truncate(cut.len() - 5);
        assert!(read_ply(&cut, Units::Metres, Up::Z).unwrap_err().contains("cut short"));
    }

    #[test]
    fn what_is_not_a_scan_is_refused_in_a_sentence() {
        assert!(read_ply(b"solid not a ply", Units::Metres, Up::Z).unwrap_err().contains("not a PLY file"));
        assert!(read_ply(b"ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nend_header\n1\n", Units::Metres, Up::Z).unwrap_err().contains("no x, y and z"));
        let mut short = text(&[[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);
        short.truncate(short.len() - 6);
        assert!(read_ply(&short, Units::Metres, Up::Z).unwrap_err().contains("cut short"));
    }

    #[test]
    fn a_floor_and_a_ceiling_are_found_and_a_cut_is_to_scale() {
        // A floor and a ceiling 8 ft apart, 10 ft by 6 ft, a point every half foot,
        // and one wall of points between them.
        let mut pts = Vec::new();
        for i in 0..=20 {
            for j in 0..=12 {
                pts.push([i as f64 * 0.5, j as f64 * 0.5, 0.0]);
                pts.push([i as f64 * 0.5, j as f64 * 0.5, 8.0]);
            }
        }
        for k in 1..16 {
            for i in 0..=20 {
                pts.push([i as f64 * 0.5, 0.0, k as f64 * 0.5]);
            }
        }
        let scan = read_ply(&text(&pts), Units::Feet, Up::Z).unwrap();
        let (floor, ceiling) = scan.floor_and_ceiling();
        assert_eq!((floor.map(|f| (f * 12.0).round()), ceiling.map(|c| (c * 12.0).round())), (Some(0.0), Some(96.0)));
        // Cut 4 ft up: only the wall's row of 21 points, 24 dots to the foot, 2 ft of margin.
        let cut = scan.cut(4.0, 2.0 / 12.0, 24.0, 2.0).unwrap();
        assert_eq!((cut.size, cut.points, cut.corner), ([336, 96], 21, [-2.0, -2.0]));
        // (The sheet is fitted to what is cut, the one wall, 10 ft long: (10 + 4) x 24
        // by (0 + 4) x 24 dots. It was fitted to every point of the scan.)
        assert_eq!((cut.strays, cut.extent), (0, [10.0, 0.0]));
    }
}
