//! A scanner's own files taken in: LAS, its packed kind (LAZ) and E57 (DECISIONS.md call 230).
//!
//! Every file here is made up by the test, a room as points, written to the layout the
//! formats' published texts give (the LAS specification's tables; E57 and LAZ through the two
//! crates that read them). No real scan is used.

use std::io::{Cursor, Seek, SeekFrom, Write};

use plumbline_plans::scan::{bring_in, file_notes, file_says, kind_of, read, read_ply, units_to_use, FileSays, Kind, Said, Units, Up};

const FT: f64 = 1.0 / 0.3048;

/// The smallest record each LAS point format takes, from the specification's tables 7, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21.
const RECORD: [usize; 11] = [20, 28, 26, 34, 57, 63, 30, 36, 38, 59, 67];

struct Vlr {
    user: &'static str,
    id: u16,
    data: Vec<u8>,
}

/// A LAS file: the public header block as the specification's table 3 lays it out, the
/// variable length records, then the points. `extra` bytes follow each record's own.
#[allow(clippy::too_many_arguments)]
fn las(minor: u8, format: u8, extra: usize, scale: [f64; 3], offset: [f64; 3], pts: &[[i32; 3]], vlrs: &[Vlr], withheld: &[usize]) -> Vec<u8> {
    let header: usize = match minor {
        0..=2 => 227,
        3 => 235,
        _ => 375,
    };
    let len = RECORD[usize::from(format & 0x3f).min(10)] + extra;
    let vlr_bytes: usize = vlrs.iter().map(|v| 54 + v.data.len()).sum();
    let start = header + vlr_bytes;
    let mut b = vec![0u8; header];
    b[0..4].copy_from_slice(b"LASF");
    b[24] = 1;
    b[25] = minor;
    b[94..96].copy_from_slice(&(header as u16).to_le_bytes());
    b[96..100].copy_from_slice(&(start as u32).to_le_bytes());
    b[100..104].copy_from_slice(&(vlrs.len() as u32).to_le_bytes());
    b[104] = format;
    b[105..107].copy_from_slice(&(len as u16).to_le_bytes());
    // Legacy count (zero for a 1.4 file with the newer formats, as the specification says), then the 64-bit count.
    let legacy = if minor >= 4 && format >= 6 { 0 } else { pts.len() as u32 };
    b[107..111].copy_from_slice(&legacy.to_le_bytes());
    for k in 0..3 {
        b[131 + 8 * k..139 + 8 * k].copy_from_slice(&scale[k].to_le_bytes());
        b[155 + 8 * k..163 + 8 * k].copy_from_slice(&offset[k].to_le_bytes());
    }
    if minor >= 4 {
        b[247..255].copy_from_slice(&(pts.len() as u64).to_le_bytes());
    }
    for v in vlrs {
        let mut h = vec![0u8; 54];
        h[2..2 + v.user.len()].copy_from_slice(v.user.as_bytes());
        h[18..20].copy_from_slice(&v.id.to_le_bytes());
        h[20..22].copy_from_slice(&(v.data.len() as u16).to_le_bytes());
        b.extend_from_slice(&h);
        b.extend_from_slice(&v.data);
    }
    for (i, p) in pts.iter().enumerate() {
        let mut r = vec![0u8; len];
        for k in 0..3 {
            r[4 * k..4 * k + 4].copy_from_slice(&p[k].to_le_bytes());
        }
        if withheld.contains(&i) {
            // Formats 0 to 5: bit 7 of the classification byte. Formats 6 to 10: bit 2 of the flags byte (table 16).
            r[15] = if format < 6 { 0x80 } else { 0x04 };
        }
        b.extend_from_slice(&r);
    }
    b
}

fn plain(pts: &[[i32; 3]]) -> Vec<u8> {
    las(2, 0, 0, [0.01; 3], [0.0; 3], pts, &[], &[])
}

/// A GeoTIFF key directory record (LASF_Projection, 34735) holding the keys given as (key, value).
fn geokeys(keys: &[(u16, u16)]) -> Vlr {
    let mut v = vec![1u16, 1, 0, keys.len() as u16];
    for (k, val) in keys {
        v.extend_from_slice(&[*k, 0, 1, *val]);
    }
    Vlr { user: "LASF_Projection", id: 34735, data: v.iter().flat_map(|x| x.to_le_bytes()).collect() }
}

fn wkt(text: &str) -> Vlr {
    let mut data = text.as_bytes().to_vec();
    data.push(0);
    Vlr { user: "LASF_Projection", id: 2112, data }
}

