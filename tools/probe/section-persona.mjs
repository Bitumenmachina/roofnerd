// ── C8: a roofer opens it cold ─────────────────────────────────────────────
// Every batch before this one proved its own claim. Nobody had opened the
// shipped program and USED it for twenty minutes the way an estimator would,
// and the twenty minutes found things no row in any of those checks was
// looking at: a coping line that priced forty-eight mitred corners on a roof
// with eight, a recap that folded eleven thousand dollars of tax into a figure
// called "Material", a lens that handed a subcontractor ten percent of waste
// under a heading saying the quantities were as measured, two fields in one
// panel with the same name on them, and a section drawn at a scale it was not
// drawn at.
//
// `.scratch/c8-persona.md` is that session written down, step by step, each
// step classed money, wrong, taste or fine. This is the same walk as a script.
// Every row is a claim the estimator made and every picture is a step he took,
// because the pictures are what Patrick reads.
//
// FRONT DOOR THROUGHOUT. The start screen, the demo button, the view switch,
// the tabs, the chips, the fields, the File menu. `solve_get` is read for what
// a number OUGHT to be — never to get the program into a state.
//
//   pnpm build:app && node tools/probe/section-persona.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  centreOf, commitStamp, detachCostPanel, dockCostPanel, errorsSoFar, launch, openDemoJob,
  pickLens, pointerRelease, showTab, showView, until, wait, watchErrors,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

let step = 0;
const shot = async (session, name) => {
  step += 1;
  const file = `section-persona-${String(step).padStart(2, '0')}-${name}-${COMMIT}.png`;
  await writeFile(join(EVIDENCE, file), Buffer.from(await session.screenshot(), 'base64'));
  console.log(`        picture: evidence/${file}`);
};

const number = (text) => {
  const m = /-?[\d,]+(\.\d+)?/.exec(String(text ?? ''));
  return m === null ? null : Number.parseFloat(m[0].replace(/,/g, ''));
};

// ── what the job on disk says, for the rows that need an expectation ───────
const readJob = async (rel) => JSON.parse(await readFile(join(ROOT, 'jobs/demo-job', rel), 'utf8'));
const conditions = await readJob('conditions.json');
const parapet = conditions.find((c) => c.id === 'c-parapet');
const field = conditions.find((c) => c.kind === 'area' && c.properties?.TAPER !== undefined);

// ── readers ────────────────────────────────────────────────────────────────
const scopesNow = (session) => session.execute(function () {
  return window.__TAURI_INTERNALS__.invoke('solve_get', { part: 'scopes' })
    .then(function (s) { return JSON.stringify(s); })
    .catch(function (e) { return JSON.stringify({ error: String(e) }); });
});

/** Every line on the Estimate, as the estimator reads it. */
const sheetNow = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.cost-line')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return {
      item: text('.line-item'),
      condition: text('.line-basis > span'),
      formula: text('.line-formula'),
      quantity: text('.line-qty'),
      unit: text('.line-unit'),
      ordered: text('.line-ordered'),
      money: text('.line-money'),
      derived: !!r.querySelector('.line-unit .derived'),
      off: r.classList.contains('off'),
    };
  }));
});

const totalsNow = (session) => session.execute(function () {
  return ((document.querySelector('.cost-totals') || {}).innerText || '').replace(/\s+/g, ' ').trim();
});

/** The Recap's own table and the ladder under it. */
const recapNow = (session) => session.execute(function () {
  // NOT trimmed: a row under a class is marked by the space it is indented by,
  // and trimming it away is how this read six classes and no adders.
  const rows = [...document.querySelectorAll('.cost-body tbody tr')].map(function (tr) {
    return [...tr.querySelectorAll('td')].map(function (td) { return td.textContent || ''; });
  });
  const summary = [...document.querySelectorAll('.summary-row')].map(function (r) {
    return [
      ((r.querySelector('.summary-name') || {}).textContent || '').trim(),
      ((r.querySelector('.summary-value') || {}).textContent || '').trim(),
    ];
  });
  return JSON.stringify({ rows, summary });
});

