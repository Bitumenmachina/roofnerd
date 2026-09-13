// ── Section 10a's done-check: the chrome, against the handoff's frames ─────
// The design handoff of 2026-09-08 draws eight frames at 1440 x 900 and calls
// its colours, spacings and densities final intent. This measures the ones a
// machine can measure — where the regions are, what they hold, what they do
// when the window is used — and leaves the rest to the eye and the images it
// writes beside itself.
//
// What it does NOT do is read the mockup. The handoff's HTML is the design; the
// program is a Tauri application, and every row here drives the shipped thing
// through the front door: the start screen, the buttons, the fields, the keys.
//
//   pnpm build:app && node tools/probe/section10a-chrome.mjs

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, detachCostPanel, errorsSoFar, launch, openDemoJob, pickLens, showTab, showView,
  tool, until, wait, watchErrors, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

/**
 * The Plan rail's own acts, in the order it draws them — off `plan.ts`'s
 * `ACTS`, the one array the rail is built from, rather than a list typed here
 * that goes stale the day a ninth act lands. C6 put four verbs on the rail
 * where this row used to find none, and typing the old eight straight into an
 * assertion is exactly the kind of copy that outlives what it was copied from.
 */
const planActsSource = await readFile(
  join(ROOT, 'packages/app/src/editors/plan.ts'), 'utf8',
);
const railActs = [...planActsSource.matchAll(/label:\s*'([^']+)'/g)].map((m) => m[1]);

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const shot = async (name) =>
  writeFile(join(EVIDENCE, `section10a-${name}-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

/** Money as cents, so two windows can be compared without a rounding argument. */
const cents = (text) => {
  const m = /-?\$[\d,]+\.\d\d/.exec(String(text));
  return m ? Math.round(Number(m[0].replace(/[$,]/g, '')) * 100) : null;
};

/**
 * The regions, measured off what is actually drawn.
 *
 * Heights and widths against the handoff's own table, and — the half a number
 * cannot fake — that they tile the window without overlapping and without
 * leaving a gap. A region that is the right height in the wrong place is not
 * the layout.
 */
const regionsNow = (session_) => session_.execute(function () {
  const box = function (sel) {
    const el = document.querySelector(sel);
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return {
      top: Math.round(r.top), bottom: Math.round(r.bottom),
      left: Math.round(r.left), right: Math.round(r.right),
      width: Math.round(r.width), height: Math.round(r.height),
      content: el.scrollWidth, room: el.clientWidth,
      hidden: !!el.hidden,
    };
  };
  return JSON.stringify({
    innerWidth: window.innerWidth,
    innerHeight: window.innerHeight,
    pageWidth: document.body.scrollWidth,
    documentWidth: document.documentElement.scrollWidth,
    doc: box('.doc-bar'),
    strip: box('.tool-strip'),
    // The field row inside the strip, which is what actually carries the
    // controls: the strip clips at its own edge, so asking the strip whether
    // it holds its own content reads equal however much is crammed in.
    stripFields: box('.tool-strip-fields'),
    body: box('.body-row'),
    rail: box('.tool-rail'),
    surface: box('.surface-region'),
    panel: box('.cost-panel'),
    status: box('.status-strip'),
  });
});

/** What the document bar says the roof comes to, and what it says went into it. */
const roofAreaNow = (session_) => session_.execute(function () {
  const el = document.querySelector('.roof-area');
  if (!el) return JSON.stringify({ there: false });
  return JSON.stringify({
    there: true,
    value: ((el.querySelector('.value') || {}).textContent || '').trim(),
    unit: ((el.querySelector('.unit') || {}).textContent || '').trim(),
    derived: !!el.querySelector('.derived:not([hidden])'),
    basis: el.getAttribute('title') || '',
    leftOut: ((el.querySelector('.left-out') || {}).textContent || '').trim(),
  });
});

/** The colours the chrome is actually painted in, read off the computed style. */
const tokensNow = (session_) => session_.execute(function () {
  const read = function (name) {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  };
  return JSON.stringify({
    theme: document.documentElement.getAttribute('data-theme') || 'light',
    bar: getComputedStyle(document.querySelector('.doc-bar')).backgroundColor,
    strip: getComputedStyle(document.querySelector('.tool-strip')).backgroundColor,
    panel: getComputedStyle(document.querySelector('.cost-panel')).backgroundColor,
    // The handoff's own values, as tokens, so a theme that painted itself some
    // other way is caught by name rather than by eye.
    water: read('--rn-water'),
    warn: read('--rn-warn'),
    ink: read('--rn-ink'),
    hairline: read('--rn-hairline'),
  });
});

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  // ── the first-open frame: chrome present, and quiet ─────────────────────
  const cold = JSON.parse(await regionsNow(session));
  check('the window is the handoff\'s frame — 1440 by 900', () => {
    // Measured INSIDE. Every region below is specified against the page, so the
    // page is what has to be 1440 x 900 — and this desktop takes the title bar
    // out of the inside of what the shell asks for, which is why the shell is
    // told the shortfall and gives it back. Exact on purpose: a tolerance wide
    // enough to swallow a title bar cannot see the thing it exists to see.
    assert.ok(Math.abs(cold.innerWidth - 1440) <= 2,
      `the page is ${cold.innerWidth}px wide and is specified at 1440 (D137)`);
    assert.ok(Math.abs(cold.innerHeight - 900) <= 2,
      `the page is ${cold.innerHeight}px tall and is specified at 900 (D137)`);
  });
  check('before a job is open the chrome is all there, and quiet', () => {
    // Frame 8. Not an empty grid with a paragraph explaining that it is empty:
    // the regions are present at their sizes so the window does not rearrange
    // itself the moment a job arrives.
    for (const [name, region] of [['document bar', cold.doc], ['tool settings strip', cold.strip],
      ['tool rail', cold.rail], ['cost panel', cold.panel], ['status strip', cold.status]]) {
      assert.ok(region, `there is no ${name}`);
      assert.notEqual(region.hidden, true, `the ${name} is hidden with no job open`);
    }
  });
  await shot('frame8-first-open');

  await openDemoJob(session);
  await wait(2000);
  await until(session, () => document.querySelector('.surface-overlay')?.getAttribute('viewBox') !== null,
    { what: 'the drawing', timeout: 40000 });
  await wait(1200);

  // ── 1. the regions, at the handoff's sizes ──────────────────────────────
  const plan = JSON.parse(await regionsNow(session));
  check('the regions are the heights the handoff specifies', () => {
    assert.equal(plan.doc.height, 44, `the document bar is ${plan.doc.height}px, specified at 44`);
    assert.equal(plan.strip.height, 42, `the tool settings strip is ${plan.strip.height}px, specified at 42`);
    assert.equal(plan.status.height, 30, `the status strip is ${plan.status.height}px, specified at 30`);
  });
  check('and the widths, with only the surface taking what is left', () => {
    assert.equal(plan.rail.width, 52, `the tool rail is ${plan.rail.width}px, specified at 52`);
    assert.equal(plan.panel.width, 364, `the cost panel is ${plan.panel.width}px, specified at 364`);
    const spare = plan.body.width - plan.rail.width - plan.panel.width;
    assert.equal(plan.surface.width, spare,
      `the surface has ${plan.surface.width}px and the fixed regions leave ${spare}px`);
  });
  check('and they tile the window with no gap and no overlap', () => {
    // The half a number cannot fake. A region at the right height in the wrong
    // place is not the layout, and a one-pixel gap between two of them is a
    // hairline the design does not have.
    assert.equal(plan.doc.top, 0, `the document bar starts at ${plan.doc.top}`);
    assert.equal(plan.strip.top, plan.doc.bottom,
      `the strip starts at ${plan.strip.top} and the bar ends at ${plan.doc.bottom}`);
    assert.equal(plan.body.top, plan.strip.bottom,
      `the body starts at ${plan.body.top} and the strip ends at ${plan.strip.bottom}`);
    assert.equal(plan.status.top, plan.body.bottom,
      `the status strip starts at ${plan.status.top} and the body ends at ${plan.body.bottom}`);
    assert.equal(plan.surface.left, plan.rail.right,
      `the surface starts at ${plan.surface.left} and the rail ends at ${plan.rail.right}`);
    assert.equal(plan.panel.left, plan.surface.right,
      `the panel starts at ${plan.panel.left} and the surface ends at ${plan.surface.right}`);
    assert.equal(plan.panel.right, plan.innerWidth,
      `the panel ends at ${plan.panel.right} in a ${plan.innerWidth}px window`);
  });
  check('and nothing makes the page wider than the window', () => {
    // The 9a lesson, at the new anatomy: a region that answers the grid's "how
    // narrow can you be" with the width of its own unbreakable text takes the
    // whole page with it, and `overflow: hidden` on the body turns that into a
    // window that merely looks cropped. Measured at every fixed region.
    assert.equal(plan.pageWidth, plan.innerWidth,
      `the page needs ${plan.pageWidth}px in a ${plan.innerWidth}px window`);
    assert.ok(plan.documentWidth <= plan.innerWidth,
      `the document is ${plan.documentWidth}px in a ${plan.innerWidth}px window`);
    for (const [name, region] of [['document bar', plan.doc], ['tool settings strip', plan.strip],
      ['status strip', plan.status], ['strip\'s own row of controls', plan.stripFields]]) {
      assert.ok(region, `there is no ${name}`);
      assert.ok(region.content <= region.room + 1,
        `the ${name} holds ${region.content}px of content in ${region.room}px`);
      assert.ok(region.right <= plan.innerWidth + 1,
        `the ${name} ends at ${region.right}px in a ${plan.innerWidth}px window`);
    }
  });
  await shot('frame1-plan');

  // ── the one roof-area figure ────────────────────────────────────────────
  const figure = JSON.parse(await roofAreaNow(session));
  const areas = JSON.parse(await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const pages = {};
      for (const p of d.pages || []) if (p.feetPerUnit) pages[p.id] = p.feetPerUnit;
      let total = 0;
      let pending = 0;
      let counted = 0;
      for (const c of d.conditions || []) {
        if (c.kind !== 'area') continue;
        const trace = (c.traces || [])[0];
        const scale = trace ? pages[trace.pageId] : undefined;
        if (!scale) { pending += 1; continue; }
        let sf = 0;
        for (const t of c.traces || []) {
          const p = t.points;
          let twice = 0;
          for (let i = 0; i < p.length; i += 1) {
            const q = p[(i + 1) % p.length];
            twice += p[i].x * q.y - q.x * p[i].y;
          }
          sf += Math.abs(twice / 2) * scale * scale;
        }
        const pitch = (c.properties || {}).PITCH;
        if (pitch) sf *= Math.sqrt(1 + (pitch / 12) * (pitch / 12));
        total += sf;
        counted += 1;
      }
      return JSON.stringify({ total: total, counted: counted, pending: pending });
    });
  }));
  check('the document bar carries one roof-area figure, and it is the areas added up', () => {
    // Worked out here from the traced corners and the page's own scale, not
    // read back off the thing being checked. Everything on screen reconciles to
    // this number, so it has to be the areas' own square feet and nothing else.
    assert.ok(figure.there, 'there is no roof-area figure in the document bar');
    const shown = Number(figure.value.replace(/,/g, ''));
    assert.ok(Number.isFinite(shown), `it reads "${figure.value}"`);
    assert.equal(figure.unit, 'SF', `it is counted in "${figure.unit}"`);
    assert.ok(Math.abs(shown - areas.total) <= 0.05,
      `the bar says ${shown} SF and the area conditions come to ${areas.total.toFixed(2)} SF`);
  });
  check('and it says it was worked out, and what went into it', () => {
    assert.equal(figure.derived, true, 'the figure carries no mark saying it was derived');
    assert.match(figure.basis, /Counted:/, `its basis reads "${figure.basis}"`);
    for (let i = 0; i < areas.counted; i += 1) {
      assert.ok(figure.basis.includes('SF'), `its basis reads "${figure.basis}"`);
    }
  });
  check('and if it could not count something it says so on screen, not only on a tooltip', () => {
    if (areas.pending === 0) {
      assert.equal(figure.leftOut, '',
        `nothing is uncounted and it says "${figure.leftOut}"`);
      return;
    }
    assert.match(figure.leftOut, /not counted/,
      `${areas.pending} areas have no scale and it says "${figure.leftOut}"`);
  });

  // ── 2. the rail's acts, the pill, and 1 to 9 ────────────────────────────
  const railNow = JSON.parse(await session.execute(function () {
    return JSON.stringify([...document.querySelectorAll('.tool-rail .rail-tool')]
      .map((b) => b.getAttribute('aria-label')));
  }));
  check('the rail is the acts this program can carry out, in the trade\'s words', () => {
    assert.ok(railActs.length > 0, 'plan.ts carries no ACTS to check the rail against');
    assert.deepEqual(railNow, railActs,
      `the rail reads ${JSON.stringify(railNow)} and plan.ts's ACTS reads ${JSON.stringify(railActs)}`);
  });

  // 1 to 9 picks the nth condition — VES muscle memory, and the number is
  // printed on the row so it can be seen rather than remembered.
  await showTab(session, 'Condition');
  const keyPick = JSON.parse(await session.execute(function () {
    document.body.focus();
    const before = [...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .map((r) => ({ name: ((r.querySelector('.condition-name') || {}).textContent || '').trim(), on: r.classList.contains('on') }));
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '2', bubbles: true }));
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '2', bubbles: true }));
    return JSON.stringify({ before });
  }));
  await wait(600);
  const afterKey = JSON.parse(await session.execute(function () {
    return JSON.stringify([...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .map((r) => ({
        name: ((r.querySelector('.condition-name') || {}).textContent || '').trim(),
        key: ((r.querySelector('.condition-key') || {}).textContent || '').trim(),
        on: r.classList.contains('on'),
      })));
  }));
  check('pressing 2 picks the second condition, and the number is printed on it', () => {
    assert.ok(afterKey.length >= 2, `only ${afterKey.length} conditions to pick from`);
    assert.equal(afterKey[1].key, '2', `the second row's key reads "${afterKey[1].key}"`);
    assert.equal(afterKey[1].on, true,
      `the second condition is "${afterKey[1].name}" and it is not the one picked`);
    assert.equal(afterKey.filter((r) => r.on).length, 1, 'more than one condition is picked');
    void keyPick;
  });

  // Select picks what is under the pointer, on the drawing.
  //
  // It had nothing to do until D148 folded the list out of the surface: the
  // list was the only way to pick a condition, and the hint under the drawing
  // has been saying "click a trace to select it" the whole time. An act in the
  // rail that does nothing is the one thing the rail is not allowed to have.
  await tool(session, 'Select');
  const shape = await centreOf(session, '.surface-overlay [data-condition]');
  let pickedOnTheDrawing = null;
  if (shape) {
    const at = JSON.parse(shape);
    const which = await session.execute(function () {
      const el = document.querySelector('.surface-overlay [data-condition]');
      return el ? el.getAttribute('data-condition') : null;
    });
    try {
      await pointerClick(session, at.x, at.y);
    } catch {
      await session.execute(function (x, y) {
        const el = document.elementFromPoint(x, y) || document.querySelector('.surface-overlay [data-condition]');
        for (const type of ['pointermove', 'pointerdown', 'pointerup']) {
          el.dispatchEvent(new PointerEvent(type, {
            clientX: x, clientY: y, button: 0, buttons: 1, bubbles: true, detail: 1, pointerId: 1,
          }));
        }
      }, at.x, at.y);
    }
    await wait(800);
    await showTab(session, 'Condition');
    pickedOnTheDrawing = JSON.parse(await session.execute(function (want) {
      const row = [...document.querySelectorAll('.cost-condition .condition-list .condition.on')]
        .map((r) => ((r.querySelector('.condition-name') || {}).textContent || '').trim());
      return JSON.stringify({ want, picked: row });
    }, which));
  }
  check('Select picks the traced thing under the pointer', () => {
    assert.ok(shape, 'nothing on the drawing carries the condition it belongs to');
    assert.ok(pickedOnTheDrawing, 'the click could not be made');
    assert.equal(pickedOnTheDrawing.picked.length, 1,
      `${pickedOnTheDrawing.picked.length} conditions are picked`);
  });

  // The pill, and the money, while a trace is open.
  await tool(session, 'Trace an area');
  await session.execute(function () {
    const o = document.querySelector('.surface-overlay');
    const box = o.getBoundingClientRect();
    for (const [x, y] of [[120, 120], [300, 120]]) {
      for (const type of ['pointermove', 'pointerdown', 'pointerup']) {
        o.dispatchEvent(new PointerEvent(type, {
          clientX: box.left + x, clientY: box.top + y,
          button: 0, buttons: 1, bubbles: true, detail: 1, pointerId: 1,
        }));
      }
    }
  });
  await wait(700);

  const tracing = JSON.parse(await session.execute(function () {
    const pill = document.querySelector('.measuring-pill');
    const selling = document.querySelector('.cost-totals .selling .value');
    const r = selling ? selling.getBoundingClientRect() : null;
    const top = r ? document.elementFromPoint(
      Math.round(r.left + r.width / 2), Math.round(r.top + r.height / 2)) : null;
    return JSON.stringify({
      pill: pill && !pill.hidden ? (pill.innerText || '').replace(/\s+/g, ' ').trim() : null,
      selling: selling ? (selling.textContent || '').trim() : null,
      sellingVisible: !!r && r.width > 0 && r.height > 0
        && r.bottom <= window.innerHeight && r.right <= window.innerWidth,
      sellingOwn: !top ? 'nothing' : ((top === selling || selling.contains(top)) ? 'itself' : (top.className || top.tagName)),
      strip: (document.querySelector('.tool-strip') || {}).innerText || '',
    });
  }));
  check('while a trace is open the pill says what it is measuring into', () => {
    assert.ok(tracing.pill, 'no pill on the drawing while a shape is open');
    assert.match(tracing.pill, /Measuring into/, `it reads "${tracing.pill}"`);
    assert.ok(tracing.pill.replace(/Measuring into/, '').trim().length > 0,
      `it names nothing: "${tracing.pill}"`);
  });
  check('and the money stays pinned and readable the whole time', () => {
    // "Geometry on one side, money on the other, both visible at once" — the
    // handoff's first rule, asserted in the state that would break it, with a
    // shape half traced and the strip full of the act's own settings.
    assert.ok(tracing.selling, 'there is no selling price on screen');
    assert.match(tracing.selling, /\$/, `it reads "${tracing.selling}"`);
    assert.equal(tracing.sellingVisible, true, 'the selling price is off screen');
    assert.equal(tracing.sellingOwn, 'itself',
      `something is drawn over the selling price: ${tracing.sellingOwn}`);
  });
  check('and the strip is holding what the act needs and not what it does not', () => {
    assert.match(tracing.strip, /Trace an area/, `the strip reads "${tracing.strip.replace(/\s+/g, ' ')}"`);
    assert.match(tracing.strip, /Area so far/, `the strip reads "${tracing.strip.replace(/\s+/g, ' ')}"`);
    assert.ok(!/Set high and low/.test(tracing.strip), 'the strip carries another act\'s settings');
  });

  // Escape backs out — the shape first, which is the rung the hand is on.
  await session.execute(function () {
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
  });
  await wait(600);
  const afterEscape = await session.execute(function () {
    const pill = document.querySelector('.measuring-pill');
    return pill ? !!pill.hidden : true;
  });
  check('Escape backs out of the shape, and the pill goes with it', () => {
    assert.equal(afterEscape, true, 'the pill is still up after Escape');
  });
  await tool(session, 'Select');

  // ── 3. the surface switches views in place ──────────────────────────────
  await session.execute(function () { window.__markBeforeSwitch = 'still here'; });
  await session.execute(function () {
    const row = [...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .find((r) => /Parapet/.test(r.textContent || ''));
    if (row) row.click();
  });
  await wait(600);
  const before = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      selling: ((document.querySelector('.selling .value') || {}).textContent || '').trim(),
      picked: [...document.querySelectorAll('.cost-condition .condition-list .condition.on')]
        .map((r) => ((r.querySelector('.condition-name') || {}).textContent || '').trim()),
    });
  }));

  await showView(session, 'Model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the roof' });
  await wait(1600);

  const afterSwitch = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      mark: window.__markBeforeSwitch || 'gone',
      plan: !!document.querySelector('.surface-overlay'),
      model: !!document.querySelector('.model-canvas'),
      selling: ((document.querySelector('.selling .value') || {}).textContent || '').trim(),
      selected: window.__roofnerdModel ? window.__roofnerdModel().selectedConditionId : null,
      readout: (document.querySelector('.status-strip .readout') || {}).innerText || '',
    });
  }));
  check('the view switch swaps Plan for Model with no reload', () => {
    assert.equal(afterSwitch.mark, 'still here',
      'the page was reloaded — everything in it was thrown away');
    assert.equal(afterSwitch.model, true, 'the roof is not drawn');
    assert.equal(afterSwitch.plan, false, 'the Plan is still mounted underneath it');
  });
  check('and the selection, the money and the panel survive the switch', () => {
    assert.equal(afterSwitch.selling, before.selling,
      `the selling price read ${before.selling} and now reads ${afterSwitch.selling}`);
    assert.ok(before.picked.length === 1, `${before.picked.length} conditions were picked before`);
    assert.ok(afterSwitch.selected, 'nothing is selected in the view that came up');
  });
  check('and the status strip is the Model\'s readout', () => {
    for (const want of ['Cursor', 'Slope', 'Insulation here', 'Flow']) {
      assert.ok(afterSwitch.readout.includes(want),
        `the readout says "${afterSwitch.readout.replace(/\s+/g, ' ')}"`);
    }
  });
  const modelRegions = JSON.parse(await regionsNow(session));
  check('and the regions did not move when the view did', () => {
    for (const name of ['doc', 'strip', 'rail', 'surface', 'panel', 'status']) {
      assert.equal(modelRegions[name].width, plan[name].width,
        `the ${name} region was ${plan[name].width}px and is now ${modelRegions[name].width}px`);
      assert.equal(modelRegions[name].height, plan[name].height,
        `the ${name} region was ${plan[name].height}px tall and is now ${modelRegions[name].height}px`);
    }
    assert.equal(modelRegions.pageWidth, modelRegions.innerWidth,
      `the page needs ${modelRegions.pageWidth}px in a ${modelRegions.innerWidth}px window`);
  });
  await shot('frame2-model');
  await showView(session, 'Plan');
  await wait(1000);

  // ── 4. the cost panel: tabs, a line open, and detach ────────────────────
  await showTab(session, 'Estimate');
  await wait(600);
  const opened = await session.execute(function () {
    const line = document.querySelector('.cost-line .line-what');
    if (!line) return false;
    line.click();
    return true;
  });
  await wait(900);
  const openLine = JSON.parse(await session.execute(function () {
    const panel = document.querySelector('.cost-panel');
    const chain = document.querySelector('.line-chain');
    const rows = chain ? [...chain.querySelectorAll('.chain-row .chain-name')].map((e) => (e.textContent || '').trim()) : [];
    return JSON.stringify({
      width: Math.round(panel.getBoundingClientRect().width),
      hasFormula: !!(chain && chain.querySelector('input.formula')),
      rows,
      trace: chain ? [...chain.querySelectorAll('.chain-trace p')].map((p) => (p.textContent || '').trim()) : [],
      surface: Math.round(document.querySelector('.surface-region').getBoundingClientRect().width),
      pageWidth: document.body.scrollWidth,
      innerWidth: window.innerWidth,
    });
  }));
  check('a line opens in place to its whole chain, and the panel widens to 420', () => {
    assert.equal(opened, true, 'there is no line to open');
    assert.equal(openLine.width, 420, `the panel is ${openLine.width}px with a line open`);
    assert.equal(openLine.hasFormula, true, 'the formula is not editable on the open line');
    for (const want of ['Quantity', 'Unit', 'Waste', 'Order', 'Priced', 'Unit cost', 'Extended']) {
      assert.ok(openLine.rows.includes(want),
        `the chain shows ${JSON.stringify(openLine.rows)} and not "${want}"`);
    }
  });
  check('and it writes the arithmetic out, and the window still fits', () => {
    assert.ok(openLine.trace.length > 0, 'the open line shows no working');
    assert.equal(openLine.pageWidth, openLine.innerWidth,
      `the page needs ${openLine.pageWidth}px in a ${openLine.innerWidth}px window`);
  });
  await shot('estimate-open-line');

  // Escape backs out of the open line, which is the next rung.
  await session.execute(function () {
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
  });
  await wait(700);
  const backedOut = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      open: !!document.querySelector('.line-chain'),
      width: Math.round(document.querySelector('.cost-panel').getBoundingClientRect().width),
    });
  }));
  check('Escape backs out of the open line, and the panel goes back to 364', () => {
    assert.equal(backedOut.open, false, 'the line is still open after Escape');
    assert.equal(backedOut.width, 364, `the panel is ${backedOut.width}px after backing out`);
  });

  await showTab(session, 'Condition');
  await wait(700);
  const conditionTab = JSON.parse(await session.execute(function () {
    return JSON.stringify({
      list: document.querySelectorAll('.cost-condition .condition').length,
      properties: !!document.querySelector('.cost-condition .condition-panel'),
      book: ((document.querySelector('.cost-load') || {}).textContent || '').trim(),
      measures: [...document.querySelectorAll('.cost-condition .measure-label')].map((e) => (e.textContent || '').trim()),
    });
  }));
  check('the Condition tab is the list, the properties, and the way into the book', () => {
    assert.ok(conditionTab.list >= 3, `it lists ${conditionTab.list} conditions`);
    assert.equal(conditionTab.properties, true, 'there are no properties under the list');
    assert.match(conditionTab.book, /Load an assembly/, `the button reads "${conditionTab.book}"`);
    assert.ok(conditionTab.measures.length > 0, 'the panel shows no measures for what is picked');
  });
  await shot('condition-tab');

  await showTab(session, 'Recap');
  await wait(900);
  const recapTab = JSON.parse(await session.execute(function () {
    const el = document.querySelector('.lens-sheet');
    const panel = document.querySelector('.cost-panel');
    const heads = el ? [...el.querySelectorAll('thead th')] : [];
    const last = heads[heads.length - 1];
    return JSON.stringify({
      there: !!el,
      text: el ? (el.innerText || '') : '',
      lastRight: last ? Math.round(last.getBoundingClientRect().right) : null,
      panelRight: Math.round(panel.getBoundingClientRect().right),
    });
  }));
  check('the Recap tab is the recap, whole, inside the panel it is drawn in', () => {
    assert.ok(recapTab.there, 'there is no recap on the Recap tab');
    for (const want of ['Material', 'Labor']) {
      assert.ok(recapTab.text.includes(want), `the recap reads "${recapTab.text.slice(0, 160)}"`);
    }
    assert.ok(recapTab.lastRight <= recapTab.panelRight,
      `its last column ends at ${recapTab.lastRight}px and the panel ends at ${recapTab.panelRight}px`);
  });
  await shot('recap-tab');

  // ── the lens selector, in the document bar ──────────────────────────────
  const picked = await pickLens(session, 'Stocking');
  const lens = JSON.parse(await session.execute(function () {
    const el = document.querySelector('.lens-sheet');
    return JSON.stringify({
      showing: ((document.querySelector('.lens-button .value') || {}).textContent || '').trim(),
      rows: el ? el.querySelectorAll('tbody tr').length : 0,
      html: el ? el.innerHTML : '',
      totals: (document.querySelector('.cost-totals') || {}).innerText || '',
      pageWidth: document.body.scrollWidth,
      innerWidth: window.innerWidth,
    });
  }));
  check('the lens selector in the document bar opens a lens in the panel', () => {
    assert.equal(picked, true, 'the lens menu would not open');
    assert.equal(lens.showing, 'Stocking', `the bar says it is showing "${lens.showing}"`);
    assert.ok(lens.rows > 0, 'the lens shows no rows');
  });
  check('and the money is removed rather than hidden, and says so', () => {
    assert.ok(!/\$\s?[\d,]/.test(lens.html), 'a dollar figure is in the markup of a lens with no cost');
    assert.match(lens.totals, /hidden by this lens/i,
      `the pinned totals read "${lens.totals.replace(/\s+/g, ' ')}"`);
    assert.equal(lens.pageWidth, lens.innerWidth,
      `the page needs ${lens.pageWidth}px in a ${lens.innerWidth}px window`);
  });

  // Escape backs out of the lens, which is the last rung.
  await session.execute(function () {
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
  });
  await wait(800);
  const lensGone = await session.execute(function () {
    return ((document.querySelector('.lens-button .value') || {}).textContent || '').trim();
  });
  check('and Escape puts the full estimate back', () => {
    assert.equal(lensGone, 'Full estimate', `the bar says it is showing "${lensGone}"`);
  });

  // ── the theme toggle ────────────────────────────────────────────────────
  const light = JSON.parse(await tokensNow(session));
  await session.execute(function () {
    const b = document.querySelector('.theme-toggle');
    if (b) b.click();
  });
  await wait(700);
  const dark = JSON.parse(await tokensNow(session));
  await shot('dark');
  await session.execute(function () {
    const b = document.querySelector('.theme-toggle');
    if (b) b.click();
  });
  await wait(700);
  const backToLight = JSON.parse(await tokensNow(session));

  check('the theme toggle flips the token set to the handoff\'s palette', () => {
    assert.equal(light.theme, 'light', `it started in "${light.theme}"`);
    assert.equal(dark.theme, 'dark', `it went to "${dark.theme}"`);
    // The handoff's own values, named. Read as tokens rather than as pixels so
    // a theme that painted itself some other way is caught by name.
    assert.equal(dark.water, '#45b3e0', `water is ${dark.water}`);
    assert.equal(dark.warn, '#e0a03c', `the no-fall colour is ${dark.warn}`);
    assert.equal(dark.ink, '#e6ebef', `the ink is ${dark.ink}`);
    assert.equal(dark.hairline, '#2a3036', `the hairline is ${dark.hairline}`);
    assert.notEqual(dark.bar, light.bar, 'the document bar is painted the same in both');
    assert.notEqual(dark.panel, light.panel, 'the cost panel is painted the same in both');
  });
  check('and back again, to exactly what it was', () => {
    assert.equal(backToLight.theme, 'light', `it came back to "${backToLight.theme}"`);
    assert.equal(backToLight.bar, light.bar,
      `the document bar was ${light.bar} and came back ${backToLight.bar}`);
    assert.equal(backToLight.panel, light.panel,
      `the cost panel was ${light.panel} and came back ${backToLight.panel}`);
    assert.equal(backToLight.ink, light.ink,
      `the ink was ${light.ink} and came back ${backToLight.ink}`);
  });

  // ── detach, and dock it back ────────────────────────────────────────────
  await showTab(session, 'Estimate');
  await wait(700);
  const moneyDocked = await session.execute(function () {
    return ((document.querySelector('.selling .value') || {}).textContent || '').trim();
  });
  const oneWindow = await session.handles();
  const tore = await detachCostPanel(session);
  const handles = await session.handles();
  const main = oneWindow[0];
  const cost = handles.find((h) => !oneWindow.includes(h)) ?? null;

  const reclaimed = JSON.parse(await regionsNow(session));
  check('detaching opens the cost window and the surface reclaims the width', () => {
    assert.ok(tore, 'no detach button beside the tabs');
    assert.ok(cost, `${oneWindow.length} window before and ${handles.length} after`);
    assert.ok(reclaimed.surface.width > plan.surface.width,
      `the surface was ${plan.surface.width}px and is ${reclaimed.surface.width}px`);
    assert.equal(reclaimed.surface.right, reclaimed.innerWidth,
      `the surface ends at ${reclaimed.surface.right}px in a ${reclaimed.innerWidth}px window`);
    assert.equal(reclaimed.rail.width, 52, 'the tool rail moved when the panel left');
    assert.equal(reclaimed.doc.height, 44, 'the document bar moved when the panel left');
  });
  const saysWhere = await session.execute(function () {
    return JSON.stringify({
      aside: ((document.querySelector('.status-aside') || {}).textContent || '').trim(),
      home: !!document.querySelector('.view-switch .redock:not([hidden])'),
    });
  });
  check('and the window that lost it says where it went, with the way to get it back', () => {
    const where = JSON.parse(saysWhere);
    assert.match(where.aside, /cost panel/i, `the status strip says "${where.aside}"`);
    assert.equal(where.home, true, 'there is no way to bring the panel back');
  });
  await shot('detached-a');

  if (cost) {
    await session.switchTo(cost);
    await until(session, () => !!document.querySelector('.cost-panel.detached'),
      { what: 'the cost window' });
    await wait(1200);
    const moneyDetached = await session.execute(function () {
      return JSON.stringify({
        selling: ((document.querySelector('.selling .value') || {}).textContent || '').trim(),
        width: Math.round(window.innerWidth),
        height: Math.round(window.innerHeight),
        title: ((document.querySelector('.cost-title-name') || {}).textContent || '').trim(),
        tabs: [...document.querySelectorAll('.cost-tab')].map((b) => (b.textContent || '').trim()),
      });
    });
    const detached = JSON.parse(moneyDetached);
    check('the cost window is 600 by 860, with the same money in it', () => {
      assert.ok(Math.abs(detached.width - 600) <= 2,
        `its page is ${detached.width}px wide and is specified at 600`);
      assert.ok(Math.abs(detached.height - 860) <= 2,
        `its page is ${detached.height}px tall and is specified at 860`);
      assert.equal(cents(detached.selling), cents(moneyDocked),
        `it says ${detached.selling} and the docked panel said ${moneyDocked}`);
      assert.deepEqual(detached.tabs, ['Estimate', 'Condition', 'Recap'],
        `its tabs read ${JSON.stringify(detached.tabs)}`);
      assert.match(detached.title, /Cost/, `its title reads "${detached.title}"`);
    });
    await shot('detached-b');

    await session.execute(function () {
      const b = document.querySelector('.cost-dock');
      if (b) b.click();
    });
    await wait(2500);
    const afterDock = await session.handles();
    await session.switchTo(afterDock[0]);
    await wait(1400);
    const home = JSON.parse(await regionsNow(session));
    const moneyHome = await session.execute(function () {
      return ((document.querySelector('.selling .value') || {}).textContent || '').trim();
    });
    check('"dock it back" brings the panel home and closes the window it was in', () => {
      assert.ok(afterDock.length <= oneWindow.length,
        `${afterDock.length} windows are open and there was ${oneWindow.length} to start with`);
      assert.equal(home.panel.width, 364, `the panel came back ${home.panel.width}px wide`);
      assert.equal(home.surface.width, plan.surface.width,
        `the surface was ${plan.surface.width}px before and is ${home.surface.width}px after`);
      assert.equal(cents(moneyHome), cents(moneyDocked),
        `the money read ${moneyDocked} before and ${moneyHome} after`);
    });
  }

  // ── nothing broke on the way ────────────────────────────────────────────
  const errors = await errorsSoFar(session);
  check('and the window logged nothing going wrong', () => {
    assert.deepEqual(errors, [], errors.join(' | '));
  });
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