/// The made-up room of the demonstration: 12 ft by 10 ft by 8 ft, in meters kept at a tenth of a millimetre
/// (so each foot-and-a-half is a whole number), a point every quarter foot on the floor, the ceiling and the
/// four walls. 9,474 points.
fn room_ints() -> Vec<[i32; 3]> {
    let q = 762; // 0.25 ft in tenths of a millimetre
    let (w, d, h) = (48, 40, 32); // 12 ft, 10 ft, 8 ft in quarter feet
    let mut p = Vec::new();
    for i in 0..=w {
        for j in 0..=d {
            p.push([i * q, j * q, 0]);
            p.push([i * q, j * q, h * q]);
        }
    }
    for k in 1..h {
        for i in 0..=w {
            p.push([i * q, 0, k * q]);
            p.push([i * q, d * q, k * q]);
        }
        for j in 1..d {
            p.push([0, j * q, k * q]);
            p.push([w * q, j * q, k * q]);
        }
    }
    p
}

fn room_las() -> Vec<u8> {
    las(2, 0, 0, [0.0001; 3], [0.0; 3], &room_ints(), &[], &[])
}

fn room_ply() -> Vec<u8> {
    let pts = room_ints();
    let mut s = format!("ply\nformat ascii 1.0\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nend_header\n", pts.len());
    for p in &pts {
        s.push_str(&format!("{:.4} {:.4} {:.4}\n", f64::from(p[0]) / 1e4, f64::from(p[1]) / 1e4, f64::from(p[2]) / 1e4));
    }
    s.into_bytes()
}

fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

/// The same points, to a millionth of a foot (a scale is not a whole number in binary).
fn same(a: &[[f64; 3]], b: &[[f64; 3]]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (0..3).all(|k| near(p[k], q[k], 1e-6)))
}

#[test]
fn a_las_point_is_a_whole_number_times_a_scale_plus_an_offset() {
    // Specification, "Xcoordinate = (Xrecord * Xscale) + Xoffset": 100 x 0.01 + 500 = 501, 200 x 0.01 + 100 = 102, 300 x 0.01 + 10 = 13.
    let bytes = las(2, 0, 0, [0.01; 3], [500.0, 100.0, 10.0], &[[100, 200, 300], [-100, 0, 1]], &[], &[]);
    let feet = read(&bytes, Units::Feet, Up::Z).unwrap();
    assert!(same(&feet.points, &[[501.0, 102.0, 13.0], [499.0, 100.0, 10.01]]), "{:?}", feet.points);
    // The same file said to be in meters: each length over 0.3048 to be feet.
    let m = read(&bytes, Units::Metres, Up::Z).unwrap();
    assert!(near(m.points[0][0], 501.0 * FT, 1e-9) && near(m.points[0][2], 13.0 * FT, 1e-9), "{:?}", m.points);
    // Up as y says: the file's y is height, and north is the file's minus z.
    let y = read(&bytes, Units::Feet, Up::Y).unwrap();
    assert!(same(&y.points[..1], &[[501.0, -13.0, 102.0]]), "{:?}", y.points);
}

#[test]
fn the_points_are_found_by_the_offset_in_the_header_past_the_records_before_them() {
    let note = Vlr { user: "made up", id: 7, data: vec![9u8; 100] };
    let bytes = las(2, 1, 0, [0.001; 3], [0.0; 3], &[[1000, 2000, 3000]], &[note], &[]);
    assert!(same(&read(&bytes, Units::Feet, Up::Z).unwrap().points, &[[1.0, 2.0, 3.0]]));
}

#[test]
fn every_point_record_format_the_specification_defines_is_read_and_bytes_after_the_record_are_skipped() {
    for format in 0u8..=10 {
        let minor = if format >= 6 { 4 } else { 2 };
        for extra in [0usize, 5] {
            let bytes = las(minor, format, extra, [0.001; 3], [0.0; 3], &[[1000, 2000, 3000], [4000, 5000, 6000]], &[], &[]);
            let scan = read(&bytes, Units::Feet, Up::Z).unwrap_or_else(|e| panic!("format {format} extra {extra}: {e}"));
            assert!(same(&scan.points, &[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]), "format {format} extra {extra}");
        }
    }
}

