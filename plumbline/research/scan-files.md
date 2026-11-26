# A scanner's own files: LAS, LAZ and E57 (call 230)

What the reader relies on, each with where it was read and the line quoted. Written 2026-11-26.
Nothing here was read from memory except where it says so.

## LAS (the reader is written here)

**Source read**: "LAS SPECIFICATION, VERSION 1.4 - R13, 15 July 2013", published by the American Society for
Photogrammetry & Remote Sensing, as a PDF copy on a state transport department's site
(`https://transportation.ky.gov/Highway-Design/Documents/ASPRS LAS SPEC 1.4 2013.pdf`; text extracted with
`pdftotext -layout`; the line numbers below are that text's). The society's own newer R15 address
(`https://www.asprs.org/wp-content/uploads/2019/07/LAS_1_4_r15.pdf`) returned a web page and not the PDF, so R15
itself was **not read**. The 1.2 text (`asprs.org/a/society/committees/lidar/Downloads/ASPRS_LAS 1_2.pdf`) was found by
search and not read. Permission to use: the copyright notice allows "unlimited use and distribution of this document, or
parts thereof, as a specification provided such use references ASPRS as the publisher" (lines 10 to 18).

| what is relied on | the line quoted |
|---|---|
| all little-endian | line 104: "All data are in little-endian format." |
| the header's fields, their sizes and order (so where each sits) | Table 3, lines 200 to 245: version major and minor, header size, offset to point data, number of variable length records, point data record format, record length, legacy count, the scale factors, the offsets, the 64-bit count |
| a point's place is a whole number times a scale plus an offset | lines 389 to 394: "take the point record X multiplied by the X scale factor, and then add the X offset. Xcoordinate = (Xrecord \* Xscale) + Xoffset" |
| the points are found by the offset in the header, not by the header's size | lines 471 to 473: "LAS file I/O software must use the 'Offset to point data' field in the Public Header Block to locate the starting position of the first Point Data Record" |
| eleven record formats, 0 to 10, each a record of one length | line 357: "LAS 1.4 defines types 0 through 10"; Tables 7 to 21 (the shortest record of each: 20, 28, 26, 34, 57, 63, 30, 36, 38, 59, 67 bytes) |
| a longer record than its format is "extra bytes" | lines 360 to 364: "If the specified size is larger than implied by the point format type ... the remaining bytes are user-specific 'extra bytes'" |
| LAS 1.4 count | lines 366 to 368 and 413 to 414: the legacy count is zero where the count passes 32 bits; "Number of point records ... must always be correctly populated, regardless of legacy mode intent" |
| the first twelve bytes of every format are x, y, z as long (4-byte signed) | Tables 7 and 15 (x, y, z "long, 4 bytes") |
| a withheld point | Table 8 (formats 0 to 5), line 563: bit 7 of the classification byte, "If set, this point should not be included in processing (synonymous with Deleted)"; Table 16 (formats 6 to 10), line 876: bit 2 of the flags byte, same words |
| variable length records: a 54-byte heading each (user id 16 bytes, record id, length after header 2 bytes) | Table 6, lines 436 to 441; extended records 60 bytes, 8-byte length, Table 22, lines 1108 to 1120 |
| **where a unit is said**: only in the coordinate system record, as GeoTIFF keys or as text; the header says none | lines 1148 to 1149 and 1161 to 1230: "The CRS is represented by either GeoTIFF or Well Know Text as indicated by the WKT Global Encoding bit"; the text record is user id "LASF_Projection", record id 2112; the key directory is record id 34735 (an array of unsigned shorts: a four-number heading, then four numbers to a key: id, where, count, value) |

**Unit codes** are the GeoTIFF specification's, not LAS's (read at `http://geotiff.maptools.org/spec/geotiff6.html`):
`ProjLinearUnitsGeoKey = 3076`, `GeogLinearUnitsGeoKey = 2052`, `VerticalUnitsGeoKey = 4099`, and "Linear_Meter = 9001,
Linear_Foot = 9002, Linear_Foot_US_Survey = 9003". `GTModelTypeGeoKey = 1024` with its codes is read at the same page (`geotiff6.html`, lines 33 and 115 to 118): "GeoTIFF defined
CS Model Type Codes: ModelTypeProjected = 1 /\* Projection Coordinate System \*/, ModelTypeGeographic = 2 /\* Geographic
latitude-longitude System \*/, ModelTypeGeocentric = 3". So a model type of 2 is degrees, from the source and not from memory. A text coordinate system is read for the length units it names (`UNIT["metre",1]`,
`UNIT["US survey foot",...]`) and for beginning `GEOGCS`/`GEOGCRS` (degrees); the text form's grammar is the OGC's
(cited at line 1154) and was **not read**, only what a made-up example of it looks like.

**Read**: LAS 1.1 to 1.4, record formats 0 to 10, scale and offset as above, extra bytes skipped and said, withheld points
left out and counted. **Refused by name**: any version but 1.1 to 1.4 ("it is LAS 2.0, which this does not read"; 1.0 is
refused too and told to be saved as 1.2 or later),
a record format over 10, a record shorter than its format needs, a scale of nothing, a file cut short (said with how many
points it says and how many it holds), and a coordinate system of degrees. Why these and not fewer: every record format
begins with the same three whole numbers, so reading all eleven costs nothing, and refusing a readable waveform file would
only send a person to a program to save it again; the waveform itself is not kept, and that is said. Why 1.1 to 1.3 are read on the 1.4 text alone: the 1.4 text says it keeps "backward compatibility with LAS 1.1 - LAS 1.3"
(lines 54 to 55; 134 to 140), with the legacy count and legacy fields kept in the first 227 bytes of its header; the 1.1, 1.2
and 1.3 documents themselves were **not read**, so a 1.1 to 1.3 file is read to the 1.4 text's layout of those fields. Why not
1.0: nothing read speaks of its header.

