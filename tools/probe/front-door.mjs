// ── The front door ─────────────────────────────────────────────────────────
// A check that bypasses the path a person uses is not a check.
//
// Every other runtime probe opened a job by calling `doc_open` straight through
// the bridge. They all passed while the program could not be given a job at
// all: File → Open a job called `window.prompt`, which this webview does not
// implement, so it returned nothing and the handler quietly gave up. "8/8" was
// true and useless in the same breath.
//
// This one starts where a person starts — the start screen — and only ever
// clicks what a person can click.
//
//   node tools/probe/front-door.mjs

import { execFileSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import { launch, scratchJob, until, wait } from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

/** Click something a person can see, by the words on it. */
const clickByText = (session, selector, text) => session.execute(function (sel, want) {
  const el = [...document.querySelectorAll(sel)].find((e) => (e.textContent || '').trim() === want);
  if (!el) return false;
  el.click();
  return true;
}, selector, text);

await mkdir(EVIDENCE, { recursive: true });
const job = await scratchJob(join(ROOT, 'jobs/demo-job'));
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });

  // ── the start screen ────────────────────────────────────────────────────
  const start = await session.execute(function () {
    const el = document.querySelector('.start');
    return el ? (el.innerText || '').slice(0, 300) : null;
  });
  check('a window with no job open shows the start screen', () => {
    assert.ok(start, 'no start screen');
    assert.match(start, /Open a job/);
  });
  check('and says in one sentence what the program is', () => {
    assert.match(start, /Trace the roof/);
  });

  // ── open a job the way a person does ────────────────────────────────────
  // Through the button on the start screen. The other way in — "Open a job…" —
  // raises the operating system's own folder picker, which no probe can click
  // through; that one is checked by hand and noted in STATUS.md. This one is a
  // real front door, it is the one somebody opening the program for the first
  // time will use, and it goes through exactly the same code.
  check('the start screen offers the demo job', () => {
    assert.match(start, /Open the demo job/, start);
  });

  const clicked = await clickByText(session, '.start-demo', 'Open the demo job');
  check('and its button can be clicked', () => assert.equal(clicked, true));
  await wait(2500);

  const opened = await session.execute(function () {
    return JSON.stringify({
      job: (document.querySelector('.menu-job') || {}).textContent || '',
      start: !!document.querySelector('.start'),
      tree: document.querySelectorAll('.tree-node').length,
      status: (document.querySelector('.status-facts') || {}).textContent || '',
    });
  });
  const state = JSON.parse(opened);

  check('the job actually opened', () => assert.ok(state.job.length > 0, `job label is "${state.job}"`));
  check('the start screen gave way to the editor', () => assert.equal(state.start, false));
  check('the tree filled with the job', () => assert.ok(state.tree >= 3, `${state.tree} node(s)`));
  check('the status bar names the job and its scale', () => {
    assert.match(state.status, /Job/);
    assert.match(state.status, /Scale/);
  });

  // ── the drawing is on screen, because the demo job has one ──────────────
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 25000 });
  await wait(1200);

  const plan = await session.execute(function () {
    return JSON.stringify({
      conditions: document.querySelectorAll('.condition').length,
      badge: (document.querySelector('.scale-badge') || {}).textContent || '',
      measures: [...document.querySelectorAll('.condition-measures')].map((e) => e.textContent),
    });
  });
  const p = JSON.parse(plan);
  check('the demo job opens to a drawing with work already on it', () => {
    assert.ok(p.conditions >= 3, `${p.conditions} condition(s)`);
    assert.match(p.badge, /1\/4|Scaled/, p.badge);
  });
  check('and its conditions carry real measures', () => {
    assert.ok(p.measures.some((m) => /SF/.test(m)), p.measures.join(' | '));
    assert.ok(p.measures.some((m) => /LF/.test(m)), p.measures.join(' | '));
  });

  // ── set a scale through the field, not a dialog ─────────────────────────
  await session.execute(function () {
    const b = [...document.querySelectorAll('.toolbar .tool')].find((e) => /Scale|Rescale/.test(e.textContent));
    if (b) b.click();
  });
  await wait(400);

  // Two clicks on the overlay, the way a hand does it.
  await session.execute(function () {
    const overlay = document.querySelector('.surface-overlay');
    const box = overlay.getBoundingClientRect();
    const at = (x, y) => {
      for (const type of ['pointermove', 'pointerdown', 'pointerup']) {
        overlay.dispatchEvent(new PointerEvent(type, {
          clientX: box.left + x, clientY: box.top + y,
          button: 0, buttons: 1, bubbles: true, detail: 1, pointerId: 1,
        }));
      }
    };
    at(80, 80);
    at(280, 80);
  });
  await wait(600);

  const askedInWindow = await session.execute(function () {
    const bar = document.querySelector('.hint');
    return JSON.stringify({
      asking: !!(bar && bar.classList.contains('asking')),
      hasField: !!document.querySelector('.scale-ask input'),
      words: bar ? (bar.innerText || '').slice(0, 120) : '',
    });
  });
  const ask = JSON.parse(askedInWindow);
  check('setting a scale asks in the window, in a field', () => {
    assert.equal(ask.asking, true, ask.words);
    assert.equal(ask.hasField, true);
  });
  check('and asks in the notation a drawing uses', () => {
    assert.match(ask.words, /How long is that/);
  });

  const applied = await session.execute(function (typed) {
    const input = document.querySelector('.scale-ask input');
    input.value = typed;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    const button = [...document.querySelectorAll('.scale-ask button')].find((b) => b.textContent === 'Set the scale');
    button.click();
    return true;
  }, `20'-0"`);
  check('the field accepts it', () => assert.equal(applied, true));
  await wait(900);

  const rescaled = await session.execute(function () {
    return (document.querySelector('.scale-badge') || {}).textContent || '';
  });
  check('and the drawing takes the new scale', () => {
    assert.match(rescaled, /20'-0"|two points/, rescaled);
  });

  // ── tear the sheet off, from the button on the area ─────────────────────
  const before = await session.handles();
  await session.execute(function () {
    const b = document.querySelector('.area-actions .icon-button[aria-label="Open in its own window"]');
    if (b) b.click();
  });
  await wait(2000);
  const after = await session.handles();
  check('the tear-off button on the area opens a second window', () => {
    assert.ok(after.length > before.length, `${before.length} → ${after.length}`);
  });

  const second = after.find((h) => !before.includes(h));
  if (second) {
    await session.switchTo(second);
    await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the sheet' });
    await wait(900);
    const sheet = await session.execute(function () {
      return JSON.stringify({
        rows: document.querySelectorAll('.sheet tbody tr:not(.group-row):not(.add-row)').length,
        selling: (document.querySelector('.selling .value') || {}).textContent || '',
        menuBar: !!document.querySelector('.menu-bar'),
      });
    });
    const sh = JSON.parse(sheet);
    check('the torn-off sheet has the job\'s priced lines in it', () => {
      assert.ok(sh.rows >= 5, `${sh.rows} line(s)`);
      assert.match(sh.selling, /\$/, sh.selling);
    });
    check('and carries no second copy of the chrome', () => assert.equal(sh.menuBar, false));

    await writeFile(join(EVIDENCE, `front-door-sheet-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));
    await session.switchTo(before[0]);
    await writeFile(join(EVIDENCE, `front-door-plan-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));
  }
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
  await job.remove();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
