//! A CAD drawing (DXF) read and written (call 221). Every figure is made up and worked by hand.

use plumbline_plans::dxf::{self, closed_format, is_dxf, read, Writer};
use plumbline_plans::{read as read_page, size_of, sort_of, strings, Sort};

/// A DXF in the ASCII form from pairs.
fn ascii(pairs: &[(i32, &str)]) -> Vec<u8> {
    let mut s = String::new();
    for (c, v) in pairs {
        s.push_str(&format!("{c:>3}\n{v}\n"));
    }
    s.into_bytes()
}

/// The same pairs in the binary form (R14 and later: two-byte codes).
fn binary(pairs: &[(i32, &str)]) -> Vec<u8> {
    let mut out = b"AutoCAD Binary DXF\r\n\x1a\0".to_vec();
    for (c, v) in pairs {
        out.extend((*c as u16).to_le_bytes());
        match *c {
            10..=59 | 110..=149 | 210..=239 => out.extend(v.parse::<f64>().unwrap().to_le_bytes()),
            60..=79 | 170..=179 | 270..=289 => out.extend(v.parse::<i16>().unwrap().to_le_bytes()),
            90..=99 => out.extend(v.parse::<i32>().unwrap().to_le_bytes()),
            _ => {
                out.extend(v.as_bytes());
                out.push(0);
            }
        }
    }
    out
}

fn with(header: &[(i32, &str)], layers: &[(&str, i32)], blocks: &[(i32, &str)], entities: &[(i32, &str)]) -> Vec<(i32, String)> {
    let mut p: Vec<(i32, String)> = vec![(0, "SECTION".into()), (2, "HEADER".into())];
    for (c, v) in header {
        p.push((*c, v.to_string()));
    }
    p.push((0, "ENDSEC".into()));
    p.extend([(0, "SECTION".to_string()), (2, "TABLES".into()), (0, "TABLE".into()), (2, "LAYER".into())]);
    for (name, flags) in layers {
        p.extend([(0, "LAYER".to_string()), (2, name.to_string()), (70, flags.to_string())]);
    }
    p.extend([(0, "ENDTAB".to_string()), (0, "ENDSEC".into()), (0, "SECTION".into()), (2, "BLOCKS".into())]);
    for (c, v) in blocks {
        p.push((*c, v.to_string()));
    }
    p.extend([(0, "ENDSEC".to_string()), (0, "SECTION".into()), (2, "ENTITIES".into())]);
    for (c, v) in entities {
        p.push((*c, v.to_string()));
    }
    p.extend([(0, "ENDSEC".to_string()), (0, "EOF".into())]);
    p
}

fn ready(p: &[(i32, String)]) -> Vec<(i32, &str)> {
    p.iter().map(|(c, v)| (*c, v.as_str())).collect()
}

/// The rectangle 240 by 144 on WALLS, a circle of radius 18 on FURNITURE at (120, 72), text, and a block of two lines set twice.
fn the_room() -> Vec<(i32, String)> {
    let line = |a: [f64; 2], b: [f64; 2]| -> Vec<(i32, String)> { vec![(0, "LINE".into()), (8, "WALLS".into()), (10, a[0].to_string()), (20, a[1].to_string()), (11, b[0].to_string()), (21, b[1].to_string())] };
    let mut e: Vec<(i32, String)> = Vec::new();
    for (a, b) in [([0., 0.], [240., 0.]), ([240., 0.], [240., 144.]), ([240., 144.], [0., 144.]), ([0., 144.], [0., 0.])] {
        e.extend(line(a, b));
    }
    e.extend([(0, "CIRCLE".to_string()), (8, "FURNITURE".into()), (10, "120".into()), (20, "72".into()), (40, "18".into())]);
    e.extend([(0, "TEXT".to_string()), (8, "NOTES".into()), (10, "60".into()), (20, "72".into()), (40, "6".into()), (1, "KITCHEN".into())]);
    for at in [("30", "30"), ("200", "100")] {
        e.extend([(0, "INSERT".to_string()), (8, "FURNITURE".into()), (2, "CHAIR".into()), (10, at.0.into()), (20, at.1.into())]);
    }
    e
}

