//! LAS, and its packed kind LAZ: a scanning crew's or a surveyor's point cloud (DECISIONS.md call 230).
//!
//! Written from the publisher's own text, "LAS Specification, Version 1.4 - R13" (ASPRS, 2013): a
//! public header block (its table 3), variable length records, the points, each point a record of
//! one of eleven formats (tables 7 to 21) whose first twelve bytes are x, y and z as whole numbers,
//! a point's place being "(record x scale) + offset" (the specification's own words). Everything is
//! little-endian. `research/scan-files.md` has each thing relied on with the line quoted.
//!
//! Read: LAS 1.1 to 1.4, record formats 0 to 10. Refused by name: any other version (1.0 and 2.x),
//! any other record format. LAZ (LASzip's packing, the points of one of these formats packed in
//! chunks) is read by the `laz` crate (Apache-2.0).

use crate::scan::{FileSays, Units};

/// What reading gave: the points as the file holds them (its own unit, its own axes), and what was left.
pub(crate) struct Raw {
    pub points: Vec<[f64; 3]>,
    pub lost: Vec<String>,
}

/// The smallest record each point format takes, in bytes (specification tables 7, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21).
const RECORD: [usize; 11] = [20, 28, 26, 34, 57, 63, 30, 36, 38, 59, 67];

/// The bytes of the public header block this reads, by where the specification's table 3 puts them.
const HEADER_LEAST: usize = 227;
const HEADER_14: usize = 375;

/// What the first 227 bytes say.
struct Head {
    minor: u8,
    header: usize,
    start: usize,
    vlrs: u32,
    format: u8,
    packed: bool,
    len: usize,
    count: u64,
    scale: [f64; 3],
    offset: [f64; 3],
}

fn le<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    bytes.get(at..at + N)?.try_into().ok()
}
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    le::<2>(bytes, at).map(u16::from_le_bytes)
}
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    le::<4>(bytes, at).map(u32::from_le_bytes)
}
fn u64_at(bytes: &[u8], at: usize) -> Option<u64> {
    le::<8>(bytes, at).map(u64::from_le_bytes)
}
fn f64_at(bytes: &[u8], at: usize) -> Option<f64> {
    le::<8>(bytes, at).map(f64::from_le_bytes)
}

fn head(bytes: &[u8]) -> Result<Head, String> {
    if !bytes.starts_with(b"LASF") {
        return Err("it is not a LAS file (a LAS file begins with the letters LASF)".into());
    }
    if bytes.len() < HEADER_LEAST {
        return Err(format!("it is cut short: its heading is {} bytes and a LAS heading is at least {HEADER_LEAST}", bytes.len()));
    }
    let (major, minor) = (bytes[24], bytes[25]);
    if major != 1 || minor == 0 || minor > 4 {
        let advice = if major == 1 && minor == 0 { "save it again as LAS 1.2 or later" } else { "save it again as LAS 1.2 or 1.4" };
        return Err(format!("it is LAS {major}.{minor}, which this does not read (it reads LAS 1.1 to 1.4): {advice}"));
    }
    let header = usize::from(u16_at(bytes, 94).unwrap_or(0));
    let want = if minor == 4 { HEADER_14 } else { HEADER_LEAST };
    if header < want || header > bytes.len() {
        return Err(format!("it is cut short: its heading says it is {header} bytes and a LAS 1.{minor} heading is at least {want}, in a file of {} bytes", bytes.len()));
    }
    let raw_format = bytes[104];
    let (format, packed) = (raw_format & 0x3f, raw_format & 0x80 != 0);
    let Some(least) = RECORD.get(usize::from(format)) else {
        return Err(format!("its points are of point record format {format}, which LAS does not define (it defines 0 to 10)"));
    };
    let len = usize::from(u16_at(bytes, 105).unwrap_or(0));
    if len < *least {
        return Err(format!("its point records are {len} bytes long and format {format} needs {least}"));
    }
    let legacy = u64::from(u32_at(bytes, 107).unwrap_or(0));
    // LAS 1.4: "this field must always be correctly populated, regardless of legacy mode intent".
    let count = if minor == 4 { u64_at(bytes, 247).filter(|n| *n > 0).unwrap_or(legacy) } else { legacy };
    let (mut scale, mut offset) = ([0.0; 3], [0.0; 3]);
    for k in 0..3 {
        scale[k] = f64_at(bytes, 131 + 8 * k).unwrap_or(0.0);
        offset[k] = f64_at(bytes, 155 + 8 * k).unwrap_or(0.0);
        let axis = ["x", "y", "z"][k];
        if !scale[k].is_finite() || scale[k] == 0.0 {
            return Err(format!("its scale along {axis} is {}: a point's place is its whole number times the scale, and a scale of nothing places every point at the offset", scale[k]));
        }
        if !offset[k].is_finite() {
            return Err(format!("its offset along {axis} is not a number"));
        }
    }
    Ok(Head { minor, header, start: u32_at(bytes, 96).unwrap_or(0) as usize, vlrs: u32_at(bytes, 100).unwrap_or(0), format, packed, len, count, scale, offset })
}

