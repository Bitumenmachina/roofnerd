//! A PDF written by hand (PDF 1.4): a page a drawing, its fills and lines as paths, its
//! lettering strokes, so a page needs no fonts. Streams are deflated.
//!
//! PORTED CODE. Ported from BIMCraft, `crates/core/src/pdf.rs`, at commit
//! f202abd85d1cf165325e2f7b250da0b5e51f5b71 (2026-11-10), https://github.com/IPedrax/bimcraft,
//! Copyright (c) 2026 IPedrax and the BIMCraft contributors, used under the MIT license
//! (BIMCraft is MIT OR Apache-2.0). The notice is in NOTICE at the repository root.
//!
//! What was changed, and nothing else:
//! * `contents` (`pdf.rs:25`) and `pdf` (`pdf.rs:68`) are taken: objects numbered the same
//!   way (1 the catalog, 2 the page tree, then each page and its contents), the
//!   cross-reference table of twenty bytes an entry, the same operators for fills (`rg`,
//!   `f`) and lines (`RG`, `w`, `j`, `d`, `S`).
//! * A page's drawing is already in points with y up (`paper::Drawing`), so BIMCraft's
//!   matrix from pixels to points (`pdf.rs:27`) is not written, and the page's size is the
//!   drawing's. The pictures a page of BIMCraft's may hold (`/Im0`, an image's object and
//!   its mask) are not taken: a plan set has none.
//! * A line's width and a dash's lengths are in points, written as they are.
//! * The same drawings give the same bytes: nothing here reads a clock.
//! * A document information dictionary (`/Info`, a title and a subject, as UTF-16 hex
//!   strings) is added by `pdf_titled` (ours, call 182): a plan set's file carries the job's
//!   title, so a reader's window and a file list name it.

use std::fmt::Write as _;
use std::io::Write as _;

use flate2::write::ZlibEncoder;
use flate2::Compression;

use super::{Drawing, Prim, Rgb};

fn deflate(bytes: &[u8]) -> Vec<u8> {
    let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
    // Writing into a Vec can't fail.
    let _ = z.write_all(bytes);
    z.finish().unwrap_or_default()
}

fn color(c: Rgb) -> String {
    format!("{:.3} {:.3} {:.3}", f32::from(c.0) / 255.0, f32::from(c.1) / 255.0, f32::from(c.2) / 255.0)
}

fn path(s: &mut String, pts: &[(f32, f32)], closed: bool) {
    for (i, (x, y)) in pts.iter().enumerate() {
        let _ = write!(s, "{x:.2} {y:.2} {} ", if i == 0 { "m" } else { "l" });
    }
    if closed {
        s.push_str("h ");
    }
}

/// A page's content stream.
pub(crate) fn contents(d: &Drawing) -> String {
    let mut s = String::from("1 J 0 j\n");
    for p in &d.prims {
        match p {
            Prim::Fill { rings, color: c } => {
                let _ = write!(s, "{} rg ", color(*c));
                for r in rings {
                    path(&mut s, r, true);
                }
                s.push_str("f\n");
            }
            Prim::Line { pts, width, color: c, dash, closed, round } => {
                let dash = dash.as_ref().map_or_else(|| "[] 0 d".to_string(), |d| format!("[{}] 0 d", d.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join(" ")));
                let _ = write!(s, "{} RG {width:.3} w {} j {dash} ", color(*c), if *round { 1 } else { 0 });
                path(&mut s, pts, *closed);
                s.push_str("S\n");
            }
        }
    }
    s
}

/// A text as a PDF string: UTF-16 big endian, in hex, with its byte order mark.
fn text_string(t: &str) -> String {
    let mut s = String::from("<FEFF");
    for u in t.encode_utf16() {
        let _ = write!(s, "{u:04X}");
    }
    s.push('>');
    s
}

/// The pages, each a drawing the size of its paper, as a PDF file.
pub fn pdf(pages: &[Drawing]) -> Vec<u8> {
    pdf_titled(pages, "", "")
}

