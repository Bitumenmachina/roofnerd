// ── C9: the labor basis, read the way a foreman reads a bid ────────────────
// The question this asks is not "did the program multiply correctly". It is
// the one a commercial roofing estimator asks of somebody else's sheet:
//
//   How many hours is that, and who did you think was putting them in?
//
// A crew of six lays so many squares a day. A shop runs so many feet an hour at
// the brake. A subcontractor's number has a name on it. A line that is somebody
// else's stays on the sheet, struck, out of the total, with the reason beside
// it. And the first figure anybody checks is hours a square.
//
// So every row below is about something on screen, through the front door — the
// demo button, the Estimate tab, the line that opens to its own chain, the
// Recap. Nothing in it is typed: the crew, the rate and the hours are worked
// out here from `jobs/demo-job/conditions.json`, which is a second arithmetic
// against the engine's, and where the two disagree the row says which numbers
// they were.
//
// RED AT C6'S HEAD. There is no `production` on any item, no hours anywhere on
// the Estimate, no fabrication line, no name on a subcontract line, no struck
// line at all and no Hours/SQ on the Recap. Every row fails, which is why it is
// written first.
//
//   pnpm build:app && node tools/probe/section-labor.mjs

import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
import {
  errorsSoFar, launch, openDemoJob, showTab, until, wait, watchErrors, commitStamp,
} from './tauri-harness.mjs';

const ROOT = resolve(import.meta.dirname, '../..');
const EVIDENCE = join(ROOT, 'evidence');
const COMMIT = commitStamp();

const results = [];
const check = (name, fn) => {
  try { fn(); results.push('PASS'); console.log(`  PASS  ${name}`); }
  catch (e) { results.push('FAIL'); console.log(`  FAIL  ${name}\n        ${e.message}`); }
};

const shot = async (session, name) =>
  writeFile(join(EVIDENCE, `${name}-${COMMIT}.png`), Buffer.from(await session.screenshot(), 'base64'));

// ── what the job says, worked out here ─────────────────────────────────────

const read = async (rel) => JSON.parse(await readFile(join(ROOT, rel), 'utf8'));
const conditions = await read('jobs/demo-job/conditions.json');
const codes = await read('jobs/demo-job/cost-codes.json');

const classOf = (code) => (codes.find((c) => c.code === code) || {}).class ?? null;

/** Hours in a working day. The engine's own constant, read rather than typed. */
const { HOURS_PER_CREW_DAY } = await import('../../packages/engine/dist/index.js');

/**
 * The rate as man-hours, worked out here from the way it is stated.
 *
 * Deliberately a second implementation and not an import: the whole claim is
 * that the crew and the rate an estimator reads on the line are the two numbers
 * the hours were actually divided by.
 */
const perManHour = (p) => {
  if (!p || !(p.rate > 0)) return null;
  const crew = p.crew > 0 ? p.crew : null;
  if (p.per === 'man-hour') return p.rate;
  if (p.per === 'shop-hour') return p.rate / (crew ?? 1);
  if (p.per === 'crew-hour') return crew === null ? null : p.rate / crew;
  if (p.per === 'crew-day') return crew === null ? null : p.rate / (crew * HOURS_PER_CREW_DAY);
  return null;
};

/** Every line on the job, with what the job file says it is. */
const jobLines = [];
for (const condition of conditions) {
  for (const item of condition.items ?? []) {
    jobLines.push({
      condition: condition.name,
      id: item.id,
      description: item.description,
      unit: item.unit,
      production: item.production ?? null,
      perHour: perManHour(item.production),
      off: item.off?.reason ?? null,
      quotedBy: item.priceSource?.from ?? null,
      class: classOf(item.costCode),
      waste: item.waste ?? 0,
      conditionWaste: condition.waste ?? 0,
    });
  }
}

const worked = jobLines.filter((l) => l.production !== null);
const shopLines = worked.filter((l) => l.production.per === 'shop-hour');
const crewLines = worked.filter((l) => l.production.per !== 'shop-hour');
const subLines = jobLines.filter((l) => l.class === 'Sub');
const struck = jobLines.filter((l) => l.off !== null);

/**
 * The verbs a roofer writes on a labor line.
 *
 * Off an Edge report's own labor sections, which are the trade's words and not
 * this program's: a line reads Install, Fabricate, Fasten, Adhere, Set, Flash,
 * Tear off, Weld, Lay, Form. It is a check on the WORDS and never on a figure.
 */
