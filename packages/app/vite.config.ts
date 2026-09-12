// ── Building the window ────────────────────────────────────────────────────
// The application is built to static files and served by the shell over its own
// protocol — there is no dev server. That is on purpose: it means the content
// security policy in tauri.conf.json is live while developing, not just in a
// release build, so "this program cannot reach the network" is something you can
// check today rather than a promise about later.
//
// It used to carry a plugin that copied two IFC runtime files into the bundle —
// a WebAssembly module and a viewer worker — and refused the build if the built
// code had stopped naming the folder they landed in. That refusal was the right
// shape and it has done its job: the viewer those files fed is gone, nothing in
// the program asks for them, and the plugin went with it rather than shipping
// 2.7 MB the window never opens. The IFC writer is untouched; it is handed its
// module rather than fetching one, which is why it needs nothing here.

import { defineConfig } from 'vite';

export default defineConfig({
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
