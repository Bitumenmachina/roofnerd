use plumbline_plans::{checksum, read, sort_of, Sort};

fn file(name: &str) -> Vec<u8> {
    std::fs::read(format!("{}/../../demos/plans/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn a_vector_plan_gives_its_picture_its_lines_and_what_it_could_not_give() {
    let bytes = file("made-up-plan.pdf");
    assert_eq!(sort_of(&bytes), Some(Sort::Pdf));
    let page = read(&bytes, 1).unwrap();
    // A letter sheet on its side, drawn 4,096 dots wide: 612 / 792 of that tall.
    assert_eq!((page.pages, page.size), (1, [792.0, 612.0]));
    assert_eq!((page.picture.width, page.picture.height), (4096, 3165));
    assert_eq!(page.picture.rgba.len(), 4096 * 3165 * 4);
    // The eight straight lines the plan was written with, from the page's lower-left
    // corner: the outline's four, the partition, the dimension line and its two ticks.
    let mut lines: Vec<[[i64; 2]; 2]> = page.lines.iter().map(|l| l.map(|p| p.map(|v| v.round() as i64))).collect();
    lines.sort();
    let mut want = vec![[[126, 126], [666, 126]], [[666, 126], [666, 486]], [[666, 486], [126, 486]], [[126, 486], [126, 126]], [[342, 126], [342, 486]], [[126, 90], [666, 90]], [[126, 82], [126, 98]], [[666, 82], [666, 98]]];
    want.sort();
    assert_eq!(lines, want);
    // The dimension line is 540 points for 30 ft: 18 points to the foot.
    let dim = page.lines.iter().find(|l| (l[0][1] - 90.0).abs() < 1e-6 && (l[1][1] - 90.0).abs() < 1e-6).unwrap();
    assert!(((dim[1][0] - dim[0][0]).abs() - 540.0).abs() < 1e-6);
    // What it could not give: the door's swing is a curve, and the words are not read.
    assert_eq!(page.lost, ["1 curved piece of line is known by its ends only", "the words and figures on the page are drawn and not read: a dimension is typed by whoever reads it"]);
    // The picture is the plan: white paper, and ink where the outline's south wall is
    // (126 points up from the bottom is row 3165 − 126 × 4096 / 792 = 2513).
    let dot = |x: u32, y: u32| {
        let i = ((y * page.picture.width + x) * 4) as usize;
        [page.picture.rgba[i], page.picture.rgba[i + 1], page.picture.rgba[i + 2]]
    };
    assert_eq!(dot(50, 50), [255, 255, 255]);
    assert!((2510..=2516).any(|y| dot(2000, y)[0] < 80), "no ink on the south wall's line");
}

#[test]
fn a_picture_is_a_picture_and_a_file_that_is_neither_is_refused() {
    for (name, sort) in [("made-up-plan.png", Sort::Png), ("made-up-plan.jpg", Sort::Jpeg)] {
        let page = read(&file(name), 1).unwrap();
        assert_eq!((page.sort, page.pages, page.size, page.lines.len()), (sort, 1, [1584.0, 1224.0], 0), "{name}");
        assert_eq!(page.lost, ["it is a picture: it has no lines of its own to snap to, and is traced over by eye"]);
        assert_eq!(read(&file(name), 2).unwrap_err(), "a picture has one page; page 2 was asked for");
    }
    assert_eq!(read(b"not a plan at all", 1).unwrap_err(), "it is not a PDF, a PNG or a JPEG");
    let pdf = file("made-up-plan.pdf");
    assert_eq!(read(&pdf, 2).unwrap_err(), "the file has 1 page; page 2 was asked for");
    assert_eq!(read(&pdf, 0).unwrap_err(), "the file has 1 page; page 0 was asked for");
    assert!(read(&pdf[..200], 1).is_err());
    // A fingerprint is the same for the same bytes and not for others.
    assert_eq!(checksum(b""), "cbf29ce484222325");
    assert_eq!(checksum(&pdf), checksum(&pdf.clone()));
    assert_ne!(checksum(&pdf), checksum(&pdf[1..]));
}

/// A one-page PDF written by hand: a page `width` by `height` points whose content is
/// `content`, with its cross-reference table worked out.
fn pdf(width: u32, height: u32, content: &str) -> Vec<u8> {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] /Contents 4 0 R /Resources << >> >>"),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
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

fn rounded(lines: &[[[f64; 2]; 2]]) -> Vec<[[i64; 2]; 2]> {
    let mut all: Vec<[[i64; 2]; 2]> = lines.iter().map(|l| l.map(|p| p.map(|v| v.round() as i64))).collect();
    all.sort();
    all
}

#[test]
fn what_a_drawn_page_holds_that_is_not_a_plain_line_is_read_or_said() {
    // A wall drawn as a solid bar, 400 by 12 points: its four edges are lines.
    let bar = read(&pdf(792, 612, "100 300 400 12 re f"), 1).unwrap();
    assert_eq!(rounded(&bar.lines), [[[100, 300], [500, 300]], [[100, 312], [100, 300]], [[500, 300], [500, 312]], [[500, 312], [100, 312]]]);
    assert!(bar.lost.is_empty(), "{:?}", bar.lost);
    // One line 400 points long and three ticks of 0.8 of a point: the ticks are not
    // kept, and how many is said.
    let ticks = read(&pdf(792, 612, "1 w 100 300 m 500 300 l S 100 100 m 100.8 100 l S 110 100 m 110.8 100 l S 120 100 m 120.8 100 l S"), 1).unwrap();
    assert_eq!(ticks.lines.len(), 1);
    assert!(ticks.lost.iter().any(|l| l.starts_with("3 lines under a point long are not kept")), "{:?}", ticks.lost);
    // A line across the page with only its middle showing through a mask from 300 to
    // 400: what is kept is that middle.
    let masked = read(&pdf(792, 612, "q 300 0 100 612 re W n 1 w 0 300 m 792 300 l S Q"), 1).unwrap();
    assert_eq!(rounded(&masked.lines), [[[300, 300], [400, 300]]]);
    assert!(masked.lost.iter().any(|l| l.contains("1 line is cut to what shows")), "{:?}", masked.lost);
    // A black line and a white one: the white one is not ink.
    let white = read(&pdf(792, 612, "1 w 0 0 0 RG 100 300 m 500 300 l S 1 1 1 RG 100 200 m 500 200 l S"), 1).unwrap();
    assert_eq!(rounded(&white.lines), [[[100, 300], [500, 300]]]);
    assert!(white.lost.iter().any(|l| l.starts_with("1 line drawn in white")), "{:?}", white.lost);
    // A dashed line is kept whole, and that is said.
    let dashed = read(&pdf(792, 612, "1 w [6 3] 0 d 100 300 m 500 300 l S"), 1).unwrap();
    assert_eq!(dashed.lines.len(), 1);
    assert!(dashed.lost.iter().any(|l| l.starts_with("1 dashed line is kept whole")), "{:?}", dashed.lost);
    // A page with no size is not given one; a file cut short has no pages and says so.
    let refusal = |bytes: &[u8]| match read(bytes, 1) {
        Err(why) => why,
        Ok(page) => panic!("read, {} lines", page.lines.len()),
    };
    assert!(refusal(&pdf(0, 0, "100 300 m 500 300 l S")).contains("gives no size of its own"));
    // A file cut off after its first few bytes has no pages. One cut in half still has
    // its page and has lost what was drawn on it: it is read, and said to be cut short.
    let whole = pdf(792, 612, "100 300 m 500 300 l S");
    let why = refusal(&whole[..40]);
    assert!(why.contains("cut short or damaged") || why.contains("could not be read"), "{why}");
    match read(&whole[..whole.len() / 2], 1) {
        Ok(half) => assert!(half.lost.iter().any(|l| l.contains("does not end as a PDF does")), "{:?}", half.lost),
        Err(why) => assert!(why.contains("cut short") || why.contains("could not be read"), "{why}"),
    }
    assert!(!read(&whole, 1).unwrap().lost.iter().any(|l| l.contains("does not end as a PDF does")));
}

#[test]
fn a_picture_too_big_to_show_whole_is_shown_smaller_and_keeps_its_own_size() {
    // 6,000 by 3,000 dots, all white: shown at 4,096 by 2,048, and its size is its own.
    let big = image::RgbaImage::from_pixel(6000, 3000, image::Rgba([255, 255, 255, 255]));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(big).write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
    let page = read(&bytes, 1).unwrap();
    assert_eq!(page.size, [6000.0, 3000.0]);
    assert_eq!((page.picture.width, page.picture.height, page.picture.rgba.len()), (4096, 2048, 4096 * 2048 * 4));
    assert!(page.lost.iter().any(|l| l.contains("6000 by 3000 dots and is shown at 4096 by 2048")), "{:?}", page.lost);
    assert_eq!(plumbline_plans::size_of(&bytes, 1).unwrap().2, [6000.0, 3000.0]);
}

// ---- The fifteenth judgment, part A.

#[test]
fn j15_18_24_a_line_blanked_out_and_a_line_off_the_page_are_not_there_to_snap_to() {
    // A sheet 400 by 300. Three lines drawn, then a white box filled over the second:
    // the first stays, the second is under the box, the third is drawn after the box and
    // stays. A fourth line is wholly off the sheet; a fifth runs off its right edge.
    let content = "0 0 0 RG 1 w\n\
        50 250 m 350 250 l S\n\
        120 150 m 280 150 l S\n\
        1 1 1 rg 100 100 200 100 re f\n\
        0 0 0 RG 110 120 m 290 120 l S\n\
        500 50 m 600 50 l S\n\
        300 40 m 450 40 l S";
    let page = read(&pdf(400, 300, content), 1).unwrap();
    let mut lines = rounded(&page.lines);
    lines.sort();
    assert_eq!(lines, [[[50, 250], [350, 250]], [[110, 120], [290, 120]], [[300, 40], [400, 40]]]);
    let lost = page.lost.join(" | ");
    assert!(lost.contains("1 line is under a white box drawn over it"), "{lost}");
    assert!(lost.contains("1 line is drawn off the page and not kept; 1 line that runs off it is cut at its edge"), "{lost}");
    // The picture agrees: no ink where the blanked line was (150 up from the bottom, at
    // the sheet's middle), ink on the line above it.
    let k = f64::from(page.picture.width) / 400.0;
    let dark = |x: f64, up: f64| {
        let (px, py) = ((x * k) as u32, ((300.0 - up) * k) as u32);
        (py.saturating_sub(3)..=py + 3).any(|y| page.picture.rgba[((y * page.picture.width + px) * 4) as usize] < 100)
    };
    assert!(!dark(200.0, 150.0), "the blanked line shows in the picture");
    assert!(dark(200.0, 250.0) && dark(200.0, 120.0));
    // A white shape that is not a square over a line: the line is kept, and that is said.
    let content = "0 0 0 RG 1 w\n120 150 m 280 150 l S\n1 1 1 rg 100 100 m 300 100 l 200 220 l h f";
    let page = read(&pdf(400, 300, content), 1).unwrap();
    assert_eq!(rounded(&page.lines), [[[120, 150], [280, 150]]]);
    assert!(page.lost.iter().any(|l| l.contains("1 white shape is drawn over lines and is not a square")), "{:?}", page.lost);
}

/// A one-page sheet of words written by hand: each line as its text and the height of
/// its letters in points, set down the page from the top, in Helvetica.
fn sheet(lines: &[(&str, u32)]) -> Vec<u8> {
    let mut content = String::from("BT\n");
    let mut y = 740;
    for (text, size) in lines {
        let escaped = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
        content.push_str(&format!("/F1 {size} Tf\n1 0 0 1 72 {y} Tm\n({escaped}) Tj\n"));
        y -= size + 8;
    }
    content.push_str("ET");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
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

/// The words of a manufacturer's sheet are read line by line, top to bottom, with how
/// tall each line's letters are. (A made-up sheet for a made-up vanity.)
#[test]
fn the_words_of_a_sheet_are_read_line_by_line() {
    let made = sheet(&[("Harbor Vanity 36", 24), ("Northfield Millwork", 12), ("Model: NV-3621", 11), ("Overall dimensions: 36\" W x 21\" D x 34-1/2\" H", 11), ("Shipping weight: 96 lb", 11)]);
    let read = plumbline_plans::words(&made, 1).unwrap();
    let text: Vec<&str> = read.iter().map(|w| w.text.as_str()).collect();
    assert_eq!(text, ["Harbor Vanity 36", "Northfield Millwork", "Model: NV-3621", "Overall dimensions: 36\" W x 21\" D x 34-1/2\" H", "Shipping weight: 96 lb"]);
    // The name is the tallest line: 24 points against 12 and 11.
    assert!((read[0].size - 24.0).abs() < 0.5 && (read[1].size - 12.0).abs() < 0.5, "{:?}", read.iter().map(|w| w.size).collect::<Vec<_>>());
    // A page with nothing written on it has no words, and a picture is not read at all.
    assert!(plumbline_plans::words(&pdf(612, 792, "100 300 400 12 re f"), 1).unwrap().is_empty());
    assert!(plumbline_plans::words(&made, 2).is_err());
    assert!(plumbline_plans::words(b"not a pdf", 1).unwrap_err().contains("only a PDF has words"));
}
