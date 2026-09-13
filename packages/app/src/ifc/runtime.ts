// ── Booting web-ifc in the window ──────────────────────────────────────────
// `write.ts` takes an already-`Init()`ed `IfcAPI` on purpose — where the wasm
// came from is the caller's business, and it differs between a Node test
// (which reads it straight off `node_modules`) and this window, which has no
// `node_modules` and no network to fetch one from.
//
// D142 removed the plugin that used to copy `web-ifc.wasm` into the bundle for
// a fragments-based viewer that is gone; the writer still needs the module,
// and it gets it the plain way instead — imported as a Vite asset, which
// bundles it under a hashed name in `dist/assets` and hands back that URL. No
// `fetch` in this file: what fetches the bytes at that URL is web-ifc's own
// Emscripten loader, at runtime, from a same-origin path the policy already
// allows (`connect-src 'self'`, D142 and D143). Single-threaded — `web-ifc.wasm`,
// never `-mt` — because the multi-thread build wants `SharedArrayBuffer`,
// which wants cross-origin isolation, which this webview is not given.

import * as WEBIFC from 'web-ifc';
// eslint-disable-next-line import/no-unresolved -- a Vite asset URL, resolved at build time
import wasmUrl from 'web-ifc/web-ifc.wasm?url';

let cached: Promise<WEBIFC.IfcAPI> | null = null;

/**
 * An `IfcAPI`, initialised once and reused.
 *
 * `Init()` compiles the module, which is not free; a window that exports IFC
 * twice in one session should pay for that once. `forceSingleThread` is
 * belt-and-braces beside naming the single-thread file: the multi-thread path
 * probes for `SharedArrayBuffer` on its own and this window has none.
 */
export function ifcApi(): Promise<WEBIFC.IfcAPI> {
  if (!cached) {
    const api = new WEBIFC.IfcAPI();
    cached = api.Init((path: string, prefix: string) => (path.endsWith('.wasm') ? wasmUrl : prefix + path), true)
      .then(() => api);
  }
  return cached;
}
