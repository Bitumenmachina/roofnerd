//! The marks on a marked-up PDF, read as quantities (call 227). Every file here is written by the test, to the PDF
//! standard's own notes (32000-1, 12.5 and 12.9), with made-up labels and figures. No file a marking program wrote is
//! read: what that program writes could not be confirmed (research/pdf-marks.md).

use plumbline_plans::marks::{figure, keys, read, Figure, Kind};

/// A page 792 by 612 points with the annotation dictionaries given (each the text between << and >>) in its order,
/// and `page_extra` added to the page's own dictionary.
fn pdf(annots: &[&str], page_extra: &str) -> Vec<u8> {
    let first = 5;
    let refs: Vec<String> = (0..annots.len()).map(|i| format!("{} 0 R", first + i)).collect();
    let mut objects: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 792 612] /Contents 4 0 R /Resources << >> /Annots [{}] {page_extra} >>", refs.join(" ")),
        "<< /Length 0 >>\nstream\n\nendstream".into(),
    ];
    for a in annots {
        objects.push(format!("<< /Type /Annot {a} >>"));
    }
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes());
    out
}

/// The scale of the made-up plan: 1/4 in to the foot, so 18 points to the foot; a point is 1/18 ft.
const MEASURE: &str = "/Measure << /Type /Measure /Subtype /RL /R (1/4\" = 1'-0\") /X [<< /U (ft) /C 0.0555555556 >>] /D [<< /U (ft) /C 1 >>] /A [<< /U (sq ft) /C 1 >>] >>";

fn south() -> String {
    format!("/Subtype /Line /Rect [120 120 672 132] /L [126 126 666 126] /Subj (South wall) /Contents (30'-0\") /NM (mk-south) {MEASURE}")
}

#[test]
fn a_length_says_its_figure_and_its_points_work_out_to_the_same_at_its_scale() {
    let bytes = pdf(&[&south()], "");
    let got = read(&bytes, 1).unwrap();
    assert_eq!((got.page, got.pages, got.marks.len()), (1, 1, 1));
    let m = &got.marks[0];
    assert_eq!((m.kind, m.label.as_str(), m.name.as_deref()), (Kind::Length, "South wall", Some("mk-south")));
    let says = m.says.as_ref().unwrap();
    assert_eq!((says.value, says.unit), (30.0, "LF"));
    assert!((m.worked.unwrap() - 30.0).abs() < 1e-6, "{:?}", m.worked);
    assert!(!m.differs());
    assert_eq!(m.unread, None);
    assert_eq!(m.scale.as_deref(), Some("1/4\" = 1'-0\""));
    // Where it is: the middle of its points, on the page from the lower-left corner.
    assert_eq!(m.points, vec![[126.0, 126.0], [666.0, 126.0]]);
    let at = m.at();
    assert!((at[0] - 396.0).abs() < 1e-6 && (at[1] - 126.0).abs() < 1e-6, "{at:?}");
}

#[test]
fn a_run_an_area_and_a_perimeter_are_read_by_their_shape_and_the_unit_they_say() {
    let run = format!("/Subtype /PolyLine /Rect [0 0 1 1] /Vertices [126 126 666 126 666 486 126 486 126 126] /Subj (Outside walls) /Contents (100 LF) /NM (mk-run) {MEASURE}");
    let floor = format!("/Subtype /Polygon /Rect [0 0 1 1] /Vertices [126 126 666 126 666 486 126 486] /Subj (Whole floor) /Contents (600 SF) /NM (mk-floor) {MEASURE}");
    let bed = format!("/Subtype /Polygon /Rect [0 0 1 1] /Vertices [126 126 342 126 342 306 126 306] /Subj (Bedroom) /Contents (120 SF) /NM (mk-bed) {MEASURE}");
    let round = format!("/Subtype /Polygon /Rect [0 0 1 1] /Vertices [126 126 666 126 666 486 126 486] /Subj (Perimeter) /Contents (100 LF) /NM (mk-per) {MEASURE}");
    let got = read(&pdf(&[&run, &floor, &bed, &round], ""), 1).unwrap().marks;
    let seen: Vec<(Kind, &str, f64, &str)> = got.iter().map(|m| (m.kind, m.label.as_str(), m.says.as_ref().unwrap().value, m.says.as_ref().unwrap().unit)).collect();
    assert_eq!(seen, [(Kind::Run, "Outside walls", 100.0, "LF"), (Kind::Area, "Whole floor", 600.0, "SF"), (Kind::Area, "Bedroom", 120.0, "SF"), (Kind::Run, "Perimeter", 100.0, "LF")]);
    // Worked from the points: 540 x 2 + 360 x 2 = 1,800 points = 100 ft; 540 x 360 / 18 / 18 = 600 SF; 216 x 180 / 324 = 120 SF.
    for (m, want) in got.iter().zip([100.0, 600.0, 120.0, 100.0]) {
        assert!((m.worked.unwrap() - want).abs() < 1e-6, "{} {:?}", m.label, m.worked);
        assert!(!m.differs() && m.unread.is_none(), "{}", m.label);
    }
}

