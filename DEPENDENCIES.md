# Dependencies

Everything shipped or built with, and its license. Permissive only: MIT, Apache-2.0, BSD,
ISC. Nothing copyleft in the shipped application.

## Shipped in the application

| Package | Version | License | What for |
|---|---|---|---|
| tauri | 2.x | MIT / Apache-2.0 | The desktop shell: windows, the held document, disk access |
| serde | 1.x | MIT / Apache-2.0 | Serializing the document |
| serde_json | 1.x | MIT / Apache-2.0 | The job's on-disk form; its BTreeMap backing gives sorted keys for free |
| @tauri-apps/api | 2.9.0 | MIT / Apache-2.0 | The window's side of the shell |
| tauri-plugin-dialog | 2.7.3 | MIT / Apache-2.0 | Asking the operating system for a drawing the estimator picked |
| @tauri-apps/plugin-dialog | 2.x | MIT / Apache-2.0 | The window's side of that |
| pdfjs-dist | 6.3.289 | Apache-2.0 | Painting a PDF page onto the trace surface. Worker bundled as a local file; nothing is fetched |
| three | 0.185.1 | MIT | The Model editor. Verified from the package's own LICENSE file, not a badge. Bundled by Vite as a local file — the view is derived geometry drawn on a canvas, and it reaches nothing |
| geo | 0.29.3 | MIT / Apache-2.0 | Polygon work in the core: area, contains, boolean ops for drainage regions and zone bands (C1 on). Its boolean backend `i_overlay` comes with it |
| cavalier_contours | 0.4.0 | MIT / Apache-2.0 | Polyline offset (C3 on) — a wall or curb's own inboard line and its coping's two edges, replacing the Model editor's retired per-segment hand offset; the zone bands are still to come |

**Where those two licences were read (D147).** Neither package ships a LICENSE file inside its
published crate: `geo-0.29.3` and `cavalier_contours-0.4.0` unpack to source, a README and a
`Cargo.toml` and nothing else. The licence above is the `license` field of each package's own
`Cargo.toml` as published, read off this machine at
`~/.cargo/registry/src/index.crates.io-<index>/<name>-<version>/Cargo.toml` — `"MIT OR Apache-2.0"`
in both. That is the publisher's own statement travelling with the code, which is what a LICENSE
file in a package would have been; the full texts live in each project's repository
(`github.com/georust/geo`, `github.com/jbuckmccready/cavalier_contours`). Recorded rather than
rounded off, because "read from the LICENSE file" was the instruction and there is no such file.

**The versions are what the toolchain floor picks.** `rust-version = "1.77"` — Tauri 2's, and this
workspace's — resolves to geo 0.29.3 and cavalier_contours 0.4.0; 0.33 and 0.9 are published and
want a newer compiler. Raising the floor is a decision about the shell, not about geometry, so it
is not taken here.

**What they bring.** 50 crates in the core's tree, every one permissive: 43 are MIT / Apache-2.0 or
plain MIT, and the rest are `byteorder` and `memchr` (Unlicense OR MIT), `earcutr` (ISC), `approx`
(Apache-2.0), `foldhash` (Zlib) and `unicode-ident` ((MIT OR Apache-2.0) AND Unicode-3.0). Nothing
copyleft, which is the rule at the top of this file. The heavy ones are `rayon` and `spade`, pulled
in by geo; nothing in this program calls them yet.

## Build only — not shipped

| Package | Version | License | What for |
|---|---|---|---|
| @tauri-apps/cli | 2.11.4 | MIT / Apache-2.0 | Builds and runs the application |
| typescript | 5.9.3 | Apache-2.0 | Types |
| vite | 8.2.2 | MIT | Bundles the front end to static files |
| puppeteer-core | 25.10.0 | Apache-2.0 | Drives the front end in headless Chrome while building. Uses the system Chrome; downloads no browser. NOT what a section's done-check runs on — see below |
| tauri-driver | 2.x (cargo) | MIT / Apache-2.0 | WebDriver in front of the release binary, so a done-check runs against the shipped runtime |
| WebKitWebDriver | system (webkit2gtk4.1) | LGPL — a separate executable, never linked or shipped | What `tauri-driver` drives. A build tool on the developer's machine, not part of the program |

## Deliberately not used

