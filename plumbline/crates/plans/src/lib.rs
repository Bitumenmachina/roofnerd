//! Plan files read for tracing over: one page of a PDF, or a picture (PNG, JPEG), as
//! pixels to show under the tracing and, where the file holds them, the straight lines
//! drawn on the page to snap to.
//!
//! The PDF reading is `hayro`'s (Apache-2.0 OR MIT): it draws a page to pixels, and it
//! hands each path it draws to a device, which is how a vector plan's own lines are read
//! out here. The pictures are the `image` crate's. Nothing here knows about a job: the
//! world crate holds the drawing and its scale, and reads no disk.
//!
//! What a file could not give is said, not dropped (`Page::lost`): a curve is known by
//! its two ends only; words are drawn and not read; a picture has no lines at all; lines
//! under a point long, lines in white, dashed lines and lines under a mask are each
//! counted and said. The edges of filled shapes are lines, as strokes are.

use hayro::hayro_interpret::font::GlyphRun;
use hayro::hayro_interpret::util::TransformExt;
use hayro::hayro_interpret::{interpret_page, BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, ImageDrawProps, InterpreterSettings, SoftMask};
use hayro::hayro_syntax::Pdf;
use hayro::kurbo::{BezPath, PathEl, Point, Rect, Shape};
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{render, PixmapSettings, RenderCache, RenderSettings};

/// The most dots a page is drawn at along its longer side.
pub const LONG_SIDE: u32 = 4096;
/// The most straight lines kept from one page. A hatched sheet can hold millions.
pub const MOST_LINES: usize = 400_000;
/// A line shorter than this, in points, is a dot or a hatch tick and is not kept.
pub const SHORTEST_LINE_PT: f64 = 1.0;

/// A picture: its rows from the top, four bytes to the dot (red, green, blue, opacity).
#[derive(Clone, Debug, PartialEq)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// One page of a plan file, read.
#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    /// What kind of file it came from.
    pub sort: Sort,
    /// How many pages the file has.
    pub pages: usize,
    /// The page's width and height in its own units: points (1/72 in) for a PDF, dots
    /// for a picture. A distance on the page means nothing until the drawing has a scale.
    pub size: [f64; 2],
    pub picture: Picture,
    /// The straight lines drawn on the page, each as two points in the page's units,
    /// measured from its lower-left corner, up and to the right.
    pub lines: Vec<[[f64; 2]; 2]>,
    /// What reading it could not give, each in a sentence.
    pub lost: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Pdf,
    Png,
    Jpeg,
}

impl Sort {
    pub fn said(self) -> &'static str {
        match self {
            Sort::Pdf => "PDF",
            Sort::Png => "PNG picture",
            Sort::Jpeg => "JPEG picture",
        }
    }
}

/// What kind of plan file these bytes are, by how they begin; none if they are neither
/// a PDF, a PNG nor a JPEG.
pub fn sort_of(bytes: &[u8]) -> Option<Sort> {
    if bytes.windows(5).take(1024).any(|w| w == b"%PDF-") {
        Some(Sort::Pdf)
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some(Sort::Png)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        Some(Sort::Jpeg)
    } else {
        None
    }
}

