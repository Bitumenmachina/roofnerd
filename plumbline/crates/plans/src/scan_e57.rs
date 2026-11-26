//! E57 (ASTM E2807): the vendor-neutral file a laser scanner exports (DECISIONS.md call 230).
//!
//! Read by the `e57` crate (MIT, pure Rust, no unsafe code). The standard keeps every length in meters ("all
//! quantities standardized in this specification are expressed in terms of SI units") and says of axes only that
//! they are right-handed: which way is up is not said. A file holds one scan or several, each with its own pose
//! (a position and a turn into the file's one frame); each is placed by it.

use crate::scan::{MOST_POINTS, Units};
use crate::scan_las::Raw;

/// What an E57 file says of its unit: meters, always.
pub(crate) fn unit() -> (Units, String) {
    (Units::Metres, "meters".into())
}

pub(crate) fn read(bytes: &[u8]) -> Result<Raw, String> {
    let cut = || "it is cut short or damaged: an E57 file ends with its own list of what it holds, and this one's does not read".to_string();
    let mut file = e57::E57Reader::new(std::io::Cursor::new(bytes)).map_err(|_| cut())?;
    let clouds = file.pointclouds();
    if clouds.is_empty() {
        return Err("it holds no scan (no list of points)".into());
    }
    let total: u64 = clouds.iter().map(|c| c.records).sum();
    if total > MOST_POINTS as u64 {
        return Err(format!("it has {total} points, more than the {MOST_POINTS} this reads: thin it in the program it was cleaned in"));
    }
    let mut points = Vec::with_capacity(total as usize);
    let (mut none, mut colour, mut intensity) = (0usize, false, false);
    for cloud in &clouds {
        colour |= cloud.has_color();
        intensity |= cloud.has_intensity();
        let mut each = file.pointcloud_simple(cloud).map_err(|_| cut())?;
        each.spherical_to_cartesian(true);
        each.apply_pose(true);
        each.intensity_to_color(false);
        each.normalize_intensity(false);
        each.normalize_color(false);
        for p in each {
            let p = p.map_err(|_| "it is cut short or damaged: its points stop before the end of its list".to_string())?;
            match p.cartesian {
                e57::CartesianCoordinate::Valid { x, y, z } => points.push([x, y, z]),
                _ => none += 1,
            }
        }
    }
    let mut lost = Vec::new();
    if clouds.len() > 1 {
        lost.push(format!("which of its {} scans each point came from (each scan is placed by its own position and turn in the file, and the points are one cloud)", clouds.len()));
    }
    let named: Vec<&str> = [(intensity, "intensity"), (colour, "colour")].iter().filter(|(on, _)| *on).map(|(_, n)| *n).collect();
    if !named.is_empty() {
        lost.push(format!("each point's {} (only where a point is is kept)", named.join(" and ")));
    }
    if none > 0 {
        lost.push(format!("{none} points the scanner marks as having no place (no return), left out"));
    }
    let pictures = file.images().len();
    if pictures > 0 {
        lost.push(format!("its {pictures} pictures (the photographs taken with the scan)"));
    }
    Ok(Raw { points, lost })
}
