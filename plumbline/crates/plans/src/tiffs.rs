//! A TIFF read as a picture of a plan (call 227 point 8): one page of the file, as dots.
//!
//! The reader is the `tiff` crate (the image-rs project's, MIT), linked for this; `image` hands out the first page of a
//! file only and a scanned set is often one file of many pages. What it reads is the packings a scanner or a plotter's
//! program writes: none, LZW, deflate, PackBits, CCITT group 4 and JPEG in strips. A packing it does not read is named,
//! never guessed at. Dots held as gray, colour, colour with a see-through channel or a palette, at 1, 2, 4, 8 or 16 to
//! the sample, are read; anything else (CMYK, Lab, more than four samples, colours kept in separate planes) is named
//! as what it is and refused. The file's own note of which way up it is seen (tag 274) is applied, as it is for a
//! camera's picture.

use std::io::Cursor;

use tiff::decoder::{Decoder, DecodingResult, Limits};
use tiff::tags::{CompressionMethod, Tag};
use tiff::{ColorType, TiffError, TiffUnsupportedError};

/// Is this how a TIFF begins? (The two byte orders, and the "big" form of the format.)
pub fn is_tiff(bytes: &[u8]) -> bool {
    matches!(bytes.get(..4), Some([b'I', b'I', 0x2A | 0x2B, 0x00]) | Some([b'M', b'M', 0x00, 0x2A | 0x2B]))
}

const DAMAGED: &str = "the TIFF could not be read: it is damaged, cut short, or not the kind of picture its name says. Save it again as a TIFF, a PNG or a JPEG";
const WHAT_TO_DO: &str = "save it again as an uncompressed, LZW, deflate, PackBits or CCITT group 4 TIFF (or as a PNG)";

/// What a packing is called by those who write it; the number where nobody here has a name for it.
fn packing_said(m: CompressionMethod) -> String {
    match m {
        CompressionMethod::Huffman => "CCITT run-length (Huffman)".into(),
        CompressionMethod::Fax3 => "CCITT group 3 fax".into(),
        CompressionMethod::JPEG => "old-style JPEG".into(),
        CompressionMethod::ZSTD => "Zstandard".into(),
        CompressionMethod::WebP => "WebP".into(),
        other => format!("one this program has no name for (packing number {})", other.to_u16()),
    }
}

fn refused(why: &TiffError) -> String {
    match why {
        TiffError::UnsupportedError(TiffUnsupportedError::UnsupportedCompressionMethod(m)) => format!("this TIFF is packed as {}, which is not read: {WHAT_TO_DO}", packing_said(*m)),
        TiffError::UnsupportedError(TiffUnsupportedError::UnsupportedColorType(c)) => format!("this TIFF holds its colours as {}, which is not read: {WHAT_TO_DO}", colour_said(*c)),
        TiffError::UnsupportedError(_) => format!("this TIFF is made in a way that is not read: {WHAT_TO_DO}"),
        TiffError::LimitsExceeded => "this TIFF is larger than this program opens: save it smaller".into(),
        _ => DAMAGED.into(),
    }
}

fn colour_said(c: ColorType) -> String {
    match c {
        ColorType::CMYK(_) | ColorType::CMYKA(_) => "ink colours (CMYK)".into(),
        ColorType::YCbCr(_) => "luminance and chroma (YCbCr)".into(),
        ColorType::Lab(_) => "Lab colours".into(),
        ColorType::Multiband { num_samples, .. } => format!("{num_samples} values to a dot"),
        _ => "a kind of dot this program does not read".into(),
    }
}

fn limits() -> Limits {
    let mut l = Limits::unlimited();
    l.decoding_buffer_size = 2 << 30;
    l.ifd_value_size = 16 << 20;
    l.intermediate_buffer_size = 256 << 20;
    l
}

fn decoder(bytes: &[u8]) -> Result<Decoder<Cursor<&[u8]>>, String> {
    Decoder::new(Cursor::new(bytes)).map(|d| d.with_limits(limits())).map_err(|_| DAMAGED.to_string())
}

/// How many pages the file has: its directories, counted.
fn count(d: &mut Decoder<Cursor<&[u8]>>) -> Result<usize, String> {
    let mut n = 1;
    while d.more_images() {
        d.next_image().map_err(|_| DAMAGED.to_string())?;
        n += 1;
    }
    Ok(n)
}

fn no_such_page(pages: usize, page: usize) -> String {
    format!("the file has {pages} page{}; page {page} was asked for", if pages == 1 { "" } else { "s" })
}

/// A page's size in dots as it is seen (turned the way the file's own note says), and how many pages the file has.
pub fn size_of(bytes: &[u8], page: usize) -> Result<(usize, [u32; 2]), String> {
    let mut d = decoder(bytes)?;
    let pages = count(&mut d)?;
    if page == 0 || page > pages {
        return Err(no_such_page(pages, page));
    }
    d.seek_to_image(page - 1).map_err(|_| DAMAGED.to_string())?;
    let (w, h) = d.dimensions().map_err(|_| DAMAGED.to_string())?;
    let (w, h) = if turns_on_its_side(&mut d) { (h, w) } else { (w, h) };
    Ok((pages, [w, h]))
}

fn orientation(d: &mut Decoder<Cursor<&[u8]>>) -> image::metadata::Orientation {
    d.find_tag_unsigned::<u8>(Tag::Orientation).ok().flatten().and_then(image::metadata::Orientation::from_exif).unwrap_or(image::metadata::Orientation::NoTransforms)
}

