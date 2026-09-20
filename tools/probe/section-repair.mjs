// ── Round 2's repair order, R4 to R14: the screens tell the truth ────────────
// One row per claim the repair order makes about what a person sees, through
// the front door, on the shipped program. Written red at the batch's own head.
//
//   node tools/probe/section-repair.mjs
//
// Pictures: evidence/section-repair-<screen>-<commit>.png

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  launch, openDemoJob, showTab, showView, until, wait, commitStamp, reachRefusals,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};
const read = (session, fn, ...args) => session.execute(fn, ...args).then((v) => JSON.parse(v));

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;
const shot = async (name) => {
  const path = join(EVIDENCE, `section-repair-${name}-${COMMIT}.png`);
  await writeFile(path, Buffer.from(await session.screenshot(), 'base64'));
  console.log(`        picture: ${path.replace(`${ROOT}/`, '')}`);
};

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await openDemoJob(session);
  await wait(1800);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 30000 });
  await wait(1000);

  // ── R4: the recap foots by hand ─────────────────────────────────────────
  await showTab(session, 'Recap');
  await wait(600);
  const recap = await read(session, function () {
    const text = (document.querySelector('.cost-recap, .recap') || document.body).innerText || '';
    const money = (label) => {
      const m = text.match(new RegExp(label + '[^$]*\\$([\\d,]+\\.\\d\\d)'));
      return m ? Math.round(parseFloat(m[1].replace(/,/g, '')) * 100) : null;
    };
    const classes = ['Material', 'Labor', 'Sub', 'Equipment', 'Other', 'Supervision']
      .map((c) => ({ c, cents: money('\\n' + c + '\\b') ?? money('^' + c + '\\b') }));
    return JSON.stringify({ classes, jobCost: money('Job cost'), overhead: money('Overhead'),
      profit: money('Profit'), contract: money('Contract amount'), bond: money('Bond'),
      selling: money('Selling price') });
  });
  check('R4 · the job cost is the sum of the printed class totals, to the cent', () => {
    const sum = recap.classes.reduce((n, x) => n + (x.cents ?? 0), 0);
    assert.ok(recap.classes.every((x) => x.cents !== null), `a class total could not be read: ${JSON.stringify(recap.classes)}`);
    assert.equal(recap.jobCost, sum, `job cost ${recap.jobCost} against classes summing to ${sum}`);
  });
  check('R4 · and the ladder under it adds up on its face', () => {
    assert.equal(recap.contract, recap.jobCost + (recap.overhead ?? 0) + recap.profit,
      `contract ${recap.contract} ≠ ${recap.jobCost} + ${recap.overhead ?? 0} + ${recap.profit}`);
    assert.equal(recap.selling, recap.contract + recap.bond, `selling ${recap.selling} ≠ ${recap.contract} + ${recap.bond}`);
  });
  await shot('recap');

  // ── R5, R9, R11, R6 (Wind): the Condition tab ────────────────────────────
  await showTab(session, 'Condition');
  await session.execute(function () {
    const rows = [...document.querySelectorAll('.condition-list .condition')];
    const p = rows.find((r) => /Parapet Wall Flashing/.test(r.textContent || ''));
    if (p) p.click();
  });
  await wait(800);
  const cond = await read(session, function () {
    const list = [...document.querySelectorAll('.condition-list .condition')].map((r) => (r.innerText || '').replace(/\s+/g, ' '));
    const panel = document.querySelector('.condition-panel');
    const tab = document.querySelector('.cost-condition');
    const values = [...document.querySelectorAll('.condition-panel .measure-value')].map((v) => {
      const r = v.getBoundingClientRect(); const t = tab ? tab.getBoundingClientRect() : { top: 0, bottom: innerHeight };
      return { text: (v.textContent || '').trim(), top: r.top, bottom: r.bottom, tabTop: t.top, tabBottom: t.bottom };
    });
    return JSON.stringify({
      list, panelText: panel ? panel.innerText : '', values,
      windNote: [...document.querySelectorAll('.condition-panel .panel-note')].map((n) => n.textContent).join(' | '),
    });
  });
  check('R5 · a closed run counts its corners, not the point it closes on', () => {
    const parapet = cond.list.find((t) => /Parapet Wall Flashing/.test(t)) || '';
    assert.match(parapet, /\b8 EA\b/, `the parapet reads "${parapet}"`);
    const curb = cond.list.find((t) => /RTU-1 Curb/.test(t)) || '';
    assert.match(curb, /\b4 EA\b/, `the curb reads "${curb}"`);
  });
  check('R9 · the picked condition’s figures are whole inside the tab at the design frame', () => {
    assert.ok(cond.values.length >= 2, `${cond.values.length} measure figures`);
    for (const v of cond.values) {
      assert.ok(v.top >= v.tabTop - 1 && v.bottom <= v.tabBottom + 1, `"${v.text}" sits ${Math.round(v.top)}–${Math.round(v.bottom)} against the tab's ${Math.round(v.tabTop)}–${Math.round(v.tabBottom)}`);
    }
  });
  check('R11 · the screen says stretch-out, not girth', () => {
    const all = cond.list.join(' ') + ' ' + cond.panelText;
    assert.doesNotMatch(all, /\bgirth\b/i, 'girth is on the Condition tab');
    assert.match(all, /stretch-out/i, 'stretch-out is nowhere on the Condition tab');
  });
  await session.execute(function () {
    const rows = [...document.querySelectorAll('.condition-list .condition')];
    const f = rows.find((r) => /Main Roof Field/.test(r.textContent || ''));
    if (f) f.click();
  });
  await wait(700);
  const wind = await read(session, function () {
    return JSON.stringify([...document.querySelectorAll('.condition-panel .panel-note')].map((n) => n.textContent || ''));
  });
  check('R6 · the Wind group says its zones are the three-zone rule, not yet ASCE 7-22’s four', () => {
    const note = wind.join(' ');
    assert.match(note, /three-zone/, `the panel's notes read: ${JSON.stringify(wind)}`);
    assert.match(note, /7-22/, `the panel's notes read: ${JSON.stringify(wind)}`);
  });
  await shot('condition');

  // ── R6: the fastening plan says it on its face ──────────────────────────
  await session.execute(function () {
    const chip = [...document.querySelectorAll('button')].find((b) => (b.textContent || '').trim() === 'fastening');
    if (chip) chip.click();
  });
  await wait(1200);
  const overlay = await read(session, function () {
    return JSON.stringify((document.querySelector('.plan-editor .drawing-note') || {}).textContent || '');
  });
  check('R6 · the fastening plan names the three-zone rule it draws and the standard it is typed to', () => {
    assert.match(overlay, /three-zone/, `the plan's note reads "${overlay}"`);
    assert.match(overlay, /7-22/, `the plan's note reads "${overlay}"`);
  });
  await shot('fastening');
  await session.execute(function () {
    const chip = [...document.querySelectorAll('button')].find((b) => (b.textContent || '').trim() === 'fastening');
    if (chip) chip.click();
  });
  await wait(400);

  // ── R7, R6, R11: the Section ────────────────────────────────────────────
  await showView(session, 'Section');
  await wait(2200);
  const section = await read(session, function () {
    const strip = [...document.querySelectorAll('.tool-strip .strip-field')].map((f) => (f.innerText || '').replace(/\s+/g, ' ').trim());
    const note = (document.querySelector('.section-editor .drawing-note, .drawing-note') || {}).textContent || '';
    const words = [...document.querySelectorAll('.section-drawing svg text')].map((t) => t.textContent || '').join(' | ');
    return JSON.stringify({ strip, note, words });
  });
  check('R7 · the strip states the scale the drawing is drawn at, and only that one', () => {
    const inStrip = section.strip.find((s) => /^Horizontal/.test(s)) || '';
    const said = inStrip.match(/1" = ([\d'\-" ]+)/);
    const drawn = section.note.match(/horizontal 1" = ([\d'\-" ]+)/);
    assert.ok(said && drawn, `strip "${inStrip}" · note "${section.note}"`);
    assert.equal(said[1].trim(), drawn[1].trim(), `the strip says 1" = ${said[1].trim()} and the drawing says 1" = ${drawn[1].trim()}`);
  });
  check('R6 · the section says the lettering is under review', () => {
    assert.match(section.note, /under review/, `the drawing's note reads "${section.note}"`);
  });
  check('R11 · the section says stretch-out at the coping', () => {
    assert.doesNotMatch(section.words, /girth/i, 'girth is on the section');
    assert.match(section.words, /stretch-out/i, 'no stretch-out on the section');
  });
  await shot('section');
  await showView(session, 'Plan');
  await wait(800);

  // ── R8, R10, R12, R13, R14, R6: the Estimate ───────────────────────────
  await showTab(session, 'Estimate');
  await wait(600);
  const est = await read(session, function () {
    const rows = [...document.querySelectorAll('.cost-line')];
    const head = (document.querySelector('.cost-head-count') || {}).textContent || '';
    const heights = rows.map((r) => Math.round(r.getBoundingClientRect().height));
    const items = rows.map((r) => ({
      item: (r.querySelector('.line-item') || {}).textContent || '',
      pieces: r.querySelector('.line-item') ? r.querySelector('.line-item').childElementCount : -1,
      basis: (r.querySelector('.line-basis') || {}).innerText || '',
      qty: (r.querySelector('.line-qty') || {}).textContent || '',
      unit: (r.querySelector('.line-unit') || {}).textContent || '',
      marks: r.querySelectorAll('.derived').length,
      formula: (r.querySelector('.line-formula, code') || {}).textContent || '',
      hasOff: !!r.querySelector('.line-off'),
    }));
    return JSON.stringify({ head, heights, items });
  });
  check('R8 · the header counts the rows that are drawn', () => {
    const n = parseInt(est.head, 10);
    assert.equal(n, est.items.length, `the header says "${est.head}" over ${est.items.length} rows`);
  });
  check('R10 · an item’s name is one piece, with no gap and no orphaned comma', () => {
    const split = est.items.filter((i) => i.pieces > 0 || /\s,\s/.test(i.item));
    assert.equal(split.length, 0, `${split.length} names are in pieces: ${split.slice(0, 3).map((i) => JSON.stringify(i.item)).join(', ')}`);
    const one = new Set(est.heights);
    assert.ok(one.size <= 2, `rows are ${[...one].join(', ')} px tall`);
  });
  check('R12 · a labor row prints its hours once and buys nothing', () => {
    const buyingHours = est.items.filter((i) => /buy [\d.,]+ HOURS/.test(i.basis));
    assert.equal(buyingHours.length, 0, `${buyingHours.length} rows read "buy … HOURS"`);
  });
  check('R12 · a count prints whole', () => {
    const eaRows = est.items.filter((i) => /^EA/.test(i.unit.trim()));
    assert.ok(eaRows.length > 0, 'no EA rows');
    const fractional = eaRows.filter((i) => /\.\d\d$/.test(i.qty.trim()));
    assert.equal(fractional.length, 0, `${fractional.length} counts carry decimals: ${fractional.slice(0, 3).map((i) => i.qty).join(', ')}`);
  });
  check('R13 · the mark is on a line the core solved and off a line that is only traced', () => {
    const traced = est.items.find((i) => /^Flash the wall/.test(i.item));
    const solved = est.items.find((i) => /^Coping cap/.test(i.item));
    assert.ok(traced && solved, `rows: ${est.items.slice(0, 4).map((i) => i.item).join(' · ')}`);
    assert.equal(traced.marks, 0, `"${traced.item}" (formula ${traced.formula}) carries ${traced.marks} marks`);
    assert.ok(solved.marks >= 1, `"${solved.item}" (formula ${solved.formula}) carries no mark`);
  });
  check('R14 · fasteners buy whole boxes', () => {
    const boxes = est.items.filter((i) => /buy [\d.,]+ BOX/.test(i.basis));
    assert.ok(boxes.length > 0, 'no row buys a BOX');
    const partial = boxes.filter((i) => /buy \d+\.\d*[1-9]\d* BOX/.test(i.basis));
    assert.equal(partial.length, 0, `${partial.length} rows buy a fraction of a box: ${partial.map((i) => i.basis.match(/buy [\d.,]+ BOX/)[0]).join(', ')}`);
  });
  // The adhesive row, opened: its note says the 120 mph row is used for 130.
  await session.execute(function () {
    const row = [...document.querySelectorAll('.cost-line')].find((r) => /Insulation adhesive.*field/.test((r.querySelector('.line-item') || {}).textContent || ''));
    const b = row && row.querySelector('.line-what');
    if (b) { b.scrollIntoView({ block: 'center' }); b.click(); }
  });
  await wait(900);
  const opened = await read(session, function () {
    const open = document.querySelector('.cost-line.open');
    return JSON.stringify(open ? (open.innerText || '') : '');
  });
  check('R6 · the adhesive line says on its face which row it is priced from and why', () => {
    assert.match(opened, /120 mph/, `the open line reads: ${opened.slice(0, 200)}`);
    assert.match(opened, /130/, `the open line reads: ${opened.slice(0, 200)}`);
  });
  await shot('estimate');
} finally {
  await app.close();
}

check('no click a helper made this run was refused for reach', () => {
  const r = reachRefusals();
  assert.equal(r.length, 0, r.map((x) => `${x.what}: ${x.why}`).join('; '));
});
const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `FAIL — ${failed} of ${results.length}` : `PASS — ${results.length}/${results.length}`);
process.exitCode = failed ? 1 : 0;
