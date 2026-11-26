//! A TIFF comes in as a plan the way a PNG does (call 227 point 8). Every picture here is made up and
//! written by the test with the format's own encoder; nothing is a real sheet.

use plumbline_plans::{read, size_of, sort_of, Sort};
use tiff::encoder::{colortype, Compression, DeflateLevel, TiffEncoder};

/// A made-up picture: `w` by `h` dots, red across, green down, blue by a third.
fn rgb(w: u32, h: u32) -> Vec<u8> {
    let mut v = Vec::new();
    for y in 0..h {
        for x in 0..w {
            v.extend_from_slice(&[(x * 255 / w.max(1)) as u8, (y * 255 / h.max(1)) as u8, ((x + y) % 3 * 100) as u8]);
        }
    }
    v
}

fn tiff_of(pages: &[(u32, u32)], how: Compression) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut e = TiffEncoder::new(&mut out).unwrap().with_compression(how);
        for (w, h) in pages {
            e.write_image::<colortype::RGB8>(*w, *h, &rgb(*w, *h)).unwrap();
        }
    }
    out.into_inner()
}

/// The same file with the packing its first picture says it is packed with set to `method` (the tag is 259; its
/// value is a short in the file's own order). The picture's bytes are left as they were: only the name is changed.
fn packed_as(mut bytes: Vec<u8>, method: u16) -> Vec<u8> {
    let little = &bytes[..2] == b"II";
    let at = |b: &[u8], i: usize| if little { u16::from_le_bytes([b[i], b[i + 1]]) } else { u16::from_be_bytes([b[i], b[i + 1]]) };
    let u32at = |b: &[u8], i: usize| if little { u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) } else { u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) };
    let ifd = u32at(&bytes, 4) as usize;
    let n = at(&bytes, ifd) as usize;
    for k in 0..n {
        let e = ifd + 2 + k * 12;
        if at(&bytes, e) == 259 {
            let v = if little { method.to_le_bytes() } else { method.to_be_bytes() };
            bytes[e + 8] = v[0];
            bytes[e + 9] = v[1];
            return bytes;
        }
    }
    panic!("no packing tag");
}

#[test]
fn a_tiff_is_known_by_how_it_begins_and_said_as_a_picture() {
    let bytes = tiff_of(&[(8, 6)], Compression::Uncompressed);
    assert_eq!(sort_of(&bytes), Some(Sort::Tiff));
    assert_eq!(Sort::Tiff.said(), "TIFF picture");
    // The big form of the format, and the other byte order, are TIFFs too.
    assert_eq!(sort_of(b"MM\x00\x2a\x00\x00\x00\x08"), Some(Sort::Tiff));
    assert_eq!(sort_of(b"II\x2b\x00\x08\x00\x00\x00"), Some(Sort::Tiff));
}

#[test]
fn a_tiff_page_is_a_picture_with_its_size_in_dots_and_no_lines() {
    let bytes = tiff_of(&[(8, 6)], Compression::Uncompressed);
    let page = read(&bytes, 1).unwrap();
    assert_eq!((page.sort, page.pages, page.size), (Sort::Tiff, 1, [8.0, 6.0]));
    assert_eq!((page.picture.width, page.picture.height), (8, 6));
    assert!(page.lines.is_empty());
    assert!(page.lost[0].contains("it is a picture: it has no lines of its own to snap to"), "{:?}", page.lost);
    // The dots are the picture's own: the first is (0, 0, 0), the last of the first row is red at 7/8 of full: 7 x 255 / 8 = 223 (my first hand figure, 222, was wrong).
    assert_eq!(&page.picture.rgba[..4], &[0, 0, 0, 255]);
    assert_eq!(&page.picture.rgba[7 * 4..7 * 4 + 4], &[223, 0, 100 * ((7 + 0) % 3) as u8, 255]);
}

#[test]
fn every_packing_the_reader_reads_gives_the_same_dots() {
    let plain = read(&tiff_of(&[(40, 30)], Compression::Uncompressed), 1).unwrap();
    for (name, how) in [("LZW", Compression::Lzw), ("deflate", Compression::Deflate(DeflateLevel::Balanced)), ("PackBits", Compression::Packbits)] {
        let packed = read(&tiff_of(&[(40, 30)], how), 1).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(packed.picture, plain.picture, "{name}");
        assert_eq!(packed.size, plain.size, "{name}");
    }
}

