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

/**
 * What the driver said, kept rather than summarised.
 *
 * The first run of section 6 printed "real pointer input is not available: ." —
 * an empty reason, because the driver answered with a `message` of empty string
 * and this class had nothing else to say. A refusal with no diagnosis in it
 * costs a whole round trip to the runtime, so the status, the body and the JSON
 * that was sent all travel with the error now.
 */
class WebDriverError extends Error {
  constructor(message, { status = 0, body = '', sent = '' } = {}) {
    super(message);
    this.status = status;
    this.body = body;
    this.sent = sent;
  }
}

/** The bits of the WebDriver protocol this project uses. */
class Session {
  constructor(base, id, capabilities) {
    this.base = base;
    this.id = id;
    this.capabilities = capabilities;
  }

  async call(method, path, body) {
    const sent = body === undefined ? '' : JSON.stringify(body);
    const response = await fetch(`${this.base}/session/${this.id}${path}`, {
      method,
      headers: { 'content-type': 'application/json' },
      ...(body === undefined ? {} : { body: sent }),
    });
    // The text first, the JSON second: a driver that answers with an empty body
    // or with something that is not JSON at all is exactly the case that needs
    // reporting, and `response.json()` throws that evidence away.
    const text = await response.text();
    let payload = {};
    try {
      payload = JSON.parse(text);
    } catch {
      payload = {};
    }
    if (!response.ok || payload?.value?.error) {
      const said = typeof payload?.value?.message === 'string' ? payload.value.message.trim() : '';
      const kind = payload?.value?.error ? ` ${payload.value.error}` : '';
      const why = said
        ? `: ${said}`
        : ` — the driver's answer was ${text.trim() ? `"${text.trim().slice(0, 300)}"` : 'empty'}`;
      throw new WebDriverError(`${method} ${path} → ${response.status}${kind}${why}`, {
        status: response.status,
        body: text.slice(0, 600),
        sent: sent.slice(0, 600),
      });
    }
    return payload.value;
  }

  title() { return this.call('GET', '/title'); }
  handles() { return this.call('GET', '/window/handles'); }
  switchTo(handle) { return this.call('POST', '/window', { handle }); }
  screenshot() { return this.call('GET', '/screenshot'); }

  /**
   * The W3C actions endpoint: real input, dispatched by the driver.
   *
   * Everything else in here reaches the page through `execute`, which is the
   * program's own code being called by name. That is fine for reading the
   * window and it is not a click: a synthetic `PointerEvent` is a JavaScript
   * object with the coordinates already in it, so a check built on one can pass
   * while the thing a hand does never worked. The 3D pick is exactly that case
   * — it reads clientX and clientY off the event and casts a ray — so section 6
   * asks for the real thing.
   */
  actions(sequence) { return this.call('POST', '/actions', { actions: sequence }); }

