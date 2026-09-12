// ── The real-runtime harness ───────────────────────────────────────────────
// Drives the SHIPPED application: the release binary, WebKitGTK rendering, the
// Rust shell holding the document, real OS windows.
//
// The Chrome harness stubbed the shell and rendered in Blink. That was useful
// while building, but it is not what an estimator runs, and a check that passes
// there is not evidence about the thing that ships.
//
// The WebDriver client here is written out rather than taken off the shelf:
// tauri-driver wants the capabilities exactly as the specification writes them,
// and the client libraries reshape them on the way out. Sixty lines of fetch
// against a documented protocol is less to go wrong than a library that has an
// opinion about the payload.

import { execFileSync, spawn } from 'node:child_process';
import { mkdtemp, cp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '../..');
const BINARY = join(ROOT, 'src-tauri/target/release/roofnerd');
const DRIVER = join(process.env.HOME ?? '', '.cargo/bin/tauri-driver');

const PORT = Number(process.env.ROOFNERD_DRIVER_PORT ?? 4444);
// tauri-driver spawns WebKitWebDriver behind itself. Give them the same number
// and the native driver fails to bind, which surfaces as a capability mismatch
// and sends you looking in entirely the wrong place.
const NATIVE_PORT = Number(process.env.ROOFNERD_NATIVE_PORT ?? 4460);

/** A throwaway copy of a job, so a probe never edits the one in the repository. */
export async function scratchJob(from = join(ROOT, 'jobs/demo-job')) {
  const dir = await mkdtemp(join(tmpdir(), 'roofnerd-job-'));
  await cp(from, dir, { recursive: true });
  return { dir, async remove() { await rm(dir, { recursive: true, force: true }); } };
}

class WebDriverError extends Error {}

/** The bits of the WebDriver protocol this project uses. */
class Session {
  constructor(base, id, capabilities) {
    this.base = base;
    this.id = id;
    this.capabilities = capabilities;
  }

  async call(method, path, body) {
    const response = await fetch(`${this.base}/session/${this.id}${path}`, {
      method,
      headers: { 'content-type': 'application/json' },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok || payload?.value?.error) {
      throw new WebDriverError(payload?.value?.message ?? `${method} ${path} failed`);
    }
    return payload.value;
  }

  title() { return this.call('GET', '/title'); }
  handles() { return this.call('GET', '/window/handles'); }
  switchTo(handle) { return this.call('POST', '/window', { handle }); }
  screenshot() { return this.call('GET', '/screenshot'); }

  /** Run a function in the page and get its return value back. */
  execute(fn, ...args) {
    return this.call('POST', '/execute/sync', { script: `return (${fn}).apply(null, arguments)`, args });
  }

  /** Run a function that is handed a `done` callback as its last argument. */
  executeAsync(fn, ...args) {
    return this.call('POST', '/execute/async', { script: `return (${fn}).apply(null, arguments)`, args });
  }

  end() { return fetch(`${this.base}/session/${this.id}`, { method: 'DELETE' }).catch(() => undefined); }
}

/**
 * Start the application under WebDriver.
 *
 * WebKitGTK needs a display. On this machine that is the live session; where
 * there is none, put `xvfb-run` in front of the probe.
 */
export async function launch({ timeout = 60000 } = {}) {
  const driver = spawn(DRIVER, ['--port', String(PORT), '--native-port', String(NATIVE_PORT)], {
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  const log = [];
  driver.stdout.on('data', (d) => log.push(String(d)));
  driver.stderr.on('data', (d) => log.push(String(d)));

  const base = `http://127.0.0.1:${PORT}`;
  const deadline = Date.now() + timeout;
  let session = null;
  let lastError = 'never answered';

  while (Date.now() < deadline && session === null) {
    await new Promise((r) => setTimeout(r, 500));
    try {
      const response = await fetch(`${base}/session`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          capabilities: {
            alwaysMatch: { browserName: 'wry', 'tauri:options': { application: BINARY } },
          },
        }),
      });
      const payload = await response.json();
      if (payload?.value?.sessionId) {
        session = new Session(base, payload.value.sessionId, payload.value.capabilities ?? {});
      } else {
        lastError = payload?.value?.message ?? JSON.stringify(payload).slice(0, 200);
      }
    } catch (e) {
      lastError = e.message;
    }
  }