#[test]
fn a_tiff_of_many_pages_gives_each_by_its_number_and_refuses_the_one_that_is_not_there() {
    let bytes = tiff_of(&[(8, 6), (12, 9), (5, 5)], Compression::Uncompressed);
    for (n, size) in [(1, [8.0, 6.0]), (2, [12.0, 9.0]), (3, [5.0, 5.0])] {
        let page = read(&bytes, n).unwrap();
        assert_eq!((page.pages, page.size), (3, size), "page {n}");
        assert_eq!(size_of(&bytes, n).unwrap(), (Sort::Tiff, 3, size), "page {n}");
    }
    // Page 2 is page 2's dots, not page 1's.
    assert_eq!(read(&bytes, 2).unwrap().picture.rgba, {
        let mut v = Vec::new();
        for c in rgb(12, 9).chunks(3) {
            v.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
        v
    });
    let none = read(&bytes, 4).unwrap_err();
    assert_eq!(none, "the file has 3 pages; page 4 was asked for");
    assert_eq!(read(&bytes, 0).unwrap_err(), "the file has 3 pages; page 0 was asked for");
    assert_eq!(size_of(&bytes, 4).unwrap_err(), "the file has 3 pages; page 4 was asked for");
    let one = tiff_of(&[(8, 6)], Compression::Uncompressed);
    assert_eq!(read(&one, 2).unwrap_err(), "the file has 1 page; page 2 was asked for");
}

#[test]
fn a_picture_that_is_a_png_and_a_tiff_of_the_same_dots_is_the_same_plan() {
    // The same 30 by 20 dots as a PNG and as a TIFF: the same size, the same dots.
    let dots = rgb(30, 20);
    let mut rgba = Vec::new();
    for c in dots.chunks(3) {
        rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
    }
    let png = plumbline_plans::png(&plumbline_plans::Picture { width: 30, height: 20, rgba }).unwrap();
    let a = read(&png, 1).unwrap();
    let b = read(&tiff_of(&[(30, 20)], Compression::Lzw), 1).unwrap();
    assert_eq!(a.size, b.size);
    assert_eq!(a.picture, b.picture);
}

#[test]
fn a_packing_the_reader_does_not_read_is_refused_by_its_name_and_says_what_to_do() {
    let base = tiff_of(&[(8, 6)], Compression::Uncompressed);
    for (method, name) in [(6u16, "old-style JPEG"), (3, "CCITT group 3 fax"), (2, "CCITT run-length (Huffman)"), (0x8005 + 1, "")] {
        if name.is_empty() {
            continue;
        }
        let bad = packed_as(base.clone(), method);
        let why = read(&bad, 1).unwrap_err();
        assert!(why.contains(name), "{method}: {why}");
        assert!(why.contains("is not read"), "{why}");
        assert!(why.contains("save it again as an uncompressed, LZW, deflate, PackBits or CCITT group 4 TIFF"), "{why}");
        // A number of the packing, in a builder's words, never the library's own text.
        assert!(!why.contains("Unsupported") && !why.contains("unsupported") && !why.contains("Err"), "{why}");
        assert_eq!(size_of(&bad, 1).map(|(_, n, s)| (n, s)), Ok((1, [8.0, 6.0])), "its size is known without drawing it");
    }
    // A packing nobody has a name for is said by its number.
    let odd = packed_as(base, 9999);
    let why = read(&odd, 1).unwrap_err();
    assert!(why.contains("packing number 9999") && why.contains("is not read"), "{why}");
}

#[test]
fn a_tiff_cut_short_or_damaged_is_said_so() {
    let bytes = tiff_of(&[(8, 6)], Compression::Uncompressed);
    let why = read(&bytes[..bytes.len() / 2], 1).unwrap_err();
    assert!(why.contains("the TIFF could not be read"), "{why}");
    let why = read(b"II*\x00\x08\x00\x00\x00", 1).unwrap_err();
    assert!(why.contains("the TIFF could not be read"), "{why}");
}

#[test]
fn gray_and_see_through_dots_and_a_file_turned_on_its_side_are_read() {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut e = TiffEncoder::new(&mut out).unwrap();
        e.write_image::<colortype::Gray8>(2, 2, &[0, 85, 170, 255]).unwrap();
        e.write_image::<colortype::Gray16>(2, 1, &[0, 65535]).unwrap();
        e.write_image::<colortype::RGBA8>(1, 1, &[10, 20, 30, 128]).unwrap();
        // 4 wide, 2 tall, turned a quarter turn by the file's own note (tag 274 = 6): seen 2 wide and 4 tall.
        let mut img = e.new_image::<colortype::Gray8>(4, 2).unwrap();
        img.encoder().write_tag(tiff::tags::Tag::Orientation, 6u16).unwrap();
        img.write_data(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    }
    let bytes = out.into_inner();
    assert_eq!(read(&bytes, 1).unwrap().picture.rgba, [0, 0, 0, 255, 85, 85, 85, 255, 170, 170, 170, 255, 255, 255, 255, 255]);
    assert_eq!(read(&bytes, 2).unwrap().picture.rgba, [0, 0, 0, 255, 255, 255, 255, 255]);
    assert_eq!(read(&bytes, 3).unwrap().picture.rgba, [10, 20, 30, 128]);
    let turned = read(&bytes, 4).unwrap();
    assert_eq!(turned.size, [2.0, 4.0]);
    assert_eq!(size_of(&bytes, 4).unwrap(), (Sort::Tiff, 4, [2.0, 4.0]));
    assert!(turned.lost.iter().any(|l| l.contains("which way up it is seen")), "{:?}", turned.lost);
}