/** A lens's own table: what it hands whoever it is for. */
const lensNow = (session) => session.execute(function () {
  const head = [...document.querySelectorAll('.cost-body thead th')]
    .map(function (th) { return (th.textContent || '').trim(); });
  const rows = [...document.querySelectorAll('.cost-body tbody tr')].map(function (tr) {
    return [...tr.querySelectorAll('td')].map(function (td) { return (td.textContent || '').trim(); });
  });
  const note = document.querySelector('.lens-note, .cost-body p');
  return JSON.stringify({ head, rows, note: note ? (note.textContent || '').trim() : '' });
});

/** The Condition panel: every field, its label, and what is in it. */
const panelNow = (session) => session.execute(function () {
  return JSON.stringify({
    fields: [...document.querySelectorAll('.condition-panel label')].map(function (l) {
      const input = l.querySelector('input');
      return {
        label: ((l.querySelector('span') || {}).textContent || '').trim(),
        value: input ? input.value : null,
      };
    }),
    derived: [...document.querySelectorAll('.condition-panel .derived-row')].map(function (r) {
      return (r.innerText || '').replace(/\s+/g, ' ').trim();
    }),
    measures: ((document.querySelector('.condition-panel small') || {}).textContent || '').trim(),
  });
});

const clickCondition = (session, name) => session.execute(function (want) {
  const c = [...document.querySelectorAll('.cost-condition .condition')]
    .find(function (e) { return ((e.querySelector('.condition-name') || {}).textContent || '').trim() === want; });
  if (!c) return false;
  (c.querySelector('.condition-name') || c).click();
  return true;
}, name);

const setField = (session, label, value) => session.execute(function (want, text) {
  const l = [...document.querySelectorAll('.condition-panel label')]
    .find(function (e) { return ((e.querySelector('span') || {}).textContent || '').trim() === want; });
  const input = l ? l.querySelector('input') : null;
  if (!input) return false;
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
  setter.call(input, String(text));
  input.dispatchEvent(new Event('input', { bubbles: true }));
  input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
  input.dispatchEvent(new Event('change', { bubbles: true }));
  input.blur();
  return true;
}, label, value);

/** The words standing on the roof, and where they landed. */
const roofWords = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.model-label')]
    .filter(function (l) { return !l.hidden; })
    .map(function (l) {
      const r = l.getBoundingClientRect();
      return {
        text: (l.innerText || '').replace(/\s+/g, ' ').trim(),
        x: Math.round(r.left), y: Math.round(r.top),
        w: Math.round(r.width), h: Math.round(r.height),
      };
    }));
});

const stripNow = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.status-strip .readout .readout-field')]
    .map(function (f) {
      return {
        key: ((f.querySelector('.k') || {}).textContent || '').trim(),
        value: ((f.querySelector('.v') || {}).textContent || '').trim(),
      };
    }));
});

const chipClick = (session, text) => session.execute(function (want) {
  const c = [...document.querySelectorAll('.view-chip')]
    .find(function (e) { return (e.textContent || '').trim() === want; });
  if (!c) return false;
  c.click();
  return true;
}, text);

/** A drawing on screen: its note, its key, its words and how much room it took. */
const drawingNow = (session, pane) => session.execute(function (sel) {
  const host = document.querySelector(sel);
  if (!host) return JSON.stringify({ there: false });
  const svg = host.querySelector('svg.drawing') || host.querySelector('svg');
  const marks = svg ? svg.querySelector('.drawing-marks') : null;
  const words = marks ? [...marks.querySelectorAll('text')].map(function (t) {
    const b = t.getBoundingClientRect();
    return {
      text: (t.textContent || '').trim(),
      role: t.getAttribute('data-role'),
      x: Math.round(b.left), y: Math.round(b.top), w: Math.round(b.width), h: Math.round(b.height),
    };
  }) : [];
  const room = host.getBoundingClientRect();
  const drawn = marks ? marks.getBoundingClientRect() : null;
  const note = document.querySelector('.drawing-note');
  const footer = note && !note.hidden ? note.getBoundingClientRect().height : 0;
  return JSON.stringify({
    there: true,
    note: note && !note.hidden ? (note.textContent || '').replace(/\s+/g, ' ').trim() : '',
    words,
    keys: words.filter(function (w) { return w.role === 'key'; }).length,
    room: { w: Math.round(room.width), h: Math.round(room.height - footer) },
    drawn: drawn ? { w: Math.round(drawn.width), h: Math.round(drawn.height) } : null,
  });
}, pane);

