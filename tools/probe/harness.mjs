// ── The probe harness ──────────────────────────────────────────────────────
// Drives the built application in a headless browser so a section's done-check
// can be run rather than described.
//
// The Rust shell is not here, so it is stood in for: one document held on this
// side, `doc_get` and `doc_set` against it, the change event broadcast to every
// open window, and reading a drawing out of the job folder. That is the same
// shape as the real shell — one document, windows subscribe, nobody owns a copy
// — which is why a two-window check here proves the real thing.
//
// Everything ABOVE that line is the shipped code, unmodified: the editors, the
// surface, the tools, the engine.

import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, resolve } from 'node:path';
import puppeteer from 'puppeteer-core';

const CHROME = process.env.ROOFNERD_CHROME ?? '/usr/bin/google-chrome';
const DIST = resolve(import.meta.dirname, '../../packages/app/dist');

const TYPES = {
  '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript',
  '.css': 'text/css', '.json': 'application/json', '.pdf': 'application/pdf',
  '.png': 'image/png', '.map': 'application/json',
};

/** Serve the built application from a loopback port, the way the shell serves it. */
export async function serve() {
  const server = createServer(async (req, res) => {
    const path = (req.url ?? '/').split('?')[0];
    // The browser asks for this on every page and its absence is not a finding.
    if (path === '/favicon.ico') { res.writeHead(200, { 'content-type': 'image/x-icon' }).end(); return; }
    const file = join(DIST, path === '/' ? 'index.html' : path);
    try {
      const body = await readFile(file);
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
      res.end(body);
    } catch {
      res.writeHead(404).end('no');
    }
  });
  await new Promise((done) => server.listen(0, '127.0.0.1', done));
  const { port } = server.address();
  return { origin: `http://127.0.0.1:${port}`, stop: () => server.close() };
}

/** Walk a JSON Pointer to the slot it names. */
function slotAt(root, pointer) {
  const parts = pointer.slice(1).split('/').map((p) => p.replace(/~1/g, '/').replace(/~0/g, '~'));
  let node = root;
  for (const part of parts.slice(0, -1)) node = Array.isArray(node) ? node[Number(part)] : node[part];
  return { parent: node, key: parts[parts.length - 1] };
}

/**
 * Open the application with a job already in it.
 *
 * `files` maps a job-relative path to bytes, standing in for the job folder.
 */