/// One variable length record: who wrote it, which it is, and what it holds.
struct Vlr<'a> {
    user: String,
    id: u16,
    data: &'a [u8],
}

/// The records between the header and the points (54 bytes of heading each, then the payload), and in LAS 1.4 the
/// extended records after the points (60 bytes of heading each, an 8-byte length).
fn records<'a>(bytes: &'a [u8], h: &Head) -> Vec<Vlr<'a>> {
    let mut out = Vec::new();
    let mut at = h.header;
    for _ in 0..h.vlrs {
        let (Some(id), Some(n)) = (u16_at(bytes, at + 18), u16_at(bytes, at + 20)) else { break };
        let Some(data) = bytes.get(at + 54..at + 54 + usize::from(n)) else { break };
        out.push(Vlr { user: user_of(&bytes[at + 2..at + 18]), id, data });
        at += 54 + usize::from(n);
    }
    if h.minor == 4 {
        if let (Some(first), Some(n)) = (u64_at(bytes, 235), u32_at(bytes, 243)) {
            let mut at = first as usize;
            for _ in 0..n {
                let (Some(id), Some(n)) = (u16_at(bytes, at + 18), u64_at(bytes, at + 20)) else { break };
                let Some(data) = (at + 60).checked_add(n as usize).and_then(|end| bytes.get(at + 60..end)) else { break };
                out.push(Vlr { user: user_of(&bytes[at + 2..at + 18]), id, data });
                at += 60 + n as usize;
            }
        }
    }
    out
}

fn user_of(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).trim_end_matches('\0').to_string()
}

/// The sentence of what a record format carries that this does not keep.
fn kept_nothing_of(format: u8, extra: usize) -> String {
    let mut named = vec!["intensity", "return numbers", "classification", "scan angle"];
    if matches!(format, 1 | 3 | 4 | 5) || format >= 6 {
        named.push("GPS time");
    }
    if matches!(format, 2 | 3 | 5 | 7 | 8 | 10) {
        named.push("colour");
    }
    if matches!(format, 8 | 10) {
        named.push("near infrared");
    }
    if matches!(format, 4 | 5 | 9 | 10) {
        named.push("waveform");
    }
    let mut text = format!("each point's {} (only where a point is is kept)", named.join(", "));
    if extra > 0 {
        text = text.replace(" (only", &format!(", and its {extra} extra bytes (only"));
    }
    text
}