/// A file's fingerprint, so a file that has changed since a drawing was made from it is
/// noticed: FNV-1a over its bytes, sixteen hexadecimal digits. It is for noticing a
/// change, not for proving one did not happen.
pub fn checksum(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Collects what a page draws: its straight lines (strokes, and the edges of filled
/// shapes), and a count of everything that is not a line or was not read as drawn.
#[derive(Default)]
struct Lines {
    height: f64,
    width: f64,
    /// Lines drawn wholly off the page, which are not kept, and lines cut at its edge.
    off_page: usize,
    off_page_cut: usize,
    /// Lines under a white box drawn over them afterwards, which are not kept; and
    /// white shapes that are not a square, under which lines are kept.
    covered: usize,
    odd_white: usize,
    kept: Vec<[[f64; 2]; 2]>,
    over: usize,
    curves: usize,
    pictures: usize,
    words: usize,
    /// Lines under `SHORTEST_LINE_PT` long, which are not kept.
    short: usize,
    /// Lines drawn in white, or in no colour at all: not ink, and not kept.
    white: usize,
    /// Lines drawn dashed, kept whole.
    dashed: usize,
    /// Lines cut to what a square clip shows of them, and lines a clip hides whole.
    cut: usize,
    hidden: usize,
    /// Clips that are not a square: lines under them are kept whole.
    odd_clips: usize,
    /// The clips in force, each a square (left, top, right, bottom, as the page is
    /// drawn) or, for a clip of another shape, none.
    clips: Vec<Option<[f64; 4]>>,
}

/// The part of a line inside a square, or none of it.
fn inside(a: Point, b: Point, [x0, y0, x1, y1]: [f64; 4]) -> Option<(Point, Point)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (mut from, mut to) = (0.0_f64, 1.0_f64);
    for (p, q) in [(-dx, a.x - x0), (dx, x1 - a.x), (-dy, a.y - y0), (dy, y1 - a.y)] {
        if p.abs() < 1.0e-12 {
            if q < 0.0 {
                return None;
            }
            continue;
        }
        let r = q / p;
        if p < 0.0 {
            from = from.max(r);
        } else {
            to = to.min(r);
        }
    }
    (to > from).then(|| (Point::new(a.x + dx * from, a.y + dy * from), Point::new(a.x + dx * to, a.y + dy * to)))
}

impl Lines {
    fn line(&mut self, a: Point, b: Point, dashed: bool) {
        if ![a.x, a.y, b.x, b.y].iter().all(|v| v.is_finite()) {
            return;
        }
        // What the clips in force show of it.
        let (mut a, mut b) = (a, b);
        for clip in self.clips.clone().into_iter().flatten() {
            match inside(a, b, clip) {
                None => {
                    self.hidden += 1;
                    return;
                }
                Some((c, d)) => {
                    if (c - a).hypot() > 1.0e-6 || (d - b).hypot() > 1.0e-6 {
                        self.cut += 1;
                    }
                    (a, b) = (c, d);
                }
            }
        }
        // What of it is on the page: a line drawn off the sheet is not on the drawing.
        if self.width > 0.0 && self.height > 0.0 {
            match inside(a, b, [0.0, 0.0, self.width, self.height]) {
                None => {
                    self.off_page += 1;
                    return;
                }
                Some((c, d)) => {
                    if (c - a).hypot() > 1.0e-6 || (d - b).hypot() > 1.0e-6 {
                        self.off_page_cut += 1;
                    }
                    (a, b) = (c, d);
                }
            }
        }
        if (b - a).hypot() < SHORTEST_LINE_PT {
            self.short += 1;
            return;
        }
        if self.kept.len() >= MOST_LINES {
            self.over += 1;
            return;
        }
        if dashed {
            self.dashed += 1;
        }
        // The page is drawn from its top; lines are kept from its lower-left corner up.
        self.kept.push([[a.x, self.height - a.y], [b.x, self.height - b.y]]);
    }

    fn path(&mut self, path: &BezPath, to_page: hayro::kurbo::Affine, dashed: bool) {
        let mut start: Option<Point> = None;
        let mut at: Option<Point> = None;
        for el in path.elements() {
            match *el {
                PathEl::MoveTo(p) => {
                    start = Some(p);
                    at = Some(p);
                }
                PathEl::LineTo(p) => {
                    if let Some(a) = at {
                        self.line(to_page * a, to_page * p, dashed);
                    }
                    at = Some(p);
                }
                PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => {
                    self.curves += 1;
                    at = Some(p);
                }
                PathEl::ClosePath => {
                    if let (Some(a), Some(s)) = (at, start) {
                        if (a - s).hypot() > 1.0e-9 {
                            self.line(to_page * a, to_page * s, dashed);
                        }
                    }
                    at = start;
                }
            }
        }
    }
}

/// A path that is a square with its sides level and upright: its left, top, right and
/// bottom as the page is drawn. None for any other shape.
fn square(path: &BezPath) -> Option<[f64; 4]> {
    let corners: Vec<Point> = path
        .elements()
        .iter()
        .filter_map(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
            _ => None,
        })
        .collect();
    let straight = path.elements().iter().all(|el| matches!(el, PathEl::MoveTo(_) | PathEl::LineTo(_) | PathEl::ClosePath));
    let bounds = path.bounding_box();
    let on_corner = |p: &Point| ((p.x - bounds.x0).abs() < 1.0e-6 || (p.x - bounds.x1).abs() < 1.0e-6) && ((p.y - bounds.y0).abs() < 1.0e-6 || (p.y - bounds.y1).abs() < 1.0e-6);
    (straight && (4..=5).contains(&corners.len()) && corners.iter().all(on_corner) && bounds.width() > 0.0 && bounds.height() > 0.0).then_some([bounds.x0, bounds.y0, bounds.x1, bounds.y1])
}

