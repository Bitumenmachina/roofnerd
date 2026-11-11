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