fn chair_block() -> Vec<(i32, String)> {
    vec![
        (0, "BLOCK".into()), (8, "0".into()), (2, "CHAIR".into()), (70, "0".into()), (10, "0".into()), (20, "0".into()),
        (0, "LINE".into()), (8, "0".into()), (10, "0".into()), (20, "0".into()), (11, "12".into()), (21, "0".into()),
        (0, "LINE".into()), (8, "0".into()), (10, "0".into()), (20, "0".into()), (11, "0".into()), (21, "12".into()),
        (0, "ENDBLK".into()), (8, "0".into()),
    ]
}

fn room_bytes(units: bool) -> Vec<u8> {
    let b = chair_block();
    let b: Vec<(i32, &str)> = b.iter().map(|(c, v)| (*c, v.as_str())).collect();
    let e = the_room();
    let e: Vec<(i32, &str)> = e.iter().map(|(c, v)| (*c, v.as_str())).collect();
    let header: Vec<(i32, &str)> = if units { vec![(9, "$INSUNITS"), (70, "1")] } else { vec![] };
    let p = with(&header, &[("WALLS", 0), ("FURNITURE", 0), ("NOTES", 0)], &b, &e);
    ascii(&ready(&p))
}

fn layer<'a>(c: &'a dxf::Cad, name: &str) -> &'a dxf::Layer {
    c.layers.iter().find(|l| l.name == name).unwrap()
}

#[test]
fn a_drawing_in_inches_comes_in_at_true_size_with_its_layers_counted() {
    let cad = read(&room_bytes(true)).unwrap();
    let u = cad.units.unwrap();
    assert_eq!((u.code, u.name, u.inches), (1, "inches", 1.0));
    assert_eq!(cad.size, [240.0, 144.0]);
    // 4 on WALLS; the block's 2 lines twice = 4 on FURNITURE (its own layer 0 takes the insert's); the circle is 1 curve.
    assert_eq!(layer(&cad, "WALLS").straight, 4);
    assert_eq!((layer(&cad, "FURNITURE").straight, layer(&cad, "FURNITURE").curves), (4, 1));
    assert_eq!(layer(&cad, "NOTES").texts, 1);
    let straight: usize = cad.layers.iter().map(|l| l.straight).sum();
    assert_eq!(straight, 8);
    // The circle is kept as 36 pieces of 10 degrees.
    assert_eq!(layer(&cad, "FURNITURE").pieces, 36);
    assert!(cad.lost().iter().any(|l| l.contains("its 1 curve is drawn as 36 straight pieces of at most 10 degrees")), "{:?}", cad.lost());
    assert_eq!(cad.blocks, vec![("CHAIR".to_string(), 2)]);
    // Said by kind: 8 lines and 1 circle, never 36 lines.
    assert_eq!(cad.held(), "8 lines, 1 circle");
    assert_eq!(cad.layer_said(layer(&cad, "FURNITURE")), "FURNITURE: 4 lines, 1 circle");
    assert_eq!(cad.words(&[])[0].text, "KITCHEN");
    // The block's layer 0 is the insert's layer: nothing is on a layer 0 of its own.
    assert!(cad.layers.iter().all(|l| l.name != "0"));
}

#[test]
fn a_layer_switched_off_is_not_among_the_lines_and_its_words_go_with_it() {
    let cad = read(&room_bytes(true)).unwrap();
    let all = cad.page(&[]);
    assert_eq!(all.lines.len(), 8 + 36);
    let off = cad.page(&["FURNITURE".to_string()]);
    assert_eq!(off.lines.len(), 4);
    assert_eq!(cad.words(&["NOTES".to_string()]).len(), 0);
    assert_eq!(all.sort, Sort::Dxf);
    // The page is as the sheet is: lower-left at 0, 0, the whole drawing.
    assert!(all.lines.iter().flatten().all(|p| (-1e-9..=240.0 + 1e-9).contains(&p[0]) && (-1e-9..=144.0 + 1e-9).contains(&p[1])));
    // The picture shows ink.
    assert!(all.picture.rgba.chunks(4).any(|p| p[0] < 100));
}