/// How many straight pieces a path has.
fn straight_pieces(path: &BezPath) -> usize {
    path.elements().iter().filter(|el| matches!(el, PathEl::LineTo(_) | PathEl::ClosePath)).count()
}

impl<'a> Device<'a> for Lines {
    fn draw_path(&mut self, path: &BezPath, props: DrawProps<'a>, mode: &DrawMode) {
        // White on white paper, or no colour at all, is not ink: such a line is not one
        // to snap to. A pattern is ink.
        if let hayro::hayro_interpret::Paint::Color(c) = &props.paint {
            let [r, g, b, alpha] = c.to_rgba().to_rgba8();
            if alpha == 0 || (r >= 250 && g >= 250 && b >= 250) {
                self.white += straight_pieces(path);
                // A white shape filled over what was drawn before it blanks it out, as a
                // draughtsman blanks part of a sheet: lines wholly under it are not on
                // the drawing any more. Worked out for a square; said for any other shape.
                if alpha == 255 && matches!(mode, DrawMode::Fill(_) | DrawMode::FillAndStroke(_, _)) {
                    let placed = props.transform * path.clone();
                    match square(&placed) {
                        Some([x0, y0, x1, y1]) => {
                            // Kept lines are held from the page's lower-left corner up.
                            let (low, high) = (self.height - y1, self.height - y0);
                            let under = |p: &[f64; 2]| p[0] >= x0 - 1.0e-6 && p[0] <= x1 + 1.0e-6 && p[1] >= low - 1.0e-6 && p[1] <= high + 1.0e-6;
                            let before = self.kept.len();
                            self.kept.retain(|l| !(under(&l[0]) && under(&l[1])));
                            self.covered += before - self.kept.len();
                        }
                        None if !self.kept.is_empty() => self.odd_white += 1,
                        None => {}
                    }
                }
                return;
            }
        }
        match mode {
            // A stroke is a line. So are the edges of a filled shape: walls are often
            // drawn as solid bars, and a bar's edges are the wall's faces.
            DrawMode::Stroke(stroke) | DrawMode::FillAndStroke(_, stroke) => self.path(path, props.transform, !stroke.dash_array.is_empty()),
            DrawMode::Fill(_) => self.path(path, props.transform, false),
            DrawMode::Invisible => {}
        }
    }
    fn push_clip_path(&mut self, clip: &ClipPath) {
        // A clip that is a square with its sides level and upright is kept as one; any
        // other shape is not worked out, and lines under it are kept whole.
        match square(&clip.path) {
            Some(sides) => self.clips.push(Some(sides)),
            None => {
                self.odd_clips += 1;
                self.clips.push(None);
            }
        }
    }
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'a>>, _: BlendMode) {}
    fn draw_glyph_run(&mut self, _: &GlyphRun<'_, 'a>, _: DrawProps<'a>, _: &DrawMode) {
        self.words += 1;
    }
    fn draw_image(&mut self, _: hayro::hayro_interpret::Image<'a, '_>, _: ImageDrawProps<'a>) {
        self.pictures += 1;
    }
    fn pop_clip(&mut self) {
        self.clips.pop();
    }
    fn pop_transparency_group(&mut self) {}
}