#[test]
fn a_las_14_file_counts_its_points_in_64_bits() {
    // Format 7 in a 1.4 file: the legacy count is zero (as the specification says) and the real count is at byte 247.
    let bytes = las(4, 7, 0, [0.001; 3], [0.0; 3], &[[1000, 2000, 3000], [4000, 5000, 6000], [7000, 8000, 9000]], &[], &[]);
    assert_eq!(&bytes[107..111], &[0, 0, 0, 0]);
    assert_eq!(read(&bytes, Units::Feet, Up::Z).unwrap().points.len(), 3);
}

#[test]
fn what_a_las_keeps_of_a_point_and_this_does_not_is_said() {
    let said = |format: u8| {
        let minor = if format >= 6 { 4 } else { 2 };
        let scan = read(&las(minor, format, 0, [0.01; 3], [0.0; 3], &[[1, 2, 3]], &[], &[]), Units::Feet, Up::Z).unwrap();
        scan.lost.join(" | ")
    };
    let f0 = said(0);
    assert!(f0.contains("intensity") && f0.contains("only where a point is is kept"), "{f0}");
    assert!(!f0.contains("colour") && !f0.contains("time"), "{f0}");
    assert!(said(1).contains("time"), "{}", said(1));
    assert!(said(2).contains("colour"), "{}", said(2));
    assert!(said(10).contains("waveform"), "{}", said(10));
    // Bytes after the record's own are named too.
    let extra = read(&las(2, 0, 4, [0.01; 3], [0.0; 3], &[[1, 2, 3]], &[], &[]), Units::Feet, Up::Z).unwrap().lost.join(" | ");
    assert!(extra.contains("4 extra bytes"), "{extra}");
}

#[test]
fn points_the_file_marks_withheld_are_left_out_and_counted() {
    // Table 8 (formats 0 to 5, bit 7 of the classification) and table 16 (formats 6 to 10, the flags): "should not be included in processing".
    for format in [0u8, 3, 6, 8] {
        let minor = if format >= 6 { 4 } else { 2 };
        let bytes = las(minor, format, 0, [0.01; 3], [0.0; 3], &[[100, 0, 0], [200, 0, 0], [300, 0, 0]], &[], &[1]);
        let scan = read(&bytes, Units::Feet, Up::Z).unwrap();
        assert!(same(&scan.points, &[[1.0, 0.0, 0.0], [3.0, 0.0, 0.0]]), "format {format}");
        assert!(scan.lost.iter().any(|l| l.contains("1 point the file marks withheld")), "{:?}", scan.lost);
    }
}

#[test]
fn a_kind_of_las_it_does_not_read_is_refused_by_its_name() {
    // Version 2.0 and 1.0, and a point record format the specification does not define (11).
    let mut v2 = plain(&[[1, 2, 3]]);
    v2[24] = 2;
    v2[25] = 0;
    assert!(read(&v2, Units::Feet, Up::Z).unwrap_err().contains("it is LAS 2.0, which this does not read"), "{:?}", read(&v2, Units::Feet, Up::Z));
    let mut v10 = plain(&[[1, 2, 3]]);
    v10[25] = 0;
    let why = read(&v10, Units::Feet, Up::Z).unwrap_err();
    assert!(why.contains("it is LAS 1.0, which this does not read") && why.contains("LAS 1.2 or later"), "{why}");
    let mut f11 = plain(&[[1, 2, 3]]);
    f11[104] = 11;
    let why = read(&f11, Units::Feet, Up::Z).unwrap_err();
    assert!(why.contains("point record format 11") && why.contains("0 to 10"), "{why}");
    // A record shorter than its format needs.
    let mut short = las(2, 3, 0, [0.01; 3], [0.0; 3], &[[1, 2, 3]], &[], &[]);
    short[105..107].copy_from_slice(&20u16.to_le_bytes());
    let why = read(&short, Units::Feet, Up::Z).unwrap_err();
    assert!(why.contains("its point records are 20 bytes long and format 3 needs 34"), "{why}");
    // A scale of nothing.
    let zero = las(2, 0, 0, [0.0, 0.01, 0.01], [0.0; 3], &[[1, 2, 3]], &[], &[]);
    assert!(read(&zero, Units::Feet, Up::Z).unwrap_err().contains("scale along x is 0"), "{:?}", read(&zero, Units::Feet, Up::Z));
}

#[test]
fn a_las_that_says_it_holds_no_points_is_refused() {
    assert!(read(&plain(&[]), Units::Feet, Up::Z).unwrap_err().contains("it holds no points"));
}

