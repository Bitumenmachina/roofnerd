// ── Reading the editors' own source in a test ──────────────────────────────
// The application is TypeScript that Vite bundles; it is never emitted to disk
// as JavaScript, so a test in Node has to read the source. Node strips the types
// on its own — it has since 23 — but it will not answer a `./thing.js` import
// with `thing.ts`, and that is how every import in this package is written
// because that is what the bundler and the compiler both want.
//
// So: one resolve hook, test-only, that tries the TypeScript file beside the
// name the source asked for. It changes nothing in the program; it is the
// smallest thing that lets a check read the same file the window runs.

import { registerHooks } from 'node:module';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

registerHooks({
  resolve(specifier, context, next) {
    if (specifier.startsWith('.') && specifier.endsWith('.js') && context.parentURL) {
      const beside = new URL(`${specifier.slice(0, -3)}.ts`, context.parentURL);
      if (existsSync(fileURLToPath(beside))) return { url: beside.href, shortCircuit: true };
    }
    return next(specifier, context);
  },
});