**Not confirmed from a source**: that real scanner-made LAS files follow the text in every writer (the text is the
standard; no real file was fetched, by the rule). The points of a file whose "number of points" disagrees with its size are
refused when short and read to the said count when long.

## LAZ (a crate is linked)

LAZ is LAS packed by LASzip, a format of its own. LASzip's own text (the LAZ specification draft at rapidlasso's address) was
**not reachable**: the address returned a page, not the document. What is relied on instead is the crate's own source
(`laz-0.14.0`, read in `~/.cargo/registry`): the packed mark is the high bit of the point record format byte
(`las-0.11.2/src/point/format.rs`: `IS_COMPRESSED_MASK: u8 = 0x80`, `point_format_id & 0x3f`); the packing is described by a
variable length record with user id "laszip encoded" and record id 22204 (`las-0.11.2/src/laz.rs` lines 18 and 19).

- **laz 0.14.0**: licence from its own `Cargo.toml`: `license = "Apache-2.0"`; `LICENSE.txt` and `COPYING` are the Apache
  License 2.0 text; its `CHANGELOG.md` line 126 says "Changed the license from LGPL to Apache 2.0 as LASZip (the projects
  laz-rs derives from)", so **older versions were LGPL and are not to be used**: `Cargo.toml` asks for `0.14`.
  Links `byteorder 1.5.0` (Unlicense OR MIT) and `num-traits 0.2.19` (MIT OR Apache-2.0); the optional `rayon` is off.
- **las 0.11.2** (MIT) was considered and is **not** linked: it only wraps laz and adds a whole LAS reader, which is
  written here from the text.
- Recommendation: **read** (linked).

## E57 (a crate is linked)

**Source read**: the standard itself is sold (ASTM E2807-11(2019)); what was read are the excerpts of its own text that a web
search returned (the publisher's sample copy at `cdn.standards.iteh.ai` and the Library of Congress format page
`loc.gov/preservation/digital/formats/fdd/fdd000563.shtml` were listed; **neither was opened**, only the excerpts the search
gave): "All quantities standardized in this specification are expressed in terms of SI units" (so lengths are meters,
always); "The coordinate system is right-handed" (and nothing says which way is up); a "rigid body transform" maps a scan's
local coordinates into the file's shared frame (a scan's pose). The standard's own pages for the pose and the point record
element names were **not read**; the crate documents them (`e57-0.11.14/src/record.rs`: "Cartesian X coordinate (in
meters)"; `pc_reader_simple.rs`: `apply_pose`, "Default setting is enabled"; `spherical_to_cartesian`, default enabled).

- **e57 0.11.14**: `license = "MIT"` in its `Cargo.toml`; `LICENSE` is the MIT text; `#![forbid(unsafe_code)]` (`src/lib.rs`
  line 15); links only `roxmltree 0.21.2` (MIT OR Apache-2.0) which links `memchr 2.8.3` (Unlicense OR MIT); the optional
  `crc32c` is off. Known limits it states: no point grouping, index packets ignored, no string attributes.
- Recommendation: **read** (linked). Every scan in a file is read and placed by its pose; points with no place are left
  out and counted; colour, intensity and pictures are not kept and that is said.
- Cargo.lock gains exactly three crates: e57, laz and roxmltree (memchr, byteorder and num-traits were already linked by others);
  `tools/licenses.sh` passes with them (every row of the table above is in its output).

## The made-up room

`demos/scan-files/make.py` writes the PLY and the LAS files; `made-up-scan-files` (an example of the plans crate, built only by `cargo build --example`, which
links the same two crates and writes with them) writes the packed and E57 copies of the same 9,474 points. The room is 12 ft
by 10 ft and 8 ft high, a point every quarter foot (0.0762 m, a whole number of tenths of a millimetre) on the floor, the
ceiling and the four walls, so a LAS with a scale of 0.0001 holds every point exactly, as whole numbers, and the PLY holds the
same numbers as text. **What this does not prove**: that the program reads a LAZ or an E57 a scanner's own program wrote. The
two files are written by the same crates that read them, and not one real scanner file was used, by the rule. A real file of
each, made safe, in `refs/` would settle it.

## BIMCraft

`grep -rln -i "\blas\b\|\blaz\b\|e57\|point_cloud\|pointcloud" crates` in BIMCraft lists `crates/core/src/cloud.rs` (a point
cloud reader for PTS, XYZ, PLY, "ASPRS LAS 1.2 to 1.4, uncompressed" and E57), `crates/core/src/cloud/e57.rs` (its own E57
reader, 961 lines, from the public standard), `crates/core/src/cloud_tests.rs`, and the window and command code that uses
them (`crates/ui-egui/src/clouds.rs` and others). Its `cloud.rs` says of LAZ: "LAZ is compressed LAS, which BIMCraft doesn't
read: decompress it to LAS first". Nothing of it was copied: the LAS reader here was written from the specification's text
above, and was compared to BIMCraft's only afterward (its record lengths table agrees, and it refuses a packed file; ours
reads one). BIMCraft is MIT OR Apache-2.0, so its E57 reader could have been ported in place of the crate; the crate was
chosen because it is maintained against real files by its own author and BIMCraft's is not tested against any.