fn turns_on_its_side(d: &mut Decoder<Cursor<&[u8]>>) -> bool {
    use image::metadata::Orientation as O;
    matches!(orientation(d), O::Rotate90 | O::Rotate270 | O::Rotate90FlipH | O::Rotate270FlipH)
}

/// A page as dots, turned the way it is seen, with whether it was turned. `most_dots` is the most dots a picture may have.
pub fn opened(bytes: &[u8], page: usize, most_dots: u64) -> Result<(image::DynamicImage, bool, usize), String> {
    let mut d = decoder(bytes)?;
    let pages = count(&mut d)?;
    if page == 0 || page > pages {
        return Err(no_such_page(pages, page));
    }
    d.seek_to_image(page - 1).map_err(|_| DAMAGED.to_string())?;
    let (w, h) = d.dimensions().map_err(|_| DAMAGED.to_string())?;
    if w == 0 || h == 0 {
        return Err("the picture has no size".into());
    }
    if u64::from(w) * u64::from(h) > most_dots {
        return Err(format!("the picture is {w} by {h} dots, more than this program opens ({} million): save it smaller", most_dots / 1_000_000));
    }
    let turn = orientation(&mut d);
    let colour = d.colortype().map_err(|e| refused(&e))?;
    if d.find_tag_unsigned::<u16>(Tag::PlanarConfiguration).ok().flatten() == Some(2) && !matches!(colour, ColorType::Gray(_)) {
        return Err(format!("this TIFF keeps each colour in a plane of its own, which is not read: {WHAT_TO_DO}"));
    }
    let palette = match colour {
        ColorType::Palette(_) => d.get_tag_u16_vec(Tag::ColorMap).ok(),
        _ => None,
    };
    let data = d.read_image().map_err(|e| refused(&e))?;
    let rgba = to_rgba(data, colour, w as usize, h as usize, palette.as_deref())?;
    let mut picture = image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(w, h, rgba).ok_or_else(|| DAMAGED.to_string())?);
    picture.apply_orientation(turn);
    Ok((picture, turn != image::metadata::Orientation::NoTransforms, pages))
}

/// The dots as four bytes each (red, green, blue, opacity).
fn to_rgba(data: DecodingResult, colour: ColorType, w: usize, h: usize, palette: Option<&[u16]>) -> Result<Vec<u8>, String> {
    let odd = || format!("this TIFF holds its colours as {}, which is not read: {WHAT_TO_DO}", colour_said(colour));
    // Every sample as 0 to 255.
    let (samples, bits): (Vec<u16>, u8) = match (&data, colour) {
        (DecodingResult::U8(v), ColorType::Gray(b) | ColorType::Palette(b)) if b < 8 => {
            // Rows are packed, most significant bit first, each row padded to a whole byte.
            let per = 8 / usize::from(b);
            let stride = (w + per - 1) / per;
            let mut out = Vec::with_capacity(w * h);
            for row in 0..h {
                let r = v.get(row * stride..(row + 1) * stride).ok_or_else(|| DAMAGED.to_string())?;
                for x in 0..w {
                    let byte = r[x / per];
                    let shift = 8 - usize::from(b) * (x % per + 1);
                    out.push(u16::from((byte >> shift) & ((1u8 << b) - 1)));
                }
            }
            (out, b)
        }
        (DecodingResult::U8(v), _) => (v.iter().map(|x| u16::from(*x)).collect(), 8),
        (DecodingResult::U16(v), _) => (v.clone(), 16),
        _ => return Err(odd()),
    };
    let scale = |s: u16, bits: u8| -> u8 {
        let max = (1u32 << bits) - 1;
        ((u32::from(s) * 255 + max / 2) / max) as u8
    };
    let mut out = Vec::with_capacity(w * h * 4);
    match colour {
        ColorType::Gray(b) => {
            for s in samples.iter().take(w * h) {
                let g = scale(*s, b.min(bits).max(1));
                out.extend_from_slice(&[g, g, g, 255]);
            }
        }
        ColorType::GrayA(_) => {
            for p in samples.chunks(2).take(w * h) {
                let g = scale(p[0], bits);
                let full = ((1u32 << bits) - 1) as u16;
                out.extend_from_slice(&[g, g, g, scale(p.get(1).copied().unwrap_or(full), bits)]);
            }
        }
        ColorType::RGB(_) => {
            for p in samples.chunks(3).take(w * h) {
                out.extend_from_slice(&[scale(p[0], bits), scale(p[1], bits), scale(p[2], bits), 255]);
            }
        }
        ColorType::RGBA(_) => {
            for p in samples.chunks(4).take(w * h) {
                out.extend_from_slice(&[scale(p[0], bits), scale(p[1], bits), scale(p[2], bits), scale(p[3], bits)]);
            }
        }
        ColorType::Palette(b) => {
            let map = palette.ok_or_else(|| DAMAGED.to_string())?;
            let n = 1usize << b.min(16);
            if map.len() < n * 3 {
                return Err(DAMAGED.to_string());
            }
            for s in samples.iter().take(w * h) {
                let i = usize::from(*s).min(n - 1);
                // A palette holds each colour as 16 bits, all the reds, then the greens, then the blues.
                out.extend_from_slice(&[(map[i] >> 8) as u8, (map[n + i] >> 8) as u8, (map[2 * n + i] >> 8) as u8, 255]);
            }
        }
        _ => return Err(odd()),
    }
    if out.len() != w * h * 4 {
        return Err(DAMAGED.to_string());
    }
    Ok(out)
}
