//! What a page's reader hands the finder beyond the lines (call 186): which lines are dashed, where a clip or the page's
//! edge cut a line off, and how many pictures the page shows. Each sheet is a PDF written here from a few lines of
//! drawing operators; nothing is a real plan.

use plumbline_plans::read;

/// A one-page PDF, 600 by 400 points, whose content is `ops`, with `image` an 8 by 8 grey picture to draw by name /Im1.
fn pdf(ops: &str, image: bool) -> Vec<u8> {
    let mut objs: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 400] /Contents 4 0 R /Resources << {} >> >>", if image { "/XObject << /Im1 5 0 R >>" } else { "" }).into_bytes(),
        format!("<< /Length {} >>\nstream\n{ops}\nendstream", ops.len()).into_bytes(),
    ];
    if image {
        let hex: String = (0..64).map(|i| if (i / 8 + i % 8) % 2 == 0 { "C8" } else { "78" }).collect::<Vec<_>>().join("") + ">";
        objs.push(format!("<< /Type /XObject /Subtype /Image /Width 8 /Height 8 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /ASCIIHexDecode /Length {} >>\nstream\n{hex}\nendstream", hex.len()).into_bytes());
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut at = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        at.push(out.len());
        out.extend(format!("{} 0 obj\n", i + 1).into_bytes());
        out.extend(o);
        out.extend(b"\nendobj\n");
    }
    let x = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).into_bytes());
    for a in at {
        out.extend(format!("{a:010} 00000 n \n").into_bytes());
    }
    out.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{x}\n%%EOF\n", objs.len() + 1).into_bytes());
    out
}

#[test]
fn dashed_lines_are_named_by_their_place_among_the_lines() {
    // One solid line, one dashed, one solid again.
    let page = read(&pdf("0.5 w 100 100 m 300 100 l S [6 3] 0 d 100 120 m 300 120 l S [] 0 d 100 140 m 300 140 l S", false), 1).unwrap();
    assert_eq!(page.lines.len(), 3);
    assert_eq!(page.dashed, vec![1], "{:?}", page.dashed);
}

#[test]
fn a_clip_and_the_pages_edge_leave_the_places_where_they_cut_a_line() {
    // A line from 100 to 500 under a clip that shows x from 50 to 300: cut at 300. A line off the page's right edge (600): cut there.
    let page = read(&pdf("q 50 50 250 300 re W n 0.5 w 100 100 m 500 100 l S Q 0.5 w 400 200 m 700 200 l S", false), 1).unwrap();
    assert_eq!(page.lines.len(), 2, "{:?}", page.lines);
    let mut cuts: Vec<[i64; 2]> = page.cuts.iter().map(|p| p.map(|v| v.round() as i64)).collect();
    cuts.sort();
    assert_eq!(cuts, vec![[300, 100], [600, 200]], "{:?}", page.cuts);
}

#[test]
fn the_pictures_a_page_shows_are_counted() {
    let page = read(&pdf("0.5 w 100 100 m 300 100 l S q 200 0 0 200 300 100 cm /Im1 Do Q", true), 1).unwrap();
    assert_eq!(page.pictures, 1);
    let page = read(&pdf("0.5 w 100 100 m 300 100 l S", false), 1).unwrap();
    assert_eq!(page.pictures, 0);
}

/// A one-page PDF 600 by 400 points of strings of words in Helvetica, each set by its own text matrix (a b c d e f).
fn text_pdf(pieces: &[(&str, f64, [f64; 6])]) -> Vec<u8> {
    let mut content = String::from("BT\n");
    for (text, size, m) in pieces {
        let escaped = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
        content.push_str(&format!("/F1 {size} Tf\n{} {} {} {} {} {} Tm\n({escaped}) Tj\n", m[0], m[1], m[2], m[3], m[4], m[5]));
    }
    content.push_str("ET");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 400] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_string(),
    ];
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut at = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        at.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for offset in at {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes());
    out
}

#[test]
fn the_strings_a_page_draws_are_read_each_where_it_stands_and_which_way_it_reads() {
    // A level string, one that reads upward (a vertical dimension), one with a word gap and a plain one; the page's lower-left is its corner.
    let page = text_pdf(&[
        ("30'-0\"", 9.0, [1.0, 0.0, 0.0, 1.0, 200.0, 300.0]),
        ("20'-0\"", 9.0, [0.0, 1.0, -1.0, 0.0, 100.0, 120.0]),
        ("5'-8 1/2\"", 9.0, [1.0, 0.0, 0.0, 1.0, 300.0, 150.0]),
        ("ENTRY DOOR", 12.0, [1.0, 0.0, 0.0, 1.0, 50.0, 50.0]),
    ]);
    let got = plumbline_plans::strings(&page, 1).unwrap();
    let said: Vec<&str> = got.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(said, ["30'-0\"", "20'-0\"", "5'-8 1/2\"", "ENTRY DOOR"], "{got:?}");
    let level = &got[0];
    assert!((level.at[0] - 200.0).abs() < 0.5 && (level.at[1] - 300.0).abs() < 0.5, "{level:?}");
    assert!(level.dir[0] > 0.99 && level.dir[1].abs() < 0.01 && (level.tall - 9.0).abs() < 0.5, "{level:?}");
    let up = &got[1];
    assert!((up.at[0] - 100.0).abs() < 0.5 && (up.at[1] - 120.0).abs() < 0.5, "{up:?}");
    assert!(up.dir[0].abs() < 0.01 && up.dir[1] > 0.99, "reads upward: {up:?}");
    // A picture is not read; a page with no words has none.
    assert!(plumbline_plans::strings(b"not a pdf", 1).is_err());
    assert!(plumbline_plans::strings(&pdf("0.5 w 100 100 m 300 100 l S", false), 1).unwrap().is_empty());
}