  if (!session) {
    driver.kill();
    throw new Error(`could not start the application under WebDriver: ${lastError}\n${log.join('')}`);
  }

  return {
    session,
    driverLog: log,
    async close() {
      await session.end();
      driver.kill();
    },
  };
}

/**
 * Open the demo job the way a person does: from the button on the start screen.
 *
 * No probe calls `doc_open` any more. Every one of them passed while the
 * program could not be given a job at all, because they all went round the
 * front door — File → Open a job called `window.prompt`, which this webview
 * does not implement, so it returned nothing and the handler gave up quietly.
 * A check that bypasses the path a person uses is not a check.
 */
export async function openDemoJob(session, { timeout = 30000 } = {}) {
  await until(session, () => document.querySelector('#editor')?.children.length > 0,
    { what: 'the window', timeout });

  const clicked = await session.execute(function () {
    const button = [...document.querySelectorAll('.start-demo')]
      .find((b) => (b.textContent || '').trim() === 'Open the demo job');
    if (!button) return false;
    button.click();
    return true;
  });
  if (!clicked) throw new Error('the start screen offers no way into the demo job');

  await until(session, () => !document.querySelector('.start'),
    { what: 'the job to open', timeout });
  return true;
}

/** Wait for something in the page, polling rather than sleeping blindly. */
export async function until(session, fn, { timeout = 20000, every = 250, what = 'a condition' } = {}) {
  const deadline = Date.now() + timeout;
  for (;;) {
    const value = await session.execute(fn).catch(() => null);
    if (value) return value;
    if (Date.now() > deadline) throw new Error(`timed out waiting for ${what}`);
    await new Promise((r) => setTimeout(r, every));
  }
}

export const wait = (ms) => new Promise((r) => setTimeout(r, ms));

/**
 * Click points on the drawing, the way a hand does.
 *
 * Pointer events on the overlay in its own client coordinates, then Enter to
 * finish the shape. Lived in one probe and is wanted by every probe that traces
 * anything, which is the usual reason a helper ends up copied.
 */
export const clickAt = (session, points) => session.execute(function (pts) {
  const o = document.querySelector('.surface-overlay');
  const box = o.getBoundingClientRect();
  for (const [x, y] of pts) {
    for (const type of ['pointermove', 'pointerdown', 'pointerup']) {
      o.dispatchEvent(new PointerEvent(type, {
        clientX: box.left + x, clientY: box.top + y,
        button: 0, buttons: 1, bubbles: true, detail: 1, pointerId: 1,
      }));
    }
  }
  document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
}, points);

/** Pick a tool off the toolbar by the word on it. */
export const tool = async (session, label) => {
  await session.execute(function (want) {
    const b = [...document.querySelectorAll('.toolbar .tool')]
      .find((e) => e.textContent.trim().startsWith(want));
    if (b) b.click();
  }, label);
  await wait(300);
};

/** Type a real dimension into the scale field and commit it. */
export const typeScale = async (session, text) => {
  await session.execute(function (value) {
    const f = document.querySelector('.hint input, .scale-field input, input.scale');
    if (!f) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(f, value);
    f.dispatchEvent(new Event('input', { bubbles: true }));
    f.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    const form = f.closest('form');
    if (form) form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
    return true;
  }, text);
  await wait(900);
};

/**
 * The commit a probe stamps on its evidence must name the code that ran. A dirty tree
 * photographed under HEAD's hash is evidence of nothing, so this refuses rather than guesses.
 * Proven red on an untracked file before it was trusted.
 */
export function commitStamp() {
  const dirty = execFileSync('git', ['status', '--porcelain'], { cwd: ROOT }).toString().trim();
  if (dirty) {
    console.error('the working tree is not clean — commit first, then take evidence:\n' + dirty);
    process.exit(2);
  }
  return execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();
}
