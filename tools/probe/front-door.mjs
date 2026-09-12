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

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  detachCostPanel, launch, scratchJob, showTab, tool, until, wait, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

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

  // The same four facts, at the regions that carry them now: the file's name
  // is the document bar's (it is the identity of the one open file), what has
  // been traced is the list where conditions live, and what the sheet is
  // scaled at is in the tool settings strip, because it is what decides
  // whether a measurement exists at all. Not one of them is asserted less.
  // Conditions live in the cost panel's Condition tab (D148). The surface is
  // the drawing; what has been traced off it is read where it is listed.
  await showTab(session, 'Condition');
  const opened = await session.execute(function () {
    return JSON.stringify({
      job: (document.querySelector('.doc-job') || {}).textContent || '',
      start: !!document.querySelector('.start'),
      conditions: document.querySelectorAll('.cost-condition .condition-list .condition').length,
      strip: (document.querySelector('.tool-strip') || {}).innerText || '',
      area: (document.querySelector('.roof-area') || {}).innerText || '',
    });
  });
  const state = JSON.parse(opened);

  check('the job actually opened', () => assert.ok(state.job.length > 0, `job label is "${state.job}"`));
  check('the start screen gave way to the drawing', () => assert.equal(state.start, false));
  check('the conditions are listed where they live now', () =>
    assert.ok(state.conditions >= 3, `${state.conditions} of them`));
  check('the tool settings strip says what the sheet is scaled at, and in what units', () => {
    assert.match(state.strip, /Scale/, `the strip reads "${state.strip.replace(/\s+/g, ' ')}"`);
    assert.match(state.strip, /Units/, `the strip reads "${state.strip.replace(/\s+/g, ' ')}"`);
  });
  check('and the document bar carries the one roof-area figure', () => {
    assert.match(state.area, /Roof area/, `it reads "${state.area.replace(/\s+/g, ' ')}"`);
    assert.match(state.area, /SF/, `it reads "${state.area.replace(/\s+/g, ' ')}"`);
  });

  // ── the drawing is on screen, because the demo job has one ──────────────
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 25000 });
  await wait(1200);

  const plan = await session.execute(function () {
    return JSON.stringify({
      conditions: document.querySelectorAll('.cost-condition .condition-list .condition').length,
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
  await tool(session, 'Scale');

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

  // It asks in the tool settings strip, which is where everything the Scale act
  // needs now lives. The claim is unchanged and it is the one this whole probe
  // exists for: the program asks in a field in the window, never in a browser
  // dialog it does not have.
  const askedInWindow = await session.execute(function () {
    const bar = document.querySelector('.tool-strip');
    return JSON.stringify({
      asking: !!document.querySelector('.tool-strip .scale-ask'),
      hasField: !!document.querySelector('.scale-ask input'),
      words: bar ? (bar.innerText || '').slice(0, 160) : '',
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

  // ── pull the cost panel onto the other monitor, from the button on it ───
  const before = await session.handles();
  await detachCostPanel(session);
  const after = await session.handles();
  check('the button beside the tabs opens a second window', () => {
    assert.ok(after.length > before.length, `${before.length} → ${after.length}`);
  });

  const second = after.find((h) => !before.includes(h));
  if (second) {
    await session.switchTo(second);
    // The detached window IS the panel: there is no surface in it, so there is
    // no `#editor` for its children to arrive in. What tells you it is up is
    // the thing it exists to show.
    await until(session, () => !!document.querySelector('.cost-panel.detached .cost-totals'),
      { what: 'the detached cost panel' });
    await wait(900);
    const sheet = await session.execute(function () {
      return JSON.stringify({
        rows: document.querySelectorAll('.cost-line').length,
        selling: (document.querySelector('.selling .value') || {}).textContent || '',
        docBar: !!document.querySelector('.doc-bar'),
        rail: !!document.querySelector('.tool-rail'),
        title: (document.querySelector('.cost-title-name') || {}).textContent || '',
        home: !!document.querySelector('.cost-dock'),
      });
    });
    const sh = JSON.parse(sheet);
    check('the detached panel has the job\'s priced lines in it', () => {
      assert.ok(sh.rows >= 5, `${sh.rows} of them`);
      assert.match(sh.selling, /\$/, sh.selling);
    });
    check('and carries no second copy of the chrome, only its own title and the way home', () => {
      assert.equal(sh.docBar, false, 'the detached window has a document bar');
      assert.equal(sh.rail, false, 'the detached window has a tool rail');
      assert.match(sh.title, /Cost/, `its title bar reads "${sh.title}"`);
      assert.equal(sh.home, true, 'there is no way to dock it back');
    });

    await writeFile(join(EVIDENCE, `front-door-cost-${COMMIT}.png`),
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