const TRADE_VERBS = [
  'install', 'fabricate', 'fasten', 'adhere', 'set', 'flash', 'tear off',
  'weld', 'lay', 'form', 'supervis', 'run', 'strip', 'cut',
];

const money = (text) => {
  const m = /-?[\d,]+\.\d\d/.exec(String(text ?? ''));
  return m === null ? null : Number.parseFloat(m[0].replace(/,/g, ''));
};

const figure = (text) => {
  const m = /-?[\d,]+(\.\d+)?/.exec(String(text ?? ''));
  return m === null ? null : Number.parseFloat(m[0].replace(/,/g, ''));
};

const close = (a, b, tolerance) => Math.abs(a - b) <= tolerance;

// ── what the window says ───────────────────────────────────────────────────

/** Every line on the Estimate tab, as the estimator sees it. */
const linesNow = (session) => session.execute(function () {
  return JSON.stringify([...document.querySelectorAll('.cost-line')].map(function (r) {
    const text = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return {
      description: text('.line-item'),
      condition: text('.line-basis > span'),
      formula: text('.line-formula'),
      crew: text('.line-crew'),
      quoted: text('.line-quoted'),
      quantity: text('.line-qty'),
      unit: (text('.line-unit') || '').replace('◈', '').trim(),
      hours: text('.line-hours'),
      hoursDerived: !!r.querySelector('.line-hours .derived'),
      money: text('.line-money'),
      struck: r.classList.contains('off'),
      why: text('.line-off'),
    };
  }));
});

/** The recap on screen: the classes, the summary block under them, the total. */
const recapNow = (session) => session.execute(function () {
  const sheet = document.querySelector('.lens-sheet');
  const selling = document.querySelector('.cost-panel .selling .value');
  const rows = [...document.querySelectorAll('.lens-summary .summary-row')].map(function (r) {
    const cell = function (sel) {
      const el = r.querySelector(sel);
      return el ? (el.textContent || '').trim() : null;
    };
    return { name: cell('.summary-name'), value: cell('.summary-value') };
  });
  const headings = [...document.querySelectorAll('.lens-sheet thead th')]
    .map(function (th) { return (th.textContent || '').trim(); });
  const body = [...document.querySelectorAll('.lens-sheet tbody tr')].map(function (tr) {
    return [...tr.querySelectorAll('td')].map(function (td) { return (td.textContent || '').trim(); });
  });
  return JSON.stringify({
    there: !!sheet,
    selling: selling ? (selling.textContent || '').trim() : null,
    summary: rows,
    headings,
    body,
    text: sheet ? (sheet.innerText || '').replace(/\s+/g, ' ').trim() : '',
  });
});

/** The basis words the Condition panel lists for whatever is picked. */
const basesNow = (session) => session.execute(function () {
  const panel = document.querySelector('.condition-panel');
  if (!panel) return JSON.stringify({ there: false });
  const list = panel.querySelector('.panel-bases');
  return JSON.stringify({
    there: true,
    words: list
      ? [...list.querySelectorAll('.basis-word')].map(function (e) { return (e.textContent || '').trim(); })
      : [],
  });
});

/** Open one line to its whole chain, by the words on it. */
const openLine = async (session, description) => {
  const found = await session.execute(function (want) {
    const row = [...document.querySelectorAll('.cost-line')].find(function (r) {
      const t = r.querySelector('.line-item');
      return t && (t.textContent || '').trim() === want;
    });
    if (!row) return false;
    const what = row.querySelector('.line-what');
    if (!what) return false;
    what.click();
    return true;
  }, description);
  await wait(500);
  return found;
};

/** A field in the open chain, by the words beside it. */
const chainField = (session, label) => session.execute(function (want) {
  const row = [...document.querySelectorAll('.line-chain .chain-row')].find(function (r) {
    const n = r.querySelector('.chain-name');
    return n && (n.textContent || '').trim() === want;
  });
  if (!row) return JSON.stringify(null);
  const input = row.querySelector('input');
  return JSON.stringify({
    value: input ? input.value : null,
    reads: (row.querySelector('.chain-value') || {}).textContent || null,
    typeable: !!input,
  });
}, label);