#[test]
fn symbols_of_one_label_are_one_count_at_the_first_and_a_note_is_a_note() {
    let stamp = |at: u32| format!("/Subtype /Stamp /Rect [{at} 200 {} 212] /Subj (Receptacle)", at + 12);
    let (a, b, c, d) = (stamp(200), stamp(260), stamp(320), stamp(380));
    let other = "/Subtype /Stamp /Rect [10 10 22 22] /Subj (Switch)".to_string();
    let note = "/Subtype /Text /Rect [500 500 520 520] /Contents (verify header)".to_string();
    let got = read(&pdf(&[&a, &b, &other, &c, &note, &d], ""), 1).unwrap().marks;
    let seen: Vec<(Kind, &str, Option<f64>)> = got.iter().map(|m| (m.kind, m.label.as_str(), m.says.as_ref().map(|s| s.value))).collect();
    assert_eq!(seen, [(Kind::Count, "Receptacle", Some(4.0)), (Kind::Count, "Switch", Some(1.0)), (Kind::Note, "verify header", None)]);
    assert_eq!(got[0].places.len(), 4);
    assert!((got[0].at()[0] - 206.0).abs() < 1e-6, "{:?}", got[0].at());
    assert_eq!(got[2].text, "verify header");
    assert!(got[2].worked.is_none() && got[2].unread.is_none());
}

#[test]
fn a_mark_whose_own_text_and_points_disagree_gives_both_and_is_flagged() {
    let north = format!("/Subtype /Line /Rect [0 0 1 1] /L [126 486 666 486] /Subj (North wall) /Contents (31'-0\") {MEASURE}");
    let m = &read(&pdf(&[&north], ""), 1).unwrap().marks[0];
    assert_eq!(m.says.as_ref().unwrap().value, 31.0);
    assert!((m.worked.unwrap() - 30.0).abs() < 1e-6);
    assert!(m.differs());
    // Within a hundredth is the same: 30.005 LF said, 30 worked.
    let near = format!("/Subtype /Line /Rect [0 0 1 1] /L [126 486 666 486] /Subj (Near) /Contents (30.005 LF) {MEASURE}");
    assert!(!read(&pdf(&[&near], ""), 1).unwrap().marks[0].differs());
    let far = format!("/Subtype /Line /Rect [0 0 1 1] /L [126 486 666 486] /Subj (Far) /Contents (30.02 LF) {MEASURE}");
    assert!(read(&pdf(&[&far], ""), 1).unwrap().marks[0].differs());
}

#[test]
fn what_cannot_be_read_is_named_with_why_and_is_given_no_figure() {
    let curve = format!("/Subtype /Circle /Rect [100 100 160 160] /Subj (Round bed) /Contents (50 SF) {MEASURE}");
    let no_scale = "/Subtype /Polygon /Rect [0 0 1 1] /Vertices [126 126 666 126 666 486] /Subj (Loose area) /Contents (80 SF)".to_string();
    let metric = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 100 0] /Subj (Metric run) /Contents (9 m) {MEASURE}");
    let silent = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 100 0] /Subj (Silent) /Contents (see detail) {MEASURE}");
    let wrong = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 100 0] /Subj (Odd) /Contents (40 SF) {MEASURE}");
    let zero = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 100 0] /Subj (Nought) /Contents (0 LF) {MEASURE}");
    let got = read(&pdf(&[&curve, &no_scale, &metric, &silent, &wrong, &zero], ""), 1).unwrap().marks;
    let why: Vec<(&str, &str)> = got.iter().map(|m| (m.label.as_str(), m.unread.as_deref().unwrap())).collect();
    assert_eq!(why[0], ("Round bed", "its edge is a curve, and a curve is not read as a measure"));
    assert_eq!(why[1], ("Loose area", "it has no scale"));
    assert_eq!(why[2], ("Metric run", "it says \"9 m\", a figure in m, a unit this program does not hold"));
    assert_eq!(why[3], ("Silent", "it says no figure"));
    assert_eq!(why[4], ("Odd", "it says 40 SF, an area on a length"));
    assert_eq!(why[5], ("Nought", "it says 0 LF, which is no quantity"));
    // A curve is no length: the figure it said is kept as said, not made a quantity.
    assert!(got.iter().all(|m| m.unread.is_some()));
}

