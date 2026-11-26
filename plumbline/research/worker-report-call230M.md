# W-230M report (notes as I go)

## Before any code

1. LAS from its source: ASPRS "LAS Specification Version 1.4 - R13" (15 July 2013), the publisher's own PDF, read from a copy on a state transport department's site (https://transportation.ky.gov/Highway-Design/Documents/ASPRS LAS SPEC 1.4 2013.pdf); asprs.org's own R15 link returned a web page, not the PDF, so R15 itself was not read. GeoTIFF key values from http://geotiff.maptools.org/spec/geotiff6.html. Quotes with line numbers go in research/scan-files.md.
2. LAZ and E57 crates: e57 0.11.14 (MIT, pure Rust, forbid(unsafe), links roxmltree MIT/Apache-2.0 -> memchr Unlicense/MIT); laz 0.14.0 (Apache-2.0 by its own manifest and LICENSE.txt; CHANGELOG line 126 says it was LGPL in older versions and was changed to Apache 2.0; links byteorder Unlicense/MIT, num-traits MIT/Apache-2.0). `las` 0.11.2 (MIT) considered, not needed (it only wraps laz). Details below once tools/licenses.sh has run.
- RED 1...
- RED 2 (stubs only, every function answering 'not built' / Silent / None): 22 of 22 new tests FAIL at their assertions, 0 pass (details below)
- code written; scan_files 22 pass
- RED 1 (before any code, tests written first): `cargo test -p plumbline-plans --test scan_files` did not compile: 10 errors (unresolved imports file_notes, file_says, kind_of, read, units_to_use, FileSays, Kind; no field file_said on Brought).
- RED 2 (stub functions only: read -> Err("not built"), kind_of -> None, file_says -> Silent): 22 passed 0, 22 FAILED at their assertions (the helper code that writes a LAZ with the laz crate and an E57 with the e57 crate ran to the assertions, so those helpers work).
- Code written. GREEN: 22 passed. Three tests were wrong in my own setup (a message wording, a room kept in feet read as meters, a 175 vs 176 count from a whole-foot wall), fixed in the tests, no program behaviour bent; see 'what is not right'.
- demos/scan-files.sh: PASS (first full run) with PLUMBLINE_BIN set; two things the demonstration found in the program: (1) a map-grid LAS (512000 m east) failed with 'place on the plan: is not a usable point' (a job's places stop at 1,000,000 ft) -> bring_in now takes the southwest corner of a non-PLY scan to zero when it is over 1,000 ft out, and the CLI says how far it was; (2) a LAS that says 0 points came out as 'it has no points' from deep in -> refused at the reader. Added test a_las_that_says_it_holds_no_points_is_refused AFTER the code (not red first); proved by breaking below.
- MUTATION (offset dropped, 0-point check off, withheld check off, far-from-zero off): 6 tests failed (a_las_point..., a_las_that_says_it_holds_no_points..., a_laz_is_read..., a_scan_in_a_map_grid..., bringing_a_las_in..., points_the_file_marks_withheld...); restored, 24 pass.

## Final (see the handback message for the whole text)
Tests: plans 231 (lib 161, 7 files, scan_files 24 new), cli 80, sheets 7, world 740, ifc 131, roofing 122: none failed.
Demonstrations: 66 command-line scripts (65 earlier + scan-files.sh), all rc=0, no FAIL in any output.
Gates (language, egress, clientdata) PASS; portmap PASS (477 items, unchanged); licenses.sh exit 0.
Patch: SCRATCH/review/call230M.patch (15 files, 1685 insertions, 24 deletions).

## Round 2
1. GTModelTypeGeoKey codes now read at their source: http://geotiff.maptools.org/spec/geotiff6.html lines 115-118 ("ModelTypeProjected = 1", "ModelTypeGeographic = 2", "ModelTypeGeocentric = 3"); quoted in research/scan-files.md. The existing degrees test holds the behaviour.
2. A scan taken to zero says so in the sheet's source: "... (taken as meters with z up; its southwest corner taken to zero: it lay 512,000 m east and 4,200,000 m north in the file's own grid)". Test a_scan_in_a_map_grid... asserts the text; demo step 4b checks `drawings` says it.
3. The maker of the made-up LAZ and E57 is now crates/plans/examples/made-up-scan-files.rs (an example, not a command); demos/scan-files.sh builds it with cargo only when target/debug/examples/made-up-scan-files is missing (it was: the demo built it). crates/plans/src/bin is gone.
Rerun: plans lib 161 + files 4,15,12,7,8,24 = 231 pass; CLI 85 pass (I mis-added 80 earlier; the right sum of those 13 files is 85); scan-files, scan, findwalls demos PASS no FAIL; gates, licences exit 0, portmap pass. Patch rewritten.
