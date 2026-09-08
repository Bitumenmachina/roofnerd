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