/** Type into a field in the open chain, the way the panel is typed into. */
const setChainField = (session, label, value) => session.execute(function (want, text) {
  const row = [...document.querySelectorAll('.line-chain .chain-row')].find(function (r) {
    const n = r.querySelector('.chain-name');
    return n && (n.textContent || '').trim() === want;
  });
  const input = row ? row.querySelector('input') : null;
  if (!input) return false;
  input.focus();
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  return true;
}, label, value);

const sellingNow = (session) => session.execute(function () {
  const el = document.querySelector('.cost-panel .selling .value');
  return el ? (el.textContent || '').trim() : null;
});

/** Pick a condition in the tree beside the panel, the way a hand picks it. */
const pickCondition = async (session, name) => {
  const found = await session.execute(function (want) {
    const row = [...document.querySelectorAll('.cost-condition .condition-list .condition')]
      .find(function (e) { return (e.textContent || '').indexOf(want) >= 0; });
    if (!row) return false;
    row.click();
    return true;
  }, name);
  await wait(600);
  return found;
};

// ── the run ────────────────────────────────────────────────────────────────

await mkdir(EVIDENCE, { recursive: true });
const app = await launch();
const { session } = app;

try {
  await watchErrors(session);
  await openDemoJob(session);
  await showTab(session, 'Estimate');
  await until(session, () => document.querySelectorAll('.cost-line').length > 0,
    { what: 'the lines to be drawn' });

  const lines = JSON.parse(await linesNow(session));
  const byWhat = new Map(lines.map((l) => [l.description, l]));

  console.log(`\nsection-labor at ${COMMIT}`);
  console.log(`  the job carries ${worked.length} lines with a rate on them,`
    + ` ${shopLines.length} of them at the brake, ${subLines.length} subcontract`
    + ` and ${struck.length} struck`);

  // ── 1. a labor line shows Quantity · Hours · $, and the hours are derived ──

  check('every labor line on the sheet shows its hours, unmarked, because the rate is the line\u2019s own', () => {
    assert.ok(crewLines.length > 0, 'the demo carries no labor line with a crew rate on it');
    for (const want of crewLines) {
      const row = byWhat.get(want.description);
      assert.ok(row, `"${want.description}" is not on the Estimate`);
      assert.ok(row.hours !== null, `"${want.description}" shows no hours`);
      assert.ok(figure(row.hours) > 0, `"${want.description}" shows hours of "${row.hours}"`);
      // The hours are the engine's arithmetic off the rate the line states, not
      // the model's; the mark is the core's alone (R13, D175), so they carry none.
      assert.equal(row.hoursDerived, false,
        `"${want.description}" shows hours with the model's mark on them`);
    }
  });

  check('the hours on each line are the quantity over the rate the line states', () => {
    for (const want of crewLines) {
      const row = byWhat.get(want.description);
      const quantity = figure(row.quantity);
      const shown = figure(row.hours);
      assert.ok(quantity !== null && shown !== null,
        `"${want.description}" reads ${row.quantity} / ${row.hours}`);
      const withWaste = quantity * (1 + want.waste / 100) * (1 + want.conditionWaste / 100);
      const mine = withWaste / want.perHour;
      assert.ok(close(mine, shown, Math.max(mine, shown) / 100),
        `"${want.description}": the sheet says ${shown} hours, this works out ${mine.toFixed(2)}`
        + ` from ${quantity} ${want.unit} at ${want.production.rate} ${want.production.per}`
        + ` with a crew of ${want.production.crew}`);
    }
  });

  // ── 2. the crew and the rate are ON the line ─────────────────────────────

  check('every labor line says the crew and the rate it assumes', () => {
    for (const want of crewLines) {
      const row = byWhat.get(want.description);
      assert.ok(row.crew, `"${want.description}" does not say what crew puts it in`);
      assert.ok(row.crew.includes(String(want.production.crew)),
        `"${want.description}" says "${row.crew}" and the job says a crew of ${want.production.crew}`);
      assert.ok(/day|hour/.test(row.crew),
        `"${want.description}" says "${row.crew}", which states no unit of time`);
    }
  });

  check('the words on the labor lines are the trade’s own', () => {
    for (const want of worked) {
      const said = (want.description || '').toLowerCase();
      assert.ok(TRADE_VERBS.some((v) => said.includes(v)),
        `"${want.description}" is a labor line with no trade verb in it`);
    }
  });

  // ── 3. the shop is its own line ──────────────────────────────────────────

  check('the metal is fabricated on its own line, at a rate an hour at the brake', () => {
    assert.ok(shopLines.length >= 2,
      `the sheet carries ${shopLines.length} fabrication lines; the coping and the cleat are two`);
    for (const want of shopLines) {
      const row = byWhat.get(want.description);
      assert.ok(row, `"${want.description}" is not on the Estimate`);
      assert.ok(/fabricate/i.test(want.description),
        `"${want.description}" is a shop line that does not say it fabricates anything`);
      assert.ok(row.crew && /hour/.test(row.crew),
        `"${want.description}" says "${row.crew}", and a shop states a rate an hour`);
      assert.ok(figure(row.hours) > 0, `"${want.description}" shows no hours`);
    }
  });

  check('fabricating a piece and installing it are two lines, never one', () => {
    for (const want of shopLines) {
      const what = want.description.replace(/^fabricate/i, '').trim().toLowerCase();
      const mate = worked.find((l) => l !== want
        && /install|set/i.test(l.description)
        && l.description.toLowerCase().includes(what.split(',')[0]));
      assert.ok(mate,
        `"${want.description}" is fabricated and nothing on the sheet installs it`);
    }
  });

  // ── 4. a subcontractor's number has a name on it ─────────────────────────

  check('every subcontract line carries the name it was quoted by, and no hours', () => {
    assert.ok(subLines.length > 0, 'the demo carries no subcontract line');
    for (const want of subLines) {
      const row = byWhat.get(want.description);
      assert.ok(row, `"${want.description}" is not on the Estimate`);
      assert.ok(want.quotedBy, `"${want.description}" is a quote with no name on it`);
      assert.ok(row.quoted && row.quoted.includes(want.quotedBy),
        `"${want.description}" does not print "${want.quotedBy}" on the line`);
      assert.equal(row.hours, null,
        `"${want.description}" is somebody else's work and this sheet puts hours against it`);
    }
  });

  // ── 5. a line that is off stays on the sheet ─────────────────────────────

  check('a struck line is on the sheet, struck, with the reason beside it', () => {
    assert.ok(struck.length > 0, 'the demo carries no line that is off');
    for (const want of struck) {
      const row = byWhat.get(want.description);
      assert.ok(row, `"${want.description}" is off the total and off the sheet as well`);
      assert.ok(row.struck, `"${want.description}" is off the total and is not drawn struck`);
      assert.ok(row.why && row.why.toLowerCase().includes(want.off.toLowerCase()),
        `"${want.description}" is struck and says "${row.why}" rather than "${want.off}"`);
    }
  });

  // ── 6. the recap: hours, and hours a square ──────────────────────────────

  await showTab(session, 'Recap');
  await wait(700);
  const recap = JSON.parse(await recapNow(session));

  check('the Recap prints Hours/SQ', () => {
    assert.ok(recap.there, 'there is no recap sheet on screen');
    const row = recap.summary.find((r) => (r.name || '').replace(/\s/g, '') === 'Hours/SQ');
    assert.ok(row, `the recap's summary reads ${JSON.stringify(recap.summary)}`);
    assert.ok(figure(row.value) > 0, `Hours/SQ reads "${row.value}"`);
  });

  check('Hours/SQ is the hours over the squares the recap itself prints', () => {
    const find = (want) => recap.summary.find((r) => (r.name || '').replace(/\s/g, '') === want);
    const hours = figure((find('Totalhours') || {}).value);
    const squares = figure((find('TotalSQ') || {}).value);
    const rate = figure((find('Hours/SQ') || {}).value);
    assert.ok(hours !== null && squares !== null && rate !== null,
      `the recap's summary reads ${JSON.stringify(recap.summary)}`);
    assert.ok(close(hours / squares, rate, rate / 100),
      `${hours} hours over ${squares} SQ is ${(hours / squares).toFixed(2)}, and it prints ${rate}`);
  });

  check('the recap counts the hours the lines on the sheet carry', () => {
    const find = (want) => recap.summary.find((r) => (r.name || '').replace(/\s/g, '') === want);
    const total = figure((find('Totalhours') || {}).value);
    let mine = 0;
    for (const l of lines) {
      if (l.struck) continue;
      const h = l.hours === null ? null : figure(l.hours);
      if (h !== null) mine += h;
    }
    assert.ok(close(mine, total, Math.max(mine, total) / 100),
      `the lines on the Estimate add to ${mine.toFixed(2)} hours and the recap says ${total}`);
  });

  check('the class table carries an hours column, and the roofing hours are in it', () => {
    assert.ok(recap.headings.some((h) => /hours/i.test(h)),
      `the recap's headings are ${JSON.stringify(recap.headings)}`);
    const labor = recap.body.find((r) => /labor/i.test(r[0] ?? ''));
    assert.ok(labor, `the recap's rows are ${JSON.stringify(recap.body)}`);
    assert.ok(labor.some((cell) => figure(cell) > 0 && /\d/.test(cell)),
      `the labor row reads ${JSON.stringify(labor)}`);
  });

  await shot(session, 'section-labor-recap');

  // ── 7. typing a rate moves the hours and the money ───────────────────────

  await showTab(session, 'Estimate');
  await wait(500);

  const target = crewLines[0];
  const before = money(await sellingNow(session));
  const opened = await openLine(session, target.description);

  check('a labor line opens to its own chain, with the rate and the crew in it', () => {
    assert.ok(opened, `"${target.description}" does not open`);
  });

  const rateField = JSON.parse(await chainField(session, 'Rate'));
  const crewField = JSON.parse(await chainField(session, 'Crew'));
  const hoursRow = JSON.parse(await chainField(session, 'Hours'));

  check('the rate and the crew are typed on the line, and the hours are not', () => {
    assert.ok(rateField && rateField.typeable, 'the chain has no rate to type into');
    assert.ok(crewField && crewField.typeable, 'the chain has no crew to type into');
    assert.ok(hoursRow, 'the chain does not show the hours it worked out');
    assert.equal(hoursRow.typeable, false, 'the hours are typed into, and they are worked out');
    assert.ok(close(Number(rateField.value), target.production.rate, target.production.rate / 100),
      `the rate field reads ${rateField.value} and the job says ${target.production.rate}`);
  });

  const hoursBefore = figure(hoursRow && hoursRow.reads);
  const halved = target.production.rate / 2;
  await setChainField(session, 'Rate', String(halved));
  await wait(900);

  const after = money(await sellingNow(session));
  const hoursAfter = figure(JSON.parse(await chainField(session, 'Hours')).reads);

  check('halving the rate doubles the hours and the selling price follows', () => {
    assert.ok(hoursBefore !== null && hoursAfter !== null,
      `the hours read "${hoursBefore}" then "${hoursAfter}"`);
    assert.ok(close(hoursAfter, hoursBefore * 2, hoursBefore / 10),
      `the rate was halved and the hours went ${hoursBefore} → ${hoursAfter}`);
    assert.ok(before !== null && after !== null,
      `the selling price reads "${before}" then "${after}"`);
    assert.ok(after > before,
      `the crew was slowed by half and the selling price went ${before} → ${after}`);
  });

  // Put it back, so the picture below is of the job as it ships.
  await setChainField(session, 'Rate', String(target.production.rate));
  await wait(700);
  const restored = money(await sellingNow(session));
  check('typing the rate back puts the selling price back where it was', () => {
    assert.ok(restored !== null && before !== null && close(restored, before, 1),
      `the selling price went ${before} → ${after} → ${restored}`);
  });

  await shot(session, 'section-labor-estimate');

  // ── 8. the condition panel says what kinds of line it carries ────────────

  await showTab(session, 'Condition');
  await wait(600);
  await pickCondition(session, 'Parapet');
  const bases = JSON.parse(await basesNow(session));

  check('the Condition panel says what kinds of cost the condition carries', () => {
    assert.ok(bases.there, 'there is no condition panel on screen');
    assert.ok(bases.words.length > 0, 'the panel lists no kinds at all');
    const said = bases.words.join(' ').toLowerCase();
    for (const want of ['material', 'labor', 'fabrication']) {
      assert.ok(said.includes(want),
        `the parapet carries ${want} and the panel lists ${JSON.stringify(bases.words)}`);
    }
  });

  const trouble = await errorsSoFar(session);
  check('nothing went wrong in the window while this ran', () => {
    assert.deepEqual(trouble, [], `the window reported ${JSON.stringify(trouble)}`);
  });
} finally {
  await app.close?.();
}

const passed = results.filter((r) => r === 'PASS').length;
console.log(`\n${passed}/${results.length}`);
process.exit(passed === results.length ? 0 : 1);
