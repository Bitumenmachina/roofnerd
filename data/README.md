# data/

Vendor data the solvers read off disk instead of a PDF.

- `profiles/` — tapered-insulation panel profiles (panel size, slopes, designations,
  thin/thick edge and fill thickness), read for the tapered-package solver (C2).
- `fastening/` — insulation fastening patterns (fastener count, layout, FM ratings),
  read for the fastening solver (C4).

Each file carries its own publisher, document id, fetched date and source, and is
transcribed from the manufacturer's own page images — product names only, no prices,
no client figures. `packages/engine/test/vendor-data.test.mjs` checks their shape
(unique designations, an unbroken taper, a positive fastener count, a well-formed
FM class) — it does not check that the numbers are right.

**This data is not authority.** The manufacturer's own document — the manual on
disk — is authority for fastening design, taper layout, or code compliance. These
files are solver input, transcribed and shape-checked, nothing more. Where a
figure was illegible on the source page it is `null`, and each file's own
`note` fields flag the handful of source typos carried verbatim or normalized
(a class code printed without its "1-", a dropped letter in a product name).