/// The most dots a picture may have for it to be opened at all: a sheet four feet by
/// three scanned at 300 to the inch is 155 million.
pub const MOST_DOTS: u64 = 250_000_000;

/// A picture file opened and turned the way its own note says it is to be seen (a
/// camera's or a scanner's), with its size as it is then.
fn opened(bytes: &[u8]) -> Result<(image::DynamicImage, bool), String> {
    use image::ImageDecoder;
    let bad = |_: image::ImageError| "the picture could not be read: it is damaged, cut short, or not the kind of picture its name says. Save it again as a PNG or a JPEG".to_string();
    let mut decoder = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().map_err(|_| "the picture could not be read: it is damaged, cut short, or not the kind of picture its name says. Save it again as a PNG or a JPEG".to_string())?.into_decoder().map_err(bad)?;
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MOST_DOTS {
        return Err(format!("the picture is {width} by {height} dots, more than this program opens ({} million): save it smaller", MOST_DOTS / 1_000_000));
    }
    let turn = decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut picture = image::DynamicImage::from_decoder(decoder).map_err(bad)?;
    picture.apply_orientation(turn);
    Ok((picture, turn != image::metadata::Orientation::NoTransforms))
}

/// Read one page (counted from 1) of a plan file.
pub fn read(bytes: &[u8], page: usize) -> Result<Page, String> {
    match sort_of(bytes) {
        None => Err("it is not a PDF, a PNG or a JPEG".into()),
        Some(Sort::Pdf) => read_pdf(bytes, page),
        Some(sort) => {
            if page != 1 {
                return Err(format!("a picture has one page; page {page} was asked for"));
            }
            let (opened, turned) = opened(bytes)?;
            let (width, height) = (opened.width(), opened.height());
            if width == 0 || height == 0 {
                return Err("the picture has no size".into());
            }
            let mut lost = vec!["it is a picture: it has no lines of its own to snap to, and is traced over by eye".to_string()];
            if turned {
                lost.push("the picture's own note says which way up it is seen, and it is shown that way; its size is as shown".into());
            }
            // Shown at no more than LONG_SIDE dots on its longer side, as a PDF's page is.
            // Its size stays its own dots, so a scale picked on it is right.
            let shown = if width.max(height) > LONG_SIDE {
                let k = f64::from(LONG_SIDE) / f64::from(width.max(height));
                let (w, h) = (((f64::from(width) * k).round() as u32).max(1), ((f64::from(height) * k).round() as u32).max(1));
                lost.push(format!("the picture is {width} by {height} dots and is shown at {w} by {h}: close in, fine detail is blurred"));
                opened.resize_exact(w, h, image::imageops::FilterType::Triangle).to_rgba8()
            } else {
                opened.to_rgba8()
            };
            Ok(Page { sort, pages: 1, size: [f64::from(width), f64::from(height)], picture: Picture { width: shown.width(), height: shown.height(), rgba: shown.into_raw() }, lines: Vec::new(), lost })
        }
    }
}