#[test]
fn a_drawing_that_does_not_say_its_unit_has_none() {
    let cad = read(&room_bytes(false)).unwrap();
    assert!(cad.units.is_none());
    assert!(cad.no_units.contains("does not say what it is drawn in"), "{}", cad.no_units);
    // The measure alone is not a unit.
    let p = with(&[(9, "$MEASUREMENT"), (70, "0")], &[], &[], &[(0, "LINE"), (8, "A"), (10, "0"), (20, "0"), (11, "5"), (21, "0")]);
    let cad = read(&ascii(&ready(&p))).unwrap();
    assert!(cad.units.is_none() && cad.no_units.contains("English"), "{}", cad.no_units);
    // Meters.
    let p = with(&[(9, "$INSUNITS"), (70, "6")], &[], &[], &[(0, "LINE"), (8, "A"), (10, "0"), (20, "0"), (11, "5"), (21, "0")]);
    let u = read(&ascii(&ready(&p))).unwrap().units.unwrap();
    assert!((u.inches - 1000.0 / 25.4).abs() < 1e-12 && u.name == "meters");
}

#[test]
fn the_binary_form_reads_as_the_text_form_does() {
    let b = chair_block();
    let b: Vec<(i32, &str)> = b.iter().map(|(c, v)| (*c, v.as_str())).collect();
    let e = the_room();
    let e: Vec<(i32, &str)> = e.iter().map(|(c, v)| (*c, v.as_str())).collect();
    let p = with(&[(9, "$INSUNITS"), (70, "1")], &[("WALLS", 0), ("FURNITURE", 0), ("NOTES", 0)], &b, &e);
    let (a, bin) = (ascii(&ready(&p)), binary(&ready(&p)));
    assert!(is_dxf(&bin) && is_dxf(&a));
    let (x, y) = (read(&a).unwrap(), read(&bin).unwrap());
    assert_eq!(x.segs, y.segs);
    assert_eq!(x.size, y.size);
    // Cut short in the middle of a pair.
    let err = read(&bin[..bin.len() - 40]).map(|_| ()).unwrap_err();
    assert!(err.contains("damaged or cut short") || err.contains("nothing"), "{err}");
}

#[test]
fn a_quarter_arc_runs_counter_clockwise_in_degrees_and_a_mirrored_plane_mirrors_it() {
    let arc = |z: &str| ascii(&ready(&with(&[], &[], &[], &[(0, "ARC"), (8, "A"), (10, "0"), (20, "0"), (40, "10"), (50, "0"), (51, "90"), (210, "0"), (220, "0"), (230, z)])));
    let cad = read(&arc("1")).unwrap();
    // 9 pieces of 10 degrees from (10, 0) to (0, 10); lower-left of what is drawn is (0, 0).
    assert_eq!(cad.layers[0].pieces, 9);
    assert!((cad.size[0] - 10.0).abs() < 1e-9 && (cad.size[1] - 10.0).abs() < 1e-9);
    let first = cad.segs[0];
    assert!((first.a[0] - 10.0).abs() < 1e-9 && first.a[1].abs() < 1e-9, "{first:?}");
    // Facing down, x is mirrored: the arc runs from (-10, 0) up to (0, 10); its origin is (-10, 0).
    let m = read(&arc("-1")).unwrap();
    assert!((m.origin[0] + 10.0).abs() < 1e-9 && m.origin[1].abs() < 1e-9, "{:?}", m.origin);
}

#[test]
fn a_rounded_side_of_a_polyline_lies_on_its_circle() {
    // Two vertices, a bulge of 1 (a half turn) on the first, closed: a circle of radius 5 about (5, 0) and the chord back.
    let p = with(&[], &[], &[], &[(0, "LWPOLYLINE"), (8, "A"), (70, "1"), (90, "2"), (10, "0"), (20, "0"), (42, "1"), (10, "10"), (20, "0")]);
    let cad = read(&ascii(&ready(&p))).unwrap();
    assert_eq!(cad.layers[0].curves, 1);
    assert_eq!(cad.layers[0].straight, 1);
    assert_eq!(cad.layers[0].pieces, 18);
    // The half turn from (0, 0) to (10, 0) with a positive bulge goes counter-clockwise: below the chord.
    let c = [5.0 + cad.origin[0], cad.origin[1] + 5.0];
    let _ = c;
    for s in cad.segs.iter().filter(|s| s.piece) {
        let (ax, ay) = (s.a[0] + cad.origin[0] - 5.0, s.a[1] + cad.origin[1]);
        assert!(((ax * ax + ay * ay).sqrt() - 5.0).abs() < 1e-9, "{s:?}");
    }
    assert!((cad.origin[1] + 5.0).abs() < 1e-9, "the arc bulges below the chord: {:?}", cad.origin);
}