/// Read the points of a LAS or a LAZ file.
pub(crate) fn read(bytes: &[u8]) -> Result<Raw, String> {
    let h = head(bytes)?;
    if says(bytes) == FileSays::Degrees {
        return Err("its points are in degrees of latitude and longitude, which are not lengths: save it again in a projected system in feet or meters".into());
    }
    if h.count as usize > crate::scan::MOST_POINTS || h.count > crate::scan::MOST_POINTS as u64 {
        return Err(format!("it has {} points, more than the {} this reads: thin it in the program it was cleaned in", h.count, crate::scan::MOST_POINTS));
    }
    if h.count == 0 {
        return Err("it holds no points (its heading says none)".into());
    }
    let count = h.count as usize;
    if h.start < h.header || h.start > bytes.len() {
        return Err(format!("it is cut short: its points begin at byte {} and the file is {} bytes", h.start, bytes.len()));
    }
    let mut points = Vec::with_capacity(count);
    let mut withheld = 0usize;
    let mut take = |record: &[u8]| {
        let (Some(x), Some(y), Some(z)) = (u32_at(record, 0), u32_at(record, 4), u32_at(record, 8)) else { return };
        // Table 8 (formats 0 to 5): bit 7 of the classification byte; table 16 (6 to 10): bit 2 of the flags byte. Both at byte 15.
        let flag = record.get(15).copied().unwrap_or(0);
        if (h.format < 6 && flag & 0x80 != 0) || (h.format >= 6 && flag & 0x04 != 0) {
            withheld += 1;
            return;
        }
        let place = |raw: u32, k: usize| f64::from(raw as i32) * h.scale[k] + h.offset[k];
        points.push([place(x, 0), place(y, 1), place(z, 2)]);
    };
    if h.packed {
        packed(bytes, &h, count, &mut take)?;
    } else {
        let have = (bytes.len() - h.start) / h.len;
        if (have as u64) < h.count {
            return Err(format!("it is cut short: it says {count} points and holds {have}"));
        }
        for record in bytes[h.start..h.start + count * h.len].chunks_exact(h.len) {
            take(record);
        }
    }
    let mut lost = vec![kept_nothing_of(h.format, h.len - RECORD[usize::from(h.format)])];
    if withheld > 0 {
        lost.push(format!("{} the file marks withheld (its word for deleted), left out", if withheld == 1 { "1 point".to_string() } else { format!("{withheld} points") }));
    }
    Ok(Raw { points, lost })
}

/// The points of a LAZ: LASzip's chunked packing, described by the record "laszip encoded" (id 22204) the file carries.
fn packed(bytes: &[u8], h: &Head, count: usize, take: &mut dyn FnMut(&[u8])) -> Result<(), String> {
    let record = records(bytes, h).into_iter().find(|r| r.user == "laszip encoded" && r.id == 22204);
    let Some(record) = record else {
        return Err("it says its points are packed (LAZ) but holds no record of how: save it again as LAS, or as LAZ from the program that made it".into());
    };
    let vlr = laz::LazVlr::from_buffer(record.data).map_err(|_| "its record of how its points are packed (LAZ) does not read: save it again as LAS".to_string())?;
    let mut cursor = std::io::Cursor::new(bytes);
    cursor.set_position(h.start as u64);
    let mut unpack = laz::LasZipDecompressor::new(cursor, vlr).map_err(|_| "it is cut short or damaged: its packed points (LAZ) cannot be opened".to_string())?;
    // A few thousand points at a time: the whole of a large file is never held twice.
    let each = 8192usize;
    let mut buf = vec![0u8; each * h.len];
    let mut done = 0usize;
    while done < count {
        let n = each.min(count - done);
        unpack.decompress_many(&mut buf[..n * h.len]).map_err(|_| format!("it is cut short or damaged: it says {count} points and its packed points (LAZ) stop after {done}"))?;
        for record in buf[..n * h.len].chunks_exact(h.len) {
            take(record);
        }
        done += n;
    }
    Ok(())
}