export async function openApp({ doc, files = {}, editor = 'plan' } = {}) {
  const site = await serve();
  const browser = await puppeteer.launch({
    executablePath: CHROME,
    headless: true,
    args: ['--no-sandbox', '--disable-gpu', '--window-size=1400,1000'],
  });

  // THE one document. Every window reads this and none of them owns it.
  const state = { doc: structuredClone(doc) };
  const windows = [];
  const problems = [];
  const encoded = Object.fromEntries(Object.entries(files).map(([k, v]) => [k, Array.from(v)]));

  const broadcast = async () => {
    for (const w of windows) {
      if (w.isClosed()) continue;
      await w.evaluate((d) => window.__applyDoc(d), state.doc).catch(() => undefined);
    }
  };

  async function attach(page, which) {
    page.on('console', (m) => { if (m.type() === 'error') problems.push(m.text()); });
    page.on('pageerror', (e) => problems.push(String(e)));
    page.on('requestfailed', (r) => problems.push(`request failed: ${r.url()} — ${r.failure()?.errorText}`));
    page.on('response', (r) => { if (r.status() >= 400) problems.push(`HTTP ${r.status()}: ${r.url()}`); });
    if (process.env.ROOFNERD_PROBE_VERBOSE) {
      page.on('console', (m) => console.log(`  [${which} ${m.type()}] ${m.text()}`));
    }

    await page.exposeFunction('__shellGet', () => state.doc);
    await page.exposeFunction('__shellSet', async (pointer, value) => {
      const { parent, key } = slotAt(state.doc, pointer);
      if (value === undefined) delete parent[key];
      else if (Array.isArray(parent)) parent[Number(key)] = value;
      else parent[key] = value;
      await broadcast();
      return null;
    });
    await page.exposeFunction('__shellRead', (relative) => {
      const bytes = encoded[relative];
      if (!bytes) throw new Error(`${relative} is not in this job`);
      return bytes;
    });

    await page.evaluateOnNewDocument(() => {
      const listeners = new Map();
      let local = {};

      window.__applyDoc = (d) => {
        local = d;
        for (const fn of listeners.get('doc:changed') ?? []) fn({ payload: d });
      };
      window.__PROBE__ = { doc: () => local };

      window.__TAURI_INTERNALS__ = {
        transformCallback: (cb) => {
          const id = `cb${Math.random().toString(36).slice(2)}`;
          window[id] = cb;
          return id;
        },
        invoke: async (command, args = {}) => {
          switch (command) {
            case 'doc_get': { local = await window.__shellGet(); return local; }
            case 'doc_folder':
            case 'demo_folder': return '/probe/job';
            case 'doc_set': return window.__shellSet(args.pointer, args.value);
            case 'doc_save': return '/probe/job';
            case 'add_page_source': return `pages/${String(args.source).split('/').pop()}`;
            case 'read_page_source': return window.__shellRead(args.relative);
            case 'open_editor': return null;
            case 'plugin:event|listen': {
              const list = listeners.get(args.event) ?? [];
              list.push((payload) => window[args.handler](payload));
              listeners.set(args.event, list);
              return 0;
            }
            default: return null;
          }
        },
      };
    });

    windows.push(page);
  }

  const page = await browser.newPage();
  await page.setViewport({ width: 1400, height: 1000 });
  await attach(page, editor);
  await page.goto(`${site.origin}/?editor=${editor}`, { waitUntil: 'load' });
  await page.waitForFunction(() => document.querySelector('#editor')?.children.length > 0);

  return {
    page,
    problems,
    doc: () => structuredClone(state.doc),
    /** Change the job from outside any window — what a second window's edit looks like. */
    async set(pointer, value) {
      const { parent, key } = slotAt(state.doc, pointer);
      if (Array.isArray(parent)) parent[Number(key)] = value;
      else parent[key] = value;
      await broadcast();
    },
    /** Tear an editor off into its own window, on the same document. */
    async tearOff(which) {
      const second = await browser.newPage();
      await second.setViewport({ width: 900, height: 900 });
      await attach(second, which);
      await second.goto(`${site.origin}/?editor=${which}`, { waitUntil: 'load' });
      await second.waitForFunction(() => document.querySelector('#editor')?.children.length > 0);
      return second;
    },
    async close() {
      await browser.close();
      site.stop();
    },
  };
}

/**
 * Wait until a drawing is actually on screen.
 *
 * A bare canvas is 300 x 150 before anything is painted into it, so "the canvas
 * has a width" is not evidence of anything. The overlay's viewBox is set by the
 * surface only once it has a sheet, which makes it the honest signal.
 */
export async function waitForSheet(page, timeout = 20000) {
  await page.waitForFunction(
    () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { timeout },
  );
  await page.waitForFunction(
    () => {
      const o = document.querySelector('.surface-overlay');
      const c = document.querySelector('.surface-sheet');
      return o && c && c.width === Number(o.getAttribute('width'));
    },
    { timeout },
  );
}

/** Click at a point in PAGE units on the trace overlay. */
export async function clickPage(page, x, y, { detail = 1 } = {}) {
  await page.evaluate((px, py, clickCount) => {
    const overlay = document.querySelector('.surface-overlay');
    const box = overlay.getBoundingClientRect();
    const zoom = box.width / overlay.viewBox.baseVal.width;
    const clientX = box.left + px * zoom;
    const clientY = box.top + py * zoom;
    const make = (type) => new PointerEvent(type, {
      clientX, clientY, button: 0, buttons: 1, bubbles: true, detail: clickCount, pointerId: 1,
    });
    overlay.dispatchEvent(make('pointermove'));
    overlay.dispatchEvent(make('pointerdown'));
    overlay.dispatchEvent(make('pointerup'));
  }, x, y, detail);
}

/** Click a toolbar button by its label. */
export const clickTool = (page, label) =>
  page.evaluate((name) => [...document.querySelectorAll('.toolbar button')]
    .find((b) => b.textContent === name).click(), label);

export const wait = (ms) => new Promise((r) => setTimeout(r, ms));
