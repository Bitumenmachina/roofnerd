# W-228M report (notes as I go)

## Progress notes
- Sources read: buildingSMART BCF-XML repo files via raw.githubusercontent.com (release_2_1 and release_3_0 Schemas/*.xsd, Documentation README via fetch, LICENSE = CC BY-ND 4.0, so NOT committed).
- Red run 1 (stub export returning Err("not built")): 8 of 8 tests in crates/ifc/tests/call228m.rs FAILED, all "not built" (saved: review/W-228M-red1.txt).
- Green after implementation: 6/8, then 8/8. The two first failures were: (1) my test compared the IFC's name to the clash list's decorated name (15 of 122 things are said in the clash list with a size/type the file's name lacks, e.g. "Return grille, 20 x 20 (return grille, first floor)" vs file's 'return grille, first floor'): ids are the same, test now holds the file's name to the job's own name for the path; (2) my test's acos of 1.0000000002 gave NaN: clamped. Neither expectation of an existing test moved.
- Baseline before work (sample whole house): sixteen sha256 880d9fd3...82b (as required), clashes/vif/ifc copies in scratchpad/bcfw.YYMx (clashes.before, vif.before, h.ifc 8c5725c3..., h.json).
- Built: crates/ifc/src/bcf.rs (writer), write.rs (guid made pub + uuid + job_title, same output), world clash.rs (storey_of_pair, extent, Clash::depth_listed; rooms_of now uses storey_of_pair, CLI clash list uses depth_listed: output byte-same), cli main.rs (export bcf [--holes]), tools/bcf-check.py, demos/bcf.sh (all 9 steps pass), README rows, PORTMAP rows (3), research/bcf.md. Cargo: zip = 9.0 deflate added to ifc (dep) and cli (dev-dep), already in the lock (via rust_xlsxwriter): Cargo.lock gains only the two dependency edges.
- ifc tests call228m 8/8 green; cli tests call228m 4/4 green (CLI tests were written after the CLI arm, not red first: said plainly).

## Final (see the handed-back report, which is the same text)
Tests: world 734 (lib 81 + 49 test files; none new), command line (plumbline) 82 (78 + 4 new), sheets 7, ifc 119 (111 + 8 new), roofing 122, plans 207 = 1,271. 64 command-line demonstrations rc=0, 0 FAIL lines (demos/bcf.sh among them, rerun on the final code). Gates, port map (476 items), licences pass. Patch: review/call228M.patch (1423 lines).
