// ── Section 6b's done-check: the panel stands beside everything ────────────
// The claim has not changed and its anatomy has. A3 asked for "the same panel
// in every editor"; the handoff has one surface with two views and a cost panel
// beside it, and the Properties are that panel's Condition tab. So what has to
// be true is the same thing said at the new regions: whatever the surface is
// showing, and in the panel's own window when it is on the other monitor, the
// same condition reads with the same measures and the same money — once, at the
// panel's width, beside the surface and never over it.
//
// The states that are easy to get wrong are the same ones: nothing picked, and
// the same condition after the window has been made to change what it shows.
// Switching is no longer a reload, which removes one failure and adds another:
// a view or a tab left mounted twice would show two of everything.
//
// It also detaches the panel, which is the window this program opens now. The
// Model window it used to tear off has no affordance any more — the detach
// button belongs to the panel — so the capabilities claim it proved is proved
// here instead, on the window that exists: `open_editor` names a window after
// what is in it, and a label outside `capabilities/default.json` is a window
// with no way to ask for the job.
//
//   pnpm build:app && node tools/probe/section6b-panel.mjs

import { mkdir, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, detachCostPanel, errorsSoFar, launch, openDemoJob, pickLens, pointerClick,
  pointerRelease, showTab, showView, typeKeys, until, wait, watchErrors, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

/**
 * Which path the keyboard and the pointer took, so no row can claim real input
 * it never had. Settled below on a click whose outcome is already known.
 */
let pointerPath = 'not tried';
let pointerRefused = '';

/**
 * Everywhere the window can be put, and how a person puts it there.
 *
 * The Properties are the Condition tab, so getting back to them is part of each
 * one — which is the point: the panel has to come back to the same condition
 * with the same numbers however the window was moved around in between.
 */
const PLACES = [
  { title: 'the Model', shows: '.model-canvas', go: async (s_) => { await showView(s_, 'Model'); } },
  { title: 'the Estimate tab', shows: '.cost-estimate', go: async (s_) => { await showTab(s_, 'Estimate'); } },
  { title: 'the Recap tab', shows: '.lens-sheet', go: async (s_) => { await showTab(s_, 'Recap'); } },
  {
    title: 'the Library',
    shows: '.library-body',
    go: async (s_) => {
      await showTab(s_, 'Condition');
      await s_.execute(function () {
        const b = document.querySelector('.cost-load');
        if (b && !b.disabled) b.click();
      });
      await wait(1200);
    },
  },
  { title: 'a lens', shows: '.lens-sheet', go: async (s_) => { await pickLens(s_, 'Stocking'); } },
];

/**
 * What the Properties panel says, and how many of them there are.
 *
 * The count matters on the Plan in particular: the panel used to be built by
 * that editor, and moving it into the area chrome without taking the old one
 * out would leave the Plan as the one window with two.
 */
const panelNow = (session) => session.execute(function () {
  const panel = document.querySelector('.cost-panel');
  const all = [...document.querySelectorAll('.condition-panel')];
  const el = all[0];
  const panelBox = panel ? panel.getBoundingClientRect() : null;
  const surface = document.querySelector('.surface-region');
  const surfaceBox = surface ? surface.getBoundingClientRect() : null;
  const shape = {
    panelThere: !!panel,
    panelHidden: panel ? !!panel.hidden : null,
    quiet: panel ? panel.classList.contains('quiet') : null,
    width: panelBox ? Math.round(panelBox.width) : null,
    right: panelBox ? Math.round(panelBox.right) : null,
    windowWidth: window.innerWidth,
    // The panel is beside the surface, not on top of it.
    overlaps: panelBox && surfaceBox ? Math.round(Math.max(0, surfaceBox.right - panelBox.left)) : null,
    surfaceWidth: surfaceBox ? Math.round(surfaceBox.width) : null,
    totals: ((document.querySelector('.cost-totals') || {}).innerText || '').replace(/\s+/g, ' ').trim(),
    tabRows: document.querySelectorAll('.cost-condition .condition-list .condition').length,
    // The surface is the drawing (D148). A list inside it would be the same
    // list twice, which is what this row exists to catch.
    listInSurface: document.querySelectorAll('.surface-region .condition').length,
  };
  if (!el) return JSON.stringify(Object.assign({ count: 0 }, shape));
  const measure = {};
  for (const cell of el.querySelectorAll('.panel-measures > div')) {
    const label = ((cell.querySelector('.measure-label') || {}).textContent || '').trim();
    measure[label] = ((cell.querySelector('.measure-value') || {}).textContent || '').trim();
  }
  return JSON.stringify(Object.assign({
    count: all.length,
    hidden: !!el.hidden,
    name: (el.querySelector('.panel-name') || {}).value || null,
    kind: ((el.querySelector('.panel-kind') || {}).textContent || '').trim(),
    measure,
    cost: ((el.querySelector('.panel-money .value') || {}).textContent || '').trim(),
    empty: ((el.querySelector('.panel-empty') || {}).textContent || '').trim(),
    text: (el.innerText || el.textContent || '').replace(/\s+/g, ' ').trim(),
  }, shape));
});

/** Come back to the Properties, wherever the window has been left. */
const showProperties = async (session) => {
  await showView(session, 'Plan');
  await pickLens(session, 'Full estimate');
  await showTab(session, 'Condition');
  await until(session, () => !!document.querySelector('.condition-panel'),
    { what: 'the Condition tab', timeout: 15000 }).catch(() => undefined);
  await wait(500);
};

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  // ── nothing open: no job, no condition, no panel ────────────────────────
  const cold = JSON.parse(await panelNow(session));
  check('on the start screen the panel is present and quiet, and says why', () => {
    // The ruling changed here and the handoff is the reason: Frame 8 keeps the
    // chrome on screen and quiet, with the panel saying in words that there is
    // nothing to price yet. A panel that vanished would make the window look
    // like it had lost something; a panel standing there blank would make it
    // look broken. This asserts BOTH halves — it is there, and it is not a
    // column of nothing.
    assert.equal(cold.panelThere, true, 'there is no cost panel on the start screen');
    assert.notEqual(cold.panelHidden, true, 'the panel is hidden with no job open');
    assert.equal(cold.quiet, true, 'the panel is at full strength with no job open');
    assert.match(cold.totals ?? '', /nothing priced yet|hidden by/i,
      `its totals read "${cold.totals}"`);
  });

  // ── the pointer, proven on a click whose outcome cannot be misread ──────
  // The same arrangement section 6 uses: the demo job is opened by clicking the
  // button WHERE IT IS, through the driver's actions endpoint. If the job opens,
  // real input works and the keystrokes below are keystrokes. If it does not,
  // the rows below say so in their names and fall back to dispatched events.
  const demoAt = await centreOf(session, '.start-demo');
  if (demoAt) {
    const at = JSON.parse(demoAt);
    try {
      await pointerClick(session, at.x, at.y);
      await wait(1500);
      const opened = await session.execute(function () { return !document.querySelector('.start'); });
      pointerPath = opened ? 'actions' : 'synthetic';
      if (!opened) pointerRefused = 'the driver took the click and the job did not open';
    } catch (e) {
      pointerPath = 'synthetic';
      pointerRefused = e.message;
    }
  } else {
    pointerPath = 'synthetic';
    pointerRefused = 'the demo button was not on screen to aim at';
  }
  console.log(`  pointer: ${pointerPath}${pointerRefused ? ` — ${pointerRefused}` : ''}`);
  // REPORTED, not asserted. Whether tauri-driver will dispatch a real pointer
  // is a fact about the harness on this box, not about the program: 6c found
  // WebKitWebDriver refusing all three shapes of a W3C click. A row that can
  // only ever be red says nothing about the build and trains the reader to skip
  // the list. What it costs is carried instead — every row below that depends
  // on real input says in its own name which path it took.
  console.log(`  REPORT  real pointer input: ${pointerPath === 'actions' ? 'available' : `refused — ${pointerRefused.split('\n')[0]}`}`);

  if (pointerPath !== 'actions') await openDemoJob(session);
  await until(session, () => !document.querySelector('.start'), { what: 'the job to open' });
  await wait(1500);

  // ── a job open, nothing picked ──────────────────────────────────────────
  await showTab(session, 'Condition');
  const fresh = JSON.parse(await panelNow(session));
  check('with a job open and nothing picked the panel says so', () => {
    assert.equal(fresh.count, 1, `${fresh.count} panels in one window`);
    assert.ok(!fresh.hidden, 'the panel is hidden with a job open');
    assert.match(fresh.empty ?? '', /Nothing picked/i,
      `it reads "${(fresh.empty || fresh.text || '').slice(0, 120)}"`);
  });
  check('and it is 364px wide, at the right, beside the surface and not over it', () => {
    // The handoff says 364 docked. The rest of this row is the part a number
    // cannot fake: the surface ends where the panel starts, and the panel ends
    // at the window.
    assert.equal(fresh.width, 364, `the panel is ${fresh.width}px wide`);
    assert.equal(fresh.overlaps, 0,
      `the surface and the panel overlap by ${fresh.overlaps}px`);
    assert.ok(fresh.right <= fresh.windowWidth,
      `the panel ends at ${fresh.right}px in a ${fresh.windowWidth}px window`);
    assert.ok(fresh.surfaceWidth > 400,
      `the surface is left with ${fresh.surfaceWidth}px`);
  });
  check('the conditions are listed where they live, and nowhere else', () => {
    // D132 took the tree away and D148 took the copy beside the drawing away
    // with it: the Condition tab is where conditions are, and the surface is
    // the drawing. Both halves — they are listed, and they are not listed
    // twice.
    assert.ok(fresh.tabRows >= 3, `the Condition tab lists ${fresh.tabRows} rows`);
    assert.equal(fresh.listInSurface, 0,
      `${fresh.listInSurface} conditions are listed inside the surface as well`);
  });

  // ── pick one, the way a person does ─────────────────────────────────────
  const picked = await session.execute(function () {
    const row = [...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .find(function (r) { return (r.textContent || '').indexOf('Parapet') >= 0; });
    if (!row) return null;
    row.click();
    return (row.textContent || '').replace(/\s+/g, ' ').trim();
  });
  await wait(900);
  const onPlan = JSON.parse(await panelNow(session));
  check('picking a condition fills the panel', () => {
    assert.ok(picked, 'the Condition tab lists no parapet to pick');
    assert.ok(onPlan.name && onPlan.name.length > 0, 'the panel names nothing');
    assert.match(onPlan.measure?.['Run'] ?? '', /\d/, `its run reads "${onPlan.measure?.['Run']}"`);
    assert.match(onPlan.cost ?? '', /\$|nothing priced/,
      `its cost line reads "${onPlan.cost}"`);
  });

  await writeFile(join(EVIDENCE, `section6b-panel-plan-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  const wanted = onPlan.name;
  const wantedMeasures = onPlan.measure;
  const wantedCost = onPlan.cost;

  // ── the window put in every state it has, and brought back ──────────────
  for (const [index, place] of PLACES.entries()) {
    await place.go(session);
    const showing = await session.execute(function (sel) {
      return !!document.querySelector(sel);
    }, place.shows);
    const whileThere = JSON.parse(await panelNow(session));

    check(`${place.title}: it is up`, () => {
      assert.ok(showing, `nothing matching ${place.shows} is in the window`);
    });
    check(`${place.title}: the panel is still one panel, beside the surface`, () => {
      // Nothing reloads any more, so the failure this guards against has changed
      // shape: a view or a tab left mounted while the next one goes up would
      // show two of everything, with two subscriptions writing into both.
      assert.ok(whileThere.panelThere, 'the cost panel is gone');
      assert.notEqual(whileThere.panelHidden, true, 'the cost panel is hidden');
      assert.ok(whileThere.count <= 1, `${whileThere.count} Properties panels in one window`);
      assert.equal(whileThere.overlaps, 0,
        `the surface and the panel overlap by ${whileThere.overlaps}px`);
      assert.ok(whileThere.right <= whileThere.windowWidth,
        `the panel ends at ${whileThere.right}px in a ${whileThere.windowWidth}px window`);
    });

    await writeFile(join(EVIDENCE, `section6b-panel-${index + 1}-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));

    await showProperties(session);
    const back = JSON.parse(await panelNow(session));

    check(`${place.title}: coming back, the panel names the same condition`, () => {
      // The row that would have caught a panel arriving with "pick a condition"
      // after the window was moved around. The selection lives in the window,
      // not in whatever was on screen a moment ago.
      assert.equal(back.count, 1, `${back.count} Properties panels in one window`);
      assert.equal(back.name, wanted, `the panel names "${back.name}"`);
      assert.equal(back.width, 364, `the panel is ${back.width}px wide`);
    });
    check(`${place.title}: with the same measures and the same money`, () => {
      for (const key of Object.keys(wantedMeasures)) {
        assert.equal(back.measure[key], wantedMeasures[key],
          `${key} reads "${back.measure[key]}" here and "${wantedMeasures[key]}" before`);
      }
      assert.equal(back.cost, wantedCost,
        `the cost reads "${back.cost}" here and "${wantedCost}" before`);
    });

    const errors = await errorsSoFar(session);
    check(`${place.title}: nothing errored while it was open`, () => {
      assert.deepEqual(errors, [], errors.join(' | '));
    });
  }

  // ── the panel, pulled onto the other monitor ────────────────────────────
  // The window this program opens now. `open_editor` names a window after what
  // is in it, and a label that is not in `capabilities/default.json` is a
  // window with no way to ask for the job — it would come up on the start
  // screen looking as though the program had lost it. "cost" is in the list.
  const oneWindow = await session.handles();
  const tore = await detachCostPanel(session);
  const handles = await session.handles();
  const torn = handles.find((h) => !oneWindow.includes(h)) ?? null;

  check('the cost panel detaches into a window of its own', () => {
    assert.ok(tore, 'no detach button beside the tabs');
    assert.ok(torn, `${oneWindow.length} window before and ${handles.length} after`);
  });

  const reclaimed = JSON.parse(await panelNow(session));
  check('and the surface reclaims the width it gave up', () => {
    // The handoff: "the viewport reclaims the width; nothing else in the layout
    // moves". Read as the surface being wider than it was with the panel beside
    // it, and the panel no longer standing in the main window at all.
    assert.ok(reclaimed.surfaceWidth > fresh.surfaceWidth,
      `the surface was ${fresh.surfaceWidth}px with the panel and is ${reclaimed.surfaceWidth}px without it`);
  });

  if (torn) {
    await session.switchTo(torn);
    await wait(2500);
    const inTorn = JSON.parse(await session.execute(function () {
      return JSON.stringify({
        start: !!document.querySelector('.start'),
        panels: document.querySelectorAll('.cost-panel').length,
        title: ((document.querySelector('.cost-title-name') || {}).textContent || '').trim(),
        home: !!document.querySelector('.cost-dock'),
        docBar: !!document.querySelector('.doc-bar'),
        rail: !!document.querySelector('.tool-rail'),
        selling: ((document.querySelector('.selling .value') || {}).textContent || '').trim(),
        width: Math.round(window.innerWidth),
        height: Math.round(window.innerHeight),
      });
    }));

    check('and the detached window has the job in it, not the start screen', () => {
      assert.ok(!inTorn.start,
        'the detached window is on the start screen — it could not ask for the job');
      assert.equal(inTorn.panels, 1, `${inTorn.panels} cost panels in it`);
      assert.match(inTorn.title, /Cost/, `its title bar reads "${inTorn.title}"`);
      assert.match(inTorn.selling, /\$/, `it shows no money: "${inTorn.selling}"`);
    });
    check('and it is the panel and nothing else, with the way home on it', () => {
      assert.equal(inTorn.docBar, false, 'it carries a second document bar');
      assert.equal(inTorn.rail, false, 'it carries a second tool rail');
      assert.equal(inTorn.home, true, 'there is no way to dock it back');
    });

    await writeFile(join(EVIDENCE, `section6b-panel-detached-${COMMIT}.png`),
      Buffer.from(await session.screenshot(), 'base64'));

    // And home again, from the window it is in. The main window takes it back
    // and this one goes away — one panel, wherever it is.
    await session.execute(function () {
      const b = document.querySelector('.cost-dock');
      if (b) b.click();
    });
    await wait(2500);
    const afterDock = await session.handles();
    check('and "dock it back" sends it home and closes the window it was in', () => {
      assert.ok(afterDock.length <= oneWindow.length,
        `${afterDock.length} windows are open and there was ${oneWindow.length} to start with`);
    });
    await session.switchTo(afterDock[0]);
    await wait(1200);
    const homeAgain = JSON.parse(await panelNow(session));
    check('and the panel is standing beside the surface again', () => {
      assert.ok(homeAgain.panelThere, 'the panel did not come back');
      assert.notEqual(homeAgain.panelHidden, true, 'it came back hidden');
      assert.equal(homeAgain.width, 364, `it came back ${homeAgain.width}px wide`);
    });
  }

  // ── a field has to keep the second character ────────────────────────────
  //
  // Every field in this program writes to the document as it is typed, and
  // every editor rebuilds itself when the document changes — `replaceChildren`,
  // then the fields again from what the job now says. The element being typed
  // into is destroyed between one keystroke and the next, and the focus goes
  // with it, so a height typed as "12" is a height of 1 and the second key was
  // never anywhere. Nothing caught it because every check in this repository
  // sets `.value` and dispatches ONE input event, which is not what a hand does.
  //
  // So: focus the field the way a hand does, send one key, let the document come
  // back around, and send the second key to WHEREVER THE FOCUS NOW IS. That last
  // clause is the whole check. Sending it to the element the check is still
  // holding would pass on a program that had lost it.
  await wait(600);
  await showProperties(session);
  await watchErrors(session);

  /** Where the focus went, which is the diagnosis when a row below goes red. */
  const focusNow = () => session.execute(function () {
    const el = document.activeElement;
    if (!el) return 'none';
    return el.tagName + (el.className ? `.${String(el.className).split(' ')[0]}` : '');
  });

  /**
   * The second keystroke, to whatever has the focus. Never to a held element.
   *
   * It follows the path the FIRST key actually took, and if the driver refuses
   * a key in the middle of the pair it falls back rather than abandoning the
   * run — a row that cannot report is worse than a row that reports weakly.
   */
  let keysRefused = '';
  const secondKey = async (ch, path) => {
    if (path === 'keys') {
      try {
        await typeKeys(session, ch);
        return { path: 'keys' };
      } catch (e) {
        keysRefused = e.message;
      }
    }
    return JSON.parse(await session.execute(function (c) {
      const el = document.activeElement;
      if (!el || (el.tagName !== 'INPUT' && el.tagName !== 'TEXTAREA')) {
        return JSON.stringify({ path: 'synthetic', wentNowhere: true });
      }
      const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      setter.call(el, el.value + c);
      el.dispatchEvent(new Event('input', { bubbles: true }));
      return JSON.stringify({ path: 'synthetic', wentNowhere: false });
    }, ch));
  };

  // ── the panel's Height field ────────────────────────────────────────────
  const heightField = () => session.execute(function () {
    const label = [...document.querySelectorAll('.condition-panel label')]
      .find(function (l) { return ((l.querySelector('span') || {}).textContent || '').trim() === 'Height'; });
    const input = label ? label.querySelector('input') : null;
    if (!input) return null;
    const r = input.getBoundingClientRect();
    return JSON.stringify({
      x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2),
      value: input.value, focused: document.activeElement === input,
    });
  });

  /** A field is emptied by script, once, so the two keystrokes start from nothing. */
  const emptyTheHeight = () => session.execute(function () {
    const label = [...document.querySelectorAll('.condition-panel label')]
      .find(function (l) { return ((l.querySelector('span') || {}).textContent || '').trim() === 'Height'; });
    const input = label ? label.querySelector('input') : null;
    if (!input) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(input, '');
    input.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  });

  const heightWas = JSON.parse((await heightField()) ?? 'null');
  await emptyTheHeight();
  await wait(1400);

  const heightEmpty = JSON.parse((await heightField()) ?? 'null');
  let heightPath = 'not tried';
  if (heightEmpty) {
    if (pointerPath === 'actions') {
      try {
        await pointerClick(session, heightEmpty.x, heightEmpty.y);
        await wait(400);
        await typeKeys(session, '1');
        heightPath = 'keys';
      } catch (e) {
        keysRefused = e.message;
      }
    }
    if (heightPath !== 'keys') {
      await session.execute(function () {
        const label = [...document.querySelectorAll('.condition-panel label')]
          .find(function (l) { return ((l.querySelector('span') || {}).textContent || '').trim() === 'Height'; });
        const input = label ? label.querySelector('input') : null;
        if (!input) return false;
        input.focus();
        const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
        setter.call(input, '1');
        input.dispatchEvent(new Event('input', { bubbles: true }));
        return true;
      });
      heightPath = 'synthetic';
    }
    await wait(1400);
  }
  const heightFocus = await focusNow();
  const heightSecond = heightEmpty ? await secondKey('2', heightPath) : null;
  await wait(1400);

  const heightAfter = JSON.parse((await heightField()) ?? 'null');
  const heightInJob = await session.execute(function () {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      const c = d.conditions.find(function (x) { return x.id === 'c-parapet'; });
      return JSON.stringify((c.properties || {}).H ?? null);
    });
  });

  check(`the panel keeps the field being typed into — "12", not "1" (${heightPath})`, () => {
    assert.ok(heightEmpty, 'no Height field in the panel to type into');
    assert.equal(heightAfter && heightAfter.value, '12',
      `the field reads "${heightAfter && heightAfter.value}" after 1 then 2`
      + ` — after the first key the focus was on ${heightFocus}`
      + `${heightSecond && heightSecond.wentNowhere ? ' and the second key went nowhere' : ''}`);
    assert.equal(JSON.parse(heightInJob), 12,
      `the job holds ${heightInJob} for the parapet's height`);
  });

  // Put the height back to what the field was actually holding, and to nothing
  // else. The figure that used to stand here as a fallback was the warehouse
  // parapet's 1 ft; this demo's wall is 42 in, and a check that writes its own
  // idea of a height into the job leaves the fixture behind it wrong for every
  // row that runs after it.
  if (heightWas) {
    await session.execute(function (value) {
      const label = [...document.querySelectorAll('.condition-panel label')]
        .find(function (l) { return ((l.querySelector('span') || {}).textContent || '').trim() === 'Height'; });
      const input = label ? label.querySelector('input') : null;
      if (!input) return false;
      const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      setter.call(input, String(value));
      input.dispatchEvent(new Event('input', { bubbles: true }));
      return true;
    }, heightWas.value);
    await wait(1200);
  }

  // ── and the formula on the line, which is the field this program is for ─
  // It is edited where it always was — on the line — and the line is opened to
  // its whole chain to get at it, which is the act a person performs.
  await showTab(session, 'Estimate');
  // Which line was opened, in the words the row shows, so the row below can find
  // it in the document. It used to be looked up as the first item on the first
  // condition, which was the same line on the warehouse this demo used to be.
  // The roof it ships now carries thirteen lines on the field alone, and "the
  // first one" is an accident of document order rather than a claim worth making.
  const openedLine = await session.execute(function () {
    const line = document.querySelector('.cost-line .line-what');
    if (!line) return null;
    line.click();
    return ((line.querySelector('.line-item') || {}).textContent || '').trim();
  });
  await wait(900);

  const formulaField = () => session.execute(function () {
    const input = document.querySelector('.line-chain input.formula');
    if (!input) return null;
    const r = input.getBoundingClientRect();
    return JSON.stringify({
      x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2),
      value: input.value, focused: document.activeElement === input,
    });
  });
  const setFormula = (value) => session.execute(function (v) {
    const input = document.querySelector('.line-chain input.formula');
    if (!input) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(input, v);
    input.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  }, value);

  const formulaWas = JSON.parse((await formulaField()) ?? 'null');
  await setFormula('');
  await wait(1400);

  const formulaEmpty = JSON.parse((await formulaField()) ?? 'null');
  let formulaPath = 'not tried';
  if (formulaEmpty) {
    if (pointerPath === 'actions') {
      try {
        await pointerClick(session, formulaEmpty.x, formulaEmpty.y);
        await wait(400);
        await typeKeys(session, '1');
        formulaPath = 'keys';
      } catch (e) {
        keysRefused = e.message;
      }
    }
    if (formulaPath !== 'keys') {
      await session.execute(function () {
        const input = document.querySelector('.line-chain input.formula');
        if (!input) return false;
        input.focus();
        const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
        setter.call(input, '1');
        input.dispatchEvent(new Event('input', { bubbles: true }));
        return true;
      });
      formulaPath = 'synthetic';
    }
    await wait(1400);
  }
  const formulaFocus = await focusNow();
  const formulaSecond = formulaEmpty ? await secondKey('2', formulaPath) : null;
  await wait(1400);

  const formulaAfter = JSON.parse((await formulaField()) ?? 'null');
  const formulaInJob = await session.execute(function (what) {
    return window.__TAURI_INTERNALS__.invoke('doc_get').then(function (d) {
      for (const c of d.conditions) {
        for (const item of c.items || []) {
          if ((item.description || '') === what) return JSON.stringify(item.formula ?? null);
        }
      }
      return JSON.stringify(null);
    });
  }, openedLine);

  check(`the open line keeps the formula being typed into — "12", not "1" (${formulaPath})`, () => {
    assert.ok(openedLine, 'there is no line in the estimate to open');
    assert.ok(formulaEmpty, 'no formula field on the open line to type into');
    assert.equal(formulaAfter && formulaAfter.value, '12',
      `the field reads "${formulaAfter && formulaAfter.value}" after 1 then 2`
      + ` — after the first key the focus was on ${formulaFocus}`
      + `${formulaSecond && formulaSecond.wentNowhere ? ' and the second key went nowhere' : ''}`);
    assert.equal(JSON.parse(formulaInJob), '12',
      `the job holds ${formulaInJob} as the formula on "${openedLine}"`);
  });

  await writeFile(join(EVIDENCE, `section6b-typing-${COMMIT}.png`),
    Buffer.from(await session.screenshot(), 'base64'));

  // And put the formula back.
  await setFormula(formulaWas ? formulaWas.value : 'SQ');
  await wait(1200);

  console.log(`  keyboard: panel ${heightPath}, line ${formulaPath}`
    + `${keysRefused ? ` — ${keysRefused.split('\n')[0]}` : ''}`);

  const typingErrors = await errorsSoFar(session);
  check('nothing errored while the fields were typed into', () => {
    assert.deepEqual(typingErrors, [], typingErrors.join(' | '));
  });
} catch (e) {
  results.push('FAIL');
  console.log(`  FAIL  the check stopped: ${e.message}`);
} finally {
  await pointerRelease(session);
  await app.close();
}

const failed = results.filter((r) => r === 'FAIL').length;
console.log(failed ? `\nFAIL — ${failed} of ${results.length}` : `\nPASS — ${results.length}/${results.length}`);
process.exit(failed ? 1 : 0);