#[test]
fn a_block_set_scaled_turned_and_in_rows() {
    let blocks = [(0, "BLOCK"), (8, "0"), (2, "B"), (70, "0"), (10, "1"), (20, "1"), (0, "LINE"), (8, "0"), (10, "1"), (20, "1"), (11, "3"), (21, "1"), (0, "ENDBLK")];
    // Base (1, 1): the line from the base 2 along x. Set at (10, 20), scale 2, turned 90: from (10, 20) to (10, 24). Two columns 7 apart along the turned x (up).
    let ins = [(0, "INSERT"), (8, "L"), (2, "B"), (10, "10"), (20, "20"), (41, "2"), (42, "2"), (50, "90"), (70, "2"), (44, "7")];
    let cad = read(&ascii(&ready(&with(&[], &[], &blocks, &ins)))).unwrap();
    assert_eq!(cad.segs.len(), 2);
    let abs = |s: &dxf::Seg| ([s.a[0] + cad.origin[0], s.a[1] + cad.origin[1]], [s.b[0] + cad.origin[0], s.b[1] + cad.origin[1]]);
    let (a, b) = abs(&cad.segs[0]);
    assert!((a[0] - 10.0).abs() < 1e-9 && (a[1] - 20.0).abs() < 1e-9 && (b[0] - 10.0).abs() < 1e-9 && (b[1] - 24.0).abs() < 1e-9, "{a:?} {b:?}");
    let (a, _) = abs(&cad.segs[1]);
    assert!((a[0] - 10.0).abs() < 1e-9 && (a[1] - 27.0).abs() < 1e-9, "{a:?}");
    assert_eq!(cad.blocks, vec![("B".to_string(), 2)]);
    // A block the file does not hold is said, not dropped.
    let ghost = read(&ascii(&ready(&with(&[], &[], &[], &[(0, "LINE"), (8, "A"), (10, "0"), (20, "0"), (11, "1"), (21, "0"), (0, "INSERT"), (8, "A"), (2, "NOPE"), (10, "0"), (20, "0")])))).unwrap();
    assert!(ghost.lost().iter().any(|l| l.contains("\"NOPE\"") && l.contains("not brought in")), "{:?}", ghost.lost());
}

#[test]
fn what_cannot_be_carried_is_counted_by_kind() {
    let e = [
        (0, "LINE"), (8, "A"), (10, "0"), (20, "0"), (11, "9"), (21, "0"),
        (0, "HATCH"), (8, "A"), (0, "HATCH"), (8, "A"), (0, "IMAGE"), (8, "A"), (0, "3DSOLID"), (8, "A"), (0, "REGION"), (8, "A"), (0, "WHATSIT"), (8, "A"),
        (0, "LINE"), (8, "A"), (67, "1"), (10, "0"), (20, "0"), (11, "9"), (21, "9"),
    ];
    let cad = read(&ascii(&ready(&with(&[], &[], &[], &e)))).unwrap();
    let said = cad.lost().join(" | ");
    assert!(said.contains("2 hatches (filled or patterned areas) are in the drawing and not brought in"), "{said}");
    assert!(said.contains("1 picture placed in the drawing is in the drawing and not brought in"), "{said}");
    assert!(said.contains("2 3D solids are in the drawing and not brought in"), "{said}");
    assert!(said.contains("1 thing of a kind this program does not know is in the drawing"), "{said}");
    assert!(said.contains("1 thing is drawn on a sheet layout (paper space) and not brought in"), "{said}");
    // A drawing of nothing carried says so, and what it holds.
    let none = read(&ascii(&ready(&with(&[], &[], &[], &[(0, "HATCH"), (8, "A")])))).map(|_| ()).unwrap_err();
    assert!(none.contains("nothing that is drawn as lines or words") && none.contains("1 hatch"), "{none}");
}

#[test]
fn a_dimension_brings_its_figure_and_says_its_lines_are_not_brought() {
    let e = [(0, "LINE"), (8, "A"), (10, "0"), (20, "0"), (11, "240"), (21, "0"), (0, "DIMENSION"), (8, "DIMS"), (2, "*D1"), (70, "0"), (11, "120"), (21, "-12"), (42, "240")];
    let cad = read(&ascii(&ready(&with(&[(9, "$INSUNITS"), (70, "1")], &[], &[], &e)))).unwrap();
    let w = cad.words(&[]);
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].text, "240 in");
    assert!(cad.lost().iter().any(|l| l.contains("1 dimension is in the drawing") && l.contains("lines and arrows are not")), "{:?}", cad.lost());
}

