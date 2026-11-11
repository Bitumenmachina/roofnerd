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