#[test]
fn a_scale_set_for_part_of_the_sheet_is_not_read_for_a_mark_with_none_of_its_own() {
    let vp = "/VP [<< /Type /Viewport /BBox [0 0 400 400] /Name (Plan) /Measure << /Type /Measure /Subtype /RL /R (x) /X [<< /U (ft) /C 0.0555 >>] /D [<< /U (ft) /C 1 >>] /A [<< /U (sq ft) /C 1 >>] >> >>]";
    let loose = "/Subtype /Line /Rect [0 0 1 1] /L [0 0 100 0] /Subj (In a part) /Contents (5 LF)".to_string();
    let m = &read(&pdf(&[&loose], vp), 1).unwrap().marks[0];
    assert_eq!(m.unread.as_deref(), Some("its scale is set for a part of the sheet, which is not read"));
}

#[test]
fn a_scale_in_inches_and_a_scale_not_rectilinear_work_or_say_why() {
    // 1/8 in to the foot: 9 points to the foot; the distance array in inches: 1 point = 12/9 in.
    let inches = "/Measure << /Subtype /RL /R (1/8 in) /X [<< /U (in) /C 1.3333333333 >>] /D [<< /U (in) /C 1 >>] /A [<< /U (sq in) /C 1 >>] >>";
    let line = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 90 0] /Subj (Ten) /Contents (10 ft) {inches}");
    let m = &read(&pdf(&[&line], ""), 1).unwrap().marks[0];
    assert!((m.worked.unwrap() - 10.0).abs() < 1e-6, "{:?}", m.worked);
    let strange = "/Measure << /Subtype /GEO /R (x) >>";
    let line = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 90 0] /Subj (Geo) /Contents (10 ft) {strange}");
    let m = &read(&pdf(&[&line], ""), 1).unwrap().marks[0];
    assert!(m.worked.is_none() && m.not_worked.as_deref() == Some("its scale is of a kind this program does not read"), "{m:?}");
    // It still says its figure; nothing was made of the points.
    assert_eq!(m.says.as_ref().unwrap().value, 10.0);
    assert!(m.unread.is_none());
}

#[test]
fn the_pages_order_stands_and_hidden_marks_and_bare_drawing_are_said_not_listed() {
    let hidden = "/Subtype /Text /Rect [0 0 1 1] /Contents (not shown) /F 2".to_string();
    let bare = "/Subtype /Square /Rect [0 0 50 50]".to_string();
    let link = "/Subtype /Link /Rect [0 0 50 50]".to_string();
    let a = "/Subtype /Text /Rect [0 0 1 1] /Contents (first)".to_string();
    let b = "/Subtype /Text /Rect [0 0 1 1] /Contents (second)".to_string();
    let got = read(&pdf(&[&a, &hidden, &bare, &link, &b], ""), 1).unwrap();
    assert_eq!(got.marks.iter().map(|m| m.label.as_str()).collect::<Vec<_>>(), ["first", "second"]);
    assert_eq!(got.left, ["1 hidden mark is not on the sheet as shown", "1 drawn mark with no words and no measure", "1 link on the page"]);
}

