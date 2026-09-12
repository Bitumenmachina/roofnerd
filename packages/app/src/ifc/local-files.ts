// ── The IFC runtime files, and where they live in the program ──────────────
// Two files arrive with the 3D toolkit and are needed at run time rather than
// at build time: the web-ifc WebAssembly module, which reads and writes IFC,
// and the fragments worker, which turns a model into drawable geometry off the
// main thread.
//
// Every tutorial for this toolkit loads both from a CDN — `unpkg.com` — and
// both libraries will do it on their own if asked: `IfcLoader.setup()` fetches
// `unpkg.com/@thatopen/components/package.json` when `autoSetWasm` is left at
// its default of true, and `FragmentsManager.getWorker()` fetches the worker
// from unpkg and hands back a blob URL. Neither is used here. This program
// makes no network call, the content security policy keeps `connect-src` at
// `'self'`, and a roof that cannot be stood up on a plane with no signal is not
// a roof this trade can work with.
//
// So the build copies both files out of the installed packages and into the
// bundle (`packages/app/vite.config.ts`), and this is the one place that says
// where they land. The build reads these same constants, so the path the
// program asks for and the path the build writes cannot drift apart, and the
// plugin stops the build if the built program has stopped naming either of
// them. Both are written out in full rather than joined from the folder and the
// file name, so that what the build looks for in the built code is the whole
// URL and not a fragment of one.

/** The folder inside the bundle both files are written to, as a URL path. */
export const RUNTIME_DIR = '/runtime/';

/**
 * The single-thread web-ifc module. Never `web-ifc-mt.wasm`: the multi-thread
 * build needs `SharedArrayBuffer`, which needs cross-origin isolation, which
 * this webview is not given and should not be.
 */
export const WASM_NAME = 'web-ifc.wasm';

/** The fragments worker, copied whole out of `@thatopen/fragments`. */
export const WORKER_NAME = 'fragments-worker.mjs';

/**
 * What `IfcLoader.setup({ wasm })` and `IfcAPI.SetWasmPath` are given. Both
 * append the file name to it, so it is a folder and it ends in a slash. It is
 * absolute — resolved against the program's own origin, never against whatever
 * page happens to be loading.
 */
export const WASM_DIR = '/runtime/';

/** What `FragmentsManager.init` is given. */
export const WORKER_URL = '/runtime/fragments-worker.mjs';
