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
//! its two ends only; words are drawn and not read; a picture has no lines at all.

use hayro::hayro_interpret::font::GlyphRun;
use hayro::hayro_interpret::util::TransformExt;
use hayro::hayro_interpret::{interpret_page, BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, ImageDrawProps, InterpreterSettings, SoftMask};
use hayro::hayro_syntax::Pdf;
use hayro::kurbo::{BezPath, PathEl, Point, Rect};
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

/// Collects what a page draws: its straight strokes, and a count of what is not a line.
struct Lines {
    height: f64,
    kept: Vec<[[f64; 2]; 2]>,
    over: usize,
    curves: usize,
    pictures: usize,
    words: usize,
}

impl Lines {
    fn stroke(&mut self, path: &BezPath, to_page: hayro::kurbo::Affine) {
        let mut start: Option<Point> = None;
        let mut at: Option<Point> = None;
        let line = |a: Point, b: Point, me: &mut Lines| {
            let (a, b) = (to_page * a, to_page * b);
            if (b - a).hypot() < SHORTEST_LINE_PT || ![a.x, a.y, b.x, b.y].iter().all(|v| v.is_finite()) {
                return;
            }
            if me.kept.len() >= MOST_LINES {
                me.over += 1;
                return;
            }
            // The page is drawn from its top; lines are kept from its lower-left corner up.
            me.kept.push([[a.x, me.height - a.y], [b.x, me.height - b.y]]);
        };
        for el in path.elements() {
            match *el {
                PathEl::MoveTo(p) => {
                    start = Some(p);
                    at = Some(p);
                }
                PathEl::LineTo(p) => {
                    if let Some(a) = at {
                        line(a, p, self);
                    }
                    at = Some(p);
                }
                PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => {
                    self.curves += 1;
                    at = Some(p);
                }
                PathEl::ClosePath => {
                    if let (Some(a), Some(s)) = (at, start) {
                        line(a, s, self);
                    }
                    at = start;
                }
            }
        }
    }
}

impl<'a> Device<'a> for Lines {
    fn draw_path(&mut self, path: &BezPath, props: DrawProps<'a>, mode: &DrawMode) {
        // A line on a drawing is a stroke. A filled shape (a wall poche, a hatch block) is
        // not a line to snap to, though its outline is when it is stroked as well.
        if matches!(mode, DrawMode::Stroke(_) | DrawMode::FillAndStroke(..)) {
            self.stroke(path, props.transform);
        }
    }
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'a>>, _: BlendMode) {}
    fn draw_glyph_run(&mut self, _: &GlyphRun<'_, 'a>, _: DrawProps<'a>, _: &DrawMode) {
        self.words += 1;
    }
    fn draw_image(&mut self, _: hayro::hayro_interpret::Image<'a, '_>, _: ImageDrawProps<'a>) {
        self.pictures += 1;
    }
    fn pop_clip(&mut self) {}
    fn pop_transparency_group(&mut self) {}
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
            let decoded = image::load_from_memory(bytes).map_err(|e| format!("the picture could not be read ({e})"))?.to_rgba8();
            let (width, height) = decoded.dimensions();
            if width == 0 || height == 0 {
                return Err("the picture has no size".into());
            }
            Ok(Page {
                sort,
                pages: 1,
                size: [f64::from(width), f64::from(height)],
                picture: Picture { width, height, rgba: decoded.into_raw() },
                lines: Vec::new(),
                lost: vec!["it is a picture: it has no lines of its own to snap to, and is traced over by eye".into()],
            })
        }
    }
}

/// What a page is without drawing it: the sort of file, how many pages the file has, and
/// the page's size (a PDF's in points, a picture's in dots). The refusals are `read`'s.
pub fn size_of(bytes: &[u8], page: usize) -> Result<(Sort, usize, [f64; 2]), String> {
    match sort_of(bytes) {
        None => Err("it is not a PDF, a PNG or a JPEG".into()),
        Some(Sort::Pdf) => {
            let pdf = Pdf::new(bytes.to_vec()).map_err(|e| format!("the PDF could not be read ({e:?})"))?;
            let pages = pdf.pages();
            let count = pages.len();
            if page == 0 || page > count {
                return Err(format!("the file has {count} page{}; page {page} was asked for", if count == 1 { "" } else { "s" }));
            }
            let (width, height) = pages[page - 1].render_dimensions();
            if !(width > 0.0 && height > 0.0) {
                return Err(format!("page {page} has no size"));
            }
            Ok((Sort::Pdf, count, [f64::from(width), f64::from(height)]))
        }
        Some(sort) => {
            if page != 1 {
                return Err(format!("a picture has one page; page {page} was asked for"));
            }
            let (width, height) = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().map_err(|e| format!("the picture could not be read ({e})"))?.into_dimensions().map_err(|e| format!("the picture could not be read ({e})"))?;
            if width == 0 || height == 0 {
                return Err("the picture has no size".into());
            }
            Ok((sort, 1, [f64::from(width), f64::from(height)]))
        }
    }
}

fn read_pdf(bytes: &[u8], page: usize) -> Result<Page, String> {
    let pdf = Pdf::new(bytes.to_vec()).map_err(|e| format!("the PDF could not be read ({e:?})"))?;
    let pages = pdf.pages();
    let count = pages.len();
    if page == 0 || page > count {
        return Err(format!("the file has {count} page{}; page {page} was asked for", if count == 1 { "" } else { "s" }));
    }
    let p = &pages[page - 1];
    let (width, height) = p.render_dimensions();
    if !(width > 0.0 && height > 0.0) {
        return Err(format!("page {page} has no size"));
    }
    let settings = InterpreterSettings::default();
    // The picture, at no more than LONG_SIDE dots on its longer side.
    let scale = LONG_SIDE as f32 / width.max(height);
    let cache = RenderCache::new();
    let pixmap = render(p, &cache, &settings, &RenderSettings::default(), &PixmapSettings { x_scale: scale, y_scale: scale, bg_color: WHITE });
    let picture = Picture { width: u32::from(pixmap.width()), height: u32::from(pixmap.height()), rgba: pixmap.data_as_u8_slice().to_vec() };
    // The lines: the page drawn again into a device that keeps its straight strokes.
    let mut lines = Lines { height: f64::from(height), kept: Vec::new(), over: 0, curves: 0, pictures: 0, words: 0 };
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
    if lines.over > 0 {
        lost.push(format!("{} lines beyond the first {MOST_LINES} are not kept", lines.over));
    }
    Ok(Page { sort: Sort::Pdf, pages: count, size: [f64::from(width), f64::from(height)], picture, lines: lines.kept, lost })
}