/// What a page is without drawing it: the sort of file, how many pages the file has, and
/// the page's size (a PDF's in points, a picture's in dots). The refusals are `read`'s.
pub fn size_of(bytes: &[u8], page: usize) -> Result<(Sort, usize, [f64; 2]), String> {
    match sort_of(bytes) {
        None => Err("it is not a PDF, a PNG or a JPEG".into()),
        Some(Sort::Pdf) => {
            let pdf = Pdf::new(bytes.to_vec()).map_err(|_| "the PDF could not be read: it is damaged, cut short, locked with a password, or not a PDF at all. Print it to a new PDF, or send it again".to_string())?;
            let pages = pdf.pages();
            let count = pages.len();
            let (width, height) = page_size(pages, page)?;
            Ok((Sort::Pdf, count, [f64::from(width), f64::from(height)]))
        }
        Some(sort) => {
            if page != 1 {
                return Err(format!("a picture has one page; page {page} was asked for"));
            }
            // Its size as it is seen: a picture whose own note turns it on its side is as
            // wide as it was tall.
            use image::ImageDecoder;
            let bad = |_: image::ImageError| "the picture could not be read: it is damaged, cut short, or not the kind of picture its name says. Save it again as a PNG or a JPEG".to_string();
            let mut decoder = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().map_err(|_| "the picture could not be read: it is damaged, cut short, or not the kind of picture its name says. Save it again as a PNG or a JPEG".to_string())?.into_decoder().map_err(bad)?;
            let (mut width, mut height) = decoder.dimensions();
            if width == 0 || height == 0 {
                return Err("the picture has no size".into());
            }
            if u64::from(width) * u64::from(height) > MOST_DOTS {
                return Err(format!("the picture is {width} by {height} dots, more than this program opens ({} million): save it smaller", MOST_DOTS / 1_000_000));
            }
            use image::metadata::Orientation as O;
            if matches!(decoder.orientation().unwrap_or(O::NoTransforms), O::Rotate90 | O::Rotate270 | O::Rotate90FlipH | O::Rotate270FlipH) {
                std::mem::swap(&mut width, &mut height);
            }
            Ok((sort, 1, [f64::from(width), f64::from(height)]))
        }
    }
}

