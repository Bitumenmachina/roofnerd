import { defineConfig } from 'vite';

// The application is built to static files and served by the shell over its own
// protocol — there is no dev server. That is on purpose: it means the content
// security policy in tauri.conf.json is live while developing, not just in a
// release build, so "this program cannot reach the network" is something you can
// check today rather than a promise about later.
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