/// The same, with its own title and subject in the file's information (none when both are empty).
pub fn pdf_titled(pages: &[Drawing], title: &str, subject: &str) -> Vec<u8> {
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut offsets = vec![0usize; 3];
    let object = |out: &mut Vec<u8>, offsets: &mut Vec<usize>, n: usize, dict: &str, stream: Option<&[u8]>| {
        if offsets.len() <= n {
            offsets.resize(n + 1, 0);
        }
        if let Some(o) = offsets.get_mut(n) {
            *o = out.len();
        }
        out.extend(format!("{n} 0 obj\n{dict}\n").bytes());
        if let Some(bytes) = stream {
            out.extend(b"stream\n");
            out.extend(bytes);
            out.extend(b"\nendstream\n");
        }
        out.extend(b"endobj\n");
    };
    // 1 the catalog, 2 the page tree; then a page and its contents, page by page.
    let mut kids = Vec::new();
    let mut next = 3;
    for d in pages {
        let (page, stream) = (next, next + 1);
        next = stream + 1;
        kids.push(format!("{page} 0 R"));
        let dict = format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.3} {:.3}] /Resources << >> /Contents {stream} 0 R >>", d.width, d.height);
        object(&mut out, &mut offsets, page, &dict, None);
        let body = deflate(contents(d).as_bytes());
        object(&mut out, &mut offsets, stream, &format!("<< /Length {} /Filter /FlateDecode >>", body.len()), Some(&body));
    }
    object(&mut out, &mut offsets, 1, "<< /Type /Catalog /Pages 2 0 R >>", None);
    object(&mut out, &mut offsets, 2, &format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), kids.len()), None);
    let info = if title.is_empty() && subject.is_empty() {
        None
    } else {
        let n = next;
        next += 1;
        object(&mut out, &mut offsets, n, &format!("<< /Title {} /Subject {} /Producer {} >>", text_string(title), text_string(subject), text_string("plumbline")), None);
        Some(n)
    };
    let start = out.len();
    let mut xref = format!("xref\n0 {next}\n0000000000 65535 f \n");
    for n in 1..next {
        // Twenty bytes an entry: the space before the line feed is the PDF's.
        let _ = writeln!(xref, "{:010} 00000 n ", offsets.get(n).copied().unwrap_or(0));
    }
    let info = info.map_or(String::new(), |n| format!(" /Info {n} 0 R"));
    let _ = write!(xref, "trailer\n<< /Size {next} /Root 1 0 R{info} >>\nstartxref\n{start}\n%%EOF\n");
    out.extend(xref.bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Drawing {
        let mut d = Drawing::new(288.0, 216.0);
        d.line(vec![(10.0, 10.0), (82.0, 10.0)], 1.0, super::super::INK, None, false);
        d.fill(vec![vec![(20.0, 20.0), (40.0, 20.0), (40.0, 40.0)]], super::super::POCHE);
        d
    }

    #[test]
    fn the_cross_reference_table_points_at_every_object() {
        let bytes = pdf(&[page(), page()]);
        let tail = String::from_utf8_lossy(&bytes[bytes.len() - 40..]).to_string();
        let start: usize = tail.split("startxref\n").nth(1).unwrap().lines().next().unwrap().parse().unwrap();
        let table = String::from_utf8_lossy(&bytes[start..]).to_string();
        let mut lines = table.lines().skip(1);
        let count: usize = lines.next().unwrap().split(' ').nth(1).unwrap().parse().unwrap();
        assert_eq!(count, 1 + 2 + 2 * 2, "the free entry, catalog and pages, and a page and its contents twice");
        for (n, entry) in lines.skip(1).take(count - 1).enumerate() {
            let offset: usize = entry[..10].parse().unwrap();
            let head = format!("{} 0 obj", n + 1);
            assert!(bytes[offset..].starts_with(head.as_bytes()), "object {} isn't at {offset}", n + 1);
        }
        assert!(bytes.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn the_same_drawings_are_the_same_bytes() {
        assert_eq!(pdf(&[page()]), pdf(&[page()]));
    }

    #[test]
    fn a_page_is_the_size_of_its_drawing() {
        let bytes = pdf(&[page()]);
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("/MediaBox [0 0 288.000 216.000]"));
    }
}