  /** Let go of whatever the driver is holding down. */
  releaseActions() { return this.call('DELETE', '/actions'); }

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

/**
 * Start collecting what the window says went wrong, into `window.__errors`.
 *
 * That global was read by two probes for as long as they existed and was set by
 * nothing at all, so "nothing errored in the page" was a row that could not go
 * red. This is the half that was missing: the unhandled errors, the rejected
 * promises nobody caught, and everything the program itself logged as an error.
 *
 * Install it after `launch`, and AGAIN after anything that reloads the window —
 * switching editors through the picker is `window.location.assign`, and a reload
 * takes every hook in the page with it. Installing twice over one page is
 * harmless; it says so and leaves the first one alone.
 */
export const watchErrors = (session) => session.execute(function () {
  if (window.__errorsWatching) return 'already watching';
  window.__errorsWatching = true;
  window.__errors = [];
  const was = console.error;
  console.error = function () {
    window.__errors.push([].map.call(arguments, String).join(' '));
    was.apply(console, arguments);
  };
  window.addEventListener('error', function (e) { window.__errors.push(String(e.message)); });
  window.addEventListener('unhandledrejection', function (e) { window.__errors.push(String(e.reason)); });
  return 'watching';
});

/**
 * What the window has logged since the watch went in.
 *
 * It refuses rather than answering an empty list when nothing was watching —
 * an unwatched page and a page where nothing went wrong are not the same
 * answer, and reading them as the same is the defect this pair exists to close.
 */
export async function errorsSoFar(session) {
  const raw = await session.execute(function () {
    return JSON.stringify(window.__errors === undefined ? null : window.__errors);
  });
  const list = JSON.parse(raw);
  if (list === null) {
    throw new Error('nothing was watching for errors — call watchErrors(session), and again after any reload');
  }
  return list;
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

/**
 * One click of the left button at a point in the window, through the driver.
 *
 * Coordinates are CSS pixels from the top-left of the viewport — the same frame
 * `clientX`/`clientY` are given in, which is what `getBoundingClientRect` hands
 * back and what the Model's raycast reads. A `pause` between down and up
 * because a zero-length press is not what a hand does and some webviews treat
 * it as nothing at all.
 *
 * It throws if the driver will not do it. The caller decides what that means —
 * section 6 says so in its output and falls back for the one click that cannot
 * be left untested, rather than reporting a real pointer it never had.
 */
/**
 * The shapes of a click, most human first, then progressively more minimal.
 *
 * WebKitWebDriver refused the first one with an empty message, and an empty
 * message is not a diagnosis — so instead of one shape and a shrug there are
 * three, each tried in turn, and whatever the driver says about each of them
 * travels out in the error. The second is the W3C minimum: move to a point in
 * the viewport, press, release. The third drops `parameters`, which the
 * specification asks for and some drivers reject.
 */
const CLICK_SHAPES = [
  {
    name: 'pointer with parameters, a pause between down and up',
    of: (x, y) => [{
      type: 'pointer',
      id: 'mouse',
      parameters: { pointerType: 'mouse' },
      actions: [
        { type: 'pointerMove', origin: 'viewport', x, y },
        { type: 'pointerDown', button: 0 },
        { type: 'pause', duration: 40 },
        { type: 'pointerUp', button: 0 },
      ],
    }],
  },
  {
    name: 'the W3C minimum — move, down, up',
    of: (x, y) => [{
      type: 'pointer',
      id: 'mouse',
      parameters: { pointerType: 'mouse' },
      actions: [
        { type: 'pointerMove', origin: 'viewport', x, y },
        { type: 'pointerDown', button: 0 },
        { type: 'pointerUp', button: 0 },
      ],
    }],
  },
  {
    name: 'the same without pointerType parameters',
    of: (x, y) => [{
      type: 'pointer',
      id: 'mouse',
      actions: [
        { type: 'pointerMove', origin: 'viewport', x, y },
        { type: 'pointerDown', button: 0 },
        { type: 'pointerUp', button: 0 },
      ],
    }],
  },
];

/**
 * One click of the left button at a point in the window, through the driver.
 *
 * Answers which shape the driver took, so a probe can say so; throws with every
 * shape's own refusal in the message when none of them is accepted. A repeated
 * click is the price of finding out: if a shape half-works the next one clicks
 * the same point again, and every place this is used, clicking twice is the same
 * as clicking once.
 */
export async function pointerClick(session, x, y) {
  const at = { x: Math.round(x), y: Math.round(y) };
  const tried = [];
  for (const shape of CLICK_SHAPES) {
    try {
      await session.actions(shape.of(at.x, at.y));
      return { shape: shape.name, at };
    } catch (e) {
      tried.push(`${shape.name} → ${e.message}${e.sent ? `\n        sent ${e.sent}` : ''}`);
    }
  }
  const error = new Error(
    `the driver would not click at ${at.x}, ${at.y} — ${CLICK_SHAPES.length} shapes refused:`
    + `\n      ${tried.join('\n      ')}`);
  error.tried = tried;
  throw error;
}

/**
 * Keystrokes, one key down and up per character, through the driver.
 *
 * They go where the page's focus is, which is the whole point of having them:
 * `input.value = x` writes to an element a check already has in its hand, and
 * says nothing about whether a person could have typed it. Two of these, with a
 * wait between them, is the smallest honest test of a field that is rebuilt
 * while it is being typed into.
 *
 * Send one character at a time when the gap between them matters.
 */
export async function typeKeys(session, text) {
  try {
    await session.actions([{
      type: 'key',
      id: 'keyboard',
      actions: [...String(text)].flatMap((ch) => [
        { type: 'keyDown', value: ch },
        { type: 'keyUp', value: ch },
      ]),
    }]);
    return { sent: String(text) };
  } catch (e) {
    // Same reasoning as the click: the refusal carries what was sent and what
    // the driver answered, because a probe that can only say "it did not work"
    // costs a whole run to find out why.
    const error = new Error(`the driver would not type ${JSON.stringify(String(text))} → ${e.message}`
      + `${e.sent ? `\n      sent ${e.sent}` : ''}`);
    error.status = e.status;
    throw error;
  }
}

/** Release everything the driver holds. Cheap insurance in a `finally`. */
export const pointerRelease = (session) => session.releaseActions().catch(() => undefined);

/**
 * The middle of something on screen, in the coordinates a pointer wants.
 *
 * Null when it is not there, so a check can say "there was nothing to click"
 * rather than clicking the top-left corner of the window.
 */
export const centreOf = (session, selector, text) => session.execute(function (sel, want) {
  const all = [...document.querySelectorAll(sel)];
  const el = want ? all.find((e) => ((e.innerText || e.textContent || '').indexOf(want) >= 0)) : all[0];
  if (!el) return null;
  const r = el.getBoundingClientRect();
  if (r.width < 1 || r.height < 1) return null;
  return JSON.stringify({ x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2) });
}, selector, text ?? null);

/**
 * The one element as a picture, cropped by the driver where it will do it and
 * the whole window where it will not. Evidence is never the reason a check
 * fails, so the fallback is silent and the caller still gets a PNG.
 */
export async function elementShot(session, selector) {
  try {
    const found = await session.call('POST', '/element', { using: 'css selector', value: selector });
    const id = Object.values(found)[0];
    return await session.call('GET', `/element/${id}/screenshot`);
  } catch {
    return await session.screenshot();
  }
}

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