#[test]
fn a_las_file_cut_short_is_refused_saying_where() {
    let pts: Vec<[i32; 3]> = (0..10).map(|i| [i, 0, 0]).collect();
    let mut bytes = plain(&pts);
    bytes.truncate(bytes.len() - 3 * 20 - 5);
    // 10 points are said; 6 whole records and part of a 7th remain.
    let why = read(&bytes, Units::Feet, Up::Z).unwrap_err();
    assert!(why.contains("it is cut short: it says 10 points and holds 6"), "{why}");
    // Cut inside the heading.
    let why = read(&plain(&pts)[..150], Units::Feet, Up::Z).unwrap_err();
    assert!(why.contains("it is cut short") && why.contains("heading"), "{why}");
}

#[test]
fn a_file_is_told_by_how_it_begins() {
    assert_eq!(kind_of(&room_ply()), Some(Kind::Ply));
    assert_eq!(kind_of(&room_las()), Some(Kind::Las));
    let mut packed = room_las();
    packed[104] |= 0x80;
    assert_eq!(kind_of(&packed), Some(Kind::Laz));
    assert_eq!(kind_of(b"ASTM-E57\x01\x00\x00\x00"), Some(Kind::E57));
    assert_eq!(kind_of(b"%PDF-1.7"), None);
    assert_eq!(Kind::Laz.said(), "LAZ");
}

#[test]
fn a_ply_is_read_as_it_was_whichever_way_it_is_asked_for() {
    let ply = room_ply();
    let (a, b) = (read(&ply, Units::Metres, Up::Z).unwrap(), read_ply(&ply, Units::Metres, Up::Z).unwrap());
    assert_eq!(a.points, b.points);
    assert_eq!(a.lost, b.lost);
    // What is no scan says it is none of the four, and still says what a PLY begins with.
    let why = read(b"solid not a scan", Units::Metres, Up::Z).unwrap_err();
    assert!(why.contains("it is not a PLY file") && why.contains("LAS") && why.contains("E57"), "{why}");
}

#[test]
fn the_room_as_a_ply_and_as_a_las_is_the_same_room() {
    let (p, l) = (read(&room_ply(), Units::Metres, Up::Z).unwrap(), read(&room_las(), Units::Metres, Up::Z).unwrap());
    assert_eq!((p.points.len(), l.points.len()), (9474, 9474));
    for (a, b) in p.points.iter().zip(&l.points) {
        assert!((0..3).all(|k| near(a[k], b[k], 1e-9)), "{a:?} {b:?}");
    }
    let (lo, hi) = l.bounds().unwrap();
    assert!(near(hi[0] - lo[0], 12.0, 0.01) && near(hi[1] - lo[1], 10.0, 0.01) && near(hi[2] - lo[2], 8.0, 0.01), "{lo:?} {hi:?}");
    let (floor, ceiling) = l.floor_and_ceiling();
    assert!(near(floor.unwrap(), 0.0, 0.01) && near(ceiling.unwrap(), 8.0, 0.01), "{floor:?} {ceiling:?}");
}