#[test]
fn words_are_cleaned_of_their_codes() {
    assert_eq!(dxf::plain_words("{\\fArial|b0|i0;KITCHEN}\\PEAST WING", true), "KITCHEN EAST WING");
    assert_eq!(dxf::plain_words("20%%d C %%p1/2 \\U+00B0", false), "20\u{b0} C \u{b1}1/2 \u{b0}".replace("\u{b0} \u{b1}", "\u{b0} \u{b1}"));
    assert_eq!(dxf::plain_words("\\S1/2;", true), "1/2");
    assert_eq!(dxf::plain_words("A\\\\B", true), "A\\B");
    // An MTEXT of two chunks.
    let e = [(0, "MTEXT"), (8, "N"), (10, "5"), (20, "5"), (40, "2"), (3, "FIRST "), (1, "SECOND\\PTHIRD"), (0, "LINE"), (8, "N"), (10, "0"), (20, "0"), (11, "9"), (21, "0")];
    let cad = read(&ascii(&ready(&with(&[], &[], &[], &e)))).unwrap();
    assert_eq!(cad.words(&[])[0].text, "FIRST SECOND THIRD");
}

#[test]
fn a_spline_passes_through_the_middle_of_its_cage() {
    // A quadratic with three control points and the knots of one piece: at the middle it is (5, 5).
    let e = [(0, "SPLINE"), (8, "S"), (70, "8"), (71, "2"), (72, "6"), (73, "3"), (40, "0"), (40, "0"), (40, "0"), (40, "1"), (40, "1"), (40, "1"), (10, "0"), (20, "0"), (10, "5"), (20, "10"), (10, "10"), (20, "0")];
    let cad = read(&ascii(&ready(&with(&[], &[], &[], &e)))).unwrap();
    assert_eq!(cad.layers[0].curves, 1);
    assert!(cad.segs.iter().any(|s| (s.a[0] + cad.origin[0] - 5.0).abs() < 1e-9 && (s.a[1] + cad.origin[1] - 5.0).abs() < 1e-9), "no point at (5, 5)");
    assert!((cad.size[0] - 10.0).abs() < 1e-9 && (cad.size[1] - 5.0).abs() < 1e-9, "{:?}", cad.size);
}

#[test]
fn an_old_polyline_with_its_vertices_and_a_frozen_layer() {
    let e = [(0, "POLYLINE"), (8, "P"), (66, "1"), (70, "1"), (0, "VERTEX"), (8, "P"), (10, "0"), (20, "0"), (0, "VERTEX"), (8, "P"), (10, "4"), (20, "0"), (0, "VERTEX"), (8, "P"), (10, "4"), (20, "3"), (0, "SEQEND"), (8, "P")];
    let cad = read(&ascii(&ready(&with(&[], &[("P", 1)], &[], &e)))).unwrap();
    // Closed: three sides.
    assert_eq!(cad.layers[0].straight, 3);
    // Frozen in the file: off as the file has it, and the page says nothing is on it.
    assert_eq!(cad.off_by_the_file(), vec!["P".to_string()]);
    assert_eq!(read_page(&ascii(&ready(&with(&[], &[("P", 1)], &[], &e))), 1).unwrap().lines.len(), 0);
    assert_eq!(cad.page(&[]).lines.len(), 3);
}

#[test]
fn the_files_are_known_by_how_they_begin() {
    assert_eq!(sort_of(&room_bytes(true)), Some(Sort::Dxf));
    assert_eq!(size_of(&room_bytes(true), 1).unwrap(), (Sort::Dxf, 1, [240.0, 144.0]));
    let mut dwg = b"AC1018".to_vec();
    dwg.extend([0u8; 100]);
    let said = closed_format(&dwg, "house.bin").unwrap();
    assert!(said.contains("AutoCAD drawing (DWG, AutoCAD 2004)") && said.contains("save it as a DXF"), "{said}");
    // Whatever its name.
    assert!(read_page(&dwg, 1).unwrap_err().contains("DWG"));
    // By name when it does not begin as one.
    assert!(closed_format(b"something else", "plan.DWG").unwrap().contains("DWG"));
    assert!(closed_format(&room_bytes(true), "plan.dwg").is_none(), "a DXF named .dwg is a DXF");
    let mut skp = vec![0xFF, 0xFE, 0xFF, 0x0E];
    skp.extend("SketchUp Model".bytes().flat_map(|c| [c, 0]));
    assert!(closed_format(&skp, "x.dat").unwrap().contains("SketchUp"));
    assert!(closed_format(b"", "house.skp").unwrap().contains("SketchUp"));
    let mut rvt = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    rvt.extend("BasicFileInfo".bytes().flat_map(|c| [c, 0]));
    let said = closed_format(&rvt, "x.dat").unwrap();
    assert!(said.contains("Revit") && said.contains("IFC"), "{said}");
    assert!(closed_format(b"%PDF-1.4 plain", "a.pdf").is_none());
    assert!(!is_dxf(b"%PDF-1.4\n"));
}

