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
        lost.push(format!("each point's {} (only where a point is is kept)", others.join(", ")));
    }
    Ok(Scan { points, lost })
}

impl Scan {
    /// The least and the most of the points, east, north and up, in feet.
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        let first = *self.points.first()?;
        Some(self.points.iter().fold((first, first), |(lo, hi), p| ([lo[0].min(p[0]), lo[1].min(p[1]), lo[2].min(p[2])], [hi[0].max(p[0]), hi[1].max(p[1]), hi[2].max(p[2])])))
    }

    /// The floor and the ceiling: the two heights, at least 5 ft apart, where most
    /// points lie (a floor and a ceiling are each a great many points at one height;
    /// a wall is few at each). Either is None where the scan does not show it.
    pub fn floor_and_ceiling(&self) -> (Option<f64>, Option<f64>) {
        let Some((lo, hi)) = self.bounds() else { return (None, None) };
        let inch = 1.0 / 12.0;
        let bins = (((hi[2] - lo[2]) / inch).ceil() as usize + 1).min(100_000);
        let mut count = vec![0usize; bins];
        for p in &self.points {
            count[(((p[2] - lo[2]) / inch).round() as usize).min(bins - 1)] += 1;
        }
        let most = count.iter().copied().max().unwrap_or(0);
        // A level surface: a height holding more than a third of what the fullest holds.
        let level: Vec<usize> = (0..bins).filter(|i| count[*i] * 3 > most && count[*i] > 0).collect();
        let height = |i: usize| lo[2] + i as f64 * inch;
        let floor = level.first().map(|i| height(*i));
        let ceiling = level.last().map(|i| height(*i)).filter(|c| floor.is_some_and(|f| c - f >= 5.0));
        (floor, ceiling)
    }

    /// The points within `band` feet of a height, seen from overhead with north up, as
    /// a picture: `dots` to the foot, with `margin` feet of paper all round.
    pub fn cut(&self, height: f64, band: f64, dots: f64, margin: f64) -> Result<Cut, String> {
        let (lo, hi) = self.bounds().ok_or("it has no points")?;
        let (wide, high) = ((hi[0] - lo[0] + 2.0 * margin) * dots, (hi[1] - lo[1] + 2.0 * margin) * dots);
        let (w, h) = (wide.round().max(1.0) as u32, high.round().max(1.0) as u32);
        if w > 12_000 || h > 12_000 {
            return Err(format!("it is {} ft by {} ft on plan, too large to draw as one sheet: cut it to the building in the program it was cleaned in", (hi[0] - lo[0]).round(), (hi[1] - lo[1]).round()));
        }
        let mut picture = image::GrayImage::from_pixel(w, h, image::Luma([255u8]));
        let mut held = 0usize;
        for p in self.points.iter().filter(|p| (p[2] - height).abs() <= band) {
            held += 1;
            // To the nearest dot: a point on a dot's edge (a wall at a whole foot) is
            // the same dot whether its file kept it as text or as bytes.
            let (x, y) = (((p[0] - lo[0] + margin) * dots).round() as i64, ((p[1] - lo[1] + margin) * dots).round() as i64);
            // A point is a dot two across, so a wall of points reads as a line.
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (px, py) = (x + dx, i64::from(h) - 1 - (y + dy));
                if px >= 0 && py >= 0 && px < i64::from(w) && py < i64::from(h) {
                    picture.put_pixel(px as u32, py as u32, image::Luma([20u8]));
                }
            }
        }
        if held == 0 {
            return Err("no point of it is at that height: nothing to draw".into());
        }
        let mut png = Vec::new();
        picture.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).map_err(|e| format!("the picture of it could not be made: {e}"))?;
        Ok(Cut { png, size: [w, h], dots, corner: [lo[0] - margin, lo[1] - margin], points: held })
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
        // Cut 4 ft up: only the wall's row of 21 points; 24 dots to the foot and 2 ft
        // of margin make (10 + 4) x 24 = 336 by (6 + 4) x 24 = 240 dots.
        let cut = scan.cut(4.0, 2.0 / 12.0, 24.0, 2.0).unwrap();
        assert_eq!((cut.size, cut.points, cut.corner), ([336, 240], 21, [-2.0, -2.0]));
    }
}