#[test]
fn what_the_file_says_of_its_unit_is_read_from_its_records() {
    let one = [[1, 2, 3]];
    let with = |v: Vlr| las(2, 0, 0, [0.01; 3], [0.0; 3], &one, &[v], &[]);
    // Silent: no record of its own system, or a record that does not name a unit.
    assert!(matches!(file_says(&plain(&one)), FileSays::Silent));
    // The GeoTIFF keys: 1024 model type (1 projected), 3076 projected linear units, 4099 vertical units; 9001 metre, 9002 foot, 9003 US survey foot.
    match file_says(&with(geokeys(&[(1024, 1), (3076, 9002)]))) {
        FileSays::Unit(Units::Feet, name) => assert_eq!(name, "feet"),
        other => panic!("{other:?}"),
    }
    match file_says(&with(geokeys(&[(1024, 1), (3076, 9003), (4099, 9003)]))) {
        FileSays::Unit(Units::Feet, name) => assert_eq!(name, "US survey feet"),
        other => panic!("{other:?}"),
    }
    match file_says(&with(geokeys(&[(1024, 1), (3076, 9001)]))) {
        FileSays::Unit(Units::Metres, name) => assert_eq!(name, "meters"),
        other => panic!("{other:?}"),
    }
    // Feet across and meters up: it says two things, and no one is taken.
    match file_says(&with(geokeys(&[(1024, 1), (3076, 9002), (4099, 9001)]))) {
        FileSays::Unclear(why) => assert!(why.contains("feet across") && why.contains("meters up"), "{why}"),
        other => panic!("{other:?}"),
    }
    // Degrees of latitude and longitude are no length.
    assert!(matches!(file_says(&with(geokeys(&[(1024, 2)]))), FileSays::Degrees));
    let why = read(&with(geokeys(&[(1024, 2)])), Units::Feet, Up::Z).unwrap_err();
    assert!(why.contains("degrees of latitude and longitude"), "{why}");
    // The written kind: the last length unit named, the angle units left out.
    let proj = r#"PROJCS["a made-up grid",GEOGCS["a made-up datum",DATUM["d",SPHEROID["s",6378137,298.25]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],UNIT["US survey foot",0.3048006096012192]]"#;
    match file_says(&with(wkt(proj))) {
        FileSays::Unit(Units::Feet, name) => assert_eq!(name, "US survey feet"),
        other => panic!("{other:?}"),
    }
    let metres = r#"PROJCS["g",GEOGCS["d",UNIT["degree",0.0174532925199433]],UNIT["metre",1]]"#;
    assert!(matches!(file_says(&with(wkt(metres))), FileSays::Unit(Units::Metres, _)));
    assert!(matches!(file_says(&with(wkt(r#"GEOGCS["d",UNIT["degree",0.0174532925199433]]"#))), FileSays::Degrees));
}

#[test]
fn a_unit_is_what_the_person_said_else_what_the_file_says_else_meters_and_never_a_guess_between_two() {
    let feet = las(2, 0, 0, [0.01; 3], [0.0; 3], &[[1, 2, 3]], &[geokeys(&[(1024, 1), (3076, 9002)])], &[]);
    let two = las(2, 0, 0, [0.01; 3], [0.0; 3], &[[1, 2, 3]], &[geokeys(&[(1024, 1), (3076, 9002), (4099, 9001)])], &[]);
    assert_eq!(units_to_use(&feet, None), Ok(Units::Feet));
    assert_eq!(units_to_use(&feet, Some(Units::Metres)), Ok(Units::Metres));
    assert_eq!(units_to_use(&plain(&[[1, 2, 3]]), None), Ok(Units::Metres));
    let why = units_to_use(&two, None).unwrap_err();
    assert!(why.contains("feet across") && why.contains("--units m or --units ft"), "{why}");
    assert_eq!(units_to_use(&two, Some(Units::Feet)), Ok(Units::Feet));
    // A PLY says nothing of its unit: meters unless told.
    assert_eq!(units_to_use(&room_ply(), None), Ok(Units::Metres));
}

fn said_in(units: Units, up: Option<Up>) -> Said {
    Said { units, up, cut_at: 4.0 }
}

fn laid() -> impl FnMut(&plumbline_plans::scan::Sheet) -> Result<String, String> {
    |s| Ok(format!("{}|{}", s.called, s.checksum))
}

#[test]
fn a_las_goes_down_the_same_road_as_a_ply_and_makes_the_same_sheet() {
    let (p, l) = (
        bring_in(&room_ply(), "room.ply", "room", &said_in(Units::Metres, Some(Up::Z)), &mut laid()).unwrap(),
        bring_in(&room_las(), "room.las", "room", &said_in(Units::Metres, Some(Up::Z)), &mut laid()).unwrap(),
    );
    assert_eq!((p.points, l.points), (9474, 9474));
    // The sheet at 4 ft: the four walls' points at that height, (12 + 4) x 24 by (10 + 4) x 24 dots... 384 by 336.
    assert_eq!((l.cut.size, l.cut.points, l.cut.strays), ([384, 336], 176, 0));
    assert!(near(l.cut.extent[0], 12.0, 0.01) && near(l.cut.extent[1], 10.0, 0.01), "{:?}", l.cut.extent);
    assert!(p.cut.png == l.cut.png, "the sheet from the LAS is not the sheet from the PLY");
    assert!(near(l.floor.unwrap(), 0.0, 0.01) && near(l.ceiling.unwrap(), 8.0, 0.01));
    // A PLY says nothing of what a file says of itself; a LAS does.
    assert!(p.file_said.is_empty());
    assert!(l.file_said.iter().any(|s| s.contains("does not say what its unit is")), "{:?}", l.file_said);
    assert_eq!(l.taken, "taken as meters with z up");
}

#[test]
fn bringing_a_las_in_says_what_the_file_says_beside_what_the_person_said() {
    // The same room kept in feet (a tenth of a millimetre over 0.3048 to the whole number), the file saying feet, set half a foot
    // off the zero so that no wall lies on a whole foot, where one point in a million is cut into either square foot by rounding.
    let feet = las(2, 0, 0, [0.0001 / 0.3048; 3], [0.5, 0.5, 0.0], &room_ints(), &[geokeys(&[(1024, 1), (3076, 9002)])], &[]);
    // Said the same: the file's word is said, and the room is the room.
    let a = bring_in(&feet, "f.las", "f", &said_in(Units::Feet, Some(Up::Z)), &mut laid()).unwrap();
    assert!(a.file_said.iter().any(|s| s == "the file says its unit is feet"), "{:?}", a.file_said);
    assert_eq!((a.cut.size, a.cut.points), ([384, 336], 176));
    // Said otherwise: taken as said, and both said.
    let b = file_notes(&feet, Units::Metres, Some(Up::Z));
    assert!(b.iter().any(|s| s == "the file says its unit is feet and you said meters: taken as meters, as you said"), "{b:?}");
    // Up: a LAS keeps height in z; said y, that is said.
    let c = file_notes(&room_las(), Units::Metres, Some(Up::Y));
    assert!(c.iter().any(|s| s.contains("a LAS file keeps height in z") && s.contains("you said y")), "{c:?}");
    assert!(!file_notes(&room_las(), Units::Metres, Some(Up::Z)).iter().any(|s| s.contains("keeps height")));
    // Up not said: worked out from the shape of the room as for a PLY, and it still reads the room's 8 ft.
    let d = bring_in(&room_las(), "r.las", "r", &said_in(Units::Metres, None), &mut laid()).unwrap();
    assert!(near(d.ceiling.unwrap() - d.floor.unwrap(), 8.0, 0.01));
}

// ---- LAZ: LAS packed by LASzip, read by the laz crate ----

/// A packed copy of a LAS file made with the laz crate's own compressor: the header with the packed mark on the
/// format, the "laszip encoded" record (22204) and the chunked points.
fn packed(las_bytes: &[u8], with_record: bool) -> Vec<u8> {
    let header = usize::from(u16::from_le_bytes([las_bytes[94], las_bytes[95]]));
    let start = u32::from_le_bytes(las_bytes[96..100].try_into().unwrap()) as usize;
    let format = las_bytes[104];
    let len = usize::from(u16::from_le_bytes([las_bytes[105], las_bytes[106]]));
    let extra = len - RECORD[usize::from(format)];
    let vlr = laz::LazVlrBuilder::new(laz::LazItemRecordBuilder::default_for_point_format_id(format, extra as u16).unwrap()).build();
    let mut data = Vec::new();
    vlr.write_to(&mut data).unwrap();
    let mut head = las_bytes[..header].to_vec();
    head[104] = format | 0x80;
    let mut file = head;
    if with_record {
        let mut h = vec![0u8; 54];
        h[2..16].copy_from_slice(b"laszip encoded");
        h[18..20].copy_from_slice(&22204u16.to_le_bytes());
        h[20..22].copy_from_slice(&(data.len() as u16).to_le_bytes());
        file.extend_from_slice(&h);
        file.extend_from_slice(&data);
        file[100..104].copy_from_slice(&1u32.to_le_bytes());
    }
    let at = file.len() as u32;
    file[96..100].copy_from_slice(&at.to_le_bytes());
    let mut cursor = Cursor::new(file);
    cursor.seek(SeekFrom::End(0)).unwrap();
    let mut c = laz::LasZipCompressor::new(&mut cursor, vlr).unwrap();
    c.compress_many(&las_bytes[start..]).unwrap();
    c.done().unwrap();
    drop(c);
    cursor.into_inner()
}

#[test]
fn a_laz_is_read_to_the_same_points_as_the_las_it_was_packed_from() {
    let las_bytes = room_las();
    let laz_bytes = packed(&las_bytes, true);
    assert_eq!(kind_of(&laz_bytes), Some(Kind::Laz));
    assert!(laz_bytes.len() < las_bytes.len() / 2, "{} of {}", laz_bytes.len(), las_bytes.len());
    let (a, b) = (read(&las_bytes, Units::Metres, Up::Z).unwrap(), read(&laz_bytes, Units::Metres, Up::Z).unwrap());
    assert!(same(&a.points, &b.points));
    // A 1.4 file of the newer formats (layered chunks) too.
    let new = las(4, 7, 0, [0.001; 3], [10.0, 20.0, 30.0], &[[1000, 2000, 3000], [4000, 5000, 6000]], &[], &[]);
    assert!(same(&read(&packed(&new, true), Units::Feet, Up::Z).unwrap().points, &[[11.0, 22.0, 33.0], [14.0, 25.0, 36.0]]));
}

#[test]
fn a_laz_that_is_damaged_or_without_its_packing_record_is_refused_in_words() {
    let las_bytes = room_las();
    let why = read(&packed(&las_bytes, false), Units::Metres, Up::Z).unwrap_err();
    assert!(why.contains("it says its points are packed (LAZ) but holds no record of how"), "{why}");
    let whole = packed(&las_bytes, true);
    let why = read(&whole[..whole.len() / 2], Units::Metres, Up::Z).unwrap_err();
    assert!(why.contains("it is cut short") && why.contains("LAZ"), "{why}");
}

// ---- E57: ASTM E2807, read by the e57 crate ----

fn e57_of(scans: &[(&[[f64; 3]], Option<e57::Transform>)]) -> Vec<u8> {
    use e57::{E57Writer, Record, RecordDataType, RecordName, RecordValue};
    let mut buf = Cursor::new(Vec::new());
    {
        let mut w = E57Writer::new(&mut buf, "made-up-file").unwrap();
        for (n, (pts, pose)) in scans.iter().enumerate() {
            let proto = vec![
                Record { name: RecordName::CartesianX, data_type: RecordDataType::Double { min: None, max: None } },
                Record { name: RecordName::CartesianY, data_type: RecordDataType::Double { min: None, max: None } },
                Record { name: RecordName::CartesianZ, data_type: RecordDataType::Double { min: None, max: None } },
            ];
            let mut pc = w.add_pointcloud(&format!("made-up-scan-{n}"), proto).unwrap();
            pc.set_transform(pose.clone());
            for p in *pts {
                pc.add_point(vec![RecordValue::Double(p[0]), RecordValue::Double(p[1]), RecordValue::Double(p[2])]).unwrap();
            }
            pc.finalize().unwrap();
        }
        w.finalize().unwrap();
    }
    buf.flush().unwrap();
    buf.into_inner()
}

fn room_metres() -> Vec<[f64; 3]> {
    room_ints().iter().map(|p| [f64::from(p[0]) / 1e4, f64::from(p[1]) / 1e4, f64::from(p[2]) / 1e4]).collect()
}

#[test]
fn an_e57_is_read_to_the_same_room_in_meters() {
    let bytes = e57_of(&[(&room_metres(), None)]);
    assert_eq!(kind_of(&bytes), Some(Kind::E57));
    let e = read(&bytes, Units::Metres, Up::Z).unwrap();
    let l = read(&room_las(), Units::Metres, Up::Z).unwrap();
    assert_eq!(e.points.len(), 9474);
    for (a, b) in e.points.iter().zip(&l.points) {
        assert!((0..3).all(|k| near(a[k], b[k], 1e-9)), "{a:?} {b:?}");
    }
    // E57 keeps lengths in meters, whatever is asked: the file says so, and a person who says feet is told.
    match file_says(&bytes) {
        FileSays::Unit(Units::Metres, name) => assert_eq!(name, "meters"),
        other => panic!("{other:?}"),
    }
    assert_eq!(units_to_use(&bytes, None), Ok(Units::Metres));
}

#[test]
fn the_scans_of_an_e57_are_each_placed_by_their_own_pose() {
    // The second scan was taken from a station 10 m east, turned a half turn about up (w 0, z 1): its (1, 0, 0) is the room's (9, 0, 0).
    let turned = e57::Transform { rotation: e57::Quaternion { w: 0.0, x: 0.0, y: 0.0, z: 1.0 }, translation: e57::Translation { x: 10.0, y: 0.0, z: 0.0 } };
    let bytes = e57_of(&[(&[[1.0, 2.0, 0.0]], None), (&[[1.0, 0.0, 0.0]], Some(turned))]);
    let scan = read(&bytes, Units::Metres, Up::Z).unwrap();
    assert_eq!(scan.points.len(), 2);
    assert!(near(scan.points[1][0], 9.0 * FT, 1e-6) && near(scan.points[1][1], 0.0, 1e-6), "{:?}", scan.points);
    assert!(scan.lost.iter().any(|l| l.contains("2 scans") && l.contains("one cloud")), "{:?}", scan.lost);
}

#[test]
fn an_e57_cut_short_is_refused_saying_so() {
    let bytes = e57_of(&[(&room_metres(), None)]);
    let why = read(&bytes[..bytes.len() / 2], Units::Metres, Up::Z).unwrap_err();
    assert!(why.contains("it is cut short or damaged") && why.contains("E57"), "{why}");
    let why = read(b"ASTM-E57\x01\x00", Units::Metres, Up::Z).unwrap_err();
    assert!(why.contains("it is cut short"), "{why}");
}

#[test]
fn an_e57_goes_down_the_same_road_and_makes_the_same_sheet() {
    let bytes = e57_of(&[(&room_metres(), None)]);
    let e = bring_in(&bytes, "room.e57", "room", &said_in(Units::Metres, Some(Up::Z)), &mut laid()).unwrap();
    let p = bring_in(&room_ply(), "room.ply", "room", &said_in(Units::Metres, Some(Up::Z)), &mut laid()).unwrap();
    assert_eq!((e.points, e.cut.size, e.cut.points), (9474, [384, 336], 176));
    assert!(e.cut.png == p.cut.png, "the sheet from the E57 is not the sheet from the PLY");
    // Said feet of a file whose lengths are always meters: both are said.
    let f = file_notes(&bytes, Units::Feet, Some(Up::Z));
    assert!(f.iter().any(|s| s == "the file says its unit is meters and you said feet: taken as feet, as you said"), "{f:?}");
    assert!(f.iter().any(|s| s == "the file does not say which way is up"), "{f:?}");
}

#[test]
fn a_scan_handed_over_as_a_drawing_is_told_where_it_goes() {
    for bytes in [room_ply(), room_las(), e57_of(&[(&room_metres(), None)])] {
        let why = plumbline_plans::read(&bytes, 1).unwrap_err();
        assert!(why.contains("scan, a cloud of points, not a drawing") && why.contains("import scan"), "{why}");
    }
    // What is no scan is told what it is not, as before.
    assert_eq!(plumbline_plans::read(b"not a plan at all", 1).unwrap_err(), "it is not a PDF, a PNG, a JPEG, a TIFF or a CAD drawing (DXF)");
}

#[test]
fn a_scan_in_a_map_grid_is_taken_to_the_zero_and_how_far_it_was_is_handed_back() {
    // The room 512,000 m east and 4,200,000 m north of the zero (a UTM-sized place: 1.68 million feet east, more than a job's places go).
    let far = las(4, 7, 0, [0.0001; 3], [512000.0, 4200000.0, 300.0], &room_ints(), &[wkt(r#"PROJCS["g",GEOGCS["d",UNIT["degree",0.0174532925199433]],UNIT["metre",1]]"#)], &[]);
    let b = bring_in(&far, "survey.las", "survey", &said_in(Units::Metres, Some(Up::Z)), &mut laid()).unwrap();
    let moved = b.moved.expect("its place is said");
    assert!(near(moved[0], 512000.0 * FT, 0.01) && near(moved[1], 4200000.0 * FT, 0.01), "{moved:?}");
    // Said where the sheet's source is read later: the unit is the file's own, the thousands set off.
    assert_eq!(b.taken, "taken as meters with z up; its southwest corner taken to zero: it lay 512,000 m east and 4,200,000 m north in the file's own grid");
    // The sheet is the sheet of the room at the zero, and lies near the zero.
    assert_eq!((b.cut.size, b.cut.points), ([384, 336], 176));
    assert!(b.cut.corner[0].abs() < 3.0 && b.cut.corner[1].abs() < 3.0, "{:?}", b.cut.corner);
    assert!(b.file_said.iter().any(|s| s == "the file says its unit is meters"), "{:?}", b.file_said);
    // A room near the zero is not moved, and a PLY is never moved (it is left where its file put it).
    assert!(bring_in(&room_las(), "r.las", "r", &said_in(Units::Metres, Some(Up::Z)), &mut laid()).unwrap().moved.is_none());
    let mut ply = String::from("ply\nformat ascii 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nend_header\n");
    for p in [[5000.0, 5000.0, 0.0], [5004.0, 5000.0, 0.0], [5000.0, 5004.0, 2.4], [5004.0, 5004.0, 2.4]] {
        ply.push_str(&format!("{} {} {}\n", p[0], p[1], p[2]));
    }
    let _ = bring_in(ply.as_bytes(), "far.ply", "far", &said_in(Units::Metres, Some(Up::Z)), &mut laid()).map(|b| assert!(b.moved.is_none()));
}