/// What the file's own records say of its unit (the GeoTIFF keys or the written form of its coordinate system).
pub(crate) fn says(bytes: &[u8]) -> FileSays {
    let Ok(h) = head(bytes) else { return FileSays::Silent };
    let all = records(bytes, &h);
    let mut across: Option<u16> = None;
    let mut up: Option<u16> = None;
    let mut geographic = false;
    let mut written: Vec<(Units, &'static str)> = Vec::new();
    let mut odd: Option<String> = None;
    for r in &all {
        if r.user != "LASF_Projection" {
            continue;
        }
        match r.id {
            // GeoKeyDirectoryTag: four whole numbers of heading, then four to a key (key, where, count, value).
            34735 => {
                let w: Vec<u16> = r.data.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                for key in w.get(4..).unwrap_or(&[]).chunks_exact(4) {
                    match (key[0], key[1]) {
                        (1024, 0) => geographic |= key[3] == 2,
                        (3076, 0) | (2052, 0) => across = Some(key[3]),
                        (4099, 0) => up = Some(key[3]),
                        _ => {}
                    }
                }
            }
            // The coordinate system as written (WKT).
            2112 => {
                let text = String::from_utf8_lossy(r.data).trim_end_matches('\0').to_string();
                match written_units(&text) {
                    Written::Degrees => geographic = true,
                    Written::Lengths(l) => written.extend(l),
                    Written::Odd(name) => odd = Some(name),
                }
            }
            _ => {}
        }
    }
    if geographic {
        return FileSays::Degrees;
    }
    if let Some(name) = odd {
        return FileSays::Unclear(format!("its coordinate system names a unit this does not know ({name})"));
    }
    let code = |c: u16| -> Result<(Units, &'static str), String> {
        match c {
            9001 => Ok((Units::Metres, "meters")),
            9002 => Ok((Units::Feet, "feet")),
            9003 => Ok((Units::Feet, "US survey feet")),
            other => Err(format!("its coordinate system names a unit this does not know (code {other})")),
        }
    };
    let mut said: Vec<(Units, &'static str, &'static str)> = Vec::new();
    for (c, where_) in [(across, "across"), (up, "up")] {
        if let Some(c) = c {
            match code(c) {
                Ok((u, n)) => said.push((u, n, where_)),
                Err(why) => return FileSays::Unclear(why),
            }
        }
    }
    if said.is_empty() {
        said.extend(written.iter().map(|(u, n)| (*u, *n, "across")));
    }
    let Some(first) = said.first().copied() else { return FileSays::Silent };
    if said.iter().all(|s| s.1 == first.1) {
        return FileSays::Unit(first.0, first.1.to_string());
    }
    let parts: Vec<String> = said.iter().map(|s| format!("{} {}", s.1, s.2)).collect();
    FileSays::Unclear(format!("it says {}", parts.join(" and ")))
}

enum Written {
    Degrees,
    Lengths(Vec<(Units, &'static str)>),
    Odd(String),
}

/// The length units a written coordinate system (WKT) names, in order, with the angle units left out; a system that is
/// only angles (it begins GEOGCS, GEOGCRS or GEODCRS) is degrees.
fn written_units(text: &str) -> Written {
    let top = text.trim_start();
    if top.starts_with("GEOGCS") || top.starts_with("GEOGCRS") || top.starts_with("GEODCRS") {
        return Written::Degrees;
    }
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("UNIT[").or_else(|| rest.find("LENGTHUNIT[")) {
        let after = &rest[at..];
        let open = after.find('[').unwrap_or(0) + 1;
        let name = after[open..].split('"').nth(1).unwrap_or("").to_string();
        rest = &after[open..];
        let lower = name.to_lowercase();
        if lower.contains("degree") || lower.contains("radian") || lower.contains("grad") {
            continue;
        }
        if lower.contains("us survey") || lower.contains("survey foot") || lower.contains("foot_us") {
            out.push((Units::Feet, "US survey feet"));
        } else if lower.contains("foot") || lower.contains("feet") {
            out.push((Units::Feet, "feet"));
        } else if lower.contains("metre") || lower.contains("meter") {
            out.push((Units::Metres, "meters"));
        } else {
            return Written::Odd(name);
        }
    }
    Written::Lengths(out)
}
