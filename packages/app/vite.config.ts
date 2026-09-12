import { createRequire } from 'node:module';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { defineConfig, type Plugin } from 'vite';
import { RUNTIME_DIR, WASM_DIR, WASM_NAME, WORKER_NAME, WORKER_URL } from './src/ifc/local-files.js';

const require = createRequire(import.meta.url);

/**
 * Copy the two IFC runtime files into the bundle, out of the installed
 * packages.
 *
 * A plugin rather than `public/`, on purpose. `public/` would mean a second
 * copy of a 1.3 MB WebAssembly module and a 1.4 MB worker committed to this
 * repository beside the lockfile that already pins them — two copies that agree
 * on the day they are made and drift the first time the package moves, with
 * nothing to say they have. Resolving them through the installed package makes
 * the lockfile the only place a version is stated, and it lets the build refuse
 * rather than ship something wrong:
 *
 *   - the multi-thread module is refused by name. It needs `SharedArrayBuffer`,
 *     which needs cross-origin isolation, which this webview is not given.
 *   - if the built code has stopped naming the folder these land in, the build
 *     stops. That is the one way self-hosting fails quietly: the files are
 *     there, the program asks a CDN for them anyway, and nothing says so until
 *     somebody opens the Model on a job site with no signal.
 *
 * The files are copied byte for byte. web-ifc is MPL-2.0 — linked, never
 * modified — and the fragments worker is MIT; neither is touched on the way in.
 */
function ifcRuntimeFiles(): Plugin {
  const wasmFrom = require.resolve('web-ifc/web-ifc.wasm');
  // `@thatopen/fragments` names `./worker` in its exports map, which is the
  // unminified 3.2 MB build. The minified one beside it is the same code and
  // loads at every open of the Model, so it is preferred and the named entry
  // is the fallback — the exports map cannot be reached past its own names.
  const fragmentsDist = dirname(require.resolve('@thatopen/fragments'));
  const minified = join(fragmentsDist, 'Worker', 'worker.min.mjs');
  const workerFrom = existsSync(minified) ? minified : require.resolve('@thatopen/fragments/worker');
  const folder = RUNTIME_DIR.replace(/^\/+|\/+$/g, '');

  return {
    name: 'roofnerd:ifc-runtime-files',
    generateBundle() {
      if (/-mt\.wasm$/.test(wasmFrom)) {
        this.error(`refusing the multi-thread web-ifc module: ${wasmFrom}`);
      }
      this.emitFile({ type: 'asset', fileName: `${folder}/${WASM_NAME}`, source: readFileSync(wasmFrom) });
      this.emitFile({ type: 'asset', fileName: `${folder}/${WORKER_NAME}`, source: readFileSync(workerFrom) });
    },
    writeBundle(_options, bundle) {
      // What this can see, and what it cannot. It can see that the built program
      // still names both paths — which is what goes wrong when a tutorial gets
      // pasted in, because the CDN defaults are what the toolkit does when
      // nobody says otherwise, and they fail silently on a machine with signal.
      // It cannot see that the name reaches the call: a path named only by the
      // read-back helper would satisfy it. The teeth for that are in the
      // runtime check, which reads the loader's own setting back and lists every
      // address the window actually fetched from.
      const code = Object.values(bundle)
        .map((item) => (item.type === 'chunk' ? item.code : ''))
        .join('');
      // The whole literal, in any of the three ways a minifier can quote it.
      // Searching for the bare text instead would let the worker's own URL
      // stand in for the wasm folder, since one contains the other — which is
      // how a check ends up unable to fail.
      const named = (path: string) => [`"${path}"`, `'${path}'`, `\`${path}\``]
        .some((form) => code.includes(form));
      for (const path of [WASM_DIR, WORKER_URL]) {
        if (!named(path)) {
          this.error(
            `nothing in the built program asks for ${path} — the IFC runtime files would be `
            + 'fetched from a content delivery network instead of read out of the bundle',
          );
        }
      }
    },
  };
}

// The application is built to static files and served by the shell over its own
// protocol — there is no dev server. That is on purpose: it means the content
// security policy in tauri.conf.json is live while developing, not just in a
// release build, so "this program cannot reach the network" is something you can
// check today rather than a promise about later.
export default defineConfig({
  plugins: [ifcRuntimeFiles()],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2022',
    // One file per entry, no inline anything: the policy forbids inline script
    // and inline style, and that is the policy doing its job.
    cssCodeSplit: false,
  },
  clearScreen: false,
});