const menuPick = (session, label) => session.execute(function (want) {
  const b = document.querySelector('.menu .menu-button');
  if (!b) return false;
  b.click();
  const item = [...document.querySelectorAll('.menu .menu-item')]
    .find(function (e) { return (e.textContent || '').trim() === want; });
  if (!item) return false;
  item.click();
  return true;
}, label);

/** Do two boxes on screen sit on top of one another? */
const overlap = (a, b) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await until(session, () => document.querySelector('#editor')?.children.length > 0, { what: 'the window' });
  await watchErrors(session);

  // ── 1 · cold open ────────────────────────────────────────────────────────
  const demoAt = await centreOf(session, '.start-demo');
  check('the start screen offers a way in without opening a file', () => {
    assert.ok(demoAt, 'there is no demo button on the start screen');
  });
  await shot(session, 'start');

  await openDemoJob(session);
  await until(session, () => document.querySelector('.view-switch'), { what: 'the job to open' });
  await wait(2000);

  const scopes = JSON.parse(await scopesNow(session));
  const fieldScope = scopes[field.id] ?? {};
  const wallScope = scopes[parapet.id] ?? {};

  // ── 2 · the Plan, and what is under the trace ────────────────────────────
  const sheet = JSON.parse(await session.execute(function () {
    const page = document.querySelector('.surface img, .surface canvas, .page-image');
    const badge = document.querySelector('.scale-badge');
    return JSON.stringify({
      page: page ? { w: Math.round(page.getBoundingClientRect().width), tag: page.tagName } : null,
      badge: badge ? (badge.textContent || '').trim() : '',
      traces: document.querySelectorAll('.surface .trace, .surface polygon, .surface polyline').length,
    });
  }));
  check('the demo is traced on a drawing, at a scale the badge states', () => {
    assert.ok(sheet.page, 'there is no sheet under the trace at all');
    assert.ok(sheet.badge.length > 0, 'the sheet carries no scale');
    assert.ok(sheet.traces > 0, 'nothing is traced on it');
  });
  await shot(session, 'plan');

  // ── 3 · the Estimate, line by line ───────────────────────────────────────
  await showTab(session, 'Estimate');
  await wait(900);
  const lines = JSON.parse(await sheetNow(session));
  await shot(session, 'estimate');

  check('every line on the sheet carries a mark: worked out, or typed', () => {
    assert.ok(lines.length > 0, 'the sheet has no lines on it');
    const bare = lines.filter((l) => !l.derived && !/^\s*-?[\d.,]+\s*$/.test(l.formula ?? ''));
    assert.deepEqual(bare.map((l) => l.item), [],
      'these lines are neither derived nor typed, so nothing says where they came from');
  });

  check('nothing on the demo is pending: every line has a quantity and a price', () => {
    const pending = lines.filter((l) => l.quantity === '—' || l.money === '—');
    assert.deepEqual(pending.map((l) => l.item), [], 'these lines could not be worked out');
  });

  // Each line against ITS OWN condition's scope. A curb and a parapet both
  // carry FLASH_SF and they are different walls with different flashing on
  // them, so a check that looked the name up in one bag would hold the curb's
  // twenty-nine feet against the parapet's seven hundred.
  const byName = new Map(conditions.map((c) => [c.name, scopes[c.id] ?? {}]));
  check('the numbers the sheet shows are the solve\'s, name by name', () => {
    let checked = 0;
    for (const line of lines) {
      const name = (line.formula ?? '').trim();
      if (!/^[A-Z][A-Z0-9_]*$/.test(name)) continue;
      const scope = byName.get((line.condition ?? '').trim());
      const want = scope?.[name];
      if (want === undefined || want === null) continue;
      const said = number(line.quantity);
      assert.ok(said !== null && Math.abs(said - want) < 0.01,
        `"${line.item}" reads ${line.quantity} where ${line.condition}'s ${name} is ${want}`);
      checked += 1;
    }
    assert.ok(checked >= 5, `only ${checked} lines could be held against the solve`);
  });

  // The line an estimator counted the corners on. The roof has as many mitred
  // pieces as the run has turns, and it had as many as the run has pieces.
  check('the mitred corner line prices the corners, not every piece of coping', () => {
    const corners = wallScope.COPING_CORNERS_EA;
    const pieces = wallScope.COPING_PCS_EA;
    assert.ok(typeof corners === 'number', 'the solve carries no corner count for the parapet');
    assert.ok(pieces > corners, `the run is ${pieces} pieces and ${corners} corners, which cannot be`);
    const mitre = lines.find((l) => /mitre/i.test(l.item ?? ''));
    assert.ok(mitre, 'no line on the sheet is a mitred corner');
    assert.equal(number(mitre.quantity), corners,
      `it buys ${mitre.quantity} of them where the parapet turns ${corners} times`);
  });

  check('a line that buys something other than what it measures says what it buys', () => {
    const bought = lines.filter((l) => l.ordered);
    assert.ok(bought.length > 0,
      'no line on the sheet says what it orders, and several are priced per ordered unit');
    for (const line of bought) {
      assert.match(line.ordered, /^buy [\d,.]+ [A-Z]+$/,
        `"${line.item}" says "${line.ordered}", which is not a quantity and a unit`);
    }
  });

  // ── 4 · the Recap ────────────────────────────────────────────────────────
  await showTab(session, 'Recap');
  await wait(900);
  const recap = JSON.parse(await recapNow(session));
  await shot(session, 'recap');

  check('a class that carries an adder says what the adder is and what it came to', () => {
    const inside = recap.rows.filter((r) => /^\s/.test(r[0] ?? ''));
    assert.ok(inside.length > 0,
      'every class prints one figure, so a burden or a tax inside one cannot be seen');
    assert.ok(inside.some((r) => /the lines themselves/.test(r[0])),
      'nothing says what the lines under a class came to before its adders');
  });

  check('each class total is its own subtotal plus the adders printed under it', () => {
    let held = 0;
    for (let i = 0; i < recap.rows.length; i += 1) {
      const row = recap.rows[i];
      if (/^\s/.test(row[0] ?? '')) continue;
      const total = number(row[1]);
      let sum = 0;
      let any = false;
      for (let j = i + 1; j < recap.rows.length && /^\s/.test(recap.rows[j][0] ?? ''); j += 1) {
        sum += number(recap.rows[j][1]) ?? 0;
        any = true;
      }
      if (!any) continue;
      assert.ok(Math.abs(sum - total) < 0.02,
        `${row[0].trim()} totals ${row[1]} and the rows under it add to ${sum.toFixed(2)}`);
      held += 1;
    }
    assert.ok(held > 0, 'no class on the recap could be added up from its own rows');
  });

  check('the sheet called the roll-up to selling price prints the roll-up', () => {
    const names = recap.summary.map((r) => r[0].toLowerCase());
    for (const want of ['job cost', 'profit', 'selling price']) {
      assert.ok(names.some((n) => n.startsWith(want)), `the recap never says ${want}`);
    }
  });

  // ── 5 · the Condition panel ──────────────────────────────────────────────
  await showTab(session, 'Condition');
  await wait(700);
  await clickCondition(session, parapet.name);
  await wait(900);
  const panel = JSON.parse(await panelNow(session));
  await shot(session, 'condition');

  check('no two fields in the panel carry the same name', () => {
    const seen = new Map();
    for (const f of panel.fields) {
      if (!f.label) continue;
      assert.ok(!seen.has(f.label),
        `"${f.label}" is on two fields at once, and one of them holds the number`);
      seen.set(f.label, true);
    }
  });

  check('the panel says what this condition measures, not what another one does', () => {
    assert.ok(!/coping and its parapet cannot drift apart/.test(panel.measures)
      || parapet.from !== undefined,
      `the help reads "${panel.measures}" on a condition that takes its measures from nothing`);
  });

  check('the wall\'s own solved numbers read on it, each with the name a formula uses', () => {
    const said = panel.derived.join(' ');
    for (const name of ['FLASH_SF', 'COPING_LF', 'COPING_CORNERS_EA']) {
      assert.ok(said.includes(name), `the panel never names ${name}`);
    }
  });

  // ── 6 · the Model ────────────────────────────────────────────────────────
  await showView(session, 'Model');
  await until(session, () => !!document.querySelector('.model-canvas'), { what: 'the model canvas' });
  await wait(2000);
  const words = JSON.parse(await roofWords(session));
  await shot(session, 'model');

  check('the words standing on the roof do not lie on top of each other', () => {
    assert.ok(words.length > 1, `only ${words.length} words are on the roof`);
    for (let i = 0; i < words.length; i += 1) {
      for (let j = i + 1; j < words.length; j += 1) {
        assert.ok(!overlap(words[i], words[j]),
          `"${words[i].text}" and "${words[j].text}" are drawn over one another`);
      }
    }
  });

  // Every reading in the strip is a word and a figure. One of them was a
  // sentence standing in a row with no name on it.
  await session.execute(function () {
    const c = document.querySelector('.model-canvas');
    const b = c.getBoundingClientRect();
    c.dispatchEvent(new MouseEvent('pointermove', {
      clientX: Math.round(b.left + b.width * 0.45),
      clientY: Math.round(b.top + b.height * 0.5),
      bubbles: true,
    }));
  });
  await wait(700);
  const strip = JSON.parse(await stripNow(session));
  check('every reading under the pointer is named', () => {
    assert.ok(strip.length > 0, 'the strip says nothing with the pointer on the roof');
    const bare = strip.filter((r) => r.key === '');
    assert.deepEqual(bare.map((r) => r.value), [], 'these readings have no name on them');
  });
  console.log(`  REPORT  the strip reads: ${strip.map((r) => `${r.key} ${r.value}`).join(' · ')}`);

  // ── 7 · the Section ──────────────────────────────────────────────────────
  await showView(session, 'Section');
  await wait(3000);
  const cut = JSON.parse(await drawingNow(session, '.section-drawing'));
  await shot(session, 'section');

  check('the cut fills the sheet it is drawn on', () => {
    assert.ok(cut.there && cut.drawn, 'there is no section drawn at all');
    const across = cut.drawn.w / cut.room.w;
    const down = cut.drawn.h / cut.room.h;
    assert.ok(Math.max(across, down) > 0.8,
      `it takes ${(across * 100).toFixed(0)}% of the width and ${(down * 100).toFixed(0)}% of the height`);
  });

  check('it is drawn at the scale it states, and states one off an architect\'s rule', () => {
    assert.match(cut.note, /horizontal 1" = \d+'-\d+"/, `the sheet says "${cut.note}"`);
    assert.ok(!/brought down to fit/.test(cut.note),
      `it states a scale it is not drawn at: "${cut.note}"`);
  });

  check('no two words on the section are drawn over one another', () => {
    const on = cut.words.filter((w) => w.w > 0 && w.h > 0);
    assert.ok(on.length > 4, `only ${on.length} words are on the drawing`);
    for (let i = 0; i < on.length; i += 1) {
      for (let j = i + 1; j < on.length; j += 1) {
        assert.ok(!overlap(on[i], on[j]),
          `"${on[i].text}" and "${on[j].text}" are drawn over one another`);
      }
    }
  });

  // ── 8 · the fastening plan ───────────────────────────────────────────────
  await showView(session, 'Plan');
  await wait(1400);
  await chipClick(session, 'fastening');
  await wait(2200);
  const fastening = JSON.parse(await drawingNow(session, '.surface'));
  await shot(session, 'fastening');

  check('the fastening plan is titled with the standard it was set out to', () => {
    assert.match(fastening.note, /^Fastening plan · /, `the title line reads "${fastening.note}"`);
    assert.ok(fastening.note.length < 80,
      `the title line is a paragraph again: "${fastening.note}"`);
  });

  check('and its key is short enough to read on the drawing', () => {
    assert.ok(fastening.keys > 0, 'the key rides in the payload and is drawn nowhere');
    assert.ok(fastening.keys <= 3, `the key is ${fastening.keys} lines`);
  });

  check('every band names the pressure it is fastened for', () => {
    const said = fastening.words.map((w) => w.text).join(' ');
    assert.ok(/psf/.test(said), 'the plan never states a pressure anywhere on itself');
  });
  await chipClick(session, 'fastening');
  await wait(800);

  // ── 9 · a typed slope moves the bill, and putting it back moves it home ──
  await showTab(session, 'Condition');
  await wait(600);
  await clickCondition(session, field.name);
  await wait(800);
  await showTab(session, 'Estimate');
  await wait(700);
  const before = await totalsNow(session);
  await showTab(session, 'Condition');
  await wait(500);
  const flatter = (field.properties.TAPER ?? 0.25) / 2;
  const typed = await setField(session, 'Taper', String(flatter));
  await wait(2200);
  await showTab(session, 'Estimate');
  await wait(1200);
  const after = await totalsNow(session);
  await shot(session, 'typed-slope');

  check('a slope typed onto the field moves the money', () => {
    assert.ok(typed, 'there is no slope field on the roof to type into');
    assert.notEqual(before, after, `the totals did not move: "${after}"`);
  });

  await showTab(session, 'Condition');
  await wait(500);
  await clickCondition(session, field.name);
  await wait(700);
  await setField(session, 'Taper', String(field.properties.TAPER));
  await wait(2200);
  await showTab(session, 'Estimate');
  await wait(1000);
  const back = await totalsNow(session);
  check('and typing it back puts the bill back to the cent', () => {
    assert.equal(back, before, 'the roof did not come back to what it was');
  });

  // ── 10 · what a subcontractor is handed ──────────────────────────────────
  const measured = JSON.parse(await sheetNow(session));
  const picked = await pickLens(session, 'Drawing');
  await wait(1600);
  const lens = JSON.parse(await lensNow(session));
  await shot(session, 'lens');

  check('the lens that says its quantities are as measured hands over the measured ones', () => {
    assert.ok(picked, 'the Drawing lens could not be picked');
    const qty = lens.head.indexOf('Quantity');
    assert.ok(qty >= 0, `the lens has no quantity column: ${lens.head.join(', ')}`);
    const item = lens.head.indexOf('Item');
    assert.ok(item >= 0, `the lens has no item column: ${lens.head.join(', ')}`);
    let held = 0;
    for (const row of lens.rows) {
      const on = measured.find((l) => l.item === (row[item] ?? '').trim());
      if (!on) continue;
      assert.equal(number(row[qty]), number(on.quantity),
        `"${name}" is ${row[qty]} to the subcontractor and ${on.quantity} on the estimate`);
      held += 1;
    }
    assert.ok(held >= 5, `only ${held} lines could be held against the sheet`);
  });

  // ── 11 · it lands a file, and says what did not make it in ───────────────
  const exported = await menuPick(session, 'Export IFC');
  await until(session, () => (document.querySelector('.status-strip')?.textContent || '')
    .includes('exported'), { what: 'the file to be written', timeout: 20000 });
  await wait(700);
  const said = await session.execute(function () {
    const m = document.querySelector('.status-message');
    if (!m) return JSON.stringify(null);
    return JSON.stringify({ text: (m.textContent || '').trim(), w: Math.round(m.scrollWidth),
      room: Math.round(m.clientWidth) });
  });
  await shot(session, 'export');
  const message = JSON.parse(said);

  check('File → Export IFC lands a file and names it', () => {
    assert.ok(exported, 'Export IFC is not on the File menu');
    assert.ok(message, 'the status bar said nothing');
    assert.match(message.text, /\.ifc\b/, `the bar reads "${message.text}"`);
  });

  check('and where something was left out of it, that is the part you can read', () => {
    if (!/left out/.test(message.text)) return;
    const upTo = Math.max(0, Math.floor(message.text.length * (message.room / message.w)));
    assert.ok(/left out/.test(message.text.slice(0, upTo)),
      `only "${message.text.slice(0, upTo)}" fits in the bar`);
  });

  // ── 12 · the money on the other monitor, and home again ──────────────────
  const off = await detachCostPanel(session);
  await wait(1500);
  const windows = (await session.handles()).length;
  await shot(session, 'panel-off');
  check('the cost panel goes to its own window', () => {
    assert.ok(off, 'the button that tears it off could not be pressed');
    assert.ok(windows > 1, `the program still has ${windows} window`);
  });
  await dockCostPanel(session);
  await wait(1500);
  await shot(session, 'panel-back');

  const errors = await errorsSoFar(session);
  check('and nothing in the window complained from the start screen to here', () => {
    assert.deepEqual(errors, [], `the window logged ${JSON.stringify(errors)}`);
  });

  console.log(`\n  ${results.filter((r) => r === 'PASS').length}/${results.length}`);
} finally {
  await pointerRelease(session);
  await app.close();
}

process.exit(results.includes('FAIL') ? 1 : 0);