| Package | Why not |
|---|---|
| Any UI framework | The ported trace code is plain DOM. A framework would sit between a roofer-legible codebase and the screen. |
| Any HTTP client | The program makes no network calls. Not having the capability is the shortest way to keep that true. |
| A straight-skeleton library | The standard way to generate a roof's hips and valleys from a bare footprint — and this program never does that, because its facets are traced. What section 6 actually needed was the thickness at a point, which is the distance to the nearest drain, and a few drains in a loop answer that exactly. Licence would have blocked it anyway: CGAL's is GPL, `polyskel` LGPL, `ladybug-geometry-polyskel` AGPL, no permissive Rust crate exists, and the npm package labelled MIT ships a WebAssembly binary compiled from that GPL CGAL. Recorded so nobody evaluates it a second time. |
| A Voronoi library (`d3-delaunay`, `polygon-clipping`, `spade`) | Evaluated and permissive — ISC and MIT — and genuinely the right answer *if* a drainage basin ever has to be an exact polygon. The heightfield is sampled on a grid, so nearest-drain is a loop and the exact cell boundary is never needed. Adding it now would be a dependency carried for a feature nobody asked for. |
| A Tauri file-system plugin | Drawings are read through `read_page_source`, which resolves inside the open job and refuses anything outside it. A general file plugin would give a window the run of the machine for no gain. |
| webdriverio | It reshapes WebDriver capabilities on the way out, and `tauri-driver` rejected the result while the raw payload worked. The protocol is documented and small; `tools/probe/tauri-harness.mjs` speaks it directly. |

## The 3D toolkit (section 10b, added 2026-09-12 — Addendum 7 / Patrick's redirection: use what exists)

| Package | Version | Licence | Why |
|---|---|---|---|
| `@thatopen/components` | 3.4.8 | MIT | Worlds, `OrthoPerspectiveCamera` (orthographic presets + orbit), `Raycasters`, `Clipper`, `IfcLoader`, `FragmentsManager` — the maintained BIM toolkit on three.js, vanilla TypeScript |
| `@thatopen/components-front` | 3.4.4 | MIT | `Highlighter`, `ClipEdges` (section-cut linework), `LengthMeasurement` / `AreaMeasurement` |
| `@thatopen/fragments` | 3.4.7 | MIT | The geometry format the toolkit's tools key on; worker self-hosted |
| `web-ifc` | 0.0.77 | MPL-2.0 (linked, never modified) | Reads and writes IFC4; `web-ifc.wasm` (single-thread build) self-hosted from the bundle — no SharedArrayBuffer on WebKitGTK, no fetch |
| `camera-controls` | 3.1.2 | MIT | Peer of `@thatopen/components` |

Not taken: `@thatopen/ui` (our chrome is ours); the multi-thread wasm (needs cross-origin isolation the
webview does not give); xeokit (AGPL); Bonsai (GPL); ifc-lite (MPL, IFCX — watched, not used).
Runtime rule unchanged: nothing is fetched; `connect-src 'self'`; the egress gate stays red on any fetch.

**Self-hosted, and how (D142).** `packages/app/vite.config.ts` copies two files byte for byte out of
the installed packages into `dist/runtime/` at build time — `web-ifc.wasm` from `web-ifc` and
`worker.min.mjs` from `@thatopen/fragments`, renamed `fragments-worker.mjs`. Neither is modified;
web-ifc is MPL-2.0 and is linked, never touched. `packages/app/src/ifc/local-files.ts` is the one
place either path is written down, and the build reads the same file, so the path the program asks
for and the path the build writes cannot drift. The build stops if the built program has stopped
naming either of them. Both of the toolkit's own CDN paths are closed in
`packages/app/src/ifc/world.ts`: `IfcLoader.setup({ autoSetWasm: false, wasm: { path, absolute } })`
and `FragmentsManager.init(<local worker>)` rather than `FragmentsManager.getWorker()`.

Two strings of the libraries' own remain in the built bundle — the unpkg addresses inside
`autoSetWasm` and `getWorker`, neither of which this program calls. They are dead code that
`connect-src 'self'` would refuse anyway; the runtime check lists every address the window actually
fetched from, which is the claim that matters.

**Attribution (D145).** `SimpleRenderer` paints a That Open Company wordmark into the viewport by
default and the library asks that it be left on where it fits the design. It does not fit an
estimator's roof — a third-party mark on a drawing that goes to an owner reads as that owner's
consultant — so `showLogo` is off and the element is taken out of the page. The MIT notice travels
with the package, and the attribution is this table.