/// The size of one page of a PDF as it is seen, in points, or why it has none.
fn page_size(pages: &[hayro::hayro_syntax::page::Page<'_>], page: usize) -> Result<(f32, f32), String> {
    let count = pages.len();
    if count == 0 {
        return Err("the file has no pages: it may be cut short or damaged".into());
    }
    if page == 0 || page > count {
        return Err(format!("the file has {count} page{}; page {page} was asked for", if count == 1 { "" } else { "s" }));
    }
    let p = &pages[page - 1];
    // A page whose own box has no size is drawn by the library as a sheet of A4. That
    // would be a size nobody gave: refused instead.
    let own = p.intersected_crop_box();
    if !(own.width() > 0.5 && own.height() > 0.5) {
        return Err(format!("page {page} gives no size of its own (its box is {} by {} points)", own.width(), own.height()));
    }
    let (width, height) = p.render_dimensions();
    if !(width > 0.0 && height > 0.0) {
        return Err(format!("page {page} has no size"));
    }
    Ok((width, height))
}

fn read_pdf(bytes: &[u8], page: usize) -> Result<Page, String> {
    let pdf = Pdf::new(bytes.to_vec()).map_err(|_| "the PDF could not be read: it is damaged, cut short, locked with a password, or not a PDF at all. Print it to a new PDF, or send it again".to_string())?;
    let pages = pdf.pages();
    let count = pages.len();
    let (width, height) = page_size(pages, page)?;
    let p = &pages[page - 1];
    let settings = InterpreterSettings::default();
    // The picture, at no more than LONG_SIDE dots on its longer side.
    let scale = LONG_SIDE as f32 / width.max(height);
    let cache = RenderCache::new();
    let pixmap = render(p, &cache, &settings, &RenderSettings::default(), &PixmapSettings { x_scale: scale, y_scale: scale, bg_color: WHITE });
    let picture = Picture { width: u32::from(pixmap.width()), height: u32::from(pixmap.height()), rgba: pixmap.data_as_u8_slice().to_vec() };
    // The lines: the page drawn again into a device that keeps its straight strokes.
    let mut lines = Lines { height: f64::from(height), width: f64::from(width), ..Lines::default() };
    let to_page = p.initial_transform(true).to_kurbo();
    let mut context = Context::new(to_page, Rect::new(0.0, 0.0, f64::from(width), f64::from(height)), &hayro::hayro_interpret::InterpreterCache::new(), p.xref(), settings);
    interpret_page(p, &mut context, &mut lines);
    let mut lost = Vec::new();
    if lines.kept.is_empty() && lines.pictures > 0 {
        lost.push(format!("it is a scan ({} picture{} and no drawn lines): there is nothing on it to snap to, and it is traced over by eye", lines.pictures, if lines.pictures == 1 { "" } else { "s" }));
    } else if lines.pictures > 0 {
        lost.push(format!("{} picture{} on the page {} shown and {} no lines to snap to", lines.pictures, if lines.pictures == 1 { "" } else { "s" }, if lines.pictures == 1 { "is" } else { "are" }, if lines.pictures == 1 { "has" } else { "have" }));
    }
    if lines.curves > 0 {
        lost.push(format!("{} curved piece{} of line {} known by {} ends only", lines.curves, if lines.curves == 1 { "" } else { "s" }, if lines.curves == 1 { "is" } else { "are" }, if lines.curves == 1 { "its" } else { "their" }));
    }
    if lines.words > 0 {
        lost.push("the words and figures on the page are drawn and not read: a dimension is typed by whoever reads it".into());
    }
    // A PDF ends with a mark. One that does not may have been cut short in the sending:
    // the library reads what is left of it, and what is left may not be all of the page.
    let tail = &bytes[bytes.len().saturating_sub(1024)..];
    if !tail.windows(5).any(|w| w == b"%%EOF") {
        lost.push("the file does not end as a PDF does: it may be cut short, and what is on the page may not be all there was".into());
    }
    let some = |n: usize, one: &str, many: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {many}") };
    if lines.short > 0 {
        lost.push(format!("{} not kept to snap to (hatching, arrowheads, ticks)", some(lines.short, "line under a point long is", "lines under a point long are")));
    }
    if lines.white > 0 {
        lost.push(format!("{} not ink, and not kept to snap to", some(lines.white, "line drawn in white or in no color is", "lines drawn in white or in no color are")));
    }
    if lines.dashed > 0 {
        lost.push(format!("{} kept whole: the gaps in it are places to snap to as well", some(lines.dashed, "dashed line is", "dashed lines are")));
    }
    if lines.cut > 0 || lines.hidden > 0 {
        lost.push(format!("part of the page is covered over: {} cut to what shows, {} left out", some(lines.cut, "line is", "lines are"), some(lines.hidden, "hidden line is", "hidden lines are")));
    }
    if lines.odd_clips > 0 {
        lost.push(format!("{} not a square: lines under {} are kept whole, hidden parts and all", some(lines.odd_clips, "cover on the page is", "covers on the page are"), if lines.odd_clips == 1 { "it" } else { "them" }));
    }
    if lines.covered > 0 {
        lost.push(format!("{} under a white box drawn over {} (part of the sheet blanked out) and {} not kept to snap to", some(lines.covered, "line is", "lines are"), if lines.covered == 1 { "it" } else { "them" }, if lines.covered == 1 { "is" } else { "are" }));
    }
    if lines.odd_white > 0 {
        lost.push(format!("{} drawn over lines and {} not a square: what {} is kept to snap to all the same", some(lines.odd_white, "white shape is", "white shapes are"), if lines.odd_white == 1 { "is" } else { "are" }, if lines.odd_white == 1 { "it covers" } else { "they cover" }));
    }
    if lines.off_page > 0 || lines.off_page_cut > 0 {
        lost.push(format!("{} drawn off the page and not kept; {} cut at its edge", some(lines.off_page, "line is", "lines are"), some(lines.off_page_cut, "line that runs off it is", "lines that run off it are")));
    }
    if lines.over > 0 {
        lost.push(format!("{} lines beyond the first {MOST_LINES} are not kept", lines.over));
    }
    Ok(Page { sort: Sort::Pdf, pages: count, size: [f64::from(width), f64::from(height)], picture, lines: lines.kept, lost })
}

// ---- The words on a page ----

/// One line of words on a PDF page, as it reads from left to right.
#[derive(Clone, Debug, PartialEq)]
pub struct Words {
    pub text: String,
    /// How tall its letters are, in points: the biggest line near the top of a
    /// manufacturer's sheet is the product's name.
    pub size: f64,
}

/// The letters a page draws, each where it is drawn.
#[derive(Default)]
struct Letters {
    /// The letter, where it starts (x to the right, y down), and how tall it is.
    all: Vec<(String, f64, f64, f64)>,
}

impl<'a> Device<'a> for Letters {
    fn draw_path(&mut self, _: &BezPath, _: DrawProps<'a>, _: &DrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'a>>, _: BlendMode) {}
    fn draw_glyph_run(&mut self, run: &GlyphRun<'_, 'a>, props: DrawProps<'a>, _: &DrawMode) {
        for glyph in run.glyphs() {
            let to_page = props.transform * glyph.transform();
            let at = to_page * Point::new(0.0, 0.0);
            // A letter's own space is a thousandth of its height to the unit.
            let c = to_page.as_coeffs();
            let tall = c[2].hypot(c[3]) * 1000.0;
            let said = match glyph.as_unicode() {
                Some(hayro_cmap::BfString::Char(c)) => c.to_string(),
                Some(hayro_cmap::BfString::String(s)) => s,
                None => continue,
            };
            self.all.push((said, at.x, at.y, tall));
        }
    }
    fn draw_image(&mut self, _: hayro::hayro_interpret::Image<'a, '_>, _: ImageDrawProps<'a>) {}
    fn pop_clip(&mut self) {}
    fn pop_transparency_group(&mut self) {}
}

/// The words on one page (counted from 1) of a PDF, line by line from the top, each line
/// from the left. A page that is a scan has letters in a picture and none to read: it
/// gives no lines, and that is for the caller to say. Only a PDF has words.
pub fn words(bytes: &[u8], page: usize) -> Result<Vec<Words>, String> {
    if sort_of(bytes) != Some(Sort::Pdf) {
        return Err("only a PDF has words that can be read; a picture of a sheet is read by eye".into());
    }
    let pdf = Pdf::new(bytes.to_vec()).map_err(|_| "the PDF could not be read: it is damaged, cut short, locked with a password, or not a PDF at all. Print it to a new PDF, or send it again".to_string())?;
    let pages = pdf.pages();
    let (width, height) = page_size(pages, page)?;
    let p = &pages[page - 1];
    let mut letters = Letters::default();
    let mut context = Context::new(p.initial_transform(true).to_kurbo(), Rect::new(0.0, 0.0, f64::from(width), f64::from(height)), &hayro::hayro_interpret::InterpreterCache::new(), p.xref(), InterpreterSettings::default());
    interpret_page(p, &mut context, &mut letters);
    // Letters on one line stand within a third of their height of each other.
    let mut all = letters.all;
    all.sort_by(|a, b| a.2.total_cmp(&b.2).then(a.1.total_cmp(&b.1)));
    let mut rows: Vec<Vec<(String, f64, f64, f64)>> = Vec::new();
    for letter in all {
        match rows.last_mut() {
            Some(row) if (letter.2 - row[0].2).abs() <= (row[0].3.max(letter.3) * 0.35).max(1.0) => row.push(letter),
            _ => rows.push(vec![letter]),
        }
    }
    let mut out = Vec::new();
    for mut row in rows {
        row.sort_by(|a, b| a.1.total_cmp(&b.1));
        let size = row.iter().map(|l| l.3).fold(0.0, f64::max);
        let mut text = String::new();
        let mut last: Option<f64> = None;
        for (said, x, _, tall) in &row {
            // How wide a letter is, is not given; no letter is wider than it is tall. So
            // a letter that starts more than its height and a fifth past the start of
            // the one before has a gap before it: a space the sheet did not draw. One
            // that starts several heights on is a column apart. Spaces the sheet does
            // draw come through as letters and need nothing from here.
            if let Some(before) = last {
                let step = x - before;
                if step > tall * 3.5 {
                    text.push_str("   ");
                } else if step > tall * 1.2 && !text.ends_with(' ') && said != " " {
                    text.push(' ');
                }
            }
            text.push_str(said);
            last = Some(*x);
        }
        let text = text.split(' ').filter(|w| !w.is_empty()).collect::<Vec<_>>().join(" ");
        if !text.is_empty() {
            out.push(Words { text, size });
        }
    }
    Ok(out)
}