#[test]
fn a_label_in_sixteen_bit_text_and_a_label_in_the_figures_words_are_read() {
    // UTF-16BE with its mark: "Dr" + e-acute as hexadecimal.
    let utf = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 90 0] /Subj <FEFF004400720065> /Contents (5 ft) {MEASURE}");
    assert_eq!(read(&pdf(&[&utf], ""), 1).unwrap().marks[0].label, "Dre");
    // No subject: the words round the figure are the label; none at all: what kind of mark it is.
    let words = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 90 0] /Contents (Hall run: 5 ft) {MEASURE}");
    let bare = format!("/Subtype /Polygon /Rect [0 0 1 1] /Vertices [0 0 90 0 90 90] /Contents (12 SF) {MEASURE}");
    let got = read(&pdf(&[&words, &bare], ""), 1).unwrap().marks;
    assert_eq!((got[0].label.as_str(), got[1].label.as_str()), ("Hall run", "Area"));
}

#[test]
fn a_page_with_no_marks_has_none_and_a_picture_and_a_missing_page_are_refused() {
    let none = std::fs::read(format!("{}/../../demos/plans/made-up-plan.pdf", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let got = read(&none, 1).unwrap();
    assert!(got.marks.is_empty() && got.left.is_empty());
    assert_eq!(read(&none, 2).unwrap_err(), "the file has 1 page; page 2 was asked for");
    let png = std::fs::read(format!("{}/../../demos/plans/made-up-plan.png", env!("CARGO_MANIFEST_DIR"))).unwrap();
    assert_eq!(read(&png, 1).unwrap_err(), "only a PDF has marks that can be read; a picture of a sheet has none");
}

#[test]
fn a_figure_in_a_marks_own_text_is_read_in_the_forms_a_builder_writes() {
    let said = |t: &str| match figure(t) {
        Figure::Said(s) => Some((s.value, s.unit)),
        _ => None,
    };
    assert_eq!(said("30'-0\""), Some((30.0, "LF")));
    assert_eq!(said("31' 6\""), Some((31.5, "LF")));
    assert_eq!(said("12' 6 1/2\""), Some((12.0 + 6.5 / 12.0, "LF")));
    assert_eq!(said("8'"), Some((8.0, "LF")));
    assert_eq!(said("30 LF"), Some((30.0, "LF")));
    assert_eq!(said("12.5 ft"), Some((12.5, "LF")));
    assert_eq!(said("18 in"), Some((1.5, "LF")));
    assert_eq!(said("1,200 sq ft"), Some((1200.0, "SF")));
    assert_eq!(said("600 SF"), Some((600.0, "SF")));
    assert_eq!(said("600 sf."), Some((600.0, "SF")));
    assert_eq!(said("Wall 2x4 at 30 LF"), Some((30.0, "LF")));
    assert_eq!(said("4 EA"), Some((4.0, "EA")));
    // No unit, no figure: a detail's number is not a quantity.
    assert_eq!(said("see 3/A5"), None);
    assert_eq!(said("2x4"), None);
    assert_eq!(said(""), None);
    // A unit not held is named as written, never converted.
    assert_eq!(figure("9.3 m2"), Figure::Unheld { text: "9.3 m2".into(), unit: "m2".into() });
    assert_eq!(figure("12 mm"), Figure::Unheld { text: "12 mm".into(), unit: "mm".into() });
}

#[test]
fn each_mark_has_one_key_its_own_name_or_its_kind_label_and_place_among_the_same() {
    let named = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 90 0] /Subj (A) /Contents (5 ft) /NM (mk-1) {MEASURE}");
    let named_again = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 90 0] /Subj (B) /Contents (5 ft) /NM (mk-1) {MEASURE}");
    let one = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 90 0] /Subj (Wall) /Contents (5 ft) {MEASURE}");
    let two = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 180 0] /Subj (Wall) /Contents (10 ft) {MEASURE}");
    let quoted = format!("/Subtype /Line /Rect [0 0 1 1] /L [0 0 90 0] /Subj (Wall) /Contents (5 ft) /NM (say \"hi\") {MEASURE}");
    let stamp = "/Subtype /Stamp /Rect [0 0 1 1] /Subj (Receptacle)".to_string();
    let got = read(&pdf(&[&named, &named_again, &one, &two, &quoted, &stamp, &stamp], ""), 1).unwrap().marks;
    let k = keys(&got);
    assert_eq!(k, ["mk-1", "mk-1 #2", "length Wall #1", "length Wall #2", "say 'hi'", "count Receptacle"]);
    assert!(k.iter().all(|k| !k.contains('"')));
}