#[test]
fn what_is_written_reads_back_as_it_was_drawn() {
    let mut w = Writer::new();
    w.layer("A-WALL", 7);
    w.line("A-WALL", [0.0, 0.0], [240.0, 0.0]);
    w.line("A-WALL", [240.0, 0.0], [240.0, 144.0]);
    w.circle("A-FURN", [120.0, 72.0], 18.0);
    w.arc("A-DOOR", [0.0, 0.0], 36.0, 0.0, 90.0);
    w.text("A-AREA-IDEN", [60.0, 72.0], 6.0, 0.0, "Kitchen");
    w.text("A-AREA-IDEN", [60.0, 63.0], 6.0, 90.0, "240 SF");
    let text = w.finish();
    assert!(text.contains("$INSUNITS") && text.contains("AC1009"));
    let cad = read(text.as_bytes()).unwrap();
    assert_eq!(cad.units.unwrap().name, "inches");
    assert_eq!(layer(&cad, "A-WALL").straight, 2);
    assert_eq!((layer(&cad, "A-FURN").curves, layer(&cad, "A-DOOR").curves), (1, 1));
    let words: Vec<String> = cad.words(&[]).into_iter().map(|t| t.text).collect();
    assert_eq!(words, vec!["Kitchen".to_string(), "240 SF".to_string()]);
    assert_eq!(strings(text.as_bytes(), 1).unwrap().len(), 2);
    // Its extents: the circle reaches (102, 54) at the lower left of the circle and the arc (0, 0).
    assert!((cad.size[0] - 240.0).abs() < 1e-9, "{:?}", cad.size);
    // Layer 0 is always declared.
    assert!(text.contains("\nLAYER\n"));
}

#[test]
fn damaged_files_are_refused_or_read_and_never_stop_the_program() {
    // Words that end in the middle of a code.
    for w in ["\\U+", "\\U+00", "\\S", "\\f", "%%", "{", "\\"] {
        let _ = dxf::plain_words(w, true);
        let _ = dxf::plain_words(w, false);
    }
    // The room cut at every length, in both forms, and with each byte in turn made a letter that is not one.
    let text = room_bytes(true);
    for n in (0..text.len()).step_by(7) {
        let _ = read(&text[..n]);
    }
    for n in (0..text.len()).step_by(11) {
        let mut broken = text.clone();
        broken[n] = b'Z';
        let _ = read(&broken);
    }
    let b = chair_block();
    let b: Vec<(i32, &str)> = b.iter().map(|(c, v)| (*c, v.as_str())).collect();
    let e = the_room();
    let e: Vec<(i32, &str)> = e.iter().map(|(c, v)| (*c, v.as_str())).collect();
    let bin = binary(&ready(&with(&[(9, "$INSUNITS"), (70, "1")], &[("WALLS", 0)], &b, &e)));
    for n in (0..bin.len()).step_by(5) {
        let _ = read(&bin[..n]);
    }
    // A block that sets itself without end is stopped, not followed.
    let loopy = [(0, "BLOCK"), (8, "0"), (2, "A"), (70, "0"), (10, "0"), (20, "0"), (0, "INSERT"), (8, "0"), (2, "A"), (10, "1"), (20, "0"), (0, "LINE"), (8, "0"), (10, "0"), (20, "0"), (11, "1"), (21, "1"), (0, "ENDBLK"), (8, "0")];
    let ins = [(0, "INSERT"), (8, "L"), (2, "A"), (10, "0"), (20, "0")];
    let cad = read(&ascii(&ready(&with(&[], &[], &loopy, &ins)))).unwrap();
    assert!(cad.lost().iter().any(|l| l.contains("not drawn: the blocks hold blocks too deep")), "{:?}", cad.lost());
}
